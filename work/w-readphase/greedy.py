#!/usr/bin/env python3
"""w-readphase — the CLASS-WIDE GREEDY chain-sink ladder over the whole workload.

One `c2rs gap` scan per rung. Each rung grants the sink token that opens the
CURRENT HEAD of the emitted widening order, then re-scans and reads the
successor. `work/w-front3/ladder.py` does this for ONE TU; this does it for the
878-TU workload at once, so the number it produces is the DEPTH OF THE CLASS,
not of a TU.

Three columns, and only the third is a payoff:

  blocked   sum of `emit_blockers` — INVARIANT unless a body is accepted, so it
            is a control, not a result
  keys      distinct emitted blocker keys — the width of the open key space
  poison    emitted functions that walked their WHOLE body through the sink
            (`expr-chain-sink-poison`). THIS is the decode reach the ladder
            bought, and it is decode-only: the poison refuses.

RENAME rungs (`w-ladders` §4) are classified, not counted: granting an opcode
`chain_skip_form` has no width for changes `expr-op-0xNN` to
`expr-chain-noform-0xNN` at the same byte and buys zero distance.

usage: greedy.py <outdir> <rounds> [seed-spec]
"""
import collections
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
C2RS = os.path.join(ROOT, "target/release/c2rs")
LIST = os.path.join(ROOT, "work/dc3-workload/files.txt")
FLAGS = os.path.join(ROOT, "work/dc3-workload/flags.txt")
DC3 = os.path.abspath(os.path.join(ROOT, "../../../../dc3-decomp"))
EXPR_RS = os.path.join(ROOT, "crates/c2-il/src/func/body/expr.rs")

POISON = "expr-chain-sink-poison"
# `w-front3/ladder.py`'s `TAIL`. `4F 12` is the FUNCTION TAIL and `chain_skip_form`
# refuses every `4F` that is not `4F 01 <varint>`, so this key is where a walk that
# ran the whole body lands. Treated as a TERMINAL, exactly as ladder.py does — with
# the ambiguity named: IL_STMT_GRAMMAR §12.6 records that the rest of the `4F NN`
# family is undetermined, so a mid-body `4F NN` would land here too.
TAIL = "expr-chain-noform-0x4F"
BADTOKEN = "expr-chain-badtoken"

NAMED = {
    "cmp-eq": 0x1F, "cmp-ne": 0x20, "cmp-le": 0x21, "cmp-lt": 0x22,
    "cmp-ge": 0x23, "cmp-gt": 0x24,
    "not": 0x1A, "or-or": 0x1B, "and-and": 0x1C,
    "shl": 0x09, "shr": 0x0A, "bit-and": 0x0B, "bit-or": 0x0C, "bit-xor": 0x0D,
    "convert": 0x2C, "intrinsic-call": 0x40, "class-descriptor": 0x66,
    "ternary": 0x43, "call-in-expr": 0x26,
    "label": 0x29, "brfalse": 0x38, "brtrue": 0x39, "jump": 0x3A,
    "switch-dispatch": 0x3B, "switch-table": 0x3C, "switch-case": 0x3D,
}


def pinned_opcodes():
    """`chain_skip_form`'s width table, DERIVED from the tree (w-ladders §4).

    Fails loudly rather than returning an empty set: an empty table would make
    every rung look like a rename, which is trap 5 aimed at this lane's own
    finding.
    """
    src = open(EXPR_RS).read()
    body = src[src.index("fn chain_skip_form"):]
    body = body[:body.index("\n}\n")]
    out = set()
    pat = (r'^\s+(0x[0-9A-Fa-f]{2}(?:\s*\|\s*0x[0-9A-Fa-f]{2})*'
           r'(?:\s*\.\.=\s*0x[0-9A-Fa-f]{2})?)\s*=>')
    for m in re.finditer(pat, body, re.M):
        s = m.group(1)
        if "..=" in s:
            a, b = s.split("..=")
            out.update(range(int(a.strip(), 16), int(b.strip(), 16) + 1))
        else:
            for t in s.split("|"):
                out.add(int(t.strip(), 16))
    if len(out) < 20:
        raise SystemExit("LADDER-NOWIDTHTABLE — parsed %d arms" % len(out))
    return out


PINNED = pinned_opcodes()


def lift_for(key):
    """(token, note) that opens `key` in the COMMITTED sink, or (None, reason)."""
    if key.startswith(POISON):
        return (None, "terminal")
    if key.startswith("expr-chain-noform") or key.startswith("expr-chain-short"):
        return (None, "noform")
    if key.startswith("expr-load-type-") or key.startswith("expr-lit-type-"):
        return ("type", "")
    if key.startswith("expr-convert-target"):
        return ("convert", "")
    if key.startswith("expr-intrinsic-"):
        return ("intrinsic", "")
    if key.startswith("expr-call-in-expr"):
        # `w-one`'s tail rule: the composite key names the byte it stopped on.
        for marker in ("-then-op-0x", "-op-0x", "-0x"):
            i = key.rfind(marker)
            if i >= 0:
                h = key[i + len(marker):][:2]
                try:
                    return ("op:%02X" % int(h, 16), "tail")
                except ValueError:
                    pass
        return ("op:26", "")
    if key.startswith("expr-op-0x"):
        return ("op:%s" % key[len("expr-op-0x"):].split("-", 1)[0], "")
    if key.startswith("expr-"):
        r = key[len("expr-"):]
        if r in NAMED:
            return ("op:%02X" % NAMED[r], "")
    return (None, "no-lift:%s" % key)


def is_rename(tok):
    if not tok or not tok.startswith("op:"):
        return False
    try:
        return int(tok[3:], 16) not in PINNED
    except ValueError:
        return False


def scan(spec, jsonl):
    env = dict(os.environ)
    env["C2RS_SINK_CHAIN"] = spec
    r = subprocess.run([C2RS, "gap", "--list", LIST, "--flags-file", FLAGS,
                        "--cwd", DC3, "--jobs", "16", "--jsonl", jsonl],
                       env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit("SCAN FAILED %d spec=%r" % (r.returncode, spec))
    emit = collections.Counter()
    graded = 0
    for line in open(jsonl):
        if not line.startswith('{"src"'):
            continue
        row = json.loads(line)
        if row.get("class") != "capture-fail":
            graded += 1
        for k, v in (row.get("emit_blockers") or {}).items():
            emit[k] += v
    if graded < 800:
        raise SystemExit("ONLY %d GRADED — refusing to report a null" % graded)
    if not emit:
        raise SystemExit("emit_blockers EMPTY — refusing")
    bad = sum(v for k, v in emit.items() if k.startswith(BADTOKEN))
    if bad:
        raise SystemExit("SINK DEAD: badtoken on %d functions, spec=%r"
                         % (bad, spec))
    m = {}
    for line in r.stdout.splitlines():
        line = line.strip()
        if line.startswith("gap-metric "):
            p = line.split()
            m[p[1]] = p[2]
    return emit, m, graded


def main():
    outdir, rounds = sys.argv[1], int(sys.argv[2])
    seed = sys.argv[3] if len(sys.argv) > 3 else "op:41"
    os.makedirs(outdir, exist_ok=True)
    spec = [t for t in seed.split(",") if t]
    rows = []
    print("%-3s %-26s %8s %6s %8s %6s %8s  %s"
          % ("#", "granted", "blocked", "keys", "reach", "match", "fnbyte",
             "new head"))
    for rnd in range(rounds):
        emit, m, graded = scan(",".join(spec),
                               os.path.join(outdir, "r%02d.jsonl" % rnd))
        poison = sum(v for k, v in emit.items() if k.startswith(POISON))
        tail = sum(v for k, v in emit.items() if k.startswith(TAIL))
        blocked = sum(emit.values())
        head = [(k, v) for k, v in emit.most_common()
                if not k.startswith(POISON) and not k.startswith(TAIL)]
        hk, hv = head[0] if head else ("<none>", 0)
        tok, note = lift_for(hk)
        rows.append({"round": rnd, "spec": ",".join(spec), "graded": graded,
                     "blocked": blocked, "keys": len(emit), "poison": poison, "tail": tail, "reach": poison + tail,
                     "match": m.get("match"), "mismatch": m.get("mismatch"),
                     "fnbyte_exact": m.get("fnbyte-exact"),
                     "head": hk, "head_n": hv, "grant": tok, "note": note,
                     "rename": is_rename(tok), "top": emit.most_common(20)})
        print("%-3d %-26s %8d %6d %8d %6s %8s  %s (%d) -> %s%s"
              % (rnd, spec[-1] if spec else "-", blocked, len(emit),
                 poison + tail,
                 m.get("match"), m.get("fnbyte-exact"), hk, hv,
                 tok or ("STOP " + note),
                 " [RENAME]" if is_rename(tok) else ""))
        json.dump(rows, open(os.path.join(outdir, "rounds.json"), "w"), indent=1)
        if tok is None:
            print("EXIT: %s" % note)
            break
        if tok in spec:
            print("EXIT: head already granted (%s) — driver cannot advance" % tok)
            break
        if is_rename(tok):
            print("EXIT: RENAME — %s has no pinned width; granting it advances "
                  "the stream by ZERO bytes" % tok)
            break
        spec.append(tok)


main()

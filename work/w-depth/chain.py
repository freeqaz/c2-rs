#!/usr/bin/env python3
"""w-depth — walk the PARSE CHAIN of a TU one sink step at a time.

The instrument is `C2RS_SINK_CHAIN` (crates/c2-il/src/func/body/expr.rs, board
#660): a poisoning, data-driven sink over `parse_expr`. This driver does the
iteration the sink cannot do for itself — scan, read the blocker keys, add a
sink token for each, re-scan — and reports, per TU:

    DEPTH   the number of distinct expression-layer refusal classes that had to
            be closed before every blocked function's expression walked to the
            end (i.e. every one of them reached `expr-chain-sink-poison`)
    SET     those classes, in the order the chain met them
    STATUS  CLEAR  | EXIT:<key>  | NOFORM:<key>  | BOUND

`EXIT` is not a failure of the TU: it is the chain leaving `parse_expr` for a
production this instrument does not cover (`mcall`, `assign`, `control_flow`,
the formals walk).  `NOFORM` is the chain reaching an opcode whose payload width
this tree has not pinned — reported rather than guessed, because a guessed width
desynchronises the stream and manufactures a fictitious successor.

Usage:
  chain.py <files.txt> <flags.txt> <cwd> <cache> <c2rs> <outdir> [--bound N]
"""

import json
import os
import subprocess
import sys

# `Block::refuse` carries no byte, so the poison renders `…:mid` or `…:eof`
# depending on the cursor.  Matching the bare string was this driver's first bug:
# it reported two CLEAR TUs as EXITs on their own terminal.
TERMINAL = "expr-chain-sink-poison"

# `4F 12` is the FUNCTION TAIL, not a line marker, and the sink refuses it by
# design (eating it would walk the instrument straight out of the body).  So
# `expr-chain-noform-0x4F` is not an unpinned opcode: it is the chain having
# consumed **every byte of the body** without ever meeting the `41` result
# annotation the expression walk stops on — a body whose value is returned
# through the `3A` jump form.  That is a TERMINAL, and calling it a NOFORM
# understated the chain on three TUs.
TAIL = "expr-chain-noform-0x4F"

# Tokens that emit NOTHING — the scope brackets, the statement end and the line
# marker.  w-brfalse had to add a whole third sink level because its chains
# substituted into `0x53` and *"reporting the successor is `0x53` as the answer
# would be reporting punctuation as work"*.  This driver reports the split
# instead of choosing: DEPTH is the whole chain, OPDEPTH is the chain minus the
# punctuation.
DELIMITERS = {"op:53", "op:54", "op:4B", "op:4F"}

# ---- key -> sink token --------------------------------------------------
# Every row is the inverse of `Block::feature`'s rendering for `ctx == "expr"`
# (crates/c2-il/src/func/body/mod.rs): `expr_opcode_name` first, then
# `cflow_opcode_name`, then the `expr-op-0xNN` fallback.
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


def sink_token(key):
    """The `C2RS_SINK_CHAIN` token that closes `key`, or None when the key is
    outside `parse_expr` and the chain therefore EXITs."""
    if key.startswith(TERMINAL) or key.startswith(TAIL):
        return "TERMINAL"
    if key.startswith("expr-chain-noform") or key.startswith("expr-chain-short"):
        return None
    # The operand-TYPE gate.  `expr-load-type-8643` / `expr-lit-type-9641`.
    if key.startswith("expr-load-type-") or key.startswith("expr-lit-type-"):
        return "type"
    if key.startswith("expr-convert-target"):
        return "convert"
    if key.startswith("expr-intrinsic-"):
        return "intrinsic"
    # `mcall::feature` renders the whole `26`-in-expression family; every one of
    # them is raised from `parse_expr`'s `0x26` arm, so one token closes them all.
    if key.startswith("expr-call-in-expr"):
        return "op:26"
    if key.startswith("expr-op-0x"):
        return "op:%s" % key[len("expr-op-0x"):]
    if key.startswith("expr-"):
        rest = key[len("expr-"):]
        if rest in NAMED:
            return "op:%02X" % NAMED[rest]
    # Every remaining refusal — `expr-ptr-arith`, `expr-empty`, `expr-load-tok`,
    # and everything with a non-`expr` ctx — is either a guard rather than a
    # token or is raised outside `parse_expr`.  Both are EXIT.
    return None


def scan(c2rs, listfile, flags, cwd, cache, spec, outdir, tag):
    env = dict(os.environ)
    if spec:
        env["C2RS_SINK_CHAIN"] = spec
    else:
        env.pop("C2RS_SINK_CHAIN", None)
    js = os.path.join(outdir, "chain-%s.jsonl" % tag)
    cmd = [c2rs, "gap", "--list", listfile, "--flags-file", flags,
           "--cwd", cwd, "--jobs", "4", "--cache", cache, "--jsonl", js]
    r = subprocess.run(cmd, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit("gap failed (%d): %s" % (r.returncode, r.stderr[-2000:]))
    out = {}
    with open(js) as fh:
        for ln in fh:
            d = json.loads(ln)
            if d.get("record") == "provenance":
                continue
            out[d["src"]] = d
    return out


def main():
    a = sys.argv[1:]
    bound = 12
    if "--bound" in a:
        i = a.index("--bound")
        bound = int(a[i + 1])
        del a[i:i + 2]
    listfile, flags, cwd, cache, c2rs, outdir = a[:6]
    os.makedirs(outdir, exist_ok=True)
    tus = [l.strip() for l in open(listfile) if l.strip()]

    results = {}
    for tu in tus:
        one = os.path.join(outdir, "one.txt")
        with open(one, "w") as fh:
            fh.write(tu + "\n")
        sinks = []          # ordered, de-duplicated
        steps = []          # (round, keys met, tokens added, exit keys)
        status = "BOUND"
        results_extra = []
        rec = None
        for rnd in range(bound + 1):
            spec = ",".join(sinks)
            tag = "%s-%d" % (tu.replace("/", "_"), rnd)
            rec = scan(c2rs, one, flags, cwd, cache, spec, outdir, tag).get(tu)
            if rec is None:
                status = "NORECORD"
                break
            blockers = rec.get("fn_blockers") or {}
            live = {k: v for k, v in blockers.items()
                    if not (k.startswith(TERMINAL) or k.startswith(TAIL))}
            if not live:
                status = "CLEAR"
                break
            added, exits = [], []
            for k in sorted(live):
                t = sink_token(k)
                if t is None:
                    exits.append(k)
                elif t != "TERMINAL" and t not in sinks and t not in added:
                    added.append(t)
            steps.append((rnd, dict(sorted(live.items())), list(added), list(exits)))
            if exits:
                # The chain has left `parse_expr`.  DEPTH is reported as what the
                # chain had already closed — a LOWER BOUND — and `also_at_exit`
                # names the expression tokens still live in the same round, which
                # are known to be on the chain but were never walked past.
                status = "EXIT:" + ";".join(exits)
                results_extra = added
                break
            if not added:
                status = "STUCK:" + ",".join(sorted(live))
                break
            sinks.extend(added)
        ops_only = [s for s in sinks if s not in DELIMITERS]
        results[tu] = {
            "depth": len(sinks),
            "opdepth": len(ops_only),
            "delims": [s for s in sinks if s in DELIMITERS],
            "sinks": sinks,
            "also_at_exit": results_extra,
            "status": status,
            "steps": steps,
            "fn_total": rec.get("fn_total") if rec else None,
            "fn_in_class": rec.get("fn_in_class") if rec else None,
        }
        print("%-46s D=%-3d OP=%-3d %-34s %s"
              % (tu, len(sinks), len(ops_only), status[:34], ",".join(sinks)),
              flush=True)

    with open(os.path.join(outdir, "chain.json"), "w") as fh:
        json.dump(results, fh, indent=1, sort_keys=True)


if __name__ == "__main__":
    main()

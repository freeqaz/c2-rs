#!/usr/bin/env python3
"""w-readphase — the CLASS-WIDE chain-sink ladder, over all 878 TUs at once.

`work/w-front3/ladder.py` climbs ONE TU's ladder. This climbs the WORKLOAD's:
one `c2rs gap` scan per rung, the sink spec accumulated, and three numbers read
off each scan that a per-TU ladder cannot produce —

  blocked      total blocked EMITTED functions (`emit_blockers` sum)
  poison       emitted functions that reached `expr-chain-sink-poison`, i.e.
               walked the WHOLE body end to end through the sink. This is the
               DECODE REACH the rung bought. A body that merely got a new key
               is not here.
  keys         distinct `emit_blockers` keys — the width of the open key space

The sink is `w-depth`'s committed, poisoned `C2RS_SINK_CHAIN` (board #660): it
pushes no `IlOp`, a body that uses it refuses anyway, and an opcode whose width
this tree has not pinned refuses as `expr-chain-noform-0xNN` rather than being
guessed. So every number here is DECODE-ONLY and cannot move one obj byte.

usage: phase.py <steps.txt> <outdir>
   steps.txt: one line per rung — `<label>\t<comma-separated sink spec>`
"""
import collections
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
C2RS = os.path.join(ROOT, "target/release/c2rs")
LIST = os.path.join(ROOT, "work/dc3-workload/files.txt")
FLAGS = os.path.join(ROOT, "work/dc3-workload/flags.txt")
DC3 = os.path.abspath(os.path.join(ROOT, "../../../../dc3-decomp"))

POISON = "expr-chain-sink-poison"
BADTOKEN = "expr-chain-badtoken"


def scan(spec, jsonl):
    env = dict(os.environ)
    if spec:
        env["C2RS_SINK_CHAIN"] = spec
    else:
        env.pop("C2RS_SINK_CHAIN", None)
    cmd = [C2RS, "gap", "--list", LIST, "--flags-file", FLAGS,
           "--cwd", DC3, "--jobs", "16", "--jsonl", jsonl]
    r = subprocess.run(cmd, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit("SCAN FAILED (%d) spec=%r\n%s" % (r.returncode, spec,
                                                           r.stderr[-2000:]))
    return r.stdout


def read(jsonl):
    emit = collections.Counter()
    fnb = collections.Counter()
    rows = 0
    graded = 0
    for line in open(jsonl):
        line = line.strip()
        if not line.startswith("{"):
            continue
        r = json.loads(line)
        if "src" not in r:
            continue
        rows += 1
        if r.get("class") != "capture-fail":
            graded += 1
        for k, v in (r.get("emit_blockers") or {}).items():
            emit[k] += v
        for k, v in (r.get("fn_blockers") or {}).items():
            fnb[k] += v
    return rows, graded, emit, fnb


def metric(out, key):
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("gap-metric " + key + " "):
            return line.split()[2]
    return "?"


def main():
    steps = []
    for line in open(sys.argv[1]):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        label, spec = line.split("\t", 1)
        steps.append((label, spec.strip()))
    outdir = sys.argv[2]
    os.makedirs(outdir, exist_ok=True)

    base_poison = None
    print("%-28s %8s %8s %6s %8s %8s  %s" % (
        "rung", "blocked", "poison", "keys", "match", "fnbyte", "head key"))
    for i, (label, spec) in enumerate(steps):
        jsonl = os.path.join(outdir, "s%02d.jsonl" % i)
        out = scan(spec, jsonl)
        open(os.path.join(outdir, "s%02d.log" % i), "w").write(out)
        rows, graded, emit, fnb = read(jsonl)
        if rows != 878:
            raise SystemExit("READ %d ROWS, NOT 878 — refusing" % rows)
        # POSITIVE CONTROL. A run that graded nothing is a FAILURE, not a
        # confirmed null (STATUS.md trap 5); the 870/8 split is the known answer.
        if graded < 800:
            raise SystemExit("ONLY %d TUs GRADED (capture-fail on the rest) — "
                             "refusing to report a null" % graded)
        if not emit:
            raise SystemExit("emit_blockers EMPTY — refusing")
        bad = sum(v for k, v in emit.items() if k.startswith(BADTOKEN))
        if bad:
            raise SystemExit("SINK DEAD: expr-chain-badtoken on %d emitted "
                             "functions, spec=%r" % (bad, spec))
        poison = sum(v for k, v in emit.items() if k.startswith(POISON))
        blocked = sum(emit.values())
        head = emit.most_common(1)[0] if emit else ("<none>", 0)
        # the head EXCLUDING the poison row, which is not a blocker to widen
        headb = [(k, v) for k, v in emit.most_common() if not k.startswith(POISON)]
        head = headb[0] if headb else ("<none>", 0)
        if base_poison is None:
            base_poison = poison
        print("%-28s %8d %8d %6d %8s %8s  %s (%d)" % (
            label, blocked, poison, len(emit),
            metric(out, "match"), metric(out, "fnbyte-exact"),
            head[0], head[1]))
        json.dump({"label": label, "spec": spec, "blocked": blocked,
                   "poison": poison, "keys": len(emit),
                   "match": metric(out, "match"),
                   "mismatch": metric(out, "mismatch"),
                   "fnbyte_exact": metric(out, "fnbyte-exact"),
                   "top": emit.most_common(25),
                   "fn_poison": sum(v for k, v in fnb.items()
                                    if k.startswith(POISON)),
                   "fn_blocked": sum(fnb.values())},
                  open(os.path.join(outdir, "s%02d.json" % i), "w"), indent=1)


main()

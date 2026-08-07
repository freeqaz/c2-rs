#!/usr/bin/env python3
"""freeze_h.py — write GRID-H's frozen prediction column.

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

    freeze_h.py <gridH.json> <out.tsv>

RULE BIND predicts the caller's body **from the callee's own emitted body**, so
unlike `w-refbind`'s and `w-ilx`'s keys its prediction cannot be written down
before `cl.exe` runs — the right-hand side of the rule is a compiler output.
What is frozen instead is the **program**: `bind.py` and `gen_grids.py` are
committed, GRID-H's sources were committed at `5832dd14` with their `sha256`,
and this file is committed with the column it produces.

**This script is structurally incapable of seeing the answer.** `callee_only`
drops every `.text` COMDAT whose symbol starts with `?f@@` before the caller of
this function can look at it, so the caller's bytes are never in scope here and
never reach the frozen column. The answer is read for the first time by
`run_grid.py --frozen`, which re-checks every `sha256` and reads this column
rather than recomputing it.

Columns: `name  sha256  refusal-clause  predicted-words  callee-words`
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, HERE)

import json  # noqa: E402
import bind  # noqa: E402
import run_grid  # noqa: E402


def callee_only(obj_path):
    """Every `.text` COMDAT of the obj EXCEPT the caller's. The caller's bytes
    are dropped here so that nothing downstream of this function can read
    them."""
    cd = run_grid.text_comdats(obj_path)
    return {k: v for k, v in cd.items() if not k.startswith("?f@@")}


def main():
    cells = json.load(open(sys.argv[1]))
    out = []
    for c in cells:
        obj, err = run_grid.compile_cell(c)
        if err:
            out.append((c["name"], c["sha256"], "COMPILE-FAILED", "", ""))
            continue
        cd = callee_only(obj)
        gname = next((k for k in cd if k.startswith("?g@@")), None)
        aname = next((k for k in cd if k.startswith("?anchor@@")), None)
        if aname is None or cd[aname][1] != 1:
            out.append((c["name"], c["sha256"], "ANCHOR-BROKEN", "", ""))
            continue
        if gname is None:
            out.append((c["name"], c["sha256"], "D3-no-callee-comdat", "", ""))
            continue
        gw, gnrel = cd[gname]
        gbody = run_grid.hexw(gw)
        if gnrel != 0:
            out.append((c["name"], c["sha256"], "D3-callee-has-relocations",
                        "", gbody))
            continue
        if c["domain"] != "in":
            out.append((c["name"], c["sha256"], c["domain"][4:] + " (registered)",
                        "", gbody))
            continue
        n = c["n_caller_formals"]
        caller_hi = 3 + n - 1 if n else 2
        try:
            pred = bind.predict(gw, len(c["beta"]), [3 + p for p in c["beta"]],
                                c["mode"], c["k_scaled"], caller_hi)
        except bind.Refused as e:
            out.append((c["name"], c["sha256"], e.why, "", gbody))
            continue
        out.append((c["name"], c["sha256"], "", run_grid.hexw(pred), gbody))

    with open(sys.argv[2], "w") as fh:
        fh.write("# GRID-H FROZEN PREDICTIONS — lane w-alloc3\n")
        fh.write("# name\tsha256\trefusal\tpredicted\tcallee-body\n")
        for r in out:
            fh.write("\t".join(r) + "\n")
    npred = sum(1 for r in out if r[3])
    print("frozen %d cells: %d carry a PREDICTION, %d carry a refusal clause"
          % (len(out), npred, len(out) - npred))
    ref = {}
    for r in out:
        if not r[3]:
            ref[r[2]] = ref.get(r[2], 0) + 1
    for k, v in sorted(ref.items(), key=lambda kv: -kv[1]):
        print("    %-34s %d" % (k, v))


main()

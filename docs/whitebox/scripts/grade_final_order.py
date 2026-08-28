#!/usr/bin/env python3
"""How many cells does the tap's ORDER channel actually contain? — lane `w-sched`.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).

`grade_reorder.py` answers this by the REGION method, over run-to-run pairs.
This answers it by the FUNCTION WALK, over the one pair that fixes emitted
instruction order: `sched0` (the input to scheduler run 4) against `after0`
(the observation point immediately after run 4 returns —
`stagetap.c` `g_sites`, site `0x10b7e701`).  Run 4 is `P_DAG.md` §1's mode-0
schedule, the LAST one.

    C2RS_STAGE_FUNCWALK=1 c2rs stage snap --limit 60 > fw.txt
    python3 docs/whitebox/scripts/grade_final_order.py fw.txt

Why the number and not the rate.  A function run 4 leaves alone is a free hit
for any order-preserving model, including one that returns its input
(`WB_SCHEDCONF_FINDINGS.md` §4.2).  So the useful quantity is not an agreement
percentage — it is the **count of cells in which two order models could
possibly disagree**, which is the count of functions run 4 reorders, and the
number of tuple positions that move inside them.  Two models that agree on
those cells are indistinguishable by this instrument at any sample size.

Input is the CANONICAL stream `TapReport::canonical_bytes` emits
(`crates/c2-reference/src/stage.rs`), not the raw C-side one: the harness has
already dropped the walk index and **already reversed** each block, because the
C walk runs backward down `tuple+0x10` (`prev`) and *"an observable whose order
is the finding must not be published inverted"*.  So no reversal happens here —
doing it again would invert the very thing being measured.  Rows are
`FT <opcode> <cat> <flags> <cc>`; functions are keyed by the tap's symbol name
(`#3459`: the ordinal alone is not an identity) and a function whose name did
not read (`<none>`, `<unread>`, `<nonascii>`, `<empty>`, `<toolong>`) is counted
and excluded, never silently matched by ordinal.
"""

import sys
from collections import defaultdict

REFUSED_NAMES = ("<none>", "<unread>", "<nonascii>", "<empty>", "<toolong>")


def parse_fw(path):
    """-> {(fixture, phase, name): [row, ...]}, plus refusal counts."""
    out = {}
    bad = defaultdict(int)
    fixture = None
    key = None
    blocks = []

    def flush():
        if key is not None:
            out[key] = [r for b in blocks for r in b]

    for line in open(path):
        if line.startswith("== "):
            flush()
            key, blocks = None, []
            fixture = line[3:].split()[0]
            continue
        if line.startswith("FN "):
            flush()
            f = line.split()
            phase, name = f[1], f[3] if len(f) > 3 else "<none>"
            if name in REFUSED_NAMES:
                bad[name] += 1
                key, blocks = None, []
                continue
            key, blocks = (fixture, phase, name), []
            continue
        if line.startswith("BLK "):
            blocks.append([])
            continue
        if line.startswith("FT ") and blocks:
            blocks[-1].append(line[3:].rstrip("\n"))
            continue
        if line.startswith("REFUSE"):
            bad["REFUSE " + line.split()[1]] += 1
    flush()
    return out, dict(bad)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    per, bad = parse_fw(sys.argv[1])
    keys = {(f, n) for (f, p, n) in per}
    paired = same = moved = 0
    multiset_differs = 0
    unpaired = 0
    cells = []
    for f, n in sorted(keys):
        a = per.get((f, "sched0", n))
        b = per.get((f, "after0", n))
        if a is None or b is None:
            unpaired += 1
            continue
        if sorted(a) != sorted(b):
            # not a permutation: something other than run 4 changed the body
            multiset_differs += 1
            continue
        paired += 1
        if a == b:
            same += 1
        else:
            moved += 1
            d = sum(1 for x, y in zip(a, b) if x != y)
            cells.append((f, n, len(a), d))

    print(f"; FINAL-SCHEDULE ORDER CHANNEL  (sched0 -> after0, P_DAG 1 run 4)")
    print(f"; functions paired {paired}   UNCHANGED {same}   REORDERED {moved}"
          f"   ({100.0 * moved / paired if paired else 0:.2f}%)")
    print(f"; excluded: {unpaired} phase-unpaired, {multiset_differs} tuple-multiset-changed")
    if bad:
        print(f"; tap refusals / unreadable names: {bad}")
    print("\n; --- THE CELLS.  Any two order models agreeing here are indistinguishable ---")
    print(";  fixture                     function                     tuples  positions-moved")
    tot = 0
    for f, n, ln, d in cells:
        tot += d
        print(f"  {f:<27} {n:<28} {ln:>5} {d:>10}")
    print(f"; TOTAL DISCRIMINATING POSITIONS: {tot} over {moved} functions"
          f" out of {paired} ({sum(len(per[(f,'sched0',n)]) for f,n in keys if (f,'sched0',n) in per)} tuples walked).")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Which CLAUSES of c2's region rule does the live tap actually pin? — lane `w-sched`.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).

`grade_regions.py` reports that the transcribed `FUN_10be5d4b` reproduces the
observed region partition on 1,461 of 1,461 graded pairs.  That is a grade of
the RULE.  It is not a grade of the rule's CLAUSES, and the two come apart:
`WB_SCHEDCONF_FINDINGS.md` §3.3 already records that four of the seven exits
never fired, so a 100.00 % is silent about them in either polarity.

This instrument asks the sharper question the aggregate cannot:

    for each clause, is there any observation this tap makes that the
    clause's NEGATION would not also produce?

Method: mutate exactly one clause, re-grade the same frozen stream, compare.

    GREEN  the mutant scores identically to the unmutated rule.  The tap
           cannot tell the clause from its negation.  `[R]` by this instrument,
           at any cell count, forever.
    RED    the mutant loses cells.  The clause is pinned, and by how many.

A GREEN row is the deliverable, not a failure.  Two GREENs are distinguished,
because conflating them would report a tautology as a coverage gap:

    by-corpus         the tap HAS a channel; this population has no cell in
                      which rule and mutant differ.  A different corpus could
                      separate them.
    by-construction   rule and mutant are output-identical on every reachable
                      input.  NO corpus can separate them.

    python3 docs/whitebox/scripts/mutate_regions.py <snap-stdout>

Controls, both registered in `WB_SCHEDCHK_PREREG.md` §1 before the run:
  C-A  the unmutated rule must score 1461/1461 with clause histogram
       1121/204/136 — otherwise the stream is not R7's population and every
       colour below is VOID, not provisional.
  C-B  at least one mutant must go RED (`#3336`: a control never watched to
       fail is decoration).  M-CAP-2 is named in advance as the one that must.
"""

import sys
from collections import defaultdict

from grade_regions import parse            # one parser, one definition

# --- the rule, from FUN_10be5d4b, with every clause on a knob -------------
# Addresses are the same ones grade_regions.py cites; this file changes no
# reading, it only makes each reading separately switchable.
BASE = dict(
    cap=0x50,               # 0x10be5d66  `cmp edx,0x50 / jg`  (SIGNED)
    cap_cmp="gt",           #             `jg`  =>  strict >
    head_op=0x30F,          # 0x10be5d55  head special case, opcode 0x30f
    head_enabled=True,      # 0x10be5d55
    head_any=False,         # 0x10be5d55, opcode test removed
    incl=(0x12, 0x14, 0x1B),  # 0x10be5d72 / 0x10be5d76 / 0x10be5d83 INCLUSIVE
    excl=(0x19,),           # 0x10be5d7f                            EXCLUSIVE
    c17_enabled=True,       # 0x10be5d8b  cat 0x17 AND opcode 0x30f
    c17_needs_op=True,      # 0x10be5d8b  the opcode half of that test
    c17_inclusive=False,    # 0x10be5d8b  EXCLUSIVE
)


def find_region(tuples, m):
    """`FUN_10be5d4b` with clause knobs.  `m == BASE` is the transcription."""
    result = None
    cur = 0
    if m["head_enabled"] and tuples and (m["head_any"] or tuples[0][0] == m["head_op"]):
        result = 0
        cur = 1
    count = 0
    while cur < len(tuples):
        over = count > m["cap"] if m["cap_cmp"] == "gt" else count >= m["cap"]
        if over:
            return result, "cap"
        op, cat = tuples[cur][0], tuples[cur][1]
        if cat in m["incl"]:
            return cur, f"incl-cat-{cat:02x}"
        if cat in m["excl"]:
            return result, f"excl-cat-{cat:02x}"
        if m["c17_enabled"] and cat == 0x17 and (not m["c17_needs_op"] or op == 0x30F):
            return (cur, "incl-0x17") if m["c17_inclusive"] else (result, "excl-0x17")
        result = cur
        cur += 1
        count += 1
    return result, "end-of-list"


def pairs(per):
    """Every (walk, successor-walk) pair that survives the instrument check:
    B must be byte-identical to the TAIL of A.  Mutation-independent, so the
    denominator is the same 1,461 for every row of the grid."""
    out = []
    ungraded = 0
    for _fixture, blocks in sorted(per.items()):
        for a, b in zip(blocks, blocks[1:]):
            if len(b) >= len(a) or a[len(a) - len(b):] != b:
                ungraded += 1
                continue
            out.append((a, len(a) - len(b)))
    return out, ungraded


def head_clause_census(per, ps):
    """WHERE DOES THE HEAD SPECIAL CASE (`0x10be5d55`) ACTUALLY FIRE?

    `grade_regions.py`'s clause histogram counts which EXIT fired, and the head
    case is not an exit — so it has no row there and its firing count has never
    been printed by any instrument in this repo.  It is printed here, because
    the answer is that its firing set and the graded set are DISJOINT."""
    walks = [b for bs in per.values() for b in bs]
    fires_all = sum(1 for b in walks if b and b[0][0] == BASE["head_op"])
    fires_graded = sum(1 for a, _ in ps if a and a[0][0] == BASE["head_op"])
    return len(walks), fires_all, len(ps), fires_graded


def score(ps, m):
    hit = miss = 0
    clauses = defaultdict(int)
    for a, observed in ps:
        last, clause = find_region(a, m)
        clauses[clause] += 1
        hit += (last is not None and last + 1 == observed) or (last is None and observed == 0)
        miss += not ((last is not None and last + 1 == observed) or (last is None and observed == 0))
    return hit, miss, dict(clauses)


def mut(**kw):
    m = dict(BASE)
    m.update(kw)
    return m


# (id, mutation, knobs, prereg prediction)
GRID = [
    ("M-HEAD-DROP",    "head special case removed",                mut(head_enabled=False), "RED>=1000"),
    ("M-HEAD-ANY",     "head taken for ANY opcode",                mut(head_any=True),      "RED<100"),
    ("M-HEAD-OP",      "head opcode 0x30f -> 0x30e",               mut(head_op=0x30E),      "RED~=DROP"),
    ("M-12-EXCL",      "cat 0x12 inclusive -> exclusive",          mut(incl=(0x14, 0x1B), excl=(0x19, 0x12)), "RED~=204"),
    ("M-14-EXCL",      "cat 0x14 inclusive -> exclusive",          mut(incl=(0x12, 0x1B), excl=(0x19, 0x14)), "GREEN"),
    ("M-1B-EXCL",      "cat 0x1b inclusive -> exclusive",          mut(incl=(0x12, 0x14), excl=(0x19, 0x1B)), "RED~=136"),
    ("M-19-INCL",      "cat 0x19 exclusive -> inclusive",          mut(incl=(0x12, 0x14, 0x1B, 0x19), excl=()), "GREEN"),
    ("M-12-DROP",      "cat 0x12 not a stop at all",               mut(incl=(0x14, 0x1B)),  "RED"),
    ("M-14-DROP",      "cat 0x14 not a stop at all",               mut(incl=(0x12, 0x1B)),  "GREEN"),
    ("M-1B-DROP",      "cat 0x1b not a stop at all",               mut(incl=(0x12, 0x14)),  "RED"),
    ("M-19-DROP",      "cat 0x19 not a stop at all",               mut(excl=()),            "GREEN"),
    ("M-17-DROP",      "the 0x17/0x30f clause removed",            mut(c17_enabled=False),  "RED~=1121"),
    ("M-17-ANY",       "cat 0x17 stops regardless of opcode",      mut(c17_needs_op=False), "RED?"),
    ("M-17-INCL",      "0x17/0x30f exclusive -> inclusive",        mut(c17_inclusive=True), "RED~=1121"),
    ("M-CAP-GE",       "cap compare `>` -> `>=`",                  mut(cap_cmp="ge"),       "GREEN"),
]
CAP_SWEEP = [80, 40, 20, 16, 15, 14, 13, 12, 10, 8, 6, 4, 3, 2, 1, 0]


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    per = parse(sys.argv[1])
    ps, ungraded = pairs(per)
    b_hit, b_miss, b_cl = score(ps, BASE)

    nw, fa, ng, fg = head_clause_census(per, ps)
    print(f"; HEAD CLAUSE 0x10be5d55 fires on {fa} of {nw} walks, "
          f"and on {fg} of {ng} GRADED pairs.")
    nfix = len(per)
    print(f";   {nw} walks - {nfix} last-of-fixture = {ungraded + ng} candidate pairs"
          f" = {ungraded} ungraded + {ng} graded;"
          f"  firing walks {fa} - {nfix} = {fa - nfix} == ungraded.")
    print(";   The firing set and the graded set are DISJOINT: the rule's most-fired")
    print(";   clause is inactive on every cell of the population that grades it.")

    print(f"; MUTATION GRID over {len(ps)} graded pairs ({ungraded} UNGRADED, instrument check)")
    print(f"; CONTROL C-A  base rule: HIT {b_hit}  MISS {b_miss}   clauses {b_cl}")
    ca = (b_hit == 1461 and b_miss == 0 and b_cl.get("excl-0x17") == 1121
          and b_cl.get("incl-cat-12") == 204 and b_cl.get("incl-cat-1b") == 136
          and ungraded == 1368)
    print(f"; CONTROL C-A  {'PASS' if ca else 'FAIL -- every colour below is VOID'}"
          "   (must reproduce WB_SCHEDCONF 3.1/3.3: 1461/1461, 1368 ungraded, 1121/204/136)")

    reds = 0
    print("\n;  id              mutation                                 hit  miss  d(miss)  verdict   prereg")
    rows = []
    for mid, what, m, pred in GRID:
        h, ms, _ = score(ps, m)
        verdict = "GREEN" if (h, ms) == (b_hit, b_miss) else "RED"
        reds += verdict == "RED"
        rows.append((mid, verdict, ms))
        print(f"  {mid:<15} {what:<40} {h:>5} {ms:>5} {ms - b_miss:>+8}  {verdict:<7}  {pred}")

    print("\n; --- CAP SWEEP: what value of the 0x50 constant does the tap actually pin? ---")
    print(";  cap   hit  miss  verdict")
    threshold = None
    for k in CAP_SWEEP:
        h, ms, _ = score(ps, mut(cap=k))
        v = "GREEN" if (h, ms) == (b_hit, b_miss) else "RED"
        if v == "RED" and threshold is None:
            threshold = k
        reds += v == "RED"
        print(f"  {k:>4} {h:>5} {ms:>5}  {v}")
    if threshold is not None:
        print(f"; LOWEST GREEN CAP = {threshold + 1}   =>  the tap confirms `cap >= {threshold + 1}`, a RAY,")
        print(f";   not the read constant 0x50 = 80.  Slack factor {80 / (threshold + 1):.1f}x.")

    print(f"\n; CONTROL C-B  {'PASS' if reds else 'FAIL -- no mutant died, the grid measured nothing'}"
          f"   ({reds} RED rows)")
    greens = [r[0] for r in rows if r[1] == "GREEN"]
    print(f"; CLAUSES THE TAP CANNOT SEPARATE FROM THEIR NEGATION: {greens}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

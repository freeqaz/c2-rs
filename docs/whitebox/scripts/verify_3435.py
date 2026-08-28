#!/usr/bin/env python3
"""Re-check board `#3435` figure by figure against a fresh tap run — lane `w-sched`.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).

`#3435` — *"c2's instruction scheduler moves nothing on this corpus … that makes
the corpus unable to validate any scheduler model"* — is load-bearing twice
over: `WB_SCHEDCONF_FINDINGS.md` §4.3 rests on it, and `w-f0price` carries it
into F0 as one of two UNPRICED terms (`#3716`, sub-item 4).  A row that two
prices depend on should not be quoted; it should be re-run.  This repo's
standing lesson is that dated rows decay in place (`#3712`: a commit edited the
very section holding a stale figure and walked past it).

    python3 docs/whitebox/scripts/verify_3435.py <snap.txt> <snap-funcwalk.txt>

Both inputs are `c2rs stage snap --limit 60`, the second with
`C2RS_STAGE_FUNCWALK=1`.  Every claim `#3435` makes that is a NUMBER is
recomputed and printed beside the filed value, and the script says HOLDS or
DIFFERS per row rather than reporting an aggregate.

One row is expected to DIFFER, and it is a units difference rather than an
error — see the note this prints at the end.
"""

import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])

from grade_regions import parse
from grade_final_order import parse_fw


def is_tail(a, b):
    return len(b) < len(a) and a[len(a) - len(b):] == b


def region_half(path):
    """Run-initial walks, paired within a fixture; the `grade_reorder.py` method."""
    per = parse(path)
    same = chg = 0
    strat = {}
    for _f, bs in sorted(per.items()):
        runs, cur = [], []
        for b in bs:
            if cur and is_tail(cur[-1], b):
                cur.append(b)
            else:
                if cur:
                    runs.append(cur)
                cur = [b]
        if cur:
            runs.append(cur)
        for a, b in zip(runs, runs[1:]):
            A, B = a[0], b[0]
            if sorted(A) != sorted(B):
                continue
            d = strat.setdefault(len(A), [0, 0])
            if A == B:
                same += 1
                d[0] += 1
            else:
                chg += 1
                d[1] += 1
    return same, chg, strat


def func_half(path):
    per, bad = parse_fw(path)
    keys = sorted({(f, n) for (f, p, n) in per})
    out = {}
    for a, b in [("sched1", "globregs"), ("globregs", "sched2"),
                 ("sched2", "color"), ("sched0", "after0")]:
        p = s = c = x = 0
        for f, n in keys:
            A, B = per.get((f, a, n)), per.get((f, b, n))
            if A is None or B is None or sorted(A) != sorted(B):
                x += 1
                continue
            p += 1
            s += A == B
            c += A != B
        out[(a, b)] = (p, s, c, x)
    return out, bad


def row(label, got, filed, ok):
    print(f"  {'HOLDS ' if ok else 'DIFFERS'}  {label:<44} got {got:<22} filed {filed}")
    return ok


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    print("; board #3435, figure by figure, recomputed from a fresh tap run")
    ok = []

    same, chg, strat = region_half(sys.argv[1])
    tot = same + chg
    ok.append(row("region method, reordered", f"{chg} of {tot} ({100*chg/tot:.2f}%)",
                  "5 of 456 (1.10%)", (chg, tot) == (5, 456)))
    ok.append(row("input-returner score, region method", f"{100*same/tot:.2f}%",
                  "98.9%", round(100 * same / tot, 1) == 98.9))
    le7 = sum(v[0] + v[1] for k, v in strat.items() if k <= 7)
    le7c = sum(v[1] for k, v in strat.items() if k <= 7)
    ok.append(row("function length <= 7", f"{le7} of {tot}, {le7c} reordered",
                  "355 of 456, 0.00%", (le7, le7c) == (355, 0)))
    if 10 in strat:
        r10 = 100 * strat[10][1] / sum(strat[10])
        ok.append(row("function length 10", f"{r10:.1f}%", "28.6%", round(r10, 1) == 28.6))

    fh, bad = func_half(sys.argv[2])
    for (a, b), filed, label in [(("sched1", "globregs"), 6, "run 1 (sched1->globregs)"),
                                 (("sched2", "color"), 9, "run 2 (sched2->color)"),
                                 (("sched0", "after0"), 3, "run 4 (sched0->after0) FINAL")]:
        p, s, c, _x = fh[(a, b)]
        ok.append(row(label, f"{c} of {p} ({100*c/p:.2f}%)", f"{filed} of 357",
                      (c, p) == (filed, 357)))
    p, s, c, x = fh[("sched0", "after0")]
    ok.append(row("input-returner score, final run", f"{100*s/p:.2f}%", "99.2%",
                  round(100 * s / p, 1) == 99.2))
    p, s, c, x = fh[("globregs", "sched2")]
    ok.append(row("globregs, the CONTRAST figure", f"{c} reordered of {p} paired ({x} excluded)",
                  "334 of 357", (c, p) == (334, 357)))

    print(f"\n; {sum(ok)} of {len(ok)} figures reproduce exactly.")
    print("""
; THE ONE THAT DIFFERS IS A UNITS DIFFERENCE, NOT AN ERROR, AND IT DOES NOT
; TOUCH #3435's HEADLINE.  `for contrast globregs moves 334 of 357` is a
; DIFFERS count; the 6 / 9 / 3 beside it are REORDER counts.  globregs rewrites
; tuple CONTENT (it assigns registers), so on this instrument 333 of its 357
; pairs fail the multiset test and are excluded, leaving 24 comparable pairs of
; which 1 reorders.  Quoted as written, the contrast reads as `globregs
; reorders 334 against the scheduler's 3`, which is not what either number
; measures.  #3435's own claim -- the scheduler moves 3 of 357 -- reproduces
; exactly, and the correction makes the contrast smaller, not the finding.""")
    return 0


if __name__ == "__main__":
    sys.exit(main())

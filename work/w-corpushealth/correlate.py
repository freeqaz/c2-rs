#!/usr/bin/env python3
"""w-corpushealth — deliverable 1: does refusal CONCENTRATE in immature source?

Two independent cuts, both at the granularity where the decomp's ruler applies.

CUT 1 (per TU).  Split the 844 `vocab-gap` TUs by whether their objdiff unit is
FINISHED — every function in the unit at match_percent_normalized == 100, which
is the decomp's own "complete unit" definition (416/967 in its own headline).
Compare the refusal RATE R_i/|S_i| across the two groups. If corpus immaturity
drove refusal, the unfinished group must refuse harder.

CUT 2 (per body, name space).  Refusal rate cannot be split by name — the scan
publishes counts, not names — so this cut bounds instead: what share of each
group's emitted bodies sit on source the decomp has not matched.

Both cuts are reported with their denominators and with the population the
ruler CANNOT see stated as its own line, never folded in.
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
ns = json.load(open(os.path.join(HERE, "namespace.json")))
jn = json.load(open(os.path.join(HERE, "joined.json")))
J = {r["src"]: r for r in jn["rows"]}
rows = [r for r in ns["rows"] if r["ok"]]


def band(rs, label):
    R = sum(r["R"] for r in rs)
    S = sum(r["S"] for r in rs)
    U = sum(r["unfin"] for r in rs)
    F = sum(r["fin"] for r in rs)
    A = sum(r["absent"] for r in rs)
    rate = 100.0 * R / S if S else 0.0
    print(f"  {label:42s} TUs {len(rs):4d}  bodies {S:7d}  refused {R:7d}  "
          f"rate {rate:6.2f}%   unfin {U:5d} ({100.0*U/S if S else 0:4.2f}%)  "
          f"fin {F:6d}  ungradeable {A:7d} ({100.0*A/S if S else 0:5.1f}%)")
    return rate, len(rs), S, R


print("=" * 100)
print("CUT 1 — refusal rate vs the decomp's own per-unit completeness")
print("=" * 100)
vg = [r for r in rows if r["cls"] == "vocab-gap"]
fin_tu, unfin_tu, unj = [], [], []
for r in vg:
    j = J[r["src"]]
    if not j["joined"]:
        unj.append(r)
    elif j["U"] == 0:
        fin_tu.append(r)
    else:
        unfin_tu.append(r)
a = band(fin_tu, "unit FINISHED (every fn norm==100)")
b = band(unfin_tu, "unit NOT finished (>=1 fn norm<100)")
c = band(unj, "no objdiff unit at all")
print()
print(f"  refusal-rate ratio  NOT-finished / FINISHED = {b[0]/a[0]:.3f}")
print("  A ratio near 1.00 means the decomp's completeness carries NO signal about")
print("  whether the port refuses; a ratio >> 1 would be the hypothesis confirmed.")
print()

# finer: bucket by the unit's unfinished FRACTION
print("=" * 100)
print("CUT 1b — the same, banded by how unfinished the unit is")
print("=" * 100)
bands = [(0.0, 0.0), (0.0001, 0.05), (0.05, 0.15), (0.15, 0.40), (0.40, 1.01)]
for lo, hi in bands:
    sel = []
    for r in vg:
        j = J[r["src"]]
        if not j["joined"] or j["F"] == 0:
            continue
        f = j["U"] / j["F"]
        if (lo == hi == 0.0 and f == 0.0) or (lo > 0 and lo <= f < hi):
            sel.append(r)
    band(sel, f"unit unfinished fraction [{lo:.2f},{hi:.2f})" if lo != hi
         else "unit unfinished fraction == 0")
print()
print("=" * 100)
print("CUT 2 — the same population split by emitted-body verdict")
print("=" * 100)
T = lambda k: sum(r[k] for r in rows)
S, R = T("S"), T("R")
print(f"  emitted bodies {S}   refused {R}   ({100.0*R/S:.2f}% refusal rate overall)")
print(f"  GRADEABLE by the decomp's ruler: {T('unfin')+T('fin')+T('vendor')} "
      f"({100.0*(T('unfin')+T('fin')+T('vendor'))/S:.1f}%)")
print(f"     on UNMATCHED source: {T('unfin')}  "
      f"= {100.0*T('unfin')/(T('unfin')+T('fin')+T('vendor')):.2f}% of the gradeable population")
print(f"     on MATCHED source:   {T('fin')}")
print(f"  NOT GRADEABLE (name in no objdiff unit — the body is not in the shipped")
print(f"     image, so objdiff can never have a verdict on it): {T('absent')} "
      f"({100.0*T('absent')/S:.1f}%)")
print()
print("  Ceilings on 'refusals explained by unmatched source':")
print(f"     measured, from the gradeable part alone : {T('unfin')} / {R} = "
      f"{100.0*T('unfin')/R:.2f}%")
gr = T("unfin") + T("fin") + T("vendor")
print(f"     extrapolated at the gradeable rate to the")
print(f"     whole emitted population                : "
      f"{S*T('unfin')/gr:.0f} / {R} = {100.0*S*T('unfin')/gr/R:.2f}%")

# ---------------------------------------------------------------------------
# DENOMINATOR CROSS-CHECK, added after the first reading. `|S_i|` (every `.text`
# function symbol, 189,371) is the LARGER denominator, so every rate above is
# the conservative one; `fnbyte-denominator` is 162,046. Printed because a rate
# quoted on one denominator and compared against a number derived from the
# other is the kind of arithmetic this repo has paid for (STATUS.md trap 0).
print()
print("=" * 100)
print("CROSS-CHECK — the same three rates on the port's OWN denominator")
print("=" * 100)


def two(rs, label):
    R = sum(r["R"] for r in rs)
    S = sum(r["S"] for r in rs)
    E = sum(r["E"] for r in rs)
    print(f"  {label:34s} R {R:6d}  |S| {S:7d} -> {100.0*R/S:5.2f}%   "
          f"E {E:7d} -> {100.0*R/E:5.2f}%")
    return 100.0 * R / S, 100.0 * R / E


a2 = two(fin_tu, "unit FINISHED")
b2 = two(unfin_tu, "unit NOT finished")
two([r for r in fin_tu if r["unfin"] == 0], "the 200-TU clean set")
two(rows, "whole workload (all classes)")
print(f"  ratio not-finished / finished:  on |S| {b2[0]/a2[0]:.3f}   "
      f"on fnbyte-denominator {b2[1]/a2[1]:.3f}   — invariant to the choice")

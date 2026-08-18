#!/usr/bin/env python3
"""xs.py — GRID-S: did c2 KEEP the call, banded by the callee's IL segment length
AND by its emitted `.text`, over the 878-TU workload.

`w-fence2`'s GRID-W banded this by emitted `.text` only.  `w-dataseam` §6.1 then
fitted a cut on the callee's `.ex` IL segment length to `fnbyte-exact`.  This
aggregation puts both units on the oracle's own axis so the IL-byte cut can be
scored against real c2 rather than against the grader.

Derived from the scan log, never accumulated.
"""
import collections
import json
import sys

rows = [json.loads(l) for l in open(sys.argv[1])]
rows = [r for r in rows if r.get("record") != "provenance"]
agg = collections.Counter()
for r in rows:
    for k, v in (r.get("emit") or {}).items():
        if k.startswith("xs-"):
            agg[k] += v


def band_table(unit, title):
    bands = sorted({k.split("=")[1] for k in agg if f"|{unit}=" in k})
    print(f"\n=== callee size banded by {title} ===")
    print(f"{'band':>7} {'kept':>8} {'inlined':>8} {'unknown':>8}  {'wrong if cut here':>18}")
    tk = ti = tu_ = 0
    for b in bands:
        k = agg.get(f"xs-kept|{unit}={b}", 0)
        i = agg.get(f"xs-inlined|{unit}={b}", 0)
        u = agg.get(f"xs-unknown|{unit}={b}", 0)
        tk += k
        ti += i
        tu_ += u
        mark = "  <-- MIXED" if k and i else ""
        print(f"{b:>7} {k:>8} {i:>8} {u:>8}{mark}")
    print(f"{'TOTAL':>7} {tk:>8} {ti:>8} {tu_:>8}")
    return tk, ti, tu_


band_table("ref", "EMITTED .text (GRID-W's unit, 16 B bands)")
band_table("il", "IL SEGMENT LENGTH (w-dataseam's unit, 16 B bands)")

print("\n=== w-dataseam's fitted cut, scored against the ORACLE ===")
print("A cut says: IL segment > N  =>  assume c2 KEPT the call (exempt it).")
print("So `inlined & gt` is the cut asserting KEPT where c2 INLINED — the")
print("unsound direction — and `kept & le` is the cut asserting INLINED where")
print("c2 KEPT.\n")
print(f"{'cut':>6} {'kept&gt':>9} {'kept&le':>9} {'inlined&gt':>11} {'inlined&le':>11} "
      f"{'WRONG':>8} {'of':>7} {'err%':>7}")
for cut in (128, 180, 192, 224, 231, 256):
    kg = agg.get(f"xs-cut{cut}|kept|gt", 0)
    kl = agg.get(f"xs-cut{cut}|kept|le", 0)
    ig = agg.get(f"xs-cut{cut}|inlined|gt", 0)
    il = agg.get(f"xs-cut{cut}|inlined|le", 0)
    # the cut's own claim: gt => kept, le => inlined
    wrong = ig + kl
    tot = kg + kl + ig + il
    pct = 100.0 * wrong / tot if tot else 0.0
    print(f"{cut:>6} {kg:>9} {kl:>9} {ig:>11} {il:>11} {wrong:>8} {tot:>7} {pct:>6.1f}%")

noil = {c: agg.get(f"xs-cut{c}|kept|noil", 0) + agg.get(f"xs-cut{c}|inlined|noil", 0)
        for c in (180,)}
print(f"\nedges with no census seg_len for the callee (unbandable): {noil[180]}")

print("\n=== the JOINT cell — is IL length a FAITHFUL proxy for emitted, or only correlated? ===")
js = sorted({k.split("|j=")[1] for k in agg if "|j=" in k})
print(f"{'ref_band':>9} {'il_band':>9} {'kept':>8} {'inlined':>8}")
for j in js:
    k = agg.get(f"xs-kept|j={j}", 0)
    i = agg.get(f"xs-inlined|j={j}", 0)
    if k + i == 0:
        continue
    a, b = j.split("_")
    print(f"{a:>9} {b:>9} {k:>8} {i:>8}")

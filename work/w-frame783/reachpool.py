#!/usr/bin/env python3
"""w-frame783 — the reach-pool (`emit-predicate-worth`, B∧C ∖ A∧B∧C) re-read at
this lane's tip, under all four readers.

`w-selbind` §4 published it as *0 of 124 by coverage, 12 of 124 by the gate's
framing, 123 of 124 by the window-free one*. The framing is shipped now, so the
"gate's framing" row is a different reader than it was and has to be re-taken
rather than carried.

    reachpool.py <factors.tsv> <jsonl>
"""
import sys, json

tsv, jl = sys.argv[1], sys.argv[2]
rows = {}
for l in open(jl):
    if '"record"' in l[:14]:
        continue
    r = json.loads(l)
    rows[r["src"]] = r

pool, abc, bc = [], [], []
for line in open(tsv):
    if line.startswith("#"):
        continue
    f = line.rstrip("\n").split("\t")
    src, _cls, A, B, C = f[0], f[1], f[2], f[3], f[4]
    inb, inc, ina = B == "1", C == "1", A == "1"
    if inb and inc:
        bc.append(src)
        if ina:
            abc.append(src)
        else:
            pool.append(src)

print(f"B∧C {len(bc)}   A∧B∧C {len(abc)}   reach-pool (B∧C ∖ A∧B∧C) {len(pool)}")


def has(src, k):
    return ((rows.get(src, {}).get("bind_checks") or {}).get(k, 0)) > 0


def cover_full(src):
    g = rows.get(src, {}).get("gl_body_starts")
    return bool(g) and g[0] == g[1]


print("\nthe reach-pool, under each reader:")
print(f"   {sum(1 for s in pool if cover_full(s)):5d} of {len(pool)}  "
      f"`.gl` SPELLS every segment's body-start  (gl_body_start_coverage n/n)")
for k, label in (
    ("selbind-emit-subset-scan-narrow-tus", "walk-free scan, INCUMBENT framing"),
    ("selbind-emit-subset-scan-precise-tus", "walk-free scan, SHIPPED framing"),
    ("selbind-emit-subset-wide-tus", "walk-free scan, window-free framing"),
    ("selbind-emit-subset-gate-tus", "the GATE's binding walk"),
):
    print(f"   {sum(1 for s in pool if has(s, k)):5d} of {len(pool)}  {label}")

print("\n…and the same four over ALL 871 graded TUs, for the join:")
allsrc = [s for s in rows if rows[s]["class"] != "capture-fail"]
print(f"   {sum(1 for s in allsrc if cover_full(s)):5d}  coverage n/n")
for k, label in (
    ("selbind-emit-subset-scan-narrow-tus", "scan, incumbent"),
    ("selbind-emit-subset-scan-precise-tus", "scan, shipped"),
    ("selbind-emit-subset-wide-tus", "scan, window-free"),
    ("selbind-emit-subset-gate-tus", "the gate's walk"),
):
    print(f"   {sum(1 for s in allsrc if has(s, k)):5d}  {label}")

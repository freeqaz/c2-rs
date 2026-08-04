#!/usr/bin/env python3
"""w-metric scratch analysis over a `c2rs gap --jsonl` file.

Prints: (1) the near-miss distribution the diagnosis needs, (2) the
progress-mass computation cross-checked two ways. Tooling only — the shipped
implementation is in crates/c2-harness/src/gap.rs.
"""
import json, sys

path = sys.argv[1]
rows = [json.loads(l) for l in open(path) if l.strip()]
rows = [r for r in rows if r.get("record") != "provenance"]
graded = [r for r in rows if r["class"] != "capture-fail"]
print(f"rows {len(rows)}  graded {len(graded)}")

by_class = {}
for r in rows:
    by_class[r["class"]] = by_class.get(r["class"], 0) + 1
print("classes:", dict(sorted(by_class.items())))

# --- (1) near-miss distribution: blocked emitted fraction per failing TU ---
fail = [r for r in graded if r["class"] != "match"]
fracs = []
no_denominator = 0
for r in fail:
    e = r["emit"].get("emit-emitted", 0)
    i = r["emit"].get("emit-in-class", 0)
    if e == 0:
        no_denominator += 1
        continue
    fracs.append((e - i) / e)
fracs.sort()
n = len(fracs)
print(f"\nfailing TUs {len(fail)}, with emitted denominator {n}, without {no_denominator}")
def pct(p):
    return fracs[min(n - 1, int(p * n))]
print("blocked-emitted-fraction deciles over failing TUs with a denominator:")
print("  " + "  ".join(f"p{int(p*100):02d}={pct(p):.2f}" for p in
                       [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]))
lo = sum(1 for f in fracs if f < 0.10)
full = sum(1 for f in fracs if f >= 0.999)
print(f"  failing TUs with <10% of emitted fns blocked: {lo} ({lo/n*100:.1f}%)")
print(f"  failing TUs with 100% of emitted fns blocked:  {full} ({full/n*100:.1f}%)")

# --- port emits an object at all (what any output-similarity metric needs) ---
emits = sum(1 for r in graded if r["class"] in ("match", "mismatch"))
print(f"\nTUs where the port emits an object (match+mismatch): {emits} of {len(graded)}"
      f"  -> output-similarity undefined on {(1-emits/len(graded))*100:.1f}%")

# --- (2) progress mass, two computations that must agree ---
def has(r, k):
    return 1 if k in r["emit"] else 0
guard = lambda r: r["class"] != "mismatch"
A = sum(has(r, "emit-set-ceiling-gate") for r in graded if guard(r))
B = sum(has(r, "emit-set-ceiling-today") for r in graded if guard(r))
C = sum(has(r, "emit-sec-reachable") for r in graded if guard(r))
# Denominator over ALL graded TUs (mismatches included) — matches the Rust
# implementation: zeroing a mismatch TU's numerator must cost, so its emitted
# functions stay in the denominator.
E = sum(r["emit"].get("emit-emitted", 0) for r in graded)
I = sum(r["emit"].get("emit-in-class", 0) for r in graded if guard(r))
g = len(graded)
P = (A / g + B / g + C / g + I / E) / 4
print(f"\nprogress mass inputs: A {A}  B {B}  C {C}  in-class/emitted {I}/{E}  graded {g}")
print(f"progress mass P = {P:.5f}  ({P*100:.2f}%)")
print(f"terms: a={A/g:.5f} b={B/g:.5f} c={C/g:.5f} f={I/E:.5f}")

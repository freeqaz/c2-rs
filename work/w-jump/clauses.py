#!/usr/bin/env python3
"""w-jump — which CLAUSE of `shapes::counted_accum_loop` each `expr-jump` body
dies on, over the whole workload.

Two instrumented scans (both scratch, both reverted):

  scan_inst.jsonl   the compound `expr-jump|…` key   -> the family membership
  scan_clause.jsonl the recognizer's own `Err` COMMITTED (w-bdnz §5.1's probe
                    pointed at the workload) -> `ctr-loop-<clause>|…`

They are joined on `(src, index)`, so the clause histogram is over EXACTLY the
2,286 bodies / 302 emitted of the family and not over some larger population the
committal instrument also moved. That join is asserted.

**The clause is a FIRST blocker inside the recognizer**, exactly as `expr-jump`
is a first blocker inside the ladder — a body that dies at clause 1 says nothing
about clause 20. It is read only in the direction that is sound: dying LATE is
evidence of being close; dying EARLY is evidence of being far.

Usage: clauses.py FAMILY.jsonl CLAUSE.jsonl [--sample CLAUSE N]
"""
import json
import sys
from collections import Counter, defaultdict


def load(path):
    """(src, index) -> (key, cflow, seg_len, name, n) for every compound row,
    per column."""
    body, emit = {}, {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        src = r.get("src", "?")
        for col, m in ((body, r.get("fn_blockers") or {}),
                       (emit, r.get("emit_blockers") or {})):
            for k, n in m.items():
                if "|" not in k:
                    continue
                p = k.split("|", 8)
                col[(src, int(p[7]))] = (p[0], p[1], int(p[4]), p[8], n)
    return body, emit


fam_b, fam_e = load(sys.argv[1])
cl_b, cl_e = load(sys.argv[2])

fam_b = {k: v for k, v in fam_b.items() if v[0] == "expr-jump"}
fam_e = {k: v for k, v in fam_e.items() if v[0] == "expr-jump"}
print(f"family: {sum(v[4] for v in fam_b.values())} bodies, "
      f"{sum(v[4] for v in fam_e.values())} emitted")

miss_b = [k for k in fam_b if k not in cl_b]
miss_e = [k for k in fam_e if k not in cl_e]
print(f"  family rows with no clause row: bodies {len(miss_b)}, "
      f"emitted {len(miss_e)}  (must be 0)")

if "--sample" in sys.argv:
    want = sys.argv[sys.argv.index("--sample") + 1]
    n = int(sys.argv[sys.argv.index("--sample") + 2])
    # `--emit` picks the column that RANKS; `--uniq` samples distinct NAMES
    # rather than distinct rows, because the body column is dominated by one
    # header inline replicated once per TU and a row sample only ever shows it.
    fam, cl = (fam_e, cl_e) if "--emit" in sys.argv else (fam_b, cl_b)
    hits = [(k, cl[k]) for k in fam if k in cl and cl[k][0] == want]
    print(f"\n=== {want}: {len(hits)} rows ===")
    if "--uniq" in sys.argv:
        seen, uniq = set(), []
        for k, v in sorted(hits, key=lambda kv: kv[1][2]):
            if v[3] in seen:
                continue
            seen.add(v[3])
            uniq.append((k, v))
        print(f"    {len(uniq)} distinct names")
        hits = uniq
    step = max(1, len(hits) // n)
    for (src, idx), v in hits[::step][:n]:
        print(f"  {src}  #{idx}  seg={v[2]}  {v[1]}")
        print(f"    {v[3]}")
    sys.exit(0)

# --- the clause histogram, both columns --------------------------------------
b, e = Counter(), Counter()
bnames = defaultdict(set)
for k, v in fam_b.items():
    c = cl_b.get(k, ("<no clause row>",))[0]
    b[c] += v[4]
    bnames[c].add(v[3])
for k, v in fam_e.items():
    e[cl_e.get(k, ("<no clause row>",))[0]] += v[4]

tb, te = sum(b.values()), sum(e.values())
print(f"\n{'clause':44s} {'bodies':>7s} {'%':>6s} {'emitted':>8s} {'%':>6s} "
      f"{'names':>6s}")
for k, n in b.most_common():
    print(f"{k:44s} {n:7d} {100*n/tb:6.1f} {e[k]:8d} "
          f"{(100*e[k]/te if te else 0):6.1f} {len(bnames[k]):6d}")
for k, n in e.most_common():
    if k not in b:
        print(f"{k:44s} {0:7d} {0.0:6.1f} {n:8d} {100*n/te:6.1f} {0:6d}")
print(f"{'TOTAL':44s} {tb:7d} {100.0:6.1f} {te:8d} {100.0:6.1f}")

# --- NEIGHBOURS vs STRANGERS -------------------------------------------------
# A NEIGHBOUR is a body board #1988's named extensions (a)-(c) could reach: the
# clause it dies on is one of those extensions and nothing structural is known to
# be wrong with it yet. Everything else is a STRANGER — it dies on a clause that
# is a different construct, not a wider version of this one.
NEIGHBOUR = {
    # (a) n_three / n_long — more formals, or a non-`int` formal
    "ctr-loop-formals-not-2", "ctr-loop-formals-alias",
    # (b) the relation: `<=`, `!=`, descending
    "ctr-loop-test-not-lt", "ctr-loop-test-brfalse",
    # (c) start != 0
    "ctr-loop-ctr-start-not-zero",
}
nb = sum(n for k, n in b.items() if k in NEIGHBOUR)
ne = sum(n for k, n in e.items() if k in NEIGHBOUR)
print(f"\nNEIGHBOURS (board #1988 extensions (a)-(c)): "
      f"{nb} bodies ({100*nb/tb:.1f}%), {ne} emitted ({100*ne/te:.1f}%)")
print(f"STRANGERS: {tb-nb} bodies ({100*(tb-nb)/tb:.1f}%), "
      f"{te-ne} emitted ({100*(te-ne)/te:.1f}%)")

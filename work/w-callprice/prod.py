#!/usr/bin/env python3
"""w-callprice — the `prod` axis of the family, on BOTH columns, with the
replication discount. Reads the compound key produced by the §2.1 instrument.

Usage: prod.py SCAN.jsonl [--tag TAG]      # --tag prints per-key detail
"""
import json
import sys
from collections import Counter, defaultdict

FAMILY = "expr-call-in-expr"
PATH = sys.argv[1]

rows_b, rows_e = [], []
for line in open(PATH):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    src = r.get("src", "?")
    for col, field in ((rows_b, "fn_blockers"), (rows_e, "emit_blockers")):
        for k, n in (r.get(field) or {}).items():
            if not k.startswith(FAMILY):
                continue
            p = k.split("|", 9)
            col.append((src, p[0], p[3], p[9], n))   # src, key, prod, name, n

tb = sum(r[4] for r in rows_b)
te = sum(r[4] for r in rows_e)
print(f"family: bodies {tb}, emitted {te}")

if "--tag" in sys.argv:
    want = sys.argv[sys.argv.index("--tag") + 1]
    hits = [r for r in rows_e if r[2] == want]
    hb = [r for r in rows_b if r[2] == want]
    keys = Counter()
    for r in hits:
        keys[r[1]] += r[4]
    names = Counter()
    for r in hits:
        names[r[3]] += r[4]
    print(f"\n=== prod tag {want!r}: {sum(r[4] for r in hb)} bodies / "
          f"{sum(r[4] for r in hits)} emitted, {len(names)} distinct names, "
          f"{len({r[0] for r in hits})} TUs ===")
    for k, v in keys.most_common(15):
        print(f"  {v:6d} emitted  {k}")
    print("  top names:")
    for k, v in names.most_common(10):
        print(f"  {v:6d}  {k}")
    sys.exit(0)

b, e = Counter(), Counter()
gb, ge = defaultdict(set), defaultdict(set)
tus = defaultdict(set)
for src, key, prod, name, n in rows_b:
    b[prod] += n
    gb[prod].add(name)
for src, key, prod, name, n in rows_e:
    e[prod] += n
    ge[prod].add(name)
    tus[prod].add(src)

print(f"\n=== the `prod` axis, EMITTED-ranked, with the discount ===")
print(f"{'prod tag':52s} {'emitted':>8s} {'%':>6s} {'names':>6s} {'TUs':>5s} "
      f"{'em/name':>7s} {'bodies':>9s} {'em/1k':>6s}")
for k, v in e.most_common(24):
    nn = len(ge[k] - {"-"})
    print(f"{k:52s} {v:8d} {100*v/te:6.2f} {nn:6d} {len(tus[k]):5d} "
          f"{(v/nn if nn else 0):7.1f} {b[k]:9d} {(1000*v/b[k] if b[k] else 0):6.1f}")
print(f"{'TOTAL':52s} {te:8d} {100.0:6.2f}")
assert sum(e.values()) == te and sum(b.values()) == tb, "prod axis does not partition"
print("  ASSERTED: the prod axis partitions both columns exactly.")

#!/usr/bin/env python3
"""w-callprice — THE DELIVERABLE'S RANKING.

Ranks the family's keys on the EMITTED column three ways and prints them side by
side, because the three disagree and the disagreement is the finding:

  RAW        emitted symbols — what the census metric actually moves
  CONSTRUCTS distinct mangled names — how much DISTINCT source work it is
  LEVERAGE   raw / constructs — emitted symbols bought per construct admitted

w-jump #2000 discounted a BODY column by TU replication. On an EMITTED column
the same replication does not discount the metric — an emitted COMDAT in 419 TUs
is 419 emitted symbols in the census — it CONCENTRATES the work. So the discount
runs the other way and both numbers have to be printed.

Also isolates the `-whole` suffix: the census's own statement that granting the
SECOND blocker finishes the body, i.e. the one-construct-away population.

Usage: rank.py SCAN.jsonl [--top N]
"""
import json
import sys
from collections import Counter, defaultdict

FAMILY = "expr-call-in-expr"
PATH = sys.argv[1]
TOP = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 22

emit, body = defaultdict(list), Counter()
for line in open(PATH):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    src = r.get("src", "?")
    for k, n in (r.get("fn_blockers") or {}).items():
        if k.startswith(FAMILY):
            body[k.split("|", 1)[0]] += n
    for k, n in (r.get("emit_blockers") or {}).items():
        if k.startswith(FAMILY):
            p = k.split("|", 9)
            emit[p[0]].append((src, p[9], n))

te = sum(n for v in emit.values() for _, _, n in v)
tb = sum(body.values())
assert te and tb


def stats(k):
    rows = emit[k]
    n = sum(x[2] for x in rows)
    names = Counter()
    for _, nm, c in rows:
        if nm != "-":
            names[nm] += c
    tus = len({x[0] for x in rows})
    top = names.most_common(1)[0] if names else ("-", 0)
    return n, len(names), tus, top


rows = [(k, *stats(k)) for k in emit]
print(f"family: {tb} bodies / {te} emitted, {len(emit)} keys on the emitted column")
allnames = Counter()
for v in emit.values():
    for _, nm, c in v:
        if nm != "-":
            allnames[nm] += c
print(f"distinct mangled names over the whole emitted column: {len(allnames)} "
      f"({te} emitted symbols → {te/len(allnames):.2f} emitted per construct; "
      f"{100*(1-len(allnames)/te):.1f} % of the column is TU replication)")

print(f"\n=== RANKED BY RAW EMITTED (what the metric moves) ===")
print(f"{'#':>3s} {'emitted':>7s} {'%':>5s} {'cons':>5s} {'lev':>5s} {'TUs':>5s} "
      f"{'bodies':>8s} {'em/1k':>6s} {'top name x':>10s}  key")
by_raw = sorted(rows, key=lambda r: -r[1])
for i, (k, n, cons, tus, top) in enumerate(by_raw[:TOP]):
    print(f"{i+1:3d} {n:7d} {100*n/te:5.1f} {cons:5d} {(n/cons if cons else 0):5.1f} "
          f"{tus:5d} {body[k]:8d} {1000*n/body[k]:6.1f} {top[1]:10d}  {k[:64]}")

print(f"\n=== RANKED BY CONSTRUCTS (distinct mangled names) — the DISCOUNT ===")
print(f"{'#':>3s} {'cons':>5s} {'emitted':>7s} {'lev':>5s} {'raw rank':>8s}  key")
raw_rank = {k: i + 1 for i, (k, *_) in enumerate(by_raw)}
for i, (k, n, cons, tus, top) in enumerate(
        sorted(rows, key=lambda r: -r[2])[:TOP]):
    print(f"{i+1:3d} {cons:5d} {n:7d} {(n/cons if cons else 0):5.1f} "
          f"{raw_rank[k]:8d}  {k[:64]}")

print(f"\n=== RANKED BY LEVERAGE (emitted per construct), keys with ≥150 emitted ===")
print(f"{'#':>3s} {'lev':>6s} {'emitted':>7s} {'cons':>5s} {'TUs':>5s} "
      f"{'raw rank':>8s}  key   [top name]")
lev = [r for r in rows if r[1] >= 150]
for i, (k, n, cons, tus, top) in enumerate(
        sorted(lev, key=lambda r: -(r[1] / max(r[2], 1)))[:TOP]):
    print(f"{i+1:3d} {n/max(cons,1):6.1f} {n:7d} {cons:5d} {tus:5d} "
          f"{raw_rank[k]:8d}  {k[:56]}")
    print(f"{'':>36s}  {top[0][:100]}  x{top[1]}")

# --- the `-whole` slice: the census's own one-construct-away statement --------
def wholeish(k):
    return k.endswith("-whole") or (k.rsplit("-", 1)[-1].startswith("whole"))


wn = sum(n for k, n, *_ in rows if wholeish(k))
wc = sum(c for k, n, c, *_ in rows if wholeish(k))
wb = sum(body[k] for k, *_ in rows if wholeish(k))
print(f"\n=== the `-whole…` slice — granting the SECOND blocker finishes the body ===")
print(f"  {wn} emitted ({100*wn/te:.1f} %), {wb} bodies ({100*wb/tb:.1f} %), "
      f"{wc} name-slots")
print(f"{'emitted':>7s} {'cons':>5s} {'lev':>5s} {'bodies':>8s}  key")
for k, n, cons, tus, top in sorted(
        [r for r in rows if wholeish(r[0])], key=lambda r: -r[1])[:14]:
    print(f"{n:7d} {cons:5d} {(n/cons if cons else 0):5.1f} {body[k]:8d}  {k[:64]}")

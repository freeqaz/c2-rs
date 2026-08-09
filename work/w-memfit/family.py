#!/usr/bin/env python3
"""w-memfit — the block-move family's first-blocker population, both columns.

A first-blocker count is a SIZE and not a price (#2025 built a 2,188-emitted key
and converted zero).  It is measured here so §5.4 of the rung quotes a number
somebody can re-derive, and so the ratio between the two callees is on record:
`memset` is the large one by a wide margin and obeys the same expansion rule.

Usage:  family.py <scan.jsonl>
"""

import collections
import json
import sys

body = collections.Counter()
emit = collections.Counter()
for line in open(sys.argv[1]):
    r = json.loads(line)
    if r.get("record") == "provenance":
        continue
    for k, v in (r.get("fn_blockers") or {}).items():
        body[k] += v
    for k, v in (r.get("emit_blockers") or {}).items():
        emit[k] += v

tb, te = sum(body.values()), sum(emit.values())
print("fn_blockers   %d keys, sum %d" % (len(body), tb))
print("emit_blockers %d keys, sum %d" % (len(emit), te))
print()
print("%-34s %9s %7s   %9s %7s" % ("key", "bodies", "%", "emitted", "%"))
fam = ["expr-intrinsic-memcpy", "expr-intrinsic-memset"]
for k in fam:
    print("%-34s %9d %6.3f%%   %9d %6.3f%%"
          % (k, body[k], 100 * body[k] / tb, emit[k], 100 * emit[k] / te))
sb = sum(body[k] for k in fam)
se = sum(emit[k] for k in fam)
print("%-34s %9d %6.3f%%   %9d %6.3f%%"
      % ("  the family", sb, 100 * sb / tb, se, 100 * se / te))
print()
print("memset / memcpy on the emitted column: %.1fx"
      % (emit["expr-intrinsic-memset"] / emit["expr-intrinsic-memcpy"]))
print()
print("the neighbouring reader clauses, for the mmio chain:")
for k in sorted(body):
    if "lit-permuted" in k or "tail-lit" in k or "multiarg-lit" in k:
        print("   %-34s bodies %6d   emitted %6d" % (k, body[k], emit[k]))

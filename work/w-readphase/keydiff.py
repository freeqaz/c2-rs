#!/usr/bin/env python3
"""w-readphase — diff two scans' emitted blocker histograms, key by key."""
import collections
import json
import sys


def emit(path):
    c = collections.Counter()
    g = 0
    for line in open(path):
        if not line.startswith('{"src"'):
            continue
        r = json.loads(line)
        if r.get("class") != "capture-fail":
            g += 1
        for k, v in (r.get("emit_blockers") or {}).items():
            c[k] += v
    if g < 800:
        raise SystemExit("ONLY %d GRADED — refusing" % g)
    return c


a, b = emit(sys.argv[1]), emit(sys.argv[2])
print("A total %d (%d keys)   B total %d (%d keys)   delta %+d"
      % (sum(a.values()), len(a), sum(b.values()), len(b),
         sum(b.values()) - sum(a.values())))
d = {k: b[k] - a[k] for k in set(a) | set(b) if b[k] != a[k]}
print("\nkeys that GREW in B (B absorbed the de-accepted functions):")
for k, v in sorted(d.items(), key=lambda kv: -kv[1])[:20]:
    print("  %+8d  %-52s  %d -> %d" % (v, k, a[k], b[k]))
print("\nkeys that SHRANK in B:")
for k, v in sorted(d.items(), key=lambda kv: kv[1])[:15]:
    print("  %+8d  %-52s  %d -> %d" % (v, k, a[k], b[k]))

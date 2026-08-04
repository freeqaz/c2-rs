#!/usr/bin/env python3
"""Sample the .ex (IL body) size of capture-cache entries.

The fork server's win is a fixed per-process cost, so it is worth a lot on a
small TU and nothing on a big one. This is the population denominator: what
sizes does the project's real cold oracle traffic actually consist of?

usage: cachesizes.py <n-to-sample> [--stride K]
"""
import os, sys

def main():
    n = int(sys.argv[1])
    stride = 1
    if "--stride" in sys.argv:
        stride = int(sys.argv[sys.argv.index("--stride") + 1])
    src = "work/capture-cache"
    szs = []
    seen = 0
    with os.scandir(src) as it:
        for ent in it:
            if len(szs) >= n:
                break
            if not ent.is_dir():
                continue
            seen += 1
            if seen % stride:
                continue
            try:
                with os.scandir(ent.path) as it2:
                    for f in it2:
                        if f.name.startswith("_CL_") and f.name.endswith("ex"):
                            szs.append(f.stat().st_size)
                            break
            except OSError:
                pass
    szs.sort()
    if not szs:
        print("SAMPLED NOTHING — failure, not a result")
        sys.exit(1)
    def pct(p):
        return szs[min(len(szs) - 1, int(len(szs) * p))]
    print("sampled %d entries (scanned %d dirs, stride %d)" % (len(szs), seen, stride))
    print("  .ex bytes: min %d  p10 %d  p50 %d  p90 %d  p99 %d  max %d"
          % (szs[0], pct(.10), pct(.50), pct(.90), pct(.99), szs[-1]))
    for thr in (10000, 20000, 50000, 100000):
        k = sum(1 for s in szs if s >= thr)
        print("  >= %7d bytes: %6d  (%.2f%%)" % (thr, k, 100.0 * k / len(szs)))

main()

#!/usr/bin/env python3
"""w-readpx — the predicted `fnbyte-exact` delta, and the confound in the
prior, named.

Every BLOCKED emitted row is `fnbyte-refused` by construction, so no candidate
population can be crossed with the byte judge directly (#2095's requirement
cannot be met by the census). The only calibrated predictor available at this
tip is the exact rate of the ALREADY-ADMITTED population, and this script
prints both the prior and the two populations that confound it.
"""
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEM = sys.argv[1] if len(sys.argv) > 1 else "hex"
BUCKETS = [(0, 8), (8, 16), (16, 32), (32, 64), (64, 128), (128, 256),
           (256, 1 << 30)]


def bucket(n):
    for lo, hi in BUCKETS:
        if lo <= n < hi:
            return (lo, hi)
    return BUCKETS[-1]


def main():
    rows = []
    for line in open(os.path.join(HERE, STEM + ".err"),
                     encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 10:
            continue
        rows.append({"tu": f[1], "name": f[2], "fnb": f[3], "key": f[4],
                     "cflow": f[5], "off": f[6], "cls": f[7],
                     "bytes": int(f[8])})
    inc = [r for r in rows if r["cls"] == "in"]

    print("## The 64-128 B bucket, where P(exact) collapses to 0.011 — WHO IS IN IT\n")
    b = [r for r in inc if 64 <= r["bytes"] < 128]
    print("in-class 64-128 B: %d, exact %d\n"
          % (len(b), sum(1 for r in b if r["fnb"] == "fnbyte-exact")))
    print("| census key | n | exact |")
    print("|---|---:|---:|")
    for k, n in collections.Counter(r["key"] for r in b).most_common(10):
        e = sum(1 for r in b if r["key"] == k and r["fnb"] == "fnbyte-exact")
        print("| `%s` | %d | %d |" % (k, n, e))
    print("\n**That is the confound, named**: the bucket is dominated by the "
          "classes `w-fltret` (#2082) and `w-fltret2` (#2091) showed c2 INLINES. "
          "The prior is not a property of size; it is a property of that "
          "population, and it must be read as a LOWER bound for a body with no "
          "inlinable callee and as the right number for one with.\n")

    prior = {}
    for lo, hi in BUCKETS:
        v = [r for r in inc if lo <= r["bytes"] < hi]
        if v:
            prior[(lo, hi)] = (
                sum(1 for r in v if r["fnb"] == "fnbyte-exact") / len(v),
                len(v))

    # the 16 reachable frontier candidates
    out = os.path.join(HERE, STEM + ".out")
    port = [l.strip().split("|")[0].strip()
            for l in open(out, encoding="utf-8", errors="replace")
            if "| WHOLE    |" in l]
    fr, on = [], False
    for line in open(out, encoding="utf-8", errors="replace"):
        if line.startswith("  FRONTIER — "):
            on = True
            continue
        if on:
            parts = [p.strip() for p in line.split("|")]
            if len(parts) != 3:
                break
            fr.append(parts[2])
    cand = [r for r in rows if r["tu"] in fr and r["fnb"] == "fnbyte-refused"
            and r["cflow"] in port]

    print("## Predicted `fnbyte-exact` delta of transcribing each of the %d "
          "CFG-reachable frontier candidates\n" % len(cand))
    print("| TU | function | bytes | size bucket | P(exact) prior | n behind the prior |")
    print("|---|---|---:|---|---:|---:|")
    tot = 0.0
    for r in sorted(cand, key=lambda r: -r["bytes"]):
        lo, hi = bucket(r["bytes"])
        p, n = prior.get((lo, hi), (0.0, 0))
        tot += p
        print("| `%s` | `%s` | %d | %d-%s B | %.3f | %d |"
              % (r["tu"].split("/")[-1], r["name"], r["bytes"], lo,
                 "inf" if hi > 1 << 20 else hi, p, n))
    print("| **sum** | | | | **%.1f** | |" % tot)
    print("\n**Predicted `fnbyte-exact` delta if all %d were transcribed: "
          "+%.0f of a possible +%d** (base 36,228 -> %.0f, FBM 0.20243 -> "
          "%.5f over the fixed denominator 178,977)."
          % (len(cand), tot, len(cand), 36228 + tot, (36228 + tot) / 178977))

    print("\n## The calibration this replaces guessing with\n")
    print("| mechanism | landed today | admitted | `fnbyte-exact` delta | TUs |")
    print("|---|---|---:|---:|---:|")
    print("| bespoke one-function transcription | 7 classes | 7 | **+7** | +7 |")
    print("| a wide reader admission | `w-fltret` | 444 | **+0** | +0 |")


main()

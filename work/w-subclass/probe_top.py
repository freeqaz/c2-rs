#!/usr/bin/env python3
"""Lane w-subclass: which frontier TUs have blocked functions the census never
gave a CFG class, i.e. `classified < blocked_total`?

The screen returns NeedsClass BEFORE it checks the shortfall, so a TU that is
BOTH blocked on a missing class AND carrying unclassified bodies reports only
the first. Under the TOP bound the class blocker vanishes and the shortfall
surfaces -- which is why TOP is 14 and not 15. Analysis only; not shipped.
"""
import json
import sys

frontier = [
    "src/Main.cpp",
    "src/keygen_xbox.cpp",
    "src/system/math/Primes.cpp",
    "src/system/negate_test.cpp",
    "src/system/rndobj/wordwrap.cpp",
    "src/system/synth_xbox/Biquad.cpp",
    "src/system/synth_xbox/IPP_basicmath_xbox.cpp",
    "src/system/utl/EncryptXTEA.cpp",
    "src/system/utl/Pool.cpp",
    "src/xdk/LIBCMT/osfinfo.cpp",
    "src/xdk/LIBCMT/undname.cpp",
    "src/xdk/LIBCMT/vsnprnc.cpp",
    "src/xdk/LIBCMT/vswprnc.cpp",
    "src/xdk/nuispeech/mmio.cpp",
    "src/xdk/nuispeech/xboxheap.cpp",
    "src/xdk/xjson/jsonwriter.cpp",
    "src/xdk/xlrc/xlrcimpl.cpp",
]

short = []
for line in open(sys.argv[1]):
    r = json.loads(line)
    if "src" not in r or r["src"] not in frontier:
        continue
    blocked = sum(r["fn_blockers"].values())
    classified = sum(n for k, n in r["fn_cflow"].items() if "|" in k)
    if classified < blocked:
        short.append((r["src"], blocked, classified))

print(f"frontier TUs with an UNCLASSIFIED shortfall: {len(short)} of {len(frontier)}")
for src, b, c in sorted(short):
    print(f"  {src:<50} blocked {b:>3}  classified {c:>3}  shortfall {b - c}")

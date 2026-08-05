#!/usr/bin/env python3
"""Lane w-subclass: does `classified` over-count?

`cfg_reach` sums EVERY crossed `class|key` row into `classified` and compares
against `sum(fn_blockers)`. But `fn_cflow`'s cross is written over every
function, and `FnVerdict::key` spells IN-CLASS labels and BLOCKER keys into one
namespace (scan.rs), so a frontier TU with any in-class function contributes
rows to `classified` that are not in `fn_blockers`. If that happens, the
`classified < blocked_total` shortfall test can be masked. Positive check with
a printed count. Analysis only; not shipped.
"""
import json
import sys

frontier = {
    "src/Main.cpp", "src/keygen_xbox.cpp", "src/system/math/Primes.cpp",
    "src/system/negate_test.cpp", "src/system/rndobj/wordwrap.cpp",
    "src/system/synth_xbox/Biquad.cpp",
    "src/system/synth_xbox/IPP_basicmath_xbox.cpp",
    "src/system/utl/EncryptXTEA.cpp", "src/system/utl/Pool.cpp",
    "src/xdk/LIBCMT/osfinfo.cpp", "src/xdk/LIBCMT/undname.cpp",
    "src/xdk/LIBCMT/vsnprnc.cpp", "src/xdk/LIBCMT/vswprnc.cpp",
    "src/xdk/nuispeech/mmio.cpp", "src/xdk/nuispeech/xboxheap.cpp",
    "src/xdk/xjson/jsonwriter.cpp", "src/xdk/xlrc/xlrcimpl.cpp",
}

over = []
allrows = 0
for line in open(sys.argv[1]):
    r = json.loads(line)
    if "src" not in r:
        continue
    blocked = sum(r["fn_blockers"].values())
    classified = sum(n for k, n in r["fn_cflow"].items() if "|" in k)
    if r["src"] in frontier:
        allrows += 1
        if classified > blocked:
            over.append((r["src"], blocked, classified, r.get("fn_in_class", 0)))

print(f"frontier TUs where classified > blocked: {len(over)} of {allrows}")
for src, b, c, ic in sorted(over):
    print(f"  {src:<50} blocked {b:>3}  classified {c:>3}  fn_in_class {ic:>3}")

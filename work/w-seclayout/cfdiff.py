#!/usr/bin/env python3
"""Per-TU diff of the counterfactual scan against the base scan.

The question the metric keys cannot answer: on the TUs the relaxed walk newly
BINDS, what refuses them next — and is that a FENCE or is it luck?  #232 is
exactly the case where the next refusal was absent and the obj went out wrong.
"""
import json


def rows(path):
    out = {}
    for line in open(path):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        out[d["src"]] = d
    return out


b = rows("work/w-seclayout/base.jsonl")
c = rows("work/w-seclayout/cf26.jsonl")

moved_class = [s for s in b if b[s]["class"] != c[s]["class"]]
print(f"TUs whose CLASS moved: {len(moved_class)}")
for s in moved_class:
    print(f"   {s}: {b[s]['class']} -> {c[s]['class']}")

moved_cause = [s for s in b if b[s].get("gate_cause") != c[s].get("gate_cause")]
print(f"\nTUs whose gate FIRST CAUSE moved: {len(moved_cause)}")
hist = {}
for s in moved_cause:
    k = (b[s].get("gate_cause"), c[s].get("gate_cause"))
    hist[k] = hist.get(k, 0) + 1
for (x, y), n in sorted(hist.items(), key=lambda kv: -kv[1]):
    print(f"   {x}  ->  {y}    x{n}")

targets = set(open("work/w-seclayout/target380.txt").read().split())
print(f"\nof the 380: {sum(1 for s in moved_cause if s in targets)} moved first cause, "
      f"{sum(1 for s in moved_class if s in targets)} moved class")

print("\nselective_bind (records, segments, unclaimed_mangled, unclaimed_inline_fit)"
      " on the READ seven:")
for s in ["src/system/synth_xbox/HeadsetXferEffect.cpp",
          "src/system/synth_xbox/MeterEffect.cpp",
          "src/system/utl/TempoMap.cpp",
          "src/xdk/LIBCMT/rtti.cpp",
          "src/xdk/nuiapi/headtracker.cpp",
          "src/system/synth/Pollable.cpp",
          "src/system/utl/UrlEncode.cpp"]:
    print(f"   {s}")
    print(f"      base {b[s]['selective_bind']}  cause {b[s]['gate_cause']}")
    print(f"      cf26 {c[s]['selective_bind']}  cause {c[s]['gate_cause']}")
    print(f"      cf26 causes {c[s]['gate_causes']}")

#!/usr/bin/env python3
"""THE HAZARD, measured over its own population.

`Bindings::per_record` is the 1:1 contract: records 1:1 with ALL `.ex`
segments, and the port then emits ONE function per segment.  It has no
clause 4 — the over-emit obligation `Bindings::selective` states explicitly
(*"a segment c2 DISCARDED that the port binds and emits"*) is discharged on
the 1:1 path by **nothing at all**, on the unstated assumption that a `.gl`
record set that covers every segment IS c2's emit set.

w-selbind proved that assumption false for the selective path (#2820).  This
asks whether it is false for the **1:1** path too, over the 29 TUs
`gl_body_start_coverage` reports as `n of n` — the population `CEILING.md` §12
calls "full coverage of this acceptance path".

factor A is exactly the test: `.ex` segments == obj `.text` COMDATs.
A coverage TU that FAILS factor A is a TU where the 1:1 path would emit more
functions than c2 did — #232's shape, at file offset 2.
"""
import json

fac = {}
for line in open("work/w-seclayout/factors.tsv"):
    if line.startswith("#"):
        continue
    p = line.rstrip("\n").split("\t")
    if len(p) < 8:
        continue
    fac[p[0]] = {"class": p[1], "A": p[2] == "1", "B": p[3] == "1",
                 "C": p[4] == "1", "letters": p[7]}

cover, hazard = [], []
for line in open("work/w-seclayout/base.jsonl"):
    d = json.loads(line)
    if d.get("record") == "provenance":
        continue
    gbs = d.get("gl_body_starts")
    if not gbs or gbs[0] != gbs[1]:
        continue
    cover.append(d["src"])
    f = fac.get(d["src"], {})
    if not f.get("A"):
        hazard.append((d["src"], gbs, f.get("letters"), d["class"],
                       d.get("gate_cause")))

print(f"`.gl` spells a body-start for EVERY `.ex` segment: {len(cover)} TUs")
print(f"of those, factor A FAILS (obj `.text` COMDATs != `.ex` segments): "
      f"{len(hazard)}")
print("\nthese are the TUs where the 1:1 path would emit a function c2 did not,")
print("and where NO clause discharges it — `Bindings::per_record` has no clause 4:")
for src, gbs, letters, cls, cause in sorted(hazard):
    print(f"   {src}")
    print(f"      gl_body_starts {gbs[0]} of {gbs[1]}   factors {letters}   "
          f"class {cls}   gate_cause {cause}")

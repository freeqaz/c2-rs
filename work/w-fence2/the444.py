#!/usr/bin/env python3
"""w-fence2 deliverable 3 — w-fltret's 444, located BY NAME, at both ends.

w-fltret (#2082) admitted 444 emitted functions of the `call-sequence-value-fp`
class and `fnbyte-exact` moved by zero: every one is byte-WRONG. A narrowed
fence must not let one reach an obj. "Re-admitted" = its TU emits an obj, i.e.
the TU's verdict is `match` or `mismatch`.
"""
import json
import sys

base, tip = sys.argv[1], sys.argv[2]


def load(p):
    out = {}
    for line in open(p):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        out[r["src"]] = r
    return out


A, B = load(base), load(tip)
KEY = "call-sequence-value-fp"
tot_a = tot_b = 0
tus = []
for src, r in sorted(A.items()):
    n = 0
    for k, v in (r.get("emit") or {}).items():
        if KEY in k and k.startswith("emit-shape"):
            n += v
    # the class shows up in the per-function census keys too
    for k, v in (r.get("fn_frames") or {}).items():
        if KEY in k:
            n += v
    if n:
        tus.append((src, n))
        tot_a += n
print(f"TUs carrying the `{KEY}` class: {len(tus)}, functions {tot_a}")
emitting = [(s, n) for s, n in tus if B.get(s, {}).get("class") in ("match", "mismatch")]
print(f"…of those, TUs that EMIT AN OBJ at the tip: {len(emitting)}")
for s, n in emitting:
    print(f"    RE-ADMITTED {s}  ({n} functions, tip class {B[s]['class']})")
print(f"RE-ADMITTED TOTAL: {sum(n for _, n in emitting)}")
print()
print("TUs whose verdict moved at all, by name:")
for s in sorted(set(A) | set(B)):
    if A.get(s, {}).get("class") != B.get(s, {}).get("class"):
        print(f"    {s}: {A.get(s, {}).get('class')} -> {B.get(s, {}).get('class')}"
              f"   [{KEY} functions here: {dict(tus).get(s, 0)}]")

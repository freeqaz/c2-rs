#!/usr/bin/env python3
"""w-fltret2 — are the two independently written readers the SAME rung?

Two sessions were dispatched on w-callprice's R2 without either knowing of the
other. This compares what each converted, per `(TU, emit_name)` against the same
base scan, and then compares the emitted `.text` words function by function.

A count agreeing proves nothing (444 and 444 could be disjoint sets); the set and
the bytes are the claim.

Usage: replicate.py [BASE.jsonl MINE.jsonl THEIRS.jsonl]
"""
import json
import sys

BASE, MINE, THEIRS = (
    sys.argv[1:4]
    if len(sys.argv) >= 4
    else (
        "work/w-fltret2/base.fndiff.jsonl",
        "work/w-fltret2/tip.fndiff.jsonl",
        "work/w-fltret2/mastertip.fndiff.jsonl",
    )
)


def load(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        out[(r["tu"], r["sym"])] = r
    return out


base, mine, theirs = load(BASE), load(MINE), load(THEIRS)
nm, nt = set(mine) - set(base), set(theirs) - set(base)
print(f"base differs {len(base)}   this lane {len(mine)}   landed {len(theirs)}")
print(f"newly differing:  this lane {len(nm)}   landed {len(nt)}")
print(f"  identical set? {nm == nt}   symmetric difference {len(nm ^ nt)}")
same = sum(1 for k in nm & nt if mine[k]["port_hex"] == theirs[k]["port_hex"])
print(f"  of the shared {len(nm & nt)}, port .text words identical on {same}")
assert nm == nt and same == len(nm), "the two readers are NOT the same rung"
print("  ASSERTED: same set, same bytes.")

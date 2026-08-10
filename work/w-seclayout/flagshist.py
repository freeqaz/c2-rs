#!/usr/bin/env python3
"""Flags-byte histogram over the records that are ACTUALLY EMITTED, per TU set.

The distinction that matters: on the 23 matching TUs the candidate rule agrees
32/32, but if every one of those 32 reads `flags == 0` the agreement is
VACUOUS — the control does not exercise the ANY branch at all and cannot
confirm it.  Print the split rather than the agreement count.
"""
import glob
import sys

sys.path.insert(0, "work/w-seclayout")
from seclayout import read_obj, IMAGE_SCN_LNK_COMDAT  # noqa: E402

hist = {}
for name in sys.argv[1:]:
    objp = glob.glob(f"work/w-seclayout/obj/{name}.obj")
    if not objp:
        continue
    secs = read_obj(objp[0])
    emitted = {}
    for s in secs:
        if s["name"] == ".text" and s["chars"] & IMAGE_SCN_LNK_COMDAT:
            for sym, _v in s["syms"]:
                emitted[sym] = s["sel"]
    for line in open(f"work/w-seclayout/cap/{name}/walk.tsv").read().splitlines()[1:]:
        _p, _s, _v, lk, fl, _i, nm = line.split("\t")
        if nm not in emitted:
            continue
        k = (lk, fl, emitted[nm])
        hist[k] = hist.get(k, 0) + 1
print("   linkage flags -> obj Selection   count")
for (lk, fl, sel), n in sorted(hist.items(), key=lambda kv: (-kv[1], str(kv[0]))):
    f = int(fl) if fl else None
    print(f"      {lk:>3}  0x{f:02x} -> {sel}   x{n}"
          f"{'   (COMDAT-linkage bit set)' if f and f & 0x20 else ''}")

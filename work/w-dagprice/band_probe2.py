#!/usr/bin/env python3
"""w-dagprice probe 2 — separate the SCHEDULER component from the band.

probe 1 (band_probe.py) showed the band's 13th entry, 0x10be663e, lies inside
the band by 2 of its 1,197 bytes and has no call relationship with the other 12.
This probe tests the 12 alone, and looks for a positive TU signal in .data.
"""
import os
import bisect
import collections

EXP = os.path.expanduser("~/ghidra-projects/export/c2")
OUT = os.path.dirname(os.path.abspath(__file__))


def rows(p):
    with open(os.path.join(EXP, p), encoding="utf-8", errors="replace") as f:
        hdr = None
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if hdr is None:
                hdr = parts
                continue
            yield dict(zip(hdr, parts))


funcs = {}
for r in rows("functions.tsv"):
    funcs[int(r["addr"], 16)] = int(r["size"])
starts = sorted(funcs)


def owner(a):
    i = bisect.bisect_right(starts, a) - 1
    if i < 0:
        return None
    s = starts[i]
    return s if a < s + funcs[s] else None


callers = collections.defaultdict(set)
callees = collections.defaultdict(set)
for r in rows("calls.tsv"):
    try:
        fi, ti = int(r["caller_addr"], 16), int(r["callee_addr"], 16)
    except ValueError:
        continue          # EXTERNAL:... thunk targets
    callers[ti].add(fi)
    callees[fi].add(ti)

DATA_KINDS = {"READ", "WRITE", "READ_WRITE", "DATA"}
data_from = collections.defaultdict(set)
for r in rows("xrefs.tsv"):
    if r["type"] not in DATA_KINDS:
        continue
    try:
        to = int(r["to"], 16)
    except ValueError:
        continue
    ff = r["from_func"]
    src = int(ff, 16) if ff not in ("-", "") else owner(int(r["from"], 16))
    data_from[to].add(src)

# ---- the 12: the band minus 0x10be663e ----------------------------------
S12 = [0x10BE5CCE, 0x10BE5CEA, 0x10BE5D06, 0x10BE5D4B, 0x10BE5DB0, 0x10BE5DF6,
       0x10BE5FBB, 0x10BE6046, 0x10BE607F, 0x10BE60C0, 0x10BE626C, 0x10BE6382]
s12 = set(S12)
print("== THE 12 (0x10be5cce .. 0x10be663d) ==")
print("  span %#x..%#x = %d bytes"
      % (S12[0], S12[-1] + funcs[S12[-1]] - 1,
         S12[-1] + funcs[S12[-1]] - S12[0]))
ext = [(a, sorted(c for c in callers[a] if c not in s12)) for a in S12]
ext = [(a, c) for a, c in ext if c]
for a, c in ext:
    print("  ENTRY %#x  from %s" % (a, ", ".join(hex(x) for x in c)))
print("  external entry points: %d of 12" % len(ext))

# ---- the 13th, and the cluster it actually belongs to --------------------
print("\n== 0x10be663e, THE 13th BAND MEMBER ==")
for a in (0x10BE663E, 0x10BE6AEB, 0x10BE6BCA, 0x10BE5CBE):
    print("  %#x size=%-5d callers=%-40s callees=%s"
          % (a, funcs.get(a, -1),
             ",".join(hex(x) for x in sorted(callers[a])) or "-",
             ",".join(hex(x) for x in sorted(callees[a])) or "-"))

# ---- band-private data for the 12 ---------------------------------------
d12 = sorted(d for d, s in data_from.items() if s & s12)
priv = [d for d in d12 if data_from[d] <= s12]
print("\n== DATA PRIVATE TO THE 12 ==")
print("  %d data addresses touched; %d private to the 12" % (len(d12), len(priv)))
runs, cur = [], [priv[0]]
for d in priv[1:]:
    if d - cur[-1] <= 0x40:
        cur.append(d)
    else:
        runs.append(cur)
        cur = [d]
runs.append(cur)
for c in runs:
    print("  cluster %#x..%#x n=%d" % (c[0], c[-1], len(c)))

# ---- the .data neighbourhood of the big cluster --------------------------
LO, HI = 0x10C3CE00, 0x10C3D400
print("\n== .data NEIGHBOURHOOD %#x..%#x ==" % (LO, HI))
print("  every referenced datum in the window, with which TU-band its "
      "referrers live in")


def zone(f):
    if f is None:
        return "?"
    if f in s12:
        return "SCHED12"
    if 0x10BE5CBE <= f <= 0x10BE717F:
        return "gap-nbr"
    if 0x10B3219F <= f <= 0x10B3433F:
        return "dag.c"
    if 0x10C1B900 <= f <= 0x10C1C400:
        return "mdmisc"
    return "other"


with open(os.path.join(OUT, "DATA_WINDOW.tsv"), "w") as fh:
    fh.write("addr\tnref\tzones\treferrers\n")
    for d in sorted(x for x in data_from if LO <= x <= HI):
        srcs = sorted(x for x in data_from[d] if x is not None)
        zs = sorted({zone(x) for x in srcs})
        line = "%08x\t%d\t%s\t%s" % (d, len(srcs), "|".join(zs),
                                     ",".join("%08x" % x for x in srcs))
        fh.write(line + "\n")
        print("  " + line)

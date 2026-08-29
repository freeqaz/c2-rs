#!/usr/bin/env python3
"""w-dagprice: measure the [dag] band's attribution from the flat Ghidra export.

Read-only against ~/ghidra-projects/export/c2 (regenerable with
docs/whitebox/scripts/ExportFlat.java).  Writes only into work/w-dagprice/.
No crates/ code.  Every number this lane publishes about the band comes from
here, so a later lane can correct the inputs rather than withdraw the figure
(ROADMAP.md 11.8).
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
    funcs[int(r["addr"], 16)] = (int(r["size"]), r["name"])

starts = sorted(funcs)


def owner(a):
    i = bisect.bisect_right(starts, a) - 1
    if i < 0:
        return None
    s = starts[i]
    return s if a < s + funcs[s][0] else None


# --- the band, as the repo states it -------------------------------------
BAND_LO, BAND_HI = 0x10BE5CCE, 0x10BE663F          # SUBSYS.md / P_DAG.md
DAGC_LO, DAGC_HI = 0x10B3219F, 0x10B3433F          # dag.c anchor band
band = [a for a in starts if BAND_LO <= a <= BAND_HI]
dagc = [a for a in starts if DAGC_LO <= a <= DAGC_HI]

print("== EXTENT ==")
print("band %#x-%#x: %d function entries" % (BAND_LO, BAND_HI, len(band)))
last = band[-1]
print("  last entry %#x size=%d -> ends %#x  (%d of its %d bytes lie inside the band)"
      % (last, funcs[last][0], last + funcs[last][0] - 1,
         BAND_HI - last + 1, funcs[last][0]))
prev = band[-2]
print("  penultimate %#x size=%d -> ends %#x"
      % (prev, funcs[prev][0], prev + funcs[prev][0] - 1))
print("dag.c band %#x-%#x: %d function entries" % (DAGC_LO, DAGC_HI, len(dagc)))
dlast = dagc[-1]
print("  last entry %#x size=%d -> ends %#x"
      % (dlast, funcs[dlast][0], dlast + funcs[dlast][0] - 1))
print("  P_DAG.md:9-10 asserts 61 = 48 + 13; measured = %d + %d = %d"
      % (len(dagc), len(band), len(dagc) + len(band)))

# --- call closure ---------------------------------------------------------
callers = collections.defaultdict(set)
callees = collections.defaultdict(set)
for r in rows("calls.tsv"):
    try:
        fi = int(r["caller_addr"], 16)
        ti = int(r["callee_addr"], 16)
    except (TypeError, ValueError, KeyError):
        continue
    callers[ti].add(fi)
    callees[fi].add(ti)

bandset = set(band)
print("\n== CALL CLOSURE ==")
ext_entries = []
for a in band:
    ext = sorted(c for c in callers[a] if c not in bandset)
    if ext:
        ext_entries.append((a, ext))
        print("  %#x entered from outside: %s" % (a, ", ".join(hex(x) for x in ext)))
print("  %d of %d band functions have an external caller" % (len(ext_entries), len(band)))
print("  outbound (band -> outside):")
for a in band:
    ext = sorted(c for c in callees[a] if c not in bandset)
    if ext:
        print("    %#x -> %s" % (a, ", ".join(hex(x) for x in ext)))

# --- data locality --------------------------------------------------------
DATA_KINDS = {"READ", "WRITE", "READ_WRITE", "DATA"}
data_from = collections.defaultdict(set)
for r in rows("xrefs.tsv"):
    if r["type"] not in DATA_KINDS:
        continue
    ff = r["from_func"]
    if ff in ("-", ""):
        ff = None
    try:
        to = int(r["to"], 16)
    except ValueError:
        continue          # Stack[...] and register operands
    src = int(ff, 16) if ff else owner(int(r["from"], 16))
    data_from[to].add(src)

band_data = sorted(d for d, srcs in data_from.items() if srcs & bandset)
private = [d for d in band_data if data_from[d] <= bandset]
shared = [d for d in band_data if not (data_from[d] <= bandset)]
print("\n== DATA LOCALITY ==")
print("  %d distinct data addresses referenced from the band" % len(band_data))
print("  %d are BAND-PRIVATE (no referrer outside the band)" % len(private))
print("  %d are shared with code outside the band" % len(shared))
with open(os.path.join(OUT, "BAND_DATA.tsv"), "w") as f:
    f.write("addr\tprivate\tnref\treferrers\n")
    for d in band_data:
        srcs = sorted(x for x in data_from[d] if x is not None)
        f.write("%08x\t%s\t%d\t%s\n"
                % (d, "yes" if data_from[d] <= bandset else "no", len(srcs),
                   ",".join("%08x" % x for x in srcs)))
if private:
    lo, hi = min(private), max(private)
    print("  band-private span %#x..%#x = %d bytes" % (lo, hi, hi - lo))
    runs, cur = [], [private[0]]
    for d in private[1:]:
        if d - cur[-1] <= 0x40:
            cur.append(d)
        else:
            runs.append(cur)
            cur = [d]
    runs.append(cur)
    print("  band-private clusters (a gap > 0x40 splits): %d" % len(runs))
    for c in runs:
        print("    %#x..%#x  n=%d" % (c[0], c[-1], len(c)))

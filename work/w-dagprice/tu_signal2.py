#!/usr/bin/env python3
"""w-dagprice probe 3b — the DATA-BLOCK test, with the positive control REPAIRED.

Probe 3 (tu_signal.py) ran the test and the POSITIVE CONTROL CAME BACK LOWER
THAN THE SUBJECT on every TU (dag.c 5, color.c 6, reader.c 7, emit.cpp 4, vs the
subject's 14).  That is a broken control, not a strong subject: c2_tus.tsv's
anchors bracket only the ICE-BEARING functions, so an anchor-extent control is a
SUBSET of its compiland, and every datum a left-out sibling also touches is
scored "shared" and thrown away.  The control is biased down by construction.

This probe repairs it by using FUNCS.tsv's `tu` column — the nearest-anchor
partition, i.e. the full hypothesised compiland — and reports BOTH.  It also
locates the negative control's 40 winners, because "0.82 % of windows beat it"
means nothing until you know whether those 40 are TUs or table blocks.
"""
import os
import bisect
import collections
import statistics

EXP = os.path.expanduser("~/ghidra-projects/export/c2")
REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.dirname(os.path.abspath(__file__))


def rows(path, base=EXP):
    with open(os.path.join(base, path), encoding="utf-8", errors="replace") as f:
        hdr = None
        for line in f:
            if line.startswith("#"):
                continue
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

RW_LO, RW_HI = 0x10C2E000, 0x10C90000
rwdata = sorted(d for d in data_from if RW_LO <= d < RW_HI)


def score(fnset, gap=0x40, minlen=4):
    priv = [d for d in rwdata if data_from[d] and data_from[d] <= fnset]
    if not priv:
        return 0, None, 0
    runs, cur = [], [priv[0]]
    for d in priv[1:]:
        if d - cur[-1] <= gap:
            cur.append(d)
        else:
            runs.append(cur)
            cur = [d]
    runs.append(cur)
    best = max(runs, key=len)
    return (len(best) if len(best) >= minlen else 0,
            (best[0], best[-1]), len(priv))


S12 = [0x10BE5CCE, 0x10BE5CEA, 0x10BE5D06, 0x10BE5D4B, 0x10BE5DB0, 0x10BE5DF6,
       0x10BE5FBB, 0x10BE6046, 0x10BE607F, 0x10BE60C0, 0x10BE626C, 0x10BE6382]
subj_n, subj_rng, _ = score(set(S12))
print("SUBJECT  the 12: run=%d %#x..%#x" % (subj_n, subj_rng[0], subj_rng[1]))

# ---- positive control, REPAIRED: the whole nearest-anchor partition -------
tu_of = {}
for r in rows("ref/FUNCS.tsv", base=os.path.join(REPO, "docs", "whitebox")):
    tu_of[int(r["addr"], 16)] = (r["tu"], r["tu_conf"])
by_tu = collections.defaultdict(set)
for a, (tu, conf) in tu_of.items():
    if tu not in ("-", "") and conf != "n/a":
        by_tu[tu].add(a)

print("\nPOSITIVE CONTROL, REPAIRED — full nearest-anchor partition per TU")
print("  (>= 12 functions, so the subject's k is not an advantage)")
res = []
for tu, fs in sorted(by_tu.items()):
    if len(fs) < 12:
        continue
    n, rng, tot = score(fs)
    res.append((n, tu, len(fs), rng, tot))
res.sort(reverse=True)
for n, tu, k, rng, tot in res:
    print("  %-32s nfun=%-4d run=%-4d %-24s private=%d"
          % (tu, k, n, "%#x..%#x" % rng if rng else "-", tot))
vals = [n for n, *_ in res]
print("  %d TUs; median run %.1f, mean %.2f, max %d; %d of %d score >= the "
      "subject's %d"
      % (len(vals), statistics.median(vals), statistics.fmean(vals), max(vals),
         sum(1 for v in vals if v >= subj_n), len(vals), subj_n))

# ---- negative control, and WHERE its winners are -------------------------
K = 12
wins = []
for i in range(len(starts) - K + 1):
    n, rng, _ = score(set(starts[i:i + K]))
    wins.append((n, starts[i], starts[i + K - 1], rng))
N = len(wins)
beat = [w for w in wins if w[0] >= subj_n]
print("\nNEGATIVE CONTROL — %d sliding windows of %d consecutive functions" % (N, K))
print("  %d (%.2f%%) match or beat the subject's run of %d"
      % (len(beat), 100.0 * len(beat) / N, subj_n))
print("  and here is where they are — the number that decides whether 0.82%% "
      "means anything:")
seen = []
for n, lo, hi, rng in beat:
    tu = tu_of.get(lo, ("?", "?"))[0]
    seen.append((lo, hi, n, tu, rng))
merged = []
for lo, hi, n, tu, rng in seen:
    if merged and lo <= merged[-1][1]:
        merged[-1] = (merged[-1][0], hi, max(merged[-1][2], n), merged[-1][3],
                      merged[-1][4])
    else:
        merged.append((lo, hi, n, tu, rng))
for lo, hi, n, tu, rng in merged:
    print("    %#x..%#x  best-run=%-4d tu=%-22s data %s"
          % (lo, hi, n, tu, "%#x..%#x" % rng if rng else "-"))
print("  -> %d overlapping windows collapse to %d distinct code regions"
      % (len(beat), len(merged)))

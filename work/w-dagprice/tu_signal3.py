#!/usr/bin/env python3
"""w-dagprice probe 3c — the rest of the gap, and two sanity checks on the test.

(a) The 32 functions ABOVE the band (0x10be6aeb..0x10be717f) that FUNCS.tsv
    hands to `except.c` by the nearest-anchor rule.  Do THEY own a block?
(b) Is .data block order the same as .text order across TUs?  If it is not,
    the subject's .data neighbours (sizeopt.c below, regasg.c above) say
    nothing about its position in link order, and this lane must not use them
    that way.
"""
import os
import bisect
import collections

EXP = os.path.expanduser("~/ghidra-projects/export/c2")
REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


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


funcs = {int(r["addr"], 16): int(r["size"]) for r in rows("functions.tsv")}
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
    data_from[to].add(int(ff, 16) if ff not in ("-", "") else owner(int(r["from"], 16)))

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
    return (len(best) if len(best) >= minlen else 0, (best[0], best[-1]), len(priv))


print("== (a) THE 32 FUNCTIONS ABOVE THE BAND, handed to except.c by the gap rule ==")
above = [a for a in starts if 0x10BE663E <= a <= 0x10BE717F]
n, rng, tot = score(set(above))
print("  %d functions %#x..%#x   run=%d %s private=%d"
      % (len(above), above[0], above[-1], n,
         "%#x..%#x" % rng if rng else "-", tot))
exc_anchor = [a for a in starts if 0x10BE4978 <= a <= 0x10BE56A2]
n2, rng2, _ = score(set(exc_anchor))
print("  except.c's own ICE-anchored functions: %d, run=%d %s"
      % (len(exc_anchor), n2, "%#x..%#x" % rng2 if rng2 else "-"))
n3, rng3, _ = score(set(exc_anchor) | set(above))
print("  the two together:                       run=%d %s"
      % (n3, "%#x..%#x" % rng3 if rng3 else "-"))
print("  -> if the union does not beat the parts, they are not one compiland")

print("\n== (b) IS .data ORDER == .text ORDER ACROSS TUs? ==")
tu_of = {}
for r in rows("ref/FUNCS.tsv", base=os.path.join(REPO, "docs", "whitebox")):
    tu_of[int(r["addr"], 16)] = r["tu"]
by_tu = collections.defaultdict(set)
for a, tu in tu_of.items():
    if tu not in ("-", ""):
        by_tu[tu].add(a)
pairs = []
for tu, fs in by_tu.items():
    if len(fs) < 12:
        continue
    n, rng, _ = score(fs)
    if n >= 6:
        pairs.append((min(fs), rng[0], tu))
pairs.sort()
print("  %d TUs with a run >= 6, ordered by first .text address:" % len(pairs))
for t, d, tu in pairs:
    print("    text %#x   data %#x   %s" % (t, d, tu))
# Spearman rho between .text rank and .data rank
tr = {tu: i for i, (t, d, tu) in enumerate(sorted(pairs))}
dr = {tu: i for i, (t, d, tu) in enumerate(sorted(pairs, key=lambda p: p[1]))}
nn = len(pairs)
if nn > 2:
    d2 = sum((tr[tu] - dr[tu]) ** 2 for _, _, tu in pairs)
    rho = 1 - 6.0 * d2 / (nn * (nn * nn - 1))
    print("  Spearman rho(.text rank, .data rank) = %.3f over n=%d" % (rho, nn))
    print("  -> rho near 0 means the .data NEIGHBOURS of the subject's block say")
    print("     NOTHING about which compilands it sits beside in link order.")

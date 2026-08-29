#!/usr/bin/env python3
"""w-dagprice probe 3 — the DATA-BLOCK test for translation-unit identity,
WITH ITS CONTROLS.

The test.  A compiland's file-scope statics are concatenated into .data/.bss by
the linker in OBJ order, so a real translation unit should own a CONTIGUOUS,
EXCLUSIVELY-REFERENCED run of data words, bounded above and below by other
compilands' runs.  Probe 2 found exactly that for the 12 scheduler functions
(0x10c3d140..0x10c3d184).

A test with no control is a test that cannot fail (#3336, and MEMORY's "printed
is not watched").  So this script scores THREE populations with one function:

  POSITIVE control  a TU whose identity is a FACT (in-anchor, from c2_tus.tsv).
                    Must score high or the test does not detect TUs at all.
  SUBJECT           the 12.
  NEGATIVE control  every sliding window of k consecutive functions in the
                    image.  If a large share of arbitrary windows also own an
                    exclusive contiguous data run, the signal is an ARTIFACT of
                    locality and proves nothing about TU identity.
"""
import os
import bisect
import collections
import statistics

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

# .data / .bss only — .rdata string and table constants are shared freely and
# say nothing about a compiland (measured: the 12 also "privately" touch four
# .rdata runs, which is why this window is stated and not assumed).
RW_LO, RW_HI = 0x10C2E000, 0x10C90000
rwdata = sorted(d for d in data_from if RW_LO <= d < RW_HI)


def score(fnset, gap=0x40, minlen=4):
    """largest contiguous run of RW data words referenced ONLY by fnset."""
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
            (best[0], best[-1]),
            len(priv))


# ---------------------------------------------------------------- SUBJECT
S12 = [0x10BE5CCE, 0x10BE5CEA, 0x10BE5D06, 0x10BE5D4B, 0x10BE5DB0, 0x10BE5DF6,
       0x10BE5FBB, 0x10BE6046, 0x10BE607F, 0x10BE60C0, 0x10BE626C, 0x10BE6382]
n, rng, tot = score(set(S12))
print("SUBJECT   the 12 scheduler functions: run=%d  %s  (private total %d)"
      % (n, "%#x..%#x" % rng if rng else "-", tot))

# ------------------------------------------------------- POSITIVE CONTROLS
# TUs whose extent is a FACT: an ICE-site anchor, from docs/whitebox/c2_tus.tsv.
POS = {
    "dag.c":     (0x10B3219F, 0x10B3433F),
    "color.c":   (0x10B2C21D, 0x10B3219F),
    "inline.c":  (0x10B5CFD4, 0x10B5F2B7),
    "reader.c":  (0x10BBC9AB, 0x10BC4307),
    "p2symtab.c": (0x10B97DFB, 0x10B9BD1A),
    "except.c":  (0x10BE4978, 0x10BE56A2),
    "emit.cpp":  (0x10BE71C9, 0x10BE7E81),
    "tuple.c":   (0x10BD398A, 0x10BD6999),
    "stack.c":   (0x10BD0C77, 0x10BD0F4A),
    "main.c":    (0x10B7E339, 0x10B7E4E9),
}
print("\nPOSITIVE CONTROLS — TUs with an ICE anchor (extent is a fact):")
pos_scores = []
for name, (lo, hi) in POS.items():
    fs = {a for a in starts if lo <= a <= hi}
    n2, rng2, tot2 = score(fs)
    pos_scores.append(n2)
    print("  %-12s nfun=%-4d run=%-4d %-24s private=%d"
          % (name, len(fs), n2, "%#x..%#x" % rng2 if rng2 else "-", tot2))

# ------------------------------------------------------- NEGATIVE CONTROL
# every sliding window of exactly 12 consecutive functions in the image.
print("\nNEGATIVE CONTROL — every sliding window of 12 consecutive functions:")
K = 12
wins = []
for i in range(len(starts) - K + 1):
    fs = set(starts[i:i + K])
    n3, _, _ = score(fs)
    wins.append(n3)
N = len(wins)
print("  N = %d windows" % N)
for thr in (4, 8, 10, 12, 14, 15):
    c = sum(1 for x in wins if x >= thr)
    print("    run >= %-3d : %5d windows (%.2f%%)" % (thr, c, 100.0 * c / N))
print("  median run %d, mean %.2f, max %d"
      % (statistics.median(wins), statistics.fmean(wins), max(wins)))
rank = sum(1 for x in wins if x >= n)
print("  the 12's run of %d is matched or beaten by %d of %d windows (%.2f%%)"
      % (n, rank, N, 100.0 * rank / N))

with open(os.path.join(OUT, "TU_SIGNAL.tsv"), "w") as fh:
    fh.write("population\tname\tnfun\tbest_run\trange\n")
    fh.write("subject\tthe 12\t12\t%d\t%08x..%08x\n" % (n, rng[0], rng[1]))
    for (name, (lo, hi)), s in zip(POS.items(), pos_scores):
        fh.write("positive\t%s\t%d\t%d\t-\n"
                 % (name, len({a for a in starts if lo <= a <= hi}), s))
    for thr in (4, 8, 10, 12, 14, 15):
        fh.write("negative\twindow12 run>=%d\t%d\t%d\t-\n"
                 % (thr, N, sum(1 for x in wins if x >= thr)))

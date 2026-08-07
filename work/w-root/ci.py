#!/usr/bin/env python3
"""ci.py — exact (Clopper-Pearson) 95 % intervals, in sample and out.

w-quar's central honesty move was to print BOTH intervals and say plainly that a
21-sample's was 17x wider than the fitting population's, so the correct claim was
"consistent with", not "better than".  This lane registered n = 200 to make that
ratio ~2, and prints the same comparison so the claim can be checked rather than
asserted.

stdlib only -- Clopper-Pearson by bisection on the binomial CDF, no scipy.
"""
from math import comb


def _cdf_le(k, n, p):
    return sum(comb(n, i) * p ** i * (1 - p) ** (n - i) for i in range(k + 1))


def cp(k, n, alpha=0.05):
    lo, hi = 0.0, 1.0
    if k > 0:
        a, b = 0.0, 1.0
        for _ in range(200):
            m = (a + b) / 2
            # P(X >= k | m) = 1 - cdf_le(k-1) ; want == alpha/2
            if 1 - _cdf_le(k - 1, n, m) < alpha / 2:
                a = m
            else:
                b = m
        lo = (a + b) / 2
    if k < n:
        a, b = 0.0, 1.0
        for _ in range(200):
            m = (a + b) / 2
            if _cdf_le(k, n, m) > alpha / 2:
                a = m
            else:
                b = m
        hi = (a + b) / 2
    return lo, hi


ROWS = [
    ("JFP_ALIAS  (incumbent)  IN SAMPLE", 238, 650),
    ("JFP_ALIAS  (incumbent)  HELD OUT ", 70, 200),
    ("M3A        (the rule)   IN SAMPLE", 460, 650),
    ("M3A        (the rule)   HELD OUT ", 132, 200),
    ("increment  M3A - incumb IN SAMPLE", 222, 650),
    ("increment  M3A - incumb HELD OUT ", 62, 200),
    ("TU reach   incumbent    IN SAMPLE", 96, 114),
    ("TU reach   incumbent    HELD OUT ", 26, 31),
    ("TU reach   M3A          IN SAMPLE", 103, 114),
    ("TU reach   M3A          HELD OUT ", 31, 31),
]

if __name__ == "__main__":
    print("%-36s %10s %9s   %s" % ("quantity", "k/n", "rate",
                                   "exact 95 % interval (width)"))
    for name, k, n in ROWS:
        lo, hi = cp(k, n)
        print("%-36s %5d/%-4d %9.5f   [%.4f, %.4f]  (%.4f)"
              % (name, k, n, k / n, lo, hi, hi - lo))
    print()
    a = cp(460, 650); b = cp(132, 200)
    print("M3A width ratio held-out / in-sample = %.2fx   (w-quar's was 17x)"
          % ((b[1] - b[0]) / (a[1] - a[0])))
    print("in-sample point %.5f is %s the held-out interval"
          % (460 / 650, "INSIDE" if b[0] <= 460 / 650 <= b[1] else "OUTSIDE"))
    print("held-out point  %.5f is %s the in-sample interval"
          % (132 / 200, "INSIDE" if a[0] <= 132 / 200 <= a[1] else "OUTSIDE"))

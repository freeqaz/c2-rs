#!/usr/bin/env python3
"""raise_check.py — lane w-sym. The holdout raise, DEMONSTRATED not asserted.

`docs/rungs/_2026-08-05-w-sym-prereg.md` §6 promises the fitter raises on any
path containing `holdout`. This file proves it, and it is a positive check with
a printed count: it must see the raise AND must see the fit table load.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402

ok = 0
for p in ("holdout.tsv", "/tmp/holdout.tsv", "HOLDOUT.TSV", "x_holdout_y.tsv"):
    try:
        S.read_rows(os.path.join(W, p) if not p.startswith("/") else p)
        print("FAIL: %s was OPENED — the fitter does not refuse" % p)
        sys.exit(1)
    except RuntimeError as e:
        assert "REFUSED" in str(e), e
        ok += 1
rows = S.read_rows(os.path.join(W, "fit.tsv"))
print("paths refused          : %d / 4" % ok)
print("fit.tsv rows loaded    : %d" % len(rows))
if ok != 4 or not rows:
    sys.exit("FAIL")
print("OK: the raise fires on every holdout-named path and only on those")

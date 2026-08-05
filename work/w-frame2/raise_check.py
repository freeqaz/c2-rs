#!/usr/bin/env python3
"""raise_check.py — lane w-frame2.

The fitter's refusal to open the holdout is a RAISE, not a convention.
Demonstrated on four spellings, because `w-sym` demonstrated it on four and the
point of the demonstration is that the refusal survives the spellings a hurried
lane would actually type.

Run it: every line must print REFUSED.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import f2lib as F  # noqa: E402

SPELLINGS = [
    os.path.join(W, "holdout.tsv"),
    "work/w-frame2/holdout.tsv",
    "./holdout.TSV",
    os.path.join(W, "..", "w-frame2", "HoldOut.tsv"),
]


def main():
    bad = 0
    for p in SPELLINGS:
        try:
            F.read_rows(p)
        except RuntimeError as e:
            print("REFUSED  %-44s  %s" % (p, str(e)[:60]))
            continue
        except SystemExit:
            pass
        print("!! OPENED %s — the guard does not hold" % p)
        bad += 1
    if bad:
        raise SystemExit("FAIL: %d spellings were not refused" % bad)
    print("all %d spellings refused" % len(SPELLINGS))
    return 0


if __name__ == "__main__":
    sys.exit(main())

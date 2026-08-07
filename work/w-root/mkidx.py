#!/usr/bin/env python3
"""mkidx.py — build a `<src>\\t<abs cache entry>` index for ONE side of the split.

    usage: mkidx.py <cacheidx.tsv> <laneroot> <names.txt> <out.tsv> [<forbid.txt>]

`forbid.txt` is a HARD REFUSAL: if any name in it appears in `names.txt` the
script exits non-zero and writes nothing.  The fitting scripts are always given
`heldout200.txt` as `forbid.txt`, so a fit that touches the held-out set is a
crash and not a silent contamination.

stdlib only; opens no cache entry.
"""
import os
import sys


def names(p):
    return [l.strip() for l in open(p) if l.strip()]


def main():
    idxp, laneroot, namesp, outp = sys.argv[1:5]
    forbid = set(names(sys.argv[5])) if len(sys.argv) > 5 else set()
    want = names(namesp)
    bad = sorted(set(want) & forbid)
    if bad:
        sys.stderr.write("REFUSED: %d forbidden TU(s) in %s, first %s\n"
                         % (len(bad), namesp, bad[:3]))
        raise SystemExit(2)
    ent = {}
    for ln in open(idxp):
        f = ln.rstrip("\n").split("\t")
        if len(f) >= 2:
            ent[f[0]] = os.path.join(laneroot, "work", "capture-cache",
                                     os.path.basename(f[1]))
    miss = [s for s in want if s not in ent]
    if miss:
        sys.stderr.write("REFUSED: %d TU(s) not in the corpus index\n" % len(miss))
        raise SystemExit(3)
    with open(outp, "w") as fh:
        for s in want:
            fh.write("%s\t%s\n" % (s, ent[s]))
    print("wrote %d rows to %s (forbidden set %d, overlap 0)"
          % (len(want), outp, len(forbid)))


if __name__ == "__main__":
    main()

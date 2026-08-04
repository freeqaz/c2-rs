#!/usr/bin/env python3
"""Byte-identity gate: fork-server obj vs spawn obj, COFF TimeDateStamp zeroed.

Both arms wrote to the SAME -Fo path in the same case dir, so any difference is
a real difference in what c2 emitted, not a path-string artefact.

Makes a POSITIVE claim: prints the number of pairs actually compared, and exits
non-zero if that number is 0 or if any pair differs.

usage: compare.py <corpus-dir> <suffix-a> <suffix-b>
"""
import os, sys

def norm(p):
    b = bytearray(open(p, 'rb').read())
    b[4:8] = b'\0\0\0\0'          # COFF TimeDateStamp, file offset 4..8
    return bytes(b)

def main():
    corpus, sa, sb = sys.argv[1], sys.argv[2], sys.argv[3]
    compared = missing_a = missing_b = 0
    diffs = []
    cases = sorted(d for d in os.listdir(corpus)
                   if os.path.isdir(os.path.join(corpus, d)))
    for c in cases:
        pa = os.path.join(corpus, c, sa + '.obj')
        pb = os.path.join(corpus, c, sb + '.obj')
        if not os.path.exists(pa):
            missing_a += 1
            continue
        if not os.path.exists(pb):
            missing_b += 1
            continue
        A, B = norm(pa), norm(pb)
        compared += 1
        if A != B:
            off = next((i for i in range(min(len(A), len(B))) if A[i] != B[i]),
                       min(len(A), len(B)))
            diffs.append((c, len(A), len(B), off))

    print("cases in corpus        : %d" % len(cases))
    print("pairs COMPARED         : %d   (%s missing %d, %s missing %d)"
          % (compared, sa, missing_a, sb, missing_b))
    print("byte-identical         : %d" % (compared - len(diffs)))
    print("DIFFERING              : %d" % len(diffs))
    for c, la, lb, off in diffs[:20]:
        print("  %s  len %d vs %d  first diff @ %d" % (c, la, lb, off))
    if compared == 0:
        print("COMPARED NOTHING — this is a FAILURE, not a pass")
        sys.exit(1)
    if diffs:
        sys.exit(1)
    print("VERDICT: %d/%d objs byte-identical with TimeDateStamp zeroed" % (compared, compared))

main()

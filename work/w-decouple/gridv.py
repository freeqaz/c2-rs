#!/usr/bin/env python3
"""GRID-V — read the `.gl` defined-record FLAGS byte (`name_nul + 5`) for every
cell in a generated family, so the variadic bit is a measurement and not an
inference from two objs.

    work/w-decouple/gridv.py <il-dir> [<il-dir> ...]

For each `.gl` in each directory, prints every framed defined record's name and
the five bytes GRID-K named: `<tag> <kind>` (2), linkage, retsize, flags.
"""
import glob
import os
import sys

MAX_NAME_TO_OFFSET = 32
SEP26 = 0x26


def symbol_runs(gl):
    def is_sep(b):
        return b == 0 or b == SEP26
    out = []
    i = 0
    n = len(gl)
    while i < n:
        if not is_sep(gl[i]):
            i += 1
            continue
        start = i + 1
        end = start
        while end < n and not is_sep(gl[end]):
            end += 1
        if end >= n or end == start:
            i += 1
            continue
        b = gl[start:end]
        if all(0x21 <= c <= 0x7e for c in b) and (
                b[0:1] == b"?" or chr(b[0]).isalpha() or b[0:1] == b"_"):
            out.append((start, end, b.decode("ascii")))
        i = end
    return out


def framed(gl, o):
    return (o >= 7 and gl[o] == 0x80 and gl[o - 7] == 0x80 and gl[o - 5] == 0x10
            and gl[o - 4] == 0 and gl[o - 3] == 0 and gl[o - 2] == 0
            and gl[o - 1] == 0)


def main():
    print("%-18s %-26s %-6s %s" % ("cell", "record name", "len", "tag kind link size FLAGS"))
    for d in sys.argv[1:]:
        for p in sorted(glob.glob(os.path.join(d, "*.gl"))):
            gl = open(p, "rb").read()
            runs = symbol_runs(gl)
            o = 0
            while o + 5 <= len(gl):
                if not framed(gl, o):
                    o += 1
                    continue
                cands = [k for k, (_, e, _) in enumerate(runs) if e <= o]
                if cands and o - runs[cands[-1]][1] <= MAX_NAME_TO_OFFSET:
                    k = cands[-1]
                    nul = runs[k][1]
                    b = gl[nul + 1:nul + 6]
                    print("%-18s %-26s %-6d %s" % (
                        os.path.basename(d), runs[k][2], len(runs[k][2]),
                        " ".join("%02x" % x for x in b)))
                o += 5


main()

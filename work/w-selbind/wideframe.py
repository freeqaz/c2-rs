#!/usr/bin/env python3
"""`gl_defined_names_framed` with the FRAME relaxed, as a classifier.

`codec::gl_offset_framed` pins `gl[o-5] == 0x10`, which constrains the record's
PREV field to `[0x1000, 0x10FF]`. Board **#2783** (w-phase7b §10.2) found PREV is
a rising per-record number, so the shipping frame sees a TU's records only while
that number stays below 0x1100. This transcribes the same walk with the frame
relaxed to `PREV < 0x10000`, i.e. `gl[o-4..o-1] == 00 00 00 00` and `gl[o-7] ==
0x80` kept, `gl[o-5]` free.

The point is NOT the record count. It is whether the relaxation is a SOUND
reader — measured here as a hard control:

    every framed record's body-start offset must BE an `.ex` `4F 1F` split point.

A record whose offset is not a split point is a false positive: the frame matched
something that is not a body-start field, and the binding it would produce is a
name attached to the wrong body.

    work/w-selbind/wideframe.py <bundle.gl> <bundle.ex> [--list] [--names N...]
"""
import sys
from collections import Counter

INLINE_NAME_MAX = 8
MAX_NAME_TO_OFFSET = 32
SEP26 = 0x26
LINKAGE_EXPORT_BIT = 0x08


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
        plausible = all(0x21 <= c <= 0x7e for c in b) and (
            b[0:1] == b"?" or chr(b[0]).isalpha() or b[0:1] == b"_"
        )
        if plausible:
            out.append((start, end, b.decode("ascii")))
        i = end
    return out


def narrow_framed(gl, o):
    return (o >= 7 and gl[o] == 0x80 and gl[o - 7] == 0x80 and gl[o - 5] == 0x10
            and gl[o - 4] == 0 and gl[o - 3] == 0 and gl[o - 2] == 0
            and gl[o - 1] == 0)


def wide_framed(gl, o):
    return (o >= 7 and gl[o] == 0x80 and gl[o - 7] == 0x80
            and gl[o - 4] == 0 and gl[o - 3] == 0 and gl[o - 2] == 0
            and gl[o - 1] == 0)


def ex_starts(ex):
    out = []
    i = 0
    while i + 1 < len(ex):
        if ex[i] == 0x4F and ex[i + 1] == 0x1F:
            out.append(i)
            i += 2
        else:
            i += 1
    return out


def classify(gl, framed):
    runs = symbol_runs(gl)
    rows = []
    p = 0
    while p + 5 <= len(gl):
        if not framed(gl, p):
            p += 1
            continue
        off = int.from_bytes(gl[p + 1:p + 5], "little")
        cands = [k for k, (_, end, _) in enumerate(runs) if end <= p]
        k = cands[-1] if cands else None
        if k is None or p - runs[k][1] > MAX_NAME_TO_OFFSET:
            rows.append((p, off, None, "name-too-far"))
            p += 5
            continue
        name = runs[k][2]
        if gl[runs[k][1]] != 0:
            v = "run-ends-26"
        elif runs[k][1] + 3 < len(gl) and (gl[runs[k][1] + 3] & LINKAGE_EXPORT_BIT):
            v = "dllexport"
        elif runs[k][0] > 0 and gl[runs[k][0] - 1] == SEP26:
            v = "26-introduced"
        else:
            v = "bound"
        rows.append((p, off, name, v))
        p += 5
    return runs, rows


def report(tag, gl, ex, framed, want, listing):
    runs, rows = classify(gl, framed)
    starts = set(ex_starts(ex))
    c = Counter(v for _, _, _, v in rows)
    offs = [o for _, o, _, _ in rows]
    fp = [(p, o, n, v) for p, o, n, v in rows if o not in starts]
    print("-- %s: %d framed records over %d .ex segments" % (tag, len(rows), len(starts)))
    for v, n in sorted(c.items()):
        print("     %-16s %d" % (v, n))
    print("     CONTROL offsets that are NOT an .ex split point (false positives): %d"
          % len(fp))
    for row in fp[:10]:
        print("        @%-7d off %-8d %-14s %s" % row)
    print("     distinct segments carrying a record: %d of %d"
          % (len(set(offs) & starts), len(starts)))
    dup = len(offs) - len(set(offs))
    print("     records sharing an offset with another record: %d" % dup)
    by_name = {}
    for p, o, n, v in rows:
        by_name.setdefault(n, []).append((p, o, v))
    for n in want:
        print("     %-26s -> %s" % (n[:26], by_name.get(n, "NO RECORD")))
    if listing:
        for p, o, n, v in rows:
            print("        @%-7d off %-8d %-14s %s" % (p, o, v, n))
    return rows


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read()
    want = []
    if "--names" in sys.argv:
        want = sys.argv[sys.argv.index("--names") + 1:]
    listing = "--list" in sys.argv
    report("NARROW (shipping `codec::gl_offset_framed`)", gl, ex, narrow_framed, want, False)
    report("WIDE   (PREV < 0x10000)", gl, ex, wide_framed, want, listing)


main()

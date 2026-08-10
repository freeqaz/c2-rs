#!/usr/bin/env python3
"""`gl_defined_names_framed` transcribed, and then run PAST its stops as a
CLASSIFIER.

`work/w-decouple/glwalk2.py` answers *"where does the shipping walk stop"*.
This one answers the next question a factor-A lane needs: *"if the walk did not
stop, how many framed defined records are there, and how do they split by the
clause that would have stopped the walk"* — because the `26`-introduced clause
is not a reader limitation, it is the port declining a per-function COMDAT, and
the emit-set question is exactly about that population.

    work/w-phase7b/glwalk3.py <bundle.gl> [bundle.ex] [--list]

Every stop clause is evaluated per record and recorded; the walk then advances
`p += 5` regardless, which is what the shipping walk does on a bound record.
That is a COUNTERFACTUAL framing, not the shipping one: a record the shipping
walk never reaches is reported here with a verdict, and the shipping walk's own
answer is the prefix up to the first non-`bound` row.
"""
import sys
from collections import Counter

INLINE_NAME_MAX = 8
MAX_NAME_TO_OFFSET = 32
SEP26 = 0x26
LINKAGE_DEFINED_EXPORT = 0x09


def symbol_runs(gl, sep26=True):
    def is_sep(b):
        return b == 0 or (sep26 and b == SEP26)
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


def gl_offset_framed(gl, o):
    return (
        o >= 7
        and gl[o] == 0x80
        and gl[o - 7] == 0x80
        and gl[o - 5] == 0x10
        and gl[o - 4] == 0x00
        and gl[o - 3] == 0x00
        and gl[o - 2] == 0x00
        and gl[o - 1] == 0x00
    )


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


def classify(gl):
    runs = symbol_runs(gl)
    rows = []
    p = 0
    while p + 5 <= len(gl):
        if not gl_offset_framed(gl, p):
            p += 1
            continue
        off = int.from_bytes(gl[p + 1:p + 5], "little")
        cands = [k for k, (_, end, _) in enumerate(runs) if end <= p]
        if not cands:
            rows.append((p, off, None, "name-too-far"))
            p += 5
            continue
        k = cands[-1]
        name = runs[k][2]
        if p - runs[k][1] > MAX_NAME_TO_OFFSET:
            v = "name-too-far"
        elif gl[runs[k][1]] != 0:
            v = "run-ends-26"
        elif (runs[k][1] + 3 < len(gl)
              and gl[runs[k][1] + 3] == LINKAGE_DEFINED_EXPORT):
            v = "dllexport"
        elif runs[k][0] > 0 and gl[runs[k][0] - 1] == SEP26:
            v = "26-introduced"
        elif len(name) <= INLINE_NAME_MAX:
            v = "name-inline-fit"
        else:
            v = "bound"
        rows.append((p, off, name, v))
        p += 5
    return runs, rows


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = None
    for a in sys.argv[2:]:
        if not a.startswith("--"):
            ex = open(a, "rb").read()
    runs, rows = classify(gl)
    print("%s: %d B .gl, %d printable runs, %d framed defined records"
          % (sys.argv[1], len(gl), len(runs), len(rows)))
    c = Counter(v for _, _, _, v in rows)
    for v, n in c.most_common():
        print("   %-16s %d" % (v, n))
    # the SHIPPING walk = the prefix of rows that are all `bound`
    pre = 0
    for _, _, _, v in rows:
        if v != "bound":
            break
        pre += 1
    print("   shipping walk binds the first %d record(s); stops at row %d (%s)"
          % (pre, pre, rows[pre][3] if pre < len(rows) else "end"))
    if ex is not None:
        st = ex_starts(ex)
        print("   .ex 4F 1F segments: %d" % len(st))
        offs = set(o for _, o, _, _ in rows)
        print("   record body-starts that ARE .ex split points: %d of %d"
              % (len(offs & set(st)), len(rows)))
    if "--list" in sys.argv:
        for p, off, name, v in rows:
            print("      @%-6d start %-7d %-14s %s" % (p, off, v, name))


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""`gl_defined_names_framed` transcribed WHOLE — all five stop clauses — and
runnable with the `INLINE_NAME_MAX` clause ON (the shipping walk) or OFF (the
widened one this lane is about).

`work/w-front5/glwalk.py` transcribes the first two clauses only, which is
enough to name the record a shipping walk stops on and NOT enough to say what a
widened walk does next — the widened walk has three more clauses to fall
through, and the unclaimed-run accounting behind it is a different question
again.

    work/w-decouple/glwalk2.py <bundle.gl> <bundle.ex> [--wide]

Prints every framed record with the name attached and the verdict, then the
binding answer against `.ex`, then the MANGLED unclaimed runs — which is what
`IlBundle::functions`' unclaimed-`.gl`-symbol gate has to account for and which
is invisible on a TU that does not bind.
"""
import sys

INLINE_NAME_MAX = 8
MAX_NAME_TO_OFFSET = 32
SEP26 = 0x26
LINKAGE_DEFINED_EXPORT = 0x09
RETSIZE_ESCAPE = 0x80


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


def linkage_needs_a_directive(gl, name_nul):
    """`gl::linkage_needs_a_directive` — `__declspec(dllexport)`."""
    return (
        name_nul + 3 < len(gl)
        and gl[name_nul + 3] == LINKAGE_DEFINED_EXPORT
    )


def looks_mangled(n):
    return "@@" in n


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


def walk(gl, wide):
    runs = symbol_runs(gl)
    claimed = [False] * len(runs)
    bound = []
    rows = []
    stop = None
    p = 0
    while p + 5 <= len(gl):
        if not gl_offset_framed(gl, p):
            p += 1
            continue
        off = int.from_bytes(gl[p + 1:p + 5], "little")
        cands = [k for k, (_, end, _) in enumerate(runs) if end <= p]
        if not cands:
            stop = (p, None, "gl-stop-name-too-far (no preceding run)")
            rows.append((p, off, None, stop[2]))
            break
        k = cands[-1]
        name = runs[k][2]
        if p - runs[k][1] > MAX_NAME_TO_OFFSET:
            stop = (p, name, "gl-stop-name-too-far (%d B)" % (p - runs[k][1]))
            rows.append((p, off, name, stop[2]))
            break
        if not wide and len(name) <= INLINE_NAME_MAX:
            stop = (p, name, "gl-stop-name-not-mangled (len %d <= %d)"
                    % (len(name), INLINE_NAME_MAX))
            rows.append((p, off, name, stop[2]))
            break
        if gl[runs[k][1]] != 0:
            stop = (p, name, "gl-stop-run-ends-26")
            rows.append((p, off, name, stop[2]))
            break
        if linkage_needs_a_directive(gl, runs[k][1]):
            stop = (p, name, "gl-stop-dllexport")
            rows.append((p, off, name, stop[2]))
            break
        if runs[k][0] > 0 and gl[runs[k][0] - 1] == SEP26:
            stop = (p, name, "gl-stop-26-introduced")
            rows.append((p, off, name, stop[2]))
            break
        claimed[k] = True
        bound.append((off, name))
        rows.append((p, off, name, "bound"))
        p += 5
    unclaimed = [n for (_, _, n), c in zip(runs, claimed)
                 if not c and looks_mangled(n)]
    if stop is not None:
        # A stopped walk yields the EMPTY pair; the unclaimed list it would
        # have produced is not what the caller sees.
        return rows, [], [], stop, runs
    return rows, bound, unclaimed, stop, runs


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read() if len(sys.argv) > 2 and not sys.argv[2].startswith("--") else None
    wide = "--wide" in sys.argv
    rows, bound, unclaimed, stop, runs = walk(gl, wide)
    print("walk: %s   %d printable runs, %d bytes of .gl"
          % ("WIDE (no INLINE_NAME_MAX clause)" if wide else "SHIPPING", len(runs), len(gl)))
    for p, off, name, verdict in rows:
        print("  record @%-5d body-start %-6d name %-40r %s" % (p, off, name, verdict))
    print("bound %d record(s); stop = %s" % (len(bound), stop))
    print("unclaimed MANGLED runs (%d): %s" % (len(unclaimed), unclaimed))
    print("all runs: %s" % [n for _, _, n in runs])
    if ex is not None:
        st = ex_starts(ex)
        print(".ex 4F 1F starts: %d %s" % (len(st), st[:24]))
        ok = len(bound) == len(st) and all(o == s for (o, _), s in zip(bound, st))
        print("per_record binds: %s" % ok)


main()

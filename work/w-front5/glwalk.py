#!/usr/bin/env python3
"""Re-walk `.gl`'s defined-record binding, the way the GATE does, and name the
record the walk stops on.

This is a transcription of `c2_il::func::gl::gl_defined_names_framed(gl, true,
codec::gl_offset_framed)` — the exact predicate `Bindings::per_record` runs, and
therefore the predicate `IlBundle::functions()` runs before any body is parsed.
It exists because the scan publishes the stop CLAUSE (`gl-stop-name-not-mangled`)
and not the RECORD, and `CEILING.md` §11.4 item 8 requires a conversion price to
be quoted against this binding rather than against a per-function instrument.

    work/w-front5/glwalk.py <bundle.gl> [bundle.ex]

Prints every framed record in `.gl` order with the name the walk would attach,
the verdict for that record, and the first stop.
"""
import sys

INLINE_NAME_MAX = 8
MAX_NAME_TO_OFFSET = 32
SEP = (0x00, 0x26)


def symbol_runs(gl, sep26=True):
    """`gl::symbol_runs` — separator-delimited printable runs."""
    def is_sep(b):
        return b == 0 or (sep26 and b == 0x26)
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
    """`codec::gl_offset_framed` — the GATE's record framing."""
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
    """`bundle::split_functions_at` head — the `4F 1F` function starts."""
    out = []
    i = 0
    while i + 1 < len(ex):
        if ex[i] == 0x4F and ex[i + 1] == 0x1F:
            out.append(i)
            i += 2
        else:
            i += 1
    return out


def main():
    gl = open(sys.argv[1], "rb").read()
    runs = symbol_runs(gl)
    print("%d printable runs, %d bytes of .gl" % (len(runs), len(gl)))
    stop = None
    bound = []
    p = 0
    while p + 5 <= len(gl):
        if not gl_offset_framed(gl, p):
            p += 1
            continue
        off = int.from_bytes(gl[p + 1:p + 5], "little")
        cands = [k for k, (_, end, _) in enumerate(runs) if end <= p]
        if not cands:
            verdict = "STOP gl-stop-name-too-far (no preceding run)"
            name = None
        else:
            k = cands[-1]
            if p - runs[k][1] > MAX_NAME_TO_OFFSET:
                verdict = "STOP gl-stop-name-too-far (%d B)" % (p - runs[k][1])
                name = runs[k][2]
            else:
                name = runs[k][2]
                if len(name) <= INLINE_NAME_MAX:
                    verdict = "STOP gl-stop-name-not-mangled (len %d <= %d)" % (
                        len(name), INLINE_NAME_MAX)
                else:
                    verdict = "bound"
        print("  record @%-5d body-start %-6d name %-40r %s"
              % (p, off, name, verdict))
        if verdict.startswith("STOP"):
            stop = (p, name, verdict)
            break
        bound.append((off, name))
        p += 5
    print("bound %d record(s); stop = %s" % (len(bound), stop))
    if len(sys.argv) > 2:
        ex = open(sys.argv[2], "rb").read()
        st = ex_starts(ex)
        print(".ex 4F 1F starts: %d %s" % (len(st), st[:20]))
        print("per_record binds: %s"
              % (len(bound) == len(st)
                 and all(o == s for (o, _), s in zip(bound, st))))


main()

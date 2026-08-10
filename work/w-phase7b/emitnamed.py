#!/usr/bin/env python3
"""Is the EMIT SET nameable from `.gl`, even though the whole segment list is
not?

`Bindings::per_record` refuses `vec.cpp` because `.gl` spells a body-start for
373 of its 811 `.ex` segments. That is a statement about the 438 bodies c2
DISCARDS. The question that decides whether the repair is a *selective* binding
or an impossibility is the other one: are the segments c2 actually EMITS among
the 373?

    work/w-phase7b/emitnamed.py <bundle.gl> <bundle.ex> <name> [<name> ...]

For each mangled name, finds its `.gl` run, reads the framed body-start offset
that follows it, and reports whether that offset is an `.ex` split point.
"""
import re
import sys


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


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = open(sys.argv[2], "rb").read()
    st = set(ex_starts(ex))
    spelled = set()
    for i in range(len(gl) - 4):
        if gl[i] == 0x80:
            spelled.add(int.from_bytes(gl[i + 1:i + 5], "little"))
    for name in sys.argv[3:]:
        b = name.encode()
        hits = [m.start() for m in re.finditer(re.escape(b), gl)]
        if not hits:
            print("  %-26s NOT IN .gl AT ALL" % name)
            continue
        for h in hits:
            tail = gl[h + len(b):h + len(b) + 48]
            # the first `80 <LE32>` after the name whose value is an `.ex` start
            found = None
            for k in range(len(tail) - 4):
                if tail[k] == 0x80:
                    v = int.from_bytes(tail[k + 1:k + 5], "little")
                    if v in st:
                        found = v
                        break
            print("  %-26s @%-7d body-start %s   is .ex split point: %s   spelled in .gl: %s"
                  % (name, h, found, found in st if found else False,
                     (found in spelled) if found else False))


main()

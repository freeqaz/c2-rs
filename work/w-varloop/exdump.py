#!/usr/bin/env python3
"""exdump.py — print the body region of every `.ex` function segment.

Lane **w-varloop**. A reading aid for writing the recognizer: it locates the
`4C 4F 11` body marker and prints the bytes after it, so the op stream can be
read against `crates/c2-il/src/func/body/`'s opcode vocabulary.

It decodes nothing and claims nothing — every classification in this lane is
made by the Rust recognizer and graded by real `c2`.

Usage:
    work/w-varloop/exdump.py <file.ex> [--from N] [--bytes N]
"""

import sys

LO = bytes([0x4C, 0x4F, 0x11])


def main():
    argv = sys.argv[1:]
    nbytes = 400
    if "--bytes" in argv:
        i = argv.index("--bytes")
        nbytes = int(argv[i + 1])
        del argv[i:i + 2]
    d = open(argv[0], "rb").read()
    at = 0
    n = 0
    while True:
        j = d.find(LO, at)
        if j < 0:
            break
        n += 1
        print("=== body marker %d at .ex offset 0x%x" % (n, j))
        chunk = d[j:j + nbytes]
        for i in range(0, len(chunk), 16):
            row = chunk[i:i + 16]
            print("  %+4d  %s" % (i, " ".join("%02x" % b for b in row)))
        at = j + 3
    if n == 0:
        print("no `4C 4F 11` body marker in %s" % argv[0])
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""exdump.py — print a cell's `.ex` body stream, statement by statement.

Board #839 is a claim about three bytes (`26 11 0a`) and where their token turns
up afterwards, so a lane that reads only census keys is reading a summary of the
thing it is supposed to be measuring. This splits the segment on the `4B`
statement terminator and prints each statement's bytes, so the bind and the
stores can be read directly.

Nothing here is a parser: it is a viewer, deliberately, so that a disagreement
between it and `crates/c2-il` is visible rather than shared.

Usage:  exdump.py <cell> [<cell> ...]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))

# `4F 01 <line>` is the source-line marker; `4B` ends a statement; `53`/`54`
# open and close a lexical scope. Enough to break the stream into readable
# pieces without claiming to decode it.
STMT_END = 0x4B


def ex_of(cell):
    d = os.path.join(HERE, "il", cell)
    for f in sorted(os.listdir(d)):
        if f.endswith(".ex"):
            return os.path.join(d, f)
    raise SystemExit("no .ex under " + d)


def main(argv):
    for cell in argv:
        path = ex_of(cell)
        data = open(path, "rb").read()
        print("== %s   %s   %d bytes" % (cell, os.path.basename(path), len(data)))
        start = 0
        for i, b in enumerate(data):
            if b != STMT_END:
                continue
            chunk = data[start:i + 1]
            # The FIRST chunk of a `.ex` is the whole pre-body region and runs to
            # ~2.7 kB. Truncating it from the front is how this file first hid a
            # segment prologue that mattered: the statement's own bytes are at the
            # END of a long chunk, never at the start. Print both ends.
            if len(chunk) > 260:
                shown = (chunk[:60].hex(" ") + "  …%d…  " % (len(chunk) - 260)
                         + chunk[-200:].hex(" "))
            else:
                shown = chunk.hex(" ")
            print("   [%04x..%04x] %s" % (start, i, shown))
            start = i + 1
        if start < len(data):
            tail = data[start:]
            print("   [%04x..%04x] %s  <tail>"
                  % (start, len(data) - 1, tail[:200].hex(" ")))
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

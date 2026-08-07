#!/usr/bin/env python3
"""segconst.py — emit a cell's WHOLE `.ex` function segment as a Rust byte array.

A test that pins a hand-assembled body pins the test author's reading of the
grammar; a test that pins the captured segment pins `c1xx`'s. Every `const` in
`leaf_store.rs`'s test module is the second kind and this is what makes them.

Usage:  segconst.py <cell> [<const-name>]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
HEAD = bytes([0x4F, 0x1F, 0x80, 0x05])


def main(argv):
    cell = argv[0]
    name = argv[1] if len(argv) > 1 else cell.upper()
    d = os.path.join(HERE, "il", cell)
    ex = [f for f in sorted(os.listdir(d)) if f.endswith(".ex")]
    data = open(os.path.join(d, ex[0]), "rb").read()
    start = data.rfind(HEAD)
    if start < 0:
        raise SystemExit("no segment head in " + cell)
    seg = data[start:]
    print("    const %s: &[u8] = &[" % name)
    for i in range(0, len(seg), 15):
        row = ", ".join("0x%02X" % b for b in seg[i:i + 15])
        print("        %s," % row)
    print("    ];")
    print("    // %d bytes, from %s" % (len(seg), ex[0]), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

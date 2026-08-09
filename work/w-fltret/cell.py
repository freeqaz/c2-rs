#!/usr/bin/env python3
"""w-fltret — cut a `4F 1F`-anchored function segment out of a captured `.ex`
and print it as a Rust byte array, the shape `mcall_tail.rs`'s test cells use.

Usage:  cell.py IL_DIR [INDEX]      # INDEX defaults to "all"

The split anchor is `4F 1F`, which is the one `PortC2::build` consumes
(ROADMAP §10.11/§10.12) and the one the existing cells are cut on — checked
against them rather than assumed: every committed cell starts `4F 1F 80 05`.
A cell also carries the module trailer of whichever function is last, so the
cut runs to the next `4F 1F` or to the end with the zero fill stripped.
"""
import glob
import sys

d = open(glob.glob(sys.argv[1] + "/*.ex")[0], "rb").read()
d = d.rstrip(b"\x00")
starts = []
i = 0
while True:
    i = d.find(b"\x4f\x1f", i)
    if i < 0:
        break
    starts.append(i)
    i += 2
want = sys.argv[2] if len(sys.argv) > 2 else "all"
ends = starts[1:] + [len(d)]
for n, (s, e) in enumerate(zip(starts, ends)):
    if want != "all" and int(want) != n:
        continue
    b = d[s:e]
    print("    // segment %d, %d bytes" % (n, len(b)))
    for j in range(0, len(b), 15):
        print("        " + " ".join("0x%02X," % x for x in b[j:j + 15]))
    print()

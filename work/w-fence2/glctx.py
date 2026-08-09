#!/usr/bin/env python3
"""w-fence2 — dump the `.gl` bytes around a named run, both sides."""
import sys

gl = open(sys.argv[1], "rb").read()
before = int(sys.argv[3]) if len(sys.argv) > 3 else 24
after = int(sys.argv[4]) if len(sys.argv) > 4 else 24
needle = sys.argv[2].encode()
at = 0
while True:
    i = gl.find(needle, at)
    if i < 0:
        break
    at = i + 1
    lo = max(0, i - before)
    hi = min(len(gl), i + len(needle) + after)
    print(f"@{i:#06x} {needle.decode()}")
    print("   pre :", gl[lo:i].hex(" "))
    print("   post:", gl[i + len(needle) : hi].hex(" "))

#!/usr/bin/env python3
"""w-fence2 — print every `.gl` symbol run with the three bytes that follow its
NUL, i.e. the `<tag> <kind> <linkage>` triple `gl.rs::linkage_needs_a_directive`
reads at a fixed `name_nul + 3`, plus whether the run is `26`-introduced and
whether a framed body-start offset (`80 <LE32>`) claims it within 32 bytes.
"""
import re
import sys

gl = open(sys.argv[1], "rb").read()

# framed offsets: `80 <LE32>` where the LE32 is a plausible .ex body start.
frames = []
p = 0
while p + 5 <= len(gl):
    if gl[p] == 0x80:
        frames.append(p)
    p += 1

print(f"{'name':38} {'sep':>4} {'after-NUL':14} {'linkage':>8}")
for m in re.finditer(rb"[\x20-\x7e]{2,}", gl):
    name = m.group().decode()
    if name.startswith("z:\\") or name.startswith("/include"):
        continue
    end = m.end()
    if gl[end : end + 1] != b"\x00":
        continue
    sep = gl[m.start() - 1] if m.start() else None
    tail = gl[end + 1 : end + 5]
    link = gl[end + 3] if end + 3 < len(gl) else None
    print(
        f"{name:38} {('%02x' % sep) if sep is not None else '--':>4} "
        f"{tail.hex(' '):14} {('%02x' % link) if link is not None else '--':>8}"
    )

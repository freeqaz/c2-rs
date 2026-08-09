#!/usr/bin/env python3
"""Split a captured .ex into 4F 1F segments and hexdump each, 16 bytes a line.

Measurement only — no decoding claim is made here beyond the split, which is
`c2_il::IlBundle::ex_segment_count`'s own `4F 1F` rule.
"""
import sys

data = open(sys.argv[1], 'rb').read()
want = int(sys.argv[2]) if len(sys.argv) > 2 else None

starts = [i for i in range(len(data) - 1) if data[i] == 0x4F and data[i + 1] == 0x1F]
segs = []
for k, s in enumerate(starts):
    e = starts[k + 1] if k + 1 < len(starts) else len(data)
    segs.append((s, data[s:e]))

print(f"{len(segs)} segments, {len(data)} B total")
for k, (off, seg) in enumerate(segs):
    if want is not None and k != want:
        continue
    print(f"--- segment {k}  file-off 0x{off:04x}  len {len(seg)}")
    for i in range(0, len(seg), 16):
        chunk = seg[i:i + 16]
        hexs = ' '.join(f'{b:02x}' for b in chunk)
        asc = ''.join(chr(b) if 32 <= b < 127 else '.' for b in chunk)
        print(f"  {i:04x}  {hexs:<47}  {asc}")

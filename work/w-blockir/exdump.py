#!/usr/bin/env python3
"""exdump.py — split a `.ex` IL bundle into function segments on the `4C 4F 11`
marker and hexdump each. Read-only measurement tooling (lane w-blockir); it is
outside the std-only Rust workspace, same status as scripts/gt_dump.py."""
import sys

data = open(sys.argv[1], 'rb').read()
MARK = bytes([0x4C, 0x4F, 0x11])
# find every marker
idx = []
i = 0
while True:
    j = data.find(MARK, i)
    if j < 0:
        break
    idx.append(j)
    i = j + 1
print(f"# {len(data)} bytes, {len(idx)} `4C 4F 11` markers at {idx}")
for n, j in enumerate(idx):
    end = idx[n + 1] if n + 1 < len(idx) else len(data)
    seg = data[j:end]
    print(f"\n== segment {n} @0x{j:x} len {len(seg)}")
    for off in range(0, len(seg), 16):
        row = seg[off:off + 16]
        print(f"  {off:04x}  " + " ".join(f"{b:02x}" for b in row))

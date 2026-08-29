#!/usr/bin/env python3
"""jumptable.py -- decode the site collector's EH-opcode dispatch. std only.

`work/w-inlclause/IMAGE_READ.md` §3.1. `FUN_10b600e6` dispatches EH-region
opcodes through a byte index table at 0x10b60522 and a dword jump table at
0x10b6050e, over the dense range 0x2ee..0x300 (`cmp ecx,0x12` at 0x10b603f5).
Both tables are inside the image's RAW .text, so unlike P_INLINE SS5's POGO
parameter tables they are quotable.

This exists because `WB_INLINE_FINDINGS` SS1 publishes SEVEN opcodes and the
table has EIGHT with a non-default arm: 0x2fe shares arm 0 with 0x2ee. A list
transcribed from arms is exactly the thing a table decode can check.

Usage: jumptable.py [PATH-TO-c2.dll]
"""
import struct, sys

PATH = sys.argv[1] if len(sys.argv) > 1 else 'compilers/X360/16.00.11886.00/c2.dll'
IDX, JT, LO, N = 0x10b60522, 0x10b6050e, 0x2ee, 19

d = open(PATH, 'rb').read()
pe = struct.unpack_from('<I', d, 0x3c)[0]
assert d[pe:pe + 4] == b'PE\0\0', 'not a PE'
nsec = struct.unpack_from('<H', d, pe + 6)[0]
optsz = struct.unpack_from('<H', d, pe + 20)[0]
base = struct.unpack_from('<I', d, pe + 24 + 28)[0]
secs = []
off = pe + 24 + optsz
for i in range(nsec):
    e = d[off + 40 * i: off + 40 * (i + 1)]
    vsz, va, rsz, ro = struct.unpack_from('<IIII', e, 8)
    secs.append((va, vsz, ro, rsz))


def fo(va):
    """File offset of a VA, or None when the VA is above the raw section data."""
    r = va - base
    for sva, vsz, ro, rsz in secs:
        if sva <= r < sva + max(vsz, rsz):
            return ro + (r - sva) if r - sva < rsz else None
    return None


i, j = fo(IDX), fo(JT)
assert i is not None and j is not None, 'a table is above the raw data (BSS)'
idx = list(d[i:i + N])
tgt = [struct.unpack_from('<I', d, j + 4 * k)[0] for k in range(max(idx) + 1)]
default = max(range(len(tgt)), key=lambda k: idx.count(k))

print(f"index bytes at 0x{IDX:08x} (opcodes 0x{LO:03x}..0x{LO + N - 1:03x}): {idx}")
print(f"jump table at 0x{JT:08x}: " + ' '.join(f"arm{k}->0x{t:08x}" for k, t in enumerate(tgt)))
print(f"default arm (most frequent) = arm{default} @ 0x{tgt[default]:08x}")
arms = {}
for k, b in enumerate(idx):
    arms.setdefault(b, []).append(LO + k)
for b in sorted(arms):
    ops = ' '.join(f"0x{o:03x}" for o in arms[b])
    tag = '  <- default, no-op' if b == default else ''
    print(f"  arm{b} @ 0x{tgt[b]:08x}: {ops}{tag}")
live = sorted(o for b, v in arms.items() if b != default for o in v)
print(f"\nOPCODES WITH A NON-DEFAULT ARM: {len(live)} -- " + ' '.join(f"0x{o:03x}" for o in live))
print("WB_INLINE_FINDINGS SS1 publishes 7: 0x2ee 0x2f0 0x2f1 0x2f4 0x2f6 0x2ff 0x300")
missing = [o for o in live if o not in (0x2ee, 0x2f0, 0x2f1, 0x2f4, 0x2f6, 0x2ff, 0x300)]
print("MISSING FROM THAT LIST: " + (' '.join(f"0x{o:03x}" for o in missing) if missing else 'none'))

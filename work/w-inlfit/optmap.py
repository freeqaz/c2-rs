#!/usr/bin/env python3
"""optmap.py -- recover c2's option-descriptor table from the code that FILLS it.

The table at 0x10c46xxx is BSS (zero at load) and written at run time, so it
cannot be read out of the image (P_INLINE.md SS5's distinction). What CAN be read
is the straight-line run of stores that builds it, at 0x10c29xxx in .text.

Record stride 12: [+0 name_ptr] [+4 value_ptr] [+8 kind WORD].
The stride and field order are FORCED by the two boolean records whose names
resolve unambiguously ("-EHs" / "-EHa", kind 0x101 / 0x501) -- not assumed.

Names are UTF-16LE. std only.
"""
import os, re, struct, sys

DLL = 'compilers/X360/16.00.11886.00/c2.dll'
ASM = os.environ.get('C2RS_OBJDUMP_ASM',
                     os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))
d = open(DLL, 'rb').read()
pe = struct.unpack_from('<I', d, 0x3c)[0]
nsec = struct.unpack_from('<H', d, pe + 6)[0]
optsz = struct.unpack_from('<H', d, pe + 20)[0]
base = struct.unpack_from('<I', d, pe + 24 + 28)[0]
off = pe + 24 + optsz
secs = []
for i in range(nsec):
    e = d[off + 40*i: off + 40*(i+1)]
    vsz, va, rsz, ro = struct.unpack_from('<IIII', e, 8)
    secs.append((va, vsz, ro, rsz))

def fo(va):
    r = va - base
    for sva, vsz, ro, rsz in secs:
        if sva <= r < sva + rsz:
            return ro + (r - sva)
    return None

def wstr(va):
    o = fo(va)
    if o is None:
        return None
    out = []
    while True:
        w = struct.unpack_from('<H', d, o)[0]
        if w == 0:
            break
        if w > 0x7e:
            return None
        out.append(chr(w))
        o += 2
        if len(out) > 40:
            return None
    return ''.join(out)

# Harvest `mov DWORD PTR ds:0xTARGET,0xIMM` and `mov WORD PTR ds:0xTARGET,0xIMM`
slots = {}
rd = re.compile(r'^([0-9a-f]{8}):\t.*\tmov\s+(DWORD|WORD) PTR ds:0x([0-9a-f]+),0x([0-9a-f]+)$')
for line in open(ASM, errors='replace'):
    m = rd.match(line.rstrip('\n'))
    if not m:
        continue
    a = int(m.group(1), 16)
    if not (0x10c29000 <= a < 0x10c2a800):
        continue
    slots[int(m.group(3), 16)] = (int(m.group(4), 16), a)

if not slots:
    sys.exit('no stores harvested -- RED')

lo, hi = min(slots), max(slots)
# Anchor the phase on the "-EHs"/"-EHa" boolean pair, found rather than assumed.
anchor = None
for t, (v, _) in slots.items():
    if wstr(v) == '-EHs' and slots.get(t + 8, (None,))[0] in (0x101, 0x501):
        anchor = t
        break
if anchor is None:
    sys.exit('anchor record "-EHs" not found -- RED')
print(f"stores harvested : {len(slots)} over 0x{lo:08x}..0x{hi:08x}")
print(f"phase anchor     : record base 0x{anchor:08x} (name '-EHs', kind "
      f"0x{slots[anchor+8][0]:x})\n")
print(f"{'record':<12}{'name':<14}{'value ptr':<12}{'kind':<8}what")
start = lo - ((lo - anchor) % 12)
for t in range(start, hi + 1, 12):
    n = slots.get(t)
    v = slots.get(t + 4)
    k = slots.get(t + 8)
    if not n:
        continue
    nm = wstr(n[0])
    if nm is None:
        continue
    vp = f"0x{v[0]:08x}" if v else "(reg)"
    kd = f"0x{k[0]:04x}" if k else "-"
    tag = ''
    if v and v[0] == 0x10c2ea98:
        tag = '<<< the inline size ceiling shift k, FUN_10b5e4cc:0x10b5e4cc'
    print(f"0x{t:08x}  {nm:<14}{vp:<12}{kd:<8}{tag}")

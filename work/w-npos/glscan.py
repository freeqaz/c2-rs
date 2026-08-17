#!/usr/bin/env python3
"""Scan a .gl for data-object-shaped records (the c2-il data_object_at frame,
loosened: attr/linkage NOT gated) and print every field, so the emitted/
not-emitted discriminator can be read off. Also prints function-record shapes
(82 07 xx) for the same names when asked."""
import sys, re

def read_varint(b, p):
    # mirror c2-il readers::read_varint: byte < 0x80 => value; 0x80 => u32 LE follows
    v = b[p]
    if v < 0x80:
        return v, p+1
    if v == 0x80:
        return int.from_bytes(b[p+1:p+5], "little"), p+5
    return None, p

def scan(path, name_filter=None):
    gl = open(path, "rb").read()
    out = []
    # find NUL-terminated printable names of len>=2
    for m in re.finditer(rb'[\x20-\x7e]{2,200}\x00', gl):
        name = m.group(0)[:-1]
        nul = m.end() - 1
        if name_filter and name_filter.encode() not in name:
            continue
        i = nul + 1
        if i >= len(gl): continue
        tag = gl[i]
        if tag & 0x80 == 0: continue
        j = i + 1
        wide = None
        if tag & 0x40:  # TAG_WIDE? check value
            pass
        # replicate: TAG_WIDE bit — need its value; c2-il TAG_WIDE
        # from codec: assume 0x40? print both interpretations
        kind_pos = j
        row = dict(name=name.decode(), off=nul, tag=tag,
                   nxt=gl[nul+1:nul+12].hex(" "))
        out.append(row)
    return out

for path in sys.argv[1:]:
    filt = None
    if "=" in path:
        path, filt = path.split("=", 1)
    print("==", path, "filter:", filt)
    for r in scan(path, filt):
        print(f"  {r['name'][:70]:70s} tag={r['tag']:02x} bytes-after-nul: {r['nxt']}")

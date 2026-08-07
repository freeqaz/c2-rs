#!/usr/bin/env python3
"""objdump.py — section table + symbol table of a COFF, for lane w-align16.

Only used to LOCALIZE a divergence the differential already found; the
differential is still the judge. Prints nothing the compare does not already
decide.

    objdump.py <a.obj> [<b.obj>]
"""
import struct
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from glread import cstr  # noqa: E402

CLASS = {2: "EXTERNAL", 3: "STATIC", 103: "FILE", 105: "LABEL"}


def dump(path):
    b = open(path, "rb").read()
    nsec = struct.unpack_from("<H", b, 2)[0]
    symptr = struct.unpack_from("<I", b, 8)[0]
    nsym = struct.unpack_from("<I", b, 12)[0]
    opt = struct.unpack_from("<H", b, 16)[0]
    off = 20 + opt
    strtab = symptr + 18 * nsym
    lines = [f"# {path}   len={len(b)}  sections={nsec}  symbols={nsym}"]
    for k in range(nsec):
        o = off + 40 * k
        raw = b[o:o + 8]
        name = (cstr(b, strtab + int(raw[1:].rstrip(b'\0').decode()))
                if raw[0:1] == b"/" else raw.rstrip(b"\0").decode("latin1"))
        vs, va, sz, ptr, rp, _lp, nr, _nl, ch = struct.unpack_from("<IIIIIIHHI", b, o + 8)
        lines.append(f"  sec[{k+1}] {name:12s} vsize={vs:<6d} size={sz:<6d} "
                     f"ptr={ptr:<6d} nreloc={nr} ch={ch:08x} align_nibble={(ch>>20)&0xF}")
        if nr:
            for j in range(nr):
                ro = rp + 10 * j
                va_, sym, ty = struct.unpack_from("<IIH", b, ro)
                lines.append(f"        reloc va={va_} sym={sym} type={ty:04x}")
    k = 0
    while k < nsym:
        o = symptr + 18 * k
        raw = b[o:o + 8]
        name = (cstr(b, strtab + struct.unpack_from("<I", b, o + 4)[0])
                if raw[0:4] == b"\0\0\0\0" else raw.rstrip(b"\0").decode("latin1"))
        val, sec, ty, sc, naux = struct.unpack_from("<IhHBB", b, o + 8)
        lines.append(f"  sym[{k:2d}] {name:26s} val={val:<6d} sec={sec:<3d} "
                     f"type={ty:04x} class={CLASS.get(sc, sc)} naux={naux}")
        for a in range(naux):
            lines.append(f"        aux {b[o+18*(a+1):o+18*(a+2)].hex(' ')}")
        k += 1 + naux
    return lines


def main():
    a = dump(sys.argv[1])
    if len(sys.argv) < 3:
        print("\n".join(a))
        return
    bl = dump(sys.argv[2])
    n = max(len(a), len(bl))
    for i in range(n):
        x = a[i] if i < len(a) else "(none)"
        y = bl[i] if i < len(bl) else "(none)"
        mark = "  " if x == y else ">>"
        print(f"{mark} A| {x}")
        if x != y:
            print(f"   B| {y}")


main()

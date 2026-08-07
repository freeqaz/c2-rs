#!/usr/bin/env python3
"""objread.py — section order + symbol table of a captured reference obj.

Read-only. The judge is the byte-exact compare, not this file; this exists to
say WHERE the divergence is, which the compare cannot.

    objread.py <caps-dir>            # one line of section order per cell
    objread.py <caps-dir> --syms     # + the full symbol table per cell
"""
import os
import struct
import sys

SYMLEN = 18


def sections(img):
    nsec = struct.unpack_from("<H", img, 2)[0]
    ptr_sym = struct.unpack_from("<I", img, 8)[0]
    nsym = struct.unpack_from("<I", img, 12)[0]
    strtab = ptr_sym + nsym * SYMLEN
    out = []
    for i in range(nsec):
        h = 20 + i * 40
        name = img[h:h + 8].rstrip(b"\0").decode("ascii", "replace")
        ch = struct.unpack_from("<I", img, h + 36)[0]
        size = struct.unpack_from("<I", img, h + 16)[0]
        ptr_raw = struct.unpack_from("<I", img, h + 20)[0]
        nrel = struct.unpack_from("<H", img, h + 32)[0]
        tag = name
        if name == ".XBLD$W" and ptr_raw:
            tag = ".XBLD$W:" + img[ptr_raw:ptr_raw + 2].decode("ascii", "replace")
        out.append((tag, size, (ch >> 20) & 0xF, nrel, bool(ch & 0x1000)))
    return out, ptr_sym, nsym, strtab


def symname(img, rec, strtab):
    if rec[:4] == b"\0\0\0\0":
        off = struct.unpack_from("<I", rec, 4)[0]
        end = img.index(b"\0", strtab + off)
        return img[strtab + off:end].decode("ascii", "replace")
    return rec[:8].rstrip(b"\0").decode("ascii", "replace")


def symbols(img, ptr_sym, nsym, strtab):
    out = []
    i = 0
    while i < nsym:
        rec = img[ptr_sym + i * SYMLEN:ptr_sym + (i + 1) * SYMLEN]
        n = symname(img, rec, strtab)
        val = struct.unpack_from("<I", rec, 8)[0]
        sec = struct.unpack_from("<h", rec, 12)[0]
        typ = struct.unpack_from("<H", rec, 14)[0]
        sc = rec[16]
        naux = rec[17]
        out.append((i, n, val, sec, typ, sc, naux))
        i += 1 + naux
    return out


def main():
    root = sys.argv[1]
    want_syms = "--syms" in sys.argv
    for cell in sorted(os.listdir(root)):
        p = os.path.join(root, cell, "ref.obj")
        if not os.path.isfile(p):
            print(f"{cell:34s} (no ref.obj)")
            continue
        img = open(p, "rb").read()
        secs, ptr_sym, nsym, strtab = sections(img)
        order = " ".join(
            s[0] + ("*" if s[4] else "") for s in secs
        )
        print(f"{cell:34s} n={len(secs):2d}  {order}")
        if want_syms:
            for s in secs:
                if s[0] in (".bss", ".data"):
                    print(f"      {s[0]:8s} size={s[1]:<5d} nibble={s[2]} nrel={s[3]}")
            for (i, n, val, sec, typ, sc, naux) in symbols(img, ptr_sym, nsym, strtab):
                cls = {2: "EXT", 3: "STA", 105: "CLR"}.get(sc, str(sc))
                print(f"      sym[{i:2d}] {n:30s} val={val:<6d} sec={sec:<3d} "
                      f"typ={typ:#06x} {cls} aux={naux}")


if __name__ == "__main__":
    main()

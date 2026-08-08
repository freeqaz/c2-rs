#!/usr/bin/env python3
"""symdump.py -- COFF symbol-table dump for one obj, filtered by a name substring.

Written for lane w-phase7's question: what STORAGE CLASS does c2 give a
`.gl` tag-0x10 ALIAS's own name in the obj?  stdlib only, read-only.

usage: symdump.py <obj> [substring]
"""
import struct, sys

CLASS = {2: "EXTERNAL", 3: "STATIC", 6: "LABEL", 105: "WEAK_EXTERNAL",
         101: "FUNCTION", 103: "FILE", 104: "SECTION"}
# PE/COFF: 1 = NOLIBRARY, 2 = SEARCH_LIBRARY, 3 = SEARCH_ALIAS.
CHAR = {1: "NOLIBRARY", 2: "SEARCH_LIBRARY", 3: "SEARCH_ALIAS"}

def main(path, needle=""):
    b = open(path, "rb").read()
    nsec, symptr, nsym = struct.unpack_from("<H", b, 2)[0], \
        struct.unpack_from("<I", b, 8)[0], struct.unpack_from("<I", b, 12)[0]
    strtab = symptr + 18 * nsym
    def name_at(off):
        e = b.index(b"\0", off)
        return b[off:e].decode("latin1")
    syms = []
    i = 0
    while i < nsym:
        o = symptr + 18 * i
        raw = b[o:o + 8]
        if raw[:4] == b"\0\0\0\0":
            nm = name_at(strtab + struct.unpack_from("<I", raw, 4)[0])
        else:
            nm = raw.split(b"\0")[0].decode("latin1")
        val, sec, typ, sc, naux = struct.unpack_from("<IhHBB", b, o + 8)
        syms.append((i, nm, val, sec, typ, sc, naux, b[o + 18:o + 18 + 18 * naux]))
        i += 1 + naux
    for (i, nm, val, sec, typ, sc, naux, aux) in syms:
        if needle and needle not in nm:
            continue
        extra = ""
        if sc == 105 and len(aux) >= 12:
            tag, ch = struct.unpack_from("<II", aux, 0)
            tname = next((s[1] for s in syms if s[0] == tag), "?")
            extra = "  -> WEAK default sym#%d %s  characteristics=%s" % (
                tag, tname, CHAR.get(ch, ch))
        print("#%-5d sec=%-4d val=%-8d type=%#06x class=%-14s aux=%d  %s%s"
              % (i, sec, val, typ, CLASS.get(sc, sc), naux, nm, extra))

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "")

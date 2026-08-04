#!/usr/bin/env python3
"""Scratch COFF (X360 PPC-BE) reader for lane w-objshape. Extends work/w6-coff.py.
Tooling only -- not workspace code, no correctness claim rides on it."""
import struct, sys

RELOC = {
    0x0000: "ABSOLUTE", 0x0001: "ADDR64", 0x0002: "ADDR32", 0x0003: "ADDR24",
    0x0004: "ADDR16", 0x0005: "ADDR14", 0x0006: "REL24", 0x0007: "REL14",
    0x000A: "ADDR32NB", 0x000B: "SECREL", 0x000C: "SECTION", 0x000F: "SECREL16",
    0x0010: "REFHI", 0x0011: "REFLO", 0x0012: "PAIR", 0x0013: "SECRELLO",
    0x0015: "GPREL", 0x0016: "TOKEN",
}
SEL = {1: "NODUPLICATES", 2: "ANY", 3: "SAME_SIZE", 4: "EXACT_MATCH",
       5: "ASSOCIATIVE", 6: "LARGEST"}
SC = {2: "EXTERNAL", 3: "STATIC", 6: "LABEL", 103: "FILE", 101: "FUNCTION",
      105: "SECTION", 3 + 0: "STATIC"}
CH_BITS = [
    (0x00000020, "CNT_CODE"), (0x00000040, "CNT_INITIALIZED_DATA"),
    (0x00000080, "CNT_UNINITIALIZED_DATA"), (0x00000200, "LNK_INFO"),
    (0x00000800, "LNK_REMOVE"), (0x00001000, "LNK_COMDAT"),
    (0x02000000, "MEM_DISCARDABLE"), (0x04000000, "MEM_NOT_CACHED"),
    (0x08000000, "MEM_NOT_PAGED"), (0x10000000, "MEM_SHARED"),
    (0x20000000, "MEM_EXECUTE"), (0x40000000, "MEM_READ"),
    (0x80000000, "MEM_WRITE"),
]


def chdec(ch):
    parts = [n for b, n in CH_BITS if ch & b]
    a = (ch >> 20) & 0xF
    if a:
        parts.append("ALIGN_%d" % (1 << (a - 1)))
    return "|".join(parts)


class Obj:
    def __init__(self, data):
        self.data = data
        (self.mach, self.nsec, self.ts, self.symptr, self.nsym,
         self.optsz, self.chars) = struct.unpack_from("<HHIIIHH", data, 0)
        self.stroff = self.symptr + self.nsym * 18
        self.strsize = struct.unpack_from("<I", data, self.stroff)[0] if self.stroff + 4 <= len(data) else 0
        self.secs = []
        off = 20 + self.optsz
        for i in range(self.nsec):
            raw = data[off:off + 40]
            nm = raw[0:8].rstrip(b"\0").decode("latin1")
            if nm.startswith("/"):
                nm = self.strat(int(nm[1:]))
            (vsz, vaddr, szraw, ptrraw, ptrrel, ptrln,
             nrel, nln, ch) = struct.unpack_from("<IIIIIIHHI", raw, 8)
            self.secs.append(dict(idx=i + 1, name=nm, vsz=vsz, vaddr=vaddr,
                                  size=szraw, ptr=ptrraw, ptrrel=ptrrel,
                                  nrel=nrel, nln=nln, ch=ch, hdroff=off))
            off += 40
        self.syms = []
        i = 0
        while i < self.nsym:
            e = data[self.symptr + i * 18: self.symptr + i * 18 + 18]
            if e[0:4] == b"\0\0\0\0":
                nm = self.strat(struct.unpack_from("<I", e, 4)[0])
                short = False
            else:
                nm = e[0:8].rstrip(b"\0").decode("latin1")
                short = True
            val, secnum, typ, sc, naux = struct.unpack_from("<IhHBB", e, 8)
            aux = [data[self.symptr + (i + 1 + k) * 18: self.symptr + (i + 2 + k) * 18]
                   for k in range(naux)]
            self.syms.append(dict(idx=i, name=nm, val=val, sec=secnum, typ=typ,
                                  sc=sc, naux=naux, aux=aux, short=short))
            i += 1 + naux

    def strat(self, o):
        return self.data[self.stroff + o:].split(b"\0")[0].decode("latin1")

    def secdata(self, s):
        return self.data[s["ptr"]: s["ptr"] + s["size"]] if s["ptr"] else b""

    def relocs(self, s):
        out = []
        for k in range(s["nrel"]):
            va, sym, ty = struct.unpack_from("<IIH", self.data, s["ptrrel"] + k * 10)
            out.append((va, sym, ty))
        return out

    def symname(self, idx):
        for s in self.syms:
            if s["idx"] == idx:
                return s["name"]
        for s in self.syms:
            if s["idx"] < idx < s["idx"] + 1 + s["naux"]:
                return "<aux of %s>" % s["name"]
        return "?%d" % idx


def auxdec(o, s):
    """Decode the aux record for section symbols / functions."""
    out = []
    for a in s["aux"]:
        if s["sc"] == 3 and s["sec"] > 0:
            ln, nrel, nln, cks, num, sel = struct.unpack_from("<IHHIHB", a, 0)
            out.append("Length=0x%x NumberOfRelocations=%d NumberOfLinenumbers=%d "
                       "CheckSum=0x%08x Number=%d Selection=%d(%s) tail=%s"
                       % (ln, nrel, nln, cks, num, sel, SEL.get(sel, "-"),
                          a[15:18].hex()))
        else:
            out.append("raw=" + a.hex())
    return out


def dump(path, show_raw=True):
    data = open(path, "rb").read()
    o = Obj(data)
    print("== %s  (%d bytes)" % (path, len(data)))
    print("hdr: Machine=0x%04x NumberOfSections=%d PointerToSymbolTable=0x%x "
          "NumberOfSymbols=%d Characteristics=0x%04x strtab@0x%x size=%d"
          % (o.mach, o.nsec, o.symptr, o.nsym, o.chars, o.stroff, o.strsize))
    print("\n-- sections")
    for s in o.secs:
        print("%2d %-12s size=0x%-5x vsz=0x%-5x ptr=0x%-5x nrel=%-3d ch=0x%08x  %s"
              % (s["idx"], s["name"], s["size"], s["vsz"], s["ptr"], s["nrel"],
                 s["ch"], chdec(s["ch"])))
    print("\n-- symbols (%d records)" % o.nsym)
    for s in o.syms:
        print("%3d %-34s Value=0x%-6x Sec=%-3d Type=0x%04x SC=%-3d(%-8s) naux=%d"
              % (s["idx"], s["name"][:34], s["val"], s["sec"], s["typ"], s["sc"],
                 SC.get(s["sc"], "?"), s["naux"]))
        for line in auxdec(o, s):
            print("      aux: " + line)
    print("\n-- relocations")
    for s in o.secs:
        if not s["nrel"]:
            continue
        print("  [%s] ptr=0x%x n=%d" % (s["name"], s["ptrrel"], s["nrel"]))
        for va, sym, ty in o.relocs(s):
            print("    off=0x%-4x type=0x%04x %-9s sym=%-3d %s"
                  % (va, ty, RELOC.get(ty, "?"), sym, o.symname(sym)))
    if show_raw:
        print("\n-- raw data")
        for s in o.secs:
            d = o.secdata(s)
            if not d:
                print("  [%s] (no raw data)" % s["name"])
                continue
            print("  [%s] %d B" % (s["name"], len(d)))
            for i in range(0, len(d), 16):
                ch = d[i:i + 16]
                print("    %04x  %-47s  %s" % (i, ch.hex(" "),
                      "".join(chr(c) if 32 <= c < 127 else "." for c in ch)))
        print("\n-- string table")
        blob = data[o.stroff + 4: o.stroff + o.strsize]
        off = 4
        for part in blob.split(b"\0")[:-1]:
            print("    +%-4d %s" % (off, part.decode("latin1")))
            off += len(part) + 1


if __name__ == "__main__":
    for p in sys.argv[1:]:
        dump(p)

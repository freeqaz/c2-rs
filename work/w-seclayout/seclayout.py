#!/usr/bin/env python3
"""w-seclayout — READ c2's real section layout for a workload TU, and say what
the port's two writers would have to produce.

Not a count.  For each obj it prints the section-name multiset, every `.text`
COMDAT with its characteristics word and its aux Selection byte, and the
function symbol that lives in it — because "COMDAT-style linkage against a
packed single-`.text` writer" (#2864/#232) is a statement about `/Ox`, and the
workload is `/O1`, which implies `/Gy`.

  seclayout.py <obj> [<obj> ...]
"""
import struct
import sys

SEL = {
    0: "-", 1: "NODUPLICATES", 2: "ANY", 3: "SAME_SIZE",
    4: "EXACT_MATCH", 5: "ASSOCIATIVE", 6: "LARGEST",
}
IMAGE_SCN_LNK_COMDAT = 0x1000


def read_obj(path):
    b = open(path, "rb").read()
    # COFF header: Machine(2) NumberOfSections(2) TimeDateStamp(4)
    # PointerToSymbolTable(4) NumberOfSymbols(4).
    nsec, _tds, symptr, nsym = struct.unpack_from("<HIII", b, 2)
    secs = []
    for i in range(nsec):
        o = 20 + 40 * i
        name = b[o:o + 8].rstrip(b"\0").decode("ascii", "replace")
        chars = struct.unpack_from("<I", b, o + 36)[0]
        raw = struct.unpack_from("<I", b, o + 16)[0]
        secs.append({"n": i + 1, "name": name, "chars": chars, "raw": raw,
                     "sel": None, "syms": []})
    strtab_off = symptr + 18 * nsym
    i = 0
    while i < nsym:
        o = symptr + 18 * i
        raw8 = b[o:o + 8]
        if raw8[:4] == b"\0\0\0\0":
            off = struct.unpack_from("<I", raw8, 4)[0]
            end = b.index(b"\0", strtab_off + off)
            name = b[strtab_off + off:end].decode("ascii", "replace")
        else:
            name = raw8.rstrip(b"\0").decode("ascii", "replace")
        val, secnum, _typ, sclass, naux = struct.unpack_from("<IhHBB", b, o + 8)
        if 1 <= secnum <= nsec:
            s = secs[secnum - 1]
            if sclass == 3 and naux == 1 and name == s["name"]:
                s["sel"] = b[o + 18 + 14]
            elif sclass == 2:                     # EXTERNAL
                s["syms"].append((name, val))
        i += 1 + naux
    return secs


def main():
    for path in sys.argv[1:]:
        secs = read_obj(path)
        names = {}
        for s in secs:
            names[s["name"]] = names.get(s["name"], 0) + 1
        print(f"== {path}   {len(secs)} sections")
        print("   section names: " + ", ".join(
            f"{k}x{v}" for k, v in sorted(names.items(), key=lambda kv: -kv[1])))
        texts = [s for s in secs if s["name"] == ".text"]
        comdat = [s for s in texts if s["chars"] & IMAGE_SCN_LNK_COMDAT]
        packed = [s for s in texts if not s["chars"] & IMAGE_SCN_LNK_COMDAT]
        print(f"   .text: {len(texts)} total — {len(comdat)} COMDAT, "
              f"{len(packed)} packed  => "
              f"{'MIXED' if comdat and packed else ('all COMDAT' if comdat else 'all packed')}")
        selhist = {}
        for s in texts:
            selhist[s["sel"]] = selhist.get(s["sel"], 0) + 1
        print("   .text aux Selection: " + ", ".join(
            f"{SEL.get(k, k)}({k})x{v}" for k, v in sorted(selhist.items(),
                                                           key=lambda kv: -kv[1])))
        for s in texts:
            sym = ", ".join(f"{n}@{v}" for n, v in s["syms"]) or "(no external)"
            print(f"     #{s['n']:>3} raw={s['raw']:>5} chars=0x{s['chars']:08x} "
                  f"sel={SEL.get(s['sel'], s['sel'])} {sym}")
        print()


if __name__ == "__main__":
    main()

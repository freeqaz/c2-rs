#!/usr/bin/env python3
"""w-three — a minimal XCOFF/PE-COFF (PPC BE) section + COMDAT dumper.

Tooling, outside the std-only workspace (same status as scripts/plot_perf.py).
It reads ONLY the reference objs c2 itself produced; it never writes an obj and
nothing here is on any acceptance path.

REFUSES rather than reporting a null on: a file too short to hold a header, a
machine id that is not the Xbox 360 PPC BE one, a section count of 0, or a
symbol table that does not close. `w-loo`'s zero-reach guard is the precedent —
without it, a mutant printed 52 margins of 0 and read as a clean null.
"""
import struct
import sys

IMAGE_FILE_MACHINE_POWERPCBE = 0x01F2
STYP = {
    0x00000020: "CNT_CODE",
    0x00000040: "CNT_INITIALIZED_DATA",
    0x00000080: "CNT_UNINITIALIZED_DATA",
    0x00000200: "LNK_INFO",
    0x00000800: "LNK_REMOVE",
    0x00001000: "LNK_COMDAT",
    0x02000000: "MEM_DISCARDABLE",
    0x40000000: "MEM_READ",
    0x80000000: "MEM_WRITE",
    0x20000000: "MEM_EXECUTE",
}
SEL = {
    1: "NODUPLICATES",
    2: "ANY",
    3: "SAME_SIZE",
    4: "EXACT_MATCH",
    5: "ASSOCIATIVE",
    6: "LARGEST",
}


def die(msg):
    print(f"REFUSE: {msg}", file=sys.stderr)
    sys.exit(2)


def dump(path):
    with open(path, "rb") as fh:
        b = fh.read()
    if len(b) < 20:
        die(f"{path}: {len(b)} bytes cannot hold a COFF header")
    machine, nsec, tstamp, symptr, nsym, optsz, chars = struct.unpack_from("<HHIIIHH", b, 0)
    if machine != IMAGE_FILE_MACHINE_POWERPCBE:
        die(f"{path}: machine 0x{machine:04x} is not PPC BE (0x01f2)")
    if nsec == 0:
        die(f"{path}: 0 sections — a section count of zero is a refusal, not a result")
    if symptr + nsym * 18 > len(b):
        die(f"{path}: symbol table does not close ({nsym} syms at {symptr}, file {len(b)})")
    strtab_off = symptr + nsym * 18

    def name_at(raw):
        if raw[:4] == b"\x00\x00\x00\x00":
            off = struct.unpack_from("<I", raw, 4)[0]
            end = b.index(b"\x00", strtab_off + off)
            return b[strtab_off + off:end].decode("latin1")
        return raw.rstrip(b"\x00").decode("latin1")

    secs = []
    for i in range(nsec):
        o = 20 + optsz + i * 40
        raw = b[o:o + 8]
        vsz, vaddr, rawsz, rawptr, relptr, lnptr, nrel, nln, fl = struct.unpack_from("<IIIIIIHHI", b, o + 8)
        secs.append(dict(idx=i + 1, name=name_at(raw), rawsz=rawsz, rawptr=rawptr, nrel=nrel, flags=fl))

    syms, i = [], 0
    while i < nsym:
        o = symptr + i * 18
        raw = b[o:o + 8]
        value, secnum, typ, sclass, naux = struct.unpack_from("<IhHBB", b, o + 8)
        s = dict(name=name_at(raw), value=value, sec=secnum, sclass=sclass, naux=naux, aux=None)
        if naux and sclass == 3:  # IMAGE_SYM_CLASS_STATIC -> section definition aux
            a = b[o + 18:o + 36]
            length, nrel, nln, cksum, number, sel = struct.unpack_from("<IHHIHB", a, 0)
            s["aux"] = dict(length=length, nrel=nrel, cksum=cksum, number=number, sel=sel)
        syms.append(s)
        i += 1 + naux

    print(f"== {path}  {len(b)} B  machine 0x{machine:04x}  {nsec} sections  {nsym} symbol slots")
    ncomdat = 0
    for s in secs:
        names = [x["name"] for x in syms if x["sec"] == s["idx"] and x["sclass"] == 3]
        comdat = bool(s["flags"] & 0x1000)
        ncomdat += comdat
        selstr = ""
        if comdat:
            for x in syms:
                if x["sec"] == s["idx"] and x["aux"]:
                    selstr = f" sel={x['aux']['sel']}({SEL.get(x['aux']['sel'], '?')})"
                    break
        fl = " ".join(v for k, v in STYP.items() if s["flags"] & k)
        print(f"  [{s['idx']:2d}] {s['name']:<12} raw={s['rawsz']:<6} nrel={s['nrel']:<3}"
              f"{' COMDAT' if comdat else '       '}{selstr}  {fl}")
        if names:
            print(f"       secdef syms: {names}")
    # every EXTERNAL symbol defined in a section, with its section name
    print("  --- defined external symbols (sclass=2, sec>0) ---")
    ext = [x for x in syms if x["sclass"] == 2 and x["sec"] > 0]
    for x in sorted(ext, key=lambda y: (y["sec"], y["value"])):
        sn = secs[x["sec"] - 1]["name"]
        sz = secs[x["sec"] - 1]["rawsz"]
        print(f"       {sn:<10} +{x['value']:<5} rawsz={sz:<5} {x['name']}")
    print("  --- undefined external symbols (sec==0) ---")
    und = [x["name"] for x in syms if x["sclass"] == 2 and x["sec"] == 0]
    for n in sorted(und):
        print(f"       UNDEF {n}")
    text = [s for s in secs if s["name"].startswith(".text")]
    tbytes = sum(s["rawsz"] for s in text)
    print(f"  SUMMARY: sections {nsec} (distinct {len({s['name'] for s in secs})}), "
          f"COMDAT {ncomdat}, .text* sections {len(text)} totalling {tbytes} B, "
          f"defined-ext {len(ext)}, undef-ext {len(und)}")
    if nsym == 0:
        die(f"{path}: 0 symbols — an empty symbol table is a refusal, not a result")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        die("usage: objdump.py <obj> [<obj>...]")
    for p in sys.argv[1:]:
        dump(p)
        print()

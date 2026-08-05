#!/usr/bin/env python3
"""survey.py — obj-SHAPE survey of the FRONTIER, lane w-tu2.

w-tu1's actual selection criterion was not construct count: it dumped every
candidate's reference obj first and picked the one whose obj shape the writer
already had. This reproduces that, mechanically, over all 18 frontier TUs.

Read-only measurement tooling. Outside the std-only Rust workspace, same status
as scripts/gt_dump.py.

For each TU it prints the features that w-tu1 named as the price a construct
count cannot see:
    sections, .pdata COMDATs, $M labels, $T symbols, REFHI/REFLO pairs,
    frames (mflr/stwu), callee-saved GPR saves, indirect calls (bcctrl),
    cr0-vs-cr6 branches, and the size of the blocked bodies.
"""

import struct
import subprocess
import sys
import os

# The writer's 10 names -- crates/c2-core/src/coff/function.rs PORT_WRITER_SECTIONS.
WRITER = {".drectve", ".debug$S", ".XBLD$W", ".text", ".pdata",
          ".rdata", ".text$yc", ".bss", ".CRT$XCU", ".data"}


def rd(o, n, f):
    return struct.unpack_from(f, o, n)[0]


def secname(raw, strtab):
    if raw[0:1] == b"/":
        off = int(raw.rstrip(b"\0")[1:])
        end = strtab.index(b"\0", off)
        return strtab[off:end].decode()
    return raw.rstrip(b"\0").decode()


def parse(path):
    d = open(path, "rb").read()
    nsec = rd(d, 2, "<H")
    symptr = rd(d, 8, "<I")
    nsym = rd(d, 12, "<I")
    strtab = d[symptr + 18 * nsym:]
    secs = []
    for i in range(nsec):
        o = 20 + 40 * i
        secs.append({
            "name": secname(d[o:o + 8], strtab),
            "raw": rd(d, o + 16, "<I"),
            "rawptr": rd(d, o + 20, "<I"),
            "nrel": rd(d, o + 32, "<H"),
            "relptr": rd(d, o + 24, "<I"),
        })
    syms = []
    i = 0
    while i < nsym:
        o = symptr + 18 * i
        nm = d[o:o + 8]
        if nm[0:4] == b"\0\0\0\0":
            off = rd(d, o + 4, "<I")
            end = strtab.index(b"\0", off)
            name = strtab[off:end].decode()
        else:
            name = nm.rstrip(b"\0").decode()
        naux = d[o + 17]
        syms.append({"name": name, "sec": rd(d, o + 12, "<h"),
                     "sc": d[o + 16], "naux": naux})
        i += 1 + naux
    return d, secs, syms


# Relocation types that cost a lane a mechanism (crates/c2-obj/src/reloc.rs).
REFHI, REFLO, PAIR = 0x0010, 0x0011, 0x0012


def survey(path, src):
    d, secs, syms = parse(path)
    names = [s["name"] for s in secs]
    outside = sorted({n for n in names if n not in WRITER})

    text = [s for s in secs if s["name"] == ".text"]
    pdata = [s for s in secs if s["name"] == ".pdata"]
    m_lab = [s for s in syms if s["name"].startswith("$M")]
    t_lab = [s for s in syms if s["name"].startswith("$T")]

    # relocation type census over every section
    rtypes = {}
    for s in secs:
        for r in range(s["nrel"]):
            o = s["relptr"] + 10 * r
            t = rd(d, o + 8, "<H")
            rtypes[t] = rtypes.get(t, 0) + 1

    # instruction-level features over .text
    frames = savegpr = bcctrl = cr0br = std31 = 0
    big = []
    for s in text:
        w = [rd(d, s["rawptr"] + 4 * k, ">I") for k in range(s["raw"] // 4)]
        if any(x == 0x7D8802A6 for x in w):          # mflr r12
            frames += 1
            big.append(s["raw"])
        for x in w:
            if x == 0x4E800421:                       # bcctrl
                bcctrl += 1
            if (x >> 26) == 62 and ((x >> 21) & 31) >= 14:   # std rN(>=14),d(r1)
                std31 += 1
            # bc with BI in cr0 (BI < 4) -- the port's shapes all use cr6
            if (x >> 26) == 16 and ((x >> 16) & 31) < 4:
                cr0br += 1
            if (x >> 26) == 18 and (x & 3) == 1:      # bl (REL24 call site)
                savegpr += 0
    return {
        "src": src, "size": len(d), "nsec": len(secs), "nsym": len(syms),
        "text": len(text), "pdata": len(pdata),
        "outside": outside, "M": len(m_lab), "T": len(t_lab),
        "refhi": rtypes.get(REFHI, 0) + rtypes.get(REFLO, 0),
        "frames": frames, "bcctrl": bcctrl, "cr0br": cr0br, "std": std31,
        "framed_bytes": sorted(big, reverse=True),
    }


def main(argv):
    rows = []
    for line in open(argv[1]):
        src = line.strip()
        if not src:
            continue
        obj = os.path.join(argv[2], src.replace("/", "_") + ".obj")
        if not os.path.exists(obj):
            print(f"  MISSING {src}")
            continue
        rows.append(survey(obj, src))

    hdr = ("  sz   sec sym txt pd  $M $T RH fr bcc cr0 std  outside-writer   src")
    print(hdr)
    print("  " + "-" * (len(hdr) - 2))
    for r in sorted(rows, key=lambda r: (len(r["outside"]), r["frames"],
                                         r["pdata"], r["M"], r["size"])):
        out = ",".join(r["outside"]) or "-"
        print(f"{r['size']:6} {r['nsec']:4} {r['nsym']:3} {r['text']:3} "
              f"{r['pdata']:2} {r['M']:3} {r['T']:2} {r['refhi']:2} "
              f"{r['frames']:2} {r['bcctrl']:3} {r['cr0br']:3} {r['std']:3}  "
              f"{out:16} {r['src']}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

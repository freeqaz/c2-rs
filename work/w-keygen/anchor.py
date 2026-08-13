#!/usr/bin/env python3
"""anchor.py — the SPLICE-0 test on an interior inline site, run against a real
reference obj.

Lane w-keygen measurement tooling. **Read-only with respect to `crates/`.**

THE QUESTION
------------
`docs/whitebox/WB_INLINE_FINDINGS.md` §6.3 and board **#1844** both state that
c2's inlined `?shuffle2` inside `?supershuffle@@YAXPAD@Z` is

    "14 frameless re-allocated words, not `?shuffle2`'s own 15-word COMDAT"

and price the anchor's remedy on that basis — "a lowering for arbitrary loop
bodies **into the caller's register allocation**". This script asks the obj
instead of the record: take the callee's own `.text` COMDAT, drop its trailing
`blr`, and search the caller's words for that exact run.

It is deliberately a *search* and not a fixed offset, so a copy that is present
but placed elsewhere still reads as SPLICE-0 and only a genuinely different
encoding reads as a miss.

WHAT IT IS NOT
--------------
This reads relocation records and section words. It never disassembles, never
consults `c2.dll`, and never asks the port anything. `mismatch` cannot move.

Usage:
    anchor.py <obj> <callee-symbol> <caller-symbol> [<caller-symbol> ...]
"""

import struct
import sys

IMAGE_REL_PPC = {
    0x0000: "ABSOLUTE", 0x0001: "ADDR64", 0x0002: "ADDR32", 0x0003: "ADDR24",
    0x0004: "ADDR16", 0x0005: "ADDR14", 0x0006: "REL24", 0x0007: "REL14",
    0x0008: "TOCREL16", 0x0009: "TOCREL14", 0x000A: "ADDR32NB",
    0x000B: "SECREL", 0x000C: "SECTION", 0x000F: "SECREL16",
    0x0010: "REFHI", 0x0011: "REFLO", 0x0012: "PAIR",
}
# Cross-checked against the port's own constants in
# `crates/c2-core/src/coff/reloc.rs`: ADDR32 0x0002, REL24 0x0006, REFHI 0x0010,
# REFLO 0x0011, PAIR 0x0012. An ad-hoc table off by one convention reads the
# `.pdata` record as ADDR64 and the data quad as TOCREL, and this lane charged
# two refusals against that misreading before checking. Do not retype it.

BLR = 0x4E800020


def load(path):
    d = open(path, "rb").read()
    nsec = struct.unpack_from("<H", d, 2)[0]
    symtab = struct.unpack_from("<I", d, 8)[0]
    nsym = struct.unpack_from("<I", d, 12)[0]
    strtab = symtab + 18 * nsym

    def name(i):
        o = symtab + 18 * i
        raw = d[o:o + 8]
        if raw[:4] == b"\0\0\0\0":
            off = struct.unpack_from("<I", raw, 4)[0]
            e = d.index(b"\0", strtab + off)
            return d[strtab + off:e].decode()
        return raw.rstrip(b"\0").decode()

    owner, i = {}, 0
    while i < nsym:
        o = symtab + 18 * i
        sec = struct.unpack_from("<h", d, o + 12)[0]
        sc, naux = d[o + 16], d[o + 17]
        if sc == 2 and sec > 0:
            owner.setdefault(sec, name(i))
        i += 1 + naux

    text, relocs = {}, {}
    for s in range(nsec):
        o = 20 + 40 * s
        if d[o:o + 8].rstrip(b"\0") != b".text":
            continue
        ln = struct.unpack_from("<I", d, o + 16)[0]
        rp = struct.unpack_from("<I", d, o + 20)[0]
        rq = struct.unpack_from("<I", d, o + 24)[0]
        nrel = struct.unpack_from("<H", d, o + 32)[0]
        sym = owner.get(s + 1, f"<sec{s + 1}>")
        text[sym] = [struct.unpack_from(">I", d, rp + 4 * k)[0] for k in range(ln // 4)]
        relocs[sym] = [
            (struct.unpack_from("<IIH", d, rq + 10 * r)[0],
             IMAGE_REL_PPC.get(struct.unpack_from("<IIH", d, rq + 10 * r)[2], "?"),
             name(struct.unpack_from("<IIH", d, rq + 10 * r)[1]))
            for r in range(nrel)
        ]
    return text, relocs


def main(argv):
    obj, callee, callers = argv[1], argv[2], argv[3:]
    text, relocs = load(obj)
    if callee not in text:
        sys.exit(f"no .text COMDAT for {callee}; have {sorted(text)[:6]}...")
    own = text[callee]
    if own[-1] != BLR:
        print(f"NOTE {callee} does not end in blr ({own[-1]:08x}); using the whole body")
        body = own
    else:
        body = own[:-1]
    print(f"{callee}: {len(own)} words, SPLICE-0 run = {len(body)} words\n")
    for c in callers:
        w = text.get(c)
        if w is None:
            print(f"  {c:32s} ABSENT")
            continue
        hits = [k for k in range(len(w) - len(body) + 1) if w[k:k + len(body)] == body]
        rel = [f"{t}@0x{v:x}->{s}" for v, t, s in relocs.get(c, []) if t == "REL24"]
        verdict = "SPLICE-0 EXACT" if hits else "NOT SPLICE-0"
        print(f"  {c:32s} {len(w):3d} w  bl:{len(rel)}  run@{hits}  {verdict}")
        if not hits:
            # Report the closest alignment, so a near miss is priced rather than
            # merely denied.
            best, bd = None, len(body) + 1
            for k in range(len(w) - len(body) + 1):
                dd = sum(1 for a, b in zip(body, w[k:k + len(body)]) if a != b)
                if dd < bd:
                    bd, best = dd, k
            print(f"        closest alignment at word {best}: {bd} of {len(body)} words differ")
            for j, (a, b) in enumerate(zip(body, w[best:best + len(body)])):
                if a != b:
                    print(f"          w{j}: comdat={a:08x} inlined={b:08x}")


if __name__ == "__main__":
    main(sys.argv)

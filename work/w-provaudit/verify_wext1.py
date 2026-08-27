#!/usr/bin/env python3
"""verify_wext1.py — the promotion condition for `W-EXT-1`, fixed in the
prereg before it was run.

`#3645` hands this lane a dead citation: `middle_interfaces.rs:634` cites
`DISCLOSURE W-EXT-1`, which exists only as a pre-draft in
`WB_READER_FINDINGS.md` §5.3. `w-disclose` declined to carry it on sight,
citing `#3626` — a carried pre-draft held **two wrong addresses in bold for
eight days**. So this lane promotes it only against the image, and only on a
condition registered in advance:

  1. `0x10c1fe40` is a function entry, and
  2. every other address the pre-draft cites lands INSIDE a function, with the
     `0x10b3d5*` group inside one and the same function (the type reader), and
  3. the bytes at `0x10b3d5c1` are an actual `shr ebx,9` / `and ebx,7` pair —
     the one clause `middle_interfaces.rs`'s `type_len` transcription rests on,
     confirmed at the BYTE level and not merely cited.

If any of the three fails, the row is NOT promoted and the citation is
repaired to point at what exists.

Usage: work/w-provaudit/verify_wext1.py [path/to/c2.dll]
"""

import bisect
import csv
import hashlib
import os
import struct
import sys

PINNED_SHA_PREFIX = "c80981c0"

# The addresses `WB_READER_FINDINGS.md` §5.3's pre-draft cites, verbatim.
CITED = {
    "0x10c1fe40": "the TYPE word reader itself (the word)",
    "0x10b3d550": "its one call site in the type reader",
    "0x10b3d5a6": "the gated LEB skip, start",
    "0x10b3d5b4": "the gated LEB skip, end",
    "0x10b3d5c1": "shr ebx,9 / and ebx,7 — the size index",
    "0x10b3d5ea": "the store",
}

# Condition 3, as bytes. `shr ebx,9` is `C1 EB 09`; `and ebx,7` is `83 E3 07`.
EXPECT_BYTES = bytes([0xC1, 0xEB, 0x09, 0x83, 0xE3, 0x07])


class Image:
    def __init__(self, path):
        with open(path, "rb") as fh:
            self.blob = fh.read()
        self.digest = hashlib.sha256(self.blob).hexdigest()
        pe = struct.unpack_from("<I", self.blob, 0x3C)[0]
        nsec = struct.unpack_from("<H", self.blob, pe + 6)[0]
        optsz = struct.unpack_from("<H", self.blob, pe + 20)[0]
        opt = pe + 24
        self.image_base = struct.unpack_from("<I", self.blob, opt + 28)[0]
        self.sections = []
        s = opt + optsz
        for i in range(nsec):
            h = s + i * 40
            vsize, vaddr, rawsz, rawptr = struct.unpack_from("<IIII",
                                                             self.blob, h + 8)
            self.sections.append((vaddr, vsize, rawptr, rawsz))

    def off(self, va):
        rva = va - self.image_base
        for vaddr, vsize, rawptr, rawsz in self.sections:
            if vaddr <= rva < vaddr + vsize:
                d = rva - vaddr
                return rawptr + d if d < rawsz else None
        return None

    def read(self, va, n):
        o = self.off(va)
        return None if o is None else self.blob[o:o + n]


def load_funcs(root):
    p = os.path.join(root, "docs/whitebox/ref/FUNCS.tsv")
    lines = [l for l in open(p) if not l.startswith("#")]
    fns = []
    for x in csv.DictReader(lines, delimiter="\t"):
        try:
            fns.append((int(x["addr"], 16), int(x["size"]), x["tu"]))
        except (ValueError, TypeError):
            pass
    fns.sort()
    return fns


def owner(fns, a):
    starts = [f[0] for f in fns]
    i = bisect.bisect_right(starts, a) - 1
    if i < 0:
        return None
    s, n, tu = fns[i]
    return (s, n, tu) if a < s + n else None


def main(argv):
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.dirname(os.path.dirname(here))
    dll = argv[1] if len(argv) > 1 else os.path.join(
        root, "compilers/X360/16.00.11886.00/c2.dll")
    if not os.path.isfile(dll):
        print(f"SKIP: pinned image absent at {dll}")
        return 2

    img = Image(dll)
    fns = load_funcs(root)
    print(f"image: {os.path.relpath(dll, root)}")
    print(f"sha256: {img.digest}")
    pinned = img.digest.startswith(PINNED_SHA_PREFIX)
    print(f"pinned prefix {PINNED_SHA_PREFIX}…: "
          f"{'MATCH' if pinned else 'MISMATCH — STOP'}")
    print(f"image base: {img.image_base:#x}")
    print(f"FUNCS.tsv: {len(fns)} functions")
    print()

    ok = True

    # ---- condition 1 -------------------------------------------------------
    print("CONDITION 1 — `0x10c1fe40` is a function ENTRY")
    entry = [f for f in fns if f[0] == 0x10C1FE40]
    if entry:
        s, n, tu = entry[0]
        print(f"  PASS  FUN_{s:08x}, size {n}, tu {tu}")
    else:
        print("  FAIL  not a function entry in FUNCS.tsv")
        ok = False
    print()

    # ---- condition 2 -------------------------------------------------------
    print("CONDITION 2 — every cited address lands inside a function, and the")
    print("              `0x10b3d5*` group lands inside ONE function")
    owners = {}
    for a_s, what in CITED.items():
        a = int(a_s, 16)
        o = owner(fns, a)
        if o is None:
            print(f"  FAIL  {a_s}  ORPHAN — inside no FUNCS.tsv function "
                  f"({what})")
            ok = False
            continue
        s, n, tu = o
        owners[a_s] = s
        print(f"  ok    {a_s}  in FUN_{s:08x} (size {n}, tu {tu})  — {what}")
    grp = {owners[k] for k in owners if k.startswith("0x10b3d5")}
    if len(grp) == 1:
        print(f"  PASS  the four `0x10b3d5*` addresses share one owner, "
              f"FUN_{list(grp)[0]:08x}")
    else:
        print(f"  FAIL  the `0x10b3d5*` group spans {len(grp)} functions: "
              f"{[hex(x) for x in sorted(grp)]}")
        ok = False
    print()

    # ---- condition 3 -------------------------------------------------------
    print("CONDITION 3 — the bytes at `0x10b3d5c1` are `shr ebx,9` / "
          "`and ebx,7`")
    got = img.read(0x10B3D5C1, len(EXPECT_BYTES))
    if got is None:
        print("  FAIL  address does not map into any section")
        ok = False
    else:
        print(f"  want  {EXPECT_BYTES.hex(' ')}   "
              f"(shr ebx,9 ; and ebx,7)")
        print(f"  got   {got.hex(' ')}")
        if got == EXPECT_BYTES:
            print("  PASS  byte-identical")
        else:
            print("  FAIL  the clause `type_len` rests on is NOT at this "
                  "address")
            ok = False
    print()

    # ---- the corroborating read, not part of the condition -----------------
    print("CORROBORATION (not a condition) — the 1/2/3-byte discriminator the")
    print("port transcribes is `b1 & 0x80` then `b1 & 0x40`. The reader's own")
    print("first bytes should test exactly those.")
    head = img.read(0x10C1FE40, 32)
    print(f"  FUN_10c1fe40 +0: {head.hex(' ') if head else '<unmapped>'}")
    print()

    verdict = ("PROMOTE" if ok and pinned else "DO NOT PROMOTE")
    print(f"VERDICT: {verdict} — W-EXT-1 "
          f"({'all three conditions hold' if ok and pinned else 'a registered condition failed'})")
    return 0 if (ok and pinned) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))

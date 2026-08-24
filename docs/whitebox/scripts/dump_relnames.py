#!/usr/bin/env python3
"""Decode c2.dll's relation-code NAME array and the three relation remap tables.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Reads the pinned image directly — no Ghidra, no flat export, no `data.tsv`.
That independence is the point: board `#2207` / `WB_SELECT_RECONCILED.md` §8
decoded these names from Ghidra's `data.tsv`, and `WB_RELATION_FINDINGS.md` §2
derived a CONFLICTING assignment from the tables' algebra alone.  This tool
reads the raw bytes so a third, independent decode can settle it.

    0x10c38690   pointer array into the name pool   (#2207, UNVERIFIED)
    0x10b197f4   top of the name string pool        (#2207, UNVERIFIED)
    0x10b189a4   signedness remap, 20 bytes         (w-c7 §1)
    0x10b189b8   `b8` — reflection or strictness    (w-c7 §1, DISPUTED)
    0x10b189cc   negation, 20 bytes                 (w-c7 §1)

Usage:
    python3 docs/whitebox/scripts/dump_relnames.py <c2.dll>
    python3 docs/whitebox/scripts/dump_relnames.py <c2.dll> --entries N
    python3 docs/whitebox/scripts/dump_relnames.py <c2.dll> --array-va 0x...
    python3 docs/whitebox/scripts/dump_relnames.py <c2.dll> --tables
    python3 docs/whitebox/scripts/dump_relnames.py <c2.dll> --find-arrays
    python3 docs/whitebox/scripts/dump_relnames.py <c2.dll> --pool

`--entries` is the M1 instrument-defect control: the walk MUST stop on its own
terms (a non-pointer, a target outside a section, a non-printable target), not
because the bound ran out.  The row `STOP:` says which happened.

`--find-arrays` is the S1d search: it scans every 4-aligned word in the image
for a run of >= 8 pointers into the name pool, so a SECOND naming array cannot
hide behind the one #2207 cited.

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md §0); the script verifies the digest FIRST and refuses
otherwise, with a non-zero exit.
"""

import hashlib
import struct
import sys

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"
PINNED_SIZE = 1347072

NAME_ARRAY_VA = 0x10C38690          # #2207 — UNVERIFIED at time of writing
NAME_POOL_TOP_VA = 0x10B197F4       # #2207 — UNVERIFIED at time of writing

REMAP_TABLES = [
    (0x10B189A4, "a4", "signedness remap (w-c7 §1)"),
    (0x10B189B8, "b8", "DISPUTED: 'strictness' (w-c7) vs reflection (#2207)"),
    (0x10B189CC, "cc", "negation (w-c7 §1)"),
]
REMAP_LEN = 20


class ImageFenceError(Exception):
    """The image is not the pinned one.  Nothing has been read."""


class Image:
    """A loaded PE with a VA -> file-offset map built from its own section table.

    The sha256 fence runs in __init__ BEFORE any structural parse, so a
    corrupted image cannot reach the PE header logic at all.
    """

    def __init__(self, path, expect=PINNED_SHA256):
        try:
            with open(path, "rb") as fh:
                blob = fh.read()
        except OSError as e:
            raise ImageFenceError(
                "IMAGE FENCE REFUSED %s\n  cannot read it: %s\n"
                "  Nothing was read." % (path, e))
        digest = hashlib.sha256(blob).hexdigest()
        if expect is not None and digest != expect:
            raise ImageFenceError(
                "IMAGE FENCE REFUSED %s\n"
                "  expected sha256 %s (%d bytes)\n"
                "  got      sha256 %s (%d bytes)\n"
                "  Nothing was read.  Populate compilers/ with "
                "scripts/fetch_compilers.sh." % (
                    path, expect, PINNED_SIZE, digest, len(blob)))
        self.blob = blob
        self.digest = digest
        e_lfanew = struct.unpack_from("<I", blob, 0x3C)[0]
        if blob[e_lfanew:e_lfanew + 4] != b"PE\0\0":
            raise ImageFenceError("not a PE: %s" % path)
        coff = e_lfanew + 4
        nsec, = struct.unpack_from("<H", blob, coff + 2)
        opt_size, = struct.unpack_from("<H", blob, coff + 16)
        opt = coff + 20
        self.image_base, = struct.unpack_from("<I", blob, opt + 28)
        sect = opt + opt_size
        self.sections = []
        for i in range(nsec):
            o = sect + i * 40
            name = blob[o:o + 8].rstrip(b"\0").decode("ascii", "replace")
            vsize, vaddr, rawsize, rawptr = struct.unpack_from("<IIII", blob, o + 8)
            self.sections.append((name, vaddr, max(vsize, rawsize), rawptr, rawsize))

    def section_of(self, va):
        rva = va - self.image_base
        for name, vaddr, vsize, rawptr, rawsize in self.sections:
            if vaddr <= rva < vaddr + vsize:
                return name
        return None

    def off(self, va):
        rva = va - self.image_base
        for name, vaddr, vsize, rawptr, rawsize in self.sections:
            if vaddr <= rva < vaddr + vsize:
                d = rva - vaddr
                if d >= rawsize:
                    return None          # zero-fill tail
                return rawptr + d
        return None

    def u32(self, va):
        o = self.off(va)
        return None if o is None else struct.unpack_from("<I", self.blob, o)[0]

    def u8(self, va):
        o = self.off(va)
        return None if o is None else self.blob[o]

    def cstr(self, va, cap=64):
        o = self.off(va)
        if o is None:
            return None
        end = self.blob.find(b"\0", o, o + cap)
        if end < 0:
            return None
        return self.blob[o:end].decode("ascii", "replace")


def printable_name(s):
    return (s is not None and 1 <= len(s) <= 16
            and all(0x20 <= ord(c) < 0x7F for c in s))


def walk_names(img, array_va, bound):
    """Walk the pointer array until it stops ON ITS OWN TERMS.

    Returns (rows, stop_reason, stopped_at_bound).  `stopped_at_bound` True is
    the M1 defect signature: the count is a property of my parameter, not of
    the image.
    """
    rows = []
    for i in range(bound):
        va = array_va + i * 4
        p = img.u32(va)
        if p is None:
            return rows, "array VA %#x is outside every section" % va, False
        if p == 0:
            return rows, "null pointer at entry %d (%#x)" % (i, va), False
        if img.off(p) is None:
            return rows, ("entry %d -> %#x, not in any raw section" % (i, p)), False
        s = img.cstr(p)
        if not printable_name(s):
            return rows, ("entry %d -> %#x, not a short printable string (%r)"
                          % (i, p, s)), False
        rows.append((i, va, p, s))
    return rows, "BOUND %d EXHAUSTED — count is my parameter, not the image's" % bound, True


def find_arrays(img, lo_va, hi_va, minrun=8):
    """S1d: every 4-aligned run of >= minrun pointers into [lo_va, hi_va)."""
    hits = []
    for name, vaddr, vsize, rawptr, rawsize in img.sections:
        base_va = img.image_base + vaddr
        run_start = None
        run_len = 0
        for off in range(0, rawsize - 3, 4):
            p = struct.unpack_from("<I", img.blob, rawptr + off)[0]
            if lo_va <= p < hi_va:
                if run_start is None:
                    run_start = base_va + off
                run_len += 1
            else:
                if run_len >= minrun:
                    hits.append((name, run_start, run_len))
                run_start, run_len = None, 0
        if run_len >= minrun:
            hits.append((name, run_start, run_len))
    return hits


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    path = argv[1]
    args = argv[2:]

    bound = 64
    array_va = NAME_ARRAY_VA
    if "--entries" in args:
        bound = int(args[args.index("--entries") + 1], 0)
    if "--array-va" in args:
        array_va = int(args[args.index("--array-va") + 1], 0)

    try:
        img = Image(path)
    except ImageFenceError as e:
        sys.stderr.write("%s\n" % e)
        return 3

    print("# image     %s" % path)
    print("# sha256    %s  (PINNED, verified)" % img.digest)
    print("# imagebase %#x   sections %s"
          % (img.image_base, ",".join(s[0] for s in img.sections)))

    if "--tables" in args:
        print()
        for va, tag, note in REMAP_TABLES:
            b = [img.u8(va + i) for i in range(REMAP_LEN)]
            print("%s  %#x  %s" % (tag, va, note))
            print("      " + " ".join("%02x" % x for x in b))
            fixed = [i for i, x in enumerate(b) if x == i]
            invol = all(b[x] == i for i, x in enumerate(b)
                        if x is not None and x < REMAP_LEN)
            print("      fixed points: %s" % (fixed,))
            print("      involution:   %s" % invol)
            pairs = sorted({tuple(sorted((i, x))) for i, x in enumerate(b)
                            if x != i and x < REMAP_LEN and b[x] == i})
            print("      2-cycles:     %s" % (pairs,))
            nonfix = [(i, x) for i, x in enumerate(b) if x != i and b[x] != i]
            print("      non-involutive entries: %s" % (nonfix,))
        return 0

    if "--find-arrays" in args:
        lo = NAME_POOL_TOP_VA - 0x400
        hi = NAME_POOL_TOP_VA + 0x40
        print()
        print("# S1d: pointer runs >= 8 into the name pool window "
              "[%#x, %#x)" % (lo, hi))
        for name, start, n in find_arrays(img, lo, hi):
            print("  %-8s %#x  %3d pointers" % (name, start, n))
        return 0

    if "--pool" in args:
        print()
        print("# raw pool bytes below %#x" % NAME_POOL_TOP_VA)
        lo = NAME_POOL_TOP_VA - 0x80
        o = img.off(lo)
        raw = img.blob[o:o + 0x88]
        for i in range(0, len(raw), 16):
            print("  %#x  %-47s  |%s|" % (
                lo + i,
                " ".join("%02x" % c for c in raw[i:i + 16]),
                "".join(chr(c) if 0x20 <= c < 0x7F else "." for c in raw[i:i + 16])))
        return 0

    rows, stop, at_bound = walk_names(img, array_va, bound)
    print("# name array at %#x (section %s), bound %d"
          % (array_va, img.section_of(array_va), bound))
    print()
    print("code  arrayVA     strVA       section  name")
    for i, va, p, s in rows:
        print("%4d  %#010x  %#010x  %-7s  %s"
              % (i, va, p, img.section_of(p), s))
    print()
    print("ENTRIES: %d      denominator: the walk's own stop condition" % len(rows))
    print("STOP:    %s" % stop)
    if at_bound:
        print("M1 DEFECT: the count is a property of --entries.  Re-run higher.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

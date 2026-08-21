#!/usr/bin/env python3
"""Dump c2.dll's PPC mnemonic table and per-opcode machine table.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Reads the pinned image directly — no Ghidra needed — and prints, for a range of
opcode numbers, the mnemonic string and the machine-model row.

    0x10b1b260   mnemonic table, stride 12, [+0] char* name      (P_DAG.md §2.1)
    0x10b202b0   machine table, stride 12, {X, slots, class}      (P_DAG.md §2.1)

Usage:
    python3 docs/whitebox/scripts/dump_opcode_tables.py <c2.dll> [lo] [hi]
    python3 docs/whitebox/scripts/dump_opcode_tables.py <c2.dll> --find add,blr

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md §0); the script verifies the digest and refuses otherwise.
"""

import hashlib
import struct
import sys

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

MNEMONIC_TABLE_VA = 0x10B1B260
MACHINE_TABLE_VA = 0x10B202B0
TABLE_STRIDE = 12


class Image:
    """A loaded PE, with a VA -> file-offset map built from its section table."""

    def __init__(self, path):
        self.blob = open(path, "rb").read()
        self.digest = hashlib.sha256(self.blob).hexdigest()
        e_lfanew = struct.unpack_from("<I", self.blob, 0x3C)[0]
        assert self.blob[e_lfanew:e_lfanew + 4] == b"PE\0\0", "not a PE"
        coff = e_lfanew + 4
        nsec, = struct.unpack_from("<H", self.blob, coff + 2)
        opt_size, = struct.unpack_from("<H", self.blob, coff + 16)
        opt = coff + 20
        self.image_base, = struct.unpack_from("<I", self.blob, opt + 28)
        sect = opt + opt_size
        self.sections = []
        for i in range(nsec):
            o = sect + i * 40
            name = self.blob[o:o + 8].rstrip(b"\0").decode("ascii", "replace")
            vsize, vaddr, rawsize, rawptr = struct.unpack_from("<IIII", self.blob, o + 8)
            self.sections.append((name, vaddr, max(vsize, rawsize), rawptr, rawsize))

    def off(self, va):
        rva = va - self.image_base
        for name, vaddr, vsize, rawptr, rawsize in self.sections:
            if vaddr <= rva < vaddr + vsize:
                d = rva - vaddr
                if d >= rawsize:
                    return None          # in the zero-fill tail (.bss-like)
                return rawptr + d
        return None

    def u32(self, va):
        o = self.off(va)
        return None if o is None else struct.unpack_from("<I", self.blob, o)[0]

    def cstr(self, va, cap=64):
        o = self.off(va)
        if o is None:
            return None
        end = self.blob.find(b"\0", o, o + cap)
        if end < 0:
            return None
        return self.blob[o:end].decode("ascii", "replace")


def row(img, op):
    name_ptr = img.u32(MNEMONIC_TABLE_VA + op * TABLE_STRIDE)
    name = img.cstr(name_ptr) if name_ptr else None
    m = MACHINE_TABLE_VA + op * TABLE_STRIDE
    x, slots, cls = img.u32(m), img.u32(m + 4), img.u32(m + 8)
    return name, x, slots, cls


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    img = Image(argv[1])
    if img.digest != PINNED_SHA256:
        print(f"REFUSE: sha256 {img.digest} is not the pinned image", file=sys.stderr)
        return 1
    if len(argv) > 2 and argv[2] == "--find":
        wanted = set(argv[3].split(","))
        for op in range(0, 0x400):
            name, x, slots, cls = row(img, op)
            if name in wanted:
                print(f"{op:#06x} {op:5d}  {name:<12} X={x} slots={slots} class={cls}")
        return 0
    lo = int(argv[2], 0) if len(argv) > 2 else 0
    hi = int(argv[3], 0) if len(argv) > 3 else 0x40
    print(f"# {argv[1]}  sha256 {img.digest[:12]}…  base {img.image_base:#x}")
    print("# op      dec  mnemonic      X  slots  class(unit)")
    for op in range(lo, hi):
        name, x, slots, cls = row(img, op)
        if name is None:
            continue
        print(f"{op:#06x} {op:5d}  {name:<12} {x:>3} {slots:>6} {cls:>6}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

#!/usr/bin/env python3
"""Dump c2.dll's PPC mnemonic table and per-opcode machine table.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Reads the pinned image directly — no Ghidra needed — and prints, for a range of
opcode numbers, the mnemonic string and the machine-model row.

    0x10b1b260   mnemonic table, stride 12, [+0] char* name      (P_DAG.md §2.1)
    0x10b202b0   machine table, stride 12, {X, slots, class}      (P_DAG.md §2.1)
    0x10c3a578   base-word table, stride 4, one PPC word/opcode   (P_ENCODE.md §2)
    0x10c39b18   encode-form table, stride 4, form 1..111         (P_ENCODE.md §3)
    0x10bfae2d   arm jump table, stride 4, 111 entries            (P_ENCODE.md §4)

Usage:
    python3 docs/whitebox/scripts/dump_opcode_tables.py <c2.dll> [lo] [hi]
    python3 docs/whitebox/scripts/dump_opcode_tables.py <c2.dll> --find add,blr
    python3 docs/whitebox/scripts/dump_opcode_tables.py <c2.dll> --encode [lo] [hi]
    python3 docs/whitebox/scripts/dump_opcode_tables.py <c2.dll> --arms

`--encode` adds read R2's three tables to each row: the base word, the encode
form and the arm VA the form dispatches to.  `--arms` inverts the jump table:
one row per distinct arm, with the forms and the opcode count it serves.

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

# Read R2 (docs/whitebox/ref/P_ENCODE.md).  The encoder FUN_10bf9f15 indexes all
# three by the SAME opcode number the two tables above use.  `_last` is 0x295,
# so 0x001..0x294 is the whole machine opcode space; past it these arrays hold
# unrelated data and must not be quoted (P_ENCODE.md §2.1).
BASE_WORD_TABLE_VA = 0x10C3A578
ENCODE_FORM_TABLE_VA = 0x10C39B18
ARM_JUMP_TABLE_VA = 0x10BFAE2D
ARM_JUMP_TABLE_LEN = 111
ARM_DEFAULT_VA = 0x10BFAE1B          # the `ja 0x10bfae1b` at 0x10bf9f51
LAST_MACHINE_OPCODE = 0x294


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


def encode_row(img, op):
    """(base_word, form, arm_va) for one opcode; arm_va is the default when the
    form is outside 1..111, which is the `edx > 0x6e` branch at 0x10bf9f4e."""
    base = img.u32(BASE_WORD_TABLE_VA + op * 4)
    form = img.u32(ENCODE_FORM_TABLE_VA + op * 4)
    if form is None or not 1 <= form <= ARM_JUMP_TABLE_LEN:
        return base, form, ARM_DEFAULT_VA
    return base, form, img.u32(ARM_JUMP_TABLE_VA + (form - 1) * 4)


def dump_encode(img, lo, hi):
    print("# op      dec  mnemonic       base_word  form  arm")
    for op in range(lo, hi):
        name, _, _, _ = row(img, op)
        base, form, arm = encode_row(img, op)
        if name is None and base in (None, 0):
            continue
        print(f"{op:#06x} {op:5d}  {name or '-':<13} {base:08x}  {form:>4}  {arm:08x}")


def dump_arms(img):
    """One row per distinct arm target, with the forms and opcodes it serves."""
    forms_of = {}
    for i in range(ARM_JUMP_TABLE_LEN):
        forms_of.setdefault(img.u32(ARM_JUMP_TABLE_VA + i * 4), []).append(i + 1)
    ops_of = {}
    for op in range(1, LAST_MACHINE_OPCODE + 1):
        _, form, _ = encode_row(img, op)
        ops_of.setdefault(form, []).append(op)
    print(f"# {len(forms_of)} distinct arms over {ARM_JUMP_TABLE_LEN} jump-table entries")
    print("# arm       nforms  nopcodes  forms")
    rows = []
    for arm, forms in forms_of.items():
        nops = sum(len(ops_of.get(f, ())) for f in forms)
        rows.append((nops, arm, forms))
    for nops, arm, forms in sorted(rows, reverse=True):
        print(f"{arm:08x}  {len(forms):>6}  {nops:>8}  {','.join(map(str, forms))}")


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    img = Image(argv[1])
    if img.digest != PINNED_SHA256:
        print(f"REFUSE: sha256 {img.digest} is not the pinned image", file=sys.stderr)
        return 1
    if len(argv) > 2 and argv[2] == "--arms":
        dump_arms(img)
        return 0
    if len(argv) > 2 and argv[2] == "--encode":
        lo = int(argv[3], 0) if len(argv) > 3 else 1
        hi = int(argv[4], 0) if len(argv) > 4 else LAST_MACHINE_OPCODE + 1
        dump_encode(img, lo, hi)
        return 0
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

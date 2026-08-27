#!/usr/bin/env python3
"""Re-derive the `.gl` RECORD dispatch tables from RAW IMAGE BYTES.

Lane `w-secported`, wave 15, board #3661-#3666.  Whitebox tooling (outside the
std-only `crates/` workspace, per CLAUDE.md).

The dispatcher is `FUN_10b9b8e9` in `p2symtab.c` -- `P_SECTION.md` §1's anchor
row.  `P_SECTION.md` and `docs/whitebox/labels/W-GLREC.tsv` both assert a
27-entry byte-index table at `0x10b9c615` and a 16-entry jump table at
`0x10b9c5d5`.  **Neither commits the table CONTENTS**, so the arm population has
never been enumerated in this tree.  This script enumerates it.

Following `dump_ilarms.py`'s registered discipline: the ONLY hard-coded address
here is the dispatch head.  Every table address, every bound, and every extent
is decoded from the operand bytes of the instructions at that head, so a wrong
carried constant cannot survive.  `--verify` prints the decoded premises beside
the two documents' claims.

    python3 work/w-secported/dump_glrec.py <c2.dll> --verify
    python3 work/w-secported/dump_glrec.py <c2.dll> --tsv > GLREC_ARMS.tsv

Image pin: sha256
c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258.
"""

import hashlib
import struct
import sys

PINNED = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

# The ONLY hard-coded address in this file.  It is the instruction that reads
# the record tag; everything after it is decoded.
TAG_READ_VA = 0x10B9B922
DISPATCHER_VA = 0x10B9B8E9   # containment check only, not a premise


class Image:
    def __init__(self, path):
        self.raw = open(path, "rb").read()
        got = hashlib.sha256(self.raw).hexdigest()
        if got != PINNED:
            raise SystemExit(f"REFUSING: sha256 {got} != pinned {PINNED}")
        e_lfanew = struct.unpack_from("<I", self.raw, 0x3C)[0]
        if self.raw[e_lfanew:e_lfanew + 4] != b"PE\0\0":
            raise SystemExit("not a PE")
        coff = e_lfanew + 4
        nsec, = struct.unpack_from("<H", self.raw, coff + 2)
        optsz, = struct.unpack_from("<H", self.raw, coff + 16)
        opt = coff + 20
        magic, = struct.unpack_from("<H", self.raw, opt)
        if magic != 0x10B:
            raise SystemExit(f"expected PE32, got magic {magic:#x}")
        self.base, = struct.unpack_from("<I", self.raw, opt + 28)
        self.sections = []
        sh = opt + optsz
        for i in range(nsec):
            o = sh + i * 40
            name = self.raw[o:o + 8].rstrip(b"\0").decode("ascii", "replace")
            vsize, vaddr, rsize, raddr = struct.unpack_from("<IIII", self.raw, o + 8)
            self.sections.append((name, vaddr, vsize, raddr, rsize))

    def off(self, va):
        rva = va - self.base
        for _name, vaddr, vsize, raddr, rsize in self.sections:
            if vaddr <= rva < vaddr + max(vsize, rsize):
                d = rva - vaddr
                if d < rsize:
                    return raddr + d
        raise SystemExit(f"VA {va:#x} not mapped")

    def u8(self, va):
        return self.raw[self.off(va)]

    def u32(self, va):
        return struct.unpack_from("<I", self.raw, self.off(va))[0]

    def bytes_at(self, va, n):
        o = self.off(va)
        return self.raw[o:o + n]


def decode(img):
    """Decode the dispatch head.  Returns the premises, none of them carried."""
    p = TAG_READ_VA

    # e8 <rel32>  : call GetByte
    if img.u8(p) != 0xE8:
        raise SystemExit(f"{p:#x}: expected E8 call, got {img.u8(p):#02x}")
    rel = struct.unpack_from("<i", img.bytes_at(p + 1, 4))[0]
    getbyte = p + 5 + rel
    p += 5

    # 0f be c0    : movsx eax,al
    if img.bytes_at(p, 3) != b"\x0f\xbe\xc0":
        raise SystemExit(f"{p:#x}: expected movsx eax,al")
    p += 3
    # 89 45 88    : mov [ebp-0x78],eax   -- the tag slot
    if img.u8(p) != 0x89:
        raise SystemExit(f"{p:#x}: expected mov [ebp-x],eax")
    tag_slot = struct.unpack_from("<b", img.bytes_at(p + 2, 1))[0]
    p += 3
    # 48          : dec eax
    if img.u8(p) != 0x48:
        raise SystemExit(f"{p:#x}: expected dec eax")
    p += 1
    # 83 f8 <imm8>: cmp eax,imm8         -- the BOUND
    if img.bytes_at(p, 2) != b"\x83\xf8":
        raise SystemExit(f"{p:#x}: expected cmp eax,imm8")
    bound = img.u8(p + 2)
    p += 3
    # 0f 87 <rel32>: ja <default arm>
    if img.bytes_at(p, 2) != b"\x0f\x87":
        raise SystemExit(f"{p:#x}: expected ja rel32")
    rel = struct.unpack_from("<i", img.bytes_at(p + 2, 4))[0]
    default_arm = p + 6 + rel
    p += 6
    # 0f b6 80 <disp32>: movzx eax,BYTE PTR [eax+disp32]  -- the BYTE INDEX table
    if img.bytes_at(p, 3) != b"\x0f\xb6\x80":
        raise SystemExit(f"{p:#x}: expected movzx eax,[eax+disp32]")
    byteidx_va = struct.unpack_from("<I", img.bytes_at(p + 3, 4))[0]
    p += 7
    # ff 24 85 <disp32>: jmp [eax*4+disp32]               -- the JUMP table
    if img.bytes_at(p, 3) != b"\xff\x24\x85":
        raise SystemExit(f"{p:#x}: expected jmp [eax*4+disp32]")
    jump_va = struct.unpack_from("<I", img.bytes_at(p + 3, 4))[0]
    p += 7

    return {
        "getbyte": getbyte,
        "tag_slot": tag_slot,
        "bound": bound,
        "default_arm": default_arm,
        "byteidx_va": byteidx_va,
        "jump_va": jump_va,
        "head_end": p,
    }


def tables(img, d):
    # tag-1 indexes the byte table, and the `ja` bounds tag-1 at `bound`, so the
    # byte table has bound+1 entries and the tag population is 1 .. bound+1.
    n_byteidx = d["bound"] + 1
    byteidx = list(img.bytes_at(d["byteidx_va"], n_byteidx))
    # the jump table's extent is decided by the largest index the byte table can
    # produce -- read one more and you invent an arm (`WB_EXPAND` P4.3's trap).
    n_jump = max(byteidx) + 1
    jump = [img.u32(d["jump_va"] + 4 * i) for i in range(n_jump)]
    return byteidx, jump, n_byteidx, n_jump


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    img = Image(sys.argv[1])
    mode = sys.argv[2] if len(sys.argv) > 2 else "--verify"
    d = decode(img)
    byteidx, jump, n_byteidx, n_jump = tables(img, d)

    if mode == "--verify":
        print("# decoded from the dispatch head, nothing carried")
        print(f"tag-read              {TAG_READ_VA:#x}  -> GetByte {d['getbyte']:#x}")
        print(f"tag slot              [ebp{d['tag_slot']:+#x}]")
        print(f"bound (cmp eax,imm8)  {d['bound']:#x}   => tags 0x01 .. {d['bound'] + 1:#04x}")
        print(f"default/fatal arm     {d['default_arm']:#x}")
        print(f"byte-index table      {d['byteidx_va']:#x}  {n_byteidx} entries")
        print(f"jump table            {d['jump_va']:#x}  {n_jump} entries")
        print(f"head ends             {d['head_end']:#x}")
        print()
        print("# claims in the tree, checked")
        for claim, got, want in [
            ("P_SECTION.md:35 byte-index table 0x10b9c615", d["byteidx_va"], 0x10B9C615),
            ("P_SECTION.md:35 byte-index 27 entries", n_byteidx, 27),
            ("P_SECTION.md:35 jump table 0x10b9c5d5", d["jump_va"], 0x10B9C5D5),
            ("P_SECTION.md:35 jump table 16 entries", n_jump, 16),
            ("P_SECTION.md:38 fatal arm 0x10b9c5ca", d["default_arm"], 0x10B9C5CA),
            ("W-GLREC.tsv byteidx[0x04]==byteidx[0x0e]", byteidx[0x04 - 1], byteidx[0x0E - 1]),
            ("W-GLREC.tsv byteidx[0x0e]==byteidx[0x10]", byteidx[0x0E - 1], byteidx[0x10 - 1]),
        ]:
            ok = "OK  " if got == want else "FAIL"
            print(f"{ok} {claim}: got {got:#x} want {want:#x}"
                  if isinstance(got, int) and got > 0xFF
                  else f"{ok} {claim}: got {got} want {want}")
        print()
        print("# arm population")
        distinct = sorted(set(jump))
        print(f"distinct arm targets: {len(distinct)}")
        for a in distinct:
            tags = [i + 1 for i, b in enumerate(byteidx) if jump[b] == a]
            print(f"  {a:#x}  tags {' '.join(f'{t:#04x}' for t in tags)}")
        print(f"  {d['default_arm']:#x}  (default: tag 0x00 and tag > {d['bound'] + 1:#04x})")

    elif mode == "--tsv":
        print("# GENERATED by work/w-secported/dump_glrec.py from the pinned image.")
        print(f"# sha256 {PINNED}")
        print(f"# dispatcher {DISPATCHER_VA:#x}, byte-index {d['byteidx_va']:#x} "
              f"({n_byteidx}), jump {d['jump_va']:#x} ({n_jump})")
        # The `ja` default target IS one of the 16 jump-table slots, so the arm
        # population is 16 SLOTS of which one is the fatal path and 15 are live
        # record handlers.  Named on its own line so a consumer never has to
        # infer which arm is the refusal.
        print(f"# fatal {d['default_arm']:#x}")
        print("tag\tslot\tarm")
        for i, b in enumerate(byteidx):
            print(f"{i + 1:#04x}\t{b}\t{jump[b]:#x}")

    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Re-derive c2.dll's IL-record dispatch tables from RAW IMAGE BYTES.

Lane `w-ilarms`, board #3567-#3572.  Whitebox tooling (outside the std-only
`crates/` workspace, per CLAUDE.md).

**This script deliberately shares NO code with `dump_ilrecord.py`.**  That is
the point: `dump_ilrecord.py` hard-codes `BYTE_TABLE_VA`, `JUMP_TABLE_VA`,
`BYTE_TABLE_LEN`, `JUMP_TABLE_LEN`, `FIRST_OPCODE` and `LAST_OPCODE` as
constants, so re-running it cannot test whether those constants are right.
This one hard-codes only the address of the *dispatch head* and derives every
table address, stride, extent and bound from the operand bytes of the
instructions it decodes there.  The two implementations are then compared;
agreement is a control and disagreement is the finding.

`w-ilarms` prereg §1: decode from raw image bytes, never from a prior lane's
artifact.  `w-relread` registered that rule and it fired; `w-relsite` then found
`P_ILRECORD.md`'s own arm-7 cell wrong in both of its clauses (#3547).

    python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --verify
    python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --arms
    python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --tsv
    python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --refuse
    python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --cross

Image pin: sha256
c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(`C2_MAP_METHOD.md` §0).  The script verifies the digest and refuses otherwise.
"""

import hashlib
import struct
import sys

PINNED = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

# The ONLY hard-coded address in this file.  Everything else is decoded.
DISPATCH_HEAD_VA = 0x10BC2E08
BODY_VA = 0x10BC2D7A          # for the containment check only; not a premise

# The operand-class and attribute tables, board #1591 / P_ILRECORD §3.  Joined
# for reporting only -- no claim here depends on them.
CLASS_TABLE_VA = 0x10B25E48
ATTR_TABLE_VA = 0x10B25F10


class Image:
    """Minimal PE reader: VA -> bytes.  Written for this lane, not reused."""

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
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if vaddr <= rva < vaddr + max(vsize, rsize):
                d = rva - vaddr
                if d < rsize:
                    return raddr + d
                raise SystemExit(f"VA {va:#x} is in {name} but past raw data")
        raise SystemExit(f"VA {va:#x} in no section")

    def bytes(self, va, n):
        o = self.off(va)
        return self.raw[o:o + n]

    def u8(self, va):
        return self.bytes(va, 1)[0]

    def u16(self, va):
        return struct.unpack("<H", self.bytes(va, 2))[0]

    def u32(self, va):
        return struct.unpack("<I", self.bytes(va, 4))[0]

    def section_of(self, va):
        rva = va - self.base
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if vaddr <= rva < vaddr + max(vsize, rsize):
                return name
        return "?"


class Dispatch:
    """Decode the two-level switch head byte by byte, deriving everything."""

    def __init__(self, img, head_va=DISPATCH_HEAD_VA):
        self.img = img
        self.head_va = head_va
        self.trace = []
        b = img.bytes(head_va, 32)
        va = head_va
        i = 0

        def emit(n, text):
            nonlocal i, va
            self.trace.append((va, b[i:i + n].hex(), text))
            i += n
            va += n

        # 8b 55 cc         mov edx, [ebp-0x34]
        if b[0:2] != bytes([0x8B, 0x55]):
            raise SystemExit(f"head is not `mov edx,[ebp+disp8]`: {b[0:4].hex()}")
        self.node_slot = struct.unpack("<b", b[2:3])[0]
        emit(3, f"mov edx, [ebp{self.node_slot:+#x}]")

        # 8d 42 ff         lea eax, [edx-1]
        if b[i:i + 2] != bytes([0x8D, 0x42]):
            raise SystemExit(f"expected `lea eax,[edx+disp8]`: {b[i:i+3].hex()}")
        self.lea_disp = struct.unpack("<b", b[i + 2:i + 3])[0]
        emit(3, f"lea eax, [edx{self.lea_disp:+#x}]")

        # 3d bc 00 00 00   cmp eax, 0xbc     (or 83 f8 imm8)
        if b[i] == 0x3D:
            self.bound = struct.unpack_from("<I", b, i + 1)[0]
            emit(5, f"cmp eax, {self.bound:#x}")
        elif b[i:i + 2] == bytes([0x83, 0xF8]):
            self.bound = b[i + 2]
            emit(3, f"cmp eax, {self.bound:#x}")
        else:
            raise SystemExit(f"expected `cmp eax,imm`: {b[i:i+5].hex()}")

        # 0f 87 rel32      ja <refusal>
        if b[i:i + 2] == bytes([0x0F, 0x87]):
            rel = struct.unpack_from("<i", b, i + 2)[0]
            self.ja_target = va + 6 + rel
            emit(6, f"ja {self.ja_target:#x}")
        elif b[i] == 0x77:
            rel = struct.unpack_from("<b", b, i + 1)[0]
            self.ja_target = va + 2 + rel
            emit(2, f"ja {self.ja_target:#x}")
        else:
            raise SystemExit(f"expected `ja`: {b[i:i+6].hex()}")

        # 0f b6 80 disp32  movzx eax, byte ptr [eax + disp32]
        if b[i:i + 3] != bytes([0x0F, 0xB6, 0x80]):
            raise SystemExit(f"expected `movzx eax,byte[eax+disp32]`: {b[i:i+7].hex()}")
        self.byte_table_va = struct.unpack_from("<I", b, i + 3)[0]
        emit(7, f"movzx eax, byte ptr [eax + {self.byte_table_va:#x}]")

        # ff 24 85 disp32  jmp dword ptr [eax*4 + disp32]
        if b[i:i + 3] != bytes([0xFF, 0x24, 0x85]):
            raise SystemExit(f"expected `jmp dword[eax*4+disp32]`: {b[i:i+7].hex()}")
        self.arm_table_va = struct.unpack_from("<I", b, i + 3)[0]
        emit(7, f"jmp dword ptr [eax*4 + {self.arm_table_va:#x}]")

        # Derived, not assumed:
        self.first_opcode = -self.lea_disp          # lea eax,[edx-1] => index = op-1
        self.n_opcodes = self.bound + 1             # `ja` is unsigned-above, so
        self.last_opcode = self.first_opcode + self.n_opcodes - 1
        self.index = list(img.bytes(self.byte_table_va, self.n_opcodes))
        self.n_arms = max(self.index) + 1
        self.arms = [img.u32(self.arm_table_va + 4 * k) for k in range(self.n_arms)]

    # -- derived views -----------------------------------------------------

    def opcodes_of_arm(self):
        d = {}
        for k, ix in enumerate(self.index):
            d.setdefault(ix, []).append(self.first_opcode + k)
        return d

    def refusal_arms(self):
        """Every arm index whose target equals the out-of-range `ja` target."""
        return [k for k, t in enumerate(self.arms) if t == self.ja_target]

    def byte_table_end(self):
        return self.byte_table_va + self.n_opcodes - 1

    def arm_table_end(self):
        return self.arm_table_va + 4 * self.n_arms - 1


PROLOGUES = {
    "push ebp; mov ebp,esp": bytes([0x55, 0x8B, 0xEC]),
    "push ebx": bytes([0x53]),
    "push esi": bytes([0x56]),
    "push edi": bytes([0x57]),
    "sub esp,imm8": bytes([0x83, 0xEC]),
    "mov edi,edi": bytes([0x8B, 0xFF]),
}


def prologue_at(img, va):
    b = img.bytes(va, 4)
    for name, pat in PROLOGUES.items():
        if b.startswith(pat):
            return name, b.hex()
    return None, b.hex()


def cmd_verify(img, d):
    print("== the dispatch head, decoded from raw bytes ==")
    for va, hexs, text in d.trace:
        print(f"  {va:08x}  {hexs:<14}  {text}")
    print()
    print("== derived, not assumed ==")
    print(f"  opcode domain        {d.first_opcode:#04x} .. {d.last_opcode:#04x}"
          f"   ({d.n_opcodes} opcodes)   [from lea disp {d.lea_disp} + cmp bound {d.bound:#x}]")
    print(f"  byte index table     {d.byte_table_va:#x} .. {d.byte_table_end():#x}"
          f"   stride 1, {d.n_opcodes} entries   [{img.section_of(d.byte_table_va)}]")
    print(f"  arm target table     {d.arm_table_va:#x} .. {d.arm_table_end():#x}"
          f"   stride 4, {d.n_arms} entries   [{img.section_of(d.arm_table_va)}]")
    print(f"  index value range    {min(d.index)} .. {max(d.index)}  =>  n_arms = {d.n_arms}")
    print(f"  out-of-range `ja`    {d.ja_target:#x}")
    print()

    distinct = sorted(set(d.arms))
    print("== T3: are the arm targets distinct? ==")
    print(f"  {d.n_arms} table entries, {len(distinct)} distinct targets")
    dupes = {}
    for k, t in enumerate(d.arms):
        dupes.setdefault(t, []).append(k)
    for t, ks in sorted(dupes.items()):
        if len(ks) > 1:
            print(f"  DUPLICATE {t:#x} <- arm indices {ks}")
    print()

    print("== T2/T4: containment and the refusal ==")
    body_lo, body_hi = BODY_VA, d.arm_table_va
    inside = sum(1 for t in d.arms if body_lo <= t < body_hi)
    print(f"  targets inside [{body_lo:#x},{body_hi:#x})   {inside} of {d.n_arms}")
    ra = d.refusal_arms()
    print(f"  arm index(es) equal to the `ja` target        {ra}")
    oa = d.opcodes_of_arm()
    n_ref = sum(len(oa.get(k, [])) for k in ra)
    print(f"  in-range opcodes routed there                 {n_ref} of {d.n_opcodes}")
    print(f"  in-range opcodes handled                      {d.n_opcodes - n_ref} of {d.n_opcodes}")
    print(f"  real arms (excluding the refusal)             {d.n_arms - len(ra)} of {d.n_arms}")
    print()

    print("== T6: the byte table's extent, read TWO ways ==")
    nxt = d.byte_table_end() + 1
    name, hexs = prologue_at(img, nxt)
    print(f"  {d.n_opcodes} entries ends at {d.byte_table_end():#x}; next VA {nxt:#x} = {hexs}")
    print(f"  prologue at {nxt:#x}: {name or 'NOT A RECOGNISED PROLOGUE'}")
    # And the gap between the two tables: arm table ends where byte table starts?
    print(f"  arm table ends {d.arm_table_end():#x}; byte table starts {d.byte_table_va:#x}"
          f"  (gap {d.byte_table_va - d.arm_table_end() - 1} B)")
    print(f"  body {BODY_VA:#x} + 5080 = {BODY_VA + 5080:#x}; arm table at {d.arm_table_va:#x}")
    print()

    print("== the alternative T2 does not exclude ==")
    tail = list(img.bytes(nxt, 16))
    print(f"  16 bytes past the byte table: {' '.join(f'{x:02x}' for x in tail)}")
    over = [x for x in tail if x < d.n_arms]
    print(f"  of those, {len(over)} of 16 would be legal arm indices "
          f"(a longer table would need ALL of them to be)")


def cmd_arms(img, d):
    oa = d.opcodes_of_arm()
    ra = set(d.refusal_arms())
    print(f"{'arm':>3}  {'target':>10}  {'n':>3}  opcodes")
    for k in range(d.n_arms):
        ops = oa.get(k, [])
        tag = "  <-- REFUSAL" if k in ra else ""
        s = " ".join(f"{o:02x}" for o in ops)
        print(f"{k:>3}  {d.arms[k]:#010x}  {len(ops):>3}  {s}{tag}")


def cmd_tsv(img, d):
    oa = d.opcodes_of_arm()
    ra = set(d.refusal_arms())
    print("arm\ttarget\tn_opcodes\topcodes\trefusal\tclasses\tattrs")
    for k in range(d.n_arms):
        ops = oa.get(k, [])
        cls = " ".join(f"{img.u8(CLASS_TABLE_VA + o):02x}" for o in ops)
        att = " ".join(f"{img.u16(ATTR_TABLE_VA + 2 * o):04x}" for o in ops)
        print(f"{k}\t{d.arms[k]:#x}\t{len(ops)}\t{' '.join(f'{o:02x}' for o in ops)}"
              f"\t{'1' if k in ra else '0'}\t{cls}\t{att}")


def cmd_refuse(img, d):
    ra = d.refusal_arms()
    oa = d.opcodes_of_arm()
    ops = sorted(o for k in ra for o in oa.get(k, []))
    print(f"# {len(ops)} of {d.n_opcodes} in-range opcodes route to the C1001 arm(s) {ra}")
    print(f"# OUT OF SCOPE for any I1 slice by construction.")
    for i in range(0, len(ops), 16):
        print("  " + " ".join(f"{o:02x}" for o in ops[i:i + 16]))
    # runs, for readability
    runs, start, prev = [], None, None
    for o in ops:
        if start is None:
            start = prev = o
        elif o == prev + 1:
            prev = o
        else:
            runs.append((start, prev))
            start = prev = o
    if start is not None:
        runs.append((start, prev))
    print("# as runs:")
    print("  " + " ".join(f"{a:02x}" if a == b else f"{a:02x}-{b:02x}" for a, b in runs))


def cmd_cross(img, d):
    """Cross-check against dump_ilrecord.py's INDEPENDENT implementation."""
    import importlib.util
    import os
    here = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, here)
    spec = importlib.util.spec_from_file_location(
        "dump_ilrecord", os.path.join(here, "dump_ilrecord.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    other = m.Image(sys.argv[1])
    ok = True

    def chk(name, mine, theirs):
        nonlocal ok
        good = mine == theirs
        ok = ok and good
        print(f"  {'AGREE ' if good else 'DIFFER'}  {name}: mine={mine} theirs={theirs}")

    chk("byte table VA", hex(d.byte_table_va), hex(m.BYTE_TABLE_VA))
    chk("arm table VA", hex(d.arm_table_va), hex(m.JUMP_TABLE_VA))
    chk("n_opcodes", d.n_opcodes, m.BYTE_TABLE_LEN)
    chk("n_arms", d.n_arms, m.JUMP_TABLE_LEN)
    chk("first opcode", hex(d.first_opcode), hex(m.FIRST_OPCODE))
    chk("last opcode", hex(d.last_opcode), hex(m.LAST_OPCODE))
    chk("ja target", hex(d.ja_target), hex(m.OUT_OF_RANGE_ARM))
    def grab(va, n):
        o = other.off(va)
        return other.blob[o:o + n]
    theirs_idx = list(grab(m.BYTE_TABLE_VA, m.BYTE_TABLE_LEN))
    chk("index table bytes", d.index == theirs_idx, True)
    theirs_arms = [struct.unpack("<I", grab(m.JUMP_TABLE_VA + 4 * k, 4))[0]
                   for k in range(m.JUMP_TABLE_LEN)]
    chk("arm target words", d.arms == theirs_arms, True)
    print(f"\n  {'ALL AGREE' if ok else 'DISAGREEMENT -- that is the finding'}")


def main():
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    img = Image(sys.argv[1])
    d = Dispatch(img)
    cmd = sys.argv[2]
    {"--verify": cmd_verify, "--arms": cmd_arms, "--tsv": cmd_tsv,
     "--refuse": cmd_refuse, "--cross": cmd_cross}[cmd](img, d)


if __name__ == "__main__":
    main()

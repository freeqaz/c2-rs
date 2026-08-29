#!/usr/bin/env python3
"""Disassemble one address range of the pinned `c2.dll`, by VA.

Lane `w-fmadd`, wave 19.  Whitebox tooling (python, outside the std-only
`crates/` workspace per CLAUDE.md).

    python3 work/w-fmadd/dumparm.py --self-test
    python3 work/w-fmadd/dumparm.py 0x10bfa49a 160

The PE section table is PARSED, not assumed: `w-encarms`'s `arm_bodies.txt`
header hard-codes `base 0x10b00000` and an RVA->offset delta of 0xC00, and a
hard-coded delta is exactly the thing that is silently wrong when it is wrong.
`--self-test` re-derives a byte string this repo already published from a
different tool (`P_ENCODE.md` §3.2's refusal at `0x10bfa81d`) and fails if it
does not reproduce, so the mapping is CHECKED rather than asserted.
"""

import hashlib
import os
import struct
import sys

IMAGE = os.environ.get(
    "C2RS_C2_DLL", "compilers/X360/16.00.11886.00/c2.dll"
)
SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"


def load():
    data = open(IMAGE, "rb").read()
    got = hashlib.sha256(data).hexdigest()
    if got != SHA256:
        sys.exit(f"image sha256 mismatch: {got} != {SHA256}")
    e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]
    assert data[e_lfanew:e_lfanew + 4] == b"PE\0\0", "not a PE"
    coff = e_lfanew + 4
    nsec, = struct.unpack_from("<H", data, coff + 2)
    opt_size, = struct.unpack_from("<H", data, coff + 16)
    opt = coff + 20
    magic, = struct.unpack_from("<H", data, opt)
    assert magic == 0x10B, f"not PE32: {magic:#x}"
    image_base, = struct.unpack_from("<I", data, opt + 28)
    secs = []
    st = opt + opt_size
    for i in range(nsec):
        o = st + i * 40
        name = data[o:o + 8].rstrip(b"\0").decode()
        vsize, va, rawsize, rawptr = struct.unpack_from("<IIII", data, o + 8)
        secs.append((name, va, vsize, rawptr, rawsize))
    return data, image_base, secs


def va_to_off(va, image_base, secs):
    rva = va - image_base
    for name, sva, vsize, rawptr, rawsize in secs:
        if sva <= rva < sva + max(vsize, rawsize):
            return rawptr + (rva - sva), name
    raise KeyError(f"VA {va:#x} is in no section")


def disasm(va, n):
    data, base, secs = load()
    off, sec = va_to_off(va, base, secs)
    raw = data[off:off + n]
    print(f"# image sha256 {SHA256}")
    print(f"# image_base {base:#x}   section {sec}   VA {va:#x} -> file offset {off:#x}")
    print(f"# raw: {raw.hex()}")
    try:
        import capstone
    except ImportError:
        sys.exit("capstone not available")
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    md.syntax = capstone.CS_OPT_SYNTAX_INTEL
    for ins in md.disasm(raw, va):
        print(f"  {ins.address:08x}  {ins.bytes.hex():<20} {ins.mnemonic} {ins.op_str}")


def self_test():
    """Two independent anchors, both already published by OTHER tools."""
    data, base, secs = load()
    ok = True

    # Anchor 1 — `P_ENCODE.md` §3.2 / `w-encarms` §3.4: the ICE arm at
    # 0x10bfa81d is `mov edx,0x3d9 ; jmp 0x10bfa531`, i.e. ba d9030000 e9...
    off, _ = va_to_off(0x10BFA81D, base, secs)
    got = data[off:off + 5].hex()
    want = "bad9030000"
    print(f"  T1 ICE arm 0x10bfa81d = {got} (want {want})")
    ok &= got == want

    # Anchor 2 — `w-encarms`'s arm_bodies.txt first line for 0x10bf9f91.
    off, _ = va_to_off(0x10BF9F91, base, secs)
    got = data[off:off + 7].hex()
    want = "833d78e9c21000"
    print(f"  T2 arm 0x10bf9f91 = {got} (want {want})")
    ok &= got == want

    # Anchor 3 — the encode-form table at 0x10c39b18, stride 4: the form of
    # `fmadd` (opcode 0x0077) must be 24 (`ENCODE_OPCODES.txt:120`).
    off, _ = va_to_off(0x10C39B18 + 4 * 0x77, base, secs)
    got, = struct.unpack_from("<I", data, off)
    print(f"  T3 form[0x0077] = {got} (want 24)")
    ok &= got == 24

    # Anchor 4 — the base-word table at 0x10c3a578, stride 4, BIG-endian PPC
    # word: `fmadd` = fc00003a.
    off, _ = va_to_off(0x10C3A578 + 4 * 0x77, base, secs)
    got, = struct.unpack_from("<I", data, off)
    print(f"  T4 base[0x0077] = {got:08x} (want fc00003a)")
    ok &= got == 0xFC00003A

    # Anchor 5 — the arm jump table at 0x10bfae2d is indexed by `form - 1`
    # (`P_ENCODE.md` §3), so entry 23 must be 0x10bfa49a.
    off, _ = va_to_off(0x10BFAE2D + 4 * 23, base, secs)
    got, = struct.unpack_from("<I", data, off)
    print(f"  T5 armtable[form 24] = {got:#010x} (want 0x10bfa49a)")
    ok &= got == 0x10BFA49A

    print("SELF-TEST", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        sys.exit(self_test())
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    disasm(int(sys.argv[1], 0), int(sys.argv[2]) if len(sys.argv) > 2 else 128)

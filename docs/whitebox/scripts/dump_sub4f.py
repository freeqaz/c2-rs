#!/usr/bin/env python3
"""dump_sub4f.py -- read R9: the `.ex` `0x4F` sub-record grammar.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Reads the pinned image directly and refuses on a digest mismatch.

What this reads, address by address:

    0x10b3d7d7   the operand-class 0x0C arm: `varint16 -> [node+0x24]`,
                 then `call 0x10b9761e`.  0x4F is the ONLY opcode whose
                 class byte in 0x10b25e48 is 0x0C, so this table is the
                 0x4F sub-record table and nothing else.
    0x10b9761e   FUN_10b9761e, 606 B, p2pragma.c -- the sub-record reader
    0x10b9763d   `movsx eax, byte ptr [esi+0x24]` -- the sub-opcode, read
                 back as a SIGNED BYTE although the caller stored a word
    0x10b97641   `mov eax,[eax*8+0x10b26268]` -- the SOLE reader of the
                 descriptor table anywhere in the image
    0x10b26268   the descriptor table: 64 entries, stride 8,
                 `{const char *fmt, u32 unread}`.  `fmt` is a NUL-terminated
                 string of FIELD-TYPE CODES, not a width.
    0x10b97860   `mov al,[eax]` -- fetch the next field-type code
    0x10b9785a   `inc [ebp-0x7c]` -- advance the format pointer one byte
    0x10b9766c   the 13-way compare cascade over field-type codes (+1
                 default), i.e. the "~14-arm switch"
    0x10b33526   the ICE reporter, `(ecx = wide source path, edx = line)`,
                 tail `int3` -- so both refusal arms are FATAL
    0x10b163a0   L"e:\\bt\\278379\\vctools\\compiler\\be\\p2\\p2pragma.c"
    0x10b26468   L"...\\be\\common\\vlines.c" -- the next object, and the
                 independent fix on the table's extent: 0x10b26268 + 64*8

    the four scalar readers, all driven off the IL cursor 0x10c46310:
    0x10c1f8fc   BYTE   -- exactly 1 byte, no escape
    0x10c1f91b   VARU   -- 2 bytes, or 4 when bit 15 of the first u16 is set
    0x10c1f9a6   VI16   -- 1 byte, or 3 when that byte is exactly 0x80
    0x10c1f9e9   VI32   -- 1 byte, or 5 when that byte is exactly 0x80
    0x10c1fca9   STR    -- VI16 count `n`, then `n` raw bytes

    the two list siblings, both in p2pragma.c:
    0x10b97502   list of (VI16 tag, BYTE) until tag == -1
    0x10b97584   BYTE-led loop; ends on BYTE == 0x0f

Usage:
    python3 dump_sub4f.py <c2.dll> --selftest      # registered gate, exit 1 on fail
    python3 dump_sub4f.py <c2.dll> --table         # the 64 descriptor rows
    python3 dump_sub4f.py <c2.dll> --arms          # the field-type arms
    python3 dump_sub4f.py <c2.dll> --tsv           # machine-readable width table
    python3 dump_sub4f.py <c2.dll> --disasm        # the full 606-byte listing
    python3 dump_sub4f.py <c2.dll> --scan <f.ex>   # decode 0x4F records in a
                                                   # captured .ex (probe engine)

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md section 0); the script verifies the digest and refuses
otherwise.

INSTRUMENT LIMITATION, stated so a green is not over-read: `--scan` locates
records by searching for the byte 0x4F rather than by parsing the whole `.ex`
grammar, which nobody has. It is therefore a SUPERSET detector -- it can
report a candidate that is really payload bytes of some other record, and it
cannot miss a real one. Every claim graded off `--scan` in
`WB_SUB4F_FINDINGS.md` is a claim of the form "every injected ground-truth
value was found", which a superset detector can still fail.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dump_opcode_tables import Image, PINNED_SHA256          # noqa: E402

try:
    import capstone
except ImportError:                                          # pragma: no cover
    capstone = None

READER_VA = 0x10B9761E
READER_LEN = 606                       # ref/FUNCS.tsv:2084 -- self-tested
TABLE_VA = 0x10B26268
TABLE_STRIDE = 8
TABLE_LEN = 64                         # fixed two ways; see --selftest
TABLE_END_VA = TABLE_VA + TABLE_STRIDE * TABLE_LEN      # 0x10b26468
SUBOP_LOAD_VA = 0x10B9763D             # movsx eax, byte ptr [esi+0x24]
TABLE_LOAD_VA = 0x10B97641             # mov eax,[eax*8+0x10b26268]
CASCADE_VA = 0x10B9766C
FETCH_VA = 0x10B97860                  # mov al, byte ptr [eax]
ADVANCE_VA = 0x10B9785A                # inc dword ptr [ebp-0x7c]
CLASS_TABLE_VA = 0x10B25E48            # operand-format class, board #1591
CLASS_0C = 0x0C
OPCODE_4F = 0x4F
CLASS_ARM_VA = 0x10B3D7D7
ICE_VA = 0x10B33526
P2PRAGMA_PATH_VA = 0x10B163A0
IL_CURSOR_VA = 0x10C46310

BYTE_VA = 0x10C1F8FC
VARU_VA = 0x10C1F91B
VI16_VA = 0x10C1F9A6
VI32_VA = 0x10C1F9E9
STR_VA = 0x10C1FCA9
LIST_TAGBYTE_VA = 0x10B97502
LIST_BYTELED_VA = 0x10B97584

# The 13 field-type codes the cascade at 0x10b9766c decides, plus the default.
# `arm` is the VA the cascade jumps to; `reads` is the payload this lane read
# out of that arm, in stream order. Every row here is [R] -- read from the
# disassembly and not, by itself, checked against any obj.
ARMS = [
    # code, arm VA,     reads,              effect
    (0x0B, 0x10B97706, ["BYTE"],            "byte -> 0x10b97d47, which reads FURTHER stream (DEFER)"),
    (0x0C, 0x10B976C7, ["STR"],             "counted byte string via 0x10c1fca9, cap 0x1020, buf 0x10c6b040"),
    (0x0D, 0x10B9786F, [],                  "ICE p2pragma.c:88 -- a dedicated NOT-IMPLEMENTED arm"),
    (0x0E, 0x10B976B9, ["VI16"],            "u16 -> node+0x10 (low half only)"),
    (0x14, 0x10B976A8, ["VI32"],            "i32 sign-extended -> node+0x10/+0x14"),
    (0x15, 0x10B97718, ["VI16"],            "i16 -> cwde -> cdq -> node+0x10/+0x14"),
    (0x16, 0x10B977FD, ["VARU"],            "-> DAT_10c2eaa0 AND DAT_10c2edd0, the TU label counter"),
    (0x17, 0x10B977D4, ["LIST_BYTELED"],    "0x10b97584 into a 0x270-cap buffer, then alloc+copy"),
    (0x1A, 0x10B97762, ["VI16", "LOOP"],    "count n -> node+8; then n * (VARU, VI16) into two arrays"),
    (0x1D, 0x10B9774F, ["GATE"],            "if DAT_10c2eb5c != 0 then as code 0x15, else ICE:160"),
    (0x1E, 0x10B9773D, ["LIST_TAGBYTE"],    "0x10b97502 into a 0x270-cap buffer, then alloc+copy"),
    (0x6C, 0x10B9780E, ["GATE"],            "'l': DAT_10c2eb4c != 0 ? VI32 : VI16 zero-extended; -> DAT_10c2e2e4"),
    (0x73, 0x10B9783D, ["VARU"],            "'s': symbol token -> 0x10b9880d; node opcode := 0x56"),
]
DEFAULT_ARM_VA = 0x10B97758            # mov edx,0xa0 -> ICE p2pragma.c:160
HANDLED_CODES = {c for c, _, _, _ in ARMS}

# Payload width of each scalar reader, as a function of the bytes at the
# cursor. Returns (nbytes, value) or (None, None) when the buffer is short.
def read_byte(b, p):
    if p >= len(b):
        return None, None
    return 1, b[p]


def read_varu(b, p):
    """0x10c1f91b: 2 bytes, or 4 when bit 15 of the first u16 is set."""
    if p + 2 > len(b):
        return None, None
    lo = b[p] | (b[p + 1] << 8)
    if not (lo & 0x8000):
        return 2, lo
    if p + 4 > len(b):
        return None, None
    hi = b[p + 2] | (b[p + 3] << 8)
    return 4, (lo & 0x7FFF) | (hi >> 1 << 15)


def read_vi16(b, p):
    """0x10c1f9a6: 1 byte sign-extended, or 0x80 + 2 more."""
    if p >= len(b):
        return None, None
    if b[p] != 0x80:
        v = b[p]
        return 1, v - 256 if v & 0x80 else v
    if p + 3 > len(b):
        return None, None
    v = b[p + 1] | (b[p + 2] << 8)
    return 3, v - 0x10000 if v & 0x8000 else v


def read_vi32(b, p):
    """0x10c1f9e9: 1 byte sign-extended, or 0x80 + 4 more."""
    if p >= len(b):
        return None, None
    if b[p] != 0x80:
        v = b[p]
        return 1, v - 256 if v & 0x80 else v
    if p + 5 > len(b):
        return None, None
    v = int.from_bytes(b[p + 1:p + 5], "little", signed=True)
    return 5, v


def read_str(b, p):
    n, cnt = read_vi16(b, p)
    if n is None or cnt is None or cnt < 0 or p + n + cnt > len(b):
        return None, None
    return n + cnt, bytes(b[p + n:p + n + cnt])


READERS = {
    "BYTE": read_byte, "VARU": read_varu, "VI16": read_vi16,
    "VI32": read_vi32, "STR": read_str,
}


def load(path):
    img = Image(path)
    if img.digest != PINNED_SHA256:
        sys.exit("image digest %s != pinned %s" % (img.digest, PINNED_SHA256))
    return img


def wstr(img, va, cap=240):
    o = img.off(va)
    if o is None:
        return None
    out = []
    for i in range(0, cap - 1, 2):
        c = img.blob[o + i] | (img.blob[o + i + 1] << 8)
        if c == 0:
            break
        out.append(chr(c) if 32 <= c < 127 else "?")
    return "".join(out)


def fmt_string(img, va, cap=64):
    """The NUL-terminated field-type-code string a descriptor points at."""
    o = img.off(va)
    if o is None:
        return None
    end = img.blob.find(b"\0", o, o + cap)
    if end < 0:
        return None
    return bytes(img.blob[o:end])


def table(img):
    """[(index, fmt_ptr, second_dword, codes_or_None)] over all 64 entries."""
    rows = []
    for i in range(TABLE_LEN):
        va = TABLE_VA + i * TABLE_STRIDE
        d0, d1 = img.u32(va), img.u32(va + 4)
        codes = fmt_string(img, d0) if d0 else None
        rows.append((i, d0 or 0, d1 or 0, codes))
    return rows


def selftest(img):
    """The gate registered in WB_SUB4F_PREREG.md section 5.4, plus the checks
    that fix the table's extent. Any failure voids the lane."""
    ok = True

    def chk(name, cond, detail=""):
        nonlocal ok
        print("  %-52s %s %s" % (name, "PASS" if cond else "FAIL", detail))
        ok = ok and bool(cond)

    print("dump_sub4f self-test (image sha256 %s):" % img.digest[:16])
    chk("digest matches the pin", img.digest == PINNED_SHA256)

    # (1) registered: the two facts this lane did not derive.
    o = img.off(TABLE_LOAD_VA)
    want = bytes.fromhex("8b04c5") + TABLE_VA.to_bytes(4, "little")
    chk("0x10b97641 is `mov eax,[eax*8+0x10b26268]`",
        bytes(img.blob[o:o + 7]) == want, img.blob[o:o + 7].hex())
    o = img.off(SUBOP_LOAD_VA)
    chk("0x10b9763d is `movsx eax, byte ptr [esi+0x24]`",
        bytes(img.blob[o:o + 4]) == bytes.fromhex("0fbe4624"),
        img.blob[o:o + 4].hex())

    # (2) 0x4F is the ONLY class-0x0C opcode -- this table is 0x4F's alone.
    o = img.off(CLASS_TABLE_VA)
    cls = list(img.blob[o:o + 0xC0])
    c0c = [i for i, c in enumerate(cls) if c == CLASS_0C]
    chk("0x4F is the only operand-class-0x0C opcode", c0c == [OPCODE_4F],
        "class-0x0C opcodes = %s" % [hex(x) for x in c0c])

    # (3) the table's extent, fixed two independent ways.
    nxt = wstr(img, TABLE_END_VA)
    chk("base + 64*8 begins the next object (vlines.c path)",
        bool(nxt) and nxt.endswith("vlines.c"), repr(nxt))
    tail = table(img)[0x34:]
    chk("entries 0x34..0x3f carry no format pointer",
        all(r[1] == 0 for r in tail),
        "%d of %d tail rows have a fmt ptr"
        % (sum(1 for r in tail if r[1]), len(tail)))

    # (4) every fmt pointer lands in the 80-byte pool BELOW the table, or in
    #     .text elsewhere -- never inside the table itself (the stride trap:
    #     a wrong stride would make entries point at each other).
    bad = [hex(r[1]) for r in table(img)
           if r[1] and TABLE_VA <= r[1] < TABLE_END_VA]
    chk("no descriptor points inside the table (stride check)", not bad, str(bad))

    # (5) the containing TU, from the ICE arm's own argument.
    p = wstr(img, P2PRAGMA_PATH_VA)
    chk("the ICE path string is p2pragma.c",
        bool(p) and p.endswith("p2pragma.c"), repr(p))

    # (6) the reader's length, from ref/FUNCS.tsv:2084 -- the last byte of the
    #     606-byte extent must be the final `call 0x10b33526` of the ICE tail.
    o = img.off(READER_VA + READER_LEN - 5)
    disp = int.from_bytes(img.blob[o + 1:o + 5], "little", signed=True)
    chk("the 606-byte extent ends on `call 0x10b33526`",
        img.blob[o] == 0xE8 and (READER_VA + READER_LEN + disp) == ICE_VA,
        "target=0x%08x" % (READER_VA + READER_LEN + disp))

    # (7) the sole-reader claim: the table's base VA appears exactly once as
    #     an immediate anywhere in the image.
    n = img.blob.count(TABLE_VA.to_bytes(4, "little"))
    chk("0x10b26268 appears exactly once image-wide", n == 1, "count=%d" % n)

    # (8) P_SUB4F.md section 6.1's load-bearing claim: of the 13 handled
    #     field-type codes, exactly one is selected by NO descriptor, and it
    #     is 0x16 -- the arm holding the label-seed install R3 cited.
    used = set()
    for _i, _d0, _d1, codes in table(img):
        used.update(codes or b"")
    orphan = HANDLED_CODES - used
    chk("0x16 is the unique handled-but-unselected field code",
        orphan == {0x16},
        "orphans=%s" % sorted("%02x" % c for c in orphan))

    print("  => %s" % ("SELFTEST PASS" if ok else "SELFTEST FAIL"))
    return ok


def show_table(img):
    print("# descriptor table 0x%08x, %d entries, stride %d, ends 0x%08x"
          % (TABLE_VA, TABLE_LEN, TABLE_STRIDE, TABLE_END_VA))
    print("# entry = { const char *fmt ; u32 (READ NOWHERE IN THE IMAGE) }")
    print("%-6s %-10s %-6s %-14s %s" % ("sub", "fmt ptr", "d1", "codes", "handled?"))
    for i, d0, d1, codes in table(img):
        if not d0 and not d1:
            continue
        if codes is None:
            cs, hs = "(NULL -- no payload)" if not d0 else "(?)", "yes: 0 fields"
        elif codes == b"":
            cs, hs = "(empty string)", "yes: 0 fields"
        else:
            cs = " ".join("%02x" % c for c in codes)
            miss = [c for c in codes if c not in HANDLED_CODES]
            hs = "yes" if not miss else "NO -> ICE:160 on %s" % (
                " ".join("%02x" % c for c in miss))
        print("0x%02x   0x%08x %-6d %-14s %s" % (i, d0, d1, cs, hs))


def show_arms(img):
    print("# the field-type cascade at 0x%08x -- 13 codes + 1 default" % CASCADE_VA)
    print("%-6s %-12s %-16s %s" % ("code", "arm VA", "reads", "effect"))
    for code, va, reads, eff in ARMS:
        ch = chr(code) if 32 <= code < 127 else "."
        print("0x%02x %s 0x%08x   %-16s %s" % (code, ch, va, ",".join(reads) or "-", eff))
    print("dflt   0x%08x   %-16s %s" % (DEFAULT_ARM_VA, "-",
                                        "ICE p2pragma.c:160 (fatal, int3)"))


def show_tsv(img):
    print("sub\tfmt_codes\tfields\tmin_bytes\tstatus")
    for i, d0, d1, codes in table(img):
        if not d0:
            if d1:
                print("%d\t\t0\t0\tNULL-no-payload" % i)
            continue
        cs = " ".join("%02x" % c for c in (codes or b""))
        miss = [c for c in (codes or b"") if c not in HANDLED_CODES]
        status = "ok" if not miss else "ICE160"
        lo = 0
        for c in (codes or b""):
            lo += {0x0B: 1, 0x0C: 1, 0x0E: 1, 0x14: 1, 0x15: 1,
                   0x16: 2, 0x1D: 1, 0x6C: 1, 0x73: 2}.get(c, 0)
        print("%d\t%s\t%d\t%d\t%s" % (i, cs, len(codes or b""), lo, status))


def show_disasm(img):
    if capstone is None:
        sys.exit("capstone required for --disasm")
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    md.detail = True
    o = img.off(READER_VA)
    print("# FUN_%08x, %d bytes, p2pragma.c" % (READER_VA, READER_LEN))
    for ins in md.disasm(bytes(img.blob[o:o + READER_LEN]), READER_VA):
        print("%08x  %-22s %s %s" % (ins.address, ins.bytes.hex(),
                                     ins.mnemonic, ins.op_str))


def scan_ex(img, path, gate_l="VI32"):
    """Superset scan of a captured .ex for 0x4F records.

    For every 0x4F byte, look up the sub-opcode's format string in the pinned
    table and, where every field-type code in it has a static width, report the
    record's total byte length and the decoded field values.
    """
    body = open(path, "rb").read()
    rows = table(img)
    fmt_by_sub = {i: codes for i, _d0, _d1, codes in rows}
    have_ptr = {i: bool(d0) for i, d0, _d1, _c in rows}
    print("# %s (%d bytes)" % (path, len(body)))
    print("%-8s %-6s %-8s %-7s %s" % ("off", "sub", "codes", "len", "values"))
    hist = {}
    p = 0
    while True:
        p = body.find(b"\x4f", p)
        if p < 0:
            break
        q = p + 1
        n, sub = read_vi16(body, q)          # the caller's varint16 at [+0x24]
        if n is None:
            break
        q += n
        idx = sub & 0xFF
        idx = idx - 256 if idx & 0x80 else idx      # the reader's movsx
        hist[sub] = hist.get(sub, 0) + 1
        if idx < 0 or idx >= TABLE_LEN:
            print("%08x 0x%02x   %-8s %-7s OUT-OF-TABLE (movsx index %d)"
                  % (p, sub & 0xFF, "-", "-", idx))
            p += 1
            continue
        codes = fmt_by_sub.get(idx)
        if not have_ptr[idx] or codes is None or codes == b"":
            print("%08x 0x%02x   %-8s %-7d %s" % (p, sub & 0xFF, "(none)", q - p,
                                                  "no payload"))
            p += 1
            continue
        vals, ok = [], True
        for c in codes:
            kind = {0x0B: "BYTE", 0x0C: "STR", 0x0E: "VI16", 0x14: "VI32",
                    0x15: "VI16", 0x16: "VARU", 0x73: "VARU"}.get(c)
            if c == 0x6C:
                kind = gate_l
            if kind is None:
                ok = False
                break
            w, v = READERS[kind](body, q)
            if w is None:
                ok = False
                break
            q += w
            vals.append(v)
        cs = " ".join("%02x" % c for c in codes)
        print("%08x 0x%02x   %-8s %-7s %s" % (
            p, sub & 0xFF, cs, (q - p) if ok else "?",
            vals if ok else "UNBOUNDED/short (variable-width field)"))
        p += 1
    print("# sub-opcode histogram: %s"
          % {("0x%02x" % k): v for k, v in sorted(hist.items())})


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    img = load(sys.argv[1])
    cmd = sys.argv[2]
    if cmd == "--selftest":
        sys.exit(0 if selftest(img) else 1)
    elif cmd == "--table":
        show_table(img)
    elif cmd == "--arms":
        show_arms(img)
    elif cmd == "--tsv":
        show_tsv(img)
    elif cmd == "--disasm":
        show_disasm(img)
    elif cmd == "--scan":
        if len(sys.argv) < 4:
            sys.exit(__doc__)
        for f in sys.argv[3:]:
            scan_ex(img, f)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()

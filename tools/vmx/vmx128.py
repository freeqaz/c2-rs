#!/usr/bin/env python3
"""vmx128.py -- a VMX128 / opcode-4-5-6 PowerPC decoder for Xbox 360 objects.

Pure stdlib. Tooling, NOT part of the std-only Rust workspace and never a
correctness gate -- the sole judge of the port stays the real `c2.dll` under
wibo plus a byte-exact obj compare (CLAUDE.md).

Tables come from `vmx128_isa.py` (generated; see its header for provenance).
This module is only the bit-field machinery and the operand printer.

WHY THIS EXISTS, in one paragraph
---------------------------------
`llvm-mc -triple=powerpc` decodes VMX128 words into *plausible legal modern
PowerPC instructions with no diagnostic*: `18 00 07 10` is `vrlimi128` and LLVM
calls it `lxvp 0, 1808(0)`. Absence of an error is not evidence of a correct
decode -- and "absence read as success" is this project's most-repeated failure
shape. So this decoder never reports a decode it has not positively matched
against a table entry, and `vmxcheck.py` never reports a *verified* decode it
has not positively matched against the real compiler's own `/FAcs` listing.

BIT NUMBERING
-------------
IBM PowerPC convention throughout: bit 0 is the MOST significant bit of the
32-bit word, bit 31 the least. A field spec `"28..30,6..11"` is a comma list of
half-open ranges concatenated most-significant-part-first, so bits 28-29 supply
the top two bits of `VDS128` and bits 6-10 the bottom five:

    VDS128 = ((w >> 2) & 3) << 5 | ((w >> 21) & 31)

THE FOUR SPLIT-REGISTER FIELDS -- the actually interesting part of VMX128
-------------------------------------------------------------------------
VMX128 has 128 vector registers, so a register number needs 7 bits, but the
AltiVec encodings it extends left only 5 contiguous bits per operand. The extra
bits were scavenged from wherever the base encoding had room, and they are NOT
in the same place for each operand:

    VDS128   bits 28,29  ++ bits 6..10     (dest, also a source in the `c` forms)
    VA128    bit 21, bit 26 ++ bits 11..15 (three disjoint pieces)
    VB128    bits 30,31  ++ bits 16..20
    VC128    bits 23..25                   (3 bits only: vC is 0..7 here)

`VA128`'s two high bits are single bits at 21 and 26, six bit positions apart
and both inside what a stock PowerPC decoder reads as opcode-extension space.
That is the mechanism behind every silent mis-decode in
`docs/VMX128_DECODE.md`: a stock decoder reads those register bits as opcode
bits, lands in a different table row, and prints a different instruction with
no complaint.
"""
import sys

try:
    from . import vmx128_isa as ISA          # package import
except ImportError:                          # run as a script
    sys.path.insert(0, __file__.rsplit("/", 1)[0])
    import vmx128_isa as ISA


def parse_spec(spec):
    """"28..30,6..11" -> [(28, 30), (6, 11)]  (IBM numbering, half-open)."""
    out = []
    for part in spec.split(","):
        if ".." in part:
            a, b = part.split("..")
            out.append((int(a), int(b)))
        else:
            a = int(part)
            out.append((a, a + 1))
    return out


_SPEC_CACHE = {}


def extract(word, spec):
    """Pull a (possibly split) field out of `word`, MSB part first."""
    parts = _SPEC_CACHE.get(spec)
    if parts is None:
        parts = _SPEC_CACHE[spec] = parse_spec(spec)
    v = 0
    for lo, hi in parts:
        width = hi - lo
        v = (v << width) | ((word >> (32 - hi)) & ((1 << width) - 1))
    return v


def field(word, name):
    """Decode a named field. Returns (value, arg-kind)."""
    spec, kind, signed = ISA.FIELDS[name]
    v = extract(word, spec)
    if signed:
        width = sum(hi - lo for lo, hi in parse_spec(spec))
        if v & (1 << (width - 1)):
            v -= 1 << width
    return v, kind


_PREFIX = {"GPR": "r", "FPR": "fr", "VR": "vr", "CRBit": "crb",
           "CRField": "cr", "SR": "sr", "SPR": "spr", "GQR": "gqr"}


def render_operand(word, name):
    v, kind = field(word, name)
    return "%s%d" % (_PREFIX.get(kind, ""), v)


class Decoded(object):
    __slots__ = ("word", "mnemonic", "operands", "table", "args", "mask",
                 "pattern")

    def __init__(self, word, mnemonic, operands, table, args, mask, pattern):
        self.word = word
        self.mnemonic = mnemonic
        self.operands = operands
        self.table = table          # "VMX128" | "opcode456" (AltiVec/base)
        self.args = args
        self.mask = mask
        self.pattern = pattern

    def text(self):
        return "%s %s" % (self.mnemonic, ",".join(self.operands))

    def __repr__(self):
        return "<%s %08x %s>" % (self.table, self.word, self.text())


# ---------------------------------------------------------------------------
# Operand PRINT order.
#
# `vmx128_isa.py` carries the isa.yaml `args` list, and for every mnemonic this
# lane could verify against the real `/FAcs` listing that list is ALSO the order
# Microsoft prints -- including the non-obvious `vmaddfp vD,vA,vC,vB`.
#
# The exception, verified: the VMX128 "combined" forms take their destination
# as a source too, and cl.exe prints that operand explicitly. isa.yaml lists
# three register args; cl prints four. Only `vmaddcfp128` is in the workload,
# so only `vmaddcfp128` is VERIFIED here -- the rest of this dict is INFERRED
# by analogy and is reported as unverified by `vmxcheck.py --coverage`.
# ---------------------------------------------------------------------------
# ---------------------------------------------------------------------------
# Four rows where Microsoft's own listing spells the mnemonic differently from
# powerpc-rs. MEASURED, not chosen: `vmxcheck.py` reported these as MISMATCH
# against `cl /FAcs` on `work/w-vmx/probe2`, on the same encoding, four times
# each, and the listing is the oracle. powerpc-rs named them after the AltiVec
# base ops (`vctsxs` = convert to signed fixed-point saturate); Microsoft names
# them after the source and destination formats.
#
# This is the one place the generated table is overridden, and it is overridden
# because a run against the real compiler said so -- which is the entire point
# of grading a community table instead of trusting it.
# ---------------------------------------------------------------------------
MS_MNEMONIC = {
    "vctsxs128": "vcfpsxws128",   # float -> signed word
    "vctuxs128": "vcfpuxws128",   # float -> unsigned word
    "vcfsx128":  "vcsxwfp128",    # signed word -> float
    "vcfux128":  "vcuxwfp128",    # unsigned word -> float
}

PRINT_ARGS = {
    # mnemonic          -> print order. "D" = the VDS128 operand printed again
    # (these ops read their destination). VERIFIED against cl's listing.
    "vmaddcfp128":  ("VDS128", "VA128", "D", "VB128"),
    # vupkd3d128's third operand is NOT `vuimm`. isa.yaml says vuimm (bits
    # 11..15); cl's listing prints D3DType (bits 11..13). On 0x1be40ff4
    # vuimm = 4 and cl says `1<NORMSHORT2>`, i.e. 1 = D3DType. Table
    # correction, found by the oracle, verified on all 8 type values.
    "vupkd3d128":   ("VDS128", "VB128", "D3DType"),
    "vpkd3d128":    ("VDS128", "VB128", "D3DType", "VMASK", "Zimm"),
    # vspltisw128 prints two operands, not three: the VB128 field is not an
    # operand of this form. Verified: `vspltisw128 vr63,3` for 0x1be30774.
    "vspltisw128":  ("VDS128", "vsimm"),
}

# cl.exe annotates the D3DType immediate with the pack format's name. All
# eight harvested from a probe over D3DType 0..7 (work/w-vmx/probe3).
D3D_TYPE_NAME = {
    0: "D3DCOLOR", 1: "NORMSHORT2", 2: "NORMPACKED32", 3: "FLOAT16-2",
    4: "NORMSHORT4", 5: "FLOAT16-4", 6: "NORMPACKED64", 7: "UNKNOWN",
}


def _render(word, mnemonic, args):
    order = PRINT_ARGS.get(mnemonic, args)
    out = []
    for a in order:
        if a == "D":
            out.append(render_operand(word, "VDS128"))
        elif a == "D3DType":
            v, _k = field(word, "D3DType")
            out.append("%d<%s>" % (v, D3D_TYPE_NAME.get(v, "UNKNOWN")))
        else:
            out.append(render_operand(word, a))
    return out


# The Xenon core is base PowerPC + AltiVec + VMX128. It is NOT a Gekko /
# Broadway, and `isa.yaml` also carries that chip's PairedSingles extension in
# the SAME primary opcode 4. That overlap is total for several rows -- `ps_sel`
# and `vmaddfp` are literally the same 32 bits -- so a decoder that leaves
# PairedSingles enabled will happily print `ps_sel fr10,fr10,fr11,fr9` for the
# `vmaddfp vr10,vr10,vr11,vr9` that is really in a dc3 object. That was
# observed here on the first run of this file, on a real workload word
# (0x114a4aee), and it is the same failure shape as the LLVM one: a plausible
# legal instruction, no diagnostic, wrong. Hence an explicit profile.
XENON_PROFILE = frozenset(["base", "AltiVec"])


def decode(word, allow_base=True, profile=XENON_PROFILE):
    """Decode one big-endian instruction word.

    Returns a `Decoded`, or None if no table row matches. None means
    UNRECOGNIZED and must never be reported as a successful decode.

    VMX128 is tried first: on this target a word whose primary opcode is 4, 5
    or 6 and which matches a VMX128 row IS a VMX128 instruction, because the
    Xenon core implements VMX128 in place of the colliding base encodings.
    `collisions()` enumerates exactly which rows the two tables share.
    """
    for name, mask, pattern, args in ISA.VMX128:
        if (word & mask) == pattern:
            return Decoded(word, MS_MNEMONIC.get(name, name),
                           _render(word, name, args), "VMX128",
                           args, mask, pattern)
    if allow_base:
        return decode_base_only(word, profile)
    return None


def decode_base_only(word, profile=XENON_PROFILE):
    """What a decoder that does NOT know VMX128 would say -- our own model of
    the stock-PowerPC answer. Used only to cross-check `llvm-mc`; the measured
    LLVM answer, not this, is what `docs/VMX128_DECODE.md` reports."""
    for name, mask, pattern, src, args in ISA.OPCODE456_OTHER:
        if src not in profile:
            continue
        if (word & mask) == pattern:
            return Decoded(word, name, _render(word, name, args), src,
                           args, mask, pattern)
    return None


def encodings_overlap(mask_a, pat_a, mask_b, pat_b):
    """Do two (mask, pattern) rows admit a common 32-bit word?"""
    return ((pat_a ^ pat_b) & mask_a & mask_b) == 0


def collisions():
    """Every (VMX128 row, non-VMX128 opcode-4/5/6 row) pair that a single word
    can satisfy simultaneously. This is exact and needs no sampling: two rows
    share a word iff their patterns agree on every bit both masks constrain."""
    out = []
    for vname, vmask, vpat, _a in ISA.VMX128:
        for oname, omask, opat, src, _b in ISA.OPCODE456_OTHER:
            if encodings_overlap(vmask, vpat, omask, opat):
                out.append((vname, oname, src, vmask & omask))
    return out


def sample_word(mask, pattern, vd=0, va=0, vb=0, vc=0, ra=0, rb=0, imm=0):
    """Build a concrete word for a row: pattern, then stuff the register
    fields, then re-assert the pattern so no register bit can corrupt it.

    The re-assert is the point. VMX128 register bits live INSIDE opcode-
    extension space, so a naive `pattern | fields` silently produces a
    different instruction. `assert (w & mask) == pattern` below is a positive
    check that the word we hand to llvm-mc really is the row we meant.
    """
    w = pattern
    free = ~mask & 0xFFFFFFFF

    def put(name, value):
        nonlocal w
        spec, _k, _s = ISA.FIELDS[name]
        parts = parse_spec(spec)
        total = sum(hi - lo for lo, hi in parts)
        bitsleft = total
        for lo, hi in parts:
            width = hi - lo
            bitsleft -= width
            chunk = (value >> bitsleft) & ((1 << width) - 1)
            shift = 32 - hi
            fmask = ((1 << width) - 1) << shift
            if fmask & ~free:
                continue          # this piece is opcode; leave the pattern
            w = (w & ~fmask) | ((chunk << shift) & fmask)

    put("VDS128", vd)
    put("VA128", va)
    put("VB128", vb)
    put("VC128", vc)
    put("rA", ra)
    put("rB", rb)
    put("vuimm", imm)
    assert (w & mask) == pattern, "sample_word corrupted the opcode"
    return w & 0xFFFFFFFF


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("usage: vmx128.py <hexword> [...]   # e.g. vmx128.py 102320c3")
        sys.exit(2)
    for h in sys.argv[1:]:
        w = int(h, 16)
        d = decode(w)
        print("%08x  %s" % (w, d.text() + "   [" + d.table + "]" if d
                            else "UNRECOGNIZED"))

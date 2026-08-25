#!/usr/bin/env python3
"""dump_relsite.py — decode c2's IL-opcode -> relation-code site from RAW IMAGE BYTES.

Lane `w-relsite`. Reads the dispatch chain at `FUN_10bbffbb` out of the pinned
`c2.dll` and prints the opcode -> relation-code table it performs, together
with the routing arm that reaches it.

**Nothing here is read from `data.tsv`, `ADDR.tsv`, `FUNCS.tsv`, a prior
lane's artifact or a findings file** — prereg `WB_RELSITE_PREREG.md` control
M1, which is `w-relread`'s D5 registered in advance. The only inputs are the
image's own bytes and its PE section headers.

THE FENCE: the sha256 is verified in `__init__`, **before any structural
parse**, so a corrupted or substituted image cannot reach the PE-header logic.
Watched refusing on a truncated image, a same-size one-bit-flipped image, and
an unreadable path (prereg M3).

    usage: dump_relsite.py <path-to-c2.dll> [--site VA] [--arm VA] [--max-arms N]

Tooling only — outside the workspace's std-only Rust constraint, same status
as `scripts/plot_perf.py` and `dump_relnames.py`.
"""

import hashlib
import struct
import sys

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

# Defaults are this lane's subject. Both are overridable so the decode can be
# pointed elsewhere and shown to depend on the image rather than on the caller.
DEFAULT_SITE = 0x10BBFFBB  # FUN_10bbffbb — the converter
DEFAULT_ARM = 0x10BC38A1  # arm 7 of the IL-record dispatch FUN_10bc2d7a

# The enum, for LABELLING ONLY. Read by `w-relread` from the 19-entry array at
# 0x10c38690 and confirmed by six consumers (board #3518). This script does not
# re-derive it and does not need it: the table it prints is numeric.
REL_NAMES = {
    0: "ILLEGAL", 1: "EQ", 2: "NE", 3: "LT", 4: "GT", 5: "LE", 6: "GE",
    7: "ULT", 8: "UGT", 9: "ULE", 10: "UGE", 11: "SO", 12: "NSO", 13: "S",
    14: "NS", 15: "VALL", 16: "NVALL", 17: "VNONE", 18: "NVNONE",
}


class FenceRefused(Exception):
    pass


class Image:
    def __init__(self, path):
        # THE FENCE, before any parse.
        try:
            with open(path, "rb") as fh:
                self.raw = fh.read()
        except OSError as exc:
            raise FenceRefused(f"cannot read it: {exc}") from None
        digest = hashlib.sha256(self.raw).hexdigest()
        if digest != PINNED_SHA256:
            raise FenceRefused(
                f"sha256 is {digest}, pinned is {PINNED_SHA256}. Nothing was read."
            )
        self.digest = digest
        self.sections = self._sections()

    def _sections(self):
        if self.raw[:2] != b"MZ":
            raise FenceRefused("not an MZ image")
        pe = struct.unpack_from("<I", self.raw, 0x3C)[0]
        if self.raw[pe:pe + 4] != b"PE\0\0":
            raise FenceRefused("no PE signature")
        nsec = struct.unpack_from("<H", self.raw, pe + 6)[0]
        optsz = struct.unpack_from("<H", self.raw, pe + 20)[0]
        base = struct.unpack_from("<I", self.raw, pe + 24 + 28)[0]
        tbl = pe + 24 + optsz
        out = []
        for i in range(nsec):
            e = tbl + i * 40
            name = self.raw[e:e + 8].rstrip(b"\0").decode("latin1")
            vsize, va, rawsize, rawptr = struct.unpack_from("<IIII", self.raw, e + 8)
            out.append((name, base + va, max(vsize, rawsize), rawptr))
        self.image_base = base
        return out

    def off(self, va):
        for name, sva, size, rawptr in self.sections:
            if sva <= va < sva + size:
                return rawptr + (va - sva)
        raise FenceRefused(f"VA {va:#x} is in no section")

    def at(self, va, n):
        o = self.off(va)
        return self.raw[o:o + n]


def decode_chain(img, site, max_arms=32):
    """Walk the dispatch chain at `site` MECHANICALLY.

    The walk stops on its own terms — on the fallthrough after the last
    conditional — never on `max_arms`, which exists only so a caller can be
    shown that the answer is the image's and not the caller's parameter
    (`w-relread` D2 / board #3483: reproducibility is not attribution).
    """
    # Step 1: find the `sub eax, imm8` inside the prologue. Bounded, and the
    # bound is reported if it is what stopped us.
    va = None
    for probe in range(site, site + 64):
        if img.at(probe, 2) == b"\x83\xe8":
            va = probe
            break
    if va is None:
        return None, [], None, "no `sub eax,imm8` within 64 B of the site", []
    first = img.at(va, 3)[2]
    cur = first
    va += 3

    order = []        # (opcode, target VA) in chain order, then the fallthrough
    skipped = []      # flag-preserving instructions stepped over, reported
    guard = 0
    while guard < 512:
        guard += 1
        b = img.at(va, 8)
        if b[0] == 0x48:                       # dec eax
            cur += 1
            va += 1
            continue
        if b[0] == 0x74:                       # je rel8
            tgt = va + 2 + (b[1] if b[1] < 0x80 else b[1] - 0x100)
            order.append((cur, tgt))
            if len(order) > max_arms:
                return first, order, None, "BOUND EXHAUSTED — count is my parameter, not the image's", skipped
            va += 2
            continue
        if b[0] == 0x89 and (b[1] & 0xC0) == 0x40:
            # `mov [reg+disp8], reg32` — a store. It writes no flags, so it may
            # sit inside the chain without breaking it. Stepped over and NAMED,
            # never silently. (The first version of this decoder terminated
            # here and reported a chain of length 0 — prereg M3, defect D1.)
            skipped.append((va, b[0:3].hex(" ")))
            va += 3
            continue
        # Anything else terminates the chain: this is the fallthrough arm.
        return first, order, va, None, skipped
    raise FenceRefused("chain did not terminate in 512 steps")


def literal_at(img, va):
    """`mov bl, imm8` (b3 ii) or `mov <r8>, imm8` (b0..b7) — return imm8."""
    b = img.at(va, 2)
    if 0xB0 <= b[0] <= 0xB7:
        return b[1], f"mov {['al','cl','dl','bl','ah','ch','dh','bh'][b[0]-0xB0]},{b[1]:#04x}"
    return None, f"(not a mov r8,imm8: {b[0]:#04x})"


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    path = argv[1]
    site, arm, max_arms = DEFAULT_SITE, DEFAULT_ARM, 32
    i = 2
    while i < len(argv):
        if argv[i] == "--site":
            site = int(argv[i + 1], 0); i += 2
        elif argv[i] == "--arm":
            arm = int(argv[i + 1], 0); i += 2
        elif argv[i] == "--max-arms":
            max_arms = int(argv[i + 1], 0); i += 2
        else:
            print(f"unknown option {argv[i]}"); return 2

    try:
        img = Image(path)
    except FenceRefused as exc:
        print(f"IMAGE FENCE REFUSED — {exc}")
        return 3

    print(f"image {path}")
    print(f"  sha256 {img.digest}  (PINNED, verified)")
    print(f"  image base {img.image_base:#x}; sections: "
          + ", ".join(f"{n}@{v:#x}" for n, v, _, _ in img.sections))
    print()

    print(f"ROUTING ARM @ {arm:#x} — raw bytes {img.at(arm, 13).hex(' ')}")
    b = img.at(arm, 13)
    if b[0:3] == b"\x8d\x4d\xc8" and b[3] == 0xE8:
        callee = arm + 8 + struct.unpack_from("<i", b, 4)[0]
        print(f"  lea ecx,[ebp-0x38] ; call {callee:#x} ; jmp ...")
        print(f"  -> callee {callee:#x}"
              + ("  == the site" if callee == site else "  != the site under test"))
    else:
        print("  (does not match the expected lea/call/jmp shape)")
    print()

    first, order, fallthrough, note, skipped = decode_chain(img, site, max_arms)
    print(f"SITE @ {site:#x} — dispatch chain")
    print(f"  raw bytes {img.at(site, 0x62).hex(' ')}")
    if first is None:
        # Defect D2, found by running the registered control: pointing `--site`
        # at a site in another shape crashed instead of refusing. The walk
        # recognises exactly `sub eax,imm8` + `dec eax`/`je`; anything else —
        # e.g. the `dec ecx` / `sub ecx,5` chain at 0x10c1ac5c — is REFUSED,
        # never mis-decoded into a table that would look like an answer.
        print(f"  DECODER REFUSED — {note}. Nothing was decoded.")
        return 1
    print(f"  `sub eax,{first:#04x}` then {len(order)} `dec`/`je` arm(s) "
          f"and one fallthrough")
    for va, hexb in skipped:
        print(f"  stepped over a flag-preserving store @{va:#x}: {hexb}")
    if note:
        print(f"  {note}")
        return 1
    print()
    print("  IL opcode -> relation code (decoded, not asserted)")
    rows = []
    for op, tgt in order:
        code, how = literal_at(img, tgt)
        rows.append((op, tgt, code, how))
    code, how = literal_at(img, fallthrough)
    rows.append((None, fallthrough, code, how))
    for op, tgt, code, how in rows:
        opname = f"{op:#04x}" if op is not None else "else"
        cname = REL_NAMES.get(code, "?")
        print(f"    {opname:>6}  @{tgt:#x}  {how:<16}  code {code:>2}  {cname}")
    print()
    print("  the subtraction present at the site is the DISPATCH INDEX: `eax` is")
    print("  dead after the last `je`, and every arm loads an unrelated literal.")
    minus = [(op, code) for op, tgt, code, how in rows if op is not None]
    hits = sum(1 for op, code in minus if code == op - 0x1E)
    print(f"  `code == opcode - 0x1E` holds on {hits} of {len(minus)} decoded arms.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

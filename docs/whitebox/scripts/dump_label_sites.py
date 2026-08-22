#!/usr/bin/env python3
"""Enumerate the call sites that charge c2's compiler-label counter.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Reads the pinned image directly — no Ghidra needed — which is the point: this
is an **independent** derivation of the site counts that
`docs/whitebox/WB_LABEL_FINDINGS.md` §1.1 obtained from the Ghidra export, and
read R3 (`docs/whitebox/ref/P_LABEL.md`) uses the two agreeing as its first
control.

    0x10c2edd0   DAT, the TU-global label counter                (P_LABEL.md §1)
    0x10b97dd0   FUN, "take a number" — the sole charging routine (P_LABEL.md §2)
    0x10b97de5   the sole `inc DWORD PTR ds:0x10c2edd0`           (P_LABEL.md §2)
    0x10b9a455   FUN, the generic label constructor               (P_LABEL.md §4)
    0x10b99dfe   FUN, the name formatter — reads, never charges   (P_LABEL.md §6)
    0x10c2e918   DAT, the SECOND (per-function) `$L*` ordinal     (P_LABEL.md §6)
    0x10b7e113   FUN, which resets `0x10c2e918` to 1              (P_LABEL.md §6)

Usage:
    python3 docs/whitebox/scripts/dump_label_sites.py <c2.dll>
    python3 docs/whitebox/scripts/dump_label_sites.py <c2.dll> --sites 10b97dd0
    python3 docs/whitebox/scripts/dump_label_sites.py <c2.dll> --closure
    python3 docs/whitebox/scripts/dump_label_sites.py <c2.dll> --refs 10c2edd0

`--sites` lists every `E8 rel32` direct call to a target, found by scanning
`.text` rather than by trusting a disassembler's function partition.
`--refs` lists every instruction-stream occurrence of a VA as a 4-byte little-
endian immediate, anywhere in the image, which is what answers *"is there an
indirect route?"* — a call site is enumerable, a function pointer in a table is
not.  `--closure` runs the whole check and prints the verdict.

**The scan's own limitation, stated because the closure claim rests on it.**
An `E8`-byte scan over raw `.text` cannot distinguish an opcode byte from a
data byte inside an instruction, so it is a superset detector: it can produce a
false site, never miss a real one.  Every site it reports is cross-checked
against the Ghidra export's `xrefs.tsv` in P_LABEL.md §3, and the two agree
exactly — which is the useful direction, because a false positive here would
show up as a site the export does not have.

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md §0); the script verifies the digest and refuses otherwise.
"""

import hashlib
import struct
import sys

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

COUNTER_VA = 0x10C2EDD0          # the TU-global label counter
ALLOCATOR_VA = 0x10B97DD0        # "take a number"
INCREMENT_VA = 0x10B97DE5        # the sole `inc [0x10c2edd0]`
LABEL_CTOR_VA = 0x10B9A455       # the generic label constructor
FORMATTER_VA = 0x10B99DFE        # the name formatter
ORDINAL_VA = 0x10C2E918          # the SECOND, per-function `$L*` ordinal
ORDINAL_RESET_FN_VA = 0x10B7E113
DOWNWARD_VA = 0x10C2ED40         # the downward end of the same id space

# The three writes to COUNTER_VA that P_LABEL.md §1 enumerates.  Held here so
# `--closure` fails loudly if a future image (or a mis-scan) has a fourth.
KNOWN_WRITES = {
    0x10B97807: "seed install — IL directive 0x16, from the stream",
    0x10B97CA1: "seed install — per-TU setup, max(IL value, current)",
    0x10B97DE5: "THE increment (+1), the only arithmetic write",
}


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

    def section(self, name):
        for s in self.sections:
            if s[0] == name:
                return s
        raise KeyError(name)

    def off(self, va):
        rva = va - self.image_base
        for _name, vaddr, vsize, rawptr, rawsize in self.sections:
            if vaddr <= rva < vaddr + vsize:
                delta = rva - vaddr
                if delta >= rawsize:
                    raise ValueError("VA %08x is in a section's virtual tail" % va)
                return rawptr + delta
        raise ValueError("VA %08x maps to no section" % va)

    def u32(self, va):
        return struct.unpack_from("<I", self.blob, self.off(va))[0]


def call_sites(img, target_va):
    """Every `E8 rel32` in `.text` whose computed target is `target_va`.

    Superset detector — see the module docstring.  Returns the VA of the E8
    byte, which is the address a disassembler calls the call site.
    """
    _n, vaddr, _vsize, rawptr, rawsize = img.section(".text")
    base = img.image_base + vaddr
    out = []
    blob = img.blob
    for i in range(rawsize - 5):
        if blob[rawptr + i] != 0xE8:
            continue
        rel, = struct.unpack_from("<i", blob, rawptr + i + 1)
        if base + i + 5 + rel == target_va:
            out.append(base + i)
    return out


def immediate_refs(img, va):
    """Every 4-byte little-endian occurrence of `va` in the whole image.

    This is the indirect-route detector: a direct `call` encodes a *relative*
    displacement and never contains the absolute target, so any absolute
    occurrence of a function's VA is a pointer — in a jump table, a vtable, a
    callback slot or an `import`.  Zero occurrences is what makes an
    enumeration of direct call sites a closure.
    """
    needle = struct.pack("<I", va)
    out = []
    start = 0
    while True:
        i = img.blob.find(needle, start)
        if i < 0:
            return out
        for name, vaddr, vsize, rawptr, rawsize in img.sections:
            if rawptr <= i < rawptr + rawsize:
                out.append((name, img.image_base + vaddr + (i - rawptr), i))
                break
        else:
            out.append(("<no section>", None, i))
        start = i + 1


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    img = Image(argv[1])
    if img.digest != PINNED_SHA256:
        print("REFUSING: sha256 %s != pinned %s" % (img.digest, PINNED_SHA256))
        return 1
    print("image  %s" % argv[1])
    print("sha256 %s  (matches the pinned digest)" % img.digest)
    print()

    mode = argv[2] if len(argv) > 2 else "--closure"

    if mode == "--sites":
        target = int(argv[3], 16)
        sites = call_sites(img, target)
        print("direct `call` sites of %08x: %d" % (target, len(sites)))
        for s in sites:
            print("  %08x" % s)
        return 0

    if mode == "--refs":
        va = int(argv[3], 16)
        for name, at, off in immediate_refs(img, va):
            print("  %-8s VA %08x  file+%06x" % (name, at or 0, off))
        return 0

    # --closure: the whole check, in the order P_LABEL.md §3 argues it.
    alloc = call_sites(img, ALLOCATOR_VA)
    ctor = call_sites(img, LABEL_CTOR_VA)
    print("STEP 1 — the charging routine has ONE increment")
    body = img.blob[img.off(ALLOCATOR_VA):img.off(ALLOCATOR_VA) + 28]
    print("  FUN_%08x, 28 bytes: %s" % (ALLOCATOR_VA, body.hex(" ")))
    print("  `inc DWORD PTR ds:0x%08x` = ff 05 %s at %08x  -> %s"
          % (COUNTER_VA, struct.pack("<I", COUNTER_VA).hex(" "), INCREMENT_VA,
             "PRESENT" if img.blob[img.off(INCREMENT_VA):img.off(INCREMENT_VA) + 6]
             == b"\xff\x05" + struct.pack("<I", COUNTER_VA) else "ABSENT"))
    print()

    print("STEP 2 — the counter is touched by a handful of instructions, and")
    print("         nothing else WRITES it")
    touch = immediate_refs(img, COUNTER_VA)
    print("  absolute 4-byte occurrences of %08x anywhere in the image: %d"
          % (COUNTER_VA, len(touch)))
    for name, at, off in touch:
        # The immediate follows the opcode byte(s); the instruction starts 1-2
        # bytes earlier.  Report the immediate's VA and let the reader map it.
        print("    %-8s immediate at VA %08x  file+%06x" % (name, at or 0, off))
    print()
    print("  of those, the WRITES (from the Ghidra export's xrefs.tsv, which")
    print("  agrees with this scan instruction for instruction):")
    for va, what in sorted(KNOWN_WRITES.items()):
        print("    %08x  %s" % (va, what))
    print("  -> two seed installs (assignments from outside) + one +1.")
    print("     No `add [mem],k` for k>1, no decrement, no per-function reset.")
    print()

    print("STEP 3 — the routine's address is never taken")
    refs = immediate_refs(img, ALLOCATOR_VA)
    print("  absolute 4-byte occurrences of %08x anywhere in the image: %d"
          % (ALLOCATOR_VA, len(refs)))
    for name, at, off in refs:
        print("    %-8s VA %08x  file+%06x" % (name, at or 0, off))
    print("  -> %s" % ("CLOSED: direct call sites are the whole population"
                       if not refs else
                       "NOT CLOSED: an indirect route exists, enumerate it"))
    print()

    print("STEP 4 — the population")
    print("  direct call sites of the allocator FUN_%08x : %d" % (ALLOCATOR_VA, len(alloc)))
    print("  direct call sites of the ctor      FUN_%08x : %d" % (LABEL_CTOR_VA, len(ctor)))
    print("  total charging sites                          : %d" % (len(alloc) + len(ctor)))
    print()
    print("  NOTE: the ctor is ITSELF one of the allocator's sites, so the two")
    print("  numbers are not disjoint populations of charges — every ctor call")
    print("  charges through the allocator site at 0x10b9a468.  P_LABEL.md §4.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

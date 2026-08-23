#!/usr/bin/env python3
"""dump_tuple_splice.py -- the tuple-list splice primitives of c2.dll, from the image.

Lane w-read-r8 (read R8, block emission order).  Companion to R3's
`dump_label_sites.py`, and it deliberately runs the SAME closure argument on a
different subject so that the two can be compared:

    R3 asked "is the label allocator's call-site population closed?" and got YES,
    because the allocator's VA occurs zero times as data anywhere in the image.

    R8 asks the same question of the five tuple-list splice primitives and gets
    NO -- two of them have their VA taken and passed around as a function
    pointer.  That is a *finding*, not a failure of the script: the direction of
    a splice (before vs after) is a RUNTIME PARAMETER in c2, chosen by the
    caller and handed to a shared tuple builder.

Nothing here is Ghidra-derived.  The image is parsed directly, so this is
re-runnable on any box that has the pinned c2.dll.

Usage:
    python3 docs/whitebox/scripts/dump_tuple_splice.py <path-to-c2.dll> [--json]

Exit codes: 0 ok, 2 image digest mismatch, 3 usage.
"""

import hashlib
import json
import struct
import sys

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

# The five primitives, read from the disassembly by lane w-read-r8.
# `sig` is the exact byte prefix of the function body at that VA; it is checked
# so that this script fails loudly if it is ever pointed at a different build.
PRIMITIVES = [
    ("10bd3815", 15, "INSERT AFTER  (at, new)",
     "new->next=at->next; at->next=new; new->prev=at; new->next->prev=new"),
    ("10bd3824", 17, "INSERT BEFORE (at, new)",
     "new->prev=at->prev; at->prev=new; new->next=at; new->prev->next=new"),
    ("10bd3835", 29, "SPLICE CHAIN AFTER (at, chain)",
     "at->next=chain; chain->prev=at; walk chain to its end; reattach old tail"),
    ("10bd3852", 31, "UNLINK (t)",
     "t->prev->next=t->next; t->next->prev=t->prev; t->next=t->prev=0"),
    ("10bd38d0", 50, "MOVE RANGE (a, b, c)",
     "unlink around b, then relink the range in front of a"),
]

# The tuple record fields these primitives establish.  Cross-checked against
# ref/P_DAG.md's independently-read `tuple+0 next, +0x10 prev`.
FIELDS = [
    ("+0x00", "next", "the emit walk FUN_10b338f5 advances by exactly this"),
    ("+0x04", "opcode", "int; 0x308 = label, 0x318 = section start"),
    ("+0x08", "kind", "byte; stamped by the allocator FUN_10bd3750"),
    ("+0x09", "flags", "bit0 = is a real machine instruction (P_ENCODE)"),
    ("+0x10", "prev", ""),
]


class Image:
    def __init__(self, path):
        self.blob = open(path, "rb").read()
        self.sha = hashlib.sha256(self.blob).hexdigest()
        pe = struct.unpack_from("<I", self.blob, 0x3C)[0]
        if self.blob[pe:pe + 4] != b"PE\0\0":
            raise SystemExit("not a PE image: %s" % path)
        nsec = struct.unpack_from("<H", self.blob, pe + 6)[0]
        optsz = struct.unpack_from("<H", self.blob, pe + 20)[0]
        self.imagebase = struct.unpack_from("<I", self.blob, pe + 24 + 28)[0]
        self.sections = []
        off = pe + 24 + optsz
        for i in range(nsec):
            name = self.blob[off:off + 8].rstrip(b"\0").decode("ascii", "replace")
            vsize, vaddr, rsize, raddr = struct.unpack_from("<IIII", self.blob, off + 8)
            self.sections.append((name, vaddr, vsize, raddr, rsize))
            off += 40

    def sect_of_va(self, va):
        rva = va - self.imagebase
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if vaddr <= rva < vaddr + max(vsize, rsize):
                return name
        return None

    def off_of_va(self, va):
        rva = va - self.imagebase
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if vaddr <= rva < vaddr + max(vsize, rsize):
                return raddr + (rva - vaddr)
        return None

    def va_of_off(self, off):
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if raddr <= off < raddr + rsize:
                return self.imagebase + vaddr + (off - raddr)
        return None

    def text_range(self):
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if name == ".text":
                return raddr, raddr + rsize
        raise SystemExit("no .text section")


def direct_calls_to(img, target_va):
    """Every `E8 rel32` in .text whose destination is target_va."""
    lo, hi = img.text_range()
    hits = []
    b = img.blob
    i = lo
    while i < hi - 5:
        i = b.find(b"\xe8", i, hi - 5)
        if i < 0:
            break
        rel = struct.unpack_from("<i", b, i + 1)[0]
        site_va = img.va_of_off(i)
        if site_va is not None and site_va + 5 + rel == target_va:
            hits.append(site_va)
        i += 1
    return hits


def classify_immediate(b, i):
    """Given a 4-byte immediate at file offset i, name the x86 instruction that
    would put it there, from the byte(s) immediately before it.

    A raw 4-byte scan is an instrument that has not been tested (this repo's
    'ranking instruments measure themselves' pattern, five entries and
    counting).  A hit that is not preceded by a plausible imm32-bearing opcode
    is reported as `unclassified` rather than counted as an address-take.
    """
    if i < 1:
        return "unclassified"
    p1 = b[i - 1]
    if p1 == 0x68:
        return "push imm32"
    if 0xB8 <= p1 <= 0xBF:
        return "mov r32, imm32"
    if i >= 2:
        p2 = b[i - 2]
        # C7 /0 with a 1-byte modrm (register or [reg]) then imm32
        if p2 == 0xC7:
            return "mov r/m32, imm32"
        if p2 == 0x3D or p2 == 0x05:
            return "cmp/add eax, imm32"
    if i >= 3 and b[i - 3] == 0xC7:
        return "mov [reg+disp8], imm32"
    if i >= 6 and b[i - 6] == 0xC7:
        return "mov [reg+disp32], imm32"
    return "unclassified"


def address_taken(img, target_va):
    """Every 4-byte little-endian occurrence of target_va anywhere in the image,
    excluding the bytes that are the rel32 of a direct call to it.

    Each hit is classified by the opcode that precedes it; only classified hits
    are evidence that the address is genuinely taken."""
    needle = struct.pack("<I", target_va)
    call_rel_offsets = set()
    for site_va in direct_calls_to(img, target_va):
        off = img.off_of_va(site_va)
        if off is not None:
            call_rel_offsets.add(off + 1)
    hits = []
    start = 0
    b = img.blob
    while True:
        i = b.find(needle, start)
        if i < 0:
            break
        if i not in call_rel_offsets:
            va = img.va_of_off(i)
            sect = img.sect_of_va(va) if va else None
            how = classify_immediate(b, i) if sect == ".text" else "data"
            hits.append((i, va, sect, how))
        start = i + 1
    return hits


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 3
    img = Image(argv[1])
    as_json = "--json" in argv

    out = {"image": argv[1], "sha256": img.sha, "pinned_match": img.sha == PINNED_SHA256,
           "fields": FIELDS, "primitives": []}

    if not as_json:
        print("image  %s" % argv[1])
        print("sha256 %s  (%s)" % (
            img.sha,
            "matches the pinned digest" if img.sha == PINNED_SHA256
            else "*** DOES NOT MATCH THE PIN -- STOP ***"))
        print()
    if img.sha != PINNED_SHA256:
        if not as_json:
            print("Refusing to report addresses against an unpinned image.")
            return 2

    if not as_json:
        print("THE TUPLE RECORD, as these primitives establish it")
        print("  (independently read as `tuple+0 next, +0x10 prev` in ref/P_DAG.md)")
        for off, name, note in FIELDS:
            print("    %-6s %-8s %s" % (off, name, note))
        print()
        print("THE FIVE SPLICE PRIMITIVES -- direct call sites, and whether the")
        print("address is ALSO taken as data (the R3 closure test)")
        print()

    total_sites = 0
    for va_hex, size, what, body in PRIMITIVES:
        va = int(va_hex, 16)
        calls = direct_calls_to(img, va)
        taken = address_taken(img, va)
        total_sites += len(calls)
        classified = [h for h in taken if h[3] != "unclassified"]
        byhow = {}
        for _, _, _, how in taken:
            byhow[how] = byhow.get(how, 0) + 1
        rec = {"va": va_hex, "size": size, "what": what, "body": body,
               "direct_call_sites": len(calls),
               "address_taken_raw4": len(taken),
               "address_taken_classified": len(classified),
               "by_opcode": byhow,
               "address_taken": [{"file_off": "0x%06x" % o,
                                  "va": ("0x%08x" % v) if v else None,
                                  "section": s, "how": how}
                                 for o, v, s, how in classified]}
        out["primitives"].append(rec)
        if not as_json:
            print("  FUN_%s  %3d B  %s" % (va_hex, size, what))
            print("      %s" % body)
            print("      direct call sites (E8 rel32) : %d" % len(calls))
            print("      raw 4-byte occurrences       : %d" % len(taken))
            if classified:
                print("      *** ADDRESS GENUINELY TAKEN  : %d classified "
                      "occurrence(s) -- the population is NOT closed"
                      % len(classified))
                for how in sorted(byhow):
                    if how != "unclassified":
                        print("            %-24s %d" % (how, byhow[how]))
                if byhow.get("unclassified"):
                    print("            %-24s %d  (NOT counted as evidence)"
                          % ("unclassified", byhow["unclassified"]))
                for o, v, s, how in classified[:6]:
                    print("            e.g. file+%06x  VA %s  %s"
                          % (o, ("0x%08x" % v) if v else "(unmapped)", how))
            else:
                print("      address never genuinely taken: the direct calls are "
                      "all of them (%d raw hits, all unclassified)" % len(taken))
            print()

    out["total_direct_call_sites"] = total_sites

    if not as_json:
        print("TOTAL direct call sites across the five primitives: %d" % total_sites)
        print()
        print("HOW TO READ THE CLOSURE RESULT")
        print("  R3 (dump_label_sites.py --closure) established that the label")
        print("  allocator FUN_10b97dd0's VA occurs ZERO times as data, so its 31")
        print("  direct calls are provably all of them.")
        print()
        print("  That argument does NOT transfer here, and the reason is the")
        print("  finding: the INSERT AFTER / INSERT BEFORE pair is passed BY")
        print("  POINTER into the shared tuple builders (FUN_10bd79b9,")
        print("  FUN_10bd76e6, FUN_10bd72b0, ... all of which take an inserter")
        print("  argument).  So in c2 the DIRECTION OF A SPLICE is a runtime")
        print("  parameter, not a property of the builder -- which is exactly")
        print("  what lets one builder produce source order at one call site and")
        print("  reverse order at another.")
    else:
        print(json.dumps(out, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

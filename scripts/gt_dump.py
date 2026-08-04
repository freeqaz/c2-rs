#!/usr/bin/env python3
"""gt_dump.py — ground-truth COFF/PPC obj dumper.

Read-only measurement tooling for the capture lane (docs/CODEGEN_FRAMED_CALLS.md,
docs/ABI_EDGES.md). Prints the section table, symbol table, relocations and a
big-endian PPC disassembly of every code section of an Xbox 360 COFF object
produced by `cl.exe` 16.00.11886.00.

Outside the std-only Rust workspace on purpose — same status as
`scripts/plot_perf.py`: tooling, never linked into the port.

Usage:
    scripts/gt_dump.py <obj> [--text-only] [--no-disasm] [--raw]
    scripts/gt_dump.py --selftest        # pin the reader, no toolchain needed

Disassembly shells out to `llvm-mc -disassemble -triple=powerpc-unknown-unknown`
when it is on PATH; without it the words are printed as hex.
"""

import struct
import subprocess
import sys

SC = {
    2: "EXTERNAL",
    3: "STATIC",
    6: "LABEL",
    103: "FILE",
    105: "SECTION",
}

# `IMAGE_REL_PPC_*`, complete, from `winnt.h`.
#
# **This table used to be three-eighths the i386 table** (found 2026-08-04 by
# lane `w-llvm`, cross-checking against a stock LLVM that can read our objs;
# fixed here by `w-gr`, which owns this file). The three defects:
#
#   * `0x000A` read **SECTION**, which is i386's name for that code. On PPC it
#     is **ADDR32NB**.
#   * `0x0013` read **SECREL**, which is i386's name for `0x000B`. On PPC
#     `0x0013` is **SECRELLO** and `0x000B` is SECREL.
#   * `0x000C` (**SECTION**) was **absent entirely**, so it printed as `0xc`.
#     w-llvm established what it is without any name table — `0x000B`/`0x000C`
#     occur in matched pairs 4 bytes apart, 18 of each, which is
#     `6 S_GPROC32 + 2 S_GDATA32 + 4 S_LABEL32 + 6 Lines` subsections.
#
#     **One refinement, measured here: it is live under `/Z7`, and NOT at the
#     workload's own flags.** `w14_dtor_delegate_neg.cpp` at
#     `/GR /O1 /Oi /EHsc /GS- /c` carries **five** relocation words — ADDR32 43,
#     REL24 12, REFHI 2, REFLO 2, PAIR 4 — and **zero** `0x000B`/`0x000C`; its
#     `.debug$S` has no relocations at all. Add `/Z7` and `.debug$S` gains
#     **30 `0x000B` and 30 `0x000C`**, matched, on the same source. So this code
#     is reachable by the debug-info lane and not by a workload capture, which
#     is why it stayed unnamed. (`REFHI + REFLO == PAIR` holds here too: 2+2=4.)
#
# A wrong *name* here has never produced a wrong obj — this file is read-only
# measurement tooling — but every hand-read of a relocation dump on this project
# went through it, and `0x0013` is the code the `.debug$S` and `.pdata` work
# argues about most.
#
# **Production objs use five of these words and no others**: ADDR32, REL24,
# REFHI, REFLO, PAIR — measured by w-llvm over 65,401 sections, with **no
# modifier bit ever set** and `REFHI + REFLO = 41,789 + 45,349 = 87,138 = PAIR`
# exactly. The rest of the table is here so a dump of something unusual is
# readable rather than silently hex.
RELOC = {
    0x0000: "ABSOLUTE",
    0x0001: "ADDR64",
    0x0002: "ADDR32",
    0x0003: "ADDR24",
    0x0004: "ADDR16",
    0x0005: "ADDR14",
    0x0006: "REL24",
    0x0007: "REL14",
    0x000A: "ADDR32NB",
    0x000B: "SECREL",
    0x000C: "SECTION",
    0x000D: "IFGLUE",
    0x000E: "IMGLUE",
    0x000F: "SECREL16",
    0x0010: "REFHI",
    0x0011: "REFLO",
    0x0012: "PAIR",
    0x0013: "SECRELLO",
    0x0014: "SECRELHI",
    0x0015: "GPREL",
    0x0016: "TOKEN",
}

# The high byte is modifier flags, not part of the type (`IMAGE_REL_PPC_TYPEMASK`
# is 0x00FF). Measured 0 times in production; decoded anyway, because a
# `hex(0x0106)` in a dump reads as an unknown relocation rather than as a negated
# REL24.
RELOC_TYPEMASK = 0x00FF
RELOC_FLAGS = (
    (0x0100, "NEG"),
    (0x0200, "BRTAKEN"),
    (0x0400, "BRNTAKEN"),
    (0x0800, "TOCDEFN"),
)


def reloc_name(ty):
    """`IMAGE_REL_PPC_*` name for a relocation word, modifier bits included.

    An unknown *type* is printed as hex rather than guessed at, and unknown
    *flag* bits are printed as hex too — a name this table cannot justify is
    worse than a number, which is the defect this function replaced.
    """
    base = RELOC.get(ty & RELOC_TYPEMASK)
    if base is None:
        base = hex(ty & RELOC_TYPEMASK)
    mods = [n for bit, n in RELOC_FLAGS if ty & bit]
    rest = ty & ~RELOC_TYPEMASK & ~sum(bit for bit, _ in RELOC_FLAGS)
    if rest:
        mods.append(hex(rest))
    return base + ("|" + "|".join(mods) if mods else "")


def u16(b, o):
    return struct.unpack_from("<H", b, o)[0]


def u32(b, o):
    return struct.unpack_from("<I", b, o)[0]


class Obj:
    def __init__(self, data):
        self.d = data
        self.machine = u16(data, 0)
        self.nsec = u16(data, 2)
        self.timestamp = u32(data, 4)
        self.symptr = u32(data, 8)
        self.nsym = u32(data, 12)
        self.optsz = u16(data, 16)
        self.flags = u16(data, 18)
        # The string table has to be located BEFORE the section headers are
        # walked: a `/NNN` section name is an offset into it (see `sec_name`).
        self.strtab_off = self.symptr + 18 * self.nsym
        self.sections = []
        off = 20 + self.optsz
        for i in range(self.nsec):
            h = data[off : off + 40]
            self.sections.append(
                {
                    "idx": i + 1,
                    "name": self.sec_name(h[0:8]),
                    "vsize": u32(h, 8),
                    "vaddr": u32(h, 12),
                    "rawsize": u32(h, 16),
                    "rawptr": u32(h, 20),
                    "relptr": u32(h, 24),
                    "nrel": u16(h, 32),
                    "chars": u32(h, 36),
                }
            )
            off += 40
        self.symbols = self._syms()

    def string(self, off):
        end = self.d.index(b"\0", self.strtab_off + off)
        return self.d[self.strtab_off + off : end].decode("latin1")

    def sec_name(self, raw8):
        """A section header's 8-byte name field, `/NNN` resolved.

        COFF spells a section name longer than 8 characters as `/` followed by a
        decimal offset into the string table. **This reader returned the literal
        `/23196`** until 2026-08-04; `crates/c2-obj`, `tools/coffdump.py` and
        `llvm-readobj` all resolve it, so this was the one reader of four that
        did not, and `crates/c2-obj` names `/NNN` as one of *"three chances for a
        second reader to disagree with this one"*.

        **It is LATENT, not live**: 0 of 65,401 real sections take this path (no
        section name c2 emits at the workload's flags exceeds 8 characters), so
        no corpus sweep could ever catch it — which is exactly why it survived,
        and why it was found by building a synthetic obj instead
        (`tools/llvm/longname_probe.py`, lane w-llvm). Reported there, fixed here
        because this file is `w-gr`'s seam.

        Anything that is not a well-formed `/NNN` inside the string table falls
        back to the literal field. A reader that *guessed* here would be worse
        than one that did not resolve at all.
        """
        name = raw8.rstrip(b"\0").decode("latin1")
        if not name.startswith("/") or not name[1:].isdigit():
            return name
        try:
            off = int(name[1:])
            size = u32(self.d, self.strtab_off)
            if off < 4 or off >= size:
                return name
            return self.string(off)
        except (ValueError, struct.error):
            return name

    def _syms(self):
        out = []
        i = 0
        while i < self.nsym:
            e = self.d[self.symptr + 18 * i : self.symptr + 18 * i + 18]
            if e[0:4] == b"\0\0\0\0":
                name = self.string(u32(e, 4))
            else:
                name = e[0:8].rstrip(b"\0").decode("latin1")
            s = {
                "idx": i,
                "name": name,
                "value": u32(e, 8),
                "sec": struct.unpack_from("<h", e, 12)[0],
                "type": u16(e, 14),
                "sc": e[16],
                "naux": e[17],
                "aux": [],
            }
            for a in range(s["naux"]):
                s["aux"].append(
                    self.d[self.symptr + 18 * (i + 1 + a) : self.symptr + 18 * (i + 1 + a) + 18]
                )
            out.append(s)
            i += 1 + s["naux"]
        return out

    def sym_by_index(self, idx):
        for s in self.symbols:
            if s["idx"] == idx:
                return s
            if s["idx"] < idx <= s["idx"] + s["naux"]:
                return s
        return None

    def relocs(self, sec):
        out = []
        for r in range(sec["nrel"]):
            o = sec["relptr"] + 10 * r
            out.append((u32(self.d, o), u32(self.d, o + 4), u16(self.d, o + 8)))
        return out

    def raw(self, sec):
        if sec["rawptr"] == 0:
            return b""
        return self.d[sec["rawptr"] : sec["rawptr"] + sec["rawsize"]]


def _mc(words):
    """Raw llvm-mc call. Returns the mnemonic lines, or None if llvm-mc is absent.

    NOTE: llvm-mc emits NOTHING for a word it cannot decode (the "invalid
    instruction encoding" diagnostic goes to stderr), so the line count is
    <= the word count and the two are NOT positionally aligned.
    """
    hexs = " ".join("0x%02x" % b for w in words for b in struct.pack(">I", w))
    try:
        out = subprocess.run(
            ["llvm-mc", "-disassemble", "-triple=powerpc-unknown-unknown"],
            input=hexs,
            capture_output=True,
            text=True,
        ).stdout
    except FileNotFoundError:
        return None
    return [l.strip() for l in out.splitlines() if l.strip() and not l.startswith(".")]


_MC1 = {}


def disasm(words):
    """Disassemble a list of BE u32 words via llvm-mc; fall back to hex.

    An EH function's `.text` opens with two relocated ZERO words (the
    `__CxxFrameHandler` / `__ehfuncinfo$` prefix, docs/EH_RECORDS.md §1) and
    llvm-mc silently drops them.  The previous implementation padded the
    shortfall with `?` at the END, which shifted EVERY mnemonic in the section
    up by two rows while still printing correct-looking output -- the exact
    "instrument lied" failure this lane exists to catch.  Alignment is now
    established per word, never inferred from a count.
    """
    if not words:
        return []
    lines = _mc(words)
    if lines is None:
        return ["%08x" % w for w in words]
    if len(lines) == len(words):
        return lines
    # A word did not decode: re-run per word so position is exact by
    # construction.  Memoised on the word value -- code sections repeat.
    out = []
    for w in words:
        if w not in _MC1:
            r = _mc([w])
            _MC1[w] = r[0] if r else "<undecodable %08x>" % w
        out.append(_MC1[w])
    return out


def _synthetic_obj(long_name=b".averylongsectionname$probe"):
    """A minimal COFF/PPC obj with ONE section whose name is a `/NNN` reference.

    Hand-built rather than captured, because the whole point is that **no obj
    `c2.dll` emits takes this path** — 0 of 65,401 real sections — so there is
    nothing to capture and a corpus sweep can never reach it.
    """
    nsec, nsym = 1, 0
    symptr = 20 + 40 * nsec
    strtab = struct.pack("<I", 4 + len(long_name) + 1) + long_name + b"\0"
    hdr = struct.pack("<HHIIIHH", 0x01F2, nsec, 0, symptr, nsym, 0, 0)
    sec = (b"/4".ljust(8, b"\0")
           + struct.pack("<IIIIIIHHI", 0, 0, 0, 0, 0, 0, 0, 0, 0)[:32])
    return hdr + sec + strtab


def selftest():
    """Pin the two defects `w-llvm` found in this file. No toolchain, no LLVM.

    Everything below is a COUNT or an explicit equality, never "no error was
    raised": both defects were silent for the file's whole existence precisely
    because nothing asserted anything about them.
    """
    checks = 0

    # ---- 1. `/NNN` long section names --------------------------------------
    o = Obj(_synthetic_obj())
    assert o.sections[0]["name"] == ".averylongsectionname$probe", o.sections[0]["name"]
    checks += 1
    # …and the fallbacks. A reader that GUESSED here would be worse than one
    # that did not resolve, so each malformed form must come back literal.
    for raw, want in (
        (b".text\0\0\0", ".text"),
        (b"/\0\0\0\0\0\0\0", "/"),            # no digits
        (b"/abc\0\0\0\0", "/abc"),            # not decimal
        (b"/99999\0\0", "/99999"),            # past the end of the string table
        (b"/0\0\0\0\0\0\0", "/0"),            # inside the size field itself
    ):
        got = o.sec_name(raw)
        assert got == want, "sec_name(%r) = %r, want %r" % (raw, got, want)
        checks += 1

    # ---- 2. the PPC relocation table ---------------------------------------
    # The three codes that were the i386 table's, by value and by name. These
    # are asserted individually rather than as a dict compare so a failure names
    # the one that regressed.
    for code, want in (
        (0x000A, "ADDR32NB"),   # was "SECTION" — i386's name for this code
        (0x000B, "SECREL"),     # was "SECREL_"
        (0x000C, "SECTION"),    # was ABSENT; live in every /Z7 .debug$S
        (0x0013, "SECRELLO"),   # was "SECREL" — i386's name for 0x000B
    ):
        assert reloc_name(code) == want, "0x%04X -> %s, want %s" % (code, reloc_name(code), want)
        checks += 1
    # The five words production actually uses (w-llvm, 65,401 sections).
    for code, want in ((0x0002, "ADDR32"), (0x0006, "REL24"),
                       (0x0010, "REFHI"), (0x0011, "REFLO"), (0x0012, "PAIR")):
        assert reloc_name(code) == want
        checks += 1
    assert len(set(RELOC.values())) == len(RELOC), "duplicate name in RELOC"
    checks += 1
    # Modifier bits are decoded, not folded into the type.
    assert reloc_name(0x0106) == "REL24|NEG", reloc_name(0x0106)
    assert reloc_name(0x0806) == "REL24|TOCDEFN", reloc_name(0x0806)
    checks += 2
    # An unknown type is a NUMBER, never a neighbouring name.
    assert reloc_name(0x0099) == "0x99", reloc_name(0x0099)
    checks += 1

    print("gt_dump selftest: %d checks PASS "
          "(/NNN long names resolved + fallbacks; the PPC relocation table)" % checks)
    assert checks >= 18, "selftest ran only %d checks — it must not shrink silently" % checks
    return 0


def main(argv):
    if len(argv) >= 2 and argv[1] == "--selftest":
        return selftest()
    if len(argv) < 2:
        print(__doc__)
        return 2
    path = argv[1]
    text_only = "--text-only" in argv
    no_disasm = "--no-disasm" in argv
    show_raw = "--raw" in argv
    o = Obj(open(path, "rb").read())

    print("== %s  machine=0x%04x nsec=%d nsym=%d size=%d" % (path, o.machine, o.nsec, o.nsym, len(o.d)))
    if not text_only:
        print("-- sections")
        for s in o.sections:
            print(
                "  %2d %-10s raw=%-6d rawptr=0x%04x rel=%-2d relptr=0x%04x chars=0x%08x"
                % (s["idx"], s["name"], s["rawsize"], s["rawptr"], s["nrel"], s["relptr"], s["chars"])
            )

    for s in o.sections:
        if not s["name"].startswith(".text"):
            if text_only:
                continue
            if s["name"] in (".rdata", ".pdata", ".data", ".bss", ".xdata"):
                data = o.raw(s)
                print("-- %s #%d raw" % (s["name"], s["idx"]))
                for i in range(0, len(data), 16):
                    print("   %04x  %s" % (i, data[i : i + 16].hex(" ")))
                for va, sym, ty in o.relocs(s):
                    t = o.sym_by_index(sym)
                    print("   reloc va=0x%x %-8s -> [%d] %s" % (va, reloc_name(ty), sym, t["name"] if t else "?"))
            continue
        data = o.raw(s)
        words = list(struct.unpack(">%dI" % (len(data) // 4), data)) if len(data) >= 4 else []
        rels = {}
        for va, sym, ty in o.relocs(s):
            rels.setdefault(va, []).append((sym, ty))
        owner = None
        for sym in o.symbols:
            if sym["sec"] == s["idx"] and sym["sc"] == 2:
                owner = sym["name"]
                break
        print("-- .text #%d (%d B) %s" % (s["idx"], s["rawsize"], owner or ""))
        if show_raw:
            print("   raw: %s" % data.hex())
        text = ["%08x" % w for w in words] if no_disasm else disasm(words)
        for i, w in enumerate(words):
            va = i * 4
            ann = ""
            for sym, ty in rels.get(va, ()):
                t = o.sym_by_index(sym)
                ann += "  ; %s -> [%d] %s" % (reloc_name(ty), sym, t["name"] if t else "?")
            print("   %04x  %08x  %-32s%s" % (va, w, text[i], ann))

    if not text_only:
        print("-- symbols")
        for sym in o.symbols:
            extra = ""
            if sym["naux"] and sym["sc"] in (3, 105):
                a = sym["aux"][0]
                extra = " aux(len=%d nrel=%d nln=%d cksum=0x%08x num=%d sel=%d)" % (
                    u32(a, 0),
                    u16(a, 4),
                    u16(a, 6),
                    u32(a, 8),
                    u16(a, 12),
                    a[14],
                )
            print(
                "  [%2d] %-40s sec=%-3d val=0x%-6x sc=%-9s type=0x%04x naux=%d%s"
                % (
                    sym["idx"],
                    sym["name"],
                    sym["sec"],
                    sym["value"],
                    SC.get(sym["sc"], str(sym["sc"])),
                    sym["type"],
                    sym["naux"],
                    extra,
                )
            )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

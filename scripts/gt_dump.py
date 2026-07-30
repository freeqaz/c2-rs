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

RELOC = {
    0x0001: "ADDR64",
    0x0002: "ADDR32",
    0x0006: "REL24",
    0x0010: "REFHI",
    0x0011: "REFLO",
    0x0012: "PAIR",
    0x0013: "SECREL",
    0x000A: "SECTION",
    0x000B: "SECREL_",
}


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
        self.sections = []
        off = 20 + self.optsz
        for i in range(self.nsec):
            h = data[off : off + 40]
            self.sections.append(
                {
                    "idx": i + 1,
                    "name": h[0:8].rstrip(b"\0").decode("latin1"),
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
        self.strtab_off = self.symptr + 18 * self.nsym
        self.symbols = self._syms()

    def string(self, off):
        end = self.d.index(b"\0", self.strtab_off + off)
        return self.d[self.strtab_off + off : end].decode("latin1")

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


def disasm(words):
    """Disassemble a list of BE u32 words via llvm-mc; fall back to hex."""
    if not words:
        return []
    hexs = " ".join("0x%02x" % b for w in words for b in struct.pack(">I", w))
    try:
        out = subprocess.run(
            ["llvm-mc", "-disassemble", "-triple=powerpc-unknown-unknown"],
            input=hexs,
            capture_output=True,
            text=True,
        ).stdout
    except FileNotFoundError:
        return ["%08x" % w for w in words]
    lines = [l.strip() for l in out.splitlines() if l.strip() and not l.startswith(".")]
    if len(lines) != len(words):
        lines = lines + ["?"] * (len(words) - len(lines))
    return lines[: len(words)]


def main(argv):
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
                    print("   reloc va=0x%x %-8s -> [%d] %s" % (va, RELOC.get(ty, hex(ty)), sym, t["name"] if t else "?"))
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
                ann += "  ; %s -> [%d] %s" % (RELOC.get(ty, hex(ty)), sym, t["name"] if t else "?")
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

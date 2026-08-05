#!/usr/bin/env python3
"""parse.py — shared reader for the w-dclass repricing.

Reads the `scripts/gt_dump.py` text dumps in `work/w-dclass/dis/` and the
`c2rs census` transcripts in `work/w-dclass/census/`, and pairs them.

**The pairing key is the ORDINAL, not the name.** Factor A (`.ex` segments ==
obj `.text` COMDATs) holds on every one of the nineteen FRONTIER TUs — checked
here, per TU, and a mismatch is a hard error rather than a silent skip. The
census *name* is read out of `.gl` and is NOT reliable: on
`src/xdk/nuispeech/xboxheap.cpp` the census names the one emitted function
`?AllocatePageBlock@…` while the obj's one `.text` COMDAT is
`??0CXboxHeap@NUISPEECH@@QAA@II@Z` — the callee, not the caller. Ten of the
nineteen TUs have `(unnamed)` census rows besides.

Outside the std-only Rust workspace on purpose — same status as
`scripts/gt_dump.py`. Read-only with respect to `crates/`.
"""

import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# The nineteen FRONTIER TUs, in the order `c2rs gap` prints them (blocked
# emitted ascending, then src). Regenerate with `reprice.py --frontier`.
FRONTIER = [
    "src/Main.cpp",
    "src/system/math/Primes.cpp",
    "src/system/math/Sort.cpp",
    "src/xdk/LIBCMT/osfinfo.cpp",
    "src/xdk/LIBCMT/undname.cpp",
    "src/xdk/LIBCMT/vswprnc.cpp",
    "src/xdk/nuispeech/xboxheap.cpp",
    "src/xdk/xjson/jsonwriter.cpp",
    "src/xdk/xlrc/xlrcimpl.cpp",
    "src/system/negate_test.cpp",
    "src/system/synth_xbox/Biquad.cpp",
    "src/xdk/LIBCMT/vsnprnc.cpp",
    "src/xdk/nuispeech/xboxmem.cpp",
    "src/system/rndobj/wordwrap.cpp",
    "src/system/utl/Pool.cpp",
    "src/xdk/nuispeech/mmio.cpp",
    "src/system/synth_xbox/IPP_basicmath_xbox.cpp",
    "src/system/utl/EncryptXTEA.cpp",
    "src/keygen_xbox.cpp",
]


class Insn:
    __slots__ = ("off", "word", "text", "mnem", "ops", "notes")

    def __init__(self, off, word, text, notes):
        self.off = off
        self.word = word
        self.text = text
        self.notes = notes            # list of "REL24 -> [n] name" style strings
        t = text.strip().replace("\t", " ")
        parts = t.split(None, 1)
        self.mnem = parts[0] if parts else "?"
        self.ops = parts[1].strip() if len(parts) > 1 else ""

    def __repr__(self):
        return "%04x %08x %s" % (self.off, self.word, self.text.strip())


class Comdat:
    __slots__ = ("secno", "name", "size", "insns", "kind")

    def __init__(self, secno, name, size, kind):
        self.secno = secno
        self.name = name
        self.size = size
        self.kind = kind              # ".text" etc
        self.insns = []


class ObjDump:
    def __init__(self):
        self.path = ""
        self.sections = []            # (no, name, raw, nrel, chars)
        self.text = []                # list[Comdat] in file order
        self.symbols = []             # (idx, name, sec, val, sc)
        self.pdata_targets = set()    # symbol names an ADDR32 in .pdata points at
        self.raw_sec_names = []


_SEC = re.compile(r"^\s+(\d+)\s+(\S+)\s+raw=(\d+)\s+rawptr=\S+\s+rel=(\d+)\s+relptr=\S+\s+chars=(\S+)")
_TEXT_HDR = re.compile(r"^-- (\.\S+) #(\d+) \((\d+) B\)\s*(.*)$")
_RAW_HDR = re.compile(r"^-- (\.\S+) #(\d+) raw\s*$")
_INSN = re.compile(r"^\s+([0-9a-f]{4})\s+([0-9a-f]{8})\s\s(.*)$")
_SYM = re.compile(r"^\s+\[\s*(\d+)\]\s+(\S+)\s+sec=(-?\d+)\s+val=(\S+)\s+sc=(\S+)")
_PDATA_REL = re.compile(r"^\s+reloc va=\S+ (\S+)\s+-> \[\d+\] (\S+)")


def read_dump(path):
    d = ObjDump()
    d.path = path
    cur = None
    mode = None
    for line in open(path, encoding="utf-8", errors="replace"):
        line = line.rstrip("\n")
        m = _TEXT_HDR.match(line)
        if m:
            cur = Comdat(int(m.group(2)), m.group(4).strip(), int(m.group(3)), m.group(1))
            if m.group(1) == ".text":
                d.text.append(cur)
            mode = "insn"
            continue
        m = _RAW_HDR.match(line)
        if m:
            cur = None
            mode = "raw:" + m.group(1)
            continue
        if line.startswith("-- symbols"):
            cur, mode = None, "sym"
            continue
        if line.startswith("-- sections"):
            cur, mode = None, "sec"
            continue
        if mode == "sec":
            m = _SEC.match(line)
            if m:
                d.sections.append((int(m.group(1)), m.group(2), int(m.group(3)),
                                   int(m.group(4)), m.group(5)))
                d.raw_sec_names.append(m.group(2))
            continue
        if mode == "sym":
            m = _SYM.match(line)
            if m:
                d.symbols.append((int(m.group(1)), m.group(2), int(m.group(3)),
                                  m.group(4), m.group(5)))
            continue
        if mode and mode.startswith("raw:.pdata"):
            m = _PDATA_REL.match(line)
            if m:
                d.pdata_targets.add(m.group(2))
            continue
        if mode == "insn" and cur is not None:
            m = _INSN.match(line)
            if m:
                body = m.group(3)
                notes = []
                if ";" in body:
                    txt, rest = body.split(";", 1)
                    notes = [p.strip() for p in rest.split(";") if p.strip()]
                else:
                    txt = body
                cur.insns.append(Insn(int(m.group(1), 16), int(m.group(2), 16), txt, notes))
            continue
    return d


_CENSUS_ROW = re.compile(
    r"^\s+\[\s*(\d+)\]\s+(ok |GAP)\s+(\S+)\s+(\S+)\s+(\S+)\s+\((\S+)\s*\)\s+(\d+) B\s+(.*)$"
)


def read_census(path):
    """-> list of dicts, in .ex segment order."""
    rows = []
    for line in open(path, encoding="utf-8", errors="replace"):
        m = _CENSUS_ROW.match(line.rstrip("\n"))
        if m:
            rows.append({
                "index": int(m.group(1)),
                "blocked": m.group(2) == "GAP",
                "key": m.group(3),
                "cflow": m.group(4),
                "eh": m.group(5),
                "eh_stmt": m.group(6),
                "seg_len": int(m.group(7)),
                "name": m.group(8).strip(),
            })
    return rows


def pair(tu_base, disdir, censusdir):
    """Pair census rows with `.text` COMDATs by ordinal. Raises on a length
    mismatch — factor A is the premise and a silent skip would hide its failure."""
    d = read_dump(os.path.join(disdir, tu_base + ".txt"))
    rows = read_census(os.path.join(censusdir, tu_base + ".txt"))
    if len(rows) != len(d.text):
        raise SystemExit(
            "FATAL %s: census has %d rows but the obj has %d .text COMDATs — "
            "factor A does not hold here and the ordinal pairing is invalid"
            % (tu_base, len(rows), len(d.text))
        )
    return d, list(zip(rows, d.text))


if __name__ == "__main__":
    # Smoke: print the pairing for every TU dumped so far.
    disdir = os.path.join(REPO, "work/w-dclass/dis")
    cendir = os.path.join(REPO, "work/w-dclass/census")
    n = 0
    for src in FRONTIER:
        b = os.path.basename(src)[:-4]
        d, pairs = pair(b, disdir, cendir)
        blocked = [(r, c) for r, c in pairs if r["blocked"]]
        print("%-24s %2d COMDATs, %2d blocked: %s"
              % (b, len(pairs), len(blocked), ", ".join(c.name or "#%d" % c.secno for _, c in blocked)))
        n += 1
    print("paired %d TUs" % n)
    sys.exit(0 if n else 1)

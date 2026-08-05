#!/usr/bin/env python3
"""codparse.py -- read an MSVC `/FAcs` assembly listing (`.cod`) into
(offset, word, mnemonic, operands) tuples.

Pure stdlib. Tooling, outside the std-only Rust workspace.

THIS FILE IS THE ORACLE SIDE OF THE VMX128 WORK.

`cl /FAcs` makes the Microsoft back end narrate the bytes it just emitted:

      00000\t102320c3\t lvx128       vr1,r3,r4

Left of the first tab is the section offset, between the tabs is the exact
big-endian instruction word that will appear in `.text`, and right of it is
Microsoft's own mnemonic and operand list for that word. That is a *published
observable of the black box* -- the same category as the obj, the error text
and the `/FAsc` listing already used elsewhere in this repo -- so nothing here
incurs a `docs/whitebox/DISCLOSURE.md` row. We are reading the compiler's
output, not its code.

The one thing to be careful about: on a relocated word the listing prints the
*symbol*, while the word itself carries a zero (or addend-only) target field --
`48000001\t bl  __savegprlr_28`. Callers that compare operand text must skip
relocated instructions. `is_relocatable()` flags the primary opcodes where that
happens; VMX128 (primary 4/5/6) is never one of them.
"""
import re
import sys

# "  00000\t102320c3\t lvx128       vr1,r3,r4"   (optionally + "\t\t; comment")
_LINE = re.compile(r"^\s+([0-9A-Fa-f]{5,8})\t([0-9A-Fa-f]{8})\t\s*(\S+)(?:\s+(\S.*?))?\s*$")
_PROC = re.compile(r"^(\S+)\s+PROC\b")
_ENDP = re.compile(r"^(\S+)\s+ENDP\b")

# Primary opcodes whose operand text can be a relocated symbol name rather
# than the literal field in the word.
_RELOCATABLE_PRIMARY = frozenset([15, 16, 18, 24, 25, 32, 33, 34, 36, 37, 38,
                                  40, 41, 42, 44, 46, 47, 48, 50, 52, 54, 58, 62])


class Insn(object):
    __slots__ = ("offset", "word", "mnemonic", "operands", "func", "line",
                 "source")

    def __init__(self, offset, word, mnemonic, operands, func, line, source):
        self.offset = offset
        self.word = word
        self.mnemonic = mnemonic
        self.operands = operands       # list[str], comment stripped
        self.func = func
        self.line = line               # 1-based line number in the .cod
        self.source = source           # path of the .cod

    def text(self):
        return "%s %s" % (self.mnemonic, ",".join(self.operands))

    def __repr__(self):
        return "<%s:%d %08x %s>" % (self.source, self.line, self.word, self.text())


def is_relocatable(word):
    return (word >> 26) in _RELOCATABLE_PRIMARY


def _split_operands(rest):
    if rest is None:
        return []
    # strip a trailing "; ..." comment, then split on commas that are not
    # inside a parenthesised displacement -- MSVC prints `0(r11)` as one arg.
    semi = rest.find(";")
    if semi >= 0:
        rest = rest[:semi]
    rest = rest.strip()
    if not rest:
        return []
    return [p.strip() for p in rest.split(",")]


def parse(path):
    """Yield `Insn` for every machine-code line in one `.cod` listing."""
    func = None
    with open(path, "r", errors="replace") as fh:
        for lineno, raw in enumerate(fh, 1):
            raw = raw.rstrip("\n").rstrip("\r")
            m = _PROC.match(raw)
            if m:
                func = m.group(1)
                continue
            if _ENDP.match(raw):
                func = None
                continue
            m = _LINE.match(raw)
            if not m:
                continue
            off = int(m.group(1), 16)
            word = int(m.group(2), 16)
            yield Insn(off, word, m.group(3), _split_operands(m.group(4)),
                       func, lineno, path)


def parse_many(paths):
    for p in paths:
        for insn in parse(p):
            yield insn


if __name__ == "__main__":
    n = 0
    for insn in parse_many(sys.argv[1:]):
        n += 1
        print("%-40s +%05x %08x  %s" % (insn.func or "-", insn.offset,
                                        insn.word, insn.text()))
    print("# %d machine-code lines" % n, file=sys.stderr)

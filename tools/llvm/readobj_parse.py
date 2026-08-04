#!/usr/bin/env python3
"""readobj_parse — turn llvm-readobj's indented text into nested Python.

llvm-readobj has no JSON writer for COFF (``--elf-output-style=JSON`` is ELF
only), so the cross-check parses the text. The grammar is small and regular:

    Key: Value
    Key: Name (0x1F0)          -- trailing parenthetical, often the raw number
    Key {                      -- a dict, closed by }
    Key [                      -- a list, closed by ]
    Key [ (0x180)              -- a flag list carrying its raw word

Repeated keys inside one scope (``Section {`` many times) are kept as a list of
``(key, value)`` pairs, so nothing is silently dropped by dict collision — a
parser that lost 460 of 461 sections to key collision would still print a
plausible answer, which is the failure mode this repo keeps finding.
"""

import re

_NUM = re.compile(r"^\((-?(?:0x[0-9A-Fa-f]+|\d+))\)$")
# llvm-readobj follows a COFF section/symbol name with its raw 8 bytes:
#   Name: .debug$S (2E 64 65 62 75 67 24 53)
# Keeping those is the point -- they are how a *long* name (`/NNN` into the
# string table) is told apart from a short one without trusting either decoder.
_BYTES = re.compile(r"^\(((?:[0-9A-F]{2} )*[0-9A-F]{2})\)$")


def _val(s):
    """'IMAGE_FILE_MACHINE_POWERPC (0x1F0)' -> ('IMAGE_FILE_MACHINE_POWERPC', 496).

    A raw-bytes parenthetical yields ``(name, None)`` with the bytes recoverable
    via :func:`raw_bytes`.
    """
    s = s.strip()
    name, num = s, None
    if s.endswith(")"):
        i = s.rfind("(")
        if i >= 0:
            m = _NUM.match(s[i:])
            if m:
                name = s[:i].strip()
                t = m.group(1)
                num = int(t, 16) if t.lower().startswith(("0x", "-0x")) else int(t)
            elif _BYTES.match(s[i:]):
                name = s[:i].strip()
    if num is None:
        try:
            num = int(name, 0)
        except ValueError:
            pass
    return name, num, s


def raw_bytes(v):
    """The 8 raw name bytes from a parsed 'Name: X (AA BB ...)' value, or None."""
    s = (v[2] if isinstance(v, tuple) else v).strip()
    if not s.endswith(")"):
        return None
    i = s.rfind("(")
    if i < 0:
        return None
    m = _BYTES.match(s[i:])
    if not m:
        return None
    return bytes(int(t, 16) for t in m.group(1).split())


class Node:
    """A scope. `pairs` preserves order and duplicates; `raw` is the flag word."""

    __slots__ = ("pairs", "raw")

    def __init__(self):
        self.pairs = []
        self.raw = None

    def all(self, key):
        return [v for k, v in self.pairs if k == key]

    def one(self, key, default=None):
        for k, v in self.pairs:
            if k == key:
                return v
        return default

    def num(self, key, default=None):
        v = self.one(key)
        if isinstance(v, tuple):
            return v[1] if v[1] is not None else default
        return default

    def name(self, key, default=None):
        v = self.one(key)
        if isinstance(v, tuple):
            return v[0]
        return default

    def rawname(self, key):
        v = self.one(key)
        return raw_bytes(v) if isinstance(v, tuple) else None


def parse(text):
    """Parse a whole llvm-readobj run into a root Node."""
    root = Node()
    stack = [root]
    for line in text.splitlines():
        s = line.strip()
        if not s:
            continue
        if s in ("}", "]"):
            if len(stack) > 1:
                stack.pop()
            continue
        if s.endswith("{"):
            key = s[:-1].strip().rstrip(":").strip()
            n = Node()
            stack[-1].pairs.append((key, n))
            stack.append(n)
            continue
        if "[" in s and s.rstrip().endswith("]") and not s.endswith(" ]"):
            # single-line list, e.g. "Flags [ ]" -- rare; treat as empty scope
            key = s.split("[", 1)[0].strip().rstrip(":").strip()
            stack[-1].pairs.append((key, Node()))
            continue
        if "[" in s:
            head, rest = s.split("[", 1)
            key = head.strip().rstrip(":").strip()
            n = Node()
            rest = rest.strip()
            if rest.startswith("(") and rest.endswith(")"):
                n.raw = _val(rest)[1]
            stack[-1].pairs.append((key, n))
            stack.append(n)
            continue
        if ":" in s:
            key, v = s.split(":", 1)
            stack[-1].pairs.append((key.strip(), _val(v)))
            continue
        # A bare token inside a flag list, e.g. "IMAGE_SCN_MEM_READ (0x40000000)"
        stack[-1].pairs.append(("@flag", _val(s)))
    return root


def find_all(node, key, out=None):
    """Every descendant scope stored under `key`, depth-first."""
    if out is None:
        out = []
    for k, v in node.pairs:
        if isinstance(v, Node):
            if k == key:
                out.append(v)
            find_all(v, key, out)
    return out

#!/usr/bin/env python3
"""names.py — MSVC decorated-name <-> C4505/C4514 pretty-name normalization.

The removal channel (`/Wall` C4514, `/W4` C4505) prints a *pretty, qualified,
parameterless* name:

    src/system\\utl/Symbol.h(25) : warning C4514: 'Symbol::operator <' : unreferenced inline function has been removed

The `.gl` name table and the obj COMDAT leaders carry *decorated* names
(`??MSymbol@@QBA_NABV0@@Z`). This module maps both sides onto one key:

    key(x) = the fully-qualified name with the parameter list, return type,
             access specifier, calling convention and cv-qualifiers stripped,
             and all whitespace removed.

Decoration is undone with `llvm-undname` (batched over stdin). The key is
deliberately *lossy*: overloads collapse onto one key. Callers must handle the
resulting many-to-one ambiguity; `Matcher` reports it rather than guessing.
"""
import re
import subprocess

UNDNAME = "llvm-undname"

_CALLCONV = re.compile(r"\b__(cdecl|stdcall|fastcall|thiscall|vectorcall|clrcall)\b")


def undname_batch(names):
    """Demangle a list of MSVC decorated names. Returns {name: demangled|None}.

    `llvm-undname` echoes the input line then the demangled line then a blank
    line; names it cannot demangle are echoed unchanged.
    """
    names = list(names)
    if not names:
        return {}
    p = subprocess.run(
        [UNDNAME], input="\n".join(names) + "\n",
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        encoding="latin1", timeout=300,
    )
    lines = p.stdout.split("\n")
    out = {}
    i = 0
    for n in names:
        # locate the echo of `n`
        while i < len(lines) and lines[i] != n:
            i += 1
        if i >= len(lines):
            out[n] = None
            continue
        d = lines[i + 1] if i + 1 < len(lines) else ""
        out[n] = None if (d == "" or d == n) else d
        i += 2
    return out


def qualname_from_demangled(d):
    """Strip access/static/virtual/return-type/callconv/params off a demangled
    name, leaving the qualified name. Returns None if it does not look like a
    function."""
    if d is None:
        return None
    s = d
    # `public: virtual `, `protected: static `, ...
    s = re.sub(r"^(public|protected|private):\s*", "", s)
    s = re.sub(r"^(static|virtual)\s+", "", s)
    s = re.sub(r"^(static|virtual)\s+", "", s)
    m = _CALLCONV.search(s)
    if m:
        s = s[m.end():].lstrip()
    else:
        # No calling convention => not a function (data symbol, vftable, RTTI
        # descriptor, string literal, ...).  `[thunk]:` forms and adjustor
        # thunks do carry one, so they survive.
        return None
    # Now s == QUALNAME(params)cv...  Walk to the first top-level '('.
    depth_ang = 0
    depth_br = 0
    in_tick = 0
    i = 0
    n = len(s)
    while i < n:
        c = s[i]
        if c == "`":
            in_tick += 1
        elif c == "'" and in_tick:
            in_tick -= 1
        elif in_tick:
            pass
        elif c == "<":
            depth_ang += 1
        elif c == ">":
            depth_ang -= 1
        elif c == "[":
            depth_br += 1
        elif c == "]":
            depth_br -= 1
        elif c == "(" and depth_ang <= 0 and depth_br <= 0:
            # `operator()` / `operator ()` — the parameter list is the NEXT group
            tail = s[:i].rstrip()
            if tail.endswith("operator"):
                j = s.find(")", i)
                if j >= 0:
                    i = j + 1
                    continue
            break
        i += 1
    return s[:i].strip()


def key(qual):
    """Canonical comparison key: whitespace removed."""
    if qual is None:
        return None
    return re.sub(r"\s+", "", qual)


def decorated_key(name, demangled):
    return key(qualname_from_demangled(demangled))


# ---------------------------------------------------------------- warnings

_WARN = re.compile(
    r"^(?P<file>.*?)\((?P<line>\d+)\)\s*:\s*warning\s+C(?P<code>4505|4514):\s*"
    r"'(?P<name>.*)'\s*:\s*unreferenced\s+(local|inline)\s+function\s+has\s+been\s+removed"
)


def parse_removals(text):
    """Parse a cl.exe log for C4505/C4514. Returns a list of
    dicts(file, line, code, name, key)."""
    out = []
    for ln in text.replace("\r", "").split("\n"):
        m = _WARN.match(ln.strip())
        if not m:
            continue
        f = m.group("file").replace("\\", "/").lower()
        f = re.sub(r"^z:/home/free/code/milohax/dc3-decomp/", "", f)
        out.append({
            "file": f,
            "line": int(m.group("line")),
            "code": m.group("code"),
            "name": m.group("name"),
            "key": key(m.group("name")),
        })
    return out

#!/usr/bin/env python3
"""glread.py — the CRATE-FREE second instrument for lane w-align. Read-only.

Two independent readings of one cell, printed side by side so they can DISAGREE
with the production cursor rather than merely echo it:

  1. the `.gl` DATA record frame, re-implemented from `data_object_at`'s own
     documented grammar (`crates/c2-il/src/func/gl.rs`) and NOT by calling it:

         <tag> [mark] <kind> 00 <02|04> <linkage> <size varint> <attr> <flags>

  2. the TRUTH, from c2's own obj — the section `Characteristics` alignment
     nibble (bits 23:20) of whatever section c2 put the object's symbol in.
     This is the oracle the whole lane is graded against; nothing here is
     inferred from the IL.

Derived from `work/w-rdata3/p01/glhex.py` (the standing `.gl` hexdumper), which
is why this file does not re-invent the run scanner.

    glread.py <capdir> [<capdir> ...]
"""
import os
import struct
import sys

ALIGN_OF_NIBBLE = {n: 1 << (n - 1) for n in range(1, 15)}


def graphic_runs(gl):
    """Every NUL-terminated run of printable non-space bytes: (start, nul, text)."""
    out, i, n = [], 0, len(gl)
    while i < n:
        if not (0x21 <= gl[i] <= 0x7E):
            i += 1
            continue
        s = i
        while i < n and 0x21 <= gl[i] <= 0x7E:
            i += 1
        if i < n and gl[i] == 0:
            out.append((s, i, gl[s:i].decode("latin1")))
    return out


def read_varint(b, p):
    """The SAME encoding `readers::read_varint` reads: `80` + LE i32, else i8."""
    if p >= len(b):
        return None, p
    m = b[p]
    if m == 0x80:
        if p + 5 > len(b):
            return None, p
        return struct.unpack_from("<i", b, p + 1)[0], p + 5
    return (m - 256 if m >= 0x80 else m), p + 1


def data_record(gl, nul):
    """`data_object_at`'s frame, re-implemented. Returns a dict or a refusal."""
    def at(i):
        return gl[i] if 0 <= i < len(gl) else None

    r = {"tag": at(nul + 1), "mark": None, "kind": None, "frame": None,
         "linkage": None, "size": None, "attr": None, "flags": None,
         "refused_at": None}
    tag = r["tag"]
    if tag is None or not (tag & 0x80):
        r["refused_at"] = "tag-bit7"
        return r
    i = nul + 2
    if tag & 0x40:
        r["mark"] = at(i)
        if r["mark"] is None or not (r["mark"] & 0x80):
            r["refused_at"] = "wide-mark"
            return r
        i += 1
    r["kind"] = at(i)
    i += 1
    r["frame"] = (at(i), at(i + 1))
    if r["frame"] != (0x00, 0x02):
        r["refused_at"] = "frame"
        return r
    r["linkage"] = at(i + 2)
    if r["linkage"] not in (0x01, 0x04):
        r["refused_at"] = "linkage"
        return r
    size, p = read_varint(gl, i + 3)
    r["size"] = size
    if size is None or size <= 0:
        r["refused_at"] = "size"
        return r
    r["attr"] = at(p)
    r["flags"] = at(p + 1)
    if r["attr"] not in (0x00, 0x80):
        r["refused_at"] = "attr"
        return r
    # The ONLY gate this lane is about.
    if (tag & ~0x40) not in (0x82, 0x84, 0x86, 0x88):
        r["refused_at"] = "align-tag"
    return r


# ---------------------------------------------------------------- COFF oracle
def coff_truth(path):
    """symbol name -> (section name, align bytes, section size), from the real obj."""
    b = open(path, "rb").read()
    nsec, symptr, nsym = struct.unpack_from("<H", b, 2)[0], \
        struct.unpack_from("<I", b, 8)[0], struct.unpack_from("<I", b, 12)[0]
    opt = struct.unpack_from("<H", b, 16)[0]
    sec_off = 20 + opt
    strtab = symptr + 18 * nsym
    secs = []
    for k in range(nsec):
        o = sec_off + 40 * k
        raw = b[o:o + 8]
        if raw[0:1] == b"/":
            name = cstr(b, strtab + int(raw[1:].rstrip(b"\0").decode()))
        else:
            name = raw.rstrip(b"\0").decode("latin1")
        size = struct.unpack_from("<I", b, o + 16)[0]
        ch = struct.unpack_from("<I", b, o + 36)[0]
        secs.append((name, size, (ch >> 20) & 0xF))
    out = {}
    k = 0
    while k < nsym:
        o = symptr + 18 * k
        raw = b[o:o + 8]
        if raw[0:4] == b"\0\0\0\0":
            name = cstr(b, strtab + struct.unpack_from("<I", b, o + 4)[0])
        else:
            name = raw.rstrip(b"\0").decode("latin1")
        secnum = struct.unpack_from("<h", b, o + 12)[0]
        naux = b[o + 17]
        if 1 <= secnum <= nsec:
            nm, sz, nib = secs[secnum - 1]
            out.setdefault(name, (nm, ALIGN_OF_NIBBLE.get(nib), sz, nib))
        k += 1 + naux
    return out, [s[0] for s in secs]


def cstr(b, o):
    e = b.index(b"\0", o)
    return b[o:e].decode("latin1")


def main():
    for d in sys.argv[1:]:
        cell = os.path.basename(d.rstrip("/"))
        gls = [f for f in os.listdir(d) if f.endswith(".gl")]
        if not gls:
            print(f"{cell}\tNO-GL")
            continue
        gl = open(os.path.join(d, gls[0]), "rb").read()
        truth, secnames = coff_truth(os.path.join(d, "ref.obj"))
        print(f"== {cell}   sections[{len(secnames)}]: {' '.join(secnames)}")
        for s, nul, txt in graphic_runs(gl):
            # `24` (`$`) is the `.gl` NAME SEPARATOR for an internal-linkage
            # object, not part of the name (`wr1c_dyninit_extern.cpp`'s header).
            # The production run scanner opens the run AFTER it, so the byte is
            # stripped here rather than the run skipped — skipping it made the
            # crate-free instrument blind to every `static` object, which is a
            # coverage hole that reads exactly like a disagreement.
            if txt.startswith("$"):
                txt = txt[1:]
            if not txt or not (txt[0] == "?" or txt[0] == "_" or txt[0].isalpha()):
                continue
            r = data_record(gl, nul)
            if r["tag"] is None or not (r["tag"] & 0x80):
                continue
            if r["refused_at"] in ("frame", "linkage", "size", "wide-mark"):
                continue          # not an ORDINARY-DATA record at all
            if r["frame"] is None or None in r["frame"]:
                continue          # ran off the end of the container
            t = truth.get(txt)
            hexes = gl[nul + 1:nul + 12].hex(" ")
            print(f"   {txt}")
            print(f"      gl: tag={r['tag']:02x} mark={fmt(r['mark'])} "
                  f"kind={fmt(r['kind'])} frame={r['frame'][0]:02x} "
                  f"{r['frame'][1]:02x} link={fmt(r['linkage'])} "
                  f"size={r['size']} attr={fmt(r['attr'])} flags={fmt(r['flags'])} "
                  f"refused={r['refused_at']}")
            print(f"      raw: {hexes}")
            if t:
                print(f"      c2: section={t[0]} align={t[1]} "
                      f"secsize={t[2]} nibble={t[3]}")
            else:
                print("      c2: (symbol not defined in a section of ref.obj)")


def fmt(v):
    return "--" if v is None else f"{v:02x}"


main()

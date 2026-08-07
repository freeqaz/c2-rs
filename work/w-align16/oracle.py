#!/usr/bin/env python3
"""oracle.py — lane w-align16's crate-free reconciliation. Read-only.

`glread.py` prints the tag's WIDTH and the SECTION's alignment nibble side by
side. Those are two different quantities and comparing them directly produces
four false disagreements (`char[4096]` reads tag `82` and sits in an ALIGN_8
section; `A13`'s one-byte `char` sits in the ALIGN_16 section its 16-aligned
neighbour forced). This file closes that gap by applying the port's own model:

    natural   = TAG_WIDTH[tag & ~0x40]                 <- read, never inferred
    placement = max(natural, 1 if n<2 else 4 if n<64 else 8)
    section   = max over the objects the section holds  (Rule B1)
    offsets   = a bump over the .gl walk, each rounded up to `placement`
                (Rule A3')

and then compares **section nibble** and **every symbol's `Value`** against c2's
own obj. Nothing here calls the crate; the promotion table is re-typed from
`container.rs`'s doc comment, which is what makes it a second instrument.

    oracle.py <capdir> [<capdir> ...]
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glread import graphic_runs, data_record, coff_truth, cstr, TAG_WIDTH  # noqa: E402


def placement_align(n, natural):
    implied = 1 if n < 2 else (4 if n < 64 else 8)
    return max(natural, implied)


def coff_full(path):
    """(sections, symbols) — sections as (name, size, nibble), symbols as
    (name, secnum, value)."""
    b = open(path, "rb").read()
    nsec = struct.unpack_from("<H", b, 2)[0]
    symptr = struct.unpack_from("<I", b, 8)[0]
    nsym = struct.unpack_from("<I", b, 12)[0]
    opt = struct.unpack_from("<H", b, 16)[0]
    sec_off = 20 + opt
    strtab = symptr + 18 * nsym
    secs = []
    for k in range(nsec):
        o = sec_off + 40 * k
        raw = b[o:o + 8]
        name = (cstr(b, strtab + int(raw[1:].rstrip(b"\0").decode()))
                if raw[0:1] == b"/" else raw.rstrip(b"\0").decode("latin1"))
        secs.append((name,
                     struct.unpack_from("<I", b, o + 16)[0],
                     (struct.unpack_from("<I", b, o + 36)[0] >> 20) & 0xF))
    syms, k = [], 0
    while k < nsym:
        o = symptr + 18 * k
        raw = b[o:o + 8]
        name = (cstr(b, strtab + struct.unpack_from("<I", b, o + 4)[0])
                if raw[0:4] == b"\0\0\0\0" else raw.rstrip(b"\0").decode("latin1"))
        syms.append((name,
                     struct.unpack_from("<h", b, o + 12)[0],
                     struct.unpack_from("<I", b, o + 8)[0]))
        k += 1 + b[o + 17]
    return secs, syms


def objects_of(capdir):
    """The `.gl` DATA records this lane reads, in `.gl` record order."""
    gls = [f for f in os.listdir(capdir) if f.endswith(".gl")]
    if not gls:
        return None
    gl = open(os.path.join(capdir, gls[0]), "rb").read()
    out = []
    for _s, nul, txt in graphic_runs(gl):
        if txt.startswith("$"):
            txt = txt[1:]
        if not txt or not (txt[0] == "?" or txt[0] == "_" or txt[0].isalpha()):
            continue
        r = data_record(gl, nul)
        if r["tag"] is None or not (r["tag"] & 0x80):
            continue
        if r["refused_at"] in ("frame", "linkage", "size", "wide-mark", "attr"):
            continue
        if r["tagalign"] is None:
            continue
        out.append((txt, r["size"], r["tagalign"], r["tag"]))
    return out


def main():
    tot = {"sec-ok": 0, "sec-bad": 0, "val-ok": 0, "val-bad": 0, "no-sym": 0}
    for d in sys.argv[1:]:
        cell = os.path.basename(d.rstrip("/"))
        objs = objects_of(d)
        ref = os.path.join(d, "ref.obj")
        if objs is None or not os.path.exists(ref):
            print(f"== {cell}  NO-CAPTURE")
            continue
        secs, syms = coff_full(ref)
        by_name = {}
        for n, sn, v in syms:
            if 1 <= sn <= len(secs) and n not in by_name:
                by_name[n] = (sn, v)
        # group the objects this lane reads by the section c2 put them in
        groups = {}
        for name, size, nat, tag in objs:
            if name not in by_name:
                tot["no-sym"] += 1
                continue
            sn, val = by_name[name]
            groups.setdefault(sn, []).append((name, size, nat, tag, val))
        print(f"== {cell}")
        for sn in sorted(groups):
            secname, secsize, nib = secs[sn - 1]
            members = groups[sn]
            want = max(placement_align(s, na) for _n, s, na, _t, _v in members)
            got = 1 << (nib - 1)
            ok = "OK " if want == got else "BAD"
            tot["sec-ok" if want == got else "sec-bad"] += 1
            print(f"   {secname:9s} nibble={nib} c2align={got:<3d} "
                  f"predicted={want:<3d} {ok}   objs={len(members)}")
            cursor = 0
            for name, size, nat, tag, val in members:
                a = placement_align(size, nat)
                at = (cursor + a - 1) // a * a
                cursor = at + size
                vok = "OK " if at == val else "BAD"
                tot["val-ok" if at == val else "val-bad"] += 1
                print(f"      {name:22s} tag={tag:02x} nat={nat:<3d} n={size:<5d} "
                      f"place={a:<3d} value={val:<5d} predicted={at:<5d} {vok}")
    print()
    print("TOTALS", tot)


main()

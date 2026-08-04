#!/usr/bin/env python3
"""Print the `.text` layout of one or two COFF objs: sections, then every
function/label symbol in address order.

Used by `order.sh` to read the packed-`.text` function ORDER straight off the
oracle's obj — the axis `docs/rungs/2026-08-04-w-cross-sep26.md` §3 left
unmeasured. Deliberately dumb: no IL, no heuristics, just the symbol table.
"""
import struct
import sys


def read(path):
    d = open(path, "rb").read()
    nsec = struct.unpack_from("<H", d, 2)[0]
    symptr, nsym = struct.unpack_from("<II", d, 8)
    secs = []
    for i in range(nsec):
        o = 20 + 40 * i
        name = d[o : o + 8].split(b"\0")[0].decode("latin1")
        size, ptr = struct.unpack_from("<II", d, o + 16)
        chars = struct.unpack_from("<I", d, o + 36)[0]
        secs.append((name, size, chars))
    strtab_off = symptr + 18 * nsym
    syms = []
    i = 0
    while i < nsym:
        o = symptr + 18 * i
        raw = d[o : o + 8]
        if raw[:4] == b"\0\0\0\0":
            off = struct.unpack_from("<I", raw, 4)[0]
            end = d.index(b"\0", strtab_off + off)
            name = d[strtab_off + off : end].decode("latin1")
        else:
            name = raw.split(b"\0")[0].decode("latin1")
        val, sec, typ, sc, naux = struct.unpack_from("<IhHBB", d, o + 8)
        syms.append((name, val, sec, sc, typ))
        i += 1 + naux
    return len(d), secs, syms


def layout(path):
    total, secs, syms = read(path)
    # every DEFINED symbol in a .text section, in address order
    text_idx = [i + 1 for i, s in enumerate(secs) if s[0].startswith(".text")]
    fns = [
        (sec, val, name)
        for (name, val, sec, sc, typ) in syms
        if sec in text_idx and sc == 2 and typ == 0x20
    ]
    fns.sort()
    return total, secs, fns


def show(tag, path):
    total, secs, fns = layout(path)
    tsec = [(n, sz, c) for (n, sz, c) in secs if n.startswith(".text")]
    print(
        "  %-5s total=%-5d sections=%d  .text=%s"
        % (tag, total, len(secs), [(n, sz, hex(c)) for n, sz, c in tsec])
    )
    print("        order: " + " ".join("%s@%d.%d" % (n, s, v) for s, v, n in fns))


if __name__ == "__main__":
    label = sys.argv[1]
    print("== %s" % label)
    show("ref", sys.argv[2])
    if len(sys.argv) > 3:
        try:
            show("port", sys.argv[3])
        except (OSError, IndexError, struct.error):
            print("  port  (no obj — refused)")

#!/usr/bin/env python3
"""objrec.py — reconcile the fresh reference obj against the cached one.

Prereg **Q15** asked for a whole-obj byte match with the `TimeDateStamp` zeroed
and got **0 of 21**.  This decomposes the difference instead of shrugging at it,
and separates the two questions Q15 conflated:

  * **Q15 as written** — whole-obj bytes equal.  It is a badly-aimed control:
    `work/w-db/cacheindex.py`'s own docstring records that duplicate entries for
    one TU are *"byte-identical apart from the `-Fo` path the obj embeds in
    `S_OBJNAME`"*, and this lane's `-Fo` is its own output directory while the
    cached obj's is the cache entry.  A control that cannot pass is not a
    control.

  * **Q15′ — the property the gate actually needs**: does the *truth* differ?
    `E(fresh) == E(cached)` and `D_all(fresh) == D_all(cached)`, per TU, by name.
    If those agree, the fresh judge and the 850's cached truth are one
    instrument and the comparison across populations is sound.

Also localized: the number of differing byte runs, their offsets, and which
COFF section each falls in — so "it is only the path" is measured, not asserted.

    usage: objrec.py <objdir> <cache-index.tsv>
"""
import os
import struct
import sys

MAIN = os.environ.get("C2RS_LANEROOT")
if not MAIN:
    raise SystemExit("set C2RS_LANEROOT")
sys.path.insert(0, os.path.join(MAIN, "work", "w-joint"))
import objsyms  # noqa: E402


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def zero_ts(b):
    return b[:4] + b"\0\0\0\0" + b[8:]


def sections(b):
    """(name, rawptr, rawsize) for every section, from the COFF header."""
    nsec, = struct.unpack_from("<H", b, 2)
    optlen, = struct.unpack_from("<H", b, 16)
    off = 20 + optlen
    out = []
    for i in range(nsec):
        rec = b[off + i * 40: off + (i + 1) * 40]
        if len(rec) < 40:
            break
        nm = rec[:8].rstrip(b"\0").decode("latin1", "replace")
        size, ptr = struct.unpack_from("<II", rec, 16)
        out.append((nm, ptr, size))
    return out


def runs(a, b):
    """Contiguous differing byte ranges between two equal-length buffers."""
    out = []
    i, n = 0, min(len(a), len(b))
    while i < n:
        if a[i] != b[i]:
            j = i
            while j < n and a[j] != b[j]:
                j += 1
            out.append((i, j - i))
            i = j
        else:
            i += 1
    if len(a) != len(b):
        out.append((n, abs(len(a) - len(b))))
    return out


def where(secs, off):
    for nm, ptr, size in secs:
        if ptr and ptr <= off < ptr + size:
            return nm
    return "<header/table>"


def sec_contents(b):
    """[(name, raw bytes)] in section-table order — the comparison a FILE-OFFSET
    diff cannot make once one section's size has changed and shifted the rest."""
    return [(nm, b[ptr:ptr + size] if ptr else b"")
            for nm, ptr, size in sections(b)]


def main():
    objdir, idxp = sys.argv[1], sys.argv[2]
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]
    nE = nD = 0
    allsecs = {}
    sec_bad = {}
    n_sec_ok = 0
    print("%-58s %5s %6s %8s  %s"
          % ("src", "E==", "D_all==", "diffruns", "sections carrying a difference"))
    for r in rows:
        src, entry = r[0], r[1]
        fresh = open(os.path.join(objdir, slug(src) + ".obj"), "rb").read()
        cached = open(os.path.join(entry, "out.obj"), "rb").read()
        sf, sc = objsyms.sets(objsyms.ObjSyms(fresh)), \
            objsyms.sets(objsyms.ObjSyms(cached))
        eq_e = sf["E"] == sc["E"]
        eq_d = sf["D_all"] == sc["D_all"]
        nE += 1 if eq_e else 0
        nD += 1 if eq_d else 0
        rr = runs(zero_ts(fresh), zero_ts(cached))
        secs = sections(fresh)
        hit = {}
        for off, ln in rr:
            hit[where(secs, off)] = hit.get(where(secs, off), 0) + ln
            allsecs[where(secs, off)] = allsecs.get(where(secs, off), 0) + ln
        print("%-58s %5s %6s %8d  %s"
              % (src[:58], "YES" if eq_e else "NO", "YES" if eq_d else "NO",
                 len(rr), sorted(hit.items())))
        # per-SECTION content compare — immune to the whole-file shift a
        # different-length `-Fo` string causes
        cf, cc = sec_contents(fresh), sec_contents(cached)
        if [n for n, _ in cf] == [n for n, _ in cc]:
            bad = [n for (n, a), (_, c) in zip(cf, cc) if a != c]
            if not bad:
                n_sec_ok += 1
            for n in bad:
                sec_bad[n] = sec_bad.get(n, 0) + 1
        else:
            sec_bad["<section table differs>"] = \
                sec_bad.get("<section table differs>", 0) + 1

    print("\nQ15-prime  E(fresh) == E(cached): %d/%d ;  D_all: %d/%d"
          % (nE, len(rows), nD, len(rows)))
    print("           every differing byte at a FILE OFFSET, by section: %s"
          % sorted(allsecs.items(), key=lambda kv: -kv[1]))
    print("\nQ15-prime-prime  per-SECTION content identical on ALL sections: "
          "%d/%d" % (n_sec_ok, len(rows)))
    print("                 sections that ever differ (TU count): %s"
          % sorted(sec_bad.items(), key=lambda kv: -kv[1]))


if __name__ == "__main__":
    main()

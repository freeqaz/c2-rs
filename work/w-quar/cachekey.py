#!/usr/bin/env python3
"""cachekey.py — recompute a c2rs capture-cache entry name from its material.

The harness's key is `digest128(material)` (crates/c2-harness/src/capture_cache.rs
`key_material` + `digest128`), and the material is stored verbatim as the entry's
`key.bin`.  So an entry can be LOCATED by construction instead of by scanning a
6.2-million-entry directory: take one known entry's `key.bin` as the template,
substitute `src-arg` and `src-bytes` for another TU at the same dc3 rev, hash.

The located entry is then VERIFIED by reading its `key.bin` back and comparing
byte for byte; a constructed key that does not round-trip is reported, never used.

    usage: cachekey.py <template-key.bin> <dc3-repo> <rev> <tulist> <out.tsv>

Reads `key.bin` only.  Never opens `out.obj` — quarantine-safe by construction.
"""
import os
import subprocess
import sys

FNV_OFFSET = 0xCBF29CE484222325
MASK = (1 << 64) - 1


def fnv1a64(seed, data):
    h = seed & MASK
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & MASK
    return h


def digest128(data):
    h1 = fnv1a64(FNV_OFFSET, data)
    h2 = fnv1a64(h1 ^ 0x9E3779B97F4A7C15, data[::-1])
    return "%016x%016x" % (h1, h2)


def file_digest(b):
    return "%d:%s" % (len(b), digest128(b))


def main():
    tmpl, dc3, rev, tulist, outp = sys.argv[1:6]
    material = open(tmpl, "rb").read()
    head, rest = material.split(b"src-arg\x00", 1)
    _oldsrc, rest2 = rest.split(b"\x00src-bytes\x00", 1)
    _olddig, tail = rest2.split(b"\x00cwd\x00", 1)
    tail = b"\x00cwd\x00" + tail

    cache = os.path.dirname(os.path.dirname(os.path.abspath(tmpl)))
    srcs = [l.strip() for l in open(tulist) if l.strip()]
    rows, bad = [], []
    for s in srcs:
        try:
            blob = subprocess.check_output(
                ["git", "-C", dc3, "show", "%s:%s" % (rev, s)])
        except subprocess.CalledProcessError:
            bad.append((s, "no blob at %s" % rev))
            continue
        m = head + b"src-arg\x00" + s.encode() + b"\x00src-bytes\x00" \
            + file_digest(blob).encode() + tail
        key = digest128(m)
        d = os.path.join(cache, key)
        kb = os.path.join(d, "key.bin")
        if not os.path.exists(kb):
            bad.append((s, "no entry %s" % key))
            continue
        if open(kb, "rb").read() != m:
            bad.append((s, "key.bin MISMATCH at %s" % key))
            continue
        base = None
        for n in os.listdir(d):
            if n.startswith("_CL_") and n.endswith("gl") and len(n) == 14:
                base = n[:-2]
        have_obj = os.path.exists(os.path.join(d, "out.obj"))
        if base is None:
            bad.append((s, "no IL quintet in %s" % key))
            continue
        rows.append((s, d, base, have_obj))

    with open(outp, "w") as fh:
        for s, d, base, have_obj in rows:
            fh.write("%s\t%s\t%s\t%d\n" % (s, d, base, have_obj))
    print("requested %d ; LOCATED+VERIFIED %d ; failed %d"
          % (len(srcs), len(rows), len(bad)))
    for s, why in bad:
        print("  FAIL %s : %s" % (s, why))


if __name__ == "__main__":
    main()

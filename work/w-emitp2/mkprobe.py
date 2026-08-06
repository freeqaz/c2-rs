#!/usr/bin/env python3
"""mkprobe.py — a symlink farm so `crates/c2-il`'s OWN `.in` reader can be run
over the same 850 TUs the emit-predicate channel is measured on.

`crates/c2-il/tests/in_init_probe.rs` keys on the file **extension**, and the
capture cache spells its members `_CL_<hash>in` with no dot.  So each TU gets a
directory of five symlinks named `il.<suffix>`.  Nothing is copied and nothing
is written into the cache.

    usage: mkprobe.py <cacheidx.tsv> <outdir>
"""
import os
import sys

SUFFIXES = ("gl", "ex", "in", "sy", "db")


def main():
    idxp, outdir = sys.argv[1:3]
    os.makedirs(outdir, exist_ok=True)
    n = 0
    for line in open(idxp):
        src, entry = line.rstrip("\n").split("\t")[:2]
        cell = src.replace("/", "__").replace("\\", "__")
        d = os.path.join(outdir, cell)
        os.makedirs(d, exist_ok=True)
        try:
            names = os.listdir(entry)
        except OSError:
            continue
        for sfx in SUFFIXES:
            for nm in names:
                if nm.startswith("_CL_") and nm.endswith(sfx):
                    link = os.path.join(d, "il." + sfx)
                    if not os.path.islink(link):
                        os.symlink(os.path.join(entry, nm), link)
                    break
        n += 1
    print("probe cells %d in %s" % (n, outdir))


if __name__ == "__main__":
    main()

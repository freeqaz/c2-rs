#!/usr/bin/env python3
"""dump08.py — print whole records carrying an element tag `08`, by owner
family, so the fill's UNIT can be reasoned about against a known ABI struct.

**A DRIVER, NOT EVIDENCE** — same status as `localize.py`.

    usage: dump08.py <cacheidx.tsv> <family-prefix> [n] [tulimit]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import localize as L  # noqa: E402


def main():
    idxp, pfx = sys.argv[1], sys.argv[2]
    want = int(sys.argv[3]) if len(sys.argv) > 3 else 8
    tul = int(sys.argv[4]) if len(sys.argv) > 4 else 8
    seen = set()
    n = 0
    for line in list(open(idxp))[:tul]:
        src, entry = line.rstrip("\n").split("\t")[:2]
        inp, glp = L.member(entry, "in"), L.member(entry, "gl")
        if inp is None or glp is None:
            continue
        idx = L.il.gl_symbol_index(open(glp, "rb").read())
        for owner, el in L.parse_v(open(inp, "rb").read()):
            nm = idx.get(owner)
            if not nm or not nm.startswith(pfx):
                continue
            if nm in seen:
                continue
            seen.add(nm)
            print("  %s" % nm[:130])
            print("      %s" % L.fmt_el(el)[:500])
            n += 1
            if n >= want:
                return


if __name__ == "__main__":
    main()

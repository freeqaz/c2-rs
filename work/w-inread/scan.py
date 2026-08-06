#!/usr/bin/env python3
"""scan.py — INSTRUMENT 1 for the w-inread grid: a crate-free record parser
over the captured `.in` streams, with every record's owner resolved to its `.gl`
name and every element spelled with its raw bytes.

Written from `work/w-tag02/GRAMMAR.md` and `work/w-mark/instream.py`'s record
framing; it imports **nothing from `crates/`**.  Instrument 2 is the shipping
reader's own cursor (`crates/c2-il/tests/in_init_probe.rs`), and
`work/w-inread/two_instruments.py` reconciles them.  Neither may be the other's
witness.

    usage: scan.py [cell...]        (default: every cell in grid_list.txt)

stdlib only.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
import localize as L  # noqa: E402


def hx(b):
    return " ".join("%02x" % c for c in b)


def spell(b, recs_off):
    """Re-walk the stream keeping BYTE SPANS, so every element can be printed
    exactly as it is spelled rather than as it is decoded."""
    return None


def one(cell, verbose=True):
    d = os.path.join(HERE, "il", cell)
    inb = glb = None
    for nm in os.listdir(d):
        p = os.path.join(d, nm)
        if nm.endswith(".in"):
            inb = open(p, "rb").read()
        elif nm.endswith(".gl"):
            glb = open(p, "rb").read()
    if inb is None:
        print("  %s NO-IN" % cell)
        return {}
    idx = L.il.gl_symbol_index(glb) if glb else {}
    recs = L.parse_v(inb)
    st = {"rec": len(recs), "e01": 0, "e02": 0, "e03": 0, "e08": 0,
          "t03": 0, "t04": 0, "bytes": len(inb)}
    for owner, el in recs:
        for k, a, w, v in el:
            if k == 0x01:
                st["e01"] += 1
                if a == 0x03:
                    st["t03"] += 1
                elif a == 0x04:
                    st["t04"] += 1
            elif k == 0x02:
                st["e02"] += 1
            elif k == 0x03:
                st["e03"] += 1
            elif k == 0x08:
                st["e08"] += 1
        if verbose:
            nm = idx.get(owner, "tok=%04x?" % owner)
            print("    %-56s %s" % (nm[:56], L.fmt_el(el)[:220]))
    return st


def main():
    cells = sys.argv[1:]
    if not cells:
        cells = [l.strip()[:-4] for l in
                 open(os.path.join(HERE, "grid_list.txt")) if l.strip()]
    tot = {}
    for c in cells:
        print("## %s" % c)
        st = one(c)
        for k, v in st.items():
            tot[k] = tot.get(k, 0) + v
        print("   %s" % " ".join("%s=%d" % (k, st[k]) for k in sorted(st)))
        print()
    print("== TOTAL over %d cells ==" % len(cells))
    print("   %s" % " ".join("%s=%d" % (k, tot[k]) for k in sorted(tot)))


if __name__ == "__main__":
    main()

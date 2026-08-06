#!/usr/bin/env python3
"""w-tag02 — reconcile the TWO instruments, cell by cell.

* **Instrument 1** is `work/w-tag02/scan.py`: a forward record parser over the
  whole `.in`, written from the grammar, importing nothing from the crate.
* **Instrument 2** is `crates/c2-il/tests/in_init_probe.rs`: the production
  reader's own cursor, reported through `IlBundle::in_init_report`.

Neither is the other's witness. This script only compares them, and it prints
COUNTS rather than a status — `docs/STATUS.md` trap 5, absence reads as success.

The comparable quantity is **tag-02 elements in records the reader can accept
whole**: instrument 1 sees every tag-02 element in the stream, instrument 2 only
those in records whose every element it models. The two differ by exactly the
records carrying a tag-`03` (byte-string) element, which the reader refuses — so
this script computes instrument 1's number BOTH ways and reports both.

Usage:
    python3 work/w-tag02/two_instruments.py <instrument2-output-file>

where the second file is the stdout of

    C2RS_IN_PROBE=<abs>/work/w-tag02/il \\
      cargo test -p c2-il --release --test in_init_probe -- --nocapture
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scan  # instrument 1  # noqa: E402

IL = scan.IL


def instrument1(cell):
    """(all tag-02 elements, tag-02 elements in reader-acceptable records)."""
    import glob
    ins = glob.glob(os.path.join(IL, cell, "*.in"))
    if not ins:
        return None
    recs, _ = scan.scan(open(ins[0], "rb").read())
    total = 0
    acceptable = 0
    for (_at, _tok, elems, _end) in recs:
        n02 = sum(1 for k, _ in elems if k == "02")
        total += n02
        # The reader models tags 01 and 02 only, admits scalar types 01/02 at
        # widths 1/2/4, and requires a tag-02 `<n>` of exactly 4.
        ok = all(
            (k == "01" and e["type"] in (1, 2) and e["width"] in (1, 2, 4))
            or (k == "02" and e["n"] == 4)
            for k, e in elems
        )
        if ok:
            acceptable += n02
    return total, acceptable


def instrument2(path):
    out = {}
    for line in open(path):
        f = line.split("\t")
        if len(f) < 2 or not f[1].startswith("records="):
            continue
        kv = dict(p.split("=", 1) for p in f[1].split() if "=" in p and not p.startswith("["))
        out[f[0]] = int(kv["symrefs"])
    return out


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    i2 = instrument2(sys.argv[1])
    cells = sorted(i2)
    agree = disagree = 0
    print("%-24s %8s %8s %8s  %s" % ("cell", "i1-all", "i1-ok", "i2", "verdict"))
    for c in cells:
        r = instrument1(c)
        if r is None:
            print("%-24s %8s %8s %8d  NO-CAPTURE" % (c, "-", "-", i2[c]))
            continue
        total, ok = r
        v = "AGREE" if ok == i2[c] else "DISAGREE"
        if ok == i2[c]:
            agree += 1
        else:
            disagree += 1
        note = "" if total == ok else "  (%d more in records the reader refuses)" % (total - ok)
        print("%-24s %8d %8d %8d  %s%s" % (c, total, ok, i2[c], v, note))
    print("---")
    print("cells=%d agree=%d disagree=%d" % (len(cells), agree, disagree))
    print("i1 tag-02 elements, all=%d  reader-acceptable=%d" % (
        sum(instrument1(c)[0] for c in cells if instrument1(c)),
        sum(instrument1(c)[1] for c in cells if instrument1(c)),
    ))
    print("i2 symrefs total=%d" % sum(i2.values()))
    return 1 if disagree else 0


if __name__ == "__main__":
    sys.exit(main())

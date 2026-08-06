#!/usr/bin/env python3
"""cell_fbm.py — per-CELL FBM verdicts out of a `c2rs gap --jsonl` record.

Lane w-empty measurement tooling. **Read-only with respect to `crates/`.**

    cell_fbm.py <gap.jsonl> [<gap.jsonl>]

One row per cell: the `fnbyte-shape|…|fnbyte-…` keys the scan wrote for that TU,
plus every `fnbyte-differs-fn|…` witness. With two files it prints the cells
whose rows CHANGED, which is the before/after read this lane grades itself on.

**No absolute path from the provenance record is ever echoed** — the record is
skipped entirely (it carries the machine's own directories, which must not reach
a committed file).
"""

import json
import sys


def load(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        src = r.get("src")
        if src is None:
            continue
        cell = src.rsplit("/", 1)[-1].replace(".cpp", "")
        emit = r.get("emit") or {}
        shapes = sorted(
            (k.split("|", 1)[1], v)
            for k, v in emit.items()
            if k.startswith("fnbyte-shape|") and k.count("|") == 2
        )
        wits = sorted(k.split("|", 1)[1] for k in emit if k.startswith("fnbyte-differs-fn|"))
        out[cell] = {
            "shapes": shapes,
            "wits": wits,
            "class": r.get("class"),
        }
    return out


def fmt(v):
    return "  ".join("%s=%d" % (k, n) for k, n in v["shapes"])


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    a = load(argv[0])
    if len(argv) == 1:
        for cell in sorted(a):
            print("%-22s %-12s %s" % (cell, a[cell]["class"], fmt(a[cell])))
            for w in a[cell]["wits"]:
                print("%-22s   witness  %s" % ("", w))
        return 0
    b = load(argv[1])
    changed = 0
    for cell in sorted(set(a) | set(b)):
        ra, rb = a.get(cell), b.get(cell)
        if ra == rb:
            continue
        changed += 1
        print("%-22s BEFORE %s" % (cell, fmt(ra) if ra else "-"))
        for w in (ra or {}).get("wits", []):
            print("%-22s        witness %s" % ("", w))
        print("%-22s AFTER  %s" % ("", fmt(rb) if rb else "-"))
        for w in (rb or {}).get("wits", []):
            print("%-22s        witness %s" % ("", w))
    print("\ncells changed: %d of %d" % (changed, len(set(a) | set(b))))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

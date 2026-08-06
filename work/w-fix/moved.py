#!/usr/bin/env python3
"""moved.py — WHICH functions moved FBM verdict, by shape and by symbol family.

Lane w-fix measurement tooling. **Read-only with respect to `crates/`.**

    moved.py <before.jsonl> <after.jsonl>

`work/w-empty/wdiff.py` answers "how many left and how many entered", keyed per
`(TU, symbol)` off the scan's own witness keys. This adds the two breakdowns
`PREREG.md` P6 and P7 register:

* per **shape** — the claim is that every moved function is `shape=tail`, and a
  count that is not split by shape cannot say so;
* per **symbol family** — `w-empty` §11.3 is the standing caution that a large
  conversion can be one template, and board #925 is that caution with a number.
  Printing the families is how the next lane sees it without re-deriving it.

Both directions are printed with counts, and the ENTERED direction is printed
even when it is empty — a report that shows only the direction it wants is the
failure mode `docs/STATUS.md` trap 5 names.
"""

import json
import re
import sys


def load(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in r.get("emit") or {}:
            if k.startswith("fnbyte-differs-fn|"):
                _, shape, words, first, sym = k.split("|", 4)
                out[(r["src"], sym)] = (shape, words, first)
    return out


def family(sym):
    """The symbol's template/class family — everything before the first `@` of a
    template argument list, which is where one instantiation stops being the
    idiom and starts being its arguments."""
    m = re.match(r"^(\?\?[01]\?\$[A-Za-z_0-9]+)@", sym)
    if m:
        return m.group(1) + "@…"
    m = re.match(r"^(\?\?[01][A-Za-z_0-9]+)@@", sym)
    if m:
        return m.group(1) + "@@…"
    m = re.match(r"^(\?[A-Za-z_0-9]+)@", sym)
    if m:
        return m.group(1) + "@…"
    return sym[:40]


def report(title, keys, table):
    print("\n=== %s: %d ===" % (title, len(keys)))
    if not keys:
        print("  (none)")
        return
    by_shape = {}
    by_family = {}
    for k in keys:
        by_shape[table[k][0]] = by_shape.get(table[k][0], 0) + 1
        f = family(k[1])
        by_family[f] = by_family.get(f, 0) + 1
    print("  by shape:  " + "  ".join("%s %d" % kv for kv in sorted(by_shape.items())))
    print("  by family (top 10 of %d):" % len(by_family))
    for f, n in sorted(by_family.items(), key=lambda kv: -kv[1])[:10]:
        print("    %-56s %d" % (f[:56], n))
    print("  distinct TUs: %d" % len({k[0] for k in keys}))


def main(argv):
    a, b = load(argv[0]), load(argv[1])
    left = sorted(set(a) - set(b))
    entered = sorted(set(b) - set(a))
    print("differs BEFORE %d   AFTER %d   left %d   entered %d"
          % (len(a), len(b), len(left), len(entered)))
    report("ENTERED differs (the regression direction)", entered, b)
    report("LEFT differs (the conversion direction)", left, a)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

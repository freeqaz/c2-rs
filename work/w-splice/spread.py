#!/usr/bin/env python3
"""spread.py — the FAMILY SPREAD of the moved population (#925 / #952).

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

    spread.py <base.jsonl> <tip.jsonl>

Board **#925** and **#952** are the same caution twice: a rung that converted 143
functions converted 143 instances of **one** template, and a rung that reports a
count without a spread invites the reader to assume breadth it did not measure.
So this prints, for the converted population:

    distinct symbols, distinct TUs, distinct source directories
    the top idioms, with what fraction of the total they carry
    the TEMPLATE ROOT of each symbol — the text before the first `@?$` or `@` —
      which is the unit `#925` was about, since 62 instantiations of one class
      template are one idiom and not 62

A single-idiom result must SAY SO, which is what the concentration line is for.
"""

import collections
import json
import re
import sys


def load(path):
    d = collections.defaultdict(set)
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-differs-fn|"):
                d[r["src"]].add(k.split("|", 4)[4])
            elif k.startswith("fnbyte-differs-why|"):
                d[r["src"]].add(k.split("|", 5)[5])
    return d


def root(sym):
    """The template/class root of a mangled name — the coarse idiom unit.

    `??0?$_List_iterator@PAVObjectDir@@…` -> `??0?$_List_iterator` and
    `?front@?$vector@V?$Key@M@@…` -> `?front@?$vector`: one qualifier is kept so
    an accessor ON a template is not merged with that template's constructor,
    and the template ARGUMENTS are dropped so 62 instantiations of one class
    count as one idiom, which is what #925 was about.
    """
    parts = sym.split("@")
    if len(parts) > 1 and parts[1].startswith("?$"):
        return "%s@%s" % (parts[0], parts[1])
    return parts[0] if parts[0] else sym[:24]


def main():
    a, b = load(sys.argv[1]), load(sys.argv[2])
    moved = [(src, s) for src in a for s in a[src] - b.get(src, set())]
    n = len(moved)
    syms = collections.Counter(s for _, s in moved)
    tus = collections.Counter(src for src, _ in moved)
    dirs = collections.Counter("/".join(src.split("/")[:3]) for src, _ in moved)
    roots = collections.Counter(root(s) for _, s in moved)

    print("=== CONVERTED: %d functions ===" % n)
    print("  distinct symbols       : %d" % len(syms))
    print("  distinct TUs           : %d" % len(tus))
    print("  distinct source dirs   : %d" % len(dirs))
    print("  distinct template roots: %d" % len(roots))
    top = roots.most_common(1)[0] if roots else ("-", 0)
    print("  largest root carries   : %d of %d (%.1f%%)  %s"
          % (top[1], n, 100.0 * top[1] / n if n else 0, top[0]))
    print("  %s" % ("SINGLE-IDIOM RESULT — say so" if roots and top[1] > 0.8 * n
                    else "NOT a single idiom: the largest root is under 80%"))

    print("\n=== TEMPLATE ROOTS (the #925 unit) ===")
    for k, c in roots.most_common(15):
        print("  %5d  %5.1f%%  %s" % (c, 100.0 * c / n, k))
    print("\n=== SOURCE DIRECTORIES ===")
    for k, c in dirs.most_common(12):
        print("  %5d  %5.1f%%  %s" % (c, 100.0 * c / n, k))
    print("\n=== TUs, top 10 of %d ===" % len(tus))
    for k, c in tus.most_common(10):
        print("  %5d  %s" % (c, k))


if __name__ == "__main__":
    main()

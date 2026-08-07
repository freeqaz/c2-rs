#!/usr/bin/env python3
"""pop.py — how many DISTINCT symbols are behind the 123 and the 286?

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

    pop.py <scan.jsonl>

`w-seq` §4.2 published the two largest SPLICE-0 failure signatures as counts of
*pairs*: 286 source-register renames and 123 destination renames. Board **#925**
and **#952** are the standing caution that a count of pairs over 878 TUs is not
a count of idioms — 62 instantiations of one class template are one idiom and
not 62. This script asks that question of the two populations a rule would be
fitted on, by joining `fnbyte-splice0|<shape>|<verdict>|<witness>` (the counts)
against `fnbyte-splice0-fn|<shape>|<verdict>|<symbol>` (the names) per TU.

Printed: for every SPLICE-0 failure witness, the number of pairs, the number of
distinct symbols, the number of distinct TUs, and the number of distinct
TEMPLATE ROOTS (the mangled name truncated at its first template argument).
"""

import collections
import json
import sys


def root(sym):
    """The template root of a mangled name: everything before the first `@@`
    of the template argument list, or the whole name when it is not a
    template."""
    if sym.startswith("??$"):
        return sym[: sym.find("@", 3) + 1] if "@" in sym[3:] else sym
    i = sym.find("?$")
    if i < 0:
        return sym
    j = sym.find("@", i)
    return sym[: j + 1] if j > 0 else sym


def main():
    # per TU: witness -> count, and symbol -> 1, both keyed by (shape, verdict)
    wit = collections.Counter()
    fnpairs = collections.Counter()  # (shape, verdict, sym) -> pairs
    fntus = collections.defaultdict(set)
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k, v in (r.get("emit") or {}).items():
            if k.startswith("fnbyte-splice0|"):
                wit[k.split("|", 1)[1]] += v
            elif k.startswith("fnbyte-splice0-fn|"):
                _, shape, verdict, sym = k.split("|", 3)
                fnpairs[(shape, verdict, sym)] += v
                fntus[(shape, verdict, sym)].add(r["src"])

    print("=== SPLICE-0 witnesses, by pairs ===")
    for w, n in wit.most_common(12):
        print(f"  {n:5d}  {w}")

    print()
    print("=== SPLICE-0 `differs` symbols, by pairs ===")
    rows = [(n, k) for k, n in fnpairs.items() if k[1] == "differs"]
    rows.sort(reverse=True)
    tot = sum(n for n, _ in rows)
    print(f"  total differing pairs {tot} over {len(rows)} distinct (shape,symbol)")
    for n, (shape, _v, sym) in rows[:15]:
        print(f"  {n:5d}  {shape:8s} {len(fntus[(shape,_v,sym)]):4d} TUs  {sym[:110]}")

    print()
    print("=== the two target populations, as IDIOMS ===")
    for shape in ("framed", "tail", "seq"):
        rs = [(n, k) for n, k in rows if k[0] == shape]
        pairs = sum(n for n, _ in rs)
        syms = {k[2] for _, k in rs}
        roots = {root(s) for s in syms}
        tus = set()
        for _, k in rs:
            tus |= fntus[k]
        print(
            f"  {shape:8s} pairs {pairs:5d}   distinct symbols {len(syms):4d}"
            f"   distinct TUs {len(tus):4d}   template roots {len(roots):4d}"
        )
        for rt in sorted(roots)[:6]:
            print(f"        root  {rt[:100]}")


main()

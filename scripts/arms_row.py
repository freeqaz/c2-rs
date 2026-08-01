#!/usr/bin/env python3
"""Size one census key off a `c2rs gap` row dump: emitted / clean / names / prod.

Tooling beside `scripts/arms_sites.py`. Board #143 asks for one row's
counterfactual, and §9.13 measured two arms of one family converting 19× apart —
so the row has to be sized off its own stock, crossed with the production axis
that says *which* refusal is in the way and with `prod`'s own `-more` reading.

Usage:
    arms_row.py <dump.tsv> <census-key> [hex-witnesses]
"""

import collections
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from arms_sites import clean, is_complete, rows  # noqa: E402


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    path, key = sys.argv[1], sys.argv[2]
    nwit = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    e = c = 0
    names = set()
    cnames = set()
    prod = collections.Counter()
    cprod = collections.Counter()
    comp = collections.Counter()
    frame = collections.Counter()
    wit = []
    for r in rows(path):
        if r["key"] != key:
            continue
        e += 1
        names.add(r["name"])
        prod[r["prod"]] += 1
        comp[r["comp"]] += 1
        frame[r["frame"]] += 1
        if clean(r["frame"], r["cflow"], r["eh"]):
            c += 1
            cnames.add(r["name"])
            cprod[r["prod"]] += 1
            if len(wit) < nwit:
                wit.append(r)
    print(f"{key}")
    print(f"  emitted {e}   clean {c}   distinct names {len(names)} "
          f"(clean: {len(cnames)})   complete&clean "
          f"{sum(1 for x in comp if is_complete(x))}")
    print("  completeness:")
    for k, v in comp.most_common():
        print(f"    {v:>7}  {k}")
    print("  frame class:")
    for k, v in frame.most_common():
        print(f"    {v:>7}  {k}")
    print("  production first-blocker (emitted | clean):")
    for k, v in prod.most_common(12):
        print(f"    {v:>7} | {cprod[k]:>7}  {k}")
    for r in wit:
        print(f"\n  {r['src']}#{r['idx']} {r['name']}")
        print(f"    prod={r['prod']} comp={r['comp']} {r['frame']} {r['cflow']} {r['eh']}")
        print(f"    mark={r['hex_mark']} hex={r['hex']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

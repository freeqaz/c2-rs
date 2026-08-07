#!/usr/bin/env python3
"""family.py — the FAMILY SPREAD of a conversion set (#925/#952).

Every recent conversion family on this project has turned out to be one or two
templates, and a lane that reports only a count invites the next one to read it
as breadth. This prints the outermost template name of every function that left
`fnbyte-differs`, and the number of distinct TUs they came from.

    family.py <base.jsonl> <tip.jsonl>
"""
import collections
import json
import re
import sys


def keyed(path):
    out = set()
    for line in open(path):
        line = line.strip()
        if line:
            r = json.loads(line)
            out.add((r["tu"], r["sym"]))
    return out


def main(argv):
    a, b = keyed(argv[1]), keyed(argv[2])
    left = sorted(a - b)
    fam = collections.Counter()
    for _tu, s in left:
        m = re.match(r"\?\?\$([A-Za-z_0-9]+)@", s)
        fam[m.group(1) if m else s] += 1
    print(f"converted: {len(left)}")
    for k, v in fam.most_common():
        print(f"  {v:5d}  ??${k}@...")
    print(f"distinct outermost templates: {len(fam)}")
    print(f"distinct TUs: {len({t for t, _ in left})}")


if __name__ == "__main__":
    main(sys.argv)

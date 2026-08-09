#!/usr/bin/env python3
"""w-readpx — the deliverable-4 ledger: every CENSUS-ADMITTED class crossed
with the BYTE judge.

This is the answer to *"a rung that would admit functions the port then emits
wrongly is worth negative"* stated as a table over the classes that are
already admitted. Read it before ranking any reader candidate: it is the only
place on the workload where "the census said in-class" and "the oracle said
byte-exact" can be compared, because every BLOCKED emitted row is
`fnbyte-refused` by construction.
"""
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEM = sys.argv[1] if len(sys.argv) > 1 else "reb"


def main():
    rows = []
    for line in open(os.path.join(HERE, STEM + ".err"),
                     encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 9:
            continue
        rows.append(f)
    inc = [r for r in rows if r[7] == "in"]
    by = collections.defaultdict(list)
    for r in inc:
        by[r[4]].append(r)
    print("## Every CENSUS-ADMITTED class, crossed with the BYTE judge\n")
    print("| class (census key) | admitted | exact | differs | reloc-differs "
          "| refused by the port | P(exact) |")
    print("|---|---:|---:|---:|---:|---:|---:|")
    for k, v in sorted(by.items(), key=lambda kv: -len(kv[1])):
        c = collections.Counter(r[3] for r in v)
        print("| `%s` | %d | %d | %d | %d | %d | %.3f |"
              % (k, len(v), c.get("fnbyte-exact", 0),
                 c.get("fnbyte-differs", 0), c.get("fnbyte-reloc-differs", 0),
                 c.get("fnbyte-refused", 0), c.get("fnbyte-exact", 0) / len(v)))
    c = collections.Counter(r[3] for r in inc)
    print("| **all %d classes** | %d | %d | %d | %d | %d | %.3f |"
          % (len(by), len(inc), c["fnbyte-exact"], c["fnbyte-differs"],
             c["fnbyte-reloc-differs"], c["fnbyte-refused"],
             c["fnbyte-exact"] / len(inc)))
    zero = [(k, len(v),
             sum(1 for r in v
                 if r[3] in ("fnbyte-differs", "fnbyte-reloc-differs")))
            for k, v in by.items() if not any(r[3] == "fnbyte-exact" for r in v)]
    zero.sort(key=lambda x: -x[1])
    print("\n**Census-admitted classes with ZERO byte-exact functions: %d of "
          "%d, %d emitted, of which %d still emit WRONG BYTES at this tip**"
          % (len(zero), len(by), sum(n for _, n, _ in zero),
             sum(w for _, _, w in zero)))
    for k, n, w in zero:
        print("  - `%s` %d emitted, %d still wrong" % (k, n, w))


main()

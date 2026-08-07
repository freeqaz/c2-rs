#!/usr/bin/env python3
"""residue.py — the differs that did NOT convert, restricted to the population
this lane was priced on.

The point prediction was 227 and the measurement is 223. A lane that reports the
gap as "4" and stops has not said which four, and `w-seq` 5.1's caution is that a
residue with no name is a residue nobody can price.

    residue.py <tip.jsonl> <substr>
"""
import json
import sys


def main(argv):
    pat = argv[2]
    for line in open(argv[1]):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        if pat in r["sym"]:
            print(f'{r["tu"]}\t{r["sym"]}')
            for k in ("shape", "port_hex", "ref_hex"):
                if k in r:
                    print(f'    {k} = {r[k]}')


if __name__ == "__main__":
    main(sys.argv)

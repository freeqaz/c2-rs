#!/usr/bin/env python3
"""blrresidue.py — the differs whose WHOLE reference body is one `4e800020`.

That is exactly the population `fnbyte-blr-stop` walks and exactly the population
this lane was priced on: a function c2 emits nothing for and the port emits a
branch for. Printed by name so the four that did not convert can be named rather
than subtracted.

    blrresidue.py <differs.jsonl>
"""
import json
import sys

BLR = ["4e800020"]


def main(argv):
    n = 0
    for line in open(argv[1]):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        if r.get("ref_hex") == BLR:
            n += 1
            print(f'{r["tu"]}\t{r["sym"]}\t{r.get("shape")}\tport={r.get("port_hex")}')
    print(f"TOTAL differs whose reference body is one blr: {n}")


if __name__ == "__main__":
    main(sys.argv)

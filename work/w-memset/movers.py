#!/usr/bin/env python3
"""movers.py — per `(TU, emit_name)` motion between two `--fnbyte-diff-jsonl`
files, and never by subtracting two totals.

`w-empty` §4 is why this exists: a rule keyed on the wrong name binding moved 14
byte-exact bodies the WRONG way while the totals looked plausible, and only a
per-symbol diff showed it. The direction with the known answer 0 is
**ENTERED** — a function that was exact and now differs — and it is printed even
when it is empty, because an absent line reads as success.

    movers.py <base.jsonl> <tip.jsonl>
"""
import json
import sys


def keyed(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        out[(r["tu"], r["sym"])] = r
    return out


def main(argv):
    a, b = keyed(argv[1]), keyed(argv[2])
    left = sorted(set(a) - set(b))   # differed at base, not at tip: CONVERTED
    entered = sorted(set(b) - set(a))  # not a differ at base, is one now
    print(f"base differs : {len(a)}")
    print(f"tip  differs : {len(b)}")
    print(f"LEFT  differs (converted) : {len(left)}")
    for k in left[:40]:
        print(f"   - {k[0]}\t{k[1]}")
    print(f"ENTERED differs (regressed, known answer 0) : {len(entered)}")
    for k in entered[:40]:
        print(f"   + {k[0]}\t{k[1]}")
    changed = [k for k in set(a) & set(b) if a[k]["port_hex"] != b[k]["port_hex"]]
    print(f"still differing but with DIFFERENT port bytes : {len(changed)}")
    for k in sorted(changed)[:20]:
        print(f"   ~ {k[0]}\t{k[1]}")


if __name__ == "__main__":
    main(sys.argv)

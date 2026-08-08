#!/usr/bin/env python3
"""Compare the `.text` COMDAT words of two `scripts/gt_dump.py` listings.

    cmp_text.py <ref-dis> <ref-fn> <cell-dis> <cell-fn>

Prints `EQUAL` / the differing word offsets. A lane instrument: every count this
lane puts on the board is produced by a script, per WB_CHOOSER_FINDINGS §8.
"""
import re
import sys


def texts(path):
    out, cur = [], None
    for line in open(path):
        m = re.match(r"-- \.text #\d+ \((\d+) B\) (\S+)", line)
        if m:
            cur = [m.group(2), int(m.group(1)), []]
            out.append(cur)
            continue
        if line.startswith("-- "):
            cur = None
            continue
        m = re.match(r"   ([0-9a-f]{4})  ([0-9a-f]{8})", line)
        if m and cur is not None:
            cur[2].append(m.group(2))
    return out


def pick(path, name):
    for n, ln, w in texts(path):
        if n == name or name in n:
            return n, ln, w
    raise SystemExit(f"no .text COMDAT matching {name!r} in {path}")


def main():
    ra, rn, ca, cn = sys.argv[1:5]
    r_name, r_len, r_w = pick(ra, rn)
    c_name, c_len, c_w = pick(ca, cn)
    print(f"{r_name} ({r_len} B)  vs  {c_name} ({c_len} B)")
    if r_w == c_w:
        print(f"EQUAL — {len(r_w)} words")
        return
    n = 0
    for i, (x, y) in enumerate(zip(r_w, c_w)):
        if x != y:
            print(f"  +0x{i * 4:04x}  ref {x}   cell {y}")
            n += 1
    if len(r_w) != len(c_w):
        print(f"  LENGTH {len(r_w)} vs {len(c_w)} words")
    print(f"DIFFER — {n} word(s)")


main()

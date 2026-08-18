#!/usr/bin/env python3
"""extract.py — read the candidate colouring order out of a /FAsc listing.

Lane w-dagorder (WB-DAGORDER2). Measurement tooling; touches no crates/ file.

The instrument, stated so it can be attacked: every cell of the frozen grid
keeps its int values live across a call, so the volatiles r11..r3 are all
disallowed and the callee-saved run r31, r30, r29, ... is handed out in
W-REGALLOC-1's fixed order. A formal arrives in r3, r4, r5, ... (PPC ABI) and is
moved to its colour by an `mr <colour>,<arrival>` in the entry run. So the
sequence of `mr` instructions IS the arrival->colour map, and

    "which formal got r31"  ==  "which candidate was coloured first".

That inference depends on W-REGALLOC-1's register order being right. It is
recorded as a dependency, not assumed silently.
"""
import re
import sys

ARRIVAL = {3: 'a', 4: 'b', 5: 'c', 6: 'd', 7: 'e', 8: 'f', 9: 'g', 10: 'h'}

PROC = re.compile(r'^(\w+)\s+PROC NEAR')
ENDP = re.compile(r'^(\w+)\s+ENDP')
MR = re.compile(r'^\s+[0-9a-f]{5}\s+[0-9a-f]{8}\s+mr\s+r(\d+),r(\d+)\s*$')
ANY = re.compile(r'^\s+([0-9a-f]{5})\s+([0-9a-f]{8})\s+(\S+)\s*(.*)$')


def cells(path):
    cur, body = None, []
    for line in open(path, encoding='utf-8', errors='replace'):
        line = line.rstrip('\n')
        m = PROC.match(line)
        if m:
            cur, body = m.group(1), []
            continue
        m = ENDP.match(line)
        if m and cur:
            yield cur, body
            cur = None
            continue
        if cur is not None:
            body.append(line)


def analyse(name, body):
    """Return (colour_map, callee_saved_used, nwords)."""
    colours = {}
    saved = set()
    nwords = 0
    for line in body:
        m = ANY.match(line)
        if m:
            nwords += 1
        m = MR.match(line)
        if m:
            dst, src = int(m.group(1)), int(m.group(2))
            # an arrival copy: formal register -> callee-saved colour
            if src in ARRIVAL and 14 <= dst <= 31:
                colours.setdefault(ARRIVAL[src], f'r{dst}')
        for r in re.findall(r'\b(?:std|ld)\s+r(\d+),', line):
            if 14 <= int(r) <= 31:
                saved.add(int(r))
    return colours, sorted(saved, reverse=True), nwords


def main():
    for path in sys.argv[1:]:
        print(f'===== {path}')
        for name, body in cells(path):
            if not name.startswith('cnd_'):
                continue
            colours, saved, nwords = analyse(name, body)
            # order the candidates by the colour they took, r31 first
            order = sorted(colours.items(), key=lambda kv: -int(kv[1][1:]))
            seq = ' '.join(f'{k}={v}' for k, v in colours.items())
            first = order[0][0] if order else '-'
            print(f'{name:10s} n_words={nwords:3d} saved={saved} '
                  f'map[{seq}] colour_order={"<".join(k for k, _ in order)} first={first}')


if __name__ == '__main__':
    main()

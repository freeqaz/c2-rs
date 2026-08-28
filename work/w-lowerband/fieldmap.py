#!/usr/bin/env python3
"""fieldmap.py -- E4 of work/w-lowerband/PREREG.md SS4: the struct-identity filter.

For a set of displacements, list every function that references each, so that a
site at +0x50 can be tested for whether it is on the SAME struct as the `.gl`
function record (whose neighbouring fields +0x4c ATTR, +0x52, +0x54, +0x58 are
read at 0x10b9bf70/0x10b9bf7b/0x10b9bf57/0x10b9bf5f).

std only; tooling, not crates/.
usage: fieldmap.py 0x4c 0x50 0x52 0x54 0x58 ...
"""
import os, re, sys

LISTING = os.environ.get('C2RS_OBJDUMP_ASM',
                         os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))
FUNCS = os.path.expanduser('~/ghidra-projects/export/c2/functions.tsv')
LINE = re.compile(r'^([0-9a-f]{8}):\t([0-9a-f ]+?)\s*\t(\S+)\s*(.*)$')


def load_funcs():
    out = []
    with open(FUNCS) as f:
        next(f)
        for ln in f:
            p = ln.rstrip('\n').split('\t')
            if len(p) >= 3:
                out.append((int(p[0], 16), int(p[1]), p[2]))
    out.sort()
    return out


def owner(funcs, va):
    lo, hi, best = 0, len(funcs) - 1, None
    while lo <= hi:
        m = (lo + hi) // 2
        if funcs[m][0] <= va:
            best = funcs[m]; lo = m + 1
        else:
            hi = m - 1
    if best and best[0] <= va < best[0] + best[1]:
        return best[2]
    return None


def main():
    disps = [d if d.startswith('0x') else '0x' + d for d in sys.argv[1:]] or \
            ['0x4c', '0x50', '0x52', '0x54', '0x58']
    pats = {d: re.compile(r'\+' + re.escape(d) + r'\]') for d in disps}
    funcs = load_funcs()
    per = {d: {} for d in disps}          # disp -> {func: [va,...]}
    with open(LISTING, errors='replace') as f:
        for ln in f:
            m = LINE.match(ln)
            if not m:
                continue
            ops = m.group(4)
            if '+0x' not in ops:
                continue
            va = int(m.group(1), 16)
            fn = owner(funcs, va)
            if fn is None:
                continue
            for d, p in pats.items():
                if p.search(ops):
                    per[d].setdefault(fn, []).append((va, m.group(3), ops.strip()))

    for d in disps:
        print(f"+{d}: {sum(len(v) for v in per[d].values())} refs in "
              f"{len(per[d])} functions")
    print()
    # functions that touch EVERY requested displacement
    common = None
    for d in disps:
        s = set(per[d])
        common = s if common is None else (common & s)
    print(f"functions touching ALL of {' '.join(disps)}: {len(common)}")
    for fn in sorted(common):
        print(f"  {fn}")
        for d in disps:
            for va, mn, ops in per[d][fn]:
                print(f"      +{d}  {va:08x}  {mn:<6} {ops[:56]}")
    print()
    # pairwise with the first displacement (the anchor)
    anchor = disps[0]
    for d in disps[1:]:
        both = set(per[anchor]) & set(per[d])
        print(f"functions touching both +{anchor} and +{d}: {len(both)}  "
              f"{' '.join(sorted(both))}")
    return 0


if __name__ == '__main__':
    sys.exit(main())

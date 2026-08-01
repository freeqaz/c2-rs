#!/usr/bin/env python3
"""Read work/grid.cod into one row per cell: the axes, and c2's schedule."""
import re, sys, collections

CELL = re.compile(r'^\?([gh]_\w+)@@[^ \t]*\s+PROC NEAR')
INSN = re.compile(r'^  ([0-9a-f]{5})\t([0-9a-f]{8})\t (\S+)\s+(.*?)\s*$')


def read(path):
    cells = {}
    cur = None
    for line in open(path):
        m = CELL.match(line)
        if m:
            cur = m.group(1)
            cells[cur] = []
            continue
        if cur is None:
            continue
        if ' ENDP' in line:
            cur = None
            continue
        m = INSN.match(line)
        if m:
            cells[cur].append((m.group(2), m.group(3), m.group(4)))
    return cells


def axes(name):
    # g_{kind}_{steps}s_{off}_{nargs}a_{addr_slot}
    _, kind, steps, off, nargs, slot = name.split('_')
    return dict(kind=kind, steps=int(steps[:-1]), off=int(off),
                nargs=int(nargs[:-1]), slot=int(slot))


def schedule(insns):
    """The shape of the sequence, with registers abstracted away."""
    return ' ; '.join(op for _, op, _ in insns)


if __name__ == '__main__':
    cells = read(sys.argv[1] if len(sys.argv) > 1 else 'work/grid.cod')
    print(f'{len(cells)} cells parsed')
    shapes = collections.Counter()
    for n, ins in cells.items():
        shapes[schedule(ins)] += 1
    print(f'\n{len(shapes)} distinct schedule shapes:')
    for s, c in shapes.most_common():
        print(f'  {c:>4}  {s}')

#!/usr/bin/env python3
"""addr_align.py -- does every CLAUSES.tsv address START an instruction?

`work/w-inlmetric/check_table.py`'s ADDRESS check asks whether an address lies
inside the function its `owner` column names, per FUNCS.tsv's entry+size. That
is a real check and it has caught a real defect. It is also **unable to fail on
a mid-instruction address**: an address 0x11b bytes past the instruction the
clause describes is still inside the same function, so containment is green and
the citation is wrong.

This is the second half of that check, and it is deliberately a SEPARATE
program rather than a new clause inside `check_table.py`: that grader is
another lane's frozen instrument and its green is quoted on the table's own
tree.

The boundary set is taken from the INDEPENDENT objdump disassembly
(`C2_MAP_METHOD.md`: `objdump -d -M intel`, PE32 read as pei-i386 at true VAs),
not from the Ghidra database the addresses were transcribed out of. Two
disassemblers agreeing that an address is mid-instruction is a stronger claim
than one of them saying so.

The listing is regenerated, never committed, so an absent listing is a SKIP and
never a failure -- the repo's standing rule for toolchain-dependent lanes.

Usage:  addr_align.py [CLAUSES.tsv] [--plant 0xADDR]

  --plant  replace the first row's address with 0xADDR before grading, so the
           RED path can be watched. `#3336`: a control nobody has seen fail is
           decoration.

Exit 0 = GREEN or SKIP. Non-zero = RED. Read the verdict line, never the code.
"""
import bisect
import csv
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LISTING = os.environ.get(
    'C2RS_OBJDUMP_ASM',
    os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))
LINE = re.compile(r'^([0-9a-f]{8}):\t')


def boundaries(path):
    out = []
    with open(path, errors='replace') as fh:
        for line in fh:
            m = LINE.match(line)
            if m:
                out.append(int(m.group(1), 16))
    out.sort()
    return out


def containing(bs, a):
    """The boundary at or below `a`, or None if `a` precedes them all."""
    i = bisect.bisect_right(bs, a) - 1
    return bs[i] if i >= 0 else None


def funcs():
    p = os.path.join(REPO, 'docs/whitebox/ref/FUNCS.tsv')
    out = []
    with open(p) as fh:
        for x in csv.DictReader([l for l in fh if not l.startswith('#')],
                                delimiter='\t'):
            try:
                out.append((int(x['addr'], 16), int(x['size'])))
            except (ValueError, TypeError):
                pass
    out.sort()
    return out


def main(argv):
    plant = None
    args = []
    i = 0
    while i < len(argv):
        if argv[i] == '--plant':
            plant = int(argv[i + 1], 16)
            i += 2
            continue
        args.append(argv[i])
        i += 1

    path = args[0] if args else os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv')

    # Displayed home-relative: this file's output is COMMITTED, and an absolute
    # machine path in a tracked file is a class-3 violation of the artifact
    # audit (CLAUDE.md, "never commit absolute machine paths").
    shown = LISTING.replace(os.path.expanduser('~'), '~', 1)

    if not os.path.exists(LISTING):
        print(f"listing: {shown}")
        print("\nADDR-ALIGN: SKIP  (objdump listing absent; regenerate per "
              "C2_MAP_METHOD.md, or set C2RS_OBJDUMP_ASM)")
        return 0

    bs = boundaries(LISTING)
    fns = funcs()
    starts = [f[0] for f in fns]
    with open(path) as fh:
        rows = list(csv.DictReader([l for l in fh if not l.startswith('#')],
                                   delimiter='\t'))
    if plant is not None and rows:
        rows[0] = dict(rows[0])
        rows[0]['addr'] = f"{plant:08x}"
        rows[0]['id'] += '(PLANTED)'

    fails = []
    for r in rows:
        a = int(r['addr'], 16)
        b = containing(bs, a)
        if b is None or b == a:
            continue
        # How far past the containing instruction's start does the address sit,
        # and is it even inside that instruction? Report both -- an address one
        # byte into a 2-byte `jne` and one 0x11b bytes into a duplicated block
        # are different defects.
        j = bisect.bisect_right(starts, a) - 1
        fn = f"FUN_{starts[j]:08x}" if j >= 0 and a < starts[j] + fns[j][1] else "(orphan)"
        fails.append(f"{r['id']}: 0x{a:08x} is +{a - b} INTO the instruction at "
                     f"0x{b:08x} (both in {fn}) -- {r['clause'][:52]}")

    print(f"listing  : {shown}")
    print(f"boundaries: {len(bs):,} instruction starts")
    print(f"rows      : {len(rows)}")
    for f in fails:
        print("  FAIL " + f)
    print(f"\nADDR-ALIGN: {'RED' if fails else 'GREEN'}  "
          f"({len(fails)} misaligned of {len(rows)} rows)")
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

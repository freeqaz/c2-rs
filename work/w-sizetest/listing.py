#!/usr/bin/env python3
"""Print the objdump listing for [START, END) of c2.dll by absolute VA.

`awk '/^10bxxxxx:/,/^10byyyyy:/'` is unreliable here for two reasons that both
bit this lane: a VA that falls MID-INSTRUCTION never starts a line, so the range
silently runs to EOF (14 MB), and objdump wraps long instructions onto
continuation lines that carry no address.  This walks addresses numerically and
keeps continuation lines with their instruction.

Usage:  python3 work/w-sizetest/listing.py 10b5fb5f 10b5fcd8
        (override the export with C2_OBJDUMP)
"""
import os
import re
import sys

ASM = os.environ.get('C2_OBJDUMP') or os.path.expanduser(
    '~/ghidra-projects/export/c2/objdump_intel.asm')
LINE = re.compile(r'^\s*([0-9a-f]{8}):\s')


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    lo = int(sys.argv[1], 16)
    hi = int(sys.argv[2], 16)
    emit = False
    with open(ASM, 'r', errors='replace') as fh:
        for line in fh:
            m = LINE.match(line)
            if m:
                va = int(m.group(1), 16)
                if va >= hi:
                    if emit:
                        break
                    continue
                emit = lo <= va < hi
            if emit:
                sys.stdout.write(line)
    return 0


if __name__ == '__main__':
    sys.exit(main())

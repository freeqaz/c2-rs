#!/usr/bin/env python3
"""grab.py LO HI -- print objdump_intel.asm lines whose VA is in [LO,HI).

Reads the independent objdump disassembly of c2.dll
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md SS: `objdump -d -M intel`, PE32 read as pei-i386 at true VAs).
Not committed with the listing; the listing is regenerated, never stored.
"""
import re, sys
import os
P = os.environ.get('C2RS_OBJDUMP_ASM',
                  os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))
lo, hi = int(sys.argv[1], 16), int(sys.argv[2], 16)
seen = False
for line in open(P, errors='replace'):
    m = re.match(r'^([0-9a-f]{8}):\t', line)
    if not m:
        continue
    a = int(m.group(1), 16)
    if lo <= a < hi:
        seen = True
        sys.stdout.write(line)
    elif seen and a >= hi:
        break

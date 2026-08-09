#!/usr/bin/env python3
"""wb-selfit — the FUN_10c0f882 arm count, three ways, from the flat export."""
import os
import re

# The flat export lives outside the repo by design (C2_MAP_METHOD.md §4 — the
# Ghidra project is never opened; only this directory is grepped).
P = os.path.expanduser(
    os.environ.get("C2RS_C2_EXPORT", "~/ghidra-projects/export/c2") + "/decomp_all.c")
src = open(P).read().split("\n")
start = 214218 - 1
end = next(i for i in range(start + 3, len(src)) if src[i] == "}")
body = src[start:end + 1]
lines = [i for i, l in enumerate(body) if re.match(r"^  (case |default)", l)]
groups, cur = [], [lines[0]]
for a, b in zip(lines, lines[1:]):
    if b == a + 1:
        cur.append(b)
    else:
        groups.append(cur)
        cur = [b]
groups.append(cur)

print(f"FUN_10c0f882  ({len(body)} decompiled lines, size 686 bytes)")
print(f"  case labels           : {len(lines)}     <- wb-select published 46")
print(f"  maximal label groups  : {len(groups)}")
print(f"  jump table entries    : "
      f"(0x10c0fbd6 - 0x10c0fb32)/4 = {(0x10c0fbd6 - 0x10c0fb32) // 4}"
      f"     <- wb-select2 published 41")
print(f"  byte index entries    : 0xad + 1 = {0xad + 1}"
      f"     <- both published 174")
print()
for g in groups:
    print(f"   {len(g):2d}  " + " ".join(body[i].strip() for i in g))

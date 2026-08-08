#!/usr/bin/env python3
"""CONFIRMATION 1 — read every `BD` token out of the fresh `bd_cc.cpp` capture.

    read_bd.py <probe.ex>

Prints each anchored `26 <tok> BD` site's fields under the claimed width

    BD <TYPE ret> <flags:1 raw byte> <varint fn-type-id>

so the flags byte can be seen to be the ONLY field that moves across four
externals with a byte-identical `int` return type, and so the three return-TYPE
widths (3/4/5) can be seen to be handled by the same reading.
"""
import sys

sys.path.insert(0, __file__.rsplit("/", 2)[0])
from bdwalk import LEGAL_OPEN, read_token_var, read_type, read_varint  # noqa: E402

b = open(sys.argv[1], "rb").read()
i = 0
n = 0
while True:
    j = b.find(b"\xbd", i)
    if j < 0:
        break
    i = j + 1
    ok = False
    for tw in (2, 4):
        k = j - 1 - tw
        if k >= 0 and b[k] == 0x26:
            r = read_token_var(b, k + 1)
            if r is not None and r[1] == tw and k + 1 + tw == j:
                ok = True
                break
    if not ok:
        continue
    t = read_type(b, j + 1)
    if t is None:
        continue
    fp = j + 1 + t[3]
    v = read_varint(b, fp + 1)
    if v is None:
        continue
    n += 1
    ty = b[j + 1:fp].hex(" ")
    idb = b[fp + 1:v[1]].hex(" ")
    land = b[v[1]]
    print(
        f"  BD  TYPE[{t[3]}] {ty:<14}  flags {b[fp]:#04x}  "
        f"id[{v[1] - fp - 1}] {idb:<14}  width {v[1] - j:2d}  "
        f"-> 0x{land:02X} {'LEGAL' if land in LEGAL_OPEN else 'DESYNC'}"
    )
print(f"\n  {n} anchored CALL tokens")

#!/usr/bin/env python3
"""CONFIRMATION 1 — read every CALL and its ARGUMENT REGION out of the fresh
`ce_args.cpp` capture, and print the byte the `4C` is followed by.

    read_ce.py <probe.ex>

Board **#1318** declined `0x4C` because its evidence was zero-argument calls
only. This prints the `4C` of calls with 0, 1, 2 and 3 arguments **from one
capture**, so the argument count is visibly the only thing that moves, and shows
that the `4C` is followed immediately by the next opcode with nothing between.

Each call is decoded by the same rules `argwalk.py` uses on the workload:
`BD <TYPE> <flags:1> <varint id>` at the width board #1314 pinned, then
`operand()`'s widths through the argument region, stopping AT the first `4C`
without ever stepping over one.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
sys.path.insert(0, os.path.join(HERE, "..", "..", "w-bd"))
from argwalk import anchored_bd, bd_end, step, _ty  # noqa: E402
from bdwalk import LEGAL_OPEN  # noqa: E402

b = open(sys.argv[1], "rb").read()
n = len(b)
i = 0
rows = 0
while True:
    j = b.find(b"\xbd", i)
    if j < 0:
        break
    i = j + 1
    anc = anchored_bd(b, j)
    if anc is None:
        continue
    e0 = bd_end(b, j)
    if e0 is None or e0 >= n:
        continue
    p, nargs, nested = e0, 0, False
    while p < n:
        o = b[p]
        if o == 0x4C:
            break
        if o == 0xBD:
            nested = True
            break
        if o == 0x55:
            q = _ty(b, p + 1)
            if q is None:
                p = None
                break
            nargs += 1
            p = q
            continue
        q = step(b, p)[0]
        if q is None or q <= p:
            p = None
            break
        p = q
    if nested:
        print(f"  BD @ {j:5d}  {anc:6s}  NESTED CALL in its argument region — anchor A excludes")
        continue
    if p is None or p >= n or b[p] != 0x4C:
        print(f"  BD @ {j:5d}  {anc:6s}  the walk did not reach a 4C")
        continue
    rows += 1
    tokw = e0 - j
    argw = p - e0
    succ = b[p + 1] if p + 1 < n else None
    print(
        f"  BD @ {j:5d}  {anc:6s}  token[{tokw}] {b[j:e0].hex(' '):<26}"
        f"  args {nargs}  argregion[{argw}] {b[e0:p].hex(' '):<44}"
        f"  4C -> 0x{succ:02X} {'LEGAL' if succ in LEGAL_OPEN else 'DESYNC'}"
    )
print(f"\n  {rows} anchored CALL tokens whose argument region was walked to its 4C")

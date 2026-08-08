#!/usr/bin/env python3
"""THE CONTROL THAT DECIDES THE RESIDUE — are `0x59` and `0x08` OPCODES?

    unwit.py <il-root>

`argwalk.py` scores a reading by `bdwalk.LEGAL_OPEN`: the byte a reading lands on
must open an operand token. Every P desync it finds on the argument-bearing
population lands on **`0x59`** (446 of 457) or **`0x08`** (11), and every one of
its zero-argument desyncs lands on `0x08` (719 of 719). Both are outside
`LEGAL_OPEN` because `control_flow.rs`'s `operand()` has no arm for them — `08`
is one of the six bytes that file lists as *deliberately unwitnessed* (`07`,
`08`, `14`, `1D`, `1E`, `25`) and refuses on purpose.

So there are exactly two readings of that residue and they are not the same
claim:

  **G — an instrument GAP.** `59` and `08` are ordinary opcodes this tree has
  never pinned. `4C` is payload-free and `LEGAL_OPEN` is incomplete.

  **W — a real WIDTH.** The call-end carries a payload after all, at least at
  these sites: `4C 59 …`, `4C 08 …`.

**They are separated by a question that never mentions `4C`:** does `59` (or
`08`) occur at a token-start position that is NOT preceded by a `4C`? If it does,
it is an opcode in its own right and G is the answer. If every occurrence in the
whole corpus sits immediately after a `4C`, W becomes the better reading and this
lane declines.

The positions are produced the same way `argwalk.py` produces them — by stepping
`operand()`'s widths forward from an anchored `BD`, a walk that **breaks at
`4C`** and therefore can only reach a `59`/`08` whose predecessor is some other
token. `LEGAL_OPEN` is not consulted at all here, so the control is independent
of the vocabulary whose incompleteness is under test.

`w-bd` widened `LEGAL_OPEN` once, mid-lane, for exactly this reason (the
`5C`/`5D`/`5E` family) and recorded doing so. This lane does **not** widen it:
`operand()` refuses `08` on purpose, and a hand-added entry would be the guess
the whole table exists to prevent. It measures the residue instead.

Read-only. Consumes captured IL, which is never committed.
"""

import collections
import glob
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "w-bd"))
from argwalk import anchored_bd, bd_end, step, _ty  # noqa: E402
from bdwalk import read_type  # noqa: E402

TARGETS = (0x59, 0x08, 0x07, 0x14, 0x1D, 0x1E, 0x25)


def main(root):
    tus = 0
    reached = collections.Counter()          # target -> times reached at a token start
    prev_op = {t: collections.Counter() for t in TARGETS}
    after_4c = collections.Counter()
    not_after_4c = collections.Counter()
    ctx = {t: [] for t in TARGETS}
    # …and the TYPE the enclosing call carries, because every residue context in
    # `argwalk.txt` shows `86 45 40` — the FLOAT operand type.
    float_ctx = collections.Counter()

    for d in sorted(glob.glob(os.path.join(root, "*"))):
        exs = glob.glob(os.path.join(d, "*.ex"))
        if not exs:
            continue
        tus += 1
        tu = open(os.path.join(d, "TU")).read().strip()
        b = open(exs[0], "rb").read()
        n = len(b)
        i = 0
        while True:
            j = b.find(b"\xbd", i)
            if j < 0:
                break
            i = j + 1
            if anchored_bd(b, j) is None:
                continue
            e0 = bd_end(b, j)
            if e0 is None or e0 >= n:
                continue
            ret = read_type(b, j + 1)
            is_float = ret is not None and b[j + 1 : j + 1 + ret[3]] == b"\x86\x45\x40"
            p, last = e0, None
            while True:
                if p >= n:
                    break
                o = b[p]
                if o == 0x4C or o == 0xBD:
                    break
                if o in TARGETS:
                    reached[o] += 1
                    prev_op[o][f"0x{last:02X}" if last is not None else "<BD-token-end>"] += 1
                    # THE CONTROL. `last` is the opcode of the token the walk
                    # stepped to get here, and the walk BREAKS at `4C`, so a
                    # non-`4C` `last` is an occurrence of this opcode that no
                    # reading of `4C` can account for.
                    if last == 0x4C:
                        after_4c[o] += 1
                    else:
                        not_after_4c[o] += 1
                    if is_float:
                        float_ctx[o] += 1
                    if len(ctx[o]) < 10:
                        ctx[o].append((tu, p, b[max(0, p - 12) : p + 8].hex(" ")))
                    break
                if o == 0x55:
                    q = _ty(b, p + 1)
                else:
                    q = step(b, p)[0]
                if q is None or q <= p:
                    break
                last = o
                p = q

    print("=" * 78)
    print("w-4c — are 0x59 and 0x08 OPCODES, or is 0x4C carrying a payload?")
    print("=" * 78)
    print(f"TUs                                        {tus}")
    print()
    print("The walk BREAKS at `4C`, so every position below was reached by")
    print("stepping some OTHER token's width. A non-`4C` predecessor is an")
    print("occurrence no reading of `4C` can account for.")
    print()
    print(f"  {'op':6s} {'reached':>10s} {'after a 4C':>12s} {'NOT after a 4C':>16s}")
    for t in TARGETS:
        if not reached[t]:
            continue
        print(
            f"  0x{t:02X}   {reached[t]:10d} {after_4c[t]:12d} {not_after_4c[t]:16d}"
        )
    print()
    print("-- the token immediately BEFORE, per target --")
    for t in TARGETS:
        if not reached[t]:
            continue
        print(f"  0x{t:02X}: " + ", ".join(f"{k}x{c}" for k, c in prev_op[t].most_common(12)))
    print()
    print("-- how many of these sit inside a call whose return TYPE is `86 45 40` (float) --")
    for t in TARGETS:
        if reached[t]:
            print(f"  0x{t:02X}   {float_ctx[t]} / {reached[t]}")
    print()
    print("-- context, ten per target --")
    for t in TARGETS:
        for tu, p, h in ctx[t]:
            print(f"  0x{t:02X}  {tu} @ {p}: {h}")


if __name__ == "__main__":
    main(sys.argv[1])

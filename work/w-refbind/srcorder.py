#!/usr/bin/env python3
"""srcorder.py — does the bind pin the STORE order to source order?

A reader over `work/w-refbind/bindgrid_dis.txt`, which this lane already
committed. It compiles nothing and probes nothing new: it re-reads the recorded
disassembly and asks a question `bindgrid.py` did not print.

Every `bindgrid` cell stores, IN SOURCE ORDER:

    the constant to s->f0, s->f1, …        offsets 0, 4, 8, …
    the producer to s->inner.a0, .a1, …    offsets 64, 68, 72, …

(every binding mode uses the same offsets, by construction — that is why
`offprobe`'s killed displacement rival cannot come back in here). So the
source order of the store offsets is derivable from the cell name alone, and
the emitted order is on the recorded lines.

Usage:  srcorder.py [bindgrid_dis.txt]
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PATH = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "bindgrid_dis.txt")

NAME = re.compile(r"^(P\d)-(\w+?)-([a-z-]+)-r(\d)k(\d)$")
STW = re.compile(r"^stw\s+\d+,\s*(\d+)\(3\)$")
# `none`-like modes: no bind survives to the IL, or it binds at displacement 0.
# Classified from this lane's own measured verdicts (refprobe.out, bindgrid.out),
# not assumed.
BOUND = {"ref", "ptr", "iptr"}

blocks = open(PATH).read().split("== ")
rows = {}
for b in blocks:
    if not b.strip() or "\n" not in b:
        continue
    head, body = b.split("\n", 1)
    rows[head.strip()] = [x.strip() for x in body.strip().splitlines()]

tot = {}
for name, words in rows.items():
    m = NAME.match(name)
    if not m:
        continue
    _, spell, mode, ru, cu = m.groups()
    ru, cu = int(ru), int(cu)
    src = [4 * i for i in range(cu)] + [64 + 4 * i for i in range(ru)]
    got = [int(x.group(1)) for x in (STW.match(w) for w in words) if x]
    if sorted(got) != sorted(src):
        print("  %-34s SKIP — %d stores emitted, %d expected"
              % (name, len(got), len(src)))
        continue
    k = "bound" if mode in BOUND else "unbound"
    t = tot.setdefault(k, [0, 0])
    t[0] += 1
    t[1] += (got == src)
    if got != src:
        print("  %-34s %-8s NOT source order: src %s -> emitted %s"
              % (name, k, src, got))

print("\n  emitted store order == SOURCE order:")
for k in ("bound", "unbound"):
    n, s = tot.get(k, [0, 0])
    print("    %-8s %d of %d cells" % (k, s, n))

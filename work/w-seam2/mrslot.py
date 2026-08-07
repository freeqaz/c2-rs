#!/usr/bin/env python3
"""mrslot.py — score the `mr r31,r3` SLOT rule, cell by cell, against real bytes.

This is the ONE modeled fact this lane would be shipping that no emitter carries
today (prereg §2.4). `w-seam`/#867 fitted

    stores_before_mr = nprod - 1 + min(u, 2)

on 24 cells and held it on an 18/18 fresh holdout, and it has never shipped.
Everything else in the composition is already shipped and graded somewhere.

`nprod` is the number of DISTINCT producers (equal literals CSE to one `li`, so
they are one producer — the same identity `scheduled_gpr_run_text` uses) and `u`
is the number of stores that materialise nothing. Printed per cell beside the
observed slot, with the prediction and a HIT/MISS, so the rule is scored rather
than assumed — and so a boundary announces itself instead of being averaged away.

Usage:  mrslot.py [grid-subdir]      (default `grid`; `grid2` is the holdout)
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))


def text_words(d):
    p = os.path.join(d, "dis.txt")
    if not os.path.exists(p):
        return None
    out, inside = [], False
    for line in open(p):
        if line.startswith("-- .text"):
            inside = True
            continue
        if inside and line.startswith("--"):
            break
        if not inside:
            continue
        m = re.match(r"\s+[0-9a-f]{4}\s+[0-9a-f]{8}\s+(.*?)\s*$", line)
        if m:
            out.append(re.sub(r"\s+", " ", m.group(1).replace("\t", " ")).strip())
    return out


def analyse(words):
    """(framed, slots, mr_index_in_stores) from the emitted words.

    `slots` is the emitted middle as a list of 'P' (a producer materialisation:
    `li`/`lis`/`ori`/`addi` into a scratch register), 'S' (a store) and 'M' (the
    `mr 31,3`), which is the same alphabet `codegen::schedule::Slot` uses.
    """
    if not any(w.startswith("stwu") for w in words):
        return False, [], None, 0, 0
    i = next(i for i, w in enumerate(words) if w.startswith("stwu")) + 1
    j = next((j for j, w in enumerate(words) if w.startswith("addi 1, 1,")), len(words))
    mid = words[i:j]
    slots, regs = [], set()
    mr_at, stores = None, 0
    for w in mid:
        if w.startswith("bl ") or w.startswith("b ") or w.startswith("REL24"):
            break
        if re.match(r"mr 31, 3$", w):
            slots.append("M")
            mr_at = stores
        elif w.startswith("st"):
            slots.append("S")
            stores += 1
        else:
            m = re.match(r"(?:li|lis|addi) (\d+),", w)
            if m:
                slots.append("P")
                regs.add(m.group(1))
            else:
                slots.append("?" + w.split()[0])
    nprod = len(regs)
    # A store whose source register is one a producer wrote is PRODUCED.
    nsw = 0
    for w in mid:
        m = re.match(r"st[bhwd] (\d+),", w)
        if m and m.group(1) not in regs:
            nsw += 1
    return True, slots, mr_at, nprod, nsw


def predict867(nprod, nsw):
    return nprod - 1 + min(nsw, 2)


def main():
    sub = sys.argv[1] if len(sys.argv) > 1 else "grid"
    grid = os.path.join(ROOT, sub)
    hits = misses = 0
    print("%-24s %-3s %-3s %-22s %-4s %-4s %s"
          % ("cell", "np", "u", "slots", "obs", "#867", ""))
    for cell in sorted(os.listdir(grid)):
        if not cell.startswith(("sa_", "h_")):
            continue
        ws = text_words(os.path.join(grid, cell))
        if ws is None:
            print("%-24s NO DIS" % cell)
            continue
        framed, slots, mr_at, nprod, nsw = analyse(ws)
        if not framed:
            print("%-24s NOT FRAMED (%s)" % (cell, "".join(slots)))
            continue
        p = predict867(nprod, nsw)
        ok = (p == mr_at)
        if ok:
            hits += 1
        else:
            misses += 1
        print("%-24s %-3d %-3d %-22s %-4s %-4s %s"
              % (cell, nprod, nsw, "".join(slots),
                 "-" if mr_at is None else mr_at, p, "HIT" if ok else "MISS"))
    print("\n#867 `nprod - 1 + min(u,2)`:  %d HIT  %d MISS" % (hits, misses))


if __name__ == "__main__":
    main()

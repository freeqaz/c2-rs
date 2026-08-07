#!/usr/bin/env python3
"""analyse.py — read GRID S's reference disassemblies and answer three questions.

1. **Does the run TRANSFER?** For every accept cell `sa_<run>_<width>_<setup>`
   there is a leaf control `sl_<run>_<width>_cnone` with the *identical* run and
   no call. Board #866 measured the transfer over 96 cells on `w-seam`'s own
   axes; this re-measures it at the workload's flags on the cells this lane would
   actually emit, and prints the two run texts side by side rather than asserting.

   The comparison DELETES the `mr 31,3` from the framed run before comparing,
   because #866's claim is precisely that the copy is **additive** — inserted
   without moving one other word. Deleting it and finding the rest identical is
   the claim; finding the rest reordered would refute it.

2. **Where does `mr 31,3` sit?** Printed as the number of STORES emitted before
   it, beside `nprod` (distinct producers) and `u` (unproduced stores), so
   `w-seam`/#867's unshipped `nprod - 1 + min(u, 2)` can be scored per cell
   instead of assumed.

3. **What is the frame?** Frame word count and prologue/epilogue words, so a
   cell that is NOT framed (#869: three of four look-alike forms tail-call)
   announces itself instead of being read as a framed body with a short run.
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(ROOT, "grid")

PROLOGUE = ("mflr", "stw", "std", "stwu")
EPILOGUE_HEAD = "addi 1, 1,"


def text_words(cell):
    """The `.text` disassembly of a cell, as (off, word, text) triples."""
    p = os.path.join(GRID, cell, "dis.txt")
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
        m = re.match(r"\s+([0-9a-f]{4})\s+([0-9a-f]{8})\s+(.*?)\s*$", line)
        if m:
            out.append((int(m.group(1), 16), m.group(2),
                        re.sub(r"\s+", " ", m.group(3).replace("\t", " ")).strip()))
    return out


def split_body(ws):
    """(prologue, middle, epilogue) by the frame's own markers.

    A body with no `stwu` is NOT framed — that is #869's whole finding, and it is
    read off the words rather than off the source shape.
    """
    if not any(w[2].startswith("stwu") for w in ws):
        return [], ws, []
    i = next(i for i, w in enumerate(ws) if w[2].startswith("stwu")) + 1
    j = next((j for j, w in enumerate(ws) if w[2].startswith(EPILOGUE_HEAD)), len(ws))
    return ws[:i], ws[i:j], ws[j:]


def run_of(mid):
    """The store run: everything before the `bl` (or the whole middle, for a leaf),
    with the terminal `blr` dropped."""
    for i, w in enumerate(mid):
        if w[2].startswith("bl ") or w[2].startswith("b "):
            return mid[:i], mid[i:]
    return [w for w in mid if not w[2].startswith("blr")], \
           [w for w in mid if w[2].startswith("blr")]


def main():
    cells = sorted(os.listdir(GRID))
    rows = []
    for cell in cells:
        if not cell.startswith("sa_"):
            continue
        ws = text_words(cell)
        if ws is None:
            rows.append((cell, "NO DIS", "", "", ""))
            continue
        pro, mid, epi = split_body(ws)
        framed = bool(pro)
        run, tail = run_of(mid)
        # the `mr 31,3` and how many stores precede it
        mr_at = None
        stores = 0
        for w in run:
            if re.match(r"mr 31, 3$", w[2]):
                mr_at = stores
                continue
            if w[2].startswith("st"):
                stores += 1
        run_txt = " ; ".join(w[2] for w in run if not re.match(r"mr 31, 3$", w[2]))

        # the leaf control with the identical run
        _, rname, wname, _ = cell.split("_")
        leaf = "sl_%s_%s_cnone" % (rname, wname)
        lws = text_words(leaf)
        if lws is None:
            verdict = "NO LEAF"
        else:
            _, lmid, _ = split_body(lws)
            lrun, _ = run_of(lmid)
            leaf_txt = " ; ".join(w[2] for w in lrun)
            verdict = "IDENT" if leaf_txt == run_txt else "DIFFER"
            if not lws:
                verdict = "NO LEAF"
        rows.append((cell, "framed" if framed else "TAIL/leaf",
                     "mr@%s" % ("-" if mr_at is None else mr_at),
                     verdict, run_txt))

    w = max(len(r[0]) for r in rows)
    for r in rows:
        print("%-*s  %-10s %-7s %-7s %s" % (w, r[0], r[1], r[2], r[3], r[4]))

    print()
    print("-- refusal controls: frame word count and shape --")
    for cell in cells:
        if not cell.startswith("sr_"):
            continue
        ws = text_words(cell)
        if ws is None:
            print("%-16s NO DIS" % cell)
            continue
        pro, mid, epi = split_body(ws)
        print("%-16s %-10s %s" % (
            cell, "framed" if pro else "TAIL/leaf",
            " ; ".join(x[2] for x in mid)))


if __name__ == "__main__":
    main()

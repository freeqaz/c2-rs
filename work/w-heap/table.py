#!/usr/bin/env python3
"""table.py — one row per frozen w-heap cell: census verdict, differential
verdict, frame word count, `mr r31,r3` run index, store bases, and whether the
call needed argument setup.

The frame word count is PRINTED and not asserted, because board #869 is exactly
the failure of trusting a cell's source shape: twelve `w-seam` cells that look
framed are tail calls.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(HERE, "grid")

FRAME = ("mflr", "stw 12,", "std 31,", "stwu", "addi 1,", "lwz 12,", "mtlr", "ld 31,", "blr")


def body(d):
    p = os.path.join(d, "dis.txt")
    if not os.path.exists(p):
        return None
    out, on = [], False
    for ln in open(p):
        if ln.startswith("-- .text"):
            on = True
            continue
        if on and ln.startswith("-- "):
            break
        if on:
            m = re.match(r"\s+([0-9a-f]{4})\s+([0-9a-f]{8})\s+(.*?)\s*$", ln)
            if m:
                out.append(m.group(3).replace("\t", " "))
    return out


def classify(ins):
    """(nwords, frame_words, mr31_run_index, bases, argsetup, stores)"""
    if ins is None:
        return None
    fw = sum(1 for i in ins if any(i.startswith(f) for f in FRAME))
    stores, bases, mr31, argsetup, n = 0, [], None, 0, 0
    for i in ins:
        if i.startswith("mr 31, 3"):
            mr31 = stores
            continue
        if any(i.startswith(f) for f in FRAME) or i.startswith("bl ") or i.startswith("mr 3, 31"):
            continue
        m = re.match(r"stw? \d+, (-?\d+)\((\d+)\)", i)
        if m:
            stores += 1
            bases.append(m.group(2))
            continue
        # a move into an ARGUMENT register that is not the r31 save/restore
        if re.match(r"mr [3-9], ", i) or re.match(r"mr 10, ", i):
            argsetup += 1
            continue
        n += 1  # a producer (li / addi / lis / ori)
    return fw, mr31, "".join(sorted(set(bases))) or "-", argsetup, stores, n


def main():
    rows = []
    for cell in sorted(os.listdir(GRID)):
        d = os.path.join(GRID, cell)
        if not os.path.isdir(d):
            continue
        cen = dif = "?"
        key = ""
        cp = os.path.join(d, "census.txt")
        if os.path.exists(cp):
            t = open(cp, errors="replace").read()
            m = re.search(r"-> (\d+/\d+) functions in class", t)
            cen = m.group(1) if m else "NO-VERDICT"
            m = re.search(r"^ +1 x (expr-\S+|call-\S+|noform\S+|badtoken\S+)", t, re.M)
            key = m.group(1) if m else ""
        gp = os.path.join(d, "gap.txt")
        if os.path.exists(gp):
            m = re.search(r"^  \[1/1\] (\S+)", open(gp, errors="replace").read(), re.M)
            dif = m.group(1) if m else "NO-DIFF"
        c = classify(body(d))
        if c is None:
            rows.append((cell, cen, dif, key, "-", "-", "-", "-", "-", "-"))
        else:
            fw, mr31, bases, argset, st, np_ = c
            rows.append((cell, cen, dif, key, str(fw), "-" if mr31 is None else str(mr31),
                         bases, str(argset), str(st), str(np_)))

    hdr = ("cell", "census", "differential", "first-refusal-key",
           "frame", "mr31@", "bases", "argset", "stores", "prod")
    w = [max(len(str(r[i])) for r in rows + [hdr]) for i in range(len(hdr))]
    def line(r):
        return "  ".join(str(r[i]).ljust(w[i]) for i in range(len(hdr))).rstrip()
    print(line(hdr))
    print("  ".join("-" * x for x in w))
    for r in rows:
        print(line(r))
    print()
    print("cells: %d | match: %d | vocab-gap: %d | other: %d" % (
        len(rows),
        sum(1 for r in rows if r[2] == "match"),
        sum(1 for r in rows if r[2] == "vocab-gap"),
        sum(1 for r in rows if r[2] not in ("match", "vocab-gap"))))


if __name__ == "__main__":
    sys.exit(main())

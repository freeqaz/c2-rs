#!/usr/bin/env python3
"""renumber2.py — move ALL TEN of this lane's board rows to #1017-#1026.

THE THIRD COLLISION. The brief allocated w-splice #986-#995. Every landing since
has eaten into that range and into the block this lane fled to:

    w-drop3   #984-#989      (first collision — this lane moved 986-989 -> 1006-1009)
    w-inread  #996-#1005      (broke the first escape plan)
    w-inl0    #990-#995       (collides with the SIX rows that never moved)
    w-relo    #1006-#1016      claimed on its branch, landing right after this one
                               (collides with the FOUR that did move)

So both halves collide now, not just the four the coordinator named. Ten rows
need ten free numbers, and the first free block above everything claimed —
master tops at #1005 and w-relo reserves through #1016 — is **#1017-#1026**.

The map preserves this lane's own reading order (the headline row first), so the
rung's narrative and the board agree:

    1006 -> 1017   the mechanism shipped          990 -> 1021   the relocation check
    1007 -> 1018   S3 off the IL for a Seq        991 -> 1022   chain truncated / open
    1008 -> 1019   the inline decision            992 -> 1023   the refusal census
    1009 -> 1020   c2 closes the chain            993 -> 1024   the family spread
                                                  994 -> 1025   text_comdat_relocs
                                                  995 -> 1026   build refuses a spliced bundle

WHY THIS IS NOT A `sed`, AGAIN. The first renumbering attempt used a regex
behind a "does this line name another lane" guard, and that guard is unsound: a
cross-reference need not name the lane it cites. It rewrote `docs/STATUS.md`'s
*"Boards #984-#989"* — **w-drop3's** — into a dangling range.

So every site is listed explicitly with the text that must be there, a missing
site is a hard failure rather than a skip, and afterwards the script asserts that
named pieces of four other lanes' records still read exactly as they did.
"""

import re
import sys

MAP = {
    "1006": "1017", "1007": "1018", "1008": "1019", "1009": "1020",
    "990": "1021", "991": "1022", "992": "1023", "993": "1024",
    "994": "1025", "995": "1026",
}

BOARD = "docs/BOARD.md"

# Cross-references to this lane's rows, by file, as (exact old, exact new).
XREF = [
    ("docs/rungs/2026-08-08-w-splice.md",
     "rows **#990**–**#995** and **#1006**–**#1009**",
     "rows **#1017**–**#1026**"),
    ("docs/INLINE_PREDICATE.md",
     "> rows **#990**–**#995**, **#1006**–**#1009**.",
     "> rows **#1017**–**#1026**."),
    ("docs/INLINE_PREDICATE.md",
     "**never binds on today's port** (#1008)",
     "**never binds on today's port** (#1019)"),
    ("docs/INLINE_PREDICATE.md",
     "**And it found that mechanism I is a FIXPOINT, like E** (#1009)",
     "**And it found that mechanism I is a FIXPOINT, like E** (#1020)"),
    ("docs/INLINE_PREDICATE.md",
     "the port's frame bookkeeping** (#1007)",
     "the port's frame bookkeeping** (#1018)"),
    ("docs/INLINE_PREDICATE.md",
     "**c2 CLOSES THE CHAIN** (#1009)", "**c2 CLOSES THE CHAIN** (#1020)"),
    ("docs/INLINE_PREDICATE.md",
     "that bound never binds (#1008)", "that bound never binds (#1019)"),
    ("docs/FUNCTION_BYTE_MATCH.md",
     "rule (#1009, #991)", "rule (#1020, #1022)"),
    ("docs/STATUS.md",
     "(2) **Mechanism I is a FIXPOINT too** (#1009)",
     "(2) **Mechanism I is a FIXPOINT too** (#1020)"),
    ("docs/STATUS.md",
     "Boards **#990**–**#995**\n> > > > and **#1006**–**#1009**;",
     "Boards **#1017**–**#1026**;"),
    ("crates/c2-core/src/splice.rs",
     "the wrong-relocation defect (#1009's 72 witnesses)",
     "the wrong-relocation defect (#1020's 72 witnesses)"),
]

# Other lanes' records that must read EXACTLY the same afterwards.
UNTOUCHED = [
    ("docs/STATUS.md", "Boards **#984**–**#989**;"),
    ("docs/STATUS.md", "Boards **#990**–**#995**;"),          # w-inl0's own
    ("docs/BOARD.md", "| **986**<sub>w-drop3</sub> |"),
    ("docs/BOARD.md", "| **990**<sub>w-inl0</sub> |"),
    ("docs/BOARD.md", "| **995**<sub>w-inl0</sub> |"),
    ("docs/BOARD.md", "| **996**<sub>w-inread</sub> |"),
    ("docs/BOARD.md", "| **1005**<sub>w-inread</sub> |"),
]


def main():
    bad = 0

    # ---- the rows themselves, matched on the lane tag so no other lane's
    #      row of the same number can be touched --------------------------
    s = open(BOARD).read()
    moved = 0
    for old, new in sorted(MAP.items(), key=lambda kv: -int(kv[0])):
        tag = "| **%s**<sub>w-splice</sub> |" % old
        if tag not in s:
            print("MISSING board row: %s" % tag)
            bad += 1
            continue
        s = s.replace(tag, "| **%s**<sub>w-splice</sub> |" % new, 1)
        moved += 1
    # this lane's own in-row citations
    for old, new in [("(**#1009**, **#991**)", "(**#1020**, **#1022**)")]:
        if old in s:
            s = s.replace(old, new)
    open(BOARD, "w").write(s)
    print("  %-40s %d row(s) moved" % (BOARD, moved))

    # ---- cross-references ------------------------------------------------
    per = {}
    for path, old, new in XREF:
        per.setdefault(path, []).append((old, new))
    for path, edits in per.items():
        t = open(path).read()
        for old, new in edits:
            if old not in t:
                print("MISSING xref in %s: %r" % (path, old[:70]))
                bad += 1
                continue
            t = t.replace(old, new, 1)
        open(path, "w").write(t)
        print("  %-40s %d xref(s)" % (path, len(edits)))

    # ---- and nobody else's record moved ----------------------------------
    for path, text in UNTOUCHED:
        if text not in open(path).read():
            print("CLOBBERED another lane's text in %s: %r" % (path, text[:60]))
            bad += 1

    # ---- the board is a set ---------------------------------------------
    n = [int(m.group(1)) for l in open(BOARD)
         if (m := re.match(r"^\| \*\*(\d+)\*\*<sub>", l))]
    import collections
    dup = sorted(k for k, v in collections.Counter(n).items() if v > 1)
    if dup:
        print("DUPLICATE board numbers remain: %s" % dup)
        bad += 1

    if bad:
        sys.exit("renumbering incomplete: %d problem(s)" % bad)
    print("w-splice now holds #1017-#1026; %d rows, %d distinct, 0 duplicates"
          % (len(n), len(set(n))))


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""renumber.py — move this lane's four colliding board rows to the free block.

WHAT HAPPENED. The brief allocated lane `w-splice` **#986-#995**. Two lanes
landed on master while this one ran and minted numbers out of that range:

    w-drop3   #984-#989   (so #986 #987 #988 #989 were claimed twice)
    w-inread  #996-#1005

`docs/BOARD.md`: *"Numbers are never reused and never renumbered."* Two rows
carrying one number is the failure that rule exists to prevent, and exactly one
of the two can keep it — the one already landed. So this lane's four move.
**#990-#995 are untouched**: no other lane took them.

    986 -> 1006    987 -> 1007    988 -> 1008    989 -> 1009

WHY THIS IS NOT A `sed`. A first attempt rewrote every `#98[6-9]` in five files
behind a "does this line name another lane" guard, and that guard is wrong: a
cross-reference does not have to name the lane it cites. It rewrote
`docs/STATUS.md`'s *"Boards #984-#989"*, which is **w-drop3's** citation, into a
dangling range — corrupting another lane's record while fixing this one's.

So every site is listed **explicitly** below, with the text that has to be
there. A site that does not match is a hard failure, not a skip: a renumbering
that silently misses a reference leaves a dangling board citation, which is the
same defect one step later.
"""

import sys

MAP = {"986": "1006", "987": "1007", "988": "1008", "989": "1009"}

# (path, exact old text, exact new text). Every one must be present.
SITES = [
    # ---- docs/BOARD.md: this lane's four rows, by their row prefix ---------
    ("docs/BOARD.md",
     "| **986**<sub>w-splice</sub> |", "| **1006**<sub>w-splice</sub> |"),
    ("docs/BOARD.md",
     "| **987**<sub>w-splice</sub> |", "| **1007**<sub>w-splice</sub> |"),
    ("docs/BOARD.md",
     "| **988**<sub>w-splice</sub> |", "| **1008**<sub>w-splice</sub> |"),
    ("docs/BOARD.md",
     "| **989**<sub>w-splice</sub> |", "| **1009**<sub>w-splice</sub> |"),
    # ---- and this lane's own cross-references INSIDE its rows -------------
    ("docs/BOARD.md",
     "which is what separates it from **#968**'s enumeration of 726",
     "which is what separates it from **#968**'s enumeration of 726"),  # no-op anchor
    ("docs/BOARD.md",
     "(**#989**, **#991**)", "(**#1009**, **#991**)"),
    # ---- the rung ---------------------------------------------------------
    ("docs/rungs/2026-08-08-w-splice.md",
     "rows **#986**–**#995**", "rows **#990**–**#995** and **#1006**–**#1009**"),
    # ---- INLINE_PREDICATE.md: all six are this lane's ---------------------
    ("docs/INLINE_PREDICATE.md",
     "> rows **#986**–**#995**.", "> rows **#990**–**#995**, **#1006**–**#1009**."),
    ("docs/INLINE_PREDICATE.md",
     "**never binds on today's port** (#988)", "**never binds on today's port** (#1008)"),
    ("docs/INLINE_PREDICATE.md",
     "**And it found that mechanism I is a FIXPOINT, like E** (#989)",
     "**And it found that mechanism I is a FIXPOINT, like E** (#1009)"),
    ("docs/INLINE_PREDICATE.md",
     "the port's frame bookkeeping** (#987)", "the port's frame bookkeeping** (#1007)"),
    ("docs/INLINE_PREDICATE.md",
     "**c2 CLOSES THE CHAIN** (#989)", "**c2 CLOSES THE CHAIN** (#1009)"),
    ("docs/INLINE_PREDICATE.md",
     "that bound never binds (#988)", "that bound never binds (#1008)"),
    # ---- FUNCTION_BYTE_MATCH.md: this lane's addendum only ----------------
    ("docs/FUNCTION_BYTE_MATCH.md",
     "Each round changed the\n   > rule (#989, #991)", "Each round changed the\n   > rule (#1009, #991)"),
    # ---- STATUS.md: this lane's paragraph only. The file ALSO carries
    #      w-drop3's "Boards #984-#989", which must NOT move.
    ("docs/STATUS.md",
     "> > > > (2) **Mechanism I is a FIXPOINT too** (#989)",
     "> > > > (2) **Mechanism I is a FIXPOINT too** (#1009)"),
    ("docs/STATUS.md",
     "> > > > roots and **87 %** of them `??0?$_List_iterator`. Boards **#986**–**#995**;",
     "> > > > roots and **87 %** of them `??0?$_List_iterator`. Boards **#990**–**#995**\n> > > > and **#1006**–**#1009**;"),
]

# Sites that must be LEFT ALONE, asserted afterwards — another lane's citations
# that a looser rewrite would have taken.
UNTOUCHED = [
    ("docs/STATUS.md", "Boards **#984**–**#989**;"),
    ("docs/BOARD.md", "| **986**<sub>w-drop3</sub> |"),
    ("docs/BOARD.md", "| **989**<sub>w-drop3</sub> |"),
    ("docs/BOARD.md", "| **996**<sub>w-inread</sub> |"),
]


def main():
    per_file = {}
    for path, old, new in SITES:
        per_file.setdefault(path, []).append((old, new))

    bad = 0
    for path, edits in per_file.items():
        s = open(path).read()
        for old, new in edits:
            if old not in s:
                print("MISSING in %s: %r" % (path, old[:70]))
                bad += 1
                continue
            if old == new:
                continue
            s = s.replace(old, new)
        open(path, "w").write(s)
        print("  %-40s %d site(s)" % (path, len(edits)))

    for path, text in UNTOUCHED:
        if text not in open(path).read():
            print("CLOBBERED another lane's text in %s: %r" % (path, text[:60]))
            bad += 1
    if bad:
        sys.exit("renumbering incomplete: %d problem(s)" % bad)
    print("renumbered 986->1006  987->1007  988->1008  989->1009; "
          "990-995 unchanged; no other lane's text touched")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""roots.py — GRID P's IL fact, read off the `.ex` and NOT off the obj.

**This is what the carrier is FOR.** `H-2Z` is 3 wrong on GRID P and all three
misses are `P7` (`CHAINBIND`) — a bind whose base is another bind, stored
through `m` with the value spelled `(int)&k`. `gridp.py` scored it
roots-DIFFER, because `PREREG.md` §3.3 committed in advance to board #1128 /
`IlOp::BoundAddr`'s own rule: *a bind's root token is its OWN token, never the
thing it hangs off.* c2 answers as though the roots were the SAME.

Two readings survive that, and **only the IL separates them**:

    (A) c2 roots `m`'s designator at `k` — the chain COLLAPSES, the roots are
        the same, and `w-self2b`'s #1231 predicate is still exactly right. What
        was wrong is the GENERATOR's assumption, i.e. #1128 does not extend to
        a bind whose base is a bind.
    (B) the two roots really are distinct tokens in the `.ex`, and the #1231
        predicate is WRONG on `P7`.

Those have opposite consequences for the next lane, and no count of objs can
tell them apart. This decodes the `.ex` and says which.

The decoder is `work/w-ilx/exdec.py`, ported from `crates/c2-il`'s own readers
so the decode is checked against a reader that ships rather than invented here.
It reads the IL and nothing else — no obj, no disassembly, no register.

SHIPS NOTHING.
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "work", "w-ilx"))
import exdec                                                    # noqa: E402
from gridp import DC3, cells                                    # noqa: E402

C2RS = os.path.join(ROOT, "target", "release", "c2rs")
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")
CELL = os.path.join(HERE, "cell")

# One representative per family, all at the SAME point, so nothing but the
# spelling varies across the table. `(2,4)` is where `P7` first misses.
SHOW = ["P1-r2k4", "P2-r2k4", "P3-r2k4", "P4-r2k4", "P5-r2k4",
        "P6-r2k4", "P7-r2k4", "P8-r2k4", "P9-r2k4"]

# c2's OWN answer, from `work/w-prod/grade.out` — copied here so the table
# carries the graded column beside the decode and neither can drift silently.
OBJ = {"P1-r2k4": "const", "P2-r2k4": "const", "P3-r2k4": "prod",
       "P4-r2k4": "prod", "P5-r2k4": "const", "P6-r2k4": "prod",
       "P7-r2k4": "const", "P8-r2k4": "prod", "P9-r2k4": "prod"}


def capture(src):
    os.makedirs(CELL, exist_ok=True)
    cpp = os.path.join(CELL, "c.cpp")
    ildir = os.path.join(CELL, "il")
    with open(cpp, "w") as f:
        f.write(src)
    os.makedirs(ildir, exist_ok=True)
    for fn in os.listdir(ildir):
        p = os.path.join(ildir, fn)
        if os.path.isfile(p):
            os.remove(p)
    r = subprocess.run([C2RS, "capture", os.path.relpath(cpp, DC3),
                        "--keep-il", ildir, "--flags-file", FLAGS,
                        "--cwd", DC3], capture_output=True, text=True,
                       cwd=ROOT)
    out = {}
    if r.returncode == 0:
        for fn in sorted(os.listdir(ildir)):
            p = os.path.join(ildir, fn)
            if os.path.isfile(p):
                out[os.path.splitext(fn)[1] or fn] = open(p, "rb").read()
    return out


def main():
    by = {c.name: c for c in cells()}
    rows = []
    for name in SHOW:
        c = by[name]
        st = capture(c.source())
        ex = st.get(".ex")
        if ex is None:
            print("  %-10s CAPTURE FAILED (a counter, not a verdict)" % name)
            continue
        binds, assigns = exdec.decode_body(ex)
        # The PRODUCER's stores are the address-valued ones. Identify them by
        # the value's KIND, never by a register and never by a source string.
        prod = [a for a in assigns if a["v"][0] == "addr-load"]
        if not prod:
            print("  %-10s no address-valued store decoded (a counter, not a"
                  " verdict)" % name)
            continue
        a = prod[0]
        ltok, llits = a["l"]
        vtok, vlits = a["v"][1], a["v"][2]
        rows.append((name, c.klass, ltok, ltok in binds, llits,
                     vtok, vtok in binds, vlits, len(binds)))

    print("  THE IL FACT — one representative per family, all at (ru,cu)=(2,4)")
    print("  `BIND` marks a token the decoder found as a temp bind head\n")
    print("  %-10s %-22s %-26s %-26s %s" %
          ("cell", "class", "STORE designator root", "VALUE expr root",
           "#binds"))
    print("  " + "-" * 96)
    for (name, klass, lt, lb, ll, vt, vb, vl, nb) in rows:
        print("  %-10s %-22s %-26s %-26s %d"
              % (name, klass,
                 "tok 0x%04x %-6s %s" % (lt, "BIND" if lb else "formal", ll),
                 "tok 0x%04x %-6s %s" % (vt, "BIND" if vb else "formal", vl),
                 nb))

    print("\n  %-10s %-22s %-9s %-9s %-8s %s"
          % ("cell", "class", "roots", "store is", "#1231", "obj (GRID P)"))
    print("  " + "-" * 84)
    wrong = []
    for (name, klass, lt, lb, ll, vt, vb, vl, nb) in rows:
        pred = "prod" if (lb and lt != vt) else "const"
        got = OBJ.get(name, "?")
        mark = ""
        if got in ("prod", "const") and pred != got:
            mark = "  **#1231 WRONG**"
            wrong.append(name)
        print("  %-10s %-22s %-9s %-9s %-8s %s%s"
              % (name, klass, "DIFFER" if lt != vt else "same",
                 "a BIND" if lb else "a formal", pred, got, mark))

    print("\n  THE READING, decided by the decode and not by a story about it:")
    if not rows:
        print("    NOTHING DECODED — this is a counter, not a result.")
        return 1
    p7 = [r for r in rows if r[0] == "P7-r2k4"]
    if not p7:
        print("    P7 did not decode — the question this file exists for is"
              " UNANSWERED.")
        return 1
    _, _, lt, lb, ll, vt, vb, vl, _ = p7[0]
    if lt == vt:
        print("""\
    (A) THE CHAIN COLLAPSES.  `F& m = k;` does NOT get a root token of its
        own — c2 roots `m`'s designator at the SAME token as `k`, so the two
        roots are equal and no bonus attaches.  `w-self2b`'s #1231 predicate
        is UNTOUCHED and still reproduces every graded cell.

        What was wrong is the GENERATOR's assumption, and it is the assumption
        `PREREG.md` §3.3 committed to in advance and named as under test:
        board #1128's "a bind's root token is its OWN token" does NOT extend to
        a bind whose base is another bind.

        `H-2Z` is refuted anyway, and this is WHY it is refuted rather than
        merely THAT it is: a rule stated over root tokens cannot be scored from
        the source spelling, because the source spelling and the decoded root
        DISAGREE on this class.  Every one of the ten dead keys was scored from
        a source spelling.""")
    else:
        print("""\
    (B) THE ROOTS ARE DISTINCT TOKENS and c2 answers as though they were not.
        `w-self2b`'s #1231 predicate is WRONG on `CHAINBIND`, on a cell it has
        never seen, and the decoded fact needs a term nobody has named.""")
    print("\n    P7 store root  tok 0x%04x %s %s" %
          (lt, "BIND" if lb else "formal", ll))
    print("    P7 value root  tok 0x%04x %s %s" %
          (vt, "BIND" if vb else "formal", vl))
    print("\n  #1231's predicate, scored on the decode: %d WRONG of %d%s"
          % (len(wrong), len(rows),
             ("   " + " ".join(wrong)) if wrong else ""))
    print("""
  THE OFFSET-ADD LISTS ABOVE ARE THE #908 HALF.  They are printed because the
  decoder can carry them; `crates/c2-il`'s `eat_offset_adds_list` now returns
  them too, and `alloc::Root::offsets` has the slot.  The seam between the two
  — `IlOp::BoundAddr`, whose `off` is a SUM — is the one named gap this lane
  leaves open, and it is a field and not an unknown.""")
    return 0


if __name__ == "__main__":
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        sys.exit(3)
    sys.exit(main())

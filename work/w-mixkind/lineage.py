#!/usr/bin/env python3
"""lineage.py — GRID X's IL fact, read off the `.ex` and NOT off the obj.

Two questions this lane owes an answer to, and neither can be settled by a count
of objs:

  **P4 (PREREG §6.3).** `M6`, `M7`, `M10` and `M12` declare in advance that
  `T& f = c;` where `T& c = a;` produces a **depth-3 bind chain** in the `.ex`
  rather than being flattened by the front end. `--freeze` could not check it.
  If the chain is flat those families are OUT OF REGIME and not evidence for
  anything, and this file says which.

  **How many levels the ALLOCATION fact needs.** `H-DERIV` is 0 wrong of GRID
  X's 60 and it is stated over a **transitive** relation — *the value's root is
  neither an ancestor nor a descendant of the store's root, through bind links*.
  `alloc::Root::base` (board #1244) carries **one** level per root. So the
  question is whether one level is enough, and `M6` is the cell that decides it:
  the value root is the store root's GRANDPARENT, so a one-level test says
  "unrelated" and c2 says otherwise.

This prints, per family, the FULL decoded bind table beside both roots, walks
the lineage, and **asserts the comparison rather than eyeballing it**.

The decoder is `work/w-ilx/exdec.py`, ported from `crates/c2-il`'s own readers.
It reads the IL and nothing else — no obj, no disassembly, no register. The
`obj` column is copied from this lane's own `grade.out` so the decode and the
graded answer sit side by side and neither can drift silently.

SHIPS NOTHING.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "work", "w-ilx"))
sys.path.insert(0, os.path.join(ROOT, "work", "w-prod"))
import exdec                                                    # noqa: E402
from gridx import cells                                        # noqa: E402
import roots as wprod_roots                                    # noqa: E402

# One representative per in-domain family, all at (2,4) — the deciding point —
# so nothing but the SPELLING varies across the table.
SHOW = ["M%d-r2k4" % i for i in range(1, 13)]


def observed():
    out = {}
    for line in open(os.path.join(HERE, "grade.out")):
        m = re.match(r"^\s{4}(\S+)\s+(prod|const|OOR.*|COMPILE-FAILED)\s*$", line)
        if m:
            out[m.group(1)] = m.group(2)
    return out


def lineage(binds, tok):
    """The token's own chain: itself, then each successive base that is ITSELF
    a bind head, walked to the first non-bind base."""
    out, cur, guard = [], tok, 0
    while cur in binds and guard < 16:
        out.append(cur)
        cur = binds[cur][0]
        guard += 1
    return out


def main():
    by = {c.name: c for c in cells()}
    obs = observed()
    rows = []
    for name in SHOW:
        c = by[name]
        st = wprod_roots.capture(c.source())
        ex = st.get(".ex")
        if ex is None:
            print("  %-10s CAPTURE FAILED (a counter, not a verdict)" % name)
            continue
        binds, assigns = exdec.decode_body(ex)
        prod = [a for a in assigns if a["v"][0] == "addr-load"]
        if not prod:
            print("  %-10s no address-valued store decoded (a counter, not a"
                  " verdict)" % name)
            continue
        a = prod[0]
        rows.append({
            "name": name, "klass": c.klass, "binds": binds,
            "lt": a["l"][0], "ll": list(a["l"][1]),
            "vt": a["v"][1], "vl": list(a["v"][2]),
            "obj": obs.get(name, "?"),
        })

    if not rows:
        print("  NOTHING DECODED — this is a counter, not a result.")
        return 1

    # ---------------------------------------------------------------- P4
    print("  THE BIND TABLES — is a chained reference a CHAIN in the `.ex`?\n")
    depth = {}
    for r in rows:
        chain = lineage(r["binds"], r["lt"])
        depth[r["name"]] = len(chain)
        print("  %-10s %-14s  %d bind(s) decoded" %
              (r["name"], r["klass"], len(r["binds"])))
        for t in sorted(r["binds"]):
            b, l = r["binds"][t]
            print("      0x%04x -> base 0x%04x %-8s %s" %
                  (t, b, "(a BIND)" if b in r["binds"] else "", l))
        print("      store root 0x%04x  lineage %s  (depth %d)" %
              (r["lt"], " -> ".join("0x%04x" % t for t in chain), len(chain)))
        print("      value root 0x%04x %s %s" %
              (r["vt"], "BIND" if r["vt"] in r["binds"] else "formal", r["vl"]))
    deep = [n for n in ("M6-r2k4", "M7-r2k4", "M10-r2k4", "M12-r2k4")
            if depth.get(n, 0) >= 3 or n == "M12-r2k4"]
    print("\n  PREREG P4: the depth-3 families decode at store-root depth %s"
          % {n: depth.get(n) for n in ("M6-r2k4", "M7-r2k4", "M10-r2k4")})
    if any(depth.get(n, 0) < 3 for n in ("M6-r2k4", "M7-r2k4", "M10-r2k4")):
        print("  ** THE CHAIN IS FLATTENED — M6/M7/M10 are OUT OF REGIME and"
              " their cells are NOT evidence for a depth claim. **")
    else:
        print("  ** the chain SURVIVES to the `.ex` — the depth axis is real. **")
    print("  (M12 stores through the PATH, so its store-root depth is 0 by"
          " construction: %s)" % depth.get("M12-r2k4"))

    # ------------------------------------------------ how many levels are used
    print("\n  HOW MANY LEVELS THE ANSWER NEEDS")
    print("  %-10s %-14s %-9s %-11s %-11s %-9s %s"
          % ("cell", "class", "rel", "1-LEVEL", "TRANSITIVE", "obj", ""))
    print("  " + "-" * 84)
    bad1 = []
    for r in rows:
        b = r["binds"]
        chain = lineage(b, r["lt"])
        desc = {t for t in b if r["lt"] in lineage(b, t)[1:]}
        rel = ("self" if r["vt"] == r["lt"] else
               "ancestor" if r["vt"] in chain[1:] else
               "descendant" if r["vt"] in desc else
               "unrelated")
        sb = r["lt"] in b
        # ONE LEVEL: only `Root::base` of each side is available.
        #
        # **A correction made before this file was published, and recorded
        # rather than quietly fixed.** The first version omitted the *"and that
        # base is ITSELF a bind head"* conjunct that `H-DERIV`, `H-STEP` and
        # `H-2Z` all carry, so it read `SELF-2B` — whose bind hangs off the
        # FORMAL's path — as related, and came back wrong on `M3` as well as
        # `M6`. That would have overstated the one-level reading's failure by a
        # whole family. Walking a bind link into a formal is not a bind link.
        def _blink(x, y):
            return x in b and b[x][0] == y and y in b
        one_rel = (r["vt"] == r["lt"]
                   or _blink(r["lt"], r["vt"])
                   or _blink(r["vt"], r["lt"]))
        p1 = "prod" if (sb and not one_rel) else "const"
        pt = "prod" if (sb and rel == "unrelated") else "const"
        mark = ""
        if r["obj"] in ("prod", "const") and p1 != r["obj"]:
            mark = "  ** 1-LEVEL WRONG **"
            bad1.append(r["name"])
        print("  %-10s %-14s %-9s %-11s %-11s %-9s%s"
              % (r["name"], r["klass"], rel, p1, pt, r["obj"], mark))

    print("\n  THE READING, decided by the decode and not by a story about it:")
    if bad1:
        print("""
    **ONE LEVEL IS NOT ENOUGH, and `%s` is the proof.** `alloc::Root::base`
    carries the store root's OWN base and stops. On these cells the value root
    is the store root's GRANDPARENT — two links away — so every predicate over
    `(tok, is_bind, base, offsets)` of the two sides reads them as UNRELATED and
    predicts the bonus, and real `c2` does not give it.

    So board #1244 closed the gap it named and named it correctly, and the fact
    is one level further down still: the relation the bytes obey is the
    TRANSITIVE bind lineage, and the carrier holds one link of it.""" %
              " / ".join(bad1))
    else:
        print("""
    ONE LEVEL IS ENOUGH on every cell this grid decoded — the transitive walk
    and the one-level test agree everywhere, so this grid does NOT establish
    that `alloc::Root::base` is short. That is a negative result about the
    grid's reach, not a licence for the one-level reading.""")
    return 0


if __name__ == "__main__":
    sys.exit(main())

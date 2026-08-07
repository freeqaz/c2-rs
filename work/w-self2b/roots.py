#!/usr/bin/env python3
"""roots.py — the IL fact, read off the `.ex` and NOT off the obj.

`w-mixed` §4.4 established that the difference between the two spellings of one
address *is* in the `.ex`, by byte-diffing two streams. That says the fact is
readable; it does not say WHAT the fact is. This file names it, decodes it, and
prints it for one representative cell of every GRID Z family — so the claim
"H-2X reads root symbol tokens" is a decode and not a story about hex.

For each producer store `<lvalue> = (int)<value>` it prints

    lv-tok  the root symbol token of the store DESIGNATOR   (`B9 <tok> <TYPE>`)
    v-tok   the root symbol token of the VALUE expression
    lv/v bind?  whether that token is a TEMP BIND head (`26 <tok>`) rather than
            a formal
    lits    the offset-add literal LIST of each (`33 <int> <varint> 27 <PTR>`),
            which `eat_offset_adds` in the shipping reader sums and cannot
            return (#908)

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
from gridz import DC3, cells, OFF_CORE_U0, OFF_E0               # noqa: E402

C2RS = os.path.join(ROOT, "target", "release", "c2rs")
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")
CELL = os.path.join(HERE, "cell")

# One representative per family, all at the SAME point, so nothing but the
# spelling varies across the table.
SHOW = ["Z1-r2k4", "Z2-r2k4", "Z3-r2k4", "Z4-r2k4", "Z5-r2k4", "Z6-r2k4"]


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
            print("  %-10s CAPTURE FAILED" % name)
            continue
        binds, assigns = exdec.decode_body(ex)
        # The PRODUCER's stores are the address-valued ones.  Identify them by
        # the value's KIND, never by a register and never by a source string.
        prod = [a for a in assigns if a["v"][0] == "addr-load"]
        if not prod:
            print("  %-10s no address-valued store decoded (counter, not a"
                  " verdict)" % name)
            continue
        a = prod[0]
        ltok, llits = a["l"]
        vtok, vlits = a["v"][1], a["v"][2]
        rows.append((name, c.klass, ltok, ltok in binds, llits,
                     vtok, vtok in binds, vlits, len(prod)))

    print("  THE IL FACT — one representative per family, all at (ru,cu)=(2,4)")
    print("  binds decoded per cell are printed as `bind` on the token that is"
          " one\n")
    print("  %-10s %-22s %-22s %-22s" %
          ("cell", "class", "STORE designator root", "VALUE expr root"))
    print("  " + "-" * 82)
    for (name, klass, lt, lb, ll, vt, vb, vl, n) in rows:
        print("  %-10s %-22s %-22s %-22s"
              % (name, klass,
                 "tok 0x%02x %-6s %s" % (lt, "BIND" if lb else "formal", ll),
                 "tok 0x%02x %-6s %s" % (vt, "BIND" if vb else "formal", vl)))
    print("\n  %-10s %-22s %-9s %-9s %s"
          % ("cell", "class", "roots", "store is", "obj (GRID Z)"))
    print("  " + "-" * 72)
    OBJ = {"Z1-r2k4": "const", "Z2-r2k4": "const", "Z3-r2k4": "prod",
           "Z4-r2k4": "prod", "Z5-r2k4": "const", "Z6-r2k4": "prod"}
    for (name, klass, lt, lb, ll, vt, vb, vl, n) in rows:
        print("  %-10s %-22s %-9s %-9s %s"
              % (name, klass, "DIFFER" if lt != vt else "same",
                 "a BIND" if lb else "a formal", OBJ.get(name, "?")))
    print("""
  READ IT OFF THE TABLE, not off a story about it:

    * the two spellings differ in the `.ex` in exactly the ROOT TOKEN of one
      `B9 <tok> <TYPE>` and in the offset-add literal LIST that follows it;
    * `prod` appears exactly where the STORE designator's root token is a BIND
      **and** the value expression's root token is a different token;
    * `Z5` has DIFFERING roots and is `const`, so the relation is NOT symmetric
      in the two tokens — which is what refutes H-2X;
    * `Z2` has a BIND store root and is `const`, so "the stores go through a
      bind" is not enough either — which is what refuted H-MIX.

  Both facts are relations between TWO `B9` roots plus one bit about one of
  them.  `alloc::Producer` carries `uses`, `kind` and `first`, and c2-il's
  `eat_offset_adds` returns the SUM of the literal list rather than the list
  (#908).  Neither can hold this.""")
    return 0


if __name__ == "__main__":
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        sys.exit(3)
    sys.exit(main())

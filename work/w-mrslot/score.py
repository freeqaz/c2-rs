#!/usr/bin/env python3
"""score.py — score the two readings of board #584's `u` against real c2 bytes.

    python3 work/w-mrslot/score.py [--tag base]

**Nothing here asks the port.**  `u_lead` is defined over the FINAL store order,
and real `c2.dll`'s disassembly *shows* the final store order, so every quantity
below is read out of `grid/<cell>/dis.txt` — c2's own emitted words at the
workload's own flags.  A cell whose port-side store order is itself wrong shows
up as a byte mismatch in `c2rs gap`, not as a silently re-labelled class.

    COUNT   (what ships today)   save_slot = nprod - 1 + min(#unproduced, 2)
    LEAD    (#584, the rung)     save_slot = nprod - 1 + min(u_lead,      2)

            u_lead = the LEADING run of unproduced stores in the FINAL order.

**THE SCORER ASSERTS ITS OWN CLASSES** on the OBSERVED population and exits
non-zero if the grid cannot decide the rung — `w-carrier`'s GRID K could not,
and it was green through four wrong emits (board #1211).
"""

import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(HERE, "grid")

PARAM_REGS = {3: "this", 4: "p", 5: "q"}
STORE = re.compile(r"^(stw|sth|stb|std)\s+(\d+),\s*(-?\d+)\((\d+)\)")
LOADIMM = re.compile(r"^(li|lis)\s+(\d+),")
MR = re.compile(r"^mr\s+(\d+),\s*(\d+)")
BL = re.compile(r"^(bl|b)\s")
TEXTLINE = re.compile(r"^\s{3}[0-9a-f]{4}\s{2}[0-9a-f]{8}\s{2}(.*?)\s*$")


def text_of(cell):
    """The `.text` mnemonics of the cell's single function, in order."""
    p = os.path.join(GRID, cell, "dis.txt")
    if not os.path.exists(p):
        return None
    out, inside = [], False
    for line in open(p):
        if line.startswith("-- .text"):
            inside = True
            continue
        if line.startswith("--") and inside:
            break
        if inside:
            m = TEXTLINE.match(line.rstrip("\n"))
            if m:
                out.append(re.sub(r"\s+", " ", m.group(1).replace("\t", " ")).strip())
    return out


class Obs:
    """What c2 emitted, as the four numbers the rung is about."""

    def __init__(self, cell):
        self.cell = cell
        self.text = text_of(cell)
        self.ok = False
        if not self.text:
            self.why = "no disassembly"
            return
        prods, stores, slot, seen, self.has_call = set(), [], None, 0, False
        for ins in self.text:
            m = LOADIMM.match(ins)
            if m:
                prods.add(int(m.group(2)))
                continue
            m = STORE.match(ins)
            if m:
                src, off, base = int(m.group(2)), int(m.group(3)), int(m.group(4))
                # -8(1)/-16(1) are the prologue's LR and r31 saves, not the run
                if base == 1:
                    continue
                stores.append((src, off, base))
                seen += 1
                continue
            m = MR.match(ins)
            if m and int(m.group(1)) == 31 and int(m.group(2)) == 3 and slot is None:
                slot = seen
                continue
            if BL.match(ins):
                self.has_call = True
        self.nprod = len(prods)
        self.prods = prods
        # A store materialises NOTHING exactly when its source register is a
        # live-in formal rather than a register some `li` wrote.
        self.kinds = ["F" if src in PARAM_REGS and src not in prods else "L"
                      for src, _, _ in stores]
        self.count = sum(1 for k in self.kinds if k == "F")
        self.lead = 0
        for k in self.kinds:
            if k != "F" or self.lead >= 2:
                break
            self.lead += 1
        self.slot = slot
        self.nstores = len(stores)
        self.stores = stores
        self.ok = True
        self.why = ""

    def pred(self, u):
        if self.nprod == 0 and u < 2:
            return None                     # REFUSED_EMPTY_POOL, both readings
        return self.nprod + min(u, 2) - 1

    @property
    def pred_count(self):
        return self.pred(self.count)

    @property
    def pred_lead(self):
        return self.pred(self.lead)


def klass_of(cell):
    p = os.path.join(GRID, cell, cell + ".cpp")
    for line in open(p):
        m = re.search(r"class=(\S+)", line)
        if m:
            return m.group(1)
    return "?"


def spec_of(cell):
    p = os.path.join(GRID, cell, cell + ".cpp")
    src = open(p).read()
    s = re.search(r"syms=(\S+) vals=(\S+)", src)
    return (s.group(1), s.group(2)) if s else ("?", "?")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", default="base")
    a = ap.parse_args()

    cells = sorted(os.listdir(GRID))
    rows, sep, hitL, missL, hitC, missC = [], [], 0, 0, 0, 0
    leafs = {}
    for c in cells:
        if klass_of(c) == "leaf-control":
            leafs[spec_of(c)] = c

    print("%-16s %-15s %5s %5s %5s %5s %6s %6s %6s" %
          ("cell", "class", "nprod", "cnt", "lead", "obs", "COUNT", "LEAD", "verdict"))
    for c in cells:
        o = Obs(c)
        k = klass_of(c)
        if not o.ok:
            print("%-16s %-15s  %s" % (c, k, o.why))
            continue
        if not o.has_call or o.slot is None:
            # leaf controls and any body with no `mr r31,r3` — nothing to score
            rows.append((c, k, o, None, None))
            continue
        pc, pl = o.pred_count, o.pred_lead
        vl = "LEAD-HIT" if pl == o.slot else "LEAD-MISS"
        vc = "COUNT-HIT" if pc == o.slot else "COUNT-MISS"
        hitL += pl == o.slot
        missL += pl != o.slot
        hitC += pc == o.slot
        missC += pc != o.slot
        if pc != pl:
            sep.append(c)
        print("%-16s %-15s %5d %5d %5d %5s %6s %6s  %s %s" %
              (c, k, o.nprod, o.count, o.lead, o.slot,
               "-" if pc is None else pc, "-" if pl is None else pl, vl, vc))
        rows.append((c, k, o, pc, pl))

    print()
    print("SCORED (framed cells with an observed `mr r31,r3`): %d" % (hitL + missL))
    print("  LEADING RUN (#584, the rung)   %d HIT  %d MISS" % (hitL, missL))
    print("  COUNT       (what ships today) %d HIT  %d MISS" % (hitC, missC))
    print("  cells that SEPARATE the two readings: %d  %s"
          % (len(sep), " ".join(sep[:8]) + (" ..." if len(sep) > 8 else "")))

    # ---- the #1169 separator: is the framed run the LEAF's run? -----------
    same = diff = nopair = 0
    for c, k, o, pc, pl in rows:
        if not o.has_call or o.slot is None:
            continue
        # A `base-bind-live` cell binds off a FORMAL, so its bound stores use
        # that formal's register as the base where the leaf twin uses `this`.
        # The pair differs in a base register by construction and says nothing
        # about the schedule, which is what this check is for.
        if k == "base-bind-live":
            nopair += 1
            continue
        lc = leafs.get(spec_of(c))
        if lc is None:
            nopair += 1
            continue
        lt = text_of(lc)
        if lt is None:
            nopair += 1
            continue
        strip = lambda t: [i for i in t
                           if not re.match(r"^(mflr|stwu|addi 1,|lwz 12|mtlr|blr|"
                                           r"std 31|ld 31|stw 12|bl |b |mr 3, 31|"
                                           r"mr 31, 3)", i)]
        if strip(o.text) == strip(lt):
            same += 1
        else:
            diff += 1
            print("  RUN DIFFERS FROM ITS LEAF TWIN: %s vs %s" % (c, lc))
            print("     framed %s" % strip(o.text))
            print("     leaf   %s" % strip(lt))
    print("  framed run == leaf run (mr/frame removed): %d same, %d DIFFER, "
          "%d unpaired" % (same, diff, nopair))

    # ---- THE SCORER ASSERTS ITS OWN CLASSES ------------------------------
    bad = []
    if len(sep) < 3:
        bad.append("only %d cells separate COUNT from LEADING RUN — this grid "
                   "cannot decide the rung (board #1211)" % len(sep))
    seen_lead = {o.lead for _, _, o, pc, pl in rows if o.ok and o.slot is not None}
    for want in (0, 1):
        if want not in seen_lead:
            bad.append("no observed cell with u_lead == %d" % want)
    seen_np = {o.nprod for _, _, o, pc, pl in rows if o.ok and o.slot is not None}
    if not seen_np & {1}:
        bad.append("no observed cell with nprod == 1")
    if hitL + missL < 20:
        bad.append("only %d scored cells" % (hitL + missL))
    for b in bad:
        print("FAIL: " + b, file=sys.stderr)
    if bad:
        sys.exit(2)
    print("  class assertions on the OBSERVED population: OK")
    return 0 if missL == 0 else 1


if __name__ == "__main__":
    sys.exit(main())

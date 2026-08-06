#!/usr/bin/env python3
"""fit.py — score RULE W and every rival on GRID S **and on the four prior
lanes' own committed logs**.

It COMPILES NOTHING.  Every number it prints comes from a log already in the
history: `work/w-spell/spellgrid.out` (this lane), `work/w-refbind/holdout_dis.txt`
and `bindgrid_dis.txt`, `work/w-next/allocgrid.out`, `work/w-seam/grida.out`,
`work/w-alloc2/freshgrid.out`.  That is w-alloc2's `mutate.py` discipline — parse
the grid log, never transcribe it.

THE GENERIC RE-GRADER
---------------------
The prior lanes lay their structs out differently (w-next's constant stores at
offset 0 and its producer at 40; w-refbind's at 32 and 96), so this file cannot
key on displacements.  It keys on the SHAPE instead:

  * collect every store; group by the register stored;
  * require exactly two distinct stored registers, each DEFINED exactly once
    (#644 — a rematerialised load or a two-instruction `addi` is out of regime);
  * the one defined by `li`/`lis` is the CONSTANT, the other is the producer;
  * `ru`/`cu` are the two store counts;
  * H-self is derived from the obj: the producer's defining instruction is
    `addi rD, rB, K`, every producer store is `<st> rD, DISP(rB)` off the SAME
    base, and `K <= DISP < K+32` — the value points at the object it is stored
    into.  No source file is read.

`bases` cannot be read off an obj — the bind is folded into the stores'
displacements (w-refbind §5) — so it is taken from each lane's own construction,
cited per log below and nowhere inferred.

THE `bases` MAPPING FOR `bindgrid`, AND THE RUN THAT GOT IT WRONG
-----------------------------------------------------------------
`work/w-spell/fit_v1.out` is this file's FIRST run, committed before the fix,
because it makes a point that a corrected number cannot.  It mapped
`bases = 1 if "-none-" in the cell name else 2`, which scored six
`P2-shift-*-r2k1` cells as RULE W refutations.  They are not: `ref-unused`,
`ptr-unused`, `ref-other`, `local-int`, `outer-ref` and `val-temp` are the six
modes w-refbind §4 measured as NONE-LIKE, and #865 says why in one line —
an *unused* bind is deleted by the front end, `S& z = *s` names `r3` itself
(displacement 0), and `int w = <expr>` names a value and not a base.  None of
them puts a second store-base value in the body.  The corrected mapping is
#865's own predicate, applied to bindgrid's declared modes, and it is stated
here rather than fitted: **bases = 2 iff the body contains a USED bind of a
sub-object at a non-zero displacement.**

Usage:  fit.py
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rule import RIVALS, rule_w, rule_w2  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))

STORE_RX = re.compile(r"^(st[bhwd]u?)\s+(\d+),\s*(-?\d+)\((\d+)\)$")
DEF_RX = re.compile(r"^([a-z][a-z0-9._]*)\s+(\d+),")
ADDI_RX = re.compile(r"^addi\s+(\d+),\s*(\d+),\s*(-?\d+)$")


def norm(line):
    """Strip a leading index and/or an encoding word from a log line."""
    p = line.split()
    while p and (p[0].isdigit() or (len(p[0]) == 8 and
                                   all(c in "0123456789abcdef" for c in p[0]))):
        p.pop(0)
    return " ".join(p).split(";")[0].strip()


def observe(words):
    """dict(mnem, is_self, ru, cu, winner, first) or a reason string."""
    st = []
    for i, w in enumerate(words):
        m = STORE_RX.match(w)
        if m:
            st.append((i, int(m.group(2)), int(m.group(3)), int(m.group(4))))
    if not st:
        return "no stores"
    regs = sorted({s[1] for s in st})
    if len(regs) != 2:
        return "%d distinct stored registers" % len(regs)
    d = {}
    for r in regs:
        ds = [(i, m.group(1)) for i, m in
              ((i, DEF_RX.match(w)) for i, w in enumerate(words))
              if m and int(m.group(2)) == r and not STORE_RX.match(words[i])]
        if len(ds) != 1:
            return "r%d is defined %d times (#644)" % (r, len(ds))
        d[r] = ds[0]
    consts = [r for r in regs if d[r][1] in ("li", "lis")]
    if len(consts) != 1:
        return "%d of the two producers is an `li`" % len(consts)
    creg = consts[0]
    preg = [r for r in regs if r != creg][0]
    pst = [s for s in st if s[1] == preg]
    cst = [s for s in st if s[1] == creg]
    mnem = d[preg][1]
    is_self = False
    m = ADDI_RX.match(words[d[preg][0]])
    if m:
        k, base = int(m.group(3)), int(m.group(2))
        is_self = all(s[3] == base and k <= s[2] < k + 32 for s in pst)
    return dict(mnem=mnem, is_self=is_self, ru=len(pst), cu=len(cst),
                winner="prod" if preg > creg else "const",
                first="prod" if d[preg][0] < d[creg][0] else "const",
                preg=preg, creg=creg)


def read_dis_log(path, bases_of):
    """A `== <cell>` / instruction-lines log -> [(cell, bases, obs|reason)]."""
    out, name, buf = [], None, []
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("=="):
            if name is not None:
                out.append((name, bases_of(name), observe(buf)))
            name, buf = line[2:].split()[0], []
        elif line.strip() and name is not None:
            w = norm(line)
            if w:
                buf.append(w)
    if name is not None:
        out.append((name, bases_of(name), observe(buf)))
    return out


# ---------------------------------------------------------------- populations
def grid_s():
    """This lane's own table, parsed out of the committed `spellgrid.out`."""
    rx = re.compile(r"^\s+(S-\S+)\s+\|\s+r(\d+)\s+r(\d+)\s+\|\s+(\w+)\s+\|"
                    r"\s+(\w+)\s+\|\s+(\S+)\s*$")
    rows = []
    for line in open(os.path.join(HERE, "spellgrid.out")):
        m = rx.match(line)
        if not m:
            continue
        name, _pr, _cr, winner, _first, mnem = m.groups()
        _s, sp, bases, rk = name.split("-")
        ru, cu = int(rk[1]), int(rk[3])
        rows.append(dict(cell=name, mnem=mnem, is_self=(sp == "self"),
                         ru=ru, cu=cu, bases=1 if bases == "1base" else 2,
                         winner=winner))
    return rows


def from_dis(path, bases_of, label):
    rows, oor = [], 0
    for name, bases, o in read_dis_log(path, bases_of):
        if isinstance(o, str):
            oor += 1
            continue
        rows.append(dict(cell=name, mnem=o["mnem"], is_self=o["is_self"],
                         ru=o["ru"], cu=o["cu"], bases=bases,
                         winner=o["winner"]))
    print("  %-46s %3d graded, %2d out of regime" % (label, len(rows), oor))
    return rows


def grida():
    """w-seam GRID A, from `work/w-seam/grida.out`.  Every cell of that grid
    carries `L& q = s->inner;` (grida.py's HEAD), so bases = 2 throughout."""
    rx = re.compile(r"^\s+(A-\S+)\s+\|\s+reg=r(\d+)\s+const=r(\d+)")
    mn = {"addi-interior": ("addi", True), "add": ("add", False),
          "slwi": ("slwi", False)}
    rows = []
    for line in open(os.path.join(ROOT, "work", "w-seam", "grida.out")):
        m = rx.match(line)
        if not m:
            continue
        name, pr, cr = m.group(1), int(m.group(2)), int(m.group(3))
        body = name[2:]
        for k in mn:
            if body.startswith(k + "-"):
                sp, rest = k, body[len(k) + 1:]
                break
        else:
            continue
        mnem, slf = mn[sp]
        rows.append(dict(cell=name, mnem=mnem, is_self=slf,
                         ru=int(rest[1]), cu=int(rest[3]), bases=2,
                         winner="prod" if pr > cr else "const"))
    print("  %-46s %3d graded" % ("w-seam GRID A (grida.out)", len(rows)))
    return rows


def freshgrid():
    """w-alloc2's fresh holdout, from `work/w-alloc2/freshgrid.out`.  Its cells
    all carry the reference binding (w-alloc2 §4.2 — `freshgrid` inherited it
    from `gapgrid`/`allocgrid`), so bases = 2.  The producer's spelling is read
    off the log's OWN label, not off a source file: `regA` is the fitted
    `(int)&q` (self), `regAdd`/`regAddi`/`regShift` are the F4 spellings."""
    lab = {"regA": ("addi", True), "regAdd": ("add", False),
           "regAddi": ("addi", False), "regShift": ("slwi", False)}
    rx = re.compile(r"^\s+(F\d\S*)\s+(\d)\s+\|\s+(\w+)=r(\d+)\s+(\w+)=r(\d+)\s+\|")
    nm = re.compile(r"r(\d+)k(\d+)")
    rows, skipped = [], 0
    for line in open(os.path.join(ROOT, "work", "w-alloc2", "freshgrid.out")):
        m = rx.match(line)
        if not m:
            continue
        cell, np = m.group(1), int(m.group(2))
        a, ar, b, br = m.group(3), int(m.group(4)), m.group(5), int(m.group(6))
        c = nm.search(cell)
        pair = {a: ar, b: br}
        plab = [k for k in pair if k in lab]
        if np != 2 or not c or len(plab) != 1 or \
                not any(k.startswith("const") for k in pair):
            skipped += 1
            continue
        plab = plab[0]
        clab = [k for k in pair if k != plab][0]
        mnem, slf = lab[plab]
        rows.append(dict(cell=cell, mnem=mnem, is_self=slf,
                         ru=int(c.group(1)), cu=int(c.group(2)), bases=2,
                         winner="prod" if pair[plab] > pair[clab] else "const"))
    print("  %-46s %3d graded, %2d not two-producer/li"
          % ("w-alloc2 fresh holdout (freshgrid.out)", len(rows), skipped))
    return rows


def score(name, rows):
    print("\n  %s — %d cells" % (name, len(rows)))
    print("    %-22s %7s %7s %8s" % ("rule", "right", "WRONG", "refused"))
    for label, fn in RIVALS:
        r = w = ref = 0
        for x in rows:
            p = fn(x["mnem"], x["is_self"], x["ru"], x["cu"], x["bases"])
            if p is None:
                ref += 1
            elif p == x["winner"]:
                r += 1
            else:
                w += 1
        print("    %-22s %7d %7d %8d" % (label, r, w, ref))
    out = {}
    for lab, fn in (("RULE W", rule_w), ("RULE W2", rule_w2)):
        bad = [x for x in rows
               if fn(x["mnem"], x["is_self"], x["ru"], x["cu"], x["bases"])
               not in (None, x["winner"])]
        for x in bad:
            print("      %s WRONG: %-30s mnem=%-6s self=%-5s ru=%d cu=%d "
                  "bases=%d observed=%s"
                  % (lab, x["cell"], x["mnem"], x["is_self"], x["ru"],
                     x["cu"], x["bases"], x["winner"]))
        out[lab] = len(bad)
    return out


# #865's predicate applied to `bindgrid`'s declared modes.  A mode is 2-base
# iff it USES a bind of a sub-object at a non-zero displacement.
BINDGRID_2BASE = ("-ref-", "-ptr-", "-iptr-")
BINDGRID_1BASE = ("-none-", "-ref-unused-", "-ptr-unused-", "-ref-other-",
                  "-local-int-", "-outer-ref-", "-val-temp-")


def bindgrid_bases(name):
    for m in BINDGRID_1BASE:          # longest-first: `-ref-unused-` before
        if m in name:                 # `-ref-`
            return 1
    for m in BINDGRID_2BASE:
        if m in name:
            return 2
    return 1


def main():
    print("POPULATIONS (nothing is compiled here)\n")
    pops = [("GRID S — this lane", grid_s())]
    print("  re-graded from committed disassembly logs:")
    pops.append((
        "w-refbind frozen holdout",
        from_dis(os.path.join(ROOT, "work", "w-refbind", "holdout_dis.txt"),
                 lambda n: 1 if "-none-" in n else 2,
                 "w-refbind holdout (holdout_dis.txt)")))
    pops.append((
        "w-refbind bisection grid",
        from_dis(os.path.join(ROOT, "work", "w-refbind", "bindgrid_dis.txt"),
                 bindgrid_bases,
                 "w-refbind bindgrid (bindgrid_dis.txt)")))
    pops.append((
        "w-next allocation grid",
        from_dis(os.path.join(ROOT, "work", "w-next", "allocgrid.out"),
                 lambda n: 2,
                 "w-next allocgrid (allocgrid.out; bind in all 24)")))
    pops.append(("w-seam GRID A", grida()))
    pops.append(("w-alloc2 fresh holdout", freshgrid()))

    total_wrong = {}
    allrows = []
    for name, rows in pops:
        if not rows:
            continue
        allrows += rows
        total_wrong[name] = score(name, rows)
    score("ALL POPULATIONS COMBINED", allrows)

    print("\n  wrong cells, by population:")
    print("    %-34s %8s %8s" % ("population", "RULE W", "RULE W2"))
    for k, v in total_wrong.items():
        print("    %-34s %8d %8d" % (k, v["RULE W"], v["RULE W2"]))
    print("\n  The incumbent (the shipped refusal) is WRONG on 0 of %d by"
          " construction. A rule wrong ANYWHERE loses to it." % len(allrows))
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""gridgen.py — GRID M, w-midrun / the EMITTER rung under `xboxheap.cpp`.

The rung: an **interior address in a store's VALUE position is a producer** —
one `addi rD,rBase,off` — and `codegen::leaf::store::parse_simple_gpr_run`
declines it, which is what `c2rs gap` reports once the reader rung above it is
lifted.

**STRUCTURAL AXES FIRST, CROSSED; values varied inside each cell.** The axes and
why each is here:

  SPELL  B(ind) / D(irect)   `BE& r = mBlk; r.n0 = &r;`  vs  `mBlk.n0 = &mBlk;`
                             Board #1128: at `xboxheap`'s width the two
                             spellings are DIFFERENT BODIES.  `w-carrier`'s
                             `k_both1`/`k_both2` measured them byte-IDENTICAL —
                             at ZERO formal stores, the one arrangement where
                             one base symbol and two agree.  So the spelling is
                             crossed with NF and never held fixed.
  BODY   L(eaf) / C(all)     the leaf emitter and board #844's composition.
  USES   1 / 2 / 3           how many stores consume the one address.
  NF     0 / 1 / 2           **ARITY** — how many formal-valued stores stand
                             beside it.  The 17th live mis-emit was found
                             because every prior fixture had <= 1 word beside
                             the address, the one arrangement where the wrong
                             rule and the right rule agree.  NF is the axis that
                             makes `order`'s `u` and the producer's slot differ.
  POS    af / fa             which kind of store LEADS the run — `w-carrier`
                             §5.2's named unvaried axis, and the axis a live
                             wrong emit hid behind for 53 frozen cells.
  ROOT   self / other        is the address stored back into the object it
                             points at, or into a different one?  `other` is the
                             only way to have an address producer with ONE base
                             symbol, which separates the producer from the
                             symbol.

...plus three classes DECLARED OUT OF THE SHIPPED DOMAIN and graded anyway, so
their answers are on record rather than assumed (PREREG §4 L2/L3, PRED-3):

  ZERO   a bind at offset 0 — c2 materialises NOTHING; the value is the base
         register itself.  The shipped rule refuses it.
  TWOP   two DISTINCT interior addresses — still single-kind, so
         `alloc::allocate` answers; outside the one-producer domain.
  MIX    an address beside a literal — peer lane `w-mixkind`'s rung.  Two cells,
         for the record only; this lane changes nothing that can move them.

and six CONTROLS that must stay green under every mutation used as evidence.

    python3 work/w-midrun/gridgen.py --freeze   write the cells + GRIDM.sha256
    python3 work/w-midrun/gridgen.py --check    re-verify the manifest

**THE GENERATOR ASSERTS ITS OWN CLASSES** and writes nothing if a named class is
absent or if a rival pair would be indistinguishable.  It does NOT predict c2's
store order: the order is c2's answer and is read off the obj by the scorer.

Compiles nothing.
"""

import argparse
import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(HERE, "grid")
MANIFEST = os.path.join(HERE, "GRIDM.sha256")

# ---------------------------------------------------------------------------
# The shared declaration.
#
# `mZero` sits at offset 0 SO THAT the zero-offset class exists at all: a bind
# whose displacement is 0 is the one interior address c2 emits no instruction
# for, and a layout without a member at 0 cannot spell it.
#
# `mBlk` is four POINTER slots so one address can be consumed up to three times
# through the bind without ever reusing an offset — `scheduled_gpr_run`
# eliminates a dead store, and a cell that tripped that would grade a different
# construct.  `mP0..mP2` are the `other`-root destinations, and `mA/mB/mC` the
# formal-valued ones.
DECL = """\
struct BE { BE* n0; BE* n1; BE* n2; BE* n3; };
struct H {
    BE mZero;          // 0    n0@0  n1@4  n2@8  n3@12
    unsigned mA;       // 16
    BE mBlk;           // 20   n0@20 n1@24 n2@28 n3@32
    unsigned mB;       // 36
    unsigned mC;       // 40
    BE mAlt;           // 44   n0@44 n1@48 n2@52 n3@56
    BE* mP0;           // 60
    BE* mP1;           // 64
    BE* mP2;           // 68
    H(unsigned p, unsigned q);
    void lf(unsigned p, unsigned q);
    BE* Grab(unsigned n);
};
"""

FSLOT = ["mA", "mB", "mC"]          # formal-valued destinations, through `this`
OSLOT = ["mP0", "mP1", "mP2"]       # `other`-root destinations, through `this`
SSLOT = ["n0", "n1", "n2"]          # `self`-root destinations, through the block


def addr_stores(spell, root, uses, member="mBlk", bindname="r"):
    """The `uses` stores that consume ONE interior address."""
    out = []
    for i in range(uses):
        if root == "self":
            dst = ("%s.%s" % (bindname, SSLOT[i])) if spell == "B" \
                else ("%s.%s" % (member, SSLOT[i]))
        else:
            dst = OSLOT[i]
        val = ("&%s" % bindname) if spell == "B" else ("&%s" % member)
        out.append("    %s = %s;" % (dst, val))
    return out


def formal_stores(n):
    return ["    %s = q;" % FSLOT[i] for i in range(n)]


def body(lines, kind):
    if kind == "L":
        return "void H::lf(unsigned p, unsigned q) {\n%s\n}\n" % "\n".join(lines)
    return "H::H(unsigned p, unsigned q) {\n%s\n\n    Grab(p);\n}\n" % "\n".join(lines)


def cell(name, klass, note, lines, kind, binds=()):
    pre = ["    BE& %s = %s;" % (b, m) for b, m in binds]
    src = (
        "// GRID M cell `%s` — w-midrun, the emitter rung under xboxheap.cpp.\n"
        "// class: %s\n"
        "// %s\n"
        "// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).\n"
        "%s\n%s" % (name, klass, note, DECL, body(pre + lines, kind))
    )
    return name, klass, src


def build():
    cells = []

    # ---- CORE: root = self, the shape `xboxheap` itself has ----------------
    for spell in ("B", "D"):
        for kind in ("L", "C"):
            binds = (("r", "mBlk"),) if spell == "B" else ()
            for uses in (1, 2, 3):
                # NF = 0: no formal store, so POS is not defined and is not
                # spelled — a cell that varied a degenerate axis would inflate
                # the count without adding an arrangement.
                nm = "m_%s%s_u%d_f0" % (spell.lower(), kind.lower(), uses)
                cells.append(cell(
                    nm, "dom",
                    "root=self uses=%d formals=0 — the producer alone" % uses,
                    addr_stores(spell, "self", uses), kind, binds))
                for nf in (1, 2):
                    for pos in ("af", "fa"):
                        a = addr_stores(spell, "self", uses)
                        f = formal_stores(nf)
                        lines = (a + f) if pos == "af" else (f + a)
                        nm = "m_%s%s_u%d_f%d_%s" % (
                            spell.lower(), kind.lower(), uses, nf, pos)
                        cells.append(cell(
                            nm, "dom",
                            "root=self uses=%d formals=%d lead=%s" % (
                                uses, nf, "address" if pos == "af" else "formal"),
                            lines, kind, binds))

    # ---- CORE: root = other — one base symbol, still a producer -----------
    for spell in ("B", "D"):
        for kind in ("L", "C"):
            binds = (("r", "mBlk"),) if spell == "B" else ()
            for uses in (1, 2):
                for pos in ("af", "fa"):
                    a = addr_stores(spell, "other", uses)
                    f = formal_stores(1)
                    lines = (a + f) if pos == "af" else (f + a)
                    nm = "o_%s%s_u%d_f1_%s" % (
                        spell.lower(), kind.lower(), uses, pos)
                    cells.append(cell(
                        nm, "dom",
                        "root=other uses=%d formals=1 lead=%s — ONE base symbol"
                        % (uses, "address" if pos == "af" else "formal"),
                        lines, kind, binds))

    # ---- ZERO: the interior address that materialises NOTHING -------------
    for spell in ("B", "D"):
        for kind in ("L", "C"):
            binds = (("z", "mZero"),) if spell == "B" else ()
            lines = addr_stores(spell, "self", 1, member="mZero", bindname="z")
            lines += formal_stores(1)
            nm = "z_%s%s_u1_f1" % (spell.lower(), kind.lower())
            cells.append(cell(
                nm, "zero",
                "OUT OF DOMAIN — offset 0: c2 emits no addi, the value IS the "
                "base register.  Graded to put the answer on record.",
                lines, kind, binds))

    # ---- TWOP: two DISTINCT addresses, still single-kind -------------------
    for spell in ("B", "D"):
        for kind in ("L", "C"):
            binds = (("r", "mBlk"), ("a", "mAlt")) if spell == "B" else ()
            if spell == "B":
                lines = ["    mP0 = &r;", "    mP1 = &a;", "    mA = q;"]
            else:
                lines = ["    mP0 = &mBlk;", "    mP1 = &mAlt;", "    mA = q;"]
            nm = "t_%s%s" % (spell.lower(), kind.lower())
            cells.append(cell(
                nm, "twop",
                "OUT OF DOMAIN — two distinct interior addresses.  Single-kind, "
                "so alloc::allocate answers; outside the one-producer gate.",
                lines, kind, binds))

    # ---- MIX: peer lane w-mixkind's rung, two cells for the record --------
    for spell in ("B", "D"):
        for kind in ("L", "C"):
            binds = (("r", "mBlk"),) if spell == "B" else ()
            lines = addr_stores(spell, "self", 2) + ["    mA = 0u;", "    mB = q;"]
            nm = "x_%s%s" % (spell.lower(), kind.lower())
            cells.append(cell(
                nm, "mix",
                "OUT OF DOMAIN — an address BESIDE a literal.  This is peer lane "
                "w-mixkind's mixed-kind rung and this lane moves nothing in it.",
                lines, kind, binds))

    # ---- CONTROLS: green at both ends, under every mutation ---------------
    cells.append(cell("c_leaf_formals", "ctrl",
                      "CONTROL — a pure formal run, no producer at all.",
                      formal_stores(3), "L"))
    cells.append(cell("c_call_formals", "ctrl",
                      "CONTROL — a pure formal run before the call.",
                      formal_stores(3), "C"))
    cells.append(cell("c_lit_leaf", "ctrl",
                      "CONTROL — ONE literal producer, the shape the schedule "
                      "and the allocator were fitted on.",
                      ["    mA = 7u;", "    mB = q;", "    mC = q;"], "L"))
    cells.append(cell("c_lit_call", "ctrl",
                      "CONTROL — one literal producer before the call.",
                      ["    mA = 7u;", "    mB = q;", "    mC = q;"], "C"))
    cells.append(cell("c_base_bind", "ctrl",
                      "CONTROL — a bind used ONLY as a base.  No producer: c2 "
                      "folds bind.off into the displacement (w-carrier P0).",
                      ["    r.n0 = 0;", "    mA = q;"], "L",
                      binds=(("r", "mBlk"),)))
    cells.append(cell("c_zero_direct", "ctrl",
                      "CONTROL — the DIRECT spelling at offset 0.  `&mZero` is "
                      "`this`, so the IL carries no AddrOf and this is already "
                      "in class on the base tree.",
                      ["    mZero.n0 = &mZero;", "    mA = q;"], "L"))
    return cells


def assert_classes(cells):
    """Write NOTHING unless every named class is present and every rival pair
    is separable.  A grid that silently lost a class is `docs/GAPS.md`'s
    absence-reads-as-success, and it has cost this project four rungs."""
    names = [c[0] for c in cells]
    klass = {}
    for n, k, _ in cells:
        klass.setdefault(k, []).append(n)
    bad = []

    if len(set(names)) != len(names):
        bad.append("duplicate cell names")
    for k, lo in (("dom", 60), ("zero", 4), ("twop", 4), ("mix", 4), ("ctrl", 6)):
        if len(klass.get(k, [])) < lo:
            bad.append("class %s has %d cells, expected >= %d"
                       % (k, len(klass.get(k, [])), lo))

    # THE RIVAL PAIR.  PRED-2 is a claim about B vs D at the SAME statement
    # word; a grid missing one half of a pair cannot separate them, and
    # `w-carrier` measured "identical" on a population that had no pair with a
    # formal store in it at all.
    unpaired = []
    for n in names:
        if n[0] in "moztx" and "_" in n:
            head, rest = n.split("_", 1)
            if rest[0] in "bd":
                twin = "%s_%s%s" % (head, "d" if rest[0] == "b" else "b", rest[1:])
                if twin not in names:
                    unpaired.append(n)
    if unpaired:
        bad.append("unpaired spelling cells: %s" % ", ".join(unpaired[:6]))

    # ARITY.  The axis the 17th live mis-emit hid behind.
    for nf in (0, 1, 2):
        if not [n for n in klass["dom"] if "_f%d" % nf in n]:
            bad.append("no dom cell at formals=%d" % nf)
    for u in (1, 2, 3):
        if not [n for n in klass["dom"] if "_u%d" % u in n]:
            bad.append("no dom cell at uses=%d" % u)
    # BOTH body kinds at every use count, or the #844 composition is graded on
    # a narrower population than the leaf.
    for u in (1, 2, 3):
        for kind in ("l", "c"):
            if not [n for n in klass["dom"]
                    if "_u%d" % u in n and n.split("_")[1][1] == kind]:
                bad.append("no dom cell body=%s uses=%d" % (kind, u))
    # BOTH leads, or `w-carrier` §5.2's axis is unvaried again.
    for pos in ("af", "fa"):
        if not [n for n in klass["dom"] if n.endswith(pos)]:
            bad.append("no dom cell with lead %s" % pos)

    if bad:
        for b in bad:
            print("CLASS ASSERTION FAILED: %s" % b, file=sys.stderr)
        sys.exit(2)
    return klass


def freeze(cells):
    os.makedirs(GRID, exist_ok=True)
    rows = []
    for name, _k, src in cells:
        d = os.path.join(GRID, name)
        os.makedirs(d, exist_ok=True)
        p = os.path.join(d, name + ".cpp")
        with open(p, "w") as f:
            f.write(src)
        rows.append("%s  %s/%s.cpp" % (
            hashlib.sha256(src.encode()).hexdigest(), name, name))
    with open(MANIFEST, "w") as f:
        f.write("\n".join(sorted(rows)) + "\n")
    return rows


def check(cells):
    if not os.path.exists(MANIFEST):
        print("no manifest", file=sys.stderr)
        sys.exit(2)
    want = dict()
    for line in open(MANIFEST):
        h, p = line.split()
        want[p] = h
    bad = 0
    for name, _k, src in cells:
        key = "%s/%s.cpp" % (name, name)
        got = hashlib.sha256(src.encode()).hexdigest()
        if want.get(key) != got:
            print("DRIFT %s" % key, file=sys.stderr)
            bad += 1
    on_disk = {p for p in want}
    if len(on_disk) != len(cells):
        print("manifest has %d rows, generator makes %d cells"
              % (len(on_disk), len(cells)), file=sys.stderr)
        bad += 1
    if bad:
        sys.exit(2)
    print("GRID M: %d cells, manifest CLEAN" % len(cells))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--freeze", action="store_true")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    cells = build()
    klass = assert_classes(cells)
    if a.freeze:
        rows = freeze(cells)
        print("GRID M frozen: %d cells" % len(rows))
        for k in sorted(klass):
            print("  %-5s %3d" % (k, len(klass[k])))
    elif a.check:
        check(cells)
    else:
        for k in sorted(klass):
            print("%-5s %3d  %s" % (k, len(klass[k]), " ".join(klass[k][:4])))


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""GRID-P — the park x LITERAL composition, frozen before its first `cl.exe`.

The widening under test is board #1920: `lit_arg_tail_call`'s in-place
requirement is a statement about the TAIL site, and at the SEQUENCE site the
permutation is decided downstream by `park_in_class`.

Neither prior grid crossed this cell:

  * `w-mmio`'s 886 cells gridded the park over permutations with **no literal
    slot**;
  * `w-memcpy`'s GRID-L (747 cells) gridded the literal slot over arguments
    that were **already in place**.

So the composition — a permutation legalised by the park, in an argument list
that also carries a `li` — has no witness in either. This grid is that cell,
and every one of its members is graded by the real `c2.dll` under wibo at the
workload's own flags, `Port=Match` or `Port=Mismatch`.

AXES, frozen (this file is committed before the first cell is compiled):

  nf    number of formals                     2, 3, 4
  ng    number of guarded early returns       1, 2
  perm  the permutation of the NON-literal argument slots, over all
        permutations of `range(nf-1)` (the last slot is the literal)
  k     the literal's value                   72 (mmio's own), 5

CONTROLS, generated in the same run and scored separately:

  id    the identity permutation — in class BEFORE this lane, so it must stay
        `match` and is the neutrality witness at the cell level
  s0    the literal in SLOT 0 — `callseq-multiarg-lit`'s (c') clause, bought
        with two `Port=Mismatch`; must stay OUT of class
  two   TWO literals — clause (c); must stay OUT of class
  ng0   NO guard at all — clause (b) `callseq-multiarg-lit-unguarded`; must
        stay OUT of class

Every cell is one file with one function, so `c2rs gap` attributes a verdict
per cell rather than per grid.
"""
import itertools
import os
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "work/w-park/grid"
os.makedirs(OUT, exist_ok=True)

TY = "void *"
names = []


def emit(name, text):
    with open(os.path.join(OUT, name + ".cpp"), "w") as f:
        f.write(text)
    names.append(name)


def callee_decl(nf, nlit, slots=None):
    """The callee's parameter types, IN SLOT ORDER.

    Written in slot order rather than "pointers then ints" because the `s0`
    control puts the literal FIRST, and a declaration that assumed the tail
    position made that cell fail to compile — it read `capture-fail` on the
    grid's first run, which is a cell that graded nothing wearing the label of
    a cell that refused. Trap 5, inside a control.
    """
    if slots is None:
        args = [TY] * (nf - nlit) + ["unsigned int"] * nlit
    else:
        args = [TY if kind == "f" else "unsigned int" for kind, _ in slots]
    return f"void cal{nf}_{nlit}({', '.join(args)});"


def body(nf, ng, slots, k_by_slot):
    """slots: list of either ('f', i) or ('l', value)."""
    formals = ", ".join(f"{TY}a{i}" for i in range(nf))
    guards = "\n".join(
        f"    if (a{i} == 0) return {5 + 6 * i};" for i in range(ng)
    )
    args = ", ".join(
        f"a{v}" if kind == "f" else str(v) for kind, v in slots
    )
    nlit = sum(1 for kind, _ in slots if kind == "l")
    return (
        f"{callee_decl(nf, nlit, slots)}\n"
        f"unsigned long f({formals}) {{\n{guards}\n"
        f"    cal{nf}_{nlit}({args});\n"
        f"    return 0;\n}}\n"
    )


# ---- the grid proper ------------------------------------------------------
for nf in (2, 3, 4):
    nnl = nf - 1  # non-literal slots
    for ng in (1, 2):
        if ng > nf:
            continue
        for perm in itertools.permutations(range(nnl)):
            for k in (72, 5):
                slots = [("f", p) for p in perm] + [("l", k)]
                tag = f"g_nf{nf}_ng{ng}_p{''.join(map(str, perm))}_k{k}"
                emit(tag, body(nf, ng, slots, None))

# ---- controls -------------------------------------------------------------
# id: identity permutation, in class before this lane.
for nf in (2, 3, 4):
    slots = [("f", i) for i in range(nf - 1)] + [("l", 72)]
    emit(f"c_id_nf{nf}", body(nf, 1, slots, None))
# s0: the literal in slot 0.
for nf in (2, 3):
    slots = [("l", 72)] + [("f", i) for i in range(nf - 1)]
    emit(f"c_s0_nf{nf}", body(nf, 1, slots, None))
# two: two literals.
for nf in (3, 4):
    slots = [("f", i) for i in range(nf - 2)] + [("l", 72), ("l", 5)]
    emit(f"c_two_nf{nf}", body(nf, 1, slots, None))
# ng0: no guard.
for nf in (2, 3):
    slots = [("f", nf - 2 - i) for i in range(nf - 1)] + [("l", 72)]
    emit(f"c_ng0_nf{nf}", body(nf, 0, slots, None))

with open(os.path.join(OUT, "list.txt"), "w") as f:
    for n in names:
        f.write(f"{OUT}/{n}.cpp\n")
print(f"{len(names)} cells -> {OUT}")

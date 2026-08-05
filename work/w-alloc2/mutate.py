#!/usr/bin/env python3
"""mutate.py — score every candidate mixed-kind allocation rule against the
oracle's own bytes, including the one w-next left unshipped.

**Why this is the must-fail set for this lane.** `codegen::alloc` is on the emit
path (`leaf/store.rs:296`), but `leaf/store.rs:250-257` builds every
`alloc::Producer` with `kind: ProducerKind::Constant`, hard-coded — the parser
has no arm that yields a register-derived producer.  So no mutation of the
MIXED branch can move an obj byte through the differential, and a green gate is
not evidence about any rule below.  The grading that IS available is the one
that matters: each rule is asked what register it hands each producer, and the
answer is compared against what real `c2.dll` did.

Input is the three committed grid logs, **parsed** rather than transcribed:

    freshgrid.out   60 cells, 56 graded   (the fresh holdout)
    opgrid.out      21 cells, 21 graded   (what the bonus attaches to)
    selfgrid.out    10 cells, 10 graded   (self vs displacement)

A rule is scored `wrong` when it names an allocation and c2 disagrees, and
`refused` when it declines.  **Refused is never wrong** — that is the additive-
refusal invariant this project ships under, and R-SHIPPED below is exactly the
rule `alloc.rs` has today.

Usage:  mutate.py
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


# --------------------------------------------------------------------------
# Parse the committed logs into cells: (name, [(tag, kind, uses, self)], obs)

FRESH_ROW = re.compile(
    r"^\s+(\S+)\s+(\d+)\s+\|\s+(.+?)\s+\|\s+(.+?)\s+\|\s+(HIT|\*\*MISS\*\*)\s*$")
OP_ROW = re.compile(r"^\s+(\S+)\s+\|\s+r(\d+)\s+r(\d+)\s+\|\s+(YES|no)\s+\|")
SELF_ROW = re.compile(
    r"^\s+(\S+)\s+\|\s+(yes|no)\s+(yes|no)\s+\|\s+(YES|no)\s+\|")


def kind_of(tag):
    """Read the producer's kind off its tag — never off the answer."""
    if tag.startswith("const") or tag.startswith("wide"):
        return "const"
    return "reg"


def fresh_cells():
    """freshgrid rows carry the full observed ranking, so uses are recoverable
    from the cell name and the kinds from the producer tags."""
    out = []
    for line in open(os.path.join(HERE, "freshgrid.out")):
        m = FRESH_ROW.match(line)
        if not m:
            continue
        name, _np, obs, _pred, _v = m.groups()
        order = []
        for tok in obs.split():
            tag, reg = tok.split("=r")
            order.append((tag, int(reg)))
        uses = _uses_from_name(name, [t for t, _ in order])
        if uses is None:
            continue
        # Source order: `freshgrid` emits the CONSTANT group first except in
        # the F2 partition, which exists precisely to swap it.
        reg_first = name.startswith("F2-")
        tags = [t for t, _ in order]
        src = sorted(tags, key=lambda t: (kind_of(t) == "reg") if not reg_first
                     else (kind_of(t) != "reg"))
        prods = [(t, kind_of(t), uses[t], _self_of(name, t), src.index(t))
                 for t in tags]
        out.append((name, prods, order))
    return out


def _uses_from_name(name, tags):
    """`F1-r3k5` / `F5-rr-a2b1k4` / `F5-rc-r1j2k3` / `F4-add-r1k2` …"""
    nums = dict(re.findall(r"([rkajb])(\d+)", name.split("-", 1)[1]))
    if not nums:
        return None
    u = {}
    for t in tags:
        if t.startswith("const7"):
            k = "j" if "j" in nums else "k"
        elif t.startswith("const9"):
            k = "k"
        elif t.startswith("wide"):
            k = "k"
        elif t == "regB":
            k = "b"
        elif t == "regA" and "a" in nums:
            k = "a"
        else:
            k = "r" if "r" in nums else "a"
        if k not in nums:
            return None
        u[t] = int(nums[k])
    return u


def _self_of(name, tag):
    """Is this producer's value stored into the object it points at?

    In `freshgrid` the register-derived producers `regA`/`regB` are consumed as
    `q.aN = (int)&q` / `r.aN = (int)&r` — self-referential by construction — and
    the `F4-*` producers (`u+v`, `u+5`, `u<<3`) point at nothing.
    """
    if kind_of(tag) == "const":
        return False
    return tag in ("regA", "regB")


def op_cells():
    """opgrid rows give the pair's winner; kinds and self-ness come from the
    cell's declared partition, which is in its name."""
    SELF = {"A-fitted": True, "B-notself": False, "C-ptrarith": False,
            "D-base-r4": True, "E-r3-store-r4": False,
            "F-ptr-r4-store-r3": False, "G-add": False, "G-addi-int": False,
            "G-shift": False, "H-shift": False, "H-add": False,
            "I-fitted": True}
    out = []
    for line in open(os.path.join(HERE, "opgrid.out")):
        m = OP_ROW.match(line)
        if not m:
            continue
        name, pr, cr, _ = m.groups()
        base = re.sub(r"-\d+v\d+$", "", name)
        if base not in SELF:
            continue
        mu = re.search(r"-(\d+)v(\d+)$", name)
        if not mu:
            continue
        pu, cu = int(mu.group(1)), int(mu.group(2))
        prods = [("prod", "reg", pu, SELF[base], 1),
                 ("const", "const", cu, False, 0)]
        order = sorted([("prod", int(pr)), ("const", int(cr))],
                       key=lambda kv: -kv[1])
        out.append((name, prods, order))
    return out


def self_cells():
    out = []
    for line in open(os.path.join(HERE, "selfgrid.out")):
        m = SELF_ROW.match(line)
        if not m:
            continue
        name, is_self, _high, won = m.groups()
        # every selfgrid cell is reg 1 use vs const 1 use
        prods = [("prod", "reg", 1, is_self == "yes", 1),
                 ("const", "const", 1, False, 0)]
        order = ([("prod", 11), ("const", 10)] if won == "YES"
                 else [("const", 11), ("prod", 10)])
        out.append((name, prods, order))
    return out


# --------------------------------------------------------------------------
# The rules. Each returns [(tag, reg), ...] r11 first, or None to REFUSE.

def _hand_out(ranked):
    return [(t, 11 - i) for i, t in enumerate(ranked)]


def r_shipped(prods):
    """What `codegen::alloc` ships TODAY: a mixed run refuses outright."""
    kinds = {p[1] for p in prods}
    if len(kinds) > 1:
        return None
    return r_clause1(prods)


def r_clause1(prods):
    """Clause 1 alone: use count descending, source order on a tie."""
    return _hand_out([p[0] for p in
                      sorted(prods, key=lambda p: (-p[2], p[4]))])


def r_wnext(prods):
    """w-next's unshipped key: uses + (register-derived ? 1 : 0), tie to reg.
    Equivalently 2*uses + (reg ? 3 : 0), a strict order on a mixed pair."""
    return _hand_out([p[0] for p in
                      sorted(prods, key=lambda p: (-(2 * p[2] + (3 if p[1] == "reg" else 0)),
                                                   p[4]))])


def r_clause2(prods):
    """Clause 2 alone: register-derived before constant, then use count."""
    return _hand_out([p[0] for p in
                      sorted(prods, key=lambda p: (0 if p[1] == "reg" else 1,
                                                   -p[2], p[4]))])


def r_hself(prods):
    """This lane's candidate: the bonus is worth 1.5 uses and attaches to a
    producer whose value is stored INTO the object it points at."""
    return _hand_out([p[0] for p in
                      sorted(prods, key=lambda p: (-(2 * p[2] + (3 if p[3] else 0)),
                                                   p[4]))])


def r_hself_reversed(prods):
    """M5: the pool walked the other way. Must be red."""
    a = r_hself(prods)
    return [(t, 11 - (len(a) - 1 - i)) for i, (t, _) in enumerate(a)]


def r_hself_bonus2(prods):
    """M2: the bonus worth 2 uses instead of 1.5."""
    return _hand_out([p[0] for p in
                      sorted(prods, key=lambda p: (-(p[2] + (2 if p[3] else 0)),
                                                   p[4]))])


def r_hself_bonus0(prods):
    """M1: the bonus worth nothing — clause 1 alone."""
    return r_clause1(prods)


RULES = [
    ("R-SHIPPED   alloc.rs today (mixed refuses)", r_shipped),
    ("M2  w-next's key: uses + (register-derived ? 1 : 0)", r_wnext),
    ("M1  clause 1 alone (use count, descending)", r_clause1),
    ("M3  clause 2 alone (register-derived first)", r_clause2),
    ("C   H-self: bonus 1.5 uses, stored-into-what-it-points-at", r_hself),
    ("M4  H-self with the bonus worth 2 uses", r_hself_bonus2),
    ("M5  H-self with the pool walked ASCENDING", r_hself_reversed),
    ("M6  H-self with the bonus worth 0", r_hself_bonus0),
]


def main():
    cells = fresh_cells() + op_cells() + self_cells()
    mixed = [c for c in cells if len({p[1] for p in c[1]}) > 1]
    print("  cells parsed from the committed logs: %d "
          "(mixed-kind, the population in question: %d)" % (len(cells), len(mixed)))
    if len(mixed) < 60:
        print("  ERROR: too few cells parsed — the logs did not read. "
              "A rule scored against 0 cells would look perfect.")
        sys.exit(1)

    print("\n  %-56s %7s %7s %7s" % ("rule", "right", "WRONG", "refused"))
    print("  " + "-" * 80)
    worst = {}
    for label, fn in RULES:
        right = wrong = refused = 0
        bad = []
        for name, prods, obs in mixed:
            got = fn(prods)
            if got is None:
                refused += 1
                continue
            if sorted(got) == sorted(obs):
                right += 1
            else:
                wrong += 1
                bad.append(name)
        worst[label] = bad
        print("  %-56s %7d %7d %7d" % (label, right, wrong, refused))

    print("\n  MUST-FAIL: every M row above is a mutation of the candidate and "
          "each must be WRONG > 0.")
    ok = True
    for label, _ in RULES:
        if label.startswith("M"):
            n = len(worst[label])
            if n == 0:
                ok = False
            print("    %-56s %s (%d wrong)" % (
                label, "RED — must-fail HELD" if n else "**GREEN — MUTATION SURVIVED**", n))
    print("    %-56s %s" % (
        "R-SHIPPED refuses every mixed cell and is wrong on none",
        "OK" if not worst[RULES[0][0]] else "**BROKEN**"))
    print("\n  first disagreements, w-next's key against the oracle:")
    for n in worst["M2  w-next's key: uses + (register-derived ? 1 : 0)"][:12]:
        print("    %s" % n)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""rule.py — RULE W, the candidate the GRID S population table states, written
as a TOTAL function of things the emitter can see, plus the rivals it is scored
against.

This module is imported by `fit.py` (scoring on GRID S and on the four prior
lanes' committed logs) and by `holdout.py` (freezing predictions).  It is
committed BEFORE the holdout's sources exist, and `holdout.py --grade` reads a
frozen prediction column rather than calling into here, so a later edit of this
file cannot move a frozen prediction.

RULE W
------
Two per-producer bits and one body bit:

    A(p)  "takes the 1-use-vs-1-use tie"
          = p's value is stored INTO THE OBJECT IT POINTS AT   (H-self, #837)
            OR p's instruction is a LOAD or a SIGN/ZERO EXTENSION

    B(p)  "survives a multi-use constant"
          = p's instruction is a PowerPC ADD form (add/addi/addis/addic)
            or an ALGEBRAIC (sign-propagating) right shift (srawi/sraw)

    bases the number of distinct store-base values in the body (#865)

    the register-derived producer takes the TOP pool register  iff

        (uses(p) >= 2  or  A(p))
      and (uses(const) == 1  or  B(p))
      and (bases == 1  or  A(p)  or  B(p))

`A` is NOT a function of the mnemonic: `addi rX,r3,96` stored into the object at
96 has it and the same instruction stored into a different object does not
(w-alloc2 §4, and GRID S's `self` vs `cross` rows, which is why PREREG S2 is a
MISS).  `B` IS a function of the mnemonic, and that is the part §3 of the prereg
forbids from being a lookup: the class table below has to decide mnemonics GRID
S never contained, and `holdout.py` grades exactly that.

THE PRINCIPLE, STATED BEFORE THE HOLDOUT IS COMPILED
----------------------------------------------------
Every mnemonic below is classified by the stated principle, not by an answer.
The ones GRID S measured are marked `#S`; every other row is a PREDICTION that
`holdout.py` can falsify.  `sub`/`subf` is the row that makes the principle
non-trivial: it is arithmetic and it is NOT an add form, and GRID S measured it
outside B.
"""

# mnemonic -> (A, B).  `#S` marks a row GRID S measured; the rest are predicted
# by the principle above and are falsifiable.
CLASS = {
    # --- ADD forms: B ------------------------------------------------------
    "add":    (False, True),    # S
    "addi":   (False, True),    # S  (A comes from the self test, not from here)
    "addis":  (False, True),
    "addic":  (False, True),
    "addic.": (False, True),
    # --- ALGEBRAIC right shifts: B ----------------------------------------
    "srawi":  (False, True),    # S
    "sraw":   (False, True),
    # --- LOADS: A ----------------------------------------------------------
    "lwz":    (True, False),    # S
    "lwzx":   (True, False),
    "lhz":    (True, False),
    "lha":    (True, False),
    "lbz":    (True, False),
    "ld":     (True, False),
    # --- SIGN / ZERO EXTENSIONS: A ----------------------------------------
    "extsh":  (True, False),    # S
    "extsb":  (True, False),
    "extsw":  (True, False),
    # --- everything else: neither -----------------------------------------
    "sub":    (False, False),   # S — arithmetic, and NOT an add form
    "subf":   (False, False),
    "subfic": (False, False),
    "subfc":  (False, False),
    "and":    (False, False),   # S
    "andc":   (False, False),
    "andi.":  (False, False),
    "nand":   (False, False),
    "or":     (False, False),   # S
    "orc":    (False, False),
    "ori":    (False, False),
    "oris":   (False, False),
    "xor":    (False, False),   # S
    "xori":   (False, False),
    "xoris":  (False, False),
    "nor":    (False, False),   # S (c2 prints `not`)
    "not":    (False, False),   # S
    "eqv":    (False, False),
    "neg":    (False, False),   # S
    "slwi":   (False, False),   # S
    "srwi":   (False, False),   # S
    "rlwinm": (False, False),   # S (the un-extended spelling of both)
    "clrlwi": (False, False),
    "rotlwi": (False, False),
    "slw":    (False, False),
    "srw":    (False, False),
    "mullw":  (False, False),
    "mulli":  (False, False),
    "divw":   (False, False),
    "divwu":  (False, False),
    "cntlzw": (False, False),
    "mr":     (False, False),
    "li":     (False, False),
    "lis":    (False, False),
}


def bits(mnem, is_self):
    """(A, B, decided).  `decided` is False for a mnemonic the frozen class
    table does not contain — RULE W then REFUSES, which is not wrong but is
    also not a decision, and `fit.py`/`holdout.py` count it separately."""
    if mnem not in CLASS:
        return (False, False, False)
    a, b = CLASS[mnem]
    return (a or is_self, b, True)


def rule_w(mnem, is_self, ru, cu, bases):
    """'prod' | 'const' | None (the rule refuses)."""
    a, b, decided = bits(mnem, is_self)
    if not decided:
        return None
    win = ((ru >= 2 or a) and (cu == 1 or b) and (bases == 1 or a or b))
    return "prod" if win else "const"


# --------------------------------------------------------------------------
# The rivals.  Every one of these is on record; each is scored beside RULE W so
# that the comparison is a WRONG column against a WRONG column and not a single
# total (STATUS trap 4).
# --------------------------------------------------------------------------

def incumbent(mnem, is_self, ru, cu, bases):
    """The SHIPPED refusal in `crates/c2-core/src/codegen/alloc.rs`: a run
    mixing a constant and a register-derived producer returns `None`.  A
    refusal is never wrong.  This is the control RULE W has to beat."""
    return None


def clause1(mnem, is_self, ru, cu, bases):
    """`alloc.rs` clause 1 alone — use count, descending; a tie goes to the
    constant (clause 2 is refuted, w-alloc2 §5)."""
    return "prod" if ru > cu else "const"


def clause1_strict(mnem, is_self, ru, cu, bases):
    """w-seam GRID A's narrow lift: strictly more uses takes the top register.
    Undefined off `ru > cu`, where it falls back to the constant."""
    return "prod" if ru > cu else "const"


def wnext_key(mnem, is_self, ru, cu, bases):
    """w-next's `uses + (register-derived ? 1 : 0)`, descending."""
    return "prod" if (ru + 1) > cu else "const"


def h_self(mnem, is_self, ru, cu, bases):
    """w-alloc2's H-self: `2*uses + (3 if self else 0)`, descending."""
    return "prod" if (2 * ru + (3 if is_self else 0)) > 2 * cu else "const"


def rule_w2(mnem, is_self, ru, cu, bases):
    """RULE W2 — RULE W with its ONE refuted clause replaced by a magnitude
    that was ALREADY PUBLISHED before this lane, not fitted on the cells that
    refuted RULE W.

    `fit.py` shows RULE W wrong on **7** cells of w-alloc2's and w-next's own
    committed logs, every one of them a `self` producer at `cu >= 3`:
    `F1-r1k5`, `F1-r2k5`, `F1-r2k6`, `F1-r3k5`, `F2-off-r1k3`, `F2-off-r2k4`,
    `diff-reg1-const3`.  RULE W says a producer with the `B` bit is immune to
    the constant's use count; those cells say the advantage is BOUNDED.

    The bound is not invented here.  **w-alloc2 §4 published it**: *"The bonus
    is a MAGNITUDE, not an override: the producer wins at 1-vs-1 and 1-vs-2 and
    loses at 1-vs-3 and 1-vs-4, which is w-next's 1.5 uses confirmed on fresh
    cells."*  That is `2*ru + 3 > 2*cu`, i.e. H-self, and it is used here
    verbatim for the self case.  The standing instruction after a refutation is
    that the refuting cells are not the cells to fit a successor on; this
    successor is fitted on GRID S plus a figure that was in the record before
    the refutation existed.

        self      : H-self's magnitude,  2*ru + 3 > 2*cu
        add-form  : uses >= 2                       (B, from GRID S)
        load/ext  : the constant has exactly 1 use  (A, from GRID S)
        neither   : uses >= 2 AND const uses == 1 AND one store base

    **It is still expected to lose**, and `holdout.py` is where it can.  The
    `add-form` and `load/ext` branches are `cu`-unbounded and `ru`-unbounded
    respectively and GRID S only reached `cu = 3` and `ru = 3`.
    """
    a, b, decided = bits(mnem, is_self)
    if not decided:
        return None
    if is_self:
        win = (2 * ru + 3 > 2 * cu)
    elif b:
        win = (ru >= 2)
    elif a:
        win = (cu == 1)
    else:
        win = (ru >= 2 and cu == 1 and bases == 1)
    return "prod" if win else "const"


RIVALS = [
    ("the shipped refusal", incumbent),
    ("RULE W", rule_w),
    ("RULE W2", rule_w2),
    ("H-self", h_self),
    ("w-next's key", wnext_key),
    ("clause 1 alone", clause1),
]

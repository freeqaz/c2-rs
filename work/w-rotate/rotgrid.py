#!/usr/bin/env python3
"""rotgrid.py — **LOOP ROTATION**: does the IL's one test site become two in the
obj, and what decides the entry form?

Lane **w-rotate**. Control: `work/w-rotate/PREREG.md`, committed at `34dab3d`
**before this file existed**.

# The question, and why it is not `loopcost.py` again

`work/w-loop/loopcost.py` asked what a leaf loop costs the *label counter* and
whether that cost is observable. It counted back edges as a by-product. This
script asks the question `docs/CFG_SHAPE.md` §8.2 **L4** leaves open and names as
unstated:

> `?c_callloop` and `?d_break` guard with a compare; `?d_cont` jumps into the
> test. I can say *that* both occur and that `?d_cont` differs by having a
> `continue`; **I cannot state the rule.**

w-loop already refuted the obvious rival (*a `continue` makes the test a join
target*): a **leaf** `+continue` keeps the guarded form. So L4 is open with one
rival dead.

# The four buckets, decided from bytes and never by eye

    GUARD     a conditional branch BEFORE the loop top, branching OUT (target at
              or past the fall-out of the back edge)
    GUARDRET  the same, folded to a `bclr` form — no displacement at all
    JUMPIN    an UNCONDITIONAL forward `b` before the loop top, targeting the
              bottom test: the IL's own entry jump, surviving
    NONE      no entry test: the first thing reached is the body (`do/while`)

`NOLOOP` (no back edge) and `MULTI` (more than one loop) are their own buckets
and are **excluded from every rate, with the exclusion printed**. Absence is
never read as a bucket — this project has recorded that failure 16 times.

# What is GRADED, as opposed to produced

Two rates, both printed as `n of m` with `m` the number of cells that reached
the classifier:

* **P2** — for every rotated cell, the *predicted* guard form
  (`GUARDRET` iff the block the loop falls out to is a bare `blr`, else `GUARD`)
  against the *measured* one. This is the rule under test, and a cell where the
  prediction is absent counts as a miss, not as a skip.
* **P3** — for every rotated compare-form cell, whether the guard's `BI` equals
  the back edge's `BI` with `BO` inverted: one condition, two sites.

Usage:

    work/w-rotate/rotgrid.py                 # the whole grid at /O1 /GS- /c
    work/w-rotate/rotgrid.py --mode '/Ox /GS- /c'
    work/w-rotate/rotgrid.py --dis NAME ...  # disassemble named cells
    work/w-rotate/rotgrid.py --only NAME ...

Exit status is non-zero only when a **control** fails, never because a
prediction did.
"""

import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402
from gt_dump import disasm  # noqa: E402

# ---------------------------------------------------------------------------
# The grid.  (name, source, axis-note)
#
# Every probe defines exactly one function `P`, so the packed `.text` of a
# leaf-only TU is that function and nothing else.  Call-bearing cells declare
# `gi` and are framed by construction — the frame class is an AXIS here, not an
# accident, because P1 claims JUMPIN is a property of call-bearing bodies.
# ---------------------------------------------------------------------------
DECL = "int gi(int);\n"

GRID = [
    # --- controls -----------------------------------------------------------
    ("ctl-noloop", "int P(int a){ return a+1; }",
     "CONTROL: no loop at all. MUST classify NOLOOP"),
    ("ctl-if", "int P(int a){ if (a) return 5; return a+1; }",
     "CONTROL: a forward branch and no back edge. MUST classify NOLOOP"),

    # --- the bottom-test pole (P6) -----------------------------------------
    ("do-while", "int P(int a){ int r=0; do { r=r+a; a=a-1; } while (a); return r; }",
     "P6: bottom-test in the IL. Predicted NONE"),
    ("do-while-ret0", "int P(int a){ int r=0; do { r=r+a; a=a-1; } while (a); return 0; }",
     "P6 + a constant return: the exit block is `li r3,0 ; blr`, not bare"),
    ("goto-back", "int P(int a){ int r=0; top: r=r+a; a=a-1; if (a) goto top; return r; }",
     "a backward goto: bottom-test with no loop keyword"),
    ("forever-break", "int P(int a){ int r=0; for(;;){ r=r+a; a=a-1; if(!a) break; } return r; }",
     "for(;;) with the only exit inside the body"),

    # --- the top-test pole, LEAF (P1) --------------------------------------
    ("while-dec", "int P(int a){ int r=0; while (a) { r=r+a; a=a-1; } return r; }",
     "P1: top-test leaf, CTR-eligible"),
    ("while-ptr", "int P(const char* s){ int r=0; while (*s) { r=r+*s; s++; } return r; }",
     "P1+P4: the SENTINEL WALK -- the test reads memory"),
    ("for-count", "int P(int a){ int r=0; for (int i=0;i<a;i++) r=r+i; return r; }",
     "P1: counted for, leaf"),
    ("for-stride3", "int P(int a){ int r=0; for (int i=0;i<a;i+=3) r=r+i; return r; }",
     "stride 3"),
    ("for-down", "int P(int a){ int r=0; for (int i=a;i>0;i--) r=r+i; return r; }",
     "counting down"),
    ("for-break", "int P(int a){ int r=0; for (int i=0;i<a;i++){ r=r+i; if(r>100) break; } return r; }",
     "a second exit"),
    ("for-cont", "int P(int a){ int r=0; for (int i=0;i<a;i++){ if(i==3) continue; r=r+i; } return r; }",
     "**P1's REGISTERED RISK CELL**: a `continue` in a LEAF"),
    ("while-cont", "int P(int a){ int r=0; int i=0; while (i<a) { i++; if(i==3) continue; r=r+i; } return r; }",
     "a `continue` in a `while`, leaf"),
    ("idx-load", "int P(const int* v,int n){ int r=0; for (int i=0;i<n;i++) r=r+v[i]; return r; }",
     "Primes.cpp's shape: indexed load, leaf"),

    # --- the top-test pole, CALL-BEARING (P1 / P5) -------------------------
    ("while-call", DECL + "int P(int a){ int r=0; while (a) { r=r+gi(a); a=a-1; } return r; }",
     "P1: the SAME graph as while-dec with a call in the body"),
    ("for-call", DECL + "int P(int a){ int r=0; for (int i=0;i<a;i++) r=r+gi(i); return r; }",
     "P5: the same graph as for-count, body content changed"),
    ("for-call-cont", DECL + "int P(int a){ int r=0; for (int i=0;i<a;i++){ if(i==3) continue; r=r+gi(i); } return r; }",
     "**§3.7a's `?d_cont`**: a `continue` AND a call. The published JUMPIN cell"),
    ("while-call-cont", DECL + "int P(int a){ int r=0; int i=0; while (i<a) { i++; if(i==3) continue; r=r+gi(i); } return r; }",
     "a `continue` and a call, in a `while`"),
    ("for-call-break", DECL + "int P(int a){ int r=0; for (int i=0;i<a;i++){ r=r+gi(i); if(r>100) break; } return r; }",
     "a call and a second exit"),
    ("do-call", DECL + "int P(int a){ int r=0; do { r=r+gi(a); a=a-1; } while (a); return r; }",
     "bottom-test with a call"),

    # --- P2: the EXIT BLOCK axis, held against a fixed loop ----------------
    #
    # Four cells whose loop is byte-identical in shape and whose only difference
    # is what the loop falls out to. If P2 is right the guard form tracks this
    # column and nothing else.
    ("exit-bare", "int P(const char* s){ int r=0; while (*s) { r=r+*s; s++; } return r; }",
     "P2: exit is the accumulator -- may coalesce to r3 (bare blr)"),
    ("exit-plus1", "int P(const char* s){ int r=0; while (*s) { r=r+*s; s++; } return r+1; }",
     "P2: the exit block computes -- `addi ; blr`, NOT bare"),
    ("exit-const", "int P(const char* s){ int r=0; while (*s) { r=r+*s; s++; } return 7; }",
     "P2: the exit block is `li r3,7 ; blr`"),
    ("exit-void", "int gv(int);\nvoid P(const char* s){ while (*s) { gv(*s); s++; } }",
     "P2: nothing to return at all"),
    ("exit-second", "int P(int a){ int r=0; while (a) { r=r+a; a=a-1; } return r; }",
     "P2 on the counted family: exit is the accumulator"),
    ("exit-second-k", "int P(int a){ int r=0; while (a) { r=r+a; a=a-1; } return r*3; }",
     "P2: the same loop, a computing exit block"),

    # --- P4: rotation WITHOUT a memory-carried test, and vice versa --------
    ("ptr-idx", "int P(const char* s){ int r=0; for (int i=0; s[i]; i++) r=r+s[i]; return r; }",
     "P4: sentinel test written as an INDEX, not a walked pointer"),
    ("ptr-nosent", "int P(const char* s,int n){ int r=0; for (int i=0;i<n;i++) r=r+s[i]; return r; }",
     "P4: a load in the BODY but a register-resident TEST -- no peel predicted"),
    ("ptr-walk-two", "int P(const char* s){ int r=0; while (*s) { r=r+*s; r=r*3; s++; } return r; }",
     "P4/P8: the sentinel walk with a LONGER body"),

    # --- P7: can any back edge be unconditional? ---------------------------
    ("inf-loop", "void P(void){ for(;;) { } }",
     "P7's risk cell: a loop with NO exit at all"),
    ("inf-call", DECL + "void P(int a){ for(;;) { gi(a); } }",
     "P7: an infinite loop with a call"),
]

# ---------------------------------------------------------------------------
# Grid B (P8) — the SENTINEL WALK with the accumulate body varied and the
# signature held EXACTLY fixed.  w-hash measured that changing the *signature*
# re-plans the whole block layout; P8 asks whether changing only the *body*
# does.  This is the cell that decides whether a body-parameterized lowering is
# possible, so it is a separate table with its own controls.
# ---------------------------------------------------------------------------
GRID_B = [
    ("b-add",    "r=r+c;",              "the plainest accumulate"),
    ("b-mul3",   "r=r*3+c;",            "Sort's shape with K=3"),
    ("b-mul127", "r=r*127+c;",          "Sort's own K"),
    ("b-xor",    "r=r^c;",              "a different operator"),
    ("b-sub",    "r=r-c;",              "subtract"),
    ("b-shift",  "r=(r<<1)+c;",         "a shift in the accumulate"),
    ("b-two",    "r=r+c; r=r*3;",       "TWO statements -- a longer body"),
    ("b-three",  "r=r+c; r=r*3; r=r-1;", "THREE statements -- longer again"),
    ("b-and",    "r=r&c;",              "bitwise and"),
    ("b-cond",   "r=r+c+1;",            "an extra constant"),
]


def grid_b_src(body):
    return ("int P(const char* s){ int r=0; while (*s) { int c=*s; %s s++; } "
            "return r; }" % body)


# ---------------------------------------------------------------------------
# Grid C (H-EXIT) — the JUMPIN boundary, registered in `PREREG.md` §6 at commit
# `9d0a9df` BEFORE this list existed.
#
#   H-EXIT: c2 DUPLICATES the loop test (bucket GUARD/GUARDRET) iff the loop
#   produces a value the EXIT BLOCK consumes.  When the loop produces nothing
#   the exit uses, the test is emitted ONCE at the bottom and entered by an
#   unconditional `b` (bucket JUMPIN) — the IL's `3A Ltest` surviving.
#
# Grid A produced two JUMPIN cells out of 42.  A rule fitted to two cells is
# what this project forbids, so every cell below carries its PREDICTION in the
# table and the script grades predicted-vs-measured as `n of m`.  A cell that
# collapses to NOLOOP (c2 deleted a dead loop) is excluded and the exclusion is
# printed — it is not scored as a hit.
#
# `ROT` means GUARD or GUARDRET; H-EXIT does not predict which of the two, and
# it does not need to — that is P2's job, and the two rules are independent.
# ---------------------------------------------------------------------------
GRID_C = [
    ("c-empty-const", "int P(const char* s){ while (*s) s++; return 7; }",
     "JUMPIN", "empty body, constant return: the loop produces NOTHING"),
    ("c-empty-ptr", "const char* P(const char* s){ while (*s) s++; return s; }",
     "ROT", "empty body but the exit returns the WALKED POINTER"),
    ("c-strlen", "int P(const char* s){ int n=0; while (*s) { n++; s++; } return n; }",
     "ROT", "the counter is returned"),
    ("c-count-const", "int P(const char* s){ int n=0; while (*s) { n++; s++; } return 7; }",
     "JUMPIN", "the same loop, counter DEAD"),
    ("c-dead-acc", "int P(const char* s){ int r=0; while (*s) { r=r+*s; s++; } return 7; }",
     "JUMPIN", "accumulate over the test operand, result dead"),
    ("c-call-void", "int gv(int);\nvoid P(const char* s){ while (*s) { gv(*s); s++; } }",
     "JUMPIN", "a call for effect; nothing returned"),
    ("c-call-ret", "int gi(int);\nint P(const char* s){ int r=0; while (*s) { r=r+gi(*s); s++; } return r; }",
     "ROT", "the same call, accumulated and RETURNED"),
    ("c-call-dead", "int gi(int);\nint P(const char* s){ int r=0; while (*s) { r=r+gi(*s); s++; } return 7; }",
     "JUMPIN", "the same call, accumulator DEAD -- the discriminating pair with c-call-ret"),
    ("c-store", "void P(const char* s,int* o){ while (*s) { *o=*o+*s; s++; } }",
     "JUMPIN", "the loop's product goes to MEMORY, not to the exit block"),
    ("c-global", "int gS;\nvoid P(const char* s){ while (*s) { gS=gS+*s; s++; } }",
     "JUMPIN", "same, through a global"),
    ("c-diff", "int P(const char* s){ const char* p=s; while (*p) p++; return (int)(p-s); }",
     "ROT", "empty body, exit consumes the walked pointer by difference"),
    ("c-ifbody", "int P(const char* s){ int r=0; while (*s) { if (*s>97) r=r+*s; s++; } return r; }",
     "ROT", "a branch INSIDE the body"),
    ("c-break", "int P(const char* s){ int r=0; while (*s) { if (*s==120) break; r=r+*s; s++; } return r; }",
     "ROT", "a second exit"),
    ("c-cnt-live", "int P(int a){ int r=0; while (a) { r=r+a; a=a-1; } return r; }",
     "ROT", "the NON-sentinel family: counted, accumulator live"),
    ("c-cnt-dead", "int P(int a){ int r=0; while (a) { r=r+a; a=a-1; } return 7; }",
     "JUMPIN", "counted, accumulator dead (may collapse to NOLOOP -- then excluded)"),
    ("c-two-out", "int P(const char* s,int* o){ int r=0; while (*s) { r=r+*s; s++; } *o=r; return r; }",
     "ROT", "the exit block both stores and returns the accumulator"),
    ("c-void-acc", "void P(const char* s,int* o){ int r=0; while (*s) { r=r+*s; s++; } *o=r; }",
     "ROT", "VOID function, but the exit block still consumes the accumulator"),
    ("c-call-count", "int gi(int);\nvoid P(int n){ for (int i=0;i<n;i++) gi(i); }",
     "JUMPIN", "a counted call loop producing nothing"),
]


# ---------------------------------------------------------------------------
# The decoder.  Only the four branch forms `CFG_SHAPE.md` §3.1 records, plus the
# two CTR opcodes, are needed; everything else is passed to llvm-mc for display
# only and never for classification.
# ---------------------------------------------------------------------------
BLR = 0x4E800020


def sext(v, bits):
    return v - (1 << bits) if v & (1 << (bits - 1)) else v


def branch(w):
    """(kind, target_delta, BO, BI) for a branch word, else None.

    kind in {'bc','b','bclr'}.  `target_delta` is the signed byte displacement
    from this instruction for 'bc'/'b', and None for 'bclr' (which has no
    displacement -- that absence is the whole of the GUARDRET signature).
    """
    op = w >> 26
    if op == 16:                                   # bc
        return ("bc", sext(w & 0xFFFC, 16), (w >> 21) & 0x1F, (w >> 16) & 0x1F)
    if op == 18:                                   # b
        return ("b", sext(w & 0x03FFFFFC, 26), None, None)
    if op == 19 and ((w >> 1) & 0x3FF) == 16:      # bclr
        return ("bclr", None, (w >> 21) & 0x1F, (w >> 16) & 0x1F)
    return None


def is_cond_bo(bo):
    """A `BO` that actually tests a CR bit (4/5/6/7 = false, 12/13/14/15 =
    true).  `BO=20` is the unconditional form (`blr`), `BO=16` is the CTR
    decrement (`bdnz`) and tests no CR bit at all."""
    return (bo & 0x14) in (0x04, 0x0C)


def text_words(o):
    """(words, reloc_offsets) for the single `.text` of a leaf-only TU, or
    None when the obj does not have exactly one."""
    idx = [i for i, s in enumerate(o.sections) if s["name"] == ".text"]
    if len(idx) != 1:
        return None
    s = o.sections[idx[0]]
    raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
    rel = set(int.from_bytes(o.d[s["relptr"] + 10 * r:s["relptr"] + 10 * r + 4],
                             "little")
              for r in range(s["nrel"]))
    words = [int.from_bytes(raw[i:i + 4], "big") for i in range(0, len(raw) - 3, 4)]
    return words, rel


def classify(words, rel):
    """The whole classification, from bytes.  Returns a dict; `bucket` is one of
    GUARD / GUARDRET / JUMPIN / NONE / NOLOOP / MULTI."""
    r = {"nwords": len(words), "back": [], "bucket": None, "peel": None,
         "exit_bare": None, "pred": None, "ctr": 0, "bdnz": 0}
    for off4, w in enumerate(words):
        off = off4 * 4
        b = branch(w)
        if (w & 0xFC1FFFFF) == 0x7C0903A6:
            r["ctr"] += 1
        if b and b[0] == "bc" and b[2] == 16:
            r["bdnz"] += 1
        if b and b[0] in ("bc", "b") and b[1] < 0 and off not in rel:
            r["back"].append((off, w, b))
        # A branch to ITSELF is a back edge with displacement ZERO, and `< 0`
        # does not see it.  `for(;;){}` emits exactly one word, `48000000`, and
        # the first version of this classifier called that NOLOOP -- reading an
        # UNCONDITIONAL back edge as the absence of a loop.  That is this
        # project's most-repeated defect (absence read as success) committed by
        # the instrument built to avoid it, so it gets its own bucket rather
        # than a widened comparison: a self-loop is not a rotation question and
        # must not be averaged into one.
        if b and b[0] == "b" and b[1] == 0 and off not in rel:
            r["selfloop"] = off
    if r.get("selfloop") is not None and not r["back"]:
        r["bucket"] = "SELFLOOP"
        return r
    if not r["back"]:
        r["bucket"] = "NOLOOP"
        return r
    # Distinct loops = distinct back-edge TARGETS.  A `continue` can add a
    # second back edge to the SAME top, which is one loop, not two.
    tops = sorted(set(off + b[1] for off, _, b in r["back"]))
    if len(tops) > 1:
        r["bucket"] = "MULTI"
        r["tops"] = tops
        return r
    top = tops[0]
    r["top"] = top
    last_back = max(off for off, _, _ in r["back"])
    fallout = last_back + 4
    r["fallout"] = fallout
    r["backword"] = next(b for off, _, b in r["back"] if off == last_back)

    # The exit block: everything the loop falls out to.  "Bare" means the single
    # word `blr` -- the P2 predictor.
    tail = words[fallout // 4:]
    r["exit_bare"] = (tail == [BLR])
    r["pred"] = "GUARDRET" if r["exit_bare"] else "GUARD"

    # The entry region is everything before the loop top.
    guards = []
    for off4, w in enumerate(words[:top // 4]):
        off = off4 * 4
        b = branch(w)
        if not b:
            continue
        if b[0] == "bclr" and is_cond_bo(b[2]):
            guards.append(("GUARDRET", off, b))
        elif b[0] == "bc" and b[1] > 0 and is_cond_bo(b[2]) \
                and off + b[1] >= fallout and off not in rel:
            guards.append(("GUARD", off, b))
        elif b[0] == "b" and b[1] > 0 and off not in rel \
                and off + b[1] >= top:
            guards.append(("JUMPIN", off, b))
    r["guards"] = guards
    r["bucket"] = guards[0][0] if guards else "NONE"
    if guards:
        r["guard"] = guards[0]

    # P4: is there a LOAD before the loop top?  The peel signature.  Byte/half/
    # word loads, D-form and update-form (opcodes 32..43 cover lwz..sthu).
    r["peel"] = any(32 <= (w >> 26) <= 43 for w in words[:top // 4])

    # ---- H-REG (see `PREREG.md` §7) ------------------------------------
    #
    # The PEEL is the function's FIRST load — in every cell of every grid it is
    # the first instruction after the prologue, and requiring "first load"
    # rather than "load nearest the top" is deliberate: `c-global` hoists a
    # second, unrelated load (`lwz` of the global) into the preheader, and a
    # rule that took the nearest one would read that instead.
    #
    # The CARRY is the first UPDATE-FORM load inside the loop — the induction
    # load, the instruction that advances the walked pointer.
    r["peel_rd"] = r["carry_rd"] = None
    for w in words[:top // 4]:
        if 32 <= (w >> 26) <= 43 and (w >> 26) not in (36, 37, 38, 39, 44, 45):
            r["peel_rd"] = (w >> 21) & 0x1F
            break
    for w in words[top // 4:fallout // 4]:
        if (w >> 26) in (33, 35, 41, 43):        # lwzu lbzu lhzu lhau
            r["carry_rd"] = (w >> 21) & 0x1F
            break
    if r["peel_rd"] is not None and r["carry_rd"] is not None:
        r["hreg"] = "JUMPIN" if r["peel_rd"] == r["carry_rd"] else "ROT"
    else:
        r["hreg"] = None

    # ---- H-SUF (see `PREREG.md` §8) ------------------------------------
    #
    # What produces the CR bit the BACK EDGE reads: an explicit compare, or the
    # record form of an instruction the body needed anyway.  The last writer of
    # that CR field before the back edge is the producer -- "last writer", not
    # "nearest compare", because board #644's warning is exactly that a producer
    # is not one contiguous instruction and a positional read finds the wrong
    # one.  `c-break` has an unrelated `cmpwi cr6` at its loop top and a record
    # form on cr0 feeding its back edge; a nearest-compare rule reads the break's
    # compare and gets the cell backwards.
    r["producer"] = None
    bb = r["backword"]
    # An UNCONDITIONAL back edge (`b`) has no `BO`/`BI` at all, so it has no CR
    # producer to find.  Asking anyway is how the first run of this block died.
    if bb[0] == "bc" and is_cond_bo(bb[2]):
        field = bb[3] >> 2
        for w in words[top // 4:(last_back) // 4]:
            op = w >> 26
            if op in (10, 11) and ((w >> 23) & 7) == field:
                r["producer"] = "cmp"
            elif op == 31 and ((w >> 1) & 0x3FF) in (0, 32) and ((w >> 23) & 7) == field:
                r["producer"] = "cmp"
            elif op == 13 and field == 0:
                r["producer"] = "record"
            elif op == 31 and (w & 1) and field == 0:
                r["producer"] = "record"
    if r["hreg"] is not None and r["producer"] is not None:
        r["hsuf"] = "JUMPIN" if (r["peel_rd"] == r["carry_rd"]
                                 or r["producer"] == "cmp") else "ROT"
    else:
        r["hsuf"] = None
    return r


def p3_same_condition(r):
    """P3: the guard tests the same CR bit as the back edge, sense inverted.

    Only askable when both sites test a CR bit -- a `bdnz` back edge tests CTR
    and has no `BI`, which is itself the finding P3 has to be restricted by."""
    if r["bucket"] not in ("GUARD", "GUARDRET"):
        return None
    gb = r["guard"][2]
    bb = r["backword"]
    if not is_cond_bo(bb[2]) or not is_cond_bo(gb[2]):
        return None                      # CTR back edge: not comparable
    return gb[3] == bb[3] and ((gb[2] ^ bb[2]) & 0x08) != 0


def run(cells, mode, wd, tag_prefix, show):
    print("%-16s %9s %6s %5s %5s %5s %7s  %s"
          % ("cell", "bucket", "pred", "P2", "P3", "peel", "words", "note"))
    reached = graded = 0
    p2_hit = p2_n = 0
    p3_hit = p3_n = 0
    rows = {}
    for name, src, note in cells:
        o = G.capture(src if src.endswith("\n") else src + "\n", mode, wd,
                      (tag_prefix + name).replace("-", "_"))
        if o is None:
            print("%-16s  CAPTURE FAILED" % name)
            continue
        reached += 1
        tw = text_words(o)
        if tw is None:
            print("%-16s  NOT ONE .text (%d) -- excluded"
                  % (name, sum(1 for s in o.sections if s["name"] == ".text")))
            continue
        r = classify(*tw)
        r["obj"] = o
        r["words"] = tw[0]
        rows[name] = r
        graded += 1
        p2 = p3 = "-"
        if r["bucket"] in ("GUARD", "GUARDRET"):
            p2_n += 1
            ok = (r["bucket"] == r["pred"])
            p2_hit += ok
            p2 = "OK" if ok else "MISS"
            v = p3_same_condition(r)
            if v is not None:
                p3_n += 1
                p3_hit += v
                p3 = "OK" if v else "MISS"
            else:
                p3 = "n/a"
        print("%-16s %9s %6s %5s %5s %5s %7d  %s"
              % (name, r["bucket"], r["pred"] or "-", p2, p3,
                 "yes" if r["peel"] else ("no" if r["peel"] is not None else "-"),
                 r["nwords"], note))
    print()
    print("  reached %d  graded %d  (a cell that did not capture is a FAILURE,"
          " not a zero)" % (reached, graded))
    buckets = {}
    for r in rows.values():
        buckets[r["bucket"]] = buckets.get(r["bucket"], 0) + 1
    print("  buckets: " + "  ".join("%s=%d" % kv for kv in sorted(buckets.items())))
    print("  P2 (guard form == exit-block predictor): %d of %d rotated cells"
          % (p2_hit, p2_n))
    print("  P3 (one condition, two sites):           %d of %d comparable cells"
          % (p3_hit, p3_n))
    if show:
        for name in show:
            if name in rows:
                print()
                dump(name, rows[name])
    return rows, reached, graded


# ---------------------------------------------------------------------------
# Grid D — HELD OUT.  Fresh cells written after H-REG was read off five named
# cells (`b-add`, `exit-const`, `exit-void`, `c-strlen`, `c-store`), so that the
# rule is graded on a population it was NOT fitted to.  Reporting a fitted rule's
# accuracy on its own fitting set is how a placement rule survives to become the
# eleventh refuted one.
# ---------------------------------------------------------------------------
GRID_D = [
    ("d-or", "int P(const char* s){ int r=0; while (*s) { r=r|*s; s++; } return r; }",
     "one accumulate op, a different operator"),
    ("d-max", "int P(const char* s){ int r=0; while (*s) { if (*s>r) r=*s; s++; } return r; }",
     "a conditional accumulate"),
    ("d-last", "int P(const char* s){ int r=0; while (*s) { r=*s; s++; } return r; }",
     "the body OVERWRITES rather than accumulates"),
    ("d-idx-cnt", "int P(const char* s){ int n=0; while (*s) { if (*s==97) n++; s++; } return n; }",
     "the body reads the char but accumulates a counter"),
    ("d-two-ptr", "int P(const char* a,const char* b){ int r=0; while (*a) { r=r+*a+*b; a++; b++; } return r; }",
     "TWO walked pointers"),
    ("d-wide", "int P(const short* s){ int r=0; while (*s) { r=r+*s; s++; } return r; }",
     "a 16-bit element -- lhzu, not lbzu"),
    ("d-word", "int P(const int* s){ int r=0; while (*s) { r=r+*s; s++; } return r; }",
     "a 32-bit element -- lwzu"),
    ("d-store-ret", "int P(const char* s,int* o){ int r=0; while (*s) { *o=*s; s++; r=r+1; } return r; }",
     "a store in the body AND a returned counter"),
    ("d-nested-acc", "int P(const char* s){ int r=0; while (*s) { r=r+*s*3+1; s++; } return r; }",
     "a deeper accumulate expression"),
    ("d-pre-inc", "int P(const char* s){ int r=0; while (*++s) { r=r+*s; } return r; }",
     "the increment INSIDE the test"),
    ("d-cmp-k", "int P(const char* s){ int r=0; while (*s!=120) { r=r+*s; s++; } return r; }",
     "a sentinel that is not zero"),
    ("d-void-store", "void P(const char* s,int* o){ while (*s) { *o=*o+1; s++; } }",
     "a store in the body, nothing returned, body ignores the char"),
]


# ---------------------------------------------------------------------------
# Grid E — HELD OUT from H-SUF's fitting set (`d-cmp-k` alone).  Every cell is a
# SENTINEL WALK, which is the family H-SUF is scoped to; the counted family is
# deliberately absent because `for-break` already shows the compare half of the
# rule does not hold there (its entry test is a CONSTANT-FOLDED specialization
# of the loop test, not a copy of it).
# ---------------------------------------------------------------------------
GRID_E = [
    ("e-sent-ne", "int P(const char* s){ int r=0; while (*s!=65) { r=r+*s; s++; } return r; }",
     "a non-zero sentinel, accumulating"),
    ("e-sent-gt", "int P(const char* s){ int r=0; while (*s>32) { r=r+*s; s++; } return r; }",
     "a relational sentinel"),
    ("e-sent-ne-cnt", "int P(const char* s){ int n=0; while (*s!=65) { n++; s++; } return n; }",
     "a non-zero sentinel, body ignores the char"),
    ("e-sent-ne-store", "void P(const char* s,int* o){ while (*s!=65) { *o=*s; s++; } }",
     "a non-zero sentinel, stores"),
    ("e-zero-mul", "int P(const char* s){ int r=0; while (*s) { r=r*5+*s; s++; } return r; }",
     "the zero sentinel, a deeper accumulate"),
    ("e-zero-two", "int P(const char* s){ int r=0; while (*s) { r=r+*s; r=r^2; s++; } return r; }",
     "the zero sentinel, two statements"),
    ("e-uns", "int P(const unsigned char* s){ int r=0; while (*s) { r=r+*s; s++; } return r; }",
     "UNSIGNED element -- Sort.cpp's own cast, no sign extension needed"),
    ("e-uns-ne", "int P(const unsigned char* s){ int r=0; while (*s!=65) { r=r+*s; s++; } return r; }",
     "unsigned AND a non-zero sentinel"),
]


def grade_named(rows, key, label, fitted=()):
    """Grade a mechanical predictor stored under `key`.  `n of m`, exclusions
    printed, and never a status."""
    hit = n = excl = 0
    lines = []
    for name, r in sorted(rows.items()):
        if r["bucket"] in ("NOLOOP", "MULTI", "SELFLOOP") or r.get(key) is None:
            excl += 1
            continue
        if name.split(":", 1)[-1] in fitted:
            continue
        got = "JUMPIN" if r["bucket"] == "JUMPIN" else "ROT"
        n += 1
        ok = (got == r[key])
        hit += ok
        lines.append("    %-16s peel=r%-2s carry=r%-2s prod=%-6s  %s=%-6s got=%-6s %s"
                     % (name, r["peel_rd"], r["carry_rd"], r["producer"],
                        key.upper(), r[key], got, "OK" if ok else "**MISS**"))
    print("  %s on %s: %d of %d graded cells (excluded %d)"
          % (key.upper(), label, hit, n, excl))
    for l in lines:
        print(l)
    return hit, n


def grade_hreg(rows, label):
    """H-REG, graded.  `n of m` with the excluded count printed."""
    hit = n = excl = 0
    lines = []
    for name, r in sorted(rows.items()):
        if r["bucket"] in ("NOLOOP", "MULTI", "SELFLOOP") or r.get("hreg") is None:
            excl += 1
            continue
        got = "JUMPIN" if r["bucket"] == "JUMPIN" else "ROT"
        n += 1
        ok = (got == r["hreg"])
        hit += ok
        lines.append("    %-16s peel=r%-2d carry=r%-2d  H-REG=%-6s  got=%-6s %s"
                     % (name, r["peel_rd"], r["carry_rd"], r["hreg"], got,
                        "OK" if ok else "**MISS**"))
    print("  H-REG on %s: %d of %d graded cells (excluded %d -- no peel/carry pair)"
          % (label, hit, n, excl))
    for l in lines:
        print(l)
    return hit, n


def dump(name, r):
    print("== %s ==  bucket=%s pred=%s top=%s fallout=%s"
          % (name, r["bucket"], r["pred"], r.get("top"), r.get("fallout")))
    lines = disasm(r["words"])
    for i, (w, ln) in enumerate(zip(r["words"], lines)):
        mark = ""
        if r.get("top") == i * 4:
            mark = "  <-- LOOP TOP"
        if r.get("fallout") == i * 4:
            mark = "  <-- FALLOUT"
        for g in r.get("guards", []):
            if g[1] == i * 4:
                mark = "  <== %s" % g[0]
        print("  %04x  %08x  %-34s%s" % (i * 4, w, ln, mark))


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    show = []
    if "--dis" in argv:
        i = argv.index("--dis")
        show = [a for a in argv[i + 1:] if not a.startswith("--")]
    only = []
    if "--only" in argv:
        i = argv.index("--only")
        only = [a for a in argv[i + 1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="wrot")
    print("mode: %s   workdir: %s" % (mode, wd))
    print()
    bad = 0

    cells = [c for c in GRID if not only or c[0] in only]
    print("== GRID A -- the entry form (L4) ==")
    rows, reached, graded = run(cells, mode, wd, "a_", show)
    if not only:
        # CONTROLS.  Each is a statement about the INSTRUMENT, not a prediction.
        for ctl in ("ctl-noloop", "ctl-if"):
            if rows.get(ctl, {}).get("bucket") != "NOLOOP":
                print("  !! CONTROL FAILED: %s classified %s, not NOLOOP"
                      % (ctl, rows.get(ctl, {}).get("bucket")))
                bad += 1
        if graded < len(cells) - 1:
            print("  !! CONTROL FAILED: only %d of %d cells graded"
                  % (graded, len(cells)))
            bad += 1
        rot = sum(1 for r in rows.values() if r["bucket"] in ("GUARD", "GUARDRET"))
        if rot == 0:
            print("  !! CONTROL FAILED: no cell rotated at all -- the classifier"
                  " cannot tell rotation from its absence")
            bad += 1

    print()
    print("== GRID B -- the sentinel walk with the BODY varied (P8) ==")
    cellsb = [(n, grid_b_src(b), note) for n, b, note in GRID_B]
    cellsb = [c for c in cellsb if not only or c[0] in only]
    rowsb, reachedb, gradedb = run(cellsb, mode, wd, "b_", show)
    # P8 is read off the register plan, which is the WORD STREAM outside the
    # body.  Printed as the entry prologue and the exit tail per cell so the
    # comparison is by bytes rather than by claim.
    print()
    print("  P8 -- the plan outside the body, per cell:")
    print("  %-10s %-7s %-30s %s" % ("cell", "words", "entry (to the loop top)", "tail"))
    plans = {}
    for name, r in rowsb.items():
        if r["bucket"] == "NOLOOP":
            print("  %-10s  NOLOOP" % name)
            continue
        entry = " ".join("%08x" % w for w in r["words"][:r["top"] // 4])
        tail = " ".join("%08x" % w for w in r["words"][r["fallout"] // 4:])
        plans.setdefault((entry, tail), []).append(name)
        print("  %-10s %-7d %-30s %s" % (name, r["nwords"], entry[:30], tail))
    print("  distinct (entry, tail) plans over %d cells: %d"
          % (len(rowsb), len(plans)))
    for k, v in plans.items():
        print("    %s" % " ".join(v))

    print()
    print("== GRID C -- the JUMPIN boundary (H-EXIT), predictions REGISTERED ==")
    print("   H-EXIT: the test is DUPLICATED iff the loop produces a value the")
    print("   exit block consumes. `ROT` = GUARD or GUARDRET; which of the two")
    print("   is P2's question and not H-EXIT's.")
    print()
    cellsc = [(n, s, note) for n, s, _p, note in GRID_C if not only or n in only]
    predc = {n: p for n, _s, p, _note in GRID_C}
    rowsc, reachedc, gradedc = run(cellsc, mode, wd, "c_", show)
    print()
    print("  H-EXIT graded, per cell:")
    print("  %-16s %-9s %-9s %s" % ("cell", "predicted", "measured", ""))
    hit = n = excl = 0
    for name, r in rowsc.items():
        if r["bucket"] in ("NOLOOP", "MULTI", "SELFLOOP"):
            excl += 1
            print("  %-16s %-9s %-9s  EXCLUDED (c2 emitted no single loop)"
                  % (name, predc[name], r["bucket"]))
            continue
        got = "JUMPIN" if r["bucket"] == "JUMPIN" else \
              ("ROT" if r["bucket"] in ("GUARD", "GUARDRET") else r["bucket"])
        n += 1
        ok = (got == predc[name])
        hit += ok
        print("  %-16s %-9s %-9s  %s" % (name, predc[name], got,
                                         "OK" if ok else "**MISS**"))
    print()
    print("  H-EXIT: %d of %d graded cells (excluded %d, reached %d, graded %d)"
          % (hit, n, excl, reachedc, gradedc))
    jump = sum(1 for r in rowsc.values() if r["bucket"] == "JUMPIN")
    rot = sum(1 for r in rowsc.values() if r["bucket"] in ("GUARD", "GUARDRET"))
    print("  both poles present in Grid C: JUMPIN=%d ROT=%d" % (jump, rot))
    if not only and (jump == 0 or rot == 0):
        print("  !! CONTROL FAILED: Grid C has only one pole, so it cannot")
        print("     discriminate H-EXIT from a constant answer.")
        bad += 1

    print()
    print("== GRID D -- HELD OUT from H-REG's fitting set ==")
    cellsd = [c for c in GRID_D if not only or c[0] in only]
    rowsd, reachedd, gradedd = run(cellsd, mode, wd, "d_", show)

    print()
    print("== GRID E -- HELD OUT from H-SUF's fitting set (sentinel walks only) ==")
    cellse = [c for c in GRID_E if not only or c[0] in only]
    rowse, reachede, gradede = run(cellse, mode, wd, "e_", show)

    print()
    print("== H-SUF, the SUFFIX-SHARING rule, scoped to the sentinel walk ==")
    print("   H-SUF: JUMPIN iff the entry's test block and the back edge's test")
    print("   block share a non-empty SUFFIX -- either because the peel and the")
    print("   induction load write the same register (the whole block is shared)")
    print("   or because the back edge's CR bit comes from an explicit COMPARE,")
    print("   which is identical for both and shareable even when the value")
    print("   computation ahead of it is not. Fitted on `d-cmp-k` alone.")
    print()
    h3, n3 = grade_named(rowse, "hsuf", "GRID E (held out)")

    print()
    print("== H-REG, graded on every grid ==")
    print("   H-REG: JUMPIN iff the PEELED load and the loop's UPDATE-FORM load")
    print("   write the SAME register -- i.e. the test block is identical for the")
    print("   entry and the back edge and can be SHARED. Different registers mean")
    print("   the block cannot be shared and the test is DUPLICATED (rotation).")
    print()
    fitted = ("b-add", "exit-const", "exit-void", "c-strlen", "c-store")
    print("   FITTED ON (excluded from the held-out number): %s" % " ".join(fitted))
    print()
    allrows = {}
    for pre, rr in (("A:", rows), ("B:", rowsb), ("C:", rowsc), ("D:", rowsd)):
        for k, v in rr.items():
            allrows[pre + k] = v
    h1, n1 = grade_hreg({k: v for k, v in allrows.items()
                         if k.split(":", 1)[1] not in fitted}, "the HELD-OUT set")
    print()
    h2, n2 = grade_hreg(allrows, "every cell (fitting set included)")
    if not only:
        jp = sum(1 for v in allrows.values() if v.get("hreg") == "JUMPIN")
        rp = sum(1 for v in allrows.values() if v.get("hreg") == "ROT")
        print()
        print("  both poles PREDICTED: JUMPIN=%d ROT=%d" % (jp, rp))
        if jp == 0 or rp == 0:
            print("  !! CONTROL FAILED: H-REG predicts one value everywhere, so its")
            print("     accuracy is a base rate and not a reading.")
            bad += 1

    print()
    print("controls failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

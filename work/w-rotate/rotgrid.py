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
    print("controls failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

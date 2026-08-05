#!/usr/bin/env python3
"""crossgrade.py — the div/mod leaf class, graded cell by cell by REAL c2, at
every mode the port claims and one it does not.

Lane **w-divmod**. Control: `work/w-divmod/PREREG.md`, committed at `465f36b`.

The emitter has **no free field** — four constant bodies per optimization mode
— so the axis it can be wrong on is not a parameter, it is the **class
boundary**: which bodies it takes and which it hands back. This grid grades both
directions, and it grades them at **five flag sets**, because the shipped
emitter carries two mode tables and a mode table is exactly the kind of thing
that is right on the mode it was read at and wrong one flag over.

  * every ACCEPT cell must come back `match` — byte-exact obj against real
    `c2.dll` under wibo, TimeDateStamp zeroed;
  * every REFUSE cell must come back `vocab-gap` or `codegen-gap`, never
    `mismatch`. A refusal that turns out to emit is the alarm, not a gap.

The `/Od` row is a **fail-closed boundary lane**, in `lanes.txt`'s own sense:
the port models no unoptimized body, so *every* cell there — accept and refuse
alike — must refuse. `mismatch 0` is the whole content of that row.

Run:  work/w-divmod/crossgrade.py [--jobs N]
Exit non-zero on ANY mismatch, on any accept cell that did not match at a mode
the port claims, and on any refuse cell that did.
"""
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
C2RS = os.path.join(REPO, "target", "release", "c2rs")

# The flag sets. `claims` says whether the port has a mode table for it, which
# is what turns an accept cell into "must match" or "must refuse".
MODES = [
    ("O1",      ["/O1", "/GS-", "/c"],                 True),
    ("O1-Oi-EHsc-GR", ["/O1", "/Oi", "/EHsc", "/GR", "/GS-", "/c"], True),
    ("Ox",      ["/Ox", "/GS-", "/c"],                 True),
    ("O2",      ["/O2", "/GS-", "/c"],                 True),
    ("Od",      ["/Od", "/GS-", "/c"],                 False),
]

# ---- ACCEPT: the whole shipped class, which is four cells -----------------
ACCEPT = {
    "smod": "int P(int a,int b){ return a%b; }",
    "sdiv": "int P(int a,int b){ return a/b; }",
    "umod": "unsigned P(unsigned a,unsigned b){ return a%b; }",
    "udiv": "unsigned P(unsigned a,unsigned b){ return a/b; }",
    # **Registered as a must-REFUSE cell and the oracle said otherwise.** A
    # `typedef` of `int` is transparent all the way down: the mangled name is
    # `?P@@YAHHH@Z`, byte for byte the plain-`int` one, and the IL carries the
    # bare `86 41 74` triple with no per-TU type id and no conversion. So this
    # is the same IL and the same bytes, and accepting it is not a widening --
    # it is the same cell spelled differently. Kept here, in the accept set,
    # with that evidence, rather than relaxed away.
    #
    # The separating control is `constp` below: `const int` parameters DO mint a
    # per-TU type id (`a6 41 80 20`) and an extra `2c` conversion, and are
    # refused. Two spellings that look equally cosmetic; one is and one is not.
    "typedef": "typedef int I; I P(I a,I b){ return a%b; }",
}

# ---- REFUSE, each with the body real c2 emits instead ---------------------
REFUSE = {
    # operand identity and arity
    "swap":     ("int P(int a,int b){ return b%a; }",
                 "dividend in slot 1: the spine reads r3 by name"),
    "repeat":   ("int P(int a,int b){ return a%a; }",
                 "a repeated leaf licenses the algebraic rewriter"),
    "three":    ("int P(int a,int b,int c){ return a%b; }",
                 "a third formal"),
    "one":      ("int P(int a){ return a%a; }", "one formal"),
    # computed operands -- the OTHER twi-placement regime
    "dvd-comp": ("int P(int a,int b){ return (a+1)%b; }",
                 "twi 6 HOISTS to the block's second slot"),
    "dvs-comp": ("int P(int a,int b){ return a%(b+1); }",
                 "in-spine but a different register plan"),
    "both-comp": ("int P(int a,int b){ return (a+1)%(b+1); }", "both"),
    "dvd-comp-div": ("int P(int a,int b){ return (a+1)/b; }", "the same for /"),
    # literal operands -- seven distinct bodies, none of them this one
    "lit-7":    ("int P(int a){ return a%7; }", "addi ; divw ; mulli ; subf"),
    "lit-2":    ("int P(int a){ return a%2; }", "srawi ; addze ; rlwinm ; subf"),
    "lit-1":    ("int P(int a){ return a%1; }", "a single addi"),
    "lit-m1":   ("int P(int a){ return a%-1; }", "a single addi"),
    "lit-0":    ("int P(int a){ return a%0; }", "no division, twi 7,r0,0"),
    "lit-big":  ("int P(int a){ return a%100000; }", "addis ; ori ; divw ; mullw ; subf"),
    "lit-d-m1": ("int P(int a){ return a/-1; }", "a bare neg"),
    "lit-lhs":  ("int P(int b){ return 100%b; }", "hoisted twi 6"),
    "lit-u0":   ("unsigned P(unsigned a){ return a%0u; }", "no division, twi 7,r0,0"),
    # widths
    "short":    ("short P(short a,short b){ return (short)(a%b); }",
                 "extsh brackets, and the two traps go ADJACENT"),
    "schar":    ("signed char P(signed char a,signed char b){ return (signed char)(a%b); }",
                 "extsb brackets, traps adjacent"),
    "uchar":    ("unsigned char P(unsigned char a,unsigned char b){ return (unsigned char)(a%b); }",
                 "rlwinm brackets, traps adjacent"),
    "llong":    ("long long P(long long a,long long b){ return a%b; }", "a divd/tdi spine"),
    "ullong":   ("unsigned long long P(unsigned long long a,unsigned long long b){ return a%b; }",
                 "divdu"),
    "mixed":    ("int P(int a,short b){ return a%b; }", "an extsh on one operand only"),
    # type SPELLINGS that emit the same bytes and are refused anyway, because
    # this lane graded neither. If any of these ever reads `mismatch` the gate
    # by type-triple equality is not doing what its docs claim.
    "long":     ("long P(long a,long b){ return a%b; }",
                 "byte-identical to int per IL_TYPE_TAGS 3.1 -- refused, ungraded"),
    "ulong":    ("unsigned long P(unsigned long a,unsigned long b){ return a%b; }", "ditto"),
    "constp":   ("int P(const int a,const int b){ return a%b; }", "a qualified type id"),
    "enum":     ("enum E { E0 }; int P(int a,int b){ return (a%b)+E0; }", "a post-op"),
    # what consumes the result
    "post-add": ("int P(int a,int b){ return (a%b)+1; }", "an addi after the spine"),
    "via-loc":  ("int P(int a,int b){ int r=a%b; return r; }", "a local"),
    "store":    ("int g; void P(int a,int b){ g=a%b; }", "a store, and a void return"),
    "two-ops":  ("int P(int a,int b){ return (a/b)+(a%b); }",
                 "TWO divisions: all four traps go adjacent"),
    "unsigned-mix": ("unsigned P(int a,int b){ return (unsigned)(a%b); }", "a conversion"),
}


def main(argv):
    jobs = "8"
    if "--jobs" in argv:
        jobs = argv[argv.index("--jobs") + 1]
    wd = tempfile.mkdtemp(prefix="wdivmodcg")
    cells = []
    for name, src in ACCEPT.items():
        rel = "a_%s.cpp" % name
        open(os.path.join(wd, rel), "w").write(src + "\n")
        cells.append((name, rel, "match", ""))
    for name, (src, why) in REFUSE.items():
        rel = "r_%s.cpp" % name
        open(os.path.join(wd, rel), "w").write(src + "\n")
        cells.append((name, rel, "refuse", why))

    lst = os.path.join(wd, "files.txt")
    open(lst, "w").write("\n".join(c[1] for c in cells) + "\n")

    total_bad = 0
    total_graded = 0
    total_mismatch = 0
    summary = []
    for mode_name, flags, claims in MODES:
        ff = os.path.join(wd, "flags_%s.txt" % mode_name)
        open(ff, "w").write("\n".join(flags) + "\n")
        r = subprocess.run(
            [C2RS, "gap", "--list", lst, "--flags-file", ff, "--cwd", wd,
             "--jobs", jobs, "--no-cache"],
            capture_output=True, text=True)
        verdict = {}
        for line in r.stdout.splitlines():
            s = line.strip()
            if s.startswith("[") and "]" in s:
                rest = s.split("]", 1)[1].split()
                if len(rest) >= 2:
                    verdict[rest[1]] = rest[0]
        bad = graded = mism = 0
        print("\n=== %s  (%s)%s %s" % (mode_name, " ".join(flags),
                                       "" if claims else "  [FAIL-CLOSED: the port"
                                       " claims no body at this mode]",
                                       "=" * 10))
        print("%-16s %-9s %-12s %s" % ("cell", "expected", "graded", "c2 emits instead"))
        print("-" * 100)
        for name, rel, expect, why in cells:
            got = verdict.get(rel, "NO-RESULT")
            if got == "NO-RESULT":
                # An ungraded cell is NOT a pass. Counted separately from a
                # wrong verdict so "absence read as success" cannot happen here.
                print("%-16s %-9s %-12s  <== NOT GRADED" % (name, expect, got))
                bad += 1
                continue
            graded += 1
            if got == "mismatch":
                mism += 1
            want_match = (expect == "match") and claims
            ok = (got == "match") if want_match else (got in ("vocab-gap", "codegen-gap"))
            if not ok:
                bad += 1
            print("%-16s %-9s %-12s %s%s"
                  % (name, "match" if want_match else "refuse", got, why,
                     "" if ok else "   <== FAIL"))
        print("%s: %d cells GRADED by the oracle, %d mismatch, %d failed"
              % (mode_name, graded, mism, bad))
        if r.returncode != 0:
            print("  gap exit %d" % r.returncode)
            print(r.stderr[-1500:])
        summary.append((mode_name, graded, mism, bad))
        total_bad += bad
        total_graded += graded
        total_mismatch += mism

    print("\n" + "=" * 100)
    print("%-18s %8s %10s %8s" % ("mode", "graded", "mismatch", "failed"))
    for m, g, mm, b in summary:
        print("%-18s %8d %10d %8d" % (m, g, mm, b))
    print("-" * 46)
    print("%-18s %8d %10d %8d" % ("TOTAL", total_graded, total_mismatch, total_bad))
    print("\ncells generated: %d (%d accept, %d refuse) x %d modes = %d selected"
          % (len(cells), len(ACCEPT), len(REFUSE), len(MODES), len(cells) * len(MODES)))
    print("cells GRADED BY THE ORACLE: %d" % total_graded)
    return 1 if total_bad else 0


sys.exit(main(sys.argv))

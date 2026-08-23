#!/usr/bin/env python3
"""globregs_c2.py -- control C2 for whitebox read R4 (docs/whitebox/WB_GLOBREGS_*).

Preregistered in docs/whitebox/WB_GLOBREGS_PREREG.md section 6 and committed
BEFORE it was run.  It asks the obj -- the sole judge -- what actually orders
c2's global-register allocation candidates, because READ_PLAN section 5.3 is
explicit that a disassembly reading is a hypothesis: "[R] says the instructions
were read correctly, NOT this is what c2 does".

THE OBSERVABLE.  In a function whose formals are all live across a call, c2
must move each incoming argument register into a callee-saved register.  The
prologue therefore contains one `mr rTARGET, rARG` per formal, and the map
formal -> callee-saved register is a direct readout of the order in which the
allocator coloured the candidates: the first candidate coloured takes the
earliest register of the callee-saved run r31, r30, ... (the selector
0x10b2e7f8 walks the fixed order [r11..r3, r31..r14] and takes the earliest
allowed, and r3..r11 are all clobbered by the call).

THE CELLS.

  G-pos    POSITIVE CONTROL.  Two probes differing by one operand MUST come
           back DIFFERENT.  If they do not the instrument is dead and every
           green below is discarded rather than published -- R1's rule, and
           docs/STATUS.md's standing trap that "mismatch 0" is not evidence.

  G-ladder N formals live across a call, N = 2..8.  Reports the permutation
           formal -> callee-saved register as data.

  G-perm   THE DISCRIMINATOR, and the reason this script exists.  All 24
           permutations of the *use* order at N=4, with the *declaration*
           order held fixed:

               f(a0,a1,a2,a3) { t = sink(7); g(a_p0,a_p1,a_p2,a_p3); ... }

           Two rival readings of the mint order predict opposite results:

             (A) mint order is the SYMBOL-TABLE (arena creation) order, i.e.
                 a declaration-side property -- then all 24 permutations give
                 the SAME formal -> register map.
             (B) mint order is PROGRAM order (first definition / first use in
                 the renamer's block-layout x tuple-forward walk) -- then the
                 map FOLLOWS the permutation and 24 cells disagree.

           This is the cell the read's central correction turns on, and it can
           go red in the most likely way the read is wrong.

  Every cell is run at BOTH /O1 and /Ox.  ref/P_REGALLOC.md section 5 (board
  #3241) measured /O1 and /Ox disagreeing on 6 of 20 cells with the relation
  exact reversal, and the workload is /O1 while the fixture corpus is /Ox --
  a characterization taken at one profile publishes the wrong rule.

RED (exit 1) = the positive control failed, i.e. the instrument is dead.
The G-perm verdict is printed as A / B / MIXED and is the deliverable; it is
not an exit code, because both A and B are real results.

Outside the std-only Rust workspace on purpose -- measurement tooling, same
status as scripts/gt_argperm.py and scripts/candid_c1.py.  Degrades cleanly to
"SKIP: toolchain absent" (exit 2).

Usage:
    scripts/globregs_c2.py                  # every cell, both modes
    scripts/globregs_c2.py --ladder         # the ladder only
    scripts/globregs_c2.py --perm           # the discriminator only
"""

import os
import struct
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
from gt_dump import Obj  # noqa: E402

SCRATCH = os.path.join(os.environ.get("C2RS_WORK", os.path.join(ROOT, "work")),
                       "w-read-r4", "scratch")
MODES = ["/O1 /GS- /c", "/Ox /GS- /c"]


def find_wibo():
    """Locate wibo without hardcoding any absolute path.

    gt_capture.sh looks at <repo>/../wibo, which is wrong when the repo root is
    a git worktree under .claude/worktrees/.  Probe the worktree case too.
    """
    if os.environ.get("C2RS_WIBO"):
        return os.environ["C2RS_WIBO"]
    rel = os.path.join("wibo", "build", "release", "wibo")
    base = ROOT
    for _ in range(5):
        base = os.path.dirname(base)
        cand = os.path.join(base, rel)
        if os.path.isfile(cand) and os.access(cand, os.X_OK):
            return cand
    return ""


def compile_obj(tag, src, mode):
    os.makedirs(SCRATCH, exist_ok=True)
    cpp = os.path.join(SCRATCH, tag + ".cpp")
    with open(cpp, "w") as f:
        f.write(src)
    env = dict(os.environ)
    w = find_wibo()
    if w:
        env["C2RS_WIBO"] = w
    r = subprocess.run([os.path.join(HERE, "gt_capture.sh"), cpp] + mode.split(),
                       capture_output=True, text=True, env=env)
    path = r.stdout.strip()
    if not path or not os.path.exists(path):
        return None
    return Obj(open(path, "rb").read())


def text_words(o, want="f"):
    """The words of function `want`'s .text section."""
    if o is None:
        return None
    for s in o.sections:
        if not s["name"].startswith(".text"):
            continue
        owner = None
        for sym in o.symbols:
            if sym["sec"] == s["idx"] and sym.get("type") == 0x20:
                owner = sym["name"]
                break
        if owner == want:
            d = o.raw(s)
            if not d:
                return None
            return list(struct.unpack(">%dI" % (len(d) // 4), d))
    return None


def arg_moves(words):
    """{arg_reg: callee_saved_reg} from the prologue `mr rT, rARG` moves.

    Only the FIRST move out of each argument register counts: later `mr rN, r3`
    forms in the body are moving something else back out of r3.
    """
    out = {}
    for w in words:
        if (w >> 26) != 31 or ((w >> 1) & 0x3FF) != 444:
            continue
        rs = (w >> 21) & 31
        ra = (w >> 16) & 31
        rb = (w >> 11) & 31
        if rs != rb:                       # a real `or`, not `mr`
            continue
        if 3 <= rs <= 10 and ra >= 14 and rs not in out:
            out[rs] = ra                   # mr rTARGET(ra), rARG(rs)
    return out


def perm_map(words, n):
    """formal index -> callee-saved register, or None if not all n resolved."""
    mv = arg_moves(words)
    got = {}
    for i in range(n):
        r = mv.get(3 + i)
        if r is None:
            return None
        got[i] = r
    return got


LADDER = """extern "C" int sink(int);
extern "C" void g(%(params)s);
extern "C" int f(%(formals)s) {
    int t = sink(7);
    g(%(args)s);
    return t;
}
"""


def ladder_src(n, order=None):
    order = order if order is not None else list(range(n))
    return LADDER % {
        "params": ",".join(["int"] * n),
        "formals": ",".join("int a%d" % i for i in range(n)),
        "args": ",".join("a%d" % i for i in order),
    }


POS_A = """extern "C" int sink(int);
extern "C" void g(int,int,int,int);
extern "C" int f(int a0,int a1,int a2,int a3) {
    int t = sink(7);
    g(a0,a1,a2,a3);
    return t + a0 * 3;
}
"""
POS_B = POS_A.replace("a0 * 3", "a0 * 5")


                                                                    # noqa: E501
# G-block -- THE SEPARATOR the 24-permutation grid cannot be.
#
# G-perm shows the map does not follow USE order, which refutes a "last use
# position" key.  It does NOT separate the two survivors, because on a
# straight-line body they coincide:
#
#   (i)  cand+0x44, the step-4 ordinal (0x10b55fac), whose counter is NOT
#        reset per block (0x10b55eb7 sits outside the block loop), so the
#        value that survives is taken in the LAST BLOCK in which the
#        candidate appears; and
#   (ii) plain symbol-arena / declaration order, which is blind to blocks.
#
# G-block makes them disagree.  Both formals are defined at entry (so their
# declaration order is fixed, a0 before a1), but each has a LATER appearance
# in a DIFFERENT block, and the two variants swap which:
#
#   VAR_A:  if (c) u(a0);  u(a1);      a1's last block is later
#   VAR_B:  if (c) u(a1);  u(a0);      a0's last block is later
#
#   (i)  predicts the map SWAPS between the variants -- whoever appears in
#        the later block carries the larger ordinal and is coloured first.
#   (ii) predicts the map is UNCHANGED -- a0 is declared first, always.
#
# CAVEAT registered with the cell: the two formals no longer have identical
# live ranges here, so cand+0x0c (the PRIMARY key) may itself differ and the
# tie tier may never be reached.  A swap is therefore evidence for a
# position-derived order without proving it is +0x44 specifically; a non-swap
# is the sharper of the two outcomes.
BLOCK_A = """extern "C" int sink(int);
extern "C" void u(int);
extern "C" int f(int a0,int a1,int c) {
    int t = sink(7);
    if (c) u(a0);
    u(a1);
    return t;
}
"""
BLOCK_B = """extern "C" int sink(int);
extern "C" void u(int);
extern "C" int f(int a0,int a1,int c) {
    int t = sink(7);
    if (c) u(a1);
    u(a0);
    return t;
}
"""


def run_block(mode):
    wa = text_words(compile_obj("gblk_a", BLOCK_A, mode))
    wb = text_words(compile_obj("gblk_b", BLOCK_B, mode))
    if wa is None or wb is None:
        return None, None
    return perm_map(wa, 2), perm_map(wb, 2)


def rname(r):
    return "r%d" % r


def run_pos(mode):
    wa = text_words(compile_obj("gpos_a", POS_A, mode))
    wb = text_words(compile_obj("gpos_b", POS_B, mode))
    if wa is None or wb is None:
        return None
    return wa != wb


def run_ladder(mode, ns):
    rows = []
    for n in ns:
        w = text_words(compile_obj("glad%d" % n, ladder_src(n), mode))
        if w is None:
            rows.append((n, None))
            continue
        rows.append((n, perm_map(w, n)))
    return rows


def run_perm(mode):
    import itertools
    base = None
    results = []
    for k, order in enumerate(itertools.permutations(range(4))):
        w = text_words(compile_obj("gperm%d" % k, ladder_src(4, list(order)), mode))
        m = perm_map(w, 4) if w is not None else None
        if base is None and m is not None:
            base = m
        results.append((order, m))
    return base, results


def main(argv):
    if not os.path.isfile(os.path.join(
            ROOT, "compilers", "X360", "16.00.11886.00", "cl.exe")):
        print("SKIP: toolchain absent")
        return 2
    probe = compile_obj("smoke", ladder_src(4), MODES[0])
    if probe is None:
        print("SKIP: toolchain absent (capture produced no obj)")
        return 2

    only = None
    if "--ladder" in argv:
        only = "ladder"
    if "--perm" in argv:
        only = "perm"
    if "--block" in argv:
        only = "block"

    dead = False
    for mode in MODES:
        print("=" * 68)
        print("MODE %s" % mode)
        print("=" * 68)

        live = run_pos(mode)
        print("G-pos (positive control)        : %s"
              % ("DIFFERENT -> instrument LIVE" if live
                 else "IDENTICAL -> INSTRUMENT DEAD"))
        if not live:
            dead = True

        if only in (None, "ladder"):
            print("\nG-ladder  formal -> callee-saved register")
            for n, m in run_ladder(mode, range(2, 9)):
                if m is None:
                    print("  N=%d  (not resolved)" % n)
                    continue
                print("  N=%d  %s" % (n, "  ".join(
                    "a%d->%s" % (i, rname(m[i])) for i in sorted(m))))

        if only in (None, "block"):
            ma, mb = run_block(mode)
            print("\nG-block  the separator: later-block appearance swapped")
            for tag, m in (("VAR_A  if(c)u(a0); u(a1);", ma),
                           ("VAR_B  if(c)u(a1); u(a0);", mb)):
                print("  %s -> %s" % (tag, "  ".join(
                    "a%d->%s" % (i, rname(m[i])) for i in sorted(m))
                    if m else "(unresolved)"))
            if ma and mb:
                print("  VERDICT: %s" % (
                    "SWAPPED -- a position-derived order (consistent with "
                    "cand+0x44, whose counter is not reset per block)"
                    if ma != mb else
                    "UNCHANGED -- declaration/arena order survives; the "
                    "later-block clause is NOT observable here"))

        if only in (None, "perm"):
            base, res = run_perm(mode)
            same = sum(1 for _, m in res if m is not None and m == base)
            resolved = sum(1 for _, m in res if m is not None)
            print("\nG-perm  24 use-order permutations, declaration order fixed")
            print("  base map: %s" % ("  ".join(
                "a%d->%s" % (i, rname(base[i])) for i in sorted(base))
                if base else "(unresolved)"))
            print("  identical to base: %d of %d resolved (%d cells total)"
                  % (same, resolved, len(res)))
            if resolved and same == resolved:
                verdict = ("A -- mint order is a DECLARATION-side property; "
                           "the use order does not move it")
            elif same <= 1:
                verdict = ("B -- mint order FOLLOWS program/use order")
            else:
                verdict = "MIXED -- neither reading is clean; see the table"
            print("  VERDICT: %s" % verdict)
            if same != resolved:
                for order, m in res:
                    if m is not None and m != base:
                        print("    use %s -> %s" % (
                            "".join(str(x) for x in order),
                            "  ".join("a%d->%s" % (i, rname(m[i]))
                                      for i in sorted(m))))
        print()

    if dead:
        print("RED: the positive control did not fire -- greens are discarded")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

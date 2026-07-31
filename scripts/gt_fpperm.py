#!/usr/bin/env python3
"""gt_fpperm.py — the **floating-point** argument permutation, over complete grids.

`scripts/gt_argperm.py` settles the GPR file over 152 cells: one scratch per
local minimum of each cycle of sigma, parked ascending and read back descending
(`docs/CODEGEN_ARG_PERM.md` §2). `docs/CODEGEN_FP_ARGS.md` §1.1 asserts the FP
file "uses f0 exactly as the GPR file uses r11, and the shapes match one for
one" — on **two** captures, a 2-cycle and a 3-cycle, which is exactly the
evidence that survived to length 3 in the GPR file and then failed.

This script asks the same question of the FP file over the same complete grids,
so the claim "the FP file obeys the GPR rule" is scored on every cell rather
than assumed from two. That is a prediction, and a refutation is the more
valuable result.

Three families:

  --pure     float f(float a1..an){ gn(a_p1, ..., a_pn); }
             a tail call: no frame, no saves, nothing but the FP moves.

  --mixed    the two files at once — the interleaving `docs/CODEGEN_FP_ARGS.md`
             §1.1 records as uncharacterized. Prints the emitted order with each
             move tagged by its file, so the schedule can be read off directly.

  --widths   the same grids with `double` formals and with a `float` callee, to
             establish that the permutation is width-agnostic and to find out
             what a **narrowing** does inside a cycle.

Usage:
    scripts/gt_fpperm.py --pure [--n 2,3,4,5] [--model]
    scripts/gt_fpperm.py --mixed
    scripts/gt_fpperm.py --widths
    scripts/gt_fpperm.py --one 3,1,2
"""

import itertools
import os
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gt_dump import Obj  # noqa: E402

MODE = "/O1 /GS- /c"


def compile_src(src, workdir, tag="fp"):
    cpp = os.path.join(workdir, "%s.cpp" % tag)
    open(cpp, "w").write(src)
    r = subprocess.run([os.path.join(HERE, "gt_capture.sh"), cpp] + MODE.split(),
                       capture_output=True, text=True)
    path = r.stdout.strip()
    if not path or not os.path.exists(path):
        sys.stderr.write(r.stderr)
        return None
    return Obj(open(path, "rb").read())


def body(o, fn_prefix="?f@@"):
    for s in o.sections:
        if not s["name"].startswith(".text"):
            continue
        owner = None
        for sym in o.symbols:
            if sym["sec"] == s["idx"] and sym["type"] == 0x0020 and sym["sec"] > 0:
                owner = sym["name"]
                break
        if owner and owner.startswith(fn_prefix):
            d = o.raw(s)
            return list(struct.unpack(">%dI" % (len(d) // 4), d))
    return None


def decode(w):
    """Just enough PPC for a move schedule in both register files."""
    op = w >> 26
    rs = (w >> 21) & 31
    ra = (w >> 16) & 31
    rb = (w >> 11) & 31
    xo = (w >> 1) & 0x3FF
    if op == 31 and xo == 444 and rs == rb:
        return ("mr", ra, rs)
    if op == 63 and xo == 72:            # fmr fD,fB
        return ("fmr", rs, rb)
    if op == 63 and xo == 12:            # frsp fD,fB
        return ("frsp", rs, rb)
    if op == 18:
        return ("bl" if (w & 1) else "b",)
    if op == 37:
        return ("stwu", rs, ra, 0x10000 - (w & 0xFFFF))
    return ("?%02d/%d" % (op, xo), w)


def seq_to_moves(words):
    start = 0
    for i, w in enumerate(words):
        if decode(w)[0] == "stwu":
            start = i + 1
            break
    out = []
    for w in words[start:]:
        d = decode(w)
        if d[0] in ("bl", "b"):
            break
        if d[0] in ("mr", "fmr", "frsp"):
            out.append(d)
        else:
            out.append(d)          # keep the unknown so a frame is visible
    return out


def fmt(moves):
    parts = []
    for m in moves:
        if m[0] == "mr":
            parts.append("r%d<-r%d" % (m[1], m[2]))
        elif m[0] == "fmr":
            parts.append("f%d<-f%d" % (m[1], m[2]))
        elif m[0] == "frsp":
            parts.append("f%d<-rsp(f%d)" % (m[1], m[2]))
        else:
            parts.append(str(m))
    return " ; ".join(parts) if parts else "(nothing)"


# ---------------------------------------------------------------------------
# The candidate model: `gt_argperm.predict_pure` with the FP file's numbering
# and its scratch pool substituted, and NOTHING else changed. If the FP file is
# the GPR rule under a relabelling, this scores 100 %.
# ---------------------------------------------------------------------------

# Scratch order, to be established by the grid. The GPR file hands out r11 then
# r10 (descending from the top of the volatile scratch range). f0 is the
# published FP scratch; what a SECOND one is has never been captured, so the
# first refutation at n>=4 is where this list gets its second entry.
FP_SCRATCH = [0, 13, 12, 11]


def predict_pure(perm, readback="asc"):
    """§2's rule with f(k+1) for r(2+k) and FP_SCRATCH for [r11, r10].

    `readback` selects the order the parked values are restored in — "desc" is
    the GPR file's rule verbatim (`docs/CODEGEN_ARG_PERM.md` §2, descending
    order of the local minimum), "asc" is what the FP grid shows, and "park" is
    the order the scratches were handed out in. The three agree at one scratch
    and separate only from n=4, which is why §1.1's two captures could not tell
    them apart.
    """
    n = len(perm)
    sigma = {k + 1: perm[k] for k in range(n)}      # dest f(k+1) <- f(perm[k])
    seen, cycles = set(), []
    for d in sorted(sigma):
        if d in seen:
            continue
        c, x = [], d
        while x not in seen:
            seen.add(x)
            c.append(x)
            x = sigma[x]
        if len(c) > 1:
            cycles.append(c)
    parks, chains = [], []
    for c in cycles:
        k = len(c)
        mins = [i for i in range(k) if c[i - 1] > c[i] < c[(i + 1) % k]]
        for i in mins:
            parks.append((c[(i + 1) % k], c[i]))
        for i in mins:
            b, j = [], (i + 1) % k
            while j not in mins:
                b.append((c[j], c[(j + 1) % k]))
                j = (j + 1) % k
            if b:
                chains.append(b)
    parks.sort()
    scratch = {src: FP_SCRATCH[idx] for idx, (src, _) in enumerate(parks)}
    moves = [("fmr", FP_SCRATCH[idx], src) for idx, (src, _) in enumerate(parks)]
    chains.sort(key=lambda b: -b[0][0])
    for b in chains:
        for d, s in b:
            moves.append(("fmr", d, s))
    if readback == "desc":
        order = sorted(parks, key=lambda p: -p[1])
    elif readback == "asc":
        order = sorted(parks, key=lambda p: p[1])
    else:                                   # "park" — scratch allocation order
        order = list(parks)
    for src, mn in order:
        moves.append(("fmr", mn, scratch[src]))
    return moves


def run_pure(n, workdir, ty="float", callee_ty=None):
    callee_ty = callee_ty or ty
    ident = tuple(range(1, n + 1))
    decl = "void g%d(%s);" % (n, ",".join([callee_ty] * n))
    rows = []
    for perm in itertools.permutations(ident):
        params = ",".join("%s a%d" % (ty, i) for i in ident)
        args = ",".join("a%d" % p for p in perm)
        src = "%s\nvoid f(%s){ g%d(%s); }\n" % (decl, params, n, args)
        o = compile_src(src, workdir, "fp%d" % n)
        if o is None:
            print("COMPILE FAIL", perm)
            continue
        rows.append((perm, seq_to_moves(body(o))))
    return rows


MIXED = [
    # (caller formals, callee formals, argument list) — every combination in
    # which BOTH files have a non-trivial permutation, plus the controls where
    # only one does.
    ("int a,int b,float c,float d", "int,int,float,float", "b,a,d,c"),
    ("int a,int b,float c,float d", "int,int,float,float", "b,a,c,d"),
    ("int a,int b,float c,float d", "int,int,float,float", "a,b,d,c"),
    ("int a,float b,int c,float d", "int,float,int,float", "c,d,a,b"),
    ("int a,float b,int c,float d", "float,int,float,int", "b,a,d,c"),
    ("int a,int b,int c,float d,float e,float f", "int,int,int,float,float,float",
     "c,a,b,f,d,e"),
    ("float a,float b,int c,int d", "float,float,int,int", "b,a,d,c"),
    ("int a,int b,float c,float d,float e", "int,int,float,float,float",
     "b,a,e,c,d"),
]


def run_mixed(workdir):
    rows = []
    for i, (params, ctys, args) in enumerate(MIXED):
        n = len(ctys.split(","))
        src = ("void g%d(%s);\nvoid f(%s){ g%d(%s); }\n"
               % (n, ctys, params, n, args))
        o = compile_src(src, workdir, "mx%d" % i)
        if o is None:
            print("COMPILE FAIL", params, args)
            continue
        rows.append(((params, args), seq_to_moves(body(o))))
    return rows


WIDTHS = [
    ("double a,double b", "double,double", "b,a"),
    ("double a,double b", "float,float", "b,a"),
    ("float a,double b", "double,double", "b,a"),
    ("double a,float b", "float,float", "b,a"),
    ("double a,double b,double c", "double,double,double", "b,c,a"),
    ("double a,double b,double c", "float,float,float", "b,c,a"),
    ("float a,float b", "double,double", "b,a"),
    ("float a,float b,float c", "double,double,double", "b,c,a"),
    ("double a,float b,double c", "double,float,double", "c,b,a"),
    ("float a,double b,float c", "float,double,float", "c,b,a"),
    # a narrowing on ONE argument of a cycle
    ("double a,double b,double c", "float,double,double", "b,c,a"),
    ("double a,double b,double c", "double,double,float", "b,c,a"),
]


def run_widths(workdir):
    rows = []
    for i, (params, ctys, args) in enumerate(WIDTHS):
        n = len(ctys.split(","))
        src = ("void g%d(%s);\nvoid f(%s){ g%d(%s); }\n"
               % (n, ctys, params, n, args))
        o = compile_src(src, workdir, "wd%d" % i)
        if o is None:
            print("COMPILE FAIL", params, args)
            continue
        rows.append(((params + "  ->  g(" + ctys + ")", args), seq_to_moves(body(o))))
    return rows


def main(argv):
    wd = tempfile.mkdtemp(prefix="gtfpperm")
    ns = [2, 3, 4, 5]
    if "--n" in argv:
        ns = [int(x) for x in argv[argv.index("--n") + 1].split(",")]
    if "--one" in argv:
        perm = tuple(int(x) for x in argv[argv.index("--one") + 1].split(","))
        n = len(perm)
        decl = "void g%d(%s);" % (n, ",".join(["float"] * n))
        params = ",".join("float a%d" % i for i in range(1, n + 1))
        args = ",".join("a%d" % p for p in perm)
        src = "%s\nvoid f(%s){ g%d(%s); }\n" % (decl, params, n, args)
        print(src)
        o = compile_src(src, wd)
        for i, w in enumerate(body(o)):
            print("  %04x  %08x  %s" % (i * 4, w, decode(w)))
        return 0
    if "--mixed" in argv:
        print("== mixed: both register files, the schedule as emitted")
        for (params, args), moves in run_mixed(wd):
            print("  f(%s) -> g(%s)" % (params, args))
            print("      %s" % fmt(moves))
        return 0
    if "--widths" in argv:
        print("== widths and boundary conversions inside a permutation")
        for (what, args), moves in run_widths(wd):
            print("  %s   args(%s)" % (what, args))
            print("      %s" % fmt(moves))
        return 0

    check = "--model" in argv
    for n in ns:
        rows = run_pure(n, wd)
        print("== pure FP, n=%d  (%d permutations)" % (n, len(rows)))
        if not check:
            for perm, moves in rows:
                print("  %-16s %s" % (str(perm), fmt(moves)))
            continue
        best, best_miss = None, None
        for variant in ("asc", "desc", "park"):
            miss = [p for p, m in rows if predict_pure(p, variant) != m]
            print("  readback=%-5s refutations: %3d / %d" % (variant, len(miss), len(rows)))
            if best_miss is None or len(miss) < len(best_miss):
                best, best_miss = variant, miss
        print("  -- refutations of the best variant (%s):" % best)
        for perm, moves in rows:
            pred = predict_pure(perm, best)
            if pred == moves:
                continue
            print("  %-18s got  %s" % (str(perm), fmt(moves)))
            print("  %-18s pred %s" % ("", fmt(pred)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

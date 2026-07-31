#!/usr/bin/env python3
"""gt_argperm.py — the argument-permutation lowering, over complete grids.

`docs/CODEGEN_FRAMED_CALLS.md` §3.2 gives the rule as "non-conflicting moves
highest destination first; a permutation is broken with r11 as the scratch",
which is right to a 3-cycle and *described* rather than explained past it.
`docs/CODEGEN_FP_ARGS.md` §1.1 adds that the FP file uses f0 the same way, and
§6 records a third rule again — when a permuted value is *also* callee-saved,
c2 emits no `r11` at all.

This script generates the complete permutation grid at each arity and prints
what c2 actually emits, so a candidate model can be checked against every cell
instead of the three that were lying around.

Two families:

  --pure     void f(int a1..an){ gn(a_p1, ..., a_pn); }
             a tail call: no frame, no saves, nothing in the body but the moves.

  --saved    void f(int a1..an){ gn(a_p1,...,a_pn); v(a_k); ... }
             the same first call, plus later single-argument calls that force
             some formals into callee-saved registers. This is the family §6
             refused at: which register the permutation reads from when the
             value is also saved.

Usage:
    scripts/gt_argperm.py --pure [--n 2,3,4,5] [--model] [--minima K]
    scripts/gt_argperm.py --saved [--n 3,4]
    scripts/gt_argperm.py --one 3,1,2            # one permutation, disassembled

`--model` scores the candidate model in `predict_pure()` against every cell and
prints only the refutations; that is the point of the script.
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


def compile_src(src, workdir, tag="ap"):
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
    """The words of the named function's .text, plus its relocations."""
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
            words = list(struct.unpack(">%dI" % (len(d) // 4), d))
            rels = {va: o.sym_by_index(sym)["name"] for va, sym, ty in o.relocs(s)}
            return words, rels
    return None, None


def decode(w):
    """Just enough PPC to read a move sequence: `mr`, `li`, `lwz`, `bl`, `b`."""
    op = w >> 26
    rs = (w >> 21) & 31
    ra = (w >> 16) & 31
    rb = (w >> 11) & 31
    if op == 31 and ((w >> 1) & 0x3FF) == 444 and rs == rb:      # or rA,rS,rS
        return ("mr", ra, rs)
    if op == 31 and ((w >> 1) & 0x3FF) == 444:
        return ("or", ra, rs, rb)
    if op == 14:
        return ("addi", rs, ra, w & 0xFFFF)
    if op == 32:
        return ("lwz", rs, ra, w & 0xFFFF)
    if op == 36:
        return ("stw", rs, ra, w & 0xFFFF)
    if op == 18:
        return ("bl" if (w & 1) else "b",)
    if op == 37:
        return ("stwu", rs, ra, 0x10000 - (w & 0xFFFF))
    if op == 31 and ((w >> 1) & 0x3FF) == 339:
        return ("mfspr",)
    if op == 62:
        return ("std", rs, ra, w & 0xFFFC)
    return ("?%02d" % op, w)


def seq_to_moves(words, rels):
    """The move sequence between the end of the prologue and the first call.

    The prologue is skipped past the `stwu r1,-F(r1)` when there is a frame —
    otherwise a Class C prologue's `bl __savegprlr_N` would be mistaken for the
    call and every framed row would come back empty. Prologue stores through r1
    are dropped for the same reason.
    """
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
        if d[0] == "mr" or d[0] == "or":
            out.append(d)
        elif d[0] in ("lwz", "stw") and d[2] != 1:
            out.append(d)
    return out


def fmt(moves):
    parts = []
    for m in moves:
        if m[0] == "mr":
            parts.append("r%d<-r%d" % (m[1], m[2]))
        elif m[0] == "lwz":
            parts.append("r%d<-[r%d+%d]" % (m[1], m[2], m[3]))
        elif m[0] == "stw":
            parts.append("[r%d+%d]<-r%d" % (m[2], m[3], m[1]))
        elif m[0] == "addi":
            parts.append("r%d<-r%d+%d" % (m[1], m[2], m[3]))
        else:
            parts.append(str(m))
    return " ; ".join(parts)


# ---------------------------------------------------------------------------
# The candidate model for the pure (no-save) case.
# ---------------------------------------------------------------------------

def predict_pure(perm):
    """THE MODEL, derived from the complete n=2..5 grids and checked against
    every cell of them by `--model`.

    Destination slot k is r(2+k) and wants formal perm[k], which lives in
    r(2+perm[k]); write sigma(d) for that source. sigma is a permutation of the
    argument registers; decompose it into cycles.

    Write each cycle as the cyclic sequence c0, c1 = sigma(c0), ... A register
    ci is a **local minimum** of that sequence when c(i-1) > ci < c(i+1),
    cyclically. Then:

      1. PARK: for every local minimum ci, park sigma(ci) = c(i+1) into a
         scratch. Scratches are handed out r11, then r10, in ASCENDING order of
         the parked source register (not in cycle order).
      2. BODY: each local minimum opens a chain c(i+1)<-c(i+2), c(i+2)<-c(i+3),
         ... which runs until the next local minimum. Inside a chain the order
         is forced by the dependencies. Chains are emitted in DESCENDING order
         of their first destination.
      3. READ BACK: each local minimum takes its parked value, ci <- scratch,
         emitted last, in descending order of ci.

    The number of scratch registers is therefore the total number of local
    minima over all cycles — 1 for a "unimodal" cycle, 2 for one with a valley,
    and one per cycle when there are several. That is why 3-cycles never need
    two (three elements cannot make a valley after the anchor) and why the
    published "r11 breaks the cycle" rule survives to length 3 and no further.
    """
    n = len(perm)
    sigma = {2 + k + 1: 2 + perm[k] for k in range(n)}
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
    parks, chains, reads = [], [], []
    for c in cycles:
        k = len(c)
        mins = [i for i in range(k) if c[i - 1] > c[i] < c[(i + 1) % k]]
        for i in mins:
            parks.append((c[(i + 1) % k], c[i]))       # (source to park, its minimum)
        for i in mins:
            body, j = [], (i + 1) % k
            while j not in mins:
                body.append((c[j], c[(j + 1) % k]))    # dest <- src
                j = (j + 1) % k
            if body:
                chains.append(body)
    parks.sort()                                        # ascending source register
    scratch = {src: 11 - idx for idx, (src, _) in enumerate(parks)}
    moves = [("mr", 11 - idx, src) for idx, (src, _) in enumerate(parks)]
    chains.sort(key=lambda b: -b[0][0])                 # descending first destination
    for body in chains:
        for d, s in body:
            moves.append(("mr", d, s))
    for src, mn in sorted(parks, key=lambda p: -p[1]):  # descending minimum
        moves.append(("mr", mn, scratch[src]))
    return moves


def n_minima(perm):
    """The scratch count §2's rule predicts: total local minima of sigma's cycles.

    Split out of predict_pure() so a grid can be FILTERED on it. The whole grid
    at n = 6 is 720 objects; the cells that decide whether a third scratch (r9)
    exists are the 61 with three local minima, and nothing is learned by
    compiling the other 659.
    """
    n = len(perm)
    sigma = {2 + k + 1: 2 + perm[k] for k in range(n)}
    seen, tot = set(), 0
    for d in sorted(sigma):
        if d in seen:
            continue
        c, x = [], d
        while x not in seen:
            seen.add(x)
            c.append(x)
            x = sigma[x]
        if len(c) > 1:
            k = len(c)
            tot += sum(1 for i in range(k) if c[i - 1] > c[i] < c[(i + 1) % k])
    return tot


def run_pure(n, workdir, show_all=False, model=False, min_minima=0):
    ident = tuple(range(1, n + 1))
    decl = "void g%d(%s);" % (n, ",".join(["int"] * n))
    rows = []
    for perm in itertools.permutations(ident):
        if n_minima(perm) < min_minima:
            continue
        params = ",".join("int a%d" % i for i in ident)
        args = ",".join("a%d" % p for p in perm)
        src = "%s\nvoid f(%s){ g%d(%s); }\n" % (decl, params, n, args)
        o = compile_src(src, workdir, "ap%d" % n)
        if o is None:
            print("COMPILE FAIL", perm)
            continue
        words, rels = body(o)
        moves = seq_to_moves(words, rels)
        rows.append((perm, moves))
    return rows


def run_saved(n, workdir):
    """The same first call, followed by one single-argument call per formal so
    that every formal is live across the first call and must be callee-saved."""
    ident = tuple(range(1, n + 1))
    rows = []
    for perm in itertools.permutations(ident):
        params = ",".join("int a%d" % i for i in ident)
        args = ",".join("a%d" % p for p in perm)
        tail = " ".join("v%d(a%d);" % (i, i) for i in ident)
        decl = ("void g%d(%s);\n" % (n, ",".join(["int"] * n))
                + "\n".join("void v%d(int);" % i for i in ident))
        src = "%s\nvoid f(%s){ g%d(%s); %s }\n" % (decl, params, n, args, tail)
        o = compile_src(src, workdir, "as%d" % n)
        if o is None:
            print("COMPILE FAIL", perm)
            continue
        words, rels = body(o)
        moves = seq_to_moves(words, rels)
        rows.append((perm, moves))
    return rows


# ---------------------------------------------------------------------------
# --dest: the DESTINATION grid, past where CODEGEN_FP_ARGS.md §0 was measured.
#
# §0's rule — a non-FP scalar takes r(2+k) for its 1-based argument slot k, an
# FP parameter consuming a slot without filling a register — is measured only
# to FIVE arguments, and EIGHT is where the argument GPRs run out (r3..r10).
# The port fails closed past 5 by arithmetic rather than by measurement, which
# is the weaker guarantee and the exact shape this lane has caught wrong twice.
#
# The instrument is deliberately not the permutation machinery: each integer
# argument is a DISTINCT CONSTANT, so c2 must materialise it with `li rD,imm`
# and the destination register is read straight off the instruction with no
# model in between. The FP arguments are the caller's own FP formals in the
# same relative order, so they are already in f1,f2,... and emit nothing —
# they are present only to consume slots.
#
#   sig "diiii" -> void g(double,int,int,int,int);
#                  void f(double x1){ g(x1, 101, 102, 103, 104); }
#
# Anything the caller cannot put in a register has to go to the stack, so a
# `stw rD,off(r1)` (and the `stwu` that makes room for it) is the homing
# answer, read the same way.
# ---------------------------------------------------------------------------
def dest_src(sig):
    """(source, {constant: slot}) for one signature over 'i' and 'd'/'f'."""
    ctys = {"i": "int", "d": "double", "f": "float"}
    decl = "void gd(%s);" % ",".join(ctys[c] for c in sig)
    formals, args, want = [], [], {}
    nfp = 0
    for k, c in enumerate(sig, 1):
        if c == "i":
            args.append(str(100 + k))
            want[100 + k] = k
        else:
            nfp += 1
            formals.append("%s x%d" % (ctys[c], nfp))
            args.append("x%d" % nfp)
    src = "%s\nvoid f(%s){ gd(%s); }\n" % (
        decl, ",".join(formals), ",".join(args))
    return src, want


def run_dest(sig, wd):
    src, want = dest_src(sig)
    o = compile_src(src, wd, tag="dest_" + sig)
    if o is None:
        return None
    words, rels = body(o)
    if words is None:
        return None
    got, stores, frame = {}, [], None
    for w in words:
        d = decode(w)
        if d[0] == "stwu":
            frame = d[3]
        elif d[0] == "addi" and d[2] == 0 and d[3] in want:
            got[want[d[3]]] = "r%d" % d[1]          # li rD, 100+k
        elif d[0] == "stw" and d[2] == 1:
            stores.append((d[1], d[3]))
        elif d[0] in ("bl", "b"):
            break
    # A constant stored rather than left in a register is a homed argument: the
    # `li` names a register and the `stw` sends it to the frame, so report the
    # slot as the stack offset it actually reached.
    byreg = {v: k for k, v in got.items()}
    for reg, off in stores:
        slot = byreg.get("r%d" % reg)
        if slot is not None:
            got[slot] = "%s->%d(r1)" % (got[slot], off)
    return got, frame, words


def dest_grid(sigs, wd):
    print("sig            frame   slot->destination"
          "   (li rD,100+k reads the destination directly)")
    bad = 0
    for sig in sigs:
        r = run_dest(sig, wd)
        if r is None:
            print("  %-12s capture failed" % sig)
            bad += 1
            continue
        got, frame, _ = r
        cells = []
        for k, c in enumerate(sig, 1):
            if c != "i":
                cells.append("%d:%s(f)" % (k, c))
            else:
                # §0's rule, stated as a prediction on every row so the grid
                # falsifies it rather than illustrating it.
                pred = "r%d" % (2 + k) if 2 + k <= 10 else "STACK"
                g = got.get(k, "MISSING")
                cells.append("%d:%s%s" % (k, g, "" if g == pred
                                          else "[PRED %s]" % pred))
        print("  %-12s %-7s %s"
              % (sig, "-" if frame is None else "0x%x" % frame,
                 "  ".join(cells)))
    return bad


def main(argv):
    wd = tempfile.mkdtemp(prefix="gtperm")
    if "--dest" in argv:
        i = argv.index("--dest")
        rest = [a for a in argv[i + 1:] if not a.startswith("--")]
        sigs = rest or (
            ["i" * n for n in range(1, 11)]
            + ["d" + "i" * n for n in range(1, 10)]
            + ["i" * n + "d" for n in range(1, 10)]
            + ["idididid", "iiiidiii", "ddiiiiii", "iiiiiidd",
               "fiiiiiii", "fffffffi", "iiiiiiidi", "diiiiiiii",
               "iiiiiiiid", "iiiiiiiii", "iiiiiiiiii"])
        return 1 if dest_grid(sigs, wd) else 0
    ns = [2, 3, 4, 5]
    if "--n" in argv:
        ns = [int(x) for x in argv[argv.index("--n") + 1].split(",")]
    if "--one" in argv:
        perm = tuple(int(x) for x in argv[argv.index("--one") + 1].split(","))
        n = len(perm)
        decl = "void g%d(%s);" % (n, ",".join(["int"] * n))
        params = ",".join("int a%d" % i for i in range(1, n + 1))
        args = ",".join("a%d" % p for p in perm)
        src = "%s\nvoid f(%s){ g%d(%s); }\n" % (decl, params, n, args)
        print(src)
        o = compile_src(src, wd)
        words, rels = body(o)
        for i, w in enumerate(words):
            print("  %04x  %08x  %s%s" % (i * 4, w, decode(w),
                                          "   ; -> " + rels[i * 4] if i * 4 in rels else ""))
        return 0

    fam = "saved" if "--saved" in argv else "pure"
    check = "--model" in argv
    mm = 0
    if "--minima" in argv:
        mm = int(argv[argv.index("--minima") + 1])
    for n in ns:
        rows = (run_saved(n, wd) if fam == "saved"
                else run_pure(n, wd, min_minima=mm))
        miss = 0
        print("== %s, n=%d  (%d permutations%s)"
              % (fam, n, len(rows),
                 ", filtered to >= %d predicted minima" % mm if mm else ""))
        # The scratch registers c2 ACTUALLY used, as a census. §2 predicts they
        # are handed out r11, r10, r9, ... one per local minimum; r9 has never
        # been observed, because n <= 5 cannot produce three minima.
        used = {}
        for _, moves in rows:
            for r in sorted({d for op, d, sx in moves if op == "mr" and d >= 9
                             and d > n + 2}):
                used[r] = used.get(r, 0) + 1
        if rows:
            print("   scratch registers observed: %s"
                  % (", ".join("r%d in %d cells" % (r, c)
                               for r, c in sorted(used.items(), reverse=True))
                     or "none"))
        for perm, moves in rows:
            if check and fam == "pure":
                pred = predict_pure(perm)
                ok = pred == moves
                if ok:
                    continue
                miss += 1
                print("  %-16s got  %s" % (str(perm), fmt(moves)))
                print("  %-16s pred %s" % ("", fmt(pred)))
            else:
                print("  %-16s %s" % (str(perm), fmt(moves)))
        if check:
            print("  refutations: %d / %d" % (miss, len(rows)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

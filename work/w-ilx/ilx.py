#!/usr/bin/env python3
"""ilx.py — GRID I: the byte-level minimal pairs of #891 and GRID S's
`self`/`cross` row.

Declared in `work/w-ilx/PREREG.md` §2, committed at the lane's first commit
before this file existed.

WHAT IT DOES
------------
For each of five PAIRS it builds two `.cpp` cells that differ in exactly one
token of C++, then

  * compiles the obj at the WORKLOAD's own flags through `work/w-frame/refobj.sh`
    and reads the allocation off the disassembly (the producer's register from
    ITS OWN store's displacement — no regex names a source register, w-refbind's
    OOR bug);
  * captures the IL bundle from the SAME file through `c2rs capture --keep-il`
    and diffs `.ex`/`.sy`/`.gl`/`.in`/`.db` byte for byte.

THE CONFOUND THIS FILE EXISTS TO AVOID  (PREREG §1.1)
-----------------------------------------------------
Names are IN the IL.  `w-spell`'s grids give every cell a unique struct and
function name so the objs can share a directory; comparing two of those captures
by bytes would compare their NAMES.  Here **both cells of a pair carry the same
struct name, the same function name, the same local names and the same
formals** — only the body text differs.  The pair TAG is used for the directory,
never for an identifier.

**AND THE SOURCE PATH IS IN THE `.gl` TOO.**  The first run of this file put
each cell in its own directory and the `.gl` streams differed in 22 byte-runs
that were all the *directory name* (`s-11-self/c.cpp` against
`s-11-cross/c.cpp`) and the content hash that follows it.  So every cell is
written to ONE shared path — `work/w-ilx/cell/c.cpp` — and compiled serially;
the artefacts are copied out afterwards.  What remains in a `.gl` diff is then
the 16-byte source-content hash and the two length/offset words around it,
which is a property of the text and cannot be removed.

PAIRS  (PREREG §2)
------------------
    S-11   self vs cross, 1base, (ru 1, cu 1)     objs DISAGREE  (P vs c)
    S-21   self vs cross, 1base, (2, 1)           objs AGREE     (control)
    X-35   &s->inner vs &q,  bind, (3, 5)         objs DISAGREE  (#891)
    X-11   &s->inner vs &q,  bind, (1, 1)         objs AGREE     (control)
    X-AE   &s->inner+bind vs &s->inner no-bind, (3,5)  objs DISAGREE

SHIPS NOTHING.  Usage:  ilx.py [--jobs N]
"""

import hashlib
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
C2RS = os.path.join(ROOT, "target", "release", "c2rs")
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")
OUT = os.path.join(HERE, "gridI")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# GRID S's layout, verbatim from `work/w-spell/spellgrid.py`:
#   head@0(32) . f0..ff@32..92 . inner@96(32) . inner2@128(32)
STRUCT = """\
struct L { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S {
    L head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L inner;
    L inner2;
};
"""
OFF_INNER, OFF_F0 = 96, 32
FSLOT = "0123456789abcdef"


def body(expr, ru, cu, bind, regfirst=False):
    """The GRID S / GRID X body shape: the constant's run first, then the
    producer's.  `bind` puts `L& q = s->inner;` at the head and stores through
    it — w-spell's `2base` and GRID X's A/B/C/D.  `regfirst` swaps the two runs,
    which is GRID X's `F`."""
    out = []
    pslot = "s->inner.a%d"
    if bind:
        out.append("    L& q = s->inner;")
        pslot = "q.a%d"
    const = ["    s->f%s = 7;" % FSLOT[i] for i in range(cu)]
    prod = ["    %s = %s;" % (pslot % i, expr) for i in range(ru)]
    return out + (prod + const if regfirst else const + prod)


def source(expr, ru, cu, bind, regfirst=False):
    return (STRUCT + "void g(S* s, int u, int v) {\n"
            + "\n".join(body(expr, ru, cu, bind, regfirst)) + "\n}\n")


# tag -> (left-label, left-spec, right-label, right-spec, expected)
#   spec = (expr, ru, cu, bind)
PAIRS = [
    ("S-11", "self", ("(int)&s->inner", 1, 1, False),
             "cross", ("(int)&s->inner2", 1, 1, False), "DISAGREE"),
    ("S-21", "self", ("(int)&s->inner", 2, 1, False),
             "cross", ("(int)&s->inner2", 2, 1, False), "AGREE"),
    ("X-35", "A", ("(int)&s->inner", 3, 5, True),
             "B", ("(int)&q", 3, 5, True), "DISAGREE"),
    ("X-11", "A", ("(int)&s->inner", 1, 1, True),
             "B", ("(int)&q", 1, 1, True), "AGREE"),
    ("X-AE", "A", ("(int)&s->inner", 3, 5, True),
             "E", ("(int)&s->inner", 3, 5, False), "DISAGREE"),
]

STORE_RX = re.compile(r"^(st[bhwd]u?)\s+(\d+),\s*(-?\d+)\((\d+)\)$")
DEF_RX = re.compile(r"^([a-z][a-z0-9._]*)\s+(\d+),")


def dis(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def observe(words, ru, cu, pbase=OFF_INNER, cbase=OFF_F0):
    """w-spell's grader, verbatim in behaviour: the producer's register is read
    off its own store's DISPLACEMENT, so no regex names a base register.

    `pbase`/`cbase` are the two runs' first store offsets and DEFAULT to GRID
    I's layout.  They are parameters because the first `holdout.py --grade` run
    used the defaults against GRID V's fresh struct and all 45 cells came back
    `OOR prod regs 0` — the instrument reporting that it had matched nothing,
    which is STATUS trap 5 doing its job.  Had `observe` returned a verdict
    instead of a counter, that grade would have looked like a result."""
    poff = [pbase + 4 * i for i in range(ru)]
    coff = [cbase + 4 * i for i in range(cu)]
    st = [(i, int(m.group(2)), int(m.group(3)))
          for i, m in ((i, STORE_RX.match(w)) for i, w in enumerate(words)) if m]
    pr = {s[1] for s in st if s[2] in poff}
    cr = {s[1] for s in st if s[2] in coff}
    if len(pr) != 1 or len(cr) != 1:
        return "OOR prod regs %d, const regs %d" % (len(pr), len(cr))
    preg, creg = pr.pop(), cr.pop()
    if preg == creg:
        return "OOR both runs store out of r%d" % preg
    for r in (preg, creg):
        n = sum(1 for w in words
                if DEF_RX.match(w) and int(DEF_RX.match(w).group(2)) == r
                and not STORE_RX.match(w))
        if n != 1:
            return "OOR r%d defined %d times (#644)" % (r, n)
    return "prod" if preg > creg else "const"


CELL = os.path.join(HERE, "cell")


def capture(cell, src, il_only=False):
    """Compile the obj AND capture the IL from ONE file at ONE shared path, so
    neither the directory name nor the file name is in the IL (PREREG §1.1).

    `il_only=True` skips the obj entirely and returns `words = None`.  That is
    what `holdout.py --freeze` calls: a freeze that disassembled the obj it is
    about to be graded against would not be a freeze."""
    os.makedirs(CELL, exist_ok=True)
    cpp = os.path.join(CELL, "c.cpp")
    open(cpp, "w").write(src)
    rel = os.path.relpath(cpp, DC3)

    words = None
    obj = os.path.join(CELL, "c.obj")
    if os.path.exists(obj):
        os.remove(obj)
    if not il_only:
        r = subprocess.run([REFOBJ, rel, obj], capture_output=True, text=True,
                           env=dict(os.environ, C2RS_DC3=DC3))
        words = dis(obj) if (r.returncode == 0 and os.path.exists(obj)) else None

    ildir = os.path.join(CELL, "il")
    os.makedirs(ildir, exist_ok=True)
    for fn in os.listdir(ildir):
        p = os.path.join(ildir, fn)
        if os.path.isfile(p):
            os.remove(p)
    r = subprocess.run([C2RS, "capture", rel, "--keep-il", ildir,
                        "--flags-file", FLAGS, "--cwd", DC3],
                       capture_output=True, text=True, cwd=ROOT)
    streams = {}
    if r.returncode == 0:
        for fn in sorted(os.listdir(ildir)):
            p = os.path.join(ildir, fn)
            if os.path.isfile(p):
                # capture names embed a per-run tag; key on the EXTENSION
                streams[os.path.splitext(fn)[1] or fn] = open(p, "rb").read()

    # keep the artefacts for the record, AFTER the shared path has done its job
    d = os.path.join(OUT, cell)
    os.makedirs(d, exist_ok=True)
    shutil.copyfile(cpp, os.path.join(d, "c.cpp"))
    for e, b in streams.items():
        open(os.path.join(d, "c" + e), "wb").write(b)
    return words, streams


def runs(a, b):
    """The differing byte RUNS between two equal-or-unequal-length streams,
    as (offset, a-bytes, b-bytes).  Contiguous differing offsets are merged."""
    n = min(len(a), len(b))
    d = [i for i in range(n) if a[i] != b[i]]
    out = []
    for i in d:
        if out and i == out[-1][1]:
            out[-1][1] = i + 1
        else:
            out.append([i, i + 1])
    return [(s, a[s:e], b[s:e]) for s, e in out]


def main():
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        return 3
    os.makedirs(OUT, exist_ok=True)
    log = open(os.path.join(HERE, "gridI_dis.txt"), "w")

    selected = reached = graded = oor = failed = 0
    verdict = {}
    for tag, ln, ls, rn, rs, expect in PAIRS:
        print("\n=== %s   %s  vs  %s   (registered: objs %s)"
              % (tag, ln, rn, expect))
        cells = {}
        for lbl, spec in ((ln, ls), (rn, rs)):
            selected += 1
            src = source(*spec)
            words, streams = capture("%s-%s" % (tag, lbl), src)
            if words is None or not streams:
                failed += 1
                print("  %-6s COMPILE/CAPTURE FAILED" % lbl)
                continue
            reached += 1
            v = observe(words, spec[1], spec[2])
            if v.startswith("OOR"):
                oor += 1
            else:
                graded += 1
            cells[lbl] = (v, streams, words)
            log.write("== %s %s\n%s\n%s\n\n"
                      % (tag, lbl, src, "\n".join(words)))
            print("  %-6s obj: %-8s | %s" % (lbl, v, "  ".join(
                "%s=%d %s" % (e, len(b), hashlib.sha256(b).hexdigest()[:8])
                for e, b in sorted(streams.items()))))
        if len(cells) != 2:
            continue
        (lv, lst, _), (rv, rst, _) = cells[ln], cells[rn]
        agree = (lv == rv)
        ok = (agree == (expect == "AGREE"))
        verdict[tag] = (lv, rv, agree)
        print("  objs %s -> %s"
              % ("AGREE" if agree else "DISAGREE", "as registered" if ok
                 else "**NOT AS REGISTERED**"))

        exts = sorted(set(lst) | set(rst))
        ident = []
        for e in exts:
            a, b = lst.get(e), rst.get(e)
            if a is None or b is None:
                print("    %-4s  present on only one side" % e)
                continue
            if a == b:
                ident.append(e)
                print("    %-4s  %6d  IDENTICAL" % (e, len(a)))
                continue
            rr = runs(a, b)
            print("    %-4s  %6d vs %6d  %s  %d differing byte-run(s)"
                  % (e, len(a), len(b),
                     "same length" if len(a) == len(b) else "DIFFERENT LENGTH",
                     len(rr)))
            for off, x, y in rr[:12]:
                print("        @0x%04x  %-6s %-26s | %-6s %s"
                      % (off, ln, x.hex(" "), rn, y.hex(" ")))
                print("        %8s ctx %s"
                      % ("", a[max(0, off - 8):off + len(x) + 8].hex(" ")))
                print("        %8s ctx %s"
                      % ("", b[max(0, off - 8):off + len(y) + 8].hex(" ")))
            if len(a) != len(b):
                i = next((j for j in range(min(len(a), len(b)))
                          if a[j] != b[j]), min(len(a), len(b)))
                k = 0
                while (k < min(len(a), len(b))
                       and a[len(a) - 1 - k] == b[len(b) - 1 - k]):
                    k += 1
                print("        edit window (first diff 0x%04x, common tail %d):"
                      % (i, k))
                print("          %-6s %dB  %s"
                      % (ln, len(a) - k - i, a[i:len(a) - k].hex(" ")))
                print("          %-6s %dB  %s"
                      % (rn, len(b) - k - i, b[i:len(b) - k].hex(" ")))

        # I5 — the first-class finding, checked positively and printed either way
        core = [e for e in (".ex", ".sy", ".gl") if e in lst and e in rst]
        allsame = bool(core) and all(lst[e] == rst[e] for e in core)
        print("    I5 (.ex/.sy/.gl byte-identical while the objs differ): %s"
              % ("**YES — THE DECISION IS OUTSIDE THE CAPTURED STREAMS**"
                 if (allsame and not agree)
                 else "no" if core else "UNCHECKED — a stream is missing"))
        print("    I1 (.sy byte-identical): %s"
              % ("HIT" if lst.get(".sy") == rst.get(".sy") else "**MISS**"))

    log.close()
    print("\n  counters:  selected %d | reached %d | GRADED %d"
          " | out-of-regime %d | compile-failed %d"
          % (selected, reached, graded, oor, failed))
    print("  pair verdicts: %s" % verdict)
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""w-memcpy GRID-M2 — the SECOND grid, against GRID-M's fence.

GRID-M refuted `M-ALWAYSCALL` at 118 of 232 cells and left one sub-class
unanimous: **every cell at a constant size >= 48 is a `bl`**, at every
alignment, for both `memcpy` and `memset`.  A fence at 48 is fitted to the grid
that produced it, which is board **#260** and is exactly the shape `w-mmio` §3
declined to ship (three fits, two refutations, the third never met a fourth
grid).  So the fence gets a grid it was NOT fitted on before anything is said
about it.

WHAT THIS GRID VARIES THAT GRID-M HELD FIXED
--------------------------------------------
  A  pointer type      long long* (align 8, a type GRID-M did not use), a
                       4-byte struct, a 32-byte struct, and `void*` — the
                       target function's OWN argument type, which GRID-M never
                       compiled
  B  operand kind      two formals (GRID-M's only case) | a formal and a LOCAL
                       array | a formal and a FILE-SCOPE array | two locals
  C  size              the band GRID-M leaves open, 44..64, plus 72 and 96
  D  optimization      the workload's `/O1` (GRID-M's) and `/Ox` and `/O2`

THE RIVAL UNDER TEST, frozen before compiling
---------------------------------------------
  F-48    constant size >= 48  ->  `bl`;  everything else -> not a `bl`
          (GRID-M's fence, applied to a population it was not fitted on)

and, as the control that says whether this grid can refute anything at all,

  F-ALL   every cell is a `bl`   (already refuted at 118 cells by GRID-M;
          re-scored here so a grid that agreed with BOTH would be visible as
          a grid that discriminates nothing)

Usage:  gridm2.py gen <outdir> | run <outdir> <root>
"""

import hashlib
import json
import os
import struct
import subprocess
import sys

SIZES = [44, 46, 47, 48, 49, 52, 56, 60, 64, 72, 96]

# (tag, C type)  — none of these four is in GRID-M
PTYPES = [("v", "void"), ("q", "long long"), ("s4", "S4"), ("s32", "S32")]

# (tag, how the two operands are obtained)
OPERANDS = ["ff", "fl", "fg", "ll"]

HDR = """// w-memcpy GRID-M2 cell %s
// %s
struct S4 { int a; };
struct S32 { double a[4]; };
extern "C" void *memcpy(void *, const void *, unsigned int);
"""


def cell_source(name, meta, ctype, size, operands):
    ptr = "void" if ctype == "void" else ctype
    decl_g = "static %s garr[64];\n" % ("char" if ptr == "void" else ptr)
    if operands == "ff":
        params = "%s *d, const %s *s" % (ptr, ptr)
        pre = ""
        args = "d, s"
    elif operands == "fl":
        params = "%s *d" % ptr
        pre = "    %s loc[64];\n" % ("char" if ptr == "void" else ptr)
        args = "d, loc"
    elif operands == "fg":
        params = "%s *d" % ptr
        pre = ""
        args = "d, garr"
    else:  # "ll"
        params = "int k"
        pre = ("    %s a[64]; %s b[64]; (void)k;\n"
               % ("char" if ptr == "void" else ptr,
                  "char" if ptr == "void" else ptr))
        args = "a, b"
    return (HDR % (name, json.dumps(meta, sort_keys=True))
            + decl_g
            + "void f(%s) {\n%s    memcpy(%s, %d);\n}\n"
            % (params, pre, args, size))


def build_cells():
    cells = []
    for ptag, ctype in PTYPES:
        for size in SIZES:
            for ops in OPERANDS:
                meta = dict(ptype=ptag, size=size, operands=ops)
                name = "m2_%s_%s_n%d" % (ptag, ops, size)
                cells.append(dict(
                    name=name, ptype=ptag, size=size, operands=ops,
                    pred=dict(**{"F-48": "call" if size >= 48 else "inline",
                                 "F-ALL": "call"}),
                    src=cell_source(name, meta, ctype, size, ops)))
    return cells


def gen(outdir):
    cells = build_cells()
    os.makedirs(outdir, exist_ok=True)
    sep = sum(1 for c in cells if c["pred"]["F-48"] != c["pred"]["F-ALL"])
    assert sep >= 20, \
        "F-48 and F-ALL differ on only %d cells — this grid cannot refute " \
        "the fence" % sep
    # Every axis must be crossed with the fence's own boundary, or the grid
    # tests the fence on one shape and calls it a population.
    for ptag, _ in PTYPES:
        below = {c["size"] for c in cells if c["ptype"] == ptag and c["size"] < 48}
        above = {c["size"] for c in cells if c["ptype"] == ptag and c["size"] >= 48}
        assert below and above, "pointer type %s does not straddle 48" % ptag
    for ops in OPERANDS:
        got = {c["ptype"] for c in cells if c["operands"] == ops}
        assert got == {p[0] for p in PTYPES}, "operand kind %s is not crossed" % ops
    for c in cells:
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
    open(os.path.join(outdir, "manifest.json"), "w").write(
        json.dumps([{k: v for k, v in c.items() if k != "src"} for c in cells],
                   indent=1, sort_keys=True))
    h = hashlib.sha256()
    for c in sorted(cells, key=lambda x: x["name"]):
        h.update(c["name"].encode())
        h.update(c["src"].encode())
    print("cells          %d" % len(cells))
    print("sizes          %s" % SIZES)
    print("pointer types  %s (none of them in GRID-M)" % [p[0] for p in PTYPES])
    print("operand kinds  %s" % OPERANDS)
    print("F-48 vs F-ALL  separated on %d cells" % sep)
    print("sha256         %s" % h.hexdigest())


def run(outdir, root, flags_rel="work/dc3-workload/flags.txt", tag=""):
    sys.path.insert(0, os.path.join(root, "scripts"))
    from gt_dump import Obj
    manifest = json.load(open(os.path.join(outdir, "manifest.json")))
    flags = os.path.join(root, flags_rel)
    c2rs = os.path.join(root, "target/release/c2rs")
    objdir = os.path.join(outdir, "obj" + tag)
    os.makedirs(objdir, exist_ok=True)
    rows = []
    for c in manifest:
        obj = os.path.join(objdir, c["name"] + ".obj")
        if not os.path.exists(obj):
            r = subprocess.run([c2rs, "compile", c["name"] + ".cpp",
                                "--keep-obj", obj, "--flags-file", flags,
                                "--cwd", outdir],
                               capture_output=True, text=True, cwd=outdir)
            if not os.path.exists(obj):
                rows.append(dict(name=c["name"], error=r.stderr.strip()[:200]))
                continue
        o = Obj(open(obj, "rb").read())
        sec = None
        for s in o.sections:
            if not s["name"].startswith(".text"):
                continue
            for sym in o.symbols:
                if sym["sec"] == s["idx"] and sym["type"] == 0x0020 \
                        and sym["name"].startswith("?f@@"):
                    sec = s
                    break
            if sec is not None:
                break
        if sec is None:
            rows.append(dict(name=c["name"], error="no ?f@@ .text"))
            continue
        d = o.raw(sec)
        words = list(struct.unpack(">%dI" % (len(d) // 4), d))
        names = set()
        for va, symidx, ty in o.relocs(sec):
            sym = o.sym_by_index(symidx)
            names.add(sym["name"] if sym else "sym%d" % symidx)
            _ = (va, ty)
        rows.append(dict(name=c["name"], nbytes=len(words) * 4,
                         relocs=sorted(names),
                         verdict="call" if "memcpy" in names else "inline",
                         words=["%08x" % w for w in words]))
    out = os.path.join(outdir, "measured%s.json" % tag)
    open(out, "w").write(json.dumps(rows, indent=1))
    print("measured %d cells -> %s" % (len(rows), out))


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    else:
        run(sys.argv[2], sys.argv[3],
            sys.argv[4] if len(sys.argv) > 4 else "work/dc3-workload/flags.txt",
            sys.argv[5] if len(sys.argv) > 5 else "")

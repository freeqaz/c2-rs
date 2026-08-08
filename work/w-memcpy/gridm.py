#!/usr/bin/env python3
"""w-memcpy GRID-M — IS `memcpy` ALWAYS A `bl`?  (`expr-intrinsic-memcpy`.)

`docs/IL_INTRINSIC_CALL.md` §3 records selector 172 as emitting `b <memcpy>`
(REL24) and §5.4's fail-closed rule as "reject every `0x40`".  §5.1 gives the
mechanism that would make admitting one on the strength of its id a WRONG EMIT
rather than a gap: *the emission depends on the literal argument values, not on
the id*.  §1.3 then records, without following it up, that

    `?t_memset` pushes one [alignment hint]; **[DC3]** `Dir.cpp` fn931 pushes
    `04` instead of `01` when the operands are 4-byte aligned, **and the
    expansion changes with it**.

PREREG P8 registers the consequence as a prediction: **c2 inlines the copy below
some size threshold, and the threshold interacts with the alignment hint**, so a
reader that admitted selector 172 and emitted a `bl` would be wrong on every
cell below it.  This grid settles that before one line of reader code is
written, because the answer decides whether clause B is shippable at all.

STRUCTURAL AXES, crossed
------------------------
  A  size literal      0,1,2,3,4,5,7,8,12,15,16,24,31,32,48,64,72,96,128,
                       256,1024   (72 is `?mmioGetInfo`'s own)
  B  pointer type      char* (hint 1) | int* (hint 4) | double* (hint 8) |
                       a 16-byte struct  — the axis that moves the HINT
  C  size kind         a compile-time constant | a formal (`unsigned n`)
  D  callee            memcpy | memset  (the sibling selector, 173, whose
                       workload footprint is 10x memcpy's)

WHAT IS READ BACK
-----------------
Per cell: the `.text` byte count of `?f@@…`, whether the obj carries a REL24
naming `memcpy`/`memset`, and the decoded word list.  The VERDICT is
`call` | `inline` | `none`, and the rule is scored against it.

THE RIVALS
----------
  M-ALWAYSCALL   every cell is a `bl`  (what §3's table alone would license,
                 and what a reader keyed on the selector id would emit)
  M-THRESH-T     `bl` iff size >= T, inline otherwise, T FROZEN PER RIVAL at
                 T in {8, 16, 32, 64} so no threshold is fitted after the fact
  M-VARCALL      `bl` iff the size is not a compile-time constant

Freezing four separate thresholds rather than one free parameter is the point:
board #260, and `w-mmio` §3's three-fits-two-refutations.  A rule with a free
constant fitted to the grid that produced it is not a measurement.

Usage:
    gridm.py gen  <outdir>          write cells + manifest, print the sha256
    gridm.py run  <outdir> <root>   compile each cell with the real c2, read
                                    the expansion back out of the obj
"""

import hashlib
import json
import os
import struct
import subprocess
import sys

# The band between two adjacent frozen thresholds is what separates them, so
# the size axis is dense between 8 and 64 rather than logarithmic — a sparse
# axis there would let two rivals through on 16 cells and call it a grid.
SIZES = [0, 1, 2, 3, 4, 5, 7, 8, 10, 12, 15, 16, 20, 24, 28, 31, 32, 36, 40,
         44, 48, 56, 64, 72, 96, 128, 256, 1024]

# (tag, C type, the alignment the type forces)
PTYPES = [("c", "char", 1), ("i", "int", 4), ("d", "double", 8),
          ("s", "S16", 16)]

THRESHOLDS = [8, 16, 32, 64]

HDR = """// w-memcpy GRID-M cell %s
// %s
struct S16 { double a; double b; };
extern "C" void *memcpy(void *, const void *, unsigned int);
extern "C" void *memset(void *, int, unsigned int);
"""


def cell_source(name, meta, fn, ptag, ctype, size, varsize):
    body_args = "d, s, n" if varsize else "d, s, %d" % size
    if fn == "memset":
        body_args = "d, 0, n" if varsize else "d, 0, %d" % size
        params = "%s *d, unsigned n" % ctype if varsize else "%s *d" % ctype
    else:
        params = ("%s *d, const %s *s, unsigned n" % (ctype, ctype)
                  if varsize else "%s *d, const %s *s" % (ctype, ctype))
    _ = ptag
    return (HDR % (name, json.dumps(meta, sort_keys=True))
            + "void f(%s) { %s(%s); }\n" % (params, fn, body_args))


def build_cells():
    cells = []
    for fn in ("memcpy", "memset"):
        for ptag, ctype, align in PTYPES:
            for size in SIZES:
                meta = dict(fn=fn, ptype=ptag, align=align, size=size,
                            varsize=False)
                name = "%s_%s_n%d" % (fn, ptag, size)
                cells.append(dict(
                    name=name, fn=fn, ptype=ptag, align=align, size=size,
                    varsize=False,
                    src=cell_source(name, meta, fn, ptag, ctype, size, False)))
            meta = dict(fn=fn, ptype=ptag, align=align, size=None,
                        varsize=True)
            name = "%s_%s_var" % (fn, ptag)
            cells.append(dict(
                name=name, fn=fn, ptype=ptag, align=align, size=None,
                varsize=True,
                src=cell_source(name, meta, fn, ptag, ctype, 0, True)))
    return cells


def predict(c):
    """Every rival's frozen per-cell verdict: 'call' or 'inline'."""
    out = {"M-ALWAYSCALL": "call",
           "M-VARCALL": "call" if c["varsize"] else "inline"}
    for t in THRESHOLDS:
        out["M-THRESH-%d" % t] = (
            "call" if (c["varsize"] or c["size"] >= t) else "inline")
    return out


def gen(outdir):
    cells = build_cells()
    for c in cells:
        c["pred"] = predict(c)
    os.makedirs(outdir, exist_ok=True)

    # ---- the generator asserts its own discrimination -------------------
    n_in = len(cells)
    for a in ("M-ALWAYSCALL", "M-VARCALL", "M-THRESH-8", "M-THRESH-16",
              "M-THRESH-32", "M-THRESH-64"):
        for b in ("M-ALWAYSCALL", "M-VARCALL", "M-THRESH-8", "M-THRESH-16",
                  "M-THRESH-32", "M-THRESH-64"):
            if a >= b:
                continue
            d = sum(1 for c in cells if c["pred"][a] != c["pred"][b])
            assert d >= 20, "rivals %s/%s differ on only %d cells" % (a, b, d)

    # `?mmioGetInfo`'s own cell must be present, and it must be one the
    # ALWAYSCALL and the THRESH rivals AGREE on — i.e. the target function
    # cannot by itself tell them apart.  Said out loud rather than discovered.
    mm = [c for c in cells if c["fn"] == "memcpy" and c["size"] == 72]
    assert mm, "the 72-byte memcpy cell is missing"
    thresh_rivals = ["M-ALWAYSCALL"] + ["M-THRESH-%d" % t for t in THRESHOLDS]
    agree72 = all(len({c["pred"][r] for r in thresh_rivals}) == 1 for c in mm)
    assert agree72, \
        "the 72-byte cells were expected NOT to separate M-ALWAYSCALL from " \
        "any frozen threshold — the target function's own size is above all " \
        "four, so `?mmioGetInfo` cannot tell them apart and the grid must"

    # Both hint families must be present at every size, or the alignment axis
    # is not crossed with the size axis and B is a sample, not an axis.
    for size in SIZES:
        got = {c["ptype"] for c in cells
               if c["fn"] == "memcpy" and c["size"] == size}
        assert got == {p[0] for p in PTYPES}, \
            "size %d is missing pointer types %s" % (size, got)

    for c in cells:
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
    manifest = [{k: v for k, v in c.items() if k != "src"} for c in cells]
    open(os.path.join(outdir, "manifest.json"), "w").write(
        json.dumps(manifest, indent=1, sort_keys=True))

    h = hashlib.sha256()
    for c in sorted(cells, key=lambda x: x["name"]):
        h.update(c["name"].encode())
        h.update(c["src"].encode())
    print("cells        %d" % n_in)
    print("sizes        %s" % SIZES)
    print("hint axis    %s" % [p[0] for p in PTYPES])
    print("rivals       %s" % sorted(cells[0]["pred"]))
    print("72 B cells   unanimous across every rival (the target discriminates "
          "NOTHING here)")
    print("sha256       %s" % h.hexdigest())


# ---------------------------------------------------------------------------


def run(outdir, root):
    sys.path.insert(0, os.path.join(root, "scripts"))
    from gt_dump import Obj

    manifest = json.load(open(os.path.join(outdir, "manifest.json")))
    flags = os.path.join(root, "work/dc3-workload/flags.txt")
    c2rs = os.path.join(root, "target/release/c2rs")
    objdir = os.path.join(outdir, "obj")
    os.makedirs(objdir, exist_ok=True)
    rows = []
    for c in manifest:
        obj = os.path.join(objdir, c["name"] + ".obj")
        if not os.path.exists(obj):
            r = subprocess.run([c2rs, "compile", c["name"] + ".cpp",
                                "--keep-obj", obj,
                                "--flags-file", flags, "--cwd", outdir],
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
        verdict = "call" if (c["fn"] in names) else (
            "inline" if len(words) > 1 else "none")
        rows.append(dict(name=c["name"], nbytes=len(words) * 4,
                         relocs=sorted(names), verdict=verdict,
                         words=["%08x" % w for w in words]))
    open(os.path.join(outdir, "measured.json"), "w").write(json.dumps(rows, indent=1))
    print("measured %d cells -> %s/measured.json" % (len(rows), outdir))


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3])

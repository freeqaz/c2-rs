#!/usr/bin/env python3
"""wb-memcpy GRID-W — the two things the disassembly reading predicts that
w-memcpy's 1,155 already-paid-for cells CANNOT decide.

The reading (docs/whitebox/WB_MEMCPY_FINDINGS.md) says the memcpy/memset
expansion decision is

    align  = max(1, byte at [node+0x38])            (the IL alignment hint)
    n      = size / align                           (64-bit signed division)
    inline iff n <= T,  T = 5 when the global at 0x10c2e310 is 0
                        T = 10 when it is non-zero
    (variable size => always a call, checked before the division)

and that 0x10c2e310 is bit 23 of the option word, set at 0x10b82392.

GRID-M and GRID-M2 compiled 408 memcpy/memset cells at ONE flag set — the dc3
workload's `/O1 /Oi` — so every one of them saw T = 5, and NONE of them can
tell "T = 5" from "T = 5 here".  GRID-M2's own docstring lists optimization as
axis D and **build_cells never implements it**: 4 ptypes x 11 sizes x 4 operand
kinds = 176, all at /O1.  So the flag is untested, and the reading's sharpest
consequence is unmeasured.

Part A tests it.  Part B tests the OTHER thing the reading does not contain:
w-memcpy found 44 cells where the copy vanishes entirely, and the lowering read
here has no such arm — at size 96 / align 8 it would emit a CALL.  So the
elimination is somebody else's pass, and the question is whose rule it is.

    PART A — the threshold flag                     180 cells
      ptype   char* (align 1) | int* (align 4) | double* (align 8)
      size    n*align for n in {4,5,6,9,10,11}  — straddles BOTH thresholds
      callee  memcpy | memset
      flags   O1 | O2 | Ox | O1Ot | O2Os

      W-T5     T is always 5                    (what a lane that only ever
                                                 compiled /O1 would ship)
      W-LEVEL  T follows the /O<n> LEVEL: 5 at /O1, 10 at /O2 and /Ox, and the
               level wins in the mixed sets (/O1 /Ot -> 5, /O2 /Os -> 10)
      W-OT     T follows FAVOR-SPEED: /O1 -> 5, /O2 -> 10, /Ox -> 10,
               /O1 /Ot -> 10, /O2 /Os -> 5                  (THE READING)

    PART B — the elimination                         72 cells
      ptype   char* | int* | double*
      size    16 (inline class at align 8) and 96 (call class at every align)
      shape   ff  two formals                                 (control: live)
              ll  two locals, dst never read afterwards
              lu  two locals, dst PASSED TO AN EXTERN afterwards
              ld  dst a local never read, src a FORMAL
              fl  dst a formal, src a local                   (control: live)
              gl  dst a file-scope static, src a local        (control: live)
      flags   O1 only

      E-LOCALS   both operands are locals => eliminated       (w-memcpy's
                 stated finding, generalized)
      E-DEADDST  the DESTINATION is a non-escaping local that is never read
                 afterwards => eliminated, whatever the source is

      They separate on `lu` (E-LOCALS: gone, E-DEADDST: live) and on `ld`
      (E-LOCALS: live, E-DEADDST: gone).

Every prediction above is written into manifest.json by `gen`, and `gen`
asserts the separations BEFORE a single cl.exe runs.

Usage:  gridw.py gen <outdir> | run <outdir> <root> [flagset] | score <outdir>
"""

import hashlib
import json
import os
import struct
import subprocess
import sys

# ---------------------------------------------------------------- flag sets

FLAGSETS = {
    "O1":   "/nologo /c /Oi /EHsc /GR /O1",
    "O2":   "/nologo /c /Oi /EHsc /GR /O2",
    "Ox":   "/nologo /c /Oi /EHsc /GR /Ox",
    "O1Ot": "/nologo /c /Oi /EHsc /GR /O1 /Ot",
    "O2Os": "/nologo /c /Oi /EHsc /GR /O2 /Os",
}

# what each rival says the threshold T is, per flag set
T_T5 = {k: 5 for k in FLAGSETS}
T_LEVEL = {"O1": 5, "O2": 10, "Ox": 10, "O1Ot": 5, "O2Os": 10}
T_OT = {"O1": 5, "O2": 10, "Ox": 10, "O1Ot": 10, "O2Os": 5}

# (tag, C element type, the alignment the reading expects c1xx to hand c2)
PTYPES = [("c", "char", 1), ("i", "int", 4), ("d", "double", 8)]

NS = [4, 5, 6, 9, 10, 11]

HDR_A = """// wb-memcpy GRID-W/A cell %s
// %s
extern "C" void *memcpy(void *, const void *, unsigned int);
extern "C" void *memset(void *, int, unsigned int);
"""


def cell_a(name, meta, ctype, size, callee):
    if callee == "memcpy":
        body = "    memcpy(d, s, %d);\n" % size
        params = "%s *d, const %s *s" % (ctype, ctype)
    else:
        body = "    memset(d, 0, %d);\n" % size
        params = "%s *d, const %s *s" % (ctype, ctype)
        body = "    (void)s;\n" + body
    return (HDR_A % (name, json.dumps(meta, sort_keys=True))
            + "void f(%s) {\n%s}\n" % (params, body))


HDR_B = """// wb-memcpy GRID-W/B cell %s
// %s
extern "C" void *memcpy(void *, const void *, unsigned int);
extern "C" void sink(void *);
"""


def cell_b(name, meta, ctype, size, shape):
    g = "static %s garr[32];\n" % ctype
    n = 32
    if shape == "ff":
        return (HDR_B % (name, json.dumps(meta, sort_keys=True)) + g
                + "void f(%s *d, const %s *s) {\n    memcpy(d, s, %d);\n}\n"
                % (ctype, ctype, size))
    if shape == "ll":
        return (HDR_B % (name, json.dumps(meta, sort_keys=True)) + g
                + "void f(int k) {\n    %s a[%d]; %s b[%d]; (void)k;\n"
                  "    memcpy(a, b, %d);\n}\n" % (ctype, n, ctype, n, size))
    if shape == "lu":
        return (HDR_B % (name, json.dumps(meta, sort_keys=True)) + g
                + "void f(int k) {\n    %s a[%d]; %s b[%d]; (void)k;\n"
                  "    memcpy(a, b, %d);\n    sink(a);\n}\n"
                % (ctype, n, ctype, n, size))
    if shape == "ld":
        return (HDR_B % (name, json.dumps(meta, sort_keys=True)) + g
                + "void f(const %s *s) {\n    %s a[%d];\n"
                  "    memcpy(a, s, %d);\n}\n" % (ctype, ctype, n, size))
    if shape == "fl":
        return (HDR_B % (name, json.dumps(meta, sort_keys=True)) + g
                + "void f(%s *d) {\n    %s b[%d];\n"
                  "    memcpy(d, b, %d);\n}\n" % (ctype, ctype, n, size))
    # "gl"
    return (HDR_B % (name, json.dumps(meta, sort_keys=True)) + g
            + "void f(int k) {\n    %s b[%d]; (void)k;\n"
              "    memcpy(garr, b, %d);\n}\n" % (ctype, n, size))


B_SHAPES = ["ff", "ll", "lu", "ld", "fl", "gl"]
B_SIZES = [16, 96]

E_LOCALS = {"ff": "live", "ll": "none", "lu": "none",
            "ld": "live", "fl": "live", "gl": "live"}
E_DEADDST = {"ff": "live", "ll": "none", "lu": "live",
             "ld": "none", "fl": "live", "gl": "live"}


def build_cells():
    cells = []
    for ptag, ctype, align in PTYPES:
        for n in NS:
            size = n * align
            for callee in ("memcpy", "memset"):
                for fs in sorted(FLAGSETS):
                    meta = dict(part="A", ptype=ptag, align=align, n=n,
                                size=size, callee=callee, flags=fs)
                    name = "wa_%s_%s_n%d_%s" % (ptag, callee[3:], n, fs)
                    cells.append(dict(
                        name=name, part="A", flags=fs, align=align, n=n,
                        size=size, callee=callee, ptype=ptag,
                        pred={
                            "W-T5": "inline" if n <= T_T5[fs] else "call",
                            "W-LEVEL": "inline" if n <= T_LEVEL[fs] else "call",
                            "W-OT": "inline" if n <= T_OT[fs] else "call",
                        },
                        src=cell_a(name, meta, ctype, size, callee)))
    for ptag, ctype, align in PTYPES:
        for size in B_SIZES:
            for shape in B_SHAPES:
                meta = dict(part="B", ptype=ptag, align=align, size=size,
                            shape=shape, flags="O1")
                name = "wb_%s_%s_n%d" % (ptag, shape, size)
                cells.append(dict(
                    name=name, part="B", flags="O1", align=align, size=size,
                    shape=shape, ptype=ptag, callee="memcpy",
                    pred={"E-LOCALS": E_LOCALS[shape],
                          "E-DEADDST": E_DEADDST[shape]},
                    src=cell_b(name, meta, ctype, size, shape)))
    return cells


def gen(outdir):
    cells = build_cells()
    os.makedirs(outdir, exist_ok=True)
    a = [c for c in cells if c["part"] == "A"]
    b = [c for c in cells if c["part"] == "B"]

    # --- the separations, asserted before anything is compiled -------------
    def sep(rows, x, y):
        return sum(1 for c in rows if c["pred"][x] != c["pred"][y])

    s1 = sep(a, "W-T5", "W-OT")
    s2 = sep(a, "W-LEVEL", "W-OT")
    s3 = sep(a, "W-T5", "W-LEVEL")
    assert s1 >= 20, "W-T5 vs W-OT separated on only %d cells" % s1
    assert s2 >= 20, "W-LEVEL vs W-OT separated on only %d cells" % s2
    assert s3 >= 20, "W-T5 vs W-LEVEL separated on only %d cells" % s3
    s4 = sep(b, "E-LOCALS", "E-DEADDST")
    assert s4 >= 8, "E-LOCALS vs E-DEADDST separated on only %d cells" % s4
    # every flag set must straddle BOTH candidate thresholds, or the grid
    # tests one threshold on one flag set and calls it a population
    for fs in FLAGSETS:
        ns = {c["n"] for c in a if c["flags"] == fs}
        assert min(ns) <= 4 and max(ns) >= 11, "flag set %s does not straddle" % fs
    # and each rival must be discriminated on EVERY pointer type, not just one
    for ptag, _, _ in PTYPES:
        rows = [c for c in a if c["ptype"] == ptag]
        assert sep(rows, "W-T5", "W-OT") >= 4, "ptype %s under-separated" % ptag
        assert sep(rows, "W-LEVEL", "W-OT") >= 4, "ptype %s under-separated" % ptag

    for c in cells:
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
    for fs, fl in FLAGSETS.items():
        open(os.path.join(outdir, "flags_%s.txt" % fs), "w").write(fl + "\n")
    open(os.path.join(outdir, "manifest.json"), "w").write(
        json.dumps([{k: v for k, v in c.items() if k != "src"} for c in cells],
                   indent=1, sort_keys=True))
    h = hashlib.sha256()
    for c in sorted(cells, key=lambda x: x["name"]):
        h.update(c["name"].encode())
        h.update(c["src"].encode())
    print("cells            %d  (A %d, B %d)" % (len(cells), len(a), len(b)))
    print("flag sets        %s" % sorted(FLAGSETS))
    print("A: W-T5 vs W-OT      separated on %d cells" % s1)
    print("A: W-LEVEL vs W-OT   separated on %d cells" % s2)
    print("A: W-T5 vs W-LEVEL   separated on %d cells" % s3)
    print("B: E-LOCALS vs E-DEADDST separated on %d cells" % s4)
    print("sha256           %s" % h.hexdigest())


# ------------------------------------------------------------------- verdict
#
# THREE arms, not two.  w-memcpy's GRID-M2 shipped a verdict function with no
# `none` arm and it reported the fence refuted by an inline expansion at 96
# bytes that was not there at all (board #984's method, its §6.2).  The byte
# count beside the relocation is what separates them, so it is read here first.

def verdict(nbytes, relocs):
    if any(r in ("memcpy", "memset") for r in relocs):
        return "call"
    if nbytes <= 4:
        return "none"
    return "inline"


def run(outdir, root, only_flags=None):
    sys.path.insert(0, os.path.join(root, "scripts"))
    from gt_dump import Obj
    manifest = json.load(open(os.path.join(outdir, "manifest.json")))
    c2rs = os.path.join(root, "target/release/c2rs")
    objdir = os.path.join(outdir, "obj")
    os.makedirs(objdir, exist_ok=True)
    rows = []
    for c in manifest:
        if only_flags and c["flags"] != only_flags:
            continue
        obj = os.path.join(objdir, c["name"] + ".obj")
        flags = os.path.join(outdir, "flags_%s.txt" % c["flags"])
        if not os.path.exists(obj):
            r = subprocess.run([c2rs, "compile", c["name"] + ".cpp",
                                "--keep-obj", obj, "--flags-file", flags,
                                "--cwd", outdir],
                               capture_output=True, text=True, cwd=outdir)
            if not os.path.exists(obj):
                rows.append(dict(name=c["name"], error=r.stderr.strip()[:300]))
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
                         verdict=verdict(len(words) * 4, names),
                         words=["%08x" % w for w in words]))
    tag = "_" + only_flags if only_flags else ""
    out = os.path.join(outdir, "measured%s.json" % tag)
    open(out, "w").write(json.dumps(rows, indent=1))
    print("measured %d cells -> %s" % (len(rows), out))


def score(outdir):
    manifest = {c["name"]: c for c in json.load(
        open(os.path.join(outdir, "manifest.json")))}
    measured = {}
    for fn in sorted(os.listdir(outdir)):
        if fn.startswith("measured") and fn.endswith(".json"):
            for r in json.load(open(os.path.join(outdir, fn))):
                measured[r["name"]] = r
    errs = [r for r in measured.values() if "error" in r]
    print("cells measured   %d   (errors %d)" % (len(measured), len(errs)))
    for r in errs[:5]:
        print("   ERROR %s: %s" % (r["name"], r.get("error")))

    for part, rivals in (("A", ["W-T5", "W-LEVEL", "W-OT"]),
                         ("B", ["E-LOCALS", "E-DEADDST"])):
        rows = [(manifest[n], m) for n, m in measured.items()
                if "error" not in m and manifest[n]["part"] == part]
        if not rows:
            continue
        print("\n=== PART %s — %d graded cells ===" % (part, len(rows)))
        for riv in rivals:
            ok = sum(1 for c, m in rows if c["pred"][riv] == m["verdict"])
            print("   %-10s %4d / %d" % (riv, ok, len(rows)))
        if part == "A":
            print("\n   measured threshold per (flag set, align):")
            seen = {}
            for c, m in rows:
                seen.setdefault((c["flags"], c["align"], c["callee"]), {})[c["n"]] = m["verdict"]
            for k in sorted(seen):
                v = seen[k]
                inl = [n for n in sorted(v) if v[n] == "inline"]
                cal = [n for n in sorted(v) if v[n] == "call"]
                oth = [(n, v[n]) for n in sorted(v) if v[n] not in ("inline", "call")]
                print("     %-6s align %d %-6s  inline n=%s  call n=%s%s"
                      % (k[0], k[1], k[2], inl, cal,
                         ("  other %s" % oth) if oth else ""))
        else:
            print("\n   measured verdict per (shape, size, align):")
            for c, m in sorted(rows, key=lambda x: (x[0]["shape"], x[0]["size"], x[0]["align"])):
                print("     %-3s size %-3d align %d  ->  %-6s (%d B, relocs %s)"
                      % (c["shape"], c["size"], c["align"], m["verdict"],
                         m["nbytes"], m["relocs"]))


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3],
            sys.argv[4] if len(sys.argv) > 4 else None)
    else:
        score(sys.argv[2])

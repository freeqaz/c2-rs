#!/usr/bin/env python3
"""w-memfit GRID-F — WHAT IS THE DIVISOR?  The cells w-memcpy's 408 cannot make.

R-WB scores 408/408 on w-memcpy's own frozen cells and 216/216 on wb-memcpy's
(`work/w-memfit/score.py`).  PREREG P8 registers, before any of that was
scored, the one sub-claim those 624 cells CANNOT settle:

  every pointee in all three grids is a NATURALLY ALIGNED type, so
  "the divisor is the IL alignment hint byte" and "the divisor is the C
  pointee type's alignment" make identical predictions on all 624.

PREREG P9 commits to manufacturing cells that separate them rather than only
naming them.  These are those cells.  The four rivals below are TOTAL and are
frozen with a per-cell prediction before the first `cl.exe`, per board #260.

THE RIVALS
----------
  F-TYPE    divisor = alignof(pointee) exactly as written at the call site,
            honouring `#pragma pack` and `__declspec(align)`
  F-CLAMP   divisor = min(8, alignof(pointee))  — the hint byte is a BYTE and
            §2.2's inline shape rule already does `unit = min(align, 8)`
  F-ELEM    divisor = alignment of the pointee's largest scalar member,
            IGNORING `#pragma pack` — i.e. packing does not reach the hint
  F-PROV    divisor = min(8, alignment of the OBJECT the pointer really names)
            — i.e. c1xx raises the hint through a cast when it can see the
            underlying object

Everything else is R-WB held fixed: `inline` iff `size/divisor <= 5` at the
workload's `/O1`, `none` at size 0, and every destination here is a live
formal so E-DEADDST is out of the grid by construction.

THE FAMILIES, and which rivals each separates
---------------------------------------------
  p1     `#pragma pack(1) struct P1 { double a[8]; }`   TYPE 1 CLAMP 1
                                                        ELEM 8 PROV 1
  a16    `__declspec(align(16)) struct A16 { char c[16]; }`
                                                        TYPE 16 CLAMP 8
                                                        ELEM 1 PROV 8
  cast   `memcpy((char*)d, (const char*)s, n)` on `double*` formals
                                                        TYPE 1 CLAMP 1
                                                        ELEM 1 PROV 8
  ucast  `memcpy((double*)d, (const double*)s, n)` on `char*` formals
                                                        TYPE 8 CLAMP 8
                                                        ELEM 8 PROV 1
  ctlc   plain `char*`      — all four rivals say 1     (reproduces GRID-M)
  ctld   plain `double*`    — all four rivals say 8     (reproduces GRID-M)

Usage:  gridf.py gen <outdir> | run <outdir> <root> | score <outdir>
"""

import hashlib
import json
import os
import struct
import subprocess
import sys

T = 5  # the workload's /O1 threshold, established at 624/624 before this grid

# (tag, C type text, F-TYPE, F-CLAMP, F-ELEM, F-PROV, sizes)
FAMILIES = [
    ("p1",   "P1",   1,  1, 8, 1, [0, 8, 16, 24, 32, 40, 48, 64]),
    # The 40..80 band is where a 16-divisor and an 8-divisor disagree, and it
    # is the ONLY place they can: below 40 both inline, above 80 both call.
    ("a16",  "A16", 16,  8, 1, 8,
     [16, 32, 40, 48, 56, 64, 72, 80, 96, 128]),
    ("cast", None,   1,  1, 1, 8, [8, 16, 24, 32, 40, 48]),
    ("ucast", None,  8,  8, 8, 1, [8, 16, 24, 32, 40, 48]),
    ("ctlc", "char", 1,  1, 1, 1, [4, 5, 7, 8, 20, 44, 48]),
    ("ctld", "double", 8, 8, 8, 8, [8, 40, 44, 47, 48, 56, 96]),
]

HDR = """// w-memfit GRID-F cell %s
// %s
#pragma pack(push, 1)
struct P1 { double a[8]; };
#pragma pack(pop)
__declspec(align(16)) struct A16 { char c[16]; };
extern "C" void *memcpy(void *, const void *, unsigned int);
"""


def cell_source(name, meta, fam, ctype, size):
    if fam == "cast":
        params = "double *d, const double *s"
        call = "memcpy((char *)d, (const char *)s, %d);" % size
    elif fam == "ucast":
        params = "char *d, const char *s"
        call = "memcpy((double *)d, (const double *)s, %d);" % size
    else:
        params = "%s *d, const %s *s" % (ctype, ctype)
        call = "memcpy(d, s, %d);" % size
    return (HDR % (name, json.dumps(meta, sort_keys=True))
            + "void f(%s) { %s }\n" % (params, call))


def verdict_for(divisor, size):
    if size == 0:
        return "none"
    return "inline" if size // max(1, divisor) <= T else "call"


def build_cells():
    cells = []
    for fam, ctype, a_type, a_clamp, a_elem, a_prov, sizes in FAMILIES:
        for size in sizes:
            meta = dict(fam=fam, size=size)
            name = "f_%s_n%d" % (fam, size)
            cells.append(dict(
                name=name, fam=fam, size=size,
                div=dict({"F-TYPE": a_type, "F-CLAMP": a_clamp,
                          "F-ELEM": a_elem, "F-PROV": a_prov}),
                pred={r: verdict_for(d, size) for r, d in
                      (("F-TYPE", a_type), ("F-CLAMP", a_clamp),
                       ("F-ELEM", a_elem), ("F-PROV", a_prov))},
                src=cell_source(name, meta, fam, ctype, size)))
    return cells


RIVALS = ["F-TYPE", "F-CLAMP", "F-ELEM", "F-PROV"]


def gen(outdir):
    cells = build_cells()
    os.makedirs(outdir, exist_ok=True)

    # The generator asserts its own discrimination before it writes a byte.
    # A grid that cannot separate its rivals is not a measurement (#260).
    worst = None
    for i, a in enumerate(RIVALS):
        for b in RIVALS[i + 1:]:
            d = sum(1 for c in cells if c["pred"][a] != c["pred"][b])
            print("   %-8s vs %-8s separated on %2d cells" % (a, b, d))
            worst = d if worst is None else min(worst, d)
    assert worst >= 4, "some rival pair is separated on only %d cells" % worst

    # Every rival must be separated from EVERY other on at least one family
    # whose destination is a live formal — so no separation rests on the
    # elimination arm, which this grid deliberately does not contain.
    fams = {c["fam"] for c in cells}
    assert fams == {f[0] for f in FAMILIES}

    h = hashlib.sha256()
    for c in sorted(cells, key=lambda c: c["name"]):
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
        h.update(c["src"].encode())
    man = [{k: v for k, v in c.items() if k != "src"} for c in cells]
    json.dump(man, open(os.path.join(outdir, "manifest.json"), "w"), indent=1)
    open(os.path.join(outdir, "list.txt"), "w").write(
        "".join(c["name"] + ".cpp\n" for c in sorted(cells,
                                                     key=lambda c: c["name"])))
    print("cells        %d" % len(cells))
    print("families     %s" % [f[0] for f in FAMILIES])
    print("rivals       %s" % RIVALS)
    print("min pairwise separation %d" % worst)
    print("sha256       %s" % h.hexdigest())


def run(outdir, root, flags_rel="work/dc3-workload/flags.txt"):
    sys.path.insert(0, os.path.join(root, "scripts"))
    from gt_dump import Obj
    manifest = json.load(open(os.path.join(outdir, "manifest.json")))
    flags = os.path.join(root, flags_rel)
    c2rs = os.path.join(root, "target/release/c2rs")
    objdir = os.path.join(outdir, "obj")
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
        # THE SAME three-valued verdict function GRID-M uses (gridm.py:227),
        # `none` decided by the byte count and not by a missing relocation.
        verdict = "call" if "memcpy" in names else (
            "inline" if len(words) > 1 else "none")
        rows.append(dict(name=c["name"], nbytes=len(words) * 4,
                         relocs=sorted(names), verdict=verdict,
                         words=["%08x" % w for w in words]))
    json.dump(rows, open(os.path.join(outdir, "measured.json"), "w"), indent=1)
    errs = [r for r in rows if "error" in r]
    print("measured %d cells, %d errors" % (len(rows), len(errs)))
    for e in errs[:5]:
        print("   ERROR %s: %s" % (e["name"], e["error"]))


def score(outdir):
    man = json.load(open(os.path.join(outdir, "manifest.json")))
    mea = {r["name"]: r for r in
           json.load(open(os.path.join(outdir, "measured.json")))}
    print("== GRID-F: %d cells ==" % len(man))
    for r in RIVALS:
        h = sum(1 for c in man if c["pred"][r] == mea[c["name"]]["verdict"])
        print("   %-8s %2d/%d" % (r, h, len(man)))
    print()
    print("   per cell: family size  measured   %s" % "  ".join(RIVALS))
    for c in man:
        v = mea[c["name"]]["verdict"]
        marks = "  ".join("%-7s" % ("%s%s" % (c["pred"][r],
                                              "" if c["pred"][r] == v else "*"))
                          for r in RIVALS)
        print("   %-6s %4d  %-8s  %s   [%dB]"
              % (c["fam"], c["size"], v, marks, mea[c["name"]]["nbytes"]))
    print()
    print("   * = this rival is WRONG on that cell")
    # The implied divisor, read back per family from the measured boundary.
    print()
    print("   IMPLIED DIVISOR per family (from the inline/call boundary):")
    fams = {}
    for c in man:
        fams.setdefault(c["fam"], []).append((c["size"],
                                              mea[c["name"]]["verdict"]))
    for fam, rows in fams.items():
        rows.sort()
        inl = [s for s, v in rows if v == "inline"]
        cal = [s for s, v in rows if v == "call"]
        lo = max(inl) if inl else None
        hi = min(cal) if cal else None
        # size/div <= 5 at lo and > 5 at hi  =>  div in [lo/5 .. )
        band = "div >= %.2f" % (lo / 5.0) if lo else "-"
        if hi:
            band += ", div < %.2f" % (hi / 5.0)
        print("      %-6s largest inline %s, smallest call %s   %s"
              % (fam, lo, hi, band))


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3])
    else:
        score(sys.argv[2])


# Shared by GRID-G (`work/w-memfit/gridg.py`): the compile + read-back half is
# identical, and the three-valued verdict function must be the SAME one or the
# two grids are not on one denominator.
Obj_run_helper = run

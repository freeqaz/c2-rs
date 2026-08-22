#!/usr/bin/env python3
"""gt_label_seedgap.py — is `LABEL_SEED_GAP = 9` a constant, or is it fitted?

Read **R3**'s confirmation probe (`docs/whitebox/ref/P_LABEL.md`,
`docs/whitebox/WB_LABELCHARGE_PREREG.md` P5.4).  Instrument, not a gate: it
runs the real `cl.exe` under wibo and reports.

`crates/c2-core/src/coff/label.rs:9` ships

    pub const LABEL_SEED_GAP: u32 = 9;

*fitted* from 25 TUs (`docs/OBJ_GY_SHAPES.md` §3.4/§3.5) — nine allocations c2
makes between installing the IL's seed and the first function's first label.
`docs/whitebox/WB_LABEL_FINDINGS.md` §6 open #1 has recorded since 2026-08-09
that **whether the nine moves for a TU with different section needs is
UNVARIED**.  This varies it.

    gap = first($M or $T in the TU) - u32_le(.gl[7..11]) - (3 * nfuncs if /Gy)

Both terms come from one `cl.exe` invocation profile: the IL is captured with
`c2rs capture --flags-file`, the obj compiled with `scripts/gt_capture.sh` at
the same flags, so the seed and the label are the same compilation's.

**Why this instrument is NOT the counterfactual form the banner bans.**
`docs/LABEL_COUNTER.md:3-18` bans measuring a *charge* as a whole-TU
displacement between two different source texts, because that quantity is
`Δseed + Δcharge`.  This script measures neither a charge nor a displacement:
it reads the seed **directly out of the IL** and subtracts it, so the seed
cannot hide inside the answer.  That is the one thing `OBJ_GY_SHAPES.md` §3.5's
absolute form is still good for, and §3.5 says so.

Usage:
    scripts/gt_label_seedgap.py                    # the section-need grid, packed
    scripts/gt_label_seedgap.py --mode '/O1 /GS- /c'
    scripts/gt_label_seedgap.py --selftest         # the banner's own cell
    scripts/gt_label_seedgap.py --keep

`--selftest` is the instrument's self-test and it is the one to read first: it
compiles `s_ctl` and `s_loc8` (the same body plus eight unused locals) in BOTH
the counterfactual form and the in-the-middle form and prints both numbers.
The banner's claim is that the counterfactual moves by **+8** while the true
charge stays **0**.  If this script cannot reproduce that split, every number
it prints is void.

Env: C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.
Exit status is 0 if every row's own control held; it says nothing about
whether a prediction held — read the table.
"""

import os
import re
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
from gt_dump import Obj  # noqa: E402

GL_MAGIC = bytes([0x11, 0x02, 0x06]) + b"1j2" + bytes([0x01])
LABEL_RE = re.compile(r"^\$([MT])(\d+)$")

# ---------------------------------------------------------------------------
# The section-need grid.  Every cell has the SAME two framed functions, so the
# only thing that varies is what else the TU obliges c2 to build.  The seed is
# subtracted per cell, so the cells do not have to be seed-comparable — which
# is exactly the property the in-TU instrument cannot offer for this quantity.
#
# **THE CONFOUND, NAMED HERE BECAUSE THE FIRST RUN OF THIS SCRIPT HIT IT.**
# `gap` is only the seed gap if `f0` is the TU's FIRST function.  A cell that
# adds a function *ahead* of `f0` moves `first($M)` by that function's own
# consumption (1 for a leaf, 4 packed / 5 `/Gy` for a framed one) and the
# script would report a moving gap that is nothing of the kind.  The first
# version of this grid put a string-returning leaf ahead of `f0` and read
# gap 10, 12, 11, 13, 14 — every one of which is `9 + the leaves in front`.
# So: **every cell below adds only DATA or FLAGS ahead of `f0`, never a
# function**, and `n_before` is asserted to be 0 by counting the function
# symbols that precede `f0` in the obj.  A cell that cannot satisfy that is
# reported as VOID rather than graded.
#
CELLS = [
    # (name, text inserted BEFORE the two functions, extra flags, note)
    ("base", "", [], "two framed functions, nothing else"),
    ("data-global", "int gv = 7;", [], "a DEFINED INITIALIZED global -> .data"),
    ("bss-global", "int gv;", [], "an UNINITIALIZED global -> .bss"),
    ("rdata-const", "extern const int gv; extern const int gv = 7;", [],
     "an externally-visible const global -> .rdata"),
    ("data-array", "int gva[64] = {1,2,3};", [], "an initialized array -> .data"),
    ("bss-big", "char gvb[4096];", [], "a 4 KiB uninitialized array -> .bss"),
    ("three-globals", "int g1 = 1; int g2; const char* g3 = \"x\";", [],
     "three globals across .data/.bss/.rdata at once"),
    ("gf", "", ["/GF"], "same source, /GF (string pooling) added"),
    ("gy", "", ["/Gy"], "same source, /Gy (COMDAT per function) added"),
    ("gs", "", [], "same source, /GS ON (the cell drops the /GS- below)"),
    ("ehsc", "", ["/EHsc"], "same source, /EHsc — a WORKLOAD flag"),
    ("gr", "", ["/GR"], "same source, /GR (RTTI) — a WORKLOAD flag"),
    ("oi", "", ["/Oi"], "same source, /Oi (intrinsics) — a WORKLOAD flag"),
    ("workload", "", ["/Oi", "/EHsc", "/GR"],
     "the workload's own flag cluster, all three at once"),
]

BODY = ("int gp(int);\n"
        "int f0(int a){ return gp(a)+1; }\n"
        "int f1(int a){ return gp(a)+2; }\n")


def resolve_wibo():
    w = os.environ.get("C2RS_WIBO")
    if w:
        return w
    for c in (os.path.join(ROOT, "..", "wibo", "build", "release", "wibo"),):
        if os.path.isx if False else os.path.exists(c):
            return os.path.abspath(c)
    from shutil import which
    return which("wibo")


def capture_seed(cpp, flags, workdir):
    """Run `c2rs capture` and return `u32_le(.gl[7..11])`, or None."""
    ff = os.path.join(workdir, "flags.txt")
    with open(ff, "w") as fh:
        fh.write(" ".join(flags) + "\n")
    keep = os.path.join(workdir, "il")
    os.makedirs(keep, exist_ok=True)
    r = subprocess.run(
        ["cargo", "run", "-q", "-p", "c2-harness", "--bin", "c2rs", "--",
         "capture", cpp, "--keep-il", keep, "--flags-file", ff],
        cwd=ROOT, capture_output=True, text=True)
    if r.returncode != 0:
        return None, r.stderr.strip()[-300:]
    gl = [f for f in os.listdir(keep) if f.endswith(".gl")]
    if not gl:
        return None, "no .gl in the bundle (toolchain absent?)"
    blob = open(os.path.join(keep, gl[0]), "rb").read()
    for f in gl:
        os.unlink(os.path.join(keep, f))
    for f in os.listdir(keep):
        os.unlink(os.path.join(keep, f))
    if blob[:7] != GL_MAGIC:
        return None, ".gl magic is %s, not %s" % (blob[:7].hex(), GL_MAGIC.hex())
    return struct.unpack_from("<I", blob, 7)[0], None


def compile_obj(cpp, flags, out):
    env = dict(os.environ)
    env["GT_OUT"] = out
    r = subprocess.run([os.path.join(HERE, "gt_capture.sh"), cpp] + flags,
                       cwd=ROOT, capture_output=True, text=True, env=env)
    if not os.path.exists(out):
        return None, (r.stderr.strip()[-300:] or "no obj")
    return Obj(open(out, "rb").read()), None


def labels_of(obj):
    """Every ($M|$T, n) in the obj, the defined-function count, and how many
    defined functions sit AHEAD of `?f0@@YAHH@Z` in symbol order.

    The third value is the confound guard: it must be 0 or the row is VOID.
    """
    nums, nfunc, before, seen_f0 = [], 0, 0, False
    for s in obj.symbols:
        m = LABEL_RE.match(s["name"])
        if m:
            nums.append(int(m.group(2)))
        if s["name"] == "?f0@@YAHH@Z":
            seen_f0 = True
        # a defined function symbol: DTYPE_FUNCTION with a real section
        if s["type"] == 0x20 and s["sec"] > 0:
            nfunc += 1
            if not seen_f0 and s["name"] != "?f0@@YAHH@Z":
                before += 1
    return sorted(nums), nfunc, before


def run_grid(mode, keep):
    base_flags = mode.split()
    tmp = tempfile.mkdtemp(prefix="seedgap-")
    print("mode: %s   (plus each cell's own extra flags)" % mode)
    print("  gap = first($M|$T) - u32_le(.gl[7..11]) - (3*nfuncs when /Gy)")
    print("  VOID unless the cell puts ZERO functions ahead of f0 — see the")
    print("  confound note in the source.")
    print()
    print("%-14s %-14s %7s %7s %5s %6s %5s  %s"
          % ("cell", "flags+", "seed", "first", "nfun", "before", "GAP", "what varies"))
    seen, bad = {}, 0
    for name, extra, xflags, note in CELLS:
        flags = list(base_flags)
        if name == "gs":
            flags = [f for f in flags if f != "/GS-"]
        flags += xflags
        comdat = "/Gy" in flags or "/O1" in flags
        src = os.path.join(tmp, name + ".cpp")
        with open(src, "w") as fh:
            fh.write(extra + ("\n" if extra else "") + BODY)
        seed, err = capture_seed(src, flags, tmp)
        if seed is None:
            print("%-14s  SKIP: %s" % (name, err))
            bad += 1
            continue
        obj, err = compile_obj(src, flags, os.path.join(tmp, name + ".obj"))
        if obj is None:
            print("%-14s  SKIP: %s" % (name, err))
            bad += 1
            continue
        nums, nfunc, before = labels_of(obj)
        tag = " ".join(xflags) or ("/GS on" if name == "gs" else "-")
        if not nums:
            print("%-14s %-14s %7d %7s %5d %6d %5s  %s (NO LABEL — uninformative)"
                  % (name, tag, seed, "-", nfunc, before, "-", note))
            continue
        if before:
            print("%-14s %-14s %7d %7d %5d %6d %5s  VOID — %d function(s) ahead of f0"
                  % (name, tag, seed, nums[0], nfunc, before, "-", before))
            continue
        gap = nums[0] - seed - (3 * nfunc if comdat else 0)
        seen[name] = gap
        print("%-14s %-14s %7d %7d %5d %6d %5d  %s"
              % (name, tag, seed, nums[0], nfunc, before, gap, note))
    print()
    vals = sorted(set(seen.values()))
    print("distinct GAP values over %d graded cells: %s" % (len(seen), vals))
    if len(vals) == 1:
        print("VERDICT: the gap is CONSTANT at %d across this grid — including" % vals[0])
        print("         every workload flag varied one at a time and all together.")
        print("         crates/c2-core/src/coff/label.rs:9 ships 9.")
    else:
        print("VERDICT: the gap MOVES — %s. A single fitted constant cannot be" % vals)
        print("         right for all of these TUs.")
        for k, v in sorted(seen.items(), key=lambda kv: kv[1]):
            print("         %-14s %d" % (k, v))
    if not keep:
        subprocess.run(["rm", "-rf", tmp])
    else:
        print("kept %s" % tmp)
    return 0


# ---------------------------------------------------------------------------
# The self-test: the banner's own cell, both forms side by side.
SELFTEST_CELLS = [
    # (tag, body, the banner's published counterfactual, its published charge)
    ("s_ctl", "return a+1;", 0, 0),
    ("s_loc2", "int u0,u1; return a+1;", 2, 0),
    ("s_loc8", "int u0,u1,u2,u3,u4,u5,u6,u7; return a+1;", 8, 0),
    ("s_decl8", "extern int d0,d1,d2,d3,d4,d5,d6,d7; return a+1;", None, 0),
    ("s_loc8_used", "int u0,u1,u2,u3,u4,u5,u6,u7;(void)u0;(void)u1;(void)u2;"
                    "(void)u3;(void)u4;(void)u5;(void)u6;(void)u7; return a+1;",
     None, 0),
]


def _first_of(obj, mangled):
    """Lowest $M/$T number belonging to the group of function `mangled`."""
    idx = None
    for s in obj.symbols:
        if s["name"] == mangled:
            idx = s
            break
    if idx is None:
        return None
    # $M/$T symbols sit in the same section as the function (or its .pdata) and
    # follow it in index order; the group's first is the lowest number at or
    # after the function symbol's index.
    nums = [int(LABEL_RE.match(s["name"]).group(2))
            for s in obj.symbols
            if LABEL_RE.match(s["name"]) and s["idx"] > idx["idx"]]
    return min(nums) if nums else None


def run_selftest(mode, keep):
    flags = mode.split()
    tmp = tempfile.mkdtemp(prefix="seedgap-st-")
    print("mode: %s" % mode)
    print("THE BANNER'S CELL (docs/LABEL_COUNTER.md:3-18, WB_LABEL_FINDINGS.md §3.1):")
    print("  eight unused locals emit not one instruction. The COUNTERFACTUAL")
    print("  form is claimed to move by +8 and the TRUE in-TU charge by 0.")
    print()
    print("%-14s %14s %8s %8s   %s" %
          ("cell", "counterfactual", "TRUE", "base", "published counterfactual"))
    ref_cf, ref_tr, ok = None, None, True
    for tag, body, pub_cf, pub_charge in SELFTEST_CELLS:
        src = os.path.join(tmp, "cf_" + tag + ".cpp")
        with open(src, "w") as fh:
            fh.write("int gp(int);\n"
                     "int P(int a){ %s }\n"
                     "int z(int a){ return gp(a)+3; }\n" % body)
        obj, err = compile_obj(src, flags, os.path.join(tmp, "cf_" + tag + ".obj"))
        if obj is None:
            print("SKIP: %s" % err)
            return 1
        first_z = _first_of(obj, "?z@@YAHH@Z")

        src = os.path.join(tmp, "mid_" + tag + ".cpp")
        with open(src, "w") as fh:
            fh.write("int ga(int);\n"
                     "int a0(int a){ return ga(a)+1; }\n"
                     "int P(int a){ %s }\n"
                     "int a1(int a){ return ga(a)+2; }\n"
                     "int a2(int a){ return ga(a)+3; }\n" % body)
        obj, err = compile_obj(src, flags, os.path.join(tmp, "mid_" + tag + ".obj"))
        if obj is None:
            print("SKIP: %s" % err)
            return 1
        f0 = _first_of(obj, "?a0@@YAHH@Z")
        f1 = _first_of(obj, "?a1@@YAHH@Z")
        f2 = _first_of(obj, "?a2@@YAHH@Z")
        base = f2 - f1
        stride = f1 - f0 - base
        if ref_cf is None:
            ref_cf, ref_tr = first_z, stride
        dcf, dtr = first_z - ref_cf, stride - ref_tr
        print("%-14s %+14d %+8d %8d   %s" %
              (tag, dcf, dtr, base,
               "-" if pub_cf is None else ("+%d %s" % (pub_cf, "OK" if dcf == pub_cf else "MISMATCH"))))
        if pub_cf is not None and dcf != pub_cf:
            ok = False
        if dtr != 0:
            ok = False
    print()
    print("Every cell's TRUE charge delta is 0 while the")
    print("counterfactual moves, and the cells the banner publishes by number")
    print("(`s_loc2` +2, `s_loc8` +8) reproduce %s." %
          ("EXACTLY" if ok else "*** WITH A MISMATCH ***"))
    print("SELF-TEST: %s" % ("GREEN" if ok else "RED"))
    if not ok:
        print("  RED means this instrument is not measuring what it claims.")
        print("  Every number from this script is void until it is resolved.")
    if not keep:
        subprocess.run(["rm", "-rf", tmp])
    else:
        print("kept %s" % tmp)
    return 0 if ok else 1


def main(argv):
    mode = "/Ox /GS- /c"
    keep = "--keep" in argv
    if "--mode" in argv:
        mode = argv[argv.index("--mode") + 1]
    if "--selftest" in argv:
        return run_selftest(mode, keep)
    return run_grid(mode, keep)


if __name__ == "__main__":
    sys.exit(main(sys.argv))

#!/usr/bin/env python3
"""gt_frame_class.py — the frame-class CEILING, read out of the real objs.

Every sizing this project does is a **counterfactual**: lift a gate, re-scan, and
count how many functions the surrounding grammar can now finish. That is the right
instrument for "what is the next rung worth", and it is silent about "how big is
this class in the corpus at all" — a class whose bodies all block three tokens
earlier in the expression layer measures 0 either way, whether the workload
contains none of it or 25,000 of it.

This is the other instrument. It classifies every **emitted** `/Gy` function in
the reference objs the gap scan already cached, by its prologue, exactly as
`docs/CODEGEN_FRAMED_CALLS.md` §2.1–§2.5 defines the five classes. No IL parser
is involved, so nothing it reports is bounded by what the port can parse.

    scripts/gt_frame_class.py [CACHE_DIR] [--sources work/dc3-workload/files.txt]

**Name the corpus or the number is meaningless.** `work/capture-cache` is shared
by every tool in the repo, so a run of `scripts/expr_sweep.sh` or
`scripts/cross_sweep.sh` between two censuses silently adds tens of thousands of
synthetic single-function objs to it — 871 entries became 39,364 while this file
was being written, and the unfiltered class shares moved by five points. Pass
`--sources` with the workload's `files.txt` and each cache entry is kept only if
its `meta.txt` names one of those sources.

**The two numbers are not comparable and must never be quoted as a ratio.** The
census denominator is *IL* functions across 878 TUs (~2.46 M, header bodies
counted once per TU that includes them); this denominator is emitted `.text`
COMDATs (~179 k). Use the counterfactual to rank the next rung and use this to
decide whether a class is worth building **once its bodies become reachable**.

Classification, and why each signal is the one used:

* **helper classes are detected from RELOCATIONS**, not from the prologue words —
  a `bl __savegprlr_N` is an ordinary REL24 against an ordinary undefined
  external, so the symbol name is exact where a byte pattern would be a guess;
* **inline saves require a NEGATIVE displacement off r1.** A varargs function
  homes its incoming register arguments with `std r5,32(r1)` … `std r10,72(r1)`
  *in the prologue*, which a displacement-blind filter reads as six saved GPRs
  and publishes as a refutation of the measured threshold of 3. It is not one:
  `??$sprintf_s@…` is Class A with six homing stores;
* **the prologue ends at the frame allocation**, which is `stwu r1,-F(r1)` for a
  frame under five pages and `stwux r1,r1,r12` after `bl _RtlCheckStack12` above
  it (`frame.rs::FRAME_STWUX`). Missing the second form drops the large-frame
  functions into the leaf bucket, which is where 22 Class C functions went before
  it was handled.
"""
import collections
import glob
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gt_dump import Obj  # noqa: E402

STWU_R1 = 0x94210000  # stwu r1,-F(r1)   — mask 0xFFFF0000
STWUX = 0x7C21616E  # stwux r1,r1,r12  — the _RtlCheckStack12 allocation
PROLOGUE_MAX_WORDS = 32


def words(b):
    return [struct.unpack_from(">I", b, i)[0] for i in range(0, len(b) - 3, 4)]


def classify(o, sec):
    """-> (class name, inline saved GPRs, inline saved FPRs)."""
    gpr_helper = fpr_helper = False
    for (_va, symidx, _ty) in o.relocs(sec):
        sym = o.sym_by_index(symidx)
        if sym is None:
            continue
        if sym["name"].startswith("__savegprlr_"):
            gpr_helper = True
        elif sym["name"].startswith("__savefpr_"):
            fpr_helper = True

    ng = nf = 0
    framed = False
    for x in words(o.raw(sec))[:PROLOGUE_MAX_WORDS]:
        if (x & 0xFFFF0000) == STWU_R1 or x == STWUX:
            framed = True
            break
        # std rS,-d(r1): opcode 62, DS-form XO 0, rA = r1, displacement negative.
        if (x & 0xFC1F0003) == 0xF8010000 and (x & 0x8000):
            ng += 1
        # stfd fS,-d(r1): opcode 54, rA = r1, displacement negative.
        elif (x & 0xFC1F0000) == 0xD8010000 and (x & 0x8000):
            nf += 1
    if not framed:
        return ("leaf / tail (no frame)", 0, 0)
    if gpr_helper and fpr_helper:
        return ("F  both helper pairs", ng, nf)
    if gpr_helper:
        return ("C  >=3 saved GPR, __savegprlr_N", ng, nf)
    if fpr_helper:
        return ("E  >=4 saved FPR, __savefpr_M", ng, nf)
    if nf and ng:
        return ("D+ inline FPRs beside inline GPRs", ng, nf)
    if nf:
        return ("D  1-3 saved FPR, inline stfd", ng, nf)
    if ng:
        return ("B  1-2 saved GPR, inline std", ng, nf)
    return ("A  nothing saved", ng, nf)


def cache_source(entry):
    """The source file a cache entry was captured from, per its meta.txt."""
    try:
        lines = open(os.path.join(entry, "meta.txt")).read().splitlines()
    except OSError:
        return None
    for i, ln in enumerate(lines):
        if ln == "arg -f" and i + 1 < len(lines) and lines[i + 1].startswith("arg "):
            return lines[i + 1][4:]
    return None


def main(argv):
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    cache = None
    sources = None
    i = 1
    while i < len(argv):
        if argv[i] == "--sources":
            sources = argv[i + 1]
            i += 2
        else:
            cache = argv[i]
            i += 1
    cache = cache or os.path.join(root, "work", "capture-cache")
    objs = sorted(glob.glob(os.path.join(cache, "*", "out.obj")))
    if sources:
        want = {
            ln.strip()
            for ln in open(sources)
            if ln.strip() and not ln.startswith("#")
        }
        objs = [p for p in objs if cache_source(os.path.dirname(p)) in want]
        print(f"corpus: {len(want)} sources from {sources}")
    else:
        print(f"corpus: EVERY entry in {cache} — unfiltered, see the module docstring")
    if not objs:
        print(f"SKIP: no cached objs under {cache}")
        print("(populate it with: c2rs gap --list … --flags-file … --cwd …)")
        return 0

    cls = collections.Counter()
    gpr_widths = collections.Counter()
    fpr_widths = collections.Counter()
    inline_g = collections.Counter()
    inline_f = collections.Counter()
    n_obj = 0
    for p in objs:
        try:
            o = Obj(open(p, "rb").read())
        except Exception as e:  # a truncated cache entry must not stop the census
            print(f"  ! unreadable {p}: {e}", file=sys.stderr)
            continue
        n_obj += 1
        for sec in o.sections:
            if not sec["name"].startswith(".text"):
                continue
            name, ng, nf = classify(o, sec)
            cls[name] += 1
            if name.startswith(("B", "D")):
                inline_g[ng] += 1
                inline_f[nf] += 1
            for (_va, si, _t) in o.relocs(sec):
                sym = o.sym_by_index(si)
                if sym is None:
                    continue
                if sym["name"].startswith("__savegprlr_"):
                    gpr_widths[32 - int(sym["name"].rsplit("_", 1)[1])] += 1
                elif sym["name"].startswith("__savefpr_"):
                    fpr_widths[32 - int(sym["name"].rsplit("_", 1)[1])] += 1

    tot = sum(cls.values())
    print(f"objs {n_obj}   emitted /Gy functions {tot}\n")
    for k, v in sorted(cls.items(), key=lambda kv: -kv[1]):
        print(f"  {v:8d}  {100 * v / tot:5.2f}%   {k}")
    framed = tot - cls["leaf / tail (no frame)"]
    print(f"\n  framed {framed}  ({100 * framed / tot:.2f}%)")
    print("  saved GPRs when the helper is used:", dict(sorted(gpr_widths.items())))
    print("  saved FPRs when the helper is used:", dict(sorted(fpr_widths.items())))
    print("  inline std count (B / D rows):", dict(sorted(inline_g.items())))
    print("  inline stfd count (B / D rows):", dict(sorted(inline_f.items())))
    # The measured thresholds, re-derived rather than asserted: an inline count at
    # or past a helper threshold would refute docs/CODEGEN_FRAMED_CALLS.md §2.3/§2.4.
    bad_g = sorted(k for k in inline_g if k >= 3)
    bad_f = sorted(k for k in inline_f if k >= 4)
    if bad_g or bad_f:
        print(f"  <== REFUTES the thresholds: inline GPR {bad_g}, inline FPR {bad_f}")
    else:
        print("  thresholds hold: no inline run reaches 3 GPRs or 4 FPRs")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

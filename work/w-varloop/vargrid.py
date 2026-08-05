#!/usr/bin/env python3
"""vargrid.py — **grade the body-parameterized lowering against real `c2`.**

Lane **w-varloop**. Control: `work/w-varloop/PREREG.md`, committed at `e6ab626`
**before this file existed** and before any file under `crates/` was touched.

# What this grades

Not positions, and not a reconstruction. Each cell is compiled by the **real
`c2.dll` under wibo** and by `PortC2`, and the two **whole objs** are compared
byte for byte with the COFF `TimeDateStamp` zeroed. The verdict is `c2rs gap`'s
own, which is the project's sole judge:

    match         the port's obj is byte-identical to c2's
    mismatch      the port emitted something and it was WRONG      <- the alarm
    codegen-gap   the port refused after the IL decoded
    vocab-gap     the IL did not decode

A must-refuse cell passing means `codegen-gap` or `vocab-gap`. **`mismatch` is
never a pass for any cell in this file.**

# The counters

`reached` and `graded` are separate and both are printed even when equal. A cell
whose capture or compile fails is a **FAILURE** with its own counter, never a
zero and never an absence. Every excluded cell prints its reason.

Usage:
    work/w-varloop/vargrid.py                 # every grid
    work/w-varloop/vargrid.py --grid A B      # named grids only
    work/w-varloop/vargrid.py --mode '/Ox /GS- /c'
    work/w-varloop/vargrid.py --jobs 8

Exit status is non-zero when a CONTROL fails or any cell reads `mismatch`.
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))


def src_of(body, sig="const char* s", init="0", decl="", extra=""):
    return (decl + "int P(%s){ int r=%s; while (*s) { int c=*s; %s s++; } "
            "return r; }\n%s" % (sig, init, body, extra))


# ---------------------------------------------------------------------------
# GRID A — the LENGTH axis.  The rules were developed against M = 1..3 only
# (`crates/c2-core/src/codegen/ptr_walk_chain_loop.rs`'s three word-for-word
# tests), so everything from M = 4 up is HELD OUT in the sense PREREG §2's V4
# requires: a length strictly greater than every one used to write the emitter.
#
# P3 in w-sched2 fitted three published cells exactly and died at N = 5.  This
# axis exists so that the same thing, if it happens here, happens in public.
# ---------------------------------------------------------------------------
ALU = ["r=r+c;", "r=r^3;", "r=r+5;", "r=r|9;",
       "r=r+11;", "r=r^13;", "r=r+17;", "r=r|19;", "r=r+23;", "r=r^29;"]


def grid_a():
    cells = []
    for n in range(1, 11):
        cells.append(("a-len%d" % n, src_of(" ".join(ALU[:n])), "match",
                      "held out at n>=4" if n >= 4 else "fitting set"))
    return cells


# ---------------------------------------------------------------------------
# GRID B — the OPERATOR axis, crossed with position.  Four admitted operators
# in both operand shapes.  `mul` is the one whose literal set is narrowed by a
# separate predicate, so it gets both a legal and an illegal constant.
# ---------------------------------------------------------------------------
def grid_b():
    return [
        ("b-mul-lit",    src_of("r=r+c; r=r*7; r=r+5;"),          "match", ""),
        ("b-mul-127",    src_of("r=r+c; r=r*127;"),               "match", ""),
        ("b-or-lit",     src_of("r=r+c; r=r|9; r=r^3;"),          "match", ""),
        ("b-xor-lit",    src_of("r=r+c; r=r^255;"),               "match", ""),
        ("b-add-neg",    src_of("r=r+c; r=r+-5;"),                "match", ""),
        ("b-char-add",   src_of("r=r+c;"),                        "match", ""),
        ("b-char-xor",   src_of("r=r+c; r=r^c;"),                 "match", ""),
        ("b-char-or",    src_of("r=r+c; r=r|c;"),                 "match", ""),
        ("b-char-mul",   src_of("r=r+c; r=r*c;"),                 "match", ""),
        ("b-char-every", src_of("r=r+c; r=r^c; r=r|c; r=r*c;"),   "match", ""),
        # The uimm16 / simm16 boundary: two ranges because two encodings.
        ("b-xor-ffff",   src_of("r=r+c; r=r^65535;"),             "match", ""),
        ("b-add-32767",  src_of("r=r+c; r=r+32767;"),             "match", ""),
    ]


# ---------------------------------------------------------------------------
# GRID C — the REGIME axis.  S3m is `pv == 0 AND M >= 4`, and the conjunction is
# what w-sched2's S3 (stated over N) got wrong at 125 of 131.  Both poles are
# driven past the threshold: `c-first` flips to SAME at M = 4, `c-last` must NOT.
# ---------------------------------------------------------------------------
C_LAST = ["r=r^3;", "r=r+5;", "r=r|9;", "r=r+11;",
          "r=r^13;", "r=r+17;", "r=r|19;", "r=r+c;"]


def grid_c():
    cells = []
    for n in range(2, 9):
        cells.append(("c-last%d" % n,
                      src_of(" ".join(C_LAST[len(C_LAST) - n:])), "match",
                      "pv = M-1, must stay TWO"))
    for n in range(3, 7):
        # `c` in the middle: pv is neither 0 nor M-1.
        ops = ["r=r^3;"] + ["r=r+c;"] + ALU[1:n]
        cells.append(("c-mid%d" % n, src_of(" ".join(ops[:n])), "match",
                      "pv interior"))
    return cells


# ---------------------------------------------------------------------------
# GRID D — the SIGNEDNESS axis, and the K0 axis.  Both regimes, because the
# unsigned record form is DIFFERENT in each (`mr.` in TWO, `cmplwi` in SAME) and
# that is the one fact w-sched2's reconstruction never had to derive.
# ---------------------------------------------------------------------------
UC = "const unsigned char* s"


def grid_d():
    return [
        ("d-u-1",     src_of("r=r+c;", sig=UC),                       "match", "TWO"),
        ("d-u-2",     src_of("r=r+c; r=r^3;", sig=UC),                "match", "TWO"),
        ("d-u-3",     src_of("r=r+c; r=r^3; r=r+5;", sig=UC),         "match", "TWO"),
        ("d-u-4",     src_of("r=r+c; r=r^3; r=r+5; r=r|9;", sig=UC),  "match", "SAME"),
        ("d-u-6",     src_of(" ".join(ALU[:6]), sig=UC),              "match", "SAME"),
        ("d-u-last",  src_of(" ".join(C_LAST[4:]), sig=UC),           "match", "TWO at M=4"),
        ("d-k0-1234", src_of("r=r+c; r=r^3;", init="1234"),           "match", ""),
        ("d-k0-neg",  src_of("r=r+c; r=r^3;", init="-1"),             "match", ""),
        ("d-k0-max",  src_of("r=r+c; r=r^3;", init="32767"),          "match", ""),
        ("d-k0-4",    src_of(" ".join(ALU[:4]), init="7"),            "match", "SAME + K0"),
    ]


# ---------------------------------------------------------------------------
# GRID E — the MUST-REFUSE cells.  Every one of these is a measured
# counterexample, not a conservative guess, and each must come back
# `codegen-gap` or `vocab-gap`.  A `mismatch` here is the failure the whole
# correctness rule exists to forbid.
# ---------------------------------------------------------------------------
def grid_e():
    return [
        # `&` selects `andi.`, which WRITES cr0: c2 demotes the record form to a
        # plain `extsb` and adds an explicit `cmpwi` — a different, longer block.
        ("e-and",      src_of("r=r+c; r=r&21;"),                   "refuse", "andi. writes cr0"),
        # `subf` is non-commutative; S5 does not speak for its operand roles.
        ("e-sub-char", src_of("r=r+c; r=r-c;"),                    "refuse", "subf"),
        # A shift reassociates and folds the length axis.
        ("e-shift",    src_of("r=r+c; r=(r<<2);"),                 "refuse", "rlwinm"),
        # `/` and `%` are w-divmod's spine and PtrWalkModLoop's, not this class's.
        ("e-mod",      src_of("r=r+c; r=r%7;"),                    "refuse", "the % spine"),
        ("e-div",      src_of("r=r+c; r=r/7;"),                    "refuse", "the / spine"),
        # A chain that never reads the character: `pv` undefined, and every rule
        # here is stated in terms of it.
        ("e-no-char",  src_of("r=r+1; r=r^3;"),                    "refuse", "pv undefined"),
        # A `mulli`-ineligible literal: c2 strength-reduces to `rlwinm`.
        ("e-mul-pow2", src_of("r=r+c; r=r*8;"),                    "refuse", "rlwinm"),
        ("e-mul-neg",  src_of("r=r+c; r=r*-3;"),                   "refuse", "x - a*3"),
        ("e-mul-wide", src_of("r=r+c; r=r*100000;"),               "refuse", "lis/ori/mullw"),
        # Literals past their immediate field: a split producer, board #644.
        ("e-add-wide", src_of("r=r+c; r=r+40000;"),                "refuse", "#644 split"),
        # The pointer off slot 0 re-plans the whole block (w-hash's `swap`).
        ("e-p1",       src_of("r=r+c;", sig="int k,const char* s"), "refuse", "pointer off slot 0"),
        # A second formal, even with the pointer first.
        ("e-p2",       src_of("r=r+c;", sig="const char* s,int k"), "refuse", "arity"),
        # A wider element: `lhz`/`lwz`, and a different stride in the update form.
        ("e-short",    src_of("r=r+c;", sig="const short* s"),      "refuse", "wider element"),
        # An accumulator init outside simm16 is a `lis`/`ori` pair.
        ("e-k0-wide",  src_of("r=r+c;", init="70000"),              "refuse", "wide li"),
        # A stride other than 1 does not fold into `lbzu`.
        ("e-stride2",  "int P(const char* s){ int r=0; while (*s) { int c=*s; r=r+c; s+=2; } "
                       "return r; }\n",                             "refuse", "stride != 1"),
        # `*s != 0` is a DIFFERENT IL production from `*s` — a `2C` widening, a
        # `33 <int> 00` and a relational opcode this grammar does not carry.
        ("e-ne0",      "int P(const char* s){ int r=0; while (*s != 0) { int c=*s; r=r+c; s++; } "
                       "return r; }\n",                             "refuse", "different test IL"),
    ]


# ---------------------------------------------------------------------------
# GRID F — **BOARD #747**, and this lane's own obligation.  Two loops of
# DIFFERENT lengths in ONE TU.  w-sched2 demonstrated the shape and left the
# port-side discharge to whichever lane built the lowering; this is that lane.
#
# Neither `scripts/expr_sweep.sh` (single-function TUs) nor
# `scripts/mode_cross.sh` (that corpus crossed with the lane registry) can
# produce this, so both would grade a one-length schedule GREEN.
#
# `f-same` is the CONTROL: two loops of the SAME length. If a mutation that
# fixes the body length turned `f-same` red too, the mutation would not have
# isolated #747's shape and the rung has to say so.
# ---------------------------------------------------------------------------
def two_fn(b1, b2):
    return ("int P(const char* s){ int r=0; while (*s) { int c=*s; %s s++; } return r; }\n"
            "int Q(const char* s){ int r=0; while (*s) { int c=*s; %s s++; } return r; }\n"
            % (b1, b2))


def grid_f():
    return [
        ("f-1-3",  two_fn(" ".join(ALU[:1]), " ".join(ALU[:3])), "match", "TWO / TWO"),
        ("f-2-6",  two_fn(" ".join(ALU[:2]), " ".join(ALU[:6])), "match", "TWO / SAME"),
        ("f-3-8",  two_fn(" ".join(ALU[:3]), " ".join(ALU[:8])), "match", "TWO / SAME"),
        ("f-4-1",  two_fn(" ".join(ALU[:4]), " ".join(ALU[:1])), "match", "SAME / TWO"),
        ("f-same", two_fn(" ".join(ALU[:3]), " ".join(ALU[:3])), "match", "CONTROL"),
    ]


# ---------------------------------------------------------------------------
# GRID G — the KNOWN-ANSWER controls.  A grid that only ever grades its own
# family cannot tell "the port is right" from "the harness is not looking".
# ---------------------------------------------------------------------------
def grid_g():
    return [
        # A body with no loop at all, already matched by the port for a long
        # time. If this reads anything but `match`, the harness is broken and
        # nothing else in this file means anything.
        ("g-noloop", "int P(int a){ return a+1; }\n", "match", "CONTROL: no loop"),
        # The loop shape with a FRAMED function beside it: `label_slots` returns
        # `None`, so the three-valued gate refuses the whole TU. Board #746/#747.
        ("g-framed", "int gz(int);\n"
                     "int P(const char* s){ int r=0; while (*s) { int c=*s; r=r+c; s++; } return r; }\n"
                     "int z9(int a){ return gz(a)+7; }\n",
         "refuse", "CONTROL: loop + framed refuses the TU"),
    ]


GRIDS = {"A": grid_a, "B": grid_b, "C": grid_c, "D": grid_d,
         "E": grid_e, "F": grid_f, "G": grid_g}


def run(cells, mode, jobs):
    """Compile every cell with real c2 and with the port; return the verdicts."""
    reached = graded = failed = 0
    passes = fails = 0
    mismatches = []
    control_failures = []
    with tempfile.TemporaryDirectory() as wd:
        paths = []
        for name, src, want, why in cells:
            p = os.path.join(wd, "%s.cpp" % name.replace("-", "_"))
            open(p, "w").write(src)
            paths.append(p)
        flags = os.path.join(wd, "flags.txt")
        open(flags, "w").write(mode + "\n")
        lst = os.path.join(wd, "list.txt")
        with open(lst, "w") as f:
            for p in paths:
                f.write("z:" + p.replace("/", "\\") + "\n")
        jsonl = os.path.join(wd, "scan.jsonl")
        r = subprocess.run(
            [os.path.join(REPO, "target/release/c2rs"), "gap",
             "--list", lst, "--flags-file", flags,
             "--jobs", str(jobs), "--jsonl", jsonl],
            capture_output=True, text=True)
        if "SKIP" in r.stdout and "toolchain absent" in r.stdout:
            print("SKIP: toolchain absent")
            return None
        verdicts = {}
        if os.path.exists(jsonl):
            for line in open(jsonl):
                line = line.strip()
                if not line:
                    continue
                # A hand-rolled reader: the field is a bare string value and the
                # workspace is std-only by policy, so the grid stays dependency
                # free too.
                def field(key):
                    k = '"%s"' % key
                    i = line.find(k)
                    if i < 0:
                        return None
                    i = line.find(":", i + len(k)) + 1
                    while i < len(line) and line[i] in ' \t':
                        i += 1
                    if line[i] != '"':
                        j = i
                        while j < len(line) and line[j] not in ',}':
                            j += 1
                        return line[i:j].strip()
                    j = line.find('"', i + 1)
                    return line[i + 1:j]
                path = field("src") or ""
                cls = field("class") or ""
                if not path or not cls:
                    continue          # the provenance record, not a verdict row
                verdicts[os.path.basename(path.replace("\\", "/")).lower()] = cls

        for name, src, want, why in cells:
            reached += 1
            key = ("%s.cpp" % name.replace("-", "_")).lower()
            got = verdicts.get(key)
            if got is None:
                failed += 1
                print("  %-12s CAPTURE/SCAN FAILURE -- no verdict row" % name)
                continue
            graded += 1
            if got == "mismatch":
                mismatches.append((name, why))
                fails += 1
                print("  %-12s MISMATCH  <- the alarm  (%s)" % (name, why))
                continue
            ok = (got == "match") if want == "match" else (got != "match")
            if ok:
                passes += 1
            else:
                fails += 1
                if why.startswith("CONTROL"):
                    control_failures.append(name)
            print("  %-12s %-13s want=%-7s %s   %s"
                  % (name, got, want, "ok" if ok else "FAIL", why))
    return dict(reached=reached, graded=graded, failed=failed, passes=passes,
                fails=fails, mismatches=mismatches,
                control_failures=control_failures)


def main():
    argv = sys.argv[1:]
    mode = "/O1 /GS- /c"
    jobs = 8
    only = None
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    if "--jobs" in argv:
        i = argv.index("--jobs")
        jobs = int(argv[i + 1])
        del argv[i:i + 2]
    if "--grid" in argv:
        i = argv.index("--grid")
        only = [g.upper() for g in argv[i + 1:]]
        del argv[i:]

    print("vargrid: mode [%s]  jobs %d" % (mode, jobs))
    tot = dict(reached=0, graded=0, failed=0, passes=0, fails=0)
    all_mm, all_cf = [], []
    for gname in sorted(GRIDS):
        if only and gname not in only:
            continue
        cells = GRIDS[gname]()
        print("\n=== GRID %s -- %d cells" % (gname, len(cells)))
        res = run(cells, mode, jobs)
        if res is None:
            return 0
        for k in tot:
            tot[k] += res[k]
        all_mm += res["mismatches"]
        all_cf += res["control_failures"]

    print("\n" + "=" * 68)
    print("reached %d   graded %d   capture/scan FAILURES %d"
          % (tot["reached"], tot["graded"], tot["failed"]))
    print("pass    %d   fail   %d" % (tot["passes"], tot["fails"]))
    print("MISMATCHES: %d %s" % (len(all_mm), [m[0] for m in all_mm]))
    print("CONTROLS FAILED: %d %s" % (len(all_cf), all_cf))
    bad = len(all_mm) or len(all_cf) or tot["failed"]
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())

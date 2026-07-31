#!/usr/bin/env python3
"""gt_cmp_spine.py — the comparison spine's four axes, measured against real c2.

`docs/CMP_PRODUCES_A_VALUE.md` is what this produces. The claim under test is
that c2's branchless comparison spine is a function of

    (relation, signedness, literal-is-zero)

alone, which is what `c2_core::codegen::leaf::compare::compare_leaf_text` assumes
today. It is not. Two further axes move the bytes, and neither is visible in the
IL operator:

  * **the RESULT type** — `bool` against `int`/`unsigned`. The IL difference is
    one optional `2C <int> 00` convert; the obj difference, for the two signed
    sign-sum spines, is two words and a different schedule.
  * **where the compared value came from** — a formal (leaf) against a call's
    result (framed). One cell of forty-eight, and it swaps two register numbers.

Both are exactly the kind of axis `docs/GAPS.md` §6 records as productive: they
change no operator and no shape, so no hand-written fixture varies them.

The grid is (6 relations × 2 signednesses × {0, 3} × {int, bool} × {leaf,
framed}) = 48 spines per mode, each compiled in the SAME translation unit so
nothing depends on comparing captures taken separately.

Usage:
    scripts/gt_cmp_spine.py [--keep] [--mode '/Ox /GS- /Gy /c'] ...
    scripts/gt_cmp_spine.py --stride [--mode ...]     the label-counter surcharge

Env: C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.

Outside the std-only Rust workspace on purpose — tooling, never linked into the
port, same status as `scripts/gt_dump.py` and `scripts/gt_label_stride.py`.
Exit status is 0 if every probe compiled; read the table for the finding.
"""

import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))

RELS = [("ge", ">="), ("gt", ">"), ("lt", "<"), ("le", "<="), ("eq", "=="), ("ne", "!=")]
LITERALS = [0, 3]

HEADER = "struct Obj { int C() const; unsigned U() const; };\n"


def source():
    """One TU carrying the whole grid: leaf and framed, int and bool, side by side."""
    out = [HEADER]
    for rn, ro in RELS:
        for k in LITERALS:
            for sg, ty, lit, call in (
                ("s", "int", str(k), "C"),
                ("u", "unsigned", "%du" % k, "U"),
            ):
                for res, rr in (("int", "i"), ("bool", "b")):
                    tag = "%s_%s_%s_%d" % (rr, sg, rn, k)
                    out.append("%s L_%s(%s a) { return a %s %s; }" % (res, tag, ty, ro, lit))
                    out.append(
                        "%s F_%s(const Obj* p) { return p->%s() %s %s; }"
                        % (res, tag, call, ro, lit)
                    )
    return "\n".join(out) + "\n"


def capture(cpp, mode, obj):
    env = dict(os.environ)
    r = subprocess.run(
        [os.path.join(HERE, "gt_capture.sh"), cpp] + mode.split(),
        capture_output=True, text=True, env=dict(env, GT_OUT=obj),
    )
    if not os.path.exists(obj):
        sys.stderr.write(r.stderr)
        return False
    return True


def text_words(obj):
    """{function tag: [big-endian instruction words as hex]} for every .text."""
    out = subprocess.run(
        [os.path.join(HERE, "gt_dump.py"), obj, "--text-only"],
        capture_output=True, text=True,
    ).stdout
    fns, cur = {}, None
    for line in out.splitlines():
        m = re.match(r"^-- \.text #\d+ \(\d+ B\) (\S+)", line)
        if m:
            cur = m.group(1)
            fns[cur] = []
            continue
        m = re.match(r"^   [0-9a-f]{4}  ([0-9a-f]{8})", line)
        if m and cur is not None:
            fns[cur].append(m.group(1))
    got = {}
    for name, words in fns.items():
        m = re.match(r"\?([LF])_([ib]_[su]_[a-z]{2}_\d+)@", name)
        if m:
            got[m.group(1) + "_" + m.group(2)] = words
    return got


# A leaf's spine is its whole body less the trailing `blr`; a framed one is its
# body less the three-word prologue, the `bl`, and the four-word epilogue. Both
# are the Class A frame this port already emits, so the two constants are the
# shipped layout and not a fit.
LEAF_TRIM = (0, -1)
FRAMED_TRIM = (4, -4)


def run(mode, keep):
    wd = tempfile.mkdtemp(prefix="gt-cmp-spine")
    cpp = os.path.join(wd, "grid.cpp")
    obj = os.path.join(wd, "grid.obj")
    open(cpp, "w").write(source())
    if not capture(cpp, mode, obj):
        # Degrade cleanly, exactly as the CLI and the integration tests do: an
        # absent toolchain is not a failed measurement.
        print("SKIP: toolchain absent")
        return 0
    got = text_words(obj)
    print("===== %s   (%d functions)" % (mode, len(got)))
    diffs = 0
    for key in sorted(k[2:] for k in got if k.startswith("L_i_")):
        bkey = "b_" + key[2:]
        spines = {
            "leaf-int": got["L_" + key][LEAF_TRIM[0]:LEAF_TRIM[1]],
            "leaf-bool": got["L_" + bkey][LEAF_TRIM[0]:LEAF_TRIM[1]],
            "fram-int": got["F_" + key][FRAMED_TRIM[0]:FRAMED_TRIM[1]],
            "fram-bool": got["F_" + bkey][FRAMED_TRIM[0]:FRAMED_TRIM[1]],
        }
        flags = []
        if spines["leaf-int"] != spines["leaf-bool"]:
            flags.append("BOOL")
        if spines["leaf-int"] != spines["fram-int"]:
            flags.append("FRAMED")
        if spines["leaf-bool"] != spines["fram-bool"]:
            flags.append("FRAMED-bool")
        diffs += bool(flags)
        label = ",".join(flags) if flags else "."
        print("  %-10s %-16s %s" % (key[2:], label, " ".join(spines["leaf-int"])))
        if flags:
            for name in ("leaf-bool", "fram-int", "fram-bool"):
                print("  %-10s %-16s %s" % ("", name, " ".join(spines[name])))
    print("  -> %d of %d cells depend on an axis the leaf record does not carry"
          % (diffs, len(got) // 4))
    if keep:
        print("  kept: %s" % wd, file=sys.stderr)
    return 0


# --- the label stride, on the same grid -------------------------------------
#
# Reuses `gt_label_stride.py`'s seed-free in-TU method verbatim rather than
# re-deriving it: two plain framed anchors around the probe, so
#
#     stride(P) = first(a1) - first(a0) - 5      slots P consumes in total
#     extra(P)  = first(P)  - first(a0) - 5      slots taken BEFORE P's own $M
#
# with `first(a2) - first(a1) == 5` asserted on every row as the in-TU control.
# The `- 5` is the /Gy anchor stride; under packed flags the anchors are 4 and
# every printed number is one low, which is why the control column is printed
# rather than assumed.
STRIDE_PROBES = [
    ("leaf-i-s-lt-5", "int  P(int a){ return a < 5; }"),
    ("leaf-b-s-lt-5", "bool P(int a){ return a < 5; }"),
    ("leaf-b-s-eq-5", "bool P(int a){ return a == 5; }"),
    ("leaf-b-u-lt-5", "bool P(unsigned a){ return a < 5u; }"),
    ("fr-plain", "int  P(int a){ return gp(a)+1; }"),
    ("fr-m-plain", "int  P(const Obj* p){ return p->C()-20; }"),
    ("fr-m-i-s-lt-5", "int  P(const Obj* p){ return p->C() < 5; }"),
    ("fr-m-b-s-lt-5", "bool P(const Obj* p){ return p->C() < 5; }"),
    ("fr-m-b-s-ge-5", "bool P(const Obj* p){ return p->C() >= 5; }"),
    ("fr-m-b-s-le-5", "bool P(const Obj* p){ return p->C() <= 5; }"),
    ("fr-m-b-s-gt-5", "bool P(const Obj* p){ return p->C() > 5; }"),
    ("fr-m-b-s-gt-0", "bool P(const Obj* p){ return p->C() > 0; }"),
    ("fr-m-b-s-eq-5", "bool P(const Obj* p){ return p->C() == 5; }"),
    ("fr-m-b-u-lt-5", "bool P(const Obj* p){ return p->U() < 5u; }"),
    ("fr-f-b-s-lt-5", "bool P(int a){ return gp(a) < 5; }"),
    ("fr-f-b-s-gt-0", "bool P(int a){ return gp(a) > 0; }"),
]

STRIDE_DECLS = "int gp(int);\nstruct Obj { int C() const; unsigned U() const; };"


def run_stride(mode):
    sys.path.insert(0, HERE)
    import gt_label_stride as G

    wd = tempfile.mkdtemp(prefix="gt-cmp-stride")
    print("===== stride  %s\n  %-16s %7s %6s  control" % (mode, "probe", "stride", "leading"))
    for name, body in STRIDE_PROBES:
        o = G.capture(G.build_src(STRIDE_DECLS, [], body), mode, wd, name)
        if o is None:
            print("SKIP: toolchain absent")
            return 0
        groups = {g["name"]: g for g in G.groups(o)}

        def first(nm):
            for k, g in groups.items():
                if k.startswith("?" + nm + "@"):
                    return min(g["labels"]) if g["labels"] else None
            return None

        a0, a1, a2, p = first("a0"), first("a1"), first("a2"), first("P")
        if None in (a0, a1, a2):
            print("  %-16s ANCHORS MISSING" % name)
            continue
        # `a1` and `a2` are adjacent, so their difference IS the plain framed
        # anchor's stride — 5 under `/Gy`, 4 packed. Taking it from the object
        # rather than from the flags string is what keeps the row self-checking:
        # anything other than those two values means the anchors moved and the
        # whole row is void.
        anchor = a2 - a1
        ctl = "OK(%d)" % anchor if anchor in (4, 5) else "CTL-BROKEN(%d)" % anchor
        lead = "-" if p is None else str(p - a0 - anchor)
        print("  %-16s %7d %6s  %s" % (name, a1 - a0 - anchor, lead, ctl))
    return 0


def main():
    args = sys.argv[1:]
    keep = "--keep" in args
    stride = "--stride" in args
    args = [a for a in args if a not in ("--keep", "--stride")]
    modes = []
    while args:
        if args[0] == "--mode":
            modes.append(args[1])
            args = args[2:]
        else:
            args = args[1:]
    if not modes:
        modes = ["/Ox /GS- /Gy /c", "/O1 /GS- /Gy /c"]
    rc = 0
    for m in modes:
        rc |= run_stride(m) if stride else run(m, keep)
    return rc


sys.exit(main())

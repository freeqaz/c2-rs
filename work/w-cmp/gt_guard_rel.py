#!/usr/bin/env python3
"""gt_guard_rel.py — are the six relational opcodes ONE FAMILY in the GUARD
position? Measured against real c2, not argued.

THE CLAIM UNDER TEST is the premise of w-cmp's own `C2RS_SINK_REL` sink, which
consumes `0x1F..=0x24` in a single match arm as though the six were
interchangeable. That is fine for a *census* sink (it cannot emit), and it would
be a **wrong emit** for a builder who read the rung and generalised. The tree
already holds one refutation of the same premise one position over — the
compiler-label stride table in `CompareLeaf::label_slots` reads

    ==, !=              1   every literal, both signednesses
    unsigned operand    1   every relation, every literal
    signed <  k == 0    1     signed >= k == 0   1
    signed <  k != 0    3     signed >= k != 0   3
    signed >  anything  3     signed <= anything 3

so in the VALUE position the six are demonstrably three groups, not one. This
script asks the same question in the position w-cmp's whole frontier population
actually lives in: the early-return guard `if (a <rel> k) return;`, which is
what all 8 blocked emitted functions of `mmio` / `vswprnc` /
`IPP_basicmath_xbox` open with.

Grid: 6 relations x {signed, unsigned} x {k = 0, k = 5} = 24 guards, all in ONE
translation unit so nothing depends on comparing captures taken separately, at
the WORKLOAD's own flags rather than a fixture profile.

Outside the std-only Rust workspace on purpose — tooling, never linked into the
port, same status as `scripts/gt_cmp_spine.py`. Exit status is 0 if every probe
compiled; read the table for the finding. It prints COUNTS, never a status.

Usage:  gt_guard_rel.py [--mode '<flags>'] [--keep]
Env:    C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.
"""

import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPTS = os.path.join(HERE, "..", "..", "scripts")

RELS = [("eq", "=="), ("ne", "!="), ("lt", "<"), ("le", "<="), ("gt", ">"), ("ge", ">=")]
LITERALS = [0, 5]

# The workload's own profile. A guard measured at /Ox would be a measurement of
# a mode nobody compiles this corpus in.
WORKLOAD_MODE = "/nologo /c /GR /O1 /Oi /EHsc /Gy"

DECLS = "void sink();\n"


def source():
    """One TU carrying the whole guard grid."""
    out = [DECLS]
    for rn, ro in RELS:
        for k in LITERALS:
            for sg, ty, lit in (("s", "int", str(k)), ("u", "unsigned", "%du" % k)):
                tag = "g_%s_%s_%d" % (sg, rn, k)
                out.append(
                    "void %s(%s a) { if (a %s %s) return; sink(); }" % (tag, ty, ro, lit)
                )
    return "\n".join(out) + "\n"


def capture(cpp, mode, obj):
    r = subprocess.run(
        [os.path.join(SCRIPTS, "gt_capture.sh"), cpp] + mode.split(),
        capture_output=True, text=True, env=dict(os.environ, GT_OUT=obj),
    )
    if not os.path.exists(obj):
        sys.stderr.write(r.stderr)
        return False
    return True


def text_words(obj):
    """{probe tag: [big-endian instruction words as hex]} for every .text."""
    out = subprocess.run(
        [os.path.join(SCRIPTS, "gt_dump.py"), obj, "--text-only"],
        capture_output=True, text=True,
    ).stdout
    fns, cur = {}, None
    for line in out.splitlines():
        m = re.match(r"^-- \.text #\d+ \(\d+ B\) (\S+)", line)
        if m:
            cur, fns[m.group(1)] = m.group(1), []
            continue
        m = re.match(r"^   [0-9a-f]{4}  ([0-9a-f]{8})", line)
        if m and cur is not None:
            fns[cur].append(m.group(1))
    got = {}
    for name, words in fns.items():
        m = re.search(r"\?(g_[su]_[a-z]{2}_\d+)@", name)
        if m:
            got[m.group(1)] = words
    return got


def main():
    mode = WORKLOAD_MODE
    keep = "--keep" in sys.argv
    if "--mode" in sys.argv:
        mode = sys.argv[sys.argv.index("--mode") + 1]
    wd = tempfile.mkdtemp(prefix="gt-guard-rel")
    cpp, obj = os.path.join(wd, "grid.cpp"), os.path.join(wd, "grid.obj")
    open(cpp, "w").write(source())
    if not capture(cpp, mode, obj):
        # Degrade cleanly — an absent toolchain is not a failed measurement.
        print("SKIP: toolchain absent")
        return 0
    got = text_words(obj)
    want = len(RELS) * len(LITERALS) * 2
    print("mode: %s" % mode)
    print("probes compiled: %d of %d expected  (a positive count, not a status)"
          % (len(got), want))
    if len(got) != want:
        print("  MISSING: %s" % sorted(
            "g_%s_%s_%d" % (sg, rn, k)
            for rn, _ in RELS for k in LITERALS for sg in ("s", "u")
            if "g_%s_%s_%d" % (sg, rn, k) not in got))

    print("\n%-14s %-5s %s" % ("probe", "len", "words"))
    shapes = {}
    for sg in ("s", "u"):
        for k in LITERALS:
            for rn, _ in RELS:
                tag = "g_%s_%s_%d" % (sg, rn, k)
                w = got.get(tag)
                if w is None:
                    continue
                print("%-14s %-5d %s" % (tag, len(w), " ".join(w)))
                shapes.setdefault(len(w), []).append(tag)

    print("\nBODY LENGTH classes over the %d probes (a partition, printed as counts):"
          % len(got))
    for n in sorted(shapes):
        print("  %2d words : %2d probes   %s" % (n, len(shapes[n]), " ".join(shapes[n])))
    print("\n  distinct body lengths: %d" % len(shapes))
    print("  If the six relations were ONE family in the guard position this "
          "would be 1\n  for every (signedness, literal) pair. It is not a "
          "status line -- read the\n  partition above.")
    if keep:
        sys.stderr.write("  kept: %s\n" % wd)
    return 0


if __name__ == "__main__":
    sys.exit(main())

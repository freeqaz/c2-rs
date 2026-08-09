#!/usr/bin/env python3
"""w-memfit — MEASURE the front end's alignment hint, per pointee type, BLACK BOX.

`WB_MEMCPY_FINDINGS.md` §2 says the expansion divides the size by
`BYTE [node+0x38]`, the front end's alignment hint, and §5.3 identifies that
byte POSITIONALLY in the captured IL by differencing three sources that differ
only in the pointer type (values `01` / `04` / `08`).

Reading the IL that `c1xx.dll` hands to `c2` is a **black-box** observation of
the toolchain — it is the port's own input format, decoded by `crates/c2-il`,
and needs no disassembler. This script re-does §5.3's difference over ALL EIGHT
pointee types that appear in `w-memcpy`'s GRID-M and GRID-M2, so that the
`align` a rival uses is MEASURED per type rather than assumed from the C type.

That matters concretely: GRID-M's own manifest records `align = 16` for its
16-byte struct `S16 { double a; double b; }`, and if the hint really were 16 the
reading would predict `inline` at size 48 where c2 emits a call. `alignof(S16)`
is 8, not 16 — this script decides which number the IL actually carries.

Usage:  hint.py gen <outdir>
        hint.py read <outdir> <root>      (after `gen`, captures + reports)
"""

import json
import os
import subprocess
import sys

# (tag, C pointee type, extra decls, alignof by the C++ type system)
TYPES = [
    ("c", "char", "", 1),
    ("i", "int", "", 4),
    ("d", "double", "", 8),
    ("s16", "S16", "struct S16 { double a; double b; };", 8),
    ("v", "void", "", 1),
    ("q", "long long", "", 8),
    ("s4", "S4", "struct S4 { int a; };", 4),
    ("s32", "S32", "struct S32 { double a[4]; };", 8),
]

SIZE = 96

SRC = """// w-memfit alignment-hint probe %s
%s
extern "C" void *memcpy(void *, const void *, unsigned int);
void f(%s *d, const %s *s) { memcpy(d, s, %d); }
"""


def gen(outdir):
    os.makedirs(outdir, exist_ok=True)
    for tag, ctype, decl, _al in TYPES:
        open(os.path.join(outdir, "h_%s.cpp" % tag), "w").write(
            SRC % (tag, decl, ctype, ctype, SIZE))
    print("wrote %d hint cells to %s" % (len(TYPES), outdir))


def ex_bytes(root, outdir, tag):
    """Capture the IL bundle for one cell and return its `.ex` stream.

    `--keep-il` resolves against the PROCESS cwd while the source resolves
    against `--cwd`, so everything here is absolute and the process runs from
    the repo root.
    """
    outdir = os.path.abspath(outdir)
    ildir = os.path.join(outdir, "il_" + tag)
    src = "h_%s.cpp" % tag
    if not os.path.isdir(ildir):
        r = subprocess.run([os.path.join(root, "target/release/c2rs"), "capture",
                            src, "--keep-il", ildir,
                            "--flags-file", os.path.join(root,
                                                         "work/dc3-workload/flags.txt"),
                            "--cwd", outdir],
                           cwd=root, capture_output=True, text=True, check=False)
        if not os.path.isdir(ildir):
            raise SystemExit("capture failed for %s:\n%s\n%s"
                             % (tag, r.stdout, r.stderr))
    for fn in os.listdir(ildir):
        if fn.endswith(".ex"):
            return open(os.path.join(ildir, fn), "rb").read()
    raise SystemExit("no .ex in %s" % ildir)


def read(outdir, root):
    ex = {t[0]: ex_bytes(root, outdir, t[0]) for t in TYPES}
    ref = ex["c"]
    # Every cell differs from the char* cell only in the alignment hint (and in
    # whatever type-table bytes the front end writes), so the positions that
    # differ ACROSS ALL EIGHT and take exactly the by-type value are the hint.
    n = min(len(b) for b in ex.values())
    diffs = [i for i in range(n)
             if len({ex[t][i] for t in ex}) > 1]
    same_len = len({len(b) for b in ex.values()}) == 1
    print("bundle .ex lengths: %s  (all equal: %s)"
          % ({t: len(ex[t]) for t in ex}, same_len))
    print("byte positions differing across the eight cells: %s" % diffs)
    for i in diffs:
        print("   offset %5d : %s" % (i, {t: "%02x" % ex[t][i] for t in ex}))
    # Report the candidate hint positions: those whose value set is exactly
    # {1,4,8} spread over the types the way alignof would.
    print()
    print("%-5s %-12s %-8s %s" % ("tag", "C type", "alignof", "IL bytes at the differing offsets"))
    rows = {}
    for tag, ctype, _d, al in TYPES:
        vals = [ex[tag][i] for i in diffs]
        rows[tag] = vals
        print("%-5s %-12s %-8d %s" % (tag, ctype, al,
                                      " ".join("%02x" % v for v in vals)))
    # The verdict: is there a position whose byte equals alignof(pointee) for
    # every one of the eight?
    hits = [i for k, i in enumerate(diffs)
            if all(ex[t[0]][i] == t[3] for t in TYPES)]
    print()
    print("offsets whose byte == alignof(pointee) for ALL EIGHT types: %s" % hits)
    json.dump({"diffs": diffs,
               "per_type": {t[0]: {"ctype": t[1], "alignof": t[3],
                                   "bytes": ["%02x" % ex[t[0]][i] for i in diffs]}
                            for t in TYPES},
               "alignof_positions": hits},
              open(os.path.join(outdir, "hints.json"), "w"), indent=1)


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    else:
        read(sys.argv[2], sys.argv[3])

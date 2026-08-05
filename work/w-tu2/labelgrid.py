#!/usr/bin/env python3
"""labelgrid.py — measure the compiler-label SURCHARGE directly, lane w-tu2.

Board #286/#287 (LABEL_COUNTER.md §4.1) measured that the control-flow surcharge
is derivable from neither the emitted obj nor the `.gl` label seed. `mmio.cpp`
needs it: its six `$M` numbers have gaps of +3 and +8, and any wrong slot is six
wrong bytes in the symbol table.

This measures the surcharge on cells of MY choosing rather than re-reading
w-label's table, because R3 is a claim about `mmio`'s own shapes.

METHOD — self-normalizing, so the TU-level seed drops out. Each cell is one TU:

    <PROBE>            a framed function whose interior control flow varies
    int Z(int x) { return zz(x) + 1; }    a fixed framed function AFTER it

A framed function contributes exactly three symbol-table label slots:
`$M(n)` at prologue end, `$M(n+1)` at function end, `$T(n+2)` on its `.pdata`.
So with no surcharge Z's first label is PROBE's first label + 3. The
SURCHARGE is `first(Z) - first(PROBE) - 3`.

Read-only measurement tooling; outside the std-only workspace, same status as
scripts/gt_dump.py.
"""

import os
import re
import struct
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))

TAIL = "\nextern int zz(int);\nint Z(int x) { return zz(x) + 1; }\n"
HEAD = "extern int g(int);\nextern int h(int);\nextern void v(int);\n"

# ---- the cells -------------------------------------------------------------
# `fit` cells are the ones a rule may be fitted on; `heldout` cells are chosen
# BEFORE any fit is attempted and are never shown to it. R7.
CELLS = {
    # --- straight line, the zero reference ---
    "straight":   ("int P(int a,int b){ return g(a)+1; }", "fit"),
    "twocall":    ("int P(int a,int b){ return g(a)+h(b); }", "fit"),

    # --- mmioGetInfo's own shape: guarded early returns, then a call ---
    "if1_ret":    ("int P(int a,int b){ if(a) return 5; return g(b)+1; }", "fit"),
    "if2_ret":    ("int P(int a,int b){ if(a) return 5; if(b) return 11;"
                   " return g(b)+1; }", "fit"),
    "if3_ret":    ("int P(int a,int b){ if(a) return 5; if(b) return 11;"
                   " if(a+b) return 7; return g(b)+1; }", "heldout"),

    # --- the same skeleton without the early return (join instead of exit) ---
    "if1_join":   ("int P(int a,int b){ int r=0; if(a) r=5; return g(r)+1; }",
                   "fit"),
    "ifelse":     ("int P(int a,int b){ int r; if(a) r=5; else r=11;"
                   " return g(r)+1; }", "fit"),

    # --- mmioSetInfo's tail: a guarded STORE, no early exit ---
    "if_store":   ("int P(int a,int b){ int r=g(a); if(b>r) return r;"
                   " return r+1; }", "heldout"),

    # --- mmioClose's shape: call, test its result, call again ---
    "call_test":  ("int P(int a,int b){ if(g(a)) return 5; return h(b)+1; }",
                   "fit"),
    "call_test2": ("int P(int a,int b){ if(g(a)) return 5; if(h(b)) return 11;"
                   " return g(b)+1; }", "heldout"),

    # --- loops, for the regime boundary ---
    "while":      ("int P(int a,int b){ while(a){ b=g(b); --a; } return b; }",
                   "fit"),
    "dowhile":    ("int P(int a,int b){ do { b=g(b); --a; } while(a);"
                   " return b; }", "fit"),

    # --- short-circuit: w-label's sharpest cells, two branches one target ---
    "and":        ("int P(int a,int b){ if(a&&b) return 5; return g(b)+1; }",
                   "heldout"),
    "or":         ("int P(int a,int b){ if(a||b) return 5; return g(b)+1; }",
                   "heldout"),

    # --- ternary, w-label's ho-ternary ---
    "ternary":    ("int P(int a,int b){ return g(a?b:b+1)+1; }", "fit"),

    # --- the STUBS: what does an 8-byte `li r3,0 ; blr` leaf charge? ---
    # mmio has 5 of them between mmioSetInfo and mmioClose.
    "stub1":      ("int P(int a,int b){ return g(a)+1; }\nint s1(){return 0;}",
                   "fit"),
    "stub5":      ("int P(int a,int b){ return g(a)+1; }\n"
                   "int s1(){return 0;}\nint s2(){return 0;}\n"
                   "int s3(){return 0;}\nint s4(){return 0;}\n"
                   "int s5(){return 0;}", "heldout"),
}


def flags():
    with open(os.path.join(REPO, "work", "dc3-workload", "flags.txt")) as f:
        return f.read().split()


def sib(name):
    d = REPO
    while d != "/":
        p = os.path.join(d, "..", name)
        if os.path.isdir(p):
            return os.path.abspath(p)
        d = os.path.dirname(d)
    return None


def compile_cell(name, src, topdir):
    # Each cell gets its OWN directory: cl.exe's `_CL_*` intermediate collides
    # across cells sharing one TMP, and the collision surfaces as a bare rc=2
    # with no diagnostic at all -- which is exactly the shape that reads as
    # "the cell has no answer" instead of "the runner is broken".
    outdir = os.path.join(topdir, name)
    os.makedirs(outdir, exist_ok=True)
    cpp = os.path.join(outdir, name + ".cpp")
    obj = os.path.join(outdir, name + ".obj")
    with open(cpp, "w") as f:
        f.write(HEAD + src + TAIL)
    wibo = os.environ.get("C2RS_WIBO") or os.path.join(
        sib("wibo"), "build", "release", "wibo")
    cl = os.path.join(REPO, "compilers", "X360", "16.00.11886.00", "cl.exe")
    if not (os.path.exists(wibo) and os.path.exists(cl)):
        print("SKIP: toolchain absent")
        sys.exit(3)
    zout = "Z:" + obj.replace("/", "\\")
    env = dict(os.environ, TMP=outdir, TEMP=outdir, WIBO_FS_CACHE="1")
    # The source is passed as a BASENAME with cwd set to the cell dir, which is
    # the invocation that works by hand. An absolute source path makes cl exit 2
    # with no diagnostic on stdout at all.
    r = subprocess.run([wibo, cl] + flags() + ["/Fo" + zout,
                                               os.path.basename(cpp)],
                       cwd=outdir, env=env, capture_output=True, text=True)
    if not (os.path.exists(obj) and os.path.getsize(obj)):
        # A positive, printed diagnosis. STATUS trap 5: a compile that produced
        # nothing must say WHY, or the grid reads its own emptiness as a result.
        tail = (r.stdout or "").strip().splitlines()[-2:]
        print(f"    [{name}] rc={r.returncode} " + " | ".join(tail))
        return None
    return obj


def labels(path):
    """Every $M/$T symbol, in symbol-table order, as (name, number)."""
    d = open(path, "rb").read()
    symptr = struct.unpack_from("<I", d, 8)[0]
    nsym = struct.unpack_from("<I", d, 12)[0]
    strtab = d[symptr + 18 * nsym:]
    out, i = [], 0
    while i < nsym:
        o = symptr + 18 * i
        nm = d[o:o + 8]
        if nm[0:4] == b"\0\0\0\0":
            off = struct.unpack_from("<I", d, o + 4)[0]
            name = strtab[off:strtab.index(b"\0", off)].decode()
        else:
            name = nm.rstrip(b"\0").decode()
        if re.fullmatch(r"\$[MT]\d+", name):
            out.append((name, int(name[2:])))
        i += 1 + d[o + 17]
    return out


def main():
    outdir = os.path.join(HERE, "grid")
    os.makedirs(outdir, exist_ok=True)
    print(f"{'cell':12} {'set':8} {'labels':34} {'surcharge':>9}")
    print("-" * 68)
    rows = []
    for name, (src, kind) in CELLS.items():
        obj = compile_cell(name, src, outdir)
        if obj is None:
            print(f"{name:12} {kind:8} COMPILE-FAIL")
            continue
        ls = labels(obj)
        if len(ls) < 6:
            print(f"{name:12} {kind:8} "
                  f"{','.join(n for n, _ in ls):34} "
                  f"{'NOT-FRAMED':>9}")
            continue
        # PROBE's three slots come first, Z's next.
        first_probe = ls[0][1]
        first_z = ls[3][1]
        sur = first_z - first_probe - 3
        rows.append((name, kind, sur))
        print(f"{name:12} {kind:8} "
              f"{','.join(str(v) for _, v in ls):34} {sur:9}")
    print()
    print("counted cells:", len(rows), "of", len(CELLS))
    return 0


if __name__ == "__main__":
    sys.exit(main())

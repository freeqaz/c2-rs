#!/usr/bin/env python3
"""gen_grids.py — write GRID-A (fit) and GRID-H (holdout) for RULE BIND.

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

    gen_grids.py <outdir>

Both grids are written by ONE run of this file, so the holdout's sources exist
before a single GRID-A obj does; `work/w-alloc3/gridH.sha256` is committed in
the same commit as this generator. `RULE W2` (board #887) is why: it passed
every previously recorded refutation cell and then died on fresh ones, so a
holdout that could have been reshaped after seeing the fit result is not a
holdout.

THE AXES, AND WHY THE SPLIT IS WHERE IT IS
------------------------------------------
`docs/rungs/2026-08-06-w-seam.md` and board #870 record the same failure three
times: a grid that varies VALUES exhaustively and STRUCTURE not at all. So the
two grids are split by *structural family*, not by a random draw over one
family. **Every family in GRID-H is absent from GRID-A**:

    GRID-A   caller formal counts 1-4; one- and two-formal callees; the three
             modes (ret / void / arith); callee bodies of 1 and 2 words; the
             `add`/`subf` pair.
    GRID-H   caller formal counts 5-8 (H-wide); THREE-formal callees at all six
             permutations (H-perm); a callee that already holds a temp in r11
             (H-temp); five producer SPELLINGS GRID-A never contains — neg,
             shift, and, mullw, sign-extension (H-spell), which is the axis
             RULE W and RULE W2 both died on; a zero-formal callee (H-noarg);
             an indexed load (H-idx); a void store taking two formals
             (H-store); a zero displacement (H-zero-off); and four cells
             registered OUT OF DOMAIN in advance (H-out-*).

Each cell gets **its own directory** — board **#1045**, where four parallel
tests sharing one PID-keyed temp dir raced and fabricated a finding that would
have reversed a decline.
"""

import hashlib
import json
import os
import sys

PREAMBLE = "struct V { int* a; int* b; int c; };\nstruct W { V* p; int q; };\n"
ANCHOR = "void ext_anchor();\nvoid anchor() { ext_anchor(); }\n"

# name -> (source text of g, param C types, return C type)
CALLEES = {
    "LD1": ("int* g(V* v) { return v->b; }", ["V*"], "int*"),
    "LD1a": ("int* g(V* v) { return v->a; }", ["V*"], "int*"),
    "LDc": ("int g(V* v) { return v->c; }", ["V*"], "int"),
    "ADD1": ("int g(int a) { return a + 1; }", ["int"], "int"),
    "SUM": ("int g(int a, int b) { return a + b; }", ["int", "int"], "int"),
    "SUB": ("int g(int a, int b) { return a - b; }", ["int", "int"], "int"),
    "LD2": ("int* g(W* w) { return w->p->b; }", ["W*"], "int*"),
    "VOID1": ("void g(V* v) { v->c = 0; }", ["V*"], "void"),
    "IDX": ("int g(int* p, int i) { return p[i]; }", ["int*", "int"], "int"),
    "SUM3": ("int g(int a, int b, int c) { return a - b + c; }", ["int"] * 3, "int"),
    "EXT": ("int g(V* v) { return (short)v->c; }", ["V*"], "int"),
    "NEG": ("int g(int a) { return -a; }", ["int"], "int"),
    "SHL": ("int g(int a) { return a << 3; }", ["int"], "int"),
    "AND": ("int g(int a, int b) { return a & b; }", ["int", "int"], "int"),
    "MUL": ("int g(int a, int b) { return a * b; }", ["int", "int"], "int"),
    "NOARG": ("int g() { return 42; }", [], "int"),
    "STORE2": ("void g(V* v, int k) { v->c = k; }", ["V*", "int"], "void"),
}


def scale(ctype):
    """The C scale factor of `+ 1` on a value of this type. `int*` steps 4."""
    return 4 if ctype.endswith("*") else 1


def cell(name, axis, callee, n, beta, mode, k=0, domain="in", extra=None):
    """One grid cell. `beta[i]` is the CALLER FORMAL INDEX supplying `g`'s
    i-th formal, so `g`'s register r(3+i) binds to the caller's r(3+beta[i])."""
    gsrc, gparams, gret = CALLEES[callee]
    assert len(beta) == len(gparams), (name, beta, gparams)
    assert len(set(beta)) == len(beta), ("beta must be injective", name)
    assert all(0 <= p < n for p in beta), (name, beta, n)

    ptypes = ["int"] * n
    for i, p in enumerate(beta):
        ptypes[p] = gparams[i]
    names = ["x%d" % j for j in range(n)]
    decl = ", ".join("%s %s" % (t, nm) for t, nm in zip(ptypes, names))
    args = ", ".join(names[p] for p in beta)

    if extra is not None:
        fsrc, fret = extra
    elif mode == "void":
        fsrc, fret = "void f(%s) { g(%s); }" % (decl, args), "void"
    elif mode == "ret":
        fsrc, fret = "%s f(%s) { return g(%s); }" % (gret, decl, args), gret
    elif mode == "arith":
        op = "+" if k >= 0 else "-"
        fsrc = "%s f(%s) { return g(%s) %s %d; }" % (gret, decl, args, op, abs(k))
        fret = gret
    else:
        raise AssertionError(mode)

    src = "// w-alloc3 cell %s — axis %s\n%s%s\n%s\n\n%s" % (
        name,
        axis,
        PREAMBLE,
        gsrc,
        fsrc,
        ANCHOR,
    )
    return {
        "name": name,
        "axis": axis,
        "callee": callee,
        "n_caller_formals": n,
        "beta": beta,
        "mode": mode,
        # the immediate the caller's trailing `addi` must carry, already scaled
        "k_scaled": k * scale(gret) if mode == "arith" else 0,
        "gret": gret,
        "domain": domain,
        "src": src,
    }


def grid_a():
    cells = []
    # A-ret / A-arith: one-formal callee, caller formal count 1..4, every
    # bound position. The 286's own shape at (n=2, p=1).
    for n in range(1, 5):
        for p in range(n):
            cells.append(cell("A-ret-n%dp%d" % (n, p), "A-ret", "LD1", n, [p], "ret"))
            cells.append(
                cell("A-arith-n%dp%d" % (n, p), "A-arith", "LD1", n, [p], "arith", -1)
            )
    # A-void: the callee returns nothing, so TEMP's r3 branch is vacuous.
    for n in range(1, 4):
        for p in range(n):
            cells.append(cell("A-void-n%dp%d" % (n, p), "A-void", "VOID1", n, [p], "void"))
    # A-two: two callee formals, identity and swapped. `subf` is s11's shape.
    for cal in ("SUM", "SUB"):
        for b in ([0, 1], [1, 0]):
            cells.append(
                cell("A-two-%s-%d%d" % (cal, b[0], b[1]), "A-two", cal, 2, b, "ret")
            )
            cells.append(
                cell(
                    "A-twoa-%s-%d%d" % (cal, b[0], b[1]),
                    "A-two",
                    cal,
                    2,
                    b,
                    "arith",
                    7,
                )
            )
    # A-len: callee body lengths, and the s01 reproduction.
    cells.append(cell("A-ret0-add1", "A-len", "ADD1", 1, [0], "ret"))
    cells.append(cell("A-arith-add1", "A-len", "ADD1", 1, [0], "arith", 5))
    cells.append(cell("A-ret-ldc", "A-len", "LDc", 2, [1], "ret"))
    cells.append(cell("A-arith-ldc", "A-len", "LDc", 2, [1], "arith", 3))
    return cells


def grid_h():
    cells = []
    # H-wide: caller formal counts 5..8 — GRID-A tops out at 4. This is the
    # population that separates "the temp is POOL_TOP" from "the temp is the
    # lowest free volatile" at four fresh pool floors.
    for n in (5, 6, 7, 8):
        for p in (0, n - 1):
            cells.append(cell("H-wide-n%dp%d-ret" % (n, p), "H-wide", "LD1", n, [p], "ret"))
            cells.append(
                cell("H-wide-n%dp%d-ar" % (n, p), "H-wide", "LD1", n, [p], "arith", -1)
            )
    # H-perm: a THREE-formal callee at all six permutations of a three-formal
    # caller. GRID-A has no three-formal callee and no permutation past a swap.
    perms = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]]
    for b in perms:
        cells.append(
            cell("H-perm-%d%d%d" % tuple(b), "H-perm", "SUM3", 3, b, "ret")
        )
    # H-perm4: a four-formal caller feeding a three-formal callee, so one
    # caller register is live-in and unbound.
    cells.append(cell("H-perm4-310", "H-perm4", "SUM3", 4, [3, 1, 0], "ret"))
    cells.append(cell("H-perm4-023", "H-perm4", "SUM3", 4, [0, 2, 3], "arith", 9))
    # H-temp: the callee ALREADY holds a temp in r11, and the arith mode wants
    # r11 for the result. GRID-A's callees hold no temp at all.
    for n in (1, 3, 5):
        cells.append(cell("H-temp-n%d" % n, "H-temp", "LD2", n, [n - 1], "arith", -1))
    cells.append(cell("H-temp-ret-n2", "H-temp", "LD2", 2, [1], "ret"))
    # H-spell: five producer spellings GRID-A never contains. RULE W (#886) and
    # RULE W2 (#887) both died on the spelling axis, so it is in the holdout.
    cells.append(cell("H-spell-neg", "H-spell", "NEG", 2, [1], "arith", 4))
    cells.append(cell("H-spell-shl", "H-spell", "SHL", 2, [1], "arith", 4))
    cells.append(cell("H-spell-and", "H-spell", "AND", 3, [2, 0], "arith", 4))
    cells.append(cell("H-spell-mul", "H-spell", "MUL", 3, [2, 0], "arith", 4))
    cells.append(cell("H-spell-ext", "H-spell", "EXT", 2, [1], "arith", 4))
    cells.append(cell("H-spell-ext-ret", "H-spell", "EXT", 2, [1], "ret"))
    # H-noarg: a callee with NO formals, so BIND is the empty substitution.
    cells.append(cell("H-noarg-ret", "H-noarg", "NOARG", 2, [], "ret"))
    cells.append(cell("H-noarg-ar", "H-noarg", "NOARG", 2, [], "arith", 6))
    # H-idx: an indexed load — a non-D-form callee body.
    cells.append(cell("H-idx-ret", "H-idx", "IDX", 3, [2, 0], "ret"))
    cells.append(cell("H-idx-ar", "H-idx", "IDX", 3, [2, 0], "arith", 8))
    # H-store: a void callee taking TWO formals, so BIND has to rename a store's
    # source field as well as its base field.
    cells.append(cell("H-store-12", "H-store", "STORE2", 3, [1, 2], "void"))
    cells.append(cell("H-store-20", "H-store", "STORE2", 3, [2, 0], "void"))
    # H-zero-off: the bound field is at displacement 0.
    cells.append(cell("H-zero-ret", "H-zero-off", "LD1a", 2, [1], "ret"))
    cells.append(cell("H-zero-ar", "H-zero-off", "LD1a", 2, [1], "arith", -1))

    # ---- registered OUT OF DOMAIN, printed rather than skipped ---------------
    cells.append(
        cell(
            "H-out-twocall-a",
            "H-out",
            "LD1",
            2,
            [1],
            "ret",
            domain="out:D7-two-call-sites",
            extra=(
                "int* f(int x0, V* x1) { int* p = g(x1); int* q = g(x1); "
                "return p < q ? p : q; }",
                "int*",
            ),
        )
    )
    cells.append(
        cell(
            "H-out-twocall-b",
            "H-out",
            "SUM",
            2,
            [0, 1],
            "ret",
            domain="out:D7-two-call-sites",
            extra=("int f(int x0, int x1) { return g(x0, x1) + g(x1, x0); }", "int"),
        )
    )
    cells.append(
        cell(
            "H-out-computed-a",
            "H-out",
            "ADD1",
            2,
            [0],
            "ret",
            domain="out:D2-computed-actual",
            extra=("int f(int x0, int x1) { return g(x0 + 1); }", "int"),
        )
    )
    cells.append(
        cell(
            "H-out-computed-b",
            "H-out",
            "LD1",
            2,
            [1],
            "ret",
            domain="out:D2-computed-actual",
            extra=("int* f(int x0, V* x1) { return g(x1 + 1); }", "int*"),
        )
    )
    return cells


def write(outdir, tag, cells):
    root = os.path.join(outdir, tag)
    os.makedirs(root, exist_ok=True)
    manifest = []
    for c in cells:
        d = os.path.join(root, c["name"])  # a DIRECTORY PER CELL — board #1045
        os.makedirs(d, exist_ok=True)
        path = os.path.join(d, c["name"] + ".cpp")
        with open(path, "w") as fh:
            fh.write(c["src"])
        c["path"] = os.path.relpath(path, os.getcwd())
        c["sha256"] = hashlib.sha256(c["src"].encode()).hexdigest()
        manifest.append("%s  %s" % (c["sha256"], c["path"]))
    with open(os.path.join(outdir, tag + ".json"), "w") as fh:
        json.dump(cells, fh, indent=1, sort_keys=True)
    with open(os.path.join(outdir, tag + ".sha256"), "w") as fh:
        fh.write("\n".join(manifest) + "\n")
    names = [c["name"] for c in cells]
    assert len(set(names)) == len(names), "duplicate cell name"
    print("%s: %d cells, %d in domain, %d registered out of domain" % (
        tag,
        len(cells),
        sum(1 for c in cells if c["domain"] == "in"),
        sum(1 for c in cells if c["domain"] != "in"),
    ))


def main():
    out = sys.argv[1]
    write(out, "gridA", grid_a())
    write(out, "gridH", grid_h())


main()

#!/usr/bin/env python3
"""grid.py — lane w-sched. Generate the schedule cross product, compile it
through REAL c2 at the workload's flags, and write fit.tsv / holdout.tsv.

The holdout partition is decided HERE, by the rule preregistered in
`docs/rungs/_2026-08-05-w-sched-prereg.md` §4, and written to a separate file
that `fit.py` refuses to open.

A cell is a list of statements. Statement k stores to `p->m{k}` (so the member
offset 4*k names the source position unambiguously in the disassembly). Its
value is one of:

    F        a formal            -> no producer instruction
    T        `this`-like: p      -> no producer instruction (a bare mr/stw of r3)
    L<c>     the constant c      -> `li  rD, c`
    A        `(unsigned)&p->mB`  -> `addi rD, r3, 44`      (base == store base)
    Q        `(unsigned)&q->mB`  -> `addi rD, r4, 44`      (base != store base)
    I        `f0 + 1`            -> `addi rD, rF, 1`
    M        `f0 * 3`            -> `mulli rD, rF, 3`      (LONGER machine latency)
    S        `f0 << 2`           -> `rlwinm`
"""
import hashlib
import itertools
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sched_lib import compile_cod, parse_cod, classify  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))
NF = 6  # formals f0..f5

HDR = r"""
struct M { unsigned m0,m1,m2,m3,m4,m5,m6,m7,m8,m9,mA,mB; };
"""

SIG = ("(M* p, M* q, unsigned f0, unsigned f1, unsigned f2, "
       "unsigned f3, unsigned f4, unsigned f5)")


def value_expr(spec, k):
    if spec == "F":
        return "f%d" % (k % NF)
    if spec == "T":
        return "(unsigned)p"
    if spec[0] == "L":
        return spec[1:]
    if spec == "A":
        return "(unsigned)&p->mB"
    if spec == "Q":
        return "(unsigned)&q->mB"
    if spec == "I":
        return "f0 + 1"
    if spec == "M":
        return "f0 * 3"
    if spec == "S":
        return "f0 << 2"
    raise ValueError(spec)


def emit_cell(cid, specs, base=None):
    """base[k] = 'p' or 'q' for the store's destination pointer."""
    if base is None:
        base = ["p"] * len(specs)
    body = " ".join("%s->m%d = %s;" % (base[k], k, value_expr(s, k))
                    for k, s in enumerate(specs))
    return "void %s%s { %s }\n" % (cid, SIG, body)


# --------------------------------------------------------------------------
# the cells
# --------------------------------------------------------------------------
def build_cells():
    cells = {}          # cid -> (tier, specs, base)

    def add(cid, tier, specs, base=None):
        assert cid not in cells, cid
        cells[cid] = (tier, specs, base)

    # ---- tier 1: one `li` producer, one consumer, n = 2..9 ----------------
    for n in range(2, 10):
        for i in range(n):
            s = ["F"] * n
            s[i] = "L0"
            add("t1_n%d_i%d" % (n, i), 1, s)

    # ---- tier 2: producer KIND x consumer index, n = 6 --------------------
    for kind in ("A", "Q", "I", "M", "S", "T"):
        for i in range(6):
            s = ["F"] * 6
            s[i] = kind
            add("t2_%s_i%d" % (kind, i), 2, s)

    # ---- tier 3: ONE producer with TWO consumers, n = 6 -------------------
    for i, j in itertools.combinations(range(6), 2):
        s = ["F"] * 6
        s[i] = s[j] = "L0"                    # same constant -> shared `li`
        add("t3_li_%d_%d" % (i, j), 3, s)
        s = ["F"] * 6
        s[i] = s[j] = "A"                     # shared address bind
        add("t3_ab_%d_%d" % (i, j), 3, s)

    # ---- tier 4: TWO producers, distinct values, n = 6 --------------------
    for i, j in itertools.combinations(range(6), 2):
        s = ["F"] * 6
        s[i], s[j] = "L1", "L2"
        add("t4_ll_%d_%d" % (i, j), 4, s)
        s = ["F"] * 6
        s[i], s[j] = "L1", "A"
        add("t4_la_%d_%d" % (i, j), 4, s)
        s = ["F"] * 6
        s[i], s[j] = "A", "L1"
        add("t4_al_%d_%d" % (i, j), 4, s)
        s = ["F"] * 6
        s[i], s[j] = "L1", "M"
        add("t4_lm_%d_%d" % (i, j), 4, s)

    # ---- tier 5: THREE producers, n = 6 (o7's regime) ---------------------
    for i, j, k in itertools.combinations(range(6), 3):
        s = ["F"] * 6
        s[i], s[j], s[k] = "L1", "L2", "L3"
        add("t5_lll_%d_%d_%d" % (i, j, k), 5, s)

    # ---- tier 6: the BASE axis, isolated ----------------------------------
    # A  : producer base == r3 == store base.  Q : producer base == r4 != store
    # base.  Plus the may-alias split-destination variants w-pair conflated.
    for kind in ("A", "Q"):
        for i in range(4):
            s = ["F"] * 4
            s[i] = kind
            add("t6_%s_1c_i%d" % (kind, i), 6, s)          # one consumer
        for i, j in itertools.combinations(range(4), 2):
            s = ["F"] * 4
            s[i] = s[j] = kind
            add("t6_%s_2c_%d_%d" % (kind, i, j), 6, s)     # two consumers
    # split destinations: stores alternate p / q  (may-alias => no reorder)
    for kind in ("A", "Q"):
        for pat in ("pppp", "pqqp", "ppqq", "qppq"):
            s = ["F"] * 4
            s[0] = s[3] = kind
            add("t6_%s_split_%s" % (kind, pat), 6, s, list(pat))
    # w-pair's F1/F2 controlled swap, with the STORE base held fixed
    for i in range(4):
        s = ["F"] * 4
        s[i] = "A"
        add("t6_g_pfix_i%d" % i, 6, s, ["p", "q", "q", "p"])

    # ---- tier 7: invariance controls (R4) ---------------------------------
    # non-contiguous offsets, a different literal, `this`-valued stores
    for i in range(6):
        s = ["F"] * 6
        s[i] = "L7"
        add("t7_lit7_i%d" % i, 7, s)
    for i in range(6):
        s = ["T"] * 6
        s[i] = "L0"
        add("t7_this_i%d" % i, 7, s)

    return cells


# --------------------------------------------------------------------------
def held_out(cid, tier, specs):
    if int(hashlib.sha1(cid.encode()).hexdigest(), 16) % 4 == 0:
        return "hash"
    if tier == 1 and len(specs) == 7:
        return "t1n7"
    if tier == 5:
        return "t5"
    if tier == 3:
        idx = [k for k, s in enumerate(specs) if s != "F"]
        if len(idx) == 2 and idx[1] - idx[0] >= 3:
            return "t3far"
    return None


# --------------------------------------------------------------------------
def canon(ann):
    """Emitted sequence -> canonical token list.

    A store is `S<k>` (k from the member offset). A non-store is `P<k>` where k
    is the source index of the FIRST later store that reads its destination
    register — that is what makes it a producer. Anything unclaimed is `?<mn>`.
    """
    toks = []
    for pos, d in enumerate(ann):
        if d["role"] == "store":
            toks.append("S%d" % (d["off"] // 4))
        elif d["mn"] == "blr":
            continue
        else:
            dst = d.get("dst")
            tag = None
            if dst:
                for e in ann[pos + 1:]:
                    if e["role"] == "store" and e.get("src") == dst:
                        tag = e["off"] // 4
                        break
            toks.append("P%d" % tag if tag is not None else "?" + d["mn"])
    return toks


def main():
    cells = build_cells()
    src = os.path.join(W, "grid.cpp")
    with open(src, "w") as f:
        f.write(HDR)
        for cid, (tier, specs, base) in cells.items():
            f.write(emit_cell(cid, specs, base))
    txt = compile_cod(src, os.path.join(W, "grid.cod"),
                      os.path.join(W, "grid.obj"))
    fns = parse_cod(txt)

    missing = [c for c in cells if c not in fns]
    if missing:
        raise SystemExit("FAIL: %d cells produced no PROC: %s"
                         % (len(missing), missing[:5]))

    rows_fit, rows_ho = [], []
    for cid, (tier, specs, base) in cells.items():
        toks = canon(classify(fns[cid]))
        bad = [t for t in toks if t.startswith("?")]
        row = "\t".join([cid, str(tier), ",".join(specs),
                         "".join(base or ["p"] * len(specs)),
                         " ".join(toks), ";".join(bad)])
        (rows_ho if held_out(cid, tier, specs) else rows_fit).append(row)

    hdr = "cid\ttier\tspecs\tbase\temitted\tunclaimed\n"
    open(os.path.join(W, "fit.tsv"), "w").write(hdr + "\n".join(rows_fit) + "\n")
    open(os.path.join(W, "holdout.tsv"), "w").write(hdr + "\n".join(rows_ho) + "\n")

    nbad = sum(1 for cid, (t, s, b) in cells.items()
               if any(x.startswith("?") for x in canon(classify(fns[cid]))))
    print("cells generated : %d" % len(cells))
    print("PROCs parsed    : %d" % len(fns))
    print("fit rows        : %d" % len(rows_fit))
    print("holdout rows    : %d" % len(rows_ho))
    print("cells with an UNCLAIMED instruction: %d" % nbad)


if __name__ == "__main__":
    main()

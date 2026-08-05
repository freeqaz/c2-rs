#!/usr/bin/env python3
"""grid.py — lane w-alloc. Generate the ALLOCATION cross product, compile it
through REAL c2 at the workload's flags, and write fit.tsv / holdout.tsv.

The holdout partition is decided HERE, by the rule preregistered in
`docs/rungs/_2026-08-05-w-alloc-prereg.md` §6, and written to a separate file
that `search.py` and `model.py` refuse to open.

The unit of observation is the FULL EMITTED SEQUENCE **with the registers** —
that is strictly more than `w-sched` recorded, and it is the whole point: this
lane is scored on which register each producer got, not on the permutation.

A cell is (signature formals, statement list). Statement k stores to `p->m{k}`.
Its value is one of:

    F<i>     formal i                -> no producer
    T        `p`                     -> no producer
    V<j>     producer j              -> one materialising instruction, shared by
                                       every statement naming the same j
and the KIND of producer j is drawn per-cell from
    li / addi-from-base / addi-from-formal / mulli / rlwinm / lis+ori
"""
import hashlib
import itertools
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from alloc_lib import compile_cod, parse_cod, classify  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

HDR = ("struct M { unsigned " +
       ",".join("m%X" % i for i in range(20)) + "; };\n")

# member k is named by its HEX digit, so the name always says the source
# position and never collides once k passes 9.
MEM = "%X".__mod__

# producer kinds: index j -> a C expression that needs exactly one instruction
KINDS = {
    "L": lambda j: "%d" % (1 + j),                       # li
    "A": lambda j: "(unsigned)&p->m%X" % (0x13 - j),     # addi rD,r3,k
    "I": lambda j: "(f0 + %d)" % (1 + j),                # addi rD,rF,k
    "M": lambda j: "(f0 * %d)" % (3 + 2 * j),            # mulli
    "S": lambda j: "(f0 << %d)" % (1 + j),               # rlwinm
    "W": lambda j: "%d" % (100000 + j),                  # lis+ori (WIDE)
}


def sig(nf):
    ps = ["M* p"] + ["unsigned f%d" % i for i in range(nf)]
    return "(" + ", ".join(ps) + ")"


def value_expr(spec, kind):
    if spec[0] == "F":
        return "f%s" % spec[1:]
    if spec == "T":
        return "(unsigned)p"
    return KINDS[kind](int(spec[1:]))


def emit_cell(cid, nf, specs, kind):
    body = " ".join("p->m%X = %s;" % (k, value_expr(s, kind))
                    for k, s in enumerate(specs))
    return "void %s%s { %s }\n" % (cid, sig(nf), body)


# --------------------------------------------------------------------------
def nprod(specs):
    return len({s for s in specs if s[0] == "V"})


def shared_set(specs):
    c = {}
    for s in specs:
        if s[0] == "V":
            c[s] = c.get(s, 0) + 1
    return {v for v, n in c.items() if n >= 2}


# --------------------------------------------------------------------------
def build_cells():
    cells = {}                      # cid -> (tier, nf, specs, kind)

    def add(cid, tier, nf, specs, kind="L"):
        assert cid not in cells, cid
        assert len(specs) <= 20, cid
        cells[cid] = (tier, nf, specs, kind)

    # ---- tier 1: SHARED vs SIMPLE, two producers, every use pattern --------
    # every word over {V0,V1} of length 2..5 that uses both letters.
    for n in range(2, 6):
        for w in itertools.product("01", repeat=n):
            if len(set(w)) < 2:
                continue
            specs = ["V" + c for c in w]
            add("t1_%s" % "".join(w), 1, 1, specs)

    # ---- tier 2: THREE producers, every use pattern at n = 3..5 -----------
    for n in range(3, 6):
        for w in itertools.product("012", repeat=n):
            if len(set(w)) < 3:
                continue
            specs = ["V" + c for c in w]
            add("t2_%s" % "".join(w), 2, 1, specs)

    # ---- tier 3: producers MIXED with formals, n = 5 (HELD OUT) -----------
    # the regime H1's two-clause ordering is most exposed on.
    for w in itertools.product("01F", repeat=5):
        if "0" not in w or "1" not in w or "F" not in w:
            continue
        specs = ["V0" if c == "0" else "V1" if c == "1" else "F0" for c in w]
        add("t3_%s" % "".join(w), 3, 1, specs)

    # ---- tier 4: producer KIND invariance ---------------------------------
    # the same use pattern under every kind; if the register depends on the
    # instruction, H1 is a machine model and not an allocation rule.
    for kind in KINDS:
        for pat in ("01", "010", "0101", "0011", "01201", "01021", "012"):
            specs = ["V" + c for c in pat]
            add("t4_%s_%s" % (kind, pat), 4, 1, specs, kind)

    # ---- tier 5: FOUR producers — pool pressure (HELD OUT) ----------------
    for w in ("0123", "01230", "01231", "01232", "012301", "012310",
              "01233", "00123", "01123", "01223", "0123012", "0120313",
              "0123123", "01230123"):
        add("t5_%s" % w, 5, 1, ["V" + c for c in w])

    # ---- tier 6: FORMAL PRESSURE — the pool descends into live-ins --------
    # nf formals held live by trailing stores, so the top of the pool is eaten
    # from below.  nf >= 6 is HELD OUT.
    for nf in range(0, 9):
        for pat in ("01", "010", "0101", "012", "01201"):
            specs = ["V" + c for c in pat] + ["F%d" % i for i in range(nf)]
            add("t6_nf%d_%s" % (nf, pat), 6, max(nf, 1), specs)

    # ---- tier 7: FIVE and SIX producers — past the pool -------------------
    for w in ("01234", "012340", "0123401234", "012345", "0123450",
              "012345012345", "0123456"):
        add("t7_%s" % w, 7, 1, ["V" + c for c in w])

    # ---- tier 8: the SPAN axis — does a shared producer's REACH matter? ---
    # V0 shared across a growing gap of formal stores.
    for gap in range(0, 6):
        specs = ["V0"] + ["F0"] * gap + ["V0", "V1", "V1"]
        add("t8_gap%d" % gap, 8, 1, specs)
        specs = ["V0", "V1"] + ["F0"] * gap + ["V0", "V1"]
        add("t8_int%d" % gap, 8, 1, specs)

    # ---- tier 9: `this`-valued stores as the unproduced filler ------------
    for pat in ("01", "010", "0101", "01201"):
        for k in (1, 2, 3):
            specs = ["V" + c for c in pat] + ["T"] * k
            add("t9_%s_t%d" % (pat, k), 9, 1, specs)

    return cells


# --------------------------------------------------------------------------
def held_out(cid, tier, nf, specs):
    """PREREGISTERED, docs/rungs/_2026-08-05-w-alloc-prereg.md §6."""
    if int(hashlib.sha1(cid.encode()).hexdigest(), 16) % 4 == 0:
        return "hash"
    if tier == 3:
        return "t3"
    if nprod(specs) == 4:
        return "p4"
    if tier == 6 and nf >= 6:
        return "nf6"
    return None


# --------------------------------------------------------------------------
def canon(ann):
    """Emitted sequence -> canonical token list, WITH REGISTERS.

    A store is `S<k>@<reg>` (k from the member offset, reg the data source).
    A producer is `P<reg>`.  Anything not classified is `?<mn>`.
    """
    toks = []
    for d in ann:
        if d["mn"] == "blr":
            continue
        if d["role"] == "store":
            toks.append("S%d@%s" % (d["off"] // 4, d["src"]))
        elif d.get("dst"):
            toks.append("P%s" % d["dst"])
        else:
            toks.append("?" + d["mn"])
    return toks


def main():
    cells = build_cells()
    src = os.path.join(W, "grid.cpp")
    with open(src, "w") as f:
        f.write(HDR)
        for cid, (tier, nf, specs, kind) in cells.items():
            f.write(emit_cell(cid, nf, specs, kind))
    txt = compile_cod(src, os.path.join(W, "grid.cod"),
                      os.path.join(W, "grid.obj"))
    fns = parse_cod(txt)

    missing = [c for c in cells if c not in fns]
    if missing:
        raise SystemExit("FAIL: %d cells produced no PROC: %s"
                         % (len(missing), missing[:5]))

    rows_fit, rows_ho, nbad = [], [], 0
    for cid, (tier, nf, specs, kind) in cells.items():
        toks = canon(classify(fns[cid]))
        bad = [t for t in toks if t.startswith("?")]
        nbad += bool(bad)
        row = "\t".join([cid, str(tier), str(nf), ",".join(specs), kind,
                         " ".join(toks), ";".join(bad)])
        (rows_ho if held_out(cid, tier, nf, specs) else rows_fit).append(row)

    hdr = "cid\ttier\tnf\tspecs\tkind\temitted\tunclaimed\n"
    open(os.path.join(W, "fit.tsv"), "w").write(hdr + "\n".join(rows_fit) + "\n")
    open(os.path.join(W, "holdout.tsv"), "w").write(hdr + "\n".join(rows_ho) + "\n")

    print("cells generated : %d" % len(cells))
    print("PROCs parsed    : %d" % len(fns))
    print("fit rows        : %d" % len(rows_fit))
    print("holdout rows    : %d" % len(rows_ho))
    print("cells with an UNCLAIMED instruction: %d" % nbad)
    per = {}
    for cid, (tier, nf, specs, kind) in cells.items():
        per[tier] = per.get(tier, 0) + 1
    print("per tier: " + "  ".join("t%d %d" % kv for kv in sorted(per.items())))


if __name__ == "__main__":
    main()

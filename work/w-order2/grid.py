#!/usr/bin/env python3
"""grid.py — lane w-order2. Generate the ORDER cross product, compile it
through REAL c2 at the workload's flags, and write fit.tsv / holdout.tsv.

The holdout partition is decided HERE, by the rule preregistered in
`docs/rungs/_2026-08-05-w-order2-prereg.md` §6, and written to a separate file
that `search.py` refuses to open and `model.py` opens only under --holdout.

The unit of observation is the FULL EMITTED SEQUENCE **with the registers**,
as `w-alloc` recorded it -- `w-sched`'s canon threw the registers away and
that is the single reason its 504 cells could not answer this question.

A cell is (formal count, statement list, producer kind). Statement k stores to
`p->m{k}`. Its value is one of

    F<i>     formal i             -> no producer, a distinct live-in register
    T        `p`                  -> no producer, the base pointer itself
    V<j>     producer j           -> one materialising instruction, shared by
                                    every statement naming the same j

The signature is kept to at most THREE formals on purpose: `w-alloc` found
`w-sched`'s `(M* p, M* q, unsigned f0..f5)` put `f4`->r9 and `f5`->r10 INSIDE
the scratch pool and ate it from below, so that grid's "conflicted" cells were
partly an artifact of its own probe signature. With nf <= 3 the pool is
r11..r7 and no cell here can create the effect it is measuring.
"""
import hashlib
import itertools
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                "..", "w-alloc"))
from alloc_lib import compile_cod, parse_cod, classify  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

HDR = ("struct M { unsigned " +
       ",".join("m%X" % i for i in range(20)) + "; };\n")

KINDS = {
    "L": lambda j: "%d" % (1 + j),                       # li
    "A": lambda j: "(unsigned)&p->m%X" % (0x13 - j),     # addi rD,r3,k
    "I": lambda j: "(f0 + %d)" % (1 + j),                # addi rD,rF,k
    "S": lambda j: "(f0 << %d)" % (1 + j),               # rlwinm
}


def sig(nf):
    return "(" + ", ".join(["M* p"] + ["unsigned f%d" % i for i in range(nf)]) + ")"


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
def canon_words(nsym, n):
    """Every surjective word of length n onto `nsym` letters, in canonical
    first-occurrence order (so 'abab' is generated and 'baba' is not)."""
    out = []
    for w in itertools.product(range(nsym), repeat=n):
        if len(set(w)) != nsym:
            continue
        seen, m = [], {}
        for c in w:
            if c not in m:
                m[c] = len(m)
        if [m[c] for c in w] != list(w):
            continue
        out.append("".join(str(c) for c in w))
    return out


def interleave(word, k):
    """Every way to place k filler stores among len(word) produced stores."""
    n = len(word) + k
    return [set(c) for c in itertools.combinations(range(n), k)]


def build_cells():
    cells = {}

    def add(cid, tier, nf, specs, kind="L"):
        assert cid not in cells, cid
        assert len(specs) <= 20, cid
        cells[cid] = (tier, nf, specs, kind)

    # ---- tier 1: all-produced, TWO producers, lengths 2..6 ----------------
    for n in range(2, 7):
        for w in canon_words(2, n):
            add("t1_%s" % w, 1, 1, ["V" + c for c in w])

    # ---- tier 2: all-produced, THREE producers, lengths 3..6 -------------
    for n in range(3, 7):
        for w in canon_words(3, n):
            add("t2_%s" % w, 2, 1, ["V" + c for c in w])

    # ---- tier 3: UNPRODUCED fillers, every interleaving ------------------
    # k distinct formals interleaved into a produced word: the regime where
    # clause (a) and clause (b) are both live at once.
    for base in canon_words(2, 3) + canon_words(2, 4):
        for k in (1, 2, 3):
            for slots in interleave(base, k):
                specs, wi, fi = [], 0, 0
                for i in range(len(base) + k):
                    if i in slots:
                        specs.append("F%d" % fi)
                        fi += 1
                    else:
                        specs.append("V" + base[wi])
                        wi += 1
                cid = "t3_%s_%s" % (base, "".join(
                    "F" if i in slots else "v" for i in range(len(base) + k)))
                add(cid, 3, k, specs)

    # ---- tier 4: the SAME formal in every filler slot ---------------------
    # does an unproduced store's VALUE identity matter, or only that it needs
    # no instruction?  Same shapes as tier 3 at k = 2, one formal.
    for base in canon_words(2, 3):
        for slots in interleave(base, 2):
            specs = []
            wi = 0
            for i in range(len(base) + 2):
                if i in slots:
                    specs.append("F0")
                else:
                    specs.append("V" + base[wi])
                    wi += 1
            add("t4_%s_%s" % (base, "".join(
                "F" if i in slots else "v" for i in range(len(base) + 2))),
                4, 1, specs)

    # ---- tier 5: producer KIND invariance for the ORDER ------------------
    # w-sched measured kind-invariance for the permutation; w-alloc found the
    # kind DOES matter for the register. Which is it for clause (b)?
    for kind in ("L", "A", "I", "S"):
        for w in ("01", "011", "0011", "0101", "01011", "01122", "01022",
                  "012", "0122", "01202"):
            add("t5_%s_%s" % (kind, w), 5, 1, ["V" + c for c in w], kind)

    # ---- tier 6: `this` as the filler ------------------------------------
    for base in canon_words(2, 3):
        for k in (1, 2):
            for slots in interleave(base, k):
                specs, wi = [], 0
                for i in range(len(base) + k):
                    if i in slots:
                        specs.append("T")
                    else:
                        specs.append("V" + base[wi])
                        wi += 1
                add("t6_%s_%s" % (base, "".join(
                    "T" if i in slots else "v" for i in range(len(base) + k))),
                    6, 1, specs)

    # ---- tier 7: LONG runs, 7 and 8 statements ---------------------------
    for w in ("0101010", "0011122", "0120120", "0122011", "0112200",
              "0101011", "0110022", "0121122", "0012012", "0102010",
              "01010101", "00111222", "01201201", "01122001", "01012012",
              "00112233"[:8], "01230123", "01102233"):
        if max(int(c) for c in w) > 2:
            add("t7_%s" % w, 7, 1, ["V" + c for c in w])
        else:
            add("t7_%s" % w, 7, 1, ["V" + c for c in w])

    # ---- tier 8: FOUR producers — past ALLOC's domain, order only --------
    for w in ("0123", "01231", "01232", "012312", "012301", "0123123",
              "0123012", "01231230", "0123321", "01230123"):
        add("t8_%s" % w, 8, 1, ["V" + c for c in w])

    # ---- tier 9: fillers AFTER the produced run, and around it -----------
    for base in canon_words(2, 3) + canon_words(3, 3):
        for tail in (1, 2):
            add("t9_%s_tail%d" % (base, tail), 9, tail,
                ["V" + c for c in base] + ["F%d" % i for i in range(tail)])
        for head in (1, 2):
            add("t9_%s_head%d" % (base, head), 9, head,
                ["F%d" % i for i in range(head)] + ["V" + c for c in base])

    return cells


# --------------------------------------------------------------------------
def counts(specs):
    c = {}
    for s in specs:
        if s[0] == "V":
            c[s] = c.get(s, 0) + 1
    return sorted(c.values(), reverse=True)


def held_out(cid, tier, nf, specs):
    """PREREGISTERED, docs/rungs/_2026-08-05-w-order2-prereg.md §6."""
    if int(hashlib.sha1(cid.encode()).hexdigest(), 16) % 3 == 0:
        return "hash"
    c = counts(specs)
    if len(c) >= 2 and c[0] == c[1] and c[0] >= 2:
        return "tie"
    if len(specs) >= 7:
        return "long"
    if tier == 6:
        return "this"
    return None


# --------------------------------------------------------------------------
def canon(ann):
    """Emitted sequence -> canonical token list, WITH REGISTERS."""
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
    per, why = {}, {}
    for cid, (tier, nf, specs, kind) in cells.items():
        per[tier] = per.get(tier, 0) + 1
        h = held_out(cid, tier, nf, specs)
        why[h] = why.get(h, 0) + 1
    print("per tier : " + "  ".join("t%d %d" % kv for kv in sorted(per.items())))
    print("holdout  : " + "  ".join("%s %d" % (k, v) for k, v in
                                    sorted(why.items(), key=lambda x: str(x[0]))))


if __name__ == "__main__":
    main()

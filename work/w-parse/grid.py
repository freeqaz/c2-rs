#!/usr/bin/env python3
"""grid.py — lane w-parse. The FOUR axes `w-order2`'s grid confounds.

`docs/ORDER.md` §5 records `xboxheap.cpp` as differing from eight cells of
`w-order2`'s grid on ONE named axis — the number of base symbols. It differs on
**four**, and only one of them was declared:

  1. FILL   the unproduced fillers are `this` twice and a formal once;
            `w-order2`'s tier-3 cells use three DISTINCT formals and its
            tier-6 `this` cells stop at two fillers.
  2. KINDS  its two producers have DIFFERENT kinds (a constant `li` and a
            base-derived `addi`); every cell of `w-order2`'s grid uses ONE
            kind for every producer of that cell (its tier 5 crosses the kind
            but never mixes two inside a cell).
  3. SYM    its last two stores go through a bound reference, a second base
            symbol.
  4. SELF   the address-valued producer's value IS that second symbol.

This grid crosses all four on `xboxheap`'s own statement word and then
generalises off it.  The unit of observation is the FULL EMITTED SEQUENCE with
the registers and with each store's own BASE register, compiled by real
`c2.dll` under wibo at the WORKLOAD's flags through `cl /FAsc`.

The holdout partition is decided HERE by the rule preregistered in
`docs/rungs/_2026-08-05-w-parse-prereg.md` §6, and written to a file the
fitter RAISES on opening.

Imports are by explicit file path (docs/ORDER.md §6's trap).
"""
import hashlib
import importlib.util
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(W, "..", ".."))


def load_by_path(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    sys.modules[name] = m
    spec.loader.exec_module(m)
    return m


LIB = load_by_path("wparse_alloc_lib",
                   os.path.join(REPO, "work", "w-alloc", "alloc_lib.py"))

# ---------------------------------------------------------------- the model --
# struct M { unsigned m0..m9; E e; unsigned t0..t3; };
#   m0..m9 at 0x00..0x24      symbol-0 store destinations, statement k -> m<k>
#   e      at 0x28            the sub-object a reference can bind to
#   e0..e7 at 0x28..0x44      symbol-1 store destinations, statement k -> e<k>
#   t0..t3 at 0x48..0x54      address-producer targets, never stored to
E_OFF = 0x28
T_OFF = 0x48
HDR = ("struct E { unsigned " + ",".join("e%d" % i for i in range(8)) + "; };\n"
       "struct M { unsigned " + ",".join("m%d" % i for i in range(10)) + "; "
       "E e; unsigned " + ",".join("t%d" % i for i in range(4)) + "; };\n")


def store_dest(sym, k):
    return "l.e%d" % k if sym else "p->m%d" % k


def slot_of(off):
    """obj byte offset -> (sym, statement index). The inverse of store_dest."""
    if off < E_OFF:
        return 0, off // 4
    if off < T_OFF:
        return 1, (off - E_OFF) // 4
    raise ValueError("offset %#x is not a store destination" % off)


KINDS = {
    "L": lambda j: "%d" % (1 + j),                  # li rD,k
    "A": lambda j: "(unsigned)&p->t%d" % j,         # addi rD,r3,0x48+4j
    "R": lambda j: "(unsigned)&p->e",               # addi rD,r3,0x28 == &l
    "I": lambda j: "(f0 + %d)" % (1 + j),           # addi rD,r4,k
    "S": lambda j: "(f0 << %d)" % (1 + j),          # rlwinm
}


def value_expr(spec, kinds):
    if spec == "T":
        return "(unsigned)p"
    if spec[0] == "F":
        return "f%s" % spec[1:]
    j = int(spec[1:])
    return KINDS[kinds[j]](j)


def emit_cell(cid, nf, specs, kinds, syms):
    sig = "(" + ", ".join(["M* p"] + ["unsigned f%d" % i
                                      for i in range(nf)]) + ")"
    body = []
    if any(syms):
        body.append("E& l = p->e;")
    for k, s in enumerate(specs):
        body.append("%s = %s;" % (store_dest(syms[k], k), value_expr(s, kinds)))
    return "void %s%s { %s }\n" % (cid, sig, " ".join(body))


# ----------------------------------------------------------------- the grid --
def canon_words(nsym, n):
    out = []
    for w in itertools.product(range(nsym), repeat=n):
        if len(set(w)) != nsym:
            continue
        m = {}
        for c in w:
            if c not in m:
                m[c] = len(m)
        if [m[c] for c in w] != list(w):
            continue
        out.append("".join(str(c) for c in w))
    return out


def build_cells():
    cells = {}

    def add(cid, tier, nf, specs, kinds, syms):
        assert cid not in cells, cid
        assert len(specs) == len(syms), cid
        cells[cid] = (tier, nf, specs, kinds, syms)

    # ---- tier X: xboxheap's own word, all four axes crossed --------------
    # specs [x0, x1, V0, x2, V1, V1]; V0 used once and FIRST, V1 used twice.
    # ORDER's rank puts V1 first (count 2); xboxheap emits V0 first.
    FILLS = {"FFF": ["F0", "F1", "F2"],
             "FTT": ["F0", "T", "T"],        # xboxheap's own
             "TTT": ["T", "T", "T"]}
    for fname, fill in FILLS.items():
        nf = sum(1 for f in fill if f[0] == "F")
        specs = [fill[0], fill[1], "V0", fill[2], "V1", "V1"]
        for ka in ("L", "A"):
            for kb in ("L", "A", "R"):
                for sym in (0, 1):
                    if kb == "R" and not sym:
                        # `&p->e` with the stores through `p->e` directly is
                        # the SELF case at one symbol -- keep it, it is the
                        # cell that separates SELF from SYM.
                        pass
                    syms = [0, 0, 0, 0, sym, sym]
                    add("x_%s_%s%s_s%d" % (fname, ka, kb, sym),
                        "X", nf, specs, [ka, kb], syms)

    # ---- tier Y: the same four axes on SHORTER words ---------------------
    # base word "011" (V0 once first, V1 twice) with k fillers, k = 1,2.
    for k in (1, 2):
        for slots in itertools.combinations(range(3 + k), k):
            for fkind in ("F", "T"):
                for ka in ("L", "A"):
                    for kb in ("L", "A", "R"):
                        for sym in (0, 1):
                            specs, wi, fi = [], 0, 0
                            base = "011"
                            for i in range(3 + k):
                                if i in slots:
                                    if fkind == "F":
                                        specs.append("F%d" % fi)
                                        fi += 1
                                    else:
                                        specs.append("T")
                                else:
                                    specs.append("V" + base[wi])
                                    wi += 1
                            syms = [1 if (sym and s == "V1") else 0
                                    for s in specs]
                            cid = "y_%s%s_%s%s%s_s%d" % (
                                "".join("f" if i in slots else "v"
                                        for i in range(3 + k)),
                                fkind, base[0], ka, kb, sym)
                            cid = "y_%s_%s_%s%s_s%d" % (
                                "".join("f" if i in slots else "v"
                                        for i in range(3 + k)),
                                fkind, ka, kb, sym)
                            if cid in cells:
                                continue
                            add(cid, "Y", fi, specs, [ka, kb], syms)

    # ---- tier Z: kind mixture ALONE, one symbol, no fillers --------------
    # every 2-producer word of length 2..5, every ordered kind pair.
    for n in range(2, 6):
        for w in canon_words(2, n):
            for ka in ("L", "A", "I", "S"):
                for kb in ("L", "A", "I", "S"):
                    if ka == kb and ka not in ("L", "A"):
                        continue
                    specs = ["V" + c for c in w]
                    add("z_%s_%s%s" % (w, ka, kb), "Z", 1, specs,
                        [ka, kb], [0] * n)

    # ---- tier S: the SYMBOL axis ALONE, one kind ------------------------
    # every 2-producer word of length 2..5, all-L producers, the V1 stores
    # through the bound reference.
    for n in range(2, 6):
        for w in canon_words(2, n):
            specs = ["V" + c for c in w]
            for sym in (0, 1):
                syms = [1 if (sym and s == "V1") else 0 for s in specs]
                add("s_%s_s%d" % (w, sym), "S", 1, specs, ["L", "L"], syms)

    # ---- tier T: filler identity ALONE ----------------------------------
    # `this` vs a formal in each filler slot, one kind, one symbol.
    for k in (1, 2, 3):
        for slots in itertools.combinations(range(3 + k), k):
            for mask in itertools.product("FT", repeat=k):
                specs, wi, fi = [], 0, 0
                base = "011"
                mi = 0
                for i in range(3 + k):
                    if i in slots:
                        if mask[mi] == "F":
                            specs.append("F%d" % fi)
                            fi += 1
                        else:
                            specs.append("T")
                        mi += 1
                    else:
                        specs.append("V" + base[wi])
                        wi += 1
                cid = "t_%s_%s" % ("".join("f" if i in slots else "v"
                                           for i in range(3 + k)),
                                   "".join(mask))
                add(cid, "T", fi, specs, ["L", "L"], [0] * (3 + k))

    return cells


# --------------------------------------------------------------- partition --
def n_producers(specs):
    return len({s for s in specs if s[0] == "V"})


def held_out(cid, specs, kinds):
    """PREREGISTERED — docs/rungs/_2026-08-05-w-parse-prereg.md §6."""
    if hashlib.md5(cid.encode()).hexdigest()[0] in "012345":
        return "hash"
    if len(specs) > 5:
        return "long"
    if n_producers(specs) >= 3:
        return "prod3"
    if len(set(kinds[:n_producers(specs)])) >= 3:
        return "kind3"
    return None


# ------------------------------------------------------------------- canon --
def canon(ann):
    """Emitted sequence -> canonical token list.

    `S<sym>.<k>@<value reg>/<base reg>` for a store, `P<reg>` for anything
    that writes a register, `?<mn>` for anything unclaimed.
    """
    toks = []
    for d in ann:
        if d["mn"] == "blr":
            continue
        if d["role"] == "store":
            try:
                sym, k = slot_of(d["off"])
            except ValueError:
                toks.append("?off%x" % d["off"])
                continue
            toks.append("S%d.%d@%s/%s" % (sym, k, d["src"], d["base"]))
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
        for cid, (tier, nf, specs, kinds, syms) in cells.items():
            f.write(emit_cell(cid, nf, specs, kinds, syms))
    txt = LIB.compile_cod(src, os.path.join(W, "grid.cod"),
                          os.path.join(W, "grid.obj"))
    fns = LIB.parse_cod(txt)

    missing = [c for c in cells if c not in fns]
    if missing:
        raise SystemExit("FAIL: %d of %d cells produced no PROC: %s"
                         % (len(missing), len(cells), missing[:5]))

    rows_fit, rows_ho, nbad = [], [], 0
    for cid, (tier, nf, specs, kinds, syms) in cells.items():
        toks = canon(LIB.classify(fns[cid]))
        bad = [t for t in toks if t.startswith("?")]
        nbad += bool(bad)
        row = "\t".join([cid, str(tier), str(nf), ",".join(specs),
                         "".join(kinds), "".join(str(s) for s in syms),
                         " ".join(toks), ";".join(bad)])
        (rows_ho if held_out(cid, specs, kinds) else rows_fit).append(row)

    hdr = "cid\ttier\tnf\tspecs\tkinds\tsyms\temitted\tunclaimed\n"
    open(os.path.join(W, "fit.tsv"), "w").write(hdr + "\n".join(rows_fit) + "\n")
    open(os.path.join(W, "holdout.tsv"), "w").write(
        hdr + "\n".join(rows_ho) + "\n")

    print("cells generated : %d" % len(cells))
    print("PROCs parsed    : %d" % len(fns))
    print("fit rows        : %d" % len(rows_fit))
    print("holdout rows    : %d" % len(rows_ho))
    print("cells with an UNCLAIMED instruction: %d" % nbad)
    per, why = {}, {}
    for cid, (tier, nf, specs, kinds, syms) in cells.items():
        per[tier] = per.get(tier, 0) + 1
        h = held_out(cid, specs, kinds)
        why[h] = why.get(h, 0) + 1
    print("per tier : " + "  ".join("%s %d" % kv for kv in sorted(per.items())))
    print("holdout  : " + "  ".join("%s %d" % (k, v) for k, v in
                                    sorted(why.items(), key=lambda x: str(x[0]))))
    if nbad:
        print("NOTE: %d cells carry an instruction the canon does not claim; "
              "they are scored as OUT of domain, never as hits." % nbad)


if __name__ == "__main__":
    main()

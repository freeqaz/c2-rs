#!/usr/bin/env python3
"""grid2.py — lane w-parse, the SECOND grid: what the base symbol actually does.

`grid.py` settled which of the four confounded axes carries `xboxheap`'s
divergence: it is the **base symbol**, and the producer kind, the filler
identity and the self-reference all move nothing.  This grid asks the next
question, which `grid.py` cannot separate because every one of its cells put
*all* of one producer's stores through the second symbol:

  * is the pinning **PAIRWISE** (SCHED §3 — two stores through different
    symbols may not be reordered past each other, and everything else is
    scheduled as usual), or is it **GLOBAL** (a run through more than one
    symbol is emitted in source order, entire), or is it **PER-SYMBOL** (each
    symbol's sub-run is scheduled by ORDER on its own and the sub-runs are
    merged in source order)?

The three agree on every cell of `grid.py`.  They disagree on a run whose
symbol-0 sub-run is one ORDER would reorder internally, which is what tier M
builds.

Two controls the first grid also could not carry:

  * **SYMFORM** — `E& l = p->e; l.eK = v;` against `p->e.eK = v;`.  The same
    destination bytes and the same offsets, one IL symbol apart.  Without it
    "the symbol" is confounded with "the offset range".
  * **SYMKIND** — a bound reference against a second pointer FORMAL `M* q`.

Holdout partition: the rule preregistered in
`docs/rungs/_2026-08-05-w-parse-prereg.md` §6, applied by the generator.
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


LIB = load_by_path("wparse_alloc_lib2",
                   os.path.join(REPO, "work", "w-alloc", "alloc_lib.py"))
G1 = load_by_path("wparse_grid1", os.path.join(W, "grid.py"))

E_OFF, T_OFF = G1.E_OFF, G1.T_OFF
HDR = G1.HDR


def slot_of(off):
    return G1.slot_of(off)


def emit_cell(cid, nf, specs, kinds, syms, symform, symkind):
    """symform in {'ref','direct'}; symkind in {'sub','formal'}."""
    params = ["M* p"]
    if symkind == "formal":
        params.append("M* q")
    params += ["unsigned f%d" % i for i in range(nf)]
    body = []
    if symkind == "sub" and symform == "ref" and any(syms):
        body.append("E& l = p->e;")
    for k, s in enumerate(specs):
        if not syms[k]:
            dest = "p->m%d" % k
        elif symkind == "formal":
            dest = "q->m%d" % k
        elif symform == "ref":
            dest = "l.e%d" % k
        else:
            dest = "p->e.e%d" % k
        body.append("%s = %s;" % (dest, G1.value_expr(s, kinds)))
    sig = "(" + ", ".join(params) + ")"
    return "void %s%s { %s }\n" % (cid, sig, " ".join(body))


def canon(ann, symkind):
    """`S<base>.<k>@<value reg>/<base reg>`.

    With symkind='formal' the two symbols land in DIFFERENT base registers and
    share the offset range, so the base register is what names the symbol;
    with 'sub' they share the base register and the offset range names it.
    """
    toks = []
    for d in ann:
        if d["mn"] == "blr":
            continue
        if d["role"] == "store":
            if symkind == "formal":
                sym = 0 if d["base"] == "r3" else 1
                k = d["off"] // 4
                if d["off"] >= E_OFF:
                    toks.append("?off%x" % d["off"])
                    continue
            else:
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


def build_cells():
    cells = {}

    def add(cid, tier, nf, specs, kinds, syms, symform, symkind):
        assert cid not in cells, cid
        cells[cid] = (tier, nf, specs, kinds, syms, symform, symkind)

    # ---- tier M: arbitrary symbol masks over a produced word -------------
    # The discriminating shape: a symbol-0 sub-run ORDER reorders internally.
    for n in (3, 4, 5):
        for w in G1.canon_words(2, n):
            specs = ["V" + c for c in w]
            for mask in itertools.product((0, 1), repeat=n):
                if 0 not in mask or 1 not in mask:
                    continue
                for symform in ("ref", "direct"):
                    cid = "m_%s_%s_%s" % (w, "".join(str(b) for b in mask),
                                          symform[0])
                    add(cid, "M", 1, specs, ["L", "L"], list(mask),
                        symform, "sub")

    # ---- tier N: the same, with UNPRODUCED fillers in the run ------------
    for n in (3, 4):
        for w in G1.canon_words(2, n):
            for k in (1, 2):
                for slots in itertools.combinations(range(n + k), k):
                    specs, wi, fi = [], 0, 0
                    for i in range(n + k):
                        if i in slots:
                            specs.append("F%d" % fi)
                            fi += 1
                        else:
                            specs.append("V" + w[wi])
                            wi += 1
                    for mask in itertools.product((0, 1), repeat=n + k):
                        if 0 not in mask or 1 not in mask:
                            continue
                        if mask[0] != 0:
                            continue        # keep the first store on symbol 0
                        cid = "n_%s_%s_%s" % (
                            w, "".join("f" if i in slots else "v"
                                       for i in range(n + k)),
                            "".join(str(b) for b in mask))
                        if cid in cells:
                            continue
                        add(cid, "N", fi, specs, ["L", "L"], list(mask),
                            "ref", "sub")

    # ---- tier Q: the SECOND FORMAL as the second symbol -------------------
    for n in (3, 4):
        for w in G1.canon_words(2, n):
            specs = ["V" + c for c in w]
            for mask in itertools.product((0, 1), repeat=n):
                if 0 not in mask or 1 not in mask:
                    continue
                cid = "q_%s_%s" % (w, "".join(str(b) for b in mask))
                add(cid, "Q", 0, specs, ["L", "L"], list(mask),
                    "ref", "formal")

    return cells


def held_out(cid, specs, kinds):
    """PREREGISTERED — docs/rungs/_2026-08-05-w-parse-prereg.md §6."""
    if hashlib.md5(cid.encode()).hexdigest()[0] in "012345":
        return "hash"
    if len(specs) > 5:
        return "long"
    if len({s for s in specs if s[0] == "V"}) >= 3:
        return "prod3"
    if len(set(kinds)) >= 3:
        return "kind3"
    return None


def main():
    cells = build_cells()
    src = os.path.join(W, "grid2.cpp")
    with open(src, "w") as f:
        f.write(HDR)
        for cid, (tier, nf, specs, kinds, syms, sf, sk) in cells.items():
            f.write(emit_cell(cid, nf, specs, kinds, syms, sf, sk))
    txt = LIB.compile_cod(src, os.path.join(W, "grid2.cod"),
                          os.path.join(W, "grid2.obj"))
    fns = LIB.parse_cod(txt)
    missing = [c for c in cells if c not in fns]
    if missing:
        raise SystemExit("FAIL: %d of %d cells produced no PROC: %s"
                         % (len(missing), len(cells), missing[:5]))

    rows_fit, rows_ho, nbad = [], [], 0
    for cid, (tier, nf, specs, kinds, syms, sf, sk) in cells.items():
        toks = canon(LIB.classify(fns[cid]), sk)
        bad = [t for t in toks if t.startswith("?")]
        nbad += bool(bad)
        row = "\t".join([cid, tier, str(nf), ",".join(specs), "".join(kinds),
                         "".join(str(s) for s in syms), sf, sk,
                         " ".join(toks), ";".join(bad)])
        (rows_ho if held_out(cid, specs, kinds) else rows_fit).append(row)

    hdr = ("cid\ttier\tnf\tspecs\tkinds\tsyms\tsymform\tsymkind\temitted\t"
           "unclaimed\n")
    open(os.path.join(W, "fit2.tsv"), "w").write(
        hdr + "\n".join(rows_fit) + "\n")
    open(os.path.join(W, "holdout2.tsv"), "w").write(
        hdr + "\n".join(rows_ho) + "\n")
    print("cells generated : %d" % len(cells))
    print("PROCs parsed    : %d" % len(fns))
    print("fit rows        : %d" % len(rows_fit))
    print("holdout rows    : %d" % len(rows_ho))
    print("cells with an UNCLAIMED instruction: %d" % nbad)
    per = {}
    for cid, v in cells.items():
        per[v[0]] = per.get(v[0], 0) + 1
    print("per tier : " + "  ".join("%s %d" % kv for kv in sorted(per.items())))


if __name__ == "__main__":
    main()

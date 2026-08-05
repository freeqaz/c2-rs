#!/usr/bin/env python3
"""supp.py — lane w-alloc. SUPPLEMENTARY probe, run after the fit and BEFORE
the freeze. Declared in the findings doc as fitting data, not as holdout.

`grid.py`'s tier 4 varies the producer KIND only over use patterns whose counts
are (1,1), (2,1), (2,2) and (2,2,1). For a REGISTER-DERIVED producer those
patterns cannot separate

    "source order"                    from    "count descending, ties forward"

because the two agree on every one of them. This probe supplies the unequal-count
patterns that separate them, for every kind, plus the MIXED-kind runs the grid
has none of, plus the multiply rule.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from alloc_lib import compile_cod, parse_cod, classify  # noqa: E402
from grid import HDR, KINDS, sig, value_expr  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

# unequal-count patterns: source order and count-descending DISAGREE on these.
PATS = ["01111", "00111", "01011", "011110", "0111", "01110", "0011122"]

MIXED = {
    # a run whose producers are NOT all the same kind — the grid has none.
    "x_LA_0101": (["V0", "V1", "V0", "V1"], ["L", "A"]),
    "x_AL_0101": (["V0", "V1", "V0", "V1"], ["A", "L"]),
    "x_LA_0011": (["V0", "V1", "V0", "V1"], ["L", "A"]),
    "x_LI_0101": (["V0", "V1", "V0", "V1"], ["L", "I"]),
    "x_IL_0101": (["V0", "V1", "V0", "V1"], ["I", "L"]),
    "x_LAS_01201": (["V0", "V1", "V2", "V0", "V1"], ["L", "A", "S"]),
    "x_ALS_01201": (["V0", "V1", "V2", "V0", "V1"], ["A", "L", "S"]),
    "x_LLA_01201": (["V0", "V1", "V2", "V0", "V1"], ["L", "L", "A"]),
    "x_LAA_01201": (["V0", "V1", "V2", "V0", "V1"], ["L", "A", "A"]),
    "x_LM_0101": (["V0", "V1", "V0", "V1"], ["L", "M"]),
    "x_ML_0101": (["V0", "V1", "V0", "V1"], ["M", "L"]),
}


def main():
    cells = {}
    for kind in KINDS:
        for pat in PATS:
            specs = ["V" + c for c in pat]
            cells["s_%s_%s" % (kind, pat)] = (1, specs, [kind] * 8)
    for cid, (specs, kinds) in MIXED.items():
        cells[cid] = (1, specs, kinds)

    src = os.path.join(W, "supp.cpp")
    with open(src, "w") as f:
        f.write(HDR)
        for cid, (nf, specs, kinds) in cells.items():
            body = " ".join(
                "p->m%X = %s;" % (k, value_expr(s, kinds[int(s[1:])])
                                  if s[0] == "V" else value_expr(s, "L"))
                for k, s in enumerate(specs))
            f.write("void %s%s { %s }\n" % (cid, sig(nf), body))

    txt = compile_cod(src, os.path.join(W, "supp.cod"),
                      os.path.join(W, "supp.obj"))
    fns = parse_cod(txt)
    missing = [c for c in cells if c not in fns]
    if missing:
        raise SystemExit("FAIL: %d cells produced no PROC: %s"
                         % (len(missing), missing[:5]))
    print("cells: %d   PROCs: %d" % (len(cells), len(fns)))

    for cid, (nf, specs, kinds) in sorted(cells.items()):
        toks = []
        for d in classify(fns[cid]):
            if d["mn"] == "blr":
                continue
            if d["role"] == "store":
                toks.append("S%d@%s" % (d["off"] // 4, d["src"]))
            elif d.get("dst"):
                toks.append("P%s" % d["dst"])
            else:
                toks.append("?" + d["mn"])
        alloc = {}
        for t in toks:
            if t[0] != "S":
                continue
            k, reg = t[1:].split("@")
            v = specs[int(k)]
            if v[0] == "V":
                alloc.setdefault(v, reg)
        cnt = {v: specs.count(v) for v in alloc}
        print("%-14s %-26s cnt=%-18s alloc=%-34s %s"
              % (cid, ",".join(specs), cnt, alloc, " ".join(toks)))


if __name__ == "__main__":
    main()

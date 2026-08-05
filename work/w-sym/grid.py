#!/usr/bin/env python3
"""grid.py — lane w-sym. The multi-symbol grid.

Built so that it CAN contain a counterexample to its own rule (the trap that
hit #581 and `w-sched`'s "184 conflicted cells"):

  A  two symbols, all-constant, no filler — every canonical producer word of
     length 2..5 over <= 3 producers x every non-trivial symbol mask.
  B  the same with 1..2 UNPRODUCED fillers, distinct formals and `this`.
  C  the second symbol is a second pointer FORMAL `M* q` — a different BASE
     REGISTER, so "symbol" cannot be a displacement-range artifact.
  D  THREE symbols                      (holdout, prereg clause 3 — R5 arity)
  E  MIXED producer kinds               (holdout, prereg clause 4 — R7, #581)
  F  FOUR distinct producers            (holdout, prereg clause 5)
  G  the register POOL: signature `(M* p, M* q)` and 4..6 producers, so
     r5..r11 are free and the pool is measurable (holdout — R8, #541/#543)
  P  ONE symbol with fillers — the population `w-order2` measured, rebuilt here
     because the v1 grid could not contain its own counterexample: tiers A–F
     all force >= 2 symbols and tier S stops at 4 statements with no filler, so
     `xboxheap`'s word THROUGH ONE SYMBOL — the 35 cells that refute the
     first-consumer rule (#582) — was in NO partition of it. **Added after the
     v1 residual was read**; the holdout rule is unchanged and mechanical, and
     the addition is disclosed in the findings doc.
  S  the `direct` control `p->e.eK = v` — same bytes, same base register, same
     displacement, ONE symbol (board #580)
  X  `xboxheap`'s own word and its twins — EXTERNAL, in neither partition

Producers span symbols wherever a word's occurrences fall under different mask
bits; those cells are the ones prereg R6 is about and they stay in FIT.

The holdout partition is decided HERE by `symlib.held_out`, the rule
preregistered in `docs/rungs/_2026-08-05-w-sym-prereg.md` §6, and written to a
file the fitter RAISES on opening.

Deterministic down-sampling of the oversized tiers uses **sha1**, not the md5
the holdout rule uses, so the sample cannot correlate with the partition.
"""
import hashlib
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402  (explicit path via sys.path[0] = this dir)

CAP = {"A": 100000, "B": 2600, "P": 100000, "C": 500, "D": 1000, "E": 800, "F": 600,
       "G": 100000, "S": 400, "X": 100000}


def canon_words(n, pmax):
    """Words of length `n` whose first occurrences are 0,1,2,… (<= pmax)."""
    out = []
    for w in itertools.product(range(pmax), repeat=n):
        seen = {}
        for c in w:
            if c not in seen:
                seen[c] = len(seen)
        if [seen[c] for c in w] == list(w):
            out.append(w)
    return out


def masks(n, vals):
    """Every assignment of `vals` to n statements that uses every value."""
    for m in itertools.product(vals, repeat=n):
        if len(set(m)) == len(vals):
            yield m


def build():
    cells = {}

    def add(cid, tier, nf, specs, kinds, syms, symform="ref", needq=False):
        assert cid not in cells, cid
        assert len(specs) == len(syms), cid
        cells[cid] = dict(tier=tier, nf=nf, specs=specs, kinds=kinds,
                          syms=list(syms), symform=symform, needq=needq)

    # ---------------------------------------------------------- tier A ------
    for n in range(2, 6):
        for w in canon_words(n, 3):
            specs = ["V%d" % c for c in w]
            kinds = ["L"] * (max(w) + 1)
            for m in masks(n, (0, 1)):
                add("a%d_%s_%s" % (n, "".join(map(str, w)),
                                   "".join(map(str, m))),
                    "A", 0, specs, kinds, m)

    # ---------------------------------------------------------- tier B ------
    FILLS = {"F": ["F0", "F1"], "T": ["T", "T"], "M": ["F0", "T"]}
    for wl in range(2, 5):
        for nfil in (1, 2):
            n = wl + nfil
            if n > 6:
                continue
            for w in canon_words(wl, 3):
                for slots in itertools.combinations(range(n), nfil):
                    for fname, fill in FILLS.items():
                        if nfil == 1 and fname == "M":
                            continue
                        specs, wi, fi = [], 0, 0
                        for i in range(n):
                            if i in slots:
                                specs.append(fill[fi])
                                fi += 1
                            else:
                                specs.append("V%d" % w[wi])
                                wi += 1
                        nf = sum(1 for s in specs if s[0] == "F")
                        kinds = ["L"] * (max(w) + 1)
                        for m in masks(n, (0, 1)):
                            add("b%s_%s_%s_%s"
                                % ("".join(map(str, w)),
                                   "".join("x" if i in slots else "v"
                                           for i in range(n)),
                                   fname, "".join(map(str, m))),
                                "B", nf, specs, kinds, m)

    # ---------------------------------------------------------- tier C ------
    for n in (3, 4):
        for w in canon_words(n, 3):
            specs = ["V%d" % c for c in w]
            kinds = ["L"] * (max(w) + 1)
            for m in masks(n, (0, 2)):
                add("c%d_%s_%s" % (n, "".join(map(str, w)),
                                   "".join(map(str, m))),
                    "C", 0, specs, kinds, m, needq=True)

    # ---------------------------------------------------------- tier D ------
    for n in (3, 4, 5):
        for w in canon_words(n, 3):
            specs = ["V%d" % c for c in w]
            kinds = ["L"] * (max(w) + 1)
            for m in masks(n, (0, 1, 2)):
                add("d%d_%s_%s" % (n, "".join(map(str, w)),
                                   "".join(map(str, m))),
                    "D", 0, specs, kinds, m, needq=True)

    # ---------------------------------------------------------- tier E ------
    for n in (3, 4, 5):
        for w in canon_words(n, 3):
            np_ = max(w) + 1
            if np_ < 2:
                continue
            for kinds in itertools.product("LAR", repeat=np_):
                if len(set(kinds)) < 2:
                    continue
                if list(kinds).count("R") > 1:
                    continue
                specs = ["V%d" % c for c in w]
                for m in masks(n, (0, 1)):
                    add("e%d_%s_%s_%s" % (n, "".join(map(str, w)),
                                          "".join(kinds),
                                          "".join(map(str, m))),
                        "E", 0, specs, list(kinds), m)

    # ---------------------------------------------------------- tier F ------
    for n in (4, 5, 6):
        for w in canon_words(n, 4):
            if max(w) + 1 != 4:
                continue
            specs = ["V%d" % c for c in w]
            kinds = ["L"] * 4
            for m in masks(n, (0, 1)):
                add("f%d_%s_%s" % (n, "".join(map(str, w)),
                                   "".join(map(str, m))),
                    "F", 0, specs, kinds, m)

    # ---------------------------------------------------------- tier G ------
    # `(M* p, M* q)` and nothing else: r5..r11 are all free, so the pool is
    # visible past r9 for the first time. Two lanes declined #543 because
    # their signature could not reach it.
    for np_ in (4, 5, 6):
        for n in (np_, np_ + 1):
            if n > 7:
                continue
            w = list(range(np_)) + [0] * (n - np_)
            specs = ["V%d" % c for c in w]
            for kinds in (["L"] * np_, ["A"] * np_):
                for m in ((0,) * n, tuple(0 if i < n - 1 else 1
                                          for i in range(n))):
                    if len(set(m)) == 1 and m[0] != 0:
                        continue
                    add("g%d_%d_%s_%s" % (np_, n, kinds[0],
                                          "".join(map(str, m))),
                        "G", 0, specs, list(kinds), m, needq=True)

    # ---------------------------------------------------------- tier P ------
    # One symbol, fillers included: the shape class ORDER (#561) was fitted on
    # and the one the first-consumer rule is known to fail. Without it this
    # grid could not contain its own counterexample.
    for wl in range(1, 4):
        for nfil in range(0, 4):
            n = wl + nfil
            if n < 2 or n > 6:
                continue
            for w in canon_words(wl, 3):
                for slots in itertools.combinations(range(n), nfil):
                    for fname, fill in FILLS.items():
                        if nfil < 2 and fname == "M":
                            continue
                        specs, wi, fi = [], 0, 0
                        for i in range(n):
                            if i in slots:
                                specs.append(fill[fi % len(fill)]
                                             if fname != "F" else "F%d" % fi)
                                fi += 1
                            else:
                                specs.append("V%d" % w[wi])
                                wi += 1
                        nf = sum(1 for s in specs if s[0] == "F")
                        kinds = ["L"] * (max(w) + 1)
                        add("p%s_%s_%s"
                            % ("".join(map(str, w)),
                               "".join("x" if i in slots else "v"
                                       for i in range(n)), fname),
                            "P", nf, specs, kinds, [0] * n)

    # ---------------------------------------------------------- tier S ------
    for n in (3, 4):
        for w in canon_words(n, 3):
            specs = ["V%d" % c for c in w]
            kinds = ["L"] * (max(w) + 1)
            for m in masks(n, (0, 1)):
                add("s%d_%s_%s" % (n, "".join(map(str, w)),
                                   "".join(map(str, m))),
                    "S", 0, specs, kinds, m, symform="direct")

    # ---------------------------------------------------------- tier X ------
    # `xboxheap`'s own statement word, and the controls that surround it.
    #   S0 formal, S1 this, S2 produced (count 1), S3 this, S4/S5 produced
    XW = ["F0", "T", "V0", "T", "V1", "V1"]
    for tag, syms in (("2sym", [0, 0, 0, 0, 1, 1]),
                      ("1sym", [0, 0, 0, 0, 0, 0]),
                      ("split", [0, 0, 1, 0, 1, 1]),
                      ("late", [0, 0, 0, 1, 1, 1])):
        for kinds in (["L", "L"], ["L", "R"], ["L", "A"], ["A", "L"]):
            add("x_%s_%s" % (tag, "".join(kinds)), "X", 1, XW, kinds, syms)
    return cells


def sample(cells):
    """Deterministic per-tier cap by sha1 — NOT the md5 the partition uses."""
    per = {}
    for cid, c in cells.items():
        per.setdefault(c["tier"], []).append(cid)
    keep = set()
    for tier, ids in per.items():
        ids.sort(key=lambda c: hashlib.sha1(c.encode()).hexdigest())
        keep.update(ids[:CAP.get(tier, 100000)])
    return {c: v for c, v in cells.items() if c in keep}


def main():
    cells = sample(build())
    src = os.path.join(W, "grid.cpp")
    with open(src, "w") as f:
        f.write(S.HDR)
        for cid, c in sorted(cells.items()):
            f.write(S.emit_cell(cid, c["nf"], c["specs"], c["kinds"],
                                c["syms"], c["symform"], c["needq"]))
    txt = S.LIB.compile_cod(src, os.path.join(W, "grid.cod"),
                            os.path.join(W, "grid.obj"))
    fns = S.LIB.parse_cod(txt)
    missing = [c for c in cells if c not in fns]
    if missing:
        raise SystemExit("FAIL: %d of %d cells produced no PROC: %s"
                         % (len(missing), len(cells), missing[:5]))

    rows = {"fit": [], "holdout": [], "external": []}
    nbad, per, badper = 0, {}, {}
    for cid, c in sorted(cells.items()):
        toks, stores, prods = S.decode(S.LIB.classify(fns[cid]), c["needq"],
                                       c["kinds"])
        bad = [t for t in toks if t.startswith("?")]
        nbad += bool(bad)
        if bad:
            badper[c["tier"]] = badper.get(c["tier"], 0) + 1
        part = ("external" if c["tier"] == "X"
                else (S.held_out(cid, c["specs"], c["kinds"], c["syms"])
                      or ""))
        row = "\t".join([
            cid, c["tier"], str(c["nf"]), ",".join(c["specs"]),
            "".join(c["kinds"]), "".join(map(str, c["syms"])), c["symform"],
            "1" if c["needq"] else "0", part, " ".join(toks),
            ",".join(map(str, stores)), ",".join(map(str, prods)),
            ";".join(bad)])
        bucket = ("external" if c["tier"] == "X"
                  else ("holdout" if part else "fit"))
        rows[bucket].append(row)
        per[c["tier"]] = per.get(c["tier"], 0) + 1

    hdr = "\t".join(S.FIELDS) + "\n"
    for name, rs in rows.items():
        open(os.path.join(W, "%s.tsv" % name), "w").write(
            hdr + "\n".join(rs) + "\n")

    print("cells generated : %d" % len(cells))
    print("PROCs parsed    : %d" % len(fns))
    print("fit rows        : %d" % len(rows["fit"]))
    print("holdout rows    : %d" % len(rows["holdout"]))
    print("external rows   : %d" % len(rows["external"]))
    print("cells with an UNCLAIMED instruction: %d  %s" % (nbad, badper))
    print("per tier : " + "  ".join("%s %d" % kv for kv in sorted(per.items())))
    if len(rows["fit"]) == 0 or len(rows["holdout"]) == 0:
        raise SystemExit("FAIL: an empty partition is a broken generator")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""grid.py — lane w-frame2. The LAYOUT grid.

Built so that it CAN contain a counterexample to the shipped rule. `w-sym`'s
7,589-cell grid contains the `[0,2]` family **six times** — one FIT cell, one
holdout cell and the four `x_split` externals — which is not a population, and a
grid that cannot produce its own counterexample in bulk is the trap that has now
hit three lanes (prereg §3).

The generator therefore sweeps the SYMBOL MASK exhaustively over a fixed word
rather than sampling it: every non-trivial two-valued mask of every canonical
producer word, at lengths 5 and 6, with the fillers held fixed. The `[0,2]`
family lives at exactly one mask of one word in `w-sym`'s grid; here every mask
of every word is present, so the falsification check of prereg §3 is satisfied
by construction — for each cell that says `slot(1) = 2` there is a sibling with
the same word and a different mask that must say `slot(1) = 1`.

  M   two symbols, exhaustive mask sweep, 2 producers, length 5-6
  N   the same at ONE producer
  O   ONE symbol — the reduction control (prereg R5)
  R   the second symbol is a second pointer FORMAL `M* q` — a different BASE
      REGISTER, so "symbol" cannot be a displacement-range artifact
  Q   THREE producers                     (holdout, partition clause 3)
  A   THREE symbols                       (holdout, partition clause 1)
  K   MIXED producer kinds                (holdout, partition clause 2)
  G   length 7                            (holdout, partition clause 4)
  S   the `direct` control `p->e.eK = v` — same bytes, one symbol (board #580)
  X   `xboxheap`'s own word at EVERY mask — EXTERNAL, in neither partition

The holdout partition is decided HERE by `f2lib.held_out`, the rule
preregistered in `docs/rungs/_2026-08-05-w-frame2-prereg.md` §4, and written to
a file the fitter RAISES on opening.

Deterministic down-sampling of the oversized tiers uses **sha1**, not the md5
the holdout rule uses, so the sample cannot correlate with the partition.
"""
import hashlib
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import f2lib as F  # noqa: E402  (explicit path via sys.path[0] = this dir)

CHUNK = 6000
CAP = {"M": 100000, "N": 100000, "O": 100000, "R": 900, "Q": 3000,
       "A": 1200, "K": 1200, "G": 2500, "S": 300, "X": 100000}


def canon_words(n, nprod):
    """Every length-`n` assignment of {filler, V0..V(nprod-1)} using EVERY
    producer, with the producers' first occurrences in index order."""
    out = []
    for w in itertools.product(range(-1, nprod), repeat=n):
        used = [c for c in w if c >= 0]
        if len(set(used)) != nprod:
            continue
        seen = {}
        for c in used:
            if c not in seen:
                seen[c] = len(seen)
        if [seen[c] for c in used] != used:
            continue
        out.append(w)
    return out


def fill(w, mode):
    """Turn a word into `specs`; `mode` fixes what the unproduced stores are.

    `T` = `this` (one register, reused), `F` = distinct formals, `M` = both.
    Held FIXED across the mask sweep so the mask is the only axis that moves.
    """
    specs, fi = [], 0
    for c in w:
        if c >= 0:
            specs.append("V%d" % c)
        elif mode == "T":
            specs.append("T")
        elif mode == "F":
            specs.append("F%d" % fi)
            fi += 1
        else:
            specs.append("T" if fi % 2 else "F%d" % fi)
            fi += 1
    return specs


def all_masks(n, vals):
    """EVERY assignment of `vals` to n statements that uses every value —
    exhaustive, not a sample. This is the axis the counterexample lives on."""
    for m in itertools.product(vals, repeat=n):
        if len(set(m)) == len(vals):
            yield m


def build():
    cells = {}

    def add(cid, tier, specs, kinds, syms, symform="ref", needq=False):
        assert cid not in cells, cid
        assert len(specs) == len(syms), cid
        nf = sum(1 for s in specs if s[0] == "F")
        cells[cid] = dict(tier=tier, nf=nf, specs=specs, kinds=kinds,
                          syms=list(syms), symform=symform, needq=needq)

    def wtag(w):
        return "".join("x" if c < 0 else str(c) for c in w)

    # ------------------------------------------------- tiers M / N / O / S --
    for n in (5, 6):
        for nprod in (1, 2):
            tier = "M" if nprod == 2 else "N"
            for w in canon_words(n, nprod):
                for mode in ("T", "F"):
                    specs = fill(w, mode)
                    kinds = ["L"] * nprod
                    for m in all_masks(n, (0, 1)):
                        add("%s%d%s%s_%s" % (tier.lower(), n, wtag(w), mode,
                                             "".join(map(str, m))),
                            tier, specs, kinds, m)
                    # the reduction control: the same word through ONE symbol
                    add("o%d%s%s" % (n, wtag(w), mode), "O",
                        specs, kinds, [0] * n)
                    # board #580's `direct` control — same bytes, one symbol
                    if n == 5 and mode == "T":
                        add("s%d%s" % (n, wtag(w)), "S", specs, kinds,
                            [1] * n, symform="direct")

    # ------------------------------------------------------------- tier R --
    # The second symbol as a second pointer FORMAL: a different BASE REGISTER.
    for w in canon_words(5, 2):
        specs = fill(w, "T")
        for m in all_masks(5, (0, 2)):
            add("r%s_%s" % (wtag(w), "".join(map(str, m))), "R",
                specs, ["L", "L"], m, needq=True)

    # ---------------------------------------- tier Q — THREE producers ------
    for n in (5, 6):
        for w in canon_words(n, 3):
            specs = fill(w, "T")
            for m in all_masks(n, (0, 1)):
                add("q%d%s_%s" % (n, wtag(w), "".join(map(str, m))), "Q",
                    specs, ["L"] * 3, m)

    # ---------------------------------------- tier A — THREE symbols --------
    for w in canon_words(5, 2):
        specs = fill(w, "T")
        for m in all_masks(5, (0, 1, 2)):
            add("a%s_%s" % (wtag(w), "".join(map(str, m))), "A",
                specs, ["L", "L"], m, needq=True)

    # ---------------------------------------- tier K — MIXED kinds ----------
    for w in canon_words(5, 2):
        specs = fill(w, "T")
        for kinds in (["L", "A"], ["A", "L"], ["L", "R"], ["A", "R"]):
            for m in all_masks(5, (0, 1)):
                add("k%s%s_%s" % (wtag(w), "".join(kinds),
                                  "".join(map(str, m))), "K", specs, kinds, m)

    # ---------------------------------------- tier G — length 7 -------------
    for w in canon_words(7, 2):
        specs = fill(w, "T")
        for m in all_masks(7, (0, 1)):
            add("g%s_%s" % (wtag(w), "".join(map(str, m))), "G",
                specs, ["L", "L"], m)

    # ---------------------------------------- tier X — EXTERNAL -------------
    # `xboxheap`'s own statement word at EVERY two-symbol mask, plus the four
    # `w-sym` named variants under their own names so the two lanes' externals
    # are the same cells.
    XW = ["F0", "T", "V0", "T", "V1", "V1"]
    for m in all_masks(6, (0, 1)):
        add("x_m%s" % "".join(map(str, m)), "X", XW, ["L", "L"], m)
    for tag, syms in (("2sym", [0, 0, 0, 0, 1, 1]),
                      ("1sym", [0, 0, 0, 0, 0, 0]),
                      ("split", [0, 0, 1, 0, 1, 1]),
                      ("late", [0, 0, 0, 1, 1, 1])):
        for kinds in (["L", "L"], ["L", "R"], ["L", "A"], ["A", "L"]):
            cid = "x_%s_%s" % (tag, "".join(kinds))
            if cid in cells:
                continue
            add(cid, "X", XW, kinds, syms)
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
    ids = sorted(cells)
    fns = {}
    for ci in range(0, len(ids), CHUNK):
        part = ids[ci:ci + CHUNK]
        src = os.path.join(W, "grid%d.cpp" % (ci // CHUNK))
        with open(src, "w") as f:
            f.write(F.HDR)
            for cid in part:
                c = cells[cid]
                f.write(F.emit_cell(cid, c["nf"], c["specs"], c["kinds"],
                                    c["syms"], c["symform"], c["needq"]))
        txt = F.LIB.compile_cod(src,
                                os.path.join(W, "grid%d.cod" % (ci // CHUNK)),
                                os.path.join(W, "grid%d.obj" % (ci // CHUNK)))
        got = F.LIB.parse_cod(txt)
        missing = [c for c in part if c not in got]
        if missing:
            raise SystemExit("FAIL: %d of %d cells produced no PROC: %s"
                             % (len(missing), len(part), missing[:5]))
        fns.update(got)
        sys.stderr.write("  chunk %d: %d PROCs\n" % (ci // CHUNK, len(got)))

    rows = {"fit": [], "holdout": [], "external": []}
    nbad, per, badper = 0, {}, {}
    for cid in ids:
        c = cells[cid]
        toks, stores, prods = F.decode(F.LIB.classify(fns[cid]), c["needq"],
                                       c["kinds"])
        bad = [t for t in toks if t.startswith("?")]
        nbad += bool(bad)
        if bad:
            badper[c["tier"]] = badper.get(c["tier"], 0) + 1
        part = ("external" if c["tier"] == "X"
                else (F.held_out(cid, c["specs"], c["kinds"], c["syms"],
                                 c["symform"]) or ""))
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

    hdr = "\t".join(F.FIELDS) + "\n"
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

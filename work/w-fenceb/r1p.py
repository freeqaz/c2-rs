#!/usr/bin/env python3
"""r1p.py — SCORE R1', on a hold-out this lane did not fit to.

Lane **w-fenceb**, board **#3089**/**#746**. Control: `work/w-fenceb/PREREG.md`,
committed at `98eea007` with grid3's sha256 **before the first `cl.exe`**.

# What this is for

`w-backedge` fitted `R1'` — count backward **targets** rather than **references**,
and charge 1 for a back edge whose target is a *named* `.sy` declaration — after
its hold-out was already open, and filed it explicitly as *"the next lane's
prereg, not this lane's result"*. It is therefore **UNSCORED**. This scores it.

Two things this adds over `labelil.py`, and both are needed for the question:

* **per-target vs per-reference is actually separable here.** `w-backedge` §4:
  *"No grid1 cell has two backward references to one target."* grid3 is a cross
  that produces them on purpose, and **the count of discriminating cells is
  printed** so that "no cells" is a loud failure rather than a silent pass.
* **the `break` term has an IL-computable detector.** A `break` is not "a
  forward branch" — forward branches are free at any arity (`e-if2/3/4` charge 0
  at 2, 3 and 4 labels). It is a forward *unconditional* `3A` that **leaps over
  a back edge**: its target is defined *after* a backward reference that sits
  after the jump itself. That predicate is PREREG §1's, frozen before the run.

Everything structural is `labelil.py`'s, imported rather than re-implemented:
the `.sy` block/declaration walker with its two reader corrections, the `.ex`
segmenter, the `4F 12 47` tail cut and the epilogue exclusion, and
`gt_label_stride`'s three-anchor seed-free stride with its in-obj base control.

    work/w-fenceb/r1p.py                    # grid3, /O1 (the workload's)
    work/w-fenceb/r1p.py --grid 1           # w-backedge's FITTING set, rescored
    work/w-fenceb/r1p.py --mode '/Ox /GS- /c'
    work/w-fenceb/r1p.py --tsv out.tsv

Exit status is non-zero only if a **control** failed. Never because a prediction
did — the table is the result.
"""

import hashlib
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
sys.path.insert(0, HERE)
import gt_label_stride as G  # noqa: E402
import labelil as L  # noqa: E402

# PREREG §0. A hold-out frozen by NAME is not frozen — `w-keygen` (#2966) had a
# population move -10.8 % under a byte-identical file. These are the hashes in
# the committed prereg and a mismatch is a hard stop.
FROZEN = {
    1: "3dd6e18f2b857875a9b11ee873137a6c1d0c5f9cd6a3cce1dfbf7e52120a62cd",
    2: "e1e2a5a2623479b472ba10a80eb8a6deb8deeb4daaae11e004de3059a96d1e54",
    3: "96cb9bea0aa0879b603552be734842c1d7455526ea6e346c5ce3268d0c621863",
}

# The eight grid3 cells that reproduce a cell `w-backedge` already compiled.
# They are CONTROLS on the instrument, not hold-out scoring, and every held-out
# score below excludes them. Their expected charges are `w-backedge`'s published
# ones, so a disagreement is an instrument fault and prints as `REPRO-FAIL`.
REPRO = {
    "w-c0b0": ("a-while", 2),
    "w-c1b1": ("h-while-brk-cont", 5),
    "f-c0b0": ("a-for", 2),
    "f-c1b0": ("h-continue", 2),
    "d-c0b0": ("a-dowhile", 1),
    "g-back1": ("a-goto-back", 1),
    "z-none": ("a-none", 0),
    "z-if3": ("e-if3", 0),
}


def load_grid(n):
    path = os.path.join(HERE, "grid%d.tsv" % n)
    raw = open(path, "rb").read()
    got = hashlib.sha256(raw).hexdigest()
    if got != FROZEN[n]:
        sys.stderr.write("FATAL: grid%d.tsv moved.\n  frozen %s\n  now    %s\n"
                         % (n, FROZEN[n], got))
        sys.exit(2)
    cells = []
    for line in raw.decode().splitlines():
        if not line or line.startswith("#"):
            continue
        name, cls, body = line.split("\t", 2)
        cells.append((name, cls, body))
    return cells


# ---------------------------------------------------------------------------
# The features. PREREG §1, verbatim.
# ---------------------------------------------------------------------------
def cflow(seg):
    """Ordered label events of one `.ex` segment, and the derived features.

    `labelil.ex_cflow`'s reading with the ORDER kept, because two of the three
    predicates PREREG §1 defines are positional: `named` is per target and
    needs the token, and `break-jump` needs the stream index of the jump, of
    the definition it targets, and of the back edges in between.
    """
    tail = seg.find(bytes([0x4F, 0x12, 0x47]))
    if tail > 0:
        seg = seg[:tail]
    ev = []
    i = 0
    while i + 2 < len(seg):
        b = seg[i]
        if b in (0x29, 0x38, 0x39, 0x3A):
            t = L.token_at(seg, i + 1)
            if t:
                ev.append(("d" if b == 0x29 else "r", t[0], b, i))
        i += 1
    defs = [t for k, t, _, _ in ev if k == "d"]
    epi = defs[-1] if defs else None
    def_at = {}
    for k, t, _, i in ev:
        if k == "d" and t not in def_at:
            def_at[t] = i
    seen = set()
    bwd = []      # (tok, opcode, index)
    fwd = []
    for k, tok, b, i in ev:
        if k == "d":
            seen.add(tok)
            continue
        if tok == epi:
            continue
        (bwd if tok in seen else fwd).append((tok, b, i))
    # per-TARGET view
    tgt = {}
    for tok, b, i in bwd:
        e = tgt.setdefault(tok, {"uncond": False, "refs": 0, "first": i})
        e["refs"] += 1
        if b == 0x3A:
            e["uncond"] = True
    # break-jumps: a forward `3A` at i to a target defined at d, with a
    # BACKWARD reference at b strictly between them.
    bwd_ix = sorted(i for _, _, i in bwd)
    brk = []
    for tok, b, i in fwd:
        if b != 0x3A:
            continue
        d = def_at.get(tok)
        if d is None:
            continue
        if any(i < x < d for x in bwd_ix):
            brk.append((tok, i, d))
    # which backward target does each break-jump leap over? (the per-LOOP
    # pricing of the break term — PREREG §2's M-TGT-L / M-REF-L)
    brk_loops = set()
    for tok, i, d in brk:
        for t2 in tgt:
            if any(i < x < d for tk, _, x in bwd if tk == t2):
                brk_loops.add(t2)
    return {
        "defs": len(set(defs)) - (1 if epi is not None else 0),
        "bwd_refs": len(bwd),
        "bwd_t": len(tgt),
        "bwd_uncond": sum(1 for _, b, _ in bwd if b == 0x3A),
        "bwd_cond": sum(1 for _, b, _ in bwd if b != 0x3A),
        "fwd_refs": len(fwd),
        "fwd_t": len({t for t, _, _ in fwd}),
        "brk": len(brk),
        "brk_loops": len(brk_loops),
        "targets": tgt,
    }


def named_tokens(recs):
    """The `.sy` declaration tokens declared in the NAMED form.

    `labelil.decl_record_end` keeps each record's tail bytes; the named form is
    `03 <k> <tok> 00 <name> 00 <b> <b>`, so the tail begins with `00`. A source
    `goto` label is the only thing that carries a name.
    """
    if not recs:
        return set(), []
    out, names = set(), []
    for kind, tok, tail in recs:
        if tail[:1] == b"\x00":
            out.add(tok)
            e = tail.find(b"\x00", 1)
            names.append(tail[1:e if e > 0 else len(tail)].decode("latin1"))
    return out, names


# ---------------------------------------------------------------------------
# The models. PREREG §2, frozen.
# ---------------------------------------------------------------------------
def m_r1(x, named):
    return 2 * x["bwd_uncond"] + x["bwd_cond"]


def m_r1p(x, named):
    """R1' — per TARGET, and a NAMED target charges 1."""
    n = 0
    for tok, e in x["targets"].items():
        n += 1 if tok in named else (2 if e["uncond"] else 1)
    return n


def m_ref_named(x, named):
    """R1 per REFERENCE, with R1's named correction only."""
    n = 0
    for tok, e in x["targets"].items():
        if tok in named:
            n += 1
        else:
            n += sum(2 if b == 0x3A else 1
                     for tk, b, _ in x["_refs"] if tk == tok)
    return n


MODELS = [
    ("R1", lambda x, n: m_r1(x, n)),
    ("R1'", lambda x, n: m_r1p(x, n)),
    ("M-TGT", lambda x, n: m_r1p(x, n) + x["brk"]),
    ("M-TGT-L", lambda x, n: m_r1p(x, n) + x["brk_loops"]),
    ("M-REF", lambda x, n: m_ref_named(x, n) + x["brk"]),
    ("M-REF-L", lambda x, n: m_ref_named(x, n) + x["brk_loops"]),
]


def main(argv):
    mode = "/O1 /GS- /c"
    gridn = 3
    tsv = None
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    if "--grid" in argv:
        i = argv.index("--grid"); gridn = int(argv[i + 1]); del argv[i:i + 2]
    if "--tsv" in argv:
        i = argv.index("--tsv"); tsv = argv[i + 1]; del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]

    cells = load_grid(gridn)
    if want:
        cells = [c for c in cells if c[0] in want]
    wd = tempfile.mkdtemp(prefix="wfb")
    print("grid %d (sha256 verified)   mode: %s   %d cells" % (gridn, mode, len(cells)))
    print("  charge   = stride - minted        (LABEL_COUNTER.md §4's `minted` correction)")
    print("  CONTROL  : the three identical anchors' sy_decls must be EQUAL")
    print("  DISCRIM  : bwd_refs > bwd_t  -- the cells per-reference and per-target disagree on")
    print()
    hdr = ("%-11s %6s %5s %5s %5s %5s %5s %5s %5s %-6s"
           % ("cell", "CHARGE", "bwdR", "bwdT", "bwdU", "bwdC", "brk", "brkL",
              "namd", "flag"))
    hdr += "".join("%9s" % m for m, _ in MODELS)
    print(hdr)
    bad = 0
    rows = []
    for name, cls, body in cells:
        tag = "z" + name.replace("-", "_")
        probe = "int P(int a){ %s }" % body
        src = G.build_src("int ga(int);", [], probe)
        o = G.capture(src, mode, wd, tag)
        if o is None:
            print("%-11s  CAPTURE FAILED" % name); bad += 1; continue
        gs = G.groups(o)

        def find(s):
            for g in gs:
                if g["name"].startswith("?" + s + "@@"):
                    return g
            return None
        a0, a1, a2, P = find("a0"), find("a1"), find("a2"), find("P")
        if not (a0 and a1 and a2 and P):
            print("%-11s  missing group" % name); bad += 1; continue
        fl = lambda g: min(g["labels"]) if g["labels"] else None
        base = fl(a2) - fl(a1)
        if base not in (4, 5):
            print("%-11s  CONTROL FAILED: base %s" % (name, base)); bad += 1; continue
        charge = fl(a1) - fl(a0) - base - G.minted(P)

        il = L.capture_il(os.path.join(wd, tag + ".cpp"), mode, wd)
        if il is None:
            print("%-11s  IL CAPTURE FAILED" % name); bad += 1; continue
        sy = open(il["sy"], "rb").read()
        ex = open(il["ex"], "rb").read()
        blocks = L.sy_label_decls(sy)
        segs = L.ex_segments(ex)
        if len(blocks) != 4 or len(segs) != 4:
            print("%-11s  IL SHAPE: %d sy blocks, %d ex segments (want 4/4)"
                  % (name, len(blocks), len(segs))); bad += 1; continue
        nrec = lambda i: (None if blocks[i]["recs"] is None
                          else len(blocks[i]["recs"]))
        anc = [nrec(i) for i in (0, 2, 3)]
        flag = ""
        if not (len(set(anc)) == 1 and anc[0] is not None):
            flag = "ANCH!"; bad += 1
        named, nms = named_tokens(blocks[1]["recs"])
        x = cflow(segs[1])
        # the per-reference model needs the raw reference list
        tail = segs[1][:segs[1].find(bytes([0x4F, 0x12, 0x47]))] \
            if segs[1].find(bytes([0x4F, 0x12, 0x47])) > 0 else segs[1]
        refs = []
        seen, i = set(), 0
        epi = None
        alld = []
        while i + 2 < len(tail):
            if tail[i] in (0x29, 0x38, 0x39, 0x3A):
                t = L.token_at(tail, i + 1)
                if t and tail[i] == 0x29:
                    alld.append(t[0])
            i += 1
        epi = alld[-1] if alld else None
        i = 0
        while i + 2 < len(tail):
            b = tail[i]
            if b in (0x29, 0x38, 0x39, 0x3A):
                t = L.token_at(tail, i + 1)
                if t:
                    if b == 0x29:
                        seen.add(t[0])
                    elif t[0] in seen and t[0] != epi:
                        refs.append((t[0], b, i))
            i += 1
        x["_refs"] = refs
        if x["bwd_refs"] > x["bwd_t"]:
            flag = ("DISCRIM " + flag).strip()
        if name in REPRO and charge != REPRO[name][1]:
            flag = ("REPRO-FAIL(%s=%d)" % REPRO[name] + " " + flag).strip()
            bad += 1
        preds = [f(x, named) for _, f in MODELS]
        line = ("%-11s %6d %5d %5d %5d %5d %5d %5d %5d %-6s"
                % (name, charge, x["bwd_refs"], x["bwd_t"], x["bwd_uncond"],
                   x["bwd_cond"], x["brk"], x["brk_loops"], len(named), flag))
        line += "".join("%8d%s" % (p, "." if p == charge else "X") for p in preds)
        print(line)
        row = {"cell": name, "charge": charge, "names": ";".join(nms),
               "repro": name in REPRO,
               "discrim": x["bwd_refs"] > x["bwd_t"]}
        for k in ("bwd_refs", "bwd_t", "bwd_uncond", "bwd_cond", "brk",
                  "brk_loops", "defs", "fwd_refs", "fwd_t"):
            row[k] = x[k]
        row["named"] = len(named)
        for (m, _), p in zip(MODELS, preds):
            row[m] = p
        rows.append(row)

    print()
    nd = sum(1 for r in rows if r["discrim"])
    print("DISCRIMINATING CELLS (bwd_refs > bwd_t): %d" % nd
          + ("   <== ZERO. The grid cannot separate per-reference from "
             "per-target and NO score below is evidence either way." if nd == 0
             else ""))
    held = [r for r in rows if not r["repro"]]
    nobrk = [r for r in rows if r["brk"] == 0]
    hnobrk = [r for r in held if r["brk"] == 0]
    print("cells %d   reproduction controls %d   HELD OUT %d   no-break %d "
          "(held-out no-break %d)"
          % (len(rows), len(rows) - len(held), len(held), len(nobrk),
             len(hnobrk)))
    print()
    print("%-9s %-16s %-16s %-18s %-16s" % ("model", "all cells", "HELD OUT",
                                            "held-out no-break", "DISCRIM cells"))
    for m, _ in MODELS:
        def sc(rs):
            n = sum(1 for r in rs if r[m] == r["charge"])
            return "%d of %d" % (n, len(rs))
        print("%-9s %-16s %-16s %-18s %-16s"
              % (m, sc(rows), sc(held), sc(hnobrk),
                 sc([r for r in rows if r["discrim"]])))

    # PREREG §5.3 — two cells sharing a whole feature vector and differing in
    # charge falsify the vector itself. Printed rather than recalled.
    key = lambda r: (r["bwd_t"], r["bwd_uncond"], r["bwd_cond"], r["brk"],
                     r["named"])
    coll = []
    for i in range(len(rows)):
        for j in range(i + 1, len(rows)):
            if key(rows[i]) == key(rows[j]) and \
               rows[i]["charge"] != rows[j]["charge"]:
                coll.append((rows[i]["cell"], rows[j]["cell"],
                             rows[i]["charge"], rows[j]["charge"]))
    print("\nFEATURE-VECTOR COLLISIONS (same (bwd_t,bwd_u,bwd_c,brk,named), "
          "different charge): %d" % len(coll))
    for a, b, ca, cb in coll:
        print("   %-11s %-11s   %d vs %d" % (a, b, ca, cb))

    if tsv:
        keys = (["cell", "charge", "repro", "discrim", "bwd_refs", "bwd_t",
                 "bwd_uncond", "bwd_cond", "brk", "brk_loops", "named",
                 "defs", "fwd_refs", "fwd_t", "names"]
                + [m for m, _ in MODELS])
        with open(tsv, "w") as fh:
            fh.write("\t".join(keys) + "\n")
            for r in rows:
                fh.write("\t".join(str(r[k]) for k in keys) + "\n")
        print("\nwrote %s (%d rows)" % (tsv, len(rows)))
    print("\ncontrols failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

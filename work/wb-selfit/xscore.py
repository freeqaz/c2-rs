#!/usr/bin/env python3
"""wb-selfit — cross-score the two wb-select readings against BOTH frozen grids.

METHOD (w-memfit §3.0's discipline, inherited verbatim):

    This script REFUSES to print a cross-score until it has re-derived, from
    each lane's own committed files, every per-cell verdict and every total
    that lane published.  A rescoring harness that cannot reproduce the
    published scores is measuring something else.

Everything read here is committed in this repo:

    docs/whitebox/grids/wb-select/frozen.tsv        grid-1 frozen predictions
    docs/whitebox/grids/wb-select2/frozen.tsv       grid-2 frozen predictions
    docs/whitebox/WB_SELECT_FINDINGS.md      §7     grid-1 measured + verdicts
    docs/whitebox/WB_SELECT_FINDINGS_R2.md   §6     grid-2 measured + verdicts

No obj is compiled and no obj is read: the toolchain is absent in this
worktree.  Every measured word below is the two lanes' own published bytes.

Usage:  python3 work/wb-selfit/xscore.py <repo-root>
"""
import re
import sys

# ---------------------------------------------------------------------------
# 0.  The canonicalisation table.  THIS IS THE THING THAT DECIDES HIT/MISS,
#     so it is written out rather than buried in a regex.
#
#     c2's own simplified-mnemonic table (0x10b1d190, read by wb-select2 §1.1)
#     maps a spelling to a real opcode.  Both lanes' grids mix the two
#     spellings freely -- `frozen.tsv` predicts `rlwinm` where gt_dump prints
#     `clrlwi`, and predicts `subc` where the machine opcode is `subfc`.
#     Strict equality on raw spellings would score wb-select's own published
#     HITs as misses, which is how a rescoring harness invents a refutation.
# ---------------------------------------------------------------------------
SIMPLIFIED = {
    # rotate-and-mask family -> rlwinm
    "clrlwi": "rlwinm", "srwi": "rlwinm", "slwi": "rlwinm",
    "rotlwi": "rlwinm", "extlwi": "rlwinm", "clrrwi": "rlwinm",
    "inslwi": "rlwinm",
    # subtract family (0x10b1d190)
    "sub": "subf", "subc": "subfc", "subi": "addi", "subis": "addis",
    "subic": "addic",
    # compare family
    "cmpw": "cmp", "cmpwi": "cmpi", "cmplw": "cmpl", "cmplwi": "cmpli",
    "cmpd": "cmp", "cmpdi": "cmpi", "cmpld": "cmpl", "cmpldi": "cmpli",
    # branch family -- every conditional form collapses to `bc`/`bclr`
    "blt": "bc", "bge": "bc", "bgt": "bc", "ble": "bc", "beq": "bc",
    "bne": "bc", "bf": "bc", "bt": "bc",
    "bltlr": "bclr", "bgelr": "bclr", "bnelr": "bclr", "beqlr": "bclr",
    "mr": "mr", "not": "not",
}


def canon(m):
    m = m.strip().lower()
    return SIMPLIFIED.get(m, m)


def canon_seq(seq):
    return tuple(canon(x) for x in seq if x.strip())


# ---------------------------------------------------------------------------
# 1.  Parsers -- committed files only.
# ---------------------------------------------------------------------------
def parse_measured_grid1(path):
    """WB_SELECT_FINDINGS.md §7's fenced block: `-- wbs_s1   li 11,10 / subc ...`"""
    out = {}
    for line in open(path):
        m = re.match(r"^--\s+(wbs_\w+)\s+(.*)$", line.strip())
        if m:
            words = [w.strip() for w in m.group(2).split("/")]
            out[m.group(1)] = [(w.split()[0], w.split()[1:]) for w in words if w]
    return out


def parse_measured_grid2(path):
    """WB_SELECT_FINDINGS_R2.md §6's table: | **S1** `sel_ltu_ab` | pred | emitted | verdict |"""
    out, verdicts = {}, {}
    for line in open(path):
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cols) != 4:
            continue
        m = re.match(r"^\*\*(S\d+)\*\*", cols[0])
        if not m:
            continue
        cell = m.group(1)
        emitted = cols[2]
        emitted = emitted.replace("**", "")
        words = [w.strip().strip("`") for w in emitted.split("·")]
        seq = []
        for w in words:
            w = w.strip().strip("`").strip()
            if not w:
                continue
            parts = w.split()
            seq.append((parts[0], parts[1:]))
        out[cell] = seq
        vm = re.search(r"\*\*(HIT|MISS)\*\*", cols[3])
        verdicts[cell] = vm.group(1) if vm else "?"
    return out, verdicts


def parse_frozen_grid1(path):
    """the tab-separated rows at the foot of grids/wb-select/frozen.tsv"""
    out = {}
    for line in open(path):
        if line.startswith("#") or not line.strip():
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 5 or f[0] == "cell":
            continue
        cell, primary, secondary, cost, rival = f[0], f[1], f[2], f[3], f[4]
        out[cell] = dict(primary=primary, secondary=secondary,
                         cost=cost, rival=rival)
    return out


def parse_published_grid1(path):
    """WB_SELECT_FINDINGS.md §7's verdict table -> (primary, secondary) per cell"""
    out = {}
    for line in open(path):
        if not line.startswith("| `wbs_"):
            continue
        cols = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cols) != 3:
            continue
        cell = cols[0].strip("`")
        prim = "HIT" if "**HIT**" in cols[1] else ("MISS" if "MISS" in cols[1] else "?")
        if "not predicted" in cols[2]:
            sec = "NOTPRED"
        elif "**HIT" in cols[2]:
            sec = "HIT"
        else:
            sec = "MISS"
        out[cell] = (prim, sec)
    return out


# ---------------------------------------------------------------------------
# 2.  THE CONTROL.  Re-derive each lane's published verdicts from its own
#     frozen predictions and its own measured words.
# ---------------------------------------------------------------------------
# Grid-1 primary column: 10 of 12 cells are literal `a ; b ; c` sequences; the
# other two (wbs_s5, wbs_s6) are CLASS predicates whose conjuncts are quoted
# here verbatim from frozen.tsv and encoded as machine-checkable tests.
GRID1_CLASS = {
    "wbs_s5": dict(
        quote="contains cmpwi ; contains a conditional branch ; "
              "contains NO subfc/subfic/subfe/cntlzw",
        need=["cmpi"], need_branchish=True,
        forbid=["subfc", "subfic", "subfe", "cntlzw"], words=None),
    "wbs_s6": dict(
        quote="contains exactly two cntlzw and one rlwinm rD,rD,27,31,31 ; "
              "contains NO subfc/subfic/subfe ; 4 or 5 words incl. blr",
        need_exact={"cntlzw": 2}, need_rlwinm_27=True,
        forbid=["subfc", "subfic", "subfe"], words=(4, 5)),
}


def grade_grid1_primary(cell, frozen, measured):
    seq = canon_seq([m for m, _ in measured])
    if cell in GRID1_CLASS:
        spec = GRID1_CLASS[cell]
        mn = [m for m, _ in measured]
        c = canon_seq(mn)
        if "need" in spec and not all(n in c for n in spec["need"]):
            return "MISS"
        if spec.get("need_branchish") and not any(
                x.startswith("bc") for x in c):
            return "MISS"
        for k, v in spec.get("need_exact", {}).items():
            if c.count(k) != v:
                return "MISS"
        if spec.get("need_rlwinm_27"):
            ok = any(m0 in ("rlwinm", "srwi") and
                     ("27" in ",".join(ops) or m0 == "srwi")
                     for m0, ops in measured)
            if not ok:
                return "MISS"
        if any(f in c for f in spec.get("forbid", [])):
            return "MISS"
        if spec.get("words") and not (spec["words"][0] <= len(c) <= spec["words"][1]):
            return "MISS"
        return "HIT"
    pred = canon_seq(frozen["primary"].split(";"))
    return "HIT" if pred == seq else "MISS"


REG = re.compile(r"^r(\d+)$")


def norm_ops(mn, ops):
    """expand the simplified rotate forms so a register comparison is possible"""
    ops = [o.strip().lstrip("r") for o in ops]
    if mn == "clrlwi" and len(ops) == 3:
        return [ops[0], ops[1], "0", ops[2], "31"]
    if mn == "srwi" and len(ops) == 3:
        n = int(ops[2])
        return [ops[0], ops[1], str(32 - n), str(n), "31"]
    if mn == "slwi" and len(ops) == 3:
        n = int(ops[2])
        return [ops[0], ops[1], str(n), "0", str(31 - n)]
    return ops


def grade_grid1_secondary(cell, frozen, measured):
    text = frozen["secondary"]
    if "not predicted" in text:
        return "NOTPRED"
    pred = []
    for w in text.split("/"):
        w = w.strip()
        if not w:
            continue
        p = w.split()
        pred.append((canon(p[0]), norm_ops(p[0].strip().lower(),
                                           " ".join(p[1:]).split(","))))
    meas = [(canon(m), norm_ops(m.strip().lower(), " ".join(o).split(",")))
            for m, o in measured]
    return "HIT" if pred == meas else "MISS"


# Grid-2 core column: every cell is prose.  The conjuncts below are transcribed
# verbatim from grids/wb-select2/frozen.tsv, one dict per cell, and the quote
# is carried beside them so a reader can check the transcription.
GRID2_SPEC = {
    "S1":  dict(quote="carry setter (subfic or addic, imm 9 or -10), 2-word 0/-1 "
                      "mask CONTAINING subfe, mask by 4 via rlwinm/andi., addi ...,3, blr. "
                      "NO cmplwi, NO conditional branch, NO cntlzw",
                need=["subfe", "addi"], need_any_setter=["subfic", "addic"],
                forbid=["cmpli", "cntlzw"], forbid_branch=True),
    "S2":  dict(quote="cmpwi (cr6) vs 10, conditional branch, two li (7 and 3), blr. "
                      "NO subfe, NO cntlzw, NO addze",
                need=["cmpi", "li"], need_branch=True,
                forbid=["subfe", "cntlzw", "addze"]),
    "S3":  dict(quote="cntlzw r3,r3 then rlwinm ...,27,31,31 then blr. NO subfe",
                seq=["cntlzw", "rlwinm", "blr"], forbid=["subfe"]),
    "S4":  dict(quote="carry-setting subtract against 0 (subfic/addic) then subfe, "
                      "then blr. NO cntlzw, NO xori",
                need=["subfe"], need_any_setter=["subfic", "addic"],
                forbid=["cntlzw", "xori"]),
    "S5":  dict(quote="srawi r3,r3,3 then addze r3,r3 then blr",
                seq=["srawi", "addze", "blr"]),
    "S6":  dict(quote="li then divw then blr. NO mulhw, NO mulhwu",
                seq=["li", "divw", "blr"], forbid=["mulhw", "mulhwu"]),
    "S7":  dict(quote="rlwinm r3,r3,0,24,31 (clrlwi 24) then blr. NOT andi.",
                seq=["rlwinm", "blr"], forbid=["andi."]),
    "S8":  dict(quote="oris r3,r3,1 then ori r3,r3,0x2345 then blr. NO lis, NO or",
                seq=["oris", "ori", "blr"], forbid=["lis", "or"]),
    "S9":  dict(quote="lbz r3,0(r3), extsb r3,r3, addi r3,r3,1, blr",
                seq=["lbz", "extsb", "addi", "blr"]),
    "S10": dict(quote="lha r3,0(r3), addi r3,r3,1, blr. extsh does NOT appear",
                seq=["lha", "addi", "blr"], forbid=["extsh"]),
    "S11": dict(quote="carry-setting subtract, 2-word 0/-1 mask containing subfe, "
                      "mask by 8 VIA RLWINM, blr. NO addi bias",
                need=["subfe", "rlwinm"], forbid=["addi"]),
    "S12": dict(quote="cmplwi vs 10 in cr6, a conditional branch, two li (1 and 2), "
                      "blr. NO subfe, NO cntlzw",
                need=["cmpli", "li"], need_branch=True,
                forbid=["subfe", "cntlzw"]),
}


def grade_grid2(cell, measured):
    spec = GRID2_SPEC[cell]
    c = list(canon_seq([m for m, _ in measured]))
    if "seq" in spec:
        # NOTE (the unbudgeted refusal, wb-selfit rung §6.1): this comparison
        # was `c != canon_seq(...)` -- a list against a tuple, which is ALWAYS
        # unequal.  It graded GRID-2 at 2/12 against a published 9/12 and the
        # control caught it.  A cross-score printed from that grader would have
        # "refuted" the second lane on a Python type mismatch.
        if tuple(c) != canon_seq(spec["seq"]):
            return "MISS"
    for n in spec.get("need", []):
        if canon(n) not in c:
            return "MISS"
    if "need_any_setter" in spec:
        if not any(canon(s) in c for s in spec["need_any_setter"]):
            return "MISS"
    if spec.get("need_branch") and not any(x.startswith("bc") for x in c):
        return "MISS"
    if spec.get("forbid_branch") and any(x.startswith("bc") for x in c):
        return "MISS"
    for f in spec.get("forbid", []):
        if canon(f) in c:
            return "MISS"
    return "HIT"


# ---------------------------------------------------------------------------
# 3.  THE CROSS-SCORE table.  For each of the 24 cells, what each READING --
#     the rule set the lane's findings doc states, NOT the frozen.tsv row --
#     predicts, with the section that supplies it.  `None` = the reading's own
#     document says the deciding pass is unread, so it ABSTAINS.
# ---------------------------------------------------------------------------
# R1 = WB_SELECT_FINDINGS.md   (lane wb-select)
# R2 = WB_SELECT_FINDINGS_R2.md (lane wb-select2)
#
# A cell marked ZERO is an against-zero relational.  BOTH documents name
# FUN_10c1a908 as located-and-unread (R1 §3.6/§9.5; R2's W-SELECT-3 row), and
# FUN_10c1b517 routes an against-zero relational there BEFORE the cost race.
# Under the SYMMETRIC policy both readings abstain on every ZERO cell.
CROSS = {
    # cell:      (R1 prediction, R1 src, R2 prediction, R2 src, zero?)
    "wbs_s1":  (["li", "subfc", "subfe", "addi", "blr"], "R1 §3.2",
                ["li", "subfc", "subfe", "addi", "blr"], "R2 §3.1+§6.3a", False),
    "wbs_s2":  (["subfc", "subfe", "addi", "blr"], "R1 §3.2",
                ["subfc", "subfe", "addi", "blr"], "R2 §3.1", False),
    "wbs_s3":  (["subfic", "subfe", "rlwinm", "addi", "blr"], "R1 §3.2+§2.4",
                ["subfic", "subfe", "rlwinm", "addi", "blr"], "R2 §3.1+§6.3a", False),
    "wbs_s4":  (["cntlzw", "rlwinm", "xori", "addi", "blr"], "R1 §3.3",
                ["cntlzw", "rlwinm", "xori", "addi", "blr"], "R2 §3.2", True),
    "wbs_s5":  (["cmpi", "li", "bclr", "li", "blr"], "R1 §3.4 (class)",
                ["cmpi", "li", "bclr", "li", "blr"], "R2 §6.3c (class)", False),
    "wbs_s6":  (None, "R1 §3.3 cost arithmetic retracted in its own §7.6",
                ["cntlzw", "cntlzw", "rlwinm", "xori", "addi", "blr"], "R2 §3.2", True),
    "wbs_b1":  (None, "R1 §7.6 excludes {0,1} results as nibble 5",
                ["li", "subfc", "subfe", "rlwinm", "blr"], "R2 §3.1+§2.1", False),
    "wbs_b2":  (None, "R1 §7.6 excludes {0,1} results as nibble 5",
                ["subfc", "subfe", "rlwinm", "blr"], "R2 §3.1+§2.1", False),
    "wbs_b3":  (None, "R1 §7.6 routes it to FUN_10c194b8, unread",
                ["cntlzw", "cntlzw", "rlwinm", "blr"], "R2 §3.2", True),
    "wbs_k1":  (["srawi", "addze", "blr"], "R1 §2.4",
                ["srawi", "addze", "blr"], "R2 §2.4", False),
    "wbs_k2":  (["rlwinm", "blr"], "R1 §2.4 / W-SELECT-5",
                ["rlwinm", "blr"], "R2 §2.4 (cell S7's own class)", False),
    "wbs_k3":  (None, "R1 §7.6 excludes {0,1} results as nibble 5",
                ["subfc", "subfe", "rlwinm", "add", "blr"], "R2 §3.1", False),
    "S1":      (["li", "subfc", "subfe", "rlwinm", "addi", "blr"], "R1 §3.2+§2.4",
                ["li", "subfc", "subfe", "rlwinm", "addi", "blr"], "R2 §3.1+§6.3a", False),
    "S2":      (["cmpi", "li", "bclr", "li", "blr"], "R1 §3.4 (class)",
                ["cmpi", "li", "bclr", "li", "blr"], "R2 §6.3c (class)", False),
    "S3":      (None, "R1 §7.6 excludes {0,1} results as nibble 5",
                ["cntlzw", "rlwinm", "blr"], "R2 §3.2", True),
    "S4":      (None, "R1 §7.6 excludes {0,1} results as nibble 5",
                ["addic", "subfe", "blr"], "R2 §3.1", True),
    "S5":      (["srawi", "addze", "blr"], "R1 §2.4",
                ["srawi", "addze", "blr"], "R2 §2.4", False),
    "S6":      (None, "R1 §9.5: the non-power-of-two divide is NOT claimed",
                ["li", "divw", "blr"], "R2 §2.4", False),
    "S7":      (["rlwinm", "blr"], "R1 §2.4 / W-SELECT-5",
                ["rlwinm", "blr"], "R2 §2.4", False),
    "S8":      (None, "R1 §2.1 names ori/or but states no wide-constant split",
                ["oris", "ori", "blr"], "R2 §2.4", False),
    "S9":      (None, "R1 has NO convert table; P4.3 withdrawn untested",
                ["lbz", "extsb", "addi", "blr"], "R2 §2.2 (table 0x10b1fd08)", False),
    "S10":     (None, "R1 has NO convert table and no lha fusion",
                ["lha", "addi", "blr"], "R2 §5.1 (derived fusion)", False),
    "S11":     (["li", "subfc", "subfe", "rlwinm", "blr"], "R1 §2.4 / W-SELECT-5",
                None, "R2 §6.3b: the rlandi form is UNREAD, retracted", False),
    "S12":     (["cmpli", "bc", "li", "li", "blr"], "R1 §8 P3.4 (value-vs-branch bit)",
                ["li", "subfc", "subfe", "addi", "blr"], "R2 §6.3c", False),
}


def main():
    root = sys.argv[1].rstrip("/")
    f1 = f"{root}/docs/whitebox/WB_SELECT_FINDINGS.md"
    f2 = f"{root}/docs/whitebox/WB_SELECT_FINDINGS_R2.md"
    g1 = f"{root}/docs/whitebox/grids/wb-select/frozen.tsv"
    g2 = f"{root}/docs/whitebox/grids/wb-select2/frozen.tsv"

    m1 = parse_measured_grid1(f1)
    m2, pub2 = parse_measured_grid2(f2)
    fr1 = parse_frozen_grid1(g1)
    pub1 = parse_published_grid1(f1)

    assert len(m1) == 12, f"grid-1 measured: {len(m1)}"
    assert len(m2) == 12, f"grid-2 measured: {len(m2)}"
    assert len(fr1) == 12, f"grid-1 frozen: {len(fr1)}"

    # ---------------- CONTROL ----------------
    print("=" * 74)
    print("CONTROL — re-derive each lane's PUBLISHED verdicts from its own files")
    print("=" * 74)
    ok = True
    np_, ns = 0, 0
    for cell in fr1:
        gp = grade_grid1_primary(cell, fr1[cell], m1[cell])
        gs = grade_grid1_secondary(cell, fr1[cell], m1[cell])
        pp, ps = pub1[cell]
        good = (gp == pp) and (gs == ps)
        ok &= good
        np_ += gp == "HIT"
        ns += gs == "HIT"
        print(f"  GRID-1 {cell:8s} primary {gp:4s} (doc {pp:4s})  "
              f"secondary {gs:7s} (doc {ps:7s})  {'OK' if good else 'MISMATCH'}")
    nsec_pred = sum(1 for c in fr1 if "not predicted" not in fr1[c]["secondary"])
    print(f"  GRID-1 TOTAL primary {np_}/12 (doc says 10/12)  "
          f"secondary {ns}/{nsec_pred} (doc says 6/10)")
    ok &= (np_ == 10 and ns == 6 and nsec_pred == 10)

    n2 = 0
    for cell in GRID2_SPEC:
        g = grade_grid2(cell, m2[cell])
        good = g == pub2[cell]
        ok &= good
        n2 += g == "HIT"
        print(f"  GRID-2 {cell:4s} core {g:4s} (doc {pub2[cell]:4s})  "
              f"{'OK' if good else 'MISMATCH'}")
    print(f"  GRID-2 TOTAL core {n2}/12 (doc says 9/12)")
    ok &= (n2 == 9)

    if not ok:
        print("\nREFUSING TO SCORE — the control did not reproduce a published "
              "verdict.  Fix the grader before reading anything below.")
        sys.exit(1)
    print("\nALL REPRODUCED — the cross-score below is on the same denominators.\n")

    # ---------------- CROSS ----------------
    meas = {}
    for c, v in m1.items():
        meas[c] = canon_seq([x for x, _ in v])
    for c, v in m2.items():
        meas[c] = canon_seq([x for x, _ in v])

    def score(readings, policy):
        res = {}
        for cell, (p1, s1, p2, s2, zero) in CROSS.items():
            pred, src = (p1, s1) if readings == 1 else (p2, s2)
            if policy == "symmetric" and zero:
                res[cell] = ("ABSTAIN", "against-zero -> FUN_10c1a908, unread by BOTH")
            elif pred is None:
                res[cell] = ("ABSTAIN", src)
            else:
                res[cell] = ("HIT" if canon_seq(pred) == meas[cell] else "MISS", src)
        return res

    for policy in ("published", "symmetric"):
        print("=" * 74)
        print(f"CROSS-SCORE — policy: {policy.upper()}")
        print("=" * 74)
        r1 = score(1, policy)
        r2 = score(2, policy)
        print(f"{'cell':10s} {'emitted':44s} {'R1':9s} {'R2':9s}")
        for cell in CROSS:
            grid = "G1" if cell.startswith("wbs") else "G2"
            print(f"{grid} {cell:7s} {' '.join(meas[cell]):44s} "
                  f"{r1[cell][0]:9s} {r2[cell][0]:9s}")
        for name, r in (("R1 (wb-select)", r1), ("R2 (wb-select2)", r2)):
            for gname, pref in (("GRID-1", "wbs"), ("GRID-2", "S")):
                cells = [c for c in CROSS
                         if (c.startswith("wbs") if pref == "wbs"
                             else not c.startswith("wbs"))]
                h = sum(1 for c in cells if r[c][0] == "HIT")
                m = sum(1 for c in cells if r[c][0] == "MISS")
                a = sum(1 for c in cells if r[c][0] == "ABSTAIN")
                print(f"  {name:16s} on {gname}: "
                      f"{h} HIT / {m} MISS / {a} ABSTAIN   "
                      f"= {h}/{h+m} of the cells it claims, {h}/12 of the grid")
        print()

    print("=" * 74)
    print("ABSTENTION REASONS (published policy)")
    print("=" * 74)
    r1 = score(1, "published")
    r2 = score(2, "published")
    for cell in CROSS:
        for nm, r in (("R1", r1), ("R2", r2)):
            if r[cell][0] == "ABSTAIN":
                print(f"  {nm} {cell:8s} {r[cell][1]}")


if __name__ == "__main__":
    main()

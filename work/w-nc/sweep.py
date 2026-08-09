#!/usr/bin/env python3
"""w-nc — the NON-CODEGEN LAST BLOCKER sweep.

Reads a `c2rs gap --jsonl` scan and answers, per TU:

  * is every emitted function the FBM instrument could grade `fnbyte-exact`?
  * if so, does the TU `match`?

A TU that answers YES/NO is the ALL-EXACT-NO-MATCH population — `w-blockir`'s
`_fltused` shape one day before it converted: every body byte-exact, the whole
obj still wrong, held by an obligation no per-function byte test can see.

Every table asserts its columns against the population total. Reads only the
scan JSONL and the factors TSV; touches no toolchain, no capture cache.

usage: sweep.py SCAN.jsonl FACTORS.tsv
"""
import json
import sys
from collections import Counter

# The FBM partition, verbatim from `c2_harness::gap::fnbytes` — every bucket a
# graded emitted function can land in. `fnbyte-denominator` is their sum and the
# scan checks that identity itself; this script re-checks it per TU so a
# per-TU reading can never be built on a broken partition.
BUCKETS = [
    "fnbyte-exact",
    "fnbyte-differs",
    "fnbyte-partial",
    "fnbyte-reloc-differs",
    "fnbyte-reloc-unknown",
    "fnbyte-refused",
    "fnbyte-unbound",
    "fnbyte-nobytes",
]


def load(scan, factors):
    tus = []
    for line in open(scan):
        r = json.loads(line)
        if "src" not in r:  # record 0 is the provenance header
            continue
        tus.append(r)
    fac = {}
    for line in open(factors):
        if line.startswith("#"):
            continue
        p = line.rstrip("\n").split("\t")
        if len(p) < 8:
            continue
        fac[p[0]] = p[7]
    return tus, fac


def emitn(r, k):
    return r["emit"].get(k, 0)


def main():
    scan, factors = sys.argv[1], sys.argv[2]
    tus, fac = load(scan, factors)

    graded = [r for r in tus if r["class"] != "capture-fail"]
    print(f"TUs in scan            {len(tus)}")
    print(f"graded (not capture-fail) {len(graded)}")
    by_class = Counter(r["class"] for r in tus)
    for k, v in sorted(by_class.items()):
        print(f"  class {k:<14} {v}")
    assert sum(by_class.values()) == len(tus), "class column does not sum"

    # ---- the partition control, per TU (PREREG §3 control 1) ---------------
    broken = 0
    for r in graded:
        d = emitn(r, "fnbyte-denominator")
        s = sum(emitn(r, b) for b in BUCKETS)
        if d != s:
            broken += 1
    print(f"\nper-TU FBM partition broken: {broken}  (must be 0)")
    assert broken == 0, "the per-TU partition does not sum; every reading below is void"

    # ---- the discriminator -------------------------------------------------
    withfns = [r for r in graded if emitn(r, "fnbyte-denominator") > 0]
    allexact = [r for r in withfns if emitn(r, "fnbyte-exact") == emitn(r, "fnbyte-denominator")]
    match_withfns = [r for r in withfns if r["class"] == "match"]
    gold = [r for r in allexact if r["class"] != "match"]

    print(f"\nTUs with fnbyte-denominator > 0        {len(withfns)}   (gap-metric fnbyte-tus)")
    print(f"  of those, class == match             {len(match_withfns)}   [G2]")
    print(f"  of those, exact == denominator       {len(allexact)}")
    print(f"ALL-EXACT-NO-MATCH (the gold)          {len(gold)}   [G3]")
    # `fnbyte-tus-full` applies a whole-TU override: a `match` TU is counted full
    # whatever the per-function route reconstructed. So the published 15 is
    # |match TUs with fns| + |gold| only if every match TU is *also* per-function
    # full. Both readings are printed rather than one asserted.
    raw_full = [
        r for r in withfns
        if emitn(r, "fnbyte-exact") == emitn(r, "fnbyte-denominator") or r["class"] == "match"
    ]
    print(f"fnbyte-tus-full (with match override)  {len(raw_full)}   [G1]")
    match_not_full = [r for r in match_withfns if emitn(r, "fnbyte-exact") != emitn(r, "fnbyte-denominator")]
    print(f"  match TUs NOT per-function full      {len(match_not_full)}  (the override's whole content)")
    for r in match_not_full:
        print(f"      {r['src']}  exact {emitn(r,'fnbyte-exact')}/{emitn(r,'fnbyte-denominator')}")

    print("\n--- the gold population, by name ---")
    if not gold:
        print("  (empty)")
    for r in gold:
        print(f"  {r['src']}")
        print(f"      class={r['class']}  reason={r['reason']}  letters={fac.get(r['src'],'?')}")
        print(f"      fnbyte: exact {emitn(r,'fnbyte-exact')}/{emitn(r,'fnbyte-denominator')}"
              f"  ex_len={r['ex_len']}  fn_total={r['fn_total']} fn_in_class={r['fn_in_class']}")

    # ---- the near-gold band: one non-exact function away -------------------
    near = []
    for r in withfns:
        if r["class"] == "match":
            continue
        d = emitn(r, "fnbyte-denominator")
        e = emitn(r, "fnbyte-exact")
        if 0 < d - e <= 2 and d > 0:
            near.append((d - e, r))
    near.sort(key=lambda t: (t[0], t[1]["src"]))
    print(f"\n--- NEAR-GOLD: 1 or 2 functions short of ALL-EXACT, not matching: {len(near)} ---")
    for gapn, r in near:
        buckets = {b: emitn(r, b) for b in BUCKETS if emitn(r, b) and b != "fnbyte-exact"}
        print(f"  short {gapn:>2}  {r['src']}  letters={fac.get(r['src'],'?')}  "
              f"exact {emitn(r,'fnbyte-exact')}/{emitn(r,'fnbyte-denominator')}  {buckets}")

    # ---- the frontier ------------------------------------------------------
    frontier = [r for r in graded if fac.get(r["src"]) == "ABC--" and r["class"] != "match"]
    print(f"\n--- FRONTIER (A and B and C, not matched): {len(frontier)} ---")
    nrefused = 0
    for r in sorted(frontier, key=lambda r: r["src"]):
        d, e = emitn(r, "fnbyte-denominator"), emitn(r, "fnbyte-exact")
        ref = emitn(r, "fnbyte-refused")
        if ref:
            nrefused += 1
        print(f"  {r['src']}")
        print(f"      exact {e}/{d}  refused {ref}  differs {emitn(r,'fnbyte-differs')}  "
              f"reloc-differs {emitn(r,'fnbyte-reloc-differs')}  unbound {emitn(r,'fnbyte-unbound')}")
    print(f"  frontier TUs carrying >=1 fnbyte-refused: {nrefused} of {len(frontier)}   [G5]")
    print(f"  frontier TUs ALL-EXACT: "
          f"{sum(1 for r in frontier if emitn(r,'fnbyte-denominator') and emitn(r,'fnbyte-exact')==emitn(r,'fnbyte-denominator'))}   [G4]")

    # ---- A and B and C, whole set -----------------------------------------
    abc = [r for r in graded if fac.get(r["src"], "").startswith("ABC")]
    print(f"\nA and B and C, all TUs: {len(abc)}   (matched {sum(1 for r in abc if r['class']=='match')},"
          f" open {sum(1 for r in abc if r['class']!='match')})")
    assert sum(1 for r in abc if r["class"] == "match") + len(frontier) == len(abc), \
        "the ABC column does not sum"

    # ---- the ZERO-BYTE population -----------------------------------------
    # `fnbyte-denominator == 0` means the reference obj has no emitted function
    # for the instrument to grade — there are **no code bytes to get wrong**.
    # A non-matching TU here is the class in its purest form: the entire
    # remaining distance is a whole-obj obligation.
    nofns = [r for r in graded if emitn(r, "fnbyte-denominator") == 0]
    nofns_nomatch = [r for r in nofns if r["class"] != "match"]
    print(f"\n--- ZERO-BYTE: fnbyte-denominator == 0 ---")
    print(f"  graded TUs with no emitted function      {len(nofns)}")
    print(f"    of those, class == match               {len(nofns) - len(nofns_nomatch)}")
    print(f"    of those, NOT matching  [zero-byte gold] {len(nofns_nomatch)}")
    assert len(nofns) + len(withfns) == len(graded), "the denominator column does not sum"
    for r in sorted(nofns_nomatch, key=lambda r: r["src"]):
        print(f"      {r['src']}  letters={fac.get(r['src'],'?')}  reason={r['reason']}  "
              f"ex_len={r['ex_len']} fn_total={r['fn_total']} fn_in_class={r['fn_in_class']}")

    print(f"\n=== THE BYTE-DISTANCE-ZERO POPULATION = {len(gold)} + {len(nofns_nomatch)} "
          f"= {len(gold) + len(nofns_nomatch)} ===")

    # ---- the refusal-STAGE taxonomy over the near-gold band ----------------
    # `fnbyte-decline|parse` is the IL READER; every other stage is downstream of
    # `Ok(func)` and so is a genuine codegen/selector question (lane `w-column`,
    # board #1473, which found the published `selector` count was 100 % reader).
    print("\n--- refusal STAGE over the near-gold band (w-column's codegen column) ---")
    stages = Counter()
    for _, r in near:
        for k, v in r["emit"].items():
            if k.startswith("fnbyte-decline|"):
                stages[k.split("|", 1)[1]] += v
    tot = sum(stages.values())
    for k, v in stages.most_common():
        print(f"  {k:<24} {v}")
    print(f"  {'TOTAL':<24} {tot}")
    short_total = sum(
        emitn(r, "fnbyte-denominator") - emitn(r, "fnbyte-exact") for _, r in near
    )
    unbound_total = sum(emitn(r, "fnbyte-unbound") for _, r in near)
    print(f"  non-exact functions in the band: {short_total} "
          f"(= declines {tot} + unbound {unbound_total})")
    assert tot + unbound_total == short_total, "the stage column does not sum to the band"


if __name__ == "__main__":
    main()

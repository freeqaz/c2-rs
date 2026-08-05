#!/usr/bin/env python3
"""truth.py — lane w-sym. The MODEL-FREE observations.

No rule is consulted here. These are the raw facts of the grid, and prereg
**R4** — the cross-symbol pin of `docs/STORE_SCHEDULE.md` §3 — is decided
entirely inside this file.

Every number is a positive check with a printed count.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402


def main():
    argv = sys.argv[1:]
    path = os.path.join(W, "holdout.tsv" if "--holdout" in argv else "fit.tsv")
    rows = (S.read_rows_unchecked(path) if "--holdout" in argv
            else S.read_rows(path))
    n = len(rows)
    if n == 0:
        raise SystemExit("FAIL: 0 rows loaded")

    one_insn, pin_ok, pin_bad = 0, 0, []
    src_store, src_prod, multi, span = 0, 0, 0, 0
    for r in rows:
        specs, syms = r["specs"], S.sched_syms(r)
        pr = S.producers(specs)
        # 1. every producer materialises EXACTLY ONE instruction (CSE)
        if sorted(r["prods"]) == sorted(pr):
            one_insn += 1
        # 2. THE PIN: the emitted symbol pattern vs the source symbol pattern
        emitted_pat = [syms[k] for k in r["stores"]]
        source_pat = list(syms)
        if emitted_pat == source_pat:
            pin_ok += 1
        else:
            pin_bad.append(r)
        # 3. shape counts
        if len(set(syms)) > 1:
            multi += 1
        if any(len({syms[k] for k in ks}) > 1 for ks in pr.values()):
            span += 1
        if r["stores"] == list(range(len(specs))):
            src_store += 1
        if r["prods"] == sorted(pr, key=lambda j: pr[j][0]):
            src_prod += 1

    print("rows                                    : %d" % n)
    print("every producer emits exactly 1 insn     : %d / %d" % (one_insn, n))
    print("MULTI-SYMBOL cells                      : %d" % multi)
    print("cells with a SPANNING producer          : %d" % span)
    print("store order == SOURCE order             : %d / %d" % (src_store, n))
    print("producer order == first-use order       : %d / %d" % (src_prod, n))
    print("--- R4, the cross-symbol PIN ---")
    print("emitted symbol pattern == source        : %d / %d" % (pin_ok, n))
    print("VIOLATIONS                              : %d" % len(pin_bad))
    for r in pin_bad[:10]:
        print("   %-24s specs=%-22s syms=%s" %
              (r["cid"], ",".join(r["specs"]), "".join(map(str, r["syms"]))))
        print("        %s" % r["emitted"])
    if one_insn != n:
        print("NOTE: %d cells do NOT have one instruction per producer — the "
              "canon or the CSE assumption is wrong on them" % (n - one_insn))
    return 0


if __name__ == "__main__":
    sys.exit(main())

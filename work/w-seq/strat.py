#!/usr/bin/env python3
"""strat.py — SPLICE-0, stratified by body length.

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

    strat.py <scan.jsonl>

`docs/STATUS.md`'s /QXSTALLS lesson: **stratify by body length before believing
any population discrimination.** The claim under test is

> SPLICE-0 holds exactly when the port's own body for the caller is ONE WORD —
> i.e. the port emits no argument setup — and fails whenever there is a setup.

If instead SPLICE-0 merely tracked *short bodies*, this table would show the
success rate falling with the reference's word count inside the `pw1` stratum,
and the `pw>1` stratum would succeed at small sizes. Both are printed.

**MEASURED, and the registered claim above is HALF WRONG** — kept as written
because a stratification that only reports the half that held is not a
stratification:

* SPLICE-**P** (setup ++ callee body) *is* exactly `pw == 1`: **578 of 578** at
  one port word and **0 of 953** at more.
* SPLICE-**0** (the callee body alone) is **not**. It holds at **578/578** in
  the `pw == 1` stratum and at **1,389 of 1,892 (73.4 %)** in the `pw > 1`
  stratum — c2 discards the port's whole setup far more often than it keeps it.
* And the failure is **not** a length effect: the `pw > 1` rate reads
  100 % · 99.8 % · 3.9 % · 0 % · 0 % · 2.4 % · 83.5 % · 72.5 % across ascending
  length buckets. A monotone fall would have been size; this is three named
  idioms (a register rename, a displacement fold, a constant fold) sitting at
  particular sizes.
"""

import collections
import json
import sys


def main():
    rows = {}
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-splice-fn|"):
                _, shape, verdict, words, sym = k.split("|", 4)
                pw, rw, cw = (int(x[2:]) for x in words.split("/"))
                rows[(r["src"], sym)] = dict(
                    shape=shape, pw=pw, rw=rw, cw=cw, spliceP=verdict
                )
            elif k.startswith("fnbyte-splice0-fn|"):
                _, shape, verdict, sym = k.split("|", 3)
                rows.setdefault((r["src"], sym), {})["splice0"] = verdict

    def bucket(n):
        for hi in (1, 2, 3, 4, 6, 8, 12, 20, 10 ** 9):
            if n <= hi:
                return "rw<=%d" % hi if hi < 10 ** 9 else "rw>20"
        return "?"

    print("=== SPLICE-0 by (port words == 1?) x reference length ===")
    print("    the /QXSTALLS control: if SPLICE-0 tracked SIZE rather than the")
    print("    presence of a setup, the two strata would not separate cleanly.")
    tab = collections.Counter()
    for r in rows.values():
        if "splice0" not in r or "pw" not in r:
            continue
        stratum = "pw==1 (no setup)" if r["pw"] == 1 else "pw>1  (a setup)"
        tab[(stratum, bucket(r["rw"]), r["splice0"])] += 1
    order = ["rw<=1", "rw<=2", "rw<=3", "rw<=4", "rw<=6", "rw<=8", "rw<=12",
             "rw<=20", "rw>20"]
    for stratum in ("pw==1 (no setup)", "pw>1  (a setup)"):
        print("\n  %s" % stratum)
        print("    %-8s %8s %8s %8s" % ("len", "exact", "differs", "rate"))
        tot_e = tot_d = 0
        for b in order:
            e = tab[(stratum, b, "exact")]
            d = tab[(stratum, b, "differs")]
            if e + d == 0:
                continue
            print("    %-8s %8d %8d %7.1f%%" % (b, e, d, 100.0 * e / (e + d)))
            tot_e += e
            tot_d += d
        if tot_e + tot_d:
            print("    %-8s %8d %8d %7.1f%%"
                  % ("ALL", tot_e, tot_d, 100.0 * tot_e / (tot_e + tot_d)))
        else:
            print("    (no rows)")


if __name__ == "__main__":
    main()

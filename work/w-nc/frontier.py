#!/usr/bin/env python3
"""w-nc — the FRONTIER and the `<=10` band, one row per EMITTED function.

For every TU in `A and B and C` that does not match, and for every TU in the
published `<=10` blocked-bodies band, print each emitted function the port fails
to reproduce with the reader clause that refused it. Nothing like this existed:
`gap.rs` prints the frontier's codegen COLUMN as four totals, and the per-body
blocker histogram is over all 2.4M bodies rather than the 162k emitted ones.

Needs a `C2RS_NC_KEYS=1` scan.

usage: frontier.py INST.jsonl INST.tsv
"""
import json
import sys


def main():
    scan, factors = sys.argv[1], sys.argv[2]
    rows = [json.loads(l) for l in open(scan)]
    rows = [r for r in rows if "src" in r]
    fac = {}
    for l in open(factors):
        if l.startswith("#"):
            continue
        p = l.rstrip("\n").split("\t")
        if len(p) >= 8:
            fac[p[0]] = p[7]

    def e(r, k):
        return r["emit"].get(k, 0)

    def fns(r):
        out = []
        for k in r["emit"]:
            if k.startswith("fnbyte-parsefn|"):
                _, nm, why = k.split("|", 2)
                out.append((nm, why))
            elif k.startswith("fnbyte-exactfn|"):
                out.append((k.split("|", 1)[1], "EXACT"))
        return sorted(out)

    graded = [r for r in rows if r["class"] != "capture-fail"]

    def block(title, sel):
        pop = [r for r in graded if sel(r)]
        print(f"\n=== {title}: {len(pop)} TUs ===")
        nfn = nex = nref = 0
        for r in sorted(pop, key=lambda r: r["src"]):
            d, x = e(r, "fnbyte-denominator"), e(r, "fnbyte-exact")
            nfn += d
            nex += x
            nref += e(r, "fnbyte-refused")
            print(f"-- {r['src']}  [{fac.get(r['src'],'?')}]  exact {x}/{d}  "
                  f"refused {e(r,'fnbyte-refused')}  differs {e(r,'fnbyte-differs')}  "
                  f"unbound {e(r,'fnbyte-unbound')}")
            for nm, why in fns(r):
                print(f"     {'':2}{nm[:60]:<60}  {why}")
        print(f"   TOTALS: emitted {nfn}, exact {nex}, refused {nref}")
        # The column must account for the population: every emitted function is
        # exact, refused, differs, unbound, reloc-differs or nobytes.
        rest = sum(e(r, k) for r in pop for k in
                   ("fnbyte-differs", "fnbyte-unbound", "fnbyte-reloc-differs",
                    "fnbyte-nobytes", "fnbyte-partial", "fnbyte-reloc-unknown"))
        assert nex + nref + rest == nfn, "the column does not sum to the population"
        print(f"   column sums: {nex} + {nref} + {rest} == {nfn}  OK")

    block("FRONTIER (A and B and C, open)",
          lambda r: fac.get(r["src"]) == "ABC--" and r["class"] != "match")
    block("<=10 blocked-bodies band, open",
          lambda r: r["fn_total"] > 0
          and r["fn_total"] - r["fn_in_class"] <= 10
          and r["class"] != "match")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""mech.py — WHY the alias channel buys zero reach, and what it would be worth
if factor C moved.  Everything here is MEASURED off the same two files as
join.py; nothing is scaled.

    usage: mech.py <factors.tsv> <scan.jsonl> <gap.jsonl>
"""
import collections
import json
import sys

MODELS = ("RGL", "JFP", "ORACLE", "JFP_ALIAS", "ALIAS_IN")


def main():
    fac, scan, gapj = sys.argv[1], sys.argv[2], sys.argv[3]

    F = {}
    for line in open(fac):
        if line.startswith("#"):
            continue
        p = line.rstrip("\n").split("\t")
        F[p[0]] = {"class": p[1], "A": p[2] == "1", "B": p[3] == "1",
                   "C": p[4] == "1", "D": p[5] == "1", "E": p[6] == "1"}
    X = {}
    for line in open(scan):
        r = json.loads(line)
        if r.get("status") != "ok":
            continue
        X[r["src"]] = {m: bool(r["v"][m]["exact"]) for m in MODELS if m in r["v"]}
    # per-TU set of section names OUTSIDE the port writer's vocabulary
    EX, READ = {}, set()
    for line in open(gapj):
        r = json.loads(line)
        if r.get("record"):
            continue
        e = r.get("emit", {})
        if "emit-sec-readable" in e:
            READ.add(r["src"])
        EX[r["src"]] = {k.split("|", 1)[1] for k in e
                        if k.startswith("emit-sec-extra|")}

    J = set(F) & set(X)
    bc = lambda s: F[s]["B"] and F[s]["C"]
    de = lambda s: F[s]["D"] or F[s]["E"]

    print("== WHY: the TUs the ALIAS channel GAINS, classified by which factor "
          "they fail ==")
    for base, new in (("JFP", "JFP_ALIAS"), ("ORACLE", "ALIAS_IN")):
        gained = {s for s in J if X[s].get(new) and not X[s].get(base)}
        print("\n  %s -> %s : %d TUs gained in per-TU exact" % (base, new, len(gained)))
        cnt = collections.Counter()
        for s in gained:
            cnt["fails B" if not F[s]["B"] else "B ok"] += 1
            cnt["fails C" if not F[s]["C"] else "C ok"] += 1
            cnt["in B and C"] += int(bc(s))
        print("        fails B: %d   B ok: %d" % (cnt["fails B"], cnt["B ok"]))
        print("        fails C: %d   C ok: %d" % (cnt["fails C"], cnt["C ok"]))
        print("        ALREADY inside B and C: %d   <-- the reach increment"
              % cnt["in B and C"])
        ex = collections.Counter()
        for s in gained:
            if not F[s]["C"]:
                ex[" + ".join(sorted(EX.get(s, set()))) or "(unreadable obj)"] += 1
        print("        of the C failures, the extra-section set that causes it:")
        for k, v in ex.most_common(8):
            print("          %5d  %s" % (v, k))
        r = sum(1 for s in gained if ".rdata$r" in EX.get(s, set()))
        print("        carrying `.rdata$r` (the RTTI section, w-rdata's target): "
              "%d of %d = %.5f" % (r, len(gained), r / max(1, len(gained))))

    print("\n== THE CONDITIONAL — what the channel is worth IF factor C moves ==")
    print("  C_rdata := the TU's extra-section set is a subset of {.rdata$r},")
    print("  i.e. factor C after the writer learns ONE more name.  This is the")
    print("  ladder's next step and it is MEASURED here, not projected.")
    steps = [("C  (today, 10 writer names)", set()),
             ("C + .rdata$r", {".rdata$r"}),
             ("C + .rdata$r + .text$yd", {".rdata$r", ".text$yd"}),
             ("C + all 3 (vocabulary closed)",
              {".rdata$r", ".text$yd", ".xdata$x"})]
    for label, taught in steps:
        Cs = {s for s in F if s in READ and EX.get(s, set()) <= taught}
        BCs = {s for s in F if F[s]["B"] and s in Cs}
        BCJ = BCs & J
        row = ["  %-30s C=%3d  B&C=%3d  (B&C on the join %3d)"
               % (label, len(Cs), len(BCs), len(BCJ))]
        for m in MODELS:
            R = {s for s in BCJ if X[s].get(m)}
            row.append("%s %3d" % (m, len(R)))
        print("%s | %s" % (row[0], "  ".join(row[1:])))

    print("\n  the same rows as INCREMENTS (what the alias channel buys):")
    for label, taught in steps:
        Cs = {s for s in F if s in READ and EX.get(s, set()) <= taught}
        BCJ = {s for s in F if F[s]["B"] and s in Cs} & J
        j = len({s for s in BCJ if X[s].get("JFP")})
        ja = len({s for s in BCJ if X[s].get("JFP_ALIAS")})
        o = len({s for s in BCJ if X[s].get("ORACLE")})
        a = len({s for s in BCJ if X[s].get("ALIAS_IN")})
        print("  %-30s  JFP_ALIAS-JFP %+4d   ALIAS_IN-ORACLE %+4d"
              % (label, ja - j, a - o))

    print("\n== THE HEADROOM INSIDE B and C — why the increment is what it is ==")
    BCJ = {s for s in J if bc(s)}
    for m in MODELS:
        R = {s for s in BCJ if X[s].get(m)}
        print("  %-11s exact on %3d of the %d B&C TUs in the join ; "
              "%3d NOT exact" % (m, len(R), len(BCJ), len(BCJ) - len(R)))
    miss = sorted(s for s in BCJ if not X[s].get("ALIAS_IN"))
    print("  the %d B&C TUs even the NEW CEILING misses, by name:" % len(miss))
    for s in miss:
        print("        %s  [%s]" % (s, F[s]["class"]))


if __name__ == "__main__":
    main()

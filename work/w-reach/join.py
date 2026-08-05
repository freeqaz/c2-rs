#!/usr/bin/env python3
"""join.py — the join, then the intersections.  MEASURED, never scaled.

    usage: join.py <factors.tsv> <scan.jsonl>

`factors.tsv` is `c2rs gap --factors-tsv` (871 graded TUs, per-TU A/B/C/D/E).
`scan.jsonl` is w-emitp's scan (850 TUs, per-TU exact for each model variant).

The join is by SOURCE PATH, printed with both differences by name before a
single intersection is quoted.  stdlib only.
"""
import json
import sys

MODELS = ("RGL", "JFP", "ORACLE", "JFP_ALIAS", "ALIAS_IN", "ALIAS_BOTH")


def main():
    fac, scan = sys.argv[1], sys.argv[2]

    # ---- the 871 -----------------------------------------------------------
    F = {}
    for line in open(fac):
        if line.startswith("#"):
            continue
        p = line.rstrip("\n").split("\t")
        F[p[0]] = {"class": p[1], "A": p[2] == "1", "B": p[3] == "1",
                   "C": p[4] == "1", "D": p[5] == "1", "E": p[6] == "1"}
    # ---- the 850 -----------------------------------------------------------
    X = {}
    for line in open(scan):
        r = json.loads(line)
        if r.get("status") != "ok":
            continue
        X[r["src"]] = {m: bool(r["v"][m]["exact"]) for m in MODELS if m in r["v"]}

    g, x = set(F), set(X)
    print("== THE JOIN, by source path — printed BEFORE any intersection ==")
    print("  gap graded TUs (factors.tsv)          : %d" % len(g))
    print("  w-emitp model TUs (scan.jsonl)        : %d" % len(x))
    print("  in BOTH (the join)                    : %d" % len(g & x))
    print("  in w-emitp and NOT gap-graded         : %d" % len(x - g))
    for s in sorted(x - g):
        print("        MODEL-ONLY %s" % s)
    print("  gap-graded and NOT in w-emitp         : %d" % len(g - x))
    for s in sorted(g - x):
        print("        GAP-ONLY   %s  [%s %s]"
              % (s, F[s]["class"],
                 "".join(k for k in "ABCDE" if F[s][k]) or "-----"))

    J = g & x

    def bc(s):
        return F[s]["B"] and F[s]["C"]

    def de(s):
        return F[s]["D"] or F[s]["E"]

    BC = {s for s in g if bc(s)}
    BCJ = {s for s in J if bc(s)}
    print("\n== THE DENOMINATORS — both printed, never one ==")
    print("  |B and C| over the 871 graded          : %d" % len(BC))
    print("  |B and C| over the %d-TU JOIN          : %d" % (len(J), len(BCJ)))
    print("  |B and C| lost to the join             : %d" % len(BC - BCJ))
    for s in sorted(BC - BCJ):
        print("        NOT IN JOIN %s  [%s]" % (s, F[s]["class"]))
    print("  |A and B and C| over the 871           : %d"
          % sum(1 for s in g if F[s]["A"] and bc(s)))
    print("  |A and B and C| over the join          : %d"
          % sum(1 for s in J if F[s]["A"] and bc(s)))
    matched = {s for s in g if F[s]["class"] == "match"}
    print("  match TUs over the 871 / in the join   : %d / %d"
          % (len(matched), len(matched & J)))

    print("\n== THE INTERSECTIONS — |{model exact} and B and C|, MEASURED ==")
    print("  %-11s %8s %8s | %8s %9s %9s | %8s"
          % ("model", "exact", "exact_J", "REACH", "of|B&C_J|", "H0 prod",
             "vs H0"))
    out = {}
    for m in MODELS:
        if m not in next(iter(X.values())):
            continue
        E = {s for s in J if X[s].get(m)}
        R = E & BCJ
        h0 = len(E) * len(BCJ) / len(J)
        out[m] = R
        print("  %-11s %8d %8d | %8d %9.5f %9.1f | %7.2fx"
              % (m, sum(1 for s in J if X[s].get(m)), len(E), len(R),
                 len(R) / len(BCJ), h0, len(R) / h0 if h0 else float("nan")))

    print("\n== THE INCREMENTS — what the ALIAS channel is worth in REACH ==")
    for base, new in (("JFP", "JFP_ALIAS"), ("ORACLE", "ALIAS_IN")):
        if new not in out:
            continue
        b, n = out[base], out[new]
        print("  %-10s -> %-10s   %3d -> %3d   gained %3d  lost %3d"
              % (base, new, len(b), len(n), len(n - b), len(b - n)))

    print("\n== THE STRUCTURAL SPLIT — reach is not conversion ==")
    print("  For each model, its B&C reach split by whether the port already has")
    print("  an accepted route to the contents (D or E) and whether it matches.")
    print("  %-11s %6s | %8s %9s %9s"
          % ("model", "REACH", "match", "D|E,!match", "!D&!E,!match"))
    for m in MODELS:
        if m not in out:
            continue
        R = out[m]
        mt = sum(1 for s in R if F[s]["class"] == "match")
        conv = sorted(s for s in R if F[s]["class"] != "match" and de(s))
        front = sum(1 for s in R if F[s]["class"] != "match" and not de(s))
        print("  %-11s %6d | %8d %9d %9d" % (m, len(R), mt, len(conv), front))
    for m in ("JFP", "JFP_ALIAS", "ORACLE", "ALIAS_IN"):
        if m not in out:
            continue
        conv = sorted(s for s in out[m]
                      if F[s]["class"] != "match" and de(s))
        print("    %-11s zero-codegen candidates (%d): %s"
              % (m, len(conv), ", ".join(conv) if conv else "NONE"))

    # The two TUs board #213's divergence names, graded by every model.
    div = sorted(s for s in g
                 if not F[s]["A"] and bc(s) and de(s) and F[s]["class"] != "match")
    print("\n== THE PROJECTION DIVERGENCE (board #213), graded by each model ==")
    for s in div:
        inj = s in J
        print("  %s  in-join=%s  %s" % (
            s, inj,
            "  ".join("%s=%s" % (m, X[s].get(m)) for m in MODELS) if inj
            else "NOT IN THE 850 — ungradeable by any of these models"))

    # The frontier under each model, for the rung's ladder line.
    FRONT = {s for s in g if F[s]["A"] and bc(s) and not de(s)
             and F[s]["class"] != "match"}
    print("\n== FRONTIER, for reference: %d (A and B and C, not match, not D or E) ==" % len(FRONT))
    print("  of those, in the join: %d" % len(FRONT & J))


if __name__ == "__main__":
    main()

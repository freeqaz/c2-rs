#!/usr/bin/env python3
"""grade.py — score frozen predictions against the compiler's own answer.

Reads `predict.py`'s jsonl and a directory of truth files (`<slug>.txt`, one
whitespace-separated decorated name per emitted code COMDAT leader) and prints,
for every model:

  * **per-TU exact BY NAME** — the headline, and the set is printed, never only
    its size (`STATUS.md` trap 8 / board #250: a count is not a set)
  * micro precision / recall / F1 — SECONDARY, and labelled as such

    usage: grade.py <predictions.jsonl> <truth-dir> [label]

stdlib only.  Reads no obj; the truth directory is built separately.
"""
import json
import os
import sys

MODELS = ("NEVER", "ALL", "RGL", "INIT", "SKIP", "JFP", "JFP_ALIAS", "ALIAS_IN")


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def main():
    predp, truthd = sys.argv[1], sys.argv[2]
    label = sys.argv[3] if len(sys.argv) > 3 else ""
    rows = [json.loads(l) for l in open(predp) if l.strip()]
    rows = [r for r in rows if r.get("status") == "ok"]

    truth, missing = {}, []
    for r in rows:
        tf = os.path.join(truthd, slug(r["src"]) + ".txt")
        if not os.path.exists(tf):
            missing.append(r["src"])
            continue
        truth[r["src"]] = set(x for x in open(tf).read().split() if x)
    if missing:
        print("!! NO TRUTH for %d TUs: %s" % (len(missing), missing[:10]))
    rows = [r for r in rows if r["src"] in truth]
    n = len(rows)
    print("== %s : %d TUs graded ==" % (label, n))

    have = [m for m in MODELS if any(m in r["P"] for r in rows)]
    print("\n%-11s %10s %10s %10s %10s  | %s"
          % ("model", "precision", "recall", "F1", "|P|", "EXACT / %d" % n))
    exact_sets = {}
    for m in have:
        tp = fp = fn = np_ = 0
        ex = []
        for r in rows:
            P = set(r["P"][m])
            E = truth[r["src"]]
            tp += len(P & E)
            fp += len(P - E)
            fn += len(E - P)
            np_ += len(P)
            if P == E:
                ex.append(r["src"])
        exact_sets[m] = ex
        pr = tp / (tp + fp) if tp + fp else 0.0
        rc = tp / (tp + fn) if tp + fn else 0.0
        f1 = 2 * pr * rc / (pr + rc) if pr + rc else 0.0
        print("%-11s %10.5f %10.5f %10.5f %10d  | %d  (%.5f)"
              % (m, pr, rc, f1, np_, len(ex), len(ex) / n if n else 0.0))

    print("\n-- per-TU EXACT, BY NAME (a count is not a set) --")
    for m in have:
        print("  %s : %d" % (m, len(exact_sets[m])))
        for s in sorted(exact_sets[m]):
            print("      %s" % s)

    out = {"n": n, "exact": dict((m, sorted(exact_sets[m])) for m in have)}
    with open(predp + ".graded", "w") as fh:
        json.dump(out, fh, indent=1)
    print("\nwrote %s.graded" % predp)


if __name__ == "__main__":
    main()

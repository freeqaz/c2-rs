#!/usr/bin/env python3
"""shatab.py — the per-TU prediction digest table that goes into the prereg.

The predicted sets themselves are committed (`pred21.sets.txt`); this is the
index over them, so a reader can check any single TU without diffing 6 MB.

    usage: shatab.py <predictions.jsonl>
"""
import hashlib
import json
import sys

MODELS = ("NEVER", "ALL", "RGL", "INIT", "SKIP", "JFP", "JFP_ALIAS")


def main():
    rows = sorted([json.loads(l) for l in open(sys.argv[1]) if l.strip()],
                  key=lambda r: r["src"])
    print("%-64s %6s %7s %7s  %s"
          % ("src", "|U|", "|JFP|", "|JFP_A|",
             "sha256(JFP_ALIAS set, sorted, LF-terminated)"))
    for r in rows:
        print("%-64s %6d %7d %7d  %s"
              % (r["src"], r["n_U"], len(r["P"]["JFP"]),
                 len(r["P"]["JFP_ALIAS"]), r["sha"]["JFP_ALIAS"]))
    print()
    for m in MODELS:
        cat = "".join(r["sha"][m] for r in rows)
        print("%-10s corpus-sha256 %s   sum|P| %d"
              % (m, hashlib.sha256(cat.encode()).hexdigest(),
                 sum(len(r["P"][m]) for r in rows)))


if __name__ == "__main__":
    main()

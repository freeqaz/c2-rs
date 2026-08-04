#!/usr/bin/env python3
"""ka6.py — the KA6 hand-check draw, plus a post-hoc draw from the LOOSE-only
edge set so the two extractors' artifact rates can be compared on equal terms.

Seeded uniform draws, seed printed, so the sample is reproducible and was not
chosen after seeing the answers.

    usage: ka6.py <x.jsonl> [seed]
"""
import json
import random
import sys


VAR = "local"


def main():
    global VAR
    path = sys.argv[1]
    seed = int(sys.argv[2]) if len(sys.argv) > 2 else 20260804
    rows = [json.loads(l) for l in open(path) if '"ok"' in l]
    ok = [r for r in rows if r["status"] == "ok"]
    pairs = []
    for r in ok:
        d = r[VAR]
        for f in d["x26"]:
            pairs.append((r["src"], f, d.get("xref", {}).get(f, [])))
    pairs.sort()
    rnd = random.Random(seed)
    samp = rnd.sample(pairs, min(20, len(pairs)))
    print("KA6 draw: seed=%d, |X|=%d pairs, n=%d" % (seed, len(pairs), len(samp)))
    for i, (src, f, refs) in enumerate(samp, 1):
        print("\n%2d. TU      %s" % (i, src))
        print("    target  %s" % f)
        print("    emitted referrer(s): %s" % (", ".join(refs) if refs else "(none recorded)"))


if __name__ == "__main__":
    main()

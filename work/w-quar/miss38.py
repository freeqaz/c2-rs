#!/usr/bin/env python3
"""miss38.py — eight of the twelve misses have EXACTLY 38 false negatives.

A constant that recurs across eight unrelated TUs is not a distribution, it is a
set.  This prints the intersection of the false-negative sets over the missed
TUs, the names in it, and — the part that decides whether the model could ever
have predicted them — whether each is in the model's node universe `U` at all.

    usage: miss38.py <predictions.jsonl> <truth-dir> <cache-index.tsv>
"""
import json
import os
import sys

MAIN = os.environ.get("C2RS_LANEROOT")
if not MAIN:
    raise SystemExit("set C2RS_LANEROOT")
for _p in ("work/w-emitp", "work/emitpred/pipeline", "work/w-roots",
           "work/w-refs", "work/w-mark", "work/w-skip", "work/w-db"):
    sys.path.insert(0, os.path.join(MAIN, _p))
import il      # noqa: E402
import refs    # noqa: E402
import boundary2  # noqa: E402


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def base_of(entry):
    for n in os.listdir(entry):
        if n.startswith("_CL_") and n.endswith("gl"):
            return n[:-2]
    return None


def main():
    predp, truthd, idxp = sys.argv[1:4]
    entries = dict((l.split("\t")[0], l.split("\t")[1])
                   for l in (x.rstrip("\n") for x in open(idxp)) if l)
    rows = sorted([json.loads(l) for l in open(predp) if l.strip()],
                  key=lambda r: r["src"])
    fns, us = {}, {}
    for r in rows:
        E = set(x for x in open(os.path.join(truthd, slug(r["src"]) + ".txt"))
                .read().split() if x)
        P = set(r["P"]["JFP_ALIAS"])
        fns[r["src"]] = E - P
        e = entries[r["src"]]
        glb = open(os.path.join(e, base_of(e) + "gl"), "rb").read()
        exb = open(os.path.join(e, base_of(e) + "ex"), "rb").read()
        recs, _ = refs.scan(glb, exb, wide_count=True)
        us[r["src"]] = set(recs)

    missed = [s for s in fns if fns[s]]
    core = set.intersection(*[fns[s] for s in missed]) if missed else set()
    print("TUs with >=1 false negative: %d" % len(missed))
    print("INTERSECTION of their FN sets: %d names" % len(core))
    print()
    for n in sorted(core):
        inu = sum(1 for s in missed if n in us[s])
        print("  %-64s  in U on %d/%d of them   [%s]"
              % (n, inu, len(missed), boundary2.kind(n)))
    print()
    tot = 0
    for s in sorted(missed):
        outside = sum(1 for n in fns[s] if n not in us[s])
        tot += outside
        print("  %-58s FN %3d ; of which OUTSIDE U %3d ; core-38 %3d"
              % (s[:58], len(fns[s]), outside, len(fns[s] & core)))
    print("\n  false negatives lying OUTSIDE the node universe U (no closure "
          "over U can ever predict them): %d of %d"
          % (tot, sum(len(v) for v in fns.values())))


if __name__ == "__main__":
    main()

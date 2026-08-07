#!/usr/bin/env python3
"""miss.py — the miss structure, and the two prereg cells score.py cannot print.

  * **Q11** `dom(alias) ∩ E` — an alias must never itself be emitted.  w-emitp
    measured 0 over 174,417 emitted names in sample; `score.py` prints
    `target in E` and not this, so it is computed here.
  * **Q14** TU *reach* — `|{TU : model exact} ∩ B∧C|`, board #302's quantity.
    The six quarantined `B∧C` TUs are named by board **#348**, which established
    the 850/871 join; they are hard-coded here because `gap.rs` has no per-TU
    `B∧C` listing (w-emitp §8 item 1).
  * the per-TU residual: how far each missed TU is from exact, in both
    directions, so "12 misses" is not left as one number.

    usage: miss.py <predictions.jsonl> <truth-dir> <cache-index.tsv>
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
import alias as al  # noqa: E402

# board #348: the six quarantined TUs that are in `B∧C` on the 850/871 join.
BNC = {
    "src/keygen_xbox.cpp",
    "src/system/os/Archive.cpp",
    "src/system/os/HolmesClient_NetSocket.cpp",
    "src/system/os/Keyboard_Xbox.cpp",
    "src/system/utl/Compress.cpp",
    "src/system/utl/Option.cpp",
}


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
    truth = dict((r["src"],
                  set(x for x in open(os.path.join(truthd, slug(r["src"]) + ".txt"))
                      .read().split() if x)) for r in rows)

    # ---- Q11 -------------------------------------------------------------
    dom_in_E = 0
    tag10 = bound = shape = 0
    for r in rows:
        e = entries[r["src"]]
        glb = open(os.path.join(e, base_of(e) + "gl"), "rb").read()
        AL, _t, st = al.scan(glb, shift=0)
        tag10 += st["tag10"]
        bound += st["bound"]
        shape += sum(1 for k, v in AL.items()
                     if k.startswith("??_E") and v.startswith("??_G")
                     and k[4:] == v[4:])
        dom_in_E += len(set(AL) & truth[r["src"]])
    print("Q9   bound / tag-0x10 : %d / %d = %.5f" % (bound, tag10, bound / tag10))
    print("Q11  dom(alias) INTERSECT E : %d  (over %d emitted names)"
          % (dom_in_E, sum(len(v) for v in truth.values())))
    print("Q12  ??_E<X> -> ??_G<X> share of bound : %d / %d = %.5f"
          % (shape, bound, shape / bound))

    # ---- Q14, and the reach increment ------------------------------------
    print()
    for m in ("JFP", "JFP_ALIAS"):
        ex = set(r["src"] for r in rows if set(r["P"][m]) == truth[r["src"]])
        print("Q14  %-10s exact %2d   exact INTERSECT B^C %d of %d"
              % (m, len(ex), len(ex & BNC), len(BNC)))
    exj = set(r["src"] for r in rows if set(r["P"]["JFP"]) == truth[r["src"]])
    exa = set(r["src"] for r in rows if set(r["P"]["JFP_ALIAS"]) == truth[r["src"]])
    print("     alias channel: gained %d, lost %d ; REACH increment %+d"
          % (len(exa - exj), len(exj - exa),
             len(exa & BNC) - len(exj & BNC)))
    print("     gained, by name : %s" % sorted(exa - exj))
    print("     ... of which in B^C : %s" % sorted((exa - exj) & BNC))

    # ---- the per-TU residual ---------------------------------------------
    print("\n-- JFP_ALIAS, per TU: how far from exact, both directions --")
    print("%-58s %5s %6s %5s %5s %6s %s"
          % ("src", "|E|", "|P|", "FN", "FP", "B^C", "verdict"))
    near1 = near3 = under = over = both = 0
    for r in rows:
        P, E = set(r["P"]["JFP_ALIAS"]), truth[r["src"]]
        fn, fp = len(E - P), len(P - E)
        d = fn + fp
        if d == 0:
            v = "EXACT"
        else:
            v = "miss"
            if d == 1:
                near1 += 1
            if d <= 3:
                near3 += 1
            if fn and not fp:
                under += 1
            elif fp and not fn:
                over += 1
            else:
                both += 1
        print("%-58s %5d %6d %5d %5d %6s %s"
              % (r["src"][:58], len(E), len(P), fn, fp,
                 "yes" if r["src"] in BNC else "-", v))
    print("\n  of the misses: %d are ONE symbol from exact, %d within three ; "
          "%d under-predict only, %d over-predict only, %d both"
          % (near1, near3, under, over, both))


if __name__ == "__main__":
    main()

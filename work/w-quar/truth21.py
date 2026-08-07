#!/usr/bin/env python3
"""truth21.py — build truth for the 21 from the SOLE JUDGE's own output.

For each quarantined TU:

  * `E(t)` — the reference obj's **code COMDAT leader set**, read by w-joint's
    `objsyms.py` unmodified (`objsyms.sets()["E"]`), which selects sections by the
    `IMAGE_SCN_CNT_CODE` characteristic and never by a `.text` name prefix.
  * `D(t)` — the same obj's defined-symbol sets, written out so the
    ORACLE-conditioned ceiling `ALIAS_IN` can be scored afterwards as an upper
    reference (prereg Q13).
  * **Q15, the toolchain control** — the freshly compiled obj is byte-compared
    against the capture-cache's own `out.obj` for the same TU at dc3 `940d07dc`,
    with the COFF `TimeDateStamp` (file offset 4..8) zeroed.  The two were
    produced by different wibo builds months apart; if they disagree, the 850's
    cached truth and this lane's fresh truth are not the same instrument and the
    verdict is withheld.
  * The `.text`-name-prefix rule is computed too and its agreement with the
    characteristic rule printed, so the trap `truth_data.py` documents is
    checked here as well.

    usage: truth21.py <objdir> <cache-index.tsv> <out-truth-dir> <out-dtruth-dir>

stdlib only.
"""
import json
import os
import sys

MAIN = os.environ.get("C2RS_LANEROOT")
if not MAIN:
    raise SystemExit("set C2RS_LANEROOT")
sys.path.insert(0, os.path.join(MAIN, "work", "w-joint"))
import objsyms  # noqa: E402


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def zero_ts(b):
    return b[:4] + b"\0\0\0\0" + b[8:]


def main():
    objdir, idxp, truthd, dtruthd = sys.argv[1:5]
    os.makedirs(truthd, exist_ok=True)
    os.makedirs(dtruthd, exist_ok=True)
    rows = [l.rstrip("\n").split("\t") for l in open(idxp) if l.strip()]

    n_ok = n_same = n_namerule = 0
    print("%-64s %6s %7s %7s  %s" % ("src", "|E|", "|D_all|", "objlen", "Q15"))
    for r in rows:
        src, entry = r[0], r[1]
        fresh = open(os.path.join(objdir, slug(src) + ".obj"), "rb").read()
        o = objsyms.ObjSyms(fresh)
        if not o.ok:
            print("  COFF-REJECT %s %s" % (src, o.err))
            continue
        s = objsyms.sets(o)
        nm = objsyms.name_rule_E(o)
        if nm == s["E"]:
            n_namerule += 1

        cached = open(os.path.join(entry, "out.obj"), "rb").read()
        same = zero_ts(fresh) == zero_ts(cached)
        n_same += 1 if same else 0
        n_ok += 1

        with open(os.path.join(truthd, slug(src) + ".txt"), "w") as fh:
            fh.write("\n".join(s["E"]) + "\n")
        json.dump({"src": src, "E": s["E"], "D_all": s["D_all"],
                   "D_data": s["D_data"], "D_lead": s["D_lead"],
                   "undef": s["U_undef"]},
                  open(os.path.join(dtruthd, slug(src) + ".json"), "w"))
        print("%-64s %6d %7d %7d  %s"
              % (src, len(s["E"]), len(s["D_all"]), len(fresh),
                 "byte-identical" if same else "*** DIFFERS ***"))

    print("\nQ15  fresh vs cached obj, TimeDateStamp zeroed: %d/%d byte-identical"
          % (n_same, n_ok))
    print("     .text-prefix rule agrees with the characteristic rule: %d/%d"
          % (n_namerule, n_ok))


if __name__ == "__main__":
    main()

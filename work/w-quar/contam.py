#!/usr/bin/env python3
"""contam.py — prove the 21-TU quarantine has never been inside a fitting set.

w-emitp2 printed `heldout ∩ cacheidx = 0` against its 850 and board #964 records
it.  This re-derives it rather than citing it, and widens the question to every
population on disk that a model in this lineage could have been fitted on,
scored on, or driven by.

**The discriminator, stated because the naive check gives a false alarm.**  The
harness's own workload scans (`c2rs gap`) run over all **878** TUs, so the 21 are
in every one of them by construction — that is not contamination, because those
rows carry no c2 *emitted-symbol-set* quantity at all: their keys are c1xx-side
(`fn_names`, `fn_total`, `ex_len`, the blocker histograms), port-side
(`emit-*` = what **PortC2** emitted, never what c2 emitted), plus `class` and
`replay_ok`, and the prereg drew its own population from exactly those class
labels.  A row is classified **EMIT-MODEL** here iff it carries one of the keys
an emit-set grader writes (`v`, `n_E`, `n_E_in_U`, `exact`, `E`), and only those
populations are required to be disjoint.  Every gap-scan row's key set is
printed with `emit-set keys: NONE` so the classification is checkable and an
absent check can never read as a pass.

    usage: contam.py <lane-root> <main-root>
"""
import json
import os
import sys

# w-emitpred prereg §"Part 1": the truth-OPEN development set.  Listed so the
# gate's own dev population is checked for leakage into the quarantine too.
DEV = [
    "src/system/char/Part.cpp",
    "src/system/char/CharWeightSetter.cpp",
    "src/system/rndobj/Gen.cpp",
    "src/system/char/CharClipDriver.cpp",
    "src/system/flow/FlowLabel.cpp",
    "src/system/synth/MoggClipMap.cpp",
    "src/system/rndobj/ShadowMap.cpp",
    "src/system/utl/HolmesUtl.cpp",
]

EMIT_MODEL_KEYS = ("v", "n_E", "n_E_in_U", "exact", "E", "n_U")


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def read_lines(p, col=0):
    if not os.path.exists(p):
        return None
    out = set()
    for ln in open(p):
        ln = ln.rstrip("\n")
        if not ln.strip():
            continue
        out.add(ln.split("\t")[col].strip())
    return out


def main():
    lane, main = sys.argv[1], sys.argv[2]
    held = read_lines(os.path.join(lane, "work/emitpred/magnitude/heldout.txt"))
    print("quarantine: %d TUs" % len(held))
    for t in sorted(held):
        print("    %s" % t)
    print()

    fail = []

    def check(label, pop, expect_zero=True, unit="pop"):
        if pop is None:
            print("  %-48s ABSENT" % label)
            return
        inter = sorted(held & pop)
        ok = (len(inter) == 0) == expect_zero
        if not ok:
            fail.append(label)
        print("  %-48s |%s| %6d   inter %2d   %s"
              % (label, unit, len(pop), len(inter),
                 "OK" if ok else "*** VIOLATION ***"))
        for x in inter[:25]:
            print("        %s" % x)

    print("== FITTING / SCORING POPULATIONS — must be DISJOINT ==")
    check("work/w-db/cacheidx.tsv (the 850 model corpus)",
          read_lines(os.path.join(lane, "work/w-db/cacheidx.tsv")))
    check("magnitude/truthlist.txt (every truth read, 857)",
          read_lines(os.path.join(lane, "work/emitpred/magnitude/truthlist.txt")))
    check("w-emitpred DEV (truth-open, 8)", set(DEV))

    print("\n== CONTROLS — the 21 MUST appear, or the test cannot fail ==")
    check("magnitude/tus.txt (878 workload)",
          read_lines(os.path.join(lane, "work/emitpred/magnitude/tus.txt")),
          expect_zero=False)
    check("dc3-workload/files.txt (878 workload)",
          read_lines(os.path.join(main, "work/dc3-workload/files.txt")),
          expect_zero=False)

    print("\n== TRUTH ARTIFACTS ON DISK — must not exist for a quarantined TU ==")
    for rel, ext in (("work/w-emit/truth", ".txt"),
                     ("work/emitpred-truth", ".txt"),
                     ("work/emitpred-truth", ".meta")):
        td = os.path.join(main, rel)
        if not os.path.isdir(td):
            print("  %-48s ABSENT" % (rel + "/*" + ext))
            continue
        present = sorted(t for t in held
                         if os.path.exists(os.path.join(td, slug(t) + ext)))
        if present:
            fail.append(rel + ext)
        print("  %-48s files %6d   inter %2d   %s"
              % (rel + "/*" + ext, len(os.listdir(td)), len(present),
                 "OK" if not present else "*** VIOLATION ***"))
        for x in present:
            print("        %s" % x)

    for lanedir in sorted(os.listdir(os.path.join(main, "work"))):
        for sub in ("dtruth", "truth"):
            dt = os.path.join(main, "work", lanedir, sub)
            if not os.path.isdir(dt) or lanedir == "w-emit":
                continue
            present = sorted(t for t in held
                             if os.path.exists(os.path.join(dt, slug(t) + ".json"))
                             or os.path.exists(os.path.join(dt, slug(t) + ".txt")))
            if present:
                fail.append("work/%s/%s" % (lanedir, sub))
            print("  %-48s files %6d   inter %2d   %s"
                  % ("work/%s/%s" % (lanedir, sub), len(os.listdir(dt)),
                     len(present), "OK" if not present else "*** VIOLATION ***"))
            for x in present:
                print("        %s" % x)

    print("\n== EVERY scan*.jsonl ON DISK, classified ==")
    n_model = n_gap = 0
    for lanedir in sorted(os.listdir(os.path.join(main, "work"))):
        d = os.path.join(main, "work", lanedir)
        if not os.path.isdir(d):
            continue
        for fn in sorted(os.listdir(d)):
            if not (fn.startswith("scan") and fn.endswith(".jsonl")):
                continue
            srcs, keys = set(), set()
            try:
                for ln in open(os.path.join(d, fn)):
                    ln = ln.strip()
                    if not ln:
                        continue
                    try:
                        r = json.loads(ln)
                    except ValueError:
                        continue
                    if isinstance(r, dict):
                        srcs.add(r.get("src"))
                        keys |= set(r.keys())
            except OSError:
                continue
            is_model = bool(keys & set(EMIT_MODEL_KEYS))
            inter = sorted(x for x in held & srcs)
            if is_model:
                n_model += 1
                ok = not inter
                if not ok:
                    fail.append("work/%s/%s" % (lanedir, fn))
                print("  EMIT-MODEL %-36s rows %6d  inter %2d  %s"
                      % ("work/%s/%s" % (lanedir, fn), len(srcs), len(inter),
                         "OK" if ok else "*** VIOLATION ***"))
                for x in inter[:10]:
                    print("        %s" % x)
            else:
                n_gap += 1
                print("  gap-scan   %-36s rows %6d  inter %2d  emit-set keys: NONE"
                      % ("work/%s/%s" % (lanedir, fn), len(srcs), len(inter)))
    print("\n  classified: %d EMIT-MODEL scans, %d harness gap scans"
          % (n_model, n_gap))
    if n_model == 0:
        print("  NOTE — an EMIT-MODEL scan jsonl is a REGENERATED artifact and\n"
              "  none is committed, so `0` here is an ABSENCE and is NOT the\n"
              "  disjointness evidence.  The evidence is one line above: every\n"
              "  emit-model scan takes `work/w-db/cacheidx.tsv` as its ONLY TU\n"
              "  list (w-emitp/scan.py `main()`, w-emitp2/scan2.py,\n"
              "  w-inread/scan2w.py all read argv[1] = that file and nothing\n"
              "  else), and that file's intersection with the quarantine is 0.\n"
              "  A model scan cannot reach a TU its input list does not name.")

    print("\nVIOLATIONS: %d %s" % (len(fail), fail if fail else ""))
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()

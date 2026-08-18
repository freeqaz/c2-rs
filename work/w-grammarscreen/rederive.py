#!/usr/bin/env python3
"""w-grammarscreen — DERIVE the results table from the raw logs.

`docs/rungs/README.md` probe rule 2: *derive the table from the logs, never
accumulate it.* This script is the only thing that produces a number this lane
quotes, and it reads only `work/w-grammarscreen/sites.jsonl` (the parsed
enumeration) and `work/w-grammarscreen/logs/<tag>.<stage>.hits` (raw probe
output). Nothing is carried forward between invocations.

    rederive.py <tag> [tag ...]

Writes `work/w-grammarscreen/reached.<tag>.txt` — the REACHED site list, in the
`file:line:col` form the panic-mode confirmation consumes.
"""
import collections
import json
import os
import sys

ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
LANE = os.path.join(ROOT, "work", "w-grammarscreen")
LOGS = os.path.join(LANE, "logs")
STAGES = ["suite", "bench", "sweep", "cross", "debug", "gate", "scan"]


def load_sites():
    """The FROZEN site set. `sites_annotated.jsonl` is the same rows with a
    derived `fn` field added by `annotate.py`; it is preferred when present and
    is asserted to be the same population, never a different one."""
    frozen = [json.loads(l) for l in open(os.path.join(LANE, "sites.jsonl"))]
    ann = os.path.join(LANE, "sites_annotated.jsonl")
    if not os.path.exists(ann):
        return frozen
    rows = [json.loads(l) for l in open(ann)]
    a = sorted((r["file"], r["line"], r["col"]) for r in rows)
    b = sorted((r["file"], r["line"], r["col"]) for r in frozen)
    assert a == b, "sites_annotated.jsonl is not the frozen population"
    return rows


def key(s):
    return (s["file"], s["line"], s["col"])


def load_hits(tag, stage):
    p = os.path.join(LOGS, "%s.%s.hits" % (tag, stage))
    if not os.path.exists(p):
        return None
    out = set()
    for line in open(p):
        line = line.strip()
        if not line:
            continue
        parts = line.rsplit(":", 2)
        if len(parts) != 3:
            continue
        try:
            out.add((parts[0], int(parts[1]), int(parts[2])))
        except ValueError:
            continue
    return out


def main():
    sites = load_sites()
    index = {key(s): s for s in sites}
    # `Location::column()` for `Block::refuse(..)` may report the PATH START
    # rather than the method ident. Both keys are admitted; which one rustc
    # actually emits is read off the log, never assumed.
    alias = {}
    for s in sites:
        if s.get("col_alt") not in (None, s["col"]):
            alias[(s["file"], s["line"], s["col_alt"])] = key(s)
    # `sitemap.json` (written by `sitemap.py` against the PATCHED tree) is the
    # authoritative translation: the probe patch shifts line numbers in
    # `func/body/mod.rs`, and `Location::column()` reports the PATH START of a
    # qualified call. It supersedes the `col_alt` alias above where both apply.
    smp = os.path.join(LANE, "sitemap.json")
    if os.path.exists(smp):
        for src, dst in json.load(open(smp)).items():
            a = src.rsplit(":", 2)
            b = dst.rsplit(":", 2)
            alias[(a[0], int(a[1]), int(a[2]))] = (b[0], int(b[1]), int(b[2]))
    print("population: %d parsed sites (%d distinct file:line:col)"
          % (len(sites), len(index)))
    for tag in sys.argv[1:]:
        print("\n================ tag %s ================" % tag)
        per_stage = {}
        for st in STAGES:
            h = load_hits(tag, st)
            if h is None:
                continue
            per_stage[st] = h
        # canonicalise every hit through the path-start alias BEFORE any set
        # algebra, so a `Block::refuse` hit lands on its own row.
        per_stage = {st: {alias.get(k, k) for k in h} for st, h in per_stage.items()}
        union = set()
        for h in per_stage.values():
            union |= h
        unknown_hits = union - set(index)
        inframe = union & set(index)
        print("stages present: %s" % ", ".join(per_stage))
        for st, h in per_stage.items():
            print("  %-6s distinct sites %5d   of which in frame %5d"
                  % (st, len(h), len(h & set(index))))
        print("UNION distinct sites %d | in frame %d / %d (%.1f%%) | out of frame %d"
              % (len(union), len(inframe), len(index),
                 100.0 * len(inframe) / max(1, len(index)), len(unknown_hits)))
        if unknown_hits:
            print("  OUT-OF-FRAME HITS (the enumeration and the probe disagree — a")
            print("  hit at a location the parser did not enumerate is an ENUMERATOR")
            print("  DEFECT and is printed in full, never dropped):")
            for k in sorted(unknown_hits)[:40]:
                print("    %s:%d:%d" % k)
            if len(unknown_hits) > 40:
                print("    ... %d more" % (len(unknown_hits) - 40))

        quiet = set(index) - inframe
        print("QUIET %d / %d (%.1f%%)" % (len(quiet), len(index),
                                          100.0 * len(quiet) / max(1, len(index))))

        # per FORM
        print("\n  by form (hit == REFUSED for ret_err/err_expr; hit == EVALUATED for ok_or):")
        forms = collections.Counter(s["form"] for s in sites)
        hitforms = collections.Counter(index[k]["form"] for k in inframe)
        for f, n in forms.most_common():
            print("    %-9s reached %5d / %5d  (%.1f%%)" % (f, hitforms[f], n, 100.0 * hitforms[f] / n))

        # per FILE
        print("\n  by file (sorted by sites, descending):")
        byfile = collections.Counter(s["file"] for s in sites)
        hitfile = collections.Counter(index[k]["file"] for k in inframe)
        zero = []
        for f, n in byfile.most_common():
            r = hitfile[f]
            if r == 0:
                zero.append((f, n))
            print("    %-62s %5d / %5d  (%.1f%%)"
                  % (f.replace("crates/c2-il/src/func/body/", ""), r, n, 100.0 * r / n))
        print("  files with ZERO reached sites: %d  %s"
              % (len(zero), ", ".join(f.split("/")[-1] for f, _ in zero)))

        # stage EXCLUSIVITY — the price of a witness (w-deadsites F2)
        print("\n  stage attribution — sites reached by EXACTLY ONE stage:")
        for st in per_stage:
            others = set()
            for st2, h2 in per_stage.items():
                if st2 != st and st2 != "gate":
                    others |= h2
            excl = (per_stage[st] & set(index)) - others
            print("    only %-6s : %5d" % (st, len(excl)))
        cheap = per_stage.get("suite", set()) & set(index)
        scanonly = (per_stage.get("scan", set()) & set(index)) - cheap
        print("    reached by the suite (CHEAP witness)          : %d" % len(cheap))
        print("    reached only outside the suite (dearer)       : %d" % (len(inframe) - len(cheap)))
        print("    reached by the 878-TU scan but not the suite  : %d" % len(scanonly))

        # ---- the split that sharpens QUIET without over-claiming -------------
        if any("fn" in s for s in sites):
            byfn = collections.defaultdict(list)
            for s in sites:
                byfn[(s["file"], s["fn"])].append(s)
            fn_entered = {
                k: any(key(s) in inframe for s in v) for k, v in byfn.items()
            }
            q_in_entered = sum(
                1 for s in sites
                if key(s) in quiet and fn_entered[(s["file"], s["fn"])]
            )
            q_in_cold = len(quiet) - q_in_entered
            cold_fns = [k for k, v in fn_entered.items() if not v]
            print("\n  QUIET, split by whether the ENCLOSING FUNCTION was entered at all:")
            print("    quiet in a function the corpus DEMONSTRABLY REACHES : %5d"
                  " (a statement about the BRANCH; a witness starts inside)" % q_in_entered)
            print("    quiet in a function NO site of which ever fired     : %5d"
                  " (a statement about DISPATCH)" % q_in_cold)
            print("    functions with zero sites reached: %d of %d"
                  % (len(cold_fns), len(byfn)))
            for k in sorted(cold_fns)[:40]:
                print("      %-30s %s  (%d sites)"
                      % (k[0].split("/")[-1], k[1], len(byfn[k])))
            if len(cold_fns) > 40:
                print("      ... %d more" % (len(cold_fns) - 40))

        out = os.path.join(LANE, "reached.%s.txt" % tag)
        with open(out, "w") as f:
            for k in sorted(inframe | unknown_hits):
                f.write("%s:%d:%d\n" % k)
        print("\n  wrote %s (%d lines: in-frame reached + any out-of-frame hit)"
              % (out, len(inframe | unknown_hits)))

        qout = os.path.join(LANE, "quiet.%s.txt" % tag)
        with open(qout, "w") as f:
            for k in sorted(quiet):
                f.write("%s:%d:%d\t%s\t%s\n" % (k[0], k[1], k[2],
                                                index[k]["form"], index[k]["ctx"]))
        print("  wrote %s (%d quiet sites)" % (qout, len(quiet)))


if __name__ == "__main__":
    main()

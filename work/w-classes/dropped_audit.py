#!/usr/bin/env python3
"""THE DROPPED-LANE AUDIT — on cases the class table was NOT measured on.

Every row of `scripts/mode_classes.txt` is an EXCLUSION: it names one
representative lane per equivalence class and the cross never grades the others.
The exclusion is derived from `mode_invariance.py`'s sample. This asks the
independent question the acceptance bar asks:

    on cases OUTSIDE that sample, do the dropped lanes produce the same
    verdict as the representative that stands in for them?

Not the same criterion `mode_invariance.py` uses — that one is stronger (IL bytes
+ reference obj bytes + `gy`) and is re-derived by `--check`. This one is weaker
and is the thing a reader actually cares about: **the verdict the cross would
have printed.** A row that survives both on held-out cases is supported by two
independent measurements over three populations.

    work/w-classes/dropped_audit.py <classes.txt> <outdir> [held-per-fragment] [jobs]

Writes `<outdir>/verdicts.tsv` and prints one line per fragment plus a corpus
verdict. Needs the toolchain; without one it prints SKIP and exits 0.
"""

import hashlib
import json
import os
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import sweep_gen           # noqa: E402
import mode_invariance     # noqa: E402


def read_groups(path):
    """`{fragment: ([[lane,...], ...], reps)}` — the equivalence classes.

    The lane list is field 2; the trailing `# a+b c+d` comment is the full
    partition, which is what makes the DROPPED lanes nameable at all. A row
    whose comment is missing is reported rather than assumed to be a singleton
    partition — an absent partition read as "nothing was dropped" is exactly the
    absence-as-success shape this file exists to avoid.
    """
    out = {}
    for line in open(path):
        if line.startswith("# measured-over-lanes:"):
            continue
        code, _, comment = line.partition("#")
        code = code.strip()
        if not code:
            continue
        parts = code.split()
        if len(parts) != 3:
            raise SystemExit("malformed row: %r" % line)
        groups = [g.split("+") for g in comment.split()] if comment.strip() else None
        out[parts[0]] = (groups, parts[1].split(","))
    return out


def held_out(frag, names, measured, k):
    """`k` cases of `frag` that the measurement did NOT sample, spread."""
    pool = [n for n in names if n not in measured]
    if len(pool) <= k:
        return pool
    sel = []
    n = len(pool)
    for i in range(k):
        lo, hi = (i * n) // k, ((i + 1) * n) // k
        d = int(hashlib.sha256(("heldout\x00%s\x00%d" % (frag, i)).encode())
                .hexdigest()[:8], 16)
        sel.append(pool[lo + d % max(1, hi - lo)])
    return sel


def main():
    classes = sys.argv[1]
    out = os.path.abspath(sys.argv[2])
    k = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    jobs = sys.argv[4] if len(sys.argv) > 4 else "16"
    per_fragment = int(os.environ.get("C2RS_AUDIT_PER_FRAGMENT", "64"))
    os.makedirs(out, exist_ok=True)
    cases_dir = os.path.join(out, "cases")
    os.makedirs(cases_dir, exist_ok=True)
    for n in os.listdir(cases_dir):
        if n.endswith(".cpp"):
            os.unlink(os.path.join(cases_dir, n))
    sweep_gen.write_cases(cases_dir, os.path.join(REPO, "scripts/sweep.d"), quiet=True)

    table = read_groups(classes)
    lanes = dict(mode_invariance.read_registry(os.path.join(REPO, "scripts/lanes.txt")))

    byfrag = {}
    for n in sorted(os.listdir(cases_dir)):
        if n.endswith(".cpp"):
            byfrag.setdefault(n.rsplit("-", 1)[0], []).append(n)

    picked = {}       # fragment -> held-out case names
    overlap = 0
    for frag, names in sorted(byfrag.items()):
        measured = set(mode_invariance.sample_cases(frag, names, per_fragment))
        sel = held_out(frag, names, measured, k)
        overlap += len(measured & set(sel))
        picked[frag] = sel

    allcases = [(f, c) for f in sorted(picked) for c in picked[f]]
    print("held-out cases: %d over %d fragments (overlap with the measured "
          "sample: %d)" % (len(allcases), len(picked), overlap))

    listp = os.path.join(out, "held.list")
    with open(listp, "w") as fh:
        for _f, c in allcases:
            fh.write("z:%s\n" % os.path.join(cases_dir, c).replace("/", "\\"))

    # One `c2rs gap` batch per lane over the WHOLE held-out list, then regroup.
    c2rs = os.environ.get("C2RS_BIN") or os.path.join(REPO, "target/release/c2rs")
    verdict = {}      # slug -> {case -> "class|reason"}
    for slug, flags in sorted(lanes.items()):
        fp = os.path.join(out, "%s.flags" % slug)
        with open(fp, "w") as fh:
            fh.write(" ".join(flags + ["/GS-", "/c"]) + "\n")
        jl = os.path.join(out, "%s.jsonl" % slug)
        subprocess.run([c2rs, "gap", "--list", listp, "--flags-file", fp,
                        "--jobs", jobs, "--jsonl", jl],
                       capture_output=True)
        m = {}
        if not os.path.exists(jl):
            raise SystemExit("lane %s produced no jsonl — a silent lane is not "
                             "a lane that agreed" % slug)
        for line in open(jl):
            r = json.loads(line)
            if r.get("record") == "provenance" or "src" not in r:
                continue
            m[os.path.basename(r["src"].replace("\\", "/"))] = "%s|%s" % (
                r.get("class"), r.get("reason"))
        if len(m) != len(allcases):
            raise SystemExit("lane %s graded %d of %d held-out cases; a missing "
                             "row is not an agreeing row" % (slug, len(m), len(allcases)))
        verdict[slug] = m
    if not verdict:
        print("SKIP: toolchain absent")
        return 0

    with open(os.path.join(out, "verdicts.tsv"), "w") as fh:
        fh.write("fragment\tcase\tlane\tverdict\n")
        for f, c in allcases:
            for slug in sorted(verdict):
                fh.write("%s\t%s\t%s\t%s\n" % (f, c, slug, verdict[slug][c]))

    print()
    print("%-27s %5s %6s %8s  %s" % ("FRAGMENT", "cases", "groups", "dropped",
                                     "held-out verdict check"))
    bad = 0
    checked_pairs = 0
    norow = 0
    for frag in sorted(picked):
        groups, reps = table.get(frag, (None, None))
        if groups is None:
            norow += 1
            print("%-27s %5d %6s %8s  NO ROW — graded at every lane, nothing dropped"
                  % (frag, len(picked[frag]), "-", "-"))
            continue
        ndrop = sum(len(g) - 1 for g in groups)
        disagree = []
        for g in groups:
            rep = sorted(g)[0]
            for other in sorted(g):
                if other == rep:
                    continue
                for c in picked[frag]:
                    checked_pairs += 1
                    if verdict[rep][c] != verdict[other][c]:
                        disagree.append((c, rep, other,
                                         verdict[rep][c], verdict[other][c]))
        if disagree:
            bad += len(disagree)
            print("%-27s %5d %6d %8d  *** %d DISAGREEMENTS" %
                  (frag, len(picked[frag]), len(groups), ndrop, len(disagree)))
            for d in disagree[:4]:
                print("      %s  %s=%s  vs  %s=%s" % (d[0], d[1], d[3], d[2], d[4]))
        else:
            print("%-27s %5d %6d %8d  all %d dropped-lane verdicts identical"
                  % (frag, len(picked[frag]), len(groups), ndrop,
                     ndrop * len(picked[frag])))
    print()
    print("held-out case-lane pairs compared: %d" % checked_pairs)
    print("fragments with NO ROW (nothing dropped): %d" % norow)
    print("DISAGREEMENTS: %d" % bad)
    return 1 if bad else 0


sys.exit(main())

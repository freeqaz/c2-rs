#!/usr/bin/env python3
"""PER-FRAGMENT graded accounting for one `mode_classes.txt`.

`scripts/mode_cross.sh` prints ONE `graded=` for the whole cross. That total is
the wrong instrument for judging a class-table change, because the change is
*supposed* to make it fall: the whole point of a row is that the cross stops
re-grading cells it has proved redundant. A total that falls is therefore
consistent with both the intended reduction and with a fragment that stopped
being graded at all — the second is `mode_classes.txt`'s one failure mode, and
the total cannot tell them apart.

So this splits the same run by fragment and reports, per fragment:

    cells      case-lane pairs the table assigns
    graded     of those, the ones the oracle ruled on
    ungraded   capture-fail (no reference obj; the oracle never ruled)
    cases-g    DISTINCT cases graded by AT LEAST ONE lane   <- the coverage claim
    mismatch   an ALARM at any value but 0

`cases-g` is the number that must not fall. `cells` is the number that is meant
to. Run it against the old table and the new one and diff the two.

    work/w-classes/frag_graded.py <classes.txt> <outdir> [jobs]

Uses `work/mode-cross/cases` — the SAME case paths `mode_cross.sh` uses, because
the capture-cache key contains the source path and a private directory means a
cold run. Do not run this while a cross is running.
"""

import json
import os
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import sweep_gen           # noqa: E402
import mode_invariance     # noqa: E402


def main():
    classes = os.path.abspath(sys.argv[1])
    out = os.path.abspath(sys.argv[2])
    jobs = sys.argv[3] if len(sys.argv) > 3 else "16"
    os.makedirs(out, exist_ok=True)

    cases = os.path.join(REPO, "work/mode-cross/cases")
    os.makedirs(cases, exist_ok=True)
    for n in os.listdir(cases):
        if n.endswith(".cpp"):
            os.unlink(os.path.join(cases, n))
    sweep_gen.write_cases(cases, os.path.join(REPO, "scripts/sweep.d"), quiet=True)

    lists = os.path.join(out, "lists")
    assign_log = os.path.join(out, "assign.log")
    with open(assign_log, "w") as fh:
        r = subprocess.run(
            ["python3", os.path.join(REPO, "scripts/mode_invariance.py"),
             "--assign", cases, "--assign-out", lists,
             "--classes", classes,
             "--registry", os.path.join(REPO, "scripts/lanes.txt")],
            stdout=fh, stderr=subprocess.STDOUT)
    if r.returncode != 0:
        sys.stdout.write(open(assign_log).read())
        raise SystemExit("assignment failed")
    for line in open(assign_log):
        if line.startswith("assigned "):
            print(line.rstrip())

    lanes = mode_invariance.read_registry(os.path.join(REPO, "scripts/lanes.txt"))
    c2rs = os.environ.get("C2RS_BIN") or os.path.join(REPO, "target/release/c2rs")

    cells = {}
    graded = {}
    ungraded = {}
    mismatch = {}
    cases_g = {}
    for slug, flags in lanes:
        lf = os.path.join(lists, "%s.list" % slug)
        if not os.path.exists(lf) or os.path.getsize(lf) == 0:
            continue
        fp = os.path.join(out, "%s.flags" % slug)
        with open(fp, "w") as fh:
            fh.write(" ".join(flags + ["/GS-", "/c"]) + "\n")
        jl = os.path.join(out, "%s.jsonl" % slug)
        subprocess.run([c2rs, "gap", "--list", lf, "--flags-file", fp,
                        "--jobs", jobs, "--jsonl", jl], capture_output=True)
        if not os.path.exists(jl):
            raise SystemExit("lane %s wrote no jsonl — a silent lane is not a "
                             "lane that graded" % slug)
        n = 0
        for line in open(jl):
            r = json.loads(line)
            if r.get("record") == "provenance" or "src" not in r:
                continue
            n += 1
            case = os.path.basename(r["src"].replace("\\", "/"))
            frag = case.rsplit("-", 1)[0]
            cells[frag] = cells.get(frag, 0) + 1
            cl = r.get("class")
            if cl == "capture-fail":
                ungraded[frag] = ungraded.get(frag, 0) + 1
            else:
                graded[frag] = graded.get(frag, 0) + 1
                cases_g.setdefault(frag, set()).add(case)
            if cl == "mismatch":
                mismatch[frag] = mismatch.get(frag, 0) + 1
        want = sum(1 for _ in open(lf))
        if n != want:
            raise SystemExit("lane %s: list has %d cells, jsonl has %d rows — a "
                             "short lane is not an agreeing lane" % (slug, want, n))
        print("  lane %-16s %6d cells" % (slug, n))

    frags = sorted(set(cells) | set(graded))
    print()
    print("%-27s %8s %8s %9s %8s %9s" %
          ("FRAGMENT", "cells", "graded", "ungraded", "cases-g", "mismatch"))
    tc = tg = tu = tm = tcg = 0
    for f in frags:
        print("%-27s %8d %8d %9d %8d %9d" %
              (f, cells.get(f, 0), graded.get(f, 0), ungraded.get(f, 0),
               len(cases_g.get(f, ())), mismatch.get(f, 0)))
        tc += cells.get(f, 0); tg += graded.get(f, 0)
        tu += ungraded.get(f, 0); tm += mismatch.get(f, 0)
        tcg += len(cases_g.get(f, ()))
    print("%-27s %8d %8d %9d %8d %9d" % ("TOTAL", tc, tg, tu, tcg, tm))
    return 0


sys.exit(main())

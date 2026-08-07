#!/usr/bin/env python3
"""run.py — grade every frozen GRID R cell, ONE DIRECTORY PER CELL (#1045).

Two independent instruments plus the reference disassembly, all at the
WORKLOAD's own `/GR /O1 /Oi /EHsc` (board #1112 — at the harness default `/Ox`
a refusal on this checklist reads as PAID when it is genuinely unpaid):

  * `c2rs gap`     the whole-TU differential against real `c2.dll` under wibo,
                   `TimeDateStamp` zeroed.  THE SOLE JUDGE.
  * `c2rs census`  the class verdict and the FIRST-REFUSAL KEY.
  * `gt_dump.py`   real c2's emitted words, which is where the FINAL store
                   order and the observed `mr` slot are read from — the scorer
                   never asks the port's own `order::store_order`.

    python3 work/w-mrslot/run.py <tag> [--bin PATH] [--jobs N] [--ref] [cell...]

`--ref` also (re)builds the reference obj + disassembly; it is tag-independent
(it is c2's answer, not the port's) so it is done once and not per tag.
"""

import argparse
import concurrent.futures as cf
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
GRID = os.path.join(HERE, "grid")
FLAGS = os.path.join(ROOT, "work/dc3-workload/flags.txt")


def cell_dir(c):
    return os.path.join(GRID, c)


def run_one(c, tag, binpath, want_ref):
    d = cell_dir(c)
    # RELATIVE path from the repo root.  An ABSOLUTE one reaches cl.exe under
    # wibo untranslated ("D8003 missing source filename"), the capture fails,
    # and a grep for the verdict prints nothing — which reads exactly like a
    # clean run.  Hence the explicit NO-VERDICT lines below.
    rel = "work/w-mrslot/grid/%s/%s.cpp" % (c, c)
    src = os.path.join(d, c + ".cpp")
    if not os.path.exists(src):
        return c, "NO SOURCE", "", ""

    with open(os.path.join(d, "census.%s.txt" % tag), "w") as f:
        subprocess.run([binpath, "census", rel, "--flags-file", FLAGS],
                       cwd=ROOT, stdout=f, stderr=subprocess.STDOUT)
    lst = os.path.join(d, "list.txt")
    with open(lst, "w") as f:
        f.write(rel + "\n")
    with open(os.path.join(d, "gap.%s.txt" % tag), "w") as f:
        subprocess.run([binpath, "gap", "--list", lst, "--flags-file", FLAGS,
                        "--cwd", ROOT, "--jobs", "1"],
                       cwd=ROOT, stdout=f, stderr=subprocess.STDOUT)

    if want_ref:
        subprocess.run(["sh", os.path.join(ROOT, "work/w-heap/refobj_local.sh"),
                        rel, os.path.join(d, "ref.obj")],
                       cwd=ROOT, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL)
        if os.path.exists(os.path.join(d, "ref.obj")):
            with open(os.path.join(d, "dis.txt"), "w") as f:
                subprocess.run([sys.executable,
                                os.path.join(ROOT, "scripts/gt_dump.py"),
                                os.path.join(d, "ref.obj")],
                               cwd=ROOT, stdout=f, stderr=subprocess.STDOUT)

    cen = open(os.path.join(d, "census.%s.txt" % tag)).read()
    gap = open(os.path.join(d, "gap.%s.txt" % tag)).read()
    verdict = next((l.strip() for l in cen.splitlines()
                    if "functions in class" in l), "NO-VERDICT (census)")
    key = next((l.strip() for l in cen.splitlines() if " GAP " in l), "")
    diff = next((l.strip() for l in gap.splitlines()
                 if l.startswith("  [1/1]")), "NO-DIFFERENTIAL")
    return c, verdict, key, diff


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tag")
    ap.add_argument("--bin", default=os.path.join(ROOT, "target/release/c2rs"))
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--ref", action="store_true")
    ap.add_argument("cells", nargs="*")
    a = ap.parse_args()

    cells = a.cells or sorted(os.listdir(GRID))
    nograde = 0
    with cf.ThreadPoolExecutor(max_workers=a.jobs) as ex:
        futs = {ex.submit(run_one, c, a.tag, a.bin, a.ref): c for c in cells}
        out = {}
        for f in cf.as_completed(futs):
            c, v, k, dfr = f.result()
            out[c] = (v, k, dfr)
    for c in cells:
        v, k, dfr = out[c]
        print("== %s\n   %s\n   %s\n   %s" % (c, v, k or "(no GAP key)", dfr))
        if "NO-VERDICT" in v or "NO-DIFFERENTIAL" in dfr:
            nograde += 1
    print("\n%d cells, %d ungraded" % (len(cells), nograde))
    if nograde:
        sys.exit(1)


if __name__ == "__main__":
    main()

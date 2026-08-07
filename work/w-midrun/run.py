#!/usr/bin/env python3
"""run.py — grade every frozen GRID M cell, ONE DIRECTORY PER CELL (#1045).

Everything at the WORKLOAD's own `/GR /O1 /Oi /EHsc` (board #1112 — at the
harness default `/Ox` a refusal on this checklist reads as PAID when it is
genuinely unpaid).

  * `c2rs gap`     the whole-TU differential against real `c2.dll` under wibo,
                   `TimeDateStamp` zeroed.  **THE SOLE JUDGE.**
  * `c2rs census`  the class verdict, the FIRST-REFUSAL KEY, and — the reason
                   this lane reads it — the `census/gate DISAGREEMENT` line.
  * `refobj`       real c2's own obj + disassembly, so the emitted words the
                   scorer reads are c2's and never the port's own model.

    python3 work/w-midrun/run.py <tag> [--bin PATH] [--jobs N] [--ref] [cell...]

`--ref` (re)builds the reference obj + disassembly; it is tag-independent (it is
c2's answer, not the port's) so it is done once and not per tag.
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


def run_one(c, tag, binpath, want_ref):
    d = os.path.join(GRID, c)
    # RELATIVE path from the repo root.  An ABSOLUTE one reaches cl.exe under
    # wibo untranslated ("D8003 missing source filename"), the capture fails,
    # and a grep for the verdict prints nothing — which reads exactly like a
    # clean run.  Hence the explicit NO-VERDICT lines below.
    rel = "work/w-midrun/grid/%s/%s.cpp" % (c, c)
    if not os.path.exists(os.path.join(d, c + ".cpp")):
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
        # `refobj_local.sh`'s second argument is the OBJ PATH, not a directory.
        obj = os.path.join(d, "ref.obj")
        subprocess.run(["sh", os.path.join(ROOT, "work/w-heap/refobj_local.sh"),
                        rel, obj], cwd=ROOT,
                       stdout=open(os.path.join(d, "refobj.txt"), "w"),
                       stderr=subprocess.STDOUT)
        if os.path.exists(obj):
            with open(os.path.join(d, "dis.txt"), "w") as f:
                subprocess.run([sys.executable,
                                os.path.join(ROOT, "scripts/gt_dump.py"),
                                obj, "--text-only"],
                               cwd=ROOT, stdout=f, stderr=subprocess.STDOUT)

    verdict, key, dis = "NO-VERDICT", "NO-KEY", ""
    for line in open(os.path.join(d, "gap.%s.txt" % tag)):
        s = line.strip()
        for k in ("match", "mismatch", "codegen-gap", "vocab-gap",
                  "capture-fail", "port-error"):
            if s.startswith("gap-metric %s 1" % k):
                verdict = k
    cen = open(os.path.join(d, "census.%s.txt" % tag)).read()
    for line in cen.splitlines():
        if "GAP " in line:
            key = line.split("GAP ", 1)[1].split()[0]
            break
        if " ok  " in line:
            key = "ok:" + line.split(" ok  ", 1)[1].split()[0]
            break
    dg = "disagree=%d" % (1 if "census/gate DISAGREEMENT" in cen else 0)
    return c, verdict, key, dg


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tag")
    ap.add_argument("--bin", default=os.path.join(ROOT, "target/release/c2rs"))
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--ref", action="store_true")
    ap.add_argument("cells", nargs="*")
    a = ap.parse_args()

    # Directories only. `cl.exe` will drop a stray `.obj` beside them if a
    # `/Fo` path is ever mis-spelled, and a stray file read as a cell prints
    # `NO SOURCE` — which inflates the row count and reads like a grid twice
    # its size (it did, once, on this lane's first `--ref` run).
    cells = a.cells or sorted(c for c in os.listdir(GRID)
                              if os.path.isdir(os.path.join(GRID, c)))
    rows = []
    with cf.ThreadPoolExecutor(max_workers=a.jobs) as ex:
        futs = [ex.submit(run_one, c, a.tag, a.bin, a.ref) for c in cells]
        for f in cf.as_completed(futs):
            rows.append(f.result())
    rows.sort()
    out = os.path.join(HERE, "out", "rows.%s.txt" % a.tag)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w") as f:
        for c, v, k, d in rows:
            f.write("%-18s %-12s %-8s %s\n" % (c, v, d, k))
    tot = {}
    for _c, v, _k, _d in rows:
        tot[v] = tot.get(v, 0) + 1
    print("GRID M [%s]  %s" % (a.tag, "  ".join(
        "%s=%d" % (k, tot[k]) for k in sorted(tot))))
    print("  -> %s" % out)


if __name__ == "__main__":
    main()

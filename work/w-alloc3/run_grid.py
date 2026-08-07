#!/usr/bin/env python3
"""run_grid.py — compile a w-alloc3 grid with real `cl.exe` and grade RULE BIND.

Lane w-alloc3 measurement tooling. **Read-only with respect to `crates/`.**

    run_grid.py <grid.json> <out.tsv> [--jobs N] [--frozen <pred.tsv>]

Real `c2.dll` under wibo is the sole judge (`CLAUDE.md`). This grades RULE BIND
against c2's own COMDAT for the caller, with the prediction computed from c2's
own COMDAT for the CALLEE in the same obj — the SPLICE-0 methodology of
`docs/rungs/2026-08-08-w-seq.md` §4, which needs no port emitter.

`--frozen` reads a committed prediction column instead of computing one, and
re-checks every source's `sha256` first. That is the holdout protocol of
`PREREG.md` §4 and `w-refbind`/`w-ilx` are its precedent: the grader must not
be able to recompute a prediction after seeing an answer.

EVERY CELL COMPILES IN ITS OWN DIRECTORY — board **#1045**.

THE PER-CELL POSITIVE CONTROL
-----------------------------
Every cell carries `void anchor(){ ext_anchor(); }`, whose callee the TU does
not define, and the grader asserts that COMDAT keeps **exactly one** REL24.
Without it, "the port and c2 agree" cannot be told from "the reader found
nothing" — `docs/STATUS.md` trap 5.
"""

import concurrent.futures
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, HERE)

import gt_dump  # noqa: E402  — the repo's one COFF reader, not a second copy
import bind  # noqa: E402


def compile_cell(c):
    d = os.path.dirname(os.path.join(ROOT, c["path"]))
    obj = os.path.join(d, c["name"] + ".obj")
    r = subprocess.run(
        [
            os.path.join(ROOT, "target/release/c2rs"),
            "compile",
            c["path"],
            "--keep-obj",
            os.path.relpath(obj, ROOT),
            "--flags-file",
            "work/w-alloc3/flags.txt",
            "--cwd",
            ROOT,
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if r.returncode != 0 or not os.path.exists(obj):
        return None, "compile-failed: " + (r.stderr or r.stdout).strip()[:200]
    return obj, None


def text_comdats(obj_path):
    """{symbol name: (words, nrelocs)} for every `.text` COMDAT."""
    o = gt_dump.Obj(open(obj_path, "rb").read())
    bysec = {}
    for s in o.symbols:
        if s["sec"] > 0 and s["value"] == 0 and s["name"].startswith("?"):
            bysec.setdefault(s["sec"], s["name"])
    out = {}
    for sec in o.sections:
        if not sec["name"].startswith(".text"):
            continue
        nm = bysec.get(sec["idx"])
        if nm is None:
            continue
        raw = o.raw(sec)
        words = [int.from_bytes(raw[i:i + 4], "big") for i in range(0, len(raw), 4)]
        out[nm] = (words, sec["nrel"])
    return out


def hexw(ws):
    return " ".join("%08x" % w for w in ws)


def grade(c, frozen):
    """-> dict(name, verdict, domain, detail, pred, got)"""
    row = {
        "name": c["name"],
        "axis": c["axis"],
        "callee": c["callee"],
        "n": c["n_caller_formals"],
        "mode": c["mode"],
        "verdict": "",
        "why": "",
        "pred": "",
        "got": "",
        "gbody": "",
        "gwords": 0,
    }
    src = open(os.path.join(ROOT, c["path"]), "rb").read()
    if hashlib.sha256(src).hexdigest() != c["sha256"]:
        row["verdict"] = "SOURCE-MOVED"
        return row
    obj, err = compile_cell(c)
    if err:
        row["verdict"] = "COMPILE-FAILED"
        row["why"] = err
        return row
    cd = text_comdats(obj)
    gname = next((k for k in cd if k.startswith("?g@@")), None)
    fname = next((k for k in cd if k.startswith("?f@@")), None)
    aname = next((k for k in cd if k.startswith("?anchor@@")), None)
    if aname is None or cd[aname][1] != 1:
        row["verdict"] = "ANCHOR-BROKEN"
        row["why"] = "anchor relocs=%s" % (cd.get(aname, (None, None))[1],)
        return row
    if gname is None or fname is None:
        row["verdict"] = "OUT-OF-DOMAIN"
        row["why"] = "D3-no-comdat"
        return row
    gw, gnrel = cd[gname]
    fw, fnrel = cd[fname]
    row["gbody"] = hexw(gw)
    row["gwords"] = len(gw) - 1
    row["got"] = hexw(fw)
    if gnrel != 0:
        row["verdict"] = "OUT-OF-DOMAIN"
        row["why"] = "D3-callee-has-relocations"
        return row
    if fnrel != 0:
        row["verdict"] = "OUT-OF-DOMAIN"
        row["why"] = "D8-c2-kept-the-call"
        return row
    if c["domain"] != "in":
        row["verdict"] = "OUT-OF-DOMAIN"
        row["why"] = c["domain"][4:] + " (registered)"
        return row

    n = c["n_caller_formals"]
    caller_hi = 3 + n - 1 if n else 2
    beta_regs = [3 + p for p in c["beta"]]
    if frozen is not None:
        p = frozen.get(c["name"])
        if p is None:
            row["verdict"] = "FROZEN-MISSING"
            return row
        if p["why"]:
            row["verdict"] = "OUT-OF-DOMAIN"
            row["why"] = p["why"] + " (frozen)"
            return row
        pred = [int(x, 16) for x in p["pred"].split()]
    else:
        try:
            pred = bind.predict(
                gw,
                len(c["beta"]),
                beta_regs,
                c["mode"],
                c["k_scaled"],
                caller_hi,
            )
        except bind.Refused as e:
            row["verdict"] = "OUT-OF-DOMAIN"
            row["why"] = e.why
            return row
    row["pred"] = hexw(pred)
    row["verdict"] = "HIT" if pred == fw else "MISS"
    return row


COLS = ["name", "axis", "callee", "n", "mode", "gwords", "verdict", "why",
        "pred", "got", "gbody"]


def main():
    gridjson = sys.argv[1]
    outtsv = sys.argv[2]
    jobs = 8
    frozen = None
    a = sys.argv[3:]
    while a:
        if a[0] == "--jobs":
            jobs = int(a[1])
            a = a[2:]
        elif a[0] == "--frozen":
            frozen = {}
            for ln in open(a[1]):
                if ln.startswith("#") or not ln.strip():
                    continue
                f = ln.rstrip("\n").split("\t")
                frozen[f[0]] = {"sha256": f[1], "why": f[2], "pred": f[3]}
            a = a[2:]
        else:
            raise SystemExit("unknown arg %s" % a[0])

    cells = json.load(open(gridjson))
    if frozen is not None:
        bad = [c["name"] for c in cells
               if c["name"] in frozen and frozen[c["name"]]["sha256"] != c["sha256"]]
        if bad:
            raise SystemExit("FROZEN SHA256 MISMATCH: %s" % bad)
        print("frozen manifest: %d cells, %d sha256 re-checked OK"
              % (len(frozen), len(cells)))

    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        rows = list(ex.map(lambda c: grade(c, frozen), cells))

    with open(outtsv, "w") as fh:
        fh.write("\t".join(COLS) + "\n")
        for r in rows:
            fh.write("\t".join(str(r[c]) for c in COLS) + "\n")

    tally = {}
    for r in rows:
        tally[r["verdict"]] = tally.get(r["verdict"], 0) + 1
    print("=== %s ===" % gridjson)
    print("  cells %d" % len(rows))
    for k in sorted(tally):
        print("  %-14s %d" % (k, tally[k]))
    print()
    print("  OUT-OF-DOMAIN by clause:")
    ood = {}
    for r in rows:
        if r["verdict"] == "OUT-OF-DOMAIN":
            ood[r["why"]] = ood.get(r["why"], 0) + 1
    for k, v in sorted(ood.items(), key=lambda kv: -kv[1]):
        print("    %-34s %d" % (k, v))
    misses = [r for r in rows if r["verdict"] == "MISS"]
    if misses:
        print()
        print("  MISSES:")
        for r in misses:
            print("    %-22s %-10s g=[%s]" % (r["name"], r["axis"], r["gbody"]))
            print("        pred %s" % r["pred"])
            print("        c2   %s" % r["got"])


if __name__ == "__main__":
    # **A MODULE-LEVEL `main()` HERE GRADED THE HOLDOUT BY ACCIDENT.**
    # `freeze_h.py` imports this file for `compile_cell` / `text_comdats`, and
    # at the moment it did so this line read `main()` with no guard — so the
    # import ran the grader over `sys.argv`, which was GRID-H's, and printed
    # the holdout's verdict before the frozen column had been written. The
    # verdict is unaffected (it came from the program frozen at `245945c2`,
    # with no refinement between the freeze and the grade) and it is reported
    # in the rung as what happened. The guard is here so the next lane that
    # imports a grader does not repeat it.
    main()

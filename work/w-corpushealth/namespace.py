#!/usr/bin/env python3
"""w-corpushealth — the NAME-SPACE bound.

The per-TU Frechet bound in join.py cannot see an unfinished symbol that
objdiff attributes to unit j but that c2 emits into TU i's obj (every STLport
template instantiation is of that kind). This closes that hole: it reads the
`.text` COMDAT symbol names out of each TU's own reference obj — produced by
real c2.dll under wibo at the workload's own flags — and intersects them with
the decomp's own per-symbol verdict.

Buckets over the emitted-body instances (sum over TUs of |S_i|):
  UNFINISHED  the name is in an authorable unit at match_percent_normalized<100
  FINISHED    the name is in an authorable unit at normalized == 100
  VENDOR      the name is in a NON-authorable unit (xdk / bink: no source, or
              source present but linked from the original lib)
  ABSENT      the name is in no objdiff unit at all — the shipped image does
              not contain this body under this name (COMDAT loser / dead code)

Only UNFINISHED is eligible to be a corpus artifact under the hypothesis.
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def _dc3():
    """`C2RS_DC3`, else the nearest ancestor holding a `dc3-decomp` sibling.

    A lane runs from `.claude/worktrees/<lane>`, so a fixed `../../..` resolves
    to the worktrees directory and not to the milohax root; `scripts/status.sh`
    documents `C2RS_DC3` for exactly this. No absolute path lives in this file.
    """
    if os.environ.get("C2RS_DC3"):
        return os.environ["C2RS_DC3"]
    d = HERE
    while d != os.path.dirname(d):
        c = os.path.join(d, "dc3-decomp")
        if os.path.isdir(c):
            return c
        d = os.path.dirname(d)
    raise SystemExit("dc3-decomp not found; set C2RS_DC3")


DC3 = _dc3()

rep = json.load(open(os.path.join(DC3, "build/373307D9/report.json")))
UNFIN, FIN, VENDOR = set(), set(), set()
for u in rep["units"]:
    auth = bool((u.get("metadata") or {}).get("complete"))
    for f in u.get("functions", []):
        n = f["name"]
        if not auth:
            VENDOR.add(n)
        elif (f.get("match_percent_normalized") or 0) < 100:
            UNFIN.add(n)
        else:
            FIN.add(n)
# a name graded authorable wins over a vendor listing of the same name
VENDOR -= (UNFIN | FIN)
UNFIN -= FIN  # a name matched in one unit is matched source

scan = [json.loads(l) for l in open(os.path.join(HERE, "base.jsonl"))]
prov = [r for r in scan if r.get("record") == "provenance"][0]
tus = {r["src"]: r for r in scan if "class" in r}

rows = []
bad = []
for src, r in tus.items():
    slug = src.replace("/", "_")
    p = os.path.join(HERE, "syms", slug + ".txt")
    lines = open(p).read().splitlines()
    if not lines or lines[0].startswith("COMPILE-FAIL"):
        rows.append(dict(src=src, cls=r["class"], ok=False))
        continue
    if lines[0].startswith("BAD"):
        bad.append((src, lines[0]))
    names = [l for l in lines[1:]]
    S = set(names)
    e = r["emit"]
    rows.append(dict(
        src=src, cls=r["class"], ok=True,
        R=e.get("fnbyte-refused-parse", 0), E=e.get("fnbyte-denominator", 0),
        X=e.get("fnbyte-exact", 0), D=e.get("fnbyte-differs", 0),
        S=len(S),
        unfin=len(S & UNFIN), fin=len(S & FIN),
        vendor=len(S & VENDOR), absent=len(S - UNFIN - FIN - VENDOR)))

ok = [r for r in rows if r["ok"]]
T = lambda k: sum(r[k] for r in ok)
R = T("R")


def pc(a, b):
    return f"{100.0*a/b:6.2f}%" if b else "   n/a"


print("=" * 78)
print("W-CORPUSHEALTH — NAME-SPACE bound: emitted bodies vs the decomp's verdict")
print("=" * 78)
print(f"objs read {len(ok)} of {len(rows)} ({len(rows)-len(ok)} COMPILE-FAIL) · "
      f"STRUCTURALLY MALFORMED: {len(bad)}")
for s, w in bad:
    print("   BAD", s, w)
print(f"workload head {prov['workload_head'][:9]}  cache {prov['cache_hits']}/"
      f"{prov['cache_misses']}  binary {prov['binary_sha'][:12]}")
print()
print(f"decomp name space: UNFINISHED {len(UNFIN)}  FINISHED {len(FIN)}  "
      f"VENDOR {len(VENDOR)}")
print()
print(f"EMITTED BODY INSTANCES over {len(ok)} objs: sum|S_i| = {T('S')}")
print(f"  (c2rs fnbyte-denominator = {T('E')}, fnbyte-refused-parse = {R})")
print(f"   UNFINISHED {T('unfin'):7d}  {pc(T('unfin'), T('S'))}")
print(f"   FINISHED   {T('fin'):7d}  {pc(T('fin'), T('S'))}")
print(f"   VENDOR     {T('vendor'):7d}  {pc(T('vendor'), T('S'))}")
print(f"   ABSENT     {T('absent'):7d}  {pc(T('absent'), T('S'))}"
      "   (in NO objdiff unit — not in the shipped image under this name)")
print()
ub = sum(min(r["R"], r["unfin"]) for r in ok)
lb = sum(max(0, r["R"] + r["unfin"] - r["S"]) for r in ok)
print("THE ANSWER — refusals attributable to source the decomp has NOT matched")
print(f"   ABSOLUTE CEILING  sum |S_i n UNFINISHED| = {T('unfin')}"
      f"  -> {pc(T('unfin'), R)} of the {R} refusals")
print(f"   tight UPPER       sum min(R_i, unfin_i)  = {ub}  -> {pc(ub, R)}")
print(f"   LOWER             sum max(0,R+unfin-|S|) = {lb}  -> {pc(lb, R)}")
print()
print("CONTROL — the same bound taken against FINISHED source (must be large,")
print("          or the ruler is not measuring what it claims)")
ubf = sum(min(r["R"], r["fin"]) for r in ok)
print(f"   sum |S_i n FINISHED| = {T('fin')}  -> {pc(T('fin'), R)} of refusals; "
      f"sum min(R_i,fin_i) = {ubf} -> {pc(ubf, R)}")
print()
byc = {}
for r in ok:
    byc.setdefault(r["cls"], []).append(r)
for c, rs in sorted(byc.items()):
    print(f"BY CLASS {c:12s} TUs {len(rs):4d}  R {sum(x['R'] for x in rs):7d}  "
          f"|S| {sum(x['S'] for x in rs):7d}  unfin {sum(x['unfin'] for x in rs):6d}  "
          f"fin {sum(x['fin'] for x in rs):7d}  absent {sum(x['absent'] for x in rs):7d}")
print()
print("TOP 15 TUs BY |S_i n UNFINISHED| (where the hypothesis has the most room)")
for r in sorted(ok, key=lambda r: -r["unfin"])[:15]:
    print(f"   unfin={r['unfin']:4d}/|S|={r['S']:5d}  R={r['R']:6d}  "
          f"min={min(r['R'],r['unfin']):4d}  {r['cls']:10s} {r['src']}")

json.dump(dict(provenance=prov, rows=rows), open(os.path.join(HERE, "namespace.json"), "w"))

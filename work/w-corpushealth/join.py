#!/usr/bin/env python3
"""w-corpushealth — join the c2-rs 878-TU scan to dc3-decomp's own progress signal.

THE ENUMERATION RULE, executable. Read-only on ../dc3-decomp.

Inputs
  work/w-corpushealth/base.jsonl        one row per TU from `c2rs gap --jsonl`
  $C2RS_DC3/build/373307D9/report.json  objdiff-cli 4.2.3, functionRelocDiffs=name_check
  $C2RS_DC3/decomp.db                   the decomp's own bookkeeping (verdicts, stubs)

Join key: report.json `units[].metadata.source_path`  ==  the scan's `src`.
Both are repo-relative POSIX paths rooted at the dc3-decomp tree, which is what
makes this join exact rather than heuristic. No fuzzy matching is done; a TU
that does not join is COUNTED AND NAMED, never dropped.

FINISHED, per the decomp's own canonical ruler (docs/PROGRESS_METRICS.md):
  a function is finished iff match_percent_normalized == 100.
"""
import json
import os
import sqlite3
import sys
from collections import Counter

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

scan = [json.loads(l) for l in open(os.path.join(HERE, "base.jsonl"))]
prov = [r for r in scan if r.get("record") == "provenance"][0]
tus = [r for r in scan if "class" in r]
assert len(tus) == 878, len(tus)

rep = json.load(open(os.path.join(DC3, "build/373307D9/report.json")))
units = rep["units"]

# ---- unit index, by source_path -------------------------------------------
by_src = {}
dup = Counter()
for u in units:
    sp = (u.get("metadata") or {}).get("source_path")
    if not sp:
        continue
    dup[sp] += 1
    by_src.setdefault(sp, []).append(u)

# ---- decomp.db -------------------------------------------------------------
db = sqlite3.connect(f"file:{os.path.join(DC3,'decomp.db')}?mode=ro", uri=True)
db_unit = {}
for unit, n, stubs, excl, comp, atl, nov in db.execute(
    """SELECT unit, COUNT(*), SUM(is_stub=1), SUM(excluded=1),
              SUM(verdict='COMPLETE'), SUM(verdict='AT_LIMIT'),
              SUM(verdict IS NULL)
       FROM functions GROUP BY unit"""
):
    db_unit[unit] = dict(n=n, stubs=stubs or 0, excluded=excl or 0,
                         complete=comp or 0, at_limit=atl or 0, no_verdict=nov or 0)

rows = []
for r in tus:
    src = r["src"]
    e = r["emit"]
    R = e.get("fnbyte-refused-parse", 0)
    Rc = e.get("fnbyte-refused-codegen", 0)
    E = e.get("fnbyte-denominator", 0)
    X = e.get("fnbyte-exact", 0)
    D = e.get("fnbyte-differs", 0)
    us = by_src.get(src, [])
    if us:
        fns = [f for u in us for f in u.get("functions", [])]
        U = sum(1 for f in fns if (f.get("match_percent_normalized") or 0) < 100)
        F = len(fns)
        ubytes = sum(int(f["size"]) for f in fns
                     if (f.get("match_percent_normalized") or 0) < 100)
        tbytes = sum(int(f["size"]) for f in fns)
        complete = all((u.get("metadata") or {}).get("complete") for u in us)
        autogen = any((u.get("metadata") or {}).get("auto_generated") for u in us)
        cats = sorted({c for u in us
                       for c in ((u.get("metadata") or {}).get("progress_categories") or [])})
    else:
        U = F = ubytes = tbytes = 0
        complete = None
        autogen = None
        cats = []
    d = db_unit.get(src, {})
    rows.append(dict(src=src, cls=r["class"], R=R, Rc=Rc, E=E, X=X, D=D,
                     U=U, F=F, ubytes=ubytes, tbytes=tbytes,
                     joined=bool(us), n_units=len(us), complete=complete,
                     autogen=autogen, cats=cats,
                     db_n=d.get("n", 0), db_stubs=d.get("stubs", 0),
                     db_excl=d.get("excluded", 0), db_complete=d.get("complete", 0),
                     db_atl=d.get("at_limit", 0), db_nov=d.get("no_verdict", 0),
                     in_db=src in db_unit))

json.dump(dict(provenance=prov, rows=rows),
          open(os.path.join(HERE, "joined.json"), "w"), indent=0)


def pct(a, b):
    return f"{100.0*a/b:.2f}%" if b else "n/a"


tot = lambda k: sum(r[k] for r in rows)
graded = [r for r in rows if r["cls"] != "capture-fail"]
J = [r for r in rows if r["joined"]]
NJ = [r for r in rows if not r["joined"]]

print("=" * 78)
print("W-CORPUSHEALTH — corpus-immaturity vs refusal, joined per TU")
print("=" * 78)
print(f"c2rs head      {prov['c2rs_head'][:8]}  binary {prov['binary_sha'][:12]}")
print(f"workload head  {prov['workload_head'][:9]}  dirty={prov['workload_dirty']}")
print(f"cache          {prov['cache_hits']} hit / {prov['cache_misses']} miss "
      f"(context {prov['cache_context'][:8]})")
print(f"objdiff        {rep['provenance']['tool_version']} "
      f"commit {rep['provenance']['tool_commit']}  "
      f"{[c for c in rep['provenance']['diff_config'] if c.startswith('functionRelocDiffs')]}")
print()
print(f"H1 JOIN        {len(J)}/878 TUs resolve to a report.json unit  ({pct(len(J),878)})")
print(f"               {len(NJ)} do not; duplicate source_paths: "
      f"{sum(1 for s,c in dup.items() if c>1)}")
print(f"               refusal mass in joined TUs: {sum(r['R'] for r in J)} "
      f"of {tot('R')}  ({pct(sum(r['R'] for r in J), tot('R'))})")
print()
print(f"TOTALS         fnbyte-refused-parse {tot('R')}   -codegen {tot('Rc')}   "
      f"exact {tot('X')}   differs {tot('D')}   denominator {tot('E')}")
print(f"               objdiff fns in joined units {sum(r['F'] for r in J)}, "
      f"unfinished (norm<100) {sum(r['U'] for r in J)}")
print()

# ---- H2: coarse, unit-level -------------------------------------------------
# **PREREG CORRECTION, recorded rather than silently applied.** The prereg's H2
# reads `metadata.complete`. That field is NOT a match measurement: it is
# objdiff's "this unit is built from source in the final link" flag, and it is
# true for exactly the 968 units that have a `source_path` (keygen_xbox carries
# `complete: true` at matched_functions 16/20). The decomp's own headline
# "Complete units (all fns norm==100) 416/967" uses the OTHER definition.
# H2 is therefore scored on **U_i == 0** — every function in the unit at
# normalized 100 — and the `metadata.complete` split is printed beside it as
# the authorable/vendor axis it actually is.
inc = [r for r in J if r["complete"] is False]
com = [r for r in J if r["complete"] is True]
fin = [r for r in J if r["U"] == 0 and r["F"] > 0]
unf = [r for r in J if r["U"] > 0]
print("H2 UNIT-LEVEL (coarse upper bound — 'this TU's unit is not finished')")
print(f"   [scored] unit FINISHED (all fns norm==100)  {len(fin)} TUs, "
      f"refusal mass {sum(r['R'] for r in fin)}")
print(f"   [scored] unit NOT finished                  {len(unf)} TUs, "
      f"refusal mass {sum(r['R'] for r in unf)}"
      f"  -> {pct(sum(r['R'] for r in unf), tot('R'))} of all refusals")
print(f"   [axis]   metadata.complete true (authorable, built from source): "
      f"{len(com)} TUs, refusal mass {sum(r['R'] for r in com)}")
print(f"   [axis]   metadata.complete false (vendor/xdk, linked from lib):  "
      f"{len(inc)} TUs, refusal mass {sum(r['R'] for r in inc)}")
print()

# ---- H3: tight Frechet bounds ----------------------------------------------
ub = sum(min(r["R"], r["U"]) for r in J)
lb = sum(max(0, r["R"] + r["U"] - max(r["E"], r["F"])) for r in J)
print("H3 FUNCTION-LEVEL (Frechet bounds on refused AND unfinished, per TU)")
print(f"   tight UPPER  sum min(R_i,U_i) = {ub}  -> {pct(ub, tot('R'))} of refusals")
print(f"   LOWER        sum max(0,R+U-max(E,F)) = {lb}  -> {pct(lb, tot('R'))}")
print(f"   (denominator: fnbyte-refused-parse {tot('R')})")
print()

# ---- H4: vocab-gap ----------------------------------------------------------
vg = [r for r in rows if r["cls"] == "vocab-gap"]
vgj = [r for r in vg if r["joined"]]
vgc = [r for r in vgj if r["complete"] is True]
vgi = [r for r in vgj if r["complete"] is False]
vg0 = [r for r in vgj if r["U"] == 0]
print("H4 VOCAB-GAP TUs (844)")
print(f"   joined {len(vgj)}; unit COMPLETE {len(vgc)}; unit INCOMPLETE {len(vgi)}; "
      f"unjoined {len(vg)-len(vgj)}")
print(f"   TUs with ZERO unfinished functions in their unit: {len(vg0)}  "
      f"({pct(len(vg0), len(vg))} of vocab-gap) — refusal there cannot be corpus artifact")
print()

# ---- H6: stubs --------------------------------------------------------------
st = db.execute("SELECT COUNT(*), COUNT(DISTINCT unit) FROM functions WHERE is_stub=1").fetchone()
stx = db.execute("SELECT COUNT(*) FROM functions WHERE is_stub=1 AND excluded=0").fetchone()[0]
print(f"H6 STUBS (decomp.db is_stub=1): {st[0]} rows over {st[1]} units; "
      f"{stx} not excluded")
# **A SECOND CORRECTION, kept in the file.** `functions.unit`'s schema comment
# says `"src/system/char/Char.cpp"`; the column actually holds the objdiff UNIT
# NAME (`default/system/char/Char`). Joining it against source paths returned
# `0 rows over 0 units` — a green-looking zero produced by a join that could
# never match, which is `docs/STATUS.md` trap 5 in one line. Mapped through
# `report.json`'s unit name -> source_path instead.
u2s = {u["name"]: (u.get("metadata") or {}).get("source_path") for u in units}
wl = {r["src"] for r in rows}
srows = list(db.execute("SELECT unit, symbol FROM functions WHERE is_stub=1"))
inwl = [r for r in srows if u2s.get(r[0]) in wl]
print(f"   of which inside the 878-TU workload: {len(inwl)} rows over "
      f"{len({r[0] for r in inwl})} units  (joined through report.json's "
      f"unit-name -> source_path; the schema comment on `functions.unit` is wrong)")
print()

print("MATCHED TUs (26) — the decomp's own verdict on them")
for r in rows:
    if r["cls"] == "match":
        print(f"   {'C' if r['complete'] else ('i' if r['complete'] is False else '?')} "
              f"U={r['U']:4d}/F={r['F']:4d}  R={r['R']:5d}  {r['src']}")
print()
print("CAPTURE-FAIL (8) — the one place real c2 itself refuses")
for r in rows:
    if r["cls"] == "capture-fail":
        print(f"   joined={r['joined']}  in_db={r['in_db']}  {r['src']}")
print()
print("TOP 20 TUs BY REFUSAL MASS, with the decomp's verdict on the same unit")
for r in sorted(rows, key=lambda r: -r["R"])[:20]:
    c = "COMPLETE" if r["complete"] else ("incomplete" if r["complete"] is False else "UNJOINED")
    print(f"   R={r['R']:6d} E={r['E']:6d}  U={r['U']:4d}/F={r['F']:4d}  {c:10s}  {r['src']}")
print()
print("UNJOINED TUs (no report.json unit) — named, not dropped")
for r in NJ:
    print(f"   R={r['R']:6d}  {r['cls']:12s} in_db={r['in_db']}  {r['src']}")

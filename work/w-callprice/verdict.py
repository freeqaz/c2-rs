#!/usr/bin/env python3
"""w-callprice — the whole-workload verdict and census, from a `gap --jsonl`.

Compares two scans as a SET of per-TU verdicts by name (never a diff), plus the
census totals and the family's own columns. Used to grade the R1 counterfactual.

Usage: verdict.py A.jsonl [B.jsonl]
"""
import json
import sys
from collections import Counter

FAMILY = "expr-call-in-expr"


def load(path):
    tu, fn_total, fn_in, em_bound, em_in = {}, 0, 0, 0, 0
    famb = fame = 0
    emitk = Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        tu[r["src"]] = r["class"]
        fn_total += r.get("fn_total", 0)
        fn_in += r.get("fn_in_class", 0)
        e = r.get("emit") or {}
        em_bound += e.get("emit-bound", 0)
        em_in += e.get("emit-in-class", 0)
        for k, n in (r.get("fn_blockers") or {}).items():
            if k.startswith(FAMILY):
                famb += n
        for k, n in (r.get("emit_blockers") or {}).items():
            emitk[k.split("|", 1)[0]] += n
            if k.startswith(FAMILY):
                fame += n
    return dict(tu=tu, fn_total=fn_total, fn_in=fn_in, em_bound=em_bound,
                em_in=em_in, famb=famb, fame=fame, emitk=emitk)


def show(tag, s):
    c = Counter(s["tu"].values())
    print(f"[{tag}] TUs {len(s['tu'])}  " +
          "  ".join(f"{k} {v}" for k, v in sorted(c.items())))
    print(f"       function census {s['fn_in']}/{s['fn_total']}   "
          f"emitted in class {s['em_in']}/{s['em_bound']}   "
          f"family {s['famb']} bodies / {s['fame']} emitted")


a = load(sys.argv[1])
show(sys.argv[1].split("/")[-1], a)
if len(sys.argv) > 2:
    b = load(sys.argv[2])
    show(sys.argv[2].split("/")[-1], b)
    print(f"\nDELTA  in-class {b['fn_in']-a['fn_in']:+d}   "
          f"emitted-in-class {b['em_in']-a['em_in']:+d}   "
          f"family bodies {b['famb']-a['famb']:+d}   "
          f"family emitted {b['fame']-a['fame']:+d}")
    moved = [(k, a["tu"][k], v) for k, v in b["tu"].items()
             if a["tu"].get(k) != v]
    print(f"per-TU verdict SET compared BY NAME: "
          f"{len(set(a['tu']) - set(b['tu']))} only-in-A, "
          f"{len(set(b['tu']) - set(a['tu']))} only-in-B, {len(moved)} changed")
    for k, x, y in moved[:20]:
        print(f"   {k}: {x} -> {y}")
    ka, kb = a["emitk"], b["emitk"]
    keys = set(ka) | set(kb)
    ch = [(k, ka.get(k, 0), kb.get(k, 0)) for k in keys if ka.get(k, 0) != kb.get(k, 0)]
    print(f"\nemitted-blocker KEY MAP: {len(keys)} keys, "
          f"{sum(1 for k in keys if k not in ka)} appeared, "
          f"{sum(1 for k in keys if k not in kb)} vanished, {len(ch)} changed")
    for k, x, y in sorted(ch, key=lambda t: t[1] - t[2], reverse=True)[:20]:
        print(f"   {y-x:+7d}  {x:7d} -> {y:7d}  {k}")

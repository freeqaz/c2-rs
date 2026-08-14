#!/usr/bin/env python3
"""w-readphase — read a `c2rs gap --jsonl` stream and print the cause structure.

Tooling, not a gate. Every number it prints is reproducible from the jsonl the
scan wrote; the command is recorded in the rung doc.

usage: ladder.py <scan.jsonl> [--head-cause CAUSE]
"""
import collections
import json
import sys


def load(path):
    rows = []
    with open(path) as fh:
        for line in fh:
            line = line.strip()
            if not line or not line.startswith("{"):
                continue
            r = json.loads(line)
            if "src" not in r:
                continue
            rows.append(r)
    return rows


def main():
    path = sys.argv[1]
    head = "gl-stop-26-introduced"
    if "--head-cause" in sys.argv:
        head = sys.argv[sys.argv.index("--head-cause") + 1]
    rows = load(path)
    print(f"rows graded: {len(rows)}")
    byclass = collections.Counter(r.get("class") for r in rows)
    print("class:", dict(byclass))

    first = collections.Counter(
        r["gate_cause"] for r in rows if r.get("gate_cause"))
    print("\nfirst-cause histogram (TUs):")
    for k, v in first.most_common():
        print(f"  {v:5d}  {k}")

    allc = collections.Counter()
    for r in rows:
        for c in r.get("gate_causes") or []:
            allc[c] += 1
    print("\nALL-cause histogram (TUs; decode_causes' independent re-ask):")
    for k, v in allc.most_common():
        print(f"  {v:5d}  {k}")

    # the head class: what else fires on it
    hd = [r for r in rows if r.get("gate_cause") == head]
    print(f"\nhead class `{head}`: {len(hd)} TUs")
    co = collections.Counter()
    arity = collections.Counter()
    for r in hd:
        cs = [c for c in (r.get("gate_causes") or []) if c != head]
        arity[len(cs) + 1] += 1
        for c in cs:
            co[c] += 1
    print("  co-firing causes (the diagnostic's counterfactual successor set):")
    for k, v in co.most_common():
        print(f"    {v:5d}  {k}")
    print("  cause-set arity within the head class:")
    for k in sorted(arity):
        print(f"    arity {k}: {arity[k]} TUs")

    # emitted-blocked mass restricted to the head class
    def sum_blockers(rs, field):
        c = collections.Counter()
        for r in rs:
            for k, v in (r.get(field) or {}).items():
                c[k] += v
        return c

    for field in ("emit_blockers", "fn_blockers"):
        allb = sum_blockers(rows, field)
        hdb = sum_blockers(hd, field)
        print(f"\n{field}: {len(allb)} keys summing {sum(allb.values())} "
              f"(whole workload)")
        print(f"{field}: {len(hdb)} keys summing {sum(hdb.values())} "
              f"(head class only)")
        print("  head-class top 15:")
        for k, v in hdb.most_common(15):
            print(f"    {v:7d}  {k}")

    # per-TU emitted totals
    tot_emit = sum((r.get("emit") or {}).get("emitted", 0) for r in rows)
    tot_inclass = sum((r.get("emit") or {}).get("in_class", 0) for r in rows)
    print(f"\nemitted total {tot_emit}, in-class {tot_inclass}")


main()

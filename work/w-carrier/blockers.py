#!/usr/bin/env python3
"""blockers.py — the per-TU `fn_blockers` / `emit_blockers` histograms out of a
scan JSONL, summed over the 878 TUs, printed sorted.

    python3 work/w-carrier/blockers.py <scan.jsonl> [<scan.jsonl>]

With two files it prints only the rows that DIFFER, plus both totals — which is
what makes "the widening is exactly as large as it says it is" a measurement
rather than a claim. `codegen-gap` cannot register a payment on this axis at all
(board #1164, it partitions per TU), so this is where a key motion has to be
read.
"""
import json
import sys


def load(path):
    fn, em, seen = {}, {}, 0
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except ValueError:
                continue
            seen += 1
            for k, v in (r.get("fn_blockers") or {}).items():
                fn[k] = fn.get(k, 0) + v
            for k, v in (r.get("emit_blockers") or {}).items():
                em[k] = em.get(k, 0) + v
    return fn, em, seen


def show(name, a, b):
    keys = sorted(set(a) | set(b))
    print(f"== {name}: total {sum(a.values())} -> {sum(b.values())}")
    for k in keys:
        x, y = a.get(k, 0), b.get(k, 0)
        if x != y:
            print(f"   {x:>8} -> {y:>8}   {k}")


def main(argv):
    fa, ea, na = load(argv[1])
    if len(argv) < 3:
        for k, v in sorted(fa.items(), key=lambda kv: -kv[1]):
            print(f"{v:>8}  {k}")
        print(f"-- {na} records, fn_blockers total {sum(fa.values())}, "
              f"emit_blockers total {sum(ea.values())}")
        return 0
    fb, eb, nb = load(argv[2])
    print(f"records: {na} -> {nb}")
    show("fn_blockers", fa, fb)
    show("emit_blockers", ea, eb)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

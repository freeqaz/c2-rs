#!/usr/bin/env python3
"""w-instr — BOTH-ENDS EVIDENCE from two `c2rs gap --jsonl` scans.

    python3 work/w-instr/bothends.py <base.jsonl> <tip.jsonl>

Names every `fn_blockers` / `emit_blockers` key that moved, both totals, the
per-TU class counts, and the `gap-metric` diff. A key that moved is printed with
BOTH numbers rather than judged, and a key that VANISHED is called out
separately — a missing key is silent, which is `docs/GAPS.md`'s most-recorded
failure shape.
"""

import collections
import json
import sys


def load(path):
    out = {}
    for ln in open(path):
        d = json.loads(ln)
        if d.get("record") == "provenance" or "src" not in d:
            continue
        out[d["src"]] = d
    return out


def agg(recs, field):
    t = collections.Counter()
    for d in recs.values():
        for k, v in (d.get(field) or {}).items():
            t[k] += v
    return t


def cmp_field(base, tip, field):
    b, t = agg(base, field), agg(tip, field)
    print("## %s" % field)
    print("  base: %d keys, sum %d" % (len(b), sum(b.values())))
    print("  tip : %d keys, sum %d" % (len(t), sum(t.values())))
    moved = sorted(k for k in set(b) | set(t) if b.get(k, 0) != t.get(k, 0))
    print("  KEYS THAT MOVED: %d" % len(moved))
    for k in moved:
        note = ""
        if k not in t:
            note = "   <<<< VANISHED"
        elif k not in b:
            note = "   <<<< NEW"
        print("    %-58s %8d -> %8d%s" % (k, b.get(k, 0), t.get(k, 0), note))
    print()


def main():
    base, tip = load(sys.argv[1]), load(sys.argv[2])
    print("# w-instr BOTH-ENDS EVIDENCE")
    print("# base %s\n# tip  %s\n" % (sys.argv[1], sys.argv[2]))
    print("## TU set")
    print("  base %d TUs, tip %d TUs, symmetric difference %d\n"
          % (len(base), len(tip), len(set(base) ^ set(tip))))
    print("## per-TU class")
    for tag, recs in (("base", base), ("tip", tip)):
        c = collections.Counter(d.get("class") for d in recs.values())
        print("  %-5s %s" % (tag, dict(sorted(c.items()))))
    print()
    for f in ("fn_blockers", "emit_blockers", "fn_gate_refusals"):
        cmp_field(base, tip, f)


if __name__ == "__main__":
    main()

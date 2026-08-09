#!/usr/bin/env python3
"""w-biquad — the per-TU verdict set, compared BY NAME across two scans.

A COUNT hides one TU lost and one gained. This prints three lists — only in
base, only in tip, and changed — and a direction for every change, so a
regression cannot cancel a gain.

    verdicts.py base.jsonl tip.jsonl
"""
import json
import sys


def load(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        # The first row is a PROVENANCE record, not a TU. Skipped by the
        # absence of `src` rather than by position, so a schema that grows a
        # second header row cannot silently drop a TU.
        if "src" not in r:
            continue
        out[r["src"]] = r.get("class", "?")
    return out


def main():
    a, b = load(sys.argv[1]), load(sys.argv[2])
    only_a = sorted(set(a) - set(b))
    only_b = sorted(set(b) - set(a))
    changed = sorted(k for k in set(a) & set(b) if a[k] != b[k])
    print(f"base {len(a)} TUs, tip {len(b)} TUs")
    print(f"only-in-base {len(only_a)}")
    for k in only_a:
        print(f"   - {k}  [{a[k]}]")
    print(f"only-in-tip  {len(only_b)}")
    for k in only_b:
        print(f"   + {k}  [{b[k]}]")
    print(f"changed      {len(changed)}")
    for k in changed:
        arrow = "TOWARD match" if b[k] == "match" else "AWAY from match"
        print(f"   ~ {k}  {a[k]} -> {b[k]}   ({arrow})")
    bad = [k for k in changed if a[k] == "match" and b[k] != "match"]
    print(f"REGRESSIONS (was match, is not): {len(bad)} {bad}")


main()

#!/usr/bin/env python3
"""partition.py — the partition motion, PER SYMBOL and never as a net.

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

    partition.py <base.jsonl> <tip.jsonl>

`fnbyte-differs` falling by 726 is consistent with 726 conversions and with
1,000 conversions beside 274 regressions. `work/w-splice/PREREG.md` §3 item 1
registers the floor in the form that can tell them apart:

    **zero symbols move `exact -> differs`**, checked per
    `(TU, FnCensus::emit_name)` — board **#918**, never
    `IlFunction::mangled_name`.

The scan publishes one `fnbyte-differs-fn|…|<sym>` key per differing function
and nothing per exact one, so `differs` is read directly and `exact` is its
complement within the graded population of each TU. A symbol that leaves the
`differs` set therefore either converted or stopped being graded, and the second
is printed as its own row rather than counted as a win.
"""

import collections
import json
import sys


def load(path):
    """`{src: {sym}}` — the differing functions, and `{src: {sym}}` for every
    function the walk reached at all."""
    differs = collections.defaultdict(set)
    reached = collections.defaultdict(set)
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        src = r["src"]
        reached[src]  # a TU with no differ still has a row
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-differs-fn|"):
                differs[src].add(k.split("|", 4)[4])
            elif k.startswith("fnbyte-differs-why|"):
                differs[src].add(k.split("|", 5)[5])
    return differs, reached


def main():
    a, ra = load(sys.argv[1])
    b, rb = load(sys.argv[2])
    srcs = sorted(set(ra) | set(rb))
    converted = []   # differs -> not-differs
    regressed = []   # not-differs -> differs
    vanished = []    # the TU stopped being graded
    for src in srcs:
        if src not in rb:
            vanished.append((src, len(a.get(src, ()))))
            continue
        if src not in ra:
            vanished.append((src, -len(b.get(src, ()))))
            continue
        for s in a.get(src, set()) - b.get(src, set()):
            converted.append((src, s))
        for s in b.get(src, set()) - a.get(src, set()):
            regressed.append((src, s))

    print("TUs graded at both ends: %d" % len(set(ra) & set(rb)))
    print("differs, base: %6d" % sum(len(v) for v in a.values()))
    print("differs, tip : %6d" % sum(len(v) for v in b.values()))
    print("\n=== CONVERTED (differs -> exact), per (TU, emit_name): %d" % len(converted))
    fam = collections.Counter(s for _, s in converted)
    print("  distinct symbols: %d" % len(fam))
    for s, n in fam.most_common(12):
        print("    %5d  %s" % (n, s[:100]))
    print("\n=== REGRESSED (exact -> differs): %d" % len(regressed))
    if not regressed:
        print("  (none) — PREREG §3 item 1 holds")
    for src, s in regressed[:40]:
        print("    %-58s %s" % (s[:58], src))
    print("\n=== TUs GRADED AT ONE END ONLY: %d" % len(vanished))
    if not vanished:
        print("  (none)")
    for src, n in vanished[:20]:
        print("    %-58s %d" % (src, n))


if __name__ == "__main__":
    main()

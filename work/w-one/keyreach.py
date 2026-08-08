#!/usr/bin/env python3
"""w-one — the REFUSAL-KEY reachability screen, over the real workload.

    python3 work/w-one/keyreach.py <scan.jsonl>

`w-mrslot` found `value_bound` had no producer that could reach it and
`w-front3` confirmed it a second way, then screened all 21 `IlOp` variants and
found no second instance at the variant level. Both searches ran over the
**source**: find the refusal's input, find every producer of it, compare.

This screen asks the same question from the other end and needs no source
analysis at all: **every refusal key `c2-il` can mint, against every key the
878-TU workload actually witnesses.** A key with zero witnesses over 2,463,443
functions is a rung nobody on this workload can reach — a rung deleted from the
roadmap for free, or a guard whose cost is zero and which can stay forever.

Two directions, and the second is the one that paid on this lane:

  UNREPORTED
           the key exists in `c2-il` and no function in the workload reports it.

           **This is NOT a list of unreachable rungs and must never be quoted as
           one.** `fn_blockers` is a FIRST-BLOCKER histogram — the reader stops
           at the first refusal by design — so "no function reports this key"
           means *no function in the workload stops HERE FIRST*, which is a
           statement about what shadows it. That is this project's own "a
           first-blocker key is a NAME, not a DISTANCE" one level up, and the
           number is large for exactly that reason: **184 of 218 on the first
           run**, which is a measurement of shadowing and not of reachability.
           It is printed as a denominator, per `w-inread`'s #1002 rule that a
           denominator is not published until it has been printed on both sides
           of a change.

  LOUD-BUT-DECLARED-IMPOSSIBLE
           the key's own source comment says the state cannot happen, and the
           workload witnesses it. That is the same defect as an unreachable rung
           pointing the other way, and it is worth strictly more, because a
           false unreachability in a comment is what sends the next lane to lift
           a clause that moves nothing (`w-front3` §5.2) — or, here, to record a
           row as *"a real refusal with no lift"* when it has one.

The comment scan is deliberately crude — a fixed phrase list over the 12 source
lines above each key — and its output is a LIST TO READ, never a count to quote.
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "crates", "c2-il", "src")

# Phrases that assert a state cannot occur. Matched case-insensitively over the
# comment lines immediately above a key's mint site.
IMPOSSIBLE = [
    "cannot be reached", "can never", "cannot happen", "never happens",
    "unreachable", "cannot occur", "impossible", "by construction cannot",
    "no producer", "nothing produces",
]

KEY = re.compile(r'"((?:expr|assign|call|store|param|chain|tail|framed|cmp|body|leaf|seq|ctor|dtor|cf)-[a-z0-9\-]{3,})"')


def source_keys():
    out = {}
    for dirpath, _, files in os.walk(SRC):
        for fn in files:
            if not fn.endswith(".rs"):
                continue
            path = os.path.join(dirpath, fn)
            lines = open(path).read().splitlines()
            in_tests = False
            for i, ln in enumerate(lines):
                if re.match(r"\s*mod tests\b", ln):
                    in_tests = True
                if in_tests:
                    continue
                for m in KEY.finditer(ln):
                    k = m.group(1)
                    ctx = "\n".join(lines[max(0, i - 12):i])
                    claim = [p for p in IMPOSSIBLE if p in ctx.lower()]
                    prev = out.get(k)
                    if prev is None or (claim and not prev[2]):
                        out[k] = (os.path.relpath(path, ROOT), i + 1, claim)
    return out


def main():
    seen = {}
    for ln in open(sys.argv[1]):
        d = json.loads(ln)
        if d.get("record") == "provenance":
            continue
        for m in ("fn_blockers", "emit_blockers", "fn_gate_refusals"):
            for k, v in (d.get(m) or {}).items():
                seen[k] = seen.get(k, 0) + v
    keys = source_keys()

    def witnesses(k):
        return sum(v for w, v in seen.items() if w == k or w.startswith(k + "-")
                   or w.startswith(k + ":"))

    unreported, loud_impossible = [], []
    for k, (path, line, claim) in sorted(keys.items()):
        n = witnesses(k)
        if n == 0:
            unreported.append((k, path, line, claim))
        elif claim:
            loud_impossible.append((k, path, line, n, claim))

    print("REFUSAL-KEY REACHABILITY over %d workload TUs" % sum(1 for _ in open(sys.argv[1])))
    print("  keys minted in crates/c2-il (non-test): %d" % len(keys))
    print("  keys witnessed as a FIRST blocker:      %d" % (len(keys) - len(unreported)))
    print("  UNREPORTED — shadowed OR unreachable, and this screen CANNOT")
    print("               tell those apart:           %d" % len(unreported))
    print("  witnessed AND declared impossible:      %d" % len(loud_impossible))
    print()
    print("== WITNESSED, AND THE SOURCE SAYS IT CANNOT BE ==")
    for k, path, line, n, claim in sorted(loud_impossible, key=lambda r: -r[3]):
        print("  %8d  %-44s %s:%d  %s" % (n, k, path, line, claim))
    print()
    print("== UNREPORTED — never the FIRST blocker. NOT an unreachability claim ==")
    for k, path, line, claim in unreported:
        print("  %-48s %s:%d%s" % (k, path, line, "  [declared impossible]" if claim else ""))


if __name__ == "__main__":
    main()

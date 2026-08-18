#!/usr/bin/env python3
"""w-grammarscreen — the SECOND, DIFFERENTLY-CONSTRUCTED count of reach.

Board #3288's transferable rule: *any enumeration whose output is quoted as a
denominator owes a second, independently constructed count.*

`Block::feature()` renders a site's `ctx` into the census key the 878-TU scan
already publishes under `blocking features`. That is PRODUCTION code, measured
on master, with no probe in the tree — so intersecting it with the probe's
reached set is a genuinely independent check on the probe.

It is PARTIAL by construction and is reported as a check, never merged into the
reach number:
  * a feature is a `ctx` plus a rendered suffix (`:eof`, `:mid`, `-0xNN`,
    `-<intrinsic>`, `-cflow-<n>`), so the mapping is by LONGEST-PREFIX;
  * 38 ctx strings are shared by 2+ sites (305 sites), so a feature can name a
    set of sites rather than one;
  * 237 sites pass a ctx VARIABLE rather than a literal and are invisible here;
  * the scan prints only the TOP 20 features plus a count of the rest, so the
    check covers the most populous refusals and not the tail.

    crosscheck.py <scan.log> <reached.txt>
"""
import json
import os
import re
import sys

ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
LANE = os.path.join(ROOT, "work", "w-grammarscreen")


def main():
    scan, reached_path = sys.argv[1], sys.argv[2]
    sites = [json.loads(l) for l in open(os.path.join(LANE, "sites.jsonl"))]
    by_ctx = {}
    for s in sites:
        if s["ctx_kind"] == "literal":
            by_ctx.setdefault(s["ctx"], []).append(s)
    reached = set()
    for line in open(reached_path):
        p = line.strip().rsplit(":", 2)
        if len(p) == 3:
            reached.add((p[0], int(p[1]), int(p[2])))

    text = open(scan, encoding="utf-8", errors="replace").read().splitlines()
    feats = []
    in_block = False
    for line in text:
        if line.strip().startswith("blocking features"):
            in_block = True
            continue
        if in_block:
            m = re.match(r"\s+(\d+) \(\s*[\d.]+%\)\s+(\S+)\s*$", line)
            if m:
                feats.append((int(m.group(1)), m.group(2)))
                continue
            if feats:
                break
    print("blocking features parsed from %s: %d" % (os.path.basename(scan), len(feats)))
    ctxs = sorted(by_ctx, key=len, reverse=True)
    hit = miss = amb = 0
    for n, f in feats:
        cand = next((c for c in ctxs if f == c or f.startswith(c)), None)
        if cand is None:
            print("  %-58s %9d  ctx NOT RESOLVED (computed ctx or a non-blk producer)" % (f, n))
            amb += 1
            continue
        ss = by_ctx[cand]
        r = [s for s in ss if (s["file"], s["line"], s["col"]) in reached]
        state = "REACHED %d/%d" % (len(r), len(ss))
        if not r:
            state += "   *** CONTRADICTION: the production census reports this feature blocking %d bodies and the probe calls every one of its sites QUIET ***" % n
            miss += 1
        else:
            hit += 1
        print("  %-58s %9d  ctx=%-40s %s" % (f, n, cand, state))
    print("\nresolved %d, contradictions %d, unresolved %d" % (hit, miss, amb))


if __name__ == "__main__":
    main()

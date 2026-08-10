#!/usr/bin/env python3
"""Per-TU rows for the named TUs, and the two set relations the rung asserts.

    work/w-selbind/detail.py <scan.jsonl> <src>...
"""
import json
import sys


def main():
    rows = {}
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        rows[r["src"]] = r
    for s in sys.argv[2:]:
        r = rows.get(s)
        if r is None:
            print("%s: ABSENT from the scan" % s)
            continue
        bc = r.get("bind_checks", {})
        print("%s\n   class %s  gl_body_starts %s  selective_bind %s"
              % (s, r["class"], r.get("gl_body_starts"), r.get("selective_bind")))
        print("   gate_causes %s" % r.get("gate_causes"))
        print("   emitted %s  named-gate %s  named-wide %s  subset-gate %s  subset-wide %s"
              % (bc.get("selbind-emitted"), bc.get("selbind-emitted-named-gate"),
                 bc.get("selbind-emitted-named-wide"),
                 bc.get("selbind-emit-subset-gate-tus", 0),
                 bc.get("selbind-emit-subset-wide-tus", 0)))

    graded = [s for s, r in rows.items() if r["class"] != "capture-fail"]

    def cover(s):
        v = rows[s].get("gl_body_starts")
        return bool(v) and v[0] == v[1]

    def sub_g(s):
        return rows[s].get("bind_checks", {}).get("selbind-emit-subset-gate-tus", 0) == 1

    def sub_w(s):
        return rows[s].get("bind_checks", {}).get("selbind-emit-subset-wide-tus", 0) == 1

    cov = {s for s in graded if cover(s)}
    g = {s for s in graded if sub_g(s)}
    w = {s for s in graded if sub_w(s)}
    print("\nSET RELATIONS over %d graded TUs" % len(graded))
    print("  full-coverage (the 1:1 path's necessary condition)      %d" % len(cov))
    print("  emit-subset GATE (the selective path's)                 %d" % len(g))
    print("  emit-subset WIDE (…with board #2783's relaxation)       %d" % len(w))
    print("  cover minus gate-subset  %d   %s"
          % (len(cov - g), sorted(cov - g)[:8]))
    print("  gate-subset minus cover  %d   %s"
          % (len(g - cov), sorted(g - cov)[:8]))
    print("  gate-subset subset of wide-subset: %s (%d outside)"
          % (g <= w, len(g - w)))


main()

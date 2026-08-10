#!/usr/bin/env python3
"""Print the fields `CEILING.md` §11.4 item 8 is about, for named TUs.

    work/w-decouple/rowfields.py <scan.jsonl> <src> [<src> ...]

`gate_cause` / `gate_causes` is the field that answers "does
`Bindings::per_record` bind this TU"; `emit-bound` / `emit-gate-segments` and
`fn_names` are the two per-function bindings that do NOT (#918, #2621), and
they are printed beside it so the disagreement is a reading rather than a
claim.
"""
import json
import sys

FIELDS = [
    "class",
    "gate_cause",
    "gate_causes",
    "fn_names",
    "fn_total",
    "detail",
]
EMIT = [
    "emit-bound",
    "emit-records",
    "emit-record-offsets",
    "emit-gate-segments",
    "emit-emitted",
    "bytefrac-denominator",
    "bytefrac-exact",
    "bytefrac-refused",
    "fnbyte-denominator",
    "fnbyte-exact",
    "fnbyte-differs",
    "fnbyte-refused",
]


def key(r):
    for k in ("src", "file", "path"):
        if k in r:
            return r[k]
    return None


def main():
    path = sys.argv[1]
    want = set(sys.argv[2:])
    for line in open(path):
        r = json.loads(line)
        k = key(r)
        if want and k not in want:
            continue
        print("=== %s" % k)
        for f in FIELDS:
            if f in r:
                print("  %-24s %s" % (f, r[f]))
        e = r.get("emit", {})
        for f in EMIT:
            if f in e:
                print("  %-24s %s" % (f, e[f]))
        for f in ("fn_cflow", "fn_blockers", "emit_blockers"):
            if r.get(f):
                print("  %-24s %s" % (f, json.dumps(r[f], sort_keys=True)))


main()

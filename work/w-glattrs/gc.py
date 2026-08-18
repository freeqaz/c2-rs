#!/usr/bin/env python3
"""gc.py — aggregate GRID-C out of a `c2rs gap --jsonl` log.

Derived from the log, never accumulated (`docs/rungs/README.md` probe rule 2).

Keys the scratch instrument writes, per TU:

    ga-real|some / ga-real|none    the SHIPPED reader's whole-file verdict
    ga-real-records                records it decoded
    ga-real-noinline               of those, records with bit 6 CLEAR
    ga-tu|inc=<yes|no>|new=<..>    the modelled verdict under each reader
    ga-rec|<direct|escape|high|..> record forms
    ga-esc-attr|<hh>               ATTR byte on an ESCAPED record
    gc-<arm>|form=<f>              per CALL EDGE: real c2's kept/inlined
    gc-<arm>|form=<f>|bit=<b>      …crossed with the decoded FN_FLAG_INLINABLE

`arm` is `w-fence2` GRID-W's observable: does the reference caller's `.text`
COMDAT carry a `REL24` naming the callee?
"""

import collections
import json
import sys


def load(path):
    tot = collections.Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record"):
            continue
        for k, v in (r.get("emit") or {}).items():
            tot[k] += v
    return tot


def main(argv):
    t = load(argv[1])
    print("== the shipped reader's whole-file verdict, over 870 captured TUs")
    for k in sorted(t):
        if k.startswith("ga-real"):
            print(f"  {k:<28} {t[k]:>8}")
    print("\n== the modelled verdict under each reader (inc = incumbent)")
    for k in sorted(t):
        if k.startswith("ga-tu"):
            print(f"  {k:<28} {t[k]:>8}")
    print("\n== SIZE forms, over records the incumbent framing reaches")
    for k in sorted(t):
        if k.startswith("ga-rec"):
            print(f"  {k:<28} {t[k]:>8}")
    print("\n== ATTR byte on ESCAPED records")
    for k in sorted(t, key=lambda x: -t[x]):
        if k.startswith("ga-esc-attr"):
            print(f"  {k:<28} {t[k]:>8}")

    print("\n== GRID-C: real c2's verdict per call edge, by the callee's SIZE form")
    arms = ["kept", "inlined", "unknown"]
    forms = sorted({k.split("form=")[1] for k in t if k.startswith("gc-") and "|bit=" not in k})
    print(f"  {'form':>12} " + "".join(f"{a:>10}" for a in arms) + f"{'total':>10}")
    for f in forms:
        row = [t.get(f"gc-{a}|form={f}", 0) for a in arms]
        print(f"  {f:>12} " + "".join(f"{v:>10}" for v in row) + f"{sum(row):>10}")
    tot = [sum(t.get(f"gc-{a}|form={f}", 0) for f in forms) for a in arms]
    print(f"  {'TOTAL':>12} " + "".join(f"{v:>10}" for v in tot) + f"{sum(tot):>10}")

    print("\n== THE ORACLE SCORE. `bit=noinline` asserts c2 KEPT the call.")
    print("   A `noinline` edge in the `inlined` column is a decode that is WRONG")
    print("   about c2, in the direction that emits bytes.")
    print(f"  {'form':>12} {'bit':>11} " + "".join(f"{a:>10}" for a in arms))
    viol = 0
    for f in forms:
        for b in ("inlinable", "noinline", "noattr"):
            row = [t.get(f"gc-{a}|form={f}|bit={b}", 0) for a in arms]
            if not any(row):
                continue
            print(f"  {f:>12} {b:>11} " + "".join(f"{v:>10}" for v in row))
            if b == "noinline":
                viol += row[1]
    print(f"\n  COUNTEREXAMPLES (noinline asserted, c2 inlined): {viol}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

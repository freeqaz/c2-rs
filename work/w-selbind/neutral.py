#!/usr/bin/env python3
"""Four-level neutrality, BY NAME and with DIRECTIONS.

#2667: the 878 workload TUs collapse to 841 basenames, so a basename compare
drops 37 rows while printing "0 MOVED". Everything here is keyed on the full
`src` path.

    work/w-selbind/neutral.py <base.jsonl> <tip.jsonl>
"""
import json
import sys


def load(p):
    rows = {}
    for line in open(p):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        rows[r["src"]] = r
    return rows


ACCEPT = {"match"}


def main():
    b = load(sys.argv[1])
    t = load(sys.argv[2])
    only_b = sorted(set(b) - set(t))
    only_t = sorted(set(t) - set(b))
    print("base %d TUs, tip %d TUs; only-in-base %d, only-in-tip %d"
          % (len(b), len(t), len(only_b), len(only_t)))
    for s in only_b[:5]:
        print("   only-in-base %s" % s)
    for s in only_t[:5]:
        print("   only-in-tip  %s" % s)

    moved = []
    toward = away = 0
    for s in sorted(set(b) & set(t)):
        if b[s]["class"] != t[s]["class"]:
            moved.append((s, b[s]["class"], t[s]["class"]))
            if t[s]["class"] in ACCEPT and b[s]["class"] not in ACCEPT:
                toward += 1
            elif b[s]["class"] in ACCEPT and t[s]["class"] not in ACCEPT:
                away += 1
    print("LEVEL 1 class verdicts:      CHANGED %d   (toward acceptance %d, away %d)"
          % (len(moved), toward, away))
    for row in moved:
        print("   %s: %s -> %s" % row)

    # LEVEL 2 — the per-TU byte triple, as a SET comparison, with the sums
    # printed underneath: a lane that moved one TU +1 and another -1 prints an
    # unchanged sum.
    def triple(r):
        e = r.get("emit", {})
        return (e.get("fnbyte-exact", 0), e.get("fnbyte-differs", 0),
                e.get("fnbyte-refused", 0))
    tmoved = []
    for s in sorted(set(b) & set(t)):
        if triple(b[s]) != triple(t[s]):
            tmoved.append((s, triple(b[s]), triple(t[s])))
    print("LEVEL 2 per-TU byte triples: CHANGED %d" % len(tmoved))
    for row in tmoved[:20]:
        print("   %s: %s -> %s" % row)
    for tag, d in (("base", b), ("tip", t)):
        se = sum(triple(r)[0] for r in d.values())
        sd = sum(triple(r)[1] for r in d.values())
        sr = sum(triple(r)[2] for r in d.values())
        print("  %-4s per-TU triple SUM: exact %d  differs %d  refused %d"
              % (tag, se, sd, sr))

    # LEVEL 1b — the gate's own first refusal, by name. Not a verdict, but the
    # field this lane's shipped clause is most likely to move.
    cmoved = []
    for s in sorted(set(b) & set(t)):
        if b[s].get("gate_cause") != t[s].get("gate_cause"):
            cmoved.append((s, b[s].get("gate_cause"), t[s].get("gate_cause")))
    print("LEVEL 1b gate FIRST cause:   CHANGED %d" % len(cmoved))
    for row in cmoved[:20]:
        print("   %s: %s -> %s" % row)
    smoved = []
    for s in sorted(set(b) & set(t)):
        if sorted(b[s].get("gate_causes", [])) != sorted(t[s].get("gate_causes", [])):
            smoved.append((s, b[s].get("gate_causes"), t[s].get("gate_causes")))
    print("LEVEL 1c gate cause SET:     CHANGED %d" % len(smoved))
    for row in smoved[:20]:
        print("   %s: %s -> %s" % row)


main()

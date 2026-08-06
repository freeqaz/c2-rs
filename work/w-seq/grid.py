#!/usr/bin/env python3
"""grid.py — THE FAMILY TABLE. The whole `fnbyte-differs` population, joined.

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

    grid.py <scan.jsonl>

Joins the scan's four per-function witness families on `(TU, symbol)` keyed on
`FnCensus::emit_name` (**#918**):

    fnbyte-differs-fn|…      what the bytes are        (w-fnbyte)
    fnbyte-differs-why|…     what the callees are      (w-seq)
    fnbyte-splice-fn|…       SPLICE-P's verdict        (w-seq)
    fnbyte-splice0-fn|…      SPLICE-0's verdict        (w-seq)

and prints the cross-tabulation the three briefs after this one are sized off.
**Every table carries its denominator** and the residue row is printed even when
it is zero (`docs/STATUS.md` trap 5).

The one thing this file must never do is report a rate without the population it
is over: `w-fnbyte` §5 records that quoting pairs against distinct symbols is how
a header-inline template is mistaken for a defect rate. Counts here are PAIRS.
"""

import collections
import json
import sys


def load(path):
    rows = {}
    spliceN = collections.Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        src = r["src"]
        for k, v in (r.get("emit") or {}).items():
            if k.startswith("fnbyte-differs-why|"):
                _, shape, ncal, dispo, refblr, sym = k.split("|", 5)
                d = rows.setdefault((src, sym), {})
                d.update(shape=shape, ncallees=int(ncal), dispo=dispo, refblr=refblr)
            elif k.startswith("fnbyte-differs-fn|"):
                _, shape, words, first, sym = k.split("|", 4)
                d = rows.setdefault((src, sym), {})
                d.update(words=words, first=first)
            elif k.startswith("fnbyte-splice-fn|"):
                _, shape, verdict, words, sym = k.split("|", 4)
                rows.setdefault((src, sym), {})["spliceP"] = verdict
            elif k.startswith("fnbyte-splice0-fn|"):
                _, shape, verdict, sym = k.split("|", 3)
                rows.setdefault((src, sym), {})["splice0"] = verdict
            elif k.startswith("fnbyte-spliceN|"):
                spliceN[k.split("|", 1)[1]] += v
    return rows, spliceN


def family(r):
    """The FAMILY, in the mission's own three-way split.

    (a) mechanism **I** — a same-TU callee the port could name, whose code c2
        put in the caller. Split by whether the port can lower that callee.
    (b) **E behind a parse refusal** — c2's whole body is one `blr` and the
        callee's IL is refused by a named production.
    (c) named, never a remainder.
    """
    d = set(r.get("dispo", "").split(","))
    refused = sorted(x for x in d if x.startswith("refused:"))
    local = {x for x in d if x.startswith("body") or x in ("empty", "reduces")}
    blr = r.get("refblr") == "refblr"
    if blr and refused:
        return "(b) E behind a parse refusal"
    if blr and not refused and not local:
        return "(c) c2 emits blr, callee is EXTERNAL"
    if blr:
        return "(c) c2 emits blr, callee parses"
    if refused and not local:
        return "(a) I, callee PARSE-REFUSED"
    if refused and local:
        return "(a) I, mixed callees"
    if "body:exact" in d and not (d & {"extern", "ambiguous"}):
        return "(a) I, callee lowered EXACT"
    if local and not (d & {"extern", "ambiguous"}):
        return "(a) I, callee lowered WRONG"
    if local:
        return "(a) I, mixed local/extern"
    return "(c) every callee EXTERNAL"


def table(title, counter, den, note=""):
    print("\n=== %s === (denominator %d)%s" % (title, den, note))
    tot = 0
    for k, n in counter.most_common():
        print("  %6d  %5.1f%%  %s" % (n, 100.0 * n / den, k))
        tot += n
    print("  %6d  ------  ACCOUNTED" % tot)
    if tot != den:
        print("  %6d  !!!!!!  UNACCOUNTED" % (den - tot))


def main():
    rows, spliceN = load(sys.argv[1])
    n = len(rows)
    print("differ pairs joined on (TU, emit_name): %d" % n)

    table("THE FAMILY TABLE",
          collections.Counter(family(r) for r in rows.values()), n)
    table("FAMILY x SHAPE",
          collections.Counter("%-34s %s" % (family(r), r["shape"])
                              for r in rows.values()), n)

    # THE PRICING GRID: can the port lower the callee, and does a splice give
    # c2's own bytes?
    print("\n=== THE PRICING GRID === (denominator %d)" % n)
    print("  %-34s %-14s %-9s %-9s %6s" %
          ("family", "shape", "SPLICE-P", "SPLICE-0", "pairs"))
    g = collections.Counter()
    for r in rows.values():
        g[(family(r), r["shape"], r.get("spliceP", "-"), r.get("splice0", "-"))] += 1
    tot = 0
    for (f, s, p, z), c in sorted(g.items(), key=lambda x: -x[1]):
        print("  %-34s %-14s %-9s %-9s %6d" % (f, s, p, z, c))
        tot += c
    print("  %-34s %-14s %-9s %-9s %6d" % ("ACCOUNTED", "", "", "", tot))

    # The headline: how many of the 3,195 have c2's own answer already sitting
    # in the same obj as some callee's emitted body.
    ok0 = sum(1 for r in rows.values() if r.get("splice0") == "exact")
    okP = sum(1 for r in rows.values() if r.get("spliceP") == "exact")
    ok0_lowerable = sum(
        1 for r in rows.values()
        if r.get("splice0") == "exact" and "body:exact" in r.get("dispo", "")
    )
    print("\n=== THE HEADLINE === (denominator %d)" % n)
    print("  SPLICE-P exact (setup ++ callee body)          : %6d  %5.1f%%"
          % (okP, 100.0 * okP / n))
    print("  SPLICE-0 exact (c2(caller) == c2(callee))      : %6d  %5.1f%%"
          % (ok0, 100.0 * ok0 / n))
    print("  ... of which the port ALREADY lowers the callee")
    print("      byte-exactly (disposition `body:exact`)    : %6d  %5.1f%%"
          % (ok0_lowerable, 100.0 * ok0_lowerable / n))

    print("\n=== SPLICE-N (two or more callees) ===")
    for k, v in sorted(spliceN.items(), key=lambda x: -x[1]):
        print("  %6d  %s" % (v, k))
    if not spliceN:
        print("  (none)")

    # #925's caution, kept live: is a big family one idiom?
    print("\n=== SYMBOL FAMILIES PER FAMILY ROW ===")
    bym = collections.defaultdict(collections.Counter)
    for (src, sym), r in rows.items():
        bym[family(r)][sym[:64]] += 1
    for f in sorted(bym, key=lambda f: -sum(bym[f].values())):
        print("  -- %s: %d pairs, %d distinct symbols"
              % (f, sum(bym[f].values()), len(bym[f])))
        for s, c in bym[f].most_common(3):
            print("       %5d  %s" % (c, s))


if __name__ == "__main__":
    main()

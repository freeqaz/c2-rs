#!/usr/bin/env python3
"""taxonomy.py — the residual `fnbyte-differs` population, by FAMILY and by
MECHANISM.

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

    taxonomy.py <scan.jsonl>

`work/w-fnbyte/differ_taxonomy.txt` classified the 4,711 by what the BYTES look
like (family A/B/C/D — shorter, longer, shared prefix). That is a description,
not a mechanism, and it cannot tell "c2 inlined the callee" from "c2 dropped the
call". This adds the axis that can: the port's own **callee set**, resolved
against the same TU's census rows, published by the scan as

    fnbyte-differs-why|<shape>|<ncallees>|<dispositions>|<refblr>|<symbol>

Keyed per `(TU, symbol)` on `FnCensus::emit_name` (**#918**) — the scan writes
one key per differing function and this reads them back. Counts are of PAIRS,
never of distinct symbols: `w-fnbyte` §5 records that quoting one against the
other is how a header-inline population is mistaken for a defect rate.

**Every table prints its denominator**, and the residue class is printed even
when it is empty (`docs/STATUS.md` trap 5).
"""

import collections
import json
import sys


def load(path):
    """(rows, splice, contains, totals) off one scan's JSONL."""
    rows = []
    splice = collections.Counter()
    contains = collections.Counter()
    totals = collections.Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k, v in (r.get("emit") or {}).items():
            if k.startswith("fnbyte-differs-why|"):
                _, shape, ncal, dispo, refblr, sym = k.split("|", 5)
                rows.append(
                    {
                        "src": r["src"],
                        "shape": shape,
                        "ncallees": int(ncal),
                        "dispo": dispo,
                        "refblr": refblr,
                        "sym": sym,
                    }
                )
            elif k.startswith("fnbyte-splice|"):
                splice[k.split("|", 1)[1]] += v
            elif k.startswith("fnbyte-contains|"):
                contains[k.split("|", 1)[1]] += v
            elif k in (
                "fnbyte-callee-total",
                "fnbyte-callee-resolved-emit",
                "fnbyte-callee-resolved-mangled",
                "fnbyte-differs",
                "fnbyte-exact",
            ):
                totals[k] += v
    return rows, splice, contains, totals


def mechanism(r):
    """The MECHANISM, from the disposition set and c2's own body.

    * `E-refused`  — a callee is parse-refused AND c2's whole body is `blr`:
      c2 applied E behind its own dead-code elimination and the port cannot
      establish emptiness. `INLINE_PREDICATE.md` §1.4's population.
    * `I-local`    — every callee is a same-TU body that parses and is not
      empty, and c2's body is not `blr`: c2 expanded it.
    * `E?-refused-body` / `I?-…` — the same disposition with the OTHER body
      observable. Printed apart rather than folded, because the two facts
      disagreeing is itself the finding for that cell.
    * `extern`     — no callee is defined here at all. Neither mechanism can be
      about the callee, so this is family (c) and is named, not a remainder.
    """
    d = set(r["dispo"].split(","))
    ref_is_blr = r["refblr"] == "refblr"
    refused = {x for x in d if x.startswith("refused:")}
    local = {x for x in d if x.startswith("body") or x in ("empty", "reduces")}
    if d == {"no-callee"}:
        return "no-callee"
    if refused and not local:
        return "E-refused" if ref_is_blr else "I/E?-refused-nonblr"
    if refused and local:
        return "mixed-refused-local"
    if local and not (d & {"extern", "ambiguous"}):
        return "E?-local-blr" if ref_is_blr else "I-local"
    if local:
        return "mixed-local-extern"
    return "extern-blr" if ref_is_blr else "extern"


def table(title, counter, den):
    print("\n=== %s === (denominator %d)" % (title, den))
    tot = 0
    for k, n in counter.most_common():
        print("  %6d  %5.1f%%  %s" % (n, 100.0 * n / den, k))
        tot += n
    print("  %6d  ------  ACCOUNTED" % tot)
    if tot != den:
        print("  %6d  !!!!!!  UNACCOUNTED (a partition break)" % (den - tot))


def main():
    rows, splice, contains, totals = load(sys.argv[1])
    n = len(rows)
    print("differ pairs carrying a why-key: %d" % n)
    print("fnbyte-differs (scan total):     %d" % totals["fnbyte-differs"])
    if n != totals["fnbyte-differs"]:
        print("  !! the forensics did not reach every differ — %d unexplained"
              % (totals["fnbyte-differs"] - n))

    table("BY SHAPE", collections.Counter(r["shape"] for r in rows), n)
    table("BY MECHANISM", collections.Counter(mechanism(r) for r in rows), n)
    table(
        "BY MECHANISM x SHAPE",
        collections.Counter("%-22s %s" % (mechanism(r), r["shape"]) for r in rows),
        n,
    )
    table("BY DISPOSITION SET", collections.Counter(r["dispo"] for r in rows), n)
    table(
        "c2's whole body is one `blr`",
        collections.Counter(r["refblr"] for r in rows),
        n,
    )

    # Family (b): the productions, each with its blocked count.
    prod = collections.Counter()
    for r in rows:
        for d in r["dispo"].split(","):
            if d.startswith("refused:"):
                prod[d[len("refused:"):]] += 1
    print("\n=== FAMILY (b): PARSE-REFUSED PRODUCTIONS === (%d differs name one)"
          % sum(1 for r in rows if "refused:" in r["dispo"]))
    for k, v in prod.most_common():
        blr = sum(
            1
            for r in rows
            if ("refused:" + k) in r["dispo"].split(",") and r["refblr"] == "refblr"
        )
        print("  %6d  (%4d with c2 body == blr)  %s" % (v, blr, k))
    if not prod:
        print("  (none)")

    # SPLICE-P.
    print("\n=== SPLICE-P (graded by real c2's own COMDATs) ===")
    for k in sorted(splice):
        print("  %6d  %s" % (splice[k], k))
    if not splice:
        print("  (none)")

    print("\n=== CONTAINMENT: is the callee's code inside c2's body? ===")
    for k in sorted(contains):
        print("  %6d  %s" % (contains[k], k))

    print("\n=== #918 CONTROL: callee resolution under the two name bindings ===")
    print("  callees named by a differing body : %d" % totals["fnbyte-callee-total"])
    print("  resolved under emit_name          : %d" % totals["fnbyte-callee-resolved-emit"])
    print("  resolved under mangled_name       : %d" % totals["fnbyte-callee-resolved-mangled"])

    # Symbol families, per mechanism — #925's caution, kept live.
    print("\n=== TOP SYMBOLS PER MECHANISM (pairs) ===")
    bym = collections.defaultdict(collections.Counter)
    for r in rows:
        bym[mechanism(r)][r["sym"][:70]] += 1
    for m in sorted(bym, key=lambda m: -sum(bym[m].values())):
        print("  -- %s (%d pairs, %d distinct symbols)"
              % (m, sum(bym[m].values()), len(bym[m])))
        for s, c in bym[m].most_common(5):
            print("      %5d  %s" % (c, s))


if __name__ == "__main__":
    main()

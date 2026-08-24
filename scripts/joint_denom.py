#!/usr/bin/env python3
"""joint_denom.py — INTERROGATE THE DISTANCE DENOMINATOR before anything uses it.

    scripts/joint_denom.py work/<lane>/base.jsonl [--near-max 10] [--out DIR]

`docs/STATUS.md`'s generated block carries two rows that cannot both be naive
readings of "how far is this TU from matching":

    TU distance to match, blocked functions | <=0: 17, ...
    878-TU dc3 workload scan                | match 25, ...

**They are two different splitters over the same `.ex` stream**, and neither
dominates the other (board **#3364**, which found three TUs matching byte-exact
while FBM *refused* their body):

  * `fn_total` / `fn_in_class` come from `IlBundle::census_functions()`, which
    splits at the **census marker `4C 4F 11`** and classifies each segment with
    `FnVerdict` (`crates/c2-harness/src/gap/scan.rs:394-397`).
  * `match` is the **differential** -- the port's whole obj against real c2's --
    and the port consumes `IlBundle::functions()`, which splits at the **gate
    marker `4F 1F`** (`gap/scan.rs:1334-1348` records the two splitters).

So `near_match_tus(k)` (`gap/report.rs:465`) is not "distance to match". This
script prints the **cross tabulation** rather than either number alone, and it
refuses to print a comparison it cannot stand behind: a zero denominator on any
axis is a loud failure (exit 4), never a silent pass.

It is offline -- it reads a `c2rs gap --jsonl` stream and needs no toolchain.
std-library Python only; nothing here is in the `crates/` std-only fence, but
there are no third-party imports either.
"""

import json
import sys
import os
from collections import Counter, defaultdict


def load(path):
    prov, rows = None, []
    with open(path) as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            o = json.loads(line)
            if o.get("record") == "provenance":
                prov = o
            else:
                rows.append(o)
    return prov, rows


def blocked(r):
    return r["fn_total"] - r["fn_in_class"]


def emit_blocked(r):
    e = r.get("emit", {})
    return max(0, e.get("emit-emitted", 0) - e.get("emit-in-class", 0))


def main(argv):
    if len(argv) < 2:
        sys.stderr.write(__doc__)
        return 2
    path = argv[1]
    near_max = 10
    out = None
    i = 2
    while i < len(argv):
        if argv[i] == "--near-max":
            near_max = int(argv[i + 1]); i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.stderr.write("unknown arg %r\n" % argv[i]); return 2

    prov, rows = load(path)
    if not rows:
        sys.stderr.write("VOID: 0 TU rows in %s -- graded nothing\n" % path)
        return 4

    print("== PROVENANCE (a number without its corpus is not a number) ==")
    if prov:
        print("  c2rs      %s%s" % (prov.get("c2rs_head", "?")[:12],
                                    " DIRTY" if prov.get("c2rs_dirty") else ""))
        print("  binary    %s" % prov.get("binary_sha", "?"))
        print("  workload  %s%s" % (prov.get("workload_head", "?")[:12],
                                    " DIRTY" if prov.get("workload_dirty") else ""))
    print("  TU rows   %d" % len(rows))

    cls = Counter(r["class"] for r in rows)
    print("\n== CLASS TABLE, the denominator of everything below ==")
    for k in sorted(cls):
        print("  %-14s %4d" % (k, cls[k]))

    graded = [r for r in rows if r["class"] != "capture-fail"]
    matchset = {r["src"] for r in rows if r["class"] == "match"}
    # near_match_tus()'s own filter, transcribed from gap/report.rs:465-476.
    censusable = [r for r in graded if r["fn_total"] > 0]
    if not matchset or not censusable:
        sys.stderr.write("VOID: match=%d censusable=%d -- a zero denominator\n"
                         % (len(matchset), len(censusable)))
        return 4

    print("\n== D_census(k) -- `near_match_tus(k)`, census splitter `4C 4F 11` ==")
    for k in (0, 1, 10, 100, 1000):
        print("  <=%-5d %4d" % (k, sum(1 for r in censusable if blocked(r) <= k)))

    d0 = {r["src"] for r in censusable if blocked(r) == 0}
    print("\n== THE CROSS TABULATION -- two sets, never one number ==")
    print("  |D_census(0)|            %4d   (census route: every `.ex` body in class)" % len(d0))
    print("  |D_match|                %4d   (gate route: whole obj byte-exact vs real c2)" % len(matchset))
    print("  |D_census(0) & D_match|  %4d" % len(d0 & matchset))
    only_c = sorted(d0 - matchset)
    only_m = sorted(matchset - d0)
    print("  in D_census(0), NOT match %3d   <- every body in class and the obj still differs" % len(only_c))
    print("  match, NOT in D_census(0) %3d   <- byte-exact obj carrying blocked census bodies (#3364's shape)" % len(only_m))
    print("\n  -- D_census(0) \\ D_match --")
    for s in only_c:
        r = next(x for x in censusable if x["src"] == s)
        print("     %-58s class=%-12s fn %d/%d" % (s, r["class"], r["fn_in_class"], r["fn_total"]))
    print("  -- D_match \\ D_census(0) --")
    for s in only_m:
        r = next((x for x in rows if x["src"] == s), None)
        print("     %-58s blocked=%-3d fn %d/%d  %s"
              % (s, blocked(r), r["fn_in_class"], r["fn_total"],
                 ",".join(sorted(r.get("fn_blockers", {})))[:60]))

    # The emit-set ceiling's own known-answer control, re-derived here rather
    # than trusted from the report line: a `match` TU whose `.ex` segment count
    # differs from its obj `.text` COMDAT-leader count. Doc comment
    # (`gap/report.rs`, `emit_set_violations`): "a nonzero here means `fn_total`
    # and `emit-emitted` are not counting the things this reading says they
    # count, and the ceiling above is void."
    viol = [r for r in rows if r["class"] == "match"
            and r["fn_total"] != r.get("emit", {}).get("emit-emitted", 0)]
    print("\n== emit_set_violations() -- KNOWN ANSWER 0 ==")
    print("  violations %d of %d match TUs" % (len(viol), len(matchset)))
    for r in viol:
        print("     %-58s fn_total=%d emit-emitted=%d"
              % (r["src"], r["fn_total"], r.get("emit", {}).get("emit-emitted", 0)))

    near = sorted((r for r in censusable
                   if 1 <= blocked(r) <= near_max and r["src"] not in matchset),
                  key=lambda r: (blocked(r), r["src"]))
    print("\n== NEAR = D_census(%d) \\ D_match, 1..%d blocked bodies ==" % (near_max, near_max))
    print("  |NEAR| = %d" % len(near))
    if not near:
        sys.stderr.write("VOID: NEAR is empty -- nothing to ladder\n")
        return 4

    # SUBSET STRUCTURE, never a ranking: per TU, the SIZE of its head-key union
    # and the union itself. The union is a LOWER BOUND on the construct set the
    # TU's closure requires (a body reports one blocker however many it has --
    # board #3131), so it is printed as a bound and labelled as one.
    print("\n== PER-TU HEAD-KEY UNION -- a LOWER BOUND on the complete set ==")
    print("  (`#3131`: the port stops at the first refusal BY DESIGN, so this is")
    print("   a floor on each TU's construct set, never the set itself.)")
    print("  %-58s %7s %6s  %s" % ("TU", "blocked", "|union|", "head keys"))
    allkeys = Counter()
    for r in near:
        ks = {k: v for k, v in r.get("fn_blockers", {}).items() if v}
        for k, v in ks.items():
            allkeys[k] += v
        print("  %-58s %7d %6d  %s"
              % (r["src"], blocked(r), len(ks), " ".join(sorted(ks))))

    sizes = sorted(len({k for k, v in r.get("fn_blockers", {}).items() if v}) for r in near)
    print("\n  union sizes over NEAR (sorted, not ranked): %s" % sizes)
    print("  min %d  median %d  max %d" % (sizes[0], sizes[len(sizes) // 2], sizes[-1]))
    print("  DISTINCT head keys over all of NEAR: %d" % len(allkeys))
    print("  blocked function slots over all of NEAR: %d" % sum(blocked(r) for r in near))

    if out:
        os.makedirs(out, exist_ok=True)
        with open(os.path.join(out, "near_tus.txt"), "w") as fh:
            for r in near:
                fh.write(r["src"] + "\n")
        with open(os.path.join(out, "near.json"), "w") as fh:
            json.dump([{"src": r["src"], "class": r["class"],
                        "fn_total": r["fn_total"], "fn_in_class": r["fn_in_class"],
                        "blocked": blocked(r), "emit_blocked": emit_blocked(r),
                        "fn_blockers": r.get("fn_blockers", {}),
                        "fn_complete": r.get("fn_complete", {})} for r in near],
                      fh, indent=1, sort_keys=True)
        print("\n  wrote %s/near_tus.txt (%d) and %s/near.json" % (out, len(near), out))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

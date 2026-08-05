#!/usr/bin/env python3
"""substitute.py — the correction `work/w-dclass/rerank.py` needs, and the
reason its greedy ladder over-counts.

Read-only measurement tooling, outside the std-only Rust workspace on purpose,
same status as `rerank.py`, `scripts/gt_dump.py` and `scripts/plot_perf.py`.
Nothing under `crates/` is touched or linked.

WHY THIS EXISTS
---------------
`rerank.py` fixed a real defect: the FRONTIER's published ranking is a MARGINAL
over a CONJUNCTION, because a TU converts only when EVERY one of its distinct
blocker keys closes. That arithmetic is right and this script keeps it.

What it also assumes, in one line, is that **closing a key REMOVES it**:

    def convertible(rows, closed):
        return [s for s, b in rows.items() if set(b) <= closed]

A blocker key is the census label on the **FIRST** refusal in a body. Making the
parser stop refusing at that byte does not make the body emittable — it makes
the census report **the NEXT unmodeled byte**. So closing a key does not remove
it from a TU's set. It **SUBSTITUTES a successor**, and the successor may be a
key that is already on the frontier — in which case the TU does not convert and
the ladder's credit for it was never real.

w-cmp measured this on the relational family with the `C2RS_SINK_REL`
counterfactual (two 878-TU scans, one env var apart, same binary):

    expr-cmp-eq  2208 -> 0        expr-brfalse  3097 -> 5484   (+2387)
    expr-cmp-ne   863 -> 0        expr-brtrue    126 ->  659   (+533)
    cmp-lt/le/gt/ge 227 -> 0      emitted census 38458 -> 38458 (+0)

`rerank.py` credited `expr-cmp-eq` with **+3 TU** at step 1 and `expr-cmp-ne`
with **+2** at step 3. The measured conversion is **0 TUs and 0 functions**:
every one of those TUs re-acquired `expr-brfalse` or `expr-brtrue`, both of
which were already frontier keys.

WHAT THIS SCRIPT DOES
---------------------
Given the SAME scan taken twice — once with a key's sink OFF and once ON — it
prints the **substitution map** (which successor keys absorbed the closed one)
and re-runs the ladder on the ON scan, so the credit a lane is about to claim
is measured rather than modelled.

It is deliberately NOT a predictor. There is no way to know a successor key
before the sink exists, because the successor does not have a name yet. The
honest workflow this encodes is: **sink first, re-rank second, build third.**

USAGE
    substitute.py <scan-off.jsonl> <scan-on.jsonl>

It EXITS NON-ZERO when it finds no differing key, because a sink that moved
nothing is a broken measurement rather than a clean result — absence read as
success is this project's most-repeated defect (16 recorded instances) and the
generalizing fix on record is a positive check with a printed count.
"""

import json
import sys
from collections import Counter, defaultdict

sys.path.insert(0, __file__.rsplit("/", 2)[0] + "/w-dclass")
try:
    from rerank import FRONTIER, convertible  # the ladder, not a second copy
except ImportError:  # pragma: no cover - path shape differs
    FRONTIER, convertible = None, None


def load(path):
    """(per-TU emit_blockers, workload-wide key totals) off a `gap --jsonl` scan."""
    rows, totals = {}, Counter()
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        b = r.get("emit_blockers") or {}
        rows[r.get("src")] = b
        for k, v in b.items():
            totals[k] += v
    return rows, totals


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    off_rows, off_tot = load(sys.argv[1])
    on_rows, on_tot = load(sys.argv[2])

    print(f"TU records: OFF {len(off_rows)}  ON {len(on_rows)}  (a positive count)")
    print(f"blocked EMITTED sites: OFF {sum(off_tot.values())}  "
          f"ON {sum(on_tot.values())}")
    if sum(off_tot.values()) == sum(on_tot.values()):
        print("  the two totals are EQUAL — the sink converted ZERO functions.")
    else:
        print(f"  CONVERTED: {sum(off_tot.values()) - sum(on_tot.values())} "
              f"blocked emitted function sites")

    closed = sorted(k for k in off_tot if on_tot.get(k, 0) == 0 and off_tot[k])
    gained = sorted((on_tot[k] - off_tot.get(k, 0), k)
                    for k in on_tot if on_tot[k] > off_tot.get(k, 0))
    print(f"\nKEYS CLOSED by the sink: {len(closed)}")
    for k in closed:
        print(f"  {k:<44} {off_tot[k]:>7} -> 0")
    print(f"\nSUCCESSOR keys that ABSORBED them: {len(gained)}")
    for d, k in sorted(gained, reverse=True):
        print(f"  {k:<44} {off_tot.get(k, 0):>7} -> {on_tot[k]:<7}  {d:+}")
    absorbed = sum(d for d, _ in gained)
    freed = sum(off_tot[k] for k in closed)
    print(f"\n  closed mass {freed} · absorbed {absorbed} · "
          f"net converted {freed - absorbed}")

    if not closed and not gained:
        print("MOVED NOTHING — refusing to exit 0 on a sink that measured nothing")
        return 2

    # ---- the per-TU substitution, over the FRONTIER only -------------------
    if FRONTIER is None:
        print("\n(rerank.py not importable — skipping the frontier ladder)")
        return 0
    print("\nPER-FRONTIER-TU SUBSTITUTION (only TUs whose key set changed)")
    sub = defaultdict(list)
    for src in FRONTIER:
        a, b = set(off_rows.get(src, {})), set(on_rows.get(src, {}))
        if a != b:
            print(f"  {src}")
            print(f"      lost: {sorted(a - b)}")
            print(f"      got : {sorted(b - a)}")
            for k in a - b:
                sub[k].extend(sorted(b - a))

    print("\nLADDER CREDIT, before and after — the number a lane would have claimed")
    for k in sorted(sub):
        off_solo = [s for s in FRONTIER if set(off_rows.get(s, {})) == {k}]
        print(f"  closing {k:<32} rerank credited +{len(off_solo)} TU")
    # A TU converted only if its blocker set is EMPTY under the sink. Anything
    # weaker is the modelled credit again, one level in.
    really = [s for s in FRONTIER if not set(on_rows.get(s, {}))]
    print(f"  MEASURED conversions after the sink: {len(really)} TU "
          f"(a TU converts only when its key set is EMPTY)")
    print(f"  ladder-credited {sum(len([s for s in FRONTIER if set(off_rows.get(s, {})) == {k}]) for k in sub)} TU "
          f"· measured {len(really)} TU")
    return 0


if __name__ == "__main__":
    sys.exit(main())

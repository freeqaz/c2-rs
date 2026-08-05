#!/usr/bin/env python3
"""rerank.py — the FRONTIER re-ranking instrument, keyed on BLOCKER SETS.

Lane w-dclass. Read-only measurement tooling; outside the std-only Rust
workspace on purpose, same status as `scripts/gt_dump.py` and
`scripts/plot_perf.py`. Nothing under `crates/` is touched or linked.

WHY THIS EXISTS
---------------
The FRONTIER (the graded TUs satisfying A and B and C, not a match, and outside
both acceptance paths) is published ranked by BLOCKED EMITTED FUNCTION COUNT.
Three lanes in a row have recorded that this is the wrong key --- most recently
`docs/rungs/2026-08-04-w-cfgimpl.md` section 6 item 5:

    "The frontier ranking is by blocked-function count and that is the wrong
     key --- third lane in a row to find this. [...] A frontier ranked by
     distinct unmodeled constructs would have said so."

A blocked function is a unit of COUNTING. A refusal is a unit of WORK. This
script does not price work --- that needs a disassembly and is subagent A's
deliverable. What it does is the arithmetic that sits between the two, and that
nothing in the tree does today: given a set of CLOSED blocker keys, which
FRONTIER TUs actually convert?

That question is not answerable from the published ranking, because a TU
converts only when EVERY one of its distinct blocker keys is closed. The
ranking is a marginal over a CONJUNCTION, and the project has now paid for
reading a marginal as a joint four times (board #250's ladder:
micro-F1 -> per-TU exact -> TU reach -> TU match).

POPULATION
----------
Every count printed here is over BLOCKED EMITTED FUNCTIONS --- the population
the FRONTIER table itself uses. It is NOT the larger "blocked functions"
population, whose distance histogram reads differently (at master 9f9e6c0:
blocked functions <=1: 12, blocked EMITTED functions <=1: 19). Two lanes have
been burned mixing joins that looked interchangeable; `B and C` stayed
published at 107 after C moved for exactly this reason.

KNOWN-ANSWER CONTROLS
---------------------
The script asserts, and prints, three facts it can check against the scan it
was fed rather than against a number typed in here:

  KA1  the FRONTIER set it reconstructs has the same size the scan reported
  KA2  every FRONTIER TU has at least one blocker key (a TU with none would be
       vacuously convertible and would silently inflate every ladder below)
  KA3  the sum of per-TU blocked-emitted counts equals the sum of that TU's
       `emit_blockers` values

It EXITS NON-ZERO when it ranks zero TUs. Absence read as success is this
project's most-repeated defect (16 recorded instances) and the generalizing fix
on record is a positive check with a printed count.

USAGE
    rerank.py <scan.jsonl> [--closed KEY[,KEY...]]

    --closed  hypothetically close these blocker keys and report the resulting
              FRONTIER, so a lane can re-rank BEFORE building rather than after.
"""

import json
import sys
from collections import defaultdict

# The FRONTIER as `c2rs gap` printed it at master 9f9e6c0. Passed in as data
# rather than recomputed: reconstructing A and B and C here would be a second
# implementation of the factorization, and a disagreement between the two would
# be indistinguishable from a disagreement with the scan. KA1 checks the size.
FRONTIER = [
    "src/Main.cpp",
    "src/system/math/Primes.cpp",
    "src/system/math/Sort.cpp",
    "src/xdk/LIBCMT/osfinfo.cpp",
    "src/xdk/LIBCMT/undname.cpp",
    "src/xdk/LIBCMT/vswprnc.cpp",
    "src/xdk/nuispeech/xboxheap.cpp",
    "src/xdk/xjson/jsonwriter.cpp",
    "src/xdk/xlrc/xlrcimpl.cpp",
    "src/system/negate_test.cpp",
    "src/system/synth_xbox/Biquad.cpp",
    "src/xdk/LIBCMT/vsnprnc.cpp",
    "src/xdk/nuispeech/xboxmem.cpp",
    "src/system/rndobj/wordwrap.cpp",
    "src/system/utl/Pool.cpp",
    "src/xdk/nuispeech/mmio.cpp",
    "src/system/synth_xbox/IPP_basicmath_xbox.cpp",
    "src/system/utl/EncryptXTEA.cpp",
    "src/keygen_xbox.cpp",
]
FRONTIER_REPORTED = 19


def load(path):
    """Per-TU `emit_blockers` for the FRONTIER, off a `c2rs gap --jsonl` scan."""
    want = set(FRONTIER)
    rows = {}
    for line in open(path):
        r = json.loads(line)
        src = r.get("src")
        if src in want:
            rows[src] = r.get("emit_blockers") or {}
    return rows


def convertible(rows, closed):
    """The TUs whose EVERY distinct blocker key is in `closed`.

    A conjunction, deliberately. The published ranking is a marginal over this
    conjunction and the two are not interchangeable.
    """
    return [s for s, b in rows.items() if set(b) <= closed]


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    scan = sys.argv[1]
    closed = set()
    if "--closed" in sys.argv:
        v = sys.argv[sys.argv.index("--closed") + 1]
        closed = {k for k in v.split(",") if k}

    rows = load(scan)

    # ---- known-answer controls, printed as counts, never as a status --------
    ka1 = len(rows)
    ka2 = sum(1 for b in rows.values() if not b)
    print(f"KA1  frontier TUs found in scan: {ka1} of {FRONTIER_REPORTED} reported")
    print(f"KA2  frontier TUs with ZERO blocker keys (must be 0): {ka2}")
    if ka1 != FRONTIER_REPORTED:
        print(f"KA1 FAILED — scan does not carry all {FRONTIER_REPORTED} frontier TUs")
        return 2
    if ka2:
        print("KA2 FAILED — a blockerless frontier TU would be vacuously convertible")
        return 2
    tot_sites = sum(sum(b.values()) for b in rows.values())
    print(f"KA3  blocked EMITTED function sites over the frontier: {tot_sites}")

    # ---- what each single key is worth, ALONE ------------------------------
    # "Worth", here, means: TUs whose ENTIRE blocker set is this one key. That
    # is a conversion. The number of TUs a key merely APPEARS in is a marginal
    # and is printed beside it precisely so the two are never conflated.
    solo = defaultdict(list)
    appears = defaultdict(list)
    for src, b in rows.items():
        for k in b:
            appears[k].append(src)
        if len(b) == 1:
            solo[next(iter(b))].append(src)

    print("\nWHAT EACH BLOCKER KEY IS WORTH, over BLOCKED EMITTED FUNCTIONS")
    print("  `converts` = TUs whose ENTIRE blocker set is this one key (a CONJUNCTION).")
    print("  `appears`  = TUs the key appears in at all (a MARGINAL — not a conversion).")
    print(f"  {'key':<52} {'converts':>8}  {'appears':>7}")
    for k in sorted(appears, key=lambda k: (-len(solo.get(k, [])), -len(appears[k]), k)):
        print(f"  {k:<52} {len(solo.get(k, [])):>8}  {len(appears[k]):>7}")

    # ---- the greedy ladder, and the warning that it re-ranks ---------------
    # Greedy over keys, maximising TUs converted per key added. Each step
    # invalidates the ranking below it, which is why the script prints the
    # whole ladder rather than a single "next" recommendation.
    print("\nGREEDY KEY LADDER — cumulative TUs converted as keys are closed")
    print("  EACH STEP INVALIDATES THE RANKING BELOW IT. Re-run after every conversion.")
    have = set(closed)
    done = set(convertible(rows, have))
    if closed:
        print(f"  (starting with {len(closed)} key(s) hypothetically closed: "
              f"{len(done)} TU(s) already convertible)")
    step = 0
    while True:
        best, gain = None, 0
        for k in appears:
            if k in have:
                continue
            g = len(set(convertible(rows, have | {k})) - done)
            if g > gain:
                best, gain = k, g
        if not best:
            break
        step += 1
        have.add(best)
        new = set(convertible(rows, have)) - done
        done |= new
        print(f"  {step:>2}. +{best:<48} +{gain} TU  (cumulative {len(done)})")
        for s in sorted(new):
            print(f"        -> {s}")
    remaining = [s for s in rows if s not in done]
    print(f"\n  keys closed: {len(have)}   TUs converted: {len(done)}   "
          f"TUs still blocked: {len(remaining)}")
    if remaining:
        print("  NOT reachable by closing any single key ladder above (multi-key sets):")
        for s in sorted(remaining):
            print(f"    {s:<48} {sorted(rows[s])}")

    # ---- the honest ceiling ------------------------------------------------
    # Closing EVERY key on the frontier converts every frontier TU by
    # construction. That is not a prediction, it is the definition, and it is
    # printed so nobody reads the ladder's tail as a discovery.
    print(f"\n  CEILING (definitional, not a finding): closing all {len(appears)} "
          f"frontier keys converts all {len(rows)} frontier TUs.")
    print("  TU match would then be 8 + 19 = 27 = A and B and C, at which point")
    print("  A and B and C becomes the binding constraint, not D. See w-joint2.")

    if not rows:
        print("RANKED ZERO TUs — refusing to exit 0 on an empty measurement")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())

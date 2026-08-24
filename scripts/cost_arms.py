#!/usr/bin/env python3
"""THE `ir0` COST PROTOCOL, committed — board #3451.

A **construct rung** re-expresses already-byte-exact code, so its byte delta is
required-zero and the only axis it can regress on is **port throughput** (the
COST CLAUSE, board #3336). Four lanes have now measured that with this
protocol; **none of them committed the instrument**. `ir0` wrote it, `w-s1bc`
rewrote it, `w-s1c2` rewrote it again (its §6.3 filed the pattern as #3451) and
this is the fourth. Each rewrite is a fresh chance to get the protocol subtly
wrong, and the protocol has already produced two runs that disagreed in SIGN —
which is exactly the kind of finding that cannot accumulate in a tool that is
retyped every time.

    scripts/cost_arms.py --arm base=/path/c2rs-base \\
                         --arm nulldup=/path/c2rs-base-copy \\
                         --arm tip=/path/c2rs-tip \\
                         --rounds 9 --port-iters 2000

# The protocol, and why each clause is here

* **Three arms, and one of them is a NULL.** `nulldup` is a byte-identical copy
  of `base` (verify it with `cmp` and this script does), so its true effect is
  exactly zero. It is not a nicety: `w-s1bc` §4.3 and `w-s1c2` §4.1 both found
  the null arm reading ±0.5–1.1 % with a CI that excluded zero on a loaded box.
  **The null arm is the noise floor, measured rather than assumed**, and a tip
  effect inside it is a bound and not a sign.
* **Arm order rotates every round, and the rotation balances ADJACENCY, not
  merely POSITION.** A fixed order lets a warming or cooling box alias onto the
  arm axis; a rotation that does not complete does **not** remove it (#3468);
  and a *cyclic* rotation that does complete **still** does not remove it
  (#3495). Both were measured on this box, on one tree, one set of binaries:

  ```text
    #3468  rounds=8, cyclic (3 arms — POSITION UNBALANCED)
                                       null +0.57 % [+0.45, +0.68]  split 76 %
    #3468  rounds=9, cyclic (3 arms — position balanced)
                                       null +0.09 % [-0.07, +0.26]  split 51 %
    #3495  rounds=9, cyclic, list `base, nulldup, tip`   (idle box)
                                       null +0.46 % [+0.32, +0.61]  split 71 %
    #3495  rounds=9, cyclic, list `base, tip, nulldup`
                                       null +0.06 % [-0.05, +0.17]  split 54 %
  ```

  The first pair is #3468's finding: at 8 rounds over 3 arms `base` and
  `nulldup` each run first three times and `tip` only twice, and that residue
  lands on the arm axis hard enough to make a **byte-identical** arm read
  +0.57 % with a CI excluding zero.

  The second pair is #3495's, and it is the reason this module no longer uses a
  cyclic rotation at all. `arms[r % n:] + arms[:r % n]` balances each arm's
  *position* within a round — over 9 rounds with 3 arms every arm is first
  three times, second three times, third three times — but over one 3-round
  cycle its adjacent pairs are `base→nulldup ×2`, `nulldup→tip ×2`,
  `tip→base ×2`: **`nulldup` always follows `base` and never precedes it.**
  Reversing exactly that one relation, by declaring the arms as
  `base, tip, nulldup` and changing nothing else, collapsed the same
  byte-identical arm from +0.46 %/71 % to +0.06 %/54 %. A null arm's true
  effect is exactly zero, so **every one of those nonzero readings is an
  artefact of where the arm sat in the declared list**.

  What #3495 did *not* establish is the mechanism: it registered "the sign will
  flip negative" and scored a MISS, because the artefact vanished rather than
  reversing — which refutes *"whichever arm follows `base` pays"*. The design
  below therefore does not assume any particular carryover mechanism. It
  removes the whole class: over one cycle **every ordered pair of arms occurs
  as an adjacency exactly as often as every other**, so no first-order
  carryover of any sign or shape can prefer one arm.

# The rotation, and why `--rounds` must be a multiple of TWICE the arm count

The execution order is one flat sequence of arm runs — round boundaries are not
pauses, so the last arm of round `r` and the first of round `r+1` are as
adjacent as any pair inside a round, and the design balances them too. The unit
that repeats is a **cycle of `L = 2n` rounds** with three exact properties,
verified from scratch at every startup (`--show-design` prints the certificate):

1. **position balance** — each arm occupies each slot of a round exactly twice
   per cycle (this is #3468's criterion, and it is subsumed, not dropped);
2. **cross-adjacency balance** — each of the `n(n-1)` ordered pairs `a→b`,
   `a≠b`, occurs exactly **twice** per cycle;
3. **self-adjacency balance** — each of the `n` pairs `a→a` occurs exactly
   **twice** per cycle.

`L = 2n` is the shortest cycle that can do this, and the counts are forced
rather than chosen: a cycle of `L` rounds has `L·n` adjacencies, of which
`L(n-1)` are inside rounds and are necessarily cross pairs. Balance needs
`n(n-1) | cross` and `n | self`; at `L = 2n` the only solution is 2 and 2, so
every one of the `2n` round boundaries is a self-repeat — each arm getting
exactly two of them, which is why that is balanced rather than a defect. `L = n`
admits no such design for `n = 3` or `n = 4` (checked exhaustively), which is
why the cycle is `2n` and not `n`.

For 3 arms that makes **6** the smallest legal `--rounds` and 12 the next —
9 is now REFUSED, and it is the count three of this protocol's four prior
readings were taken at. See `--rotation cyclic`, which reproduces the old
behaviour for controlled comparison only.
* **The estimator is the per-fixture MINIMUM over rounds**, not the mean of
  medians. Interference only ever makes a run slower, so the minimum is the
  least-contaminated estimate of the same quantity, and it is what `ir0` used.
* **Pairing is per fixture.** Fixtures span three orders of magnitude in
  absolute time; an unpaired mean is a measurement of which fixtures ran.
* **The reference column never enters.** `c2rs perf` prints a reference median
  too. It is real `c2.dll` under wibo and has nothing to do with the port's
  cost; including it would put a 1000x-slower number in the denominator.
* **The SIGN SPLIT is published beside the mean.** An effect of exactly zero
  must split its sign about 50/50. `w-s1c2`'s null arm split **40 %**, and this
  lane's unbalanced run split **76 %** — in both cases the split, not the mean,
  is what said the run could not answer its own question. A mean alone hides
  this, and it is the reason the defect above was findable at all.

# What it CANNOT do, stated so it is not overread

* It measures **this box, now**. Report the load context at both ends; this
  script prints it.
* `--rounds` under about 6 will not separate anything from the null, and 6 over
  3 arms is the smallest balanced setting worth running.
* **A balanced rotation removes arm ORDER, not everything.** It does not remove
  a box whose load changes monotonically across a whole run, and it says nothing
  about cache or allocator state the arms might not share. What certifies a
  given run is the null arm's own reading — CI containing zero and a split near
  50 % — and that is a check to perform per run, never an assumption.
* **The design balances FIRST-ORDER adjacency only**, i.e. what ran immediately
  before. It says nothing about second-order carryover (what ran two slots
  back), and it is not claimed to.
* **One adjacency out of `rounds·n` is necessarily unbalanced, and it is the
  last one.** The cycle is balanced as a *circle*; a real run is a *line*, so
  the wrap from the final arm back to the first never happens. `rounds·n − 1`
  adjacencies cannot divide evenly among `n²` pair classes, so some class must
  be short by one — this design makes it exactly one, which is the floor. At
  `--rounds 12` over 3 arms that is 1 of 36.
* **`--rotation cyclic` is retained ONLY to reproduce a prior reading.** It is
  the defective design #3495 measured; it prints a warning and its numbers are
  not comparable to a balanced run's.
* It is a **timing** instrument, not a grader. Per board #1406 an instrument
  whose output is quoted as evidence should run under `cargo test` or
  `scripts/gate.sh`; this one cannot, for the same reason `scripts/plot_perf.py`
  cannot, and that tension is recorded rather than papered over. **It has no
  authority over correctness** — the byte judge is the sole judge — and nothing
  it prints licenses an emit.

Std-lib only, no external modules; the workspace's zero-dependency rule does
not bind `scripts/`, but there is no reason to spend a dependency here.
"""

import argparse
import filecmp
import itertools
import os
import re
import statistics
import subprocess
import sys
import time

# ---------------------------------------------------------------------------
# THE CARRYOVER-BALANCED ROTATION (board #3521; supersedes the cyclic rotation
# #3468 shipped, which #3495 proved balances POSITION and not ADJACENCY).
#
# `carryover_cycle(n)` returns L = 2n rounds, each a permutation of 0..n-1.
# `verify_design` re-derives every count from the sequence itself and is the
# only thing the runner trusts — the generator is never taken on faith, so a
# generator bug cannot silently ship an unbalanced rotation.
# ---------------------------------------------------------------------------

DESIGN_STEP_BUDGET = 2_000_000
DESIGN_TIME_BUDGET_S = 40.0


class DesignSearchExhausted(Exception):
    """The bounded search did not find a design. Never degrade — refuse."""


def carryover_cycle(n):
    """A cycle of L = 2n rounds balanced for position, cross- and self-adjacency.

    Deterministic: a depth-first search in fixed lexicographic order, no RNG, so
    the same `n` always yields the same design on every box and every run.

    The arithmetic forces the target (see the module doc): at L = 2n each of the
    n(n-1) ordered cross pairs must occur exactly twice and each of the n self
    pairs exactly twice, which means **every round boundary is a self-repeat**.
    That is the constraint the search leans on: given the arm that ended round
    r, round r+1 is chosen from the (n-1)! permutations that start with it.
    """
    if n < 2:
        raise DesignSearchExhausted("a rotation needs at least two arms")
    length = 2 * n
    perms = [tuple(p) for p in itertools.permutations(range(n))]
    by_first = {}
    for p in perms:
        by_first.setdefault(p[0], []).append(p)

    cross = {}
    pos = [[0] * n for _ in range(n)]
    selfc = [0] * n
    chosen = []
    steps = [0]
    deadline = time.monotonic() + DESIGN_TIME_BUDGET_S

    def add(p):
        ok = True
        for i in range(n - 1):
            k = (p[i], p[i + 1])
            cross[k] = cross.get(k, 0) + 1
            if cross[k] > 2:
                ok = False
        for i, a in enumerate(p):
            pos[a][i] += 1
            if pos[a][i] > 2:
                ok = False
        return ok

    def remove(p):
        for i in range(n - 1):
            cross[(p[i], p[i + 1])] -= 1
        for i, a in enumerate(p):
            pos[a][i] -= 1

    def rec(k):
        steps[0] += 1
        if steps[0] > DESIGN_STEP_BUDGET or (
            steps[0] % 4096 == 0 and time.monotonic() > deadline
        ):
            raise DesignSearchExhausted(
                f"no balanced {length}-round cycle found for {n} arms within "
                f"{DESIGN_STEP_BUDGET} steps / {DESIGN_TIME_BUDGET_S:.0f}s"
            )
        if k == length:
            # Close the circle: the last arm wraps onto the first.
            if chosen[-1][-1] != chosen[0][0]:
                return False
            if selfc[chosen[-1][-1]] + 1 > 2:
                return False
            return all(
                cross.get((a, b), 0) == 2
                for a in range(n)
                for b in range(n)
                if a != b
            )
        if k == 0:
            cands = [tuple(range(n))]  # WLOG: the criteria are label-symmetric
            prev = None
        else:
            prev = chosen[-1][-1]
            if selfc[prev] >= 2:
                return False
            cands = by_first[prev]
        for p in cands:
            if prev is not None:
                selfc[prev] += 1
            ok = add(p)
            chosen.append(p)
            if ok and rec(k + 1):
                return True
            chosen.pop()
            remove(p)
            if prev is not None:
                selfc[prev] -= 1
        return False

    if not rec(0):
        raise DesignSearchExhausted(
            f"no balanced {length}-round cycle exists for {n} arms "
            f"(search completed)"
        )
    return chosen


def verify_design(cycle, n):
    """Re-derive every balance count from the flat sequence. Generator-agnostic.

    Returns (ok, report). The report is printed on every run so a reader can
    check the claim rather than take it.
    """
    length = len(cycle)
    seq = [a for rnd in cycle for a in rnd]
    total = len(seq)
    adj = {}
    for t in range(total):
        k = (seq[t], seq[(t + 1) % total])  # circular: the cycle repeats
        adj[k] = adj.get(k, 0) + 1
    pos = [[0] * n for _ in range(n)]
    for rnd in cycle:
        if sorted(rnd) != list(range(n)):
            return False, {"error": f"round {rnd} is not a permutation of {n} arms"}
        for p, a in enumerate(rnd):
            pos[a][p] += 1
    crossv = sorted({adj.get((a, b), 0) for a in range(n) for b in range(n) if a != b})
    selfv = sorted({adj.get((a, a), 0) for a in range(n)})
    posv = sorted({pos[a][p] for a in range(n) for p in range(n)})
    ok = (
        length % n == 0
        and len(crossv) == 1
        and len(selfv) == 1
        and len(posv) == 1
        and posv[0] == length // n
    )
    return ok, {
        "rounds": length,
        "cross_each": crossv[0] if len(crossv) == 1 else crossv,
        "self_each": selfv[0] if len(selfv) == 1 else selfv,
        "pos_each": posv[0] if len(posv) == 1 else posv,
    }


def cyclic_cycle(n):
    """#3468's rotation, kept ONLY as a control. Balances position, not adjacency."""
    return [tuple(list(range(n))[r % n:] + list(range(n))[: r % n]) for r in range(n)]

# `c2rs perf` prints, per fixture:
#   name.cpp    888B    10.343 ms    11.01 µs    939x    Match
# The PORT median is column 4 and the reference is column 3; only the port is
# read. `Match` is required — an arm that stopped matching is not a slower arm,
# it is a broken one, and its timing is meaningless.
ROW = re.compile(
    r"^\s*(?P<fixture>\S+\.cpp)\s+\S+\s+"
    r"(?P<ref>[\d.]+)\s*(?P<refunit>[mµun]?s)\s+"
    r"(?P<port>[\d.]+)\s*(?P<portunit>[mµun]?s)\s+"
    r"\S+\s+(?P<verdict>\w+)\s*$"
)

UNIT = {"s": 1e9, "ms": 1e6, "µs": 1e3, "us": 1e3, "ns": 1.0}


def run_arm(binary, port_iters, fixtures):
    """One `c2rs perf` run. Returns {fixture: port_ns} for MATCHING fixtures."""
    cmd = [binary, "perf", "--port-iters", str(port_iters)]
    if fixtures:
        cmd += ["--fixtures", ",".join(fixtures)]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        raise SystemExit(f"{binary} perf exited {p.returncode}\n{p.stdout}\n{p.stderr}")
    if "SKIP: toolchain absent" in p.stdout:
        raise SystemExit("SKIP: toolchain absent — the cost protocol needs the oracle")
    out = {}
    for line in p.stdout.splitlines():
        m = ROW.match(line)
        if not m:
            continue
        if m.group("verdict") != "Match":
            continue
        out[m.group("fixture")] = float(m.group("port")) * UNIT[m.group("portunit")]
    if not out:
        raise SystemExit(f"{binary} perf produced no Match rows — refusing to report 0 of 0")
    return out


def ci95(xs):
    """Normal-approximation 95 % CI of the mean. n is 100+ here."""
    if len(xs) < 2:
        return (float("nan"), float("nan"))
    m = statistics.fmean(xs)
    h = 1.96 * statistics.stdev(xs) / (len(xs) ** 0.5)
    return (m - h, m + h)


def show_design(n, rotation):
    """Print the rotation for `n` arms and the counts that certify it.

    The point is that the certificate is re-derived by `verify_design` from the
    flat sequence, so this output is a check and not a restatement of intent.
    Running it on `--rotation cyclic` prints the defect: unequal pair counts.
    """
    try:
        cycle = carryover_cycle(n) if rotation == "balanced" else cyclic_cycle(n)
    except DesignSearchExhausted as exc:
        raise SystemExit(f"REFUSING: {exc}")
    ok, report = verify_design(cycle, n)
    letters = [chr(ord("A") + i) for i in range(n)] if n <= 26 else \
        [str(i) for i in range(n)]
    print(f"{rotation} rotation, {n} arms, cycle = {len(cycle)} rounds")
    for r, rnd in enumerate(cycle):
        print(f"  round {r + 1:>3}  {' '.join(letters[i] for i in rnd)}")
    seq = [a for rnd in cycle for a in rnd]
    adj = {}
    for t in range(len(seq)):
        k = (seq[t], seq[(t + 1) % len(seq)])
        adj[k] = adj.get(k, 0) + 1
    print("\n  adjacency counts over one cycle (circular; rows = predecessor)")
    print("      " + " ".join(f"{c:>3}" for c in letters))
    for a in range(n):
        print(f"    {letters[a]} " + " ".join(f"{adj.get((a, b), 0):>3}" for b in range(n)))
    pos = [[0] * n for _ in range(n)]
    for rnd in cycle:
        for p, a in enumerate(rnd):
            pos[a][p] += 1
    print("\n  position counts (rows = arm, cols = slot in round)")
    for a in range(n):
        print(f"    {letters[a]} " + " ".join(f"{pos[a][p]:>3}" for p in range(n)))
    print(f"\n  VERIFY: {'BALANCED' if ok else 'NOT BALANCED'} — {report}")
    print(f"  legal --rounds: positive multiples of {len(cycle)}")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--arm", action="append", metavar="NAME=PATH",
                    help="an arm; the FIRST is the baseline every other is paired against")
    ap.add_argument("--rounds", type=int, default=12,
                    help="must be a multiple of TWICE the arm count; see the module doc")
    ap.add_argument("--rotation", choices=("balanced", "cyclic"), default="balanced",
                    help="balanced (default) = carryover-balanced, #3521; "
                         "cyclic = #3468's position-only rotation, a CONTROL ONLY")
    ap.add_argument("--show-design", type=int, metavar="N", default=None,
                    help="print the rotation for N arms with its balance "
                         "certificate and exit; runs nothing")
    ap.add_argument("--allow-unbalanced", action="store_true",
                    help="run an incomplete rotation anyway, and print what it costs")
    ap.add_argument("--port-iters", type=int, default=2000)
    ap.add_argument("--fixtures", default="",
                    help="comma-separated; default is every fixture c2rs perf knows")
    ap.add_argument("--null-arm", default="nulldup",
                    help="the arm asserted byte-identical to the baseline")
    args = ap.parse_args()

    if args.show_design is not None:
        return show_design(args.show_design, args.rotation)
    if not args.arm:
        raise SystemExit("--arm is required (or use --show-design N)")

    arms = []
    for spec in args.arm:
        if "=" not in spec:
            raise SystemExit(f"--arm wants NAME=PATH, got {spec!r}")
        name, path = spec.split("=", 1)
        if not os.path.isfile(path):
            raise SystemExit(f"arm {name}: no such binary {path}")
        arms.append((name, path))
    if len(arms) < 2:
        raise SystemExit("at least two arms (a baseline and one other)")
    base_name, base_path = arms[0]

    # THE NULL ARM'S OWN PRECONDITION, checked rather than trusted. A "null" arm
    # that is not byte-identical is not a noise floor, and the whole reading of
    # every other arm rests on it.
    null = [p for n, p in arms if n == args.null_arm]
    if null:
        if not filecmp.cmp(base_path, null[0], shallow=False):
            raise SystemExit(
                f"arm {args.null_arm} is NOT byte-identical to {base_name}: it cannot "
                f"be the null. Copy the baseline binary, do not rebuild it."
            )
        print(f"null arm {args.null_arm} verified byte-identical to {base_name}")
    else:
        print(f"WARNING: no arm named {args.null_arm!r} — this run has NO noise floor, "
              f"and any number it prints is a mean without a scale")

    # THE ROTATION MUST COMPLETE **AND** IT MUST BALANCE ADJACENCY. #3468's
    # criterion was `rounds % n == 0` — completing a cyclic rotation. #3495
    # showed that is necessary and not sufficient: on the completed cyclic
    # rotation a byte-identical arm still read +0.46 % with a 71 % sign split,
    # because `nulldup` followed `base` twice per cycle and never preceded it.
    # The cycle is now 2n rounds long, so the legal counts are multiples of 2n.
    n_arms = len(arms)
    try:
        cycle = carryover_cycle(n_arms) if args.rotation == "balanced" \
            else cyclic_cycle(n_arms)
    except DesignSearchExhausted as exc:
        raise SystemExit(
            f"REFUSING: {exc}. This script will not fall back to an unbalanced "
            f"rotation — a silently degraded rotation is the exact defect #3495 "
            f"filed. Run with fewer arms, or extend the design search."
        )
    ok, report = verify_design(cycle, n_arms)
    if not ok and args.rotation == "balanced":
        # The generator is never trusted: the verifier re-derives the counts
        # from the flat sequence, and a disagreement stops the run. (For
        # `cyclic` the verifier is EXPECTED to say NOT BALANCED — that is the
        # control's whole point, and it is printed rather than enforced.)
        raise SystemExit(
            f"REFUSING: the balanced rotation for {n_arms} arms does NOT "
            f"verify: {report}. The generator and the verifier disagree; nothing "
            f"is measured until they do."
        )
    cycle_len = len(cycle)
    if args.rotation == "cyclic":
        print("WARNING (--rotation cyclic): this is #3468's POSITION-ONLY "
              "rotation, proven by #3495 to leave adjacency unbalanced. It is a "
              "control, not a measurement. Its numbers are NOT comparable to a "
              "balanced run's.")
        print(f"  cyclic rotation certificate: {report} "
              f"(cross_each/self_each unequal is the defect, not a bug here)")
    else:
        print(f"rotation: carryover-balanced, cycle = {cycle_len} rounds, "
              f"verified — each arm in each slot {report['pos_each']}x, each "
              f"ordered pair a->b {report['cross_each']}x, each a->a "
              f"{report['self_each']}x per cycle")
    if args.rounds <= 0 or args.rounds % cycle_len != 0:
        near = max(1, round(args.rounds / cycle_len)) * cycle_len
        msg = (f"--rounds {args.rounds} is not a positive multiple of "
               f"{cycle_len} (the {args.rotation} cycle for {n_arms} arms): the "
               f"rotation does not complete, so arm order aliases onto the arm "
               f"axis and the null arm will read a nonzero effect. Use {near}.")
        if not args.allow_unbalanced:
            raise SystemExit(msg)
        print(f"WARNING (--allow-unbalanced): {msg}")

    fixtures = [f for f in args.fixtures.split(",") if f.strip()]
    try:
        load = os.getloadavg()
    except OSError:
        load = None
    print(f"rounds={args.rounds} port_iters={args.port_iters} "
          f"load_at_start={load[0]:.1f}" if load else f"rounds={args.rounds}")

    # per arm, per fixture: the minimum port median over rounds
    best = {name: {} for name, _ in arms}
    for r in range(args.rounds):
        # ROTATE: round r takes its slot order from the verified cycle.
        order = [arms[i] for i in cycle[r % cycle_len]]
        for name, path in order:
            got = run_arm(path, args.port_iters, fixtures)
            for f, ns in got.items():
                cur = best[name].get(f)
                if cur is None or ns < cur:
                    best[name][f] = ns
        print(f"  round {r + 1}/{args.rounds} done "
              f"({' '.join(n for n, _ in order)})", flush=True)

    try:
        load_end = os.getloadavg()[0]
    except OSError:
        load_end = float("nan")

    # THE POPULATION: fixtures every arm matched in every round. Stated as a
    # denominator, because an arm-dependent population is an arm-dependent
    # measurement.
    common = set(best[base_name])
    for name, _ in arms[1:]:
        common &= set(best[name])
    common = sorted(common)
    print(f"\npopulation: n = {len(common)} fixtures matched by every arm "
          f"(baseline saw {len(best[base_name])})")
    if not common:
        raise SystemExit("no fixture matched in every arm — nothing to pair")

    print(f"load at end: {load_end:.1f}")
    print(f"\n{'pair':<12} {'mean':>9} {'95% CI':>22} {'median':>9} "
          f"{'aggregate':>10} {'slower on':>16}")
    for name, _ in arms[1:]:
        ratios = [best[name][f] / best[base_name][f] - 1.0 for f in common]
        pcts = [x * 100.0 for x in ratios]
        lo, hi = ci95(pcts)
        agg = (sum(best[name][f] for f in common)
               / sum(best[base_name][f] for f in common) - 1.0) * 100.0
        slower = sum(1 for x in ratios if x > 0)
        tag = f"{name}{' (NULL)' if name == args.null_arm else ''}"
        print(f"{tag:<12} {statistics.fmean(pcts):>+8.2f}% "
              f"[{lo:>+7.2f}, {hi:>+7.2f}] {statistics.median(pcts):>+8.2f}% "
              f"{agg:>+9.2f}% {slower:>7} of {len(common)} "
              f"({100.0 * slower / len(common):.0f}%)")

    # The five costliest fixtures per non-baseline arm, published rather than
    # smoothed into the mean — and with the caveat that a fixture whose base is
    # a few microseconds is dominated by absolute jitter.
    for name, _ in arms[1:]:
        rows = sorted(
            ((best[name][f] / best[base_name][f] - 1.0) * 100.0, f, best[base_name][f])
            for f in common
        )[-5:][::-1]
        print(f"\n  {name}: five costliest fixtures")
        for pct, f, b in rows:
            print(f"    {f:<34} {pct:>+7.1f}%   base {b / 1000.0:8.2f} µs"
                  f"{'   <- base under 8 µs; jitter dominates' if b < 8000 else ''}")


if __name__ == "__main__":
    sys.exit(main())

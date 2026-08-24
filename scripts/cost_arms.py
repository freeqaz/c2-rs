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
* **Arm order rotates every round, and `--rounds` MUST be a multiple of the
  arm count.** A fixed order lets a warming or cooling box alias onto the arm
  axis; a rotation that does not complete does **not** remove it. This is not a
  theoretical nicety — it is what four lanes of this protocol were losing.
  Measured here 2026-08-24 on one tree, one pair of binaries, minutes apart:

  ```text
    rounds=8  (3 arms — UNBALANCED)   null +0.57 % [+0.45, +0.68]  split 76 %
    rounds=9  (3 arms — balanced)     null +0.09 % [-0.07, +0.26]  split 51 %
  ```

  At 8 rounds over 3 arms, `base` and `nulldup` each run first three times and
  `tip` only twice, while `tip` runs last three times and `nulldup` twice. On a
  box whose load is moving, that residue lands on the arm axis, and the NULL arm
  — byte-identical to the baseline — reads a **significantly positive effect
  with a CI excluding zero**. `w-s1bc` §4.3 and `w-s1c2` §4.1/§4.3 both reported
  exactly that symptom (null arms at −1.08 %, +0.47 %, +0.52 %, sign splits of
  40 % and 52 %) and read it as box noise setting a ±1–1.7 % floor. **On the
  balanced rotation the floor is not there**: the same protocol resolves a
  +0.99 % effect with a CI of [+0.63, +1.35]. The script now refuses an
  unbalanced `--rounds`.
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
import os
import re
import statistics
import subprocess
import sys

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


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--arm", action="append", required=True, metavar="NAME=PATH",
                    help="an arm; the FIRST is the baseline every other is paired against")
    ap.add_argument("--rounds", type=int, default=9,
                    help="must be a multiple of the arm count; see the module doc")
    ap.add_argument("--allow-unbalanced", action="store_true",
                    help="run an incomplete rotation anyway, and print what it costs")
    ap.add_argument("--port-iters", type=int, default=2000)
    ap.add_argument("--fixtures", default="",
                    help="comma-separated; default is every fixture c2rs perf knows")
    ap.add_argument("--null-arm", default="nulldup",
                    help="the arm asserted byte-identical to the baseline")
    args = ap.parse_args()

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

    # THE ROTATION MUST COMPLETE. See the module doc: at 8 rounds over 3 arms
    # the null arm reads +0.57 % [+0.45, +0.68] on a binary byte-identical to
    # the baseline, and at 9 it reads +0.09 % [-0.07, +0.26]. An arm that ran
    # last more often than it ran first is an arm with a handicap, and no amount
    # of pairing removes it.
    if args.rounds % len(arms) != 0:
        msg = (f"--rounds {args.rounds} is not a multiple of {len(arms)} arms: the "
               f"rotation does not complete, so arm ORDER aliases onto the arm axis "
               f"and the null arm will read a nonzero effect. Use "
               f"{len(arms) * max(1, round(args.rounds / len(arms)))}.")
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
        # ROTATE: round r starts at arm r % len(arms).
        order = arms[r % len(arms):] + arms[: r % len(arms)]
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

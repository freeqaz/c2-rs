#!/usr/bin/env python3
"""THE SPEEDUP-ROW PROTOCOL — board **#3609**, for `#3583`.

`scripts/cost_arms.py` prices the **port** across two trees and deliberately
throws the reference column away (*"including it would put a 1000x-slower
number in the denominator"*). That is right for a construct rung's cost clause
and **wrong for the number `docs/STATUS.md` publishes**, which is

    speedup(fixture) = ref_median / port_median
    geomean          = exp(mean(ln speedup))   over MATCHED fixtures only

— a ratio whose **numerator is the reference**. `#3583` filed a 664x -> 553x
step and named three candidates (workload / `repo_root()` / box state) without
anyone having split the ratio into its two sides, so no candidate could be
tested: a move in the port and a move in the reference are indistinguishable in
the published digit and have completely different causes.

This script is `cost_arms.py` for the **published** metric. It **imports** that
module rather than restating it — `#3451` is the record of one protocol being
retyped four times, and the rotation, the toolchain pin, the per-arm preflight
and the arm-identity block are exactly the same clauses here. What it adds is
three columns instead of one:

    geomean speedup   (the published number)
    geomean ref_ns    (wibo process spawn; ref_iters = 5 samples per fixture)
    geomean port_ns   (in-process; port_iters = 2000 samples per fixture)

and it reports each of them **per round** as well as pooled, because the
round-to-round spread of one arm IS the metric's noise floor — the thing the
row has never been published with. `#3551` measured a floor for the *port*
under build-directory variation (0.93 %); that is a different quantity from the
spread of a ratio whose numerator is a process spawn, and it must not be quoted
as this row's floor.

    scripts/perf_arms.py --arm pre=work/w-perfstep/arm1/target/release/c2rs \\
                         --arm post=work/w-perfstep/arm2/target/release/c2rs \\
                         --arm postdup=work/w-perfstep/dup1/c2rs \\
                         --rounds 12

and, for the one candidate that can be isolated as a SINGLE variable rather
than across 175 commits (`#3611`):

    scripts/perf_arms.py --repo-root-ab work/w-perfstep/arm2/target/release/c2rs \\
                         --rounds 6

which builds its own `walk` / `pinn` / `wdup` arms around ONE binary, differing
only in whether `C2RS_REPO_ROOT` is exported. **Nothing absolute is baked into
this file**: the repo root is derived from `__file__` at runtime and the binary
comes from `argv`.

# What it inherits from `cost_arms.py`, unchanged

* **The carryover-balanced rotation** (`#3521`) and its certificate, verified
  from the flat sequence at startup. `--rounds` must be a multiple of `2n`.
* **The toolchain pin** (`pin_toolchain`, `#3575`). An arm built before
  `w-hygiene`'s runtime `repo_root()` resolves `compilers/` against its own
  build tree, prints `SKIP: toolchain absent` and **exits 0** (`#3470`); an
  explicit `C2RS_COMPILERS`/`C2RS_WIBO` is the only thing that reaches it, and
  this protocol's whole point is to run such an arm.
* **The per-arm preflight with a denominator** (`preflight_arm`). Only a
  denominator catches an absence (`#3470`, `#1002`).
* **The arm-identity block** (`arm_identity`): md5, size, build directory. A
  sha is not an arm (`#3525`).
* **The null arm's precondition**, checked with `cmp` and not assumed.

# What it does NOT inherit, and why

* **The estimator.** `cost_arms.py` takes the per-fixture minimum over rounds.
  That is right for the port, where interference only ever makes a run slower.
  It is kept here for the arm-vs-arm ladder, but the **headline** is the
  per-round geomean, unminimised — because the published number is a single
  run, and a minimum over rounds is a quantity `status.sh` never computes. Both
  are printed; they answer different questions and the rung says which.
* **The claim that a clean null makes two runs comparable.** It does not
  (`#3523`), and this script cannot fix that either. It prints the per-round
  series so a reader can see the drift instead of being handed a mean.

# What it CANNOT do

It measures **this box, now**, over the fixture population the arms share. It
is a timing instrument, not a grader — `#1406`'s tension, recorded in
`cost_arms.py` and inherited whole. Its `--self-test` is pure arithmetic and
parsing and does run under `scripts/gate.sh`; the timing half cannot.

Std-lib only.
"""

import argparse
import atexit
import filecmp
import math
import os
import shutil
import statistics
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import cost_arms  # noqa: E402  (path insert must precede the import)


def geomean(xs):
    """Geometric mean of positive values; None if empty. Mirrors `perf.rs`.

    `crates/c2-harness/src/perf.rs::geomean` ignores non-positive entries and
    returns `None` on an empty set. Same rule here so the number this prints is
    the number the harness prints, not a near-miss of it.
    """
    pos = [x for x in xs if x > 0.0]
    if not pos:
        return None
    return math.exp(sum(math.log(x) for x in pos) / len(pos))


def run_arm_both(binary, port_iters, fixtures):
    """One `c2rs perf` run -> {fixture: (ref_ns, port_ns)} for MATCHING rows.

    The `Match` filter is `cost_arms.run_arm`'s and is kept: an arm that stopped
    matching is not a slower arm, it is a broken one.
    """
    cmd = [binary, "perf", "--port-iters", str(port_iters)]
    if fixtures:
        cmd += ["--fixtures", ",".join(fixtures)]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        raise SystemExit(f"{binary} perf exited {p.returncode}\n{p.stdout}\n{p.stderr}")
    if "SKIP: toolchain absent" in p.stdout:
        raise SystemExit(
            f"{binary} printed 'SKIP: toolchain absent' MID-RUN, having passed "
            f"preflight — the toolchain moved underneath the run (#3470)"
        )
    out = {}
    for line in p.stdout.splitlines():
        m = cost_arms.ROW.match(line)
        if not m or m.group("verdict") != "Match":
            continue
        out[m.group("fixture")] = (
            float(m.group("ref")) * cost_arms.UNIT[m.group("refunit")],
            float(m.group("port")) * cost_arms.UNIT[m.group("portunit")],
        )
    if not out:
        raise SystemExit(f"{binary} perf produced no Match rows — refusing to report 0 of 0")
    return out


def repo_root_ab_arms(binary):
    """Build the three `repo_root()` A/B arms at runtime. Board **#3611**.

    ---- what is being isolated ------------------------------------------------

    `w-hygiene` made `c2-reference`'s `repo_root()` resolve at RUNTIME where it
    was `env!("CARGO_MANIFEST_DIR")` (**#3470**), and `#3583` named that as a
    candidate for a 16.7 % move in the published speedup — *"a per-call cost on
    a possibly-hot path"*. Comparing the two published TREES cannot isolate it:
    they are 175 commits apart. **`C2RS_REPO_ROOT` can.** It short-circuits
    `repo_root()` at its first line, so ONE binary run with and without it
    differs in exactly one thing: whether the ancestor walk executes.

    Three arms, because a two-arm run has no noise floor:

      * `walk` — `C2RS_REPO_ROOT` unset; the walk runs (19 `statx`, once).
      * `pinn` — `C2RS_REPO_ROOT` exported; the walk is skipped entirely.
      * `wdup` — a byte-identical COPY of `walk`; the NULL.

    ---- why this lives in the script and not in three files under `work/` -----

    It was three committed shell scripts until the coordinator pointed out that
    `work/` is `.gitignore` line 24 and they had been force-added past it
    (**#3156**). They are METHOD, not scratch — they are how the A/B is
    reproduced — so they belong on a tracked surface. Generating them here is
    strictly better than moving them to `scripts/`: there is one locator for the
    procedure instead of four, and the wrappers cannot drift from the runner
    that consumes them.

    **No absolute path is written into this file.** `C2RS_REPO_ROOT` is derived
    from `__file__` at runtime and the binary comes from `argv` — which is the
    same property `#3611` measured as costing +0.00 % [-0.12, +0.12].

    Returns [(name, path), ...] with `walk` first (the baseline).
    """
    binary = os.path.abspath(binary)
    if not os.path.isfile(binary):
        raise SystemExit(f"--repo-root-ab: no such binary {binary}")
    root = cost_arms.repo_root_of_this_script()
    tmp = tempfile.mkdtemp(prefix="c2rs-reporoot-ab-")
    atexit.register(shutil.rmtree, tmp, True)

    walk = os.path.join(tmp, "walk")
    with open(walk, "w") as fh:
        fh.write(
            "#!/bin/sh\n"
            "# ARM `walk` — C2RS_REPO_ROOT UNSET, so repo_root() performs its\n"
            "# runtime ancestor walk. Generated by scripts/perf_arms.py.\n"
            "unset C2RS_REPO_ROOT\n"
            f'exec "{binary}" "$@"\n'
        )
    pinn = os.path.join(tmp, "pinn")
    with open(pinn, "w") as fh:
        fh.write(
            "#!/bin/sh\n"
            "# ARM `pinn` — the SAME binary with C2RS_REPO_ROOT SET, which\n"
            "# short-circuits repo_root() at its first line and skips the walk.\n"
            f'C2RS_REPO_ROOT="{root}"\n'
            "export C2RS_REPO_ROOT\n"
            f'exec "{binary}" "$@"\n'
        )
    wdup = os.path.join(tmp, "wdup")
    # A COPY, never a second write: `cost_arms.py`'s null-arm rule is that a
    # null which was regenerated is not a null. `filecmp` checks it downstream.
    shutil.copyfile(walk, wdup)
    for p in (walk, pinn, wdup):
        os.chmod(p, 0o755)
    print(f"repo_root A/B arms generated in {tmp}")
    print(f"  walk  C2RS_REPO_ROOT unset      -> repo_root() walks")
    print(f"  pinn  C2RS_REPO_ROOT={root}")
    print(f"  wdup  byte-identical copy of walk (the NULL)")
    return [("walk", walk), ("pinn", pinn), ("wdup", wdup)]


def spread_pct(xs):
    """max/min - 1, in percent. The plain statement of a series' range."""
    lo, hi = min(xs), max(xs)
    return (hi / lo - 1.0) * 100.0 if lo > 0 else float("nan")


def self_test():
    """Watch each control go RED before any green is quoted (`#3336`, `#1236`).

    Three reds and two greens, all on fabricated `c2rs perf` text — the parse
    and the geomean are the only things this file owns that can be wrong
    without a box.
    """
    bad = 0
    print("perf_arms self-test — every control watched failing before its green:")

    # --- geomean: the green, then the two reds ------------------------------
    g = geomean([1.0, 100.0])
    if g is not None and abs(g - 10.0) < 1e-9:
        print("  geomean(1,100) = 10                         -> ok (green)")
    else:
        print(f"  geomean(1,100) = {g}, expected 10           *** FAIL ***")
        bad += 1
    if geomean([]) is None:
        print("  geomean([]) is None                         -> REFUSED (red)")
    else:
        print("  geomean([]) returned a value                *** FAIL ***")
        bad += 1
    g = geomean([0.0, 4.0, 4.0])
    if g is not None and abs(g - 4.0) < 1e-9:
        print("  geomean ignores non-positive entries        -> ok (matches perf.rs)")
    else:
        print(f"  geomean([0,4,4]) = {g}, expected 4          *** FAIL ***")
        bad += 1

    # --- the ROW parse: a Match row is read, a non-Match row is NOT ----------
    #
    # THE RED THAT MATTERS. The published geomean is over MATCHED fixtures; a
    # parser that let a `NotImplemented` row through would silently change the
    # population, which is `GAPS.md` 1's trap arriving through the tool.
    rows = [
        "  ok.cpp      888B    10.343 ms    11.01 us    939x    Match",
        "  ni.cpp      888B    10.343 ms     0.00 us      -     NotImplemented",
        "  bad.cpp     888B    10.343 ms    11.01 us    939x    Mismatch",
        "this is not a row at all",
    ]
    kept = {}
    for line in rows:
        m = cost_arms.ROW.match(line)
        if m and m.group("verdict") == "Match":
            kept[m.group("fixture")] = (
                float(m.group("ref")) * cost_arms.UNIT[m.group("refunit")],
                float(m.group("port")) * cost_arms.UNIT[m.group("portunit")],
            )
    if list(kept) == ["ok.cpp"]:
        print("  parse keeps Match, drops NotImplemented/Mismatch/garbage -> ok")
    else:
        print(f"  parse kept {sorted(kept)}, expected ['ok.cpp'] *** FAIL ***")
        bad += 1
    if kept:
        ref_ns, port_ns = kept["ok.cpp"]
        # 10.343 ms / 11.01 us = 939.4x — the row's own printed speedup, so the
        # units table is checked against the harness's own arithmetic.
        got = ref_ns / port_ns
        if abs(got - 939.4) < 1.0:
            print(f"  units: 10.343 ms / 11.01 us = {got:.1f}x  -> ok "
                  f"(the row's own 939x)")
        else:
            print(f"  units: got {got:.1f}x, expected ~939x     *** FAIL ***")
            bad += 1

    # --- spread_pct ---------------------------------------------------------
    s = spread_pct([100.0, 110.0, 105.0])
    if abs(s - 10.0) < 1e-9:
        print("  spread_pct([100,110,105]) = 10.0%           -> ok")
    else:
        print(f"  spread_pct = {s}, expected 10.0             *** FAIL ***")
        bad += 1

    # --- the repo_root A/B generator: one red, then the greens ---------------
    #
    # The null is the whole reading of the other two arms, so "wdup is a COPY of
    # walk" and "pinn differs from walk" are both checked here rather than
    # trusted. A generator that wrote wdup a second time would still be
    # byte-identical today and would stop being so the moment a timestamp or a
    # tmpdir name entered the template — which is why `cost_arms.py` words its
    # rule as "copy the baseline binary, do not rebuild it".
    try:
        repo_root_ab_arms(os.path.join(os.sep, "nonexistent", "c2rs"))
        print("  repo-root-ab   *** ACCEPTED A MISSING BINARY ***")
        bad += 1
    except SystemExit:
        print("  repo-root-ab   -> REFUSED a missing binary (red)")
    generated = repo_root_ab_arms(sys.executable)  # any real executable will do
    names = [n for n, _ in generated]
    paths = dict(generated)
    if names != ["walk", "pinn", "wdup"]:
        print(f"  repo-root-ab   *** arm names {names} ***")
        bad += 1
    elif not filecmp.cmp(paths["walk"], paths["wdup"], shallow=False):
        print("  repo-root-ab   *** wdup is NOT byte-identical to walk ***")
        bad += 1
    elif filecmp.cmp(paths["walk"], paths["pinn"], shallow=False):
        print("  repo-root-ab   *** pinn is identical to walk — the A/B varies "
              "NOTHING and would read 0 by construction ***")
        bad += 1
    else:
        print("  repo-root-ab   -> walk/pinn/wdup: null is a copy, pinn differs "
              "(green)")
    # And the arms must carry no path this repo did not hand them.
    with open(paths["pinn"]) as fh:
        body = fh.read()
    if cost_arms.repo_root_of_this_script() in body and "C2RS_REPO_ROOT" in body:
        print("  repo-root-ab   -> pinn exports the RUNTIME-derived repo root "
              "(green; nothing absolute is baked in this file)")
    else:
        print("  repo-root-ab   *** pinn does not export a derived repo root ***")
        bad += 1

    print(f"perf_arms self-test: {'PASS' if bad == 0 else f'FAIL ({bad})'}")
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--arm", action="append", metavar="NAME=PATH",
                    help="an arm; the FIRST is the baseline every other is paired against")
    ap.add_argument("--rounds", type=int, default=6,
                    help="must be a multiple of TWICE the arm count (cost_arms.py's cycle)")
    ap.add_argument("--port-iters", type=int, default=2000,
                    help="the harness default, which is what status.sh collects at")
    ap.add_argument("--fixtures", default="",
                    help="comma-separated; default is every fixture c2rs perf knows")
    ap.add_argument("--null-arm", default="postdup",
                    help="the arm asserted byte-identical to the baseline")
    ap.add_argument("--repo-root-ab", metavar="BINARY", default=None,
                    help="isolate w-hygiene's runtime repo_root() as a SINGLE "
                         "variable: generate walk/pinn/wdup arms around ONE "
                         "binary, differing only in whether C2RS_REPO_ROOT is "
                         "exported. Implies --null-arm wdup. Board #3611")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()

    if args.self_test:
        return self_test()
    if args.repo_root_ab:
        if args.arm:
            raise SystemExit("--repo-root-ab builds its own arms; drop --arm")
        arms = repo_root_ab_arms(args.repo_root_ab)
        args.null_arm = "wdup"
    else:
        if not args.arm:
            raise SystemExit("--arm is required (or --repo-root-ab, or --self-test)")
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
    fixtures = [f for f in args.fixtures.split(",") if f.strip()]

    comp, wibo, how = cost_arms.pin_toolchain()
    print(f"toolchain {how}: C2RS_COMPILERS={comp}")
    print(f"                 C2RS_WIBO={wibo}")
    print("preflight — each arm proves it can grade before anything is timed:")
    for name, path in arms:
        n = cost_arms.preflight_arm(name, path, fixtures)
        print(f"  {name:<10} graded {n} Match fixtures at --port-iters 1")

    print("arm identity (md5 / size / build dir) — a sha is NOT an arm (#3525):")
    for name, path in arms:
        digest, size, where = cost_arms.arm_identity(path)
        print(f"  {name:<10} {digest}  {size:>10,} B  {where}")

    null = [p for n, p in arms if n == args.null_arm]
    if null:
        if not filecmp.cmp(base_path, null[0], shallow=False):
            raise SystemExit(
                f"arm {args.null_arm} is NOT byte-identical to {base_name}: it "
                f"cannot be the null. Copy the baseline binary, do not rebuild it."
            )
        print(f"null arm {args.null_arm} verified byte-identical to {base_name}")
    else:
        print(f"WARNING: no arm named {args.null_arm!r} — this run has NO noise floor")

    n_arms = len(arms)
    cycle = cost_arms.carryover_cycle(n_arms)
    ok, report = cost_arms.verify_design(cycle, n_arms)
    if not ok:
        raise SystemExit(f"REFUSING: rotation does not verify: {report}")
    print(f"rotation: carryover-balanced, cycle = {len(cycle)} rounds, verified — {report}")
    if args.rounds <= 0 or args.rounds % len(cycle) != 0:
        raise SystemExit(
            f"--rounds {args.rounds} is not a positive multiple of {len(cycle)}; "
            f"the rotation does not complete and arm order aliases onto the arm axis"
        )

    try:
        load0 = os.getloadavg()[0]
    except OSError:
        load0 = float("nan")
    print(f"rounds={args.rounds} port_iters={args.port_iters} load_at_start={load0:.1f}")

    # per arm: list of per-round {fixture: (ref, port)}
    series = {name: [] for name, _ in arms}
    for r in range(args.rounds):
        order = [arms[i] for i in cycle[r % len(cycle)]]
        for name, path in order:
            series[name].append(run_arm_both(path, args.port_iters, fixtures))
        print(f"  round {r + 1}/{args.rounds} done ({' '.join(n for n, _ in order)})",
              flush=True)

    try:
        load1 = os.getloadavg()[0]
    except OSError:
        load1 = float("nan")

    # THE DENOMINATOR: fixtures every arm matched in every round.
    common = None
    for name, _ in arms:
        for run in series[name]:
            common = set(run) if common is None else (common & set(run))
    common = sorted(common or [])
    seen = {name: len(series[name][0]) for name, _ in arms}
    print(f"\npopulation: n = {len(common)} fixtures matched by EVERY arm in EVERY "
          f"round (per-arm first-round counts: "
          f"{', '.join(f'{k} {v}' for k, v in seen.items())})")
    if not common:
        raise SystemExit("no fixture matched in every arm and round — nothing to pair")
    print(f"load at end: {load1:.1f}")

    # ---- THE HEADLINE: per-round geomeans, unminimised ---------------------
    #
    # This is the quantity `status.sh` publishes: one run, no minimum over
    # rounds. Its round-to-round spread is the noise floor the row has never
    # carried.
    print(f"\nPER-ROUND geomeans over the common {len(common)} fixtures "
          f"(this is what status.sh collects: ONE run, no minimum)")
    print(f"{'arm':<10} {'round':>5} {'speedup':>10} {'ref_ns':>12} {'port_ns':>10}")
    per_round = {name: [] for name, _ in arms}
    for name, _ in arms:
        for i, run in enumerate(series[name]):
            sp = geomean([run[f][0] / run[f][1] for f in common])
            gr = geomean([run[f][0] for f in common])
            gp = geomean([run[f][1] for f in common])
            per_round[name].append((sp, gr, gp))
            print(f"{name:<10} {i + 1:>5} {sp:>9.1f}x {gr:>12,.0f} {gp:>10,.0f}")

    print(f"\nPER-ARM spread across rounds (max/min - 1) — THE NOISE FLOOR, in the "
          f"units the row is published in")
    print(f"{'arm':<10} {'speedup min':>12} {'max':>9} {'spread':>9} "
          f"{'ref spread':>11} {'port spread':>12}")
    for name, _ in arms:
        sps = [t[0] for t in per_round[name]]
        refs = [t[1] for t in per_round[name]]
        ports = [t[2] for t in per_round[name]]
        print(f"{name:<10} {min(sps):>11.1f}x {max(sps):>8.1f}x "
              f"{spread_pct(sps):>8.1f}% {spread_pct(refs):>10.1f}% "
              f"{spread_pct(ports):>11.1f}%")

    # ---- THE LADDER: per-fixture MINIMUM over rounds, arm vs baseline -------
    #
    # `cost_arms.py`'s estimator, applied to both sides separately. Least
    # contaminated, and the right one for "did the TREE move the number".
    print(f"\nMIN-OVER-ROUNDS estimator (cost_arms.py's), each side separately")
    print(f"{'arm':<10} {'speedup':>10} {'vs base':>9} {'ref_ns':>12} {'vs base':>9} "
          f"{'port_ns':>10} {'vs base':>9}")
    best = {}
    for name, _ in arms:
        b = {}
        for run in series[name]:
            for f, (rf, pt) in run.items():
                cur = b.get(f)
                b[f] = (min(cur[0], rf), min(cur[1], pt)) if cur else (rf, pt)
        best[name] = b
    b_sp = geomean([best[base_name][f][0] / best[base_name][f][1] for f in common])
    b_rf = geomean([best[base_name][f][0] for f in common])
    b_pt = geomean([best[base_name][f][1] for f in common])
    for name, _ in arms:
        sp = geomean([best[name][f][0] / best[name][f][1] for f in common])
        rf = geomean([best[name][f][0] for f in common])
        pt = geomean([best[name][f][1] for f in common])
        tag = f"{name}{' (NULL)' if name == args.null_arm else ''}"
        print(f"{tag:<10} {sp:>9.1f}x {100 * (sp / b_sp - 1):>+8.2f}% "
              f"{rf:>12,.0f} {100 * (rf / b_rf - 1):>+8.2f}% "
              f"{pt:>10,.0f} {100 * (pt / b_pt - 1):>+8.2f}%")

    # ---- PAIRED, per fixture: the CI and the sign split --------------------
    #
    # `cost_arms.py`'s reading rule: a mean without a sign split hides the case
    # that says the run could not answer its own question.
    print(f"\nPAIRED per-fixture vs {base_name} (min-over-rounds), port side and "
          f"ref side separately")
    print(f"{'arm':<10} {'side':<6} {'mean':>9} {'95% CI':>22} {'median':>9} "
          f"{'slower on':>16}")
    for name, _ in arms[1:]:
        for side, idx in (("port", 1), ("ref", 0)):
            pcts = [(best[name][f][idx] / best[base_name][f][idx] - 1.0) * 100.0
                    for f in common]
            lo, hi = cost_arms.ci95(pcts)
            up = sum(1 for x in pcts if x > 0)
            print(f"{name:<10} {side:<6} {statistics.fmean(pcts):>+8.2f}% "
                  f"[{lo:>+7.2f}, {hi:>+7.2f}] {statistics.median(pcts):>+8.2f}% "
                  f"{up:>7} of {len(common)} ({100.0 * up / len(common):.0f}%)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

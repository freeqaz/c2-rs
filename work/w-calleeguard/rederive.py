#!/usr/bin/env python3
"""rederive.py — DERIVE the results table from the raw logs. Never accumulate it.

`docs/rungs/README.md` § "Two rules a probe must satisfy", rule 2: all three of
`w-mutcensus`' colour-rule corrections reapplied retroactively to every run
already on disk *because* its table was derived. A campaign that emits its
conclusions incrementally cannot correct its own classifier.

Classification, from the prereg §4.4:

  INVALID  if the `census_gate` target ran for < 1 s (a skipping differential is
           0.00 s; a grading one is tens of seconds — w-mutcensus D6 / #3219),
           or the target count is not the one EXPECTED for that run's tree.

The prereg registered the target rule as `targets != 43`, which was the count of
the tree phase R ran against. Landing A1 and A2 (deviations.md) adds two
integration-test targets, so the tip's count is 45. Rather than relax the rule
to "43 or 45" — which would accept a phase-R run that had silently gained two
targets and a tip run that had silently lost them — the expectation is stated
PER RUN, below, and a run whose id is not in the map is INVALID rather than
assumed. This is the classifier correction rule 2 exists for: it reapplies to
every log already on disk because the table is derived, never accumulated.

  RED      failed > 0.
  GREEN    failed == 0.

Usage: rederive.py [logdir]
"""
import re
import sys
import pathlib

LOGDIR = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else
                      pathlib.Path(__file__).parent / "logs")

# The target count each run is expected to report. 43 is the tree phase R ran
# against; 45 is the tip, after A1 (`callee_unresolved_sites`) and A2
# (`require_toolchain`) land. The two D6 runs are the deliberate no-toolchain
# demonstrations: their `census_gate` IS 0.00s, which is the point, so the
# duration rule is suspended for them and their colour is read from the failures.
EXPECTED_TARGETS = {
    "N0": 43, "N0R": 43, "R5": 43, "R6": 43, "R7": 43, "R8": 43,
    "N0T": 45, "G5": 45, "G6": 45, "G7": 45, "G8": 45,
    "C1a": 45, "C1b": 45, "N1": 45,
    "D6a.INVALID": 45, "D6b": 45,
}

RUNNING = re.compile(r"^\s+Running (\S+) \(")
DOCTEST = re.compile(r"^\s+Doc-tests (\S+)")
RESULT = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out; finished in ([\d.]+)s")
FAILLINE = re.compile(r"^\s{4}(\S+)$")


def parse(path):
    passed = failed = targets = 0
    cur = None
    durations = {}
    fails = []
    in_failures_block = False
    rc = wall = None
    for line in path.read_text(errors="replace").splitlines():
        m = RUNNING.match(line) or DOCTEST.match(line)
        if m:
            cur = m.group(1)
            in_failures_block = False
            continue
        if line.startswith("failures:"):
            in_failures_block = True
            continue
        if in_failures_block:
            m = FAILLINE.match(line)
            if m and not m.group(1).startswith("---"):
                fails.append(m.group(1))
                continue
            if line.strip() == "":
                continue
            in_failures_block = False
        m = RESULT.match(line)
        if m:
            targets += 1
            passed += int(m.group(2))
            failed += int(m.group(3))
            if cur:
                durations[cur] = float(m.group(7))
            in_failures_block = False
            continue
        if line.startswith("MUTANT-RC "):
            rc = int(line.split()[1])
        if line.startswith("MUTANT-WALL "):
            wall = int(line.split()[1])

    want = EXPECTED_TARGETS.get(path.stem)
    gate = max((v for k, v in durations.items() if "census_gate" in k), default=None)
    if want is None:
        colour = "INVALID(unregistered run id)"
    elif targets != want:
        colour = "INVALID(targets=%d != %d)" % (targets, want)
    elif gate is None:
        colour = "INVALID(no census_gate target)"
    elif gate < 1.0 and not path.stem.startswith("D6"):
        colour = "INVALID(census_gate %.2fs < 1s)" % gate
    else:
        colour = "RED" if failed else "GREEN"
    return dict(id=path.stem, passed=passed, failed=failed, targets=targets,
                gate=gate, colour=colour, rc=rc, wall=wall,
                fails=sorted(set(fails)))


def main():
    rows = [parse(p) for p in sorted(LOGDIR.glob("*.log"))]
    w = max((len(r["id"]) for r in rows), default=4)
    print(f"{'id':<{w}}  {'colour':<28} {'pass':>5}/{'fail':<4} {'tgts':>4} "
          f"{'census_gate':>11} {'wall':>6}  failing tests")
    for r in rows:
        g = "-" if r["gate"] is None else f"{r['gate']:.2f}s"
        print(f"{r['id']:<{w}}  {r['colour']:<28} {r['passed']:>5}/{r['failed']:<4} "
              f"{r['targets']:>4} {g:>11} {str(r['wall'])+'s':>6}  "
              + (", ".join(r["fails"]) if r["fails"] else "—"))


if __name__ == "__main__":
    main()

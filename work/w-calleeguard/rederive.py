#!/usr/bin/env python3
"""rederive.py — DERIVE the results table from the raw logs. Never accumulate it.

`docs/rungs/README.md` § "Two rules a probe must satisfy", rule 2: all three of
`w-mutcensus`' colour-rule corrections reapplied retroactively to every run
already on disk *because* its table was derived. A campaign that emits its
conclusions incrementally cannot correct its own classifier.

Classification, from the prereg §4.4:

  INVALID  if the `census_gate` target ran for < 1 s (a skipping differential is
           0.00 s; a grading one is tens of seconds — w-mutcensus D6 / #3219),
           or the target count is not 43.
  RED      failed > 0.
  GREEN    failed == 0.

Usage: rederive.py [logdir]
"""
import re
import sys
import pathlib

LOGDIR = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else
                      pathlib.Path(__file__).parent / "logs")

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

    gate = max((v for k, v in durations.items() if "census_gate" in k), default=None)
    if gate is None:
        colour = "INVALID(no census_gate target)"
    elif gate < 1.0:
        colour = "INVALID(census_gate %.2fs < 1s)" % gate
    elif targets != 43:
        colour = "INVALID(targets=%d != 43)" % targets
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

#!/usr/bin/env python3
"""`w-deadsites` — DERIVE the results table from the logs. Never accumulate it.

`docs/rungs/README.md` probe rule 2. Every number this lane publishes comes out
of `work/w-deadsites/logs/` by running this script, so a classifier correction
reapplies retroactively to every run already on disk.

    rederive.py            the whole table
"""

import os
import re
import sys

LOGS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "logs")

# The 26 open rows plus the 8 controls, in `w-mutcensus` id order.
OPEN = ["CS2", "CS3", "CS4", "CS9",
        "CA2", "CA6", "CA8", "CA9", "CA10", "CA13", "CA16", "CA18",
        "B2", "B3", "B4", "B5", "B6", "B7", "B8",
        "BU3", "D1", "D2", "G2", "L2", "L3", "L9"]
CONTROLS = ["CS5", "CS6", "CS7", "CS8", "X1", "X2", "X3", "X4"]


def suite(tag):
    """(passed, failed, targets, census_gate seconds, failing test names)."""
    p = os.path.join(LOGS, f"{tag}.suite.log")
    if not os.path.exists(p):
        return None
    passed = failed = targets = 0
    gate = None
    seen_gate = False
    fails = []
    with open(p, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if "Running tests/census_gate.rs" in line:
                seen_gate = True
            m = re.match(r"test result: \w+\. (\d+) passed; (\d+) failed;", line)
            if m:
                passed += int(m.group(1))
                failed += int(m.group(2))
                targets += 1
                if seen_gate and gate is None:
                    d = re.search(r"finished in ([\d.]+)s", line)
                    gate = float(d.group(1)) if d else None
            if line.startswith("    ") and line.strip().endswith("... FAILED"):
                fails.append(line.strip()[:-11].strip())
            m2 = re.match(r"^failures:$", line.strip())
    with open(p, encoding="utf-8", errors="replace") as fh:
        txt = fh.read()
    for m in re.finditer(r"^---- (\S+) stdout ----", txt, re.M):
        if m.group(1) not in fails:
            fails.append(m.group(1))
    return passed, failed, targets, gate, sorted(set(fails))


def hits(tag):
    p = os.path.join(LOGS, f"{tag}.hits")
    if not os.path.exists(p):
        return None
    out = set()
    with open(p, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            t = line.strip()
            if t:
                out.add(t)
    return out


def panics(tag):
    """Every `w-deadsites <ID>` marker anywhere in the run's logs.

    Grepped from the raw text rather than from an exit code: a panic that is
    caught and merely COUNTED (the gate has a `panics=` column) would leave no
    trace in a status, and this lane's whole claim is about branches that were
    never taken."""
    found = set()
    for suffix in ("suite.log", "gate.log", "scan.log", "driver.log"):
        p = os.path.join(LOGS, f"{tag}.{suffix}")
        if not os.path.exists(p):
            continue
        with open(p, encoding="utf-8", errors="replace") as fh:
            for m in re.finditer(r"w-deadsites ([A-Z0-9]+)", fh.read()):
                found.add(m.group(1))
    return found


def main():
    print("== suite rows ==")
    print(f"{'tag':6} {'passed':>7} {'failed':>7} {'targets':>8} {'census_gate':>12}  failing")
    for tag in sorted(t[:-10] for t in os.listdir(LOGS) if t.endswith(".suite.log")):
        s = suite(tag)
        if not s:
            continue
        p, f, n, g, fails = s
        gs = f"{g:.2f}s" if g is not None else "-"
        flag = "  INVALID(no differential)" if (g is not None and g < 1.0) else ""
        print(f"{tag:6} {p:>7} {f:>7} {n:>8} {gs:>12}  {' '.join(fails) if fails else '-'}{flag}")

    print()
    print("== reachability screen (behaviour-preserving first-hit probe) ==")
    tags = sorted(t[:-5] for t in os.listdir(LOGS) if t.endswith(".hits"))
    tags = [t for t in tags if hits(t)]
    print("runs:", " ".join(tags))
    print(f"{'id':6} " + " ".join(f"{t:>6}" for t in tags) + "   verdict")
    for sid in CONTROLS + OPEN:
        cells = []
        for t in tags:
            cells.append("FIRES" if sid in (hits(t) or set()) else "quiet")
        kind = "ctl " if sid in CONTROLS else "    "
        print(f"{sid:6} " + " ".join(f"{c:>6}" for c in cells) + f"   {kind}")

    print()
    print("== panic!() confirmation ==")
    for tag in sorted(t[:-10] for t in os.listdir(LOGS) if t.endswith(".suite.log")):
        pk = panics(tag)
        if tag.startswith("Q"):
            print(f"{tag}: markers seen = {sorted(pk) if pk else 'NONE'}")


if __name__ == "__main__":
    sys.exit(main())

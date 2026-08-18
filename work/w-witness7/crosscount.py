#!/usr/bin/env python3
"""`w-witness7` — EVERY published count, derived a SECOND way, and diffed.

Board **#3288** in its stronger form (`w-grammarscreen`, on master): *a
published count that was never diffed against a second, differently-built one
is unverified in **either** direction* — one member of that defect class was an
**over**-count wrong at its own base, so "the second count is smaller" is not
the failure mode to watch for.

So: for each figure this lane publishes, build it twice by two constructions
that do not share a parser, and print `AGREE` or `DIFFER`. A `DIFFER` line is
the finding; an all-`AGREE` run is what licenses quoting the numbers.

    crosscount.py
"""

import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
LOGS = os.path.join(ROOT, "work/w-witness7/logs")

ROWS = []


def row(what, a_desc, a, b_desc, b):
    ROWS.append((what, a_desc, a, b_desc, b))


def read(p):
    with open(p, encoding="utf8", errors="replace") as fh:
        return fh.read()


# --------------------------------------------------------------- the suite --
def suite(tag):
    t = read(os.path.join(LOGS, f"{tag}.suite.log"))
    # A: cargo's own per-target summary lines, summed.
    a_pass = sum(int(m.group(1)) for m in
                 re.finditer(r"^test result: \w+\. (\d+) passed;", t, re.M))
    a_tgt = len(re.findall(r"^test result: ", t, re.M))
    # B: the individual result lines, and the `Running` headers — a different
    #    parser over a different part of the log entirely.
    b_pass = len(re.findall(r"^test \S+ \.\.\. ok$", t, re.M))
    b_tgt = len(re.findall(r"^\s+Running ", t, re.M)) + \
        len(re.findall(r"^\s+Doc-tests ", t, re.M))
    row(f"{tag} suite passed", "sum of `test result:` lines", a_pass,
        "count of `test … ok` lines", b_pass)
    row(f"{tag} suite targets", "count of `test result:` lines", a_tgt,
        "count of `Running`/`Doc-tests` headers", b_tgt)


# ----------------------------------------------------------------- the gate --
def gate(tag):
    p = os.path.join(LOGS, f"{tag}.gate.log")
    if not os.path.exists(p):
        return
    t = read(p)
    # A: the block between the dashed header and the next blank line.
    lines = t.split("\n")
    a, on = 0, False
    for l in lines:
        if re.match(r"^-{20} ", l):
            on = True
            continue
        if on and not l.strip():
            on = False
        elif on:
            a += 1
    # B: every line whose second column is a verdict word, anywhere in the log.
    b = len([l for l in lines
             if re.match(r"^\S+ +(PASS|FAIL|SKIP|REFUSED|NO-RESULT) ", l)])
    row(f"{tag} gate lane rows", "the dashed-header block", a,
        "lines with a verdict in column 2", b)


# ----------------------------------------------------------------- the scan --
def scan(tag):
    p = os.path.join(LOGS, f"{tag}.scan.log")
    if not os.path.exists(p):
        return
    lines = read(p).split("\n")
    # A: the anchored regex the standing block prescribes (#3269).
    a = len([l for l in lines if re.match(r"^ *gap-metric \S+ \S+$", l)])
    # B: PARSE — split on whitespace and require exactly three fields whose
    #    first is the literal `gap-metric`. No regex at all (#3288: the
    #    mitigation is to parse rather than to write a cleverer pattern).
    b = 0
    for l in lines:
        f = l.split()
        if len(f) == 3 and f[0] == "gap-metric":
            b += 1
    row(f"{tag} gap-metric keys", "anchored regex", a, "3-field parse", b)


# ------------------------------------------------------- the lane's own work --
def guards():
    mine = ["nonformal_sites", "census_key_routing"]
    tests_dir = os.path.join(ROOT, "crates/c2-harness/tests")
    # A: `#[test]` attributes in the two new files.
    a_tests = sum(read(os.path.join(tests_dir, f"{m}.rs")).count("#[test]")
                  for m in mine)
    # B: the suite delta, base to tip.
    t0 = read(os.path.join(LOGS, "N0.suite.log"))
    t1 = read(os.path.join(LOGS, "T1.suite.log"))
    n0 = sum(int(m.group(1)) for m in
             re.finditer(r"^test result: \w+\. (\d+) passed;", t0, re.M))
    n1 = sum(int(m.group(1)) for m in
             re.finditer(r"^test result: \w+\. (\d+) passed;", t1, re.M))
    row("new tests", "`#[test]` count in the two files", a_tests,
        "suite delta T1 - N0", n1 - n0)

    # The seven sites, counted two ways.
    sites = ["CS3", "CS4", "CS9", "CA6", "CA8", "B2", "B7"]
    # A: sites whose TIP mutant run has one of THIS LANE's tests in its failing
    #    set. `CS3` is represented by `M-CS3` (its registered mutation).
    mut_of = {"CS3": "M-CS3", "CS4": "M-CS4", "CS9": "M-CS9", "CA6": "M-CA6",
              "CA8": "M-CA8", "B2": "M-B2", "B7": "M-B7"}
    my_tests = set()
    for m in mine:
        my_tests |= set(re.findall(r"^fn (\w+)\(", read(os.path.join(tests_dir, f"{m}.rs")), re.M))
    a = 0
    for s in sites:
        p = os.path.join(LOGS, f"{mut_of[s]}.tip.suite.log")
        fails = failing_set(read(p))
        if fails & my_tests:
            a += 1
    # B: sites named in an assertion message of the two files. A different
    #    construction entirely — it reads the tests' own prose, not the runs.
    b = 0
    for s in sites:
        pat = f"`{s}`"
        if any(pat in read(os.path.join(tests_dir, f"{m}.rs")) for m in mine):
            b += 1
    row("sites guarded", "tip mutant runs failing one of this lane's tests", a,
        "sites named in the two files' assertions/docs", b)

    # The REFUSING population: sites for which a real capture publishes the
    # site's key at base, and sites whose key MOVED under the site's mutation.
    fc = read(os.path.join(LOGS, "flipcheck.log"))
    base_keys = {
        "CS3": "static-scan-loop-object-out-of-class:eof",
        "B2": "static-scan-loop-object-out-of-class:eof",
        "B7": "callee-unresolved-tail-call:eof",
        "CA6": "call-arg-nonformal:eof",
        "CA8": "call-arg-computed:eof",
        "CS9": "opt-mode-00800005",
        "CS4": "store-run-bind-address-producer:eof",
    }
    a = sum(1 for s, k in base_keys.items()
            if any(l.startswith("BASE") and k in l for l in fc.split("\n")))
    moved = {"M-CS3", "M-CS4", "M-CS9", "M-CA6", "M-CA8", "M-B2", "M-B7"}
    b = 0
    for m in sorted(moved):
        got = [l for l in fc.split("\n") if l.startswith(m + " ")]
        if got:
            b += 1
    row("refusing population", "sites whose key is OBSERVED on a base capture", a,
        "sites whose key MOVES under the site's own mutation", b)


def failing_set(t):
    names, infail = set(), False
    for l in t.split("\n"):
        if l.strip() == "failures:":
            infail = True
            continue
        if infail:
            m = re.match(r"^\s{4}(\S+)$", l)
            if m and not m.group(1).startswith("---"):
                names.add(m.group(1).split("::")[-1])
            elif not l.strip() or l.startswith("test result"):
                infail = False
    return names


def main():
    for tag in ("N0", "T1"):
        suite(tag)
    gate("T2")
    scan("T2")
    guards()

    bad = 0
    w = max(len(r[0]) for r in ROWS)
    for what, ad, a, bd, b in ROWS:
        ok = "AGREE " if a == b else "DIFFER"
        if a != b:
            bad += 1
        print(f"{ok}  {what:<{w}}  {a:>6} ({ad})  |  {b:>6} ({bd})")
    print(f"\n{len(ROWS)} figures, {bad} DIFFER")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())

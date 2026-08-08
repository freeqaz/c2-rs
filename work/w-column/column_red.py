#!/usr/bin/env python3
"""column_red.py — the RED test for lane w-column's codegen column.

A test suite that has never been seen to fail is a test suite of unknown power.
This applies one mutation at a time to the shipped code, runs the ONE test that
is supposed to catch it, and requires that test to go **red** — then restores
the tree and checks it is byte-identical to where it started.

Two traps observed on this box this week, and both are guarded here:

  * **an early guard can make a later assertion unreachable**, so a mutation can
    be "caught" by a check that has nothing to do with it. Every arm therefore
    records the VERBATIM first assertion message and
  * **a shared message prefix** let a later refusal satisfy an earlier case's
    expectation — two of six mutations silently passed. So every arm's expected
    message must lead with its own DISTINCT WORD, and the run FAILS if two arms
    share one.

Green control arms are included: mutations that must NOT be caught by the named
test would be a lie, so the greens are instead *unmutated* runs of the same
tests, which must pass. A suite that is red on everything is as useless as one
that is red on nothing.

    usage:  python3 work/w-column/column_red.py [--list] [--only ARM]

Exit 0 iff every arm behaves as declared. Writes nothing outside the repo and
restores every file it touches; `crates/` is verified EMPTY in `git diff` at the
end and the run fails if it is not.
"""

import argparse
import hashlib
import os
import re
import subprocess
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
FNBYTES = os.path.join(REPO, "crates/c2-harness/src/gap/fnbytes.rs")
FACTORS = os.path.join(REPO, "crates/c2-harness/src/gap/factors.rs")

# (name, file, old, new, test, expected leading word in the failure message)
#
# `old` must occur EXACTLY ONCE in the file — a mutation applied in two places
# is a different experiment from the one declared, and this script refuses it
# rather than reporting on it.
ARMS = [
    (
        "parse-is-codegen",
        FNBYTES,
        "        !matches!(self, Decline::Parse)",
        "        true",
        "a_parse_refusal_is_not_a_codegen_refusal",
        "parser",
    ),
    (
        "selector-not-codegen",
        FNBYTES,
        "        !matches!(self, Decline::Parse)",
        "        !matches!(self, Decline::Parse | Decline::Selector)",
        "a_parse_refusal_is_not_a_codegen_refusal",
        "Selector",
    ),
    (
        "parse-refiled-as-selector",
        FNBYTES,
        'return g(FnByte::Refused, "parse-refused", Some(Decline::Parse));',
        'return g(FnByte::Refused, "parse-refused", Some(Decline::Selector));',
        "grade_one_files_a_parse_refusal_under_the_parser_and_not_the_selector",
        "unlowered",
    ),
    (
        "reader-folded-into-price",
        FACTORS,
        "        self.wrong + self.cg_refused\n",
        "        self.wrong + self.cg_refused + self.reader\n",
        "the_codegen_column_counts_the_unmeasurable_half_separately",
        "measurable",
    ),
    (
        "partition-always-holds",
        FACTORS,
        "        self.exact + self.wrong + self.cg_refused + self.reader + self.ungraded\n            != self.denominator\n",
        "        false\n",
        "the_codegen_partition_control_fires_on_a_short_row",
        "!=",
    ),
    (
        "reloc-differs-dropped",
        FACTORS,
        'wrong: g("fnbyte-differs") + g("fnbyte-reloc-differs"),',
        'wrong: g("fnbyte-differs"),',
        "the_codegen_column_counts_the_unmeasurable_half_separately",
        "differs",
    ),
    (
        "metrics-gated-on-a-frontier",
        FACTORS,
        '            m.push(("frontier-codegen-denominator", s(|c| c.denominator).to_string()));',
        '            if !cols.is_empty() { m.push(("frontier-codegen-denominator", s(|c| c.denominator).to_string())); }',
        "the_codegen_column_metrics_survive_an_empty_frontier",
        "frontier-codegen-denominator",
    ),
    (
        "wrong-credited-like-exact",
        FACTORS,
        'exact: g("fnbyte-exact"),',
        'exact: g("fnbyte-exact") + g("fnbyte-differs"),',
        "a_wrong_emit_raises_the_codegen_price_and_never_lowers_it",
        "…distinguished",
    ),
]

# NOTE on `parse-refiled-as-selector`. This arm was DECLARED UNCAUGHT on its
# first run and the declaration was correct: every other test in the suite
# writes `fnbyte-refused-parse` into the count map directly, so none of them can
# see the producer, and the arm passed all of them. The repair was a test, not a
# relabelling — `grade_one_files_a_parse_refusal_under_the_parser_and_not_the_
# selector` calls `grade_one` itself against a hand-built census row whose gate
# is `Err`. An arm claimed to be caught by a test that cannot see it is exactly
# the shared-prefix failure this file exists to prevent, so the intermediate
# state is recorded here rather than deleted.

GREEN = [
    "a_parse_refusal_is_not_a_codegen_refusal",
    "the_codegen_column_counts_the_unmeasurable_half_separately",
    "the_codegen_partition_control_fires_on_a_short_row",
    "the_codegen_column_metrics_survive_an_empty_frontier",
    "the_refusal_split_is_published_with_its_own_control",
    "a_wrong_emit_raises_the_codegen_price_and_never_lowers_it",
    "grade_one_files_a_parse_refusal_under_the_parser_and_not_the_selector",
]


def sha(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def run_test(name):
    """Run one test by exact name. Returns (passed, first failure message)."""
    p = subprocess.run(
        ["cargo", "test", "--release", "-p", "c2-harness", "--lib", name, "--", "--exact",
         "gap::tests::" + name],
        cwd=REPO, capture_output=True, text=True,
    )
    if "test result: ok" in p.stdout and " 1 passed" in p.stdout:
        return True, ""
    # The message a `assert!`/`assert_eq!` printed, verbatim.
    m = re.search(r"panicked at [^\n]*\n(.*?)(?:\nnote: run with|\n\n)", p.stdout, re.S)
    msg = (m.group(1) if m else p.stdout[-800:]).strip()
    return False, msg


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--only")
    a = ap.parse_args()

    caught = [x for x in ARMS if x[5] is not None]
    if a.list:
        for arm in ARMS:
            tag = "RED" if arm[5] else "UNCAUGHT-BY-UNIT-SUITE"
            print(f"{tag:24} {arm[0]:28} -> {arm[4]}")
        for g in GREEN:
            print(f"{'GREEN':24} {g}")
        print(f"\n{len(caught)} red arms, {len(ARMS) - len(caught)} declared uncaught, "
              f"{len(GREEN)} green controls")
        return 0

    # Trap: two arms whose expected message shares a leading word means a later
    # refusal can satisfy an earlier case. Checked BEFORE anything runs.
    words = [arm[5] for arm in caught]
    if len(set(words)) != len(words):
        dup = [w for w in words if words.count(w) > 1]
        print(f"FAIL: two red arms expect the same leading word {sorted(set(dup))} — "
              f"a later refusal could satisfy an earlier arm's expectation")
        return 1
    print(f"leading words: {len(set(words))} of {len(words)} DISTINCT")

    before = {FNBYTES: sha(FNBYTES), FACTORS: sha(FACTORS)}
    ok = True

    print("\n== GREEN CONTROLS (unmutated tree) ==")
    for g in GREEN:
        if a.only and a.only != g:
            continue
        passed, msg = run_test(g)
        print(f"  {'PASS' if passed else 'FAIL'}  {g}")
        if not passed:
            print(f"        {msg}")
            ok = False

    print("\n== RED ARMS ==")
    for name, path, old, new, test, word in ARMS:
        if a.only and a.only != name:
            continue
        if word is None:
            print(f"  DECLARED-UNCAUGHT  {name}  (guarded by the scan's own "
                  f"`fnbyte-decline-selector 0`, not by {test})")
            continue
        with open(path) as f:
            src = f.read()
        n = src.count(old)
        if n != 1:
            print(f"  FAIL  {name}: the mutation site occurs {n} times, not once — "
                  f"refusing to run a different experiment from the declared one")
            ok = False
            continue
        try:
            with open(path, "w") as f:
                f.write(src.replace(old, new))
            passed, msg = run_test(test)
        finally:
            with open(path, "w") as f:
                f.write(src)
        if passed:
            print(f"  FAIL  {name}: {test} still PASSES under the mutation — "
                  f"the assertion is unreachable or does not test this")
            ok = False
            continue
        if word not in msg:
            print(f"  FAIL  {name}: {test} went red, but on the WRONG assertion "
                  f"(expected a message containing {word!r})\n        {msg}")
            ok = False
            continue
        first = msg.splitlines()[0] if msg else ""
        print(f"  RED   {name}: {test}")
        print(f"        {first}")
        if len(msg.splitlines()) > 1:
            print(f"        {' '.join(msg.split())[:220]}")

    for path, h in before.items():
        if sha(path) != h:
            print(f"FAIL: {os.path.relpath(path, REPO)} was NOT restored")
            ok = False
    diff = subprocess.run(["git", "diff", "--stat", "--", "crates/"],
                          cwd=REPO, capture_output=True, text=True).stdout.strip()
    print(f"\nfinal crates/ diff: {'EMPTY' if not diff else 'DIRTY — ' + diff}")
    if diff:
        ok = False

    print(f"\n{'ALL ARMS PASS' if ok else 'FAILURES ABOVE'} — "
          f"{len(caught)} red, {len(ARMS) - len(caught)} declared uncaught, "
          f"{len(GREEN)} green")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())

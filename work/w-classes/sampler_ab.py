#!/usr/bin/env python3
"""A/B the OLD strided sampler against the NEW stratified one, same budget.

The regeneration widened two rows — `51-dtor-member` 8 -> 14 lanes and
`60-ctor-epilogue` 4 -> 7. A widening is free in coverage terms, but the
INTERESTING question is which of the two changes bought it:

  * the sampler (stride -> stratified), or
  * the budget (24 -> 64 cases per fragment).

Only the first would mean the incumbent table was wrong *because the instrument
was*. So run both samplers at the SAME budget and compare the partitions.

    work/w-classes/sampler_ab.py <per-fragment> <fragment-substring> [jobs]
"""

import os
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import mode_invariance as mi   # noqa: E402


def strided(frag, cs, p):
    k = max(1, (len(cs) + p - 1) // p)
    return cs[::k][:p]


def run(label, sampler, p, only, jobs):
    mi.sample_cases = sampler
    out = os.path.join(REPO, "work/w-classes/ab", label)
    sys.argv = ["mode_invariance.py", "--out", out, "--per-fragment", str(p),
                "--jobs", str(jobs), "--only", only]
    print("=" * 72)
    print("SAMPLER: %s   per-fragment %d   fragments matching %r" % (label, p, only))
    print("=" * 72)
    mi.main()


def main():
    p = int(sys.argv[1])
    only = sys.argv[2]
    jobs = sys.argv[3] if len(sys.argv) > 3 else "24"
    new = mi.sample_cases
    run("strided", strided, p, only, jobs)
    run("stratified", new, p, only, jobs)


main()

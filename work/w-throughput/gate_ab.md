# The `--jobs` A/B: the verdict counts, shown identical

## Method, and why it is this method

`mode_cross.sh` keys the capture cache on the source path and therefore keeps a
**stable** case directory under `<repo_root>/work/mode-cross/cases`. A fresh
worktree starts that directory empty, so the FIRST cross in a worktree is cold —
the script prices it at 5 min 45 s cold vs 13.8 s warm at 8 jobs over 61,539
cells, and this gate's cross is 90,812 cells. **A base-vs-tip gate comparison out
of a fresh worktree measures the cache, not the concurrency.**

The base tree still contributes two things that no cache state can distort: the
sweep leg (which never touches the cache — it drives `c2rs diff`, board #282),
and the identity of the binary under test:

    base run (jobs 4):   sha fbe465a1ffb1   tree f49fe5e1
    tip runs:            sha fbe465a1ffb1   tree f49fe5e1-dirty

**The pinned harness is byte-identical at both ends.** This lane changes one test
file, two shell scripts and docs — nothing the binary is built from. That is also
why the 878-TU scan cannot move, and it is checked rather than assumed.

## THE EQUIVALENCE — the sweep's counts at 4 workers and at 16

Same 19,556-case corpus, same pinned binary, `--require-graded`:

    --jobs 4    checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0
    --jobs 16   checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0

Digit for digit. **The parallel split is an equivalence, not an approximation.**
Nothing about *what* is graded depends on the worker count: the corpus is
generated before the split, `awk` assigns every case to exactly one worker by
line number (`(NR - 1) % j`), and the workers' own counts are summed and
reconciled against the selected count afterwards.

## THE BOX WAS CONTENDED, AND ONE OF THESE TIMINGS SAYS SO OUT LOUD

    leg / setting                      wall     load avg during
    sweep, --jobs 4  (base tree)       262 s    7 - 11
    sweep, --jobs 16 (tip tree)        309 s    67 - 100

**At `--jobs 16` the sweep was SLOWER than at `--jobs 4`, and that is a
measurement of the box, not of the change.** Four `gate.sh` processes were
running simultaneously (`ps -e -o comm= | grep -c gate.sh` → 4): this lane's plus
three from concurrent lanes, on 32 logical cores. A 16-worker gate on a box
already carrying 60+ runnable processes is queueing, not parallelising.

That is exactly why the timing pair quoted in the rung is taken **back to back on
one tree with the cache warm**, and why the counts above — which do not move with
load — are the part of this file that licenses the default change.

RETAKE_BLOCK

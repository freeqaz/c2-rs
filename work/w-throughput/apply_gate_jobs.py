#!/usr/bin/env python3
"""Raise scripts/gate.sh's default `--jobs` from 4 to 16, and say why.

Applied once, by lane w-throughput. Kept in the lane record so the exact edit is
readable next to the measurement that justified it.
"""
import sys, pathlib

p = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "scripts/gate.sh")
s = p.read_text()

SUBS = []

# ---- 1. the constant itself ----------------------------------------------------
SUBS.append((
    "jobs=4\n",
    """# ---- THE DEFAULT CONCURRENCY, AND WHY IT IS 16 AND NOT 4 ----------------------
#
# `jobs=4` stood here from this file's creation commit (`25def085`, 2026-07-31)
# to 2026-08-08 and was never revisited: `git log -L347,347` has exactly one
# entry and no board row explains the number. It is an untuned constant, not a
# safety limit — this is a 32-core box, and it governs three legs (lane
# parallelism, `C2RS_SWEEP_JOBS`, and the `C2RS_JOBS` handed to `mode_cross.sh`),
# all three of which are embarrassingly parallel over per-case scratch dirs.
#
# MEASURED on this box 2026-08-08, same tree, same corpus, back to back:
#
#     gate.sh --require-graded                ~300 s   (sweep 262 s, cross  27 s)
#     gate.sh --jobs 16 --require-graded       105 s   (sweep  82 s, cross  21 s)
#
# and **the two verdict blocks are identical, digit for digit** — 18/18 lanes,
# 5,184 fixture-verdicts, sweep `checked=19556 mismatches=0 graded=19460
# ungraded=96 unknown=0`, cross `checked=90812 mismatches=0 graded=90424
# ungraded=388`. The parallel split is an EQUIVALENCE, not an approximation, and
# that is a statement this file has already made once for 4 -> 8 in the cost
# table below. Nothing about what is graded moves with this number: the corpus is
# generated before the split, `awk` assigns every case to exactly one worker by
# line number, and the worker counts are reconciled against the selected count
# afterwards.
#
# **Why 16 and not 24.** The knee is ~24 (68 s standalone; 32 gives 70 s, i.e.
# nothing) — so 24 buys 14 s more on an IDLE box. 16 is the physical core count
# (32 logical), and past it each extra worker is a whole `cl` + wibo process tree
# competing for a hyperthread sibling: the first twelve workers buy 180 s and the
# next eight buy 14 s. The box also routinely carries two or three lanes gating
# at once, where a per-gate 24 means 72 concurrent capture trees and the 68 s no
# longer holds. 16 is the value that is fast when the box is busy and within 20 %
# of the best when it is idle.
#
# **What breaks first, checked rather than assumed.** Not inodes: the in-flight
# draw is set by the CORPUS, not by the concurrency — measured 19,810 free-inode
# low-water draw at 4 workers and 19,885 at 16, against the 150,000 floor the
# preflight enforces (`/tmp` had 753,110 free on the run that produced the table
# above). Not wibo: board #201 measured 48 concurrent captures to one distinct
# `.gl` sha and censuses at 1/14/32 jobs byte-identical, and `census_gate` has
# been running 2x16 capture threads on every merge for weeks. The failure
# direction of over-concurrency is a DISK red, which `GATE: FAIL (DISK)` / exit 3
# already tells apart from a mismatch.
#
# **The false-green question, answered by making it happen.** A worker that dies
# at higher concurrency must not shorten the corpus quietly. It cannot: each
# worker writes its own `checked.N` only after finishing its chunk, and a missing
# file makes the sum fall short of the selected count, which is `FATAL: selected
# R cases and only C were graded` (exit 3) inside `expr_sweep.sh` and, redundantly,
# `SHORT — selected R cases, reached C` in `sweep_verdict` here. Lane w-throughput
# verified that by KILLING a worker mid-run rather than by reading the code;
# the transcript is in `docs/rungs/2026-08-08-w-throughput.md` §3.
#
# `--jobs` is unchanged as an override and the header line below still prints the
# value actually used, so a run's own output says what concurrency produced it.
jobs=16
"""))

# ---- 2. the usage line ---------------------------------------------------------
SUBS.append((
    "#   scripts/gate.sh --jobs 4              lanes in parallel (default 4); also the\n",
    "#   scripts/gate.sh --jobs 24             lanes in parallel (default 16); also the\n",
))

# ---- 3. the cost table ---------------------------------------------------------
SUBS.append((
    """#     12 lanes alone, --jobs 8                      7 s
#     sweep alone, serial (as it was written)   9 min 51 s
#     sweep alone, --jobs 8                     1 min 26 s
#     THIS GATE, --jobs 8                       1 min 34 s
#
# Both sweep runs printed `checked=14484 mismatches=0` — the parallel split is an
# equivalence, not an approximation.""",
    """#     12 lanes alone, --jobs 8                      7 s
#     sweep alone, serial (as it was written)   9 min 51 s
#     sweep alone, --jobs 8                     1 min 26 s
#     THIS GATE, --jobs 8                       1 min 34 s
#
# Both sweep runs printed `checked=14484 mismatches=0` — the parallel split is an
# equivalence, not an approximation.
#
# RE-MEASURED 2026-08-08 (lane w-throughput), 18 lanes and a 19,556-case corpus,
# `--require-graded`, box carrying two other lanes — the table above is a
# 12-lane/14,484-case measurement and is kept as the record of what was true then:
#
#     THIS GATE, --jobs 4 (the old default)    ~300 s  (sweep 262 s, cross 27 s)
#     THIS GATE, --jobs 16 (today's default)    105 s  (sweep  82 s, cross 21 s)
#
# Same verdict block at both: 18/18 lanes, 5,184 fixture-verdicts, sweep
# `checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0`, cross
# `checked=90812 mismatches=0 graded=90424 ungraded=388`. See the block above
# `jobs=` for why 16 and not 24, and for the killed-worker demonstration.""",
))

# ---- 4. C2RS_JOBS is DELIBERATELY left at 8 ------------------------------------
SUBS.append((
    ': "${C2RS_JOBS:=8}"\nexport C2RS_JOBS\n',
    """# **This one is deliberately NOT raised, and the reason is that it MULTIPLIES.**
# `C2RS_JOBS` is read only by `mode_lane.sh` (`c2rs gap --jobs`), so it is the
# per-lane thread count *inside* each of the `$jobs` lanes running at once:
# raising `jobs` 4 -> 16 already took the lane leg from 4x8 to 16x8 concurrent
# capture threads. `mode_cross.sh` never sees this value — the cross is invoked
# with `C2RS_JOBS="$jobs"` explicitly, below.
#
# And the leg it governs is not where the time is: MEASURED 2026-08-08, the lane
# leg is 16 s of a ~300 s gate at `--jobs 4` and 2 s of a 105 s gate at
# `--jobs 16`. There is nothing left in it to win, and a second concurrency knob
# raised at the same time as the first would make the next timing unattributable.
# It stays 8, and it stays overridable from the environment.
: "${C2RS_JOBS:=8}"
export C2RS_JOBS
""",
))

for old, new in SUBS:
    n = s.count(old)
    if n != 1:
        sys.exit(f"REFUSED: pattern occurs {n} times, expected 1:\n{old[:90]}")
    s = s.replace(old, new)

p.write_text(s)
print(f"applied {len(SUBS)} edits to {p}")

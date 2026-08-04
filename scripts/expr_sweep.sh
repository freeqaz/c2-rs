#!/bin/sh
# Generated differential sweep over the shapes that claim byte-exactness.
#
# Enumerates small translation units over a set of axes, compiles each against the
# real toolchain, and reports every byte MISMATCH. This is the thing that found the
# reassociation and repeated-leaf mis-emits: ~20 wrong-bytes bugs in the
# straight-line class that the hand-written corpus had never separated, because
# every fixture in it happened to use distinct operands in ascending order.
#
# The lesson is in `docs/GAPS.md`: a green fixture run is only as strong as the
# corpus's ability to *separate* the candidate rules, and a hand-picked corpus is
# systematically biased toward the shapes whoever wrote it was already thinking
# about. Enumeration has no such bias.
#
# A MISMATCH is an alarm, not a gap — the port emitted bytes and they were wrong.
# Either fix the lowering or tighten the gate until it refuses. NotImplemented is
# fine and expected for most cases here.
#
# ---- the fragment contract (docs/ARCHITECTURE_SEAMS.md §2.4) -------------------
#
# The generator lives in `scripts/sweep.d/`, ONE FILE PER AXIS. A rung adds a NEW
# file; two rungs adding fragments never conflict, and two rungs claiming the same
# fragment name is an add/add conflict git flags loudly.
#
# Each fragment defines exactly:
#
#     def cases(emit):
#         emit("int f(int a) { return a + 1; }\n")   # one .cpp case
#
# `emit` is supplied by the LOADER (`scripts/sweep_gen.py`), which owns the
# counter and namespaces the output by fragment (`10-int-chains-0007.cpp`). A
# fragment therefore cannot see or touch another fragment's counter: the
# `n`-shadowing trap that silently rewound the file counter and overwrote 1,233
# already-written cases is now *unrepresentable*, not merely fixed. The loader
# prints a per-fragment count, **fails if any fragment emits zero cases** — the
# observable symptom of that bug is now a hard error — and **fails if what it
# counted is not what is on disk**, which is that bug's actual damage.
#
# The loader is a module rather than an inline block because it has a second
# consumer: `scripts/cross_sweep.py` grades the CROSS PRODUCT of the shape
# families these cases exercise, and two copies of the enumeration is the
# "one rule, two implementations" shape `docs/GAPS.md` §6 keeps recording.
#
# Usage:  scripts/expr_sweep.sh [outdir] [max-cases]
#         scripts/expr_sweep.sh /tmp/sweep 400     # a STRIDED subset, see below
#         C2RS_SWEEP_ONLY=fp scripts/expr_sweep.sh # only fragments matching "fp"
#         C2RS_SWEEP_JOBS=8 scripts/expr_sweep.sh  # grade 8 cases at a time
#
# `C2RS_SWEEP_ONLY` is for iterating on one axis; it makes the total meaningless
# by design, so the driver says so out loud and any gate run must be unfiltered.
#
# ---- `max-cases` is a STRIDE, not a prefix (changed 2026-08-04) ----------------
#
# It used to be `head -n N` over `cases.txt`, which is sorted by fragment name — so
# a "quick subset" was **the alphabetically first fragments and nothing else**.
# Measured on today's corpus: `head -400` covers **1 of the 47 fragments**, and the
# case that carried board #232 — the live `Port=Mismatch` this sweep found — is
# **line 9,538 of 14,484**. So every prefix under 66 % of the corpus, i.e. every
# subset small enough to be worth taking, was STRUCTURALLY BLIND to it. A biased sample of an enumeration defeats the
# only property the enumeration has (`docs/GAPS.md`: a hand-picked corpus is
# biased toward the shapes whoever picked it was thinking about — and a prefix of
# a sorted list is hand-picked by the sort).
#
# So `N` now selects every `ceil(total/N)`-th case, which keeps every fragment
# represented in proportion: the same budget of 400 reaches **46 of 47** fragments
# (the missing one, `52-callee-name`, is smaller than the stride — a sample is
# still a sample, and this is the honest count, not "all of them"). It is still a sample and still cannot establish what
# a full run establishes; `scripts/gate.sh` therefore refuses to print an
# unqualified PASS over one.
#
# ---- grading is PARALLEL --------------------------------------------------------
#
# Each case is an independent `c2rs diff`; nothing is shared but the capture cache,
# which has been cross-process safe since board #181 (an `O_EXCL` lockfile per key,
# fail-open). MEASURED here 2026-08-04, 14,484 cases, warm cache, 32-core host —
# **9 min 51 s serial, 1 min 26 s at `C2RS_SWEEP_JOBS=8`**, with `checked=14484
# mismatches=0` from both. That cost is the whole reason the biased `max-cases`
# knob existed and the whole reason this sweep was not in the merge gate while
# #232 survived 241 commits; at 8 jobs it is affordable unconditionally, so the
# argument for leaving it out is spent.
#
# Workers write per-worker count and mismatch files and the driver SUMS THE COUNTS:
# a worker that dies contributes a short count and the reconciliation below fails,
# rather than contributing a silence that reads as zero.
#
# Needs the toolchain (see CLAUDE.md); without it every case reports SKIP and the
# sweep is vacuous, so it checks for that up front.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-/tmp/c2rs-expr-sweep}"
limit="${2:-0}"
mkdir -p "$out"

# Refuse to share an output directory with another live sweep. This driver
# `rm -f`s the whole case set before regenerating it, so two concurrent runs
# against the default `/tmp/c2rs-expr-sweep` delete each other's cases mid-grade
# — one run's cleanup lands in the middle of the other's grading, and BOTH
# results are then meaningless while looking perfectly ordinary. That happened:
# an agent ran two sweeps at once, spotted it, killed both and re-ran with a
# private outdir. Nothing in the output would have said so.
#
# `mkdir` is the lock because it is atomic on every filesystem this runs on.
# The default stays shared on purpose — the generated cases are worth inspecting
# after a run — so the collision is made IMPOSSIBLE rather than made unlikely by
# a per-PID default that would leave litter nobody reads.
_lock="$out/.sweep.lock"
if ! mkdir "$_lock" 2>/dev/null; then
    echo "REFUSING: another sweep holds $out (lock: $_lock)." >&2
    echo "  Two sweeps in one outdir delete each other's cases and both results" >&2
    echo "  are silently wrong. Pass a private outdir:" >&2
    echo "      scripts/expr_sweep.sh /tmp/c2rs-sweep-\$\$" >&2
    echo "  If no sweep is running, the previous one was killed: rmdir '$_lock'" >&2
    exit 2
fi
trap 'rmdir "$_lock" 2>/dev/null || true' EXIT INT TERM

rm -f "$out"/*.cpp "$out"/cases.txt 2>/dev/null || true

# Build unconditionally and run a RUN-PRIVATE COPY of the binary — never
# `target/release/c2rs` directly. `scripts/harness_bin.sh` has the two failures
# this closes: the `if [ ! -x ]` guard that let a sweep grade today's cases with
# yesterday's code (47 phantom mismatches, false-green in the other direction),
# and a gate reading a file the rest of the tree may rewrite mid-run (one sweep
# did die that way at 6,225 cases; the mechanism is an unproven hypothesis and
# `harness_bin.sh` says so — the fix rests on the structural property, not on
# that observation). The identity line it prints — build
# time, content sha, tree HEAD — is what makes "which code produced this number"
# answerable from the log.
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$out"
c2rs="$C2RS_PINNED"

python3 "$repo_root/scripts/sweep_gen.py" "$out" "$repo_root/scripts/sweep.d"

ls "$out"/*.cpp | sort > "$out/cases.txt"
total=$(wc -l < "$out/cases.txt")
stride=1
if [ "$limit" -gt 0 ] 2>/dev/null && [ "$limit" -lt "$total" ]; then
    stride=$(( (total + limit - 1) / limit ))
    awk -v k="$stride" 'NR % k == 1 || k == 1' "$out/cases.txt" > "$out/cases.run"
else
    cp "$out/cases.txt" "$out/cases.run"
fi
run=$(wc -l < "$out/cases.run")

# Bail out loudly rather than reporting a vacuous pass.
first=$(head -1 "$out/cases.run")
if "$c2rs" diff "$first" 2>&1 | grep -q "SKIP"; then
    echo "SKIP: toolchain absent — the sweep would be vacuous"
    exit 0
fi

jobs="${C2RS_SWEEP_JOBS:-4}"
case "$jobs" in ''|*[!0-9]*) jobs=4 ;; esac
[ "$jobs" -ge 1 ] || jobs=1

if [ "$stride" -eq 1 ]; then
    echo "sweeping $run of $total generated cases"
else
    echo "sweeping $run of $total generated cases (STRIDE $stride — a SAMPLE, not the corpus)"
fi
echo "  grading at $jobs job(s)"

# Split into `jobs` chunks by line number, so every case lands in exactly one
# worker and the chunk sizes are derivable from the case count alone.
part="$out/parts"
rm -rf "$part"; mkdir -p "$part"
awk -v j="$jobs" -v d="$part" '{ print > (d "/chunk." ((NR - 1) % j)) }' "$out/cases.run"

w=0
while [ "$w" -lt "$jobs" ]; do
    (
        _n=0; _m=0
        if [ -f "$part/chunk.$w" ]; then
            while read -r f; do
                _n=$((_n + 1))
                verdict=$("$c2rs" diff "$f" 2>&1 | tail -1)
                case "$verdict" in
                    *Mismatch*)
                        _m=$((_m + 1))
                        # The FILE NAME first, then the source line. #232 took an
                        # extra investigation because only the source line was
                        # printed and ten cases share it — a mismatch you cannot
                        # re-run is a mismatch somebody calls unreproducible.
                        echo "MISMATCH  $f  |  $(head -1 "$f")" >> "$part/mismatch.$w"
                        ;;
                esac
            done < "$part/chunk.$w"
        fi
        echo "$_n" > "$part/checked.$w"
        echo "$_m" > "$part/mism.$w"
    ) &
    w=$((w + 1))
done
wait

# Sum the workers' own counts. A worker killed mid-chunk writes no `checked.N` at
# all, so the sum comes up short and the reconciliation below fails — the count
# is the evidence the work happened, never the exit status (STATUS.md trap 5).
checked=0; mismatch=0; reported=0
w=0
while [ "$w" -lt "$jobs" ]; do
    if [ -f "$part/checked.$w" ]; then
        checked=$((checked + $(cat "$part/checked.$w")))
        reported=$((reported + 1))
    fi
    [ -f "$part/mism.$w" ] && mismatch=$((mismatch + $(cat "$part/mism.$w")))
    w=$((w + 1))
done
cat "$part"/mismatch.* 2>/dev/null || true

if [ "$checked" -ne "$run" ]; then
    echo "checked=$checked mismatches=$mismatch"
    echo "FATAL: selected $run cases and only $checked were graded" >&2
    echo "  $reported of $jobs workers reported a count. A short count is a worker" >&2
    echo "  that died; the cases it held were never graded and this run establishes" >&2
    echo "  nothing about them." >&2
    exit 3
fi

echo "checked=$checked mismatches=$mismatch"
[ "$mismatch" -eq 0 ] || exit 1

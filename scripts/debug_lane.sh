#!/bin/sh
# THE DEBUG LANE — run the fixture corpus through a **debug-profile** `c2rs`.
#
# Board #3074. Every standing instrument in this repo runs `--release`:
# `scripts/gate.sh`, `scripts/expr_sweep.sh`, `scripts/mode_cross.sh`, the 878-TU
# workload scan, `scripts/status.sh`, and the workspace test row. In a release
# build `debug_assert!` is compiled out and integer overflow wraps silently — so
# the entire verification apparatus is **structurally incapable** of reporting
# either. The port's own emitter carried a FALSE assertion for four days and
# every gate run in that window was green, because no instrument could execute
# it. That is absence-read-as-success reaching the emitter's own assertions.
#
# This script is the missing positive check. It is **not** a byte judge and does
# not replace one: `mismatch` is still graded by `gate.sh` against real c2. What
# this adds is the two classes of fault a release build cannot express:
#
#   * a `debug_assert` that is FALSE — 75 of them live under `crates/`, and
#     until this ran, 0 of 75 were reachable by any standing instrument;
#   * an arithmetic overflow — the dev profile's `overflow-checks`, which found
#     a `usize` underflow in `gap/fnbytes.rs` on its first run.
#
# **A debug run that grades DIFFERENTLY from the release run is itself a
# failure**, so the counts are printed for a digit-for-digit compare against the
# matching `scripts/mode_lane.sh` row. They agreed exactly on first measurement
# (`/Ox /Gy`: graded 381, match 150, mismatch 0, both profiles).
#
# Usage:  scripts/debug_lane.sh              # every lane in scripts/lanes.txt
#         scripts/debug_lane.sh /Ox /Gy      # one lane, flags given literally
#
# env:    C2RS_LANES             lane registry to read (default scripts/lanes.txt)
#         C2RS_DEBUG_LANE_WORK   run directory (default $TMPDIR/c2rs-debug-lane-$$)
#         C2RS_JOBS              per-lane `c2rs gap` concurrency (default 8)
#         C2RS_DEBUG_LANE_LANES  how many LANES run at once (default 4)
#
# ---- THE LANES RUN CONCURRENTLY (lane `w-gateperf`, 2026-08-18) ----------------
#
# They used to run one after another, and MEASURED at the tip of that lane this
# row was **74 s of a 142 s `gate.sh --jobs 16 --require-graded` run — 52 % of
# the whole gate**, having been ~9 % of it before the sweep was served from the
# capture cache. `scripts/gate.sh`'s own lane leg has always run its 18 lanes
# `$jobs` at a time; this row is the same 18 lanes over the same corpus and had
# no reason to be the sequential one.
#
# **The counters are collected from FILES, not accumulated across a `&`.** A
# `fails=$((fails + 1))` inside a backgrounded subshell is discarded, which
# would turn every failing lane into a silent pass — the exact shape
# `gate.sh --selftest`'s own header warns about ("a `fails` counter incremented
# inside `$(...)` lives in a subshell and is discarded, which would make this
# selftest itself an instrument that reports green from an absence"). So each
# lane writes its own result line to its own file, the reader walks the registry
# afterwards in registry order, and **a lane with no result file is a hard
# failure** rather than a lane that contributed nothing. That is `expr_sweep.sh`'s
# short-count rule, and it makes a killed worker impossible to mistake for a
# clean one.
#
# Output is UNCHANGED: the same `DEBUG-LANE-RESULT` lines in the same registry
# order, and the same `DEBUG-LANE-TOTAL`. `gate.sh`'s `debug_verdict` sees
# exactly what it saw before.
#
# Exit status is non-zero if any lane panicked or reported a mismatch. One
# `DEBUG-LANE-RESULT` line per lane, plus one `DEBUG-LANE-TOTAL` line.
#
# ---- WIRED INTO `scripts/gate.sh` AS OF 2026-08-17 (lane w-gatewire) ----------
#
# This header used to read *"NOT wired into `scripts/gate.sh`. Wiring it in is a
# shared-gate decision and belongs to whoever owns the gate."* It is wired now:
# **a debug-profile panic fails the gate and therefore blocks a merge.** The
# reason it waited is on the record and is worth keeping — the rung that shipped
# this file said *"making a debug panic a merge blocker is the user's decision,
# not the coordinator's and not this lane's"* — and the user has now made it.
#
# Two consequences for anyone editing this file:
#
#   * **The output contract is now load-bearing.** `gate.sh`'s `debug_verdict`
#     re-derives its verdict from the `DEBUG-LANE-RESULT` / `DEBUG-LANE-TOTAL`
#     lines, by FIELD NAME, and counts the result lines against `lanes=`. Renaming
#     a field or dropping the total line turns the row `NO-RESULT`, which fails
#     the gate — deliberately, since a row that reports nothing must not read as
#     a row that found nothing.
#   * **The price is real and it is the gate's largest single row.** The figure
#     quoted in `docs/rungs/2026-08-14-dbgassert.md` §"the blindness" for THIS
#     script is 125 s warm (18 lanes); the 0.65 s in that same section belongs to
#     a different candidate — a debug `cargo test --workspace --lib` unit row,
#     which is NOT what this is. Re-measured in
#     `docs/rungs/2026-08-17-gatewire.md`.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
# THE RUN DIRECTORY IS PER RUN (board #3128, lane w-fenceb). It used to be one
# fixed `${TMPDIR:-/tmp}/c2rs-debug-lane`, shared by every tree, every lane and
# every concurrent invocation on the box — so two agents running this at once,
# from two worktrees, wrote one `list.txt`, one `<slug>/flags.txt` and one
# `<slug>/report.txt`, and each parsed its counts out of whichever report won.
#
# `scripts/mode_lane.sh`'s own header documents this EXACT defect, found and
# fixed there first: *"each lane overwriting the others' flags file and report,
# and the mismatch count was then parsed out of whichever report won. That is a
# false green by the same mechanism as a stale binary: the number comes from a
# run nobody asked for."* This script reintroduced it, and it is the instrument
# whose whole job is to catch what the release gate cannot.
#
# What it cost, concretely: a full sweep read two `/O1` lanes ONE MATCH LOW with
# `mismatch=0 panics=0 rc=0`, and the two low values were exactly the numbers a
# tree WITHOUT the lane's change produces. Four concurrent gate directories were
# live on the box at the time. See `work/w-fenceb/transient.sh`.
work="${C2RS_DEBUG_LANE_WORK:-${TMPDIR:-/tmp}/c2rs-debug-lane-$$}"
mkdir -p "$work"

# The binary under test must be a DEBUG build of THIS tree, which is the whole
# point — a release binary here grades nothing this script exists for.
#
# AND IT MUST BE A RUN-PRIVATE COPY, for the second half of #3128: this was the
# only one of the three standing lane runners that ran the LIVE
# `target/debug/c2rs` while `gate.sh` and `mode_lane.sh` both pin. A peer
# rebuilding the shared `target/` mid-sweep swapped the binary under a running
# grade — `scripts/harness_bin.sh` exists for exactly that and is now used here
# too, so the three runners agree.
cd "$repo_root"
cargo build -p c2-harness --bin c2rs >"$work/build.log" 2>&1 || {
    echo "FAIL: debug build of c2rs failed; see $work/build.log"
    exit 1
}
[ -x "$repo_root/target/debug/c2rs" ] || {
    echo "FAIL: no debug c2rs at $repo_root/target/debug/c2rs"; exit 1; }
cp "$repo_root/target/debug/c2rs" "$work/c2rs"
c2rs="$work/c2rs"

list="$work/list.txt"
: > "$list"
for f in "$repo_root"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$list"
done
total=$(wc -l < "$list" | tr -d ' ')

if [ $# -gt 0 ]; then
    printf '%s\t%s\n' "cli" "$*" > "$work/lanes.tsv"
else
    # `<slug> <flags...>` — comments and blank lines are not lanes.
    #
    # `C2RS_LANES` is honoured for the reason `scripts/gate.sh` has always
    # honoured it, and because this script is now a ROW of that gate: a
    # `gate.sh --lane Ox` run hands its ALREADY-FILTERED registry down here, so
    # the row grades the lanes the rest of the run walked. Hardcoding
    # `scripts/lanes.txt` made a one-lane gate run all eighteen lanes in this row
    # — the expensive direction, and a `graded/total` that disagreed with the
    # registry every other row was counted against.
    sed -e 's/#.*//' -e '/^[[:space:]]*$/d' "${C2RS_LANES:-$repo_root/scripts/lanes.txt}" \
        | awk '{ slug=$1; $1=""; sub(/^ /,""); printf "%s\t%s\n", slug, $0 }' \
        > "$work/lanes.tsv"
fi
n_lanes=$(wc -l < "$work/lanes.tsv" | tr -d ' ')
[ "$n_lanes" -gt 0 ] || { echo "FAIL: no lanes — an empty registry grades nothing"; exit 1; }

lane_jobs="${C2RS_DEBUG_LANE_LANES:-4}"
case "$lane_jobs" in ''|*[!0-9]*) lane_jobs=4 ;; esac
[ "$lane_jobs" -ge 1 ] || lane_jobs=1
echo "debug lane: $n_lanes lanes x $total fixtures at $lane_jobs lane(s) at once, binary=$c2rs"

# ---- PHASE 1: grade, `$lane_jobs` lanes at a time ------------------------------
#
# Each lane writes its OWN `result` file and nothing else is shared. Clear them
# FIRST, as their own pass, so a result file that EXISTS is necessarily from this
# run — `gate.sh` does exactly this for its lane logs, and for the same reason: a
# lane the loop never launches must not be graded from a previous run's leavings.
while IFS="$(printf '\t')" read -r slug flags; do
    [ -n "$slug" ] || continue
    rm -f "$work/$slug/result"
done < "$work/lanes.tsv"

running=0
while IFS="$(printf '\t')" read -r slug flags; do
    [ -n "$slug" ] || continue
    (
        d="$work/$slug"
        mkdir -p "$d"
        echo "$flags /GS- /c" > "$d/flags.txt"

        if "$c2rs" gap --list "$list" --flags-file "$d/flags.txt" --limit 1 --jobs 1 2>&1 \
                | grep -q "SKIP"; then
            echo "SKIP DEBUG-LANE-RESULT SKIP lane=$slug flags=[$flags] graded=0 total=$total match=0 mismatch=0 panics=0" > "$d/result"
            exit 0
        fi

        set +e
        "$c2rs" gap --list "$list" --flags-file "$d/flags.txt" --jobs "${C2RS_JOBS:-8}" \
            > "$d/report.txt" 2>&1
        rc=$?
        set -e
        panics=$(grep -ci 'panicked at' "$d/report.txt" || true)
        graded=$(sed -n 's|^GAP REPORT (\([0-9]*\) TUs.*|\1|p' "$d/report.txt" | head -1)
        match=$(sed -n 's|^  match  *\([0-9]*\) .*|\1|p' "$d/report.txt" | head -1)
        mm=$(sed -n 's|^  mismatch  *\([0-9]*\) .*|\1|p' "$d/report.txt" | head -1)
        graded=${graded:-0}; match=${match:-0}; mm=${mm:-0}

        verdict=PASS
        # A lane that graded nothing is a failure, not a pass — `lanes.txt`'s own
        # vacuity rule, restated here because this script does not go through
        # `mode_lane.sh`.
        [ "$graded" -eq "$total" ] || verdict=FAIL
        [ "$panics" -eq 0 ] || verdict=FAIL
        [ "$mm" -eq 0 ] || verdict=FAIL
        [ "$rc" -eq 0 ] || verdict=FAIL
        echo "$verdict DEBUG-LANE-RESULT $verdict lane=$slug flags=[$flags] graded=$graded total=$total match=$match mismatch=$mm panics=$panics rc=$rc" > "$d/result"
    ) &
    running=$((running + 1))
    if [ "$running" -ge "$lane_jobs" ]; then wait; running=0; fi
done < "$work/lanes.tsv"
wait

# ---- PHASE 2: read the results, IN REGISTRY ORDER, in THIS shell ---------------
#
# In this shell, so `fails` and `lanes_run` survive. In registry order, so the
# output is byte-identical to what the sequential version printed and
# `gate.sh`'s `debug_verdict` sees no change. And counted POSITIVELY: a lane
# whose result file is missing is a lane that died, which is a hard failure and
# not an absent contribution.
fails=0
lanes_run=0
seen=0
while IFS="$(printf '\t')" read -r slug flags; do
    [ -n "$slug" ] || continue
    d="$work/$slug"
    if [ ! -f "$d/result" ]; then
        echo "DEBUG-LANE-RESULT FAIL lane=$slug flags=[$flags] graded=0 total=$total match=0 mismatch=0 panics=0 rc=NO-RESULT"
        echo "    the lane produced no result file at all — it was killed, or its"
        echo "    subshell died before writing one. Its fixtures were never graded."
        fails=$((fails + 1))
        continue
    fi
    seen=$((seen + 1))
    _dl_v=$(cut -d' ' -f1 < "$d/result")
    cut -d' ' -f2- < "$d/result"
    [ "$_dl_v" = SKIP ] || lanes_run=$((lanes_run + 1))
    [ "$_dl_v" = FAIL ] && fails=$((fails + 1)) || true
    if [ -f "$d/report.txt" ] && grep -qi 'panicked at' "$d/report.txt"; then
        grep -A2 -i 'panicked at' "$d/report.txt" | head -12 | sed 's/^/    /'
    fi
done < "$work/lanes.tsv"

# The short-count reconciliation, stated as a count and never as a status.
if [ "$seen" -ne "$n_lanes" ]; then
    echo "FAIL: $seen of $n_lanes lanes reported a result. The rest were never graded"
    echo "  and this run establishes nothing about them."
    fails=$((fails + 1))
fi

echo "DEBUG-LANE-TOTAL lanes=$n_lanes ran=$lanes_run failed=$fails"
[ "$fails" -eq 0 ] || exit 1

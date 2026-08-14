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
# Exit status is non-zero if any lane panicked or reported a mismatch. One
# `DEBUG-LANE-RESULT` line per lane, plus one `DEBUG-LANE-TOTAL` line.
#
# NOT wired into `scripts/gate.sh`. Wiring it in is a shared-gate decision and
# belongs to whoever owns the gate; the price is measured and recorded in
# `docs/rungs/2026-08-14-dbgassert.md` §"the blindness".
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
    sed -e 's/#.*//' -e '/^[[:space:]]*$/d' "$repo_root/scripts/lanes.txt" \
        | awk '{ slug=$1; $1=""; sub(/^ /,""); printf "%s\t%s\n", slug, $0 }' \
        > "$work/lanes.tsv"
fi
n_lanes=$(wc -l < "$work/lanes.tsv" | tr -d ' ')
[ "$n_lanes" -gt 0 ] || { echo "FAIL: no lanes — an empty registry grades nothing"; exit 1; }

echo "debug lane: $n_lanes lanes x $total fixtures, binary=$c2rs"
fails=0
lanes_run=0
while IFS="$(printf '\t')" read -r slug flags; do
    d="$work/$slug"
    mkdir -p "$d"
    echo "$flags /GS- /c" > "$d/flags.txt"

    if "$c2rs" gap --list "$list" --flags-file "$d/flags.txt" --limit 1 --jobs 1 2>&1 | grep -q "SKIP"; then
        echo "DEBUG-LANE-RESULT SKIP lane=$slug flags=[$flags] graded=0 total=$total match=0 mismatch=0 panics=0"
        continue
    fi

    set +e
    "$c2rs" gap --list "$list" --flags-file "$d/flags.txt" --jobs "${C2RS_JOBS:-8}" \
        > "$d/report.txt" 2>&1
    rc=$?
    set -e
    lanes_run=$((lanes_run + 1))
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
    [ "$verdict" = PASS ] || fails=$((fails + 1))
    echo "DEBUG-LANE-RESULT $verdict lane=$slug flags=[$flags] graded=$graded total=$total match=$match mismatch=$mm panics=$panics rc=$rc"
    if [ "$panics" -gt 0 ]; then
        grep -A2 -i 'panicked at' "$d/report.txt" | head -12 | sed 's/^/    /'
    fi
done < "$work/lanes.tsv"

echo "DEBUG-LANE-TOTAL lanes=$n_lanes ran=$lanes_run failed=$fails"
[ "$fails" -eq 0 ] || exit 1

#!/bin/sh
# Run every fixture through the differential at a chosen optimization mode.
#
# `c2rs diff` hardcodes the default `/Ox /GS- /c` capture profile, so the fixture
# suite has only ever verified the port against **`/Ox`** — while the dc3 workload
# compiles `/O1`, which emits different code for the same source
# (`docs/OPT_MODE.md`). This is the missing lane: it drives the fixtures through
# `c2rs gap`, which does take `--flags-file`, so the same corpus can be graded in
# either mode.
#
# Usage:  scripts/mode_lane.sh [/O1|/Ox|/O2|/Od] [extra cl flags...]
#         scripts/mode_lane.sh /O1
#
# `mismatch` is the alarm: it means the port emitted bytes for a mode and they were
# wrong. `codegen-gap` is the honest refusal — a shape not yet re-targeted for that
# mode. Exits non-zero on any mismatch.
#
# THE LANE SET IS `scripts/lanes.txt` AND THE GATE IS `scripts/gate.sh`. This
# script runs ONE lane; running the right set of them is not a thing to remember.
#
# Every exit path prints exactly one `LANE-RESULT` line, which is the lane's whole
# machine-readable contract with `gate.sh`:
#
#     LANE-RESULT <PASS|FAIL|SKIP> flags=[…] graded=<n> total=<n> match=<n> mismatch=<n>
#
# The gate requires that line to be present and re-derives the verdict from its
# fields; it does not take a zero exit status as evidence a lane ran. A lane that
# dies before printing it is a lane with NO RESULT, which the gate reports as a
# failure and never as a pass.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
mode="${1:-/O1}"
[ $# -gt 0 ] && shift
# The run directory is PER MODE. It used to be one shared `/tmp/c2rs-mode-lane`
# holding `flags.txt`, `list.txt` and `report.txt` — so running the four lanes
# concurrently (the obvious thing to do, they are independent) had each lane
# overwriting the others' flags file and report, and the mismatch count was then
# parsed out of whichever report won. That is a false green by the same mechanism
# as a stale binary: the number comes from a run nobody asked for.
work="${C2RS_MODE_LANE_WORK:-/tmp/c2rs-mode-lane}/$(printf '%s%s' "$mode" "$*" | tr -c 'A-Za-z0-9' '-')"
mkdir -p "$work"

# Build unconditionally and run a RUN-PRIVATE COPY — see `scripts/harness_bin.sh`
# for the stale-binary and republished-under-a-running-gate failures this closes.
# The four mode lanes are part of the merge gate; a lane that passes because it
# ran yesterday's binary is exactly the false green the gate exists to prevent.
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$work"
c2rs="$C2RS_PINNED"
flags="$work/flags.txt"
list="$work/list.txt"
echo "$mode /GS- /c $*" > "$flags"

# `cl.exe` runs under wibo, so the sources have to be named as `Z:\…` paths.
: > "$list"
for f in "$repo_root"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$list"
done
total=$(wc -l < "$list")

lane_flags="$(cat "$flags")"

if "$c2rs" gap --list "$list" --flags-file "$flags" --limit 1 --jobs 1 2>&1 | grep -q "SKIP"; then
    echo "SKIP: toolchain absent — the lane would be vacuous"
    echo "LANE-RESULT SKIP flags=[$lane_flags] graded=0 total=$total match=0 mismatch=0"
    exit 0
fi

echo "grading $total fixtures at $mode"
out="$work/report.txt"
"$c2rs" gap --list "$list" --flags-file "$flags" --jobs "${C2RS_JOBS:-8}" \
    --jsonl "$work/scan.jsonl" > "$out" 2>&1 || true
sed -n '/GAP REPORT/,$p' "$out"

bucket() { sed -n "s|^  $1  *\([0-9]*\) .*|\1|p" "$out" | head -1; }
mm=$(bucket mismatch); mm=${mm:-0}
match=$(bucket match);  match=${match:-0}

# ---- vacuity guard ------------------------------------------------------------
#
# Everything below reads a number out of the report with `sed`, and a number that
# is NOT THERE parses as zero. So a lane in which nothing was graded at all passed
# every check here: `mismatch` absent -> 0 -> exit 0, with a green line in the gate
# table and no denominator anywhere to contradict it. This lane shipped that hole
# for its whole existence; `sweep_mode.sh` had already been bitten by the identical
# mechanism and grown the guard, which is exactly the "one rule, two
# implementations" shape `docs/GAPS.md` §6 keeps recording.
#
# The SKIP pre-check above does not cover it. SKIP is the toolchain being ABSENT;
# this is the toolchain being present and every TU failing to capture — which is
# what a relative outdir, an exhausted tmpfs inode table (`df -i`, not `df -h`) or
# a bad flag string all look like. So it is checked POSITIVELY: the run must have
# GRADED something. Never as an enumeration of the ways a run can come back empty,
# because the next empty run will be empty in a way nobody enumerated.
graded=$mm
for cls in match codegen-gap vocab-gap port-error; do
    n=$(bucket "$cls")
    graded=$((graded + ${n:-0}))
done
capfail=$(bucket capture-fail)
if [ "$graded" -eq 0 ]; then
    echo
    echo "VACUOUS LANE at [$lane_flags]: $total fixtures submitted, NONE graded"
    echo "(capture-fail ${capfail:-?}). Every check below reads a number that is not"
    echo "in the report and parses it as 0, so this would otherwise have passed."
    sed -n '/top capture-fail reasons/,/^$/p' "$out" | head -8
    echo "LANE-RESULT FAIL flags=[$lane_flags] graded=0 total=$total match=0 mismatch=0"
    exit 1
fi

if [ "$mm" -ne 0 ]; then
    echo
    echo "MISMATCH at $mode — the port emitted wrong bytes, not a gap:"
    grep -F "mismatch" "$out" | grep -v "^  mismatch" || true
    echo "LANE-RESULT FAIL flags=[$lane_flags] graded=$graded total=$total match=$match mismatch=$mm"
    exit 1
fi

echo "LANE-RESULT PASS flags=[$lane_flags] graded=$graded total=$total match=$match mismatch=0"

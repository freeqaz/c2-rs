#!/bin/sh
# Run the GENERATED sweep cases at an arbitrary compile mode — including `/EHsc`.
#
# ---- why this exists -----------------------------------------------------------
#
# Two instruments in this repo grade generated cases, and until now neither could
# see the exception-handling surface at all:
#
#   * `expr_sweep.sh` drives `c2rs diff`, and `c2rs diff` hardcodes the capture
#     profile `/Ox /GS- /c` (`c2-reference/src/lib.rs`). No `/EH`, ever, at any
#     invocation. So every generated axis — the instrument that has found FOUR
#     live mis-emits the hand-written corpus never found — has only been graded
#     with exceptions off.
#
#   * `mode_lane.sh` does take arbitrary flags and does run `/EHsc` lanes, but it
#     grades the `fixtures/cpp/` corpus. The generated cases are not in it.
#
# The intersection "generated case × `/EHsc`" was therefore empty, and it is
# exactly where the port has an acceptance surface: on the dc3 workload the port
# admits **35,964 `eh-bare` functions in class** (`empty-dtor-delegation`,
# `empty-dtor-member`, `empty-ctor-base`, …), and those shapes only carry EH
# markers when compiled `/EHsc`. Compiled `/Ox` they are ordinary functions and
# the marker path is never exercised.
#
# This is the same defect WEC found one level up, restated: **every standing mode
# lane compiled without `/EH`, which made the entire EH surface vacuous.** Two
# `/EHsc` fixture lanes closed that for fixtures. This closes it for the generated
# axes, which are the ones that actually find things.
#
# The general rule, and it is the third time it has paid: a green run is sound
# only over the configurations it was RUN at. A flag that no lane varies is not
# "verified as irrelevant" — it is untested, and it looks identical to verified
# from the outside.
#
# ---- usage ---------------------------------------------------------------------
#
#   scripts/sweep_mode.sh [mode] [outdir] [max-cases] [extra cl flags...]
#
#   scripts/sweep_mode.sh /EHsc                      # the case this was written for
#   scripts/sweep_mode.sh /EHsc /tmp/swm-$$ 400      # a quick subset
#   scripts/sweep_mode.sh /Od  /tmp/swm-od           # any other mode
#
# `mode` is spliced into `<mode> /O1 /GS- /c`. `/O1` because that is what the real
# workload compiles (`docs/OPT_MODE.md`); pass `/Ox` explicitly as an extra flag if
# you want the `c2rs diff` profile instead.
#
# A MISMATCH is an ALARM, not a gap — the port emitted bytes and they were wrong.
# Either fix the lowering or tighten the gate until it refuses. `codegen-gap` is
# the honest refusal and is expected for most cases here. Exits non-zero on any
# mismatch.
#
# Needs the toolchain (see CLAUDE.md); without it the run reports SKIP rather than
# a vacuous pass, because a vacuous pass is the worst failure mode this repo has.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
mode="${1:-/EHsc}"
[ $# -gt 0 ] && shift
out="${1:-/tmp/c2rs-sweep-mode}"
[ $# -gt 0 ] && shift
limit="${1:-0}"
[ $# -gt 0 ] && shift

mkdir -p "$out"

# Same outdir lock as `expr_sweep.sh`, for the same reason: this driver `rm -f`s
# the whole case set before regenerating it, so two concurrent runs against one
# outdir delete each other's cases mid-grade and BOTH results are meaningless
# while looking perfectly ordinary. `mkdir` is the lock because it is atomic on
# every filesystem this runs on.
_lock="$out/.sweep.lock"
if ! mkdir "$_lock" 2>/dev/null; then
    echo "REFUSING: another sweep holds $out (lock: $_lock)." >&2
    echo "  Two sweeps in one outdir delete each other's cases and both results" >&2
    echo "  are silently wrong. Pass a private outdir:" >&2
    echo "      scripts/sweep_mode.sh $mode /tmp/c2rs-sweep-mode-\$\$" >&2
    echo "  If no sweep is running, the previous one was killed: rmdir '$_lock'" >&2
    exit 2
fi
trap 'rmdir "$_lock" 2>/dev/null || true' EXIT INT TERM

rm -f "$out"/*.cpp "$out"/cases.txt "$out"/list.txt 2>/dev/null || true

# Build unconditionally and run a RUN-PRIVATE COPY — never `target/release/c2rs`
# directly. See `scripts/harness_bin.sh` for the stale-binary and
# republished-under-a-running-gate failures this closes; the false-GREEN direction
# is the one that matters.
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$out"
c2rs="$C2RS_PINNED"

# The SAME loader `expr_sweep.sh` and `cross_sweep.py` use. Deliberately not a
# second copy of the enumeration: "one rule, two implementations" is the shape
# `docs/GAPS.md` §6 keeps recording, and a fragment added for one driver must be
# graded by all of them.
python3 "$repo_root/scripts/sweep_gen.py" "$out" "$repo_root/scripts/sweep.d"

ls "$out"/*.cpp | sort > "$out/cases.txt"
total=$(wc -l < "$out/cases.txt")
if [ "$limit" -gt 0 ] 2>/dev/null; then
    head -n "$limit" "$out/cases.txt" > "$out/cases.run"
else
    cp "$out/cases.txt" "$out/cases.run"
fi
run=$(wc -l < "$out/cases.run")

flags="$out/flags.txt"
echo "$mode /O1 /GS- /c $*" > "$flags"

# `cl.exe` runs under wibo, so the sources have to be named as `Z:\…` paths.
: > "$out/list.txt"
while read -r f; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$out/list.txt"
done < "$out/cases.run"

if "$c2rs" gap --list "$out/list.txt" --flags-file "$flags" --limit 1 --jobs 1 2>&1 \
    | grep -q "SKIP"; then
    echo "SKIP: toolchain absent — the sweep would be vacuous"
    exit 0
fi

echo "grading $run of $total generated cases at [$(cat "$flags")]"
report="$out/report.txt"
"$c2rs" gap --list "$out/list.txt" --flags-file "$flags" --jobs "${C2RS_JOBS:-8}" \
    --jsonl "$out/scan.jsonl" > "$report" 2>&1 || true
sed -n '/GAP REPORT/,$p' "$report"

mm=$(sed -n 's/^  mismatch  *\([0-9]*\) .*/\1/p' "$report" | head -1)
[ "${mm:-0}" -eq 0 ] || {
    echo
    echo "MISMATCH at [$(cat "$flags")] — the port emitted wrong bytes, not a gap."
    echo "This is an ALARM and it outranks every other piece of work. The failing"
    echo "cases are in $out; grade one on its own with:"
    echo "    $c2rs gap --list <one-case-list> --flags-file $flags --jobs 1"
    grep -F "mismatch" "$report" | grep -v "^  mismatch" || true
    exit 1
}

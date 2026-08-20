#!/bin/bash
#
# partest.sh — run the workspace test suite with the test BINARIES in parallel.
#
# ## Why this exists
#
# `cargo test --workspace` runs the test binaries **strictly one at a time**.
# Within one binary libtest threads; across binaries nothing overlaps. Measured
# at master `e82c9ede6` (lane `w-suitecost`): the sum of the 48 targets' own
# "finished in" numbers was **199.2 s** against a **200 s** wall — a ratio of
# 0.996. There is no overlap at all, and three binaries are 89 % of the total
# (`cli_flags` 82.8 s, `census_gate` 64.8 s, `fixture_profiles` 29.7 s). Those
# three are independent: they spawn the toolchain, they do not share mutable
# state, and the box has 32 cores. Running them concurrently is free wall clock.
#
# ## What it does NOT do
#
# **It does not narrow what runs.** Every target `cargo test --workspace` would
# run is run here, and the executed-test set is compared **by name**, not by
# count. A runner that is faster because a binary silently dropped out is the
# #3219/#3231 defect — a mutant reads GREEN because its catcher never executed,
# with a clean suite, the right target count and the right exit code. So:
#
#   * The target list is taken from `cargo test --no-run --message-format=json`
#     filtered on `profile.test == true` — the same predicate cargo itself uses
#     to decide what to run — plus the doc-test targets, which that JSON does
#     not report and which `cargo test --workspace` does run.
#   * `--verify` re-runs the suite serially through cargo and diffs the two
#     `<target> :: <test name> :: <verdict>` lists. Identical or it fails.
#
# ## Usage
#
#   scripts/partest.sh [--jobs N] [--test-threads T] [--out DIR]
#                      [--report-time] [--no-doc] [--portable]
#                      [--] [libtest args...]
#
#   --jobs N          test binaries in flight (default 8)
#   --test-threads T  libtest threads inside each binary (default: libtest's own,
#                     i.e. one per core). Passed through unchanged.
#   --out DIR         where per-target logs and the name list go
#                     (default work/partest)
#   --report-time     ask libtest for per-test durations (needs RUSTC_BOOTSTRAP,
#                     which this script sets for the child only)
#   --no-doc          skip the doc-test job (it is one cargo invocation)
#   --portable        do NOT demand a toolchain (see below). Sets
#                     C2RS_REQUIRE_TOOLCHAIN=0 explicitly, so the choice is
#                     visible in the log rather than being an absent variable.
#
# Everything after `--` is handed to each test binary verbatim, so
# `scripts/partest.sh -- --exact some::test` works.
#
# ## `C2RS_REQUIRE_TOOLCHAIN=1` IS THE DEFAULT HERE — board #3247's closure
#
# Until 2026-08-20 this script said *"this script does not set it for you"*, and
# neither did anything else: `grep -rn REQUIRE_TOOLCHAIN crates scripts` found
# the variable **armed and fired by nothing**. Board **#3247** closed with, in
# those words, *"STILL OPEN … NOTHING SETS THE VARIABLE."* The measurement it
# was opened on: `cargo test --workspace --release` reads **1,660 / 0 / 43** in a
# provisioned worktree and **byte-identically 1,660 / 0 / 43** in one with no
# `compilers/`, because 132 of ~179 integration tests print `SKIP: toolchain
# absent` and PASS — and cargo swallows that line for a passing test. Two
# registered REDs read GREEN that way (#3219, #3231), with a clean suite, the
# right target count and the right exit code. A fresh `git worktree add` has no
# `compilers/` (it is gitignored and does not follow a new worktree), so this is
# not a hypothetical: it is the default state of every new lane's tree.
#
# **The demand is unconditional, and that is the point.** This script does NOT
# probe for `compilers/` first: a shell-side "does the toolchain resolve"
# predicate would be a SECOND implementation of a rule `Toolchain::locate()`
# already owns, and "one rule, two implementations" is the hazard shape this
# repo keeps paying for (GAPS §6, board #3134). So the script carries only the
# DEMAND, and `crates/c2-harness/tests/require_toolchain.rs` remains the single
# authority on whether the demand is met. A toolchain-less run therefore fails
# on exactly one named test —
# `require_toolchain::a_run_that_claims_to_grade_must_have_a_toolchain_to_grade_with`
# — instead of passing green having graded nothing.
#
# An explicit `C2RS_REQUIRE_TOOLCHAIN` already in the environment is HONOURED
# and never overwritten: the caller has spoken. `--portable` is the opt-out for
# a run that deliberately grades nothing (the portable lane is entitled to be
# empty — that is `gate.sh --require-graded`'s own argument, from the other
# side).
#
# What this does NOT close, stated rather than papered over: a **partially**
# provisioned run — `compilers/` present but `strace` or
# `i686-w64-mingw32-gcc` absent — still skips a subset silently, because
# `Toolchain::locate()` succeeds and the per-test `has_strace()` / `has_mingw()`
# guards are what skip. The demand is total over "did this run have a toolchain
# at all", not over "did every gated test execute".
#
# Outputs, all under `--out`:
#
#   <target>.log     that binary's own stdout+stderr, verbatim
#   names.txt        `<target> :: <test> :: <verdict>`, sorted — the identity set
#   summary.txt      per-target passed/failed/ignored/seconds, and the totals
#   durations.tsv    per-target seconds, reused next run to schedule
#                    longest-first (LPT). Advisory only: correctness never
#                    depends on it, and a missing file just means cargo's order.

set -uo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

JOBS=8
TEST_THREADS=""
OUT="work/partest"
REPORT_TIME=0
DO_DOC=1
VERIFY=0
PORTABLE=0
PASSTHRU=()

while [ $# -gt 0 ]; do
    case "$1" in
        --jobs) JOBS="$2"; shift 2 ;;
        --jobs=*) JOBS="${1#*=}"; shift ;;
        --test-threads) TEST_THREADS="$2"; shift 2 ;;
        --test-threads=*) TEST_THREADS="${1#*=}"; shift ;;
        --out) OUT="$2"; shift 2 ;;
        --out=*) OUT="${1#*=}"; shift ;;
        --report-time) REPORT_TIME=1; shift ;;
        --no-doc) DO_DOC=0; shift ;;
        --portable) PORTABLE=1; shift ;;
        --verify) VERIFY=1; shift ;;
        --) shift; PASSTHRU=("$@"); break ;;
        # The window moves with the header. It was `3,60p` when the header ended
        # at line 60; the #3247 block pushed the outputs list to 91, and a stale
        # window would have silently stopped printing the usage lines it exists
        # to print. Ends at the `Outputs` heading.
        -h|--help) sed -n '3,101p' "$0"; exit 0 ;;
        *) echo "partest.sh: unknown option $1" >&2; exit 2 ;;
    esac
done

case "$JOBS" in ''|*[!0-9]*) echo "partest.sh: --jobs wants a number" >&2; exit 2 ;; esac
[ "$JOBS" -ge 1 ] || { echo "partest.sh: --jobs must be >= 1" >&2; exit 2; }

mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"

# ---------------------------------------------------------------------------
# THE TOOLCHAIN DEMAND (board #3247). See the header block for the measurement.
# ---------------------------------------------------------------------------
# Precedence, and it is deliberate: an explicit value in the environment beats
# everything (the caller has spoken), then `--portable`, then the default. The
# state is PRINTED on every run — an absent variable is the thing this closes,
# so its replacement must not itself be silent.
if [ -n "${C2RS_REQUIRE_TOOLCHAIN+set}" ]; then
    REQUIRE_SRC="the environment"
elif [ "$PORTABLE" -eq 1 ]; then
    C2RS_REQUIRE_TOOLCHAIN=0
    REQUIRE_SRC="--portable"
else
    C2RS_REQUIRE_TOOLCHAIN=1
    REQUIRE_SRC="partest.sh's default (board #3247)"
fi
export C2RS_REQUIRE_TOOLCHAIN
case "$C2RS_REQUIRE_TOOLCHAIN" in
    ''|0)
        echo "partest.sh: C2RS_REQUIRE_TOOLCHAIN=$C2RS_REQUIRE_TOOLCHAIN from $REQUIRE_SRC —"
        echo "            this run makes NO CLAIM to have graded anything against real c2.dll."
        echo "            A green suite here is compatible with 132 integration tests having"
        echo "            printed 'SKIP: toolchain absent' and passed." ;;
    *)
        echo "partest.sh: C2RS_REQUIRE_TOOLCHAIN=$C2RS_REQUIRE_TOOLCHAIN from $REQUIRE_SRC —"
        echo "            a run with no toolchain will FAIL on"
        echo "            require_toolchain::a_run_that_claims_to_grade_must_have_a_toolchain_to_grade_with"
        echo "            rather than passing green having graded nothing. Use --portable to opt out." ;;
esac

# ---------------------------------------------------------------------------
# 1. Build, and enumerate exactly what cargo would run.
# ---------------------------------------------------------------------------
# `profile.test == true` is cargo's own run predicate. It has to be read out of
# the *profile* object and not off the whole line: the artifact JSON also
# carries `target.test`, which is true for the plain `c2rs` binary that cargo
# builds for `CARGO_BIN_EXE_c2rs` and never runs. Matching the line as a whole
# picks that up and would add a 44th "target" that cargo does not execute.
BUILD_LOG="$OUT/build.log"
if ! cargo test --workspace --release --no-run --message-format=json \
        >"$OUT/artifacts.json" 2>"$BUILD_LOG"; then
    echo "partest.sh: build failed — see $BUILD_LOG" >&2
    tail -40 "$BUILD_LOG" >&2
    exit 1
fi

mapfile -t EXES < <(awk '
  index($0, "\"reason\":\"compiler-artifact\"") == 0 { next }
  {
    p = index($0, "\"profile\":")
    if (p == 0) next
    prof = substr($0, p)
    e = index(prof, "}")
    if (e == 0) next
    prof = substr(prof, 1, e)
    if (index(prof, "\"test\":true") == 0) next
    q = index($0, "\"executable\":\"")
    if (q == 0) next
    rest = substr($0, q + 14)
    e2 = index(rest, "\"")
    if (e2 < 2) next
    print substr(rest, 1, e2 - 1)
  }
' "$OUT/artifacts.json")

if [ "${#EXES[@]}" -eq 0 ]; then
    echo "partest.sh: enumerated ZERO test binaries. That is a broken runner, not" >&2
    echo "  an empty suite — refusing to report success. See $OUT/artifacts.json" >&2
    exit 1
fi

label_of() { local b; b="$(basename "$1")"; echo "${b%-*}"; }

# ---------------------------------------------------------------------------
# 2. Longest-first ordering (LPT). Advisory.
# ---------------------------------------------------------------------------
# With a pool of N and one binary that is 40 % of the total, starting that one
# last costs its whole duration. Previous run's per-target seconds are reused as
# the estimate; unknown targets sort first-come after the known ones.
ORDER_FILE="$OUT/durations.tsv"
declare -a ORDERED
if [ -s "$ORDER_FILE" ]; then
    while IFS= read -r exe; do ORDERED+=("$exe"); done < <(
        for e in "${EXES[@]}"; do
            l="$(label_of "$e")"
            d="$(awk -v L="$l" '$1==L{print $2; exit}' "$ORDER_FILE")"
            printf '%s\t%s\n' "${d:-0}" "$e"
        done | sort -rn -k1,1 | cut -f2
    )
else
    ORDERED=("${EXES[@]}")
fi

# ---------------------------------------------------------------------------
# 3. Run the pool.
# ---------------------------------------------------------------------------
CHILD_ARGS=()
[ -n "$TEST_THREADS" ] && CHILD_ARGS+=("--test-threads=$TEST_THREADS")
if [ "$REPORT_TIME" -eq 1 ]; then
    export RUSTC_BOOTSTRAP=1
    CHILD_ARGS+=("-Z" "unstable-options" "--report-time")
fi
[ "${#PASSTHRU[@]}" -gt 0 ] && CHILD_ARGS+=("${PASSTHRU[@]}")

run_one() {
    local exe="$1" label="$2" t0 t1
    t0=$(date +%s.%N)
    "$exe" ${CHILD_ARGS[@]+"${CHILD_ARGS[@]}"} >"$OUT/$label.log" 2>&1
    local rc=$?
    t1=$(date +%s.%N)
    awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.2f\n", b-a}' >"$OUT/$label.secs"
    echo "$rc" >"$OUT/$label.rc"
}

run_doc() {
    local t0 t1
    t0=$(date +%s.%N)
    cargo test --workspace --release --doc --no-fail-fast \
        >"$OUT/DOC.log" 2>&1
    local rc=$?
    t1=$(date +%s.%N)
    awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.2f\n", b-a}' >"$OUT/DOC.secs"
    echo "$rc" >"$OUT/DOC.rc"
}

rm -f "$OUT"/*.log "$OUT"/*.secs "$OUT"/*.rc 2>/dev/null
WALL0=$(date +%s.%N)

inflight=0
if [ "$DO_DOC" -eq 1 ]; then
    run_doc &
    inflight=$((inflight + 1))
fi
for exe in "${ORDERED[@]}"; do
    while [ "$inflight" -ge "$JOBS" ]; do
        wait -n
        inflight=$((inflight - 1))
    done
    run_one "$exe" "$(label_of "$exe")" &
    inflight=$((inflight + 1))
done
wait
WALL1=$(date +%s.%N)
WALL=$(awk -v a="$WALL0" -v b="$WALL1" 'BEGIN{printf "%.1f", b-a}')

# ---------------------------------------------------------------------------
# 4. Aggregate — names first, counts second.
# ---------------------------------------------------------------------------
# The name list is the artifact this runner is graded on. `<target> :: <test>
# :: <verdict>`, sorted, and it must equal what a serial `cargo test` produces.
: >"$OUT/names.txt"
: >"$OUT/summary.txt"
: >"$OUT/durations.tsv.new"

# `suite.log` is a CARGO-SHAPED concatenation, and it exists because there is
# already a consumer of the workspace suite in `scripts/`: `status.sh`'s
# `collect_tests` runs the serial `cargo test --workspace --release` for
# STATUS.md's `tests` row, and `status.sh --tests-log <file>` will accept a log
# instead — subject to four checks, of which two are structural (every target
# cargo LAUNCHED must have REPORTED, and the log must END on a `test result:`
# line). Emitting the banner cargo emits, verbatim in shape, means the fast
# runner can feed that row without a second parsing vocabulary. See this rung's
# §8 item 5 for why the wiring itself is not done here.
: >"$OUT/suite.log"

TOT_P=0; TOT_F=0; TOT_I=0; TOT_T=0; SUM_SECS=0; WORST_RC=0
for f in "$OUT"/*.rc; do
    label="$(basename "$f" .rc)"
    rc="$(cat "$f")"
    secs="$(cat "$OUT/$label.secs" 2>/dev/null || echo 0)"
    [ "$rc" -ne 0 ] && WORST_RC=1
    printf '%s\t%s\n' "$label" "$secs" >>"$OUT/durations.tsv.new"

    # `DOC.log` already carries its own `Doc-tests <crate>` banners, one per
    # crate, so it is copied through unbannered; a binary gets the one cargo
    # would have printed for it.
    if [ "$label" != "DOC" ]; then
        exepath="$label"
        for e in "${EXES[@]}"; do
            [ "$(label_of "$e")" = "$label" ] && { exepath="$e"; break; }
        done
        printf '     Running %s (%s)\n' "$label" "$exepath" >>"$OUT/suite.log"
    fi
    cat "$OUT/$label.log" >>"$OUT/suite.log"

    # Doc-test output carries a `Doc-tests <crate>` banner per crate; give each
    # its own label so a doc test cannot be confused with a unit test of the
    # same name.
    awk -v L="$label" '
      /^ *Doc-tests /  { sub(/^ *Doc-tests +/, ""); cur = "doc-" $0; next }
      /^test result:/  { next }
      /^test .* \.\.\. / {
        line = $0
        sub(/^test /, "", line)
        i = index(line, " ... ")
        name = substr(line, 1, i - 1)
        verdict = substr(line, i + 5)
        sub(/ <[0-9.]+s>$/, "", verdict)     # --report-time suffix
        sub(/,.*$/, "", verdict)             # "ignored, <reason>"
        gsub(/^[ \t]+|[ \t]+$/, "", verdict)
        printf "%s :: %s :: %s\n", (cur == "" ? L : cur), name, verdict
      }
    ' "$OUT/$label.log" >>"$OUT/names.txt"

    read -r p fl ig nt < <(awk '
      /^test result:/ { for (i = 1; i <= NF; i++) {
          if ($i == "passed;")  p  += $(i-1)
          if ($i == "failed;")  fl += $(i-1)
          if ($i == "ignored;") ig += $(i-1)
        }
        n++
      }
      END { print p+0, fl+0, ig+0, n+0 }' "$OUT/$label.log")
    TOT_P=$((TOT_P + p)); TOT_F=$((TOT_F + fl)); TOT_I=$((TOT_I + ig)); TOT_T=$((TOT_T + nt))
    SUM_SECS=$(awk -v a="$SUM_SECS" -v b="$secs" 'BEGIN{printf "%.2f", a+b}')
    printf '%-34s %6s s  rc=%s  %s passed %s failed %s ignored (%s target(s))\n' \
        "$label" "$secs" "$rc" "$p" "$fl" "$ig" "$nt" >>"$OUT/summary.txt"
done

sort -o "$OUT/names.txt" "$OUT/names.txt"
mv "$OUT/durations.tsv.new" "$ORDER_FILE"

{
    echo
    echo "partest: jobs=$JOBS test-threads=${TEST_THREADS:-libtest-default} doc=$DO_DOC"
    echo "partest: wall ${WALL}s   sum-of-target-walls ${SUM_SECS}s   overlap $(awk -v s="$SUM_SECS" -v w="$WALL" 'BEGIN{printf "%.2fx", s/w}')"
    echo "partest: $TOT_P passed; $TOT_F failed; $TOT_I ignored; $TOT_T targets; $(wc -l <"$OUT/names.txt") named results"
    echo "partest: load $(uptime | sed 's/.*load average: //')"
} | tee -a "$OUT/summary.txt"

sort "$OUT/summary.txt" >/dev/null   # no-op; summary is read in file order

# ---------------------------------------------------------------------------
# 5. --verify: the by-name identity proof against a serial cargo run.
# ---------------------------------------------------------------------------
if [ "$VERIFY" -eq 1 ]; then
    echo "partest: --verify — running the serial cargo suite for the by-name diff"
    cargo test --workspace --release --no-fail-fast >"$OUT/serial.log" 2>&1
    SERIAL_RC=$?
    awk '
      / *Running .*\(/ { s = $0; sub(/.*\(/, "", s); sub(/\).*/, "", s)
                         n = split(s, parts, "/"); b = parts[n]
                         sub(/-[^-]*$/, "", b); cur = b; next }
      / *Doc-tests /   { s = $0; sub(/^ *Doc-tests +/, ""); cur = "doc-" $0; next }
      /^test result:/  { next }
      /^test .* \.\.\. / {
        line = $0; sub(/^test /, "", line)
        i = index(line, " ... ")
        name = substr(line, 1, i - 1)
        verdict = substr(line, i + 5)
        sub(/ <[0-9.]+s>$/, "", verdict)
        sub(/,.*$/, "", verdict)
        gsub(/^[ \t]+|[ \t]+$/, "", verdict)
        printf "%s :: %s :: %s\n", cur, name, verdict
      }
    ' "$OUT/serial.log" | sort >"$OUT/names-serial.txt"

    if diff -u "$OUT/names-serial.txt" "$OUT/names.txt" >"$OUT/names.diff"; then
        echo "partest: BY-NAME IDENTICAL — $(wc -l <"$OUT/names.txt") results, serial and parallel"
    else
        echo "partest: BY-NAME DIFFERENT — see $OUT/names.diff" >&2
        head -40 "$OUT/names.diff" >&2
        WORST_RC=1
    fi
    [ "$SERIAL_RC" -ne 0 ] && WORST_RC=1
fi

if [ "$TOT_F" -ne 0 ] || [ "$WORST_RC" -ne 0 ]; then
    echo "partest: FAILED ($TOT_F failing test(s))" >&2
    exit 1
fi
exit 0

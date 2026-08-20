#!/bin/sh
# THE STATUS COLLECTOR — one command that answers "where is this project".
#
# ---- why this exists -----------------------------------------------------------
#
# Answering "where are we at" took, on 2026-08-02, four separate invocations, a
# guess at `gap`'s three required arguments, and a read through 7,976 lines of
# ROADMAP to find out which of the numbers in it were still current. Every number
# turned out to be current — but *establishing* that cost more than producing it,
# and the only reason it was cheap enough to do at all is that the metrics were
# already there. What was missing was the one command that runs them together.
#
# So this script is not a new instrument. It runs the instruments that already
# exist, in one pass, and renders `docs/STATUS.md`'s generated block. The doc is
# the cache; this is the thing that refills it.
#
# ---- what this collector promises ----------------------------------------------
#
# The same promise `gate.sh` makes, for the same reason: **a metric that did not
# produce a value prints NO-RESULT, never 0 and never blank.** This project has
# recorded eight instruments reporting green from an absence, including one whose
# every check `sed`-ed a number out of a report and read the missing number as 0.
# A status block is exactly the artifact that failure mode likes — a table of
# zeroes reads as "measured and nothing there", which is indistinguishable from
# "never ran" once it is pasted into a doc and the terminal is closed.
#
# Concretely:
#
#   * Every metric is collected by a function that prints exactly one
#     `STATUS-METRIC <key> <value>` line on every exit path. The renderer walks
#     the METRIC REGISTRY below, not the set of lines that happen to exist, and
#     a registered key with no line renders NO-RESULT.
#   * A zero exit status is never accepted as evidence a metric ran.
#   * Toolchain-absent is its own verdict (`SKIP: toolchain absent`), exits 0 per
#     CLAUDE.md, and cannot be mistaken for a measurement.
#   * The block is stamped with the pinned binary's sha and the tree HEAD (via
#     `pin_harness`, scripts/harness_bin.sh), so "which code produced this number"
#     is answerable from the doc rather than reconstructed later.
#
# ---- what it deliberately does NOT do ------------------------------------------
#
# It does not run `scripts/gate.sh` (12 lanes), `expr_sweep.sh` or `cross_sweep.sh`.
# Those are the MERGE gate — they answer "is this tree safe to land", which is a
# different question from "where is this project", and they cost minutes rather
# than seconds. STATUS.md links them; it does not inline them. Run the gate before
# landing, run this to report.
#
# ---- usage ---------------------------------------------------------------------
#
#   scripts/status.sh                 collect and print the markdown block
#   scripts/status.sh --write         also rewrite docs/STATUS.md's generated block
#   scripts/status.sh --check         validate the registry and parsers; no toolchain
#   scripts/status.sh --raw           print the STATUS-METRIC lines, unrendered
#   scripts/status.sh --jobs N        gap scan concurrency (default 16)
#   scripts/status.sh --tests-log F   read the workspace-test row out of a log the
#                                     caller already produced, instead of running
#                                     `cargo test --workspace --release` again
#                                     (~206 s idle, ~300 s under load). The log is
#                                     accepted only if it is FRESHER than every
#                                     input the suite reads and INTERNALLY
#                                     RECONCILED — see collect_tests below. It is
#                                     never a cache: without this flag the suite
#                                     runs, and a log that fails any check renders
#                                     NO-RESULT with the reason, never a number.
#
# Environment:
#   C2RS_DC3      the dc3-decomp source tree     (default: <repo>/../dc3-decomp)
#   C2RS_WORKLOAD the 878-TU workload list dir   (default: <repo>/work/dc3-workload)
#                 `work/dc3-workload` is gitignored, so a fresh WORKTREE does not
#                 have it and every gap-derived metric renders NO-RESULT — 19 of
#                 the 23. `C2RS_DC3` was added so a lane in a worktree could
#                 regenerate this block; that closed the dc3-tree half and left
#                 this one, which is the half a worktree actually trips on.
#   C2RS_BIN      skip the build, use this c2rs  (identity is then the caller's)
#   C2RS_JOBS     default for --jobs
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
. "$repo_root/scripts/harness_bin.sh"

jobs="${C2RS_JOBS:-16}"
dc3="${C2RS_DC3:-$repo_root/../dc3-decomp}"
workload="${C2RS_WORKLOAD:-$repo_root/work/dc3-workload}"
do_write=0
do_check=0
do_raw=0
tests_log=""

while [ $# -gt 0 ]; do
    case "$1" in
        --write) do_write=1 ;;
        --check) do_check=1 ;;
        --raw)   do_raw=1 ;;
        --jobs)  shift; jobs="$1" ;;
        --tests-log)
            shift
            # A missing value must not become "option absent" — that is bug 3 of
            # `cli_flags.rs`, and here it would silently re-run the suite the
            # caller asked to skip.
            [ $# -gt 0 ] || { echo "status.sh: --tests-log needs a path" >&2; exit 2; }
            tests_log="$1"
            ;;
        -h|--help) sed -n '1,60p' "$0"; exit 0 ;;
        *) echo "status.sh: unknown argument '$1'" >&2; exit 2 ;;
    esac
    shift
done

# ---- THE METRIC REGISTRY -------------------------------------------------------
#
# The list of metrics lives HERE and only here, for the reason `lanes.txt` exists:
# a metric that is not enumerated is a metric that does not get collected, and its
# absence from the report looks identical to it having nothing to say.
#
#   <key>  <needs-toolchain>  <label>
#
# Render order is registry order.
METRICS='
tests            no   Workspace tests (cargo test --workspace --release)
selftest         yes  Oracle self-test (c2rs selftest)
fixture-gate     yes  Fixture port gate (c2rs perf)
perf             yes  Port speedup, geomean over matched fixtures
workload         yes  878-TU dc3 workload scan (c2rs gap)
census           yes  Per-function census (driver, not target)
emitted-census   yes  Emitted-function census
residue          yes  Emitted-census residue
distance-bodies  yes  TU distance to match, blocked functions
distance-emitted yes  TU distance to match, blocked emitted functions
emit-ceiling     yes  Emit-set ceiling, LO-anchored (segments == COMDATs)
emit-ceiling-gate yes Emit-set ceiling, GATE-anchored (4F 1F — what the port consumes)
emit-model       yes  Emit-set MODEL ceiling (today / repaired / wall)
binding          yes  .gl binding invariants (records / arity / conflicts)
factors          yes  Phase-7 factors over the graded TUs (A / B / C / D / E)
joint-ceilings   yes  Joint ceilings (B∧C, A∧B∧C)
frontier         yes  Pre-Phase-7 FRONTIER (codegen breadth alone / if A were free)
emit-predicate-worth yes Emit-predicate worth, B∧C − A∧B∧C (board #213)
section-ladder   yes  Factor-C section ladder (writer names / workload names / next step)
progress-mass    yes  PROGRESS MASS (driver, not target — docs/PROGRESS_METRIC.md)
fnbyte-match     yes  FUNCTION BYTE MATCH (driver, not target — docs/FUNCTION_BYTE_MATCH.md)
fnbyte-partition yes  FBM partition (the under-report, and the controls)
fnbyte-per-tu    yes  Per-TU FBM (how close is the other 870)
plan-emitset     yes  OBJECT PLAN — emit set (predicted from IL vs the reference obj)
plan-control     yes  OBJECT PLAN — the NAMED control on the byte-exact TUs
plan-inventory   yes  OBJECT PLAN — reference-side inventory (weak / COMDAT / undef)
'
# --- the five rows above, added 2026-08-04 by lane w-gr on lane w-bc's spec ----
#
# **Factor C, `A∧B∧C` and the FRONTIER had never been in the generated block at
# all.** They lived only in hand-written prose in `STATUS.md`, and all three went
# stale twice in a single day. `B∧C` was worse: it was published as **107**,
# measured when `C = 114`, and stayed there while C moved to 169 — it is **151**,
# and the projection it feeds moved from `+82` to `+124`. A number that can only
# go stale by a *dependency* moving is exactly the number a collector should own.
#
# `crates/c2-harness/src/gap.rs` prints a `GAP-METRICS` block of stable
# `gap-metric <key> <value>` pairs for this purpose (w-bc). The keys are an
# interface; `p_metric` below is the ONE reader of them.

# Every metric `collect_gap` is responsible for — the keys its four early-return
# paths have to report a REASON for, not merely leave unset.
#
# This list used to be spelled out FOUR TIMES inside `collect_gap`, once per
# early return. w-gr's five GAP-METRICS rows were the fifth thing that would have
# had to be added to all four, and the failure mode is silent in the direction
# that matters: a key missing from one copy renders `NO-RESULT` with no reason
# attached on exactly the path that produced it, which is indistinguishable from
# a metric that ran and had nothing to say. One list; `--check` proves it covers
# the registry.
GAP_KEYS='workload census emitted-census residue distance-bodies
          distance-emitted emit-ceiling emit-ceiling-gate emit-model binding
          factors joint-ceilings frontier emit-predicate-worth section-ladder
          progress-mass fnbyte-match fnbyte-partition fnbyte-per-tu
          plan-emitset plan-control plan-inventory'

results_file=""

emit() { printf 'STATUS-METRIC %s %s\n' "$1" "$2" >> "$results_file"; }

# A metric value that came back empty is a MISSING metric, not a zero one.
# Every parser routes through this.
val_or_missing() {
    _v="$1"
    if [ -z "$_v" ]; then printf 'NO-RESULT'; else printf '%s' "$_v"; fi
}

lookup() {
    _k="$1"
    _line=$(grep "^STATUS-METRIC $_k " "$results_file" 2>/dev/null | head -1 || true)
    if [ -z "$_line" ]; then printf 'NO-RESULT'; return; fi
    printf '%s' "${_line#STATUS-METRIC $_k }"
}

# Toolchain absence is detected PER COMMAND, from that command's own output, the
# way `mode_lane.sh` does it — not by a separate probe run up front. A probe is a
# second thing that can disagree with the first, and the cheap probes here
# (`perf`, `gap`) are the very commands being collected, so probing meant running
# them twice. `main.rs` guarantees every subcommand degrades to this exact line.
log_says_skip() { grep -q 'SKIP: toolchain absent' "$1" 2>/dev/null; }

# ---- THE PARSERS, defined ONCE ------------------------------------------------
#
# Each of these takes a log path and prints the field, or nothing. `--check` runs
# THESE functions against a captured real report; the collectors call the same
# functions on live output. That identity is the whole point.
#
# The first version of this script inlined the `sed` in the collector and again
# in the check. Breaking the collector's copy left the check green — verified,
# not supposed: a deliberately corrupted parser reported PASS. That is
# `harness_bin.sh`'s "a rule with three implementations" defect reproduced inside
# the instrument written to avoid it, and it is why these are functions.
p_match()      { sed -n 's/^ *match  *\([0-9][0-9]*\).*/\1/p'      "$1" | head -1; }
p_mismatch()   { sed -n 's/^ *mismatch  *\([0-9][0-9]*\).*/\1/p'   "$1" | head -1; }
p_codegen()    { sed -n 's/^ *codegen-gap  *\([0-9][0-9]*\).*/\1/p' "$1" | head -1; }
p_vocab()      { sed -n 's/^ *vocab-gap  *\([0-9][0-9]*\).*/\1/p'  "$1" | head -1; }
p_capfail()    { sed -n 's/^ *capture-fail  *\([0-9][0-9]*\).*/\1/p' "$1" | head -1; }
p_census()     { sed -n 's/.*FUNCTION CENSUS (P2b): \(.*\)$/\1/p'  "$1" | head -1; }
p_emitted()    { sed -n 's/.*EMITTED CENSUS (§8): \(.*\)$/\1/p'    "$1" | head -1; }
p_residue()    { sed -n 's/^ *bound .*|  *residue \(.*\)$/residue \1/p' "$1" | head -1; }
p_dist_body()  { sed -n 's/.*TU distance to matching (blocked functions) — \(.*\)$/\1/p' "$1" | head -1; }
p_dist_emit()  { sed -n 's/.*TU distance to matching (blocked EMITTED functions) — \(.*\)$/\1/p' "$1" | head -1; }
p_ceiling()    { sed -n 's/.*emit-set ceiling: \([0-9]* of [0-9]*\) graded TUs.*/\1 graded TUs/p' "$1" | head -1; }
p_ceilgate()   { sed -n 's/^ *emit-set ceiling, GATE-anchored (`4F 1F`, what the port consumes), over the \([0-9]*\) known: \([0-9]*\) .*/\2 of \1 graded TUs/p' "$1" | head -1; }
p_model()      { sed -n 's/.*emit-set MODEL ceiling: \([0-9]*\) of [0-9]* TUs bind every emitted symbol today; \([0-9]*\) would.*; \([0-9]*\) carry.*/\1 today \/ \2 repaired \/ \3 wall/p' "$1" | head -1; }
p_binding()    { sed -n 's/^ *binding: \(.*\)$/\1/p'               "$1" | head -1; }
p_fixgate()    { sed -n 's/^summary: \(.*\)$/\1/p'                 "$1" | head -1; }

# **The GAP-METRICS block** (`GapReport::metrics`, lane w-bc). One reader for the
# whole block, keyed by name, because the alternative is a `sed` per figure and
# the comment above records what happened the last time this file had a parser
# per call site.
#
# Absence is ABSENCE. `ladder-head` is deliberately *omitted* by `gap.rs` when the
# writer's section vocabulary is closed, and a reader that defaulted it to 0 would
# report a closed ladder as reaching `C = 0`. So this prints nothing for a missing
# key and every caller routes through `val_or_missing`, which turns nothing into
# `NO-RESULT`.
p_metric()     { sed -n "s/^ *gap-metric $2 \(.*\)\$/\1/p" "$1" | head -1; }

# Two or more metrics into one row, with the same absence discipline: if ANY part
# is missing the whole row is NO-RESULT, never a sentence with a hole in it.
#   join_metrics <log> <format-with-%s...> <key>...
_metric_row() { # <log> <template> <key>...
    _log="$1"; _tpl="$2"; shift 2
    _out="$_tpl"
    for _k in "$@"; do
        _v=$(p_metric "$_log" "$_k")
        [ -n "$_v" ] || { printf ''; return; }
        _out=$(printf '%s' "$_out" | sed "s|@|$_v|")
    done
    printf '%s' "$_out"
}
p_geomean()    { sed -n 's/.*geomean speedup over the [0-9]* matched fixture(s): \([0-9]*x\).*/\1/p' "$1" | head -1; }

# ---- collectors ----------------------------------------------------------------
# Each prints exactly one STATUS-METRIC line per key on EVERY exit path.

# ---- THE TEST LEG, AND THE ONE WAY TO NOT PAY FOR IT TWICE ---------------------
#
# `collect_tests` runs the **whole** workspace suite — 206 s on this box, 300 s
# under lane load — and a merge ritual that runs `cargo test --workspace
# --release` and then `status.sh --write` pays for it twice, for a report whose
# other unique contribution (the 878-TU scan) is **2.1 s warm**.
#
# `--tests-log FILE` lets a caller hand over the run it already did. The obvious
# way to build that is a false green with this project's own name on it — a log
# from *before* the change is a passing suite for code that was never tested — so
# the reuse path is gated on FOUR positive checks, each of which renders
# `NO-RESULT` **with the reason in the value** rather than falling back to a
# number:
#
#   1. the file exists and is non-empty;
#   2. **nothing cargo would have read is newer than the log.** The input set is
#      re-derived from the tree on every run (`_tests_inputs`), not remembered:
#      `crates/`, `fixtures/`, `Cargo.toml`/`Cargo.lock`, the data files the
#      registry tests read (`scripts/lanes.txt`, `scripts/sweep.d`,
#      `scripts/sweep_gen.py`, `docs/rungs/`) and every path an `include_str!`
#      in `crates/` actually names — which is how `work/w-inl0/cells/*.cpp`
#      is in this list and would not have been if it were typed by hand;
#   3. **every target cargo launched reported a result.** `Running`/`Doc-tests`
#      lines and `test result:` lines are counted and must be EQUAL and non-zero.
#      A target that died without printing a result, or a log truncated
#      mid-suite, is a short count — the same reconciliation `expr_sweep.sh` and
#      `mode_cross.sh` use, and the answer to the newest instance of STATUS.md's
#      trap 5 (a runner that reported `ok` for every target with 169 tests
#      silently not run);
#   4. the log ENDS on a `test result:` line, so an interrupted run cannot pass
#      check 3 by having been cut between two targets.
#
# **Check 2 is conservative on purpose and you WILL hit it.** `docs/rungs/` is in
# the closure because `rung_registry.rs` reads it, so writing a rung doc after
# running the suite makes the log stale — correctly, because the suite's inputs
# did change. The ritual that works is: write the rung doc, THEN
# `cargo test --workspace --release 2>&1 | tee <log>`, then the gate, then
# `status.sh --write --tests-log <log>`. The refusal names the offending file, so
# it says which of these it was rather than leaving you to guess.
#
# What it deliberately does NOT do is *cache*. There is no state, no sentinel and
# no "skip if unchanged": every invocation without `--tests-log` runs the suite,
# exactly as before. `work/fable-perf/PROPOSAL.md` §6 declined "teach it to
# consume a prior test log" on the grounds that a stale log is worse than the
# duplication, and that is right about a bare log path; checks 2-4 are the price
# of disagreeing with it.
_tests_inputs() {
    # Printed relative to $repo_root; consumed by `find` there.
    printf '%s\n' Cargo.toml Cargo.lock crates fixtures \
        scripts/lanes.txt scripts/sweep.d scripts/sweep_gen.py docs/rungs
    grep -rho 'include_str!("\.\./\.\./\.\./[^"]*")' "$repo_root/crates" 2>/dev/null \
        | sed 's|include_str!("\.\./\.\./\.\./||; s|")$||' | sort -u
}

# Print the first test input strictly newer than $1, or nothing.
_tests_input_newer_than() {
    _tin_log="$1"
    ( cd "$repo_root" || exit 0
      _tin_paths=""
      for _p in $(_tests_inputs); do
          [ -e "$_p" ] && _tin_paths="$_tin_paths $_p"
      done
      [ -n "$_tin_paths" ] || exit 0
      # shellcheck disable=SC2086
      find $_tin_paths -newer "$_tin_log" \( -type f -o -type l \) -print 2>/dev/null | head -1
    )
}

collect_tests_from_log() {
    _log="$1"
    if [ ! -f "$_log" ]; then
        emit tests "NO-RESULT (--tests-log MISSING: no such file: $_log)"; return 0
    fi
    if [ ! -s "$_log" ]; then
        emit tests "NO-RESULT (--tests-log EMPTY: $_log)"; return 0
    fi
    _newer=$(_tests_input_newer_than "$_log")
    if [ -n "$_newer" ]; then
        emit tests "NO-RESULT (--tests-log STALE: $_newer is newer than the log, so the log did not test this tree)"
        return 0
    fi
    _r=$(grep -cE '^[[:space:]]*(Running|Doc-tests) ' "$_log" || true)
    _t=$(grep -cE '^test result' "$_log" || true)
    if [ "${_t:-0}" -eq 0 ] || [ "${_r:-0}" -eq 0 ]; then
        emit tests "NO-RESULT (--tests-log NO-RUN: $_log has no cargo test run in it — $_r launched, $_t reported)"
        return 0
    fi
    if [ "$_r" -ne "$_t" ]; then
        emit tests "NO-RESULT (--tests-log SHORT: cargo launched $_r targets and only $_t reported a result)"
        return 0
    fi
    if [ "$(grep -vE '^[[:space:]]*$' "$_log" | tail -1 | cut -c1-12)" != 'test result:' ]; then
        emit tests "NO-RESULT (--tests-log INTERRUPTED: $_log does not END on a 'test result:' line)"
        return 0
    fi
    _p=$(grep -oE '[0-9]+ passed' "$_log" | awk '{s+=$1} END{if(NR)print s; else print ""}')
    _f=$(grep -oE '[0-9]+ failed' "$_log" | awk '{s+=$1} END{if(NR)print s; else print ""}')
    if [ -z "$_p" ]; then emit tests NO-RESULT; return 0; fi
    if [ "${_f:-0}" -ne 0 ]; then
        emit tests "FAILING: $_p passed, $_f failed"; return 0
    fi
    emit tests "$_p passed, $(val_or_missing "$_f") failed, $_t targets"
}

collect_tests() {
    if [ -n "$tests_log" ]; then
        collect_tests_from_log "$tests_log"
        return 0
    fi
    _log="$work_dir/tests.log"
    if ! (cd "$repo_root" && cargo test --workspace --release) > "$_log" 2>&1; then
        # A failing suite is a RESULT, and a loud one — not a missing metric.
        _p=$(grep -oE '[0-9]+ passed' "$_log" | awk '{s+=$1} END{print s+0}')
        _f=$(grep -oE '[0-9]+ failed' "$_log" | awk '{s+=$1} END{print s+0}')
        emit tests "FAILING: $_p passed, $_f failed"
        return 0
    fi
    _p=$(grep -oE '[0-9]+ passed' "$_log" | awk '{s+=$1} END{if(NR)print s; else print ""}')
    _f=$(grep -oE '[0-9]+ failed' "$_log" | awk '{s+=$1} END{if(NR)print s; else print ""}')
    _t=$(grep -cE '^test result' "$_log" || true)
    if [ -z "$_p" ]; then emit tests NO-RESULT; return 0; fi
    emit tests "$_p passed, $(val_or_missing "$_f") failed, $_t targets"
}

collect_selftest() {
    _log="$work_dir/selftest.log"
    if ! "$c2rs" selftest > "$_log" 2>&1; then
        emit selftest "FAILING (non-zero exit)"
        return 0
    fi
    if log_says_skip "$_log"; then emit selftest "SKIP: toolchain absent"; return 0; fi
    _p=$(grep -cE '[[:space:]]PASS([[:space:]]|$)' "$_log" || true)
    _f=$(grep -cE '[[:space:]]FAIL([[:space:]]|$)' "$_log" || true)
    if [ "${_p:-0}" -eq 0 ]; then emit selftest NO-RESULT; return 0; fi
    emit selftest "$_p PASS, $_f FAIL"
}

# **The perf geomean is WALL-CLOCK and this script runs it under load.**
#
# `collect_perf` fires after `collect_tests` (a full `cargo test --workspace
# --release`) and alongside whatever else is on the box; on 2026-08-05 two
# collections of the same unchanged code read **674x** and **481x**. Nothing in
# `crates/` had moved — the second ran while three gates were saturating the
# machine.
#
# It is collected anyway, because the alternative (dropping it) loses the
# project's own thesis metric from the one page that answers "where is this".
# But it is labelled in `STATUS.md`'s what-each-number-is-for table as
# load-sensitive, and **a move in it is not signal until it is retaken on a
# quiet box.** Do not rank lanes by it. Deliberately NOT "fixed" by pinning
# CPUs or retrying: a benchmark that quietly re-runs until it likes its own
# answer is worse than one that admits it is wall-clock.
collect_perf() {
    _log="$work_dir/perf.log"
    if ! "$c2rs" perf > "$_log" 2>&1; then
        emit fixture-gate NO-RESULT
        emit perf NO-RESULT
        return 0
    fi
    if log_says_skip "$_log"; then
        emit fixture-gate "SKIP: toolchain absent"
        emit perf "SKIP: toolchain absent"
        return 0
    fi
    emit fixture-gate "$(val_or_missing "$(p_fixgate "$_log")")"
    _g=$(p_geomean "$_log")
    if [ -z "$_g" ]; then emit perf NO-RESULT
    else emit perf "$_g geomean over matched fixtures"; fi
}

collect_gap() {
    _log="$work_dir/gap.log"
    _list="$workload/files.txt"
    _flags="$workload/flags.txt"

    if [ ! -f "$_list" ] || [ ! -f "$_flags" ]; then
        _why="NO-RESULT (no $workload/{files.txt,flags.txt} — run scripts/gen_dc3_workload.sh)"
        for _k in $GAP_KEYS; do
            emit "$_k" "$_why"
        done
        return 0
    fi
    if [ ! -d "$dc3" ]; then
        _why="NO-RESULT (dc3 tree absent at $dc3 — set C2RS_DC3)"
        for _k in $GAP_KEYS; do
            emit "$_k" "$_why"
        done
        return 0
    fi

    if ! "$c2rs" gap --list "$_list" --flags-file "$_flags" --cwd "$dc3" \
                     --jobs "$jobs" > "$_log" 2>&1; then
        for _k in $GAP_KEYS; do
            emit "$_k" "NO-RESULT (gap scan exited non-zero; see $_log)"
        done
        return 0
    fi

    if log_says_skip "$_log"; then
        for _k in $GAP_KEYS; do
            emit "$_k" "SKIP: toolchain absent"
        done
        return 0
    fi

    _m=$(p_match "$_log"); _mm=$(p_mismatch "$_log")
    if [ -z "$_m" ] || [ -z "$_mm" ]; then
        emit workload NO-RESULT
    else
        emit workload "match $_m, mismatch $_mm, codegen-gap $(val_or_missing "$(p_codegen "$_log")"), vocab-gap $(val_or_missing "$(p_vocab "$_log")"), capture-fail $(val_or_missing "$(p_capfail "$_log")")"
    fi

    emit census          "$(val_or_missing "$(p_census    "$_log")")"
    emit emitted-census  "$(val_or_missing "$(p_emitted   "$_log")")"
    emit residue         "$(val_or_missing "$(p_residue   "$_log")")"
    emit distance-bodies "$(val_or_missing "$(p_dist_body "$_log")")"
    emit distance-emitted "$(val_or_missing "$(p_dist_emit "$_log")")"
    emit emit-ceiling    "$(val_or_missing "$(p_ceiling   "$_log")")"
    emit emit-ceiling-gate "$(val_or_missing "$(p_ceilgate "$_log")")"
    emit emit-model      "$(val_or_missing "$(p_model     "$_log")")"
    emit binding         "$(val_or_missing "$(p_binding   "$_log")")"

    # ---- the GAP-METRICS rows (lane w-bc's spec, wired by w-gr) ---------------
    #
    # Five rows, each through `val_or_missing`, each registered in METRICS above
    # so `--check` proves it rather than tolerating its absence.
    emit factors "$(val_or_missing "$(_metric_row "$_log" \
        'A @ (LO @) · B @ · C @ · D @ · E @, of @ graded' \
        factor-a factor-a-lo factor-b factor-c factor-d factor-e graded)")"
    emit joint-ceilings "$(val_or_missing "$(_metric_row "$_log" \
        'B∧C @ · A∧B∧C @ · A∧B∧C∧D @' b-and-c a-and-b-and-c a-and-b-and-c-and-d)")"
    emit frontier "$(val_or_missing "$(_metric_row "$_log" \
        '@ reachable by codegen breadth alone; @ if factor A were free' \
        frontier frontier-if-a)")"
    emit emit-predicate-worth "$(val_or_missing "$(_metric_row "$_log" \
        '+@ TUs (B∧C − A∧B∧C)' emit-predicate-worth)")"
    # The ladder head is the one key `gap.rs` OMITS rather than zeroing, when the
    # writer's vocabulary already covers the workload. A closed ladder is a real
    # and different state from "the next step reaches 0", so it gets its own
    # sentence instead of a defaulted number.
    _ladder_steps=$(p_metric "$_log" ladder-steps)
    if [ "${_ladder_steps:-x}" = "0" ]; then
        emit section-ladder "$(val_or_missing "$(_metric_row "$_log" \
            '@ writer names cover all @ workload names — the ladder is CLOSED' \
            writer-sections workload-sections)")"
    else
        emit section-ladder "$(val_or_missing "$(_metric_row "$_log" \
            '@ writer names of @ workload names; @ steps left, next +@ → C = @' \
            writer-sections workload-sections ladder-steps ladder-head ladder-head-c)")"
    fi

    # ---- the two CONTINUOUS metrics (lanes w-metric and w-fuzzy) -------------
    #
    # `docs/PROGRESS_METRIC.md` §7 specified the progress-mass collector and did
    # not make it: "when it lands, progress-mass rides along for free". It lands
    # here, beside FBM, because the two exist for the same reason — TU match is
    # a per-TU conjunction and moves only when a TU's LAST defect closes — and a
    # reader who sees one without the other will read the wrong kind of progress.
    #
    # **Both are DRIVERS, never targets.** `STATUS.md`'s prose says so; the
    # labels in the registry say so; and neither appears in `scripts/gate.sh`.
    # The value is always rendered with its denominator, never alone: a bare
    # `0.16251` invites being read as "16 % done", and it is not — see the docs.
    emit progress-mass "$(val_or_missing "$(_metric_row "$_log" \
        'P = @ · emitted in class @/@ · mismatch-zeroed TUs @' \
        progress-mass progress-emitted-in-class progress-emitted-total \
        progress-mismatch-zeroed)")"
    # FBM's ratio NEVER renders without its denominator, and the whole-TU credit
    # is spelled out rather than folded in: the two numerators are graded by
    # different routes (the port's per-function selector, and the oracle's own
    # whole-obj verdict) and a reader must be able to tell them apart.
    emit fnbyte-match "$(val_or_missing "$(_metric_row "$_log" \
        'FBM = @ · @ exact + @ whole-TU of @ emitted functions, over @ TUs (@ at 100%); @ are byte-exact before relocations are graded' \
        fnbyte-match fnbyte-exact fnbyte-whole-tu fnbyte-denominator fnbyte-tus \
        fnbyte-tus-full fnbyte-exact-bytes)")"
    # The partition, with the instrument's OWN under-report first. A row that
    # published the ratio and dropped `fnbyte-partial` would hide the size of
    # what FBM cannot yet grade, which is the shape this project charges for.
    #
    # **`fnbyte-reloc-unknown` is the same shape one field along** (lane
    # `w-relo`): the byte-exact functions whose reference relocation table did
    # not decode, i.e. the population RELOC-EQ could NOT reach. It renders
    # beside `reloc-differs` for the same reason `partial` renders beside the
    # ratio, and its own must-fail mutation is in `--check`.
    emit fnbyte-partition "$(val_or_missing "$(_metric_row "$_log" \
        'partial @ (FBM under-reports by this) · differs @ · reloc-differs @ · reloc-unknown @ (UNGRADED residue) · refused @ · unbound @ · @ credited fns relocate, every record graded · controls: partition-broken @, reloc-reach-broken @, match-TU differs @, match-TU reloc-differs @, census disagree @' \
        fnbyte-partial fnbyte-differs fnbyte-reloc-differs fnbyte-reloc-unknown \
        fnbyte-refused fnbyte-unbound \
        fnbyte-exact-relocated fnbyte-partition-broken fnbyte-reloc-partition-broken \
        fnbyte-match-tu-differs fnbyte-match-tu-reloc-differs \
        fnbyte-census-disagree)")"
    emit fnbyte-per-tu "$(val_or_missing "$(_metric_row "$_log" \
        '@ of @ TUs with emitted functions are 100% byte-exact per function' \
        fnbyte-tus-full fnbyte-tus)")"
    # ---- THE OBJECT PLAN (lane w-objplan) -----------------------------------
    #
    # The structural manifest curve. THREE denominators and never a bare ratio:
    # `observable` (the reference obj decoded) ⊇ `known` (the port also
    # answered) ⊇ `exact`. `distinct` is the FREE-component detector — a
    # component that takes one value across the whole workload gives a 100% that
    # measures nothing.
    #
    # **NECESSARY BUT NOT SUFFICIENT for `match`.** This row is a progress
    # instrument; the byte judge is unchanged and `plan-*` gates nothing.
    emit plan-emitset "$(val_or_missing "$(_metric_row "$_log" \
        'members: observable @ | known @ | exact @ | differs @ | distinct @ ;; seed names @ of the reference obj @ emitted, empty on @ TUs, subset on @ (@ over-claimed, @ the closure still owes) ;; .gl-record order agrees @ of @ (REFUTED as a predictor; characterization only) ;; bounds-violations @' \
        plan-emitset-members-observable plan-emitset-members-known \
        plan-emitset-members-exact plan-emitset-members-differs \
        plan-emitset-members-distinct \
        plan-emitset-seed-size plan-emitset-observed-size \
        plan-emitset-seed-empty-tus plan-emitset-seed-subset \
        plan-emitset-seed-extra plan-emitset-seed-missing \
        plan-emitset-glorder-agrees plan-emitset-glorder-known \
        plan-bounds-violations)")"
    # The control is pinned BY NAME in docs/plan/CONTROL_TUS.txt. `diff` is the
    # identity diff against that file and a NONZERO value is a finding about the
    # tree or the workload stamp, reported before any number above it.
    emit plan-control "$(val_or_missing "$(_metric_row "$_log" \
        '@ pinned | @ found | set-diff @ | @ exact on every shipped component | @ shortfall cell(s) = @ differs + @ unknown (only differs reds)' \
        plan-control-pinned plan-control-found plan-control-diff \
        plan-control-exact plan-control-shortfall \
        plan-control-differs plan-control-unknown)")"
    # NOT a curve: read off real c2's objs, describing the population the
    # un-conjuncted lanes must serve. Re-derives figures this project has only
    # ever carried.
    emit plan-inventory "$(val_or_missing "$(_metric_row "$_log" \
        'weak @ records over @ TUs | COMDAT sections @ (@ associative over @ TUs; @ of UNKNOWN selection) | undefined externals @ over @ TUs | sections @ (@ distinct attribute sequences) | relocation records @' \
        plan-obs-weak-records plan-obs-weak-tus plan-obs-comdat-sections \
        plan-obs-comdat-assoc-sections plan-obs-comdat-assoc-tus \
        plan-obs-comdat-sel-unknown \
        plan-obs-undef-records plan-obs-undef-tus plan-obs-sections \
        plan-obs-sections-attrs-distinct plan-obs-reloc-records)")"
}

# ---- --check : prove the parsers and the registry, with no toolchain ------------
#
# The answer to "has anyone ever seen this report a missing metric correctly?".
if [ "$do_check" -eq 1 ]; then
    work_dir=$(mktemp -d "${TMPDIR:-/tmp}/c2rs-status-check.XXXXXX")
    results_file="$work_dir/metrics"
    : > "$results_file"
    fails=0

    # 1. the registry is non-empty and every row is well formed
    nreg=$(printf '%s\n' "$METRICS" | grep -c '[^[:space:]]' || true)
    [ "$nreg" -gt 0 ] || { echo "CHECK FAIL: empty metric registry"; fails=$((fails+1)); }
    printf '%s\n' "$METRICS" | grep '[^[:space:]]' | while read -r k t l; do
        case "$t" in yes|no) ;; *) echo "CHECK FAIL: $k has bad toolchain flag '$t'"; exit 1 ;; esac
        [ -n "$l" ] || { echo "CHECK FAIL: $k has no label"; exit 1; }
    done || fails=$((fails+1))

    # 2. an unset metric renders NO-RESULT, not blank and not 0
    got=$(lookup never-collected)
    [ "$got" = "NO-RESULT" ] || { echo "CHECK FAIL: absent metric rendered '$got'"; fails=$((fails+1)); }

    # 3. an empty parser result renders NO-RESULT
    got=$(val_or_missing "")
    [ "$got" = "NO-RESULT" ] || { echo "CHECK FAIL: empty value rendered '$got'"; fails=$((fails+1)); }

    # 4. a real value survives round-trip through the results file
    emit probe 'match 6, mismatch 0'
    got=$(lookup probe)
    [ "$got" = "match 6, mismatch 0" ] || { echo "CHECK FAIL: round-trip gave '$got'"; fails=$((fails+1)); }

    # 5. the parsers hit on captured real output rather than on nothing.
    #    A parser that matches nothing returns "", which routes to NO-RESULT —
    #    correct, but indistinguishable from a metric that genuinely has no value.
    #    So the shapes are pinned here against literal lines from a real report.
    probe_log="$work_dir/probe.log"
    cat > "$probe_log" <<'EOF'
  match             6    0.7%
  mismatch          0    0.0%
  FUNCTION CENSUS (P2b): 706402/2462571 functions in class (28.69%)
  EMITTED CENSUS (§8): 38456/178968 emitted functions in class (21.49%)
    bound 169693  |  residue 9275: 2004 compiler-generated (no IL body), 7271 unexplained  (5.18% of the denominator)
    TU distance to matching (blocked functions) — ≤0: 1, ≤1: 10, ≤10: 25
    emit-set ceiling: 25 of 871 graded TUs have `.ex` segments == obj `.text` COMDATs
    emit-set MODEL ceiling: 324 of 871 TUs bind every emitted symbol today; 420 would if `bind.rs` lost none; 451 carry an emitted symbol with NO `.gl` body record and are a wall
  emit-set ceiling, LO-anchored, over ALL graded TUs: 27
  emit-set ceiling, GATE-anchored (`4F 1F`, what the port consumes), over the 871 known: 28 (+1 entering, -0 leaving vs the LO-anchored set)
    binding: 1515160 records, 420 nameless, 2 before the first row
summary: 100 port Match, 0 mismatch, 110 not-implemented (of 210)
  geomean speedup over the 100 matched fixture(s): 653x faster than standalone c2
  GAP-METRICS — stable `key value` pairs for scripts/status.sh; keys are an interface, do not rename.
    gap-metric graded 871
    gap-metric factor-a 28
    gap-metric factor-a-lo 27
    gap-metric factor-b 338
    gap-metric factor-c 169
    gap-metric factor-d 8
    gap-metric factor-e 2
    gap-metric b-and-c 151
    gap-metric a-and-b-and-c 27
    gap-metric a-and-b-and-c-and-d 6
    gap-metric frontier 19
    gap-metric frontier-if-a 124
    gap-metric emit-predicate-worth 124
    gap-metric writer-sections 10
    gap-metric workload-sections 13
    gap-metric ladder-steps 3
    gap-metric ladder-head .rdata$r
    gap-metric ladder-head-c 590
    gap-metric progress-mass 0.20728
    gap-metric progress-emitted-in-class 38458
    gap-metric progress-emitted-total 178975
    gap-metric progress-mismatch-zeroed 0
    gap-metric fnbyte-match 0.16251
    gap-metric fnbyte-exact 29084
    gap-metric fnbyte-denominator 178975
    gap-metric fnbyte-differs 0
    gap-metric fnbyte-partial 9374
    gap-metric fnbyte-refused 131292
    gap-metric fnbyte-unbound 9225
    gap-metric fnbyte-partition-broken 0
    gap-metric fnbyte-census-disagree 0
    gap-metric fnbyte-exact-relocated 0
    gap-metric fnbyte-match-tu-differs 0
    gap-metric fnbyte-whole-tu 2
    gap-metric fnbyte-tus-full 4
    gap-metric fnbyte-tus 865
    gap-metric fnbyte-exact-bytes 29084
    gap-metric fnbyte-reloc-differs 0
    gap-metric fnbyte-reloc-unknown 0
    gap-metric fnbyte-reloc-graded 29084
    gap-metric fnbyte-reloc-partition-broken 0
    gap-metric fnbyte-match-tu-reloc-differs 0
    gap-metric plan-observable 869
    gap-metric plan-emitset-members-observable 869
    gap-metric plan-emitset-members-known 700
    gap-metric plan-emitset-members-exact 61
    gap-metric plan-emitset-members-differs 639
    gap-metric plan-emitset-members-distinct 820
    gap-metric plan-emitset-seed-subset 655
    gap-metric plan-emitset-seed-extra 12
    gap-metric plan-emitset-seed-missing 3456
    gap-metric plan-emitset-seed-size 777
    gap-metric plan-emitset-observed-size 4233
    gap-metric plan-emitset-seed-empty-tus 9
    gap-metric plan-emitset-observed-empty-tus 5
    gap-metric plan-emitset-glorder-known 700
    gap-metric plan-emitset-glorder-agrees 44
    gap-metric plan-obs-emitset-order-distinct 1
    gap-metric plan-bounds-violations 0
    gap-metric plan-control-pinned 26
    gap-metric plan-control-found 26
    gap-metric plan-control-diff 0
    gap-metric plan-control-exact 26
    gap-metric plan-control-shortfall 0
    gap-metric plan-control-unknown 0
    gap-metric plan-control-differs 0
    gap-metric plan-obs-weak-records 1234
    gap-metric plan-obs-weak-tus 675
    gap-metric plan-obs-comdat-sections 98765
    gap-metric plan-obs-comdat-assoc-sections 4321
    gap-metric plan-obs-comdat-assoc-tus 450
    gap-metric plan-obs-comdat-sel-unknown 0
    gap-metric plan-obs-undef-records 54321
    gap-metric plan-obs-undef-tus 860
    gap-metric plan-obs-sections 123456
    gap-metric plan-obs-sections-attrs-distinct 812
    gap-metric plan-obs-reloc-records 234567
EOF
    # Known answers against the captured report above. These call the SAME
    # functions the collectors call — corrupt a parser and this goes red.
    check_parse() { # <parser-fn> <expected>
        _got=$("$1" "$probe_log")
        if [ "$_got" != "$2" ]; then
            echo "CHECK FAIL: $1 gave '$_got', expected '$2'"
            return 1
        fi
    }
    check_parse p_match      '6'                                                  || fails=$((fails+1))
    check_parse p_mismatch   '0'                                                  || fails=$((fails+1))
    check_parse p_census     '706402/2462571 functions in class (28.69%)'         || fails=$((fails+1))
    check_parse p_emitted    '38456/178968 emitted functions in class (21.49%)'   || fails=$((fails+1))
    check_parse p_residue    'residue 9275: 2004 compiler-generated (no IL body), 7271 unexplained  (5.18% of the denominator)' || fails=$((fails+1))
    check_parse p_dist_body  '≤0: 1, ≤1: 10, ≤10: 25'                             || fails=$((fails+1))
    check_parse p_fixgate    '100 port Match, 0 mismatch, 110 not-implemented (of 210)' || fails=$((fails+1))
    check_parse p_geomean    '653x'                                               || fails=$((fails+1))
    check_parse p_model      '324 today / 420 repaired / 451 wall'                || fails=$((fails+1))
    check_parse p_ceiling    '25 of 871 graded TUs'                               || fails=$((fails+1))
    # The two ceilings are DIFFERENT metrics on the same page, and the probe log
    # deliberately carries both with different values (25 vs 28): a parser that
    # picked up the wrong line would read 25 here and the check would go red.
    check_parse p_ceilgate   '28 of 871 graded TUs'                               || fails=$((fails+1))
    check_parse p_binding    '1515160 records, 420 nameless, 2 before the first row' || fails=$((fails+1))

    # ---- the GAP-METRICS rows (lane w-bc's block, wired by w-gr) --------------
    #
    # These five carry factor C, `B∧C`, `A∧B∧C` and the FRONTIER, which had never
    # been in the generated block at all — they lived in hand-written prose and
    # all of them went stale twice in one day. `B∧C` was published as 107,
    # measured at `C = 114`, and is 151 at `C = 169`. A figure that goes stale
    # because a *dependency* moved is the one a collector must own.
    check_metric() { # <key> <expected>
        _got=$(p_metric "$probe_log" "$1")
        if [ "$_got" != "$2" ]; then
            echo "CHECK FAIL: p_metric $1 gave '$_got', expected '$2'"
            return 1
        fi
    }
    check_metric factor-c            '169'      || fails=$((fails+1))
    check_metric b-and-c             '151'      || fails=$((fails+1))
    check_metric a-and-b-and-c       '27'       || fails=$((fails+1))
    check_metric frontier            '19'       || fails=$((fails+1))
    check_metric emit-predicate-worth '124'     || fails=$((fails+1))
    # A non-numeric value, and one containing a `$` — `.rdata$r` is the current
    # ladder head and a `sed` that let the shell touch it would come back empty.
    check_metric ladder-head         '.rdata$r' || fails=$((fails+1))
    # A key that is NOT in the block must read empty, so `val_or_missing` can
    # turn it into NO-RESULT. If this returned "0" the closed-ladder case below
    # would silently render "C = 0".
    _got=$(p_metric "$probe_log" no-such-metric)
    [ -z "$_got" ] || { echo "CHECK FAIL: absent gap-metric gave '$_got', expected empty"; fails=$((fails+1)); }
    # A key that is a PREFIX of another must not match it: `factor-a` and
    # `factor-a-lo` differ by 1 in the probe, so a loose pattern goes red here.
    check_metric factor-a            '28'       || fails=$((fails+1))
    check_metric factor-a-lo         '27'       || fails=$((fails+1))

    # ---- the OBJECT PLAN rows (lane w-objplan) -------------------------------
    #
    # Same discipline as the block above and for the same reason: a renamed key
    # returns NO-RESULT, which is trap 5 (absence read as success) with the mask
    # on, and nothing else in the pipeline would notice.
    #
    # The prefix trap is live here too and sharper than `factor-a`'s:
    # `plan-emitset-seed-size` and `plan-emitset-seed-subset` share the prefix
    # `plan-emitset-seed-s`, and `plan-observable` is a prefix of
    # `plan-observable-…` shaped keys. The probe log deliberately gives them
    # DIFFERENT values (777 vs 655), so a loose pattern reads the wrong one and
    # this check goes red rather than agreeing by coincidence.
    check_metric plan-observable                    '869' || fails=$((fails+1))
    check_metric plan-emitset-members-exact         '61'  || fails=$((fails+1))
    check_metric plan-emitset-glorder-agrees        '44'  || fails=$((fails+1))
    check_metric plan-emitset-members-distinct      '820' || fails=$((fails+1))
    check_metric plan-obs-emitset-order-distinct    '1'   || fails=$((fails+1))
    # THE CLAIMANT'"'"'S OWN SIZE, beside the containment claim. Without these two,
    # `seed-subset` is unfalsifiable in the flattering direction -- the empty set
    # is a subset of everything -- and this lane'"'"'s first workload run printed
    # exactly that and could not tell the two readings apart.
    check_metric plan-emitset-seed-size             '777' || fails=$((fails+1))
    check_metric plan-emitset-observed-size         '4233' || fails=$((fails+1))
    check_metric plan-emitset-seed-empty-tus        '9'   || fails=$((fails+1))
    check_metric plan-bounds-violations             '0'   || fails=$((fails+1))
    check_metric plan-control-diff                  '0'   || fails=$((fails+1))
    check_metric plan-obs-weak-tus                  '675' || fails=$((fails+1))
    # A `plan-*` key that is NOT in the block must read EMPTY so `val_or_missing`
    # can turn it into NO-RESULT. If it read `0` the whole object-plan row would
    # render a plausible sentence full of zeros — a curve that says "no progress"
    # where the truth is "this key was renamed".
    _got=$(p_metric "$probe_log" plan-emitset-members-exactly)
    [ -z "$_got" ] || { echo "CHECK FAIL: absent plan key gave '$_got', expected empty"; fails=$((fails+1)); }
    # …and a whole composed row with one missing part must be EMPTY, never a
    # sentence with a hole in it.
    _got=$(_metric_row "$probe_log" 'exact @ of @' plan-emitset-members-exact no-such-plan-key)
    [ -z "$_got" ] || { echo "CHECK FAIL: incomplete plan row gave '$_got'"; fails=$((fails+1)); }

    # The composed rows. A row whose parts are all present renders fully; a row
    # with ANY part missing must be empty, never a sentence with a hole in it.
    _got=$(_metric_row "$probe_log" 'B∧C @ · A∧B∧C @ · A∧B∧C∧D @' \
        b-and-c a-and-b-and-c a-and-b-and-c-and-d)
    [ "$_got" = 'B∧C 151 · A∧B∧C 27 · A∧B∧C∧D 6' ] \
        || { echo "CHECK FAIL: joint-ceilings row gave '$_got'"; fails=$((fails+1)); }
    _got=$(_metric_row "$probe_log" '@ steps, next +@ → C = @' \
        ladder-steps ladder-head ladder-head-c)
    [ "$_got" = '3 steps, next +.rdata$r → C = 590' ] \
        || { echo "CHECK FAIL: section-ladder row gave '$_got'"; fails=$((fails+1)); }
    _got=$(_metric_row "$probe_log" 'a @ b @' b-and-c no-such-metric)
    [ -z "$_got" ] \
        || { echo "CHECK FAIL: a row with a missing part gave '$_got', expected empty"; fails=$((fails+1)); }
    [ "$(val_or_missing "$_got")" = "NO-RESULT" ] \
        || { echo "CHECK FAIL: an incomplete row did not render NO-RESULT"; fails=$((fails+1)); }

    # **A CLOSED ladder must not render as `C = 0`.** `gap.rs` OMITS
    # `ladder-head` when the writer's vocabulary covers the workload — board
    # w-bc asserts that on its side — so a collector that defaulted the key to 0
    # would report the best possible state as the worst one. Probe log with the
    # head removed and `ladder-steps 0`:
    closed_log="$work_dir/closed.log"
    sed -e 's/^    gap-metric ladder-steps 3$/    gap-metric ladder-steps 0/' \
        -e '/gap-metric ladder-head/d' "$probe_log" > "$closed_log"
    [ -z "$(p_metric "$closed_log" ladder-head)" ] \
        || { echo "CHECK FAIL: closed-ladder probe still has a ladder-head"; fails=$((fails+1)); }
    _got=$(_metric_row "$closed_log" '@ steps, next +@ → C = @' \
        ladder-steps ladder-head ladder-head-c)
    [ -z "$_got" ] \
        || { echo "CHECK FAIL: closed ladder rendered a step row '$_got'"; fails=$((fails+1)); }
    _got=$(_metric_row "$closed_log" '@ writer names cover all @ workload names — the ladder is CLOSED' \
        writer-sections workload-sections)
    [ "$_got" = '10 writer names cover all 13 workload names — the ladder is CLOSED' ] \
        || { echo "CHECK FAIL: closed-ladder row gave '$_got'"; fails=$((fails+1)); }

    # ---- the two CONTINUOUS metrics (w-metric's P, w-fuzzy's FBM) -------------
    #
    # Pinned against the tip's real figures. `fnbyte-match` and `progress-mass`
    # are the only two keys on this page that are RATIOS, and a ratio is the
    # thing that goes stale silently: it can move because either half moved, and
    # a reader cannot tell which from the number alone. Both rows therefore
    # render with their denominators, and these checks pin that.
    check_metric progress-mass       '0.20728'  || fails=$((fails+1))
    check_metric fnbyte-match        '0.16251'  || fails=$((fails+1))
    check_metric fnbyte-denominator  '178975'   || fails=$((fails+1))
    check_metric fnbyte-partial      '9374'     || fails=$((fails+1))
    # A control whose value is legitimately 0 must still PARSE as 0 rather than
    # as absent — the two render identically in a table and mean opposite things
    # ("checked, clean" vs "never computed"). `p_metric` returns empty for a
    # missing key, so a zero coming back non-empty is the thing to pin.
    check_metric fnbyte-match-tu-differs '0'    || fails=$((fails+1))
    check_metric fnbyte-partition-broken '0'    || fails=$((fails+1))
    # Prefix discipline again: `fnbyte-match` is a strict prefix of
    # `fnbyte-match-tu-differs` and they differ in the probe (0.16251 vs 0), so a
    # loose pattern reads the wrong line and this goes red.
    _got=$(p_metric "$probe_log" fnbyte-exact)
    [ "$_got" = '29084' ] || { echo "CHECK FAIL: p_metric fnbyte-exact gave '$_got'"; fails=$((fails+1)); }

    # **MUST-FAIL MUTATION — FBM's ratio must never render without its
    # under-report.** `fnbyte-partial` is the count of emitted functions the port
    # DID select and the instrument cannot yet grade, i.e. the size of FBM's own
    # under-report. A partition row that dropped it would still render a
    # plausible sentence, and the report would understate the port while looking
    # complete. Mutation: delete the key; the row must go EMPTY (hence
    # NO-RESULT), never a sentence with a hole in it.
    nopart_log="$work_dir/nopartial.log"
    grep -v 'gap-metric fnbyte-partial ' "$probe_log" > "$nopart_log"
    _got=$(_metric_row "$nopart_log" \
        'partial @ · differs @ · refused @ · unbound @ · controls: @, @, @' \
        fnbyte-partial fnbyte-differs fnbyte-refused fnbyte-unbound \
        fnbyte-partition-broken fnbyte-match-tu-differs fnbyte-census-disagree)
    [ -z "$_got" ] \
        || { echo "CHECK FAIL: the FBM partition rendered without its under-report: '$_got'"; \
             fails=$((fails+1)); }
    [ "$(val_or_missing "$_got")" = "NO-RESULT" ] \
        || { echo "CHECK FAIL: an FBM partition missing fnbyte-partial did not render NO-RESULT"; \
             fails=$((fails+1)); }

    # **MUST-FAIL MUTATION — the RELOCATION verdict must never render without
    # its UNGRADED residue** (lane `w-relo`, `docs/STATUS.md` trap 0).
    # `fnbyte-reloc-differs 0` beside a silent `fnbyte-reloc-unknown` reads as
    # "every relocation checks out" when what happened may be that none was
    # graded — the exact shape of objdiff's `total_code == 0 -> 100.0`, one
    # field along. Mutation: delete the residue key; the partition row must go
    # empty, never render a clean-looking verdict over an unstated population.
    noresid_log="$work_dir/noresidue.log"
    grep -v 'gap-metric fnbyte-reloc-unknown ' "$probe_log" > "$noresid_log"
    _got=$(_metric_row "$noresid_log" \
        'reloc-differs @ · reloc-unknown @ · controls: @, @' \
        fnbyte-reloc-differs fnbyte-reloc-unknown \
        fnbyte-reloc-partition-broken fnbyte-match-tu-reloc-differs)
    [ -z "$_got" ] \
        || { echo "CHECK FAIL: the relocation verdict rendered without its ungraded residue: '$_got'"; \
             fails=$((fails+1)); }
    [ "$(val_or_missing "$_got")" = "NO-RESULT" ] \
        || { echo "CHECK FAIL: a relocation row missing fnbyte-reloc-unknown did not render NO-RESULT"; \
             fails=$((fails+1)); }

    # And the reach identity's own zero must PARSE as zero, not as absent — the
    # same argument the `match-TU differs` pin makes, for the five-alarm control.
    check_metric fnbyte-reloc-partition-broken  '0' || fails=$((fails+1))
    check_metric fnbyte-match-tu-reloc-differs  '0' || fails=$((fails+1))
    check_metric fnbyte-exact-bytes         '29084' || fails=$((fails+1))

    # **MUST-FAIL MUTATION — the ratio must never render without its
    # denominator.** `FBM = 0.16251` alone reads as "16 % done" and is not; the
    # whole design rule is that the value travels with the population it was
    # taken over. Mutation: delete `fnbyte-denominator`; the headline row must
    # go empty rather than publish a bare ratio.
    noden_log="$work_dir/nodenom.log"
    grep -v 'gap-metric fnbyte-denominator ' "$probe_log" > "$noden_log"
    _got=$(_metric_row "$noden_log" \
        'FBM = @ · @ exact + @ whole-TU of @ emitted functions, over @ TUs (@ at 100%)' \
        fnbyte-match fnbyte-exact fnbyte-whole-tu fnbyte-denominator fnbyte-tus \
        fnbyte-tus-full)
    [ -z "$_got" ] \
        || { echo "CHECK FAIL: FBM rendered a bare ratio with no denominator: '$_got'"; \
             fails=$((fails+1)); }

    # …and the same for the progress mass, whose §7 note says the same thing.
    nopm_log="$work_dir/nopm.log"
    grep -v 'gap-metric progress-emitted-total ' "$probe_log" > "$nopm_log"
    _got=$(_metric_row "$nopm_log" 'P = @ · emitted in class @/@ · mismatch-zeroed TUs @' \
        progress-mass progress-emitted-in-class progress-emitted-total \
        progress-mismatch-zeroed)
    [ -z "$_got" ] \
        || { echo "CHECK FAIL: progress mass rendered without its denominator: '$_got'"; \
             fails=$((fails+1)); }

    # **A scan that graded nothing emits NEITHER continuous key.** `gap.rs`
    # OMITS both (`Option`), for the reason objdiff's `calc_fuzzy_match_percent`
    # is a cautionary tale: it returns 100.0 over zero code bytes. A collector
    # that defaulted a missing ratio to anything at all would republish that bug.
    empty_log="$work_dir/emptyscan.log"
    grep -v 'gap-metric progress-\|gap-metric fnbyte-' "$probe_log" > "$empty_log"
    for _k in progress-mass fnbyte-match fnbyte-denominator; do
        _got=$(p_metric "$empty_log" "$_k")
        [ -z "$_got" ] || { echo "CHECK FAIL: empty-scan probe still has $_k = '$_got'"; \
                            fails=$((fails+1)); }
        [ "$(val_or_missing "$_got")" = "NO-RESULT" ] \
            || { echo "CHECK FAIL: $_k over an empty scan did not render NO-RESULT"; \
                 fails=$((fails+1)); }
    done

    # **EVERY RENDERED ROW IS ONE LINE.** The generated block is a markdown
    # table, so a value containing a newline does not render as a long cell — it
    # ends the row and the rest becomes stray prose, silently, in the file
    # `CLAUDE.md` points readers at first.
    #
    # This is not hypothetical: the FBM partition template was written as a
    # single-quoted string broken across source lines, where `\` is a LITERAL
    # backslash and not a continuation, and it shipped a backslash and two
    # newlines into the value. Caught by running the collector, not by `--check`,
    # which is why the check now exists. Every template this file has is
    # rendered here and measured.
    check_one_line() { # <label> <rendered>
        _n=$(printf '%s' "$2" | wc -l | tr -d ' ')
        if [ "$_n" != "0" ]; then
            echo "CHECK FAIL: the $1 row rendered $((_n + 1)) lines; a markdown cell is one line"
            return 1
        fi
        case "$2" in
            *\\*) echo "CHECK FAIL: the $1 row contains a literal backslash — \
a single-quoted template broken across source lines"; return 1 ;;
        esac
    }
    check_one_line progress-mass "$(_metric_row "$probe_log" \
        'P = @ · emitted in class @/@ · mismatch-zeroed TUs @' \
        progress-mass progress-emitted-in-class progress-emitted-total \
        progress-mismatch-zeroed)" || fails=$((fails+1))
    check_one_line fnbyte-match "$(_metric_row "$probe_log" \
        'FBM = @ · @ exact + @ whole-TU of @ emitted functions, over @ TUs (@ at 100%)' \
        fnbyte-match fnbyte-exact fnbyte-whole-tu fnbyte-denominator fnbyte-tus \
        fnbyte-tus-full)" || fails=$((fails+1))
    check_one_line fnbyte-partition "$(_metric_row "$probe_log" \
        'partial @ (FBM under-reports by this) · differs @ · refused @ · unbound @ · @ credited fns carry a reloc FBM does not check · controls: partition-broken @, match-TU differs @, census disagree @' \
        fnbyte-partial fnbyte-differs fnbyte-refused fnbyte-unbound \
        fnbyte-exact-relocated fnbyte-partition-broken fnbyte-match-tu-differs \
        fnbyte-census-disagree)" || fails=$((fails+1))
    check_one_line fnbyte-per-tu "$(_metric_row "$probe_log" \
        '@ of @ TUs with emitted functions are 100% byte-exact per function' \
        fnbyte-tus-full fnbyte-tus)" || fails=$((fails+1))
    # …and the control on the control: a deliberately broken template must trip it.
    if check_one_line self-test "$(printf 'a\nb')" >/dev/null 2>&1; then
        echo "CHECK FAIL: check_one_line accepted a two-line value"; fails=$((fails+1))
    fi
    if check_one_line self-test 'a\b' >/dev/null 2>&1; then
        echo "CHECK FAIL: check_one_line accepted a literal backslash"; fails=$((fails+1))
    fi

    # ---- `--tests-log` FAILS CLOSED, four ways --------------------------------
    #
    # The reuse path exists to not run the suite twice, so the thing it must
    # never do is report a number for a suite that did not test this tree. Each
    # case below runs `collect_tests_from_log` for real (no toolchain, no cargo)
    # into a PRIVATE results file — `lookup` takes the first line for a key, so
    # sharing one would silently grade only the first case.
    _tl_dir="$work_dir/tests-log"
    mkdir -p "$_tl_dir"
    _saved_results="$results_file"
    check_tests_log() { # <name> <expect-prefix> <logfile>
        results_file="$_tl_dir/results.$1"
        : > "$results_file"
        collect_tests_from_log "$3"
        _got=$(lookup tests)
        case "$_got" in
            "$2"*) ;;
            *) echo "CHECK FAIL: --tests-log $1 rendered '$_got', expected '$2…'"
               fails=$((fails+1)) ;;
        esac
        results_file="$_saved_results"
    }

    # A complete, internally consistent log: 2 targets launched, 2 reported.
    _tl_good="$_tl_dir/good.log"
    cat > "$_tl_good" <<'EOF'
   Compiling c2-harness v0.1.0
     Running unittests src/lib.rs (target/release/deps/c2_il-0000)

running 3 tests

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/cli_flags.rs (target/release/deps/cli_flags-0000)

running 4 tests

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
EOF
    # `find -newer` is strictly-newer, and the checked-out tree is older than a
    # file written now — but say so positively rather than relying on it.
    touch "$_tl_good"
    check_tests_log good '7 passed, 0 failed, 2 targets' "$_tl_good"

    # 1. absent
    check_tests_log absent 'NO-RESULT (--tests-log MISSING' "$_tl_dir/nope.log"
    # 2. empty
    : > "$_tl_dir/empty.log"
    check_tests_log empty 'NO-RESULT (--tests-log EMPTY' "$_tl_dir/empty.log"
    # 3. STALE — the same good log, backdated so a real tree file is newer.
    cp "$_tl_good" "$_tl_dir/stale.log"
    touch -t 200001010000 "$_tl_dir/stale.log"
    check_tests_log stale 'NO-RESULT (--tests-log STALE' "$_tl_dir/stale.log"
    # 4. SHORT — three targets launched, two reported. This is the case the
    #    whole option turns on: the log looks fine and passes every other check.
    sed 's|Running tests/cli_flags|Running tests/gone.rs (target/release/deps/gone-0000)\n\n     Running tests/cli_flags|' \
        "$_tl_good" > "$_tl_dir/short.log"
    touch "$_tl_dir/short.log"
    check_tests_log short 'NO-RESULT (--tests-log SHORT' "$_tl_dir/short.log"
    # 5. INTERRUPTED — the good log with trailing noise after the last result, so
    #    the launched/reported counts still reconcile and only the end-of-log
    #    check can see it. `^C` in the middle of a suite is the real shape; this
    #    is the version of it that survives every earlier gate.
    { cat "$_tl_good"; printf '\nerror: process didn'\''t exit successfully\n'; } \
        > "$_tl_dir/interrupted.log"
    touch "$_tl_dir/interrupted.log"
    check_tests_log interrupted 'NO-RESULT (--tests-log INTERRUPTED' "$_tl_dir/interrupted.log"
    # 6. a FAILING suite is a result, not a missing metric
    sed 's/0 failed/1 failed/; s/^test result: ok/test result: FAILED/' "$_tl_good" \
        > "$_tl_dir/failing.log"
    touch "$_tl_dir/failing.log"
    check_tests_log failing 'FAILING: ' "$_tl_dir/failing.log"
    # 7. the closure is DERIVED, not remembered: the `include_str!` cells under
    #    work/ must be in it, which is the surprise nobody would have typed.
    if ! _tests_inputs | grep -q '^work/'; then
        echo "CHECK FAIL: _tests_inputs names no work/ path — the include_str! closure"
        echo "  is not being re-derived, so a change to a frozen grid cell would leave"
        echo "  a --tests-log accepted as fresh"
        fails=$((fails+1))
    fi

    # ---- EVERY REGISTERED METRIC HAS A COLLECTOR ------------------------------
    #
    # **The direction that was missing, and it was missing for the whole life of
    # this script.** `--check` proved the registry was well formed and the
    # parsers hit; nothing proved a registered key is ever `emit`ted. Verified by
    # mutation while writing this: renaming one collector's target
    # (`emit frontier` → `emit frontier-XX`) left `--check` PASS, and the row
    # would have rendered `NO-RESULT` in every future report — which reads as
    # "this metric had nothing to say" and not as "nobody computed it".
    #
    # A text check on this file, deliberately: the alternative is running the
    # collectors, which needs the toolchain and the dc3 tree, and `--check` is
    # the toolchain-free half. Same shape as `lane_registry.rs` grepping
    # `cross_sweep.py` for its registry read.
    for _row in $(printf '%s\n' "$METRICS" | grep '[^[:space:]]' | awk '{print $1}'); do
        if ! grep -qE "^ *emit +$_row +" "$0"; then
            echo "CHECK FAIL: metric '$_row' is registered but nothing emits it —" \
                 "it would render NO-RESULT forever and read as 'nothing to say'"
            fails=$((fails+1))
        fi
    done

    # …and every key `collect_gap`'s early-return paths report a reason for must
    # be a registered metric, and vice versa for the gap half. The four paths
    # share ONE list now (`GAP_KEYS`); this is what keeps it aligned with the
    # registry after it stopped being four copies.
    for _k in $GAP_KEYS; do
        grep -q "^$_k  *yes " <<EOF || { echo "CHECK FAIL: GAP_KEYS names '$_k', not in METRICS"; fails=$((fails+1)); }
$(printf '%s\n' "$METRICS" | grep '[^[:space:]]')
EOF
    done
    _ngap=0; for _k in $GAP_KEYS; do _ngap=$((_ngap+1)); done
    [ "$_ngap" -ge 15 ] || { echo "CHECK FAIL: GAP_KEYS has only $_ngap keys"; fails=$((fails+1)); }

    rm -rf "$work_dir"
    if [ "$fails" -eq 0 ]; then
        echo "STATUS CHECK: PASS — $nreg metrics registered, parsers pinned, absence renders NO-RESULT"
        exit 0
    fi
    echo "STATUS CHECK: FAIL — $fails problem(s)"
    exit 1
fi

# ---- collect -------------------------------------------------------------------

work_dir=$(mktemp -d "${TMPDIR:-/tmp}/c2rs-status.XXXXXX")
results_file="$work_dir/metrics"
: > "$results_file"
trap 'rm -rf "$work_dir"' EXIT INT TERM

pin_harness "$repo_root" "$work_dir" >"$work_dir/pin.log" 2>&1 || {
    echo "FATAL: could not build/pin the harness — refusing to report a status \
from a binary this run cannot name" >&2
    cat "$work_dir/pin.log" >&2
    exit 1
}
c2rs="$C2RS_PINNED"
# Just the content hash. `pin_harness` also stamps a tree HEAD, but printing both
# it and the renderer's HEAD invites the reader to compare two hashes taken at
# different instants — they legitimately differ when the tree moves mid-run, and
# a status block is the wrong place to explain that. One hash, one tree line.
identity=$(val_or_missing "$(sed -n 's/^  sha \([0-9a-f][0-9a-f]*\).*/\1/p' "$work_dir/pin.log" | head -1)")

collect_tests
collect_selftest
collect_perf
collect_gap

if [ "$do_raw" -eq 1 ]; then
    cat "$results_file"
    exit 0
fi

# ---- render --------------------------------------------------------------------
# The table is built by walking the REGISTRY, not the results file. A metric
# cannot vanish out of the table; it can only render NO-RESULT.

block="$work_dir/block.md"
{
    printf '<!-- BEGIN GENERATED: scripts/status.sh — do not hand-edit -->\n'
    _head=$(cd "$repo_root" && git rev-parse --short HEAD 2>/dev/null || echo '?')
    (cd "$repo_root" && git diff --quiet HEAD 2>/dev/null) || _head="$_head-dirty"
    # The WORKLOAD commit belongs in the stamp too. Half the numbers below are
    # measured against `../dc3-decomp`, which is a LIVE repo other agents merge
    # into: the census moved 706402/2463318 -> 706552/2463393 across one
    # morning's dc3 commits with `crates/` untouched, and nothing on this page
    # said which corpus either figure described. A workload-versioned number
    # whose corpus is unrecorded is not reproducible, and two of them are not
    # comparable. (Lane w-repro; lane w-prov measured the attribution.)
    _wl='?'
    if [ -d "$dc3" ]; then
        _wl=$(cd "$dc3" && git rev-parse --short HEAD 2>/dev/null || echo 'UNVERSIONED')
        if [ "$_wl" != 'UNVERSIONED' ]; then
            (cd "$dc3" && git diff --quiet HEAD 2>/dev/null) || _wl="$_wl-dirty"
        fi
    fi
    printf 'Collected %s · tree `%s` · binary `%s` · workload `%s`\n\n' \
        "$(date '+%Y-%m-%d')" "$_head" "$identity" "$_wl"
    printf '| metric | value |\n|---|---|\n'
    printf '%s\n' "$METRICS" | grep '[^[:space:]]' | while read -r k t l; do
        printf '| %s | %s |\n' "$l" "$(lookup "$k")"
    done
    printf '\n<!-- END GENERATED -->\n'
} > "$block"

nres=$(grep -c 'NO-RESULT' "$block" || true)
nskip=$(grep -c 'SKIP: toolchain absent' "$block" || true)

cat "$block"

if [ "$do_write" -eq 1 ]; then
    status_md="$repo_root/docs/STATUS.md"
    if [ ! -f "$status_md" ]; then
        echo "status.sh: $status_md does not exist — create it with the two markers first" >&2
        exit 1
    fi
    if ! grep -q '<!-- BEGIN GENERATED' "$status_md" || \
       ! grep -q '<!-- END GENERATED' "$status_md"; then
        echo "status.sh: $status_md has no generated block markers — refusing to guess where to write" >&2
        exit 1
    fi
    tmp="$work_dir/STATUS.md"
    awk -v blockfile="$block" '
        /<!-- BEGIN GENERATED/ { while ((getline line < blockfile) > 0) print line; skip=1; next }
        /<!-- END GENERATED/   { skip=0; next }
        !skip { print }
    ' "$status_md" > "$tmp"
    mv -f "$tmp" "$status_md"
    echo
    echo "wrote docs/STATUS.md"
fi

echo
if [ "$nskip" -gt 0 ]; then
    echo "STATUS: SKIPPED — $nskip toolchain-dependent metric(s) not collected (toolchain absent)"
elif [ "$nres" -gt 0 ]; then
    echo "STATUS: INCOMPLETE — $nres metric(s) produced NO-RESULT; the report is not a measurement"
else
    echo "STATUS: COMPLETE — every registered metric produced a value"
fi
exit 0

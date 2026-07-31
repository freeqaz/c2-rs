#!/bin/sh
# THE MODE-LANE GATE — run every lane in `scripts/lanes.txt`, one result each.
#
# ---- why this exists -----------------------------------------------------------
#
# `mode_lane.sh` runs ONE lane and has always worked. Nothing enumerated the lanes,
# so the set that actually ran on any given day was the set somebody remembered to
# type — and the four recorded through `docs/` (`/Ox`, `/O1`, `/O2`, `/Ox /Gy`)
# contain no `/EH` at all, on a workload that compiles `/EHsc` on every TU. Two
# `/EHsc` lanes were added, went green, and caught a live wrong-bytes emit every
# other lane was blind to; nothing whatsoever made them run again.
#
# A lane that exists but is not enumerated is a lane that does not run. The list is
# now data (`scripts/lanes.txt`) and this is the one command that runs it.
#
# ---- what this gate promises ---------------------------------------------------
#
# The promise is deliberately stated as a POSITIVE: **every lane in the registry
# produced a result, and the gate says how many.** Not "no lane failed" — the
# expensive failure class on this project is not a lane going red, it is a lane
# going ABSENT and the absence reading as zero. Eight instruments here have now
# reported green from an absence, including one whose every check `sed`-ed a number
# out of a report and read the missing number as 0, passing a run that graded
# literally nothing.
#
# So, concretely:
#
#   * Each lane must print a `LANE-RESULT` line. The gate parses that line and
#     re-derives the verdict from its fields. **A zero exit status is not accepted
#     as evidence that a lane ran** — a lane that dies, is killed, or is skipped by
#     the loop prints no such line and is reported `NO-RESULT`, which fails.
#   * `PASS` additionally requires `graded > 0`: a lane that submitted 197 fixtures
#     and graded none of them is not a passing lane, whatever its exit status.
#     (That is the toolchain PRESENT and every capture failing — a relative outdir,
#     an exhausted tmpfs inode table, a bad flag — and it is a different thing from
#     the toolchain being absent.)
#   * The result table is rendered by walking the REGISTRY, not by walking whatever
#     result files happen to exist, and the number of rendered rows is compared
#     against the registry length before any verdict is printed. A lane cannot
#     vanish out of the table.
#   * `SKIP` is its own verdict and prints as `SKIP`, never as `PASS`. An all-SKIP
#     run prints `GATE: SKIPPED` and says in the headline that nothing was graded.
#     It exits 0 because CLAUDE.md requires the toolchain-absent path to degrade
#     cleanly — but it cannot be mistaken for green by anything reading the output.
#   * A PARTIAL skip fails. If the toolchain is present, every lane runs; some
#     lanes skipping and others not means a lane declined for a lane-specific
#     reason, which is a fault, not a degradation.
#
# `--selftest` proves all of the above against fabricated lane logs, needs no
# toolchain, and is the answer to "has anyone ever seen this gate fail?".
#
# ---- usage ---------------------------------------------------------------------
#
#   scripts/gate.sh                       run every lane in the registry
#   scripts/gate.sh --lane O1-Oi-EHsc     run named lanes only (repeatable)
#   scripts/gate.sh --jobs 4              lanes in parallel (default 4)
#   scripts/gate.sh --list                print the registry and exit
#   scripts/gate.sh --check               validate the registry only; no toolchain
#   scripts/gate.sh --selftest            prove the gate fails when it should
#   scripts/gate.sh --work DIR            run directory (default /tmp/c2rs-gate-$$)
#
# Lane run directories stay PER LANE, inherited from `mode_lane.sh`, which uses one
# per mode precisely because a shared directory had concurrent lanes overwriting
# each other's flags file and report and the mismatch count then came out of
# whichever report won.
set -eu

TAB=$(printf '\t')
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
registry="${C2RS_LANES:-$repo_root/scripts/lanes.txt}"
jobs=4
work=""
want=""
mode=run

while [ $# -gt 0 ]; do
    case "$1" in
        --list)     mode=list ;;
        --check)    mode=check ;;
        --selftest) mode=selftest ;;
        --lane)     shift; want="$want $1" ;;
        --jobs)     shift; jobs="$1" ;;
        --work)     shift; work="$1" ;;
        --registry) shift; registry="$1" ;;
        -h|--help)  sed -n '2,/^set -eu$/p' "$0" | sed '$d'; exit 0 ;;
        *) echo "gate.sh: unknown argument '$1' (try --help)" >&2; exit 2 ;;
    esac
    shift
done
[ -n "$work" ] || work="/tmp/c2rs-gate-$$"

# --------------------------------------------------------------------------------
# The registry, parsed once. `slug<TAB>flags`, one per line, comments stripped.
# --------------------------------------------------------------------------------
parse_registry() {
    _pr_src="$1"; _pr_dst="$2"
    if [ ! -f "$_pr_src" ]; then
        echo "FATAL: no lane registry at $_pr_src" >&2
        return 1
    fi
    sed 's/#.*//' "$_pr_src" \
        | awk 'NF >= 2 { slug=$1; $1=""; sub(/^[ \t]+/,""); printf "%s\t%s\n", slug, $0 }' \
        > "$_pr_dst"
    _pr_n=$(wc -l < "$_pr_dst")
    # An EMPTY registry is a gate that runs nothing and exits 0 — the exact shape
    # this whole file exists to make impossible. It is a hard error, never a pass.
    if [ "$_pr_n" -eq 0 ]; then
        echo "FATAL: lane registry $_pr_src defines NO lanes." >&2
        echo "  A gate with an empty lane list grades nothing and would exit 0." >&2
        return 1
    fi
    _pr_dup=$(cut -f1 "$_pr_dst" | sort | uniq -d)
    if [ -n "$_pr_dup" ]; then
        echo "FATAL: duplicate lane slug(s) in $_pr_src: $_pr_dup" >&2
        echo "  Two rows under one slug means one silently replaces the other's" >&2
        echo "  result while the table still shows the expected number of rows." >&2
        return 1
    fi
    return 0
}

# --------------------------------------------------------------------------------
# Verdict for ONE lane, derived from its log. Deliberately a pure function of the
# log text, so `--selftest` can drive it with fabricated logs and no toolchain.
#
# Emits: <verdict>|<graded>|<total>|<match>|<mismatch>|<detail>
# --------------------------------------------------------------------------------
lane_verdict() {
    _lv_log="$1"; _lv_status="${2:-}"

    if [ ! -f "$_lv_log" ]; then
        echo "NO-RESULT|0|0|0|0|the lane produced no log at all"
        return 0
    fi
    _lv_line=$(grep -m1 '^LANE-RESULT ' "$_lv_log" 2>/dev/null || true)
    if [ -z "$_lv_line" ]; then
        echo "NO-RESULT|0|0|0|0|log has no LANE-RESULT line (exit ${_lv_status:-?})"
        return 0
    fi

    _lv_v=$(printf '%s\n' "$_lv_line" | awk '{print $2}')
    _lv_g=$(printf '%s\n' "$_lv_line" | sed -n 's/.* graded=\([0-9][0-9]*\).*/\1/p')
    _lv_t=$(printf '%s\n' "$_lv_line" | sed -n 's/.* total=\([0-9][0-9]*\).*/\1/p')
    _lv_m=$(printf '%s\n' "$_lv_line" | sed -n 's/.* match=\([0-9][0-9]*\).*/\1/p')
    _lv_x=$(printf '%s\n' "$_lv_line" | sed -n 's/.* mismatch=\([0-9][0-9]*\).*/\1/p')

    # Every field must be PRESENT. An unparseable result line is a lane that did
    # not report — NOT a lane that reported zeros. That distinction is the entire
    # bug class this gate is built around.
    if [ -z "$_lv_g" ] || [ -z "$_lv_t" ] || [ -z "$_lv_m" ] || [ -z "$_lv_x" ]; then
        echo "NO-RESULT|0|0|0|0|malformed LANE-RESULT line"
        return 0
    fi

    case "$_lv_v" in
    SKIP)
        echo "SKIP|0|$_lv_t|0|0|toolchain absent"
        ;;
    PASS)
        # Re-derive rather than believe. A lane claiming PASS while having graded
        # nothing, or while carrying a mismatch, is a lane wrong about itself, and
        # the gate is the second opinion.
        if [ "$_lv_g" -eq 0 ]; then
            echo "FAIL|0|$_lv_t|$_lv_m|$_lv_x|claimed PASS having graded 0 of $_lv_t"
        elif [ "$_lv_x" -ne 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|claimed PASS with mismatch=$_lv_x"
        elif [ "${_lv_status:-0}" != "0" ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|claimed PASS but exited $_lv_status"
        else
            echo "PASS|$_lv_g|$_lv_t|$_lv_m|$_lv_x|"
        fi
        ;;
    FAIL)
        if [ "$_lv_x" -ne 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|MISMATCH — the port emitted wrong bytes"
        elif [ "$_lv_g" -eq 0 ]; then
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|vacuous — 0 of $_lv_t graded"
        else
            echo "FAIL|$_lv_g|$_lv_t|$_lv_m|$_lv_x|lane reported FAIL"
        fi
        ;;
    *)
        echo "NO-RESULT|0|0|0|0|unrecognized verdict '$_lv_v'"
        ;;
    esac
    return 0
}

# --------------------------------------------------------------------------------
# Walk the REGISTRY (never the directory listing) and produce one row per lane.
# --------------------------------------------------------------------------------
collect() {
    _c_reg="$1"; _c_run="$2"; _c_out="$3"
    : > "$_c_out"
    while IFS="$TAB" read -r _c_slug _c_flags; do
        [ -n "$_c_slug" ] || continue
        _c_st=""
        if [ -f "$_c_run/$_c_slug.status" ]; then _c_st=$(cat "$_c_run/$_c_slug.status"); fi
        _c_v=$(lane_verdict "$_c_run/$_c_slug.log" "$_c_st")
        printf '%s\t%s\t%s\n' "$_c_slug" "$_c_flags" "$_c_v" >> "$_c_out"
    done < "$_c_reg"
}

decide() {
    _d_reg="$1"; _d_res="$2"; _d_run="${3:-}"
    _d_n=$(wc -l < "$_d_reg")
    _d_rows=$(wc -l < "$_d_res")

    echo
    echo "LANE                 VERDICT     graded/total  match  mismatch  flags"
    echo "-------------------- ---------- ------------- ------ --------- --------------------"
    awk -F"$TAB" '{
        split($3, f, "|")
        printf "%-20s %-10s %6s/%-6s %6s %9s  %s%s\n", $1, f[1], f[2], f[3], f[4], f[5],
               $2, (f[6] == "" ? "" : "   <- " f[6])
    }' "$_d_res"
    echo

    # COMPLETENESS FIRST, and as its own statement. If the table has fewer rows
    # than the registry has lanes, nothing else printed here means anything — a
    # lane silently dropped from the walk is precisely how an absence becomes a
    # green. Checked before any verdict is computed.
    if [ "$_d_rows" -ne "$_d_n" ]; then
        echo "GATE: FAIL — the registry has $_d_n lanes and the table has $_d_rows rows."
        echo "  Rows are produced by walking the registry, so this means the walk itself"
        echo "  broke. No verdict below this line would be trustworthy."
        return 1
    fi

    _d_pass=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="PASS") c++} END{print c+0}' "$_d_res")
    _d_fail=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL") c++} END{print c+0}' "$_d_res")
    _d_skip=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="SKIP") c++} END{print c+0}' "$_d_res")
    _d_none=$(awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="NO-RESULT") c++} END{print c+0}' "$_d_res")
    _d_graded=$(awk -F"$TAB" '{split($3,f,"|"); g+=f[2]} END{print g+0}' "$_d_res")

    echo "lanes:  $_d_n in the registry — $_d_pass PASS, $_d_fail FAIL, $_d_skip SKIP, $_d_none NO-RESULT"
    echo "graded: $_d_graded fixture-verdicts across all lanes"
    if [ -n "$_d_run" ] && [ -d "$_d_run" ]; then echo "logs:   $_d_run/<lane>.log"; fi

    if [ "$_d_none" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_none lane(s) produced NO RESULT:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="NO-RESULT") printf "    %-20s %-24s (%s)\n", $1, $2, f[6]}' "$_d_res"
        echo "  A lane that did not run is a failure, not a pass. Nothing in this run"
        echo "  establishes anything about the configurations those lanes cover."
        return 1
    fi

    if [ "$_d_fail" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_fail lane(s) failed:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL") printf "    %-20s %-24s (%s)\n", $1, $2, f[6]}' "$_d_res"
        if awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="FAIL" && f[5]+0 > 0) found=1} END{exit !found}' "$_d_res"; then
            echo
            echo "  *** A MISMATCH IS AN ALARM AND OUTRANKS EVERY OTHER PIECE OF WORK. ***"
            echo "  The real c2.dll under wibo plus a byte-exact obj compare is the sole"
            echo "  judge; outside its class the port must REFUSE, not mis-emit."
        fi
        return 1
    fi

    if [ "$_d_skip" -eq "$_d_n" ]; then
        echo
        echo "GATE: SKIPPED — all $_d_n lanes skipped, NOTHING WAS GRADED."
        echo "  The toolchain is absent (see CLAUDE.md); this exits 0 by design and is"
        echo "  NOT a green gate. This run establishes nothing about the port."
        return 0
    fi
    if [ "$_d_skip" -gt 0 ]; then
        echo
        echo "GATE: FAIL — $_d_skip of $_d_n lanes skipped while $_d_pass ran."
        echo "  Toolchain absence skips EVERY lane. A partial skip means a lane declined"
        echo "  for a reason of its own, which is a fault, not a degradation:"
        awk -F"$TAB" '{split($3,f,"|"); if (f[1]=="SKIP") printf "    %-20s %s\n", $1, $2}' "$_d_res"
        return 1
    fi

    echo
    echo "GATE: PASS — $_d_pass/$_d_n lanes ran and every one of them graded a corpus."
    return 0
}

# --------------------------------------------------------------------------------
# Registry load + `--lane` filter, shared by every mode.
# --------------------------------------------------------------------------------
mkdir -p "$work"
reg="$work/registry.tsv"
parse_registry "$registry" "$reg"

# `--lane` filters the registry, and the filtered list then IS the registry for
# every check above — so `--lane` naming nothing is an empty registry and a hard
# error, never a run of zero lanes that exits 0.
if [ -n "$want" ]; then
    sel="$work/selected.tsv"; : > "$sel"
    for w in $want; do
        if ! awk -F"$TAB" -v w="$w" '$1==w{found=1} END{exit !found}' "$reg"; then
            echo "FATAL: --lane '$w' is not in $registry. Known lanes:" >&2
            cut -f1 "$reg" | sed 's/^/    /' >&2
            exit 2
        fi
        awk -F"$TAB" -v w="$w" '$1==w' "$reg" >> "$sel"
    done
    reg="$sel"
fi
nlanes=$(wc -l < "$reg")

case "$mode" in
list)
    echo "lane registry: $registry  ($nlanes lanes)"
    awk -F"$TAB" '{printf "  %-20s %s\n", $1, $2}' "$reg"
    exit 0 ;;
check)
    echo "lane registry: $registry"
    echo "  $nlanes lanes, slugs unique, every row parses."
    awk -F"$TAB" '{printf "  %-20s %s\n", $1, $2}' "$reg"
    exit 0 ;;
esac

if [ "$mode" = selftest ]; then
    # ----------------------------------------------------------------------------
    # Prove the gate fails when it should, with no toolchain and no compiler.
    # Every case fabricates lane logs and drives the REAL collect+decide path —
    # not a reimplementation of it, which would only prove the copy agrees.
    #
    # No command substitution anywhere in the loop: a `fails` counter incremented
    # inside `$(...)` lives in a subshell and is discarded, which would make this
    # selftest itself an instrument that reports green from an absence.
    # ----------------------------------------------------------------------------
    st="$work/selftest"; rm -rf "$st"; mkdir -p "$st"
    printf 'A\t/O1\nB\t/O1 /EHsc\n' > "$st/reg.tsv"
    fails=0
    cases=0
    CASE_DIR=""

    check_that() {  # <label> <ok?0/1>
        if [ "$2" -eq 0 ]; then
            printf '        %s\n' "also: $1"
        else
            printf '  FAIL  %s\n' "also: $1"
            fails=$((fails + 1))
        fi
    }

    run_case() {  # <name> <PASS|FAIL> <slug=body>...
        _rc_name="$1"; _rc_want="$2"; shift 2
        CASE_DIR="$st/$_rc_name"
        rm -rf "$CASE_DIR"; mkdir -p "$CASE_DIR"
        for _rc_spec in "$@"; do
            _rc_slug=${_rc_spec%%=*}
            _rc_body=${_rc_spec#*=}
            case "$_rc_body" in
                MISSING) : ;;
                NOLINE)
                    echo "grading 197 fixtures at /O1" > "$CASE_DIR/$_rc_slug.log"
                    echo 0 > "$CASE_DIR/$_rc_slug.status" ;;
                *)
                    printf '%s\n' "$_rc_body" > "$CASE_DIR/$_rc_slug.log"
                    echo 0 > "$CASE_DIR/$_rc_slug.status" ;;
            esac
        done
        collect "$st/reg.tsv" "$CASE_DIR" "$CASE_DIR/results.tsv"
        _rc_got=PASS
        if ! decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "" > "$CASE_DIR/out.txt" 2>&1; then
            _rc_got=FAIL
        fi
        _rc_hdl=$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt" || echo 'GATE: <none printed>')
        cases=$((cases + 1))
        if [ "$_rc_got" = "$_rc_want" ]; then
            printf '  ok    %-32s %s\n' "$_rc_name" "$_rc_hdl"
        else
            printf '  FAIL  %-32s wanted %s, got %s — %s\n' "$_rc_name" "$_rc_want" "$_rc_got" "$_rc_hdl"
            fails=$((fails + 1))
        fi
    }
    saw()    { if grep -q "$1" "$CASE_DIR/out.txt"; then check_that "$2" 0; else check_that "$2" 1; fi; }
    saw_no() { if grep -q "$1" "$CASE_DIR/out.txt"; then check_that "$2" 1; else check_that "$2" 0; fi; }

    P='LANE-RESULT PASS flags=[/O1 /GS- /c] graded=197 total=197 match=91 mismatch=0'
    M='LANE-RESULT FAIL flags=[/O1 /EHsc /GS- /c] graded=197 total=197 match=90 mismatch=1'
    V='LANE-RESULT FAIL flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    S='LANE-RESULT SKIP flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    L1='LANE-RESULT PASS flags=[/O1 /EHsc /GS- /c] graded=0 total=197 match=0 mismatch=0'
    L2='LANE-RESULT PASS flags=[/O1 /EHsc /GS- /c] graded=197 total=197 match=90 mismatch=3'

    echo "gate.sh --selftest: driving the real collect+decide with fabricated lane logs"
    echo

    run_case both-pass PASS "A=$P" "B=$P"
    saw 'GATE: PASS' 'a wholly green run does say PASS'

    run_case lane-B-mismatch FAIL "A=$P" "B=$M"
    saw '^    B ' 'the failing lane is NAMED'
    saw 'ALARM'   'a mismatch raises the alarm banner'

    run_case lane-B-vacuous              FAIL "A=$P" "B=$V"
    run_case lane-B-no-log-at-all        FAIL "A=$P" "B=MISSING"
    saw 'NO RESULT' 'a lane that never ran is NO RESULT, not a pass'

    run_case lane-B-exit-0-no-result     FAIL "A=$P" "B=NOLINE"
    saw 'NO RESULT' 'exit 0 alone is not evidence a lane ran'

    run_case lane-B-lies-graded-0        FAIL "A=$P" "B=$L1"
    run_case lane-B-lies-with-mismatch   FAIL "A=$P" "B=$L2"
    run_case both-absent-is-not-a-skip   FAIL "A=MISSING" "B=MISSING"

    run_case all-skip PASS "A=$S" "B=$S"
    saw    'GATE: SKIPPED' 'all-skip says SKIPPED and that nothing was graded'
    saw_no 'GATE: PASS'    'all-skip never says PASS'

    run_case partial-skip FAIL "A=$P" "B=$S"

    # The completeness assertion itself: a table short a row must fail even when
    # every row it does contain is a PASS.
    CASE_DIR="$st/short-table"; mkdir -p "$CASE_DIR"
    printf 'A\t/O1\tPASS|197|197|91|0|\n' > "$CASE_DIR/results.tsv"
    cases=$((cases + 1))
    if decide "$st/reg.tsv" "$CASE_DIR/results.tsv" "" > "$CASE_DIR/out.txt" 2>&1; then
        printf '  FAIL  %-32s a 1-row table for a 2-lane registry PASSED\n' short-table
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s\n' short-table "$(grep -m1 '^GATE: ' "$CASE_DIR/out.txt")"
    fi

    # An empty registry must be a hard error, not a run of zero lanes.
    : > "$st/empty.txt"
    cases=$((cases + 1))
    if parse_registry "$st/empty.txt" "$st/empty.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s an empty registry parsed clean\n' empty-registry
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused (a gate over 0 lanes cannot exist)\n' empty-registry
    fi

    # As must a duplicated slug.
    printf 'A /O1\nA /Ox\n' > "$st/dup.txt"
    cases=$((cases + 1))
    if parse_registry "$st/dup.txt" "$st/dup.tsv" >/dev/null 2>&1; then
        printf '  FAIL  %-32s a duplicated slug parsed clean\n' duplicate-slug
        fails=$((fails + 1))
    else
        printf '  ok    %-32s refused\n' duplicate-slug
    fi

    # And the registry actually shipped must parse, and must carry an /EH lane —
    # the specific hole this whole registry was built to close. If somebody
    # deletes those rows, this is what notices.
    cases=$((cases + 1))
    _n_real=$(wc -l < "$work/registry.tsv")
    _n_eh=$(cut -f2 "$work/registry.tsv" | grep -c -- '/EH' || true)
    if [ "$_n_real" -lt 2 ] || [ "$_n_eh" -lt 1 ]; then
        printf '  FAIL  %-32s %s lanes, %s of them /EH\n' shipped-registry "$_n_real" "$_n_eh"
        fails=$((fails + 1))
    else
        printf '  ok    %-32s %s lanes, %s of them compile /EH\n' shipped-registry "$_n_real" "$_n_eh"
    fi

    echo
    if [ "$cases" -lt 14 ]; then
        echo "gate.sh --selftest: FAIL — only $cases cases ran; the selftest itself was"
        echo "  truncated, and a truncated selftest is the failure it exists to catch."
        exit 1
    fi
    if [ "$fails" -eq 0 ]; then
        echo "gate.sh --selftest: PASS — $cases cases, the gate fails on every one that should."
        exit 0
    fi
    echo "gate.sh --selftest: FAIL — $fails of $cases checks did not behave as required."
    exit 1
fi

# --------------------------------------------------------------------------------
# run
# --------------------------------------------------------------------------------
echo "lane gate: $nlanes lanes from $registry"
echo "  run dir: $work   (per-lane run dirs under $work/lanes/)"

# Pin ONE binary for the whole gate and hand it to every lane. Stronger than each
# lane pinning its own copy: all $nlanes lanes are then provably grading the same
# code, and the sha below is the answer to "which binary produced this table".
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$work"
export C2RS_BIN="$C2RS_PINNED"
export C2RS_MODE_LANE_WORK="$work/lanes"
: "${C2RS_JOBS:=8}"
export C2RS_JOBS

started=$(date +%s)
running=0
while IFS="$TAB" read -r slug flags; do
    [ -n "$slug" ] || continue
    (
        st=0
        # shellcheck disable=SC2086
        sh "$repo_root/scripts/mode_lane.sh" $flags > "$work/$slug.log" 2>&1 || st=$?
        echo "$st" > "$work/$slug.status"
    ) &
    running=$((running + 1))
    if [ "$running" -ge "$jobs" ]; then wait; running=0; fi
done < "$reg"
wait
elapsed=$(( $(date +%s) - started ))

collect "$reg" "$work" "$work/results.tsv"
echo
echo "wall clock: ${elapsed}s for $nlanes lanes at --jobs $jobs (C2RS_JOBS=$C2RS_JOBS)"
decide "$reg" "$work/results.tsv" "$work"

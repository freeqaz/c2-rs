#!/bin/sh
# mutate_gate.sh — break the LADDER-RED ROW eight ways and show `--selftest` go red.
#
#     sh work/w-hatchroot/mutate_gate.sh
#
# Lane `w-hatchroot`, board #1406's second half. `work/w-hatchroot/row_red.sh`
# fires the RUNNER's five refusals against scratch checkouts. This is the other
# half of the same argument, aimed at the CLASSIFIER and the RULING: a
# `--selftest` that passes proves nothing unless breaking the thing it checks
# makes it fail, and **which** case it makes fail.
#
# The two traps this file is written against, both learned the hard way here:
#
#   TRAP A — an early guard can make a later assertion unreachable. Each mutation
#   below removes EXACTLY ONE check and holds every earlier quantity fixed, so
#   the case that reddens is the case whose guard was removed.
#
#   TRAP B — a shared message prefix lets a later gate's refusal satisfy an
#   earlier case's expectation. M7 is that mutation in person: it collapses
#   `LADDER-ARMS-FAILED` onto `hatch-red`'s `ARMS-FAILED`, and two cases must go
#   red — the one asserting the word AND the cross-row distinctness assertion.
#   A lane had two of six mutations pass silently for exactly this.
#
# A mutation that reddens NOTHING is reported as a failure of this file, not as a
# pass: it means the case it was aimed at is not asserting what it claims to.
# So is a mutation that reddens a case it was not aimed at while missing its own.
set -u

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
GATE="$repo_root/scripts/gate.sh"
# TWO CONSTRAINTS ON WHERE THE MUTANT LIVES, and the second cost an hour.
#   * `scripts/`, because `repo_root` is `dirname($0)/..`.
#   * a filename CONTAINING `gate.sh`, because `gate_pid_live` decides a pid is a
#     live gate by grepping `/proc/<pid>/cmdline` for `gate\.sh` — and the four
#     reaper cases drive that against THIS shell. Named `.gate-mutant.sh` the
#     first time, those four failed on every mutation INCLUDING an unmutated
#     copy: a noise floor of 4 that would have made a surviving mutation look
#     red. Measured rather than reasoned about — the CONTROL run below is what
#     says the floor is 0.
MUT="$repo_root/scripts/.mut-gate.sh"
LOG="$(dirname "$0")/mutate_gate.log"
trap 'rm -f "$MUT"' EXIT INT TERM

BASE_CASES=$(sh "$GATE" --selftest 2>&1 | sed -n 's/^gate.sh --selftest: PASS — \([0-9]*\) cases.*/\1/p')
echo "unmutated, in place: $BASE_CASES cases, PASS"

# THE CONTROL: an UNMUTATED copy, run from the mutant's own path. Anything red
# here is an artifact of being a copy and not of any mutation, and every
# mutation's reddened set is read against it.
cp "$GATE" "$MUT"; chmod +x "$MUT"
sh "$MUT" --selftest > "$LOG.M0-control" 2>&1
CONTROL_RC=$?
CONTROL_RED=$(sed -n 's/^  FAIL  \([a-z0-9-]*\) .*/\1/p' "$LOG.M0-control" | sort -u | tr '\n' ' ')
rm -f "$MUT"
echo "unmutated, as a copy at $(basename "$MUT"): exit $CONTROL_RC, reddened: ${CONTROL_RED:-NONE}"
if [ -n "$CONTROL_RED" ]; then
    echo "*** THE CONTROL IS NOT CLEAN. Every mutation below is read against a"
    echo "*** noise floor, and a mutation that reddens only these SURVIVED."
fi
echo

PASS=0
FAIL=0

run_mutation() {   # <id> <expect-case-substring...> ; python fragment on stdin
    _id="$1"; shift
    python3 - "$GATE" "$MUT" > /dev/null || { echo "$_id: MUTATION DID NOT APPLY"; FAIL=$((FAIL+1)); return; }
    chmod +x "$MUT"
    sh "$MUT" --selftest > "$LOG.$_id" 2>&1
    _rc=$?
    _red=$(sed -n 's/^  FAIL  \([a-z0-9-]*\) .*/\1/p' "$LOG.$_id" | sort -u | tr '\n' ' ')
    printf '%s\n' "----------------------------------------------------------------------"
    printf '%s\n' "$_id"
    printf '  selftest exit : %d\n' "$_rc"
    printf '  cases reddened: %s\n' "${_red:-NONE}"
    _ok=1
    _new=""
    for _c in $_red; do
        case " $CONTROL_RED " in *" $_c "*) : ;; *) _new="$_new $_c" ;; esac
    done
    printf '  new vs control: %s\n' "${_new:-NONE}"
    if [ -z "$_new" ]; then
        printf '  => *** MUTATION SURVIVED *** nothing reddened beyond the control\n'
    else
        _ok=0
        for _want in "$@"; do
            case " $_red " in
                *" $_want "*) : ;;
                *) printf '  => *** WRONG CASE *** expected %s to redden\n' "$_want"; _ok=1 ;;
            esac
        done
        [ "$_ok" -eq 0 ] && printf '  => RED as expected\n'
    fi
    if [ "$_ok" -eq 0 ]; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); fi
    rm -f "$MUT"
}

# ---- M1: no TRUNCATED check ---------------------------------------------------
run_mutation M1-drop-truncated ladderred-short-run-is-not-a-pass <<'PY'
import sys
s = open(sys.argv[1]).read()
old = """        if [ "$_lv_exp" -le 0 ] || [ "$_lv_n" -ne "$_lv_exp" ]; then
            echo "FAIL|$_lv_n|$_lv_exp|$_lv_r|$_lv_g|LADDER-TRUNCATED $_lv_n of $_lv_exp declared arms ran — a short run is not a pass"
            return 0
        fi
"""
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, ""))
PY

# ---- M2: no VACUOUS check -----------------------------------------------------
run_mutation M2-drop-vacuous ladderred-no-red-arms-is-vacuous <<'PY'
import sys
s = open(sys.argv[1]).read()
old = """        if [ "$_lv_r" -le 0 ] || [ "$_lv_g" -le 0 ] || [ $((_lv_r + _lv_g)) -ne "$_lv_n" ]; then
            echo "FAIL|$_lv_n|$_lv_exp|$_lv_r|$_lv_g|LADDER-VACUOUS $_lv_r red and $_lv_g green do not account for $_lv_n arms"
            return 0
        fi
"""
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, ""))
PY

# ---- M3: no EXIT check --------------------------------------------------------
run_mutation M3-drop-exit ladderred-green-then-nonzero-exit <<'PY'
import sys
s = open(sys.argv[1]).read()
old = """        if [ "$_lv_st" != "0" ]; then
            echo "FAIL|$_lv_n|$_lv_exp|$_lv_r|$_lv_g|LADDER-EXIT reported every arm passing and then exited $_lv_st"
            return 0
        fi
"""
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, ""))
PY

# ---- M4: an UNRECOGNIZED log falls through to PASS ----------------------------
run_mutation M4-unrecognized-passes ladderred-junk-log-is-no-result <<'PY'
import sys
s = open(sys.argv[1]).read()
old = '    echo "NO-RESULT|0|$_lv_exp|0|0|LADDER-UNRECOGNIZED no ALL-ARMS-PASS line and no FAILED line — an unenumerated outcome is the next silence"\n'
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, '    echo "PASS|0|$_lv_exp|0|0|"\n'))
PY

# ---- M5: absent ladder tuple is no longer a failure ---------------------------
run_mutation M5-absent-tuple-allowed ladderred-absent-tuple-fails-the-gate <<'PY'
import sys
s = open(sys.argv[1]).read()
old = """    if [ -z "$_d_lr" ]; then
        echo
        echo "GATE: FAIL — no ladder-red verdict was produced at all."
"""
new = """    if false; then
        echo
        echo "GATE: FAIL — no ladder-red verdict was produced at all."
"""
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, new))
PY

# ---- M6: REFUSED reddens the gate instead of exiting 0 ------------------------
# The one that must not redden a peer. Note it reddens the BOTH-ROWS case too,
# and that is correct rather than sloppy: both cases assert a zero exit.
run_mutation M6-refused-is-a-fail ladderred-refused-exits-zero bothrows-refused-print-both-suffixes <<'PY'
import sys
s = open(sys.argv[1]).read()
old = """        PASS|REFUSED) : ;;
        *)
            echo
            echo "GATE: FAIL — ladder-red reported an unrecognized verdict '$_d_lrv'."
"""
new = """        PASS) : ;;
        *)
            echo
            echo "GATE: FAIL — ladder-red reported an unrecognized verdict '$_d_lrv'."
"""
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, new))
PY

# ---- M7: the headline suffix goes away ---------------------------------------
# A REFUSED row that exits 0 and prints an UNQUALIFIED PASS is a silent skip,
# which is the whole reason the suffix exists.
run_mutation M7-no-headline-suffix ladderred-refused-exits-zero bothrows-refused-print-both-suffixes <<'PY'
import sys
s = open(sys.argv[1]).read()
old = '    if [ "$_d_lrv" = REFUSED ]; then _d_hrq="$_d_hrq (LADDER-RED REFUSED)"; fi\n'
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, ""))
PY

# ---- M8: TRAP B — the word collapses onto hatch-red's -------------------------
run_mutation M8-word-collapse ladderred-failed-arm-is-a-fail ladderred-shares-no-word-with-hatchred <<'PY'
import sys
s = open(sys.argv[1]).read()
old = 'echo "FAIL|0|$_lv_exp|0|0|LADDER-ARMS-FAILED $_lv_d"'
new = 'echo "FAIL|0|$_lv_exp|0|0|ARMS-FAILED $_lv_d"'
assert s.count(old) == 1
open(sys.argv[2], "w").write(s.replace(old, new))
PY

echo
echo "======================================================================"
echo "mutations: $PASS reddened the case they were aimed at, $FAIL did not"
[ "$FAIL" -eq 0 ] || { echo "FAILED"; exit 1; }
echo "ALL $PASS MUTATIONS RED — every one on the case its guard belongs to"

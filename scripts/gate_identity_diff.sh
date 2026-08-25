#!/bin/sh
# gate_identity_diff.sh — the required-zero identity diff, as a committed
# instrument. Board **#3579**, closing `#3451`'s complaint for this procedure.
#
# Needs `sed`, `awk`, `grep`, `diff`. No toolchain, no build, no network.
#
# ---- why this exists ---------------------------------------------------------
#
# `work/coordinator/gatebase/HOWTO_DIFF.md` states the rule every merge touching
# `crates/` is bound by:
#
#   > every merge touching `crates/` diffs the count-bearing rows against the
#    PRE-MERGE base and quotes the diff in the merge message. Never the exit
#    code alone.
#
# It was adopted after `w-latent` measured that a change can be `GATE: PASS` +
# `GATE_EXIT=0` + `mismatch 0` and still cost **six matched TUs**. **`mismatch 0`
# is NOT `match unchanged`**, no gate's own verdict can see that class, and the
# identity diff is the only thing that can.
#
# And it is a PROSE PROCEDURE that every lane retypes. `#3451` is the standing
# row for exactly this — *"the instrument four lanes have retyped and none
# committed"* — filed against the cost protocol, which `w-permute` then
# committed as `scripts/scan_pair.sh`. This is the same shape one artifact
# over, and the three steps below are each a place a hand-typed version goes
# wrong:
#
#   1. **Cut to the count-bearing columns BEFORE diffing.** The `flags` column
#      carries free prose that changes with invocation flags and tree state, so
#      a whole-line diff shows rows "moving" between two runs that graded
#      identically. That cost `w-latent` two messages of wrong denominators.
#   2. **Normalise the run dir.** `gate.sh` names it after its own PID, so
#      every diff is otherwise non-empty for a reason that means nothing.
#   3. **Drop the two `n/a`-mismatch rows** (`hatch-red`, `ladder-red`). The
#      denominator is **21** — 18 mode lanes + `expr-sweep` + `mode-cross` +
#      `debug-lane` — and it is verified BY ENUMERATION, not asserted. (23 data
#      rows total; "25" was line-counting and is wrong.)
#
# ---- what it refuses ---------------------------------------------------------
#
# A denominator, because only a denominator catches an absence (`#3470`,
# `#1002`). A table that yielded a row count other than 21 is not a table this
# script will diff — a silently-short extraction and a genuinely-identical pair
# both produce "no differences", and those must not look alike.
#
# It does NOT read `GATE:` or any exit status. That is deliberate and it is the
# whole point of the rule: `gate.sh` prints `GATE: REFUSED (DIRTY crates/)` and
# **exits 0**, so a status is not evidence. Compare a count.
#
#   exit 0  the 21 rows are identical
#   exit 1  they differ — the moved rows are printed
#   exit 2  could not run (missing file, or a row count that is not 21)
#   exit 3  --self-test found the diff cannot detect a known signature
#
# Usage:
#   scripts/gate_identity_diff.sh BASE.txt TIP.txt
#   scripts/gate_identity_diff.sh --rows TABLE.txt    print the 21 rows only
#   scripts/gate_identity_diff.sh --self-test         fabricate #3515's
#                                                     signature and require the
#                                                     diff to find it

set -eu

WANT_ROWS=21

rows() {
    [ -f "$1" ] || { echo "gate_identity_diff.sh: no such file: $1" >&2; return 2; }
    # The trailing `|| true` is load-bearing and was added after this script's
    # own test caught its absence. A `grep` that matches nothing exits 1, and
    # under `set -e` that aborts the command substitution in `checked_rows`
    # BEFORE the row-count check runs — so pointing the tool at a file that is
    # not a gate table exited 2 with the correct verdict and **no diagnostic at
    # all**. Right answer, unreadable reason, on exactly the input a mistyped
    # path produces.
    sed 's|/tmp/c2rs-gate-[0-9]*|RUNDIR|g' "$1" \
      | awk '/^[A-Za-z][A-Za-z0-9-]* +(PASS|FAIL|REFUSED|SKIP|NO-RESULT) /{print $1, $2, $3, $4}' \
      | grep -Ev '^(hatch-red|ladder-red) ' \
      | grep -v '^LANE ' || true
}

checked_rows() { # <file> <label>
    _out="$(rows "$1")" || return 2
    _n="$(printf '%s' "$_out" | grep -c . || true)"
    if [ "$_n" -ne "$WANT_ROWS" ]; then
        echo "gate_identity_diff.sh: $2 yielded $_n count-bearing rows, expected $WANT_ROWS." >&2
        echo "  A short extraction and an identical pair both print 'no differences'." >&2
        echo "  Refusing to diff (board #3470, #1002)." >&2
        return 2
    fi
    printf '%s\n' "$_out"
}

do_diff() {
    _b="$(mktemp)"; _t="$(mktemp)"
    trap 'rm -f "$_b" "$_t"' EXIT INT TERM
    checked_rows "$1" "base ($1)" > "$_b" || return 2
    checked_rows "$2" "tip  ($2)" > "$_t" || return 2
    echo "count-bearing rows: $WANT_ROWS base, $WANT_ROWS tip (enumerated, not asserted)"
    if diff "$_b" "$_t" > /dev/null 2>&1; then
        echo "IDENTITY DIFF: 0 lines over $WANT_ROWS rows — required-zero byte delta HOLDS"
        return 0
    fi
    echo "IDENTITY DIFF: ROWS MOVED"
    diff "$_b" "$_t" | grep -E '^[<>]' | sed 's/^/  /'
    _lines="$(diff "$_b" "$_t" | grep -cE '^[<>]' || true)"
    _moved="$(diff "$_b" "$_t" | grep -c '^<' || true)"
    echo "  $_lines diff lines = $_moved row(s) moved"
    echo "  See HOWTO_DIFF.md's one-TU-refused signature before concluding: six"
    echo "  /O1 lanes at -1 each plus debug-lane at the SUM, everything else"
    echo "  still, is ONE TU being refused. A distributed change looks different."
    return 1
}

# ---- --self-test -------------------------------------------------------------
#
# A diff nobody has watched produce a nonzero is not a diff (`#1236`). This
# fabricates `#3515`'s measured one-TU-refused signature into a synthetic table
# and requires the procedure to find EXACTLY it: 14 lines over 7 rows.
#
# The table is generated here rather than read from `work/`, so the self-test
# is portable and cannot go red because somebody's scratch directory was reaped
# — which is this lane's own subject (`#3552`).
self_test() {
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/c2rs-gatediff-selftest-XXXXXX")"
    trap 'rm -rf "$tmp"' EXIT INT TERM
    base="$tmp/base.txt"
    {
        echo "  run dir: /tmp/c2rs-gate-999999"
        echo "LANE                 VERDICT     graded/total  match  mismatch  flags"
        for r in "O1 186" "O1-EHsc 187" "O1-Oi 188" "O1-Oi-EHsc 189" \
                 "Ox 157" "Ox-EHsc 157" "Ox-Gy 157" "Ox-Gy-EHsc 157" \
                 "O2 163" "O2-EHsc 163" "Od 21" "Od-EHsc 21" \
                 "O1-Oi-GR 188" "O1-Oi-EHsc-GR 189" "Ox-GR 157" "Ox-EHsc-GR 157" \
                 "Od-GR 21" "Od-EHsc-GR 21"; do
            set -- $r
            printf '%-20s PASS          391/391       %3d         0  /flags prose\n' "$1" "$2"
        done
        printf '%-20s PASS        19556/19556   19460         0  generated cases\n' expr-sweep
        printf '%-20s PASS        90812/90812   90424         0  case-lane cells\n' mode-cross
        printf '%-20s PASS           18/18       2479         0  debug binary\n'    debug-lane
        printf '%-20s REFUSED         0/14          0       n/a  arms\n'            hatch-red
        printf '%-20s PASS            8/8           8       n/a  width table\n'     ladder-red
    } > "$base"

    fails=0

    n="$(rows "$base" | grep -c . || true)"
    if [ "$n" -eq "$WANT_ROWS" ]; then
        echo "  enumeration: $n count-bearing rows (hatch-red/ladder-red dropped)"
    else
        echo "  ENUMERATION WRONG: $n rows, expected $WANT_ROWS" >&2
        fails=$((fails + 1))
    fi

    # CONTROL: identical inputs must be silent, and must exit 0.
    if do_diff "$base" "$base" >/dev/null 2>&1; then
        echo "  control: a table against itself                      -> 0 lines, exit 0"
    else
        echo "  CONTROL FAILED: a table differs from itself" >&2
        fails=$((fails + 1))
    fi

    # THE SIGNATURE. #3515, measured: INLINE_DECLINE_LOOP_BYTES 80 -> 4096.
    tip="$tmp/tip.txt"
    sed -e 's/^\(O1  *PASS  *391\/391  *\)186/\1185/' \
        -e 's/^\(O1-EHsc  *PASS  *391\/391  *\)187/\1186/' \
        -e 's/^\(O1-Oi  *PASS  *391\/391  *\)188/\1187/' \
        -e 's/^\(O1-Oi-EHsc  *PASS  *391\/391  *\)189/\1188/' \
        -e 's/^\(O1-Oi-GR  *PASS  *391\/391  *\)188/\1187/' \
        -e 's/^\(O1-Oi-EHsc-GR  *PASS  *391\/391  *\)189/\1188/' \
        -e 's/^\(debug-lane  *PASS  *18\/18  *\)2479/\12473/' \
        "$base" > "$tip"

    # THE MUTATION MUST HAVE APPLIED. A sed that matched nothing leaves a
    # CLEAN copy and the case below "passes" by testing the control twice —
    # #3516's mutation-not-applied failure, arriving through the control.
    if cmp -s "$base" "$tip"; then
        echo "  FABRICATION DID NOT APPLY — the signature case would test the control twice" >&2
        fails=$((fails + 1))
    else
        out="$(do_diff "$base" "$tip" 2>&1 || true)"
        lines="$(printf '%s\n' "$out" | grep -cE '^ +[<>]' || true)"
        moved="$(printf '%s\n' "$out" | grep -cE '^ +<' || true)"
        if [ "$lines" -eq 14 ] && [ "$moved" -eq 7 ]; then
            echo "  #3515's one-TU-refused signature                    -> 14 lines, 7 rows"
        else
            echo "  SIGNATURE WRONG: got $lines lines / $moved rows, expected 14 / 7" >&2
            printf '%s\n' "$out" >&2
            fails=$((fails + 1))
        fi
        if do_diff "$base" "$tip" >/dev/null 2>&1; then
            echo "  SIGNATURE CASE EXITED 0 — a moved row did not fail the diff" >&2
            fails=$((fails + 1))
        else
            echo "  the signature case exits NONZERO"
        fi
    fi

    # THE DENOMINATOR REFUSAL: a truncated table is not a clean diff.
    short="$tmp/short.txt"
    head -6 "$base" > "$short"
    do_diff "$base" "$short" >/dev/null 2>&1 && st=0 || st=$?
    if [ "$st" -eq 2 ]; then
        echo "  a TRUNCATED table -> exit 2 (a short extraction is not 'no differences')"
    else
        echo "  TRUNCATED table reported exit $st, expected 2" >&2
        fails=$((fails + 1))
    fi

    if [ "$fails" -gt 0 ]; then
        echo "SELF-TEST FAIL: $fails case(s)." >&2
        return 3
    fi
    echo "SELF-TEST PASS: enumeration 21, control silent, #3515's signature found"
    echo "  exactly (14 lines / 7 rows) and nonzero, truncation refused."
    return 0
}

case "${1:-}" in
    --self-test) self_test ;;
    --rows)      shift; [ $# -ge 1 ] || { echo "usage: $0 --rows TABLE.txt" >&2; exit 2; }
                 checked_rows "$1" "$1" ;;
    "")          echo "usage: $0 BASE.txt TIP.txt | --rows T | --self-test" >&2; exit 2 ;;
    *)           [ $# -eq 2 ] || { echo "usage: $0 BASE.txt TIP.txt" >&2; exit 2; }
                 do_diff "$1" "$2" ;;
esac

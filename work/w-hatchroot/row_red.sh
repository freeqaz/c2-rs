#!/bin/sh
# row_red.sh — make the LADDER-RED ROW's RUNNER-side refusals go red on purpose.
#
#     sh work/w-hatchroot/row_red.sh
#
# Lane `w-hatchroot`, board #1406's second half. `gate.sh --selftest` drives the
# CLASSIFIER (`ladder_red_verdict`, a pure function of a log) and the RULING
# (`decide`) with fabricated tuples — 16 cases of it. What it cannot drive is the
# RUNNER (`ladder_red_run`), because that one touches `git` and a real tree.
#
# A guard nobody has seen fire is a guard nobody has tested. So this fires the
# runner's five words against **real scratch checkouts**, never against this one:
#
#   LADDER-MISSING     ladder_red.py absent from the tree
#   LADDER-NOSUBJECT   ladder.py absent — "the instrument is gone", which is a
#                      DIFFERENT fact from "the guards stopped working" and must
#                      not be allowed to wear ARMS-FAILED's word
#   LADDER-NOGIT       the tree is not a checkout, so the interlock is unreadable
#   LADDER-DIRTY       the width table the arms read differs from HEAD
#   LADDER-RESIDUE     the arms modified crates/, which they have no path to do
#
# **Every one of them is a real reproduction, and `LADDER-RESIDUE` is the one
# worth arguing about.** It is a postcondition on a code path that does not
# exist — `ladder_red.py` never writes into `crates/` — so it cannot be fired by
# running the real arms at all. It is fired here by a scratch tree whose
# `ladder_red.py` DOES write there, which is what the postcondition is actually
# defending against: a future edit, or a SIGKILL between the write and the
# restore. That is weaker evidence than a defect reproduction and this file says
# so rather than counting it as one.
#
# The functions are not reimplemented: they are EXTRACTED FROM `scripts/gate.sh`
# and sourced. A red test against a copy of the code is a red test against a copy
# of the code.
set -u

repo_root_real="$(cd "$(dirname "$0")/../.." && pwd)"
GATE="$repo_root_real/scripts/gate.sh"
[ -f "$GATE" ] || { echo "no $GATE"; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT INT TERM

# ---- extract the two functions, verbatim, from the gate -----------------------
awk '
    /^ladder_red_verdict\(\) \{/ { on = 1 }
    on { print }
    /^ladder_red_run\(\) \{/ { inrun = 1 }
    inrun && /^\}/ { exit }
' "$GATE" > "$TMP/rows.sh"
_n=$(grep -c . "$TMP/rows.sh")
if [ "$_n" -lt 40 ] || ! grep -q '^ladder_red_run() {' "$TMP/rows.sh"; then
    echo "EXTRACTION FAILED — got $_n lines and no ladder_red_run. Refusing to"
    echo "run: a red test against a truncated copy of the code proves nothing."
    exit 2
fi
echo "extracted $_n lines of ladder_red_verdict + ladder_red_run from scripts/gate.sh"
# shellcheck disable=SC1090
. "$TMP/rows.sh"

PASS=0
FAIL=0
WORDS=""

fire() {   # <name> <expect-verdict> <expect-leading-word> <repo_root>
    _name="$1"; _wantv="$2"; _wantw="$3"; repo_root="$4"
    printf '\n%s\n' "======================================================================"
    printf 'CASE %s — expect %s / leading word %s\n' "$_name" "$_wantv" "$_wantw"
    printf '%s\n' "======================================================================"
    _t=$(ladder_red_run "$TMP/$_name.log")
    printf '  tuple : %s\n' "$_t"
    _v=$(printf '%s\n' "$_t" | cut -d'|' -f1)
    _d=$(printf '%s\n' "$_t" | cut -d'|' -f6)
    _w=$(printf '%s\n' "$_d" | cut -d' ' -f1)
    printf '  VERBATIM: %s\n' "$_d"
    if [ "$_v" = "$_wantv" ] && [ "$_w" = "$_wantw" ]; then
        printf '  => RED as expected (%s / %s)\n' "$_v" "$_w"
        PASS=$((PASS + 1))
    else
        printf '  => *** CASE FAILED *** wanted %s/%s, got %s/%s\n' \
            "$_wantv" "$_wantw" "$_v" "${_w:-none}"
        FAIL=$((FAIL + 1))
    fi
    WORDS="$WORDS
$_w"
}

# ---- scratch-tree builders ----------------------------------------------------
fake_red() {   # <dir> [<extra-line-to-run>]
    mkdir -p "$1/work/w-ladders"
    {
        echo '#!/usr/bin/env python3'
        echo 'import sys, os'
        echo 'ARMS = ["W1", "W2", "W3", "G1", "G2"]'
        echo 'if "--list" in sys.argv:'
        echo '    print("\n".join(ARMS)); raise SystemExit(0)'
        [ $# -ge 2 ] && echo "$2"
        echo 'print("ALL 5 ARMS PASS — 3 red, 2 green")'
    } > "$1/work/w-ladders/ladder_red.py"
}

fake_subject() {   # <dir>
    mkdir -p "$1/work/w-front3"
    echo "# a stand-in for ladder.py" > "$1/work/w-front3/ladder.py"
}

fake_tree() {   # <dir>
    mkdir -p "$1/crates/c2-il/src/func/body"
    echo "fn chain_skip_form(b: u8) {}" > "$1/crates/c2-il/src/func/body/expr.rs"
}

# 1. LADDER-MISSING — nothing to run.
D="$TMP/t-missing"; mkdir -p "$D"; fake_tree "$D"; fake_subject "$D"
fire missing NO-RESULT LADDER-MISSING "$D"

# 2. LADDER-NOSUBJECT — the arms are there, the instrument they fire is not.
#    Held apart from ARMS-FAILED on purpose: without this word, every arm fails
#    on an import error and the row says the guards broke.
D="$TMP/t-nosubject"; mkdir -p "$D"; fake_tree "$D"; fake_red "$D"
fire nosubject NO-RESULT LADDER-NOSUBJECT "$D"

# 3. LADDER-NOGIT — both files present, no checkout, so no interlock.
D="$TMP/t-nogit"; mkdir -p "$D"; fake_tree "$D"; fake_subject "$D"; fake_red "$D"
fire nogit REFUSED LADDER-NOGIT "$D"

# 4. LADDER-DIRTY — a REAL checkout whose width table differs from HEAD. Trap A:
#    everything the earlier gates read is held fixed and only this moves.
D="$TMP/t-dirty"; mkdir -p "$D"; fake_tree "$D"; fake_subject "$D"; fake_red "$D"
git -C "$D" init -q
git -C "$D" -c user.email=r@r -c user.name=r add -A >/dev/null 2>&1
git -C "$D" -c user.email=r@r -c user.name=r commit -qm base >/dev/null 2>&1
echo "// a peer lane, mid-wave" >> "$D/crates/c2-il/src/func/body/expr.rs"
fire dirty REFUSED LADDER-DIRTY "$D"

# 5. LADDER-RESIDUE — a clean checkout whose arms DO write into crates/. Not a
#    defect reproduction: `ladder_red.py` has no such path. This is the
#    postcondition against a future edit and against a SIGKILL, fired the only
#    way it can be, and the report says which.
D="$TMP/t-residue"; mkdir -p "$D"; fake_tree "$D"; fake_subject "$D"
fake_red "$D" 'open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "crates/c2-il/src/func/body/expr.rs"), "a").write("// written by the arms\n")'
git -C "$D" init -q
git -C "$D" -c user.email=r@r -c user.name=r add -A >/dev/null 2>&1
git -C "$D" -c user.email=r@r -c user.name=r commit -qm base >/dev/null 2>&1
fire residue FAIL LADDER-RESIDUE "$D"

# 6. THE GREEN CONTROL. Same builders, nothing perturbed: a clean checkout with
#    both files and arms that behave must come back PASS with an EMPTY detail. A
#    red test whose control cannot come back green is testing the builders.
D="$TMP/t-green"; mkdir -p "$D"; fake_tree "$D"; fake_subject "$D"; fake_red "$D"
git -C "$D" init -q
git -C "$D" -c user.email=r@r -c user.name=r add -A >/dev/null 2>&1
git -C "$D" -c user.email=r@r -c user.name=r commit -qm base >/dev/null 2>&1
repo_root="$D"
_t=$(ladder_red_run "$TMP/green.log")
printf '\n%s\n' "======================================================================"
printf 'CASE green — expect PASS and an EMPTY detail\n'
printf '%s\n' "======================================================================"
printf '  tuple : %s\n' "$_t"
if [ "$(printf '%s\n' "$_t" | cut -d'|' -f1)" = PASS ] \
   && [ -z "$(printf '%s\n' "$_t" | cut -d'|' -f6)" ] \
   && [ "$(printf '%s\n' "$_t" | cut -d'|' -f3)" = 5 ]; then
    echo "  => GREEN as required (5 of 5 declared arms, no detail)"
    PASS=$((PASS + 1))
else
    echo "  => *** CONTROL FAILED ***"
    FAIL=$((FAIL + 1))
fi

# ---- report -------------------------------------------------------------------
_nw=$(printf '%s\n' "$WORDS" | grep -c .)
_uw=$(printf '%s\n' "$WORDS" | grep . | sort -u | grep -c .)
echo
echo "======================================================================"
echo "distinct leading words fired: $_uw of $_nw refusals — $(printf '%s\n' "$WORDS" | grep . | sort -u | tr '\n' ' ')"
if [ "$_nw" -ne "$_uw" ]; then
    echo "*** TWO REFUSALS SHARE A WORD — trap B, and every case above that"
    echo "*** asserted on the shared one proved nothing."
    FAIL=$((FAIL + 1))
fi
echo "cases: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || { echo "FAILED"; exit 1; }
echo "ALL $PASS CASES PASS — 5 runner refusals fired, 1 green control"

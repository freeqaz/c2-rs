#!/bin/sh
# w-gate — demonstrate that each assertion added to `gate.sh --selftest` fires,
# INDIVIDUALLY, and that the control stays green.
#
# A test that goes red everywhere identifies nothing (a recorded lesson here), and
# a recorded failure on this project is a count floor tripping FIRST so that the
# assertions behind it never executed. So each mutation below is aimed at exactly
# one property, and this script records which assertion names went red — not just
# that the selftest failed.
#
# Every mutation runs against ITS OWN fake repo root (a directory of symlinks to
# the real tree, with one mutated `scripts/gate.sh`), because a lane on this box
# once had four parallel probes share one temp dir and fabricated a finding.
#
# Usage:  sh work/w-gate/mutate.sh            # runs every mutation
set -eu

repo="$(cd "$(dirname "$0")/../.." && pwd)"
out="$repo/work/w-gate/mutations"
rm -rf "$out"; mkdir -p "$out"

# The one case that must stay GREEN under every mutation aimed at the demand.
CONTROL='demand-off-all-skip-still-exits-0'
# ...and the demand case itself, tracked beside it so a mutation aimed at the
# DEFAULT path (M3) is visibly distinguishable from one aimed at the demand.
DEMAND='require-graded-all-skip-fails'

# Build a fake repo root whose only real file is a mutated gate.sh.
fake_root() {  # <name>
    _fr="$out/$1/root"
    rm -rf "$out/$1"; mkdir -p "$_fr/scripts"
    for _e in "$repo"/* "$repo"/.git; do
        _b=$(basename "$_e")
        [ "$_b" = scripts ] && continue
        [ -e "$_e" ] || continue
        ln -s "$_e" "$_fr/$_b"
    done
    for _e in "$repo"/scripts/*; do
        _b=$(basename "$_e")
        [ "$_b" = gate.sh ] && continue
        ln -s "$_e" "$_fr/scripts/$_b"
    done
    cp "$repo/scripts/gate.sh" "$_fr/scripts/gate.sh"
    chmod +x "$_fr/scripts/gate.sh"
    # `toolchain_hint` looks for `<repo_root>/../wibo/build/release/wibo`, which
    # from a fake root two directories deep resolves to nothing. Without this the
    # fake tree always has one MISSING default and the "every default present"
    # arm is never reached — the fixture would silently stop exercising a case
    # rather than exercising it and failing, which is this project's own defect.
    if [ -e "$repo/../wibo" ]; then
        ln -s "$(cd "$repo/.." && pwd)/wibo" "$out/$1/wibo"
    fi
    printf '%s\n' "$_fr"
}

# <name> <description> <sed-expr>...
mutate() {
    _name="$1"; _desc="$2"; shift 2
    _root=$(fake_root "$_name")
    for _s in "$@"; do
        sed -i "$_s" "$_root/scripts/gate.sh"
    done
    if cmp -s "$repo/scripts/gate.sh" "$_root/scripts/gate.sh"; then
        echo "### $_name — MUTATION DID NOT APPLY (sed matched nothing). ABORT."
        exit 1
    fi
    _rc=0
    sh "$_root/scripts/gate.sh" --selftest --work "$out/$_name/work" \
        > "$out/$_name/selftest.txt" 2>&1 || _rc=$?
    echo "### $_name — $_desc"
    echo "    selftest exit $_rc"
    if [ "$_rc" -eq 0 ]; then
        echo "    *** NOTHING WENT RED. The assertion does not cover this mutation. ***"
    fi
    grep '^  FAIL' "$out/$_name/selftest.txt" | sed 's/^  FAIL  */    RED: /' || true
    for _sent in "$CONTROL" "$DEMAND"; do
        if grep -q "^  ok    $_sent" "$out/$_name/selftest.txt"; then
            echo "    still green: $_sent"
        else
            echo "    ALSO RED:    $_sent"
        fi
    done
    grep -m1 -E '^gate.sh --selftest: (PASS|FAIL)' "$out/$_name/selftest.txt" \
        | sed 's/^/    /' || true
    echo
}

echo "w-gate mutation demonstration — one property per mutation"
echo "baseline: $(sh "$repo/scripts/gate.sh" --selftest --work "$out/baseline-work" 2>&1 | tail -1)"
echo

mutate M1-demand-never-fires \
    "the demand block is unreachable (require_graded can never equal 9)" \
    's/if \[ "${require_graded:-0}" -eq 1 \]; then/if [ "${require_graded:-0}" -eq 9 ]; then/'

mutate M2-demand-inverted \
    "the count test is inverted: fires when something WAS graded" \
    's/if \[ "\$_d_units" -eq 0 \]; then/if [ "$_d_units" -ne 0 ]; then/'

mutate M3-skipped-returns-1 \
    "the DEFAULT all-skip path is made to exit 1 (breaks the control, not the demand)" \
    's/^        toolchain_hint$/        toolchain_hint; return 1/'

mutate M4-banner-drops-the-count \
    "the banner reports a status instead of the count it compared" \
    's/units graded, summed over this whole gate:  \$_d_units/nothing was graded/'

mutate M5-no-resolution-hint \
    "the demand banner no longer prints WHY the toolchain did not resolve" \
    's/^                toolchain_hint$/                :/'

mutate M6-hint-names-c2rs-dc3 \
    "the hint sends the lane to set C2RS_DC3, which cannot fix this" \
    's|^    echo "  If compilers/ is missing|    echo "  Or try C2RS_DC3. If compilers/ is missing|'

mutate M7-hint-hardcodes-the-version-dir \
    "the hint stops reading X360_TOOLCHAIN_REL from the Rust source" \
    's|^        _th_dir="\$_th_croot/\$_th_ver"$|        _th_dir="$_th_croot/X360/9.99.99999.99"|'

mutate M8-reap-only-not-refused \
    "--require-graded --reap-only is allowed through instead of refused" \
    's/^        exit 2 ;;$/        require_graded=0 ;;/'

# M9 is DELIBERATELY not a specific mutation, and the point is what it kills.
# Without the guard, `$(( 0 + + ))` is an expansion error and bash EXITS a
# non-interactive shell on one — the selftest dies mid-run with no verdict line at
# all. An absent count without num() is not a demand that fails; it is a gate that
# stops having opinions, which is strictly worse.
mutate M9-num-guard-removed \
    "num() returns the raw field: an absent count kills the run instead of failing it" \
    's/^        ..|\*\[!0-9\]\*) echo 0 ;;$/        XXNOMATCHXX) echo 0 ;;/'

mutate M10-demand-fires-on-every-run \
    "the demand fires regardless of the count (a check that is always on)" \
    's/if \[ "\$_d_units" -eq 0 \]; then/if [ "$_d_units" -lt 999999999 ]; then/'

mutate M11-hint-drops-the-override-names \
    "the hint stops attaching an override to each path it checked" \
    's/^        printf .       %-10s %s\\n. "" "\$_th_src"$/        printf "       %-10s %s\\n" "" "see the docs"/'

mutate M12-hint-drops-the-fix-command \
    "the hint names the symptom and not the one command that fixes it" \
    's/^    echo "  FIX, from the main repo:.*$/    echo "  FIX: work it out yourself."/'

mutate M13-hint-invents-an-override \
    "the hint names a variable the resolver does not read (the drift itself)" \
    's/"C2RS_CL_EXE"$/"C2RS_CLEXE"/'

mutate M14-hint-never-says-missing \
    "the found-or-MISSING column only ever prints one value" \
    's/^            _th_mark="MISSING"$/            _th_mark="found  "/'

mutate M15-hint-always-claims-they-exist \
    "the everything-is-present note is printed even when nothing is present" \
    's/^    if \[ "\$_th_gone" -eq 0 \]; then$/    if [ "$_th_gone" -ge 0 ]; then/'

mutate M16-hint-never-declines-to-blame \
    "the note is dropped, so four found lines print under 'did not resolve'" \
    's/^    if \[ "\$_th_gone" -eq 0 \]; then$/    if [ "$_th_gone" -eq 99 ]; then/'

mutate M17-reap-refusal-writes-first \
    "the --reap-only refusal creates the run tree before refusing" \
    's/^        exit 2 ;;$/        mkdir -p "$work"; exit 2 ;;/'

mutate M18-no-note-on-inspection-modes \
    "the demand is silently ignored by --list instead of saying so" \
    's/^        echo "gate.sh: note — --require-graded has no effect.*$/        : ;/'

mutate M19-demand-breaks-list \
    "an exported demand makes --list exit 2" \
    's/^        require_graded=0 ;;$/        exit 2 ;;/'

mutate M20-plain-pass-returns-1 \
    "the unqualified PASS path is broken (both green-run cases, demand or not)" \
    's/^    echo "GATE: PASS — \$_d_pass\/\$_d_n lanes ran and every one of them graded a corpus,"$/    echo "GATE: PASS — broken"; return 1/'

# THE ONE MUTATION THAT DELIBERATELY TOUCHES TWO THINGS, because it emulates a
# REFACTOR rather than a typo: the demand check stops returning, so control falls
# through to the outcome line below it, and that line is made to say PASS. This is
# exactly the relocation `saw_no 'GATE: PASS'` exists to forbid — a nothing-graded
# run under the demand must not contain a string anything can read as green. The
# control goes red too, because the headline it asserts on is the one being
# rewritten; that is a property of the emulation, not of the assertion.
mutate M21-check-moved-below-the-outcome \
    "the demand no longer returns, and the outcome below it calls itself a PASS" \
    '/GATE: FAIL (NOTHING GRADED)/,/^        fi$/ s/^            return 1$/            : ;/' \
    's/GATE: SKIPPED — all \$_d_n lanes/GATE: PASS — all $_d_n lanes/'

# The ORDERING property, emulated: the demand is put AHEAD of the lane-FAIL check
# and fires unconditionally, so a run carrying a mismatch is relabelled as a
# nothing-graded run and loses its alarm. Deliberately not specific — hoisting the
# demand past a failure path reddens every case that depends on that path, and the
# names say which. The two assertions this exists for are
# `also: a mismatch under the demand still raises the mismatch alarm` and
# `also: and is never relabelled as a nothing-graded run`.
mutate M22-demand-hoisted-past-the-mismatch \
    "the demand fires first and unconditionally, so a mismatch is relabelled" \
    's/if \[ "\$_d_units" -eq 0 \]; then/if [ "$_d_units" -lt 999999999 ]; then/' \
    's/^    if \[ "\$_d_fail" -gt 0 \]; then$/    if [ "$_d_fail" -gt 999999 ]; then/'

echo "done — per-mutation logs under $out/<name>/selftest.txt"

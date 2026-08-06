#!/bin/sh
# mutate.sh — the three registered mutations of `work/w-inl0/ADDENDUM-1.md` §3.
#
# Each removes ONE guard, VERIFIES the edit actually landed (`git diff --stat`
# non-empty on the file it names — board #951: a mutation that did not mutate is
# a green run that means nothing), runs the test that must go RED, and restores
# the tree.
#
# Usage: work/w-inl0/mutate.sh {M1|M2|M3}
set -eu
: "${C2RS_WIBO:=/home/free/code/milohax/wibo/build/wibo}"
: "${C2RS_COMPILERS:=/home/free/code/milohax/c2-rs/compilers}"
export C2RS_WIBO C2RS_COMPILERS

NOEFF=crates/c2-il/src/func/body/shapes/no_effect.rs
ELIDE=crates/c2-core/src/elide.rs

restore() { git checkout -- "$NOEFF" "$ELIDE"; }
trap restore EXIT

case "${1:-}" in
M1)
    file="$NOEFF"
    python3 - <<'PY'
p = 'crates/c2-il/src/func/body/shapes/no_effect.rs'
s = open(p).read()
old = """    let again = eat_temp_addr(seg, &mut q)?;
    if again != dest {
        return None;
    }"""
new = """    let again = eat_temp_addr(seg, &mut q)?;
    let _ = (again, dest); // M1: the SAME-TEMPORARY guard, removed"""
assert old in s, 'M1 did not find its guard'
open(p, 'w').write(s.replace(old, new))
PY
    test -n "$(git diff --stat -- "$file")" || { echo "M1 DID NOT MUTATE $file"; exit 1; }
    echo "-- M1 mutated $file:"; git diff --stat -- "$file"
    cargo test -p c2-il --lib a_different_temporary_in_the_argument_is_refused 2>&1 | tail -5
    ;;
M2)
    file="$ELIDE"
    python3 - <<'PY'
p = 'crates/c2-core/src/elide.rs'
s = open(p).read()
old = "                    Reduction::NoEffectCall(callee) => (false, Some(callee)),"
new = "                    Reduction::NoEffectCall(callee) => (true, Some(callee)), // M2"
assert old in s, 'M2 did not find its guard'
open(p, 'w').write(s.replace(old, new))
PY
    test -n "$(git diff --stat -- "$file")" || { echo "M2 DID NOT MUTATE $file"; exit 1; }
    echo "-- M2 mutated $file:"; git diff --stat -- "$file"
    cargo build --release -p c2-harness 2>&1 | grep -E '^error' && exit 1
    work/w-inl0/scan.sh work/w-inl0/mut_M2_scan
    grep -E "fnbyte-(differs|exact|noeffect-ref-other|noeffect-ref-blr|noeffect-admitted) " work/w-inl0/mut_M2_scan.txt
    cargo test --release -p c2-harness --test dead_temp_elision 2>&1 | grep -E "^test |test result"
    ;;
M3)
    file="$NOEFF"
    python3 - <<'PY'
p = 'crates/c2-il/src/func/body/shapes/no_effect.rs'
s = open(p).read()
old = "    eat_return_plumbing(seg, &mut p, false, depth).ok()?;"
new = "    let _ = eat_return_plumbing(seg, &mut p, false, depth); // M3: TOTALITY removed"
assert old in s, 'M3 did not find its guard'
open(p, 'w').write(s.replace(old, new))
PY
    test -n "$(git diff --stat -- "$file")" || { echo "M3 DID NOT MUTATE $file"; exit 1; }
    echo "-- M3 mutated $file:"; git diff --stat -- "$file"
    cargo test -p c2-il --lib trailing_bytes_after_the_function_tail_are_refused 2>&1 | tail -5
    ;;
*)
    echo "usage: $0 {M1|M2|M3}"; exit 2 ;;
esac

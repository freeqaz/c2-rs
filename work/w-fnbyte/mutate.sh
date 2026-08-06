#!/bin/sh
# mutate.sh — MUST-FAIL mutations for lane w-fnbyte's reconstruction test.
#
# Each mutation breaks one property the test claims to hold, and the test must
# go RED. A mutation that leaves the test green means the test does not check
# what its name says. Restores with `git checkout` — every file it touches is
# COMMITTED first (w-seam's #874 destroyed its own uncommitted tests this way).
#
# Usage: mutate.sh <1|2|3|all>
set -eu
cd "$(dirname "$0")/../.."

FN=crates/c2-harness/src/gap/fnbytes.rs
CD=crates/c2-core/src/comdat.rs

restore() { git checkout -- "$FN" "$CD"; }
trap restore EXIT

run() {
    label="$1"
    if cargo test --release -p c2-harness --test fnbyte_gy >/tmp/c2rs-mut.$$ 2>&1; then
        echo "MUTATION $label: GREEN  <-- FAILURE, the test does not check this"
        rc=1
    else
        echo "MUTATION $label: RED    ($(grep -c '^' /tmp/c2rs-mut.$$) lines) — $(grep -m1 -o "panicked at[^\"]*" /tmp/c2rs-mut.$$ || echo 'assert')"
        grep -m1 -A2 "^thread .* panicked" /tmp/c2rs-mut.$$ | sed 's/^/    /' || true
        rc=0
    fi
    rm -f /tmp/c2rs-mut.$$
    return $rc
}

m1() {
    # M1 — put `tail` back in the blind spot: refuse to reconstruct it.
    python3 - "$FN" <<'PY'
import sys
p=sys.argv[1]; s=open(p,encoding="utf-8").read()
old="""    let selected = select_function(func, mode).map_err(|_| ("refused", Decline::Selector))?;
    let shape = selected_tag(&selected);"""
new="""    let selected = select_function(func, mode).map_err(|_| ("refused", Decline::Selector))?;
    let shape = selected_tag(&selected);
    if shape == "tail" { return Err((shape, Decline::GyShape)); }  // MUTATION 1"""
assert old in s, "M1 anchor missing"
open(p,"w",encoding="utf-8").write(s.replace(old,new,1))
PY
    run "1 (tail back to Partial — the blind spot restored)"
}

m2() {
    # M2 — drop the reconstructed tail branch word. The body is then a prefix of
    # c2's and the specimen is no longer byte-exact.
    python3 - "$CD" <<'PY'
import sys
p=sys.argv[1]; s=open(p,encoding="utf-8").read()
old="""            t.extend_from_slice(&codegen::encode_tail_branch(branch_off));"""
new="""            let _ = codegen::encode_tail_branch(branch_off);  // MUTATION 2: branch dropped"""
assert old in s, "M2 anchor missing"
open(p,"w",encoding="utf-8").write(s.replace(old,new,1))
PY
    run "2 (the reconstructed tail branch word is not appended)"
}

m3() {
    # M3 — compare only the first half of the words. Every clean corpus still
    # reads `differs 0`; only a per-word mutation can see it.
    python3 - "$FN" <<'PY'
import sys
p=sys.argv[1]; s=open(p,encoding="utf-8").read()
old="""    if port == reference {
        return FnByte::Exact;
    }"""
new="""    let half = (reference.len() / 8) * 4;  // MUTATION 3: compare a PREFIX only
    if port.len() == reference.len() && port[..half] == reference[..half] {
        return FnByte::Exact;
    }"""
assert old in s, "M3 anchor missing"
open(p,"w",encoding="utf-8").write(s.replace(old,new,1))
PY
    run "3 (only the first half of each body is compared)"
}

case "${1:-all}" in
    1) m1 ;;
    2) restore; m2 ;;
    3) restore; m3 ;;
    all)
        fails=0
        m1 || fails=$((fails+1)); restore
        m2 || fails=$((fails+1)); restore
        m3 || fails=$((fails+1)); restore
        echo "mutations run: 3, green (BAD): $fails"
        [ "$fails" -eq 0 ]
        ;;
esac

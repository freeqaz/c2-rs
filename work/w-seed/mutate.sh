#!/bin/sh
# mutate.sh — the three REGISTERED must-fail edits (PREREG §1.2 P9).
#
# Board **#951**: a mutation that did not change its file is a mutation that
# proved nothing, and a mutation run that hangs is not a mutation run that
# passed. So every edit below is verified with `git diff --stat` to be non-empty
# BEFORE its test run is read, and every run is under `timeout` so a hang reports
# as its own outcome and not as a failure or a pass.
#
# The three:
#   M1  delete the TOTALITY TERMINAL -- `eat_return_plumbing`'s fail-closed end.
#       Without it "there is nothing else in this body" is a search over the part
#       that was walked, and a SEED asserts the whole.
#   M2  open the CLOSED VOCABULARY -- accept any statement between the scope and
#       the return plumbing. This is the edit that makes a body with a CALL in it
#       seed, which is precisely what `Reduction`'s step (2) forbids and what the
#       cycle re-derivation rests on.
#   M3a give `NoEffectNothing` a LINK as well as a seed.
#       **REGISTERED AS MUST-FAIL AND IT CAME BACK GREEN**, and that is a finding
#       rather than a hole: the fixpoint skips a name that is already in `in_r`
#       (`if in_r[i] { continue; }`), so a SEEDED name is never asked for its link
#       at all and the arm is INERT as the loop is written. Kept, and reported
#       green, because a mutation quietly rewritten until it goes red proves
#       nothing. What it establishes is where the cycle argument's step (2)
#       actually lives: in the READER's vocabulary (M2), not in this arm.
#   M3b make `NoEffectCall` seed as well as link -- the edit that erases the
#       link/seed distinction outright. This is the one the cycle argument needs
#       and the one w-inl0's own M2 measured from the other side.
#
# Usage: work/w-seed/mutate.sh          (run from the worktree root)
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$WT"
. "$WT/work/w-seed/env.sh"

NE=crates/c2-il/src/func/body/shapes/no_effect.rs
EL=crates/c2-core/src/elide.rs

restore() { git checkout -- "$NE" "$EL"; }
trap restore EXIT

run() {
    # $1 = tag, $2..$ = cargo test filter args
    tag=$1
    shift
    changed=$(git diff --stat -- "$NE" "$EL")
    if [ -z "$changed" ]; then
        echo "$tag: FILE UNCHANGED — the mutation did not apply. NOT A RESULT."
        return
    fi
    echo "$tag: applied ($(echo "$changed" | tail -1))"
    if timeout 900 cargo test --release "$@" >"work/w-seed/mut_$tag.txt" 2>&1; then
        echo "$tag: GREEN — THE GUARD IS NOT TESTED. This is a failure of the grid."
    else
        rc=$?
        if [ "$rc" -eq 124 ]; then
            echo "$tag: TIMEOUT — a hang is its own outcome, not a red test."
        else
            echo "$tag: RED (exit $rc) — $(grep -c '^test .* FAILED' "work/w-seed/mut_$tag.txt" || true) test(s) failed"
            grep -E '^    [a-z_]+$' "work/w-seed/mut_$tag.txt" | sort -u | sed 's/^/      /'
        fi
    fi
    restore
}

echo "=== M1 — delete the totality terminal"
python3 - <<'PY'
p = "crates/c2-il/src/func/body/shapes/no_effect.rs"
s = open(p).read()
old = """    // THE FAIL-CLOSED TERMINAL. This must reach the end of the segment, and it is
    // what makes the walk total and the seed honest.
    eat_return_plumbing(seg, &mut p, false, depth).ok()?;
    Some(())
}

/// The one statement: two literal operands, the bind, the discard."""
new = """    let _ = depth;
    Some(())
}

/// The one statement: two literal operands, the bind, the discard."""
assert old in s, "M1 anchor not found"
open(p, "w").write(s.replace(old, new))
PY
run M1 -p c2-il no_effect

echo "=== M2 — open the closed vocabulary"
python3 - <<'PY'
p = "crates/c2-il/src/func/body/shapes/no_effect.rs"
s = open(p).read()
old = """    eat_opt_stmt_marker(seg, &mut p);
    eat_nothing_stmt(seg, &mut p)?;"""
new = """    eat_opt_stmt_marker(seg, &mut p);
    // MUTATION M2: skip to the return plumbing instead of requiring the one
    // statement -- i.e. accept ANY body, calls included.
    while p < seg.len() && seg[p] != 0x3A {
        p += 1;
    }"""
assert old in s, "M2 anchor not found"
open(p, "w").write(s.replace(old, new))
PY
run M2 -p c2-il no_effect

echo "=== M3a — give the seed a link (registered must-fail; reported as measured)"
python3 - <<'PY'
p = "crates/c2-core/src/elide.rs"
s = open(p).read()
old = "Reduction::NoEffectNothing => (true, None),"
new = 'Reduction::NoEffectNothing => (true, Some("?any@@YAXXZ")),'
assert old in s, "M3a anchor not found"
open(p, "w").write(s.replace(old, new))
PY
run M3a -p c2-core elide

echo "=== M3b — erase the link/seed distinction: make NoEffectCall seed too"
python3 - <<'PY'
p = "crates/c2-core/src/elide.rs"
s = open(p).read()
old = "Reduction::NoEffectCall(callee) => (false, Some(callee)),"
new = "Reduction::NoEffectCall(callee) => (true, Some(callee)),"
assert old in s, "M3b anchor not found"
open(p, "w").write(s.replace(old, new))
PY
run M3b -p c2-core elide

echo "=== done"

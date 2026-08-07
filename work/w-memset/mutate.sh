#!/bin/sh
# mutate.sh — the three registered must-fail edits (`PREREG.md` P8).
#
# Each is verified to have CHANGED the file it names before its run is read
# (board #951: a mutation that does not mutate reads exactly like a mutation
# nothing catches), and each `cargo` invocation runs under `timeout` so a hang is
# reported as its own outcome and cannot pass for a red.
#
# Restores with `git checkout` — the tests were committed first.
#
# Usage: work/w-memset/mutate.sh   (from the worktree root)
set -u
F=crates/c2-il/src/func/body/shapes/no_effect.rs

restore() { git checkout -- "$F"; }
trap restore EXIT

run() {
    n="$1"
    changed=$(git diff --stat -- "$F" | wc -l)
    if [ "$changed" -eq 0 ]; then
        echo "M$n: THE MUTATION DID NOT CHANGE THE FILE — its result means nothing"
        return
    fi
    echo "--- M$n: $F changed, running"
    timeout 900 cargo test --release -p c2-il no_effect 2>&1 |
        grep -E '^test .*(ok|FAILED)$|^test result' | sed 's/^/    /'
    rc=$?
    [ "$rc" -eq 124 ] && echo "M$n: TIMEOUT — reported as its own outcome, NOT as red"
    restore
}

echo "=== M1 — drop the LABEL matching (the head goto, the continue, the exit branch)"
python3 - "$F" <<'PY'
import sys, re
p = sys.argv[1]; s = open(p).read()
s = s.replace("""    if eat_label(seg, &mut p, 0x29)? != l_cond {
        return None;
    }""", """    eat_label(seg, &mut p, 0x29)?;""")
s = s.replace("""    if eat_label(seg, &mut p, 0x3A)? != l_incr {
        return None;
    }""", """    eat_label(seg, &mut p, 0x3A)?;""")
s = s.replace("""    if eat_label(seg, &mut p, 0x29)? != l_exit {
        return None;
    }""", """    eat_label(seg, &mut p, 0x29)?;""")
s = s.replace("""    if l_cond == l_incr || l_cond == l_exit || l_incr == l_exit {
        return None;
    }""", "")
open(p, "w").write(s)
PY
run 1

echo "=== M2 — drop the INDUCTION STEP's formals test (the purity guard)"
python3 - "$F" <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
old = """    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    if !formals.contains(&tok) {
        return None;
    }
    // The stride, as a literal."""
new = """    let (_tok, w) = read_token_var(seg, q)?;
    q += w;
    // The stride, as a literal."""
assert old in s
s = s.replace(old, new)
open(p, "w").write(s)
PY
run 2

echo "=== M3 — drop the TOTALITY terminal (the fail-closed end of segment)"
python3 - "$F" <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
old = """    eat_return_plumbing(seg, &mut p, false, depth).ok()?;
    Some(callee_tok)
}

/// `<op> <token-var>` — a branch or a label, returning the token it names."""
new = """    let _ = eat_return_plumbing(seg, &mut p, false, depth);
    Some(callee_tok)
}

/// `<op> <token-var>` — a branch or a label, returning the token it names."""
assert old in s
s = s.replace(old, new)
open(p, "w").write(s)
PY
run 3

echo "=== restored"
git diff --stat -- "$F"

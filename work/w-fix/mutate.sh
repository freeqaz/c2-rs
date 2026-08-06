#!/bin/sh
# mutate.sh — the MUST-FAIL mutations for the FIXPOINT form of mechanism E.
#
# Lane w-fix measurement tooling. Restores `crates/` with `git checkout`, so
# **the tests it runs were committed first** (board #874, lane w-seam).
#
# `work/w-fix/PREREG.md` P9 registers three mutations, each removing one thing
# the fixpoint depends on. Each must go RED — in the toolchain-backed cell
# tests, in the `c2-core` unit tests, or in the 878-TU FBM partition — and each
# must go red with a DISTINCT message naming what was removed. A mutation that
# stays green is a property nothing is testing.
#
# | # | mutation | what it removes |
# |---|---|---|
# | 1 | the step sets `changed` whether or not the name was already admitted | **THE RECURSION GUARD.** The iteration is no longer monotone, so it never quiesces; the round ceiling is what makes it terminate instead of hanging, and `the_round_ceiling_cannot_fire` is what notices |
# | 2 | the step ignores the callee's membership and admits every elidable tail call | **THE FIXPOINT IS APPLIED THROUGH A NON-EMPTY LINK** — `k5`/`k6`/`k7`'s stop and `k12`'s mechanism-I chain both become elisions |
# | 3 | a name in a call cycle is seeded | **A CYCLE IS TREATED AS REDUCING TO NOTHING** — c2 emits a branch for every member of `k10`/`k11` |
#
# Mutation 1 must be watched for a HANG as well as for a failure: the whole
# point of the ceiling is that it cannot hang, so each cargo invocation is run
# under `timeout` and a timeout is reported as its own outcome, never as RED.
#
# Usage:  mutate.sh [--scan]     (--scan also runs the 878-TU FBM partition)
set -eu

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
E=crates/c2-core/src/elide.rs
SCAN=0
[ "${1:-}" = "--scan" ] && SCAN=1
# A generous ceiling: the whole suite is seconds. Anything near this is a hang.
TMO=300

restore() { git checkout -- "$E"; }
trap restore EXIT

green=0
hung=0
total=0

run_one() {
    n="$1"; what="$2"
    total=$((total + 1))
    echo "=============================================================="
    echo "MUTATION $n — $what"
    echo "=============================================================="
    if ! timeout "$TMO" cargo build --release --workspace --all-targets >/dev/null 2>&1; then
        echo "  RED (build): the mutation does not compile"
        restore
        return
    fi
    red=0
    log="${TMPDIR:-/tmp}/w-fix-mut$n.log"
    : > "$log"
    for t in "cargo test --release -p c2-core --lib elide" \
             "cargo test --release -p c2-harness --test empty_elision"; do
        rc=0
        # shellcheck disable=SC2086
        timeout "$TMO" $t >> "$log" 2>&1 || rc=$?
        if [ "$rc" = "124" ]; then
            echo "  HUNG: \`$t\` did not finish in ${TMO}s — a mutation that HANGS is"
            echo "        not a mutation that went red. The round ceiling failed."
            hung=$((hung + 1))
            red=1
        elif [ "$rc" != "0" ]; then
            red=1
        fi
    done
    echo "  failing tests: $(grep -c '^test .* FAILED' "$log" || true)"
    grep -h "THE RECURSION GUARD\|THE FIXPOINT WAS APPLIED\|A CYCLE WAS TREATED\|THE FIXPOINT DID NOT\|assertion .* failed" "$log" \
        | sed 's/^/    /' | head -6 || true
    if [ "$SCAN" = "1" ]; then
        timeout "$TMO" cargo build --release -p c2-harness >/dev/null 2>&1
        timeout 900 ./target/release/c2rs gap --list work/dc3-workload/files.txt \
            --flags-file work/dc3-workload/flags.txt \
            --cwd "${C2RS_DC3:?set C2RS_DC3 to the dc3 tree}" --jobs 16 2>/dev/null \
            | grep -E "gap-metric (fnbyte-exact|fnbyte-differs|fnbyte-elided|fnbyte-partition-broken) " \
            | sed 's/^ */    /' || echo "    (scan did not complete)"
    fi
    if [ "$red" = "1" ]; then
        echo "  RESULT: RED"
    else
        echo "  RESULT: GREEN  <-- BAD, nothing tests this"
        green=$((green + 1))
    fi
    restore
}

# ---- 1. remove the recursion guard -----------------------------------------
# THE GUARD IS `if in_r[i] { continue; }`, and it is the whole monotonicity
# argument: without it an already-admitted name is re-admitted every round, so
# `changed` is set forever and the iteration never quiesces. It must TERMINATE
# anyway — via the round ceiling — and go red there.
#
# A FIRST ATTEMPT AT THIS MUTATION CAME BACK GREEN and is recorded rather than
# quietly replaced: setting `changed` from `in_r[j]` instead of from the
# transition looks like the same edit and is not, because the skip above it
# means a node with an admitted callee is never revisited. A mutation that
# leaves the guard in place tests nothing, which is exactly what it reported.
python3 - "$E" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """                if in_r[i] {
                    continue;
                }
                let Some(callee) = link[i] else { continue };"""
new = """                let Some(callee) = link[i] else { continue };"""
assert old in s, "mutation 1 anchor moved"
open(p, "w").write(s.replace(old, new))
PY
run_one 1 "the already-admitted skip is deleted (removes THE RECURSION GUARD)"

# ---- 2. apply the fixpoint through a non-empty link ------------------------
python3 - "$E" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """                if let Ok(j) = names.binary_search_by(|n| (*n).cmp(callee)) {
                    if in_r[j] {
                        in_r[i] = true;
                        changed = true;
                    }
                }"""
new = """                let _ = callee;
                if !in_r[i] {
                    in_r[i] = true;
                    changed = true;
                }"""
assert old in s, "mutation 2 anchor moved"
open(p, "w").write(s.replace(old, new))
PY
run_one 2 "every elidable tail call is admitted regardless of its callee (FIXPOINT THROUGH A NON-EMPTY LINK)"

# ---- 3. seed a cycle -------------------------------------------------------
# A name that steps to a name that steps back to it is seeded, so the cycle
# enters the set — the one shape the least fixpoint exists to keep out.
python3 - "$E" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """        let mut in_r = seed.clone();"""
new = """        let mut in_r = seed.clone();
        for i in 0..names.len() {
            if let Some(c) = link[i] {
                if let Ok(j) = names.binary_search_by(|n| (*n).cmp(c)) {
                    if link[j] == Some(names[i]) {
                        in_r[i] = true;
                    }
                }
            }
        }"""
assert old in s, "mutation 3 anchor moved"
open(p, "w").write(s.replace(old, new))
PY
run_one 3 "a two-node call cycle is seeded (A CYCLE TREATED AS REDUCING TO NOTHING)"

echo
echo "mutations run: $total, green (BAD): $green, hung (WORSE): $hung"
timeout "$TMO" cargo build --release --workspace --all-targets >/dev/null 2>&1
[ "$green" = "0" ] && [ "$hung" = "0" ]

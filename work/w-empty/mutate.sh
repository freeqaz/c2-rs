#!/bin/sh
# mutate.sh — the MUST-FAIL mutations for mechanism E.
#
# Lane w-empty measurement tooling. Restores `crates/` with `git checkout`, so
# **the tests it runs were committed first** (board #874, lane w-seam).
#
# `work/w-empty/PREREG.md` P6 registers three mutations of the CODE, each
# dropping one condition of `crates/c2-core/src/elide.rs`'s predicate. Each must
# go RED — in the toolchain-backed cell tests, in the c2-core unit tests, or in
# the FBM partition over the 878-TU workload — and each must go red with a
# DISTINCT message naming the condition that was dropped. A mutation that stays
# green is a condition nothing is testing.
#
# | # | mutation | the condition it drops |
# |---|---|---|
# | 1 | `is_empty_callee` answers `true` for every name | **same-TU** — membership in this bundle's set IS the condition |
# | 2 | `of_named` admits every name, empty-bodied or not | **emptiness** |
# | 3 | `of_named` admits exactly the NON-empty names | applies E to non-empty callees, and to nothing else |
#
# Usage:  mutate.sh [--scan]     (--scan also runs the 878-TU FBM partition)
set -eu

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
E=crates/c2-core/src/elide.rs
SCAN=0
[ "${1:-}" = "--scan" ] && SCAN=1

restore() { git checkout -- "$E"; }
trap restore EXIT

green=0
total=0

run_one() {
    n="$1"; what="$2"
    total=$((total + 1))
    echo "=============================================================="
    echo "MUTATION $n — $what"
    echo "=============================================================="
    cargo build --release --workspace --all-targets >/dev/null 2>&1 || {
        echo "  RED (build): the mutation does not compile"
        restore
        return
    }
    red=0
    log="${TMPDIR:-/tmp}/w-empty-mut$n.log"
    cargo test --release -p c2-harness --test empty_elision > "$log" 2>&1 || red=1
    cargo test --release -p c2-core --lib elide >> "$log" 2>&1 || {
        red=1
        echo "  (c2-core unit tests RED too)"
    }
    echo "  cell tests:  $(grep -c '^test .* FAILED' "$log" || true) FAILED"
    grep -h "panicked at\|assertion .* failed\|THE SAME-TU\|THE EMPTINESS" "$log" \
        | sed 's/^/    /' | head -6 || true
    if [ "$SCAN" = "1" ]; then
        cargo build --release -p c2-harness >/dev/null 2>&1
        ./target/release/c2rs gap --list work/dc3-workload/files.txt \
            --flags-file work/dc3-workload/flags.txt \
            --cwd "${C2RS_DC3:-$root/../../../../dc3-decomp}" --jobs 16 2>/dev/null \
            | grep -E "gap-metric (fnbyte-exact|fnbyte-differs|fnbyte-elided|fnbyte-partition-broken) " \
            | sed 's/^ */    /'
    fi
    if [ "$red" = "1" ]; then
        echo "  RESULT: RED"
    else
        echo "  RESULT: GREEN  <-- BAD, this condition is untested"
        green=$((green + 1))
    fi
    restore
}

# ---- 1. drop the SAME-TU condition ----------------------------------------
python3 - "$E" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """    pub fn is_empty_callee(&self, name: &str) -> bool {
        self.empty.binary_search_by(|p| p.as_str().cmp(name)).is_ok()
    }"""
new = """    pub fn is_empty_callee(&self, name: &str) -> bool {
        let _ = name;
        true
    }"""
assert old in s, "mutation 1 anchor moved"
open(p, "w").write(s.replace(old, new))
PY
run_one 1 "is_empty_callee answers true for every name (drops SAME-TU)"

# ---- 2. drop the EMPTINESS condition ---------------------------------------
python3 - "$E" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """            if all_empty {"""
new = """            if all_empty || true {"""
assert old in s, "mutation 2 anchor moved"
open(p, "w").write(s.replace(old, new))
PY
run_one 2 "of_named admits every same-TU name (drops EMPTINESS)"

# ---- 3. apply E to NON-empty callees ---------------------------------------
python3 - "$E" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """            if all_empty {"""
new = """            if !all_empty {"""
assert old in s, "mutation 3 anchor moved"
open(p, "w").write(s.replace(old, new))
PY
run_one 3 "of_named admits exactly the NON-empty names (inverts EMPTINESS)"

echo
echo "mutations run: $total, green (BAD): $green"
cargo build --release --workspace --all-targets >/dev/null 2>&1
[ "$green" = "0" ]

#!/bin/sh
# mutate.sh — drive each POSITIVE check of `census_gate.rs` DIRECTLY and record
# the exact message it produced.
#
# Board #1236: a guard nobody has seen fire is not a guard. The checks in
# `census_gate.rs` run emptiness -> discriminating>0 -> cell floor -> breadth
# floor -> the disagreement pin, and an earlier one that fires first makes every
# later one unreachable (the lane-registry trap, GAPS §7). So each mutation below
# is chosen to hold every EARLIER check's quantity at its measured value and move
# only the one under test, and the pass criterion is that all five produce
# DISTINCT first lines.
#
#   sh work/w-disagree/mutate.sh <mut> ...    (default: all five)
#
# Every mutation is applied with `python3 -c`, run, and reverted with
# `git checkout --` in a trap, so an interrupted run cannot leave the tree dirty.
set -u
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
out="work/w-disagree/out"
mkdir -p "$out"

CEN=crates/c2-il/src/func/census.rs
SEL=crates/c2-core/src/codegen/select.rs
REF=crates/c2-reference/src/lib.rs

revert() { git checkout -- "$CEN" "$SEL" "$REF" 2>/dev/null || true; }
trap revert EXIT INT TERM

patch() { python3 - "$@" <<'PY'
import sys
path, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(path).read()
if old not in s:
    sys.exit("MUTATION SITE NOT FOUND in %s: %r" % (path, old[:60]))
open(path, "w").write(s.replace(old, new, 1))
PY
}

run() {
    name="$1"; filt="$2"
    echo "=================================================================="
    echo "MUTATION $name"
    C2RS_CENSUS_GATE_JOBS=16 cargo test --release -p c2-harness --test census_gate "$filt" \
        -- --nocapture --test-threads 1 > "$out/mut.$name.txt" 2>&1
    echo "  exit=$?"
    grep -E '^census/gate \[' "$out/mut.$name.txt" | head -4
    sed -n '/^thread .* panicked at/,+2p' "$out/mut.$name.txt" | head -6
}

want="${*:-A B C D E}"
for m in $want; do
case "$m" in
# ---------------------------------------------------------------- A
# The port refuses a shape the census accepts IN QUANTITY. Every earlier
# quantity is untouched: `function_gate` still runs on every cell, so captured,
# in-class, discriminating and the shape-key count are all unchanged.
# Must fire: the DISAGREEMENT pin (fixtures) / the NEW FAMILY check (generated).
A)
  revert
  patch "$SEL" \
'    match select_function(func, mode)? {' \
'    if !func.params.is_empty() { return Err(out_of_class("MUTATION A: a function with a formal")); }
    match select_function(func, mode)? {'
  run A the_census_and_the_port_agree
  ;;
# ---------------------------------------------------------------- B
# Every capture fails while the toolchain still resolves. Sources are found, the
# generator runs, and NOTHING is graded.
# Must fire: POPULATION EMPTY.
B)
  revert
  patch "$REF" \
'        std::fs::create_dir_all(work_dir)?;' \
'        return Err(io::Error::other("MUTATION B: every capture fails"));
        #[allow(unreachable_code)]
        std::fs::create_dir_all(work_dir)?;'
  run B the_census_and_the_port_agree_about_what_is_in_class
  ;;
# ---------------------------------------------------------------- C
# The census still calls bodies in class, but the port never reaches its own
# dispatch on ANY of them. captured is held at its measured value; the
# disagreement total goes to the whole in-class population, which is exactly the
# case where a disagreement count is large and says nothing.
# Must fire: NO DISCRIMINATING CELLS -- and NOT the disagreement pin.
C)
  revert
  patch "$CEN" \
'                            match shape_to_function(sh, &name, &src, &resolve, &resolve_data) {' \
'                            match None::<IlFunction>.or_else(|| shape_to_function(sh, &name, &src, &resolve, &resolve_data)).filter(|_| false) {'
  run C the_census_and_the_port_agree_about_what_is_in_class
  ;;
# ---------------------------------------------------------------- D
# The same, for HALF the population: discriminating stays well above zero and
# falls below the floor. captured and the emptiness check are held fixed.
# Must fire: DISCRIMINATING CELLS COLLAPSED.
D)
  revert
  patch "$CEN" \
'                            match shape_to_function(sh, &name, &src, &resolve, &resolve_data) {' \
'                            match shape_to_function(sh, &name, &src, &resolve, &resolve_data).filter(|_| name.len() % 2 == 0) {'
  run D the_census_and_the_port_agree_about_what_is_in_class
  ;;
# ---------------------------------------------------------------- E
# Every discriminating cell keeps its verdict and loses its NAME: captured,
# in-class, discriminating and the disagreement set are all EXACTLY as measured,
# and only the number of distinct census shape keys moves -- 35 -> 1.
# Must fire: DISCRIMINATING BREADTH COLLAPSED.
E)
  revert
  patch "$CEN" \
'            FnVerdict::InClass(s) => (*s).to_string(),' \
'            FnVerdict::InClass(_) => "MUTATION-E-one-key".to_string(),'
  run E the_census_and_the_port_agree_about_what_is_in_class
  ;;
esac
done
revert
echo "=================================================================="
echo "tree restored:"; git status --short -- "$CEN" "$SEL" "$REF"

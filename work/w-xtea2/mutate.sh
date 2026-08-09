#!/bin/sh
# The MUST-FAIL mutations for the `_neg` cells of `memcpy_tail`.
#
# Each mutation deletes exactly ONE shipping clause of `try_parse_memcpy_tail`
# and grades the `_neg` fixture against real `c2.dll` at the workload's own
# `/O1 /Oi`. A cell whose clause can be deleted without the obj going `mismatch`
# is a cell that proves nothing — `w-pool2` §5.1's rule, and the reason each
# mutation targets the cell's OWN clause rather than a shared one.
#
# Always reverts, including on failure.
#
#     work/w-xtea2/mutate.sh [M1|M2|M3|M4]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
src="$repo/crates/c2-il/src/func/body/shapes/memcpy_tail.rs"
neg="fixtures/cpp/wxtea2_mcpy_rev_neg.cpp fixtures/cpp/wxtea2_mcpy_short_neg.cpp fixtures/cpp/wxtea2_mcpy_srcoff_neg.cpp fixtures/cpp/wxtea2_mcpy_stmt_neg.cpp"
pos="fixtures/cpp/wxtea2_memcpy_tail.cpp"

restore() { cp "$here/out/memcpy_tail.rs.orig" "$src"; }
mkdir -p "$here/out"
cp "$src" "$here/out/memcpy_tail.rs.orig"
trap restore EXIT

run_one() {
    cargo build --release -p c2-harness --manifest-path "$repo/Cargo.toml" >/dev/null 2>&1
    # shellcheck disable=SC2086 -- $neg is a deliberate word list
    "$here/one.sh" "/O1 /Oi" $neg "$pos" 2>&1 | sed -n 's/^  \(match\|mismatch\|vocab-gap\|codegen-gap\|port-error\) *\([0-9]*\).*/\1 \2/p'
}

want="${1:-all}"

for m in M1 M2 M3 M4; do
    [ "$want" = all ] || [ "$want" = "$m" ] || continue
    restore
    case "$m" in
    M1) # the register-plan clause: N1 (the swapped operands)
        python3 - "$src" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
a='''    if dst.tok != params[0] || src.tok != params[1] {
        return Err(blk(seg, p, "mcpytail-operands-are-not-already-in-the-argument-registers"));
    }
'''
assert s.count(a)==1, "M1 anchor"
open(p,'w').write(s.replace(a,''))
PY
        ;;
    M2) # the call-window clause: N2 (a length c2 expands inline)
        python3 - "$src" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
a='    if !(MEMCPY_CALL_STEP..=0x7FFF).contains(&len) {'
assert s.count(a)==1, "M2 anchor"
open(p,'w').write(s.replace(a,'    if !(1..=0x7FFF).contains(&len) {'))
PY
        ;;
    M3) # the source-offset clause: N3 (a second `addi`)
        python3 - "$src" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
a='''    if src.off != 0 {
        return Err(blk(seg, p, "mcpytail-source-carries-a-member-offset"));
    }
'''
assert s.count(a)==1, "M3 anchor"
open(p,'w').write(s.replace(a,''))
PY
        ;;
    M4) # the return plumbing: N4 (a second statement after the copy)
        python3 - "$src" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
a='    eat_return_plumbing(seg, &mut p, false, depth)?;'
assert s.count(a)==1, "M4 anchor"
open(p,'w').write(s.replace(a,'    let _ = eat_return_plumbing(seg, &mut p, false, depth);'))
PY
        ;;
    esac
    printf '%s ' "$m"
    run_one | tr '\n' ' '
    printf '\n'
done
restore
cargo build --release -p c2-harness --manifest-path "$repo/Cargo.toml" >/dev/null 2>&1
echo "reverted; baseline:"
run_one | tr '\n' ' '
printf '\n'

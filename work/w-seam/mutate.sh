#!/bin/sh
# mutate.sh — the MUST-FAIL mutations for everything lane w-seam ships.
#
# This lane ships TESTS only, so every mutation is against `cargo test`.  A test
# that does not go RED under the mutation it claims to catch is decoration, and
# `docs/GAPS.md` §7's rule is that a lane states the mutations it RAN rather than
# the ones it would have written.
#
# Each mutation is applied with a literal string substitution, the affected
# crate's tests are run, the tree is restored, and the restoration is VERIFIED
# by re-running clean at the end.  Nothing here touches `work/` or the toolchain.
#
# Usage:  work/w-seam/mutate.sh
set -eu

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

ALLOC=crates/c2-core/src/codegen/alloc.rs
STORE=crates/c2-core/src/codegen/leaf/store.rs

restore() {
    git checkout -- "$ALLOC" "$STORE"
}
trap restore EXIT

red=0
green=0

run_mutation() {
    name="$1"
    shift
    printf '\n== %s\n' "$name"
    if cargo test -p c2-core --release --lib >"$root/work/w-seam/mut_$name.log" 2>&1; then
        printf '   GREEN — THE MUTATION SURVIVED. The test does not catch it.\n'
        green=$((green + 1))
    else
        n=$(grep -c '^test .* FAILED$' "$root/work/w-seam/mut_$name.log" || true)
        printf '   RED — %s failing test(s)\n' \
            "$(grep -oE '[0-9]+ failed' "$root/work/w-seam/mut_$name.log" | head -1)"
        grep '^    codegen::' "$root/work/w-seam/mut_$name.log" | head -8 || true
        red=$((red + 1))
    fi
    restore
}

# ---------------------------------------------------------------------------
# M1 — drop the unconditional `blr`.  This is board #844's own line: it is what
# makes `scheduled_gpr_run_text` a WHOLE BODY, and a lane composing it with a
# frame has to change exactly this.  If no test notices, the structural claim is
# not pinned by anything.
python3 - "$STORE" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
# The `blr` appears at the end of BOTH `scheduled_gpr_run_text` and
# `store_leaf_text`.  This mutation targets the SCHEDULED one — board #844's own
# line — so the anchor carries the comment that precedes only that copy.
old = """    text.extend_from_slice(&encode_blr());
    Some(Ok(text))
}

pub fn store_leaf_text("""
assert s.count(old) == 1, s.count(old)
new = """    Some(Ok(text))
}

pub fn store_leaf_text("""
open(p, "w").write(s.replace(old, new))
PY
run_mutation M1-drop-the-blr

# ---------------------------------------------------------------------------
# M2 — LIFT the mixed refusal for the strict-gap sub-case, which is exactly what
# GRID A was built to test and exactly what a lane trying to convert
# `xboxheap.cpp` would reach for.  Real c2 gives r11 to the CONSTANT on 12 of
# 36 graded cells here, so this mutation emits a WRONG REGISTER rather than a
# refusal.
python3 - "$ALLOC" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """    let constant = producers[0].kind == ProducerKind::Constant;
    if producers
        .iter()
        .any(|p| (p.kind == ProducerKind::Constant) != constant)
    {
        return None;
    }"""
new = """    let constant = producers[0].kind == ProducerKind::Constant;
    let mixed = producers
        .iter()
        .any(|p| (p.kind == ProducerKind::Constant) != constant);
    // M2: lift the refusal where clause 1 decides with no tie.
    let strict_gap = producers.len() == 2 && producers[0].uses != producers[1].uses;
    if mixed && !strict_gap {
        return None;
    }"""
assert s.count(old) == 1
open(p, "w").write(s.replace(old, new))
PY
run_mutation M2-lift-the-strict-gap

# ---------------------------------------------------------------------------
# M3 — run clause 4 FORWARD instead of in reverse source order.  The `C11`
# cell's two constants are both at 2 uses, so this swaps r11 and r10 against
# real c2's bytes.  It is the control that the new store test is asserting
# MEASURED bytes and not the emitter's own output.
python3 - "$ALLOC" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """            if constant && a.uses >= 2 {
                b.first.cmp(&a.first)
            } else {
                a.first.cmp(&b.first)
            }"""
new = """            a.first.cmp(&b.first)"""
assert s.count(old) == 1
open(p, "w").write(s.replace(old, new))
PY
run_mutation M3-clause-4-forward

# ---------------------------------------------------------------------------
printf '\n== mutations: %d RED, %d GREEN\n' "$red" "$green"
printf '== restoring and re-running clean\n'
restore
if cargo test -p c2-core --release --lib >"$root/work/w-seam/mut_clean.log" 2>&1; then
    grep -E '^test result:' "$root/work/w-seam/mut_clean.log"
    printf '== the tree is restored and GREEN\n'
else
    printf '== THE TREE DID NOT RESTORE CLEAN — inspect before committing\n'
    exit 1
fi
[ "$green" -eq 0 ] || exit 1

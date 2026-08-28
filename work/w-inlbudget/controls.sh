#!/bin/sh
# C1/C2 — plant a defect in a registered surface's own constant, require the
# committed DOMAIN.txt baseline to go RED, restore, require GREEN.
# #3746's trap: a `guards` entry whose domain cannot reach it moves ZERO lines
# and is a FALSE coverage claim. This is how each claim here was tested.
set -u
cd "$(dirname "$0")/../.." || exit 2

run() { # file  old-literal  new-literal  label
    f="$1"; old="$2"; new="$3"; label="$4"
    cp "$f" "$f.ctlbak"
    if ! grep -qF -- "$old" "$f"; then
        echo "  $label: MUTATION TARGET ABSENT ('$old') — the control would test the control"
        rm -f "$f.ctlbak"; return 1
    fi
    perl -0pi -e "s/\Q$old\E/$new/" "$f"
    if cmp -s "$f" "$f.ctlbak"; then
        echo "  $label: MUTATION DID NOT APPLY"
        mv "$f.ctlbak" "$f"; return 1
    fi
    out="$(cargo test -q -p c2-core --release --lib \
             surface::tests::the_decision_surface_domain_matches_the_committed_baseline \
             2>&1)"
    moved="$(printf '%s' "$out" | sed -n 's/.*DOMAIN MOVED — \([0-9]*\) line(s).*/\1/p' | head -1)"
    if printf '%s' "$out" | grep -q "test result: ok"; then
        echo "  $label: GREEN — 0 lines moved.  *** FALSE COVERAGE CLAIM ***"
        rc=1
    else
        echo "  $label: RED — $moved domain line(s) moved"
        rc=0
    fi
    mv "$f.ctlbak" "$f"
    # **`mv` PRESERVES THE BACKUP'S MTIME, WHICH IS OLDER THAN THE ARTIFACT
    # CARGO JUST BUILT FROM THE MUTATION.** Without this `touch`, the restored
    # tree links the MUTATED test binary and the closing green check reads a
    # stale build — which is exactly how this control first reported
    # "RESTORED TREE IS RED" on a tree whose sources were correct. The
    # per-mutation readings above are unaffected (the `perl` rewrite always
    # post-dates the artifact), but a restore that does not rebuild is a
    # control that cannot see itself finish.
    touch "$f"
    return $rc
}

fails=0
S=crates/c2-core/src/splice.rs
M=crates/c2-core/src/coff/mangle.rs
O=crates/c2-core/src/codegen/order.rs
N=crates/c2-core/src/codegen/nonce_add_run.rs

echo "C1 — the model's own default, flipped:"
run "$S" "divide_among_remaining_sites: true," "divide_among_remaining_sites: false," \
        "BUDGET_C2.divide := false" || fails=$((fails+1))
run "$S" "site_level_delta: 1," "site_level_delta: 2," \
        "BUDGET_C2.site_level_delta := 2" || fails=$((fails+1))

echo "C2 — every const this lane claims as a surface guard:"
run "$S" "INLINE_BUDGET_FLOOR: i64 = 1000;"      "INLINE_BUDGET_FLOOR: i64 = 1001;"      "INLINE_BUDGET_FLOOR 1000->1001"   || fails=$((fails+1))
run "$S" "INLINE_BUDGET_CEILING: i64 = 35_000;"  "INLINE_BUDGET_CEILING: i64 = 35_001;"  "INLINE_BUDGET_CEILING 35000->35001" || fails=$((fails+1))
run "$S" "INLINE_LEVEL_DEPTH_CAP: i64 = 16;"     "INLINE_LEVEL_DEPTH_CAP: i64 = 17;"     "INLINE_LEVEL_DEPTH_CAP 16->17"    || fails=$((fails+1))
run "$S" "INLINE_CHARGE_EXEMPT_MAX: i64 = 40;"   "INLINE_CHARGE_EXEMPT_MAX: i64 = 41;"   "INLINE_CHARGE_EXEMPT_MAX 40->41"  || fails=$((fails+1))
run "$M" "LITERAL_TEXT_BYTE_LIMIT: usize = 32;"  "LITERAL_TEXT_BYTE_LIMIT: usize = 33;"  "LITERAL_TEXT_BYTE_LIMIT 32->33"   || fails=$((fails+1))
run "$O" "MAX_MULTISYM_PRODUCERS: usize = 2;"    "MAX_MULTISYM_PRODUCERS: usize = 3;"    "MAX_MULTISYM_PRODUCERS 2->3"      || fails=$((fails+1))
run "$O" "MAX_SYMBOL_CROSSINGS: usize = 2;"      "MAX_SYMBOL_CROSSINGS: usize = 3;"      "MAX_SYMBOL_CROSSINGS 2->3"        || fails=$((fails+1))
run "$O" "HEAD_SLOTS_MAX: usize = 2;"            "HEAD_SLOTS_MAX: usize = 3;"            "HEAD_SLOTS_MAX 2->3"              || fails=$((fails+1))
run "$N" "DS_MAX: i32 = 0x7FF8;"                 "DS_MAX: i32 = 0x7FFC;"                 "DS_MAX 0x7FF8->0x7FFC"            || fails=$((fails+1))

# **REFUTED, and by this control.** `HEAD_SLOTS_MAX` was written into
# `UNCOVERED` with the reasoning that `layout_slots` only reads `u` through
# `i.min(u)` and that no run in the domain has enough producers to see `u = 3`.
# The control disagrees: **47 lines move**, because `leading_unproduced` is
# rendered in its own right and `store_order`'s `for u in (0..=head_slots).rev()`
# search changes with the cap. It is a claimed guard of `order.store_run` above,
# and the `UNCOVERED` row is gone. `#3746` says a coverage claim must be
# measured; the same rule caught a NON-coverage claim.

echo "CONTROL: the unmutated tree must be GREEN"
if cargo test -q -p c2-core --release --lib \
     surface::tests::the_decision_surface_domain_matches_the_committed_baseline 2>&1 \
     | grep -q "test result: ok"; then
    echo "  restored tree: GREEN"
else
    echo "  RESTORED TREE IS RED — a control did not restore" ; fails=$((fails+1))
fi

echo
if [ "$fails" -gt 0 ]; then echo "CONTROLS: $fails FAILED"; exit 1; fi
echo "CONTROLS: every claimed guard was watched RED, and the tree restores GREEN"

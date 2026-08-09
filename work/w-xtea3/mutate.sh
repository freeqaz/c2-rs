#!/bin/sh
# MUST-FAIL MUTATION — apply one named clause-deletion, rebuild, grade the ONE
# fixture that clause is supposed to fence, restore the tree.
#
# A fence nobody has broken on purpose is a fence nobody has graded
# (`w-xtea2` #2664/#2665: an over-fenced cell grades NONE of its clauses, and
# the repair is merging clauses rather than adding cells).
#
#     work/w-xtea3/mutate.sh M1
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
which="$1"
nonce="crates/c2-il/src/func/body/shapes/nonce_add_run.rs"
round="crates/c2-il/src/func/body/shapes/xtea_round_loop.rs"
xenc="crates/c2-il/src/func/body/shapes/xtea_encrypt_loop.rs"

restore() { git -C "$repo" checkout -- "$nonce" "$round" "$xenc"; }
trap restore EXIT

case "$which" in
M1)  # the run-length fence: admit a ONE-statement run and emit the two-statement plan
    python3 - "$repo/$nonce" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """    let mut elems = Vec::with_capacity(RUN_LEN);
    for _ in 0..RUN_LEN {
        elems.push(eat_stmt(seg, &mut p, this_tok, src_tok, addend_tok)?);
    }"""
new = """    let mut elems = Vec::with_capacity(RUN_LEN);
    elems.push(eat_stmt(seg, &mut p, this_tok, src_tok, addend_tok)?);
    match eat_stmt(seg, &mut p, this_tok, src_tok, addend_tok) {
        Ok(e) => elems.push(e),
        Err(_) => elems.push(Elem {
            dst_off: elems[0].dst_off + ELEM,
            src_off: elems[0].src_off + ELEM,
        }),
    }"""
assert old in s
open(p, 'w').write(s.replace(old, new, 1))
PY
    fixture=fixtures/cpp/wxtea3_nonce1_neg.cpp ;;
M2)  # the addend clause, deleted WHOLE — the fact is the CONJUNCTION (#2665),
     # so a mutation that breaks half of it leaves the other half refusing
    python3 - "$repo/$nonce" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """    let four = value_class(tag, kind).is_some();
    *p += tw;
    // The `2C` widening, read non-committally so that the conjunction below can
    // be one refusal rather than two.
    let widened = eat_widen8(seg, p, "nonce-addend-widening").is_ok();
    if !(four && widened) {
        return Err(blk(seg, *p, "nonce-addend-is-not-a-4-byte-value-widened-to-eight"));
    }"""
new = """    let _ = value_class(tag, kind);
    *p += tw;
    let _ = eat_widen8(seg, p, "nonce-addend-widening");"""
assert old in s
open(p, 'w').write(s.replace(old, new, 1))
PY
    fixture=fixtures/cpp/wxtea3_nonce_u64_neg.cpp ;;
M3)  # the round-constant statement: make `sum += DELTA` optional
    python3 - "$repo/$round" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """    if eat_push(seg, &mut p, "xtea-delta")? != sum {
        return Err(blk(seg, p, "xtea-round-update-is-not-the-sum"));
    }
    let delta = eat_lit(seg, &mut p, "xtea-delta-k")?;
    if delta != DELTA {
        return Err(blk(seg, p, "xtea-round-constant-is-not-the-measured-delta"));
    }
    eat_assign_end(seg, &mut p, 0x0F, "xtea-delta-store")?;"""
new = """    let delta = DELTA;
    let mut q = p;
    if eat_push(seg, &mut q, "xtea-delta").ok() == Some(sum)
        && eat_lit(seg, &mut q, "xtea-delta-k").ok() == Some(DELTA)
        && eat_assign_end(seg, &mut q, 0x0F, "xtea-delta-store").is_ok()
    {
        p = q;
    }"""
assert old in s
open(p, 'w').write(s.replace(old, new, 1))
PY
    fixture=fixtures/cpp/wxtea3_nosum_neg.cpp ;;
M4)  # the fixed right-shift: accept any shift and emit the measured one
    python3 - "$repo/$round" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """    if eat_lit(seg, p, "xtea-half-round-shr-k")? != SHR_K {
        return Err(blk(seg, *p, "xtea-half-round-right-shift-is-not-five"));
    }"""
new = """    let _ = eat_lit(seg, p, "xtea-half-round-shr-k")?;"""
assert old in s
open(p, 'w').write(s.replace(old, new, 1))
PY
    fixture=fixtures/cpp/wxtea3_shift6_neg.cpp ;;
M5)  # the framed loop's nonce bump: make the second statement optional
    python3 - "$repo/$xenc" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = """    eat_opt_stmt_marker(seg, &mut p);
    if eat_nonce_elem(seg, &mut p, this, i, "xenc-bump")? != nonce_off {
        return Err(blk(seg, p, "xenc-bumped-member-is-not-the-one-the-call-read"));
    }
    if eat_lit(seg, &mut p, "xenc-bump-k")? != 1 {
        return Err(blk(seg, p, "xenc-nonce-step-is-not-one"));
    }
    eat_assign_end(seg, &mut p, 0x0F, "xenc-bump-store")?;"""
new = """    let mut q = p;
    eat_opt_stmt_marker(seg, &mut q);
    if eat_nonce_elem(seg, &mut q, this, i, "xenc-bump").ok() == Some(nonce_off)
        && eat_lit(seg, &mut q, "xenc-bump-k").ok() == Some(1)
        && eat_assign_end(seg, &mut q, 0x0F, "xenc-bump-store").is_ok()
    {
        p = q;
    }"""
assert old in s
open(p, 'w').write(s.replace(old, new, 1))
PY
    fixture=fixtures/cpp/wxtea3_nobump_neg.cpp ;;
*)  echo "unknown mutation $which"; exit 2 ;;
esac

cargo build --release --manifest-path "$repo/Cargo.toml" -p c2-harness --bin c2rs >/dev/null 2>&1 \
    || { echo "$which DID NOT BUILD"; exit 1; }
echo "== $which  $fixture"
sh "$here/one.sh" /O1 "$fixture" | sed -n '/GAP REPORT/,/capture-fail/p'

//! **W-OSFINFO — a range-and-flag guarded two-level table lookup whose two
//! failure statements are TAIL-MERGED with its success statement.**
//!
//! ```c
//!   int f(int fh) {
//!       if (fh >= 0 && (unsigned)fh < (unsigned)LIMIT) {   // two globals: one
//!           int i = fh >> K_SHIFT;                          // read by VALUE,
//!           E *e = (E *)((char *)TABLE[i]                   // one INDEXED
//!                        + (fh & K_MASK) * K_ELEM);
//!           if ((e->OFF_FILE & K_BIT) != 0) {               // a BYTE field
//!               if (e->OFF_HND != K_INVALID) {              // a WORD field
//!                   e->OFF_HND = K_INVALID;
//!                   return K_OK;
//!               }
//!           }
//!       }
//!       *errfn1() = K_ERRNO;
//!       *errfn2() = K_DOSERRNO;
//!       return K_FAIL;
//!   }
//! ```
//!
//! This is `src/xdk/LIBCMT/osfinfo.cpp`'s `_free_osfhnd`, a FRONTIER TU with
//! exactly one emitted function — so the TU converts on this class or on none.
//! It is the **third and last** TU of the undefined-external seam
//! (`vswprnc` · `undname` · this).
//!
//! ## Why a TRANSCRIPTION and not a general `cflow-if-n` lowering
//!
//! The same argument [`super::alloc_init_or_fail`],
//! [`super::guard_chain_shared_tail`] and [`super::if_call_join`] make, and it
//! is `docs/ARCHITECTURE_SEAMS.md` §7's. What ships here is **thirty-one words
//! of one named function class, `/O1` only**, `NotImplemented` outside.
//! **Accepting this shape is not a claim about `cflow-if-n` as a class.**
//!
//! ## The five things a general lowering gets wrong
//!
//! Read off the real obj at the workload's own flags
//! (`work/w-osfinfo/ref/osfinfo/dis.txt`) and decoded token by token in
//! `work/w-osfinfo/OSFINFO_BODY.md`, both committed **before** this file was
//! written. The emitted words are in `c2_core::codegen::osf_handle_guard`'s
//! module doc; the facts the READER has to pin are:
//!
//! 1. **The two entry guards use DIFFERENT compare forms on the SAME operand.**
//!    `fh >= 0` converts the loaded value to a **signed** width-4 type and c2
//!    emits `cmpwi cr6,r3,0`; `fh < LIMIT` converts the *other* operand to an
//!    **unsigned** one and c2 emits `cmplw cr6,r3,r11`. Four words apart, same
//!    register. A class that reached for one form throughout emits the right
//!    program with one wrong word — so both `2C` conversions are required, by
//!    signedness, and the sides they land on are not interchangeable.
//! 2. **The two globals are reached differently and that decides two
//!    instructions.** `LIMIT` arrives as a `B9` value READ — its low half is a
//!    **`lwz` displacement** — and `TABLE` as a `26` designator, whose low half
//!    is an `addi`. A recognizer that took both as designators would be right
//!    about the symbols and wrong about the words.
//! 3. **The flag member is a BYTE and the handle member is a WORD**, and the
//!    read TYPE is the only thing that says so: `30 82 11 …` is the `lbz` and
//!    `30 86 41 …` is the `lwz`. Getting it backwards links.
//! 4. **`OFF_HND` is pinned to 0**, because the success store and the error
//!    store are ONE word. At any other offset they are two, the block plan is
//!    different, and this class has not been graded on it.
//! 5. **THREE labels and TWO of them fall into each other with no statement
//!    between.** `29 <Lfield> · 54 06 · 54 05 · 4F 01 xx · 54 04 · 29 <Lrange>`
//!    is the whole gap: all four guards reach one block through two label
//!    definitions and **zero jumps**. A recognizer that required a jump between
//!    them refuses; one that collapsed them into a single label would be right
//!    here by accident and wrong on the first body that puts a statement in the
//!    gap.
//!
//! ## The fence
//!
//! Every clause below is required literally, and each names the measurement
//! behind it rather than a preference.
//!
//! * **`/O1` only, asked FIRST, in the PARSER.** Board **#1638**, which has
//!   fired twice. A mode clause that lived only in the emitter would make the
//!   census count this body in class while `PortC2` refused it;
//!   `census_gate.rs` is the cross-check.
//! * **A function with exactly ONE formal and no `this`.** `fh` arrives in r3
//!   and is read four times, the last of them *after* nothing — no value in this
//!   body outlives a `bl`, which is what makes the frame `saved_gprs: 0`.
//! * **Four names, all distinct.** Two data symbols and two callees. A body
//!   naming the same symbol twice is ONE undefined external in c2's table, so
//!   the symbol table is a record shorter and every index after it moves.
//! * **The element size may not be a power of two** and **the outer scale must
//!   be exactly 4.** The body multiplies twice and c2 picks `slwi` or `mulli` by
//!   whether the constant is a power of two; with one witness of each form there
//!   is nothing to fit, so the chooser is refused rather than guessed (board
//!   **#1706**).
//! * **Both masks must be `2^n − 1`**, because the class has a `clrlwi` and no
//!   other masking word — and the flag mask must additionally stay **inside the
//!   loaded byte's value bits** (`≤ 127`). At or above the sign bit a `char`
//!   load needs an `extsb` that this class does not emit.
//! * **The compared sentinel and the stored sentinel must be the same
//!   literal.** They reach a `cmpwi` and a `li` that the emitter drives from ONE
//!   field; two literals would be a field this class cannot vary independently.
//! * **Every label distinct.** Two aliasing labels are one block, and every
//!   displacement after the alias would be right for a program this is not.

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::calls::eat_call_token;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, is_int4_type, is_ptr_to_4, read_token_var, read_type,
    read_varint,
};
use crate::func::OsfHandleGuard;

/// The outer table's element scale, in bytes. **Pinned, not carried** — see the
/// class doc's fence. `c2_core::codegen::osf_handle_guard::K_SCALE` is the
/// emitter's copy and the two are asserted equal by that module's tests.
/// PROV[O] the outer table's element scale in bytes, measured. Pinned rather than carried (see the class doc's fence); `c2_core::codegen::osf_handle_guard::K_SCALE` is the emitter's copy and a test asserts the two equal.
pub(crate) const OSF_TABLE_SCALE: i32 = 4;

/// Consume any TYPE and return its three discriminating fields.
fn eat_any_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8, u32), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) => {
            *p += w;
            Ok((tag, kind, id))
        }
        None => Err(blk(seg, *p, what)),
    }
}

/// `26 <tok>` — a symbol push. Returns the token.
fn eat_designator(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `B9 <tok> <TYPE>` — a value read. Returns the token and the type's two
/// discriminating bytes, because for this class a type decides an instruction.
fn eat_load(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, u8, u8), Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    Ok((tok, tag, kind))
}

/// `29 <tok>` — a label definition.
fn eat_label(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x29) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `<op> <tok>` for a transfer opcode. Returns the target label.
fn eat_transfer(seg: &[u8], p: &mut usize, op: u8, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, op) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume `54 <k>`, requiring the exact depth `k`.
///
/// The depths are pinned rather than merely decoded, for
/// [`super::alloc_init_or_fail`]'s reason: they are the only place the *bracing*
/// of the source shows up in this stream, and a differently braced body is a
/// different block plan.
fn eat_close(seg: &[u8], p: &mut usize, k: u8, what: &'static str) -> Result<(), Block> {
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x54) || !eat_byte(seg, p, k) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// `33 <TYPE> <varint>` — a literal that has to fit `simm16`, because every one
/// of them lands in a `li`/`addi`/`cmpwi`/`mulli` immediate field.
fn eat_lit(seg: &[u8], p: &mut usize, what: &'static str) -> Result<i32, Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    eat_any_type(seg, p, what)?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(blk(seg, *p, "osf-literal-wider-than-simm16"));
    }
    Ok(k)
}

/// `2C <TYPE> <varint>` — a conversion. Returns the target type's two
/// discriminating bytes: **the conversion's target decides the compare form**,
/// which is fact 1 of the class doc.
fn eat_convert(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8), Block> {
    if !eat_byte(seg, p, 0x2C) {
        return Err(blk(seg, *p, what));
    }
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    Ok((tag, kind))
}

/// `30 <TYPE>` — an indirect read. Returns the type, which is what decides
/// `lbz` against `lwz`.
fn eat_deref(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8), Block> {
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, what));
    }
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    Ok((tag, kind))
}

/// `32 <TYPE>` then `4B` — a store and its statement end.
fn eat_store_end(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8), Block> {
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, what));
    }
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, what));
    }
    Ok((tag, kind))
}

/// `28 00 00` — the **subscript add**, whose two trailing bytes have no known
/// meaning (`docs/IL_EXPR_LAYER.md` §4).
///
/// Required at their witnessed value rather than skipped. A byte nobody has
/// characterized is a byte that may select something, and the whole class is
/// thirty-one fixed words: if those two ever carry a scale or a signedness this
/// body would emit the same words for a different program.
fn eat_subscript_add(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    if seg.get(*p..*p + 3) != Some(&[0x28, 0x00, 0x00][..]) {
        return Err(blk(seg, *p, what));
    }
    *p += 3;
    Ok(())
}

/// True for a TYPE naming a **signed** width-4 integer — the target of the first
/// guard's conversion, and therefore `cmpwi`.
fn is_signed4(tag: u8, kind: u8) -> bool {
    is_int4_type(tag, kind) && (kind & 0x0F) == 0x1
}

/// True for a TYPE naming an **unsigned** width-4 integer — the target of the
/// second guard's conversion, and therefore `cmplw`.
fn is_unsigned4(tag: u8, kind: u8) -> bool {
    is_int4_type(tag, kind) && (kind & 0x0F) == 0x2
}

/// True for a TYPE naming a **one-byte** object of either sign — the `lbz`.
///
/// Both signs are admitted because the flag mask is fenced below the sign bit
/// (`k_bit ≤ 127`), which makes the widening dead: for any mask inside the value
/// bits, `(signed char)x & mask` and `(zero-extended x) & mask` are the same
/// number, so c2's bare `lbz` is right either way and no `extsb` is owed.
fn is_byte_type(tag: u8, kind: u8) -> bool {
    (tag & 0x0F) == 0x2 && (kind >> 4) == 1 && matches!(kind & 0x0F, 0x1 | 0x2)
}

/// `mask` is `2^n − 1` for some `n ≥ 1` — the only masks this class has a word
/// for.
fn is_low_contiguous(mask: i32) -> bool {
    mask > 0 && (mask as u32).checked_add(1).is_some_and(u32::is_power_of_two)
}

/// One entry guard: `B9 <fh> <T>` … `<rel>` `38 <label>`, with the conversion in
/// between deciding the compare form.
///
/// `rhs` is the caller's, because the two guards differ in what the right-hand
/// side is — a literal zero for the first, a second global's value for the
/// second — and that difference *is* the two compare forms.
struct Guard {
    label: u32,
}

/// **The recognizer.** `start` is the first byte after the body's own `53`, the
/// leading line markers and the outer `if`'s own `53` — all eaten by
/// `eat_scopes`, so the cursor arrives on the first guard's `B9`. `lo` is the
/// `4C 4F 11` marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` on the first byte that is not its grammar, so a
/// body that declines still reports its dispatch arm's blocker and no census key
/// moves.
pub(crate) fn try_parse_osf_handle_guard(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not in the emitter.**
    // Board **#1638**, which has fired twice. Asked FIRST, before any body byte
    // is read, so the refusal cannot depend on how far the walk got.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "osf-not-o1"));
    }
    // A free function with ONE formal: `fh` in r3 and no `this`. `parse_params`
    // prepends the `this` token when the pre-body region binds one and REFUSES
    // when the binding is undetermined, so "no `this`" is an established fact
    // and not a count.
    let params = parse_params(seg, lo)?;
    let formals = parse_formals(seg, lo)?;
    if params.len() != 1 || formals.len() != 1 || params[0] != formals[0] {
        return Err(blk(seg, start, "osf-not-one-formal-free-fn"));
    }
    let fh = params[0];

    let mut p = start;

    // ---- guard 1: `fh >= 0` — converted to SIGNED, so `cmpwi` --------------
    let g1 = eat_range_guard_low(seg, &mut p, fh)?;

    // ---- guard 2: `fh < LIMIT` — converted to UNSIGNED, so `cmplw` --------
    let (limit_tok, g2) = eat_range_guard_high(seg, &mut p, fh)?;
    if g2.label != g1.label {
        return Err(blk(seg, p, "osf-range-guards-branch-apart"));
    }
    let l_range = g1.label;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "osf-range-scopes"));
    }

    // ---- `i = fh >> K_SHIFT;` ----------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let index_tok = eat_designator(seg, &mut p, "osf-index-designator")?;
    if index_tok == fh {
        return Err(blk(seg, p, "osf-index-is-the-formal"));
    }
    let (tok, _, _) = eat_load(seg, &mut p, "osf-index-source")?;
    if tok != fh {
        return Err(blk(seg, p, "osf-index-source-is-not-the-formal"));
    }
    let (tag, kind) = eat_convert(seg, &mut p, "osf-index-convert")?;
    if !is_signed4(tag, kind) {
        // An unsigned shift is a `srwi` and a different word.
        return Err(blk(seg, p, "osf-index-shift-is-not-arithmetic"));
    }
    let k_shift = eat_lit(seg, &mut p, "osf-index-shift")?;
    if !(1..=31).contains(&k_shift) {
        return Err(blk(seg, p, "osf-index-shift-out-of-range"));
    }
    if !eat_byte(seg, &mut p, 0x0A) {
        return Err(blk(seg, p, "osf-index-not-a-shift-right"));
    }
    eat_store_end(seg, &mut p, "osf-index-store")?;

    // ---- `e = (E*)((char*)TABLE[i] + (fh & K_MASK) * K_ELEM);` -------------
    eat_opt_stmt_marker(seg, &mut p);
    let entry_tok = eat_designator(seg, &mut p, "osf-entry-designator")?;
    if entry_tok == fh || entry_tok == index_tok {
        return Err(blk(seg, p, "osf-entry-aliases-an-earlier-value"));
    }
    let table_tok = eat_designator(seg, &mut p, "osf-table-designator")?;
    let (tok, _, _) = eat_load(seg, &mut p, "osf-table-index")?;
    if tok != index_tok {
        return Err(blk(seg, p, "osf-table-index-is-not-the-shifted-value"));
    }
    let k_scale = eat_lit(seg, &mut p, "osf-table-scale")?;
    if k_scale != OSF_TABLE_SCALE {
        // The outer table's elements are pointers and c2 emits `slwi rD,rS,2`.
        // Any other scale is either a different shift amount or a `mulli`, and
        // which of the two c2 picks is the chooser this class refuses.
        return Err(blk(seg, p, "osf-table-scale-is-not-four"));
    }
    if !eat_byte(seg, &mut p, 0x04) {
        return Err(blk(seg, p, "osf-table-scale-not-a-multiply"));
    }
    eat_subscript_add(seg, &mut p, "osf-table-subscript")?;
    let (tag, kind) = eat_deref(seg, &mut p, "osf-table-load")?;
    if !is_ptr_to_4(tag, kind) && !is_int4_type(tag, kind) {
        // The `lwzx` loads a width-4 value. A narrower element is a different
        // instruction entirely.
        return Err(blk(seg, p, "osf-table-element-is-not-width-4"));
    }
    let (tok, _, _) = eat_load(seg, &mut p, "osf-inner-index")?;
    if tok != fh {
        return Err(blk(seg, p, "osf-inner-index-is-not-the-formal"));
    }
    let k_mask = eat_lit(seg, &mut p, "osf-inner-mask")?;
    if !is_low_contiguous(k_mask) {
        return Err(blk(seg, p, "osf-inner-mask-is-not-2n-minus-1"));
    }
    if !eat_byte(seg, &mut p, 0x0B) {
        return Err(blk(seg, p, "osf-inner-mask-not-a-bit-and"));
    }
    let k_elem = eat_lit(seg, &mut p, "osf-element-size")?;
    if k_elem <= 0 || (k_elem as u32).is_power_of_two() {
        // c2 emits a `slwi` for a power-of-two element size and a `mulli`
        // otherwise; with one witness of each form the chooser is not fitted.
        return Err(blk(seg, p, "osf-element-size-is-a-power-of-two"));
    }
    if !eat_byte(seg, &mut p, 0x04) {
        return Err(blk(seg, p, "osf-element-size-not-a-multiply"));
    }
    eat_subscript_add(seg, &mut p, "osf-entry-subscript")?;
    eat_store_end(seg, &mut p, "osf-entry-store")?;

    // ---- guard 3: `(e->OFF_FILE & K_BIT) != 0` — a BYTE, on cr0 ------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "osf-flag-scope"));
    }
    let off_file = eat_member(seg, &mut p, entry_tok, "osf-flag-member")?;
    let (tag, kind) = eat_deref(seg, &mut p, "osf-flag-load")?;
    if !is_byte_type(tag, kind) {
        // A width-4 flag field is a `lwz` where this class emits `lbz`.
        return Err(blk(seg, p, "osf-flag-member-is-not-a-byte"));
    }
    let (tag, kind) = eat_convert(seg, &mut p, "osf-flag-convert")?;
    if !is_signed4(tag, kind) {
        return Err(blk(seg, p, "osf-flag-not-widened-to-int"));
    }
    let k_bit = eat_lit(seg, &mut p, "osf-flag-mask")?;
    // TWO keys, not one: the two facts decline for different reasons and a
    // single key would report a non-contiguous mask as a sign-bit problem. Found
    // by the `_neg` probe run, which is what that file is for.
    if !is_low_contiguous(k_bit) {
        return Err(blk(seg, p, "osf-flag-mask-is-not-2n-minus-1"));
    }
    if k_bit > 0x7F {
        // At or above the loaded byte's sign bit a `char` costs an `extsb` this
        // class does not emit; below it the widening is dead and the bare `lbz`
        // is right for either sign.
        return Err(blk(seg, p, "osf-flag-mask-reaches-the-sign-bit"));
    }
    if !eat_byte(seg, &mut p, 0x0B) {
        return Err(blk(seg, p, "osf-flag-not-a-bit-and"));
    }
    if eat_lit(seg, &mut p, "osf-flag-test-literal")? != 0 {
        return Err(blk(seg, p, "osf-flag-test-not-against-zero"));
    }
    if !eat_byte(seg, &mut p, 0x20) {
        return Err(blk(seg, p, "osf-flag-test-relation"));
    }
    let l_field = eat_transfer(seg, &mut p, 0x38, "osf-flag-branch")?;

    // ---- guard 4: `e->OFF_HND != K_INVALID` — a WORD ----------------------
    let off_hnd = eat_member(seg, &mut p, entry_tok, "osf-live-member")?;
    if off_hnd != 0 {
        // **The pin.** The success store and the error store are ONE word, which
        // is only legal at displacement zero. See the class doc's fact 4.
        return Err(blk(seg, p, "osf-handle-member-is-not-at-offset-zero"));
    }
    let (tag, kind) = eat_deref(seg, &mut p, "osf-live-load")?;
    if !is_int4_type(tag, kind) {
        return Err(blk(seg, p, "osf-handle-member-is-not-a-word"));
    }
    let k_invalid = eat_lit(seg, &mut p, "osf-live-sentinel")?;
    if !eat_byte(seg, &mut p, 0x20) {
        return Err(blk(seg, p, "osf-live-relation"));
    }
    if eat_transfer(seg, &mut p, 0x38, "osf-live-branch")? != l_field {
        return Err(blk(seg, p, "osf-live-branch-elsewhere"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "osf-live-scopes"));
    }

    // ---- `e->OFF_HND = K_INVALID;` — the success half of the merged store --
    eat_opt_stmt_marker(seg, &mut p);
    if eat_member(seg, &mut p, entry_tok, "osf-success-store")? != off_hnd {
        return Err(blk(seg, p, "osf-success-store-is-a-different-member"));
    }
    if eat_lit(seg, &mut p, "osf-success-value")? != k_invalid {
        // The compared literal and the stored literal reach a `cmpwi` and a `li`
        // that the emitter drives from ONE field.
        return Err(blk(seg, p, "osf-success-value-is-not-the-sentinel"));
    }
    eat_store_end(seg, &mut p, "osf-success-store-end")?;

    // ---- `return K_OK;` ----------------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let (k_ok, ret_ty) = eat_return(seg, &mut p, "osf-success-return")?;
    let l_epi = eat_transfer(seg, &mut p, 0x3A, "osf-success-jump")?;

    // ---- the two label definitions with NOTHING between them --------------
    eat_close(seg, &mut p, 0x08, "osf-live-close-8")?;
    eat_close(seg, &mut p, 0x07, "osf-live-close-7")?;
    if eat_label(seg, &mut p, "osf-field-label")? != l_field {
        return Err(blk(seg, p, "osf-field-label"));
    }
    eat_close(seg, &mut p, 0x06, "osf-field-close-6")?;
    eat_close(seg, &mut p, 0x05, "osf-field-close-5")?;
    eat_close(seg, &mut p, 0x04, "osf-field-close-4")?;
    if eat_label(seg, &mut p, "osf-range-label")? != l_range {
        return Err(blk(seg, p, "osf-range-label"));
    }
    eat_close(seg, &mut p, 0x03, "osf-range-close-3")?;

    // ---- the error block: two calls, two stores through their results ------
    let (errno_tok, k_errno) = eat_store_through_call(seg, &mut p, "osf-errno")?;
    let (doserrno_tok, k_doserrno) = eat_store_through_call(seg, &mut p, "osf-doserrno")?;

    // ---- `return K_FAIL;` --------------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let (k_fail, ret_ty2) = eat_return(seg, &mut p, "osf-fail-return")?;
    if ret_ty2 != ret_ty {
        return Err(blk(seg, p, "osf-two-return-types"));
    }
    if eat_transfer(seg, &mut p, 0x3A, "osf-fail-jump")? != l_epi {
        return Err(blk(seg, p, "osf-fail-jump-elsewhere"));
    }
    eat_close(seg, &mut p, 0x02, "osf-wind-2")?;
    if eat_label(seg, &mut p, "osf-epilogue-label")? != l_epi {
        return Err(blk(seg, p, "osf-epilogue-label"));
    }
    // The function tail. Landing exactly on it is the whole acceptance claim: a
    // walk that ends anywhere else consumed a byte it did not understand.
    // PROV[O] the seven-byte `.ex` function tail, read off captures. See `alloc_init_or_fail::FN_TAIL`.
    const FN_TAIL: [u8; 7] = [0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00];
    if seg.get(p..p + FN_TAIL.len()) != Some(&FN_TAIL[..]) {
        return Err(blk(seg, p, "osf-not-the-function-tail"));
    }

    // Every label distinct.
    let labels = [l_range, l_field, l_epi];
    for i in 0..labels.len() {
        for j in i + 1..labels.len() {
            if labels[i] == labels[j] {
                return Err(blk(seg, p, "osf-labels-alias"));
            }
        }
    }
    // Four names must be four names.
    let names = [limit_tok, table_tok, errno_tok, doserrno_tok];
    for i in 0..names.len() {
        for j in i + 1..names.len() {
            if names[i] == names[j] {
                return Err(blk(seg, p, "osf-externals-alias"));
            }
        }
    }

    Ok(BodyShape::OsfHandleGuard(OsfHandleGuard {
        params,
        limit_tok,
        table_tok,
        errno_tok,
        doserrno_tok,
        k_shift,
        k_mask,
        k_elem,
        off_file,
        k_bit,
        off_hnd,
        k_invalid,
        k_ok,
        k_errno,
        k_doserrno,
        k_fail,
    }))
}

/// `B9 <fh> <T> · 2C <signed4> · 33 <T> 0 · 23 · 38 <L>` — the `fh >= 0` guard.
///
/// The conversion's target is required **signed**: that is what makes c2 emit
/// `cmpwi` here and `cmplw` four words later on the same register.
fn eat_range_guard_low(seg: &[u8], p: &mut usize, fh: u32) -> Result<Guard, Block> {
    let (tok, _, _) = eat_load(seg, p, "osf-low-guard-load")?;
    if tok != fh {
        return Err(blk(seg, *p, "osf-low-guard-names-the-wrong-value"));
    }
    let (tag, kind) = eat_convert(seg, p, "osf-low-guard-convert")?;
    if !is_signed4(tag, kind) {
        return Err(blk(seg, *p, "osf-low-guard-is-unsigned-so-c2-emits-cmplwi"));
    }
    if eat_lit(seg, p, "osf-low-guard-literal")? != 0 {
        return Err(blk(seg, *p, "osf-low-guard-not-against-zero"));
    }
    if !eat_byte(seg, p, 0x23) {
        return Err(blk(seg, *p, "osf-low-guard-relation"));
    }
    let label = eat_transfer(seg, p, 0x38, "osf-low-guard-branch")?;
    Ok(Guard { label })
}

/// `B9 <fh> <T> · B9 <limit> <T> · 2C <unsigned4> · 22 · 38 <L>` — the
/// `fh < LIMIT` guard. Returns the limit's token.
///
/// The limit arrives as a **value read**, not a designator, which is what makes
/// its low half a `lwz` displacement rather than an `addi`.
fn eat_range_guard_high(seg: &[u8], p: &mut usize, fh: u32) -> Result<(u32, Guard), Block> {
    let (tok, _, _) = eat_load(seg, p, "osf-high-guard-load")?;
    if tok != fh {
        return Err(blk(seg, *p, "osf-high-guard-names-the-wrong-value"));
    }
    let (limit_tok, tag, kind) = eat_load(seg, p, "osf-high-guard-limit")?;
    if limit_tok == fh {
        return Err(blk(seg, *p, "osf-high-guard-limit-is-the-formal"));
    }
    if !is_int4_type(tag, kind) {
        return Err(blk(seg, *p, "osf-high-guard-limit-is-not-a-word"));
    }
    let (tag, kind) = eat_convert(seg, p, "osf-high-guard-convert")?;
    if !is_unsigned4(tag, kind) {
        return Err(blk(seg, *p, "osf-high-guard-is-signed-so-c2-emits-cmpw"));
    }
    if !eat_byte(seg, p, 0x22) {
        return Err(blk(seg, *p, "osf-high-guard-relation"));
    }
    let label = eat_transfer(seg, p, 0x38, "osf-high-guard-branch")?;
    Ok((limit_tok, Guard { label }))
}

/// `B9 <base> <T> · 33 <int> <k> · 27 <T>` — a member reference `base->k`,
/// returning the byte offset. `27` emits no instruction of its own: the offset
/// lands in the following load or store's displacement field.
fn eat_member(
    seg: &[u8],
    p: &mut usize,
    want_base: u32,
    what: &'static str,
) -> Result<i32, Block> {
    let (tok, _, _) = eat_load(seg, p, what)?;
    if tok != want_base {
        return Err(blk(seg, *p, "osf-member-base-is-the-wrong-value"));
    }
    let k = eat_lit(seg, p, what)?;
    if !(0..=0x7FFF).contains(&k) {
        return Err(blk(seg, *p, "osf-member-offset-negative"));
    }
    if !eat_byte(seg, p, 0x27) {
        return Err(blk(seg, *p, "osf-member-not-an-offset-op"));
    }
    eat_any_type(seg, p, what)?;
    Ok(k)
}

/// `33 <T> <k> · 41 <T>` — a return value. Returns the literal and the return
/// type, so the two returns can be required to agree.
fn eat_return(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(i32, (u8, u8)), Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(blk(seg, *p, "osf-return-wider-than-simm16"));
    }
    if !eat_byte(seg, p, 0x41) {
        return Err(blk(seg, *p, "osf-return-not-a-result"));
    }
    let (rtag, rkind, _) = eat_any_type(seg, p, what)?;
    if (rtag, rkind) != (tag, kind) {
        return Err(blk(seg, *p, "osf-return-literal-type-differs"));
    }
    Ok((k, (tag, kind)))
}

/// `26 <fn> · BD … · 4C · 33 <int4> <k> · 32 <int4> · 4B` — a store through a
/// nullary call's returned pointer. Returns the callee's token and the literal.
///
/// **Both calls take no arguments**, which is what makes the frame's outgoing
/// parameter area the 8-slot minimum and therefore the whole frame 96 bytes. An
/// argument here would be a `li` this class has no word for.
fn eat_store_through_call(
    seg: &[u8],
    p: &mut usize,
    what: &'static str,
) -> Result<(u32, i32), Block> {
    eat_opt_stmt_marker(seg, p);
    let callee = eat_designator(seg, p, what)?;
    eat_call_token(seg, p)?;
    if !eat_byte(seg, p, 0x4C) {
        return Err(blk(seg, *p, "osf-call-takes-arguments"));
    }
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, "osf-call-store-not-a-literal"));
    }
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    if !is_int4_type(tag, kind) {
        return Err(blk(seg, *p, "osf-call-store-is-not-a-word"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(blk(seg, *p, "osf-call-store-wider-than-simm16"));
    }
    let (stag, skind) = eat_store_end(seg, p, "osf-call-store-end")?;
    if !is_int4_type(stag, skind) {
        return Err(blk(seg, *p, "osf-call-store-type-is-not-a-word"));
    }
    Ok((callee, k))
}

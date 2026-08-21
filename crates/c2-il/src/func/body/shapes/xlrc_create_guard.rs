//! **W-XLR — a two-stage create/attach guard whose four failure paths converge
//! on one returned status, in a function that saves SIX callee-saved GPRs
//! through the `__savegprlr_26` helper.**
//!
//! ```c
//!   long f(ID id, unsigned *outSize, C **outC, T **outT) {
//!       unsigned size = K_INIT;          // ADDRESS-TAKEN — a stack object
//!       long result = 0;
//!       C *c = create(&size);
//!       if (c == 0) {
//!           if (size < K_BOUND) result = K_LO; else result = K_HI;
//!       } else {
//!           T *t = attach(c, id, size);
//!           if (t == 0) result = K_FAIL;
//!           else { *outSize = size; *outC = c; *outT = t; }
//!       }
//!       return result;
//!   }
//! ```
//!
//! This is `src/xdk/xlrc/xlrcimpl.cpp`'s `CXLrcImpl_CreateClientWithTransport`,
//! a FRONTIER TU with exactly one emitted function — so the TU converts on this
//! class or on none.
//!
//! ## Why a TRANSCRIPTION and not a general `cflow-if-n` lowering
//!
//! The same argument [`super::alloc_init_or_fail`],
//! [`super::guard_chain_shared_tail`], [`super::if_call_join`] and
//! [`super::osf_handle_guard`] make, and it is `docs/ARCHITECTURE_SEAMS.md` §7's.
//! What ships is **thirty-eight words of one named function class, `/O1` only**,
//! `NotImplemented` outside. **Accepting this shape is not a claim about
//! `cflow-if-n` as a class** and `PORT_CFG_CLASSES` is unchanged.
//!
//! ## The five things a general lowering gets wrong
//!
//! Read off the real obj at the workload's own flags
//! (`work/w-xlr/ref/xlrcimpl/dis.txt`) and decoded token by token in
//! `work/w-xlr/PREREG.md` §1.1–§1.2, both committed **before** this file was
//! written. The emitted words are in `c2_core::codegen::xlrc_create_guard`'s
//! module doc; the facts the READER has to pin are:
//!
//! 1. **The two wide status constants SHARE ONE `lis`, hoisted above the branch
//!    that chooses between them.** `0x8007000E` and `0x800710DD` differ only
//!    below bit 16, so c2 emits `lis r26,0x8007` *before* the `cmplwi` and one
//!    `ori` in each arm — 4 words where a per-statement lowering writes 5, and
//!    every displacement after it is then wrong. So the class **refuses two
//!    constants whose high halves differ** (`k_lo >> 16 != k_hi >> 16`): that
//!    body is a different block plan and this emitter has never been graded on
//!    it. Board #1706 — anything the emitter cannot vary must be refused here.
//! 2. **The initialized local's address is taken, so it is a STACK OBJECT and
//!    not a register.** It is stored once (`stw r11,80(r1)`), its address is
//!    passed (`addi r3,r1,80`), and it is **re-loaded three times** afterwards
//!    because the callee may have written it. A parse that folded it — which is
//!    what `assign.rs` does to every local it admits — would emit `li` where c2
//!    emits `lwz` in three places and would drop the store entirely. The fact is
//!    taken **positively** from `.sy`'s flags word (`0x0021`,
//!    [`crate::func::sy::SyBlock::addr_locals`]), never from absence.
//! 3. **The two null tests read DIFFERENT condition registers, and neither is
//!    `cr6`.** The first is `mr. r31,r3` — a record-form move that tests the
//!    call's result *while copying it* and writes **cr0**, so no compare
//!    instruction is issued at all; the second is `cmplwi cr0,r3,0`, an ordinary
//!    compare that also writes cr0 because its value dies immediately. The
//!    middle guard, on the reloaded stack object, is the only one on **cr6**.
//!    A class that reached for one form throughout emits the right program with
//!    two wrong words.
//! 4. **The three trailing stores go through POINTER VALUES, not designators.**
//!    `*outSize = size` reads `B9 <outSize> <ptr> · B9 <size> <u4> · 32 <u4>`:
//!    the destination is a *loaded* pointer, so the store is `stw rS,0(rD)` with
//!    no relocation and no `addi`. Reading them as designators would be right
//!    about the program and wrong about every one of the three words.
//! 5. **There are THREE unconditional intra-section `b` words and the label lead
//!    is 2, not 3.** Board **#1761**'s rule — *"the lead is the number of
//!    unconditional intra-section `b` words"* — predicts 3 here and is
//!    **REFUTED**; what fits is `docs/LABEL_COUNTER.md` §1.1's surcharge for a
//!    first-introduced `__savegprlr_N`/`__restgprlr_N` pair, which is **+2**.
//!    See [`crate::IlFunction::label_lead`], where the correction is written
//!    beside the rule it corrects.
//!
//! ## The fence
//!
//! * **`/O1` only, asked FIRST, in the PARSER.** Board **#1638**, which has
//!   fired twice. `census_gate.rs` is the cross-check.
//! * **Exactly FOUR formals and no `this`**, because they occupy r3–r6 and are
//!   copied to r30–r27 by four pinned `mr` words.
//! * **Exactly FOUR locals**, all distinct and distinct from the formals, and
//!   **exactly ONE of them address-taken** — `.sy`'s `addr_locals` must be that
//!   one token and nothing else. A second stack object is four more bytes of
//!   frame and a different `stwu` immediate.
//! * **Two distinct callees.** A body naming one symbol twice is one undefined
//!   external in c2's table, so the symbol table is a record shorter and every
//!   index after it moves.
//! * **Every wide constant must have a non-zero high half AND a non-zero low
//!   half.** With a zero high half c2 emits a single `li`; with a zero low half
//!   it emits a single `lis`. Either is a shorter body this class has no witness
//!   of, so both are refused rather than guessed.
//! * **The two arm constants must share a high half** (fact 1).
//! * **Every label distinct.** Two aliasing labels are one block, and every
//!   displacement after the alias would be right for a program this is not.

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, is_int4_type, is_ptr_to_4, read_token_var, read_type,
    read_varint,
};
use crate::func::XlrcCreateGuard;

/// How many formals this class admits. They land in r3–r6 and are copied to
/// r30–r27 by four words the emitter does not vary.
const XLRC_FORMALS: usize = 4;

/// One TYPE as this class needs it: the two discriminating fields `read_type`
/// decodes, **and** the raw bytes.
///
/// The bytes are carried because "the same type" has to mean the same type and
/// not merely the same width. Two 4-byte pointer types with different pointee
/// ids are interchangeable to `is_ptr_to_4` and are not interchangeable to this
/// body — the create call's result and the attach call's result are both
/// `86 43 xx 20` and storing one through the other's `out` pointer is a program
/// this class does not emit.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Ty<'a> {
    tag: u8,
    kind: u8,
    bytes: &'a [u8],
}

impl Ty<'_> {
    /// A 4-byte integer of either sign.
    fn is_word(&self) -> bool {
        is_int4_type(self.tag, self.kind)
    }

    /// A 4-byte **unsigned** integer, which is the only thing that makes the
    /// middle guard a `cmplwi`.
    ///
    /// **This clause is a live wrong-emit fence, not decoration.** The
    /// relational opcodes are sign-agnostic (`docs/CODEGEN_W6_COMPARE.md` §1.1),
    /// so `int size` and `unsigned size` produce the **same `22` byte** and
    /// differ only here — and c2 emits `cmpwi cr6,r11,4` for the signed one
    /// where this class's emitter has an unconditional `cmplwi`. Without the
    /// clause the accepted body is one wrong word, in an obj that links.
    fn is_unsigned_word(&self) -> bool {
        is_int4_type(self.tag, self.kind) && (self.kind & 0x0F) == 0x2
    }

    /// A 4-byte pointer — one GPR, and therefore one `mr` or one `stw`.
    fn is_ptr4(&self) -> bool {
        is_ptr_to_4(self.tag, self.kind)
    }
}

/// Consume a TYPE at `*p`.
fn eat_ty<'a>(seg: &'a [u8], p: &mut usize, what: &'static str) -> Result<Ty<'a>, Block> {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) => {
            let bytes = &seg[*p..*p + w];
            *p += w;
            Ok(Ty { tag, kind, bytes })
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

/// `B9 <tok> <TYPE>` — a value read. Returns the token and the TYPE's bytes.
fn eat_load<'a>(seg: &'a [u8], p: &mut usize, what: &'static str) -> Result<(u32, Ty<'a>), Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    let ty = eat_ty(seg, p, what)?;
    Ok((tok, ty))
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
/// Pinned rather than merely decoded, for [`super::osf_handle_guard`]'s reason:
/// the depths are the only place the *bracing* of the source shows up in this
/// stream, and a differently braced body is a different block plan.
fn eat_close(seg: &[u8], p: &mut usize, k: u8, what: &'static str) -> Result<(), Block> {
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x54) || !eat_byte(seg, p, k) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// `33 <TYPE> <varint>` — a literal. Returns the value and the TYPE's bytes.
fn eat_lit<'a>(seg: &'a [u8], p: &mut usize, what: &'static str) -> Result<(i32, Ty<'a>), Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    let ty = eat_ty(seg, p, what)?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    Ok((k, ty))
}

/// `32 <TYPE>` then `4B` — a store and its statement end. Returns the TYPE.
fn eat_store_end<'a>(seg: &'a [u8], p: &mut usize, what: &'static str) -> Result<Ty<'a>, Block> {
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, what));
    }
    let ty = eat_ty(seg, p, what)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, what));
    }
    Ok(ty)
}

/// `BD <TYPE ret> <conv 00> <varint fn-type-id>` — the CALL token, decoded
/// exactly as [`super::calls::eat_call_token`] decodes it (cdecl only), and
/// returning the return TYPE because this class compares it against the
/// destination's.
fn eat_call_head<'a>(seg: &'a [u8], p: &mut usize, what: &'static str) -> Result<Ty<'a>, Block> {
    if !eat_byte(seg, p, 0xBD) {
        return Err(blk(seg, *p, what));
    }
    let ty = eat_ty(seg, p, what)?;
    match seg.get(*p) {
        Some(0x00) => *p += 1,
        _ => return Err(blk(seg, *p, "xlrc-call-conv")),
    }
    read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    Ok(ty)
}

/// `55 <TYPE>` — one argument's terminator. Returns the TYPE's bytes.
fn eat_arg_end<'a>(seg: &'a [u8], p: &mut usize, what: &'static str) -> Result<Ty<'a>, Block> {
    if !eat_byte(seg, p, 0x55) {
        return Err(blk(seg, *p, what));
    }
    eat_ty(seg, p, what)
}

/// `26 <t_dst> · 33 <T> k · 32 <T> · 4B` — an assignment of a literal to a
/// named destination. Returns the destination token, the literal and the store
/// TYPE.
///
/// The literal's TYPE and the store's TYPE are required **equal as bytes**, not
/// merely equal in width: a conversion between them would be a visible `2C`
/// this grammar has no slot for, so a disagreement means the walk has lost the
/// stream.
fn eat_lit_assign<'a>(
    seg: &'a [u8],
    p: &mut usize,
    what: &'static str,
) -> Result<(u32, i32, Ty<'a>), Block> {
    eat_opt_stmt_marker(seg, p);
    let dst = eat_designator(seg, p, what)?;
    let (k, lit_ty) = eat_lit(seg, p, what)?;
    let store_ty = eat_store_end(seg, p, what)?;
    if lit_ty != store_ty {
        return Err(blk(seg, *p, "xlrc-literal-type-is-not-the-store-type"));
    }
    Ok((dst, k, store_ty))
}

/// True for a wide constant this class can materialize with the pinned
/// `lis`+`ori` pair: **both halves non-zero**.
///
/// With a zero high half c2 emits one `li`; with a zero low half it emits one
/// `lis`. Either is a shorter body and a different block plan, and the class has
/// no witness of either — refused, not guessed.
fn is_two_word_constant(k: i32) -> bool {
    let u = k as u32;
    (u >> 16) != 0 && (u & 0xFFFF) != 0
}

/// **The recognizer.** `start` is the body's first statement byte — the `26` of
/// `size = K_INIT` — and `lo` is the `4C 4F 11` marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` on the first byte that is not its grammar, so a
/// body that declines still reports `try_parse_assign_body_detail`'s blocker
/// (`assign-rhs-call-0x26`, which is what `xlrcimpl.cpp` read at this lane's
/// base) and no census key moves.
pub(crate) fn try_parse_xlrc_create_guard(
    seg: &[u8],
    start: usize,
    lo: usize,
    addr_locals: &[u32],
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not in the emitter.**
    // Board **#1638**, which has fired twice. Asked FIRST, before any body byte
    // is read, so the refusal cannot depend on how far the walk got.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "xlrc-not-o1"));
    }
    // Four formals and no `this`. `parse_params` prepends the `this` token when
    // the pre-body region binds one and REFUSES when the binding is
    // undetermined, so "no `this`" is an established fact and not a count.
    let params = parse_params(seg, lo)?;
    let formals = parse_formals(seg, lo)?;
    if params.len() != XLRC_FORMALS || formals.len() != XLRC_FORMALS || params != formals {
        return Err(blk(seg, start, "xlrc-not-four-formals-free-fn"));
    }

    let mut p = start;

    // ---- `size = K_INIT;` — the ADDRESS-TAKEN stack object ------------------
    let (t_size, k_init, u4_ty) = eat_lit_assign(seg, &mut p, "xlrc-init-size")?;
    if !u4_ty.is_unsigned_word() {
        // See [`Ty::is_unsigned_word`] — the signed sibling is one wrong word.
        return Err(blk(seg, p, "xlrc-stack-object-is-not-an-unsigned-word"));
    }
    if !(0..=0xFFFF).contains(&k_init) {
        // The value lands in one `li` immediate and is compared by one
        // `cmplwi`; outside 16 bits both words change.
        return Err(blk(seg, p, "xlrc-init-wider-than-imm16"));
    }
    // **The positive `.sy` fact, and the ONLY thing that says this token is four
    // bytes of frame rather than a register.** Exactly one address-taken local,
    // and it is this one: a second stack object moves the `stwu` immediate.
    if addr_locals != [t_size] {
        return Err(blk(seg, p, "xlrc-not-exactly-one-address-taken-local"));
    }

    // ---- `result = 0;` -----------------------------------------------------
    let (t_result, k_zero, i4_ty) = eat_lit_assign(seg, &mut p, "xlrc-init-result")?;
    if k_zero != 0 {
        return Err(blk(seg, p, "xlrc-result-not-initialized-to-zero"));
    }
    if !i4_ty.is_word() {
        return Err(blk(seg, p, "xlrc-result-is-not-a-word"));
    }

    // ---- `c = create(&size);` ---------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let t_client = eat_designator(seg, &mut p, "xlrc-create-dst")?;
    let create_tok = eat_designator(seg, &mut p, "xlrc-create-callee")?;
    let ret_ty = eat_call_head(seg, &mut p, "xlrc-create-call")?;
    if !ret_ty.is_ptr4() {
        // The result is parked in r31 with one `mr.` and stored with one `stw`;
        // a wider or an aggregate return is neither.
        return Err(blk(seg, p, "xlrc-create-result-is-not-a-four-byte-pointer"));
    }
    // The one argument is the stack object's ADDRESS — a `26` designator push,
    // not a `B9` value read. That is what makes the word `addi r3,r1,80`.
    if eat_designator(seg, &mut p, "xlrc-create-arg")? != t_size {
        return Err(blk(seg, p, "xlrc-create-arg-is-not-the-stack-object"));
    }
    eat_arg_end(seg, &mut p, "xlrc-create-arg-end")?;
    if !eat_byte(seg, &mut p, 0x4C) {
        return Err(blk(seg, p, "xlrc-create-extra-argument"));
    }
    if eat_store_end(seg, &mut p, "xlrc-create-store")? != ret_ty {
        return Err(blk(seg, p, "xlrc-create-result-converted"));
    }

    // ---- `if (c == 0)` — the RECORD-FORM test on cr0 -----------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-outer-if-scope"));
    }
    let (tok, ty) = eat_load(seg, &mut p, "xlrc-outer-test-load")?;
    if tok != t_client || ty != ret_ty {
        return Err(blk(seg, p, "xlrc-outer-test-names-the-wrong-value"));
    }
    let (k, _) = eat_lit(seg, &mut p, "xlrc-outer-test-literal")?;
    if k != 0 {
        return Err(blk(seg, p, "xlrc-outer-test-not-against-null"));
    }
    if !eat_byte(seg, &mut p, 0x1F) {
        return Err(blk(seg, p, "xlrc-outer-test-relation"));
    }
    let l_else = eat_transfer(seg, &mut p, 0x38, "xlrc-outer-branch")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-outer-then-scopes"));
    }

    // ---- `if (size < K_BOUND)` — the ONLY cr6 test, on the RELOADED object --
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-inner-if-scope"));
    }
    let (tok, ty) = eat_load(seg, &mut p, "xlrc-inner-test-load")?;
    if tok != t_size || ty != u4_ty {
        return Err(blk(seg, p, "xlrc-inner-test-names-the-wrong-value"));
    }
    let (k_bound, _) = eat_lit(seg, &mut p, "xlrc-inner-test-literal")?;
    if !(0..=0xFFFF).contains(&k_bound) {
        return Err(blk(seg, p, "xlrc-bound-wider-than-uimm16"));
    }
    if !eat_byte(seg, &mut p, 0x22) {
        return Err(blk(seg, p, "xlrc-inner-test-relation"));
    }
    let l_hi = eat_transfer(seg, &mut p, 0x38, "xlrc-inner-branch")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-inner-then-scopes"));
    }

    // ---- `result = K_LO;` then the arm's jump ------------------------------
    let (dst, k_lo, ty) = eat_lit_assign(seg, &mut p, "xlrc-lo-assign")?;
    if dst != t_result || ty != i4_ty {
        return Err(blk(seg, p, "xlrc-lo-assign-names-the-wrong-value"));
    }
    eat_close(seg, &mut p, 0x08, "xlrc-lo-close-8")?;
    eat_close(seg, &mut p, 0x07, "xlrc-lo-close-7")?;
    let l_inner_join = eat_transfer(seg, &mut p, 0x3A, "xlrc-lo-jump")?;

    // ---- `else result = K_HI;` --------------------------------------------
    if eat_label(seg, &mut p, "xlrc-hi-label")? != l_hi {
        return Err(blk(seg, p, "xlrc-hi-label"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-hi-scopes"));
    }
    let (dst, k_hi, ty) = eat_lit_assign(seg, &mut p, "xlrc-hi-assign")?;
    if dst != t_result || ty != i4_ty {
        return Err(blk(seg, p, "xlrc-hi-assign-names-the-wrong-value"));
    }
    eat_close(seg, &mut p, 0x08, "xlrc-hi-close-8")?;
    eat_close(seg, &mut p, 0x07, "xlrc-hi-close-7")?;
    if eat_label(seg, &mut p, "xlrc-inner-join-label")? != l_inner_join {
        return Err(blk(seg, p, "xlrc-inner-join-label"));
    }
    eat_close(seg, &mut p, 0x06, "xlrc-inner-join-close-6")?;
    eat_close(seg, &mut p, 0x05, "xlrc-then-close-5")?;
    eat_close(seg, &mut p, 0x04, "xlrc-then-close-4")?;
    let l_end = eat_transfer(seg, &mut p, 0x3A, "xlrc-then-jump")?;

    // ---- the `else` arm: `t = attach(c, id, size);` ------------------------
    if eat_label(seg, &mut p, "xlrc-else-label")? != l_else {
        return Err(blk(seg, p, "xlrc-else-label"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-else-scopes"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    let t_transport = eat_designator(seg, &mut p, "xlrc-attach-dst")?;
    let attach_tok = eat_designator(seg, &mut p, "xlrc-attach-callee")?;
    let att_ty = eat_call_head(seg, &mut p, "xlrc-attach-call")?;
    if !att_ty.is_ptr4() {
        return Err(blk(seg, p, "xlrc-attach-result-is-not-a-four-byte-pointer"));
    }
    // **Three arguments, in REVERSE source order** — the stream pushes the last
    // one first, and the emitter's three `mr`/`lwz` words are keyed on that
    // order. `size` is read as a VALUE here (it is reloaded from the stack
    // object), `id` is the first formal and `c` is the create call's result.
    let (tok, ty) = eat_load(seg, &mut p, "xlrc-attach-arg3")?;
    if tok != t_size || ty != u4_ty {
        return Err(blk(seg, p, "xlrc-attach-arg3-is-not-the-stack-object"));
    }
    eat_arg_end(seg, &mut p, "xlrc-attach-arg3-end")?;
    let (tok, ty) = eat_load(seg, &mut p, "xlrc-attach-arg2")?;
    if tok != params[0] {
        return Err(blk(seg, p, "xlrc-attach-arg2-is-not-the-first-formal"));
    }
    // **One GPR, established by the TYPE and not assumed from the count.** The
    // first formal reaches r4 through a single `mr r4,r30`; a formal wider than
    // a register occupies two of them and every argument after it moves.
    if !(ty.is_word() || ty.is_ptr4()) {
        return Err(blk(seg, p, "xlrc-first-formal-is-not-one-register"));
    }
    eat_arg_end(seg, &mut p, "xlrc-attach-arg2-end")?;
    let (tok, ty) = eat_load(seg, &mut p, "xlrc-attach-arg1")?;
    if tok != t_client || ty != ret_ty {
        return Err(blk(seg, p, "xlrc-attach-arg1-is-not-the-create-result"));
    }
    eat_arg_end(seg, &mut p, "xlrc-attach-arg1-end")?;
    if !eat_byte(seg, &mut p, 0x4C) {
        return Err(blk(seg, p, "xlrc-attach-extra-argument"));
    }
    if eat_store_end(seg, &mut p, "xlrc-attach-store")? != att_ty {
        return Err(blk(seg, p, "xlrc-attach-result-converted"));
    }

    // ---- `if (t == 0)` — the second cr0 test, an ordinary `cmplwi` ---------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-fail-if-scope"));
    }
    let (tok, ty) = eat_load(seg, &mut p, "xlrc-fail-test-load")?;
    if tok != t_transport || ty != att_ty {
        return Err(blk(seg, p, "xlrc-fail-test-names-the-wrong-value"));
    }
    let (k, _) = eat_lit(seg, &mut p, "xlrc-fail-test-literal")?;
    if k != 0 {
        return Err(blk(seg, p, "xlrc-fail-test-not-against-null"));
    }
    if !eat_byte(seg, &mut p, 0x1F) {
        return Err(blk(seg, p, "xlrc-fail-test-relation"));
    }
    let l_ok = eat_transfer(seg, &mut p, 0x38, "xlrc-fail-branch")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-fail-scopes"));
    }
    let (dst, k_fail, ty) = eat_lit_assign(seg, &mut p, "xlrc-fail-assign")?;
    if dst != t_result || ty != i4_ty {
        return Err(blk(seg, p, "xlrc-fail-assign-names-the-wrong-value"));
    }
    eat_close(seg, &mut p, 0x08, "xlrc-fail-close-8")?;
    eat_close(seg, &mut p, 0x07, "xlrc-fail-close-7")?;
    let l_outer_join = eat_transfer(seg, &mut p, 0x3A, "xlrc-fail-jump")?;

    // ---- the success arm: three stores THROUGH POINTER VALUES --------------
    if eat_label(seg, &mut p, "xlrc-ok-label")? != l_ok {
        return Err(blk(seg, p, "xlrc-ok-label"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xlrc-ok-scopes"));
    }
    for (ix, (src, src_ty)) in
        [(t_size, u4_ty), (t_client, ret_ty), (t_transport, att_ty)].into_iter().enumerate()
    {
        eat_opt_stmt_marker(seg, &mut p);
        let (dst, dst_ty) = eat_load(seg, &mut p, "xlrc-out-store-dst")?;
        if dst != params[ix + 1] {
            return Err(blk(seg, p, "xlrc-out-store-is-not-the-matching-formal"));
        }
        // The destination is a POINTER VALUE held in a parked register, so the
        // store is `stw rS,0(rD)` — one word, no relocation, no `addi`. The
        // type is what says the formal is one register wide.
        if !dst_ty.is_ptr4() {
            return Err(blk(seg, p, "xlrc-out-store-formal-is-not-a-four-byte-pointer"));
        }
        let (tok, ty) = eat_load(seg, &mut p, "xlrc-out-store-value")?;
        if tok != src || ty != src_ty {
            return Err(blk(seg, p, "xlrc-out-store-value-is-not-the-matching-local"));
        }
        if eat_store_end(seg, &mut p, "xlrc-out-store-end")? != src_ty {
            return Err(blk(seg, p, "xlrc-out-store-converted"));
        }
    }
    eat_close(seg, &mut p, 0x08, "xlrc-ok-close-8")?;
    eat_close(seg, &mut p, 0x07, "xlrc-ok-close-7")?;
    if eat_label(seg, &mut p, "xlrc-outer-join-label")? != l_outer_join {
        return Err(blk(seg, p, "xlrc-outer-join-label"));
    }
    eat_close(seg, &mut p, 0x06, "xlrc-outer-join-close-6")?;
    eat_close(seg, &mut p, 0x05, "xlrc-else-close-5")?;
    eat_close(seg, &mut p, 0x04, "xlrc-else-close-4")?;
    if eat_label(seg, &mut p, "xlrc-end-label")? != l_end {
        return Err(blk(seg, p, "xlrc-end-label"));
    }
    eat_close(seg, &mut p, 0x03, "xlrc-end-close-3")?;

    // ---- `return result;` --------------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let (tok, ty) = eat_load(seg, &mut p, "xlrc-return-load")?;
    if tok != t_result || ty != i4_ty {
        return Err(blk(seg, p, "xlrc-return-is-not-the-status"));
    }
    if !eat_byte(seg, &mut p, 0x41) {
        return Err(blk(seg, p, "xlrc-return-operator"));
    }
    if eat_ty(seg, &mut p, "xlrc-return-type")? != i4_ty {
        return Err(blk(seg, p, "xlrc-return-type-is-not-the-status-type"));
    }
    let l_epi = eat_transfer(seg, &mut p, 0x3A, "xlrc-return-jump")?;
    eat_close(seg, &mut p, 0x02, "xlrc-wind-2")?;
    if eat_label(seg, &mut p, "xlrc-epilogue-label")? != l_epi {
        return Err(blk(seg, p, "xlrc-epilogue-label"));
    }
    // The function tail. Landing exactly on it is the whole acceptance claim: a
    // walk that ends anywhere else consumed a byte it did not understand.
    const FN_TAIL: [u8; 7] = [0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00];
    if seg.get(p..p + FN_TAIL.len()) != Some(&FN_TAIL[..]) {
        return Err(blk(seg, p, "xlrc-not-the-function-tail"));
    }

    // ---- the fences that are not a token position --------------------------
    //
    // **The shared `lis`.** Two arm constants whose high halves differ are four
    // words where this class emits three, and every displacement after the
    // hoist is then wrong.
    if (k_lo as u32) >> 16 != (k_hi as u32) >> 16 {
        return Err(blk(seg, p, "xlrc-arm-constants-do-not-share-a-lis"));
    }
    for k in [k_lo, k_hi, k_fail] {
        if !is_two_word_constant(k) {
            return Err(blk(seg, p, "xlrc-status-constant-is-not-lis-plus-ori"));
        }
    }
    // Every label distinct.
    let labels = [l_else, l_hi, l_inner_join, l_end, l_ok, l_outer_join, l_epi];
    for i in 0..labels.len() {
        for j in i + 1..labels.len() {
            if labels[i] == labels[j] {
                return Err(blk(seg, p, "xlrc-labels-alias"));
            }
        }
    }
    // Two callees must be two symbols.
    if create_tok == attach_tok {
        return Err(blk(seg, p, "xlrc-callees-alias"));
    }
    // Four locals, all distinct and none of them a formal.
    let locals = [t_size, t_result, t_client, t_transport];
    for i in 0..locals.len() {
        for j in i + 1..locals.len() {
            if locals[i] == locals[j] {
                return Err(blk(seg, p, "xlrc-locals-alias"));
            }
        }
        if params.contains(&locals[i]) {
            return Err(blk(seg, p, "xlrc-local-is-a-formal"));
        }
    }

    Ok(BodyShape::XlrcCreateGuard(XlrcCreateGuard {
        params,
        create_tok,
        attach_tok,
        k_init,
        k_bound,
        k_lo,
        k_hi,
        k_fail,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{IlFunction, XlrcCreateGuardFn};

    fn xlrcimpl() -> IlFunction {
        IlFunction {
            params: vec![0x09fc, 0x09fd, 0x09fe, 0x09ff],            body: crate::func::BodyShape::XlrcCreateGuard(XlrcCreateGuardFn {
                params: vec![0x09fc, 0x09fd, 0x09fe, 0x09ff],
                create: "?CreateClient@CXLrcImpl@@YAPAVCXLrcClient@@PAI@Z".to_string(),
                attach: "CXLrcClient_CreateTransport".to_string(),
                k_init: 4,
                k_bound: 4,
                k_lo: 0x8007_000Eu32 as i32,
                k_hi: 0x8007_10DDu32 as i32,
                k_fail: 0x8000_4005u32 as i32,
            }),
            ..IlFunction::base("CXLrcImpl_CreateClientWithTransport", &None)
        }
    }

    /// **The label lead is 2, and that is a MEASUREMENT that refutes #1761.**
    ///
    /// `xlrcimpl.cpp`'s `.gl` counter is 2575, `plan_labels` seeds at
    /// `2575 + 9 + 3·1 = 2587`, and the reference obj's labels are
    /// `$M2589`/`$M2590`/`$T2591`. The body has **three** unconditional
    /// intra-section `b` words (`+0x4c`, `+0x54`, `+0x78`), so #1761's
    /// `b`-counting rule predicts 3 — and the lane ran both counterfactuals
    /// against real `c2.dll`: forced to 0 the obj reads `mismatch`, forced to 3
    /// it reads `mismatch`, and at 2 it reads `match`.
    ///
    /// The number that fits is `docs/LABEL_COUNTER.md` §1.1's +2 for a
    /// first-introduced `__savegprlr_N`/`__restgprlr_N` pair.
    #[test]
    fn the_label_lead_is_two_and_the_gy_stride_is_seven() {
        let f = xlrcimpl();
        assert_eq!(f.label_lead(), 2);
        assert!(f.is_framed());
        assert_eq!(f.label_slots(true), Some(7), "/Gy: base 5 + a lead of 2");
        assert_eq!(f.label_slots(false), Some(6), "packed: base 4 + the same lead");
    }

    /// The two callees travel on `callees()` in `.text` order, and the frame's
    /// helper pair does NOT — it is minted by the emitter and placed after `$T`,
    /// so listing it here would put two symbols in the wrong half of the table.
    #[test]
    fn callees_are_the_two_il_named_ones_and_not_the_frame_helpers() {
        let f = xlrcimpl();
        let names: Vec<&str> = f.callees().collect();
        assert_eq!(
            names,
            vec![
                "?CreateClient@CXLrcImpl@@YAPAVCXLrcClient@@PAI@Z",
                "CXLrcClient_CreateTransport"
            ]
        );
        assert!(names.iter().all(|n| !n.starts_with("__savegprlr")
            && !n.starts_with("__restgprlr")));
    }
}

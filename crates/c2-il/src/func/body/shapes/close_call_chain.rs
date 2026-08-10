//! **W-MMIO3 — the guarded close chain**: `src/xdk/nuispeech/mmio.cpp`'s
//! `mmioClose`, the last of that TU's three blocked bodies and the last 124 of
//! its 380 bytes. `w-ifn` shipped the other two ([`super::guard_ret_chain`]) and
//! declined this one at six mechanisms; `w-decouple` re-priced it at eight.
//!
//! ```c
//!   R f(P p, U u) {
//!       if (p == 0) return K;         // pointer guard, cr6, arm in source order
//!       V r1 = g(p, L1);              // a BOUND call statement, same-TU callee
//!       if (r1 != 0) return r1;       // braceless early return on the RESULT
//!       T *t = (T *)p;                // a reinterpreting assignment — no code
//!       V r2 = t->fp(t, A1, u, A3);   // an INDIRECT call through a loaded member
//!       if (r2 != 0) return r2;       // a second braceless early return
//!       h(p, 0, 0, 0);                // ELIDED — same-TU, pure, result unused
//!       k(p);                         // a void call to an EXTERNAL
//!       return 0;
//!   }
//! ```
//!
//! ## Why a TRANSCRIPTION and not a general `cflow-if-n` lowering
//!
//! [`super::guard_ret_chain`]'s argument, `docs/ARCHITECTURE_SEAMS.md` §7's.
//! What ships is **thirty-one words of one named function class, `/O1` only**,
//! `NotImplemented` outside. **Accepting this shape is not a claim about
//! `cflow-if-n` as a class**, and `c2_harness::gap::factors::PORT_CFG_CLASSES`
//! is not widened for it.
//!
//! ## The four things this class has that its neighbour does not
//!
//! 1. **A BOUND CALL STATEMENT.** `26 <dst> · 26 <callee> BD … 4C · 2C <T> 0 ·
//!    32 <T> · 4B` — the assignment's destination push and the callee push are
//!    the same opcode one after the other, and only the `BD` after the *second*
//!    token tells them apart ([`super::super::parse_segment_shape`]'s `0x26`
//!    arm makes exactly that test).
//! 2. **A BRACELESS EARLY RETURN ON A CALL RESULT.** One `53`, one `54 04` —
//!    one scope shallower than [`super::guard_ret_chain`]'s braced arm, so it
//!    is a second grammar and not a parameter of the first. It compares with
//!    `20` (`!=`) where every guard in this seam compares with `1F`, and c2
//!    lowers it on **cr0** with the branch sense INVERTED: the arm returns a
//!    value that is already in r3, so it costs no instruction and folds into
//!    the branch to the epilogue.
//! 3. **AN INDIRECT CALL.** The callee push is not `26 <tok>` at all — it is a
//!    whole expression, `B9 <base> <PTR> · 33 <INT> <off> · 27 <T> · 30 <T>`,
//!    a member load — and the `BD` follows the loaded VALUE. Every other call
//!    production in this crate resolves a callee NAME; this one has none.
//! 4. **AN ELIDED CALL.** The statement IS in the `.ex` stream and the obj
//!    carries no branch, no relocation and no symbol for it.
//!
//! ## The two INTERPROCEDURAL facts, and why they are not asked here
//!
//! This function sees one `.ex` segment, so it can only *record* the two callee
//! tokens the facts are about. Both are asked at
//! [`crate::IlBundle::functions`], the parser's **bundle** level — which is
//! what board #139 means and is where `w-mmioclose` already put
//! [`crate::func::gl::gl_function_attrs`]:
//!
//! * **the ELISION** needs `h` to be a sibling whose own body has no side
//!   effect (`w-ifn` #2351's D2 rule, 10 cells at `/O1` and at `/Ob0`);
//! * **the r5 PARK** needs `g` to be a sibling whose own body writes only r3,
//!   so `u` survives the `bl` in the register the `bctrl` wants
//!   (`WB_CHOOSER_FINDINGS` §2.3 M-RULE + *"coalescing beats allocation"*), and
//!   needs `k` to be an EXTERNAL, so that the r31 park is forced rather than
//!   chosen.
//!
//! What this file DOES fence about them is everything local: the elided call's
//! result must be unused and its arguments must be side-effect-free, and the
//! parked formal must be the indirect call's THIRD argument (the coalescing
//! target) and nothing else.
//!
//! ## The fence
//!
//! * **`/O1` only, asked FIRST, in the PARSER** (board #1638). The materialised
//!   common epilogue is reached from four places and tail-duplicates above
//!   `/O1` on a threshold nothing here has fitted.
//! * **Exactly TWO formals and no `this`.** A different arity moves every
//!   argument register.
//! * **Exactly ONE guard**, on formal 0, against null, on a POINTER (`cmplwi`,
//!   not `cmpwi` — [`super::guard_ret_chain`]'s fact 1), returning a `simm16`.
//! * **Both early returns compare `!= 0` against the local the immediately
//!   preceding call assigned**, and return that same local. A different operand
//!   is a value that is not already in r3 and therefore a different block plan.
//! * **The indirect call's first argument is the cast local and its third is
//!   formal 1.** The third is the whole reason for the r5 park; the first is
//!   why r3 is reloaded from r31 rather than left alone.
//! * **Every label distinct, and the guard and both early returns branch to the
//!   SAME epilogue label.** Two aliasing labels are one block.
//! * **The void call's argument is formal 0 and its result type is void.**

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::guard_ret_chain::{
    eat_any_type, eat_arg_sep, eat_close, eat_guard, eat_label, eat_lit_any, eat_load,
    eat_transfer, GUARD_ENTRY_DEPTH,
};
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat_byte, eat_int_like, eat_opt_stmt_marker, is_ptr4_kind, read_token_var, read_varint,
};

/// The most arguments this class will read in one call's argument region. Not a
/// fact about c2 — a bound so a malformed stream cannot make the reader loop.
/// Every witness argument region here is 1, 2 or 4 long.
const MAX_ARGS: usize = 8;

/// One argument of a call, in **stream** order (which is the reverse of source
/// order — [`super::guard_ret_chain`]'s fact 2, and both of this body's
/// argument regions confirm it independently).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallArg {
    /// `33 <TYPE> <k>` — a literal.
    Lit(i32),
    /// `B9 <tok> <TYPE> [2C <TYPE> 0]` — a value read, with the optional
    /// reinterpreting conversion.
    Load(u32),
}

/// **The token-level form of [`crate::func::CloseCallChain`]**, produced here
/// and resolved to names by `IlBundle::shape_to_function`. Callee tokens carry
/// only a `.gl` symbol index; this file never resolves one, for the reason
/// every other production in this crate does not either.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseCallChainShape {
    pub params: Vec<u32>,
    pub guard_ret: i32,
    pub call1_tok: u32,
    pub call1_arg1: i32,
    pub fnptr_off: i32,
    pub icall_arg1: i32,
    pub icall_arg3: i32,
    pub elided_tok: u32,
    pub void_call_tok: u32,
}

/// `BD <ret TYPE> <conv> <varint fn-type-id>` — the CALL token, decoded exactly
/// as [`super::calls::eat_call_token`] decodes it, and returning whether the
/// return type is `void`.
///
/// A local copy rather than a call into `calls` because that one classifies the
/// return type as *"is it a real/FP class"* and this class needs the different
/// question *"is it void"* — the void call's `82 07 03` against the three
/// value-returning calls' `86 41 12`.
fn eat_call_token_void(seg: &[u8], p: &mut usize, what: &'static str) -> Result<bool, Block> {
    if !eat_byte(seg, p, 0xBD) {
        return Err(blk(seg, *p, what));
    }
    let (tag, kind) = eat_any_type(seg, p, what)?;
    // `82 07 xx` is the void function record's return type in every `.gl` and
    // `.ex` this seam has read; `read_type` hands back the tag and the kind and
    // the kind's low nibble is what separates them.
    let is_void = tag == 0x82 && kind == 0x07;
    // Calling convention. `00` is cdecl/stdcall and nothing else is in class —
    // `04` is fastcall and `40` is varargs, both of which need argument passing
    // this port does not implement.
    match seg.get(*p) {
        Some(0x00) => *p += 1,
        _ => return Err(blk(seg, *p, "ccc-call-conv")),
    }
    read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    Ok(is_void)
}

/// A call's argument region — `( <lit|load> 55 <TYPE> )* 4C` — in stream order.
///
/// Every argument must be a literal or a value read: this class emits `li` and
/// `mr` for its arguments and has no representation for a computed one, and the
/// ELIDED call additionally needs its arguments to be **side-effect-free** for
/// the elision to be sound at all (`w-ifn` D2's rule is about the *callee*'s
/// purity and says nothing about an argument that is itself a call).
fn eat_args(seg: &[u8], p: &mut usize, what: &'static str) -> Result<Vec<CallArg>, Block> {
    let mut out = Vec::new();
    loop {
        if eat_byte(seg, p, 0x4C) {
            return Ok(out);
        }
        if out.len() == MAX_ARGS {
            return Err(blk(seg, *p, what));
        }
        let arg = if seg.get(*p) == Some(&0x33) {
            CallArg::Lit(eat_lit_any(seg, p, what)?)
        } else {
            let (tok, _) = eat_load(seg, p, what)?;
            // The optional reinterpreting conversion. Required to carry no
            // offset: an offset here is an `addi` this class does not emit.
            if eat_byte(seg, p, 0x2C) {
                eat_any_type(seg, p, what)?;
                if read_varint(seg, p).ok_or(blk(seg, *p, what))? != 0 {
                    return Err(blk(seg, *p, "ccc-arg-convert-has-an-offset"));
                }
            }
            CallArg::Load(tok)
        };
        eat_arg_sep(seg, p, what)?;
        out.push(arg);
    }
}

/// `2C <TYPE> 0 · 32 <TYPE> · 4B` — the tail of an assignment statement whose
/// right-hand side has just been consumed. The `2C` is optional: the witness
/// carries it on all three of its assignments and a same-typed one would not.
fn eat_assign_tail(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    if eat_byte(seg, p, 0x2C) {
        eat_any_type(seg, p, what)?;
        if read_varint(seg, p).ok_or(blk(seg, *p, what))? != 0 {
            return Err(blk(seg, *p, "ccc-assign-convert-has-an-offset"));
        }
    }
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, what));
    }
    eat_any_type(seg, p, what)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, "ccc-assign-stmt-end"));
    }
    Ok(())
}

/// `26 <tok>` — an assignment's DESTINATION push, i.e. a `26` NOT followed by a
/// callee push and a `BD`. Returns the destination token.
fn eat_assign_dst(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `if (<local> != 0) return <local>;` **without braces** — one scope, one
/// close, and the arm returns the very value the compare read.
///
/// ```text
///   53  [line]  B9 <v> <T>  33 <T> 0  20  38 <Lskip>
///   53  [line]  B9 <v> <T>  [2C <INT> 0]  41 <INT>  3A <Lepi>
///   [line]  54 04  [line]  29 <Lskip>  54 03
/// ```
///
/// Returns `(Lskip, Lepi)`. The braced form — [`super::guard_ret_chain`]'s
/// `eat_guard` — has **two** opening `53` and **two** closes; this one has one
/// of each, which is the whole difference and it is a different block plan
/// rather than a shallower spelling of the same one.
fn eat_early_return_on(seg: &[u8], p: &mut usize, v: u32) -> Result<(u32, u32), Block> {
    if !eat_byte(seg, p, 0x53) {
        return Err(blk(seg, *p, "ccc-early-return-scope"));
    }
    eat_opt_stmt_marker(seg, p);
    let (t, _) = eat_load(seg, p, "ccc-early-return-operand")?;
    if t != v {
        return Err(blk(seg, *p, "ccc-early-return-operand-is-not-the-call-result"));
    }
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, "ccc-early-return-literal"));
    }
    eat_any_type(seg, p, "ccc-early-return-literal-type")?;
    if read_varint(seg, p).ok_or(blk(seg, *p, "ccc-early-return-literal-value"))? != 0 {
        return Err(blk(seg, *p, "ccc-early-return-not-against-zero"));
    }
    // `20` is `!=`. `1F` — the `==` every guard in this seam uses — is a
    // different branch sense and a different block order, so it is refused here
    // rather than carried as a field.
    if !eat_byte(seg, p, 0x20) {
        return Err(blk(seg, *p, "ccc-early-return-not-cmp-ne"));
    }
    let skip = eat_transfer(seg, p, 0x38, "ccc-early-return-branch")?;

    if !eat_byte(seg, p, 0x53) {
        return Err(blk(seg, *p, "ccc-early-return-arm-scope"));
    }
    eat_opt_stmt_marker(seg, p);
    let (t2, _) = eat_load(seg, p, "ccc-early-return-arm-operand")?;
    if t2 != v {
        return Err(blk(seg, *p, "ccc-early-return-arm-returns-another-value"));
    }
    if eat_byte(seg, p, 0x2C) {
        eat_any_type(seg, p, "ccc-early-return-arm-convert")?;
        if read_varint(seg, p).ok_or(blk(seg, *p, "ccc-early-return-arm-convert-value"))? != 0 {
            return Err(blk(seg, *p, "ccc-early-return-arm-convert-has-an-offset"));
        }
    }
    if !eat_byte(seg, p, 0x41) || !eat_int_like(seg, p) {
        return Err(blk(seg, *p, "ccc-early-return-arm-result-type"));
    }
    let epi = eat_transfer(seg, p, 0x3A, "ccc-early-return-arm-jump")?;

    eat_close(seg, p, GUARD_ENTRY_DEPTH + 1, "ccc-early-return-close-arm")?;
    eat_opt_stmt_marker(seg, p);
    let skip2 = eat_label(seg, p, "ccc-early-return-label")?;
    if skip2 != skip {
        return Err(blk(seg, *p, "ccc-early-return-label-is-not-the-branch-target"));
    }
    eat_close(seg, p, GUARD_ENTRY_DEPTH, "ccc-early-return-close")?;
    Ok((skip, epi))
}

/// **The production.** Returns `Err` on the first byte that is not its grammar,
/// on its own cursor, so a body that declines still reports its dispatch arm's
/// blocker and no census key moves.
pub(crate) fn try_parse_close_call_chain(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** — board #1638, asked FIRST,
    // before any body byte is read, so the refusal cannot depend on how far the
    // walk got.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "ccc-not-o1"));
    }
    let params = parse_params(seg, lo)?;
    let formals = parse_formals(seg, lo)?;
    if params.len() != 2 || formals.len() != 2 || params[0] != formals[0] {
        return Err(blk(seg, start, "ccc-not-two-formals-free-fn"));
    }
    let ix = |t: u32| params.iter().position(|&x| x == t);

    let mut p = start;

    // ---- the one guard -------------------------------------------------------
    let g = eat_guard(seg, &mut p)?;
    if ix(g.tok) != Some(0) {
        return Err(blk(seg, p, "ccc-guard-is-not-formal-0"));
    }

    // ---- `V r1 = g(p, L1);` — the bound call statement ------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let r1 = eat_assign_dst(seg, &mut p, "ccc-call1-dst")?;
    if ix(r1).is_some() {
        return Err(blk(seg, p, "ccc-call1-assigns-a-formal"));
    }
    let call1_tok = eat_assign_dst(seg, &mut p, "ccc-call1-callee")?;
    if eat_call_token_void(seg, &mut p, "ccc-call1-token")? {
        return Err(blk(seg, p, "ccc-call1-returns-void"));
    }
    let a1 = eat_args(seg, &mut p, "ccc-call1-args")?;
    // Stream order is the REVERSE of source order, so this is `g(params[0], L1)`.
    let [CallArg::Lit(call1_arg1), CallArg::Load(a1p0)] = a1[..] else {
        return Err(blk(seg, p, "ccc-call1-args-are-not-(literal, formal)"));
    };
    if ix(a1p0) != Some(0) {
        return Err(blk(seg, p, "ccc-call1-first-argument-is-not-formal-0"));
    }
    if !(-0x8000..=0x7FFF).contains(&call1_arg1) {
        return Err(blk(seg, p, "ccc-call1-literal-wider-than-simm16"));
    }
    eat_assign_tail(seg, &mut p, "ccc-call1-assign")?;

    // ---- `if (r1 != 0) return r1;` -------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let (skip1, epi1) = eat_early_return_on(seg, &mut p, r1)?;

    // ---- `T *t = (T *)p;` ----------------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let t = eat_assign_dst(seg, &mut p, "ccc-cast-dst")?;
    if ix(t).is_some() || t == r1 {
        return Err(blk(seg, p, "ccc-cast-destination-is-not-a-fresh-local"));
    }
    let (tsrc, tsrc_ptr) = eat_load(seg, &mut p, "ccc-cast-source")?;
    if !tsrc_ptr || ix(tsrc) != Some(0) {
        return Err(blk(seg, p, "ccc-cast-source-is-not-pointer-formal-0"));
    }
    eat_assign_tail(seg, &mut p, "ccc-cast-assign")?;

    // ---- `V r2 = t->fp(t, A1, u, A3);` — the INDIRECT call --------------------
    eat_opt_stmt_marker(seg, &mut p);
    let r2 = eat_assign_dst(seg, &mut p, "ccc-icall-dst")?;
    if ix(r2).is_some() || r2 == r1 || r2 == t {
        return Err(blk(seg, p, "ccc-icall-destination-is-not-a-fresh-local"));
    }
    // The callee is an EXPRESSION, not a `26 <tok>` push: `B9 <t> <PTR> ·
    // 33 <INT> <off> · 27 <T> · 30 <T>`, a member load whose VALUE the `BD`
    // then calls.
    let (ibase, ibase_ptr) = eat_load(seg, &mut p, "ccc-icall-base")?;
    if !ibase_ptr || ibase != t {
        return Err(blk(seg, p, "ccc-icall-base-is-not-the-cast-local"));
    }
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "ccc-icall-member-offset"));
    }
    let fnptr_off = read_varint(seg, &mut p).ok_or(blk(seg, p, "ccc-icall-member-offset-value"))?;
    if !(0..=0x7FFF).contains(&fnptr_off) {
        return Err(blk(seg, p, "ccc-icall-member-offset-out-of-range"));
    }
    if !eat_byte(seg, &mut p, 0x27) {
        return Err(blk(seg, p, "ccc-icall-not-a-member-offset-add"));
    }
    eat_any_type(seg, &mut p, "ccc-icall-member-type")?;
    if !eat_byte(seg, &mut p, 0x30) {
        return Err(blk(seg, p, "ccc-icall-member-is-not-loaded"));
    }
    eat_any_type(seg, &mut p, "ccc-icall-fnptr-type")?;
    if eat_call_token_void(seg, &mut p, "ccc-icall-token")? {
        return Err(blk(seg, p, "ccc-icall-returns-void"));
    }
    let a2 = eat_args(seg, &mut p, "ccc-icall-args")?;
    // Stream order reversed: `t->fp(t, A1, params[1], A3)`.
    let [CallArg::Lit(icall_arg3), CallArg::Load(a2u), CallArg::Lit(icall_arg1), CallArg::Load(a2t)] =
        a2[..]
    else {
        return Err(blk(seg, p, "ccc-icall-args-are-not-(lit, formal, lit, base)"));
    };
    if a2t != t {
        return Err(blk(seg, p, "ccc-icall-first-argument-is-not-the-cast-local"));
    }
    // **The whole reason for the r5 park.** `params[1]` is live from the
    // prologue to here, across exactly one call, and r5 is the register THIS
    // argument position wants — `WB_CHOOSER_FINDINGS` §2.3's *"coalescing beats
    // allocation"*. Any other position is a different plan.
    if ix(a2u) != Some(1) {
        return Err(blk(seg, p, "ccc-icall-third-argument-is-not-formal-1"));
    }
    if !(-0x8000..=0x7FFF).contains(&icall_arg1) || !(-0x8000..=0x7FFF).contains(&icall_arg3) {
        return Err(blk(seg, p, "ccc-icall-literal-wider-than-simm16"));
    }
    eat_assign_tail(seg, &mut p, "ccc-icall-assign")?;

    // ---- `if (r2 != 0) return r2;` -------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let (skip2, epi2) = eat_early_return_on(seg, &mut p, r2)?;

    // ---- `h(p, 0, 0, 0);` — the ELIDED call -----------------------------------
    //
    // A BARE call statement: no `26 <dst>` in front and no `32` assignment
    // tail behind, so its result is unused, which is clause 1 of `w-ifn` D2's
    // rule. Clause 2 — the callee is a sibling with a side-effect-free body —
    // is a fact about another `.ex` segment and is asked at
    // `IlBundle::functions`. The arguments are required to be literals or bare
    // formal loads by `eat_args`, which is this file's own contribution to the
    // soundness of dropping the statement: D2's cells vary the CALLEE and say
    // nothing about an argument that is itself a call.
    eat_opt_stmt_marker(seg, &mut p);
    let elided_tok = eat_assign_dst(seg, &mut p, "ccc-elided-callee")?;
    if eat_call_token_void(seg, &mut p, "ccc-elided-token")? {
        return Err(blk(seg, p, "ccc-elided-call-returns-void"));
    }
    let ae = eat_args(seg, &mut p, "ccc-elided-args")?;
    if !matches!(ae.last(), Some(CallArg::Load(x)) if ix(*x) == Some(0)) {
        return Err(blk(seg, p, "ccc-elided-call-is-not-on-formal-0"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "ccc-elided-call-is-not-a-bare-statement"));
    }

    // ---- `k(p);` — the void call to an external -------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let void_call_tok = eat_assign_dst(seg, &mut p, "ccc-void-callee")?;
    if !eat_call_token_void(seg, &mut p, "ccc-void-token")? {
        return Err(blk(seg, p, "ccc-void-call-does-not-return-void"));
    }
    let av = eat_args(seg, &mut p, "ccc-void-args")?;
    let [CallArg::Load(avp0)] = av[..] else {
        return Err(blk(seg, p, "ccc-void-call-is-not-one-argument"));
    };
    if ix(avp0) != Some(0) {
        return Err(blk(seg, p, "ccc-void-call-argument-is-not-formal-0"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "ccc-void-call-is-not-a-bare-statement"));
    }

    // ---- `return 0;` and the plumbing ----------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "ccc-tail-literal"));
    }
    if read_varint(seg, &mut p).ok_or(blk(seg, p, "ccc-tail-literal-value"))? != 0 {
        return Err(blk(seg, p, "ccc-tail-is-not-return-zero"));
    }
    super::super::expr::eat_return_plumbing(seg, &mut p, true, GUARD_ENTRY_DEPTH as usize - 1)?;

    // ---- every label distinct, and ONE epilogue -------------------------------
    if epi1 != g.epi || epi2 != g.epi {
        return Err(blk(seg, p, "ccc-arms-branch-to-different-epilogues"));
    }
    let labels = [g.skip, skip1, skip2, g.epi];
    for (i, a) in labels.iter().enumerate() {
        if labels[i + 1..].contains(a) {
            return Err(blk(seg, p, "ccc-labels-alias"));
        }
    }
    // Two calls that are the same function are one relocation target and this
    // class has no witness for it; the elided one aliasing either would make
    // the elision decide a call the obj keeps.
    if elided_tok == call1_tok || elided_tok == void_call_tok || call1_tok == void_call_tok {
        return Err(blk(seg, p, "ccc-callees-alias"));
    }

    Ok(BodyShape::CloseCallChain(CloseCallChainShape {
        params,
        guard_ret: g.ret,
        call1_tok,
        call1_arg1,
        fnptr_off,
        icall_arg1,
        icall_arg3,
        elided_tok,
        void_call_tok,
    }))
}

/// Suppresses the unused-import warning on [`is_ptr4_kind`], which this file
/// reaches only through [`eat_load`]'s boolean. Kept as an explicit no-op
/// rather than by dropping the import, because the pointer-ness of the guard
/// operand and of the cast source is a MEASURED fact of this class
/// ([`super::guard_ret_chain`]'s fact 1: `cmplwi` against `cmpwi`) and the next
/// person to widen this file needs the helper in front of them.
#[allow(dead_code)]
fn ptr4(tag: u8, kind: u8) -> bool {
    is_ptr4_kind(tag, kind)
}

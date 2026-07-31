//! **WCB/WCR — `return a->m() <rel> b->n();`**: two member calls in one
//! expression, compared. The first call's result is live across the second `bl`,
//! so the body is **Class B** — 1 or 2 saved GPRs, `std`/`ld` inline.
//!
//! ```text
//!   26 <m1> B9 <r1> <ptr4> [2C…] 99 <ptr4> 00  BD <ret> 00 <id>  4C
//!   26 <m2> B9 <r2> <ptr4> [2C…] 99 <ptr4> 00  BD <ret> 00 <id>  4C
//!   1F | 22 | 24            the `==`, `<` or `>` operator
//!   [ 2C <int4> 00 ]        an `int`-typed result converts; a `bool` one does not
//!   41 <TYPE> …             returned
//! ```
//!
//! ## Why this is a rung and `calls-2plus` did not say so
//!
//! `docs/CMP_PRODUCES_A_VALUE.md` built the comparison half of this row, measured
//! it at **+0**, and reverted — because the row is not `p->m() <rel> k`, it is a
//! **comparator**, and 89.8 % of it has two calls with the first result live
//! across the second. That is this file. WCB's `==` arm unblocked
//! `expr-call-in-expr-recv-load-then-cmp-eq-and-type-int1-whole2`, **6,001
//! functions on the 878-TU workload**; WCR's `>`/`<` arms add **67**, of which
//! **66 compare POINTERS** (see [`operand_signedness`]).
//!
//! ## The four things that are measured here and are not guessable
//!
//! **1. c2 chooses the call order, and it is neither source order nor
//! evaluation order.** The two calls are emitted in the order c1xx **numbered
//! their receivers** — see [`alloc_rank`], which is a parameter-position rule and
//! deliberately not a token-value one. The source's left operand may be either
//! one. Twelve
//! grid cells fix it (`work/WCB/probe/p5.cpp`, every ordered pair of three
//! pointer formals in both source orders, with and without a leading `int`
//! formal), and the refuter that separates "ascending token" from "ascending
//! parameter index" is a **member function**:
//!
//! ```text
//!   bool V::e1(const V* a) const { return m() == a->m(); }
//!     mr r31,r3 ; mr r3,r4 ; bl ?m ; mr r30,r3 ; mr r3,r31 ; bl ?m ; …
//! ```
//!
//! `this` is r3 and parameter index 0, yet `a`'s call goes **first** — because
//! c1xx numbers `this` *after* the declared formals. A model that ordered by
//! parameter index (which is what [`super::super::chain::leaves_ascending`] does,
//! correctly, for its own operands) emits both moves backwards here. So the rank
//! is computed with `this` moved to the end, and the source's left/right roles
//! are carried separately, in [`crate::func::SeqTail::Cmp`]'s `lhs_first`. Under
//! an order relation a wrong call order costs **four** words, not two: it swaps
//! the two `mr`s *and* the spine's two operand-carrying instructions.
//!
//! **2. The spines are a register-register family the port had never emitted.**
//! `docs/CMP_PRODUCES_A_VALUE.md` reading 4: where the literal comparison is
//! `addi r11,a,-k`, these are `subf`/`subfc` over two registers. All three live
//! in `c2_core::codegen::leaf::compare::cmp_of_two_call_results`, beside the
//! *leaf* spines whose temp-allocation rule they share, rather than beside the
//! call sequence whose frame they share.
//!
//! **3. The `bool` result is NOT a different spine for any relation this file
//! admits.** Reading 1 of that document warns that a `bool` result changes the
//! bytes — it does, and over two call results it is signed `>=` and `<=`, which
//! grow by `clrlwi r3,t,24` and an extra temp. `==`, `!=`, `>` and `<` are
//! byte-identical in `int`, `bool` and `unsigned` in all four modes
//! (`scripts/gt_cmp_rr.py`). The two that are not are the two refused by name.
//!
//! **4. Signedness is not in the opcode.** `22` is both `<`s.
//! [`operand_signedness`] is the only place it is read, from the two calls'
//! result TYPEs, and a 4-byte **pointer** takes the unsigned spine byte for byte.
//!
//! ## What is refused, each with its own census key
//!
//! * `!=`, `<=` and `>=` (`mcall-cmp-rel-{ne,le,ge}`) — **0** functions each on
//!   the workload in this shape, measured with these per-relation keys. `!=` is a
//!   different three words; `>=`/`<=` are the four/five-word sign-sum spines
//!   whose length moves with the result type;
//! * an operand outside the 4-byte integer and pointer classes
//!   (`mcall-cmp-rel-operand-type-<type>`) — **693** functions, and every one of
//!   them is `86 45`, a **`float`**. That is not a spine away: `p->mf() >
//!   q->mf()` is `fcmpu` plus a **conditional branch**, with `f31` saved by
//!   `stfd`/`lfd` beside the GPRs. Basic blocks and an FP callee-saved model,
//!   not a comparison widening;
//! * either call taking an explicit argument (`mcall-cmp-args`) — the
//!   marshalling would interleave with the callee-saved move, and which of the
//!   two is hoisted is a rule `shapes/calls.rs`'s `plan_saved_gprs` refuses to
//!   guess;
//! * a receiver that is not one of this function's own formals
//!   (`call-arg-nonformal`, the shared key) — a global or a local is a load, not
//!   a register move;
//! * more than eight formals (`framed-arg-over-eight-formals`, the shared key).

use crate::func::body::expr::eat_return_plumbing;
use crate::func::body::{Block, BodyShape, SeqCall, SeqTail};
use crate::func::readers::{
    eat_byte, eat_int_like, eat_value_type, is_int4_type, is_ptr4_kind, read_type,
    ValueClass,
};
use crate::func::{IlOp, Rel, SeqCmp};

use super::calls::{
    arg_loads_are_formals, eat_call_args, eat_call_token, eat_callee_push, plan_saved_gprs,
    MAX_REGISTER_FORMALS,
};
use super::mcall_tail::eat_receiver_this;
use super::params::parse_params;
use super::this_binding::{parse_this_token, ThisBinding};

/// The relational operand-stream opcode. Spelled here as a [`Rel`] round-trip rather
/// than as a bare byte so the one table in [`Rel::from_opcode`] stays the only
/// place a relational byte is named.
fn eat_relation(seg: &[u8], p: &mut usize) -> Option<Rel> {
    let rel = Rel::from_opcode(*seg.get(*p)?)?;
    *p += 1;
    Some(rel)
}

/// Parse the second member call and the relation that consumes both results.
///
/// Entered from [`super::mcall_tail::try_parse_member_tail_call`] with the first
/// call already decoded and the cursor at the byte after its `4C`. `Err(None)`
/// keeps the non-committal contract: the caller falls through and the body keeps
/// its de-conflated `expr-call-in-expr-recv-…` census key.
pub(crate) fn try_parse_member_cmp_calls(
    seg: &[u8],
    at: usize,
    lo: usize,
    depth: usize,
    first_args: &[Vec<IlOp>],
    callee1: u32,
    recv1: u32,
    ret1_at: usize,
) -> Result<BodyShape, Option<Block>> {
    let mut p = at;
    // The second member call, read through the SAME locators the tail form uses.
    // A second copy of the receiver decode is the drift `docs/GAPS.md` §6
    // instance #9 records — in particular the `volatile` gate, which lives in
    // `eat_receiver_this`'s operand-type read and nowhere else.
    let callee2 = eat_callee_push(seg, &mut p).map_err(|_| None)?;
    let recv2 = eat_receiver_this(seg, &mut p).map_err(|_| None)?;
    // The result TYPE of each call, **peeked** (the byte position is `BD`, which
    // `eat_call_token` consumes next). Only the order relations read it; see
    // [`operand_signedness`].
    let signedness = operand_signedness(seg, ret1_at, p);
    let ret2 = eat_call_token(seg, &mut p).map_err(|_| None)?;
    let args2 = eat_call_args(seg, &mut p).map_err(|_| None)?;

    // The relation. Not this production unless a relational byte stands here —
    // an arithmetic post-op (`02`/`03`) over two calls is a *different* spine
    // (`add r3,r30,r3`, one word) and belongs to whichever rung takes it.
    let Some(rel) = eat_relation(seg, &mut p) else {
        return Err(None);
    };
    // **The result's class, and the annotation that has to restate it.** The
    // comparison produces a `bool`; an `int`/`unsigned`-typed result converts it
    // back with `2C <int4> 00` and annotates `41 <int4>`, a `bool` one carries
    // neither. The *bytes are the same* either way — `work/WCB/probe/p7.cpp`'s
    // `c3` (`int`) and `p3.cpp`'s `a1` (`bool`) are the same 76 bytes — so the
    // class is consumed and discarded rather than carried into the shape.
    //
    // It is still **required to agree**, for the reason
    // `parse_segment_shape`'s own one-byte-unsigned arm gives: a `bool` value
    // annotated `int` is the `rlwinm` mask c2 emits for a widening, and
    // admitting it as a no-op would be wrong bytes rather than a gap. The two
    // arms mirror that site's, including which one calls
    // [`eat_return_plumbing`] with `has_result_type` — the shared `41` gate is
    // `eat_int_like_or_ptr4` and is deliberately not widened to one-byte types.
    let converted = seg.get(p) == Some(&0x2C);
    if converted {
        let mut q = p + 1;
        if !(eat_int_like(seg, &mut q) && eat_byte(seg, &mut q, 0x00)) {
            return Err(None);
        }
        p = q;
    }
    // The body must END here: the comparison's value is what the function
    // returns. Anything else (a branch on it — `-and-branch-more`, 9,490
    // functions of this family) needs basic blocks.
    if converted {
        eat_return_plumbing(seg, &mut p, true, depth).map_err(|_| None)?;
    } else {
        if !(eat_byte(seg, &mut p, 0x41) && eat_value_type(seg, &mut p, ValueClass::Int1u)) {
            return Err(None);
        }
        eat_return_plumbing(seg, &mut p, false, depth).map_err(|_| None)?;
    }

    // ---- from here the body parses to the end of the segment, so every refusal
    // is a codegen-class one over a COMPLETE body and gets its own key
    // (`docs/GAPS.md` §6: give a new gate a key on the way in). ----

    // **`==`, `>` and `<`, and nothing else.** The three refused relations have
    // **0** functions each on the 878-TU workload in this shape, measured with
    // this key split per relation, and two of them are the ones
    // `docs/CMP_PRODUCES_A_VALUE.md` reading 1 warns about: signed `>=` and `<=`
    // over two call results are **two words longer with a `bool` result** than
    // with an `int` one, so a single spine would be wrong-length — with
    // `.pdata FuncLen` and both `$M` values wrong to match — on exactly the
    // source spelling (`a->m() >= b->n()` returns `bool`) a rung would reach
    // first. `>`, `<`, `==` and `!=` are byte-identical across `int`, `bool` and
    // `unsigned` in all four modes (`scripts/gt_cmp_rr.py`); `!=` is refused
    // anyway, because a production with no workload witness is graded by its own
    // fixtures alone and that is the trade `docs/CMP_PRODUCES_A_VALUE.md` was
    // declined for.
    let cmp = match rel {
        Rel::Eq => SeqCmp::Eq,
        Rel::Gt | Rel::Lt => {
            // The order spines are the ONLY place the operand signedness is read,
            // and there is no fallback: `22` is both `<`s.
            let Some(signed) = signedness else {
                return Err(Some(crate::func::body::blk_type(
                    seg,
                    ret1_at + 1,
                    p,
                    "mcall-cmp-rel-operand-type",
                )));
            };
            SeqCmp::Order { greater: rel == Rel::Gt, signed }
        }
        _ => return Err(Some(Block { ctx: rel_refusal_key(rel), byte: None, off: p, aux: 0 })),
    };
    // A `float`/`double` result would oblige the TU to carry `_fltused`, which
    // is the `call-ret-fp` refusal one production out — asked through the shared
    // [`CallRet`] so this position cannot drift from the others.
    ret2.discarded(p).map_err(Some)?;
    // Both calls must be NULLARY. `this` is the only argument either one carries;
    // an explicit argument marshals into r4… beside the callee-saved move, and
    // which of the two is hoisted is exactly what `plan_saved_gprs` refuses to
    // guess (`callseq-saved-with-first-call-setup`, 11 of 17 probes wrong for the
    // model that assumed it).
    if first_args.len() != 1 || !args2.is_empty() {
        return Err(Some(Block { ctx: "mcall-cmp-args", byte: None, off: p, aux: 0 }));
    }

    let params = parse_params(seg, lo).map_err(Some)?;
    // Both receivers must be this function's own formals — the emission is a
    // register move, and a global or a local would be a load. `this` is in the
    // list at index 0 (`params::parse_params`), so a member function's implicit
    // receiver passes here exactly like a declared one.
    let recv_ops = [IlOp::Load(recv1), IlOp::Load(recv2)];
    if !arg_loads_are_formals(&recv_ops, &params) {
        return Err(Some(Block { ctx: "call-arg-nonformal", byte: None, off: p, aux: 0 }));
    }
    // Past the eighth formal a parameter is stack-homed and reading it is
    // `lwz r3,<slot>(r1)`. The refusal is on the whole formals LIST, not on a
    // receiver's index, because that is the predicate `select_text` raises —
    // the same reasoning and the same key as every other framed shape.
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Some(Block {
            ctx: "framed-arg-over-eight-formals",
            byte: None,
            off: p,
            aux: 0,
        }));
    }

    // **The call order.** c2 emits the two calls in the order c1xx **numbered
    // their receivers**, and that is a fact about the parameter list, not about
    // the source: see [`alloc_rank`] and the module header's grid. A tie (the
    // same receiver twice, `p->m() == p->n()`) keeps the IL order.
    let member = match parse_this_token(seg, lo) {
        Some(ThisBinding::Absent) => false,
        Some(ThisBinding::Bound(_)) => true,
        // `parse_params` above already refused an undetermined binding.
        None => return Err(Some(Block { ctx: "this-undetermined", byte: None, off: p, aux: 0 })),
    };
    let (Some(r1), Some(r2)) = (
        alloc_rank(&params, member, recv1),
        alloc_rank(&params, member, recv2),
    ) else {
        // Unreachable: `arg_loads_are_formals` just proved both are in `params`.
        return Err(Some(Block { ctx: "mcall-cmp-recv-rank", byte: None, off: p, aux: 0 }));
    };
    let lhs_first = r1 <= r2;
    let (c0, c1) = if lhs_first {
        ((callee1, recv1), (callee2, recv2))
    } else {
        ((callee2, recv2), (callee1, recv1))
    };
    let calls = vec![
        SeqCall { callee_tok: c0.0, arg_ops: vec![IlOp::Load(c0.1)], arg_sources: None },
        SeqCall { callee_tok: c1.0, arg_ops: vec![IlOp::Load(c1.1)], arg_sources: None },
    ];
    // The saved FORMALS, through the one locator the Class A/B statement
    // sequence already uses — the second call's receiver is read after the first
    // `bl`, so it is exactly what that rule returns. `extra_saved = 1` is the
    // first call's *result*, which is live across the second `bl` and takes the
    // next descending register; it is passed in so the `MAX_INLINE_SAVED_GPRS`
    // gate is applied to the TOTAL and a body that would need `__savegprlr_29`
    // refuses there rather than mis-emitting a Class C prologue.
    let saved = plan_saved_gprs(&params, &calls, 1, p).map_err(Some)?;
    Ok(BodyShape::CallSeq { params, calls, tail: SeqTail::Cmp { cmp, lhs_first }, saved })
}

/// **The compared operands' class and signedness**, from the two calls' result
/// TYPEs — `Some(true)` for a signed order comparison, `Some(false)` for an
/// unsigned one, and `None` when the class is outside what the two order spines
/// cover or the two operands disagree.
///
/// The relational **opcode does not carry it**: signed and unsigned `<` are both
/// `22` ([`Rel`]'s own doc comment records this), and the two lower to different
/// spines — five words against three. It has to come from the operand type, and
/// this is the only place it is read.
///
/// ## Which classes, and why the pointer one is here
///
/// | class | order comparison |
/// |---|---|
/// | `86 41 …` signed 4-byte integer | the five-word `eqv`/`addze` spine |
/// | `86 42 …` unsigned 4-byte integer | the three-word `subfe` spine |
/// | `86 43 …` / `86 44 …` 4-byte pointer ([`is_ptr4_kind`]) | **the same three words, byte for byte** |
/// | anything else | `None` |
///
/// The pointer row is measured, not deduced: `bool f(const U* p, const U* q)
/// { return p->mp() < q->mp(); }` is `subc r11,r30,r3 ; subfe r11,r11,r11 ;
/// clrlwi r3,r11,31`, the same three words `p->um() < q->un()` emits, and it is
/// **66 of this rung's 67 realized functions** on the 878-TU workload. Every
/// pointee width lands on the same `86 43` in a `BD` result position — `char*`,
/// `void*`, `double*`, `long long*` and a struct pointer were captured together
/// — which is why the shared [`is_ptr4_kind`] predicate is the right one and a
/// literal triple would not be.
///
/// ## Both operands are read, and they must agree in CLASS as well as sign
///
/// A mixed comparison cannot reach here — c1xx inserts an explicit `2C` convert
/// on whichever side needs one, which lands either between the first `4C` and
/// the second `26` or between the second `4C` and the relation, and both refuse
/// in the grammar (measured in both directions, for `int`/`unsigned` and for
/// `void*`/`const char*`). That makes this a **second lock** on a decode the
/// spine's correctness depends on, and it costs one comparison. Comparing the
/// *class* rather than just the resolved sign bit is the same reasoning one step
/// finer: a pointer and an `unsigned` both answer "unsigned", so a sign-only
/// check would silently admit a pair the grammar has never been observed to
/// produce.
///
/// Both positions are **peeked** rather than consumed. `p1`/`p2` point at each
/// call's `BD` token, whose next field is the result TYPE;
/// [`super::calls::eat_call_token`] resolves that same TYPE to
/// [`super::calls::CallRet`] (real / not real) and is left doing exactly that,
/// because widening a five-call-site shared locator to carry a second fact is
/// what `ROADMAP.md` §6d prices.
fn operand_signedness(seg: &[u8], p1: usize, p2: usize) -> Option<bool> {
    /// The operand classes an order comparison over two call results may take.
    /// `Ptr` is deliberately its own variant rather than a spelling of
    /// `Int { signed: false }`: the two share a spine and not a type.
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum Operand {
        Int { signed: bool },
        Ptr,
    }
    let one = |p: usize| -> Option<Operand> {
        if seg.get(p) != Some(&0xBD) {
            return None;
        }
        let (tag, kind, _, _) = read_type(seg, p + 1)?;
        // `86 41 …` is the signed 4-byte class and `86 42 …` the unsigned one
        // (`docs/IL_TYPE_TAGS.md` §2). `is_int4_type` is the shared predicate for
        // "either of those, in any cv-qualification"; the low nibble then names
        // which, and asking it only after that predicate has passed is what keeps
        // a narrow, a `bool` or a floating type out of the arithmetic.
        if is_int4_type(tag, kind) {
            return Some(Operand::Int { signed: kind & 0x0F == 0x1 });
        }
        is_ptr4_kind(tag, kind).then_some(Operand::Ptr)
    };
    let (a, b) = (one(p1)?, one(p2)?);
    if a != b {
        return None;
    }
    Some(matches!(a, Operand::Int { signed: true }))
}

/// The refusal key for a relation this production does not emit, **carrying the
/// relation**.
///
/// A single `mcall-cmp-rel` key measured 760 and said nothing about which of the
/// five relations they were, so the row could only be ranked as a block. Splitting
/// it is the "instrument the production" move ROADMAP §6n records as one of the
/// two estimating methods that work — and it is free, because the census walk is
/// already here.
fn rel_refusal_key(rel: Rel) -> &'static str {
    match rel {
        // Unreachable: the caller admits `==`.
        Rel::Eq => "mcall-cmp-rel-eq",
        Rel::Ne => "mcall-cmp-rel-ne",
        Rel::Le => "mcall-cmp-rel-le",
        Rel::Lt => "mcall-cmp-rel-lt",
        Rel::Ge => "mcall-cmp-rel-ge",
        Rel::Gt => "mcall-cmp-rel-gt",
    }
}

/// **Where c1xx numbered this receiver**, as a sort key over `params` — which is
/// what decides which of the two calls c2 emits first.
///
/// For a free function it is the parameter index. For a **member** function it is
/// not: `parse_params` puts `this` at index 0 because that is the register it
/// occupies, and c1xx numbers it **after** every declared formal. Both facts are
/// true and they are different orderings, which is `docs/GAPS.md` §6's recurring
/// shape — one field carrying two facts — with the two facts being "which
/// register" and "which symbol number".
///
/// ```text
///   bool H::q(const U* a) const { return m() == a->m(); }
///     mr r31,r3 ; mr r3,r4 ; bl ?m@U ; mr r30,r3 ; mr r3,r31 ; bl ?m@H
/// ```
///
/// `this` is r3 and `params[0]`, and `a`'s call still goes first.
///
/// **This is deliberately a parameter-position rule and not a token-value one.**
/// The first version of it compared the tokens [`crate::func::readers::read_token_var`]
/// returns, which was wrong twice over. It is not monotone — the two-byte form is
/// little-endian, so consecutive tokens `0x09FF, 0x0A00` come back as `0xFF09,
/// 0x000A` and the order inverts at every low-byte wrap; that was a live
/// `Port=Mismatch @ offset 8`, and the sweep axis added for it reproduces exactly
/// two failing alignments in every 264. And refusing the four-byte form, whose
/// layout has no captured witness, cost **5,971 of this rung's 6,000 functions**:
/// a real translation unit declares tens of thousands of symbols, so its tokens
/// are essentially all wide. The allocation order is knowable without decoding
/// either form.
fn alloc_rank(params: &[u32], member: bool, tok: u32) -> Option<usize> {
    let ix = params.iter().position(|&t| t == tok)?;
    Some(match (member, ix) {
        (false, i) => i,
        // `this`: numbered after every declared formal.
        (true, 0) => params.len(),
        (true, i) => i - 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail, SeqTail};
    use crate::func::test_fixtures::NO_LOCALS;

    /// `bool t1(const U* p, const U* q) { return p->m() == q->n(); }` — the base
    /// shape, transcribed verbatim from a live-toolchain capture
    /// (`c2rs census … --keep-il`) rather than hand-assembled. The two receivers
    /// are formals 0 and 1 and the source's left operand is the one c2 calls
    /// first, so `lhs_first` is true and nothing is reordered.
    const MC_CMP_PLAIN: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xEE, 0x09,
        0x46, 0x2D, 0xED, 0x09, 0x2D, 0xEC, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xB9,
        0xEC, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41,
        0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE5, 0x09, 0xB9, 0xED, 0x09, 0x86,
        0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80,
        0x05, 0x10, 0x00, 0x00, 0x4C, 0x1F, 0x41, 0x82, 0x12, 0x30, 0x3A, 0xEF, 0x09, 0x54, 0x02,
        0x29, 0xEF, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `bool t2(const U* p, const U* q) { return q->n() == p->m(); }` — the SAME
    /// two calls with the source operands swapped. c2 emits `p`'s call first
    /// either way (ascending receiver rank), so this segment must decode to the
    /// same two calls in the same order with `lhs_first` FALSE. That pair is the
    /// whole ordering claim: a recognizer that carried the IL order into the
    /// shape passes the first test and fails this one.
    const MC_CMP_REORDERED: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xF2, 0x09,
        0x46, 0x2D, 0xF1, 0x09, 0x2D, 0xF0, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0xB9,
        0xF1, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41,
        0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE4, 0x09, 0xB9, 0xF0, 0x09, 0x86,
        0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80,
        0x05, 0x10, 0x00, 0x00, 0x4C, 0x1F, 0x41, 0x82, 0x12, 0x30, 0x3A, 0xF3, 0x09, 0x54, 0x02,
        0x29, 0xF3, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `bool H::q(const U* a) const { return m() == a->m(); }` — the refuter.
    /// `this` is parameter index 0 (and r3) but its token `0x0A00` is HIGHER than
    /// `a`'s `0x09FE`, because c1xx numbers the implicit receiver after the
    /// declared formals. So the calls are emitted `a` first, `this` second —
    /// the opposite of what ordering by parameter index gives — and the source's
    /// left operand (`this`) is the second call, `lhs_first` false.
    const MC_CMP_THIS: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x53, 0x53, 0x26, 0xF7, 0x09,
        0xB9, 0x00, 0x0A, 0xA6, 0x43, 0x8A, 0x20, 0x99, 0x86, 0x43, 0x8D, 0x20, 0x00, 0x46, 0x2D,
        0xFE, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF5, 0x09, 0xB9, 0x00, 0x0A, 0xA6, 0x43, 0x8A,
        0x20, 0x99, 0x86, 0x43, 0x8B, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x0B, 0x10,
        0x00, 0x00, 0x4C, 0x26, 0xE4, 0x09, 0xB9, 0xFE, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86,
        0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C,
        0x1F, 0x41, 0x82, 0x12, 0x30, 0x3A, 0x01, 0x0A, 0x54, 0x02, 0x29, 0x01, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];

    /// `bool sgt(const U* p, const U* q) { return p->m() > q->n(); }` — the
    /// SIGNED order spine, both callees returning `int` (`BD 86 41 74`).
    const MC_SGT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xF0, 0x09,
        0x46, 0x2D, 0xEF, 0x09, 0x2D, 0xEE, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xB9,
        0xEE, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41,
        0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE5, 0x09, 0xB9, 0xEF, 0x09, 0x86,
        0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80,
        0x05, 0x10, 0x00, 0x00, 0x4C, 0x24, 0x41, 0x82, 0x12, 0x30, 0x3A, 0xF1, 0x09, 0x54, 0x02,
        0x29, 0xF1, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `bool ult(const U* p, const U* q) { return p->um() < q->un(); }` — the
    /// UNSIGNED spine, same shape, `BD 86 42 75` in both call tokens. The
    /// operator byte `22` is the same one a *signed* `<` carries; only these
    /// TYPE triples separate the three-word spine from the five-word one.
    const MC_ULT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xF4, 0x09,
        0x46, 0x2D, 0xF3, 0x09, 0x2D, 0xF2, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE6, 0x09, 0xB9,
        0xF2, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x86, 0x42,
        0x75, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE7, 0x09, 0xB9, 0xF3, 0x09, 0x86,
        0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x86, 0x42, 0x75, 0x00, 0x80,
        0x06, 0x10, 0x00, 0x00, 0x4C, 0x22, 0x41, 0x82, 0x12, 0x30, 0x3A, 0xF5, 0x09, 0x54, 0x02,
        0x29, 0xF5, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int ugt_i(const U* p, const U* q) { return p->um() > q->un(); }` — the
    /// cell where the two type facts DISAGREE: unsigned operands, `int` result.
    /// The `2C 86 41 74 00` convert says `int` and the spine is still the
    /// unsigned three-word one.
    const MC_UGT_I: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x04, 0x53, 0x53, 0x26, 0xF8, 0x09,
        0x46, 0x2D, 0xF7, 0x09, 0x2D, 0xF6, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE6, 0x09, 0xB9,
        0xF6, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x86, 0x42,
        0x75, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE7, 0x09, 0xB9, 0xF7, 0x09, 0x86,
        0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x86, 0x42, 0x75, 0x00, 0x80,
        0x06, 0x10, 0x00, 0x00, 0x4C, 0x24, 0x2C, 0x86, 0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74,
        0x3A, 0xF9, 0x09, 0x54, 0x02, 0x29, 0xF9, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The base shape decodes to a **Class B** call sequence: two calls in
    /// emission order, one saved formal (the second call's receiver), and the
    /// `Cmp` tail whose `saved_gprs()` is therefore 2 — the saved formal plus
    /// the first call's result.
    #[test]
    fn two_member_calls_compared_for_equality_are_a_class_b_sequence() {
        let Some(BodyShape::CallSeq { params, calls, tail, saved }) =
            parse_segment(MC_CMP_PLAIN, NO_LOCALS)
        else {
            panic!("`return p->m() == q->n();` is the two-call comparator");
        };
        // The tokens as `read_token_var` returns them (`EC 09` -> 0xEC09).
        assert_eq!(params, vec![0xEC09, 0xED09], "p then q, un-reversed from `2D`");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xEC09)], "p's call is first");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0xED09)]);
        assert_ne!(calls[0].callee_tok, calls[1].callee_tok, "?m and ?n");
        assert_eq!(tail, SeqTail::Cmp { cmp: SeqCmp::Eq, lhs_first: true });
        assert_eq!(saved, vec![1], "q survives the first `bl` and takes r31");
    }

    /// **The ordering claim**, as the pair that separates it from "IL order".
    /// Swapping the source operands changes `lhs_first` and NOTHING else: the
    /// same two calls, in the same order, with the same saved formal. The emitted
    /// difference is the spine's two `subf` operands, four words later.
    #[test]
    fn the_source_operand_order_moves_only_the_spine_not_the_calls() {
        let Some(BodyShape::CallSeq { calls, tail, saved, .. }) =
            parse_segment(MC_CMP_REORDERED, NO_LOCALS)
        else {
            panic!("`return q->n() == p->m();` is the same production");
        };
        // `p` is formal 0 in this segment too (token `F0 09` against q's `F1 09`).
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xF009)], "p's call is STILL first");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0xF109)]);
        assert_eq!(
            tail,
            SeqTail::Cmp { cmp: SeqCmp::Eq, lhs_first: false },
            "…and it is now the RHS"
        );
        assert_eq!(saved, vec![1]);
    }

    /// **`this` is the cell that refutes ordering by parameter index.** It has
    /// index 0 and the highest token, so the declared formal's call goes first
    /// and the save is `mr r31,r3` — the hoisted arm, the only one this family
    /// reaches. Captured: `mr r31,r3 ; mr r3,r4 ; bl ?m ; mr r30,r3 ; mr r3,r31 ;
    /// bl ?m`.
    #[test]
    fn this_has_parameter_index_zero_and_the_highest_token_so_its_call_goes_last() {
        let Some(BodyShape::CallSeq { params, calls, tail, saved }) =
            parse_segment(MC_CMP_THIS, NO_LOCALS)
        else {
            panic!("a member function comparing `m()` against `a->m()`");
        };
        // **And these two tokens straddle a low-byte wrap**, which is why this
        // segment is the one kept: `a` is `FE 09` and `this` is `00 0A`, i.e.
        // allocation values 0x09FE and 0x0A00 — consecutive — but
        // `read_token_var` returns 0xFE09 and 0x000A, which order the *other*
        // way. The first version of the rule compared those raw values and was
        // `Port=Mismatch @ offset 8` on exactly this segment; the rank rule does
        // not look at them at all.
        assert_eq!(params, vec![0x000A, 0xFE09], "`this` at index 0, `a` at 1");
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xFE09)], "`a`'s call is first");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0x000A)], "`this`'s call is second");
        assert_eq!(tail, SeqTail::Cmp { cmp: SeqCmp::Eq, lhs_first: false });
        assert_eq!(saved, vec![0], "`this` is what survives the first `bl`");
    }

    /// **The whole relation axis, in one mutation sweep**: `==`, `>` and `<` are
    /// admitted and the other three refuse **by name and by relation, in the
    /// parser**, so the census and the gate cannot disagree about any of them.
    ///
    /// Written as a mutation of one accepted segment so the only thing that
    /// varies between the six rows is the operator byte — the axis a
    /// hand-written fixture would have one point on. The refusal keys are
    /// per-relation (`rel_refusal_key`) because a single `mcall-cmp-rel` measured
    /// 760 and could not say which relations they were; split, it says 692 `>`,
    /// 68 `<`, and **0 each** for the three below.
    #[test]
    fn equality_and_the_two_order_relations_are_admitted_and_the_rest_refuse_by_name() {
        let at = MC_CMP_PLAIN
            .windows(5)
            .position(|w| w[0] == 0x1F && w[1] == 0x41 && w[2] == 0x82)
            .expect("the `==` operator byte");
        assert!(matches!(
            parse_segment(MC_CMP_PLAIN, NO_LOCALS),
            Some(BodyShape::CallSeq { tail: SeqTail::Cmp { cmp: SeqCmp::Eq, .. }, .. })
        ));
        // `MC_CMP_PLAIN`'s two callees both return `int`, so every admitted
        // order row here is the SIGNED spine.
        for (op, label, greater) in [(0x22u8, "<", false), (0x24, ">", true)] {
            let mut seg = MC_CMP_PLAIN.to_vec();
            seg[at] = op;
            let Some(BodyShape::CallSeq { tail, saved, .. }) = parse_segment(&seg, NO_LOCALS)
            else {
                panic!("{label} over two call results is the same production");
            };
            assert_eq!(
                tail,
                SeqTail::Cmp {
                    cmp: SeqCmp::Order { greater, signed: true },
                    lhs_first: true
                },
                "{label} keeps the call order and moves only the spine"
            );
            assert_eq!(saved, vec![1], "{label} is the same Class B frame as `==`");
        }
        for (op, label, key) in [
            (0x20u8, "!=", "mcall-cmp-rel-ne"),
            (0x21, "<=", "mcall-cmp-rel-le"),
            (0x23, ">=", "mcall-cmp-rel-ge"),
        ] {
            let mut seg = MC_CMP_PLAIN.to_vec();
            seg[at] = op;
            assert_eq!(parse_segment(&seg, NO_LOCALS), None, "{label} must refuse");
            assert_eq!(
                parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
                key,
                "{label} must refuse by name AND by relation, in the parser"
            );
        }
    }

    /// **The operand signedness comes from the call's result TYPE, and nothing
    /// else could supply it** — `<` is opcode `0x22` for both signednesses, and
    /// the two lower to a five-word spine and a three-word one.
    ///
    /// Three live segments, transcribed from one capture of
    /// `bool sgt(…){ return p->m() >  q->n();  }`,
    /// `bool ult(…){ return p->um() < q->un(); }` and
    /// `int  ugt(…){ return p->um() > q->un(); }` — so the signed/unsigned pair
    /// differs *only* in the TYPE triples (`86 41 74` against `86 42 75`), and
    /// the third adds the `2C` convert an `int`-typed result carries.
    #[test]
    fn the_operand_signedness_is_read_from_the_call_result_type() {
        let cmp_of = |seg: &[u8]| match parse_segment(seg, NO_LOCALS) {
            Some(BodyShape::CallSeq { tail: SeqTail::Cmp { cmp, .. }, .. }) => cmp,
            other => panic!("expected a two-call comparator, got {other:?}"),
        };
        assert_eq!(cmp_of(MC_SGT), SeqCmp::Order { greater: true, signed: true });
        assert_eq!(cmp_of(MC_ULT), SeqCmp::Order { greater: false, signed: false });
        // The `int`-typed result of an UNSIGNED comparison: the convert says
        // `int` and the operands are still unsigned. Two facts, and reading the
        // wrong one picks the five-word spine for a three-word body.
        assert_eq!(cmp_of(MC_UGT_I), SeqCmp::Order { greater: true, signed: false });
    }

    /// A comparison whose two operands have **different** result types cannot
    /// reach the spine — and it is refused by the *grammar*, not by
    /// [`operand_signedness`], because c1xx puts an explicit `2C` convert on
    /// whichever side needs one. Measured in both directions:
    /// `p->m() > q->un()` puts it between the first `4C` and the second `26`,
    /// `p->um() > q->n()` between the second `4C` and the relation.
    ///
    /// Pinned here as a **unit** on the predicate, because the two grammar
    /// refusals are `Err(None)` — invisible to a census key — and the agreement
    /// check is the second lock that stays behind if either ever widens.
    #[test]
    fn a_mixed_signedness_comparison_has_no_operand_signedness() {
        // The two `BD` positions of the signed segment.
        let bd: Vec<usize> = MC_SGT
            .windows(4)
            .enumerate()
            .filter(|(_, w)| w[0] == 0xBD && w[1] == 0x86 && w[2] == 0x41 && w[3] == 0x74)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(bd.len(), 2, "two int-returning calls");
        assert_eq!(operand_signedness(MC_SGT, bd[0], bd[1]), Some(true));
        // Rewrite the SECOND call's return type to `unsigned` and the pair no
        // longer agrees.
        let mut mixed = MC_SGT.to_vec();
        mixed[bd[1] + 2] = 0x42;
        mixed[bd[1] + 3] = 0x75;
        assert_eq!(operand_signedness(&mixed, bd[0], bd[1]), None, "int vs unsigned");
        // A `bool`-returning callee is not the 4-byte integer class at all.
        let mut boolean = MC_SGT.to_vec();
        boolean[bd[1] + 1] = 0x82;
        boolean[bd[1] + 2] = 0x12;
        boolean[bd[1] + 3] = 0x30;
        assert_eq!(operand_signedness(&boolean, bd[0], bd[1]), None, "not int4");
        // And a position that is not a `BD` at all answers `None` rather than
        // decoding whatever byte happens to stand there.
        assert_eq!(operand_signedness(MC_SGT, bd[0] + 1, bd[1]), None);
    }

    /// **The label surcharge is a fact about the signed order spine only**, and
    /// it is the one number in this rung that no byte compare of a single-function
    /// obj would catch: it moves the `$M` numbers of every *later* function in
    /// the TU. Measured `scripts/gt_cmp_rr.py --stride`, four modes.
    #[test]
    fn only_the_signed_order_spine_takes_leading_label_slots() {
        use crate::func::SeqTail as PubTail;
        let lead = |cmp| PubTail::Cmp { cmp, lhs_first: true }.label_lead();
        assert_eq!(lead(SeqCmp::Order { greater: true, signed: true }), 2);
        assert_eq!(lead(SeqCmp::Order { greater: false, signed: true }), 2);
        assert_eq!(lead(SeqCmp::Order { greater: true, signed: false }), 0);
        assert_eq!(lead(SeqCmp::Order { greater: false, signed: false }), 0);
        assert_eq!(lead(SeqCmp::Eq), 0, "the shipped `==` stride is unchanged");
        assert_eq!(PubTail::Void.label_lead(), 0);
        assert_eq!(PubTail::CallValue { add_k: 3 }.label_lead(), 0);
        assert_eq!(PubTail::Lit(3).label_lead(), 0);
    }

    /// **The allocation rank**, as the table the whole rung's call order rests on.
    /// A free function's is its parameter index; a member function's `this` is
    /// `params[0]` and ranks LAST, because c1xx numbers the implicit receiver
    /// after every declared formal.
    #[test]
    fn this_ranks_after_every_declared_formal_although_it_is_parameter_zero() {
        let p3 = [0xA0u32, 0xA1, 0xA2];
        // Free function: rank is the index.
        assert_eq!(alloc_rank(&p3, false, 0xA0), Some(0));
        assert_eq!(alloc_rank(&p3, false, 0xA1), Some(1));
        assert_eq!(alloc_rank(&p3, false, 0xA2), Some(2));
        // Member function: `this` is params[0] and ranks last; the declared
        // formals shift down by one and keep their relative order.
        assert_eq!(alloc_rank(&p3, true, 0xA0), Some(3), "`this` is numbered last");
        assert_eq!(alloc_rank(&p3, true, 0xA1), Some(0));
        assert_eq!(alloc_rank(&p3, true, 0xA2), Some(1));
        // A token that is not a formal has no rank — the call site has already
        // refused it (`call-arg-nonformal`), and this is the second lock.
        assert_eq!(alloc_rank(&p3, true, 0xFF), None);
    }

    /// The **total** saved-GPR count is what the helper-class gate has to see.
    /// `saved` holds one formal; the tail holds the first call's result; the
    /// prologue must `std` two registers. A consumer reading `saved.len()` gets 1
    /// and emits a Class A prologue with a Class B body.
    #[test]
    fn the_tail_s_saved_call_result_counts_toward_the_gpr_total() {
        let Some(BodyShape::CallSeq { saved, tail, .. }) =
            parse_segment(MC_CMP_PLAIN, NO_LOCALS)
        else {
            panic!("the two-call comparator");
        };
        assert_eq!(saved.len(), 1, "one saved FORMAL");
        assert_eq!(tail, SeqTail::Cmp { cmp: SeqCmp::Eq, lhs_first: true });
        // …plus the first call's result, which is the PUBLIC shape's business:
        // `bundle.rs` maps this body onto `c2_il::CallSeq`, and `saved_gprs()` is
        // the one place the two are summed.
        use crate::func::SeqTail as PubTail;
        let seq = crate::func::CallSeq {
            calls: Vec::new(),
            tail: PubTail::Cmp { cmp: SeqCmp::Eq, lhs_first: true },
            saved: vec![1],
        };
        assert_eq!(seq.saved_gprs(), 2);
        // Class A/B statement sequences are unchanged: their tails save nothing.
        assert!(!PubTail::Void.saves_a_call_result());
        assert!(!PubTail::CallValue { add_k: 3 }.saves_a_call_result());
        assert!(!PubTail::Lit(3).saves_a_call_result());
    }
}

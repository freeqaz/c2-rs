pub(crate) mod chain;
pub use self::chain::{chain_form, ChainForm};
pub use self::shapes::leaf_store::FP_SCRATCH;
pub(crate) mod expr;
pub(crate) mod mcall;
pub(crate) mod shapes;

use self::chain::{
    canonical_chain_for_codegen, has_repeated_leaf, straight_line_out_of_class_ctx, ChainReject,
};
use self::expr::{
    eat_fn_tail, eat_return_head, eat_return_plumbing, eat_scopes, intrinsic_name,
    parse_expr_classed, parse_formals, BODY_SCOPE_DEPTH,
};
use self::shapes::parse_params;
use self::shapes::{
    eat_ctor_this_epilogue, parse_call_shape, try_parse_addr_leaf, try_parse_assign_body_detail,
    try_parse_cmp_shift_or, try_parse_compare, try_parse_cond_tail_pair,
    try_parse_div_mod_leaf, try_parse_early_return_seq,
    try_parse_empty_ctor_base_delegation,
    try_parse_guarded_seq,
    try_parse_empty_dtor_delegation,
    try_parse_float_leaf,
    try_parse_fp_tail_call,
    try_parse_indirect_load_leaf, try_parse_member_tail_call, try_parse_ptr_identity_leaf,
    try_parse_ptr_walk_chain_loop,
    try_parse_guard_chain_shared_tail,
    try_parse_if_call_join,
    try_parse_ptr_walk_loop,
    try_parse_static_scan_loop,
    try_parse_store_leaf, try_parse_store_run, try_parse_store_run_bind,
    try_parse_store_run_call,
};
use super::readers::{
    eat_byte, eat_value_type, read_token_var, read_type, read_varint, ValueClass,
};
use super::sy::SyView;
use super::{CompareLeaf, IlOp, Rel};

// ---- the two DISPATCH axes ------------------------------------------------
//
// **Neither of these is a census key, and that is the whole point.** A census key
// names the *construct* a body needs; these name **which recognizer looked at the
// body and where inside it the refusal happened**. Six ranking rungs running, the
// answer to "what is this large blocking row" has been "a private limit inside a
// recognizer that already ships", and no census key can say that — the key is
// minted by `mcall`'s completeness walk, which runs to the side of the production
// and cannot see whether the production was even entered.
//
// The two axes answer two different halves of that, and they compose:
//
// * [`DISPATCH`] — **which arm of [`parse_segment_shape`]'s ladder claimed this
//   body.** A member-call construct whose body does not *begin* with the member
//   call (it is a store's right-hand side, or a plain call's argument) never
//   reaches the member-call productions at all, so **no widening inside any of
//   them can move it**. Before this axis existed those bodies were
//   indistinguishable, in every table, from the ones a widening could move.
// * [`PROD`] — **which non-committal bail inside the member-call productions
//   fired.** The 37 tag sites live in `shapes::mcall_{tail,chain,cmp}`; the
//   carrier here is deliberately independent of them, so those files can be
//   tagged one call at a time without touching this module.
//
// Both are **diagnostic only, structurally**: nothing but [`crate::func::census`]
// reads them, acceptance never branches on them, and a body's verdict is
// identical whether or not a tag was ever set.
//
// ### Why every state is NAMED, including "nothing happened"
//
// A tag that is only set on the interesting path reports the *previous* body's
// value on the boring one, and an axis whose default is "absent" makes the
// largest population on the board invisible rather than large. So the defaults
// are positive claims — `disp-not-run` means *the ladder did not run for this
// body*, `prod-not-entered` means *the member-call productions were never
// entered* — they are reset per body, and the report prints them like any other
// row. `prod-entered-untagged` is the same discipline applied to the tag sites
// themselves: it counts the bodies that entered a production, declined
// non-committally, and hit **no** tagged bail — i.e. it is the exact measure of
// how much of `mcall_*.rs` is still untagged, and it reaches 0 when that work is
// finished.

/// The dispatch ladder never ran for this body (it was refused on its NAME, before
/// any byte of it was read — see `census`'s varargs arm).
pub(crate) const DISP_NOT_RUN: &str = "disp-not-run";
/// [`try_parse_member_tail_call`] — and therefore `mcall_chain` and `mcall_cmp`,
/// which are reached only through it — was never entered for this body.
pub(crate) const PROD_NOT_ENTERED: &str = "prod-not-entered";
/// A member-call production was entered, declined **non-committally**, and no
/// tagged bail inside it fired. The residue that measures the tag coverage of
/// `shapes::mcall_{tail,chain,cmp}`; its target is 0.
pub(crate) const PROD_ENTERED_UNTAGGED: &str = "prod-entered-untagged";
/// A member-call production accepted the body. Not a blocker at all — the
/// in-class control group for this axis.
pub(crate) const PROD_ACCEPTED: &str = "prod-accepted";
/// A member-call production **committed** and then refused, so the body's census
/// key is that gate's own and no first-blocker attribution is owed.
pub(crate) const PROD_COMMITTED_REFUSAL: &str = "prod-committed-refusal";

thread_local! {
    static DISPATCH: std::cell::Cell<&'static str> = const { std::cell::Cell::new(DISP_NOT_RUN) };
    static PROD: std::cell::Cell<&'static str> = const { std::cell::Cell::new(PROD_NOT_ENTERED) };
}

/// Record which dispatch arm the ladder is in. Called only from
/// [`parse_segment_shape`].
#[inline]
fn disp(tag: &'static str) {
    DISPATCH.with(|c| c.set(tag));
}

/// **The tag call for a non-committal bail inside a member-call production.**
///
/// Returns `None` so it drops straight into the two shapes those productions
/// already use for "not this production, cursor untouched":
///
/// ```ignore
/// eat_receiver_this(seg, &mut p).map_err(|_| prod_tag("tail-recv-not-b9-load"))?;
/// return Err(prod_tag("chain-body-does-not-end-at-call"));
/// ```
///
/// The name is the tag's own words — a construct or a limit, never a line number,
/// which moves whenever the file is edited and says nothing to a reader of the
/// report. Last write wins, so an inner production's tag correctly replaces the
/// outer one's on the way out.
pub(crate) fn prod_tag(tag: &'static str) -> Option<Block> {
    PROD.with(|c| c.set(tag));
    None
}

/// Reset both axes to their named defaults. Per body, and **before** the parse —
/// a stale tag from the previous segment is the one failure mode that would make
/// this whole instrument report fiction.
pub(crate) fn dispatch_reset() {
    DISPATCH.with(|c| c.set(DISP_NOT_RUN));
    PROD.with(|c| c.set(PROD_NOT_ENTERED));
}

/// The dispatch arm that claimed the last body parsed on this thread.
pub(crate) fn dispatch_site() -> &'static str {
    DISPATCH.with(|c| c.get())
}

/// The member-call production first-blocker of the last body parsed on this thread.
pub(crate) fn prod_site() -> &'static str {
    PROD.with(|c| c.get())
}

/// **How many CALL tokens a function segment contains** — the D6 frame measure
/// (`docs/IL_CALL_IN_EXPR.md` §18).
///
/// The question every remaining census row has to answer is whether its lowering
/// is *local*, and the coarsest form of that question is whether the body needs a
/// **frame**: a body that issues two or more calls must save LR, because the first
/// `bl` clobbers it and the return address is still needed. That is a property of
/// the body alone, so it is measurable without any codegen — and it is measurable
/// **outside** the modeled grammar, which is the point: the grammar stops at the
/// first unmodeled byte, and this walk does not stop at all.
///
/// The walk is not a parse and is **graded rather than asserted**. A `BD` counts
/// only when *every* field of the decoded CALL token
/// `BD <ret TYPE> <conv> <varint fn-type-id>` (`docs/IL_CALL_IN_EXPR.md` §0) is
/// present and reads a **measured** value, and the cursor then skips the whole
/// token so a `BD` inside a consumed payload cannot be recounted. Everything else
/// advances one byte.
///
/// The three gates, and each is a field that never varied — so it is required
/// literally and fails closed, rather than being skipped as "probably constant":
///
/// * the **calling-convention byte is `00`**. 15,095 of 16,100 `BD`-plus-TYPE
///   sites in `src/lazer/meta_ham/HamUI.cpp` read `00` and the rest are spread
///   over 200-odd distinct bytes — the signature of a payload byte, not a field.
/// * the **fn-type-id uses `read_varint`'s `80` escape form**: 15,090 of 15,095.
/// * its value is **≥ 0x1000**. Function-type ids are allocated per TU from
///   0x1000 (`parse_call_shape`), so the short varint form cannot spell one.
///   Measured range 0x1001…0x1081 across the fixtures and 0x1001…0xFA89 in the
///   wild TU; exactly one candidate site fell below and it is a false positive.
///
/// A bare `67` (virtual dispatch) is **not** counted: a virtual call carries its
/// own `BD` as well, so counting the `67` too double-counted it — measured, and
/// removing it is part of what took the grade from 98.0 % to 98.7 %.
///
/// **The grade, MEASURED.** Over the 110 fixtures plus the D6 probes — every TU
/// where `.gl` binds one name per segment, so segment *k* pairs 1:1 with emitted
/// function *k* — this count agrees with the reference obj's own `bl`/`b` count on
/// **696 of 705 functions (98.7 %)**. Both failure directions are named and both
/// are one-sided:
///
/// * **undercount** — an `0x40` intrinsic that lowers to a real branch is not a
///   `BD` (`memcpy`, `memset`, `dynamic_cast`, an aggregate copy): 6 witnesses.
/// * **overcount** — c2 inlined or folded a call the IL still spells (an intra-TU
///   callee it cloned, a destructor whose second call folded away): 3 witnesses.
///
/// The **in-class functions are the standing control group** and the census
/// reports them: a shape the whole-body parser accepted as a leaf cannot contain
/// two calls, so `calls-2plus` among `indirect-load-leaf` / `straight-line` /
/// `empty-body` is a direct read of the residual false-positive rate.
///
/// Diagnostic only. Nothing here is consulted by the emitter or by acceptance.
pub(crate) fn call_tokens(seg: &[u8]) -> usize {
    /// The floor of the per-TU function-type id space (`parse_call_shape`).
    const FN_TYPE_ID_MIN: i32 = 0x1000;
    let mut n = 0usize;
    let mut p = 0usize;
    while p < seg.len() {
        if seg[p] != 0xBD {
            p += 1;
            continue;
        }
        let ok = read_type(seg, p + 1).and_then(|(_, _, _, tw)| {
            let q = p + 1 + tw;
            // the calling-convention byte, then the escape-form fn-type id
            if seg.get(q) != Some(&0x00) || seg.get(q + 1) != Some(&0x80) {
                return None;
            }
            let mut e = q + 1;
            let id = read_varint(seg, &mut e)?;
            (id >= FN_TYPE_ID_MIN).then_some(e)
        });
        match ok {
            Some(q) => {
                n += 1;
                p = q;
            }
            None => p += 1,
        }
    }
    n
}

/// One recognized whole-body shape of a single `.ex` function segment. Every
/// accepted body is *exactly* one of these — the parser (see [`parse_segment`])
/// is a positive whole-stream parse that reaches the segment's end, so anything
/// it does not model produces `None` and the caller reports `NotImplemented`.
/// Which sub-object a [`BodyShape::EmptyDtorDelegation`] destroys, and therefore
/// which of the two receiver productions its address came from
/// (`docs/IL_CALL_IN_EXPR.md` §14.3). Recorded rather than inferred from `adjust`
/// — a **member** at offset 0 and a **base** at adjust 0 emit the identical four
/// bytes, so the emitter cannot tell them apart and only the census wants to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DtorSubObject {
    /// The single non-virtual base, reached through the `this`-adjust intrinsic
    /// 2113 (`docs/IL_CALL_IN_EXPR.md` §5 — the D1 shape).
    Base,
    /// A data member, reached by a plain `27` byte-offset add with no intrinsic
    /// (§14.3, §15).
    Member,
}

/// One call inside a [`BodyShape::CallSeq`], with its argument setup already
/// validated and normalized by [`super::shapes::tail_call_shape`] — the same
/// locator every other call shape's arguments go through, so the marshalling has
/// one implementation and not a per-shape one.
///
/// The two argument forms are the two the tail call already had: an operand
/// stream computed into r3 (0 or 1 argument, `arg_ops` empty for a nullary call),
/// or a register permutation over the formals (2+ bare-parameter arguments).
///
/// [`Self::link_args`] is a **third** form and is not a variant of either: it
/// belongs to a call whose slot 0 is already filled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SeqCall {
    pub(crate) callee_tok: u32,
    pub(crate) arg_ops: Vec<IlOp>,
    /// The argument slots in slot order — the token-carrying twin of
    /// [`crate::func::SeqCall::arg_slots`], and see that field for why it is a
    /// slot list rather than the `Option<Vec<usize>>` permutation it was until
    /// lane `w-memcpy`.
    pub(crate) arg_slots: Option<Vec<SlotArg>>,
    /// **WCL — this call is a CHAIN LINK**: its receiver is the previous call's
    /// result, already sitting in r3, so its own explicit arguments start at
    /// argument slot **1** and its marshalling is a different lowering from
    /// every other call's. `Some` for exactly the links of
    /// [`super::shapes::mcall_chain`]; `None` everywhere else, including for a
    /// chain's innermost call, whose argument list is complete and goes through
    /// [`super::shapes::tail_call_shape`] like any other.
    ///
    /// One entry per explicit argument, in **slot order** (ascending), which is
    /// the order c2 emits them in — the opposite of the order it uses for a call
    /// whose list starts at slot 0. That is measured, not assumed; see
    /// `c2_core::codegen::calls::link_setup_text`.
    pub(crate) link_args: Option<Vec<SlotArg>>,
}

/// One explicit argument of a **chain link**, already resolved to the only two
/// things a link is admitted to carry.
///
/// It is a small closed enum rather than an operand stream because a link's
/// argument is never *computed*: the register it would be computed into is the
/// callee-saved file the saves live in (`addi r4,r31,1`, captured), which is a
/// second lowering of the leaf selector rather than a use of it, and
/// `super::shapes::calls::plan_saved_gprs` refuses it by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SlotArg {
    /// A formal, by index into the `params` list. It is live across the previous
    /// `bl`, so it is in a callee-saved GPR by the time the link runs, and
    /// `plan_saved_gprs` is what put it there.
    Formal(usize),
    /// A literal — `li r<slot>,k`. It costs no callee-saved register, so a chain
    /// whose only link arguments are literals stays **Class A**.
    Lit(i32),
    /// **WR1 — the address of a named data symbol**, by its `.gl` operand token.
    /// Produced only by a *tail call*'s argument list
    /// (`super::shapes::calls::tail_call_shape`); a **chain link** never carries
    /// one, because the address would have to survive the previous `bl` and
    /// nothing captures where c2 keeps it. `bundle::slot_arg` resolves the token
    /// to a mangled name; an unresolvable one refuses.
    SymAddr(u32),
}

/// One arm of a [`CondTailPairShape`], with the callee still a `.gl` token.
/// [`crate::func::CondArm`] is its resolved twin.
///
/// The slot list is the **resolved** [`crate::func::SlotArg`] rather than this
/// module's token-carrying one, because an arm admits no data-symbol address:
/// WR1's `lis` is hoisted ahead of the *whole* argument setup, and where that
/// lands relative to a conditional branch has no capture. With `SymAddr` out of
/// the vocabulary the two types coincide, so carrying the resolved one keeps a
/// conversion step — and a place for it to be wrong — out of `shape_to_function`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CondArmShape {
    pub(crate) callee_tok: u32,
    pub(crate) slots: Vec<crate::func::SlotArg>,
}

/// **W8 — the two-arm conditional tail call**, as parsed.
/// [`crate::func::CondTailPair`] is its resolved twin; the only difference is
/// callee token vs callee name, exactly as for every other call shape here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CondTailPairShape {
    pub(crate) params: Vec<u32>,
    pub(crate) cmp_param: usize,
    pub(crate) rel: crate::func::Rel,
    pub(crate) signed: bool,
    pub(crate) k: i32,
    pub(crate) then_arm: CondArmShape,
    pub(crate) else_arm: CondArmShape,
}

impl CondTailPairShape {
    /// The register schedule — **the parser's last gate**. The emitter runs the
    /// same function on the resolved twin, so the census and the emission gate
    /// cannot disagree about what is in class.
    pub(crate) fn plan(&self) -> Option<crate::func::CondPlan> {
        crate::func::plan_cond_pair(
            self.params.len(),
            self.cmp_param,
            &self.then_arm.slots,
            &self.else_arm.slots,
        )
    }
}

/// **W10 — the guard on a [`BodyShape::CallSeq`]: the FRAMED × BRANCHING cell.**
///
/// `work/w-frame/RANKING.md` §4: over the 105 functions the port emits
/// byte-exact, 28 are framed, 2 branch, and **zero are both**, while 10 of the
/// 17 FRONTIER TUs need the product. This is the field that makes the product
/// expressible.
///
/// The guarded call is always `calls[0]`. There is no index, because the
/// production admits the guard only as the body's **first** statement: every
/// witness for a guard later in a sequence needs a callee-saved formal, which
/// [`super::shapes::guarded_seq`] refuses for reasons its module doc gives.
/// There is no `else` arm either, and that refusal is a **measurement** — see
/// the same module doc: `/Ox` and `/O2` tail-duplicate the join block and the
/// epilogue where `/O1` shares them behind an intra-section `b`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SeqGuardShape {
    /// Index into the sequence's `params` of the compared formal.
    pub(crate) cmp_param: usize,
    /// The **source** relation. The emitted branch is its negation, because the
    /// IL's `38` is brFALSE (`docs/CFG_SHAPE.md` §1 prediction A3).
    pub(crate) rel: Rel,
    /// `cmpwi` when true, `cmplwi` when false — from the operand's TYPE triple
    /// alone; the relational opcodes are sign-agnostic (§3.2).
    pub(crate) signed: bool,
    /// The comparison literal, inside the 16-bit immediate field.
    pub(crate) k: i32,
}

/// **W11 — one guarded EARLY RETURN ahead of a [`BodyShape::CallSeq`].**
///
/// `if (formal <rel> k) return <literal>;` (or `return;`) written before the
/// sequence. A body carries a `Vec` of these because the guards chain, and the
/// chain is the point: `work/w-conv/PREREG.md` §2 counts a real label→offset map
/// as the missing mechanism **14 of the 17 FRONTIER TUs** want, and the
/// intra-section `b` it forces as the one **10** of them want.
///
/// Distinct from [`SeqGuardShape`], which guards a *call* that falls through
/// into the sequence. These guards leave the function. The two are refused in
/// combination — one production per body — although c2 composes them happily
/// (`work/w-conv/p/probe3.cpp::x6`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SeqEarlyReturnShape {
    /// **W-SMALL — the short-circuit `&&`'s extra conditions**, in source order,
    /// after the one this struct's own `cmp_param`/`rel`/`signed`/`k` carry.
    /// Empty for a plain single-test guard, which is every shape before this.
    ///
    /// `if (a != 0 && b != 0) return 5;` is not a second statement — it is ONE
    /// guard whose IL repeats the condition-and-branch group with the **same**
    /// skip label:
    ///
    /// ```text
    ///   b9 e4 09 … 20  38 e8 09          <- test a, brFALSE -> e8
    ///   b9 e5 09 … 20  38 e8 09          <- test b, brFALSE -> e8   (SAME label)
    ///   53 33 … 05 41 …                  <- the one arm
    ///   29 e8 09                         <- e8 defined once, after the arm
    /// ```
    ///
    /// which is byte-for-byte the single-guard IL with 15 bytes inserted. Two
    /// separate `if`s are a different production and already parse: they mint
    /// **two** labels (`e8`, `e9`) and **two** arms.
    ///
    /// `||` is NOT this shape and is deliberately not admitted here — it emits
    /// the other branch sense (`39` in the IL, `409a` in the text) and moves the
    /// arm block to the **end**, after the fall-through path's epilogue. That is
    /// a block layout nothing has graded.
    pub(crate) and_conds: Vec<(usize, Rel, bool, i32)>,
    /// Index into the sequence's `params` of the compared formal. The scrutinee
    /// is read **in its home argument register**: this class admits no
    /// entry-block move, exactly as [`SeqGuardShape`] does and for the same
    /// measured reason.
    pub(crate) cmp_param: usize,
    /// The **source** relation.
    ///
    /// The emitted sense depends on whether the arm is empty, and that is a
    /// measurement rather than a convenience:
    ///
    /// * a **value-returning** arm (`value` is `Some`) is a real block, so the
    ///   branch is the edge *past* it and carries the **negation** of `rel`
    ///   — `docs/CFG_SHAPE.md` §1 A3, ten cells;
    /// * a **void** arm (`value` is `None`) is empty, so c2 deletes the block
    ///   and points the branch straight at the epilogue with `rel` itself.
    ///   `void w1(int a){ if(a!=0) return; v0(); v1(); }` emits
    ///   `cmpwi cr6,r3,0 ; bf 26,+12 -> epilogue`, where the value form of the
    ///   same guard emits `bt 26`. Measured at `/O1` and `/Ox`, 1 and 2 guards
    ///   (`work/w-conv/p/probe3.cpp::w1`/`w2`).
    ///
    /// It is the **empty-arm inversion** `work/w-cross/PREREG.md` §1 found in
    /// `src/system/negate_test.cpp`, in the smallest body that has it.
    pub(crate) rel: Rel,
    /// `cmpwi` when true, `cmplwi` when false — from the operand's TYPE triple
    /// alone; the relational opcodes are sign-agnostic.
    pub(crate) signed: bool,
    /// The comparison literal, inside the 16-bit immediate field.
    pub(crate) k: i32,
    /// The returned literal, or `None` for `return;`.
    ///
    /// **Every exit value in the body must be distinct**, including the
    /// sequence's own [`SeqTail::Lit`], and the parser enforces it. Where two
    /// exits share a value c2 **merges the arms**: with two guards both
    /// returning 5 it emits one arm and branches *backwards* into it with the
    /// sense inverted (`409afff4  bf 26,-12`), and a guard returning the
    /// sequence's own literal loses its arm entirely. The merge also costs a
    /// **sixth** compiler-label slot where every cell in this class costs five,
    /// so admitting it without noticing is six wrong bytes in the symbol table
    /// as well as a wrong block. `work/w-conv/PREREG.md` §3.1 has both measurements.
    pub(crate) value: Option<i32>,
}

/// What a [`BodyShape::CallSeq`] does after its last call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SeqTail {
    /// Nothing — the body returns void (or discards the last call's result too).
    Void,
    /// The **last call's** result is the return value, plus a literal `add_k`
    /// (0 for a bare `return g();`, non-zero for `return g() + k;` — the same
    /// `addi r3,r3,k` post-op [`BodyShape::FramedCall`] carries).
    CallValue { add_k: i32 },
    /// `return <literal>;` after the last statement call — one `li r3,k`.
    Lit(i32),
    /// **WCO** — the last call's result is a pointer and the body **reads
    /// through it**: `return p->a()->b()->m;` is one `lwz r3,off(r3)`.
    ///
    /// The sibling of [`Self::CallValue`], which is the same designator without
    /// the `30` load — `return &p->a()->b()->m;` is `addi r3,r3,off` and is
    /// already spelled by `CallValue { add_k: off }`. The two differ by one
    /// instruction and by nothing else, which is why the address form needed no
    /// variant and this one does. Measured, `work/WCO/probe/p1.cpp`.
    CallLoad { off: i32 },
    /// **WFL** — the same designator step whose member is **floating point**:
    /// `float f(O* p){ return p->a()->b()->m; }` is one `lfs f1,off(r3)`, and a
    /// `double` member is `lfd`.
    ///
    /// Its own variant rather than a `double` flag on [`Self::CallLoad`] for the
    /// reason `CallLoad` is not a flag on `CallValue`: it is a **different
    /// register file**. The value lands in `f1`, not r3, and the body's obj
    /// acquires the undefined external `_fltused`
    /// ([`crate::func::IlFunction::touches_floating_point`]) — a TU-level
    /// obligation no integer tail carries. Measured, `work/WFL/probe/p1.cpp`
    /// `/O1 /GS- /c`: `c0230004` = `lfs f1,4(r3)` and `c8230010` =
    /// `lfd f1,16(r3)`.
    ///
    /// `double` is the **loaded** width, not the returned one. A `float` member
    /// returned as a `double` is byte-identical to the unpromoted form — `lfs`
    /// loads and converts in one instruction — so the promotion is free and the
    /// emitted opcode still follows the member.
    CallLoadFp { off: i32, double: bool },
    /// **WCB/WCR** — `return <call> <rel> <call>;`: the two calls' results
    /// compared and materialized to a 0/1 in r3. `lhs_first` says whether the
    /// source's left operand is the call emitted *first*; c2 orders the pair by
    /// receiver token, not by source position, so the two are independent.
    /// See [`super::shapes::mcall_cmp`].
    Cmp { cmp: crate::func::SeqCmp, lhs_first: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BodyShape {
    /// Straight-line all-`int` arithmetic leaf (`return a+b+c`, `return a+5`,
    /// `return 42`, …): a postfix LOAD/LIT/ADD/SUB/MUL stream returning `int`.
    StraightLine { params: Vec<u32>, ops: Vec<IlOp> },
    /// Bare terminal void tail call (`void f(){ g(); }`): exactly one CALL whose
    /// void result is discarded, with **nothing** after its `4C 4B` void
    /// call-end but the return plumbing → codegen emits a single `b <callee>`.
    VoidTailCall { callee_tok: u32 },
    /// Integer tail call `return g(<arg>)` (and the identity-fold `g(a) + 0`):
    /// exactly one int-returning CALL whose single argument is a modeled
    /// sub-expression (`arg_ops`), a `55 <int> 4C` call-end, and a **net-identity
    /// post-op** (absent, or `+ 0` folded away). Codegen computes the argument
    /// into r3 (the leaf selector), then `b <callee>` — a 5-section leaf, the
    /// integer analog of [`BodyShape::VoidTailCall`]. `arg_ops` is a bare
    /// `[Load]` for the passthrough `g(a)`, or e.g. `[Load, Lit, Add]` for the
    /// arg-setup `g(a + 1)` (→ `addi r3,r3,1 ; b g`). `params` are the formals
    /// (token→register mapping the arg-setup needs).
    IntTailCall { params: Vec<u32>, arg_ops: Vec<IlOp>, callee_tok: u32 },
    /// **The single-argument floating-point tail call** — `return g(x);` and
    /// `g(x);` where `x` is an FP formal. The whole emission is at most one
    /// instruction plus the branch: `fmr f1,f<n+1>` (elided when the argument is
    /// already f1), or `frsp f1,f<n+1>` when the callee's formal is the narrower
    /// width.
    ///
    /// Kept apart from [`BodyShape::IntTailCall`] because the argument register
    /// is in a different **file**: `params` here is the FP formals *alone*, in FP
    /// order, so entry `n` is `f(n+1)` — exactly [`BodyShape::FloatLeaf`]'s
    /// convention and for the same reason (`docs/CODEGEN_FP_ARGS.md` §0). A
    /// positional model puts `float f(int k, float b){ return g(b); }`'s argument
    /// in f2 and emits an `fmr` c2 does not.
    ///
    /// `arg_tok` is the argument formal's token; `narrowing` says the callee's
    /// formal is `float` where the source is `double`, which is the one case that
    /// emits `frsp` instead of `fmr` — **fused**, not `fmr` then `frsp`. See
    /// [`super::shapes::try_parse_fp_tail_call`] for the captures.
    FpTailCall { params: Vec<u32>, arg_tok: u32, narrowing: bool, callee_tok: u32 },
    /// `return g(x1, …, xn)` with `n >= 2` and **every argument a floating-point
    /// formal** — the other half of the FP tail-call family, W34.
    ///
    /// `params` is the FP formals alone in FP-file order (entry `k` is `f(k+1)`,
    /// exactly as [`BodyShape::FpTailCall`] carries it) and `arg_sources[i]`
    /// indexes it for the value destination `f(i+1)` wants. Because every argument
    /// is FP, the destination numbering is `1..=n` with nothing else consuming a
    /// slot in that file — which is the whole reason this shape is separate from
    /// [`BodyShape::MultiArgTailCall`]: a call that mixes the two files needs
    /// moves in both, and their schedules **interleave**
    /// (`docs/CODEGEN_FP_ARGS.md` §1.1).
    FpMultiArgTailCall { params: Vec<u32>, arg_sources: Vec<usize>, callee_tok: u32 },
    /// `return g(a1, …, an)` with `n >= 2` and every argument a bare parameter or
    /// (WLA) a **literal**. `arg_sources[i]` is what argument slot `i` wants:
    /// [`SlotArg::Formal`] indexes `params`, [`SlotArg::Lit`] is one `li r<3+i>,k`.
    /// Codegen turns the list into a register permutation, or into the `li`s
    /// alone, plus the tail branch.
    ///
    /// One list rather than a permutation *plus* a literal side-table, because
    /// "what does argument slot `i` want" is **one** fact and this file's history
    /// is what happens when one fact grows two carriers. The two forms do not mix
    /// in class — [`super::shapes::calls::lit_arg_tail_call`] admits a literal
    /// only beside formals that are already in place — but they share the field,
    /// so no consumer can read one and miss the other.
    MultiArgTailCall { params: Vec<u32>, arg_sources: Vec<SlotArg>, callee_tok: u32 },
    /// Framed non-leaf `return g(a) + k` (k ≠ 0): exactly one int-returning CALL
    /// whose argument region is exactly the single passthrough LOAD, a `55 <int>`
    /// call-end, then exactly one literal `+ k` (ADD, commutative), returned. A
    /// zero `k` is NOT framed — it folds to [`BodyShape::IntTailCall`].
    ///
    /// `params` are the formals and `arg_ops` is the argument — a bare `[Load]`
    /// of one of them, which is **not necessarily the formal already in r3**.
    /// Both are carried for the same reason [`BodyShape::IntTailCall`] carries
    /// them: the argument register move is a function of the formal's position,
    /// and dropping the list here made the emitter assume position 0 (a live
    /// wrong-bytes emit — `c2_core::codegen::framed_call_text`).
    FramedCall { add_k: i32, callee_tok: u32, params: Vec<u32>, arg_ops: Vec<IlOp> },
    /// **Class A many-calls** (#35 step 2, rung 1): a framed body that is a
    /// sequence of statement-position calls — results discarded — with **no value
    /// live across any call**, so nothing is saved and the frame is the shipped
    /// 96-byte Class A one.
    ///
    /// ```text
    ///   void f()           { g1(); g2(); }        3-word prologue, bl, bl, epilogue
    ///   void f(int a)      { g1(a); g2(); }       a dies at the first call
    ///   void f()           { g1(1); g2(2); }      li r3,k before each bl
    ///   int  f(int a)      { g1(a); return 5; }   li r3,5 after the last bl
    ///   int  f()           { g1(); return g2(); } the last call's value IS the result
    ///   int  f()           { g1(); return g2()+1; }
    /// ```
    ///
    /// **The last call is NOT a tail call.** Measured: every one of the bodies
    /// above ends `bl <callee>` … `addi r1,r1,96` … `blr`, never `b <callee>` —
    /// the tail-call transform is off the moment the function is framed
    /// (`docs/CODEGEN_FRAMED_CALLS.md` §7 rung 1, byte evidence in
    /// `docs/CODEGEN_PPC_MVP.md`). A *single* statement call with nothing after it
    /// **is** tail-called (`void f(int a){ g(a); }` → a bare `b g`), so that case
    /// is routed to [`BodyShape::IntTailCall`]/[`BodyShape::MultiArgTailCall`]
    /// instead and never reaches here.
    ///
    /// The Class A / Class B boundary is a liveness one: a formal read after the
    /// first call has to survive a `bl`, and `void f(int a,int b){ g1(a);
    /// g2(b); }` puts `b` in `r31` with a `std`/`ld` pair around the frame.
    /// [`Self::CallSeq::saved`] carries which formals those are; see
    /// [`super::shapes::plan_saved_gprs`] for the rule, the refutation ladder and
    /// the two boundaries it refuses by name.
    CallSeq {
        params: Vec<u32>,
        calls: Vec<SeqCall>,
        tail: SeqTail,
        /// **Class B**: the parameter indices copied into callee-saved GPRs,
        /// taking `r31`, `r30`, … in this order. Empty is Class A (nothing
        /// survives a call). See [`super::shapes::plan_saved_gprs`] for the rule
        /// and the refutation ladder behind it.
        saved: Vec<usize>,
        /// **W10** — `Some` when the first call (and, with an `else` arm, the
        /// second) is guarded by a conditional branch. `None` for every
        /// sequence the Class A/B rungs already shipped, so this field is the
        /// whole of the framed × branching widening on the IL side.
        guard: Option<SeqGuardShape>,
        /// **W11** — the guarded early returns written ahead of the sequence,
        /// in source order. Empty for every sequence the earlier rungs shipped.
        ///
        /// A sibling of `guard` and not a variant of it: these guards *leave*
        /// the function where `guard`'s falls through, so they emit a different
        /// block, a different branch target and — for a value arm — an
        /// intra-section `b` that `guard` never needs. Both non-empty at once is
        /// refused, so the two block plans cannot interleave.
        early: Vec<SeqEarlyReturnShape>,
    },
    /// W6 comparison leaf: `return <formal> <rel> <literal>;` materialized to a
    /// boolean branchlessly and converted back to `int`/`unsigned`.
    Compare(CompareLeaf),
    /// **The pointer-walk accumulate loop** — the first body class here with a
    /// **back edge**. See [`super::shapes::ptr_walk_loop`] for the whole
    /// accept/refuse boundary and [`crate::func::PtrWalkModLoop`] for the fields.
    PtrWalkModLoop(crate::func::PtrWalkModLoop),
    /// **W-CFG1 — the two-armed `if`/`else` whose arms are CALLS and whose join
    /// is a real block.** The first `cflow-if-n` body this crate accepts. See
    /// [`super::shapes::if_call_join`] for the whole accept/refuse boundary and
    /// [`crate::func::IfCallJoin`] for the fields.
    IfCallJoin(crate::func::IfCallJoin),
    /// **W-EXTDATA — a `||` guard chain SUNK to the end of the function and
    /// TAIL-MERGED with a second error block, around one call whose first
    /// argument is the ADDRESS OF A FUNCTION** (`_vswprintf_s_l`). See
    /// [`shapes::guard_chain_shared_tail`] for the whole accept/refuse boundary
    /// and [`crate::func::GuardChainSharedTail`] for the fields.
    GuardChainSharedTail(crate::func::GuardChainSharedTail),
    /// **W-DATA — the static-array scan loop.** The first body class here whose
    /// function DEFINES the data it reads. See
    /// [`super::shapes::static_scan_loop`] for the whole accept/refuse boundary
    /// and [`crate::func::StaticScanLoop`] for the fields.
    StaticScanLoop(crate::func::StaticScanLoop),
    /// **The body-parameterized pointer-walk loop** — the first shape here
    /// whose emitted body has no fixed length. See
    /// [`super::shapes::ptr_walk_chain_loop`] for the accept/refuse boundary
    /// and [`crate::func::PtrWalkChainLoop`] for the operation list that is the
    /// whole difference from the variant above.
    PtrWalkChainLoop(crate::func::PtrWalkChainLoop),
    /// **The integer divide/modulo leaf** — `return a / b;` / `return a % b;`
    /// over two formals. See [`super::shapes::div_mod_leaf`] for the whole
    /// accept/refuse boundary and [`crate::func::DivModLeaf`] for the fields.
    DivModLeaf(crate::func::DivModLeaf),
    /// **W43** — `return ((unsigned)(P != 0) << SH) | C;`. See
    /// [`crate::func::CmpShiftOr`].
    CmpShiftOr(crate::func::CmpShiftOr),
    /// **W8 — the two-arm conditional tail call.** The first shape in this enum
    /// whose lowering emits a conditional branch. See
    /// [`super::shapes::cond_tail`] for the grammar and for the three
    /// measurements that draw the class boundary.
    CondTailPair(CondTailPairShape),
    /// W13a floating-point leaf: a straight-line chain over float (or double)
    /// *parameters* — no constants, no conversions, no contraction.
    FloatLeaf { params: Vec<u32>, ops: Vec<IlOp>, double: bool },
    /// An **empty function body** (`void f() {}`): the body opens directly on the
    /// `3A` assign of the return plumbing with no expression before it. Emits a
    /// bare `blr`.
    EmptyBody,
    /// The **compiler-generated empty destructor** that destroys exactly one
    /// sub-object and nothing else: either its single non-virtual **base** through
    /// the `this`-adjust intrinsic at adjust 0, or a single destructible **member**
    /// at byte offset `adjust` reached by a plain `27` offset add
    /// (`docs/IL_CALL_IN_EXPR.md` §5, §15). The call has no result and nothing
    /// follows it, so the whole function is
    /// `[addi r3,r3,adjust ;] b <sub-object-dtor>`.
    ///
    /// `adjust == 0` emits exactly what [`BodyShape::VoidTailCall`] emits; a
    /// nonzero `adjust` prepends the one `addi`, expressed as the argument-setup
    /// operand stream `[Load(this), Lit(adjust), Add]` so it lowers through the
    /// existing integer tail-call emitter rather than a new one (`bundle.rs`).
    /// Kept as its own variant so the census can attribute the movement, and
    /// because its grammar admits two opaque trailers that must not be admitted
    /// anywhere else. See [`shapes::try_parse_empty_dtor_delegation`].
    ///
    /// `eh` is the `/EHsc` bit of the two opaque trailers (`/EH…` clears `0x10`
    /// in both), and it is **not** decoration: this body is `eh-bare` and an
    /// `eh-bare` function costs one extra label-counter slot, so the same source
    /// compiled with and without `/EHsc` emits the same four bytes of `.text`
    /// and different `$M`/`$T` numbers for every framed function behind it.
    /// See [`crate::IlFunction::eh_bare`].
    EmptyDtorDelegation {
        callee_tok: u32,
        this_tok: u32,
        adjust: i32,
        sub_object: DtorSubObject,
        eh: bool,
    },
    /// **WEC — the empty constructor that delegates to ONE base sub-object**:
    /// `struct D : B { D(); };  D::D() {}`. The mirror image of
    /// [`BodyShape::EmptyDtorDelegation`]'s base form, and *not* a leaf: an MSVC
    /// constructor returns `this` in r3, `this` is live across the base
    /// constructor's `bl`, so c2 frames the body and homes `this` in `r31`.
    ///
    /// ```text
    ///   mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
    ///   mr r31,r3 ; bl <base ctor> ; mr r3,r31
    ///   addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
    /// ```
    ///
    /// `unwind_tok` is the base **destructor** the IL names as the unwind action
    /// for the sub-object that just went live. It emits **nothing** — no `bl`,
    /// no relocation, and no symbol — and it is carried only so the TU-level
    /// unclaimed-`.gl`-symbol gate can account for a name that is in `.gl` and
    /// legitimately absent from the obj. `None` when the base has no destructor,
    /// which is the same body with the whole second half of the statement and
    /// the `5C`/`5D` trailers missing (`eh-none`, and byte-identical `.text`).
    ///
    /// See [`shapes::try_parse_empty_ctor_base_delegation`].
    EmptyCtorBaseDelegation {
        callee_tok: u32,
        this_tok: u32,
        params: Vec<u32>,
        unwind_tok: Option<u32>,
        eh: bool,
    },
    /// An **indirect-load leaf**: the whole body is one load through a pointer
    /// (`return *p;`, `return s->m;`, `return p[k];`, `return mMember;`), which c2
    /// lowers to a single `lwz rD, off(rBase)`. `ops` is always exactly
    /// `[Load(base), LoadInd { off }]` and `params` includes a member function's
    /// `this` at index 0. See [`try_parse_indirect_load_leaf`].
    IndirectLoad { params: Vec<u32>, ops: Vec<IlOp> },
    /// An **address leaf**: the whole body is one sub-object *address*
    /// (`return &s->m;`, `return &p->Base::m;`, `return s->arr;`), which c2
    /// lowers to a single `addi rD, rBase, off` — or to nothing at all when
    /// `off` is 0. `ops` is always exactly `[Load(base), AddrOf { off }]` and
    /// `params` includes a member function's `this` at index 0.
    ///
    /// Kept apart from [`BodyShape::IndirectLoad`] because the two differ by the
    /// single `30` token and emit different instructions — admitting one as the
    /// other is a wrong-bytes emit, not a gap. See [`shapes::try_parse_addr_leaf`].
    AddrLeaf { params: Vec<u32>, ops: Vec<IlOp> },
    /// A **store leaf**: the whole body is one store through a sub-object
    /// designator (`s->m = v;`, `p->Base::m = v;`, `s->arr[2] = v;`, `*p = v;`,
    /// `s->m = 7;`), which c2 lowers to a single `stb`/`sth`/`stw`/`std` at a
    /// folded displacement — plus one `li` when the stored value is a literal.
    /// `ops` is always exactly `[Load(base), Load(value) | Lit(k),
    /// StoreInd { off, width }]` and `params` includes a member function's
    /// `this` at index 0.
    ///
    /// Kept apart from [`BodyShape::IndirectLoad`] and [`BodyShape::AddrLeaf`]
    /// for the reason those two are kept apart from each other: the three
    /// designate the same address and emit three different instructions, so
    /// admitting one as another is a wrong-bytes emit rather than a gap. See
    /// [`shapes::try_parse_store_leaf`].
    StoreLeaf { params: Vec<u32>, ops: Vec<IlOp> },
    /// A **store run**: a whole body that is a *sequence* of the store
    /// statements [`BodyShape::StoreLeaf`] admits one of, ending on the void
    /// return plumbing or on a `return *this` — `void S::set(int u,int v)
    /// { a = u; b = v; }`, `T& T::set(int u,int v){ a=u; b=v; return *this; }`.
    /// `ops` is the statements' op groups concatenated, in **source order**,
    /// each `[Load(base), Load(value) | Lit(k), StoreInd { off, width }]` or
    /// `[Load(base), StoreIndFp { off, double, src }]`.
    ///
    /// Kept apart from [`BodyShape::StoreLeaf`] because it is a different
    /// production with two gates the single store does not have (no literal
    /// value in a run, no two statements writing overlapping bytes of one base)
    /// and a tail the single store does not admit. See
    /// [`shapes::try_parse_store_run`].
    StoreRun { params: Vec<u32>, ops: Vec<IlOp> },
    /// **F3 — a store run followed by a CALL**, the composition
    /// `src/xdk/nuispeech/xboxheap.cpp`'s constructor is: a
    /// [`BodyShape::StoreRun`]'s statements, then one statement-position member
    /// call on `this` whose argument setup is **empty**, then the constructor's
    /// `return this`.
    ///
    /// Kept apart from [`BodyShape::StoreRun`] because it is a different
    /// production with a gate the run does not have — board #1129's *"the call's
    /// argument setup writes `r3`"* regime boundary — and because it is
    /// **framed** where the run is a leaf: only the constructor form frames at
    /// all, the `void`, `return <call>` and discarded-`int` forms are frame words
    /// 0 and tail-call behind the run (board #1131).
    ///
    /// **The composition carrier is board #844 and it LANDED** (`w-seam2`).
    /// `shape_to_function` maps this variant onto a [`crate::func::CallSeq`]
    /// whose [`crate::func::CallSeq::store_run`] holds the run, whose `saved` is
    /// the receiver and whose tail is
    /// [`crate::func::SeqTail::SavedFormal`] — the same sequence the generated
    /// base-delegating constructor already used, with a run in front of it.
    ///
    /// **`IlFunction::ops` stays EMPTY for this shape, and that is the whole
    /// repair.** Before the carrier, `ops` and the call fields were
    /// *alternatives* `c2_core::codegen::select` tried in a fixed order, so a
    /// function carrying both emitted one and silently dropped the other — a
    /// store run without its `bl` is board #232's exact shape, and #232 was live
    /// for 255 commits. An ordering fix would leave two fields that can both be
    /// set; carrying the run *inside* the sequence leaves nothing for a dispatch
    /// order to get wrong. [`crate::func::IlFunction::store_run_carried_twice`]
    /// is the backstop and `select_function` refuses a violation by name.
    ///
    /// The EMITTER's own domain is narrower than this production's: it serves
    /// runs of formal- and literal-valued stores only, and refuses an `AddrOf`
    /// value (the mixed-kind run of boards #836/#868) and the `nprod == 0,
    /// u <= 1` corner where the copy's slot rule is not measured. See
    /// `c2_core::codegen::store_run_call`. See
    /// [`shapes::try_parse_store_run_call`] for the whole accept/refuse boundary.
    StoreRunCall {
        params: Vec<u32>,
        ops: Vec<IlOp>,
        callee_tok: u32,
        /// **How many argument slots the call occupies, receiver included.**
        ///
        /// Every slot `i` holds `params[i]` by this production's own gate, so
        /// this is exactly *which formals are still live at the `bl`* — and that
        /// turned out to decide the RUN's emitted order, not just the call's.
        /// See [`crate::func::StoreRunPrefix`] for the three bodies that
        /// separate it. Carried rather than recomputed because the emitter sees
        /// an EMPTY argument setup by construction and cannot count the slots.
        live_args: usize,
    },
    /// **#839 — a store run whose base is a C++ REFERENCE BIND.**
    ///
    /// `auto& listHead = mListHead;` — the spelling
    /// `src/xdk/nuispeech/xboxheap.cpp` actually ships — which `c1xx` writes as a
    /// store into a **local**, whose token then stands in later stores' BASE
    /// position. Two reader obligations, not one (board **#1160**,
    /// `w-f23` §5.1): a `26 <tok>` local admitted as a store *destination*, and
    /// that local admitted as a store *base* **carrying its own base symbol**.
    ///
    /// **The bind is NOT folded into the formal, and that is the whole point.**
    /// `w-heap` §4.2 (board **#1128**) measured that the same constructor with
    /// and without the bind emits **different bodies** — both producers swap and
    /// one store moves — and this lane reproduced it from its own captures
    /// (`work/w-bind/grid/b_target_{bind,direct}/dis.txt`, four words apart). A
    /// reader that rewrote `l.mNext` to `this->mListHead.mNext` would hand the
    /// emitter the *other* body's op stream, which is board #232's direction.
    /// So the run's ops keep `IlOp::Load(<local>)` in the base position and the
    /// binding travels beside them in [`Self::binds`], undischarged.
    ///
    /// **`shape_to_function` returns `None` for this variant** — there is no
    /// carrier, and the residue is filed under [`STORE_RUN_BIND_NO_CARRIER`].
    /// Two independent things are missing and both are `crates/c2-core`'s:
    /// `IlFunction` cannot spell "a local bound to formal + offset", and
    /// `codegen::alloc`'s mixed-kind refusal (boards #836/#868) is live on the
    /// target body anyway. See [`shapes::try_parse_store_run_bind`].
    StoreRunBind {
        params: Vec<u32>,
        /// The bindings, in source order, each `local := formal + off`.
        binds: Vec<RefBind>,
        /// The run's op groups. A store through a bound local carries
        /// `IlOp::Load(<local token>)` as its base and the offset **inside** the
        /// bound object — never the sum.
        ops: Vec<IlOp>,
        /// `Some(callee)` when the run is followed by board #1129's call, i.e.
        /// the [`BodyShape::StoreRunCall`] tail. `None` for the plain run tail.
        callee_tok: Option<u32>,
        /// [`BodyShape::StoreRunCall::live_args`], or 0 without a call.
        live_args: usize,
    },
}

/// One `local := <formal> + off` reference binding, as board **#839** spells it.
///
/// Deliberately **not** a substitution: `off` is carried so a future emitter can
/// discharge it, and is never added into a store's own displacement by the
/// reader, because the sum is what makes the two source spellings' op streams
/// identical and their emitted bodies are not (board #1128).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RefBind {
    /// The bound local's `.ex` token — the one that stands in a store's base
    /// position, and the one whose membership in
    /// [`crate::func::sy::SyView::ptr_locals`] is checked **positively**.
    pub(crate) tok: u32,
    /// The formal the bound object hangs off.
    pub(crate) base_tok: u32,
    /// The byte offset of the bound sub-object within it. Never 0 — see
    /// [`shapes::try_parse_store_run_bind`] for the measurement that excludes it.
    pub(crate) off: i32,
}

/// **Why** a function segment fell outside the modeled class (P2b census).
///
/// The positive parser fails closed at the *first* byte it cannot account for.
/// Recording that point — the grammar production it was in, the offending byte,
/// and the offset — turns an opaque `None` into a rankable census key: over a
/// real workload the histogram of [`Block::feature`] *is* the widening order
/// (docs/ROADMAP.md §G5/P2b). Purely diagnostic: acceptance is unchanged, and
/// [`parse_segment`] still returns a bare `Option`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    /// The grammar production the parse was inside (`"expr"`, `"call-end"`, …).
    pub ctx: &'static str,
    /// The byte that could not be consumed. `None` at end-of-segment — and also
    /// for a refusal that is **not about a byte at all** (a post-parse predicate
    /// over the operand stream, a width the `.sy` layer withheld, a name-level
    /// refusal). Those two cases are told apart by [`Self::off`] against
    /// [`Self::seg_len`], never by this field alone; see [`Self::at_end`].
    pub byte: Option<u8>,
    /// Byte offset within the function segment.
    pub off: usize,
    /// **The length of the segment [`Self::off`] indexes.** Carried so
    /// [`Self::feature`] can say whether the refusal was raised at the end of the
    /// segment or in the middle of it — an offset is meaningless without the
    /// frame it indexes, so the two travel together and no producer records one
    /// without the other.
    ///
    /// **Why this field exists.** `:eof` is a *ranking signal*, not decoration: a
    /// key that ends `:eof` says the refusal was raised **after** the parse
    /// reached the segment end, so every function under it is grammar-complete by
    /// construction and no second blocker hides behind it. That property is what
    /// makes such a row directly sizeable. The renderer used to print `:eof` for
    /// **any** `byte: None`, and ~73 producers raise a byte-less refusal at a
    /// *mid-segment* cursor — so the signal was being claimed by rows that had
    /// not parsed past their own blocking point. `assign-dst-not-formal:eof` was
    /// ranked at 13,887 on that reading and measured **+0** twice; 4,466 of its
    /// bodies were `cflow-loop` bodies, which cannot be at the end of anything.
    /// See `docs/GAPS.md` §6.
    pub seg_len: usize,
    /// Context payload for the operand-*type* blocks, where the single blocking
    /// byte is uninformative: an operand's 3-byte inline type differs from the
    /// modeled `86 41 74` (int), but the first byte `86` is shared by every
    /// type, so reporting it buckets `unsigned`, `float`, `pointer`, … together.
    /// Packed big-endian in the low 24 bits; 0 when unused.
    ///
    /// **Why `u64`.** The `26`-in-expression family (`mcall`) packs a *pair* of
    /// constructs here — the receiver form and the construct that blocks the body
    /// **after** it (`docs/IL_CALL_IN_EXPR.md` §16) — and the pair does not fit in
    /// 32 bits without truncating an intrinsic selector, which would silently
    /// *merge* two census buckets. Merging buckets is the one failure a census
    /// instrument cannot survive, so the field is widened instead of squeezed.
    /// Every other producer uses the low 24 bits exactly as before.
    pub aux: u64,
}

/// Census `ctx` for a function whose body parses in class but whose
/// optimization-settings word is not one this port emits under.
///
/// Raised **after** the body parse and only for an otherwise-in-class function,
/// deliberately: gating it up front would replace every real function's actual
/// blocking feature with this one and destroy the histogram that ranks the
/// roadmap. Applied last, it removes exactly the over-claim and nothing else.
pub(crate) const OPT_MODE: &str = "opt-mode";

/// Census `ctx` for a **pointer-walk accumulate loop outside `/O1`**.
///
/// Its own key beside [`OPT_MODE`] rather than folded into it, because the two
/// say different things: `opt-mode` means *this port has never been verified at
/// this mode*, and this means *it has, and `c2` emits a different body here*.
/// `/Ox` and `/O2` compile the class's own source to twenty-one words against
/// `/O1`'s twenty — a strength-reduced multiply, a hoisted trap and an explicit
/// `cmpli` loop close (`c2_core::codegen::ptr_walk_loop`).
///
/// It is raised in the census and not only in codegen so that the two agree:
/// `crates/c2-harness/tests/census_gate.rs` asserts that every function the
/// census calls in class is one `PortC2` emits, and a mode-conditional refusal
/// that lived in codegen alone would be an error term on the published
/// numerator (`docs/GAPS.md` §6, roadmap #44).
pub(crate) const PTR_WALK_LOOP_NOT_O1: &str = "ptr-walk-loop-not-o1";

/// Census `ctx` for a **body-parameterized pointer-walk loop outside `/O1`**.
///
/// Its own key beside [`PTR_WALK_LOOP_NOT_O1`], not folded into it, for the
/// reason that one is not folded into [`OPT_MODE`]: the two shapes are refused
/// at `/Ox` for the same *kind* of reason but they are different classes, and a
/// shared key would make a histogram row that no rung could size. Every cell
/// behind this shape's schedule, allocation and entry form was captured at
/// `/O1` (`docs/rungs/2026-08-05-w-varloop.md`).
///
/// Raised in the census and not only in codegen so the two agree —
/// `crates/c2-harness/tests/census_gate.rs` asserts every function the census
/// calls in class is one `PortC2` emits, and a mode-conditional refusal living
/// in codegen alone is an error term on the published numerator.
pub(crate) const PTR_WALK_CHAIN_LOOP_NOT_O1: &str = "ptr-walk-chain-loop-not-o1";

/// Census `ctx` for a body that parses as a call shape whose callee token has no
/// `.gl` symbol. See the census for why this is a refusal and not a fallback.
pub(crate) const CALLEE_UNRESOLVED_TAIL: &str = "callee-unresolved-tail-call";

/// **F3's residue key** — the body is a store run followed by a call, it parses
/// **to the end of the segment**, and the only thing left wrong with it is that
/// `IlFunction` has no carrier for the composition.
///
/// Its own key rather than one of the `callee-unresolved-*` family, because the
/// callee resolves perfectly in every one of these bodies: nothing about the
/// *symbol* is missing. What is missing is the seam board **#844** owns — `ops`
/// and the call fields are alternatives in `c2_core::codegen::select`, so a
/// function carrying both emits one and drops the other, which is board #232's
/// direction. Filing it under `callee-unresolved-tail-call` would name the wrong
/// construct and hide the population #844 is sized from.
///
/// Raised with [`Block::at_end`], and this shape is entitled to it: the arm runs
/// only for a body the whole-segment parser already accepted, and acceptance
/// requires the cursor to reach `seg.len()`. So the `:eof` it renders is the
/// true statement — the body is grammar-complete and directly sizeable.
pub(crate) const STORE_RUN_CALL_NO_CARRIER: &str = "store-run-call-no-emitter-carrier";

/// **W-DATA — the body is a static-array scan loop and the OBJECT it reads is
/// outside the class.**
///
/// Its own key rather than one of the `callee-unresolved-*` family, for the
/// reason [`STORE_RUN_CALL_NO_CARRIER`] has one: nothing about a *callee* is
/// missing here, and nothing about the body is either — it is grammar-complete
/// and this parser accepted it. What refused is
/// `Bindings::resolve_data_def`, over the object the body subscripts: it is
/// not a COMDAT (a namespace-scope `static`, placed *before* `.text` — board
/// #1682), or not initialized (a `.bss` COMDAT), or thread-local, or its `.in`
/// value did not decode to exactly its `.gl` size.
///
/// **Filing it under `callee-unresolved-tail-call` is what this key exists to
/// stop, and that is not hypothetical**: GRID B's `n0` and `n1` cells read
/// exactly that before this constant existed, so two cells that refuse in the
/// OBJECT resolver were labelled with a refusal about a symbol they do not
/// have. A residue nobody can name is a residue nobody can size.
///
/// `Block::at_end` is earned for the same reason the constant above earns it:
/// the arm runs only for a body the whole-segment parser already accepted, so
/// the `:eof` it renders is the true statement.
pub(crate) const STATIC_SCAN_LOOP_OBJECT: &str = "static-scan-loop-object-out-of-class";

/// **#839's residue key** — the body is a store run whose base is a C++
/// reference bind, it parses **to the end of the segment**, and what is left
/// wrong with it is that nothing downstream can spell the binding.
///
/// Its own key, and not [`STORE_RUN_CALL_NO_CARRIER`]'s, because the two are
/// blocked on different things: F3's composition seam landed (board #844,
/// `w-seam2`) and these bodies still cannot be emitted. `IlFunction` has no
/// field that says *"this token is `params[i]` plus 8"*, and inventing one is a
/// `crates/c2-core` change with an emitted-order claim behind it — the two
/// spellings' bodies differ by four words, so a carrier that discharged the
/// binding into the displacement would emit the wrong one.
///
/// Raised with [`Block::at_end`], and this shape is entitled to it for
/// [`STORE_RUN_CALL_NO_CARRIER`]'s reason: the arm runs only for a body the
/// whole-segment parser accepted, and acceptance requires the cursor to reach
/// `seg.len()`. The `:eof` is the true statement, so the row is directly
/// sizeable rather than hiding a second blocker.
///
/// **Board #1199 landed and this key's DOMAIN SHRANK to one case** (`w-carrier`).
/// The carrier exists — [`crate::func::IlOp::BoundAddr`] — so a bind body is no
/// longer blocked on *having nowhere to put the fact*. What remains under this
/// name is the residual: a bind body whose callee token does not resolve, and
/// anything a future widening leaves unclassified. The four things that DO block
/// a bind body today each carry their own key below, because a shared one would
/// make each of their residues unsizeable — and one of them is the frontier's
/// last refusal.
pub(crate) const STORE_RUN_BIND_NO_CARRIER: &str = "store-run-bind-no-emitter-carrier";

/// **#836/#868's residue key, and the reason board #1199 is worth paying** —
/// the bind body's run puts the bound name in a store's **VALUE** position
/// *beside a literal*, so the run has two producers of different kinds: an
/// interior address (`addi rD,rBase,off`) and a constant (`li`).
///
/// `codegen::alloc::allocate` refuses a mixed-kind run **wholesale**. That is not
/// caution: over 81 mixed cells graded against real `c2.dll`, clause 1 alone is
/// wrong on 29, clause 2 alone on 35, `w-next`'s key on 20, and the refusal on
/// **0** (board **#836**). The narrow lift — clause 1 where it decides with no
/// tie — was measured over 36 cells and is **12 MISS** (board **#868**): the
/// `addi`-interior spelling is 12/12 and `slwi` is 0/12, and `ProducerKind`
/// cannot tell the two apart. `w-heap`'s own `j1_lit2` refutes clause 1 on this
/// exact mix (board **#1134**).
///
/// **`src/xdk/nuispeech/xboxheap.cpp` lands here**, and that is the point of the
/// key: before board #1199 the target was blocked on a missing representation
/// and #868/#836 could not be *measured* at all, because nothing reached the
/// allocation question. Now it is one named, countable row.
///
/// # The allocation question was asked, and it is still open — board #1264
///
/// Lane `w-mixkind` took `w-prod`'s widened carrier (`alloc::Root::base`) and
/// stated the first allocation key that could hold the fact nine of the ten
/// dead keys were missing. **GRID X** — 66 cells, `sha256` and every rival's
/// predictions committed before one compiled, 66 reached, **66 graded, 0 OOR,
/// 0 compile-failed**, all against real `c2.dll` at the workload's own flags:
///
/// ```text
///   H-CHAIN   2 wrong of 60 in domain        <== the eleventh death
///   H-2Z      8 wrong        H-STEP  4       H-DEPTH  4      H-2X  19
///   the shipped REFUSAL      0 wrong of the same 60
/// ```
///
/// So **this key does not move**, and the reason is not caution: eleven rules
/// have now been fitted at this seam and the refusal has out-scored every one
/// of them on every holdout ever built.
///
/// **Two things the lane established that a successor needs.**
///
/// * The relation the bytes obey is the **transitive bind lineage** — the value
///   root is neither an ancestor nor a descendant of the store root through bind
///   links — and `alloc::Root::base` holds **one link** of it. GRID X's
///   `DEEP-GP` prices the gap. Board #1244 named the missing element correctly
///   and under-scoped how much of it is missing.
/// * **This key is not the whole rung.** Lifting it needs
///   `codegen::leaf::store::parse_simple_gpr_run` to admit a bound **VALUE**;
///   without that the reader accepts, `PortC2` refuses, and the scan prints
///   `census/gate disagreement: 1` (`w-mrslot` §5, re-derived rather than
///   carried at `w-mixkind` §5). A lane that lifts this clause alone breaks the
///   invariant `codegen::select::function_gate` exists to hold.
pub(crate) const STORE_RUN_BIND_MIXED_KIND: &str = "store-run-bind-mixed-kind-alloc";

/// The bind body's run puts the bound name in a store's **value** position with
/// **no** literal beside it — one register-derived producer and nothing to mix
/// with, so board #836's refusal does not apply.
///
/// Refused anyway, and the reason is a MEASUREMENT rather than caution:
/// `work/w-carrier/grid/k_both1`, `k_both2` and `k_val1` are **byte-identical**
/// to their direct twins (`k_both1_c`, …), and the direct twin is the F2
/// address-valued run, which `codegen::leaf::store` refuses — its group is four
/// ops where every group the emitter models is three. Emitting one spelling of a
/// pair whose objs are identical while refusing the other is a divergence with no
/// grid behind it, so this lane declines the family and names it. See
/// `docs/rungs/2026-08-08-w-carrier.md`.
pub(crate) const STORE_RUN_BIND_ADDR_PRODUCER: &str = "store-run-bind-address-producer";

/// The bind body's run carries **more than one distinct producer**.
///
/// With one symbol `codegen::order::store_order` is exact to three producers; a
/// bind *is* a second base symbol (board #1128), and on more than one symbol the
/// walk can fail outright — `work/w-carrier/grid/k_2const` is such a cell, and
/// real `c2` emits source order there where the model has no answer at all. The
/// gate is drawn at **one** producer, which is the region `w-carrier` proved
/// `store_order` cannot refuse, rather than at `MAX_MULTISYM_PRODUCERS`.
pub(crate) const STORE_RUN_BIND_MULTI_PRODUCER: &str = "store-run-bind-multi-producer";

/// The bind body's run crosses more than
/// `codegen::order::MAX_SYMBOL_CROSSINGS` base-symbol group boundaries.
///
/// `layout_slots` is exact only while a producer's value crosses at most two
/// symbol-group boundaries before it is first consumed; past that the clause is
/// 98.6 % and board **#621** measured a rival that answers the whole population
/// at 99.44 % / 97.30 % and refused to ship it. The count is taken over the whole
/// run in SOURCE order, which is an upper bound on the emitter's own `nsw`
/// because the emitted symbol pattern is always the source pattern (board #601,
/// 7,589 of 7,589 cells) — so this gate is provably at least as strict as the
/// one it stands in for.
pub(crate) const STORE_RUN_BIND_SYMBOL_CROSSINGS: &str = "store-run-bind-symbol-crossings";

/// **THE REFUSAL THE SWEEP EARNED, AND THE CORRECTION THAT RETIRED IT** — board
/// #1212, closed by `w-mrslot`.
///
/// ```text
///   H::H(unsigned a, unsigned b) { BE& lh = mListHead; mCount = 0;
///                                  lh.mNext = (BE*)this; Reset(); }
///   real c2:  li 11,0 ; mr 31,3 ; stw 11,20(3) ; stw 3,8(3) ; bl
///   the port: li 11,0 ; stw 11,20(3) ; mr 31,3 ; stw 3,8(3) ; bl
/// ```
///
/// **The copy landed after ZERO stores and board #867's rule said one.** Three
/// `88-store-run-call` cases and 56 cross cells graded `Port=Mismatch` on
/// `w-carrier`'s first emitter, whose own 53-cell frozen grid was green through
/// every one (board #1211). The mechanism is `codegen::store_run_call`'s own
/// documented shortcut: `save_slot(nprod, u)` was fed the **COUNT** of unproduced
/// stores, and the file argued that equals #584's `u`, the **leading run** of
/// unproduced stores in the *final* order — *"they cannot be [separated]:
/// `store_order` forbids a store whose producer has rank `j` from occupying a
/// position below `u + j`, so the leading run is always at least
/// `min(2, total)`"*.
///
/// **That argument holds only on a SINGLE-symbol run**, which is every cell the
/// shortcut was ever measured on. `codegen::order`'s own
/// `the_two_readings_of_u_agree_on_every_single_symbol_run` enumerates 5,460 to
/// say so, and `the_layout_u_is_the_leading_run_not_the_count` exhibits the
/// multi-symbol cell where they differ. **A bind IS a second base symbol** — the
/// whole of board #1128 — so the carrier is exactly what opened the region.
///
/// `w-carrier` refused rather than corrected, and said why: the correction
/// governs every #844 body and would have rested on the four cells that refuted
/// that lane. `w-mrslot` took it on a frozen grid instead — GRID R, 145 cells
/// sha256'd before the first `cl.exe`, 93 with an observed `mr r31,r3`, 30 of
/// them separating the two readings, every quantity read out of real `c2.dll`'s
/// own emitted words:
///
/// | reading of #584's `u` | HIT | MISS |
/// |---|---:|---:|
/// | **leading run** | **93** | **0** |
/// | count | 63 | **30** |
///
/// **The key is retained with no producer.** It is `store-run-bind-call-tail-mr-slot`
/// in every scan, rung and board row written before 2026-08-09, and a key whose
/// text is deleted cannot be matched against those records — `docs/GAPS.md` §7's
/// "a lane nobody enumerates is a lane that does not run", applied to a census
/// key. `the_call_tail_key_has_no_producer_since_1212` is the invariant that it
/// stays that way.
// Retired keys have no producer by definition, so the only consumer is the
// invariant that says so — `the_call_tail_key_has_no_producer_since_1212`.
#[allow(dead_code)]
pub(crate) const STORE_RUN_BIND_CALL_TAIL_RETIRED: &str = "store-run-bind-call-tail-mr-slot";

/// The bind body's run is not a stream of three-op GPR store groups.
///
/// A floating-point group is two ops, an F2 address-valued group is four, and a
/// load-valued one is four; `codegen::leaf::store::parse_simple_gpr_run` matches
/// exactly three. Refused positively so a bind body cannot reach an emitter
/// through a group shape nothing graded.
pub(crate) const STORE_RUN_BIND_GROUP_SHAPE: &str = "store-run-bind-group-shape";

/// **Board #1199** — why a [`BodyShape::StoreRunBind`] body is refused, or `None`
/// when the bind itself is fine and the refusal is somewhere else (an
/// unresolvable callee).
///
/// This is `shapes::bind_run_ops` asked a second time for its *reason*, and it is
/// the **same** function `crate::func::bundle::shape_to_function` decides
/// acceptance with — one decision procedure, two callers, which is `GAPS.md` §6's
/// rule in the form that matters here: the key the census prints and the answer
/// the model gives cannot drift.
pub(crate) fn bind_refusal_key(shape: &BodyShape) -> Option<&'static str> {
    match shape {
        BodyShape::StoreRunBind {
            params,
            binds,
            ops,
            live_args,
            // Board #1212 lifted the call-tail refusal, so `bind_run_ops` no
            // longer needs to know whether there IS a call — the one thing a
            // call changed was which reading of #584's `u` the emitter needed,
            // and that is answered in `codegen::order` now.
            callee_tok: _,
        } => shapes::bind_run_ops(params, binds, ops, *live_args).err(),
        _ => None,
    }
}

pub(crate) const CALLEE_UNRESOLVED_DTOR: &str = "callee-unresolved-dtor-delegation";
pub(crate) const CALLEE_UNRESOLVED_FRAMED: &str = "callee-unresolved-framed-call";
pub(crate) const CALLEE_UNRESOLVED_SEQ: &str = "callee-unresolved-call-sequence";

/// **WR1** — census `ctx` for a body that parses as a tail call materializing a
/// data symbol's address whose token has no `.gl` symbol name at all. The
/// dominant member is a **string literal**: its record carries the `25`
/// separator `gl::gl_symbol_index` excludes, and admitting it needs a `.rdata`
/// emitter this port does not have, in two different forms
/// (`docs/IL_CALL_IN_EXPR.md` §17.2 items 2–4).
///
/// Its own key, and not `CALLEE_UNRESOLVED_TAIL`, because the callee resolves
/// perfectly well in every one of these bodies — filing them under the callee's
/// name would be the mis-attribution `docs/GAPS.md` §6 keeps recording, and it
/// would hide the one number a follow-on rung has to be sized from.
pub(crate) const DATA_SYM_UNRESOLVED: &str = "data-sym-unresolved";

/// **WR1** — census `ctx` for a data symbol that DOES resolve to a `.gl` name
/// but whose linkage byte does not say *undefined external*: a global defined in
/// this TU, or a static one. Refused because it puts a `.data`/`.bss` section
/// into the middle of the section table and the port emits a fixed shell
/// (`docs/IL_CALL_IN_EXPR.md` §17.2 item 7).
///
/// Kept apart from [`DATA_SYM_UNRESOLVED`] because the two are different jobs:
/// this one needs a section emitter, that one needs a name.
pub(crate) const DATA_SYM_LINKAGE: &str = "data-sym-not-extern";

/// **The grammar-completeness axis** — `docs/ROADMAP.md` §9.11 / §9.14.
///
/// One closed vocabulary for the one question the roadmap ranks by: *is
/// anything hiding behind this row, or is its count directly a widening
/// estimate?* Two independent producers answer it and they answer it in
/// different fields of the rendered key:
///
/// * `mcall`'s completeness walker writes `-whole` / `-whole{k}` / `-more`, and
///   leaves the suffix off when the second construct has no production at all;
/// * the byte-less refusals write `:eof` / `:mid` — whether the parse had
///   reached the end of the segment when the refusal was raised.
///
/// **That is the corruption §9.11 records.** WR1 moved 39,967 functions out of
/// `expr-call-in-expr-data-addr-*` — where they carried the first encoding —
/// into `call-arg-multi-sym` and next-blocker keys, where they carry the
/// second. Nothing was lost and every new name is truthful, but a table built
/// by grepping the key for `-whole` **under-counts that family by 18,931**, and
/// a ranking is exactly such a table. §9.13 had to re-derive the join by hand
/// to check a 1,399-row figure.
///
/// So the reading is a **field with a name**, computed from the block's own
/// state, and never a substring of the rendered key. Grepping the key was the
/// defect; a second, better-informed grep would be the same defect. The variant
/// carries its **provenance** rather than merging the two producers, for the
/// reason [`Block::feature`] refuses to merge `:eof` into the byte-named
/// buckets: the two signals are not the same claim, and a reader that wants
/// them summed can sum them, while a reader given a sum can never take it back
/// apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Complete {
    /// The body's grammar is finished by the named construct(s) — `-whole`,
    /// `-whole{k}`. Its count is directly a widening estimate.
    WholeGrammar,
    /// Measured, and the named constructs together are still not enough
    /// (`-more`). Something is hiding behind the row.
    MoreGrammar,
    /// A `-then-` pair whose second half has **no production**, so no
    /// both-handled figure exists. UNMEASURED is a result, not a zero.
    UnmeasuredGrammar,
    /// The parse had reached the end of the segment (`:eof`) — the same claim
    /// [`Complete::WholeGrammar`] makes, reached by the other producer.
    WholeSegmentEnd,
    /// A byte-less refusal raised with segment still ahead of it (`:mid`).
    PartialSegmentEnd,
    /// **The residue, named and printed.** A keyed byte refusal carries neither
    /// signal: `expr-op-0x27` says nothing about whether anything is behind it.
    /// Reported rather than folded into any of the above, because a totality
    /// claim whose residue is invisible is the failure mode this axis exists to
    /// close.
    NoSignal,
    /// The body is in class. Not a completeness question.
    InClass,
}

impl Complete {
    /// The census-key spelling. Closed set, `complete-` prefixed so it can never
    /// collide with a blocking-feature key.
    pub fn name(self) -> &'static str {
        match self {
            Complete::WholeGrammar => "complete-whole:grammar",
            Complete::MoreGrammar => "complete-more:grammar",
            Complete::UnmeasuredGrammar => "complete-unmeasured:grammar",
            Complete::WholeSegmentEnd => "complete-whole:segment-end",
            Complete::PartialSegmentEnd => "complete-partial:segment-end",
            Complete::NoSignal => "complete-none",
            Complete::InClass => "complete-in-class",
        }
    }

    /// Whether this reading says the body's grammar is finished — the join
    /// §9.13 had to compute by hand across the two encodings.
    pub fn is_whole(self) -> bool {
        matches!(self, Complete::WholeGrammar | Complete::WholeSegmentEnd)
    }
}

impl Block {
    /// This refusal's [`Complete`] reading, from the block's own state.
    pub(crate) fn completeness(self) -> Complete {
        if self.ctx == mcall::CALL_IN_EXPR {
            return mcall::completeness(self.aux);
        }
        match self.byte {
            Some(_) => Complete::NoSignal,
            None if self.off >= self.seg_len => Complete::WholeSegmentEnd,
            None => Complete::PartialSegmentEnd,
        }
    }

    /// A refusal that is **not about a single byte**, raised at `off` in `seg`.
    ///
    /// The constructor for every predicate the byte stream cannot express: a
    /// post-parse property of the decoded operand list, a parameter width the
    /// `.sy` layer withheld, an argument count past the register file. `seg_len`
    /// is *derived* from the segment rather than passed, so a producer cannot
    /// record an offset against the wrong frame — which is the whole reason the
    /// segment is a parameter here instead of the length being one.
    ///
    /// Whether this renders `:eof` or `:mid` is decided by `off` against
    /// `seg.len()` and by nothing else. A producer that means "at the end" says
    /// so with [`Self::at_end`], which is the same thing spelled positively.
    pub(crate) fn refuse(seg: &[u8], off: usize, ctx: &'static str) -> Block {
        Block { ctx, byte: None, off, seg_len: seg.len(), aux: 0 }
    }

    /// A refusal raised **after the parse reached the end of the segment** — the
    /// post-parse gates, which run only on a body that already parsed end to end
    /// (`eat_fn_tail` returns `Ok` only at `p == seg.len()`, so an in-class body's
    /// cursor *is* the segment end).
    ///
    /// This is the one thing a positional test cannot derive on its own: the
    /// producer knows the parse completed, and states it. Stated positively —
    /// "this was raised at the end" — rather than as an absence.
    pub(crate) fn at_end(seg: &[u8], ctx: &'static str) -> Block {
        Block::refuse(seg, seg.len(), ctx)
    }

    /// A short, stable census key naming the blocking *feature*.
    ///
    /// Operand-stream opcodes get a named bucket when the byte's meaning is
    /// verified against a live capture, and a `expr-op-0xNN` bucket otherwise —
    /// the point of the census is to *measure* the unknown vocabulary, so an
    /// honest hex bucket is a result, not a placeholder. Structural blocks
    /// (call-end, return plumbing, formals) name their production instead.
    pub fn feature(self) -> String {
        // Intrinsic-call blocks report their **selector**, which [`Block::aux`]
        // carries as the decoded id (see [`intrinsic_selector`]). This is the
        // whole point of decoding the production: `0x40` alone is one opaque 9 %
        // bucket, while the selector splits it into a handful of named
        // constructs with wildly different lowerings — `fabs` is one
        // instruction, `memcmp` is a 15-instruction loop, and the dominant
        // 2113–2119 class-layout family is a pointer adjustment whose emission
        // depends on its *literal* arguments, not on the id.
        // The per-function optimization-settings word, when it is not one this
        // port emits under. Rendered from [`Block::aux`] for the same reason the
        // intrinsic selector is: the word IS the feature, and `ctx` is a
        // `&'static str`. `docs/OPT_MODE.md` decodes the values.
        if self.ctx == OPT_MODE {
            return format!("opt-mode-{:08x}", self.aux);
        }
        if self.ctx == "expr-intrinsic" || self.ctx == "call-intrinsic" {
            return format!("{}-{}", self.ctx, intrinsic_name(self.aux as i32));
        }
        // **The DIVIDE / MODULO key, with its operand TYPE** (`lane w-divsplit`,
        // board **#816**; see [`expr::EXPR_TYPED_OP`]).
        //
        // A **REFINEMENT** of the two published keys, not a re-key: every value
        // this produces starts with the exact string `expr-op-0x05` or
        // `expr-op-0x06`, so a prefix reader of either is unchanged and the
        // partition can only split, never merge or move sideways. That is the
        // opposite direction from the operand-type coarsening below, and it is
        // asserted by `the_div_mod_key_is_an_exact_refinement`.
        //
        // The `<tag><kind>` spelling is the one `expr-load-type-8641` already
        // uses, and it is deliberately not an interpreted name: the kind's low
        // nibble is the type class (1 signed · 2 unsigned · 3 data pointer ·
        // 4 code pointer · 5 real · 6 aggregate · 7 void, `docs/IL_TYPE_TAGS.md`
        // §1), so `expr-op-0x05-8641` is a signed 4-byte integer division and a
        // float one would read `…-8645`. Naming it `-int` / `-float` would be
        // this file's oldest recorded mistake — `expr_opcode_name` guessed three
        // of six relationals wrong from their numeric order — one level up.
        if self.ctx == expr::EXPR_TYPED_OP {
            let b = self.byte.unwrap_or(0);
            return match self.aux {
                0 => format!("expr-op-0x{b:02X}-notype"),
                a => format!(
                    "expr-op-0x{b:02X}-{:02X}{:02X}",
                    (a >> 16) & 0xFF,
                    (a >> 8) & 0xFF,
                ),
            };
        }
        // The `26`-in-expression family (D2, `docs/IL_CALL_IN_EXPR.md` §14). The
        // whole bucket used to be one key — 286,240 functions, 12.9 % of the
        // blocked workload, naming 0.2 % of its own contents — and `mcall` walks
        // the production far enough to say which construct the `26` opened, plus
        // whether the *whole* segment would parse if that one form were admitted.
        // Everything is in `aux` because `ctx` is a `&'static str` and neither the
        // intrinsic selector nor the residue opcode is one.
        if self.ctx == mcall::CALL_IN_EXPR {
            return mcall::feature(self.aux);
        }
        // Operand-type blocks report the type's `<tag> <kind>` — **and not its
        // id**, which is the whole content of this key's history.
        //
        // A TYPE is `<tag> <kind> <LEB128 id>` (`docs/IL_TYPE_TAGS.md` §1). The
        // first two bytes are fixed vocabulary — the tag is the slot's width plus
        // a qualifier (`86` plain, `A6` const, `96` volatile), the kind's low
        // nibble is the type *class* (1 signed · 2 unsigned · 3 data pointer ·
        // 4 code pointer · 5 real · 6 aggregate · 7 void) — so together they name
        // the construct a widening would have to implement. The **id is an index
        // into the TU's own type table**: every distinct pointee and every
        // typedef gets a fresh one, and the same construct is numbered
        // differently in every TU.
        //
        // Putting that id in the bucket *name* shattered one construct into 256
        // shards, and a ranked histogram cannot show a shattered construct at
        // all. It hid `expr-load-type-A643` — a const-qualified 4-byte pointer
        // operand, 666,907 functions, 31 % of the blocked workload — behind rows
        // a fifth its size, and it hid the same class a second time by absorbing
        // 82.9 % of the address-leaf rung's gain in shards no ranking could
        // attribute. `GAPS.md` §6 had recorded the failure since the first census
        // and it was regrouped **by hand** for one analysis instead of being
        // fixed, which is exactly why it recurred.
        //
        // The id is not discarded — [`Block::aux`] still carries the whole triple
        // packed exactly as [`blk_type`] wrote it, and [`super::census::FnCensus`]
        // keeps the raw bytes of the site. It is kept out of the *name*, which is
        // the only place it did damage.
        if self.aux != 0 {
            return format!(
                "{}-{:02X}{:02X}",
                self.ctx,
                (self.aux >> 16) & 0xFF,
                (self.aux >> 8) & 0xFF,
            );
        }
        // The two byte-less renderings, and the difference between them is a
        // fact about the parse rather than a spelling.
        //
        // `:eof` PROMISES that the parse reached the end of the segment before
        // this refusal was raised — so the body is grammar-complete, nothing is
        // hiding behind the row, and its count is directly a widening estimate.
        // It is earned by `off == seg_len` and by nothing else. Both ways to
        // reach that offset are exact rather than approximate: `blk` reads
        // `seg.get(p)` at the live cursor, which is `None` only past the last
        // byte, and [`Block::at_end`] is raised by the post-parse gates, which
        // run only after `eat_fn_tail` has already required `p == seg.len()`.
        //
        // `:mid` is the honest complement: a refusal with no blocking byte,
        // raised at a cursor with segment left to parse. It is a **separate
        // bucket**, not a merge into the byte-named one — the same `ctx` can
        // legitimately produce both keys (a predicate checked once mid-parse for
        // a call argument and once post-parse for a whole body), and collapsing
        // them would destroy exactly the distinction this renderer exists to
        // draw. Merging buckets is the one failure a census instrument cannot
        // survive (`docs/GAPS.md` §6).
        let b = match self.byte {
            Some(b) => b,
            None if self.off >= self.seg_len => return format!("{}:eof", self.ctx),
            None => return format!("{}:mid", self.ctx),
        };
        if self.ctx == "expr" {
            // Operand-stream opcodes VERIFIED against live-toolchain captures
            // (docs/CODEGEN_W6_COMPARE.md pins the relational and logical ones
            // by compiling a probe per relation and reading the emitted byte).
            //
            // Only add a name here once a capture has established it. An earlier
            // revision of this table guessed the relational opcodes from their
            // numeric order and got `!=`, `<=` and `>=` wrong while missing `==`
            // entirely — which silently mislabelled census buckets, the one
            // thing this instrument exists to avoid. A hex bucket is a result;
            // a wrong name is a lie that survives into the roadmap.
            //
            // Signedness is NOT in the opcode: signed and unsigned probes emit
            // the same byte and differ only in the operand type (`86 41 74` int
            // vs `86 42 75` unsigned).
            let named = expr_opcode_name(b).or_else(|| cflow_opcode_name(b));
            return match named {
                Some(n) => format!("expr-{n}"),
                None => format!("expr-op-0x{b:02X}"),
            };
        }
        // A **control-flow** opcode met by a straight-line production. The byte is
        // the same in every `ctx` and it always means the same thing, so it gets
        // the same name everywhere — `body-cflow-label` and
        // `return-scope-close-cflow-label` are both a `29` and both say so, while
        // staying separate buckets because the production they interrupted is
        // different work.
        //
        // A pure RENAME: one old key maps to exactly one new key, so no bucket
        // merges and no recorded comparison is invalidated. What it buys is that
        // the largest structural blockers stop being hex — `body-0x29` (48,102) is
        // a `do`/`while`'s top label and `call-ref-0x3A` (5,335) is a branch to a
        // label that is not the epilogue, and neither of those is readable from
        // "0x29" or "0x3A".
        if let Some(n) = cflow_opcode_name(b) {
            return format!("{}-cflow-{n}", self.ctx);
        }
        format!("{}-0x{b:02X}", self.ctx)
    }
}

/// The **capture-verified** names of the control-flow opcodes
/// (`docs/IL_STMT_GRAMMAR.md` §7–§9, §11).
///
/// Same discipline as [`expr_opcode_name`], and the same reason: a wrong name is a
/// lie that survives into the roadmap. The two that could be guessed wrong are the
/// conditional pair, and they are not guessed — `fixtures/cpp/wcf_shapes.cpp` holds
/// the controlled witness, two functions in one TU differing only by a `!`:
///
/// ```text
///   if (a)  return 1; return 2;   b9 <a> 86 41 74  38 <L>  53 …then… 54 04  29 <L>
///   if (!a) return 1; return 2;   b9 <a> 86 41 74  39 <L>  53 …then… 54 04  29 <L>
/// ```
///
/// Both load `a` itself — the `!` never becomes an opcode — and both define `<L>`
/// *after* the then-clause, so `<L>` is "skip the then". The branch to it is taken
/// when the condition is false, and negating the condition swaps `38` for `39`.
/// So `38` is branch-if-FALSE and `39` is branch-if-TRUE, on this toolchain, from
/// a tracked fixture. `&&`/`||` corroborate independently in the same file: `a &&
/// b` emits `38` twice (short-circuit on false) and `a || b` emits `39` then `38`.
///
/// `mcall`'s `Blocker::Branch` recorded the polarity as UNDETERMINED on the
/// strength of two wild witnesses that could not separate the senses. It is
/// determined now, and both key producers read this one table so they cannot
/// disagree about what a byte is called.
pub(crate) fn cflow_opcode_name(b: u8) -> Option<&'static str> {
    match b {
        0x29 => Some("label"),           // define label <tok>              §7
        0x38 => Some("brfalse"),         // branch if the value is FALSE    §7
        0x39 => Some("brtrue"),          // branch if the value is TRUE     §7
        0x3A => Some("jump"),            // unconditional; also break /
        //                                  continue / goto / return        §8.4, §9
        0x3B => Some("switch-dispatch"), // dispatch on the table symbol    §11
        0x3C => Some("switch-table"),    // table header `3C <TYPE> <def>`  §11
        0x3D => Some("switch-case"),     // one case entry `3D <label>`     §11
        _ => None,
    }
}

/// The **capture-verified** names of the operand-stream opcodes, shared by the
/// `expr-*` census keys and by `mcall`'s second-blocker keys so the two can never
/// disagree about what a byte is called.
///
/// Only add a name here once a capture has established it. An earlier revision of
/// this table guessed the relational opcodes from their numeric order and got `!=`,
/// `<=` and `>=` wrong while missing `==` entirely — which silently mislabelled
/// census buckets, the one thing this instrument exists to avoid. A hex bucket is a
/// result; a wrong name is a lie that survives into the roadmap.
///
/// Signedness is NOT in the opcode: signed and unsigned probes emit the same byte
/// and differ only in the operand type (`86 41 74` int vs `86 42 75` unsigned).
pub(crate) fn expr_opcode_name(b: u8) -> Option<&'static str> {
    #[allow(clippy::match_same_arms)]
    {
            match b {
                0x1F => Some("cmp-eq"),   // ==
                0x20 => Some("cmp-ne"),   // !=
                0x21 => Some("cmp-le"),   // <=
                0x22 => Some("cmp-lt"),   // <
                0x23 => Some("cmp-ge"),   // >=
                0x24 => Some("cmp-gt"),   // >
                0x1A => Some("not"),      // !
                0x1B => Some("or-or"),    // ||
                0x1C => Some("and-and"),  // &&
                0x09 => Some("shl"),      // <<
                0x0A => Some("shr"),      // >>
                0x0B => Some("bit-and"),  // &
                0x0C => Some("bit-or"),   // |
                0x0D => Some("bit-xor"),  // ^
                0x2C => Some("convert"),  // `2C <TYPE> <varint>` — the real cast
                // `0x40` is a SECOND call token — the intrinsic call — not a
                // cast. It occupies the slot `BD` occupies:
                //   33 <int-TYPE> <selector>  40 <TYPE result>  (<expr> 55 <TYPE>)*  4C
                // An earlier revision of this table guessed "cast" from a single
                // witness where it followed a literal. It follows a bare `int`
                // constant at 6838 of 6839 aligned sites across three real TUs —
                // which is the selector, not a cast operand. Selectors seen:
                // 15 abs, 17 fabs, 159/160 _rotl/_rotr, 164 strcpy, 165 strcmp,
                // 167 strlen, 170 memcmp, 172 memcpy, 173 memset, 1973 sqrt,
                // and the dominant 2113-2119 class-layout adjustment family.
                0x40 => Some("intrinsic-call"),
                // The class-pair descriptor of that same family — NOT a call.
                0x66 => Some("class-descriptor"),
                0x43 => Some("ternary"),  // `43 42 ...` conditional select
                0x26 => Some("call-in-expr"),
                _ => None,
            }
    }
}

/// Build a [`Block`] at the current parse position.
pub(crate) fn blk(seg: &[u8], p: usize, ctx: &'static str) -> Block {
    Block { ctx, byte: seg.get(p).copied(), off: p, seg_len: seg.len(), aux: 0 }
}

/// Build an operand-*type* [`Block`]: `p` points at the 3-byte inline type that
/// is not the modeled int (`86 41 74`), `report_at` at the operand it belongs
/// to. Packs the triple into [`Block::aux`].
///
/// The whole triple is packed, id included — an analysis that wants the id has
/// it — but [`Block::feature`] renders only `<tag> <kind>`, because the id is a
/// per-TU table index and a bucket named after one is 256 buckets. See that
/// method's comment for what the sharding cost.
pub(crate) fn blk_type(seg: &[u8], p: usize, report_at: usize, ctx: &'static str) -> Block {
    let g = |i: usize| seg.get(p + i).copied().unwrap_or(0) as u64;
    Block {
        ctx,
        byte: seg.get(p).copied(),
        off: report_at,
        seg_len: seg.len(),
        aux: (g(0) << 16) | (g(1) << 8) | g(2),
    }
}

/// **The positive whole-body parser (W4b2-v).** Parse a single `.ex` function
/// segment as *exactly one* of the recognized [`BodyShape`]s, tokenizing
/// the entire operand stream from the `4C 4F 11` ('LO') marker to the end of the
/// segment. Acceptance is by a complete positive match — every token is
/// consumed through a fixed-pattern `eat` or a typed read, and the parse must
/// reach the segment end — so a second CALL, any computation after a terminal
/// call, a non-trivial call-argument region, or any unmodeled byte fails the
/// whole function closed (`None` → the caller reports `NotImplemented`). This
/// replaces the earlier trio of neighborhood-scanning gates (`parse_body`,
/// `is_tail_call`, `parse_framed_call`) that each accepted on a *local* byte
/// pattern and so over-accepted trailing/second-call computation.
///
/// Grammar (verified against live-toolchain captures of every fixture + probe):
/// ```text
///   body   := 'LO'(4C 4F 11) 'SS'(53) stmt?  ( arith | vcall | icall )
///   stmt   := 4F 01 NN                                    (multi-fn only)
///   arith  := expr(→41)  <return int>                     LOAD:=B9 tok INT
///   vcall  := 26 tok  CALL  4C 4B  <return void>          LIT :=33 INT varint
///   icall  := 26 tok  CALL  expr(→55)  55 INT 4C  postop  <return int>
///   postop := ε | 33 INT k 02                             expr:=(LOAD|LIT|02|03|04)+
///   CALL   := BD <ret TYPE> <conv> <varint fn-type-id>    (8-13 bytes, decoded)
/// ```
/// The `CALL` line used to read `BD <3-byte ret type> 00 80 01 10 00 00 (fixed 10
/// bytes)`. That was never an anchor: the trailing value is a per-TU **function-type
/// id**, keyed on the signature and assigned in declaration order of distinct
/// function types, so `0x1001` is merely the first one a single-callee fixture TU
/// happens to create. Every field is self-delimiting and is decoded — see
/// [`parse_call_shape`].
/// `<return …>` is the shared plumbing consumed by [`eat_return_plumbing`]
/// (result-type for int, then assign/return/tail/segment-or-module end). An
/// `icall` is classified by its `postop`: **absent, or `+ 0`** → an integer
/// tail call [`BodyShape::IntTailCall`] (the argument `expr` computed into r3,
/// then `b <callee>`; `g(a)`, `g(a)+0`, `g(a+1)` all land here). A **non-zero
/// `+ k`** over a *bare passthrough* argument (`expr == [Load]`) → the framed
/// [`BodyShape::FramedCall`] (whose `k` fits a signed-16-bit `addi`). A non-zero
/// `+ k` over a *computed* argument (`g(a+1)+1`), or a `* k`/`- k`/wide `k`/a
/// second literal/a second call, all reject. The `callee` name is not in `.ex`;
/// the caller pairs it from `.gl`.
pub(crate) fn parse_segment(seg: &[u8], sy: SyView) -> Option<BodyShape> {
    parse_segment_detail(seg, sy).ok()
}

/// [`parse_segment`] with the fail-closed *reason* preserved (P2b census).
/// Acceptance is identical — `parse_segment` is `.ok()` of this — so the census
/// can never disagree with the gate about what is in class.
pub(crate) fn parse_segment_detail(seg: &[u8], sy: SyView) -> Result<BodyShape, Block> {
    // The two dispatch axes are per-body, so they are cleared HERE — at the one
    // entry every reader goes through — rather than at each tag site. A tag left
    // over from the previous segment would attribute this body's row to a
    // recognizer that never saw it.
    dispatch_reset();
    let r = parse_segment_shape(seg, sy);
    // D2's whole-body-completeness bit. `parse_expr` classified the construct but
    // has no view of the segment as a whole, and this is the one place that has
    // both the block and the `LO` offset. Refusals only, and an `Err` stays an
    // `Err` — the census key moves, acceptance does not. See
    // [`mcall::whole_body_is_one_value`] for why the bit is worth more than the
    // sub-bucket count it decorates.
    match r {
        Err(b) if b.ctx == mcall::CALL_IN_EXPR => {
            match crate::func::body_start(seg) {
                Some(lo) => Err(mcall::mark_whole(seg, lo, b)),
                None => Err(b),
            }
        }
        other => other,
    }
}

fn parse_segment_shape(seg: &[u8], sy: SyView) -> Result<BodyShape, Block> {
    // Offset 0, and NOT the segment end: the whole segment was searched and the
    // marker is not in it, so the parse never started. Reporting it at the end
    // would claim `:eof` — "the parse ran out of segment" — for a body whose
    // grammar is entirely unexamined.
    // **Both forms of the body-start token**, through the crate's one locator.
    // `4C 4F 11` where a segment has one; the grammar-gated bare `4C` of a
    // `??__E`/`??__F` thunk where it does not (ROADMAP §10.12). W-LO taught the
    // splitter and the codec this and deliberately left the parser behind,
    // because every reader below added a hard-coded 3 to the offset; that is now
    // [`ops_start`]'s job and this is the last site.
    let lo = crate::func::body_start(seg).ok_or_else(|| {
        disp("disp-no-lo-marker");
        Block::refuse(seg, 0, "lo-marker")
    })?;

    // Every shape below maps a formal token to an argument register by its
    // **position** in the formals list. That is only the same thing as its
    // register number while each parameter occupies exactly one register, and a
    // by-value aggregate wider than 8 bytes does not: it shifts every later
    // parameter along. So the precondition is established once, here, for all of
    // them, rather than re-derived per shape.
    //
    // This is the fourth instance of the pattern in GAPS §6 — two facts sharing
    // one field, indistinguishable across the whole corpus because every fixture
    // parameter was a scalar. It emitted `lwz r3,0(r4)` for `lwz r3,0(r6)` in
    // `int gb(Big v, H* h) { return h->mi; }`, in class, on mainline, with all
    // four mode lanes and the 2,885-case sweep green.
    //
    // `.sy` is the only layer that carries a parameter's width (`.ex`'s formals
    // region is tokens alone), so a segment whose `.sy` block did not bind has
    // *undetermined* widths and refuses — it does not fall back to assuming one
    // register each, which is precisely the assumption that was wrong.
    // Only asserted when there *is* a formals list to assert it about. A segment
    // whose formals region does not parse cannot reach any shape that maps a
    // formal to a register — every one of those re-reads the same list through the
    // one anchor ([`formals_marker`]) and refuses there — so this gate declines to
    // restate a refusal it does not own, and the census keeps reporting the real
    // blocker instead of `formals-marker` for every such body.
    if let Ok(formals) = parse_formals(seg, lo) {
        if let Err(ctx) = sy.formals_are_one_register_each(&formals) {
            disp("disp-formals-width");
            return Err(Block::refuse(seg, lo, ctx));
        }
    }

    let locals = sy.locals;
    let mut p = crate::func::ops_start(seg, lo);
    // 'SS' statement-start — the body's own lexical scope — then any further brace
    // scopes and line markers. A body wrapped in braces used to refuse here as
    // `body-0x53`, the largest single blocking feature on the real workload.
    if !eat_byte(seg, &mut p, 0x53) {
        disp("disp-stmt-start");
        return Err(blk(seg, p, "stmt-start"));
    }
    let mut depth = BODY_SCOPE_DEPTH;
    if let Err(b) = eat_scopes(seg, &mut p, &mut depth) {
        disp("disp-scopes");
        return Err(b);
    }

    match *seg.get(p).ok_or_else(|| {
        disp("disp-body-truncated");
        blk(seg, p, "body")
    })? {
        // An EMPTY body opens directly on the return plumbing's `3A` assign —
        // there is no expression at all. `eat_return_plumbing` still has to
        // reach the segment end, so any trailing statement or unexpected operand
        // fails the function closed exactly as it does for every other shape.
        0x3A => {
            disp("disp-empty-body");
            eat_return_head(seg, &mut p, false, depth)?;
            // …and then, for a **constructor**, the `return this` that sits
            // between the RETURN and the tail. It emits nothing — `this` is
            // already in r3 and an empty body cannot have moved it — so the shape
            // is the same `EmptyBody` either way. Absent, this is a no-op and the
            // tail follows immediately, exactly as before.
            // [`shapes::eat_ctor_this_epilogue`] has the capture and the reason
            // the leaf restriction is not conservatism.
            eat_ctor_this_epilogue(seg, &mut p, lo);
            eat_fn_tail(seg, &mut p)?;
            Ok(BodyShape::EmptyBody)
        }
        // `26 <tok>` opens BOTH a call (the callee push) and an assignment
        // statement (the destination push), and the two are told apart by exactly
        // one byte: whether a `BD` CALL opcode follows the pushed token.
        //
        // Dispatching on that byte rather than trying the assignment parse and
        // falling back matters for the *measurement*, not for what is accepted.
        // Falling back meant every assignment-body refusal was re-reported as
        // whatever byte `parse_call_shape` then tripped over — nearly always the
        // RHS's `B9` — so `call-token-0xB9` was a conflated bucket holding pointer
        // operands, casts, `if` statements and more, all filed under a name that
        // described none of them. It has been the #1 entry at ~18% of blocked
        // functions and was directing the widening order at least twice this week.
        // Now each side reports its own reason.
        0x26 => {
            let mut probe = p + 1;
            let is_call = match read_token_var(seg, probe) {
                Some((_, w)) => {
                    probe += w;
                    seg.get(probe) == Some(&0xBD)
                }
                None => false,
            };
            if is_call {
                disp("disp-plain-call");
                // …and the **floating-point** tail call, which shares this
                // production's call head and has its own argument grammar (the
                // integer operand vocabulary cannot spell an FP value, so every
                // body in the class blocks at `expr-load-type-8645`/`-8885`).
                // Tried first and non-committally: it works on a copy of the
                // cursor and returns None with no side effects, so a body that
                // declines still reports its own blocker below and no census key
                // moves.
                if let Some(shape) = try_parse_fp_tail_call(seg, p, lo, sy) {
                    disp("disp-fp-tail-call");
                    return Ok(shape);
                }
                parse_call_shape(seg, &mut p, lo, None)
            } else {
                disp("disp-stmt-26");
                // …and the **member call as a whole body** (W36) — `p->m(a…);` and
                // `return p->m(a…);`, whose method push is the very `26` this arm
                // just decided was not a callee. It is not: the receiver sits
                // between the method push and the `BD`, so the one-byte test above
                // cannot see the call at all and the statement reaches the
                // assignment parser, which reads the receiver as a LOAD and stops
                // on the `99` bind. That was `expr-op-0x99` — 280,283 functions and
                // the largest key on the board — filed as an opcode.
                //
                // Tried before the assignment parse. A body that is not this
                // production leaves the cursor untouched and falls through, keeping
                // its own blocker (now the de-conflated `expr-call-in-expr-recv-*`
                // key, via [`mcall::reanchor_chain`]); one that IS this production
                // and parsed to the end of the segment reports the codegen-class
                // gate that refused it, under that gate's own key, rather than
                // vanishing back into the grammar bucket.
                // …and the **empty constructor delegating to one base**, which
                // is the generated destructor's own production reached without
                // the leading `33 <int> 0` literal — one production split across
                // two census buckets by one byte, which is why it arrives here
                // and its twin arrives in the `0x33` arm below. Tried before the
                // member-call parse because its head IS a member call (`26` then
                // a receiver) and that parse would report a blocker for a body
                // this one accepts whole. Non-committal: works on a copy of the
                // cursor and returns None with no side effects, so a declining
                // body keeps its own `expr-intrinsic-this-adjust` key.
                if let Some(shape) = try_parse_empty_ctor_base_delegation(seg, p, lo, depth) {
                    disp("disp-empty-ctor-base-delegation");
                    return Ok(shape);
                }
                // **The entry to all three member-call productions**, and so the
                // one place the [`PROD`] axis can be armed. `mcall_chain` and
                // `mcall_cmp` are reached only from inside this call, so a body
                // that does not get here provably cannot be moved by a widening in
                // any of the three — which is the fact the `disp-*` axis exists to
                // publish.
                //
                // Armed to `PROD_ENTERED_UNTAGGED` rather than left at
                // `PROD_NOT_ENTERED`, so the two are never confused: the residue
                // under this name is the population whose refusal is inside a
                // production and not yet attributed to a site.
                disp("disp-member-call");
                prod_tag(PROD_ENTERED_UNTAGGED);
                match try_parse_member_tail_call(seg, p, lo, depth) {
                    Ok(shape) => {
                        prod_tag(PROD_ACCEPTED);
                        return Ok(shape);
                    }
                    // Committed, then refused: the body's key is that gate's own,
                    // so it owes no first-blocker attribution and must not be
                    // ranked as if a tag site had declined.
                    Err(Some(b)) => {
                        prod_tag(PROD_COMMITTED_REFUSAL);
                        return Err(b);
                    }
                    Err(None) => {}
                }
                // **The POINTER-WALK ACCUMULATE LOOP** — the first body class
                // here with a back edge. It opens on the same `26 <local>` an
                // assignment statement does (its first statement *is* one:
                // `ret = 0`), so it is tried immediately ahead of the assignment
                // parser and never reaches the member-call productions above,
                // whose `BD` test already declined.
                //
                // Non-committal in the sense the whole ladder is: the
                // recognizer works on its own cursor and returns `Err` on the
                // very first byte that is not its grammar, so a body that
                // declines still reports `try_parse_assign_body_detail`'s
                // blocker and no census key moves. The *only* population that
                // can reach an accept is a body whose statement list is exactly
                // this loop, byte for byte, and `assign` refuses every one of
                // those today at its first `3A`.
                if let Ok(shape) = try_parse_ptr_walk_loop(seg, p, lo, locals, sy.ptr_locals) {
                    disp("disp-ptr-walk-loop");
                    return Ok(shape);
                }
                // **W-CFG1 — the `if`/`else`-with-a-join whose arms are calls.**
                // It opens on the same `26 <local>` both loops above do — its
                // first statement is `acc = 0` too — so it is tried here, on the
                // same terms: its own cursor, `Err` on the first byte that is
                // not its grammar, no census key moved by a decline.
                //
                // Its grammar is disjoint from both loops' at the **second**
                // statement, whichever is asked first: the loops require a `53`
                // opening a `for` whose first statement assigns a pointer local,
                // or the `29 <label>` of a top-test `while`; this one requires a
                // `53` whose first token is the `B9 <formal>` of a relational
                // test. So the order between the three is free, and it is last
                // because the two loops name matched workload TUs.
                if let Ok(shape) = try_parse_if_call_join(seg, p, lo, locals, sy.ptr_locals) {
                    disp("disp-if-call-join");
                    return Ok(shape);
                }
                // **W-DATA — the static-array scan loop.** It opens on the same
                // `26 <local>` the two loops above and `if_call_join` do — its
                // first statement is `j = 0` — so it is tried here on the same
                // terms: its own cursor, `Err` on the first byte that is not its
                // grammar, no census key moved by a decline.
                //
                // Its grammar separates from `ptr_walk_loop`'s at the SECOND
                // statement, whichever is asked first: that one requires a `53`
                // opening a `for` whose first statement assigns a **pointer**
                // local, this one requires the rotation's `3A <Ltest>`
                // immediately. It separates from `ptr_walk_chain_loop`'s at the
                // same point (a top-test `while`'s `29 <label>`) and from
                // `if_call_join`'s at the `53` opening a relational test. So the
                // order among the four is free; this one is last because the
                // other three name TUs that were matched before it.
                if let Ok(shape) = try_parse_static_scan_loop(seg, p, lo, locals) {
                    disp("disp-static-scan-loop");
                    return Ok(shape);
                }
                // **The body-parameterized loop**, beside its fixed-length
                // sibling and after it. The order is free rather than
                // load-bearing and saying which is the point: the two grammars
                // are disjoint at their second statement — `ptr_walk_loop`
                // requires a `53` opening a `for` scope whose first statement
                // assigns a *pointer* local, this one requires the `29 <label>`
                // of a top-test `while` — so neither can take a body from the
                // other whichever is asked first. It is second because its
                // sibling names a matched workload TU and a reader should meet
                // that one first.
                if let Ok(shape) =
                    try_parse_ptr_walk_chain_loop(seg, p, lo, locals, sy.ptr_locals)
                {
                    disp("disp-ptr-walk-chain-loop");
                    return Ok(shape);
                }
                // **#839 — a store run whose FIRST statement is the reference
                // bind**, so it opens on this `26` and never reaches the store
                // block below. The recognizer is the same one that block calls;
                // only the entry differs, exactly as `try_parse_store_run` and
                // `try_parse_store_run_call` share `collect_store_run`.
                //
                // Tried immediately ahead of the assignment parser, and after
                // both pointer-walk loops, for the reason the whole ladder gives:
                // it works on its own cursor and returns `None` with no side
                // effects, so a body that declines still reports
                // `try_parse_assign_body_detail`'s blocker and no census key
                // moves. The only population it can accept is one whose
                // statement list is a bind plus a store run, which `assign`
                // refuses today at the `32` of its first store.
                if let Some(shape) = try_parse_store_run_bind(seg, p, lo, sy, depth) {
                    disp("disp-store-run-bind");
                    return Ok(shape);
                }
                disp("disp-assign");
                try_parse_assign_body_detail(seg, p, lo, locals, depth)
            }
        }
        // Straight-line arithmetic opens with a LOAD or a bare literal — and so
        // does a W6 comparison leaf, which is tried first because its whole-body
        // shape is strictly more specific (a LOAD/LIT pair consumed by a
        // relational opcode). `try_parse_compare` is non-committal: it works on
        // a copy of the cursor and returns None without side effects, so a
        // non-comparison body falls through to the arithmetic parse unchanged.
        b @ (0xB9 | 0x33) => {
            // The arm label is set on ENTRY and each leaf production overwrites it
            // only when it **accepts**. For a blocked body every one of them
            // declined, so "the last one tried" would be an artefact of the
            // ordering rather than a fact about the body; `disp-expr-*` is the true
            // statement — the ladder reached the expression layer and the refusal
            // is inside `parse_expr`.
            //
            // **Split by the opening byte, because the two halves are different
            // findings.** A body opening on `B9` is an expression whose first
            // token is a LOAD. A body opening on `33` is one whose first token is a
            // LITERAL — and `33 86 41 74 00` (int 0) followed by a `26` method push
            // is the generated-destructor prologue, i.e. a body that is a member
            // call *after one literal*. Those reach this arm on their first byte
            // and are never offered to the member-call productions at all, which is
            // a dispatch fact and not a limit inside any of them. Merged into one
            // `disp-expr` row that distinction is invisible, and it is the whole
            // question of whether the row is takeable.
            disp(if b == 0x33 { "disp-expr-lit" } else { "disp-expr-load" });
            // **W8 — the two-arm conditional tail call**, and the only
            // production in this ladder that consumes a `38`. Tried first
            // because it is the only one that can: every recognizer below reads
            // the same `B9 <formal> <T> · 33 <T> <k> · <rel>` prefix and then
            // requires a `2C`/`41`/`4C` where this shape has a conditional
            // branch, so none of them can reach it and it cannot take a body
            // from any of them. Non-committal like the rest — a cursor copy and
            // an `Option`, so a declining body keeps its own blocker key
            // (`expr-cmp-eq`, the frontier's largest bucket, which is exactly
            // the population this shape is drawn out of).
            if let Some(shape) = try_parse_cond_tail_pair(seg, p, lo, depth) {
                disp("disp-cond-tail-pair");
                return Ok(shape);
            }
            // **W10 — a guarded call in a framed call SEQUENCE**, the framed ×
            // branching cell. Tried immediately after `cond_tail` and for the
            // same reason: it is the only other production in this ladder that
            // consumes a `38`, and every recognizer below reads the same
            // `B9 <formal> <T> · 33 <T> <k> · <rel>` prefix and then requires a
            // `2C`/`41`/`4C` where this shape has a conditional branch.
            //
            // **After `cond_tail`, not before**, and the adjacency is
            // load-bearing: the two shapes share that whole prefix and diverge
            // only at the arm's terminator — `cond_tail`'s arms end in a
            // `3A <epilogue>` (a tail call, fold band 3, no frame), this one's
            // fall through into a further call (a framed sequence). Neither can
            // take a body from the other, because `try_parse_guarded_seq`
            // requires an unguarded call after the `if` and `cond_tail`
            // requires the else arm to be the body's last statement. Ordered
            // this way anyway, so the narrower and older class keeps first
            // refusal. Non-committal: a cursor copy and an `Option`.
            if let Some(shape) = try_parse_guarded_seq(seg, p, lo, depth) {
                disp("disp-guarded-seq");
                return Ok(shape);
            }
            // **W11 — guarded EARLY RETURNS ahead of a framed sequence.**
            //
            // After `guarded_seq`, and the adjacency is load-bearing in the same
            // way `guarded_seq`'s is: the two share the whole
            // `B9 <formal> <T> · 33 <T> <k> · <rel> · 38 <L>` prefix and diverge
            // at the arm's FIRST byte — `guarded_seq`'s arm opens on a `26`
            // callee push, this one's on a `33` literal or a bare `3A`. Neither
            // can take a body from the other, and ordering the older class first
            // keeps its refusal keys stable. Non-committal: a cursor copy and an
            // `Option`.
            if let Some(shape) = try_parse_early_return_seq(seg, p, lo, depth) {
                disp("disp-early-return-seq");
                return Ok(shape);
            }
            // **W-EXTDATA — the sunk-`||`-guard, shared-tail body.** Tried after
            // the three productions above and separated from all of them by ONE
            // byte, which is the whole argument for the ordering being free:
            // they consume a `38` (brfalse) after the relation, because an `if`
            // branches AROUND its block; this one consumes a `39` (brtrue),
            // because a `||` short-circuits INTO its block. So none of them can
            // reach a body of this shape and it cannot take one of theirs,
            // whichever is asked first. It is last because the other three name
            // classes that were matched before it.
            //
            // Non-committal on the same terms as the rest of the ladder: its own
            // cursor, `Err` on the first byte outside its grammar, so a body that
            // declines still reports this arm's blocker (`expr-cmp-eq`, the
            // frontier's largest bucket and the population this shape is drawn
            // out of) and no census key moves.
            if let Ok(shape) = try_parse_guard_chain_shared_tail(seg, p, lo) {
                disp("disp-guard-chain-shared-tail");
                return Ok(shape);
            }
            // **The integer divide/modulo leaf.** Tried here because it is the
            // only production in this ladder that consumes an `05`/`06`, and
            // because its grammar diverges from every neighbour on the byte
            // after the second operand: the comparison leaf and W43 both need a
            // `33` literal where this has a second `B9` LOAD, and the
            // straight-line chain below reaches the same two bytes only to
            // refuse them (`expr-op-0x05` / `expr-op-0x06`, which is where this
            // population is counted today). So it can take a body from nothing
            // above it and nothing above it can take a body from it — the
            // ordering here is a statement, not a dependency.
            //
            // Non-committal like the rest: a cursor copy and an `Option`, so a
            // body that is not exactly this shape still reports its own blocker
            // and no census key moves.
            if let Some(shape) = try_parse_div_mod_leaf(seg, p, lo) {
                disp("disp-div-mod-leaf");
                return Ok(shape);
            }
            if let Some(shape) = try_parse_compare(seg, p, lo) {
                disp("disp-compare-leaf");
                return Ok(shape);
            }
            // **W43**, after the plain comparison leaf: the two share a prefix
            // and only this one continues past the `2C` into a `33`, so the
            // order is a tie-break that cannot change either verdict — stated
            // rather than relied on, because `select.rs`'s order IS load-bearing
            // and this file's reads as if it were the same kind of list.
            if let Some(shape) = try_parse_cmp_shift_or(seg, p, lo) {
                disp("disp-cmp-shift-or");
                return Ok(shape);
            }
            if let Some(shape) = try_parse_float_leaf(seg, p, lo, sy) {
                disp("disp-float-leaf");
                return Ok(shape);
            }
            if let Some(shape) = try_parse_indirect_load_leaf(seg, p, lo) {
                disp("disp-indirect-load-leaf");
                return Ok(shape);
            }
            // …and the pointer *identity* leaf (`return p;` / `return this;` /
            // a ptr→ptr cast of either), which is the same production minus the
            // `30` load. Tried after it, because a body that has a `30` is a
            // getter and this one must not see it: the shape between the two —
            // an offset add with no `30`, `return &s->m;` — emits an `addi` and
            // is refused by both. Non-committal like the others: it works on a
            // copy of the cursor and returns None with no side effects.
            if let Some(shape) = try_parse_ptr_identity_leaf(seg, p, lo) {
                disp("disp-ptr-identity-leaf");
                return Ok(shape);
            }
            // …and the **address** leaf, which is that same shape *with* the
            // offset add the identity refuses (`return &s->m;`, `return s->arr;`,
            // `return &p->Base::m;`) and which emits the one `addi` the identity
            // must not. Tried after both, so a body that has a `30` is still a
            // getter and a bare pointer is still an identity; this one is anchored
            // on the `41` result following the adds. Non-committal: it works on a
            // copy of the cursor and returns None with no side effects.
            if let Some(shape) = try_parse_addr_leaf(seg, p, lo) {
                disp("disp-addr-leaf");
                return Ok(shape);
            }
            // …and the generated empty destructor, whose body opens on a literal
            // `0` and is otherwise a member call. Anchored on `33 <int> 0` then a
            // `26`, so it cannot collide with the intrinsic-2117 designator above
            // (whose literal is the selector `2117`) nor with a real arithmetic
            // leaf (whose literal is followed by an operand or an operator).
            // Non-committal: works on a copy of the cursor, returns None with no
            // side effects, so a declining body still reports its own blocker.
            if let Some(shape) = try_parse_empty_dtor_delegation(seg, p, lo, depth) {
                disp("disp-empty-dtor-delegation");
                return Ok(shape);
            }
            // …and the **store** leaf, the third consumer of the same sub-object
            // designator (`s->m = v;`, `p->Base::m = v;`, `*p = v;`). Tried after
            // all of them because it is the only one that ends on a `32 <TYPE> 4B`
            // statement rather than on a `41` result — nothing above can reach it
            // and it can reach nothing above. Non-committal: works on a copy of the
            // cursor and returns None with no side effects, so a declining body
            // still reports its own blocker.
            if let Some(shape) = try_parse_store_leaf(seg, p, lo, sy) {
                disp("disp-store-leaf");
                return Ok(shape);
            }
            // …and the **store run** (W37): the same statement, a *sequence* of
            // them, plus the `return *this` tail. Tried after the single store so
            // that shape keeps its own census key and its byte-graded lowering
            // untouched; this one takes everything the leaf's "and the body ends
            // here" limit refused. Non-committal like the others: it works on a
            // copy of the cursor and returns None with no side effects, so a
            // declining body still reports its own blocker.
            if let Some(shape) = try_parse_store_run(seg, p, lo, sy, depth) {
                disp("disp-store-run");
                return Ok(shape);
            }
            // …and **F3**, the same run with a trailing CALL, which the run above
            // cannot admit because its own tail requires the body to end at the
            // last store. Tried after it so a body that *does* end there keeps
            // the store run's census key and its byte-graded lowering untouched;
            // this one takes only what "and the body ends here" refused. It is
            // the composition `src/xdk/nuispeech/xboxheap.cpp` is, and its
            // regime gate is board #1129's — the call's argument setup must be
            // empty. Non-committal like the others: it works on a copy of the
            // cursor and returns None with no side effects, so a declining body
            // still reports its own blocker.
            if let Some(shape) = try_parse_store_run_call(seg, p, lo, sy, depth) {
                disp("disp-store-run-call");
                return Ok(shape);
            }
            // …and **#839**, either of those runs with a C++ REFERENCE BIND in
            // it. Tried last of the four because a run with no bind is refused
            // by it outright, so the two productions above keep every body they
            // already had and their census keys with them.
            //
            // **THIS IS THE SECOND OF TWO DISPATCH SITES FOR ONE RECOGNIZER**,
            // and the other is in the `0x26` arm above. The arm a body reaches
            // is decided by the first byte of its FIRST STATEMENT, and a bind
            // may come first (`b_bind_first`, `b_leaf_bind`) or in the middle
            // (`b_target_bind`, which is `xboxheap.cpp`'s own spelling) — so one
            // site would silently cover half the shape. This lane's prereg
            // registered dispatch order as the loss it expected to take, and
            // this is where it took it.
            if let Some(shape) = try_parse_store_run_bind(seg, p, lo, sy, depth) {
                disp("disp-store-run-bind");
                return Ok(shape);
            }
            let (ops, cls) = parse_expr_classed(seg, &mut p, 0x41)?;
            // The expression itself parsed. Everything from here down is the
            // straight-line class's own gates, which is a materially different
            // answer to "where did this body stop" than `disp-expr` — the
            // `bare-nonformal` population lives here, not in the expression layer.
            disp("disp-straight-line");
            // The result annotation, and the ONE place the value's class has to be
            // carried across it. `eat_return_plumbing`'s own `41` gate is
            // `eat_int_like_or_ptr4` — shared with three byte-graded shapes and
            // deliberately not widened (`ROADMAP.md` §6d) — so a one-byte-unsigned
            // body consumes its annotation here instead, and requires it to
            // **restate the class**. That requirement is the whole gate: a `bool`
            // value annotated `int` is the `rlwinm` mask c2 emits for
            // `unsigned u(bool b){ return b; }` (`5463063e`), and admitting it as
            // a register move would be wrong bytes rather than a gap. Inside the
            // class nothing is emitted at all — `return false;` is `li r3,0`,
            // `return b;` is a bare `blr`, and from any other argument register it
            // is the same `mr r3,r4` the integer identity emits, all of which the
            // ordinary selector already produces from `[Lit(k)]` / `[Load(t)]`.
            if cls == Some(ValueClass::Int1u) {
                if !(eat_byte(seg, &mut p, 0x41) && eat_value_type(seg, &mut p, ValueClass::Int1u))
                {
                    return Err(blk(seg, p, "result-type"));
                }
                eat_return_plumbing(seg, &mut p, false, depth)?;
            } else {
                eat_return_plumbing(seg, &mut p, true, depth)?;
            }
            let params = parse_params(seg, lo)?;
            // A parameter used twice licenses c2's algebraic rewriter.
            if has_repeated_leaf(&ops) {
                return Err(Block::refuse(seg, p, "expr-repeated-leaf"));
            }
            // Gates that used to live in codegen; see
            // `straight_line_out_of_class_ctx`, which names *which* of them fired
            // so the row can be ranked clause by clause.
            if let Some(ctx) = straight_line_out_of_class_ctx(&ops, &params) {
                return Err(Block::refuse(seg, p, ctx));
            }
            // The shared canonicalize-or-refuse decision. Both producers of a
            // `StraightLine` call it, so they cannot hand codegen different
            // streams for the same expression — see `canonical_chain_for_codegen`.
            let ops = match canonical_chain_for_codegen(&ops, &params) {
                Ok(c) => c,
                Err(ChainReject::Order) => {
                    return Err(Block::refuse(seg, p, "expr-noncanonical-order"))
                }
                Err(ChainReject::Additive) => {
                    return Err(Block::refuse(seg, p, "expr-noncanonical-additive"))
                }
                Err(ChainReject::Affine) => {
                    return Err(Block::refuse(seg, p, "expr-affine-pending-imm"))
                }
                // `lane w-build`: the chain's intermediate registers are not
                // determined by the rule `select_text` implements. Its own key,
                // because it names a live wrong-bytes emit rather than a gap —
                // see `intermediate_alloc_determined`.
                Err(ChainReject::Alloc) => {
                    return Err(Block::refuse(seg, p, "expr-alloc-undetermined"))
                }
            };
            Ok(BodyShape::StraightLine { params, ops })
        }
        _ => {
            disp("disp-body-byte");
            Err(blk(seg, p, "body"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::test_fixtures::*;

    // ---- positive whole-body parser (W4b2-v) --------------------------------
    //
    // Every fixture below is a REAL `.ex` function segment captured from the
    // live 16.00.11886.00 toolchain (`/Bd /d2nop /Ox /GS- /c`), transcribed from
    // the `4F 1F` split point. Straight-line segments include the `46` formals
    // marker; call segments start at the `LO` marker (call shapes carry no
    // formal list). Each accepted segment is a *last* function, so it ends at
    // the module end `… 4F 02 20 00 4F 01 NN 4D` — the parser must reach it.

    #[test]
    fn parse_segment_accepts_straight_line_add3() {
        // `int add3(int a,int b,int c){ return a+b+c; }` (mvp_add3, single fn).
        let seg: &[u8] = &[
            0x46, 0x2D, 0xE5, 0x09, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals c,b,a
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD c
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type int
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // separator + GT terminate
            0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D, // module end
        ];
        assert_eq!(
            parse_segment(&free_fn(seg), NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xE309, 0xE409, 0xE509], // a, b, c
                ops: vec![
                    IlOp::Load(0xE309),
                    IlOp::Load(0xE409),
                    IlOp::Add,
                    IlOp::Load(0xE509),
                    IlOp::Add,
                ],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_bare_literal_return_and_wide() {
        // `int konst(){ return 42; }` — empty formal list (`46` then `LO`), a
        // bare literal, and the multi-function statement markers `4F 01 NN`.
        let konst: &[u8] = &[
            0x46, // formals marker, empty list
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0E, // LO SS + stmt marker
            0x33, 0x86, 0x41, 0x74, 0x2A, // LIT 42
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xEA, 0x09, 0x4F, 0x01, 0x0F, // ASSIGN + stmt marker
            0x54, 0x02, 0x29, 0xEA, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x10,
            0x4D,
        ];
        assert_eq!(
            parse_segment(&free_fn(konst), NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(42)],
            })
        );
        // `int kw(){ return 70000; }` — the wide (`0x80` + 4-byte LE) varint.
        let kw: &[u8] = &[
            0x46, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0D, // formals/LO/stmt
            0x33, 0x86, 0x41, 0x74, 0x80, 0x70, 0x11, 0x01, 0x00, // LIT 70000 (wide)
            0x41, 0x86, 0x41, 0x74, 0x3A, 0xEA, 0x09, 0x4F, 0x01, 0x0E, 0x54, 0x02, 0x29, 0xEA,
            0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01,
            0x0F, 0x4D,
        ];
        assert_eq!(
            parse_segment(&free_fn(kw), NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(70000)],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_nonlast_function_reaching_segment_end() {
        // `int add2(int a,int b){ return a+b; }` as the FIRST function of a
        // multi-fn TU: the segment is split before the next `4F 1F`, so it ends
        // right after `47 54 01 54 00` (no module end). The parse must accept by
        // reaching that segment end, not by finding a module marker.
        let seg: &[u8] = &[
            0x46, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals b,a
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x07, // LO SS + stmt marker
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xE6, 0x09, 0x4F, 0x01, 0x08, // ASSIGN + stmt marker
            0x54, 0x02, 0x29, 0xE6, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // GT terminate = segment end
        ];
        assert_eq!(
            parse_segment(&free_fn(seg), NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xE309, 0xE409],
                ops: vec![IlOp::Load(0xE309), IlOp::Load(0xE409), IlOp::Add],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_bare_void_tail_call() {
        // `void f(){ g(); }` (mvp_call): exactly one void call, `4C 4B`, then
        // only the return plumbing → a bare `b g` tail call.
        assert_eq!(
            parse_segment(&free_fn(MVP_CALL), NO_LOCALS),
            Some(BodyShape::VoidTailCall { callee_tok: 0xE309 })
        );
    }

    #[test]
    fn parse_segment_accepts_framed_call() {
        // `int f(int a){ return g(a) + 1; }` (mvp_framed): int call, single
        // passthrough arg, `55` call-end, exactly one `+1` post-op.
        assert_eq!(
            parse_segment(&free_fn(MVP_FRAMED), NO_LOCALS),
            Some(BodyShape::FramedCall {
                add_k: 1,
                callee_tok: 0xE409,
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
            })
        );
        // W41: `int f(int a){ return g(a) - 1; }` (mvp_call_submod) is the SAME
        // shape with the immediate negated. It sat in the rejection list above
        // from W4b2 to W41 on the stated ground that "c2 does not canonicalize
        // `-1` to `+(-1)`" — an argument, not a capture, and false: the two objs
        // differ in exactly the 16-bit immediate field of one word. The two
        // segments are byte-identical apart from the post-op opcode, which is
        // what makes this pair the pin.
        assert_eq!(
            parse_segment(&free_fn(GA_SUBMOD), NO_LOCALS),
            Some(BodyShape::FramedCall {
                add_k: -1,
                callee_tok: 0xE409,
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
            }),
            "g(a) - 1 is the framed call with a negative immediate"
        );
    }

    #[test]
    fn parse_segment_accepts_int_tail_call_family() {
        // The three int tail-call shapes (formals `46 2d e509` = param a → r3):
        //   passthrough `g(a)` and identity-fold `g(a)+0` → arg `[Load a]`;
        //   arg-setup `g(a+1)` → arg `[Load a, Lit 1, Add]`. All are
        //   `IntTailCall` (a net-identity post-op is a tail call, not framed).
        assert_eq!(
            parse_segment(&free_fn(INT_TAILRET), NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "passthrough g(a)"
        );
        assert_eq!(
            parse_segment(&free_fn(INT_PLUS0), NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "identity-fold g(a)+0 routes to a tail call, not FramedCall{{add_k:0}}"
        );
        assert_eq!(
            parse_segment(&free_fn(INT_ARGTAIL), NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509), IlOp::Lit(1), IlOp::Add],
                callee_tok: 0xE409,
            }),
            "arg-setup g(a+1)"
        );
    }

    #[test]
    fn parse_segment_routes_framed_nonzero_but_folds_zero_k() {
        // Routing contrast at the post-op: a NON-zero `+k` over a bare
        // passthrough arg is FramedCall (6-section frame); a ZERO `+k` folds to
        // an IntTailCall (5-section leaf). Same shape but for the immediate.
        assert_eq!(
            parse_segment(&free_fn(MVP_FRAMED), NO_LOCALS),
            Some(BodyShape::FramedCall {
                add_k: 1,
                callee_tok: 0xE409,
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
            }),
            "g(a)+1 is framed"
        );
        assert!(
            matches!(parse_segment(&free_fn(INT_PLUS0), NO_LOCALS), Some(BodyShape::IntTailCall { .. })),
            "g(a)+0 must NOT be FramedCall{{add_k:0}}"
        );
    }

    #[test]
    fn parse_segment_rejects_all_out_of_class_call_shapes() {
        // The W4b2-i/-v out-of-class probes — each a real captured segment the
        // positive parse must reject at the parser level (→ None →
        // NotImplemented), never mis-emit. Named by their `.cpp`. (The bare
        // arg-setup tail calls `g(a)`/`g(a)+0`/`g(a+1)` are now ACCEPTED —
        // see `parse_segment_accepts_int_tail_call_family`.)
        let cases: &[(&str, &[u8])] = &[
            // `g(a) - 1` used to be here. W41 measured it: `- k` is the SAME
            // `addi` as `+ k` with a negated immediate (`3863ffff` against
            // `38630001`), so it is ACCEPTED — see the framed acceptance test.
            // `* 5` stays, and is the case it was wrongly grouped with: a
            // constant multiply strength-reduces to a shift/add sequence.
            ("g(a) * 5 (mulmod)", GA_MULMOD),
            ("g(a) + 70000 (widemod)", GA_WIDEMOD),
            // `g(); g();` used to be here. It is the Class A many-call shape now
            // (#35 step 2 rung 1) — see the acceptance test below. `g(); return
            // a+1;` stays out: the `a` is read after the call, so it must survive
            // one, and c2 answers with a callee-saved register (Class B).
            ("g(); return a+1; (call_then_stmt)", CALL_THEN_STMT),
            ("g(a + 1) + 1 (argframed_plusk)", ARGFRAMED_PLUSK),
            ("g(a) + g(a + 1) (two_framed_calls)", TWO_FRAMED_CALLS),
            ("g(a) + 1 + 2 (plus1plus2)", PLUS1PLUS2),
        ];
        for (label, seg) in cases {
            assert_eq!(parse_segment(&free_fn(seg), NO_LOCALS), None, "must reject: {label}");
        }
    }

    #[test]
    fn parse_segment_rejects_unmodeled_arithmetic_ops() {
        // add3.cpp seg with a comparison/ternary (`24` GT, `43 42` CB) — the
        // parser must fail closed on the first unmodeled opcode, not skip it.
        let cmp: &[u8] = &[
            0x46, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, // formals
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x10, // LO SS stmt
            0xB9, 0xED, 0x09, 0x86, 0x41, 0x74, // LOAD
            0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74, // LOAD
            0x24, // GT — unmodeled → reject
            0x43, 0x42, 0x00, 0x00, 0x41, 0x86, 0x41, 0x74,
        ];
        assert_eq!(parse_segment(&free_fn(cmp), NO_LOCALS), None);
    }

    #[test]
    fn a_four_byte_token_parses_as_one_operand_not_two() {
        // The misalignment this fixes: reading a 4-byte token as 2 bytes leaves
        // the parse standing on the token's own tail, which then looks like an
        // unknown opcode. Build a straight-line body whose single LOAD carries a
        // wide token and check it decodes as exactly one Load of that token.
        let seg: &[u8] = &[
            0x46, 0x2D, 0xA4, 0x96, 0x03, 0x00, // formals: one wide token
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xA4, 0x96, 0x03, 0x00, 0x86, 0x41, 0x74, // LOAD <wide> int
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
        ];
        assert_eq!(
            parse_segment(&free_fn(seg), NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xA496_0300],
                ops: vec![IlOp::Load(0xA496_0300)],
            })
        );
    }

    // ---- P2b function-level census ------------------------------------------

    #[test]
    fn census_agrees_with_the_gate_on_every_pinned_segment() {
        // This used to compare `parse_segment` with `parse_segment_detail` and
        // could not fail: the former is literally `.ok()` of the latter, so it
        // asserted a function equals itself. It protected only against someone
        // re-forking the two, which is worth keeping — hence the first assertion —
        // but it never checked the invariant its name claims.
        //
        // The invariant that matters is that **everything the parser accepts, the
        // emitter can emit**. That cannot be tested from this crate (c2-il cannot
        // depend on c2-core), so what is pinned here is the half that can be: the
        // specific shapes whose emission gates used to live in codegen, and which
        // the census therefore counted as in-class while the port refused them.
        // Each must now be refused by the parser. The other half — that no
        // *accepted* shape is refused downstream — is guarded by the fixture
        // differential and `scripts/expr_sweep.sh`.
        let all: &[&[u8]] = &[
            MVP_CALL, MVP_FRAMED, INT_TAILRET, INT_PLUS0, INT_ARGTAIL, GA_SUBMOD, GA_MULMOD,
            GA_WIDEMOD, TWO_CALLS, CALL_THEN_STMT, ARGFRAMED_PLUSK, TWO_FRAMED_CALLS, PLUS1PLUS2,
        ];
        for seg in all {
            assert_eq!(
                parse_segment(&free_fn(seg), NO_LOCALS).is_some(),
                parse_segment_detail(&free_fn(seg), NO_LOCALS).is_ok(),
                "the two entry points have been re-forked"
            );
        }

        // Shapes that parse as a well-formed straight-line body but that
        // `select_text` declines. Each is refused in the parser now.
        let params = vec![0x10u32, 0x11];
        let a = IlOp::Load(0x10);
        let b = IlOp::Load(0x11);
        for (ops, why) in [
            (vec![a, IlOp::Lit(3), IlOp::Mul], "multiply by a constant"),
            (vec![IlOp::Load(0x99)], "bare non-formal token"),
            (vec![IlOp::Lit(5), a, IlOp::Sub], "const - reg needs subfic"),
            (vec![IlOp::Lit(-70000)], "negative wide constant"),
        ] {
            assert!(
                straight_line_out_of_class_ctx(&ops, &params).is_some(),
                "parser must refuse: {why}"
            );
        }
        // ...and the neighbours that really do emit must stay accepted. A bare
        // non-first formal is one of them now: it is the single `mr r3,rN` W18
        // grades (`fixtures/cpp/w18_reg_move.cpp`), not a refusal.
        for (ops, why) in [
            (vec![a, b, IlOp::Add], "a + b"),
            (vec![a], "bare first parameter"),
            (vec![b], "bare non-first parameter -> mr r3,r4"),
            (vec![IlOp::Lit(70000)], "positive wide constant"),
        ] {
            assert!(
                straight_line_out_of_class_ctx(&ops, &params).is_none(),
                "parser must accept: {why}"
            );
        }
    }

    #[test]
    fn census_names_the_first_blocking_opcode() {
        // A comparison (`24` GT) in the operand stream buckets as `expr-cmp-gt`,
        // and the offset points at the `24` itself — not at some later byte.
        let cmp: &[u8] = &[
            0x46, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x10,
            0xB9, 0xED, 0x09, 0x86, 0x41, 0x74, 0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74,
            0x24, // GT
            0x43, 0x42, 0x00, 0x00, 0x41, 0x86, 0x41, 0x74,
        ];
        // `b.off` indexes the segment that was PARSED, so hold on to it.
        let cmp = free_fn(cmp);
        let b = parse_segment_detail(&cmp, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-cmp-gt");
        assert_eq!(cmp[b.off], 0x24);
    }

    /// Retype the argument LOAD's inline type in a copy of [`INT_TAILRET`],
    /// leaving every other byte intact, and return the resulting block.
    fn load_typed(t: [u8; 3]) -> Block {
        let mut seg = INT_TAILRET.to_vec();
        let load = seg
            .windows(6)
            .position(|w| w == [0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74])
            .unwrap();
        seg[load + 3..load + 6].copy_from_slice(&t);
        let seg = free_fn(&seg);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(seg[b.off], 0xB9, "reported at the LOAD, not mid-type");
        b
    }

    #[test]
    fn census_reports_the_operand_type_class_not_its_shared_first_byte() {
        // Every 4-byte type's inline TYPE starts `86`, so bucketing on that byte
        // would merge pointer, float and aggregate operands into one meaningless
        // class. The bucket must carry the `kind` byte, which is the class.
        //
        // The two POINTER rows this table used to carry (`8643`, `A643` — which
        // were 45.9 % of the blocked workload between them) are gone from it,
        // because the LOAD position now admits them; the test below is their
        // replacement, and the classes that still refuse are still keyed by class.
        for (t, want) in [
            ([0x86u8, 0x45, 0x40], "expr-load-type-8645"), // float
            ([0x88, 0x85, 0x41], "expr-load-type-8885"),   // double
            ([0x88, 0x81, 0x13], "expr-load-type-8881"),   // long long
            ([0x86, 0x46, 0x80], "expr-load-type-8646"),   // aggregate
            ([0x82, 0x07, 0x03], "expr-load-type-8207"),   // void
        ] {
            assert_eq!(load_typed(t).feature(), want, "type {t:02X?}");
        }
    }

    /// The rung: a 4-byte pointer TYPE at the LOAD is an operand, not a blocker.
    /// Retyping the argument LOAD of [`INT_TAILRET`] — one field, every other byte
    /// left alone — must PARSE rather than bucket, and the resulting shape must be
    /// the same `int-tail-call` the int spelling produced. The negative half is the
    /// table above: the classes that are not 4-byte pointers still refuse at the
    /// same position with the same key.
    ///
    /// **The `volatile` spellings are in the refusing half, and they used not to
    /// be.** This test originally asserted all four tag spellings alike, from
    /// `is_ptr4_kind`'s whitelist rather than from a capture of each — and
    /// `int f(int x, int* volatile p) { return *p; }` emitted `lwz r3,0(r4)`
    /// where c2 homes the pointer in a frame and reads it back
    /// (`Port=Mismatch @ 8`, W32, `docs/rungs/2026-07-31-volatile-formal.md`). A
    /// volatile operand is a memory object; `const` is free. One bit of one byte,
    /// and a whole stack frame — so the two halves are asserted apart.
    #[test]
    fn a_four_byte_pointer_at_the_load_is_an_operand_not_a_blocker() {
        let int_shape = parse_segment(&free_fn(INT_TAILRET), NO_LOCALS).unwrap();
        let retyped = |t: [u8; 3]| {
            let mut seg = INT_TAILRET.to_vec();
            // Three-byte spellings, so the substitution is field-for-field and no
            // other byte of the segment moves. (A real `int*` id is usually two
            // LEB bytes — `86 43 F4 08` — and `read_type` walks either.)
            let load = seg
                .windows(6)
                .position(|w| w == [0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74])
                .unwrap();
            seg[load + 3..load + 6].copy_from_slice(&t);
            free_fn(&seg)
        };
        for t in [
            [0x86u8, 0x43, 0x74], // a data pointer
            [0xA6, 0x43, 0x74],   // const-qualified: `int* const`, and `this`
            [0x86, 0x44, 0x74],   // a CODE pointer, kind class 4
        ] {
            assert_eq!(
                parse_segment(&retyped(t), NO_LOCALS).as_ref(),
                Some(&int_shape),
                "pointer type {t:02X?} must parse as the int spelling does"
            );
        }
        // …and the volatile pair refuses, at the same position, under a key that
        // names the tag it refused on.
        for (t, want) in [
            ([0x96u8, 0x43, 0x74], "expr-load-type-9643"), // volatile
            ([0xB6, 0x43, 0x74], "expr-load-type-B643"),   // const volatile
        ] {
            assert_eq!(parse_segment(&retyped(t), NO_LOCALS), None, "type {t:02X?}");
            assert_eq!(
                parse_segment_detail(&retyped(t), NO_LOCALS)
                    .unwrap_err()
                    .feature(),
                want
            );
        }
    }

    /// The arithmetic guard, at the grammar level. `int* f(int* p){ return p+1; }`
    /// is transcribed verbatim from a live capture of `/tmp` probe `parith.cpp`
    /// (`docs/IL_CALL_IN_EXPR.md` §21.1) — note the literal is already **4**, the
    /// scaled byte offset, which is the measurement that says the guard is a
    /// conservatism and not a rescue. It refuses anyway, under its own key, and
    /// the identical body with an `int` operand still parses.
    #[test]
    fn a_pointer_operand_is_barred_from_arithmetic() {
        let ptr_add: &[u8] = &[
            0x46, 0x2D, 0xE3, 0x09, // formals: p = e309
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xE3, 0x09, 0x86, 0x43, 0xF4, 0x08, // LOAD p, type int*
            0x33, 0x86, 0x41, 0x12, 0x04, // LIT (long) 4 — c1xx already scaled it
            0x02, // ADD
            0x41, 0x86, 0x43, 0xF4, 0x08, // result-type int*
            0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, // assign + return
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        let seg = free_fn(ptr_add);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        // `:mid`: the guard fires on the ADD, with the result type and the whole
        // return plumbing still unread.
        assert!(b.off < b.seg_len, "the refusal is at the ADD, with the plumbing still ahead");
        assert_eq!(b.feature(), "expr-ptr-arith:mid");
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
        // The same body with an `int` operand and the same literal is exactly the
        // shape the port has emitted since the MVP, so the guard is keying on the
        // pointer and not on the addition. Written out rather than patched: the
        // int TYPE is three bytes where the pointer one is four, so a field-for-
        // field substitution would not be one.
        let int_add: &[u8] = &[
            0x46, 0x2D, 0xE3, 0x09, //
            0x4C, 0x4F, 0x11, 0x53, //
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD p, type int
            0x33, 0x86, 0x41, 0x12, 0x04, // LIT (long) 4
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type int
            0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, //
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        assert!(
            parse_segment(&free_fn(int_add), NO_LOCALS).is_some(),
            "the int spelling of the same chain must still parse"
        );
        // …and a pointer operand with NO arithmetic is admitted, so the guard is
        // not simply refusing every pointer: drop the `33 <long> 4 02` and the
        // body is the pointer identity the rung admits.
        let mut plain = ptr_add.to_vec();
        plain.drain(15..21); // the LIT (5 bytes) and the ADD
        assert!(parse_segment(&free_fn(&plain), NO_LOCALS).is_some());
    }

    /// **`:eof` means the parse reached the end of the segment, and the two
    /// cases are told apart.** The positive and the negative for the census
    /// instrument's one ranking signal.
    ///
    /// A key ending `:eof` is read as "the refusal was raised *after* the parse
    /// reached the segment end", which makes every function under it
    /// grammar-complete by construction — nothing else is hiding behind the row,
    /// so its count is directly a widening estimate. Both halves are checked
    /// here because only the pair pins the property: a renderer that printed
    /// `:eof` for everything passes the first assertion.
    ///
    /// Both bodies are reproduced from hand-written source through the live
    /// toolchain, not just transcribed: `int rep(int a){ return a+a; }` reports
    /// `expr-repeated-leaf:eof` and `int* ptr(int* p){ return p+1; }` reports
    /// `expr-ptr-arith:mid` under `c2rs census`.
    #[test]
    fn the_eof_suffix_is_earned_by_reaching_the_segment_end() {
        // `int rep(int a,int b,int c){ return a+a; }` — every byte from the
        // captured `add3` fixture above, with the second LOAD naming `a` again.
        // The repeated leaf licenses c2's algebraic rewriter, so the body is
        // refused — but only AFTER `eat_return_plumbing`, which returns `Ok`
        // solely at `p == seg.len()` (`eat_fn_tail`). So this refusal is at the
        // segment end and the parse has accounted for every byte of the body.
        let repeated: &[u8] = &[
            0x46, 0x2D, 0xE5, 0x09, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals c,b,a
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a — the repeat
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type int
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // separator + GT terminate
            0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D, // module end
        ];
        let seg = free_fn(repeated);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.ctx, "expr-repeated-leaf");
        assert_eq!(
            b.off, b.seg_len,
            "a post-parse refusal is raised at the segment end, which is what :eof reports"
        );
        assert_eq!(b.feature(), "expr-repeated-leaf:eof");

        // The negative, and the defect this test exists for: the identical
        // *shape* of refusal — no blocking byte, `byte: None` — raised at a
        // cursor with segment still ahead of it. It must NOT claim `:eof`, and
        // it must not be merged into the byte-named bucket either.
        let ptr_add: &[u8] = &[
            0x46, 0x2D, 0xE3, 0x09, //
            0x4C, 0x4F, 0x11, 0x53, //
            0xB9, 0xE3, 0x09, 0x86, 0x43, 0xF4, 0x08, // LOAD p, type int*
            0x33, 0x86, 0x41, 0x12, 0x04, // LIT (long) 4
            0x02, // ADD
            0x41, 0x86, 0x43, 0xF4, 0x08, // result-type int*
            0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, //
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        let seg = free_fn(ptr_add);
        let m = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(m.byte, None, "the two cases are the same shape of block");
        assert!(
            m.off < m.seg_len,
            "this refusal has segment left after it, so a second blocker may be in it"
        );
        assert_eq!(m.feature(), "expr-ptr-arith:mid");
        assert!(
            !m.feature().ends_with(":eof"),
            "a mid-segment refusal must never claim the completeness :eof promises"
        );
    }

    #[test]
    fn the_operand_type_bucket_does_not_shard_on_the_per_tu_type_id() {
        // THE de-sharding invariant. A TYPE's third field is an index into the
        // TU's own type table — every pointee and every typedef gets a fresh one
        // — so two ids under one `<tag> <kind>` are the *same* construct numbered
        // twice, and a key that carried the id split one construct into 256
        // buckets that no ranked histogram could add back up. `86 45 40` and
        // `86 45 83` are two `float`s numbered twice — the pointer pair this
        // used to be written over (`86 43 F4` `int*`, `86 43 83` `void*`) is no
        // longer a blocker at all, so the invariant is now carried by a class
        // that still shards.
        let a = load_typed([0x86, 0x45, 0x40]);
        let b = load_typed([0x86, 0x45, 0x83]);
        assert_eq!(a.feature(), b.feature(), "one construct, one bucket");
        // …and the id is *kept*, just not in the name: `aux` still holds the
        // whole triple, so an analysis that wants the type table index has it.
        assert_ne!(a.aux, b.aux);
        assert_eq!(a.aux, 0x864540);
        assert_eq!(b.aux, 0x864583);
    }

    #[test]
    fn the_operand_type_rekey_is_an_exact_coarsening() {
        // The re-key must be a *partition* of the old one: every block's new key
        // is a function of its old key, so functions can only merge, never move
        // sideways. Checked here at the level the property lives at — the key
        // formatter — over the four shapes `feature` can take, because the parse
        // itself is untouched and so every `Block` is bit-identical to before.
        let old = |b: Block| -> String {
            if b.ctx == "expr-intrinsic" || b.ctx == "call-intrinsic" {
                return format!("{}-{}", b.ctx, intrinsic_name(b.aux as i32));
            }
            if b.ctx == mcall::CALL_IN_EXPR {
                return mcall::feature(b.aux);
            }
            if b.aux != 0 {
                return format!(
                    "{}-{:02X}{:02X}{:02X}",
                    b.ctx,
                    (b.aux >> 16) & 0xFF,
                    (b.aux >> 8) & 0xFF,
                    b.aux & 0xFF
                );
            }
            match b.byte {
                None => format!("{}:eof", b.ctx),
                Some(x) if b.ctx == "expr" => match expr_opcode_name(x) {
                    Some(n) => format!("expr-{n}"),
                    None => format!("expr-op-0x{x:02X}"),
                },
                Some(x) => format!("{}-0x{x:02X}", b.ctx),
            }
        };
        // Only the pairings the parser can actually produce: `aux` is nonzero
        // for the operand-type blocks ([`blk_type`]), for the two intrinsic
        // contexts, and for `mcall`'s packed pair — nowhere else.
        let mut cases: Vec<(&'static str, u64)> = Vec::new();
        for ctx in ["expr-load-type", "expr-lit-type"] {
            for aux in [0x864174u64, 0x864175, 0x8643F4, 0xA64383, 0x888541, 0x000012] {
                cases.push((ctx, aux));
            }
        }
        for ctx in ["expr-intrinsic", "call-intrinsic"] {
            for aux in [15u64, 2113, 2117, 0xDF] {
                cases.push((ctx, aux));
            }
        }
        cases.push((mcall::CALL_IN_EXPR, 11));
        for ctx in ["expr", "body", "call-token", "fn-tail", "stmt-start"] {
            cases.push((ctx, 0));
        }
        let mut map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        for (ctx, aux) in cases {
            for byte in [None, Some(0x24u8), Some(0xB9)] {
                // `off: 0, seg_len: 0` — cursor at the end of an empty segment,
                // which is the shape a genuine `:eof` block has. The property
                // under test is the operand-TYPE rekey, and holding the eof axis
                // fixed is what keeps it the only thing moving.
                let b = Block { ctx, byte, off: 0, seg_len: 0, aux };
                let (o, n) = (old(b), b.feature());
                // Same old key ⇒ same new key. That is exactly "the new
                // partition is a coarsening of the old one", and it is what
                // makes the census difference attributable.
                match map.get(&o) {
                    Some(prev) => assert_eq!(prev, &n, "old key {o} maps to two new keys"),
                    None => {
                        map.insert(o.clone(), n.clone());
                    }
                }
                // Nothing outside the operand-type family may move at all.
                if !ctx.ends_with("-type") {
                    assert_eq!(o, n, "non-type key moved");
                }
            }
        }
        // And the family really did merge, or the test above is vacuous.
        let merged: Vec<_> = map.iter().filter(|(o, n)| o != n).collect();
        assert!(merged.len() >= 4, "expected the type family to fold: {merged:?}");
    }

    #[test]
    fn the_call_token_count_is_the_number_of_calls_the_body_issues() {
        // The D6 frame measure (§18). Pinned on the segments whose call count is
        // known from their *source*, not from a re-read of the walk — including
        // the ones with no call at all, because a counter that never returns 0
        // would report every leaf as needing a frame.
        for (seg, want, what) in [
            (MVP_CALL, 1usize, "void f(){ g(); }"),
            (MVP_FRAMED, 1, "int f(int a){ return g(a)+1; }"),
            (TWO_CALLS, 2, "void f(){ g(); g(); }"),
            (CALL_THEN_STMT, 1, "void call then a second statement"),
            (TWO_FRAMED_CALLS, 2, "two framed calls"),
            (PLUS1PLUS2, 1, "int f(int a){ return g(a)+1+2; }"),
            (GA_SUBMOD, 1, "int f(int a){ return g(a)-1; }"),
            // …and the leaves, because a counter that never returns 0 would
            // report every leaf as needing a frame.
            (IND_DEREF, 0, "return *p;"),
            (IND_THIS_GETTER, 0, "return mMember;"),
            (NARROW_LL_MEMBER, 0, "a long long member load"),
        ] {
            assert_eq!(call_tokens(&free_fn(seg)), want, "{what}");
        }
    }

    #[test]
    fn a_call_token_inside_a_consumed_payload_is_not_recounted() {
        // The walk skips the whole `BD <TYPE> <conv> <varint>` token, so a `BD`
        // byte that is *part* of one cannot be counted twice. Force the case by
        // planting `BD` in the function-type id's escape payload: `80` + 4 LE
        // bytes, one of which is `BD`.
        let mut seg = MVP_CALL.to_vec();
        let bd = seg.windows(2).position(|w| w == [0xBD, 0x82]).unwrap();
        // `BD 82 07 03 00 | 80 01 10 00 00` → keep the shape, poison the payload.
        seg[bd + 6] = 0xBD;
        seg[bd + 7] = 0xBD;
        assert_eq!(
            call_tokens(&free_fn(&seg)),
            1,
            "a BD inside the consumed token is not a second call"
        );
    }

    #[test]
    fn every_field_of_the_call_token_is_required_literally() {
        // Three fields that never varied over 15,095 wild sites. A measure that
        // skipped any of them would count a `BD` payload byte as a call — which
        // is exactly what the in-class control group caught (§18): the loose
        // version read 10,088 in-class LEAVES as `calls-2plus`.
        let base = MVP_CALL.to_vec();
        assert_eq!(call_tokens(&free_fn(&base)), 1);
        let bd = base.windows(2).position(|w| w == [0xBD, 0x82]).unwrap();
        for (off, poison, why) in [
            (4usize, 0x01u8, "calling convention must be 00"),
            (5, 0x01, "the fn-type id must use the 80 escape form"),
        ] {
            let mut seg = base.clone();
            seg[bd + off] = poison;
            assert_eq!(call_tokens(&free_fn(&seg)), 0, "{why}");
        }
        // …and the id's own value: `80 01 10 00 00` is 0x1001 little-endian, so
        // clearing the high byte of the low halfword leaves 0x0001, below the floor.
        let mut seg = base.clone();
        seg[bd + 7] = 0x00;
        assert_eq!(
            call_tokens(&free_fn(&seg)),
            0,
            "a fn-type id below 0x1000 is not one c2 allocated"
        );
    }

    // ---- the two DISPATCH axes ------------------------------------------
    //
    // These grade the *instrument*, not the grammar. Every assertion below is
    // stated POSITIVELY — "this arm must claim this body", "this many must be
    // tagged" — because the failure mode the axes exist to close is a population
    // that renders as an absence, and a test written as "no unexpected value
    // appeared" passes just as happily when nothing ran at all.

    /// **Positive coverage: each arm the ladder reaches NAMES itself.**
    ///
    /// Six pinned bodies, six different arms, each asserted by name with its own
    /// failure message — so a regression says which arm broke rather than that
    /// "the axis" did. The distinct-arm count at the end is not redundant with
    /// the loop: a count of six cases is also satisfied by six readings of ONE
    /// stuck tag, which is exactly what a broken instrument produces.
    #[test]
    fn the_dispatch_axis_names_the_arm_that_claimed_each_body() {
        let cases: &[(&str, &[u8], &str)] = &[
            ("MVP_CALL", MVP_CALL, "disp-plain-call"),
            ("BOOL_LIT", BOOL_LIT, "disp-straight-line"),
            ("NARROW_SHORT_TO_INT_REFUSED", NARROW_SHORT_TO_INT_REFUSED, "disp-expr-load"),
            ("STORE_MEMBER", STORE_MEMBER, "disp-store-leaf"),
            ("IND_DEREF", IND_DEREF, "disp-indirect-load-leaf"),
            ("DTOR_DELEGATE", DTOR_DELEGATE, "disp-empty-dtor-delegation"),
            ("BOUND_ARG_CANON", BOUND_ARG_CANON, "disp-assign"),
        ];
        let mut seen: Vec<&str> = Vec::new();
        for (name, body, want) in cases {
            let seg = free_fn(body);
            let _ = parse_segment_detail(&seg, NO_LOCALS);
            assert_eq!(
                dispatch_site(),
                *want,
                "{name} must be claimed by {want}: the dispatch axis says which \
                 recognizer looked at the body, and a wrong arm here mis-attributes \
                 every census row that body is in"
            );
            if !seen.contains(want) {
                seen.push(want);
            }
        }
        assert_eq!(
            seen.len(),
            7,
            "seven DISTINCT arms must have been read, not seven readings of one arm"
        );
        // …and the other half of the expression arm, which no pinned fixture
        // reaches: a body whose first token is a LITERAL rather than a LOAD.
        // The split is not cosmetic — `33 86 41 74 00` then a `26` method push is
        // the generated-destructor prologue, so `disp-expr-lit` is where a member
        // call preceded by one literal statement lands, and it is the largest
        // population that no member-call widening can reach.
        let body: Vec<u8> = vec![
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0x33, 0x86, 0x41, 0x74, 0x05, // LIT int 5
            0x9B, 0x00, 0x00, // a byte with no production
        ];
        let seg = free_fn(&body);
        let _ = parse_segment_detail(&seg, NO_LOCALS);
        assert_eq!(
            dispatch_site(),
            "disp-expr-lit",
            "a body whose first token is a LITERAL must be told apart from one \
             whose first token is a LOAD: the two halves of this arm are different \
             findings and merging them hides which is takeable"
        );
    }

    /// **Every body gets a NAMED reading, including "nothing happened".**
    ///
    /// `prod-not-entered` is a claim, not a hole: it says the member-call
    /// productions were never entered, which is precisely the fact that makes a
    /// widening inside them unable to move the body. The pinned bodies below are
    /// all such bodies, and the axis must say so out loud.
    #[test]
    fn a_body_that_reaches_no_tagged_site_still_reads_a_named_default() {
        let never_enter: &[(&str, &[u8])] = &[
            ("MVP_CALL", MVP_CALL),
            ("BOOL_LIT", BOOL_LIT),
            ("STORE_MEMBER", STORE_MEMBER),
            ("DTOR_DELEGATE", DTOR_DELEGATE),
        ];
        let mut named = 0;
        for (name, body) in never_enter {
            let seg = free_fn(body);
            let _ = parse_segment_detail(&seg, NO_LOCALS);
            assert_eq!(
                prod_site(),
                PROD_NOT_ENTERED,
                "{name} never reaches try_parse_member_tail_call, so the production \
                 axis must SAY that rather than leave the row empty"
            );
            named += 1;
        }
        assert_eq!(
            named, 4,
            "all four pinned bodies must have been graded — a loop that ran zero \
             times reports no failures either"
        );
        // …and the other end of the same discipline: a body that DID enter a
        // production and declined non-committally. It must read a **named
        // per-site tag** — not the `prod-not-entered` default (it did enter) and
        // not `prod-entered-untagged` (the residue, which the 37 tag sites in
        // `shapes::mcall_{tail,chain,cmp}` drove to 0 on the 878-TU workload).
        //
        // The residue constant is NOT retired, and this is why: it is still the
        // value the ladder arms the axis to, so a tag site removed or a bail
        // added without one lands back here and prints as its own row. The test
        // that grades the sites themselves under mutation is
        // `mcall_tail::assert_no_decline_lands_in_the_residue`; this one grades
        // the carrier's contract that no state renders as an absence.
        let seg = free_fn(BOUND_ARG_CANON);
        let _ = parse_segment_detail(&seg, NO_LOCALS);
        assert_ne!(
            prod_site(),
            PROD_NOT_ENTERED,
            "this body DOES enter a member-call production, so reading the \
             not-entered default would be the axis losing a body it saw"
        );
        assert_ne!(
            prod_site(),
            PROD_ENTERED_UNTAGGED,
            "a body that entered a member-call production and declined must reach \
             a NAMED tag site: `prod-entered-untagged` is the tag-coverage \
             residue, and a body still sitting in it is a refusal the report can \
             only render as an absence"
        );
    }

    /// **No tag may ever be stale.** The axes are thread-locals, so the one way
    /// they can lie is by reporting the *previous* body's reading for a body that
    /// set nothing. The order below is the adversarial one — a body that sets a
    /// non-default value immediately before one that sets none.
    #[test]
    fn the_dispatch_axes_are_reset_per_body() {
        let seg = free_fn(BOUND_ARG_CANON);
        let _ = parse_segment_detail(&seg, NO_LOCALS);
        assert_ne!(
            prod_site(),
            PROD_NOT_ENTERED,
            "precondition: this body must leave a NON-default production reading \
             (it enters the chain production and declines at a named site), or the \
             staleness check below has nothing to detect"
        );
        assert_eq!(
            dispatch_site(),
            "disp-assign",
            "precondition: this body must leave a non-default dispatch reading too"
        );
        let seg = free_fn(BOOL_LIT);
        let _ = parse_segment_detail(&seg, NO_LOCALS);
        assert_eq!(
            prod_site(),
            PROD_NOT_ENTERED,
            "the production axis must be cleared per body: this body never enters a \
             production, so inheriting the previous one's tag would attribute it to \
             a recognizer that never saw it"
        );
        assert_eq!(
            dispatch_site(),
            "disp-straight-line",
            "the dispatch axis must be cleared per body as well"
        );
        // …and the explicit reset, which is what the census's varargs arm uses:
        // that arm never calls the parser at all.
        dispatch_reset();
        assert_eq!(
            dispatch_site(),
            DISP_NOT_RUN,
            "an explicit reset must leave the named `disp-not-run` — the ladder did \
             not run for this body, which is a fact, not a blank"
        );
        assert_eq!(
            prod_site(),
            PROD_NOT_ENTERED,
            "an explicit reset must leave the named `prod-not-entered` too"
        );
    }

    /// **The carrier the 37 tag sites in `shapes::mcall_{tail,chain,cmp}` will
    /// use.** Those files belong to another lane; this pins the seam they write
    /// against, so their work is adding call sites and nothing else.
    ///
    /// Two properties, and both matter: the tag's own words survive verbatim (a
    /// report row IS the tag), and `prod_tag` evaluates to the `None` those
    /// productions already return for "not this production, cursor untouched" —
    /// so a site is `.map_err(|_| prod_tag("…"))?` with no other change.
    #[test]
    fn prod_tag_is_the_seam_the_member_call_productions_write_against() {
        dispatch_reset();
        let r: Option<Block> = prod_tag("tail-recv-not-a-plain-b9-load");
        assert!(
            r.is_none(),
            "prod_tag must evaluate to None so it drops into the existing \
             non-committal bail idiom without changing what is returned"
        );
        assert_eq!(
            prod_site(),
            "tail-recv-not-a-plain-b9-load",
            "the tag's own words must reach the census verbatim — the report row IS \
             the tag, and a line number would say nothing to its reader"
        );
        // Last write wins, so an inner production's tag replaces the outer arm's
        // on the way out — which is what makes `mcall_chain`'s bails visible even
        // though `mcall_tail` armed the axis first.
        prod_tag("chain-receiver-not-a-plain-b9-load");
        assert_eq!(
            prod_site(),
            "chain-receiver-not-a-plain-b9-load",
            "the LAST tag written must win: a chain bail happens after the tail \
             production armed the axis, and the chain's is the specific one"
        );
    }
}



/// **W-R1 — the `??__E` dynamic-initializer thunk decodes.**
///
/// The transcript is the whole `.ex` function segment of
/// `src/system/synth/tomcrypt/TomCryptLicense.cpp` at the workload's own flags
/// (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …`), `4F 1F` header included.
/// `src/system/zlib/ZlibLicense.cpp` captures a **byte-identical** 2,839 B `.ex`
/// — everything that separates the two TUs is in `.gl` and `.in` — so this one
/// transcript grades both.
#[cfg(test)]
mod wr1_dyninit {
    use super::*;
    use crate::func::test_fixtures::NO_LOCALS;

    /// `static Licenses sLicense("system/src/tomcrypt", (Licenses::Requirement)0);`
    /// lowering to `??__EsLicense@@YAXXZ`, whose six instructions are
    ///
    /// ```text
    ///   lis  r11,`string'   lis  r10,sLicense
    ///   addi r4,r11,…       addi r3,r10,…      li r5,0    b ??0Licenses@@QAA@…@Z
    /// ```
    const TOMCRYPT_DYNINIT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00,
        0x4F, 0x33, 0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01,
        0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38,
        0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F,
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, // block start, line 3
        0x53, 0x53, 0x26, 0xFA, 0x09, // SS SS, result-ref
        0x46, // formals: EMPTY (`??__E…@@YAXXZ` takes none)
        0x4C, 0x53, // the BARE `LO` — no `4F 11` record — then the body's SS
        0x26, 0xEA, 0x09, // push the callee (the constructor)
        0x26, 0xF9, 0x09, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, // &sLicense …
        0x99, 0x86, 0x43, 0x8D, 0x20, 0x00, // … bound as the receiver
        0xBD, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, // CALL
        0x33, 0x86, 0x41, 0x83, 0x20, 0x00, // arg 2: literal 0, typed as the enum
        0x55, 0x86, 0x41, 0x83, 0x20, //         `Licenses::Requirement`
        0x26, 0xFC, 0x09, 0x2C, 0x86, 0x43, 0x85, 0x20, 0x00, // arg 1: the string
        0x55, 0x86, 0x43, 0x85, 0x20, //         literal's address
        0x4C, 0x4B, // void call end — the result is discarded
        0x3A, 0xFB, 0x09, 0x54, 0x02, 0x29, 0xFB, 0x09, // return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // function tail
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x04, // module end, line 4 …
        0x53, 0x54, 0x00, // … its EMPTY module-level scope (`??__E` last) …
        0x4D, // … and the terminator
    ];

    #[test]
    fn the_dynamic_initializer_thunk_decodes_to_a_three_slot_tail_call() {
        // Slot 0 is `this` (the object), slot 1 the string, slot 2 the literal —
        // r3, r4, r5 — which is the order c2's own listing emits the `addi`s and
        // the `li` in. The callee token is resolved to `??0Licenses@@QAA@…@Z`
        // by `.gl`, not by `.ex`, so it stays a token here.
        assert_eq!(
            parse_segment(TOMCRYPT_DYNINIT, NO_LOCALS),
            Some(BodyShape::MultiArgTailCall {
                params: Vec::new(),
                arg_sources: vec![
                    SlotArg::SymAddr(0xF909), // slot 0 -> r3, the `.bss` object
                    SlotArg::SymAddr(0xFC09), // slot 1 -> r4, the `.rdata` string
                    SlotArg::Lit(0),          // slot 2 -> r5, `li r5,0`
                ],
                callee_tok: 0xEA09,
            })
        );
    }

    /// The byte claims this rung rests on, asserted against the transcript so a
    /// future edit to the constant cannot quietly move them.
    #[test]
    fn the_transcript_carries_the_bare_lo_and_the_empty_module_scope() {
        // No composed marker anywhere; the body opens on `4C 53` at 61, right
        // after the empty formals list, so the operand stream starts at `lo + 1`.
        assert_eq!(crate::func::readers::find_subslice(TOMCRYPT_DYNINIT, &[0x4C, 0x4F, 0x11]), None);
        let lo = crate::func::body_start(TOMCRYPT_DYNINIT).expect("a body start");
        assert_eq!(lo, 61);
        assert_eq!(crate::func::ops_start(TOMCRYPT_DYNINIT, lo), 62);
        assert!(crate::func::body_start_is_bare(TOMCRYPT_DYNINIT));
        // The module trailer's optional empty scope.
        assert_eq!(&TOMCRYPT_DYNINIT[TOMCRYPT_DYNINIT.len() - 4..], &[0x53, 0x54, 0x00, 0x4D]);
    }

    /// **The fence, stated as a test.** Strip the three trailer bytes and the
    /// body still decodes (the trailer is optional, not required); strip the
    /// module end itself and it refuses.
    #[test]
    fn the_empty_module_scope_is_optional_and_the_module_end_is_not() {
        let mut plain = TOMCRYPT_DYNINIT.to_vec();
        plain.drain(plain.len() - 4..plain.len() - 1); // drop `53 54 00`
        assert!(parse_segment(&plain, NO_LOCALS).is_some());
        let mut truncated = TOMCRYPT_DYNINIT.to_vec();
        truncated.pop(); // drop the `4D`
        assert!(parse_segment(&truncated, NO_LOCALS).is_none());
    }
}

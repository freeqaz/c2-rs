//! **The dead-temporary call body** — a body whose whole content is one call
//! plus the materialization of a temporary nothing else ever reads.
//!
//! > # Three readers now, and the third is DIFFERENT IN KIND (board **#1053**)
//! >
//! > [`no_effect_call`] and [`no_effect_loop`] return a **callee token**: they
//! > say *"this body emits nothing **provided** that callee reduces to
//! > nothing"*, which is a **link** into E's least fixpoint and never a seed.
//! >
//! > [`no_effect_nothing`] returns a **bool**, because the body it reads has no
//! > callee at all. It says *"this body emits nothing, **unconditionally**"* —
//! > and that is a strictly stronger claim, which **SEEDS** the fixpoint.
//! > `c2_core::elide::Reduction::NoEffectNothing` is the variant, and the
//! > termination and cycle arguments were re-derived rather than inherited when
//! > it was added; see that type's doc.
//! >
//! > The three vocabularies are **disjoint by construction**: the first two
//! > require a call token that the third's closed vocabulary excludes, and the
//! > third requires a statement the first two's argument walk never reaches.
//! > `the_three_no_effect_shapes_are_disjoint` asserts it in all directions
//! > rather than leaving it to the reading.
//!
//! # What this is for, and what it is NOT
//!
//! This is **not** a body shape the port emits. Nothing here reaches
//! [`super::super::BodyShape`], `select_function` or the COFF writer, and
//! [`super::super::parse_segment`] is byte-for-byte unchanged by it: a body this
//! module recognizes is still **parse-refused**, still counts in
//! `fnbyte-refused`, and still refuses its whole TU in
//! [`crate::IlBundle::functions`].
//!
//! What it produces is one fact, for one consumer: **mechanism E's fixpoint**
//! (`c2_core::elide::TuEmptyCallees`). E fires when a tail call's callee
//! *reduces to nothing*, and until now the only way to establish that was
//! [`crate::IlFunction::empty_body`] — a body that **decodes empty**. Board
//! **#980**'s 370 workload functions are the case where the callee's body is not
//! empty and still emits nothing, and lane `w-seq` (board #966/#971) measured
//! that every one of them is blocked on this production.
//!
//! # The shape, and where each byte came from
//!
//! ```cpp
//!   struct true_tag {};                                   // an empty tag type
//!   template <class I> inline void aux(I, I, const true_tag&) {}
//!   template <class I, class T> inline void dr(I f, I l, T*) {
//!       aux(f, l, true_tag());                            // <- THIS BODY
//!   }
//! ```
//!
//! `true_tag()` is value-initialized into a stack temporary — which c1xx spells
//! as an **intrinsic `memset`** (selector 173, `docs/IL_INTRINSIC_CALL.md` §1) —
//! and the temporary's address is then passed by reference. The call's callee
//! `aux` has an empty body, so c2 emits nothing for the callee, nothing for the
//! temporary, and the whole function is one `4e800020`.
//!
//! This is STLport's `__destroy_range` (`stl/_construct.h:172`) with the names
//! shortened, and it is the callee of every one of board #980's 370
//! `??$_Destroy_Range@…` differs.
//!
//! ```text
//!   4C 4F 11 53                                the body marker and its scope
//!   26 <callee-tok>                            the callee symbol push
//!   BD <ret TYPE> 00 <varint fn-type-id>       the CALL token
//!     33 86 41 74 80 AD 00 00 00               the intrinsic selector, 173
//!     40 <TYPE>                                the intrinsic-call token
//!       33 86 41 74 <align> 55 86 41 74        the alignment hint
//!       33 86 41 74 <count> 55 86 41 74        the byte count
//!       33 86 41 74 <fill>  55 86 41 74        the fill byte
//!       9B <TYPE> <temp-tok> 2C <TYPE> <v>     the destination — a TEMPORARY
//!       55 <TYPE>
//!     4C                                       the intrinsic's apply
//!     9B <TYPE> <temp-tok> 2C <TYPE> <v> 44    THE SAME temporary, by reference
//!     55 <TYPE>
//!     ( B9 <formal-tok> <TYPE> 55 <TYPE> )*    the formals, as they are
//!     ( 33 <TYPE> <lit>    55 <TYPE> )*        literals, as they are
//!   4C 4B                                      apply, and discard the result
//!   <the return plumbing, to the segment end>
//! ```
//!
//! # Why this is sound, stated as the four things that make it so
//!
//! 1. **The walk is TOTAL over the segment.** It starts at the body marker and
//!    ends by requiring [`eat_return_plumbing`]'s fail-closed terminal, so every
//!    byte of the body is accounted for by a production above. "The temporary is
//!    read nowhere else" is therefore not a search — there *is* nowhere else.
//! 2. **The answer is conditional, and the condition is the callee.** This
//!    returns the callee's *token*, never a verdict. The caller resolves it and
//!    asks E's own fixpoint whether that callee reduces to nothing; a body whose
//!    callee does not is not admitted. So the rule is keyed on the **callee's
//!    decoded IL**, which is board **#950**'s standing requirement — the
//!    relocation observable reads "nothing happened" on a self-recursive body
//!    that is plainly not nothing, and nothing here reads a relocation.
//! 3. **The temporary is a `9B` temp bind, twice, with the same token.** The
//!    destination of the write and the argument passed are required to be
//!    literally the same token; a body that memsets one slot and passes another
//!    is refused rather than assumed equivalent.
//! 4. **A cycle is still never admitted.** This module says nothing about
//!    cycles; it hands a *link* to the least fixpoint, which admits a name only
//!    on a `false → true` transition and never seeds one. `elide.rs`'s
//!    `a_cycle_is_not_elided_and_terminates` is unchanged and still passes.
//!
//! # What it deliberately does NOT accept
//!
//! * **A real `memset`.** The production for `memset(p, 0, n)` on a *pointer*
//!   carries a `void*` result type and its destination is an operand stream, not
//!   a `9B` temp bind. c2 lowers that one to `b <memset>` with a REL24
//!   (`docs/IL_CAST_CONVERT.md` §1.3) — the opposite of nothing.
//! * **An argument that is anything else** — a computed expression, a data
//!   symbol's address, a nested call, a second temporary bound by a different
//!   token. The argument vocabulary is a closed list of three forms and anything
//!   outside it refuses the whole body.
//! * **A call whose result is used.** `4C` must be followed by `4B`, the
//!   discard.
//! * **A `float`/`double` result**, for [`CallRet::discarded`]'s reason: the TU
//!   acquires `_fltused` and the obj grows a symbol.

use super::calls::eat_call_head;
use super::super::expr::{
    eat_return_plumbing, eat_scopes, intrinsic_selector, parse_formals, BODY_SCOPE_DEPTH,
};
use crate::func::readers::{eat_byte, eat_opt_stmt_marker, read_token_var, read_type, read_varint, INT_TYPE};

/// The intrinsic selector id for `memset` (`docs/IL_INTRINSIC_CALL.md` §3).
/// Named here rather than spelled `173` at the site, because the *number* is
/// what the whole production hangs on.
const INTRINSIC_MEMSET: i32 = 173;

/// **The recognizer.** `Some(callee_token)` when this segment's body emits
/// nothing **provided** the callee that token names reduces to nothing.
///
/// `None` — always, for every other body — and the cursor is not shared with any
/// other parse, so this can never change what [`super::super::parse_segment`]
/// accepts or which census key a refusal reports.
pub(crate) fn no_effect_call(seg: &[u8]) -> Option<u32> {
    let lo = crate::func::body_start(seg)?;
    // The formals list, so an argument load can be required to name one. A
    // segment whose formals region does not parse is refused rather than read
    // with an empty list — an empty list would silently turn every load into a
    // non-formal and change *which* bodies are refused for *which* reason.
    let formals = parse_formals(seg, lo).ok()?;

    let mut p = crate::func::ops_start(seg, lo);
    if !eat_byte(seg, &mut p, 0x53) {
        return None;
    }
    let mut depth = BODY_SCOPE_DEPTH;
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    eat_opt_stmt_marker(seg, &mut p);

    // The whole point of THIS shape is the temporary; a body with none of them
    // is an ordinary void call and belongs to the shapes that already parse it.
    // Inside a loop (`no_effect_loop`) there is no such shape, which is the one
    // place the two callers of `eat_no_effect_call_stmt` differ.
    let callee_tok = eat_no_effect_call_stmt(seg, &mut p, &formals, true)?;
    // The `}` of the statement, as a line marker. `eat_return_head` opens on the
    // `3A` directly, so a body whose call and whose return are on two source
    // lines — which is every one in the workload — needs this and the pinned
    // one-line cell does not. Measured: without it the reader fires **0** times
    // on the 878-TU workload and 1 time on the hand cell.
    eat_opt_stmt_marker(seg, &mut p);
    // The fail-closed terminal: this must reach the end of the segment, which is
    // what makes the walk total and the "read nowhere else" claim structural.
    eat_return_plumbing(seg, &mut p, false, depth).ok()?;
    Some(callee_tok)
}

/// **One whole discarded call statement**, from the callee push through the
/// `4C 4B` that applies it and throws the result away. `Some(callee_token)`.
///
/// Factored out of [`no_effect_call`] **byte for byte** so that
/// [`no_effect_loop`] reads the loop's single statement with the *same* closed
/// argument vocabulary rather than a second copy of it. `w-relo`'s merge is the
/// reason this is a factoring and not a new walk: two lanes wrote the same
/// reader in different files, auto-merged without a conflict marker, and the
/// duplicate walks were caught only by a compile error.
///
/// `require_temp` is the one difference between the two callers, and it is a
/// *narrowing* in the body case, never a widening in the loop case: a plain
/// discarded call at the top of a body has an accepted shape already, and inside
/// a loop it has none.
fn eat_no_effect_call_stmt(
    seg: &[u8],
    p: &mut usize,
    formals: &[u32],
    require_temp: bool,
) -> Option<u32> {
    let mut q = *p;
    let (callee_tok, ret) = eat_call_head(seg, &mut q).ok()?;
    // A discarded `float`/`double` result drags `_fltused` into the TU. The same
    // gate every other discarded-call site applies, through the same predicate.
    ret.discarded(seg, q).ok()?;

    // ---- the argument region: a closed vocabulary of three forms ------------
    let mut temps = 0usize;
    loop {
        // Source-line markers are decode-only — c2 emits nothing at one — and a
        // real workload body carries them between statements and inside argument
        // lists alike. Eating them here is what the accepted shapes do at every
        // statement boundary; **it widens nothing**, because the vocabulary below
        // is still closed and a marker is not a member of it.
        eat_opt_stmt_marker(seg, &mut q);
        match *seg.get(q)? {
            0x4C => {
                q += 1;
                break;
            }
            0x9B => return None, // a temp bind that is not preceded by its memset
            0x33 => {
                // Either the intrinsic selector (the temporary) or a plain
                // literal push. `intrinsic_selector` requires the `40` to follow,
                // so the two cannot be confused.
                match intrinsic_selector(seg, q) {
                    Some(INTRINSIC_MEMSET) => {
                        eat_dead_temp_arg(seg, &mut q)?;
                        temps += 1;
                    }
                    Some(_) => return None,
                    None => eat_lit_push(seg, &mut q)?,
                }
            }
            0xB9 => eat_formal_push(seg, &mut q, formals)?,
            _ => return None,
        }
    }
    if require_temp && temps == 0 {
        return None;
    }
    // The result is DISCARDED. Without this a value-consuming call would be read
    // as emitting nothing while its result is still wanted.
    if !eat_byte(seg, &mut q, 0x4B) {
        return None;
    }
    *p = q;
    Some(callee_tok)
}

/// The compound-assign operator the induction step uses: `+=`.
///
/// Pinned rather than accepted as "some operator", because what makes the step
/// harmless is that it computes a value into a local nothing else reads — and
/// *which* value it computes is irrelevant only for as long as the operator
/// cannot be one with a side effect of its own. `l02`, `l12` and the workload's
/// `??$__destroy_range_aux@…` all carry `0F`; nothing else is graded.
const OP_ADD_ASSIGN: u8 = 0x0F;

/// The comparison opcodes the loop's exit test may use, **each with the cell
/// that graded it**: `20` is `!=` (`l02`, and every workload site) and `22` is
/// `<` (`l12`).
///
/// The trip count is irrelevant to "this body emits nothing" — the body is the
/// only thing that could emit — so this list is a *completeness* choice and not
/// a soundness one, and it is a list rather than "any byte" because a byte this
/// grid has not seen may not be a comparison at all. `38 <label>` is required
/// immediately after it, which pins the operator's width to one.
const LOOP_CMP_OPS: [u8; 2] = [0x20, 0x22];

/// **The destroy-loop body.** `Some(callee_token)` when this segment's whole
/// content is
///
/// ```text
///   53                            the loop's own scope
///   3A <Lcond>                    goto COND
///   29 <Lincr>            INCR:
///      26 <formal> 33 <TYPE> <k> 0F <TYPE> 4B     one pure induction step
///   29 <Lcond>            COND:
///      B9 <formal> <TYPE>  B9 <formal> <TYPE>  <cmp>  38 <Lexit>
///      <one discarded call statement>
///      3A <Lincr>                 continue
///   29 <Lexit>            EXIT:
///   54 <n>  <return plumbing to the segment end>
/// ```
///
/// — **provided** the callee that token names reduces to nothing. Like
/// [`no_effect_call`] this is a **condition and never a verdict**: it returns the
/// token, `c2_core::elide`'s least fixpoint decides, and a refused body still
/// contributes a link and never a seed.
///
/// This is STLport's `__destroy_range_aux(_first, _last, __false_type)` — the
/// overload a **class** element type takes, against the `__true_type` one whose
/// empty body `w-inl0` already closes. It is level 3 of board **#980**'s
/// five-level chain and the production `fnbyte-blr-stop2` prices at **228**.
///
/// # Why a LOOP can be read this way at all
///
/// The soundness is [`no_effect_call`]'s four properties plus one the loop earns:
///
/// 1. **The walk is TOTAL** — it ends on [`eat_return_plumbing`]'s fail-closed
///    terminal, so the induction variable is provably read nowhere after the
///    loop.
/// 2. **The answer is CONDITIONAL on the callee.**
/// 3. **The induction step is PURE**: one lvalue that must name one of *this
///    function's own formals* (so it cannot be a data symbol — `elide.rs`'s
///    condition 3, one level down), one literal, one operator, discarded.
/// 4. **The exit test reads only formals**, for the same reason.
/// 5. **EVERY LABEL IS MATCHED.** The three labels are read and required to be
///    the three this shape mints and no others: the head's `3A` names the same
///    label as the `29` that opens the test, the tail's `3A` names the same label
///    as the `29` that opens the step, the `38` names the `29` that closes the
///    loop, and the three are pairwise distinct. A body with a fourth branch
///    target is refused rather than read as this loop with something extra in it.
///
/// **The trip count is not modelled and does not need to be.** If the body emits
/// nothing then no number of iterations of it emits anything, and the induction
/// and the test are pure by (3) and (4). What this does *not* license is any
/// claim about **termination** — nothing here says the loop halts, only that c2
/// emits no code for it, which is what the grid measured (`l01`, `l09`) at `/O1`
/// **and** at `/Ob0`.
pub(crate) fn no_effect_loop(seg: &[u8]) -> Option<u32> {
    let lo = crate::func::body_start(seg)?;
    let formals = parse_formals(seg, lo).ok()?;

    let mut p = crate::func::ops_start(seg, lo);
    if !eat_byte(seg, &mut p, 0x53) {
        return None;
    }
    let mut depth = BODY_SCOPE_DEPTH;
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    // Exactly one scope deeper than the body: the `for`'s own. A body that opens
    // two is a body with a block this reader has not walked.
    if depth != BODY_SCOPE_DEPTH + 1 {
        return None;
    }

    // ---- the loop head: `goto COND`, then the INCR label -------------------
    eat_opt_stmt_marker(seg, &mut p);
    let l_cond = eat_label(seg, &mut p, 0x3A)?;
    let l_incr = eat_label(seg, &mut p, 0x29)?;

    // ---- the induction step ------------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    eat_induction_step(seg, &mut p, &formals)?;

    // ---- COND: the exit test ----------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_label(seg, &mut p, 0x29)? != l_cond {
        return None;
    }
    eat_formal_load(seg, &mut p, &formals)?;
    eat_formal_load(seg, &mut p, &formals)?;
    if !LOOP_CMP_OPS.contains(seg.get(p)?) {
        return None;
    }
    p += 1;
    let l_exit = eat_label(seg, &mut p, 0x38)?;

    // ---- the body: ONE discarded call statement ---------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let callee_tok = eat_no_effect_call_stmt(seg, &mut p, &formals, false)?;

    // ---- `continue`, then EXIT --------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_label(seg, &mut p, 0x3A)? != l_incr {
        return None;
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_label(seg, &mut p, 0x29)? != l_exit {
        return None;
    }
    // Three distinct targets, and no fourth: the walk is total, so any other
    // branch in this body would have had to be consumed above.
    if l_cond == l_incr || l_cond == l_exit || l_incr == l_exit {
        return None;
    }

    // ---- close the loop's scope and reach the segment end ------------------
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    if depth != BODY_SCOPE_DEPTH {
        return None;
    }
    eat_return_plumbing(seg, &mut p, false, depth).ok()?;
    Some(callee_tok)
}

/// The **`void`** operand TYPE's tag/kind pair, `82 07`. It is the second literal
/// of the pseudo-destructor statement, and the pair the census names when it
/// refuses that body: `expr-lit-type-8207`.
///
/// Two bytes and not three, unlike [`INT_TYPE`]: the trailing field is a per-TU
/// type id and pinning it would make this reader a property of one bundle's type
/// table. The tag and the kind are what say *this literal is `void`*, and `void`
/// is what makes it free of [`CallRet::discarded`]'s hazard.
const VOID_TYPE: [u8; 2] = [0x82, 0x07];

/// **The body that emits nothing AT ALL.** `true` when this segment's whole
/// content is
///
/// ```text
///   53                          the body scope, and NOTHING deeper
///   <line marker>
///   33 <INT_TYPE> <varint>      an int literal      — value unconstrained
///   33 82 07 <id> <varint>      a VOID literal      — value and id unconstrained
///   44                          the bind
///   4B                          the discard
///   <line marker>
///   <return plumbing, to the segment end>
/// ```
///
/// This is `p->~T()` on a class with a **trivial** destructor: STLport's
/// `__destroy_aux(_pointer, __false_type)`, level 5 of board #980's chain and the
/// production `fnbyte-blr-stop3` prices at **227**. Read out of the workload with
/// `c2rs census --fn __destroy_aux` on `src/lazer/meta_ham/CharacterProvider.cpp`;
/// GRID-N's `n01` is the same shape compiled standalone.
///
/// # This one SEEDS, and that is a different claim from the other two
///
/// [`no_effect_call`] and [`no_effect_loop`] hand E's fixpoint a **link**: a
/// callee token, and a promise that is conditional on what that callee does. This
/// hands it a **seed** — an unconditional assertion that c2 emits nothing for this
/// function. Three things make that checkable rather than assumed:
///
/// 1. **The walk is TOTAL.** It begins at the body marker and ends by requiring
///    [`eat_return_plumbing`]'s fail-closed terminal, so *every byte* of the
///    segment is consumed by a production above. "There is nothing else in this
///    body" is structural and not a search. Mutation **M1** removes the terminal
///    and `trailing_bytes_after_the_nothing_statement_are_refused` goes red.
/// 2. **The vocabulary is CLOSED, and contains no call token.** `26` (which is
///    also the data-symbol push), `B9`, `BD`, `40`, `4C`, `67`, `9B` and every
///    label opcode are outside it, so a body this returns `true` for **names no
///    callee and materializes no data symbol**. That is what licenses
///    `link = None` in `elide.rs`, which is in turn what keeps a cycle out of the
///    seed set — see `Reduction`'s doc for the re-derivation. Mutation **M2**
///    opens the vocabulary and 4 GRID-N cells go red.
/// 3. **The literal TYPES are pinned and the literal VALUES are not.** A literal
///    is pure whatever its value and the statement is discarded, so the value
///    cannot change what is emitted — constraining it would be #644's mistake, the
///    same one [`no_effect_call`]'s align/count/fill deliberately avoid. The type
///    is a soundness constraint and not a fitting: a `float`/`double` literal
///    drags `_fltused` into the TU and the obj **grows a symbol**, which is
///    [`super::calls::CallRet::discarded`]'s reason one operand over. `int` and
///    `void` are the two the capture carries and the only two admitted.
///
/// # What it deliberately does NOT accept
///
/// * **Two of these statements** (`n10`). Two discarded pseudo-destructors emit
///   nothing just as one does; this is a match declined on purpose, because the
///   shape that was graded is the one with a single statement in it.
/// * **The same statement with a call beside it** (`n06`, `n11`). That is what
///   keeps a cycle member out of the seed set, so it is not a conservatism that
///   may later be relaxed for free — relaxing it costs the termination argument.
/// * **A body refused for a different reason**, `body-0x67` above all (`n04`).
///   That refusal is what keeps E safe from an INDIRECT call site
///   (`docs/INLINE_PREDICATE.md` §1.3, board #921), and admitting one is board
///   #232's shape.
///
/// The `44` is consumed as a **byte**, not as an operator whose arity this module
/// claims to know. `eat_dead_temp_arg` reads the same opcode after exactly one
/// operand and this statement carries two before it; nothing here depends on
/// resolving that, because the whole statement is pinned as a sequence and any
/// deviation refuses.
pub(crate) fn no_effect_nothing(seg: &[u8]) -> bool {
    nothing_body(seg).is_some()
}

fn nothing_body(seg: &[u8]) -> Option<()> {
    let lo = crate::func::body_start(seg)?;
    // Parsed and discarded: this shape names no formal. A segment whose formals
    // region does not decode is one whose body offsets are not trustworthy either,
    // so it refuses here rather than being walked with an empty list — the same
    // fail-closed reason `no_effect_call` gives.
    parse_formals(seg, lo).ok()?;

    let mut p = crate::func::ops_start(seg, lo);
    if !eat_byte(seg, &mut p, 0x53) {
        return None;
    }
    let mut depth = BODY_SCOPE_DEPTH;
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    // NOTHING deeper than the body's own scope. A body that opens one has a block
    // this reader has not walked, and the statement walk below would read its
    // first statement as the whole body.
    if depth != BODY_SCOPE_DEPTH {
        return None;
    }

    eat_opt_stmt_marker(seg, &mut p);
    eat_nothing_stmt(seg, &mut p)?;
    // The `}` of the statement, as a line marker — `no_effect_call`'s measured
    // reason: the statement and the return sit on two source lines in every
    // workload body and on one in a pinned cell.
    eat_opt_stmt_marker(seg, &mut p);
    // THE FAIL-CLOSED TERMINAL. This must reach the end of the segment, and it is
    // what makes the walk total and the seed honest.
    eat_return_plumbing(seg, &mut p, false, depth).ok()?;
    Some(())
}

/// The one statement: two literal operands, the bind, the discard.
fn eat_nothing_stmt(seg: &[u8], p: &mut usize) -> Option<()> {
    let mut q = *p;
    eat_lit_operand(seg, &mut q, &INT_TYPE)?;
    eat_lit_operand(seg, &mut q, &VOID_TYPE)?;
    if !eat_byte(seg, &mut q, 0x44) {
        return None;
    }
    // DISCARDED. Without this the statement's value would still be wanted by
    // whatever follows, and the walk would be reading a fragment of a larger
    // expression as a whole body.
    if !eat_byte(seg, &mut q, 0x4B) {
        return None;
    }
    *p = q;
    Some(())
}

/// `33 <TYPE> <varint>` where the TYPE's leading bytes are exactly `want` — a
/// bare literal operand, with no `55` argument push after it.
///
/// `want` is the whole [`INT_TYPE`] triple for the first operand and `VOID_TYPE`'s
/// tag/kind **pair** for the second; the asymmetry is deliberate and its reason is
/// on `VOID_TYPE`.
fn eat_lit_operand(seg: &[u8], p: &mut usize, want: &[u8]) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x33) {
        return None;
    }
    if seg.get(q..q + want.len())? != want {
        return None;
    }
    // Walked with `read_type` and not skipped by `want.len()`: the id is a varint
    // and the aggregate ladder moves the width, so the end of the TYPE is read off
    // the stream rather than assumed. A wrong width here is a parse desync inside
    // a reader that feeds an ELISION, which is the worst failure this project has.
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    read_varint(seg, &mut q)?;
    *p = q;
    Some(())
}

/// `<op> <token-var>` — a branch or a label, returning the token it names.
fn eat_label(seg: &[u8], p: &mut usize, op: u8) -> Option<u32> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, op) {
        return None;
    }
    let (tok, w) = read_token_var(seg, q)?;
    *p = q + w;
    Some(tok)
}

/// `B9 <token-var> <TYPE>` — a load of one of this function's own formals, with
/// no `55` push after it. The exit test's two operands.
fn eat_formal_load(seg: &[u8], p: &mut usize, formals: &[u32]) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    if !formals.contains(&tok) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    *p = q + w;
    Some(())
}

/// `26 <formal> 33 <TYPE> <k> 0F <TYPE> 4B` — the loop's induction step.
///
/// The `26` lvalue **must name one of this function's own formals**. That is the
/// whole of the purity argument and it is not a formality: `26` is also the
/// data-symbol push — `an_argument_outside_the_vocabulary_refuses_the_body`
/// mutates a formal load into exactly this opcode — so without the membership
/// test a step that incremented a **global** would read as pure and the body
/// would materialize a data symbol, which is `elide.rs`'s condition 3 one level
/// down and `w-fix`'s `k16` cell.
///
/// The literal's TYPE and VALUE are read and **not constrained**: `l02` carries
/// the stride `4` and `l12` carries `8`, and a rule that pinned either would be
/// #644's "one producer, one contiguous field" mistake with a different field.
fn eat_induction_step(seg: &[u8], p: &mut usize, formals: &[u32]) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x26) {
        return None;
    }
    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    if !formals.contains(&tok) {
        return None;
    }
    // The stride, as a literal. `eat_lit_push` requires the trailing `55` push
    // that closes an *argument*; an operand of an in-place operator has none, so
    // the three fields are walked here.
    if !eat_byte(seg, &mut q, 0x33) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    read_varint(seg, &mut q)?;
    if !eat_byte(seg, &mut q, OP_ADD_ASSIGN) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    if !eat_byte(seg, &mut q, 0x4B) {
        return None;
    }
    *p = q;
    Some(())
}

/// `33 <TYPE> <varint> 55 <TYPE>` — a literal standing as a whole argument.
fn eat_lit_push(seg: &[u8], p: &mut usize) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x33) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    read_varint(seg, &mut q)?;
    eat_push(seg, &mut q)?;
    *p = q;
    Some(())
}

/// `B9 <tok> <TYPE> 55 <TYPE>` — one of this function's own formals, as it is.
fn eat_formal_push(seg: &[u8], p: &mut usize, formals: &[u32]) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    if !formals.contains(&tok) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    eat_push(seg, &mut q)?;
    *p = q;
    Some(())
}

/// `55 <TYPE>` — the argument push that closes every argument form.
fn eat_push(seg: &[u8], p: &mut usize) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x55) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    *p = q + w;
    Some(())
}

/// `9B <TYPE> <tok> 2C <TYPE> <varint>` — a **temporary**, converted to the type
/// the use wants. Returns the temporary's token.
///
/// `0x9B` is the temp bind (`super::mcall_tail`'s `temp_bind` axis); `0x2C` is
/// the CONVERT, whose trailing varint is decoded only to find its end.
fn eat_temp_addr(seg: &[u8], p: &mut usize) -> Option<u32> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x9B) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    if !eat_byte(seg, &mut q, 0x2C) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    read_varint(seg, &mut q)?;
    *p = q;
    Some(tok)
}

/// The whole temporary: the intrinsic `memset` that initializes it, and the same
/// temporary immediately passed by reference.
///
/// The two tokens must be **equal**. A body that writes one slot and passes
/// another is a body this module does not understand, and it refuses rather than
/// treating the pair as interchangeable.
fn eat_dead_temp_arg(seg: &[u8], p: &mut usize) -> Option<()> {
    let mut q = *p;
    // `33 <int> <173>` — the selector. Already validated by the caller through
    // `intrinsic_selector`; re-walked here so this function owns its own cursor.
    if !eat_byte(seg, &mut q, 0x33) {
        return None;
    }
    if seg.get(q..q + INT_TYPE.len())? != INT_TYPE {
        return None;
    }
    q += INT_TYPE.len();
    if read_varint(seg, &mut q)? != INTRINSIC_MEMSET {
        return None;
    }
    // `40 <TYPE>` — the intrinsic-call token. It carries no trailing field
    // (`docs/IL_INTRINSIC_CALL.md` §1.1).
    if !eat_byte(seg, &mut q, 0x40) {
        return None;
    }
    // **The result type is `int`, and that is a MEASURED discriminator.** On
    // `src/lazer/game/BustAMovePanel.cpp` the byte run `33 86 41 74 80 AD 00 00
    // 00` occurs **455** times and the byte after it splits three ways: **446**
    // `40 86 41 74` — the temporary-materialization form this module reads —
    // **7** `40 86 43 83 08`, a `void *` result, which is `memset(p, 0, n)` on a
    // real pointer and which c2 lowers to `b <memset>` with a REL24, and **2**
    // that are not followed by a `40` at all (an ordinary literal 173, not a
    // selector). Requiring the int form keeps the first two apart at the token
    // rather than downstream.
    if seg.get(q..q + INT_TYPE.len())? != INT_TYPE {
        return None;
    }
    q += INT_TYPE.len();
    // The three int literals: the alignment hint c1xx adds, the byte count and
    // the fill. Their *values* are not constrained — the write lands in a
    // temporary this body's own grammar proves nothing else reads — but each
    // must be a literal, so a computed count (a real `memset` over a range)
    // cannot reach here.
    for _ in 0..3 {
        eat_lit_push(seg, &mut q)?;
    }
    // The destination, and the apply.
    let dest = eat_temp_addr(seg, &mut q)?;
    eat_push(seg, &mut q)?;
    if !eat_byte(seg, &mut q, 0x4C) {
        return None;
    }
    // The same temporary, bound (`44`, payload-free — `super::control_flow`) and
    // pushed as the reference argument.
    let again = eat_temp_addr(seg, &mut q)?;
    if again != dest {
        return None;
    }
    if !eat_byte(seg, &mut q, 0x44) {
        return None;
    }
    eat_push(seg, &mut q)?;
    *p = q;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `??$dr@PAHH@@YAXPAH0PAH@Z` — the whole `.ex` segment, transcribed
    /// verbatim from a live capture of `work/w-inl0/explore/e01.cpp` at the
    /// workload's own flag axes (`/GR /O1 /Oi /EHsc`), not hand-assembled. It is
    /// STLport's `__destroy_range` with the names shortened; c2's whole `.text`
    /// COMDAT for it is `4e800020`.
    const DEAD_TEMP_CALL: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0E, 0x53, 0x53, 0x26, 0x09, 0x0A,
        0x46, 0x2D, 0x07, 0x0A, 0x2D, 0x06, 0x0A, 0x2D, 0x05, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26,
        0x11, 0x0A, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x33, 0x86, 0x41,
        0x74, 0x80, 0xAD, 0x00, 0x00, 0x00, 0x40, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01,
        0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86,
        0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0x9B, 0x82, 0x16, 0x86, 0x20, 0x0B, 0x0A, 0x2C,
        0xA6, 0x43, 0x8D, 0x20, 0x00, 0x55, 0xA6, 0x43, 0x8D, 0x20, 0x4C, 0x9B, 0x82, 0x16, 0x86,
        0x20, 0x0B, 0x0A, 0x2C, 0xA6, 0x43, 0x8D, 0x20, 0x00, 0x44, 0x55, 0x86, 0x43, 0x8E, 0x20,
        0xB9, 0x06, 0x0A, 0x86, 0x43, 0xF4, 0x08, 0x55, 0x86, 0x43, 0xF4, 0x08, 0xB9, 0x05, 0x0A,
        0x86, 0x43, 0xF4, 0x08, 0x55, 0x86, 0x43, 0xF4, 0x08, 0x4C, 0x4B, 0x3A, 0x0A, 0x0A, 0x54,
        0x02, 0x29, 0x0A, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The token the `26` push carries — the callee
    /// (`??$aux@PAH@@YAXPAH0ABUtrue_tag@@@Z`).
    const DEAD_TEMP_CALLEE: u32 = 0x110A;

    /// Locate a byte run inside the pinned segment. Every mutation below edits a
    /// field it has *found*, never an index it has counted: an offset written by
    /// hand is a second transcription of the capture and can rot against it
    /// silently.
    fn at(pat: &[u8]) -> usize {
        crate::func::readers::find_subslice(DEAD_TEMP_CALL, pat)
            .unwrap_or_else(|| panic!("pattern {pat:02x?} is not in the pinned segment"))
    }

    #[test]
    fn the_dead_temporary_call_body_is_recognized_and_names_its_callee() {
        assert_eq!(no_effect_call(DEAD_TEMP_CALL), Some(DEAD_TEMP_CALLEE));
    }

    /// **The fact this module exists to keep true**: recognizing the body does
    /// **not** accept it. `parse_segment` still refuses, so the census, the gate
    /// and `IlBundle::functions` are all unchanged — board #971 condition 4.
    #[test]
    fn recognizing_the_body_does_not_accept_it() {
        use crate::func::test_fixtures::NO_LOCALS;
        assert!(crate::func::body::parse_segment(DEAD_TEMP_CALL, NO_LOCALS).is_none());
        let b = crate::func::body::parse_segment_detail(DEAD_TEMP_CALL, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-intrinsic-memset");
    }

    /// **The temporary must be the SAME temporary.** Repoint the argument's temp
    /// bind at a different token and the body is refused — the write and the use
    /// are not assumed to be the same slot, they are required to be.
    #[test]
    fn a_different_temporary_in_the_argument_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        let temp = &[0x9B, 0x82, 0x16, 0x86, 0x20, 0x0B, 0x0A];
        // The SECOND temp bind — the one the argument pushes.
        let second = crate::func::readers::find_subslice(&seg[at(temp) + 1..], temp).unwrap()
            + at(temp)
            + 1;
        seg[second + 6] = 0x0C;
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **A real `memset` is not this shape.** Give the intrinsic the `void*`
    /// result type the 7 pointer-`memset` sites of `BustAMovePanel.cpp` carry —
    /// the form `docs/IL_CAST_CONVERT.md` §1.3 records, which c2 lowers to
    /// `b <memset>` with a REL24 — and the recognizer declines rather than
    /// reading it as a dead temporary.
    #[test]
    fn a_pointer_result_memset_is_not_a_dead_temporary() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        let k = at(&[0x40, 0x86, 0x41, 0x74]);
        seg.splice(k..k + 4, [0x40, 0x86, 0x43, 0x83, 0x08]);
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **A different intrinsic is not this shape.** 172 is `memcpy`, whose
    /// expansion is also a REL24 tail call.
    #[test]
    fn a_different_intrinsic_selector_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg[at(&[0x80, 0xAD, 0x00, 0x00, 0x00, 0x40]) + 1] = 0xAC;
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **The result must be discarded.** Turn the `4B` into anything else and the
    /// walk refuses instead of reading a consumed value as emitting nothing.
    #[test]
    fn a_call_whose_result_is_not_discarded_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        let k = at(&[0x4C, 0x4B, 0x3A]);
        seg[k + 1] = 0x41;
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **The walk is TOTAL** — trailing bytes after the function tail refuse.
    /// This is what makes "the temporary is read nowhere else" structural rather
    /// than a search.
    #[test]
    fn trailing_bytes_after_the_function_tail_are_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg.push(0x26);
        seg.push(0x11);
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **The source-line markers a real body carries.** The pinned cell puts its
    /// call and its return on one line; every workload body does not, and a
    /// `4F 01 <line>` sits between the `4B` and the return plumbing. Splicing one
    /// in must not change the answer — measured the hard way, this reader fired
    /// **0** times on the whole workload until it ate them.
    #[test]
    fn a_line_marker_before_the_return_plumbing_is_eaten() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        let k = at(&[0x4C, 0x4B, 0x3A]);
        seg.splice(k + 2..k + 2, [0x4F, 0x01, 0x46]);
        assert_eq!(no_effect_call(&seg), Some(DEAD_TEMP_CALLEE));
    }

    /// **An ordinary void call with no temporary is not this shape** — it has its
    /// own accepted body shape and must not come through here as well.
    #[test]
    fn a_body_with_no_temporary_is_not_recognized() {
        // `4C 4F 11 53 26 <tok> BD <void> 00 <id> 4C 4B <plumbing>`.
        let seg = &[
            0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x53, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0x09,
            0x0A, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A,
            0x0A, 0x0A, 0x54, 0x02, 0x29, 0x0A, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        assert_eq!(no_effect_call(seg), None);
    }

    /// An argument that is neither a literal, a formal nor the temporary refuses
    /// the whole body — the vocabulary is closed, not a default.
    #[test]
    fn an_argument_outside_the_vocabulary_refuses_the_body() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg[at(&[0xB9, 0x06, 0x0A])] = 0x26; // a data-symbol push
        assert_eq!(no_effect_call(&seg), None);
    }

    /// A load that names something other than one of this function's own formals
    /// is refused: the formals list is read from the segment and consulted.
    #[test]
    fn a_load_of_a_non_formal_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg[at(&[0xB9, 0x06, 0x0A]) + 1] = 0x60; // a token the formals region does not list
        assert_eq!(no_effect_call(&seg), None);
    }

    // =====================================================================
    // THE DESTROY LOOP — `no_effect_loop`. The segment below is a live capture
    // of GRID-L's `l02`, whose source was frozen at `work/w-memset/CELLS.sha256`
    // before its first `cl.exe`; `crates/c2-harness/tests/destroy_loop_elision.rs`
    // grades the same source against real c2. These mutate the BYTES, which is
    // how the guards no hand cell can reach — a mismatched label, an ungraded
    // comparison opcode — are graded at all.
    // =====================================================================

    /// `?aux@@YAXPAH0ABUfalse_tag@@@Z` from GRID-L `l02`: the whole `.ex`
    /// segment, transcribed from a live capture at the workload's own flag axes
    /// and not hand-assembled. It is STLport's
    /// `__destroy_range_aux(_first, _last, __false_type)` with the names
    /// shortened, and real c2 emits one `4e800020` for its whole caller chain.
    const DESTROY_LOOP: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x08, 0x53, 0x53, 0x26, 0xF0, 0x09,
        0x46, 0x2D, 0xEF, 0x09, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x53,
        0x3A, 0xF2, 0x09, 0x29, 0xF3, 0x09, 0x26, 0xED, 0x09, 0x33, 0x86, 0x41, 0x12, 0x04, 0x0F,
        0x86, 0x43, 0xF4, 0x08, 0x4B, 0x29, 0xF2, 0x09, 0xB9, 0xED, 0x09, 0x86, 0x43, 0xF4, 0x08,
        0xB9, 0xEE, 0x09, 0x86, 0x43, 0xF4, 0x08, 0x20, 0x38, 0xF4, 0x09, 0x26, 0xEB, 0x09, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0xB9, 0xED, 0x09, 0x86, 0x43, 0xF4,
        0x08, 0x55, 0x86, 0x43, 0xF4, 0x08, 0x4C, 0x4B, 0x3A, 0xF3, 0x09, 0x29, 0xF4, 0x09, 0x54,
        0x03, 0x3A, 0xF1, 0x09, 0x54, 0x02, 0x29, 0xF1, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54,
        0x00,
    ];

    /// The token the loop's `26 <tok> BD` push carries — the callee `?leaf`.
    const DESTROY_LOOP_CALLEE: u32 = 0xEB09;

    /// Locate a byte run inside the pinned loop segment; see [`at`] for why a
    /// found offset and never a counted one.
    fn at_loop(pat: &[u8]) -> usize {
        crate::func::readers::find_subslice(DESTROY_LOOP, pat)
            .unwrap_or_else(|| panic!("pattern {pat:02x?} is not in the pinned loop segment"))
    }

    #[test]
    fn the_destroy_loop_body_is_recognized_and_names_its_callee() {
        assert_eq!(no_effect_loop(DESTROY_LOOP), Some(DESTROY_LOOP_CALLEE));
    }

    /// **The two shapes are disjoint.** A loop is not a dead-temporary call and
    /// a dead-temporary call is not a loop. The census asks one and then the
    /// other, so a body that answered both would make the field's meaning depend
    /// on the order of two readers.
    #[test]
    fn the_two_no_effect_shapes_are_disjoint() {
        assert_eq!(no_effect_call(DESTROY_LOOP), None);
        assert_eq!(no_effect_loop(DEAD_TEMP_CALL), None);
    }

    /// **THREE shapes now, and the third is the one that SEEDS** (board #1053).
    ///
    /// Every direction, and not only the new ones: `no_effect_nothing` is the only
    /// reader whose `true` is an *unconditional* claim, so a body reaching it that
    /// one of the other two would also read is the case where the fixpoint would
    /// get a seed where it should have got a link. It cannot happen — their
    /// vocabularies exclude each other's central token — and that is asserted here
    /// rather than left to the reading.
    #[test]
    fn the_three_no_effect_shapes_are_disjoint() {
        // The two CALL-bearing shapes never look like nothing.
        assert!(!no_effect_nothing(DEAD_TEMP_CALL));
        assert!(!no_effect_nothing(DESTROY_LOOP));
        // …and the nothing-body names no callee for either of them to return.
        assert_eq!(no_effect_call(NOTHING_BODY), None);
        assert_eq!(no_effect_loop(NOTHING_BODY), None);
    }

    /// **Recognizing the loop does not accept it** — the same containment
    /// `recognizing_the_body_does_not_accept_it` states for the call shape, and
    /// the whole of board #971 condition 4.
    #[test]
    fn recognizing_the_loop_does_not_accept_it() {
        use crate::func::test_fixtures::NO_LOCALS;
        assert!(crate::func::body::parse_segment(DESTROY_LOOP, NO_LOCALS).is_none());
        let b = crate::func::body::parse_segment_detail(DESTROY_LOOP, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "return-scope-close-cflow-label");
    }

    /// **EVERY LABEL IS MATCHED.** Repoint the `continue` branch at the exit
    /// label and the body is refused: the reader requires the three targets this
    /// shape mints, not merely three branches in the right order.
    ///
    /// **No hand cell can reach this guard** — every source-level perturbation
    /// GRID-L can express changes the statement sequence first, and the walk
    /// refuses there — so it is graded on the bytes or not at all.
    #[test]
    fn a_continue_that_does_not_name_the_step_label_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        // The `3A <Lincr>` that closes the loop body, told from the head's
        // `3A <Lcond>` by the label it names.
        let k = at_loop(&[0x4C, 0x4B, 0x3A, 0xF3, 0x09]);
        seg[k + 3] = 0xF4; // Lexit, not Lincr
        assert_eq!(no_effect_loop(&seg), None);
    }

    /// The head's `goto` must name the label that opens the exit test.
    #[test]
    fn a_head_branch_that_does_not_name_the_test_label_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        let k = at_loop(&[0x53, 0x53, 0x3A, 0xF2, 0x09]);
        seg[k + 3] = 0xF3; // Lincr, not Lcond
        assert_eq!(no_effect_loop(&seg), None);
    }

    /// The `38` branch-false must name the label that closes the loop.
    #[test]
    fn an_exit_branch_that_does_not_name_the_exit_label_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        let k = at_loop(&[0x20, 0x38, 0xF4, 0x09]);
        seg[k + 2] = 0xF3;
        assert_eq!(no_effect_loop(&seg), None);
    }

    /// **THE INDUCTION STEP MUST NAME A FORMAL.** `26` is also the data-symbol
    /// push, so a step that incremented a **global** would otherwise read as
    /// pure while the body materializes a data reference — `elide.rs`'s
    /// condition 3 one level down, and `w-fix`'s `k16`.
    #[test]
    fn an_induction_step_over_a_non_formal_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0x26, 0xED, 0x09, 0x33, 0x86, 0x41, 0x12]) + 1] = 0x60;
        assert_eq!(no_effect_loop(&seg), None);
    }

    /// The step's operator is `+=` and nothing else is graded.
    #[test]
    fn an_ungraded_induction_operator_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0x04, 0x0F, 0x86, 0x43, 0xF4, 0x08, 0x4B]) + 1] = 0x10;
        assert_eq!(no_effect_loop(&seg), None);
    }

    /// **The STRIDE is read and not matched** — `l12` carries 8 where this cell
    /// carries 4, and a reader that pinned either would be #644's mistake with a
    /// different field.
    #[test]
    fn the_induction_stride_is_read_and_not_matched() {
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0x12, 0x04, 0x0F, 0x86, 0x43, 0xF4, 0x08]) + 1] = 0x08;
        assert_eq!(no_effect_loop(&seg), Some(DESTROY_LOOP_CALLEE));
    }

    /// **The exit test reads only this function's own formals.**
    #[test]
    fn a_test_operand_that_is_not_a_formal_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0xB9, 0xEE, 0x09, 0x86, 0x43, 0xF4, 0x08, 0x20]) + 1] = 0x60;
        assert_eq!(no_effect_loop(&seg), None);
    }

    /// **An ungraded comparison opcode is refused**, and the one the grid *did*
    /// compile is taken. `20` is `!=` (this cell, and every workload site) and
    /// `22` is `<` (`l12`); a third byte here may not be a comparison at all.
    #[test]
    fn an_ungraded_comparison_opcode_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0x08, 0x20, 0x38, 0xF4, 0x09]) + 1] = 0x21;
        assert_eq!(no_effect_loop(&seg), None);
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0x08, 0x20, 0x38, 0xF4, 0x09]) + 1] = 0x22;
        assert_eq!(no_effect_loop(&seg), Some(DESTROY_LOOP_CALLEE));
    }

    /// **The walk is TOTAL** — trailing bytes after the function tail refuse,
    /// which is what makes "the induction variable is read nowhere after the
    /// loop" structural rather than a search.
    #[test]
    fn trailing_bytes_after_the_loop_are_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        seg.push(0x26);
        seg.push(0x11);
        assert_eq!(no_effect_loop(&seg), None);
    }

    /// **The loop's call must be DISCARDED**, and its arguments must stay inside
    /// the closed vocabulary the dead-temporary reader already owns — the two
    /// share one walk (`eat_no_effect_call_stmt`), so this is that sharing under
    /// test from the loop's side.
    #[test]
    fn a_loop_call_outside_the_argument_vocabulary_is_refused() {
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0x00, 0xB9, 0xED, 0x09, 0x86, 0x43, 0xF4, 0x08, 0x55]) + 1] = 0x26;
        assert_eq!(no_effect_loop(&seg), None);
        let mut seg = DESTROY_LOOP.to_vec();
        seg[at_loop(&[0x4C, 0x4B, 0x3A, 0xF3, 0x09]) + 1] = 0x41; // consumed, not discarded
        assert_eq!(no_effect_loop(&seg), None);
    }
    // =====================================================================
    // BOARD #1053 — THE SEED. `no_effect_nothing`, and the guards no `.cpp`
    // can reach.
    //
    // GRID-N's eleven cells grade the RULE against real c2. What they cannot
    // reach is a mistyped literal, a truncated statement or a spliced-in
    // second one: every source-level perturbation changes the statement
    // sequence first and the walk refuses there, so the census key moves and
    // the cell stops being about the guard. Those are graded on the bytes of
    // a pinned live capture or not at all — `w-memset` §4.1's finding, one
    // reader over.
    // =====================================================================

    /// `??$da@US@@@@YAXPAUS@@@Z` — the whole `.ex` segment, transcribed
    /// **verbatim** out of a live capture of GRID-N's `n01` at the workload's own
    /// flags (`work/w-seed/extractseg.py` on the bundle `c2rs census --keep-il`
    /// kept), not hand-assembled.
    ///
    /// It is `p->~T()` on a class with a trivial destructor — STLport's
    /// `__destroy_aux(_pointer, __false_type)` with the names shortened — and c2's
    /// whole `.text` COMDAT for it is `4e800020` with no relocation.
    ///
    /// **It is the workload's production and not a cell's dialect** (#953, which
    /// says that need not hold): the workload's own
    /// `??$__destroy_aux@V?$Key@M@@@stlpmtx_std@@...` on
    /// `src/lazer/meta_ham/CharacterProvider.cpp` reads
    ///
    /// ```text
    ///   4c 4f 11 53 4f 01 36 . 33 86 41 74 00 . 33 82 07 03 00 . 44 . 4b . 4f 01 38 . 3a ...
    /// ```
    ///
    /// and this segment reads the same, modulo the two `4F 01 <line>` markers a
    /// one-line cell does not carry — the difference `no_effect_call` already
    /// measured and eats.
    const NOTHING_BODY: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0E, 0x53, 0x53, 0x26, 0xF6, 0x09,
        0x46, 0x2D, 0xF4, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x33, 0x82,
        0x07, 0x03, 0x00, 0x44, 0x4B, 0x3A, 0x08, 0x0A, 0x54, 0x02, 0x29, 0x08, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The whole statement, as one run — every mutation below is expressed
    /// relative to a pattern it has FOUND, never to an index it counted.
    const NOTHING_STMT: &[u8] = &[
        0x33, 0x86, 0x41, 0x74, 0x00, 0x33, 0x82, 0x07, 0x03, 0x00, 0x44, 0x4B,
    ];

    fn at_nothing(pat: &[u8]) -> usize {
        crate::func::readers::find_subslice(NOTHING_BODY, pat)
            .unwrap_or_else(|| panic!("pattern {pat:02x?} is not in the pinned segment"))
    }

    /// The positive, and the whole rung in one line.
    #[test]
    fn the_pseudo_destructor_body_emits_nothing() {
        assert!(no_effect_nothing(NOTHING_BODY));
    }

    /// **Recognizing it does not ACCEPT it** — #971 condition 4, the containment
    /// every reader in this module is under. The census key is unchanged too,
    /// because a widening that moved it would be a widening of the parser.
    #[test]
    fn recognizing_the_nothing_body_does_not_accept_it() {
        use crate::func::test_fixtures::NO_LOCALS;
        assert!(crate::func::body::parse_segment(NOTHING_BODY, NO_LOCALS).is_none());
        let b = crate::func::body::parse_segment_detail(NOTHING_BODY, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-lit-type-8207");
    }

    /// **THE TOTALITY TERMINAL** — mutation **M1**'s target.
    ///
    /// The walk must reach the end of the segment. Without that, "there is nothing
    /// else in this body" is a search over the part that was walked rather than a
    /// property of the whole, and a seed asserts the whole.
    #[test]
    fn trailing_bytes_after_the_nothing_statement_are_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        seg.push(0x26);
        seg.push(0x11);
        assert!(!no_effect_nothing(&seg));
    }

    /// **A SECOND STATEMENT is refused**, spliced in at the byte level so the
    /// statement sequence is the only thing that changed. GRID-N's `n10` is the
    /// source-reachable half; this is the one that pins the walk.
    #[test]
    fn a_second_nothing_statement_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(NOTHING_STMT);
        seg.splice(k..k, NOTHING_STMT.iter().copied());
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE DISCARD IS REQUIRED.** `4B` is what says the statement's value is
    /// thrown away; without it the walk would be reading a fragment of a larger
    /// expression as a whole body.
    #[test]
    fn a_nothing_statement_whose_value_is_consumed_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        seg[at_nothing(&[0x44, 0x4B]) + 1] = 0x41;
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE BIND IS REQUIRED.** `44` is consumed as a byte, not as an operator
    /// whose arity this module claims to know — but it is consumed, so a statement
    /// without it is a statement this reader has not seen.
    #[test]
    fn a_nothing_statement_without_the_bind_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x44, 0x4B]);
        seg.remove(k);
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE LITERAL TYPES ARE PINNED**, and this is the soundness half rather
    /// than a fit. A `float`/`double` literal drags `_fltused` into the TU and the
    /// obj **grows a symbol** — `CallRet::discarded`'s reason one operand over — so
    /// the two types the capture carries are the only two admitted.
    ///
    /// `86 41 12` is `long`, which c2 treats identically to `int` for every
    /// operator this crate accepts; it is refused here anyway, because the pin is
    /// the exact [`INT_TYPE`] triple and widening it is a decision with a cell
    /// behind it, not a convenience.
    #[test]
    fn a_nothing_literal_of_another_type_is_refused() {
        for wrong in [
            [0x86u8, 0x41, 0x12], // long
            [0x86, 0x42, 0x75],   // unsigned
            [0x88, 0x85, 0x41],   // double — the one that would grow a symbol
        ] {
            let mut seg = NOTHING_BODY.to_vec();
            let k = at_nothing(&[0x33, 0x86, 0x41, 0x74, 0x00]);
            seg.splice(k + 1..k + 4, wrong);
            assert!(
                !no_effect_nothing(&seg),
                "a literal typed {wrong:02x?} was read as the int operand"
            );
        }
        // …and the VOID operand's tag/kind pair, one bit over.
        let mut seg = NOTHING_BODY.to_vec();
        seg[at_nothing(&[0x33, 0x82, 0x07, 0x03, 0x00]) + 2] = 0x08;
        assert!(!no_effect_nothing(&seg));
    }

    /// **THE LITERAL VALUES ARE NOT CONSTRAINED**, and that is #644's rule applied
    /// rather than an oversight: a literal is pure whatever its value and the
    /// statement is discarded, so the value cannot change what is emitted.
    /// Constraining a field that happens to be constant on one corpus is exactly
    /// the mistake `no_effect_call`'s align/count/fill avoid.
    ///
    /// The **void type's id** is not constrained either — it is a per-TU type-table
    /// index, and pinning it would make this reader a property of one bundle.
    #[test]
    fn the_nothing_values_and_the_void_type_id_are_not_constrained() {
        let mut seg = NOTHING_BODY.to_vec();
        seg[at_nothing(&[0x33, 0x86, 0x41, 0x74, 0x00]) + 4] = 0x07; // int value
        assert!(no_effect_nothing(&seg));

        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x33, 0x82, 0x07, 0x03, 0x00]);
        seg[k + 3] = 0x09; // the void TYPE's id
        seg[k + 4] = 0x7F; // the void literal's value
        assert!(no_effect_nothing(&seg));
    }

    /// **THE FORMAL-LOAD VARIANT IS DECLINED, and it is the residue this lane
    /// publishes rather than takes.**
    ///
    /// The workload carries a second body under the same census key: where a class
    /// element type with a trivial destructor folds `p` away to a literal, an
    /// **enum** element type keeps it, and
    /// `??$__destroy_aux@W4CubeFace@RndCubeTex@@...` on
    /// `src/system/rndobj/CubeTex.cpp` reads
    ///
    /// ```text
    ///   4c 4f 11 53 4f 01 36 . b9 <formal> 86 43 c9 50 . 33 82 07 03 00 . 44 . 4b . ...
    /// ```
    ///
    /// — a formal LOAD where this one has an int literal. It is very probably just
    /// as pure, and it is **not admitted**: GRID-N has no cell for it, and adding
    /// the arm now would be fitting a reader so that four more functions convert.
    /// It is worth exactly **4** `fnbyte-differs`
    /// (`fnbyte-blr-stop3-expr-lit-type-8207` at the tip) and it is board **#1090**.
    #[test]
    fn a_formal_load_in_place_of_the_int_literal_is_declined() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x33, 0x86, 0x41, 0x74, 0x00]);
        // `B9 <formal-tok> <TYPE>` — the shape CubeTex.cpp carries, using this
        // segment's own formal token so nothing but the operand form changed.
        seg.splice(k..k + 5, [0xB9, 0xF4, 0x09, 0x86, 0x43, 0xC9, 0x50]);
        assert!(
            !no_effect_nothing(&seg),
            "the formal-load variant was admitted — it is a SECOND production and \
             GRID-N graded no cell for it (board #1090)"
        );
    }

    /// **A DEEPER SCOPE IS REFUSED.** A body that opens a block is a body with a
    /// statement this reader has not walked, and the walk would read the block's
    /// first statement as the whole body.
    #[test]
    fn a_nothing_body_that_opens_a_deeper_scope_is_refused() {
        let mut seg = NOTHING_BODY.to_vec();
        let k = at_nothing(&[0x4C, 0x4F, 0x11, 0x53]);
        seg.insert(k + 4, 0x53);
        assert!(!no_effect_nothing(&seg));
    }
}

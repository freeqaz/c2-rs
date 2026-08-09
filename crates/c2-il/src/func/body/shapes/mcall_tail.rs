//! **W36 — the member call as a whole body**: `p->m(a…);` and `return p->m(a…);`
//! where `p` is a plain pointer-valued formal.
//!
//! **W41** adds the two things the 10,494-function residue of that row turned out
//! to be: a **pointer conversion on the receiver** (440), and a **literal `± k`
//! on the result**, which makes the body a *framed* non-leaf call rather than a
//! tail one (3,559) — see [`framed_member_call`].
//!
//! ```text
//!   26 <method>              push the method symbol      the callee
//!   B9 <recv> <TYPE ptr4>    the receiver value          `this`
//!   [ 2C <TYPE ptr4> 00 ]    a pointer conversion on it  (W41)
//!   99 <TYPE ptr4> 00        bind it as argument zero
//!   BD <ret TYPE> 00 <id>    the CALL token
//!   ( <operands> 55 <T> )*   the explicit arguments, rightmost first
//!   4C                       apply
//!     4B | 41 <T>                    statement end, or the result is returned
//!   | 33 <T> k (02|03) 41 <T>        …or `± k` and then returned  (W41, framed)
//! ```
//!
//! **The whole production is the existing tail call with one extra argument
//! slot.** `p->m(x)` is `m(p, x)` on this ABI — `this` is argument zero, in r3 —
//! so the emission is a register permutation over the formals plus `b <method>`,
//! which is exactly [`super::calls::tail_call_shape`]'s job and needs **no
//! codegen at all**: the receiver is appended to the argument list as slot 0 and
//! everything downstream (the identity case that emits nothing, the single
//! `mr r3,rN`, the permutation walk with its measured 3-cycle limit, the
//! `.gl` callee resolution, `.pdata`, `/Gy`) is the code that already grades.
//!
//! ## Why this row was invisible
//!
//! `expr-op-0x99` was **280,283 functions, 11.4 % of everything blocked and the
//! largest single key on the board** — and it was never a missing token. The body
//! dispatch consumes a statement-head `26 <method>` as an assignment
//! *destination* (the byte after it is the receiver's `B9`, not a `BD`), the
//! assignment parser hands the rest to `parse_expr`, and `parse_expr` reads the
//! receiver as an ordinary LOAD and stops on the `99` under its generic `expr`
//! fall-through. So the construct was filed as an **opcode** while the identical
//! production one byte different — `x = p->m();` — reached
//! [`crate::func::body::mcall::classify`] and was filed as a member call all
//! along. `GAPS.md` §6's unstable-*attribution* hazard, in the form that costs
//! coverage rather than correctness: the row carried no whole-body-completeness
//! bit at all, so no ranking taken from it could see what was complete behind it.
//!
//! [`crate::func::body::mcall::reanchor_chain`] now repairs the anchor, which
//! de-conflates the row 1:1 into the `expr-call-in-expr-recv-*` family and prints
//! its `-whole` counts. This file takes the largest sub-shape those counts name.
//!
//! ## What is refused, and why each refusal is a *measurement*
//!
//! Everything the [`super::calls`] tail call already refuses (a computed
//! argument in a multi-argument call, a non-formal argument, a duplicated one, a
//! multi-cycle or >3-cycle permutation, a non-cdecl convention, more than eight
//! arguments) is refused here through the **same** locator, under the same census
//! keys — there is no second copy of any of it. On top of that:
//!
//! * a receiver that is not a plain `B9 <tok> <ptr4 TYPE>` — a member
//!   (`p->q.m()`), a dereference, a named object, another call's result, an
//!   adjusted base (`intrinsic 2113`) or a chain. Each is a *different* receiver
//!   production with its own lowering, and the census already names them
//!   (`expr-call-in-expr-recv-field`, `-recv-deref`, `-recv-object`,
//!   `-recv-call`, `-recv-intrinsic-this-adjust`, `-chained`);
//! * a **non-zero `99` bind offset**. `docs/IL_EXPR_LAYER.md` §7 records that
//!   field as UNKNOWN and zero at every observed site, and a field that never
//!   varied is indistinguishable from a constant (`GAPS.md` §6) — so it is
//!   required literally and its exceptions get their own key rather than being
//!   skipped;
//! * a body that does not **end** at this call: a second statement after it is
//!   the Class A statement-call sequence with a member call in it, which is a
//!   further rung and is refused by name here rather than routed into a
//!   production that has never been graded with a receiver argument.

use crate::func::body::expr::{eat_return_plumbing, eat_scopes, BODY_SCOPE_DEPTH};
use crate::func::body::{blk, prod_tag, Block, BodyShape};
use crate::func::readers::{
    eat_byte, eat_operand_type, eat_value_type, is_ptr4_kind, read_token_var, read_type,
    read_varint, ValueClass,
};
use crate::func::IlOp;

use super::calls::{
    arg_loads_are_formals, ArgSite, eat_call_args, eat_call_postop, eat_call_token, eat_callee_push,
    eat_sym_addr_value, tail_call_shape, MAX_REGISTER_FORMALS,
};
use super::params::parse_params;

// ---------------------------------------------------------------------------
// W-ARMS — the receiver-designator site, decomposed (board #142)
// ---------------------------------------------------------------------------

/// **The construct standing where the receiver designator should be.**
///
/// §9.13 measured the three receiver-designator sites at 37,060 blocked emitted
/// functions — the largest single site on the emitted board, larger than any
/// census key — and could say nothing about what is *in* them, because all three
/// productions map [`eat_receiver_this`]'s `Err(Block)` onto one flat tag and
/// **throw the `Block` away**. The site was one undifferentiated bucket by
/// construction, so #142 ("the other clean-not-whole receiver arms") had no
/// instrument at all: the census key those rows carry is minted by whichever
/// reader stopped *last*, which §9.13 showed is a different reader.
///
/// This refines the tag in place and keeps the old name as a **prefix**
/// (`<old site>/<construct>`), so every published figure keyed on the old string
/// is recovered by a prefix test and nothing that ranked on it silently changes
/// meaning.
///
/// Three properties make it a measurement rather than a relabelling:
///
/// 1. **It names the CONSTRUCT, never the position** — the rule
///    [`crate::func::body::mcall::Fail::blocker`] states for the completeness
///    walker, and the same vocabulary (`off-add`, `deref-load`, `plain-call`,
///    `virtual`, `temp-bind`, `convert`, `ternary`, `call-in-expr`), so the two
///    axes can be crossed without a translation table. A byte with no construct
///    behind it gets an **honest hex bucket**, which is a result; "other" would
///    be an absence, and an absence cannot be ranked.
/// 2. **It is total.** Every `(ctx, byte)` [`eat_receiver_this`] can produce has
///    a name, including EOF and every one of the 256 byte values, and
///    `every_receiver_refusal_has_a_name` enumerates the domain rather than
///    sampling it. §9.14.6's finding is the precedent: a witness list misses the
///    class that has no witnesses.
/// 3. **It is read-only over the census.** It changes no verdict, no acceptance
///    and no count; only the `prod` axis's string. Asserted by re-running the
///    whole 878-TU scan and comparing every published number, because an
///    instrument whose inertness is argued rather than run is this project's
///    dominant failure mode.
///
/// The intrinsic arm is split by **selector**, not lumped: 2113 `this-adjust` is
/// board #127/#140 and was measured at 472 emitted, while 2117
/// `base-member-addr` is a designator with an entirely different lowering. One
/// bucket over both would have re-created exactly the conflation §9.13 spent a
/// lane undoing.
struct RecvVocab {
    /// No `B9` at all — the designator opens on something else.
    no_b9: RecvSlot,
    /// A `B9` receiver was read and the `99` bind is not where it must be.
    then: RecvSlot,
    b9_token_unreadable: &'static str,
    b9_not_a_ptr4: &'static str,
    b9_convert_not_class_preserving: &'static str,
    bind_type_not_ptr4: &'static str,
    bind_offset_nonzero: &'static str,
    bind_tail_unreadable: &'static str,
    /// A refusal context this table does not know. Unreachable by construction
    /// and **printed** rather than folded into a neighbour, so that a future
    /// context added to [`eat_receiver_this`] shows up as its own row instead of
    /// silently joining one that is already being ranked.
    ctx_unknown: &'static str,
}

/// One byte position's names: the seven class-layout intrinsics by selector, the
/// named constructs, EOF, and the 256-entry honest hex table.
struct RecvSlot {
    this_adjust: &'static str,
    base_upcast: &'static str,
    base_downcast: &'static str,
    vbase_upcast: &'static str,
    base_member_addr: &'static str,
    vbase_member_addr: &'static str,
    dynamic_cast: &'static str,
    intrinsic_other: &'static str,
    off_add: &'static str,
    literal: &'static str,
    deref_load: &'static str,
    store: &'static str,
    operand_load: &'static str,
    chain_bind: &'static str,
    stmt_end: &'static str,
    branch: &'static str,
    plain_call: &'static str,
    call_in_expr: &'static str,
    virtual_dispatch: &'static str,
    temp_bind: &'static str,
    convert: &'static str,
    ternary: &'static str,
    class_descriptor: &'static str,
    eof: &'static str,
    op: [[&'static str; 16]; 16],
}

/// `<prefix>op-0x<hi><lo>` for one high nibble.
macro_rules! recv_ops_row {
    ($s:literal, $q:literal, $hi:literal) => {
        [
            concat!($s, $q, "op-0x", $hi, "0"), concat!($s, $q, "op-0x", $hi, "1"),
            concat!($s, $q, "op-0x", $hi, "2"), concat!($s, $q, "op-0x", $hi, "3"),
            concat!($s, $q, "op-0x", $hi, "4"), concat!($s, $q, "op-0x", $hi, "5"),
            concat!($s, $q, "op-0x", $hi, "6"), concat!($s, $q, "op-0x", $hi, "7"),
            concat!($s, $q, "op-0x", $hi, "8"), concat!($s, $q, "op-0x", $hi, "9"),
            concat!($s, $q, "op-0x", $hi, "a"), concat!($s, $q, "op-0x", $hi, "b"),
            concat!($s, $q, "op-0x", $hi, "c"), concat!($s, $q, "op-0x", $hi, "d"),
            concat!($s, $q, "op-0x", $hi, "e"), concat!($s, $q, "op-0x", $hi, "f"),
        ]
    };
}

/// One byte position's whole vocabulary under `<site>/<position->`.
macro_rules! recv_slot {
    ($s:literal, $q:literal) => {
        RecvSlot {
            this_adjust: concat!($s, $q, "this-adjust"),
            base_upcast: concat!($s, $q, "base-upcast"),
            base_downcast: concat!($s, $q, "base-downcast"),
            vbase_upcast: concat!($s, $q, "vbase-upcast"),
            base_member_addr: concat!($s, $q, "base-member-addr"),
            vbase_member_addr: concat!($s, $q, "vbase-member-addr"),
            dynamic_cast: concat!($s, $q, "dynamic-cast"),
            intrinsic_other: concat!($s, $q, "intrinsic-other"),
            off_add: concat!($s, $q, "off-add"),
            literal: concat!($s, $q, "literal"),
            deref_load: concat!($s, $q, "deref-load"),
            store: concat!($s, $q, "store"),
            operand_load: concat!($s, $q, "operand-load"),
            chain_bind: concat!($s, $q, "chain-bind"),
            stmt_end: concat!($s, $q, "stmt-end"),
            branch: concat!($s, $q, "branch"),
            plain_call: concat!($s, $q, "plain-call"),
            call_in_expr: concat!($s, $q, "call-in-expr"),
            virtual_dispatch: concat!($s, $q, "virtual"),
            temp_bind: concat!($s, $q, "temp-bind"),
            convert: concat!($s, $q, "convert"),
            ternary: concat!($s, $q, "ternary"),
            class_descriptor: concat!($s, $q, "class-descriptor"),
            eof: concat!($s, $q, "eof"),
            op: [
                recv_ops_row!($s, $q, "0"), recv_ops_row!($s, $q, "1"), recv_ops_row!($s, $q, "2"),
                recv_ops_row!($s, $q, "3"), recv_ops_row!($s, $q, "4"), recv_ops_row!($s, $q, "5"),
                recv_ops_row!($s, $q, "6"), recv_ops_row!($s, $q, "7"), recv_ops_row!($s, $q, "8"),
                recv_ops_row!($s, $q, "9"), recv_ops_row!($s, $q, "a"), recv_ops_row!($s, $q, "b"),
                recv_ops_row!($s, $q, "c"), recv_ops_row!($s, $q, "d"), recv_ops_row!($s, $q, "e"),
                recv_ops_row!($s, $q, "f"),
            ],
        }
    };
}

/// The whole vocabulary for one production arm, under its own site prefix.
macro_rules! recv_vocab {
    ($s:literal) => {
        RecvVocab {
            no_b9: recv_slot!($s, "/no-b9-"),
            then: recv_slot!($s, "/then-"),
            b9_token_unreadable: concat!($s, "/b9-token-unreadable"),
            b9_not_a_ptr4: concat!($s, "/b9-not-a-ptr4"),
            b9_convert_not_class_preserving: concat!($s, "/b9-convert-not-class-preserving"),
            bind_type_not_ptr4: concat!($s, "/bind-type-not-ptr4"),
            bind_offset_nonzero: concat!($s, "/bind-offset-nonzero"),
            bind_tail_unreadable: concat!($s, "/bind-tail-unreadable"),
            ctx_unknown: concat!($s, "/ctx-unknown"),
        }
    };
}

static RECV_TAIL: RecvVocab = recv_vocab!("tail-recv-not-a-plain-b9-load");
static RECV_CHAIN: RecvVocab = recv_vocab!("chain-recv-not-a-plain-b9-load");
static RECV_CMP: RecvVocab = recv_vocab!("cmp-second-recv-not-a-plain-b9-load");

impl RecvSlot {
    /// Name the construct at `at`. `seg` is needed twice over, and both are
    /// measurements this axis got wrong on its first run:
    ///
    /// * the **intrinsic selector** — the `33` byte alone says "a literal opens
    ///   here", and lumping 2113 with 2117 would re-create the conflation §9.13
    ///   undid;
    /// * the **offset add** — `33 <int-like> <k>` followed by `27`/`28` is a
    ///   *byte-offset add on the designator* (`p->f.m()`), and the literal is
    ///   only the operand that feeds it. The first version of this table filed
    ///   5,806 emitted functions under `op-0x33`, the byte the run stopped in
    ///   front of, which is precisely the defect §9.14.7 records for `op-0x55`
    ///   and which #139 exists to cure. It is named for the construct now, and
    ///   the two-token lookahead is what makes that possible.
    fn at(&self, seg: &[u8], at: usize) -> &'static str {
        let Some(&b) = seg.get(at) else { return self.eof };
        match b {
            0x33 => match crate::func::body::expr::intrinsic_selector(seg, at) {
                Some(2113) => self.this_adjust,
                Some(2114) => self.base_upcast,
                Some(2115) => self.base_downcast,
                Some(2116) => self.vbase_upcast,
                Some(2117) => self.base_member_addr,
                Some(2118) => self.vbase_member_addr,
                Some(2119) => self.dynamic_cast,
                Some(_) => self.intrinsic_other,
                // Not an intrinsic head: a plain literal push. Look one token on
                // — an `27`/`28` behind it makes the whole run an offset add.
                None => {
                    let mut q = at + 1;
                    let is_off_add = crate::func::readers::eat_int_like(seg, &mut q)
                        && crate::func::readers::read_varint(seg, &mut q).is_some()
                        && matches!(seg.get(q), Some(&0x27) | Some(&0x28));
                    if is_off_add {
                        self.off_add
                    } else {
                        self.literal
                    }
                }
            },
            0x27 | 0x28 => self.off_add,
            0x30 => self.deref_load,
            // The indirect STORE — `mcall`'s own walk stops on this byte with
            // `Stop::Store`. A statement that ends in one is an ASSIGNMENT, not
            // a call: the body dispatch offers every statement-head `26` to this
            // production, so a store landing here says the production was
            // entered speculatively and there is no receiver at all.
            0x32 => self.store,
            0xB9 => self.operand_load,
            0x99 => self.chain_bind,
            0x41 | 0x4B => self.stmt_end,
            0x38 | 0x39 => self.branch,
            0xBD => self.plain_call,
            0x26 => self.call_in_expr,
            0x67 | 0x9A => self.virtual_dispatch,
            0x9B => self.temp_bind,
            0x2C => self.convert,
            0x43 => self.ternary,
            0x66 => self.class_descriptor,
            _ => self.op[(b >> 4) as usize][(b & 0x0F) as usize],
        }
    }
}

impl RecvVocab {
    /// Render one [`eat_receiver_this`] refusal as a census-ready `prod` value.
    fn tag(&self, seg: &[u8], b: &Block) -> &'static str {
        match b.ctx {
            "mcall-recv" => self.no_b9.at(seg, b.off),
            "mcall-recv-tok" => self.b9_token_unreadable,
            "mcall-recv-type" => self.b9_not_a_ptr4,
            "mcall-recv-convert" => self.b9_convert_not_class_preserving,
            "mcall-bind" => self.then.at(seg, b.off),
            "mcall-bind-type" => self.bind_type_not_ptr4,
            "mcall-bind-offset" => self.bind_offset_nonzero,
            "mcall-bind-tail" => self.bind_tail_unreadable,
            _ => self.ctx_unknown,
        }
    }
}

/// Which production arm asked. Three arms, three site prefixes — kept apart
/// because §9.13's own table is per arm and #128 moved 11,406 emitted functions
/// *between* them, which a merged bucket could not have shown.
#[derive(Clone, Copy)]
pub(crate) enum RecvArm {
    Tail,
    Chain,
    CmpSecond,
}

/// The refined `prod` tag for a receiver-designator refusal. Always returns
/// `None`, exactly as [`prod_tag`] does, so it drops into the existing
/// non-committal bail idiom without changing what any production returns.
pub(crate) fn recv_prod_tag(arm: RecvArm, seg: &[u8], b: &Block) -> Option<Block> {
    let v = match arm {
        RecvArm::Tail => &RECV_TAIL,
        RecvArm::Chain => &RECV_CHAIN,
        RecvArm::CmpSecond => &RECV_CMP,
    };
    prod_tag(v.tag(seg, b))
}

/// **W-ADJUST — a NAMED DATA OBJECT standing as the receiver**: `gObj.m(a)`,
/// where the receiver designator is a data symbol's *address* rather than a
/// pointer already in a register.
///
/// ```text
///   26 <sym>                 the object symbol
///   [ 2C <TYPE ptr4> <b> ]   at most one cv-strip / decay convert
///   99 <TYPE ptr4> 00        bind it as argument zero
/// ```
///
/// Returns the symbol token with the cursor past the bind, or `None` with the
/// cursor **untouched** — the caller then hands the same `26` to the chain
/// production, which is what it used to get unconditionally.
///
/// The address itself comes from the one locator WR1 wrote for it
/// ([`super::calls::eat_sym_addr_value`]), asked for a `99` terminator instead of
/// a `55` one. A second copy of that walk is exactly the drift `docs/GAPS.md` §6
/// instance #9 records, and here it would decide independently whether an offset
/// run or a `30` load may ride along — the two spellings whose emission is a
/// third instruction WR1 measured and refused.
fn eat_receiver_object(seg: &[u8], p: &mut usize) -> Option<u32> {
    let save = *p;
    let tok = eat_sym_addr_value(seg, p, 0x99)?;
    if eat_this_bind(seg, p).is_err() {
        *p = save;
        return None;
    }
    Some(tok)
}

/// Try the member-call body at `start` (the statement-head `26`).
///
/// `Err(None)` means **not this production** — the cursor is untouched, no census
/// key moves, and the caller falls through to the assignment parse exactly as
/// before. That is the non-committal contract every other `try_parse_*` has.
///
/// `Err(Some(b))` means **this IS the production, and it parsed to the end of the
/// segment**, but a codegen-class gate refuses it. Those refusals are reported
/// under their own keys rather than swallowed, because `GAPS.md` §6 records the
/// rule twice: *give a new gate a key on the way in, not after someone asks what it
/// cost*, and *a gate raised after the whole-body parse succeeds is free to measure,
/// because its refusals are already complete bodies*. Without this the 11,052
/// bodies the grammar measure calls complete and the argument gates refuse would sit
/// invisibly inside `expr-call-in-expr-recv-load-whole` and the rung's own residue
/// would be a rumour.
///
/// `depth` is the lexical depth the statement parse reached, so a braced body
/// (`void f(A* p){ { p->m(); } }`) closes its scopes exactly rather than being read
/// as a shorter one — the same requirement every other shape's plumbing carries.
pub(crate) fn try_parse_member_tail_call(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Result<BodyShape, Option<Block>> {
    let mut p = start;
    // The dispatch arm already matched the `26`, so the only way this half can
    // decline is a method-symbol token the varint reader cannot spell.
    let callee_tok =
        eat_callee_push(seg, &mut p).map_err(|_| prod_tag("tail-method-symbol-token-unreadable"))?;
    // **A second method symbol where the receiver must be**: the method pushes
    // stack LIFO, so `p->a()->b()` is `26 <b> 26 <a> B9 <p> …` and the `26` this
    // production just read as its callee is the *outermost* method of a chain.
    // See [`super::mcall_chain`] — the largest `-whole` row on the board, and the
    // reason the split is here rather than one locator deeper: `eat_receiver_this`
    // must keep meaning "the receiver designator", not "the receiver or some
    // number of further methods".
    // **W-ADJUST — …or a NAMED DATA OBJECT standing where the receiver goes.**
    // `gObj.m(a)` pushes the object's *symbol*, so its designator opens on the
    // same `26` a stacked method does and the chain production used to get every
    // one of them ([`eat_receiver_object`] is the discriminator, and it declines
    // with the cursor untouched so a real chain reaches the chain production
    // exactly as before).
    let mut recv_sym: Option<u32> = None;
    if seg.get(p) == Some(&0x26) {
        match eat_receiver_object(seg, &mut p) {
            Some(tok) => recv_sym = Some(tok),
            None => {
                return super::mcall_chain::try_parse_member_chain_call(
                    seg, p, lo, depth, callee_tok,
                )
            }
        }
    }
    let recv_tok = match recv_sym {
        Some(tok) => tok,
        None => eat_receiver_this(seg, &mut p)
            .map_err(|b| recv_prod_tag(RecvArm::Tail, seg, &b))?,
    };
    // The `BD` this call's result TYPE hangs off, kept because
    // [`super::mcall_cmp`] needs the type's **signedness** and `eat_call_token`
    // resolves it only to real / not-real. Recorded rather than re-found: going
    // back to it from a later cursor is the kind of reverse scan that reads a
    // different byte when the receiver's encoding changes width.
    let ret_at = p;
    let ret = eat_call_token(seg, &mut p)
        .map_err(|_| prod_tag("tail-no-cdecl-call-token-after-the-receiver"))?;
    let mut args = eat_call_args(seg, &mut p)
        .map_err(|_| prod_tag("tail-argument-not-in-the-operand-vocabulary"))?;

    // `this` is argument slot 0, and the argument list is in **stream** order —
    // rightmost source argument first, so slot `i` is `args[len-1-i]`. The receiver
    // therefore goes on the END of the list, not the front. Getting this backwards
    // is invisible on a nullary call and on any call whose permutation happens to be
    // symmetric, which is exactly the shape of defect `GAPS.md` §6 keeps recording,
    // so `member_tail_call_puts_this_in_slot_zero` pins it against a capture where
    // the two readings differ.
    //
    // **W-ADJUST**: a named object's address is a *relocation*, not a load, so it
    // enters the slot list as [`IlOp::SymAddr`] and takes WR1's
    // `sym_addr_tail_call` path — the `lis`/`addi` quad plus whatever the other
    // slots need, with the address `addi` last. Handing it `Load` instead would
    // be a `mr` from a register that holds a *token id*, i.e. wrong bytes rather
    // than a refusal, which is why the discrimination is made here at the point
    // the slot is built and not left to codegen.
    args.push(match recv_sym {
        Some(tok) => vec![IlOp::SymAddr(tok)],
        None => vec![IlOp::Load(recv_tok)],
    });

    // The body must END here. Either the result is discarded (`4B`) and the return
    // is void, or it is the returned value (`41 <TYPE>`, consumed by the plumbing).
    // Both lower to the same bare tail branch — the callee leaves its result in the
    // register the caller's own return would use — so the two arms differ only in
    // which plumbing they require.
    //
    // A body that does NOT end here is a second statement after the call — the
    // statement-call SEQUENCE with a member call in it. This paragraph used to
    // read *"a further rung … refused by name here rather than routed into a
    // production that has never been graded with a receiver argument"*, and
    // **lane `w-mcall` is that rung** (board **#1962**): the sequence is
    // attempted below, and only when it declines does the body fall through to
    // the assignment parse and keep its de-conflated
    // `expr-call-in-expr-recv-load-*` key.
    let mut depth = depth;
    if eat_byte(seg, &mut p, 0x4B) {
        // The result is discarded, so a `float`/`double` one still obliges the TU
        // to carry `_fltused` and the port has no model of that — see
        // [`super::calls::CallRet`] and `docs/GAPS.md` §6 instance #14.
        ret.discarded(seg, p).map_err(Some)?;
        // **W-MCALL — the statement-call SEQUENCE whose first statement is this
        // member call** (lane `w-mcall`, board **#1962**).
        //
        // `p->m(a…);` is `m(p, a…)` on this ABI, so this is a statement-position
        // call with one more argument slot, and `args` already carries the
        // receiver in slot 0 from the push above. [`BodyShape::CallSeq`] lowers
        // a sequence of those byte-exactly at Class A and Class B alike, so the
        // whole rung is this route plus the one in
        // [`super::calls::parse_call_sequence_from`] — and **no byte of
        // `crates/c2-core` moves**.
        //
        // Run on a SCRATCH cursor, and the production tag is re-armed on
        // failure. `prod_tag` is last-write-wins
        // (`mod.rs::prod_tag_is_the_seam_the_member_call_productions_write_against`),
        // so a failed attempt would otherwise overwrite this production's own
        // tag and move the `prod` axis for bodies whose verdict did not change.
        // That hazard is `work/w-mcall/PREREG.md` §2.2, frozen before the code.
        //
        // **The depth gate is INERT on every cell tried, and it is kept
        // fail-closed anyway — measured, not argued** (board **#1963**). The
        // sequence loop asks for the return plumbing at [`BODY_SCOPE_DEPTH`], so
        // a braced body (`void f(A* p){ { p->m(); p->n(); } }`) would be read at
        // the wrong depth. A scratch counterfactual replaced this condition with
        // `true` and re-censused `work/w-mcall/probe/p4.cpp`: the braced cell's
        // verdict **did not move** — the loop's own plumbing parse refuses it on
        // the unconsumed `54 03` regardless. So this clause holds nothing today
        // and is not claimed as a fence that fires; board **#1148**'s lesson is
        // that a recorded unreachability is a statement about the cells someone
        // thought of, and the conservative direction is to keep the guard.
        if depth == BODY_SCOPE_DEPTH {
            let mut q = p;
            if let Ok(shape) = super::calls::parse_call_sequence_from(
                seg,
                &mut q,
                lo,
                vec![(callee_tok, args.clone())],
                None,
                Vec::new(),
            ) {
                return Ok(shape);
            }
            // Re-arm: the failed attempt above may have written a tag of its own.
            prod_tag("tail-void-body-does-not-end-at-the-call");
        }
        // A brace scope closes **between** the statement end and the return
        // branch, not after it: `void f(A* p){ { p->m(); } }` captures
        // `… 4C 4B · 54 03 · 3A <lbl> · 54 02 · 29 <lbl> …`, so the inner close
        // sits on the far side of the `3A` from the outer one and
        // `eat_return_head`'s own run — which starts after the branch — cannot
        // reach it. Consumed here at the statement boundary, exactly as
        // `try_parse_assign_body_detail` consumes it between two statements, and
        // the plumbing is then asked for the depth that is actually left.
        eat_scopes(seg, &mut p, &mut depth)
            .map_err(|_| prod_tag("tail-void-brace-scopes-do-not-close"))?;
        eat_return_plumbing(seg, &mut p, false, depth)
            .map_err(|_| prod_tag("tail-void-body-does-not-end-at-the-call"))?;
    } else if seg.get(p) == Some(&0x41) {
        eat_return_plumbing(seg, &mut p, true, depth)
            .map_err(|_| prod_tag("tail-returned-body-does-not-end-at-the-call"))?;
    } else if recv_sym.is_some() {
        // **W-ADJUST's boundary, stated as its own key.** The two productions
        // below both re-spell the receiver as `IlOp::Load(recv_tok)` from their
        // own copy of the token — `mcall_cmp` to build the second call's slot
        // list, `framed_member_call` to rebuild the first — and a data symbol's
        // *address* is not that. Rather than thread a second operand form through
        // two shapes this rung has no captures for, the object receiver refuses
        // there by name, so what it costs is a census row instead of an argument.
        return Err(prod_tag("tail-object-receiver-is-not-a-tail-call"));
    } else if seg.get(p) == Some(&0x26) {
        // …or a **second member call** stands where the result would be consumed,
        // and the two results are compared: `return a->m() == b->n();`. The first
        // call's result is then live across the second `bl`, which makes the body
        // **Class B** — see [`super::mcall_cmp`]. Tried before the literal post-op
        // because a `26` cannot open one.
        return super::mcall_cmp::try_parse_member_cmp_calls(
            seg, p, lo, depth, &args, callee_tok, recv_tok, ret_at,
        );
    } else {
        // …or the call's result is consumed by a literal `± k` and *then* returned,
        // which makes the body a **framed non-leaf call** rather than a tail one.
        // See [`framed_member_call`].
        return framed_member_call(seg, p, lo, depth, args, callee_tok, recv_tok);
    }

    // From here the body is a member call that parses to the end of the segment, so
    // every remaining refusal is a **codegen-class** one over a complete body and is
    // reported under its own key.
    if args.len() > MAX_REGISTER_FORMALS {
        return Err(Some(Block::refuse(seg, p, "mcall-args-overflow")));
    }
    let params = parse_params(seg, lo).map_err(Some)?;
    tail_call_shape(seg, args, params, callee_tok, p, ArgSite::Tail).map_err(Some)
}

/// **W41 — `return p->m() ± k;`**: the member call whose result is consumed by a
/// literal add, which makes the body a **framed non-leaf call** and not a tail one.
///
/// ```text
///   … 4C            the member call's apply
///   33 <T> k 02|03  the post-op literal        — [`eat_call_postop`]
///   41 <T> …        the result is returned
/// ```
///
/// The emission is [`BodyShape::FramedCall`], **which needs no codegen at all**:
/// `this` is argument zero exactly as it is for the tail form, so the argument
/// setup is the same `select_text` register move and the rest is the shipped
/// 0x24-byte frame. MEASURED, every word read off the reference obj
/// (`work/w41/probe/p1.cpp`, `p5.cpp` at `/O1 /GS- /c`):
///
/// ```text
///   int f(A* p)            { return p->gi() - 20; }   bl ; addi r3,r3,-20
///   int f(int k, A* p)     { return p->gi() - 20; }   mr r3,r4 ; bl ; addi r3,r3,-20
///   int f(int j,int k,A*p) { return p->gi() - 20; }   mr r3,r5 ; bl ; addi r3,r3,-20
///   E*  f(A* p)            { return p->ge() - 1; }    bl ; addi r3,r3,-20   (sizeof E)
///   int f(A* p)            { return p->gi() + 0; }    b ?gi        — the identity FOLD
///   int f(A* p)            { return p->gi() - 40000; } addis + addi         — REFUSED
/// ```
///
/// **This is where the row was**, and it is not where its name said. The whole of
/// `expr-call-in-expr-recv-load-whole` — 10,494 functions, the residue W36 left —
/// decomposes over the 878-TU workload into exactly three shapes, measured by a
/// member-call *production* first-blocker histogram (`docs/rungs/`'s W41 §
/// reproduction): 6,463 with a value live across the call (**Class B**, a frame
/// class this port does not have), **3,559 here**, 440 whose receiver carries a
/// pointer conversion, and 32 residue. Not one function of it is the "member call
/// preceded by assignment statements" the row was scheduled as.
///
/// Every one of the 3,559 is a **subtraction**, and the free-function twin
/// `return g(a) - k;` is **0** functions — which is why the `03` byte had never
/// been asked for at the one locator that decodes this region.
fn framed_member_call(
    seg: &[u8],
    at: usize,
    lo: usize,
    depth: usize,
    args: Vec<Vec<IlOp>>,
    callee_tok: u32,
    recv_tok: u32,
) -> Result<BodyShape, Option<Block>> {
    let mut p = at;
    // Not this production: the cursor is untouched and the body keeps its own
    // census key, exactly as the non-committal contract requires.
    let k = eat_call_postop(seg, &mut p)
        .map_err(|_| prod_tag("framed-result-not-consumed-by-a-literal-post-op"))?;
    eat_return_plumbing(seg, &mut p, true, depth)
        .map_err(|_| prod_tag("framed-post-op-body-does-not-end-at-the-return"))?;

    // From here the body parses to the end of the segment, so every refusal is a
    // codegen-class one over a complete body and is reported under its own key.
    //
    // **Only the receiver.** [`BodyShape::FramedCall`] carries a single operand
    // stream, so it can spell "put this formal in r3" and nothing else; a member
    // call with explicit arguments would need the permutation the *tail* form
    // gets from [`tail_call_shape`], and c2 does emit one under a frame
    // (`int f(S* p,int a,int b){ return p->ga(b) - 20; }` is `mr r4,r5 ; bl ;
    // addi`), so this is a real limit and not a restatement of one.
    if args.len() != 1 {
        return Err(Some(Block::refuse(seg, p, "mcall-framed-args")));
    }
    let params = parse_params(seg, lo).map_err(Some)?;
    // A net post-op of **0** is not a framed call at all: `p->m() + 0` == `p->m()`
    // and the optimizer folds it to the bare tail branch. MEASURED — `zero_add`,
    // `zero_sub` and `plain` in `work/w41/probe/p5.cpp` are the same 4-byte
    // `b ?g@S@@QAAHXZ`, so emitting a frame here would be wrong bytes rather than
    // a gap. Routed to the tail production, which is the same decision
    // [`super::calls::parse_call_shape`] makes for the free-function form.
    if k == 0 {
        return tail_call_shape(seg, args, params, callee_tok, p, ArgSite::Tail).map_err(Some);
    }
    let arg_ops = vec![IlOp::Load(recv_tok)];
    // The receiver has to be one of this function's own formals: the framed path
    // emits a register *move*, and a global or a local would be a load.
    if !arg_loads_are_formals(&arg_ops, &params) {
        return Err(Some(Block::refuse(seg, p, "call-arg-nonformal")));
    }
    // Past the eighth formal a parameter is stack-homed and its setup is
    // `lwz r3,<slot>(r1)`, not a register move. The refusal is on the whole formals
    // LIST rather than on the receiver's index, because that is the predicate
    // `select_text` actually raises — the same reasoning, and the same key, as the
    // free-function framed call beside it.
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Some(Block::refuse(seg, p, "framed-arg-over-eight-formals")));
    }
    Ok(BodyShape::FramedCall { add_k: k, callee_tok, params, arg_ops })
}

/// `B9 <tok> <TYPE ptr4> [ 2C <TYPE ptr4> 00 ] · 99 <TYPE ptr4> 00` — the receiver
/// value and its bind as argument zero. Returns the receiver's token.
///
/// The receiver's TYPE goes through [`eat_operand_type`] rather than a local
/// tag/kind test, so this position inherits the **`volatile` gate** with it:
/// `GAPS.md` §6's thirteenth live mis-emit was a `volatile` formal read that c2
/// homes in the frame, and it was pre-existing across seven shapes because each had
/// asked the question itself. One locator.
pub(crate) fn eat_receiver_this(seg: &[u8], p: &mut usize) -> Result<u32, Block> {
    // **The base-adjusted receiver (`p->Base::m()`, intrinsic 2113) is NOT here,
    // and that is a measurement rather than an omission** — board #127, W-ADJUST.
    // Its completion counterfactual was run at this exact site and is **472
    // emitted functions of the 8,790 the row carries (5.4 %)**, 434 of them at
    // adjust offset 0. See `docs/rungs/2026-08-01-w-adjust.md`; do not re-derive
    // the row's worth from its census size.
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, "mcall-recv"));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "mcall-recv-tok"))?;
    *p += w;
    // A pointer, positively — not merely "some 4-byte operand". An `int` receiver
    // is not a receiver, and the class is what the `99` binds.
    match eat_operand_type(seg, p) {
        Some(ValueClass::Ptr4) => {}
        _ => return Err(blk(seg, *p, "mcall-recv-type")),
    }
    // An optional **pointer→pointer conversion** on the receiver, `2C <TYPE> 00`.
    // It emits nothing — the value stays in the register it is already in — and
    // the class is required to be preserved through the shared [`eat_value_type`]
    // locator, so a conversion that changes what the value *is* still refuses.
    //
    // Which C++ spellings produce it was measured rather than guessed
    // (`work/w41/probe/p2.cpp`, `p4.cpp`): a C-style or `static_cast` to the
    // receiver's own type is folded away and emits no `2C` at all; `const`
    // qualification on either the pointer, the pointee or the method emits none
    // either; and a base-class adjustment is `intrinsic 2113`, a different
    // production with a different lowering. What is left, and what this admits,
    // is a cast that genuinely changes the pointee type without changing the
    // address — `const_cast<S*>(p)->m()` and `((S*)v)->m()` from a `void*`.
    // Both are `b ?m@S@@QAAXXZ`, identical to the uncast form.
    //
    // Worth **440** functions of `expr-call-in-expr-recv-load-whole` on the
    // 878-TU workload, every one of which the production already accepted in full
    // apart from this one token.
    if seg.get(*p) == Some(&0x2C) {
        let mut q = *p + 1;
        if eat_value_type(seg, &mut q, ValueClass::Ptr4) && eat_byte(seg, &mut q, 0x00) {
            *p = q;
        } else {
            return Err(blk(seg, *p, "mcall-recv-convert"));
        }
    }
    eat_this_bind(seg, p)?;
    Ok(tok)
}

/// `99 <TYPE ptr4> 00` — **bind the value on top of the operand stack as argument
/// zero**, the `this` bind alone.
///
/// Split out of [`eat_receiver_this`] byte for byte, every refusal key unchanged,
/// because a **chain link** is this bind with no receiver designator in front of
/// it: `p->a()->b()` binds the first call's *result*
/// ([`super::mcall_chain::try_parse_member_chain_call`]). A second copy of the
/// three gates below is the drift `docs/GAPS.md` §6 instance #9 records — and the
/// `mcall-bind-offset` key in particular exists so that what the literal `00`
/// requirement costs is a number rather than an argument, which a private copy
/// would silently re-decide.
pub(crate) fn eat_this_bind(seg: &[u8], p: &mut usize) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x99) {
        return Err(blk(seg, *p, "mcall-bind"));
    }
    // The bound value's own TYPE — a pointer to the class the method belongs to.
    // Required to be a width-4 pointer for the same reason the receiver is: this is
    // the token that says the call is a *direct* member dispatch on an ordinary
    // object pointer (virtual dispatch is `67`/`9A`, a different opcode pair).
    //
    // For a chain link it carries a second fact for free: the *previous* call's
    // result is a class pointer. A link whose bound value is an `int` or a
    // `float` cannot exist, so the intermediate calls need no return-type gate of
    // their own.
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_ptr4_kind(tag, kind) => *p += w,
        _ => return Err(blk(seg, *p, "mcall-bind-type")),
    }
    // The trailing field. UNKNOWN (`docs/IL_EXPR_LAYER.md` §7) and `00` at every
    // observed site, including a member function of a class with a base — so it is
    // required literally, and what that costs is a census key rather than an
    // argument.
    let save = *p;
    match read_varint(seg, p) {
        Some(0) => {}
        Some(_) => {
            *p = save;
            return Err(Block::refuse(seg, save, "mcall-bind-offset"));
        }
        None => return Err(blk(seg, *p, "mcall-bind-tail")),
    }
    Ok(())
}

/// **The tag-coverage property of this family, as an executable check.**
///
/// `prod-entered-untagged` is the residue the production axis prints on every
/// scan: a body that entered a member-call production, declined
/// non-committally, and reached **no** tagged bail. The workload measures it
/// over real IL; this measures it over *adversarial* IL, which is the half a
/// scan cannot reach — a byte the captures happen never to spell is exactly the
/// site a future edit re-opens, and an untagged site does not fail anything, it
/// simply makes one report row read as an absence.
///
/// The sweep mutates one byte at a time to each of the vocabulary bytes this
/// family dispatches on, plus one (`FF`) that no production spells, and asserts
/// the residue is never observed. Two properties are separated deliberately:
///
/// * the residue assertion is **inside** the loop, so it fires on the first
///   witness and names it; and
/// * the "did the sweep reach the productions at all" floor is checked
///   **after**, so it can never pre-empt the assertion that matters. Reverting
///   a tag site to a bare `None` leaves `entered` completely unchanged — the
///   same bodies enter the same production — so only the in-loop assertion can
///   fire, which is what makes a failure evidence about the tag and not about
///   the corpus.
#[cfg(test)]
pub(super) fn assert_no_decline_lands_in_the_residue(
    corpus: &[(&str, &[u8])],
    min_entered: usize,
) {
    use crate::func::body::{
        parse_segment_detail, prod_site, PROD_ENTERED_UNTAGGED, PROD_NOT_ENTERED,
    };
    use crate::func::test_fixtures::NO_LOCALS;

    // The bytes this family branches on — the method push, the receiver load, the
    // `this` bind, the designator openers, the statement end, the result
    // annotation — plus `00` and `FF`, which stand for "a field went unreadable".
    const ALPHABET: [u8; 10] =
        [0x00, 0x26, 0x2C, 0x30, 0x33, 0x41, 0x4B, 0x4C, 0x99, 0xFF];
    assert!(!corpus.is_empty(), "the sweep needs at least one captured body");
    let mut entered = 0usize;
    for (name, body) in corpus {
        for i in 0..body.len() {
            for v in ALPHABET {
                if body[i] == v {
                    continue;
                }
                let mut seg = body.to_vec();
                seg[i] = v;
                let accepted = parse_segment_detail(&seg, NO_LOCALS).is_ok();
                let site = prod_site();
                // A mutation that breaks the dispatch never offers the body to a
                // production, and `prod-not-entered` is that fact stated — not a
                // hole, and not this check's business.
                if site == PROD_NOT_ENTERED {
                    continue;
                }
                entered += 1;
                assert_ne!(
                    site, PROD_ENTERED_UNTAGGED,
                    "{name}[{i}] := {v:#04x} (accepted={accepted}) entered a \
                     member-call production, declined, and reached NO tagged bail. \
                     The gap report would print it as `prod-entered-untagged` — a \
                     population rendered as an absence, which is the one failure \
                     this axis exists to close"
                );
            }
        }
    }
    assert!(
        entered >= min_entered,
        "the sweep must actually REACH the productions: only {entered} mutations \
         entered one, under the floor of {min_entered}. A loop that never enters \
         a production observes no residue either, and would report success by \
         doing nothing"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail, BodyShape, SeqTail};
    use crate::func::test_fixtures::*;

    /// Every refusal context [`eat_receiver_this`] can raise, verbatim. A test
    /// that transcribed a *subset* would pass while a whole context fell into
    /// `ctx-unknown`, which is the shape of failure §9.14.6 records for a
    /// witness list: the class that is wrong is the one with no witness.
    const RECV_CTXS: [&str; 8] = [
        "mcall-recv",
        "mcall-recv-tok",
        "mcall-recv-type",
        "mcall-recv-convert",
        "mcall-bind",
        "mcall-bind-type",
        "mcall-bind-offset",
        "mcall-bind-tail",
    ];

    fn recv_block(ctx: &'static str, off: usize, seg_len: usize) -> Block {
        Block { ctx, byte: None, off, seg_len, aux: 0 }
    }

    /// **Totality, enumerated over the whole domain** — every context × every one
    /// of the 256 byte values × EOF gets a name, and the name is never the
    /// `ctx-unknown` residue.
    ///
    /// The domain is enumerated rather than sampled for §9.14.6's reason, and
    /// the residue is asserted *inside* the loop so a failure names its witness
    /// instead of reporting a count.
    #[test]
    fn every_receiver_refusal_has_a_name() {
        for arm in [RecvArm::Tail, RecvArm::Chain, RecvArm::CmpSecond] {
            let v = match arm {
                RecvArm::Tail => &RECV_TAIL,
                RecvArm::Chain => &RECV_CHAIN,
                RecvArm::CmpSecond => &RECV_CMP,
            };
            for ctx in RECV_CTXS {
                // EOF: the refusal offset is past the end of the segment.
                let name = v.tag(&[], &recv_block(ctx, 0, 0));
                assert!(
                    !name.ends_with("/ctx-unknown"),
                    "{ctx} at EOF fell into the unnamed residue"
                );
                for b in 0u8..=0xFF {
                    let seg = [b];
                    let name = v.tag(&seg, &recv_block(ctx, 0, 1));
                    assert!(
                        !name.ends_with("/ctx-unknown"),
                        "{ctx} with byte {b:#04x} fell into the unnamed residue — a \
                         population rendered as an absence is the one failure this \
                         axis exists to close"
                    );
                    assert!(
                        name.starts_with(match arm {
                            RecvArm::Tail => "tail-recv-not-a-plain-b9-load/",
                            RecvArm::Chain => "chain-recv-not-a-plain-b9-load/",
                            RecvArm::CmpSecond => "cmp-second-recv-not-a-plain-b9-load/",
                        }),
                        "{name} must keep the published site name as its prefix, so a \
                         figure keyed on the old string is recovered by a prefix test"
                    );
                }
            }
        }
    }

    /// **Injectivity** — two constructs never share a name, so no two rows of the
    /// decomposition can be summed into a double count. §9.14.4 checks the
    /// completeness vocabulary the same way and for the same reason.
    ///
    /// Checked *within* a position and *across* the two positions: `no-b9-off-add`
    /// and `then-off-add` are different facts about different bytes, and a
    /// vocabulary that collapsed them would report a receiver that is an offset
    /// add and one that is followed by an offset add as one arm.
    #[test]
    fn the_receiver_vocabulary_is_injective() {
        let mut seen: std::collections::BTreeMap<&'static str, (usize, u8)> = Default::default();
        for (i, ctx) in RECV_CTXS.iter().enumerate() {
            for b in 0u8..=0xFF {
                let seg = [b];
                let name = RECV_TAIL.tag(&seg, &recv_block(ctx, 0, 1));
                // Two different bytes MAY share a construct name (`27`/`28` are
                // both `off-add`, `67`/`9A` are both `virtual`) — that is the
                // vocabulary naming a construct rather than a byte. What may not
                // collide is two different *positions*.
                if let Some(&(j, ob)) = seen.get(name) {
                    let same_position = matches!(
                        (RECV_CTXS[j], *ctx),
                        ("mcall-recv", "mcall-recv") | ("mcall-bind", "mcall-bind")
                    ) || j == i;
                    assert!(
                        same_position,
                        "{name} is produced by two different positions \
                         ({} byte {ob:#04x} and {ctx} byte {b:#04x})",
                        RECV_CTXS[j]
                    );
                } else {
                    seen.insert(name, (i, b));
                }
            }
        }
        // The two byte positions must not share a single name.
        let designator: std::collections::BTreeSet<&str> = (0u8..=0xFF)
            .map(|b| RECV_TAIL.tag(&[b], &recv_block("mcall-recv", 0, 1)))
            .collect();
        let bind: std::collections::BTreeSet<&str> = (0u8..=0xFF)
            .map(|b| RECV_TAIL.tag(&[b], &recv_block("mcall-bind", 0, 1)))
            .collect();
        assert!(
            designator.is_disjoint(&bind),
            "the designator and bind positions share a name; the decomposition \
             would sum two different facts"
        );
    }

    /// **The arity check.** Totality residue 0 is not a control (#144): a table
    /// that named every byte identically would pass it. This varies the thing the
    /// classifier actually *branches* on beyond the opcode — the intrinsic
    /// SELECTOR — and asserts the seven class-layout ids separate.
    ///
    /// 2113 `this-adjust` is board #127/#140, measured at 472 emitted functions,
    /// and 2117 `base-member-addr` is a designator with a different lowering
    /// entirely. One bucket over both re-creates exactly the conflation §9.13
    /// spent a lane undoing, and the opcode byte alone cannot tell them apart.
    #[test]
    fn the_intrinsic_receiver_arm_separates_by_selector() {
        // `33 86 41 74 <varint id> 40` — the intrinsic head `intrinsic_selector`
        // reads. Every id in this family needs the varint ESCAPE (`80` + LE32),
        // which is the encoding the two captured selectors in the tree carry
        // verbatim: `80 41 08 00 00` is 2113 (`ctor_dtor::SELECTOR_2113`) and
        // `80 45 08 00 00` is 2117 (`designator::SELECTOR_2117`).
        let head = |id: i32| -> Vec<u8> {
            let mut v = vec![0x33, 0x86, 0x41, 0x74, 0x80];
            v.extend_from_slice(&id.to_le_bytes());
            v.push(0x40);
            v
        };
        assert_eq!(
            &head(2113)[4..9],
            &[0x80, 0x41, 0x08, 0x00, 0x00],
            "the test's own encoder must reproduce the captured selector bytes"
        );
        let names: Vec<&str> = (2113..=2119)
            .map(|id| RECV_TAIL.tag(&head(id), &recv_block("mcall-recv", 0, 16)))
            .collect();
        let uniq: std::collections::BTreeSet<&&str> = names.iter().collect();
        assert_eq!(
            uniq.len(),
            7,
            "the seven class-layout selectors must separate, got {names:?}"
        );
        assert!(names[0].ends_with("/no-b9-this-adjust"), "2113 is `this-adjust`: {}", names[0]);
        assert!(
            names[4].ends_with("/no-b9-base-member-addr"),
            "2117 is `base-member-addr`: {}",
            names[4]
        );
        // An id outside the family is named as such, not folded into one of the
        // seven — and a `33` that is NOT an intrinsic head keeps the hex bucket
        // rather than a name it has not earned.
        assert!(RECV_TAIL
            .tag(&head(173), &recv_block("mcall-recv", 0, 16))
            .ends_with("/no-b9-intrinsic-other"));
    }

    /// **The second arity axis: the same opcode, two constructs, decided by what
    /// stands one token later.**
    ///
    /// `33 <int-like> <k>` is a literal push. Behind a `27`/`28` the run is a
    /// *byte-offset add on the receiver designator* (`p->f.m()`); with anything
    /// else behind it, it is a bare literal. The first version of this table saw
    /// only the opcode and filed **5,806 emitted functions** under `op-0x33` —
    /// the byte the run stopped in front of, which is §9.14.7's defect and the
    /// one #139 exists to cure. An opcode-only test would have passed on it.
    ///
    /// The witness is transcribed from the workload dump
    /// (`src/lazer/game/HamUser.cpp#723`), so the encoding is a capture and not
    /// an assumption: `b9 <tok> a6 43 d5 37 · 33 86 41 74 00 · 27 a6 43 d0 34 ·
    /// 99 …`.
    #[test]
    fn a_literal_behind_an_offset_add_is_named_for_the_add_not_the_byte() {
        // The captured run, from the `33` on.
        let off_add: &[u8] = &[0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0xA6, 0x43, 0xD0, 0x34];
        assert!(
            RECV_TAIL.tag(off_add, &recv_block("mcall-bind", 0, off_add.len()))
                .ends_with("/then-off-add"),
            "got {}",
            RECV_TAIL.tag(off_add, &recv_block("mcall-bind", 0, off_add.len()))
        );
        // The `28` form of the same construct.
        let off_add28: &[u8] = &[0x33, 0x86, 0x41, 0x74, 0x08, 0x28, 0x00, 0x00];
        assert!(RECV_TAIL
            .tag(off_add28, &recv_block("mcall-bind", 0, off_add28.len()))
            .ends_with("/then-off-add"));
        // …and the same opcode with something else behind it stays a literal.
        let bare: &[u8] = &[0x33, 0x86, 0x41, 0x74, 0x13, 0x0F, 0x86, 0x41, 0x74, 0x4B];
        assert!(
            RECV_TAIL.tag(bare, &recv_block("mcall-recv", 0, bare.len()))
                .ends_with("/no-b9-literal"),
            "got {}",
            RECV_TAIL.tag(bare, &recv_block("mcall-recv", 0, bare.len()))
        );
        // The literal's own width is an axis too: a `80`-escaped offset must not
        // fall out of the construct because its varint is five bytes, not one.
        let wide: &[u8] =
            &[0x33, 0x86, 0x41, 0x74, 0x80, 0x40, 0x01, 0x00, 0x00, 0x27, 0xA6, 0x43, 0xD0, 0x34];
        assert!(
            RECV_TAIL.tag(wide, &recv_block("mcall-bind", 0, wide.len()))
                .ends_with("/then-off-add"),
            "got {}",
            RECV_TAIL.tag(wide, &recv_block("mcall-bind", 0, wide.len()))
        );
    }

    /// The indirect **store** is a construct and must be named as one: the body
    /// dispatch offers every statement-head `26` to this production, so a `32`
    /// standing where the `this` bind belongs says the statement is an
    /// ASSIGNMENT and there is no receiver in it at all. Filed as `op-0x32` that
    /// fact is invisible, and it is 1,100 emitted functions of the site.
    #[test]
    fn an_indirect_store_at_the_bind_position_is_named_a_store() {
        // `?mash@@YAXPAE0@Z`, src/keygen_xbox.cpp#4, from the workload dump.
        let st: &[u8] = &[0x32, 0x86, 0x43, 0xF5, 0x08, 0x4B];
        assert!(RECV_TAIL.tag(st, &recv_block("mcall-bind", 0, st.len())).ends_with("/then-store"));
    }

    /// `void mv_one(Obj *o) { o->set1(); }` — the minimal member call: the receiver
    /// is the only formal, so it is already in r3 and the whole body is `b <set1>`.
    ///
    /// Transcribed verbatim from a live-toolchain capture of
    /// `fixtures/cpp/w36_member_call.cpp` (`c2rs census … --keep-il`), not
    /// hand-assembled — the point of the production is where the receiver sits
    /// relative to the CALL token, and only a capture settles that.
    const MC_NULLARY: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xED, 0x09,
        0x46, 0x2D, 0xEC, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0xB9, 0xEC, 0x09, 0x86,
        0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80,
        0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xEE, 0x09, 0x54, 0x02, 0x29, 0xEE, 0x09, 0x4F,
        0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];

    /// `void mv_swap(Obj *o, int a, int b) { o->v2(b, a); }` — the case where
    /// putting `this` at the FRONT of the argument list instead of the end emits a
    /// different permutation. Verbatim capture, same TU discipline as above.
    const MC_SWAP: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xF1, 0x09,
        0x46, 0x2D, 0xF0, 0x09, 0x2D, 0xEF, 0x09, 0x2D, 0xEE, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26,
        0xE7, 0x09, 0xB9, 0xEE, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0xB9, 0xEF, 0x09, 0x86, 0x41,
        0x74, 0x55, 0x86, 0x41, 0x74, 0xB9, 0xF0, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74,
        0x4C, 0x4B, 0x3A, 0xF2, 0x09, 0x54, 0x02, 0x29, 0xF2, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];

    /// W41: `int fsub(S* p) { return p->g() - 20; }` — the member call whose
    /// result a literal is subtracted from, which makes the body a **framed**
    /// non-leaf call. The post-op is `… 4C · 33 86 41 74 14 · 03 · 41 …` — the
    /// literal 20, then the SUB byte, and the emission is
    /// `bl ?g@S@@QAAHXZ ; 3863ffec addi r3,r3,-20`.
    ///
    /// Transcribed verbatim from a live-toolchain capture
    /// (`c2rs census … --keep-il`), same TU discipline as the two above.
    const MC_FRAMED_SUB: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xED, 0x09,
        0x46, 0x2D, 0xEC, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xB9, 0xEC, 0x09, 0x86,
        0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80,
        0x04, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41, 0x74, 0x14, 0x03, 0x41, 0x86, 0x41, 0x74,
        0x3A, 0xEE, 0x09, 0x54, 0x02, 0x29, 0xEE, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// W41: `void fcast(void* v) { ((S*)v)->v(); }` — the receiver carries a
    /// **pointer conversion** between its LOAD and the `99` bind:
    /// `B9 <v> 86 43 83 08 · 2C 86 43 81 20 00 · 99 …`, `void*` to `S*`. It emits
    /// nothing (the address is the same and the register does not move) and the
    /// body is the bare `b ?v@S@@QAAXXZ`.
    ///
    /// Which spellings produce this was measured, not guessed
    /// (`work/w41/probe/p2.cpp`, `p4.cpp`): a C-style or `static_cast` to the
    /// receiver's *own* type emits no `2C` at all, cv-qualification emits none
    /// either, and a base-class adjustment is `intrinsic 2113`. What is left is a
    /// cast that changes the pointee without changing the address.
    const MC_RECV_CAST: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xF0, 0x09,
        0x46, 0x2D, 0xEF, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0xB9, 0xEF, 0x09, 0x86,
        0x43, 0x83, 0x08, 0x2C, 0x86, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xF1, 0x09,
        0x54, 0x02, 0x29, 0xF1, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **W-ADJUST**: `void n0() { gDbg.nul(); }` — the receiver is a NAMED DATA
    /// OBJECT, so its designator is `26 <sym> · 2C A6 43 81 20 00 · 99 …` where
    /// every other member call has `B9 <tok> <TYPE ptr4> …`. The `2C` is c2's own
    /// cv-strip on the object's address and is not optional in this spelling.
    ///
    /// Transcribed verbatim from a live-toolchain capture
    /// (`c2rs census work/wadjust/probe/p1.cpp --keep-il`), same TU discipline as
    /// the four above: the whole question is which token stands where the
    /// receiver goes, and only a capture settles that.
    const MC_RECV_OBJECT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xEC, 0x09,
        0x46, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0x26, 0xEB, 0x09, 0x2C, 0xA6, 0x43, 0x81,
        0x20, 0x00, 0x99, 0x86, 0x43, 0x83, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x03,
        0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xED, 0x09, 0x54, 0x02, 0x29, 0xED, 0x09, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x04, 0x4D,
    ];

    /// **The body the object receiver must NOT steal**: `void ch(B* b)
    /// { b->a()->m(); }` — a two-link chain, whose head is `26 <m> 26 <a> B9 <b>`
    /// and therefore byte-identical to an object receiver for two whole tokens.
    /// Verbatim capture (`work/wadjust/probe/p7.cpp`).
    const MC_CHAIN: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xF4, 0x09,
        0x46, 0x2D, 0xF3, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0x26, 0xEC, 0x09, 0xB9,
        0xF3, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x86, 0x43,
        0x83, 0x20, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x8A, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xF5, 0x09,
        0x54, 0x02, 0x29, 0xF5, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20,
        0x00, 0x4F, 0x01, 0x04, 0x4D,
    ];

    /// **W-ADJUST — the named object's address reaches codegen as `SymAddr`, not
    /// as a `Load`.**
    ///
    /// The distinction is the whole rung. A `Load` of this token would be a `mr`
    /// out of a register that holds nothing — wrong bytes rather than a refusal —
    /// and the slot list is the only place the two can still be told apart, since
    /// downstream every argument is "the thing in slot i".
    #[test]
    fn a_named_object_receiver_enters_slot_zero_as_an_address() {
        assert_eq!(
            parse_segment(MC_RECV_OBJECT, NO_LOCALS),
            Some(BodyShape::MultiArgTailCall {
                params: vec![],
                arg_sources: vec![crate::func::body::SlotArg::SymAddr(60169)],
                callee_tok: 58377,
            })
        );
    }

    /// **…and the same two leading bytes on a CHAIN still reach the chain
    /// production.** `26 <m> 26 <x>` opens both, and the discriminator is the
    /// token *after* the second symbol — `B9` for a chain's innermost receiver,
    /// `2C`/`99` for an object's address. Stated in both directions, because a
    /// discriminator that only ever saw one side is the shape `GAPS.md` §6
    /// records: the accept is above, and this is the decline.
    #[test]
    fn the_object_receiver_declines_a_chain_without_consuming_it() {
        // The chain body still parses as the chain production's own shape.
        assert!(matches!(
            parse_segment(MC_CHAIN, NO_LOCALS),
            Some(BodyShape::CallSeq { .. })
        ));
        // And the locator itself declines with the cursor exactly where it was —
        // the non-committal contract the caller depends on to hand the same `26`
        // on. The offsets are found by search so a capture edit cannot silently
        // point them at the wrong byte.
        let at = |seg: &[u8]| {
            seg.windows(3)
                .position(|w| w[0] == 0x4C && w[1] == 0x4F && w[2] == 0x11)
                .expect("the LO marker")
                + 3
        };
        // …past `53` and the callee push, to the second `26`.
        let second_26 = |seg: &[u8]| {
            let mut p = at(seg) + 1;
            assert_eq!(seg[p], 0x26);
            p += 1;
            p += crate::func::readers::read_token_var(seg, p).expect("callee token").1;
            p
        };
        let p0 = second_26(MC_CHAIN);
        let mut p = p0;
        assert_eq!(
            eat_receiver_object(MC_CHAIN, &mut p),
            None,
            "a chain's second method push is not a named object's address"
        );
        assert_eq!(p, p0, "a declining locator must not move the cursor");
        // The positive side, at the same position in the object-receiver body.
        let q0 = second_26(MC_RECV_OBJECT);
        let mut q = q0;
        assert_eq!(eat_receiver_object(MC_RECV_OBJECT, &mut q), Some(60169));
        assert!(q > q0, "the accepting locator must consume the designator");
    }

    /// **Every non-committal decline out of the tail production is attributed to
    /// a named site**, under adversarial mutation of the captured bodies —
    /// so no row of the gap report can read `prod-entered-untagged`, which is the
    /// residue rendered as an absence.
    #[test]
    fn no_decline_out_of_the_tail_production_lands_in_the_residue() {
        super::assert_no_decline_lands_in_the_residue(
            &[
                ("MC_NULLARY", MC_NULLARY),
                ("MC_SWAP", MC_SWAP),
                ("MC_FRAMED_SUB", MC_FRAMED_SUB),
                ("MC_RECV_CAST", MC_RECV_CAST),
                ("MC_RECV_OBJECT", MC_RECV_OBJECT),
                ("MC_CHAIN", MC_CHAIN),
            ],
            // MEASURED: 4,730 of the mutations reach the production. The floor is
            // set just under it so a corpus edit that guts the sweep is caught,
            // while a tag edit — which cannot move this number at all — is graded
            // only by the in-loop assertion.
            4_500,
        );
    }

    #[test]
    fn a_member_call_with_a_literal_postop_is_a_framed_call() {
        // `this` is the only argument, so the setup is empty and the body is the
        // shipped 0x24-byte frame with a NEGATIVE immediate. The whole point of
        // the rung: no new codegen, only `add_k`'s sign.
        assert_eq!(
            parse_segment(MC_FRAMED_SUB, NO_LOCALS),
            Some(BodyShape::FramedCall {
                add_k: -20,
                callee_tok: 58377,
                params: vec![60425],
                arg_ops: vec![IlOp::Load(60425)],
            })
        );
    }

    #[test]
    fn a_postop_of_zero_folds_to_the_tail_call_rather_than_a_frame() {
        // `p->m() + 0` == `p->m()`, and c2 emits the bare `b <method>` for both —
        // MEASURED, `zero_add`/`zero_sub`/`plain` in `work/w41/probe/p5.cpp` are
        // the same four bytes. Emitting a frame here would be wrong bytes, not a
        // gap, so the routing is pinned in both directions.
        let at = MC_FRAMED_SUB
            .windows(6)
            .position(|w| w[0] == 0x33 && w[4] == 0x14 && w[5] == 0x03)
            .expect("the post-op literal");
        for (k, op) in [(0x00u8, 0x03u8), (0x00, 0x02)] {
            let mut seg = MC_FRAMED_SUB.to_vec();
            seg[at + 4] = k;
            seg[at + 5] = op;
            assert!(
                matches!(
                    parse_segment(&seg, NO_LOCALS),
                    Some(BodyShape::IntTailCall { .. })
                ),
                "a net-zero post-op must fold to the tail call, not FramedCall{{add_k:0}}"
            );
        }
        // …and a MULTIPLY is the neighbour SUB was wrongly grouped with: it
        // strength-reduces to a shift/add sequence and is not one `addi`.
        let mut seg = MC_FRAMED_SUB.to_vec();
        seg[at + 5] = 0x04;
        assert!(parse_segment(&seg, NO_LOCALS).is_none(), "a `* k` post-op must refuse");
    }

    #[test]
    fn the_receiver_may_carry_a_pointer_conversion_and_it_emits_nothing() {
        // The receiver is the sole argument, so the shape is the one-operand
        // tail call the W36 nullary case produces — the conversion contributes
        // nothing to it, which is the claim.
        assert_eq!(
            parse_segment(MC_RECV_CAST, NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![61193],
                arg_ops: vec![IlOp::Load(61193)],
                callee_tok: 58633,
            })
        );
        // The conversion's target class is required, not skipped: retyping it to
        // `int` (`86 41 74`) makes it a value change rather than an address
        // reinterpretation, and the body refuses through the shared
        // `eat_value_type` locator.
        let at = MC_RECV_CAST
            .windows(5)
            .position(|w| w[0] == 0x2C && w[1] == 0x86 && w[2] == 0x43 && w[4] == 0x20)
            .expect("the receiver conversion");
        let mut seg = MC_RECV_CAST.to_vec();
        seg.splice(at + 1..at + 5, [0x86, 0x41, 0x74]);
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
    }

    /// The offset of the `99` bind's trailing field in a captured segment: the
    /// `99`, its 4-byte TYPE, then the one-byte varint, with the `BD` after it.
    fn bind_tail(seg: &[u8]) -> usize {
        seg.windows(7)
            .position(|w| w[0] == 0x99 && w[1] == 0x86 && w[2] == 0x43 && w[6] == 0xBD)
            .expect("the 99 bind")
            + 5
    }

    /// The offset of the receiver LOAD's TYPE: `B9`, a 2-byte token, then the TYPE.
    fn recv_type(seg: &[u8]) -> usize {
        seg.windows(7)
            .position(|w| w[0] == 0xB9 && w[3] == 0x86 && w[4] == 0x43 && w[6] == 0x20)
            .expect("the receiver load")
            + 3
    }

    #[test]
    fn a_nullary_member_call_is_a_tail_call_whose_argument_is_the_receiver() {
        // One formal, already in r3, so the argument setup is empty and the emission
        // is the bare `b <method>` — the same `IntTailCall` the free-function
        // statement call `void f(int a){ g(a); }` produces.
        assert_eq!(
            parse_segment(MC_NULLARY, NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![60425],
                arg_ops: vec![IlOp::Load(60425)],
                callee_tok: 58633,
            })
        );
    }

    #[test]
    fn member_tail_call_puts_this_in_slot_zero() {
        // `o->v2(b, a)` is `v2(o, b, a)`: slot 0 is the receiver, slot 1 is `b`
        // (formal 2) and slot 2 is `a` (formal 1) — a 2-cycle over r4/r5 with r3
        // already in place. The argument list is in STREAM order (rightmost source
        // argument first), so the receiver belongs on the END of it; pushing it on
        // the front would give `[1, 2, 0]`, a 3-cycle, and three wrong `mr`s.
        assert_eq!(
            parse_segment(MC_SWAP, NO_LOCALS),
            Some(BodyShape::MultiArgTailCall {
                params: vec![60937, 61193, 61449],
                arg_sources: vec![
                    crate::func::body::SlotArg::Formal(0),
                    crate::func::body::SlotArg::Formal(2),
                    crate::func::body::SlotArg::Formal(1),
                ],
                callee_tok: 59145,
            })
        );
    }

    #[test]
    fn the_bind_s_trailing_field_is_required_to_be_zero() {
        // UNKNOWN and `00` at every observed site (`IL_EXPR_LAYER.md` §7). A field
        // that never varied is indistinguishable from a constant, so it is required
        // rather than skipped and its exceptions get their own key.
        let at = bind_tail(MC_NULLARY);
        assert_eq!(MC_NULLARY[at], 0x00);
        let mut seg = MC_NULLARY.to_vec();
        seg[at] = 0x04;
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
    }

    #[test]
    fn a_non_pointer_receiver_declines() {
        // `int` where the pointer must be. The body still refuses, and it refuses
        // through the shared operand-type locator rather than a local tag test —
        // which is also how it inherits the `volatile` gate.
        let at = recv_type(MC_NULLARY);
        let mut seg = MC_NULLARY.to_vec();
        seg.splice(at..at + 4, [0x86, 0x41, 0x74]);
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
    }

    #[test]
    fn the_census_still_names_the_member_call_when_the_shape_declines() {
        // Decoding is not accepting, and a declining body must keep the
        // de-conflated `expr-call-in-expr-recv-*` key rather than falling back to
        // the opcode bucket `expr-op-0x99` this rung exists to empty. Declined here
        // by retyping an ARGUMENT to `float` (`86 45 76`), which the integer operand
        // vocabulary cannot spell — the same way the workload's 1,255
        // `recv-load-then-type-real-…` bodies decline.
        let at = MC_SWAP
            .windows(6)
            .position(|w| w[0] == 0xB9 && w[3] == 0x86 && w[4] == 0x41 && w[5] == 0x74)
            .expect("an argument load")
            + 3;
        let mut seg = MC_SWAP.to_vec();
        seg[at + 1] = 0x45;
        seg[at + 2] = 0x76;
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert!(
            b.feature().starts_with("expr-call-in-expr-recv-load"),
            "{}",
            b.feature()
        );
    }

    // -----------------------------------------------------------------------
    // W-MCALL — the member call in STATEMENT-SEQUENCE position (board #1960)
    // -----------------------------------------------------------------------
    //
    // Every segment below is transcribed verbatim from a live-toolchain capture
    // of the lane's own fixtures at the workload's flags
    // (`c2rs capture fixtures/cpp/wmcall_seq{,_neg}.cpp --keep-il`, split on the
    // `4F 1F` gate marker by `work/w-mcall/pin.py`), never hand-assembled: the
    // one fact a hand-written segment gets backwards is the argument list's
    // stream order, and that is exactly what these tests are about.

    /// `void wmcall_two(S* s){ s->a(); s->b(); }` — `wmcall_seq.cpp` cell P1.
    const MCS_TWO: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x37, 0x53, 0x53, 0x26, 0xF2, 0x09,
        0x46, 0x2D, 0xF1, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x38, 0x26, 0xE5, 0x09, 0xB9,
        0xF1, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07,
        0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x39, 0x26, 0xE6, 0x09,
        0xB9, 0xF1, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82,
        0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x3A, 0x3A, 0xF3,
        0x09, 0x54, 0x02, 0x29, 0xF3, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `void wmcall_two_recv(S* s, S* t){ s->a(); t->a(); }` — cell P4. Two
    /// formals, one callee-saved GPR, and the parked one is the SECOND.
    const MCS_TWO_RECV: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x4B, 0x53, 0x53, 0x26, 0xFD, 0x09,
        0x46, 0x2D, 0xFC, 0x09, 0x2D, 0xFB, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x4C, 0x26,
        0xE5, 0x09, 0xB9, 0xFB, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x4D,
        0x26, 0xE5, 0x09, 0xB9, 0xFC, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20,
        0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01,
        0x4E, 0x3A, 0xFE, 0x09, 0x54, 0x02, 0x29, 0xFE, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54,
        0x00,
    ];

    /// `void wmcall_free_then(S* s){ wmcall_free(); s->a(); }` — cell P6. The
    /// FREE call is first, so this body never enters
    /// [`try_parse_member_tail_call`] at all: the member call is read by the
    /// sequence LOOP. Its base census key was `call-token-0xB9`.
    const MCS_FREE_THEN: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x5A, 0x53, 0x53, 0x26, 0x03, 0x0A,
        0x46, 0x2D, 0x02, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x5B, 0x26, 0xF0, 0x09, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x09, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x5C, 0x26,
        0xE5, 0x09, 0xB9, 0x02, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x5D,
        0x3A, 0x04, 0x0A, 0x54, 0x02, 0x29, 0x04, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `void S::both(){ a(); b(); }` — cell P8, the workload's own spelling: the
    /// receiver is the IMPLICIT `this`, so the formals list is the `this` group
    /// alone and there is no `2D` parameter run at all.
    const MCS_THIS: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x6A, 0x53, 0x53, 0x26, 0xE9, 0x09,
        0xB9, 0x09, 0x0A, 0xA6, 0x43, 0x83, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0x46, 0x4C,
        0x4F, 0x11, 0x53, 0x4F, 0x01, 0x6B, 0x26, 0xE5, 0x09, 0xB9, 0x09, 0x0A, 0xA6, 0x43, 0x83,
        0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x6C, 0x26, 0xE6, 0x09, 0xB9, 0x09, 0x0A, 0xA6, 0x43,
        0x83, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04,
        0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x6D, 0x3A, 0x0A, 0x0A, 0x54, 0x02, 0x29, 0x0A,
        0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x6E,
        0x4D,
    ];

    /// `int wmcall_neg_value_tail(S* s){ s->a(); return s->get(); }` — the
    /// integer VALUE TAIL. It was w-mcall's decline **D3** and
    /// `wmcall_seq_neg.cpp` cell N6; **lane `w-fltret` paid it**, so this cell is
    /// a positive now and the fixture cell was re-taken.
    const MCS_VALUE_TAIL: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x61, 0x53, 0x53, 0x26, 0x15, 0x0A,
        0x46, 0x2D, 0x14, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x62, 0x26, 0xE4, 0x09, 0xB9,
        0x14, 0x0A, 0x86, 0x43, 0xA3, 0x20, 0x99, 0x86, 0x43, 0x83, 0x20, 0x00, 0xBD, 0x82, 0x07,
        0x03, 0x00, 0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x63, 0x26, 0xE6, 0x09,
        0xB9, 0x14, 0x0A, 0x86, 0x43, 0xA3, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86,
        0x41, 0x74, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x16,
        0x0A, 0x4F, 0x01, 0x64, 0x54, 0x02, 0x29, 0x16, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54,
        0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x65, 0x4D,
    ];

    /// `void wmcall_neg_guarded(S* s, int c){ if (c) { s->a(); } s->b(); }` —
    /// cell N4, the GUARDED sequence (W10) this rung excludes by name.
    const MCS_GUARDED: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x53, 0x53, 0x53, 0x26, 0x0E, 0x0A,
        0x46, 0x2D, 0x0D, 0x0A, 0x2D, 0x0C, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x54, 0x53,
        0xB9, 0x0D, 0x0A, 0x86, 0x41, 0x74, 0x38, 0x10, 0x0A, 0x53, 0x53, 0x4F, 0x01, 0x55, 0x26,
        0xE4, 0x09, 0xB9, 0x0C, 0x0A, 0x86, 0x43, 0xA3, 0x20, 0x99, 0x86, 0x43, 0x83, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x56,
        0x54, 0x05, 0x4F, 0x01, 0x57, 0x54, 0x04, 0x29, 0x10, 0x0A, 0x54, 0x03, 0x26, 0xE5, 0x09,
        0xB9, 0x0C, 0x0A, 0x86, 0x43, 0xA3, 0x20, 0x99, 0x86, 0x43, 0x83, 0x20, 0x00, 0xBD, 0x82,
        0x07, 0x03, 0x00, 0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x4F, 0x01, 0x58, 0x3A, 0x0F,
        0x0A, 0x54, 0x02, 0x29, 0x0F, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **The class, at its smallest member: `BodyShape::CallSeq` and Class B.**
    ///
    /// There is no Class A two-member-call body — a member call needs its
    /// receiver, so the receiver of the second call is live across the first
    /// `bl` by construction — which is why this rung's smallest cell already
    /// carries a callee-saved GPR where the free-function sequence's does not.
    #[test]
    fn a_member_call_sequence_is_a_class_b_call_seq() {
        let Some(BodyShape::CallSeq { params, calls, tail, saved, guard: None, .. }) =
            parse_segment(MCS_TWO, NO_LOCALS)
        else {
            panic!("a two-statement member-call body is a statement-call sequence");
        };
        assert_eq!(params, vec![0xF109], "the one formal, `s`");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].callee_tok, 0xE509, "?a — the first statement");
        assert_eq!(calls[1].callee_tok, 0xE609, "?b — the second");
        assert_eq!(tail, SeqTail::Void);
        assert_eq!(saved, vec![0], "the receiver is parked in r31 across both calls");
    }

    /// **`this` is argument SLOT 0, in every call of the sequence.**
    ///
    /// The argument list is in stream order — rightmost source argument first —
    /// so the receiver goes on the END of it and comes out of the marshalling as
    /// slot 0. Getting that backwards is invisible on a nullary call, which is
    /// what `member_tail_call_puts_this_in_slot_zero` pins for the tail form;
    /// this is the same fact one production over, and the two-receiver cell is
    /// what makes it visible (the two calls' slot-0 tokens DIFFER).
    #[test]
    fn the_receiver_is_argument_slot_zero_in_every_sequence_call() {
        let Some(BodyShape::CallSeq { params, calls, saved, .. }) =
            parse_segment(MCS_TWO_RECV, NO_LOCALS)
        else {
            panic!("a two-receiver member-call body is a statement-call sequence");
        };
        assert_eq!(params, vec![0xFB09, 0xFC09], "`s` then `t`");
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xFB09)], "call 1's `this` is `s`");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0xFC09)], "call 2's `this` is `t`");
        // The SECOND receiver is the parked one: the first call's is already in
        // r3 and dies there.
        assert_eq!(saved, vec![1]);
    }

    /// **The other reader route.** With a free call first the body never enters
    /// [`try_parse_member_tail_call`]; the member call is read inside
    /// `parse_call_sequence_from`'s statement arm, whose base census key for this
    /// exact body was `call-token-0xB9`.
    #[test]
    fn the_sequence_loop_reads_a_member_call_after_a_free_one() {
        let Some(BodyShape::CallSeq { calls, saved, .. }) =
            parse_segment(MCS_FREE_THEN, NO_LOCALS)
        else {
            panic!("a free call then a member call is a statement-call sequence");
        };
        assert_eq!(calls.len(), 2);
        assert!(calls[0].arg_ops.is_empty(), "the free call takes no arguments");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0x020A)], "the member call's `this`");
        assert_eq!(saved, vec![0], "`s` is live across the free call's `bl`");
    }

    /// **The workload's own spelling** — a member function calling its own
    /// methods, where the parked formal is the implicit `this` and the formals
    /// list has no `2D` run at all.
    #[test]
    fn the_implicit_this_receiver_parses_as_a_sequence() {
        let Some(BodyShape::CallSeq { params, calls, saved, .. }) =
            parse_segment(MCS_THIS, NO_LOCALS)
        else {
            panic!("an implicit-this member-call body is a statement-call sequence");
        };
        assert_eq!(params, vec![0x090A], "the implicit `this`");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0x090A)]);
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0x090A)]);
        assert_eq!(saved, vec![0]);
    }

    /// **W-FLTRET — the VALUE TAIL is admitted**, and the receiver still lands in
    /// slot 0 of the last call exactly as it does in the statement positions.
    ///
    /// This test asserted the **opposite** until w-fltret: w-mcall's decline D3
    /// was *"`SeqTail::CallValue` marshals a receiver into slot 0 and a post-op
    /// region, and the two have never been graded together"*. They are graded
    /// together now (`fixtures/cpp/wfltret_value_tail.cpp` cells F3 and F8, a
    /// whole-TU byte-exact match at `/O1` and `/Ox`), so the test is turned
    /// around rather than deleted — #1710a, a test that vanishes takes its
    /// coverage with it.
    #[test]
    fn a_member_call_in_the_value_tail_is_admitted_with_this_in_slot_zero() {
        let Some(BodyShape::CallSeq { calls, saved, tail, .. }) =
            parse_segment(MCS_VALUE_TAIL, NO_LOCALS)
        else {
            panic!("the integer member value tail is a statement-call sequence");
        };
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0x140A)], "the statement call's `this`");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0x140A)], "the VALUE call's `this`");
        assert_eq!(saved, vec![0], "`s` is live across the first `bl`");
        assert_eq!(tail, crate::func::body::SeqTail::CallValue { add_k: 0 });
    }

    /// **The FLOAT value tail is the same body with a different tail**, and the
    /// tail is what puts `_fltused` in the obj.
    ///
    /// `float v_float(O* o){ o->Poll(); return o->Level(); }`, captured
    /// (`work/w-fltret/probe/v1.cpp`, segment 0). Note the two CALL tokens: the
    /// statement call's return TYPE is `82 07 03` (void) and the value call's is
    /// `86 45 40` (a 4-byte real), and the result annotation `41 86 45 40` stands
    /// **immediately** after the `4C` — no `2C` conversion, which is the
    /// same-width rule this reader enforces.
    const MCS_VALUE_TAIL_FP: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0D, 0x53, 0x53, 0x26, 0xF1, 0x09,
        0x46, 0x2D, 0xF0, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xB9, 0xF0, 0x09, 0x86,
        0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80,
        0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x26, 0xE5, 0x09, 0xB9, 0xF0, 0x09, 0x86, 0x43, 0x81,
        0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x45, 0x40, 0x00, 0x80, 0x05, 0x10,
        0x00, 0x00, 0x4C, 0x41, 0x86, 0x45, 0x40, 0x3A, 0xF2, 0x09, 0x54, 0x02, 0x29, 0xF2, 0x09,
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The same cell with the value call's result **converted** — a `double`
    /// callee narrowed to a `float` result, which c2 lowers as an extra
    /// `frsp fr1,fr1` (`work/w-fltret/probe/v3.cod`). The only edit is the
    /// `2C 86 45 40 00` conversion spliced between the `4C` and the `41`, and
    /// the value CALL token's result type widened to `88 85 41`; everything else
    /// is byte for byte the cell above, so the pair separates **exactly** the
    /// immediacy rule and nothing else.
    const MCS_VALUE_TAIL_FP_NARROW: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0D, 0x53, 0x53, 0x26, 0xF1, 0x09,
        0x46, 0x2D, 0xF0, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xB9, 0xF0, 0x09, 0x86,
        0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80,
        0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x26, 0xE5, 0x09, 0xB9, 0xF0, 0x09, 0x86, 0x43, 0x81,
        0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x88, 0x85, 0x41, 0x00, 0x80, 0x05, 0x10,
        0x00, 0x00, 0x4C, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x41, 0x86, 0x45, 0x40, 0x3A, 0xF2, 0x09,
        0x54, 0x02, 0x29, 0xF2, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn the_float_value_tail_is_what_puts_fltused_in_the_obj() {
        let Some(BodyShape::CallSeq { calls, saved, tail, .. }) =
            parse_segment(MCS_VALUE_TAIL_FP, NO_LOCALS)
        else {
            panic!("the float member value tail is a statement-call sequence");
        };
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0xF009)], "the value call's `this`");
        assert_eq!(saved, vec![0]);
        assert_eq!(tail, crate::func::body::SeqTail::CallValueFp);

        // The whole point of the variant: the resolved function reports itself
        // FP-touching although its body has no FP instruction at all. W36 lost a
        // symbol by missing a shape in this producer, and the failure mode is an
        // obj one symbol short on every positive case at once —
        // `Port=Mismatch @ offset 12`, the COFF header's `NumberOfSymbols`.
        let fp = crate::func::IlFunction {
            call_seq: Some(crate::func::CallSeq {
                early: Vec::new(),
                guard: None,
                calls: Vec::new(),
                tail: crate::func::SeqTail::CallValueFp,
                saved: Vec::new(),
                store_run: None,
            }),
            ..crate::func::IlFunction::base("?v_float@@YAMPAUO@@@Z", &None)
        };
        assert!(fp.touches_floating_point(), "the FP value tail is a `_fltused` producer");
        // …and the integer sibling, which emits the identical instruction
        // stream, is not.
        let int_tail = crate::func::IlFunction {
            call_seq: Some(crate::func::CallSeq {
                early: Vec::new(),
                guard: None,
                calls: Vec::new(),
                tail: crate::func::SeqTail::CallValue { add_k: 0 },
                saved: Vec::new(),
                store_run: None,
            }),
            ..crate::func::IlFunction::base("?v_int@@YAHPAUO@@@Z", &None)
        };
        assert!(!int_tail.touches_floating_point());
    }

    /// **A conversion on the returned real is refused, and refused WITHOUT
    /// re-keying.** The narrowing direction costs `frsp fr1,fr1`, so admitting
    /// it would be wrong bytes; the widening direction wears the identical `2C`
    /// and costs nothing, and decline D6 gives that one up rather than build a
    /// width model. The second assertion is the no-re-key property: the block
    /// the body reports is the free-function reader's own, unchanged.
    #[test]
    fn a_converted_real_result_is_refused_and_keeps_its_key() {
        assert!(parse_segment(MCS_VALUE_TAIL_FP_NARROW, NO_LOCALS).is_none());
        let b = parse_segment_detail(MCS_VALUE_TAIL_FP_NARROW, NO_LOCALS).unwrap_err();
        assert_eq!(
            b.feature(),
            "expr-call-in-expr-recv-load-then-type-real-whole",
            "{}",
            b.feature()
        );
    }

    /// **A GUARDED sequence never admits a member call** — decline D4. That
    /// class is Class A only and hoists its entry block; no obj in this repo
    /// grades it with a receiver in slot 0, so `parse_call_sequence_from`
    /// excludes `guard`/`early` by name rather than by an argument.
    #[test]
    fn a_guarded_sequence_does_not_admit_a_member_call() {
        assert!(parse_segment(MCS_GUARDED, NO_LOCALS).is_none());
        let b = parse_segment_detail(MCS_GUARDED, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-brfalse", "{}", b.feature());
    }

    /// **The member arm can only turn an `Err` into an `Ok`.**
    ///
    /// Truncating a positive cell one byte at a time is the adversarial half a
    /// capture corpus cannot reach: at every prefix the parse must either accept
    /// (only the untruncated segment does) or refuse, and it must never panic
    /// and never run off the end. The atomic reader's cursor discipline is what
    /// this exercises — a member arm that consumed bytes on the way to a decline
    /// would hand the sequence loop a cursor in the middle of a token.
    #[test]
    fn the_member_arm_only_ever_turns_an_err_into_an_ok() {
        for cut in 1..MCS_TWO.len() {
            let seg = &MCS_TWO[..cut];
            assert!(
                parse_segment(seg, NO_LOCALS).is_none(),
                "a truncated segment is not a body ({cut} of {})",
                MCS_TWO.len()
            );
            // The detail parse must produce a Block rather than panicking.
            let _ = parse_segment_detail(seg, NO_LOCALS);
        }
        assert!(parse_segment(MCS_TWO, NO_LOCALS).is_some(), "the whole segment IS a body");
    }
}

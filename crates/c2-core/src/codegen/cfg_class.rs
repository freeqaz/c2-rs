//! **The emitter's CFG-class registry** — `docs/CEILING.md` §6.1 **phase 1**
//! (*"Emitter CFG classes — `cflow-loop`, `cflow-if-n`, `cflow-if-2`"*), built
//! as a construct rung at a required-zero byte delta.
//!
//! # The hole this closes, quoted from the thing that had it
//!
//! `c2_harness::gap::factors::PORT_CFG_CLASSES` is *the* answer to "which
//! control-flow shapes can this port express", and its own doc says what is
//! wrong with it:
//!
//! > *"This list is the screen's single assumption and it is the thing to
//! > re-check when a variant is added: it is a **hand-maintained mirror of a
//! > `c2-core` enum**, and nothing in the type system ties the two together."*
//!
//! > *"`Selected` has seven variants — `Plain`, `Tail`, `Float`, `Framed`,
//! > `Seq`, `CondPair` — and between them they cover **straight-line** bodies
//! > and **one two-arm conditional**."*
//!
//! Both sentences were false at `e82c9ede6`, in the direction a hand-maintained
//! mirror always fails in: the first sentence names *seven* and lists *six*,
//! and [`super::select::Selected`] has **eighteen** variants. *"Re-check when a
//! variant is added"* is an instruction to a human, and twelve variants were
//! added without it happening, **because nothing can fail when it is not**.
//!
//! # What is here, and what makes it not a second mirror
//!
//! Three things, and the third is the only one that is new in kind:
//!
//! 1. [`CflowClass`] — the census's **own** control-flow vocabulary, all seven
//!    shapes (`c2_il`'s `CfShape`), so the emitter can *name* the class it
//!    lowered into. Not three: `docs/CEILING.md` §6.1's phase-1 row names
//!    `cflow-loop`, `cflow-if-n` and `cflow-if-2`, and the census also mints
//!    **`cflow-multi-exit`** and **`cflow-switch`**, which no phase in the plan
//!    covers.
//! 2. [`Lowering`] — one variant per **dispatch arm** of
//!    [`super::select::select_function`], because the unit of a CFG-class claim
//!    is a *lowering* and not a `Selected` variant: `Selected::plain` carries
//!    [`Lowering::PtrWalkLoop`] (a back edge) **and** every straight-line leaf,
//!    so a claim keyed on `Selected` cannot be stated without over-claiming.
//! 3. [`class_of`] — an **exhaustive `match`**. Adding a [`Lowering`] without
//!    declaring its CFG class does not produce a stale comment; it produces a
//!    **compile error**. That is the tie the `PORT_CFG_CLASSES` doc says does
//!    not exist, and it is the only part of this module that a future lane
//!    cannot forget to run.
//!
//! [`lowering_of`] *is* still a mirror of `select_function`'s dispatch order —
//! that is stated rather than hidden. The difference from the mirror it
//! replaces is that this one is **graded on every run** against
//! `select_function` itself and against the census's own `cflow` string, over
//! every fixture, in both directions (see
//! `crates/c2-harness/tests/census_gate.rs`). Eliminating the mirror entirely
//! means tagging `select_function`'s thirty-nine `return Ok(Selected::…)` sites
//! with their arm, which is a **serial-spine** edit to `select.rs` and is
//! priced in `docs/rungs/2026-08-19-cfgclass.md` rather than taken here.
//!
//! # This module is a DECLARATION, never a gate
//!
//! Nothing here is consulted by an emitter, appears in an accept/refuse path,
//! or moves a numerator. `select_function` does not call it. It exists to be
//! *asked* — by the screen, by a test, by a future phase-1 lane — which is the
//! same standing this crate's other measurement-only surfaces have.
//!
//! # What is NOT claimed
//!
//! [`SHIPPED_CFG_CLAIMS`] carries a [`Claim::Partial`] for every class the port
//! lowers *some* body of and cannot lower *in general*, and **a partial claim
//! deliberately does not reach the screen** — `PORT_CFG_CLASSES` is derived
//! from the [`Claim::Whole`] rows alone, so `cfg-reach-shipped` is unmoved by
//! this module's existence. Five lanes declined to widen that list
//! (`w-rotate` §7, `w-sched2` §8, `w-subclass`, `w-blockir` §10,
//! `w-cflowlabel`) and board **#761** is the standing row; recording the
//! partial claim is what those lanes said they had no way to do, and it is not
//! the same act as shipping it.

use c2_il::IlFunction;

use super::leaf::addr::addr_leaf_text;
use super::leaf::compare::{cmp_shift_or_text, compare_leaf_text};
use super::leaf::float::float_leaf_text;
use super::leaf::load::indirect_load_text;
use super::leaf::store::store_leaf_text;
use super::select::OptMode;

/// **The census's control-flow vocabulary**, all seven shapes.
///
/// One-for-one with `c2_il`'s private `CfShape`, whose `name()` produces
/// exactly the strings [`Self::census_str`] renders. It is spelled here rather
/// than re-exported because `CfShape` is crate-private to `c2-il` and because
/// this enum means something different: `CfShape` is *what the census decoded*,
/// this is *what the emitter can produce*.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CflowClass {
    /// One basic block, no branch. `cflow-straight`.
    Straight,
    /// `cflow-multi-exit` — more than one exit, no forward join. **Named in no
    /// phase of `docs/CEILING.md` §6.1.**
    MultiExit,
    /// One forward conditional. `cflow-if-1`.
    If1,
    /// Two forward conditionals. `cflow-if-2`.
    If2,
    /// Three or more forward conditionals. `cflow-if-n`.
    IfN,
    /// At least one back edge. `cflow-loop`.
    Loop,
    /// A jump table. `cflow-switch`. **Named in no phase of §6.1**, and
    /// `CFG_SHAPE.md` §8.3 item S1 records that zero frontier TUs need it.
    Switch,
}

impl CflowClass {
    /// The census key **without** the `+expr-modeled` suffix — the spelling
    /// `PORT_CFG_CLASSES` matches against.
    ///
    /// The suffixed form is *the same CFG* with the statement layer fully
    /// decoded (`c2_il`'s `CfResidue`), which is why the screen's list carries
    /// both spellings of every class it claims and why
    /// [`Self::census_str_modeled`] is derived here rather than typed twice.
    pub const fn census_str(self) -> &'static str {
        match self {
            CflowClass::Straight => "cflow-straight",
            CflowClass::MultiExit => "cflow-multi-exit",
            CflowClass::If1 => "cflow-if-1",
            CflowClass::If2 => "cflow-if-2",
            CflowClass::IfN => "cflow-if-n",
            CflowClass::Loop => "cflow-loop",
            CflowClass::Switch => "cflow-switch",
        }
    }

    /// The `+expr-modeled` spelling of the same class. Same CFG, decoded
    /// operand stream.
    ///
    /// Spelled as literals rather than `format!` **because `PORT_CFG_CLASSES`
    /// is a `const`**: the screen's list is derived from this function, so it
    /// has to be `const fn` returning `&'static str`. A `String` here would
    /// have forced the census spellings to be typed a second time in the
    /// harness, which is the exact duplication this module exists to remove.
    pub const fn census_str_modeled(self) -> &'static str {
        match self {
            CflowClass::Straight => "cflow-straight+expr-modeled",
            CflowClass::MultiExit => "cflow-multi-exit+expr-modeled",
            CflowClass::If1 => "cflow-if-1+expr-modeled",
            CflowClass::If2 => "cflow-if-2+expr-modeled",
            CflowClass::IfN => "cflow-if-n+expr-modeled",
            CflowClass::Loop => "cflow-loop+expr-modeled",
            CflowClass::Switch => "cflow-switch+expr-modeled",
        }
    }

    /// Every class, so a caller can iterate the vocabulary without spelling it
    /// a second time.
    pub const ALL: [CflowClass; 7] = [
        CflowClass::Straight,
        CflowClass::MultiExit,
        CflowClass::If1,
        CflowClass::If2,
        CflowClass::IfN,
        CflowClass::Loop,
        CflowClass::Switch,
    ];

    /// Parse a census `cflow` string — either spelling — back into a class.
    /// `None` for a body the census never assigned a control-flow class to
    /// (the `cf-expr-…` bail), which is `CfgReach::Unclassified`'s population
    /// on the screen side and is **not** a class this enum has a variant for.
    pub fn from_census_str(s: &str) -> Option<CflowClass> {
        let bare = s.strip_suffix("+expr-modeled").unwrap_or(s);
        CflowClass::ALL.into_iter().find(|c| c.census_str() == bare)
    }

    /// A one-line rendering for tables.
    pub const fn short(self) -> &'static str {
        match self {
            CflowClass::Straight => "straight",
            CflowClass::MultiExit => "multi-exit",
            CflowClass::If1 => "if-1",
            CflowClass::If2 => "if-2",
            CflowClass::IfN => "if-n",
            CflowClass::Loop => "loop",
            CflowClass::Switch => "switch",
        }
    }
}

/// **One dispatch arm of [`super::select::select_function`].**
///
/// The order of the variants is the order of the arms, and that order is
/// load-bearing in `select_function` itself (its doc enumerates why: a tail
/// call must be asked ahead of every leaf recognizer, and so on). Keeping the
/// two in the same order is what makes [`lowering_of`] readable beside the
/// function it is graded against.
///
/// **Every variant is a shape the port already emits byte-exactly**, which is
/// what makes this registry a construct rung's material: nothing here is a
/// class the port aspires to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lowering {
    /// `func.framed_call` → `Selected::Framed`.
    FramedCall,
    /// `func.call_seq` → `Selected::Seq`.
    CallSeq,
    /// `func.cond_pair` → `Selected::CondPair`. W8, the port's first branch.
    CondPair,
    /// `func.ctor_forward_call` → `Selected::CtorForwardCall`.
    CtorForwardCall,
    /// `func.fp_store_diamond` → `Selected::FpStoreDiamond`.
    FpStoreDiamond,
    /// `func.tail_call` → `Selected::tail`, all five of its sub-arms.
    TailCall,
    /// `func.guard_chain_shared_tail` → `Selected::GuardChainSharedTail`.
    GuardChainSharedTail,
    /// `func.alloc_init_or_fail` → `Selected::AllocInitOrFail`.
    AllocInitOrFail,
    /// `func.osf_handle_guard` → `Selected::OsfHandleGuard`.
    OsfHandleGuard,
    /// `func.guard_ret_chain` → `Selected::GuardRetChain`.
    GuardRetChain,
    /// `func.close_call_chain` → `Selected::CloseCallChain`.
    CloseCallChain,
    /// `func.xlrc_create_guard` → `Selected::XlrcCreateGuard`.
    XlrcCreateGuard,
    /// `func.json_utf8_copy` → `Selected::JsonUtf8Copy`.
    JsonUtf8Copy,
    /// `func.if_call_join` → `Selected::IfCallJoin`.
    IfCallJoin,
    /// `func.pool_free_list` → `Selected::plain`.
    PoolFreeList,
    /// `func.pool_ctor_chain` → `Selected::plain`.
    PoolCtorChain,
    /// `func.memcpy_tail` → `Selected::memcpy_tail`.
    MemcpyTail,
    /// `func.nonce_add_run` → `Selected::plain`.
    NonceAddRun,
    /// `func.xtea_round_loop` → `Selected::plain`.
    XteaRoundLoop,
    /// `func.xtea_encrypt_loop` → `Selected::XteaEncryptLoop`.
    XteaEncryptLoop,
    /// `func.ptr_walk_loop` → `Selected::plain`.
    PtrWalkLoop,
    /// `func.static_scan_loop` → `Selected::plain`.
    StaticScanLoop,
    /// `func.global_store_leaf` → `Selected::plain`.
    GlobalStoreLeaf,
    /// `func.counted_accum_loop` → `Selected::plain`.
    CountedAccumLoop,
    /// `func.float_walk_loop` → `Selected::plain`.
    FloatWalkLoop,
    /// `func.ptr_walk_chain_loop` → `Selected::plain`.
    PtrWalkChainLoop,
    /// `func.div_mod_leaf` → `Selected::plain`.
    DivModLeaf,
    /// `func.empty_body()` → `Selected::plain`, a bare `blr`.
    EmptyBody,
    /// `func.float_leaf` → `Selected::Float`.
    FloatLeaf,
    /// `indirect_load_text` → `Selected::plain`.
    IndirectLoadLeaf,
    /// `addr_leaf_text` → `Selected::plain`.
    AddrLeaf,
    /// `store_leaf_text` → `Selected::plain`.
    StoreLeaf,
    /// `func.cmp_shift_or` → `Selected::plain`.
    CmpShiftOr,
    /// `func.compare` → `Selected::plain`.
    CompareLeaf,
    /// The fall-through: `select_text`, the ordinary arithmetic selector.
    Straightline,
}

impl Lowering {
    /// Every lowering, in `select_function`'s dispatch order.
    pub const ALL: [Lowering; 35] = [
        Lowering::FramedCall,
        Lowering::CallSeq,
        Lowering::CondPair,
        Lowering::CtorForwardCall,
        Lowering::FpStoreDiamond,
        Lowering::TailCall,
        Lowering::GuardChainSharedTail,
        Lowering::AllocInitOrFail,
        Lowering::OsfHandleGuard,
        Lowering::GuardRetChain,
        Lowering::CloseCallChain,
        Lowering::XlrcCreateGuard,
        Lowering::JsonUtf8Copy,
        Lowering::IfCallJoin,
        Lowering::PoolFreeList,
        Lowering::PoolCtorChain,
        Lowering::MemcpyTail,
        Lowering::NonceAddRun,
        Lowering::XteaRoundLoop,
        Lowering::XteaEncryptLoop,
        Lowering::PtrWalkLoop,
        Lowering::StaticScanLoop,
        Lowering::GlobalStoreLeaf,
        Lowering::CountedAccumLoop,
        Lowering::FloatWalkLoop,
        Lowering::PtrWalkChainLoop,
        Lowering::DivModLeaf,
        Lowering::EmptyBody,
        Lowering::FloatLeaf,
        Lowering::IndirectLoadLeaf,
        Lowering::AddrLeaf,
        Lowering::StoreLeaf,
        Lowering::CmpShiftOr,
        Lowering::CompareLeaf,
        Lowering::Straightline,
    ];

    /// A stable name for messages and tables.
    pub const fn name(self) -> &'static str {
        match self {
            Lowering::FramedCall => "framed_call",
            Lowering::CallSeq => "call_seq",
            Lowering::CondPair => "cond_pair",
            Lowering::CtorForwardCall => "ctor_forward_call",
            Lowering::FpStoreDiamond => "fp_store_diamond",
            Lowering::TailCall => "tail_call",
            Lowering::GuardChainSharedTail => "guard_chain_shared_tail",
            Lowering::AllocInitOrFail => "alloc_init_or_fail",
            Lowering::OsfHandleGuard => "osf_handle_guard",
            Lowering::GuardRetChain => "guard_ret_chain",
            Lowering::CloseCallChain => "close_call_chain",
            Lowering::XlrcCreateGuard => "xlrc_create_guard",
            Lowering::JsonUtf8Copy => "json_utf8_copy",
            Lowering::IfCallJoin => "if_call_join",
            Lowering::PoolFreeList => "pool_free_list",
            Lowering::PoolCtorChain => "pool_ctor_chain",
            Lowering::MemcpyTail => "memcpy_tail",
            Lowering::NonceAddRun => "nonce_add_run",
            Lowering::XteaRoundLoop => "xtea_round_loop",
            Lowering::XteaEncryptLoop => "xtea_encrypt_loop",
            Lowering::PtrWalkLoop => "ptr_walk_loop",
            Lowering::StaticScanLoop => "static_scan_loop",
            Lowering::GlobalStoreLeaf => "global_store_leaf",
            Lowering::CountedAccumLoop => "counted_accum_loop",
            Lowering::FloatWalkLoop => "float_walk_loop",
            Lowering::PtrWalkChainLoop => "ptr_walk_chain_loop",
            Lowering::DivModLeaf => "div_mod_leaf",
            Lowering::EmptyBody => "empty_body",
            Lowering::FloatLeaf => "float_leaf",
            Lowering::IndirectLoadLeaf => "indirect_load_leaf",
            Lowering::AddrLeaf => "addr_leaf",
            Lowering::StoreLeaf => "store_leaf",
            Lowering::CmpShiftOr => "cmp_shift_or",
            Lowering::CompareLeaf => "compare_leaf",
            Lowering::Straightline => "straightline",
        }
    }
}

/// **The declaration: which CFG classes each lowering emits.**
///
/// A **set**, not a class, and that is this lane's headline measurement rather
/// than a design preference. The first version of this function returned one
/// [`CflowClass`] per [`Lowering`]; grading it against the census over 1,820
/// fixture bodies produced **53** disagreements, all of them on two arms:
///
/// | arm | census classes observed |
/// |---|---|
/// | [`Lowering::CallSeq`] | `cflow-straight`, `cflow-if-1`, `cflow-if-2`, `cflow-if-n`, `cflow-multi-exit` |
/// | [`Lowering::TailCall`] | `cflow-straight`, `cflow-multi-exit` |
///
/// **A lowering is not a function of one CFG class**, because a `Selected`
/// variant is not one either — `Selected::Seq`'s guards and early returns are
/// control flow the census counts and the variant's name does not mention.
/// `PORT_CFG_CLASSES`' sentence *"between them they cover straight-line bodies
/// and one two-arm conditional"* is refuted by the port's own fixtures.
///
/// An **exhaustive `match`**, which is the whole mechanism: adding a
/// [`Lowering`] without declaring its classes is a **compile error**, not a
/// stale comment. Every arm is graded against the census's own `cflow` string
/// over every fixture body the port accepts, at **both** capture profiles
/// (`crates/c2-harness/tests/census_gate.rs`), and a wrong arm is a red test.
///
/// # A set is a superset claim, and the direction it can be wrong in is named
///
/// The test asserts *observed ∈ declared*. So a class listed here and never
/// observed is **not** caught by it — that is the over-claiming direction, and
/// it is covered separately by
/// `every_declared_class_is_observed_at_least_once`, which fails on a declared
/// class no capture profile ever produced. Without that second test the safe
/// move would be to list all seven everywhere, which is a declaration that says
/// nothing.
pub const fn classes_of(l: Lowering) -> &'static [CflowClass] {
    match l {
        // ---- one basic block, no branch --------------------------------
        Lowering::FramedCall => &[CflowClass::Straight],
        Lowering::CtorForwardCall => &[CflowClass::Straight],
        Lowering::MemcpyTail => &[CflowClass::Straight],
        Lowering::NonceAddRun => &[CflowClass::Straight],
        Lowering::GlobalStoreLeaf => &[CflowClass::Straight],
        Lowering::DivModLeaf => &[CflowClass::Straight],
        Lowering::EmptyBody => &[CflowClass::Straight],
        Lowering::FloatLeaf => &[CflowClass::Straight],
        Lowering::IndirectLoadLeaf => &[CflowClass::Straight],
        Lowering::AddrLeaf => &[CflowClass::Straight],
        Lowering::StoreLeaf => &[CflowClass::Straight],
        Lowering::CmpShiftOr => &[CflowClass::Straight],
        Lowering::CompareLeaf => &[CflowClass::Straight],
        Lowering::Straightline => &[CflowClass::Straight],

        // ---- the two arms whose class is NOT single, measured ----------
        Lowering::CallSeq => &[
            CflowClass::Straight,
            CflowClass::MultiExit,
            CflowClass::If1,
            CflowClass::If2,
            CflowClass::IfN,
        ],
        Lowering::TailCall => &[CflowClass::Straight, CflowClass::MultiExit],

        // ---- one forward conditional -----------------------------------
        //
        // **`PoolFreeList` is here and not above, and the correction is worth
        // the line.** `codegen::pool_free_list`'s own header says both bodies
        // "fold their guard to a conditional" — no branch in the emitted words —
        // and this file first declared it `Straight` on the strength of that
        // sentence. The census disagreed on three fixture bodies. It is right
        // and the sentence is about a different thing: a `CflowClass` here is
        // the class of the **input IL**, which is what `PORT_CFG_CLASSES`
        // matches against, and never the shape of the bytes that come out.
        // A registry keyed on emitted control flow would be the wrong key for
        // the only question the screen asks.
        Lowering::PoolFreeList => &[CflowClass::If1],
        Lowering::CondPair => &[CflowClass::If1],
        Lowering::FpStoreDiamond => &[CflowClass::If1],

        // ---- three or more forward conditionals ------------------------
        // Measured `if-n`, not `if-1`: `w-undname`'s guarded allocation has
        // three forward conditionals in the IL however few survive the fold.
        Lowering::AllocInitOrFail => &[CflowClass::IfN],
        Lowering::GuardChainSharedTail => &[CflowClass::IfN],
        Lowering::OsfHandleGuard => &[CflowClass::IfN],
        Lowering::GuardRetChain => &[CflowClass::IfN],
        Lowering::CloseCallChain => &[CflowClass::IfN],
        Lowering::XlrcCreateGuard => &[CflowClass::IfN],
        Lowering::IfCallJoin => &[CflowClass::IfN],

        // ---- a back edge -----------------------------------------------
        Lowering::JsonUtf8Copy => &[CflowClass::Loop],
        Lowering::PoolCtorChain => &[CflowClass::Loop],
        Lowering::XteaRoundLoop => &[CflowClass::Loop],
        Lowering::XteaEncryptLoop => &[CflowClass::Loop],
        Lowering::PtrWalkLoop => &[CflowClass::Loop],
        Lowering::StaticScanLoop => &[CflowClass::Loop],
        Lowering::CountedAccumLoop => &[CflowClass::Loop],
        Lowering::FloatWalkLoop => &[CflowClass::Loop],
        Lowering::PtrWalkChainLoop => &[CflowClass::Loop],
    }
}

/// `true` when `l` is declared to emit `c`.
pub fn emits(l: Lowering, c: CflowClass) -> bool {
    classes_of(l).contains(&c)
}

/// Every lowering declared to emit `c`, in [`Lowering::ALL`] order.
pub fn lowerings_emitting(c: CflowClass) -> Vec<Lowering> {
    Lowering::ALL.into_iter().filter(|l| emits(*l, c)).collect()
}

/// **Which dispatch arm of [`super::select::select_function`] takes this
/// function**, or `None` when none does.
///
/// # This is a mirror, and it is the graded kind
///
/// It re-states `select_function`'s dispatch order rather than being derived
/// from it, and the module header says why that is where the line was drawn.
/// What makes it different in kind from the mirror it replaces is that
/// **nothing about it is asserted**: `census_gate.rs` runs it beside
/// `select_function` over every fixture and requires the two to agree in
/// **both** directions —
///
/// * `lowering_of(f).is_some()` where `select_function(f)` refuses is the
///   **over-claiming** direction, and is the one that matters (board **#3270**:
///   a predicate can be 39.6 % wrong about c2 and free in the metric used to
///   choose it);
/// * `lowering_of(f).is_none()` where `select_function(f)` accepts is the
///   under-claiming direction, and is the one a lazily-written test catches by
///   accident.
///
/// Both are must-fail-mutated in the rung.
pub fn lowering_of(func: &IlFunction, mode: OptMode) -> Option<Lowering> {
    // Board #844's carrier gate, asked ahead of the dispatch exactly as
    // `select_function` asks it: a body carrying both `ops` and a call field is
    // refused outright, so no arm below is reachable for it.
    if super::store_run_call::gate_carrier(func).is_err() {
        return None;
    }
    if func.framed_call().is_some() {
        return Some(Lowering::FramedCall);
    }
    if func.call_seq().is_some() {
        return Some(Lowering::CallSeq);
    }
    if func.cond_pair().is_some() {
        return Some(Lowering::CondPair);
    }
    if func.ctor_forward_call().is_some() {
        return Some(Lowering::CtorForwardCall);
    }
    if func.fp_store_diamond().is_some() {
        return Some(Lowering::FpStoreDiamond);
    }
    if func.tail_call().is_some() {
        return Some(Lowering::TailCall);
    }
    if func.guard_chain_shared_tail().is_some() {
        return Some(Lowering::GuardChainSharedTail);
    }
    if func.alloc_init_or_fail().is_some() {
        return Some(Lowering::AllocInitOrFail);
    }
    if func.osf_handle_guard().is_some() {
        return Some(Lowering::OsfHandleGuard);
    }
    if func.guard_ret_chain().is_some() {
        return Some(Lowering::GuardRetChain);
    }
    if func.close_call_chain().is_some() {
        return Some(Lowering::CloseCallChain);
    }
    if func.xlrc_create_guard().is_some() {
        return Some(Lowering::XlrcCreateGuard);
    }
    if func.json_utf8_copy().is_some() {
        return Some(Lowering::JsonUtf8Copy);
    }
    if func.if_call_join().is_some() {
        return Some(Lowering::IfCallJoin);
    }
    if func.pool_free_list().is_some() {
        return Some(Lowering::PoolFreeList);
    }
    if func.pool_ctor_chain().is_some() {
        return Some(Lowering::PoolCtorChain);
    }
    if func.memcpy_tail().is_some() {
        return Some(Lowering::MemcpyTail);
    }
    if func.nonce_add_run().is_some() {
        return Some(Lowering::NonceAddRun);
    }
    if func.xtea_round_loop().is_some() {
        return Some(Lowering::XteaRoundLoop);
    }
    if func.xtea_encrypt_loop().is_some() {
        return Some(Lowering::XteaEncryptLoop);
    }
    if func.ptr_walk_loop().is_some() {
        return Some(Lowering::PtrWalkLoop);
    }
    if func.static_scan_loop().is_some() {
        return Some(Lowering::StaticScanLoop);
    }
    if func.global_store_leaf().is_some() {
        return Some(Lowering::GlobalStoreLeaf);
    }
    if func.counted_accum_loop().is_some() {
        return Some(Lowering::CountedAccumLoop);
    }
    if func.float_walk_loop().is_some() {
        return Some(Lowering::FloatWalkLoop);
    }
    if func.ptr_walk_chain_loop().is_some() {
        return Some(Lowering::PtrWalkChainLoop);
    }
    if func.div_mod_leaf().is_some() {
        return Some(Lowering::DivModLeaf);
    }
    if func.empty_body() {
        return Some(Lowering::EmptyBody);
    }
    // ---- the tail arms, whose emitters REFUSE without needing `base_off` ----
    //
    // Every arm from here down is entered on a carrier and can still refuse
    // *inside*, and `select_function` propagates that refusal with `?` rather
    // than falling through to the next arm. So each of these returns `None`
    // outright on a refusal — falling through would name an arm
    // `select_function` never reaches, which is the over-claiming direction.
    //
    // **This was found by measurement, not by reading.** The first version of
    // this function returned `Some(FloatLeaf)` on `func.float_leaf().is_some()`
    // and the grading test reported exactly one over-claim,
    // `w13_fscratch.cpp :: ?fm13@@YAMMMMMMMMMMMMMM@Z`, whose thirteen float
    // formals `float_leaf_text` declines. One cell in 1,820, in the unsound
    // direction, at zero cost to every other number — board **#3270**'s shape.
    if let Some(double) = func.float_leaf() {
        // The mode is not available on this path and the policy cannot change
        // an `is_ok`: `FpTempPolicy` picks *which* scratch register, never
        // whether one is available (`take_fp` fails only when all 14 are live,
        // which is policy-independent). `Ox` is passed as the arbitrary one and
        // this comment is why that is sound rather than convenient.
        return float_leaf_text(func, double, crate::codegen::select::OptMode::Ox)
            .is_ok()
            .then_some(Lowering::FloatLeaf);
    }
    // The three predicate-shaped leaves are CALLED, not re-implemented: they
    // already return `Option<Result<…>>`, so asking them here is asking the
    // same question `select_function` asks, in the same words — including the
    // INNER `Result`, which is the arm's own refusal.
    if let Some(r) = indirect_load_text(func) {
        return r.is_ok().then_some(Lowering::IndirectLoadLeaf);
    }
    if let Some(r) = addr_leaf_text(func) {
        return r.is_ok().then_some(Lowering::AddrLeaf);
    }
    if let Some(r) = store_leaf_text(func, mode) {
        return r.is_ok().then_some(Lowering::StoreLeaf);
    }
    if let Some(cso) = func.cmp_shift_or() {
        return cmp_shift_or_text(cso, mode).is_ok().then_some(Lowering::CmpShiftOr);
    }
    if let Some(cmp) = func.compare() {
        return compare_leaf_text(cmp, mode).is_ok().then_some(Lowering::CompareLeaf);
    }
    // The fall-through arm.
    if super::straightline::select_text(func, mode).is_ok() {
        return Some(Lowering::Straightline);
    }
    None
}

/// How much of a [`CflowClass`] the port claims.
///
/// The two cases are the same distinction `c2_harness`'s `CfgSub` makes, one
/// level up: `CfgSub` restricts a *shipped* claim to named census keys, and
/// this says whether the claim is shipped at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Claim {
    /// **The class in general.** The port lowers an arbitrary body of this
    /// class. This is the only kind of claim that reaches the screen.
    Whole,
    /// **These lowerings and nothing else.** The port emits *some* body of this
    /// class byte-exactly and has no general lowering for it.
    ///
    /// This is the claim `w-rotate` §7 and `w-sched2` §8 both measured and had
    /// nowhere to record — *"`cflow-loop`, restricted to the sentinel walk at
    /// `/O1`, pointer formal at slot 0, chains of single-word producers with no
    /// hoisted literal"*. Recording it is not shipping it: see
    /// [`SHIPPED_CFG_CLAIMS`].
    Partial(&'static [Lowering]),
}

/// One class's claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassClaim {
    pub class: CflowClass,
    pub claim: Claim,
}

/// **What the port claims about each CFG class**, and the one place a screen
/// should read it from.
///
/// # The `Whole` rows are the only ones that reach `PORT_CFG_CLASSES`
///
/// `c2_harness::gap::factors::PORT_CFG_CLASSES` is **derived** from this
/// constant's `Whole` rows (two census spellings each, bare and
/// `+expr-modeled`), and a test asserts the derived list equals the four-entry
/// literal it replaced. So this module's arrival moves `cfg-reach-shipped` by
/// **zero**, by construction and not by luck.
///
/// # The `Partial` rows are the honest part, and shipping them is a SEPARATE lane
///
/// A `Partial` row says *"the port emits some body of this class byte-exactly"*
/// — which is true, and which the screen has never been able to say. It is
/// **not** the claim `PORT_CFG_CLASSES` makes. Turning one into a
/// `CfgSub::Keys` entry is a widening of a published claim and is priced
/// against the oracle in `docs/rungs/2026-08-19-cfgclass.md` §5 before anyone
/// ships it; board **#761** and five lanes stand behind not doing it here.
///
/// # `Switch` is absent, and the absence is the claim
///
/// The port has no `cflow-switch` lowering at all, so there is no row for it —
/// not a `Partial(&[])`, which would read as a claim of nothing where the truth
/// is the absence of a lowering. `CFG_SHAPE.md` §8.3 item S1.
pub const SHIPPED_CFG_CLAIMS: &[ClassClaim] = &[
    ClassClaim { class: CflowClass::Straight, claim: Claim::Whole },
    ClassClaim { class: CflowClass::If1, claim: Claim::Whole },
    // ---- the partial claims. NONE of these reaches the screen. ----------
    //
    // Every list below is exactly `lowerings_emitting(class)` minus the arms
    // the wholesale rows already cover, and a unit test re-derives it rather
    // than trusting the typing.
    ClassClaim {
        class: CflowClass::MultiExit,
        claim: Claim::Partial(&[Lowering::CallSeq, Lowering::TailCall]),
    },
    ClassClaim {
        class: CflowClass::If2,
        claim: Claim::Partial(&[Lowering::CallSeq]),
    },
    ClassClaim {
        class: CflowClass::IfN,
        claim: Claim::Partial(&[
            Lowering::CallSeq,
            Lowering::AllocInitOrFail,
            Lowering::GuardChainSharedTail,
            Lowering::OsfHandleGuard,
            Lowering::GuardRetChain,
            Lowering::CloseCallChain,
            Lowering::XlrcCreateGuard,
            Lowering::IfCallJoin,
        ]),
    },
    ClassClaim {
        class: CflowClass::Loop,
        claim: Claim::Partial(&[
            Lowering::JsonUtf8Copy,
            Lowering::PoolCtorChain,
            Lowering::XteaRoundLoop,
            Lowering::XteaEncryptLoop,
            Lowering::PtrWalkLoop,
            Lowering::StaticScanLoop,
            Lowering::CountedAccumLoop,
            Lowering::FloatWalkLoop,
            Lowering::PtrWalkChainLoop,
        ]),
    },
];

impl ClassClaim {
    /// `true` when this claim is the wholesale one.
    pub const fn is_whole(&self) -> bool {
        matches!(self.claim, Claim::Whole)
    }
}

/// The classes claimed **wholesale**, in [`CflowClass::ALL`] order — the exact
/// input `PORT_CFG_CLASSES` is derived from.
pub fn whole_claim_classes() -> Vec<CflowClass> {
    let mut v: Vec<CflowClass> = SHIPPED_CFG_CLAIMS
        .iter()
        .filter(|c| c.is_whole())
        .map(|c| c.class)
        .collect();
    v.sort();
    v
}

/// Every census class string a `Whole` claim covers, both spellings, in the
/// order `PORT_CFG_CLASSES` lists them (class, then its `+expr-modeled` form).
///
/// **This is the list the screen must equal**, and `census_gate.rs` asserts it
/// does.
pub fn whole_claim_census_strings() -> Vec<&'static str> {
    let mut v = Vec::new();
    for c in whole_claim_classes() {
        v.push(c.census_str());
        v.push(c.census_str_modeled());
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every declared class renders a census string the census could have
    /// produced, and every census string round-trips.
    #[test]
    fn census_strings_round_trip() {
        for c in CflowClass::ALL {
            assert_eq!(CflowClass::from_census_str(c.census_str()), Some(c));
            assert_eq!(CflowClass::from_census_str(c.census_str_modeled()), Some(c));
        }
        assert_eq!(CflowClass::from_census_str("cf-expr-0x05"), None);
        assert_eq!(CflowClass::from_census_str("cflow-straightish"), None);
    }

    /// `Lowering::ALL` is the whole enum. A variant added without extending the
    /// array is invisible to every table in the screen, so the array is checked
    /// against the exhaustive `name()` match rather than trusted.
    #[test]
    fn lowering_all_is_complete_and_has_no_duplicates() {
        let mut names: Vec<&str> = Lowering::ALL.iter().map(|l| l.name()).collect();
        let n = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), n, "Lowering::ALL has a duplicate");
        assert_eq!(n, 35, "Lowering::ALL is not the whole enum");
    }

    /// **The `Whole` claims are exactly the classes the screen ships**, and
    /// this is the c2-core end of the identity `census_gate.rs` takes at the
    /// harness end. Frozen against the four-entry literal `PORT_CFG_CLASSES`
    /// carried at master `e82c9ede6`.
    #[test]
    fn whole_claims_reproduce_the_frozen_port_cfg_classes_list() {
        assert_eq!(
            whole_claim_census_strings(),
            vec![
                "cflow-straight",
                "cflow-straight+expr-modeled",
                "cflow-if-1",
                "cflow-if-1+expr-modeled",
            ]
        );
    }

    /// **No class is claimed twice**, and a `Partial` row lists only lowerings
    /// [`classes_of`] actually puts in that class. A row listing some other
    /// class's lowering would be a claim about a population it does not
    /// describe.
    #[test]
    fn claims_are_one_per_class_and_internally_consistent() {
        let mut seen: Vec<CflowClass> = Vec::new();
        for c in SHIPPED_CFG_CLAIMS {
            assert!(!seen.contains(&c.class), "{:?} claimed twice", c.class);
            seen.push(c.class);
            if let Claim::Partial(ls) = c.claim {
                assert!(!ls.is_empty(), "{:?}: an empty Partial is not a claim", c.class);
                for l in ls {
                    assert!(
                        emits(*l, c.class),
                        "{} is listed under {:?} and does not emit it",
                        l.name(),
                        c.class
                    );
                }
            }
        }
    }

    /// **Every (lowering, class) pair whose class is not claimed wholesale is
    /// listed in that class's `Partial` row**, and the row is *exactly* that
    /// set — no more, no fewer.
    ///
    /// This is the direction that catches the failure `PORT_CFG_CLASSES`
    /// actually had: a lowering added for a branchy class and silently
    /// unrecorded, so the registry under-states what the port emits while
    /// looking complete. The equality (rather than containment) is what stops
    /// the opposite: a row padded with arms that do not emit the class.
    #[test]
    fn every_partial_row_is_exactly_its_class_s_lowerings() {
        let whole = whole_claim_classes();
        for c in CflowClass::ALL {
            if whole.contains(&c) {
                continue;
            }
            let expected = lowerings_emitting(c);
            let row = SHIPPED_CFG_CLAIMS.iter().find(|cc| cc.class == c);
            if expected.is_empty() {
                assert!(
                    row.is_none(),
                    "{:?} has a claim row and no lowering emits it — an absent \
                     lowering must be an absent row, not an empty claim",
                    c
                );
                continue;
            }
            let row = row.unwrap_or_else(|| {
                panic!(
                    "{:?} is emitted by {} lowering(s) and has no claim row: {:?}",
                    c,
                    expected.len(),
                    expected.iter().map(|l| l.name()).collect::<Vec<_>>()
                )
            });
            let Claim::Partial(ls) = row.claim else {
                panic!("{:?} is claimed Whole but is not in whole_claim_classes()", c)
            };
            let mut got: Vec<Lowering> = ls.to_vec();
            got.sort();
            let mut want = expected.clone();
            want.sort();
            assert_eq!(
                got.iter().map(|l| l.name()).collect::<Vec<_>>(),
                want.iter().map(|l| l.name()).collect::<Vec<_>>(),
                "{:?}'s Partial row is not exactly the set of lowerings that emit it",
                c
            );
        }
    }

    /// Every declared class list is non-empty, sorted and duplicate-free. A
    /// lowering that emits nothing is not a lowering.
    #[test]
    fn declared_class_lists_are_well_formed() {
        for l in Lowering::ALL {
            let cs = classes_of(l);
            assert!(!cs.is_empty(), "{} declares no CFG class", l.name());
            let mut sorted = cs.to_vec();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), cs.len(), "{} declares a class twice", l.name());
            assert_eq!(sorted.as_slice(), cs, "{}'s class list is not sorted", l.name());
        }
    }

    /// **`Switch` is emitted by nothing**, and there is therefore no claim row
    /// for it. The absence is the claim — `CFG_SHAPE.md` §8.3 item S1.
    #[test]
    fn nothing_emits_switch_and_nothing_claims_it() {
        assert!(lowerings_emitting(CflowClass::Switch).is_empty());
        assert!(!SHIPPED_CFG_CLAIMS.iter().any(|c| c.class == CflowClass::Switch));
    }
}

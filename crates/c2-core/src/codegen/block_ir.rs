//! The block/terminator IR — `docs/CFG_SHAPE.md` §6.2 item **A**, built.
//!
//! > **A. Basic blocks with an explicit emission order, and terminators.** A
//! > block is a straight-line instruction run ending in exactly one terminator:
//! > fall-through, `bc(cond, target)`, `b(target)`, `bclr(cond)`, `blr`, or a
//! > tail `b(symbol)`. The *order of blocks in the output* must be an explicit
//! > property, not implied by traversal, because §3.4 shows it is the IL's
//! > statement order and §3.4.1 shows one measured case where it is not — a
//! > lowering must be able to state the order it chose, so the two can be told
//! > apart.
//!
//! # What this is, in one paragraph
//!
//! [`BodyLayout`] is a two-phase builder for **one function body**. Every block
//! is [`declare`](BodyLayout::declare)d first, which mints its identity and
//! nothing else, and [`place`](BodyLayout::place)d second, which appends it to
//! the **emission order** with its straight-line bytes and its one
//! [`Terminator`]. [`finish`](BodyLayout::finish) walks that order once, lays
//! the bytes down, and hands the branches to the existing label map. Emission
//! order is therefore *placement* order — a thing the lowering did on purpose
//! and can be asked about — and never the order a traversal happened to reach
//! blocks in, which is the property §6.2 item A is written to demand.
//!
//! # This is a RE-EXPRESSION, and its whole success criterion is zero moved bytes
//!
//! It was built as a construct rung (`docs/rungs/README.md` § "Lane kinds";
//! precedent board **#290**) by taking [`super::cond_tail`] — a class that was
//! **already byte-exact against real `c2.dll`** — and rebuilding its emitter on
//! top of these types. Nothing here was designed from first principles and then
//! believed: every rule in it either produces the bytes `CFG_SHAPE.md` §4.1 read
//! off the real obj, or it is wrong. The lane converts **zero** TUs by design; a
//! conversion would have meant behaviour moved.
//!
//! # What it deliberately does NOT contain (`CFG_SHAPE.md` §6.3)
//!
//! No code motion, no cost model, no loop rotation, no CTR-loop discovery, no
//! neutrality classifier — and, specifically, **no instruction scheduler**. Lane
//! `w-dagorder` established that c2 runs a cycle-driven dependence-DAG list
//! scheduler, which is §6.2 item **F** and is much larger than board #1823
//! priced it. These types are shaped so that a scheduler *could later* be
//! expressed as a pass over a block's `body` — that is the point of a block
//! owning its instruction run — but no order is assumed here beyond the one the
//! already-byte-exact class produces today.
//!
//! Items **E** (a condition-code model with two producers), **F** (values live
//! across block boundaries) and **G** (a per-shape record of §3.5's folds) are
//! **not** built. [`Terminator::Bc`] carries a raw `(BO, BI)` pair for exactly
//! that reason: `BI` already encodes which condition register the branch reads
//! (`cr6` bit 26 for a compare, `cr0` bit 2 for a record form), so this IR can
//! *carry* item E's distinction without *modelling* it, and a later item-E lane
//! adds the producer side without changing a byte here.
//!
//! ## ✔ 2026-08-14, lane `w-ir-e` — **item E's producer side is built, and the
//! paragraph above held: not a byte here moved**
//!
//! The prediction is scored rather than quietly overwritten. [`super::cond`] is
//! the model — `CondProducer` (an explicit compare, which *names* its field;
//! a record form, which cannot), `Cond` (producer + `BO` + bit, with `BI`
//! **derived** from the producer), and a decoder that reads a producer off an
//! instruction run. [`Terminator::Bc`]'s representation is untouched; what is
//! new is that [`Terminator::bc`] can *derive* the pair, that
//! [`Terminator::reads_crf`] can be asked which field a branch reads, and that
//! [`BasicBlock::cond_source`] can be asked which instruction in the block's own
//! run wrote it. [`BodyLayout::place`] refuses a block where those two disagree
//! — §3.2's `409a…`-for-`4082…` hazard, board **#188**, made structural.
//!
//! One correction rides along: item E's own text pairs *compare* with *cr6*, and
//! this crate's shipped bytes refute that as a biconditional —
//! `close_call_chain` and `alloc_init_or_fail` both compare into **cr0**. The
//! model carries the field on the compare instead of assuming it; see
//! [`super::cond`]'s header.
//!
//! ## ✔ 2026-08-15, lane `w-layout` — **the layout owns the POSITIONS, and that
//! is board #3124's prerequisite**
//!
//! [`BodyLayout`] had exactly **one** production client when `w-item-d` counted
//! (#3124), and the crate had **23** branch sites in **13** other lowerings that
//! computed every displacement themselves and patched at a **fixed offset** —
//! *"the one shape a re-layout cannot serve, because a fixed site has nowhere to
//! grow"*. This lane moved the ones that can move.
//!
//! What made it possible is **one** new fact, and finding that it was one is the
//! result: [`FinishedBody::start_of`]. The branch sites were never the whole
//! problem. Every one of those lowerings also *publishes* offsets off the same
//! running byte vector — a `bl`'s `REL24` site, a float constant's
//! `REFHI`/`REFLO` pair, the prologue's length — and a lowering cannot hand its
//! branch positions to a layout while keeping those in a counter of its own,
//! because both come off the same `t.len()`. Stating them as
//! `start_of(block) + k` makes `k` a constant of **one block's own run** instead
//! of a constant of the whole body, and [`FinishedBody::at`] checks that it is.
//!
//! **Nothing here relaxes anything and no byte moved** — the criterion was a
//! required-zero delta and it held. The residue is stated rather than implied:
//! five lowerings still cannot reach a layout at all, three because
//! [`LabelMap`]'s invariant 4 refuses their back edge (#746, and it is right to)
//! and two because their back edge is `bdnz`, for which [`Terminator`] has no
//! variant and `CFG_SHAPE.md` §6.3 declines the discovery that would justify
//! one.
//!
//! ## ✔ 2026-08-15, lane `w-fencea` — **invariant 4 gained an ADMISSION, and the
//! loop bodies came in through it**
//!
//! Board **#3144**: the fence the paragraph above calls *"right to"* refuse was
//! measured at zero cost by `#3089`, stayed literally true, and was made binding
//! by `w-layout`'s own success — it blocked **7 of the 8** residual sites at a
//! counter benefit of **zero**, because four shipped classes emit their back
//! edge through `reach::direct` and the map never sees it. [`LabelMap`]'s
//! `w-fencea` correction has the reading; the mechanism is
//! [`LabelMap::admitting_back_edges`] and [`BodyLayout::admitting_back_edges`],
//! and **[`BodyLayout::new`] is unchanged**, so every client that had a refusal
//! still has one.
//!
//! # The one fact this module does not own
//!
//! **The fixup list is [`super::labels::LabelMap`]'s, not this module's.** That
//! is §6.2 item **B**, board **#290**, and it is the single reader of "a pending
//! intra-section branch site, its target, and whether the reference is legal".
//! Everything that makes a branch correct lives there and is delegated to, never
//! re-derived here:
//!
//! * the **two encodings** ([`labels::Form`], §6.2 item C, board #191) — a `bc`
//!   or an intra-section `b` carries a **true self-relative displacement** and
//!   takes **no relocation**; an external `b`/`bl` carries a section-start
//!   placeholder and a `REL24`. `Form` deliberately has no external variant, and
//!   that is why [`Terminator::TailCall`] is a *different terminator* rather than
//!   a third `Form`: an external tail branch is a relocation, not a label
//!   reference, and this IR keeps it out of the map exactly as #290 does;
//! * the **forward-only rule** (`LabelMap::resolve` invariant 4) — c2 charges the
//!   compiler-label counter +1..+4 for a body with a backward intra-section
//!   branch while `coff::plan_labels` charges 0, so a backward reference is a
//!   wrong `$M` in an obj that still links. A block IR is the obvious place to
//!   "helpfully" relax that; this one does not, and
//!   [`tests::a_backward_branch_is_refused_by_the_label_maps_own_rule`] asserts
//!   the refusal text still comes from `labels.rs`, so a second copy of the rule
//!   cannot appear here without that test going red;
//! * the **displacement range check** (§6.2 item D) and the long-branch
//!   expansion's absence.
//!
//! # Naming, and a collision that was checked for rather than discovered
//!
//! The type is `BasicBlock` and **not** `Block` because [`c2_il::Block`] already
//! exists, is public, is re-exported from `c2_il`'s root, and means something
//! else entirely (the census *blocking record* — where a parse gave up). This
//! module is also **not** glob-re-exported from [`super`], unlike most of its
//! siblings, so nothing here can arrive in a module's ambient scope by accident.
//! Both decisions are frozen in the lane's prereg
//! (`docs/rungs/2026-08-13-ircond.md` §1.3), because this repo has been bitten
//! three times by lanes colliding through *semantics* with no textual conflict.

use super::cond::{bc_reads_crf, cond_source, Cond, CondSource};
use super::encode::{encode_bclr, encode_blr};
use super::labels::{ChargedClass, Form, Label, LabelMap};
use super::select::out_of_class;
use crate::BackendError;

/// The identity of a block within one [`BodyLayout`].
///
/// Opaque and `Copy`. It carries an index into the layout that minted it, so an
/// id from one body used against another is caught by [`BodyLayout::place`]'s
/// and [`BodyLayout::finish`]'s bounds checks rather than silently naming a
/// neighbour's block — the same treatment [`Label`] gets in
/// [`super::labels`], for the same reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlockId(usize);

/// **Why** the blocks are in the order they are in.
///
/// §6.2 item A: *"a lowering must be able to state the order it chose, so the
/// two can be told apart"*. The two are §3.4's measured rule and §3.4.1's single
/// counter-example, and this enum is how a body says which one it is claiming.
///
/// There is exactly one variant, and that is a statement about evidence rather
/// than an oversight. §3.4.1's inverted layout (`?d_join`: c2 tail-merges the
/// arms, hoists the survivor above the compare, and the fall-through becomes the
/// *else*) is downstream of **code motion**, which `CFG_SHAPE.md` §6.3 declines
/// to characterise and this port does not implement; a body whose arms end in
/// the same call is out of class. A second variant would be a name for a layout
/// no lowering here can produce — `cond_tail.rs`'s "a mechanism with no fact
/// behind it" — so it is left for the lane that measures one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockOrder {
    /// **§3.4**: blocks land in `.text` in the order their statements appear in
    /// the `.ex` stream — the condition, the `bc` to the *else entry*, the
    /// then-block, the else-block, the join. Ten cells, every one consistent,
    /// including `?d_cold`, whose then-block is six calls long and still stays
    /// in line as the fall-through.
    IlStatement,
}

/// The one control transfer that ends a [`BasicBlock`].
///
/// All six of §6.2 item A's kinds, and no seventh. Each is spelled with an
/// encoder that already exists in [`super::encode`] and is already byte-graded
/// against real `c2.dll` by a shipped lowering, so none of them is a mechanism
/// invented here:
///
/// | variant | bytes | already graded by |
/// |---|---|---|
/// | [`FallThrough`](Self::FallThrough) | none | every straight-line body |
/// | [`Bc`](Self::Bc) | `encode_bc`, via [`Form::Bc`] | `cond_tail`, `labels` |
/// | [`B`](Self::B) | `encode_b_intra`, via [`Form::B`] | `calls`' early return, `labels` |
/// | [`Bclr`](Self::Bclr) | `encode_bclr` | `pool_free_list`, `counted_accum_loop`, `float_walk_loop` |
/// | [`Blr`](Self::Blr) | `encode_blr` | every leaf |
/// | [`TailCall`](Self::TailCall) | a zero placeholder + a `REL24` the caller writes | `cond_tail`, `Terminator::TailCall` |
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terminator {
    /// Control falls into the next block in emission order. Emits nothing.
    ///
    /// The last block in a body may not carry this — there is nothing for it to
    /// fall into, and a body that ends without a transfer runs off the end of
    /// its own `.text`. [`BodyLayout::finish`] refuses it.
    FallThrough,
    /// `bc` to `taken`; falls through to the next block otherwise.
    ///
    /// `bo`/`bi` are the raw PowerPC fields, exactly as [`Form::Bc`] takes them
    /// — see this module's header on why the condition model is **not** built
    /// here. `bi` is `cr_bi(field, bit)`; `super::encode`'s `CR_COMPARE`,
    /// `BO_TRUE`/`BO_FALSE` and `CR_BIT_*` are the constants that spell it.
    ///
    /// **The sense is the lowering's business, not this IR's.** For an `if`, c2's
    /// `bc` is the edge to the *else* and its condition is the **negation** of
    /// the IL relation (§3.4, §4.2 item 2, ten cells) — that rule lives in
    /// [`super::cond_tail::branch_sense`] and is not restated here, because a
    /// second copy of it is a second thing to keep true.
    Bc { bo: u8, bi: u8, taken: BlockId },
    /// An unconditional **intra-section** `b` to `target`.
    ///
    /// Never an external call: that is [`Self::TailCall`]. The two are the same
    /// opcode with different encodings (§3.3, board #191) and keeping them as
    /// one variant is precisely the corruption #191 names.
    B { target: BlockId },
    /// A conditional return, `bclr`.
    ///
    /// This is `CFG_SHAPE.md` §3.5's **fold band 2**, and `cond_tail.rs` records
    /// that the band is out of the two-arm conditional class. The variant exists
    /// because three shipped lowerings already emit `encode_bclr` inline
    /// (`pool_free_list`, `counted_accum_loop`, `float_walk_loop`) — it is a
    /// terminator this crate demonstrably produces, so spelling it is a
    /// description of what is here, not a bet on what might be.
    Bclr { bo: u8, bi: u8 },
    /// An unconditional return, `blr`.
    Blr,
    /// A tail `b` to an **external** symbol.
    ///
    /// Emits a **zero placeholder word** whose offset [`FinishedBody`] reports,
    /// and takes a `REL24` — neither of which this IR can finish, because the
    /// word encodes its own `.text` offset and the function's placement is the
    /// caller's (`CFG_SHAPE.md` §3.3; `Terminator::TailCall` carries an unfinished
    /// text for the same reason). The callee's *name* is deliberately not
    /// carried: the emitters pair these sites with `IlFunction`'s callees in
    /// block order, and that pairing already has exactly one reader per emitter.
    TailCall,
}

impl Terminator {
    /// A `bc` to `taken` on a **modelled** condition — `CFG_SHAPE.md` §6.2 item
    /// **E**, from the consumer's end.
    ///
    /// The point of the constructor is that `BI` is *derived from the producer*
    /// ([`Cond::bi`]) instead of spelled beside it. `Terminator::Bc`'s raw form
    /// stays public and stays the representation — item A carries `(BO, BI)`,
    /// and this changes none of that — but a lowering that says which
    /// instruction wrote its condition can no longer name a different field by
    /// hand.
    pub fn bc(cond: Cond, taken: BlockId) -> Self {
        Terminator::Bc { bo: cond.bo(), bi: cond.bi(), taken }
    }

    /// A conditional return on a modelled condition. Same derivation; §3.5's
    /// fold band 2.
    pub fn bclr(cond: Cond) -> Self {
        Terminator::Bclr { bo: cond.bo(), bi: cond.bi() }
    }

    /// The condition-register field this terminator **reads**, or `None` if it
    /// reads none.
    ///
    /// Delegates the "`BO` ignores the CR" rule to [`bc_reads_crf`], which is
    /// the one reader of it: `blr` is `bclr` at `BO = 20`, and `BI >> 2 = 0`
    /// there is an artefact rather than a claim that a return reads cr0.
    pub fn reads_crf(self) -> Option<u8> {
        match self {
            Terminator::Bc { bo, bi, .. } | Terminator::Bclr { bo, bi } => bc_reads_crf(bo, bi),
            Terminator::FallThrough
            | Terminator::B { .. }
            | Terminator::Blr
            | Terminator::TailCall => None,
        }
    }
}

/// A straight-line instruction run ending in exactly one [`Terminator`].
///
/// "Exactly one" is enforced by the type rather than by a check: [`Self::term`]
/// is a field, not a list, so a block with two terminators or none is
/// unspellable. `body` holds only instructions that do **not** transfer control
/// — which is what makes a later pass over a block's instruction run (a
/// scheduler, a liveness walk) a well-posed thing to write, and is the property
/// `c2_il::IlFunction::ops` lacks (§6.1 item 2: a flat postfix stream with no
/// terminator concept, into which a branch cannot be added without the stream
/// ceasing to mean "evaluate these in order").
#[derive(Clone, Debug)]
pub struct BasicBlock {
    id: BlockId,
    name: &'static str,
    body: Vec<u8>,
    term: Terminator,
}

impl BasicBlock {
    /// This block's identity in its layout.
    pub fn id(&self) -> BlockId {
        self.id
    }

    /// The block's name. Diagnostic only — it appears in refusal text, and a
    /// refusal that cannot say *which* block it is about is one somebody has to
    /// re-derive.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The straight-line instruction run, big-endian PowerPC words, control
    /// transfers excluded.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// The one terminator.
    pub fn terminator(&self) -> Terminator {
        self.term
    }

    /// **Which instruction in this block's own run wrote the condition its
    /// terminator reads** — `CFG_SHAPE.md` §6.2 item **E**, the producer side.
    ///
    /// This is the accessor item E's demand reduces to once item A exists: the
    /// producer is a property of the straight-line run, and until `body` was a
    /// thing there was nowhere for the question to be asked. Three-valued, and
    /// the three are not interchangeable — see [`CondSource`]. In particular a
    /// producer in a **predecessor** block is legal and reads
    /// [`CondSource::NotInThisBlock`], not an error.
    pub fn cond_source(&self) -> CondSource {
        cond_source(&self.body)
    }
}

/// A finished body: the bytes, the sites the caller still has to fill, and
/// **where every block landed**.
#[derive(Clone, Debug)]
pub struct FinishedBody {
    /// The whole body, with a **zero word** at each [`Terminator::TailCall`]
    /// site and every intra-section branch already patched.
    pub text: Vec<u8>,
    /// Offsets within [`Self::text`] of the tail-call placeholder words, **in
    /// emission order**. Each takes a `REL24` and each encodes its own `.text`
    /// offset, so only the caller can write them.
    pub tail_sites: Vec<u32>,
    /// The order the lowering claimed, travelling with the bytes it produced —
    /// §6.2 item A's "a lowering must be able to state the order it chose".
    pub order: BlockOrder,
    /// Where each **declared** block's run begins in [`Self::text`], and how
    /// long that run is (terminator excluded), indexed by [`BlockId`]. Read
    /// through [`Self::start_of`] / [`Self::at`], never directly: an
    /// out-of-range id must be a refusal and not a panic.
    ///
    /// Every entry is `Some` — [`BodyLayout::finish`] refuses a declared block
    /// that was never placed — but the `Option` is kept rather than collapsed,
    /// because the vector is built as the blocks go down and a `0` for "not yet"
    /// is exactly the legal-looking wrong answer this crate keeps refusing
    /// elsewhere (block 0 really does start at 0).
    starts: Vec<Option<(u32, u32)>>,
}

impl FinishedBody {
    /// **Where `id`'s straight-line run begins** in [`Self::text`] — board
    /// **#3124**'s one fact.
    ///
    /// This is what makes a lowering's published offsets *the layout's* rather
    /// than its own. Before this existed, every emitter here recorded a `bl`'s
    /// `REL24` site, a float constant's `REFHI`/`REFLO` pair and its prologue's
    /// length by reading `t.len()` off a running byte vector, which pins those
    /// positions to a body nothing may insert into — *"a fixed site has nowhere
    /// to grow"*. A position stated as `start_of(block) + k` grows with its
    /// block, and `k` is a constant of **one block's own run** instead of a
    /// constant of the whole body.
    ///
    /// It performs no relaxation and this crate has none: see the module header
    /// and [`super::reach`]. What it does is make one possible.
    pub fn start_of(&self, id: BlockId) -> Result<u32, BackendError> {
        match self.starts.get(id.0).copied().flatten() {
            Some((at, _)) => Ok(at),
            None => Err(out_of_class(
                "the start of a block this body never placed — a block id from a \
                 different function's layout",
            )),
        }
    }

    /// A position **inside** `id`'s own run: [`Self::start_of`] plus `k`, with
    /// `k` checked against the block it claims to be in.
    ///
    /// The check is the reason this exists rather than callers writing the
    /// addition. `k` is a constant of one block's run, so a `k` that reaches
    /// past that run is a lowering naming a position in a block it is not in —
    /// which would still be *some* offset in the body, would still produce a
    /// `REL24` at a plausible word, and would move the moment either block's
    /// length changed. That is the same failure shape as
    /// [`BodyLayout::place`]'s ragged-run refusal, one level up.
    ///
    /// `k == len` is refused with the rest: the position one past a run's end is
    /// the *next* block's start, and it has its own name.
    pub fn at(&self, id: BlockId, k: u32) -> Result<u32, BackendError> {
        let start = self.start_of(id)?;
        let len = self.run_len(id)?;
        if k >= len {
            return Err(out_of_class(&format!(
                "a position {k} bytes into a block whose own run is {len} bytes: \
                 the offset belongs to a different block, and stating it here \
                 would pin it to a body nothing may insert into (board #3124)"
            )));
        }
        Ok(start + k)
    }

    /// The length of `id`'s straight-line run, **terminator excluded** — the
    /// denominator [`Self::at`] checks against.
    ///
    /// The terminator is excluded on purpose. A `bl` inside a block's run and
    /// the block's own `b` to a label are different kinds of site: the first is
    /// a word the lowering owns and patches, the second is
    /// [`super::labels::LabelMap`]'s and the lowering must never touch it. An
    /// `at` that admitted the terminator word would let a lowering name — and
    /// then overwrite — a site the fixup pass had already patched.
    pub fn run_len(&self, id: BlockId) -> Result<u32, BackendError> {
        match self.starts.get(id.0).copied().flatten() {
            Some((_, len)) => Ok(len),
            None => Err(out_of_class(
                "the run length of a block this body never placed — a block id \
                 from a different function's layout",
            )),
        }
    }
}

/// The block/terminator IR for **one function body**, with an explicit emission
/// order.
///
/// Scoped to one body on purpose, exactly as [`LabelMap`] is: a block offset is
/// a `.text`-section offset, the port emits one COMDAT per function, and a
/// layout that outlived a body would hold offsets in two coordinate systems.
pub struct BodyLayout {
    order: BlockOrder,
    /// One entry per **declared** block, indexed by [`BlockId`]: its label in
    /// the shared map, its name, and its position in the emission order once it
    /// has been placed.
    declared: Vec<(Label, &'static str, Option<usize>)>,
    /// The emission order. Position `i` is the `i`th block laid into `.text`.
    placed: Vec<BasicBlock>,
    /// **The one fixup list** — `CFG_SHAPE.md` §6.2 item B, board #290. Not
    /// re-implemented here; see this module's header.
    labels: LabelMap,
}

impl BodyLayout {
    /// A new, empty layout claiming `order`.
    ///
    /// Its map is [`LabelMap::new`]'s, so **a back edge is refused** — invariant
    /// 4, unchanged, and what all nine of the pre-`w-fencea` clients get.
    pub fn new(order: BlockOrder) -> Self {
        Self {
            order,
            declared: Vec::new(),
            placed: Vec::new(),
            labels: LabelMap::new(),
        }
    }

    /// A layout for a body of `class`, whose **back edge** the map will resolve
    /// — `LabelMap::admitting_back_edges`, and nothing else.
    ///
    /// The admission is the map's, passed through. This module deliberately
    /// makes **no** decision about it and holds no copy of the rule: a block IR
    /// is the obvious place to grow a second, friendlier one, and
    /// [`tests::a_backward_branch_is_refused_by_the_label_maps_own_rule`] reads
    /// `labels.rs`' own words so that a second copy cannot appear here without
    /// going red.
    ///
    /// The fence is per **body**, not per site: [`Self::finish`] resolves every
    /// branch through the one map, which is why `ptr_walk_loop`'s *forward*
    /// entry guard was fenced off by its own body's back edge (board **#3144**).
    /// That is exactly why the admission belongs on the layout's constructor and
    /// not on a terminator.
    pub fn admitting_back_edges(order: BlockOrder, class: ChargedClass) -> Self {
        Self {
            order,
            declared: Vec::new(),
            placed: Vec::new(),
            labels: LabelMap::admitting_back_edges(class),
        }
    }

    /// The order this layout claims.
    pub fn order(&self) -> BlockOrder {
        self.order
    }

    /// The blocks laid down so far, **in emission order**.
    ///
    /// This is the accessor a later pass reads: item E's condition-code model,
    /// item F's cross-block liveness and item G's shape check all need to walk
    /// the blocks of a body, and none of them has anywhere to stand today.
    pub fn blocks(&self) -> &[BasicBlock] {
        &self.placed
    }

    /// Mint a block identity. Declares that the block *exists*; says nothing
    /// about where it goes or what is in it.
    ///
    /// Two phases rather than one because a terminator names its target and a
    /// forward target has not been placed yet — which is item B's whole premise
    /// (`3A`/`38`/`39` carry no direction, so the target's offset is unknown when
    /// the branch is emitted) restated one level up.
    ///
    /// `name` appears in refusal text only.
    pub fn declare(&mut self, name: &'static str) -> BlockId {
        let label = self.labels.mint(name);
        self.declared.push((label, name, None));
        BlockId(self.declared.len() - 1)
    }

    /// Append `id` to the emission order with its instruction run and its one
    /// terminator.
    ///
    /// Refuses rather than overwrites, on four counts, each of which would
    /// otherwise be a legal-looking body in the wrong shape:
    ///
    /// 1. an `id` this layout never declared — the cross-body case;
    /// 2. a **second** placement of one block — two positions claiming one
    ///    block, which would make "emission order" ambiguous and silently give
    ///    every branch to it one of two answers;
    /// 3. a `body` that is not a whole number of 4-byte words — PowerPC
    ///    instructions are words, and a misaligned run puts every later block,
    ///    every displacement and every relocation offset out by the remainder;
    /// 4. a terminator that reads a **condition-register field this block's own
    ///    run did not write** — `CFG_SHAPE.md` §6.2 item **E**, added by lane
    ///    `w-ir-e`. See the check itself for why it fires on a positive
    ///    disagreement only.
    pub fn place(
        &mut self,
        id: BlockId,
        body: Vec<u8>,
        term: Terminator,
    ) -> Result<(), BackendError> {
        let pos = self.placed.len();
        let (_, name, placed_at) = self
            .declared
            .get_mut(id.0)
            .ok_or_else(|| out_of_class("a block id from a different function's layout"))?;
        let name = *name;
        if let Some(prev) = *placed_at {
            return Err(out_of_class(&format!(
                "block `{name}` placed twice, at positions {prev} and {pos}: two \
                 positions claiming one block makes the emission order ambiguous, \
                 which is the one property CFG_SHAPE.md §6.2 item A requires to be \
                 explicit"
            )));
        }
        if body.len() % 4 != 0 {
            return Err(out_of_class(&format!(
                "block `{name}` holds {} bytes, which is not a whole number of \
                 PowerPC words: every later block, displacement and relocation \
                 offset would be out by the remainder",
                body.len()
            )));
        }
        // ---- item E: the branch must read the field its own run WROTE -------
        //
        // §3.2's hazard, made structural rather than left to a comment. A block
        // whose run ends in `addic.` (cr0) under a terminator spelled
        // `cr_bi(CR_COMPARE, bit)` emits `409a…` where the obj has `4082…` —
        // two bytes, in a word that still disassembles to a plausible branch,
        // which is board **#188** and the fuzzy-invisible class
        // `docs/CODEGEN_PPC_MVP.md` warns about.
        //
        // It fires only on a **positive disagreement**. `NotInThisBlock` is
        // legal and common (the producer is in a predecessor), and `Unknown` is
        // not an accusation — the two are distinct answers in
        // [`super::cond::CondSource`] precisely so that this check cannot turn
        // "I could not read the run" into "the run is wrong".
        if let Some(read) = term.reads_crf() {
            if let CondSource::InBlock(p) = cond_source(&body) {
                if p.crf() != read {
                    return Err(out_of_class(&format!(
                        "block `{name}`'s branch reads cr{read}, but the last \
                         condition-register writer in its own instruction run is \
                         {} writing cr{}: CFG_SHAPE.md §3.2's two producers, and \
                         a `BI` of {} where this block's own bytes want {} — \
                         board #188, a legal-looking branch on a bit nothing set",
                        p.what(),
                        p.crf(),
                        4 * read,
                        4 * p.crf(),
                    )));
                }
            }
        }

        *placed_at = Some(pos);
        self.placed.push(BasicBlock { id, name, body, term });
        Ok(())
    }

    /// Lay every placed block down in emission order, resolve the branches, and
    /// consume the layout.
    ///
    /// The branch resolution is [`LabelMap::resolve`]'s, called once, at the
    /// end, when every offset is known — this method contributes no displacement
    /// arithmetic of its own. Its own three checks are the ones the map cannot
    /// make because they are about *blocks*:
    ///
    /// 1. **The body has at least one block.** An empty body is not a body, and
    ///    an empty `.text` for a function is a shape no oracle byte backs.
    /// 2. **Every declared block was placed.** A declared-and-unplaced block is
    ///    a lowering that built a target and then forgot to emit it; the map
    ///    would catch it only if something happened to branch there, and would
    ///    say "no block defined this label" rather than naming the omission.
    /// 3. **The last block does not fall through.** There is nothing after it.
    ///
    /// Every failure is an ordinary `Err` and never a panic: the port must
    /// degrade to `NotImplemented` honestly, and a `debug_assert` is compiled
    /// out of the release build the gate actually runs.
    pub fn finish(mut self) -> Result<FinishedBody, BackendError> {
        if self.placed.is_empty() {
            return Err(out_of_class(
                "a body with no basic blocks: an empty .text for a function is a \
                 shape no measured byte backs",
            ));
        }
        for (_, name, placed_at) in &self.declared {
            if placed_at.is_none() {
                return Err(out_of_class(&format!(
                    "block `{name}` was declared and never placed: it has an \
                     identity and no position in the emission order"
                )));
            }
        }

        let last = self.placed.len() - 1;
        let mut text: Vec<u8> = Vec::new();
        let mut tail_sites: Vec<u32> = Vec::new();
        let mut starts: Vec<Option<(u32, u32)>> = vec![None; self.declared.len()];

        for (pos, blk) in self.placed.iter().enumerate() {
            // The label binds to where the block STARTS, so it is defined before
            // the block's own bytes go down.
            let (label, ..) = self.declared[blk.id.0];
            self.labels.define(label, &text)?;
            // …and so does the answer `FinishedBody::start_of` gives, from the
            // same number, at the same moment. Board #3124: a lowering's
            // published offsets are the LAYOUT's, so they can only ever be this
            // one — not a second count kept beside it.
            starts[blk.id.0] = Some((text.len() as u32, blk.body.len() as u32));
            text.extend_from_slice(&blk.body);
            match blk.term {
                Terminator::FallThrough => {
                    if pos == last {
                        return Err(out_of_class(&format!(
                            "the last block `{}` falls through: there is nothing \
                             after it, so control would run off the end of the \
                             function's own .text",
                            blk.name
                        )));
                    }
                }
                Terminator::Bc { bo, bi, taken } => {
                    let target = self.label_of(taken)?;
                    self.labels.reference(&mut text, target, Form::Bc { bo, bi });
                }
                Terminator::B { target } => {
                    let target = self.label_of(target)?;
                    self.labels.reference(&mut text, target, Form::B);
                }
                Terminator::Bclr { bo, bi } => text.extend_from_slice(&encode_bclr(bo, bi)),
                Terminator::Blr => text.extend_from_slice(&encode_blr()),
                Terminator::TailCall => {
                    tail_sites.push(text.len() as u32);
                    text.extend_from_slice(&[0; 4]);
                }
            }
        }

        self.labels.resolve(&mut text)?;
        Ok(FinishedBody { text, tail_sites, order: self.order, starts })
    }

    /// The label a terminator's target names, or a refusal if the id is not
    /// this layout's. Never indexes without checking — an out-of-range id here
    /// would otherwise panic in a release build.
    fn label_of(&self, id: BlockId) -> Result<Label, BackendError> {
        self.declared
            .get(id.0)
            .map(|(label, ..)| *label)
            .ok_or_else(|| out_of_class("a branch to a block id from a different function's layout"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::cond::CondProducer;
    use crate::codegen::encode::{
        cr_bi, encode_bctrl, BO_ALWAYS, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_COMPARE,
    };

    const MR_R11_R4: [u8; 4] = [0x7c, 0x8b, 0x23, 0x78];
    /// `addic. r11,r11,-1` — `?c_do`'s loop counter, §3.2's record-form row.
    const ADDIC_R11_M1: [u8; 4] = [0x35, 0x6b, 0xff, 0xff];
    const CMPLWI_CR6_R3_0: [u8; 4] = [0x2b, 0x03, 0x00, 0x00];
    const MR_R4_R5: [u8; 4] = [0x7c, 0xa4, 0x2b, 0x78];
    const MR_R3_R11: [u8; 4] = [0x7d, 0x63, 0x5b, 0x78];
    const MR_R5_R11: [u8; 4] = [0x7d, 0x65, 0x5b, 0x78];
    const LI_R4_0: [u8; 4] = [0x38, 0x80, 0x00, 0x00];

    fn bytes(runs: &[[u8; 4]]) -> Vec<u8> {
        runs.iter().flatten().copied().collect()
    }

    /// **The known-answer control, at the IR's own level.** `?MemFree`, the
    /// thirty-six bytes `docs/CFG_SHAPE.md` §4.1 read off the real obj, built
    /// out of three [`BasicBlock`]s instead of out of a running byte vector.
    ///
    /// This is the assertion the whole module exists to satisfy: the `bc` at
    /// 0x08 must come out `40 9a 00 10`, which is a displacement of +16 that
    /// **nothing in this file computes** — [`LabelMap`] derives it from where
    /// the else block landed.
    #[test]
    fn the_memfree_shape_lays_out_to_the_bytes_the_real_obj_carries() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let entry = l.declare("entry");
        let then = l.declare("then");
        let els = l.declare("else");

        l.place(
            entry,
            bytes(&[MR_R11_R4, CMPLWI_CR6_R3_0]),
            Terminator::Bc {
                bo: BO_FALSE,
                bi: cr_bi(CR_COMPARE, CR_BIT_EQ),
                taken: els,
            },
        )
        .unwrap();
        l.place(then, bytes(&[MR_R4_R5, MR_R3_R11]), Terminator::TailCall).unwrap();
        l.place(els, bytes(&[MR_R5_R11, LI_R4_0]), Terminator::TailCall).unwrap();

        assert_eq!(l.blocks().len(), 3);
        let body = l.finish().unwrap();
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            0x7c, 0x8b, 0x23, 0x78, // mr     r11,r4
            0x2b, 0x03, 0x00, 0x00, // cmplwi cr6,r3,0
            0x40, 0x9a, 0x00, 0x10, // bne    cr6,+16   <- resolved by the label map
            0x7c, 0xa4, 0x2b, 0x78, // mr     r4,r5
            0x7d, 0x63, 0x5b, 0x78, // mr     r3,r11
            0x00, 0x00, 0x00, 0x00, // b      XMemFree     <- the caller's
            0x7d, 0x65, 0x5b, 0x78, // mr     r5,r11
            0x38, 0x80, 0x00, 0x00, // li     r4,0
            0x00, 0x00, 0x00, 0x00, // b      RtlFreeHeap  <- the caller's
        ];
        assert_eq!(body.text, want);
        assert_eq!(body.text.len(), 0x24);
        assert_eq!(body.tail_sites, vec![0x14, 0x20]);
        assert_eq!(body.order, BlockOrder::IlStatement);
    }

    /// **Emission order is placement order, and it is not declaration order.**
    ///
    /// Three blocks declared `entry`, `a`, `b` and placed `entry`, `b`, `a`. The
    /// arms come out in **placement** order, and the `bc`'s displacement — which
    /// nothing here computes — follows the placement too: `a` is the branch
    /// target and lands last, so the branch is +12 and not the +4 it would be if
    /// the builder had inferred the order from the declarations.
    ///
    /// This is the positive demonstration of §6.2 item A's requirement that the
    /// order be a property the lowering *states*.
    #[test]
    fn emission_order_is_placement_order_not_declaration_order() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let entry = l.declare("entry");
        let a = l.declare("a");
        let b = l.declare("b");
        l.place(
            entry,
            bytes(&[CMPLWI_CR6_R3_0]),
            Terminator::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: a },
        )
        .unwrap();
        l.place(b, bytes(&[MR_R4_R5]), Terminator::TailCall).unwrap();
        l.place(a, bytes(&[MR_R3_R11]), Terminator::TailCall).unwrap();
        let body = l.finish().unwrap();
        // `bc` sits at 4; `a` was placed last and starts at 16. +12.
        assert_eq!(&body.text[4..8], &[0x40, 0x9a, 0x00, 0x0c]);
        // …and the arms are in the order they were PLACED, not declared.
        assert_eq!(&body.text[8..12], &MR_R4_R5);
        assert_eq!(&body.text[16..20], &MR_R3_R11);
        assert_eq!(body.tail_sites, vec![12, 20]);
    }

    /// A fall-through emits **no bytes** and lets the next block start
    /// immediately — the thing that makes it a terminator at all rather than an
    /// absence.
    #[test]
    fn a_fall_through_emits_no_bytes_and_the_next_block_starts_immediately() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("a");
        let b = l.declare("b");
        l.place(a, bytes(&[MR_R4_R5]), Terminator::FallThrough).unwrap();
        l.place(b, bytes(&[MR_R3_R11]), Terminator::Blr).unwrap();
        let body = l.finish().unwrap();
        assert_eq!(body.text.len(), 12);
        assert_eq!(&body.text[0..4], &MR_R4_R5);
        assert_eq!(&body.text[4..8], &MR_R3_R11);
        assert_eq!(&body.text[8..12], &[0x4e, 0x80, 0x00, 0x20]); // blr
    }

    /// An unconditional intra-section `b` carries its **true self-relative
    /// displacement** and takes no relocation — the same map, the other
    /// [`Form`].
    #[test]
    fn an_intra_section_b_is_patched_with_its_true_displacement() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("a");
        let skipped = l.declare("skipped");
        let join = l.declare("join");
        l.place(a, bytes(&[MR_R4_R5]), Terminator::B { target: join }).unwrap();
        l.place(skipped, bytes(&[MR_R3_R11, MR_R5_R11]), Terminator::FallThrough).unwrap();
        l.place(join, bytes(&[LI_R4_0]), Terminator::Blr).unwrap();
        let body = l.finish().unwrap();
        // The `b` sits at 4 and `join` lands at 16: +12.
        assert_eq!(&body.text[4..8], &[0x48, 0x00, 0x00, 0x0c]);
        assert_eq!(body.tail_sites, Vec::<u32>::new());
    }

    /// `bclr` — §3.5's fold band 2, spelled as a terminator.
    #[test]
    fn a_bclr_terminator_emits_the_conditional_return() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("guard");
        let b = l.declare("rest");
        l.place(
            a,
            bytes(&[CMPLWI_CR6_R3_0]),
            Terminator::Bclr { bo: BO_TRUE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ) },
        )
        .unwrap();
        l.place(b, Vec::new(), Terminator::Blr).unwrap();
        let body = l.finish().unwrap();
        assert_eq!(&body.text[4..8], &encode_bclr(BO_TRUE, cr_bi(CR_COMPARE, CR_BIT_EQ)));
        assert_eq!(body.text.len(), 12);
    }

    /// A tail call leaves a **zero** word and reports its offset — it is not a
    /// label reference and it never enters the map, which is board #191's rule
    /// made structural.
    #[test]
    fn a_tail_call_leaves_a_zero_word_and_reports_its_offset() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("entry");
        l.place(a, bytes(&[MR_R3_R11]), Terminator::TailCall).unwrap();
        let body = l.finish().unwrap();
        assert_eq!(body.tail_sites, vec![4]);
        assert_eq!(&body.text[4..8], &[0, 0, 0, 0]);
    }

    // ---- the refusals. Each is exercised POSITIVELY: the guard is made to
    // fire, and the message it produced is read. A guard nobody has seen fire is
    // a guard nobody has tested.

    #[test]
    fn a_last_block_that_falls_through_is_refused_and_names_itself() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("a");
        let tail = l.declare("the-tail");
        l.place(a, bytes(&[MR_R4_R5]), Terminator::FallThrough).unwrap();
        l.place(tail, bytes(&[MR_R3_R11]), Terminator::FallThrough).unwrap();
        let s = format!("{:?}", l.finish().unwrap_err());
        assert!(s.contains("the-tail"), "{s}");
        assert!(s.contains("falls through"), "{s}");
    }

    #[test]
    fn a_declared_but_unplaced_block_is_refused_and_names_itself() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("entry");
        let _ghost = l.declare("the-ghost");
        l.place(a, bytes(&[MR_R3_R11]), Terminator::Blr).unwrap();
        let s = format!("{:?}", l.finish().unwrap_err());
        assert!(s.contains("the-ghost"), "{s}");
        assert!(s.contains("never placed"), "{s}");
    }

    #[test]
    fn placing_one_block_twice_is_refused_and_names_both_positions() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("twice");
        l.place(a, bytes(&[MR_R3_R11]), Terminator::Blr).unwrap();
        let s = format!("{:?}", l.place(a, bytes(&[MR_R4_R5]), Terminator::Blr).unwrap_err());
        assert!(s.contains("twice"), "{s}");
        assert!(s.contains("positions 0 and 1"), "{s}");
    }

    #[test]
    fn a_block_that_is_not_a_whole_number_of_words_is_refused() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("ragged");
        let s = format!(
            "{:?}",
            l.place(a, vec![0x38, 0x60, 0x00], Terminator::Blr).unwrap_err()
        );
        assert!(s.contains("ragged"), "{s}");
        assert!(s.contains("3 bytes"), "{s}");
    }

    #[test]
    fn a_body_with_no_blocks_is_refused() {
        let l = BodyLayout::new(BlockOrder::IlStatement);
        let s = format!("{:?}", l.finish().unwrap_err());
        assert!(s.contains("no basic blocks"), "{s}");
    }

    /// A `BlockId` from another body is caught on both paths — placing it, and
    /// branching to it — rather than naming a neighbour's block.
    #[test]
    fn a_block_id_from_another_layout_is_refused_on_both_paths() {
        let mut other = BodyLayout::new(BlockOrder::IlStatement);
        let _ = other.declare("o0");
        let stray = other.declare("o1");

        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let s = format!("{:?}", l.place(stray, Vec::new(), Terminator::Blr).unwrap_err());
        assert!(s.contains("different function's layout"), "{s}");

        let mut l2 = BodyLayout::new(BlockOrder::IlStatement);
        let a = l2.declare("a");
        l2.place(a, Vec::new(), Terminator::B { target: stray }).unwrap();
        let s2 = format!("{:?}", l2.finish().unwrap_err());
        assert!(s2.contains("different function's layout"), "{s2}");
    }

    /// **The backward-branch refusal is [`LabelMap`]'s, and this asserts it is
    /// still the one that fires.**
    ///
    /// A block IR is the obvious place to grow a second, friendlier copy of this
    /// rule. If one ever appears, this test goes red, because it reads the
    /// message for `labels.rs`'s own words — the counter and `plan_labels` — and
    /// not merely for "backward".
    #[test]
    fn a_backward_branch_is_refused_by_the_label_maps_own_rule() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let top = l.declare("loop-top");
        let back = l.declare("back-edge");
        l.place(top, bytes(&[MR_R4_R5]), Terminator::FallThrough).unwrap();
        l.place(
            back,
            bytes(&[CMPLWI_CR6_R3_0]),
            Terminator::Bc { bo: BO_TRUE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: top },
        )
        .unwrap();
        let s = format!("{:?}", l.finish().unwrap_err());
        assert!(s.contains("BACKWARD"), "{s}");
        assert!(s.contains("loop-top"), "{s}");
        assert!(s.contains("plan_labels"), "{s}");
    }

    /// The displacement range check (§6.2 item D) reaches through this IR
    /// unaltered, and the long-branch expansion is still named as not built.
    #[test]
    fn a_bc_past_its_field_is_refused_through_the_same_map() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let entry = l.declare("entry");
        let bulk = l.declare("bulk");
        let far = l.declare("far");
        l.place(
            entry,
            Vec::new(),
            Terminator::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: far },
        )
        .unwrap();
        // 40,000 bytes of `ori r0,r0,0` between the branch and its target: well
        // past the 14-bit `BD` field, and comfortably inside `LI`'s 24-bit one,
        // so this is a statement about the FIELD and not about the length.
        l.place(bulk, vec![0x60; 40_000], Terminator::FallThrough).unwrap();
        l.place(far, Vec::new(), Terminator::Blr).unwrap();
        let s = format!("{:?}", l.finish().unwrap_err());
        assert!(s.contains("displacement field"), "{s}");
        assert!(s.contains("3.3.1"), "{s}");
    }

    /// A positive enumeration: the IR spells **all six** of §6.2 item A's
    /// terminator kinds, and one body exercises every one of them. Written as a
    /// count so that deleting a variant fails here rather than quietly narrowing
    /// what "item A" means.
    #[test]
    fn the_ir_spells_all_six_of_item_as_terminator_kinds() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let b0 = l.declare("fallthrough");
        let b1 = l.declare("cond");
        let b2 = l.declare("uncond");
        let b3 = l.declare("condret");
        let b4 = l.declare("ret");
        let b5 = l.declare("tail");
        l.place(b0, Vec::new(), Terminator::FallThrough).unwrap();
        l.place(
            b1,
            Vec::new(),
            Terminator::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: b4 },
        )
        .unwrap();
        l.place(b2, Vec::new(), Terminator::B { target: b3 }).unwrap();
        l.place(b3, Vec::new(), Terminator::Bclr { bo: BO_TRUE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ) })
            .unwrap();
        l.place(b4, Vec::new(), Terminator::Blr).unwrap();
        l.place(b5, Vec::new(), Terminator::TailCall).unwrap();

        let kinds: Vec<Terminator> = l.blocks().iter().map(|b| b.terminator()).collect();
        assert_eq!(kinds.len(), 6);
        assert!(matches!(kinds[0], Terminator::FallThrough));
        assert!(matches!(kinds[1], Terminator::Bc { .. }));
        assert!(matches!(kinds[2], Terminator::B { .. }));
        assert!(matches!(kinds[3], Terminator::Bclr { .. }));
        assert!(matches!(kinds[4], Terminator::Blr));
        assert!(matches!(kinds[5], Terminator::TailCall));

        let body = l.finish().unwrap();
        // 4 words of control transfer: bc, b, bclr, blr, and the tail
        // placeholder. The fall-through contributes nothing, which is what makes
        // the total 5 and not 6.
        assert_eq!(body.text.len(), 20);
        assert_eq!(body.tail_sites, vec![16]);
    }

    // ---- item E, the producer side (lane `w-ir-e`) -----------------------

    /// **The refusal fires, and it names both fields.** A block whose run ends
    /// in `addic.` — a record form, cr0 — under a branch spelled with the
    /// compare's `BI` is exactly board #188's defect, and it is refused here
    /// rather than emitted.
    #[test]
    fn a_branch_reading_a_field_its_own_run_did_not_write_is_refused() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let top = l.declare("loop-body");
        let out = l.declare("out");
        let s = format!(
            "{:?}",
            l.place(
                top,
                bytes(&[ADDIC_R11_M1]),
                // The record form wrote cr0; this reads cr6.
                Terminator::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: out },
            )
            .unwrap_err()
        );
        assert!(s.contains("loop-body"), "{s}");
        assert!(s.contains("reads cr6"), "{s}");
        assert!(s.contains("a record form writing cr0"), "{s}");
        assert!(s.contains("#188"), "{s}");
    }

    /// **The control for the refusal above**: the identical block with the `BI`
    /// its own run wrote is accepted, and lays down `?c_do`'s real obj word.
    #[test]
    fn the_same_block_with_the_field_its_run_wrote_is_accepted_and_emits_the_obj_word() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let top = l.declare("loop-top");
        let body = l.declare("loop-body");
        let out = l.declare("out");
        l.place(top, Vec::new(), Terminator::FallThrough).unwrap();
        l.place(
            body,
            bytes(&[ADDIC_R11_M1]),
            Terminator::bc(Cond::record_form(BO_FALSE, CR_BIT_EQ), out),
        )
        .unwrap();
        l.place(out, Vec::new(), Terminator::Blr).unwrap();
        let b = l.finish().unwrap();
        // `bne cr0,+4` — the `?c_do` word at this displacement. The BI byte is
        // 0x82 and not 0x9a, which is the whole of §3.2.
        assert_eq!(&b.text[4..8], &[0x40, 0x82, 0x00, 0x04]);
        assert_eq!(b.text.len(), 12);
    }

    /// A producer in a **predecessor** block is legal, and the check does not
    /// invent an accusation out of it. `?d_join`'s shape, and every guard chain
    /// that compares once and branches twice.
    #[test]
    fn a_producer_in_a_predecessor_block_is_not_an_accusation() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let cmp = l.declare("compare");
        let mid = l.declare("middle");
        let out = l.declare("out");
        l.place(cmp, bytes(&[CMPLWI_CR6_R3_0]), Terminator::FallThrough).unwrap();
        // No CR writer of its own: the answer is NotInThisBlock, positively.
        l.place(
            mid,
            bytes(&[MR_R4_R5]),
            Terminator::bc(Cond::compare(BO_FALSE, CR_BIT_EQ), out),
        )
        .unwrap();
        l.place(out, Vec::new(), Terminator::Blr).unwrap();
        assert_eq!(l.blocks()[1].cond_source(), CondSource::NotInThisBlock);
        assert_eq!(l.finish().unwrap().text.len(), 16);
    }

    /// **An unmodelled word does not manufacture a refusal.** A block holding a
    /// call has no readable producer — the volatile CR fields do not survive one
    /// — and `Unknown` is not `NotInThisBlock` and is certainly not "wrong".
    #[test]
    fn an_unmodelled_word_leaves_the_question_open_rather_than_refusing() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("after-a-call");
        let out = l.declare("out");
        let mut run = bytes(&[CMPLWI_CR6_R3_0]);
        run.extend_from_slice(&encode_bctrl());
        l.place(
            a,
            run,
            // Reads cr0 after a call: this model cannot say it is wrong, and
            // says so rather than guessing either way.
            Terminator::Bc { bo: BO_FALSE, bi: cr_bi(0, CR_BIT_EQ), taken: out },
        )
        .unwrap();
        l.place(out, Vec::new(), Terminator::Blr).unwrap();
        assert_eq!(l.blocks()[0].cond_source(), CondSource::Unknown);
        assert_ne!(l.blocks()[0].cond_source(), CondSource::NotInThisBlock);
    }

    /// The two constructors derive `(BO, BI)` from the producer, and the two
    /// producers give the two `BI`s §3.2 measures — 26 and 2 — from the same
    /// `(BO, bit)`.
    #[test]
    fn the_terminator_constructors_derive_bo_and_bi_from_the_producer() {
        let out = BodyLayout::new(BlockOrder::IlStatement).declare("x");
        let from_compare = Terminator::bc(Cond::compare(BO_FALSE, CR_BIT_EQ), out);
        let from_record = Terminator::bc(Cond::record_form(BO_FALSE, CR_BIT_EQ), out);
        assert!(matches!(from_compare, Terminator::Bc { bo: 4, bi: 26, .. }));
        assert!(matches!(from_record, Terminator::Bc { bo: 4, bi: 2, .. }));
        assert_eq!(from_compare.reads_crf(), Some(6));
        assert_eq!(from_record.reads_crf(), Some(0));
        assert_eq!(
            Terminator::bclr(Cond::compare(BO_TRUE, CR_BIT_EQ)),
            Terminator::Bclr { bo: BO_TRUE, bi: 26 }
        );
    }

    /// A terminator that transfers control without consulting the condition
    /// register reads **no** field — `blr` included, whose `BI` is 0 and whose
    /// `BO` ignores it.
    #[test]
    fn the_terminators_that_read_no_condition_register_say_none() {
        let out = BodyLayout::new(BlockOrder::IlStatement).declare("x");
        assert_eq!(Terminator::Blr.reads_crf(), None);
        assert_eq!(Terminator::FallThrough.reads_crf(), None);
        assert_eq!(Terminator::TailCall.reads_crf(), None);
        assert_eq!(Terminator::B { target: out }.reads_crf(), None);
        assert_eq!(Terminator::Bclr { bo: BO_ALWAYS, bi: 0 }.reads_crf(), None);
        assert_eq!(Terminator::Bclr { bo: BO_TRUE, bi: 26 }.reads_crf(), Some(6));
    }

    /// A block reports the producer in **its own** run, and the scan is
    /// backwards: `?b_ifn`'s three compares in one body, each consumed by its
    /// own branch.
    #[test]
    fn a_block_reports_the_producer_in_its_own_run() {
        let mut l2 = BodyLayout::new(BlockOrder::IlStatement);
        let b = l2.declare("entry");
        let c = l2.declare("out");
        l2.place(
            b,
            bytes(&[CMPLWI_CR6_R3_0, MR_R4_R5, ADDIC_R11_M1]),
            Terminator::bc(Cond::record_form(BO_FALSE, CR_BIT_EQ), c),
        )
        .unwrap();
        l2.place(c, Vec::new(), Terminator::Blr).unwrap();
        assert_eq!(
            l2.blocks()[0].cond_source(),
            CondSource::InBlock(CondProducer::RecordForm)
        );
    }

    // ---- board #3124: the layout owns the POSITIONS ----------------------

    /// **The one fact, positively.** `?MemFree`'s three blocks land at 0, 12 and
    /// 24 — the offsets `CFG_SHAPE.md` §4.1's published bytes put them at — and
    /// the finished body says so rather than the lowering having counted.
    ///
    /// 12 and 24 are not 8 and 16: the `bc` and the two tail placeholders are
    /// terminator words and they are *in* the text, so a lowering that added up
    /// its own `body` lengths would be two words low at the third block. That is
    /// the arithmetic board #3124 is about.
    #[test]
    fn a_finished_body_says_where_every_block_landed() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let entry = l.declare("entry");
        let then = l.declare("then");
        let els = l.declare("else");
        l.place(
            entry,
            bytes(&[MR_R11_R4, CMPLWI_CR6_R3_0]),
            Terminator::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: els },
        )
        .unwrap();
        l.place(then, bytes(&[MR_R4_R5, MR_R3_R11]), Terminator::TailCall).unwrap();
        l.place(els, bytes(&[MR_R5_R11, LI_R4_0]), Terminator::TailCall).unwrap();
        let b = l.finish().unwrap();
        assert_eq!(b.start_of(entry).unwrap(), 0);
        assert_eq!(b.start_of(then).unwrap(), 0x0c);
        assert_eq!(b.start_of(els).unwrap(), 0x18);
        // …and each run is its own bytes, terminator excluded.
        assert_eq!(b.run_len(entry).unwrap(), 8);
        assert_eq!(b.run_len(els).unwrap(), 8);
        // The tail placeholders sit at the end of each arm's run.
        assert_eq!(b.tail_sites, vec![b.start_of(then).unwrap() + 8, b.start_of(els).unwrap() + 8]);
    }

    /// **A position stated as `start_of(block) + k` follows the block, and a
    /// constant of the whole body does not.** Board #3124 in one assertion: the
    /// same lowering, the same `k`, one arm four bytes longer, and the published
    /// site moves by four without anything being edited.
    #[test]
    fn a_block_relative_position_moves_with_its_block_and_a_body_constant_does_not() {
        let build = |pad: usize| {
            let mut l = BodyLayout::new(BlockOrder::IlStatement);
            let entry = l.declare("entry");
            let arm = l.declare("arm");
            let out = l.declare("out");
            let mut run = bytes(&[CMPLWI_CR6_R3_0]);
            run.extend(std::iter::repeat(MR_R4_R5).take(pad).flatten());
            l.place(
                entry,
                run,
                Terminator::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: out },
            )
            .unwrap();
            // `k = 4`: the SECOND word of the arm's own run, wherever the arm is.
            l.place(arm, bytes(&[MR_R3_R11, MR_R5_R11]), Terminator::FallThrough).unwrap();
            l.place(out, Vec::new(), Terminator::Blr).unwrap();
            let b = l.finish().unwrap();
            (b.start_of(arm).unwrap(), b.at(arm, 4).unwrap())
        };
        assert_eq!(build(0), (8, 12));
        assert_eq!(build(1), (12, 16));
    }

    /// `at` refuses a `k` that reaches out of the block it names — including
    /// exactly one past the end, which is the *next* block's start and has its
    /// own name.
    #[test]
    fn a_position_past_its_own_blocks_run_is_refused_and_names_both_numbers() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("entry");
        let b = l.declare("out");
        l.place(a, bytes(&[MR_R4_R5, MR_R3_R11]), Terminator::FallThrough).unwrap();
        l.place(b, bytes(&[MR_R5_R11]), Terminator::Blr).unwrap();
        let body = l.finish().unwrap();
        assert_eq!(body.at(a, 0).unwrap(), 0);
        assert_eq!(body.at(a, 4).unwrap(), 4);
        let s = format!("{:?}", body.at(a, 8).unwrap_err());
        assert!(s.contains("8 bytes into a block whose own run is 8 bytes"), "{s}");
        assert!(s.contains("#3124"), "{s}");
    }

    /// **The terminator word is not `at`-addressable**, and that is the rule and
    /// not an off-by-one: the terminator is the fixup pass's site, and a
    /// lowering that could name it could overwrite a branch `LabelMap` had
    /// already patched.
    #[test]
    fn the_terminator_word_is_outside_the_run_a_lowering_may_name() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("guard");
        let b = l.declare("out");
        l.place(
            a,
            bytes(&[CMPLWI_CR6_R3_0]),
            Terminator::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_EQ), taken: b },
        )
        .unwrap();
        l.place(b, Vec::new(), Terminator::Blr).unwrap();
        let body = l.finish().unwrap();
        // The `bc` is at 4 and the block starts at 0, so `k = 4` names it…
        assert_eq!(body.run_len(a).unwrap(), 4);
        assert!(body.at(a, 4).is_err());
        // …and the next block starts past it, which is where 8 lives.
        assert_eq!(body.start_of(b).unwrap(), 8);
    }

    /// A `BlockId` from another body is a refusal on the position accessors too,
    /// and never a neighbour's offset.
    #[test]
    fn the_position_accessors_refuse_a_block_id_from_another_layout() {
        let mut other = BodyLayout::new(BlockOrder::IlStatement);
        let _ = other.declare("o0");
        let stray = other.declare("o1");
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("entry");
        l.place(a, Vec::new(), Terminator::Blr).unwrap();
        let body = l.finish().unwrap();
        for s in [
            format!("{:?}", body.start_of(stray).unwrap_err()),
            format!("{:?}", body.at(stray, 0).unwrap_err()),
            format!("{:?}", body.run_len(stray).unwrap_err()),
        ] {
            assert!(s.contains("different function's layout"), "{s}");
        }
    }

    /// A block's accessors report what was placed — the surface a later pass
    /// (item E, F or G) reads.
    #[test]
    fn a_placed_block_reports_its_name_body_and_terminator() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let a = l.declare("entry");
        l.place(a, bytes(&[MR_R3_R11]), Terminator::Blr).unwrap();
        let blk = &l.blocks()[0];
        assert_eq!(blk.name(), "entry");
        assert_eq!(blk.id(), a);
        assert_eq!(blk.body(), &MR_R3_R11[..]);
        assert_eq!(blk.terminator(), Terminator::Blr);
        assert_eq!(l.order(), BlockOrder::IlStatement);
    }
}

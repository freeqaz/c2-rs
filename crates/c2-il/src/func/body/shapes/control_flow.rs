//! **The control-flow statement layer — DECODE ONLY.**
//!
//! Nothing in this file can make a function in class. It adds no arm to
//! [`super::super::parse_segment_shape`]'s ladder, it constructs no
//! [`BodyShape`](super::super::BodyShape), and every body it understands still
//! refuses. What it adds is a **measurement**: given a function segment, walk the
//! whole body as the flat statement token stream of `docs/IL_STMT_GRAMMAR.md` §0
//! and say which control-flow shape it is — or which byte stopped the walk.
//!
//! ## Why decode without lowering
//!
//! `docs/ARCHITECTURE_SEAMS.md` §7: lowering control flow forces a real block IR
//! (`IlFunction.body: BodyShape` plus basic blocks), which is a serial restructure
//! sequenced with or behind the frame/liveness spine. The restructure has never
//! been *sized* — every estimate of it has come from adding up census rows named
//! after the byte a straight-line parser happened to stop on, and
//! `docs/GAPS.md` §6's unstable-attribution rule says that number is not the
//! shape's population. This scanner is the counterfactual that replaces the
//! guess: it reports, per body, whether the statement layer decodes end to end and
//! what CFG the body actually has, so the block-IR work can be ranked against the
//! expression-layer work on the same footing.
//!
//! ## The two things that must hold for a body to count as decoded
//!
//! Taken from the throwaway Python validator that produced
//! `docs/IL_STMT_GRAMMAR.md` §13, because they are what made that document
//! falsifiable rather than merely consistent:
//!
//! 1. the walk lands **exactly** on the 7-byte function tail `4F 12 47 54 01 54
//!    00` — not near it, on it;
//! 2. every `54 <k>` scope close carries `k == the depth remaining after the pop`.
//!
//! (2) is the falsification test. A wrong field width anywhere desynchronizes it
//! almost immediately, and (1) always. Measured there: with the TYPE read as a
//! fixed three bytes, 34 bodies land on a *wrong* function tail — that is what
//! over-acceptance looks like in this grammar, and it is why this scanner decodes
//! every field rather than matching fixed patterns.
//!
//! ## The expression layer here is a SKIP layer, and that bounds the claim
//!
//! To reach the tail the walk has to step over operand-stream tokens, and it does
//! so by **width only** — it does not model, type-check or accept them. So
//! "the statement layer decoded" means *"this body's control flow is fully
//! readable"*, NOT *"this body would be in class if control flow were lowered"*.
//! The second question needs the expression layer too, and [`CfBody::residue`] is
//! how this file answers it honestly: it records whether every token the walk
//! stepped over was inside the modeled operand vocabulary, so the counterfactual
//! splits into "the block IR would have to serve this shape" and the strictly
//! smaller "…and nothing else is missing".
//!
//! This is the same discipline `mcall`'s `-whole` / `-more` suffixes apply one
//! layer down, for the same reason: a completeness claim with no production behind
//! it is the failure a census instrument cannot survive.

use super::super::mcall::eat_class_descriptor;
use super::super::{blk, Block};
use crate::func::readers::{
    eat, eat_byte, is_int4_type, is_ptr_to_4, read_token_var, read_type, read_varint,
};

/// The lexical depth a function body starts at: the formals scope is 1 and the
/// body is 2 (`docs/IL_STMT_GRAMMAR.md` §1), so the body's own `53` opens 3.
/// PROV[O] `docs/IL_STMT_GRAMMAR.md` §1 — the formals scope is depth 1 and the body 2, read off captures. Same fact as `expr::BODY_SCOPE_DEPTH` and `sy::SECTION`'s preorder.
const PRE_BODY_DEPTH: u32 = 2;
/// Deeper than any real function; a stream claiming more has desynchronized.
/// (The widest witness is 40 nested braces at depth 42 — **[P] `p6.cpp`**.)
/// PROV[N] not load-bearing — a desynchronization guard, "deeper than any real function"; the widest witness is 40 nested braces at depth 42. It bounds a parse, not an emit.
const MAX_DEPTH: u32 = 96;
/// The function tail every decoded body lands exactly on.
/// PROV[O] the seven-byte `.ex` function tail, read off captures. See `alloc_init_or_fail::FN_TAIL`.
const FN_TAIL: [u8; 7] = [0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00];

/// A body's control-flow **shape**, as decoded. Purely a census axis — no arm of
/// this enum is accepted by anything.
///
/// The split is by *what a lowering would have to build*, not by source syntax,
/// because source syntax is not recoverable and does not matter: `for`, `while`
/// and `do`/`while` all lower to the same back edge, and `break`, `continue`,
/// `goto` and `return` are all the same `3A <label>` (§8.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CfShape {
    /// One jump and one label, and they are the epilogue's. The body is a single
    /// basic block; a lowering needs no CFG at all. This is the shape the port's
    /// existing straight-line, leaf and tail-call classes already live in — so a
    /// body here that still refuses is blocked on *expression* vocabulary, never
    /// on control flow, and counting it against the block-IR restructure would be
    /// double-counting the expression rungs.
    Straight,
    /// No conditional branch, but more than one jump or label: several `return`s
    /// (or a `goto`) converging on the epilogue. Still a DAG with a single exit,
    /// and the cheapest real CFG — no join needs a value merged, because every arm
    /// jumps to the same epilogue that reads the result.
    MultiExit,
    /// Forward conditional branches only, and no switch: `if`, `if`/`else`, `&&`,
    /// `||`, `!` and the conditional expression, in any nesting. A DAG. Payload is
    /// the number of conditional branches, capped — one is the diamond, many is a
    /// short-circuit chain or a nest, and they are different sizes of work.
    Forward(u8),
    /// At least one branch targets a label defined **earlier in the byte stream** —
    /// a back edge, i.e. a loop. The distinguishing cost, and the reason this is
    /// its own shape rather than a `Forward` with a bigger number: a back edge
    /// needs register allocation *across* it, which is the frame/liveness spine's
    /// work and not the block IR's alone (`docs/IL_STMT_GRAMMAR.md` §14.2 step 5).
    Loop,
    /// Carries `3B` / `3C` / `3D` — a switch. Its own shape because it needs a
    /// jump table in `.rdata` or `.text` on top of everything `Forward` needs
    /// (§11), so it can never be part of a first block-IR rung.
    Switch,
}

impl CfShape {
    /// The census sub-key, without the `cflow-` prefix.
    pub(crate) fn name(self) -> &'static str {
        match self {
            CfShape::Straight => "straight",
            CfShape::MultiExit => "multi-exit",
            CfShape::Forward(1) => "if-1",
            CfShape::Forward(2) => "if-2",
            CfShape::Forward(_) => "if-n",
            CfShape::Loop => "loop",
            CfShape::Switch => "switch",
        }
    }
}

/// What a decoded body's **operand stream** needs beyond the modeled vocabulary.
///
/// The point of the field: `cflow-<shape>` alone says how many bodies the block IR
/// must serve, which is an upper bound on what lowering control flow is worth. This
/// splits that population into the part that is *only* waiting on the block IR and
/// the part that is waiting on the expression layer as well — the two numbers a
/// rung has to be ranked from, and a single count cannot be both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CfResidue {
    /// Every operand token the walk stepped over is one the **modeled shapes
    /// already consume**: an int4/ptr4 LOAD or literal, the plain `+ - *` chain, a
    /// class-preserving `2C`, the `41` result annotation, an int4 `32` store, and
    /// the call quadruple `26 <tok>` · `BD` · `55 <TYPE>` · `4C`. A body here is
    /// blocked on **control flow alone** — its operand vocabulary is inside the
    /// class the port has been byte-graded on.
    ///
    /// The membership test is the *same* one the accepting parser applies at the
    /// same positions ([`is_int4_type`] / [`is_ptr_to_4`], the pair behind
    /// `eat_int_like_or_ptr4`), deliberately: a residue computed from a looser
    /// vocabulary than the emitter's would report bodies as "waiting on control
    /// flow alone" that are in fact waiting on a type gate too, which is the
    /// over-claim a counterfactual exists to avoid.
    Modeled,
    /// Something else: a call, a float, a member designator, an intrinsic, a
    /// conversion, a temporary bind. Naming *which* would re-derive the existing
    /// `expr-*` histogram inside this axis, so it does not — the existing key
    /// already says which, and this field only says "not only control flow".
    Expression,
}

/// One body's decoded control flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CfBody {
    pub(crate) shape: CfShape,
    pub(crate) residue: CfResidue,
}

impl CfBody {
    /// The census key for the control-flow axis.
    pub(crate) fn key(self) -> String {
        match self.residue {
            CfResidue::Modeled => format!("cflow-{}+expr-modeled", self.shape.name()),
            CfResidue::Expression => format!("cflow-{}", self.shape.name()),
        }
    }
}

/// **The exception-handling state markers a body carries** — the `5C` / `5D` /
/// `5E` trailer family, counted rather than interpreted.
///
/// This is the raw material of the EH axis ([`EhMarkers::key`]); what it is FOR
/// is on that method. Everything here is a count the walk collected, and it is
/// collected whether or not the walk finished, because "a marker was seen before
/// the stop" is itself the answer for a body that does not decode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EhMarkers {
    /// `5C <TYPE> <varint>` trailers — one per statement in which an object with
    /// a destructor became live. In a compiler-generated constructor or
    /// destructor that is the sub-object statement; in an ordinary function it is
    /// the statement that declared the local or materialized the temporary.
    pub(crate) live_stmts: usize,
    /// The largest `<n>` on a `5D` / `5E` count trailer — how many destructible
    /// objects the body's EH state tracks at once.
    pub(crate) count: u32,
    /// `5D` / `5E` trailers standing immediately before a `4B`, i.e. the ones
    /// that are a statement of their own rather than an operand of the statement
    /// they sit inside. Subtracted out below so the trailer is not counted as a
    /// body statement.
    pub(crate) trailer_stmts: usize,
    /// `4B` statement ends seen anywhere in the body.
    pub(crate) stmt_ends: usize,
    /// Any `5C` / `5D` / `5E` at all.
    pub(crate) any: bool,
    /// **CALL tokens issued while at least one destructible object is live** —
    /// the thing `docs/EH_RECORDS.md` §9.4's predicate actually asks for.
    ///
    /// A `4C` argument-list close seen while the running live count (raised by
    /// `5C`, lowered by a `5D`/`5E` count trailer) is non-zero is an **outbound
    /// control transfer at a non-empty live set**, i.e. a state. One is enough:
    /// `maxState >= 1` and the whole `§1`–`§5` EH record set exists. Zero of them
    /// and `maxState = 0` and none of it is emitted, however many statements the
    /// body has.
    pub(crate) calls_live: usize,
}

impl EhMarkers {
    /// Statements that are neither a `5C` object-goes-live statement nor a
    /// `5D`/`5E` trailer's own — i.e. **the "anything else" of the boundary**.
    ///
    /// A `return` statement carries no `4B` (`docs/IL_STMT_GRAMMAR.md` §9) and so
    /// is not counted here. That is the conservative direction for the cheap side
    /// and it is stated rather than hidden: a body whose only extra statement were
    /// a bare `return;` would read as bare.
    pub(crate) fn other_stmts(self) -> usize {
        self.stmt_ends
            .saturating_sub(self.live_stmts)
            .saturating_sub(self.trailer_stmts)
    }

    /// **The EH census axis, on the REFUTED statement-count predicate.** Which
    /// side of `docs/EH_RECORDS.md` §6's boundary this body falls on:
    ///
    /// > Exactly one sub-object statement and nothing else is a bare branch. A
    /// > second sub-object, or any other statement beside it, is the WHOLE EH
    /// > RECORD.
    ///
    /// **That predicate is false in BOTH directions** — `docs/EH_RECORDS.md` §9.4
    /// and §10. It is kept, and still reported beside [`EhMarkers::state_key`],
    /// for exactly one reason: §7.3's published split is keyed on it, so the two
    /// axes crossed are what reconciles the old numbers with the new ones. It is
    /// **not** the axis to rank from. See [`EhMarkers::state_key`].
    ///
    /// Nothing in the blocking-feature key says which side a body is on — the
    /// probe `work/WEH/probe/p1.cpp` has a cheap constructor and an EH
    /// constructor filed under the *same* key `expr-intrinsic-this-adjust` — and
    /// that is the whole reason this axis exists. `docs/EH_RECORDS.md` §7 is the
    /// measurement: 14 hand-written functions, both sides, at the workload's own
    /// flags. Twelve are classified and the key agrees with whether the obj
    /// carries an `__ehfuncinfo$` in **all twelve**; the other two stop decoding
    /// before any marker and claim nothing.
    ///
    /// `decoded` is whether the statement walk reached the function tail. It
    /// matters because **the bare shape always decodes**: every token in it is one
    /// this scanner knows, so a body that carries a marker and then stops is
    /// certainly not bare. That is what makes `eh-partial` an answer rather than
    /// an absence.
    pub(crate) fn key(self, decoded: bool) -> &'static str {
        if !decoded {
            return if self.any { "eh-partial" } else { "eh-unknown" };
        }
        if !self.any {
            return "eh-none";
        }
        if self.live_stmts >= 2 || self.count >= 2 {
            return "eh-multi";
        }
        if self.other_stmts() > 0 {
            return "eh-plus-stmt";
        }
        "eh-bare"
    }

    /// **The EH census axis, on the MEASURED predicate.** `docs/EH_RECORDS.md`
    /// §9.4, from bytes:
    ///
    /// > An EH record set exists iff **`maxState >= 1`**, i.e. iff at least one
    /// > outbound control transfer occurs while a destructible object is live.
    /// > `S = 0` and the entire §1–§5 apparatus disappears with it.
    ///
    /// The statement count does not enter it, and reading it as one is wrong in
    /// **both** directions — §10 grades 31 hand-written functions against their
    /// own objs and the statement rule misses 6 of them, 4 the expensive way
    /// (`int P(int a){ SE s; int x=a*3; return x; }` has "another statement
    /// beside" the object and gets no record at all) and 2 the cheap way
    /// (`int P(int a){ SE s; return gp(a); }` has NO other statement — a `return`
    /// carries no `4B` — and gets the whole record set).
    ///
    /// Four values:
    ///
    /// * `eh-none` — decoded, no `5C`/`5D`/`5E`. No destructible object is ever
    ///   live, so `/EHsc` costs it nothing.
    /// * `eh-state0` — decoded, a marker, and **no** call while an object was
    ///   live. `maxState = 0`: no `__CxxFrameHandler` prefix, one `.pdata`, no EH
    ///   `.rdata`, no funclet, function symbol `Value = 0`. **The cheap side.**
    /// * `eh-state1` — at least one call while an object was live. `maxState >= 1`
    ///   and the whole record set exists. Claimed **whether or not the walk
    ///   finished**: a transfer already seen at a non-empty live set cannot be
    ///   un-seen by whatever stopped the walk later.
    /// * `eh-partial` — a marker was seen, no such call was, and then the walk
    ///   stopped. Undecided: the calls that would have decided it may be in the
    ///   part that was never read. **Not on either side.**
    /// * `eh-unknown` — the walk stopped before any marker; nothing is claimed.
    ///
    /// Note `eh-partial` is weaker here than on the statement axis, where it was
    /// argued onto the EH side structurally. That argument does not survive the
    /// change of predicate — the cheap side is no longer "the shape that always
    /// decodes" — so the bucket goes back to claiming nothing.
    pub(crate) fn state_key(self, decoded: bool) -> &'static str {
        // Proof beats completeness: one transfer at a non-empty live set settles
        // it, and a walk that stopped afterwards cannot unsettle it.
        if self.calls_live > 0 {
            return "eh-state1";
        }
        if !decoded {
            return if self.any { "eh-partial" } else { "eh-unknown" };
        }
        if !self.any {
            return "eh-none";
        }
        "eh-state0"
    }
}

/// One body's two decode-only readings: the control-flow verdict (or the byte
/// that stopped it) and the EH markers counted up to that point.
pub(crate) struct CfScan {
    pub(crate) body: Result<CfBody, Block>,
    pub(crate) eh: EhMarkers,
    /// Whether the walk reached the function tail — the same fact as
    /// `body.is_ok()`, named because [`EhMarkers::key`] reads it.
    pub(crate) decoded: bool,
    /// **Which operand token took this body out of the modeled class**, or
    /// `None` if none did — see [`Scan::off_class`]. Board **#1345**.
    ///
    /// It rides on `CfScan` and NOT on [`CfBody`] on purpose: a `CfBody` is
    /// `PartialEq` and a dozen pinned-segment tests assert whole struct
    /// literals against it, so a field here would have made every one of them
    /// carry a diagnostic they are not about. It is also collected **whether or
    /// not the walk finished**, for the same reason [`CfScan::eh`] is — a body
    /// that left the class and then stopped left the class.
    pub(crate) off_reason: Option<&'static str>,
    /// **The first pass's token → position map**, kept instead of discarded —
    /// see [`LabelTable`].
    ///
    /// It rides on `CfScan` and not on [`CfBody`] for the reason
    /// [`CfScan::off_reason`] does: `CfBody` is `PartialEq` and a dozen pinned
    /// segment tests assert whole struct literals against it. It is also
    /// collected **whether or not the walk finished**, for the same reason the
    /// EH counts are — but every question it answers is meaningless on a partial
    /// walk, which is why [`super::step5::CfgAdmit`]'s first clause is
    /// `decoded`.
    pub(crate) labels: LabelTable,
}

/// A branch or label site, in stream order.
#[derive(Clone, Copy)]
struct Site {
    tok: u32,
    /// Offset of the *opcode*, so "defined before" is a plain comparison.
    at: usize,
}

/// **The token → position map `docs/IL_STMT_GRAMMAR.md` §14.2 step 5 asks for**,
/// as a product of the first pass rather than a second walk.
///
/// Step 5's text is *"build a token → position map in a first pass over the
/// body, since `3A` carries no direction"*. The first pass is [`scan_full`] —
/// there is no cheaper one, because finding a `29` at all requires every field
/// width before it, and a byte scan for `0x29` hits operand payload. What was
/// missing is not the pass, it is that the pass **threw the map away**: only
/// [`CfShape`] survived it, and [`shape_of`] re-derived the one question it
/// needed (*is any target defined earlier*) by a nested search it did not keep.
///
/// Keeping it answers three more questions that nothing in this tree could ask,
/// and each is a way the CFG can be un-lowerable that [`CfShape`] reports as
/// `Forward`:
///
/// * [`LabelTable::unresolved`] — a `38`/`39`/`3A` names a token **no `29`
///   defines**. Its position is not merely late, it is *unknown*, so its
///   direction is unknown too. A body here is not a forward DAG; it is a body
///   whose CFG this tree cannot claim to have read.
/// * [`LabelTable::duplicate_defs`] — two `29`s carrying **one** token. The map
///   is then not a function and "the position of `tok`" has no referent, so
///   every other question below is unanswerable rather than false.
/// * [`LabelTable::dead_defs`] — a `29` nothing branches to. Harmless for
///   lowering and recorded because it is the control: it is the one of the three
///   that must be allowed to be non-zero (§9's epilogue label is reached by
///   fallthrough in a body with no early return), so a predicate that refused on
///   it would refuse the straight-line class.
///
/// **Complete for `29`/`38`/`39`/`3A` and for nothing else.** `3B`/`3C`/`3D`
/// carry label tokens too (§11) and this table does not record them, so on a
/// `switch` body it is partial **by construction**. That is why
/// [`super::step5::CfgAdmit`] refuses `CfShape::Switch` *before* it reads any
/// question off this table, and not as a matter of taste.
#[derive(Clone, Debug, Default)]
pub(crate) struct LabelTable {
    /// `29 <tok>` definitions in stream order, `(token, offset of the 29)`.
    defs: Vec<(u32, usize)>,
    /// `38`/`39`/`3A` targets in stream order, `(token, offset of the opcode)`.
    refs: Vec<(u32, usize)>,
}

impl LabelTable {
    /// Where `tok` is defined, or `None` if no `29` defines it.
    ///
    /// **`None` is not "late".** Every caller has to decide what an unknown
    /// position means for it, and the one that matters —
    /// [`LabelTable::back_edges`] — treats it as *not a back edge*, which is the
    /// permissive reading. That is safe only because
    /// [`LabelTable::unresolved`] is a separate refusal clause tested first;
    /// collapsing the two would let an undefined target pass as forward.
    pub(crate) fn position_of(&self, tok: u32) -> Option<usize> {
        self.defs.iter().find(|(t, _)| *t == tok).map(|(_, at)| *at)
    }

    /// References whose target is defined **earlier in the byte stream** — a back
    /// edge, i.e. a loop. §14.2 step 5's own refusal clause is written over
    /// exactly this set.
    pub(crate) fn back_edges(&self) -> usize {
        self.refs
            .iter()
            .filter(|(tok, at)| self.position_of(*tok).is_some_and(|d| d < *at))
            .count()
    }

    /// References naming a token **no `29` defines**.
    pub(crate) fn unresolved(&self) -> usize {
        self.refs
            .iter()
            .filter(|(tok, _)| self.position_of(*tok).is_none())
            .count()
    }

    /// Definitions **nothing references**. The control of the three: it is
    /// expected to be non-zero on ordinary bodies and is never a refusal.
    ///
    /// Deliberately not consulted by [`super::step5::CfgAdmit`], and the
    /// attribute is how that stays visible: a lane that adds it as a clause
    /// would refuse `il_stmt_early_return.cpp`, a `Forward` modeled body whose
    /// skip label is reached by fallthrough. `step5`'s own test names that
    /// segment.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn dead_defs(&self) -> usize {
        self.defs
            .iter()
            .filter(|(tok, _)| !self.refs.iter().any(|(r, _)| r == tok))
            .count()
    }

    /// Tokens carrying **more than one** `29`. Non-zero means the map is not a
    /// function and no other question here has an answer.
    pub(crate) fn duplicate_defs(&self) -> usize {
        self.defs
            .iter()
            .enumerate()
            .filter(|(i, (tok, _))| self.defs[..*i].iter().any(|(t, _)| t == tok))
            .count()
    }

    /// How many label definitions and how many references the body carries —
    /// the two raw counts, so a reader can see the map is not empty before
    /// believing a zero from any question above. Absence reads as success unless
    /// something forbids it.
    pub(crate) fn sizes(&self) -> (usize, usize) {
        (self.defs.len(), self.refs.len())
    }
}

/// Scanner state. Kept in one struct because the walk is one loop and the
/// alternative — threading six `&mut`s through twenty arms — is how the widths
/// drift apart.
struct Scan<'a> {
    seg: &'a [u8],
    p: usize,
    depth: u32,
    labels: Vec<Site>,
    conds: Vec<Site>,
    jumps: Vec<Site>,
    switches: usize,
    /// Set the moment a stepped-over operand token is outside the modeled class.
    off_class: bool,
    /// **Which token took it out of the class**, first one only — see
    /// [`Scan::off_class`]. `None` iff `off_class` is false, which is the
    /// invariant the accounting control checks.
    off_reason: Option<&'static str>,
    /// The EH-state trailer counts — see [`EhMarkers`].
    eh: EhMarkers,
    /// **How many destructible objects are live right here**, in stream order:
    /// `5C` raises it by one, a `5D`/`5E` count trailer lowers it by that
    /// trailer's own `<n>`. Scratch for [`EhMarkers::calls_live`], which is the
    /// only thing that reads it.
    ///
    /// Saturating, deliberately: a trailer whose `<n>` exceeds the count raised
    /// so far means this walk did not see every `5C` (an inlined constructor, a
    /// marker inside a production this scanner steps over by width), and
    /// clamping to zero is the direction that under-claims the EH side.
    eh_live: u32,
}

impl<'a> Scan<'a> {
    fn at(&self, k: usize) -> Option<u8> {
        self.seg.get(self.p + k).copied()
    }

    /// A typed field read that fails closed. Every width in this file goes
    /// through one of these three, never through a constant.
    fn ty(&mut self, ctx: &'static str) -> Result<(u8, u8), Block> {
        match read_type(self.seg, self.p) {
            Some((tag, kind, _, w)) => {
                self.p += w;
                Ok((tag, kind))
            }
            None => Err(blk(self.seg, self.p, ctx)),
        }
    }

    fn tok(&mut self, ctx: &'static str) -> Result<u32, Block> {
        match read_token_var(self.seg, self.p) {
            Some((t, w)) => {
                self.p += w;
                Ok(t)
            }
            None => Err(blk(self.seg, self.p, ctx)),
        }
    }

    fn vint(&mut self, ctx: &'static str) -> Result<i32, Block> {
        let mut q = self.p;
        match read_varint(self.seg, &mut q) {
            Some(v) => {
                self.p = q;
                Ok(v)
            }
            None => Err(blk(self.seg, self.p, ctx)),
        }
    }

    /// `4F 01 <varint>` source-line markers, any number of them. They appear at
    /// **every** statement boundary and two in a row is normal (§3); a decoder that
    /// consumes one is the width bug that mis-filed 57,928 functions.
    fn line_markers(&mut self) {
        while self.at(0) == Some(0x4F) && self.at(1) == Some(0x01) {
            let mut q = self.p + 2;
            if read_varint(self.seg, &mut q).is_none() {
                return; // malformed payload — leave `p` put and let the walk block
            }
            self.p = q;
        }
    }

    /// Note that the operand stream left the modeled class, **with the token's
    /// own reason**, so the call sites read as a list of what is not modeled.
    ///
    /// That sentence was this method's doc comment for weeks and the method took
    /// no argument. The reason was in the code — twenty-one arms, each with a
    /// paragraph explaining precisely which construct it is — and none of it
    /// reached a histogram, so `CfResidue::Expression` was one bucket holding
    /// twenty-one facts and board **#1344**'s 518,991 was unattributable.
    /// It takes the argument now.
    ///
    /// **First reason wins.** A body that leaves the class twice is off-class for
    /// the first thing it hit, which is the same convention the blocker keys use
    /// (`the port stops at the first refusal by design`) and the only one under
    /// which the per-reason counts sum to the off-class total. The sum is a
    /// published control (`cflow-offclass-accounted`), not an assumption.
    fn off_class(&mut self, why: &'static str) {
        if residue_admits(why) {
            return;
        }
        self.off_class = true;
        if self.off_reason.is_none() {
            self.off_reason = Some(why);
        }
    }
}

/// **Decode one function body's control flow.** `lo` is the `4C 4F 11` body
/// marker's offset within `seg`.
///
/// `Ok` means the walk consumed every byte of the body through a decoded field and
/// landed exactly on the function tail with the depth invariant intact. `Err` names
/// the production it stopped in and the byte — the same fail-closed contract, and
/// the same [`Block`] vocabulary, the accepting parser uses, so a caller cannot
/// confuse "decoded" with "accepted".
/// It also keeps the EH-marker counts it collected **whether or not it
/// finished**. One walk, two readings — the census needs both and must not pay
/// for the body twice.
pub(crate) fn scan_full(seg: &[u8], lo: usize) -> CfScan {
    let mut s = Scan {
        seg,
        // `lo + 3` for a composed `4C 4F 11`, `lo + 1` for the bare `4C` of a
        // `??__E`/`??__F` thunk. One locator (`func::bundle::ops_start`).
        p: crate::func::ops_start(seg, lo),
        depth: PRE_BODY_DEPTH,
        labels: Vec::new(),
        conds: Vec::new(),
        jumps: Vec::new(),
        switches: 0,
        off_class: false,
        off_reason: None,
        eh: EhMarkers::default(),
        eh_live: 0,
    };
    let body = walk(&mut s);
    // The map is built from the SAME sites `shape_of` classifies, in the same
    // pass, so the two can never disagree about what the body contains — only
    // about what to conclude from it. That is the point: `CfgAdmit`'s back-edge
    // clause and `CfShape::Loop` are then two readings of one collection rather
    // than two collections, and the consistency control between them
    // (`cfg-admit-backedge-shape-disagree`) is checking the readings, which is
    // the only thing that can drift.
    let labels = LabelTable {
        defs: s.labels.iter().map(|x| (x.tok, x.at)).collect(),
        refs: s.conds.iter().chain(s.jumps.iter()).map(|x| (x.tok, x.at)).collect(),
    };
    CfScan { decoded: body.is_ok(), body, eh: s.eh, off_reason: s.off_reason, labels }
}

fn walk(s: &mut Scan) -> Result<CfBody, Block> {
    if !eat_byte(s.seg, &mut s.p, 0x53) {
        return Err(blk(s.seg, s.p, "cf-body-open"));
    }
    s.depth += 1;
    while step(s)? {}
    // (1) The parse must land ON the tail. `4F 12` is what ended the statement
    // loop; anything but the full seven bytes here is a walk that stopped in the
    // right neighbourhood for the wrong reason, which §13 measured as the
    // over-acceptance mode of this grammar.
    if !eat(s.seg, &mut s.p, &FN_TAIL) {
        return Err(blk(s.seg, s.p, "cf-tail"));
    }
    Ok(CfBody { shape: shape_of(s), residue: residue_of(s) })
}

/// **`C2RS_CFRESIDUE_ADMIT` — the residue's own counterfactual, and the reason
/// board #1345 does not need a widening to be answered.**
///
/// A comma-separated set of [`Scan::off_class`] reason names. Every arm named
/// here stops taking a body out of [`CfResidue::Modeled`], so one scan reports
/// *what the counterfactual would become* if the vocabulary were widened by
/// exactly that set — **and, in the same run, what the over-claim on the other
/// side becomes**, which is the half a bare widening never publishes.
///
/// #1345 states the rule this exists to satisfy: *whatever the residue is,
/// report it with its validated relationship to the port's class, or report the
/// bracket.* A shipped widening cannot do that, because after it ships there is
/// nothing left to compare against; the pair has to be measurable at one tree.
/// So the vocabulary is **not** widened — the default is empty and every digit
/// of every published `cflow-*` key is unchanged without the variable — and the
/// widening becomes a thing you *price* instead of a thing you *do*.
///
/// It is safe to commit for the reason #1345 gives itself: `control_flow.rs` is
/// **decode-only**. Nothing reads [`CfResidue`] except the census report — not
/// acceptance, not `shape_to_function`, not the emitter — so this variable
/// cannot move an obj byte in either direction, and `mismatch` is structurally
/// unable to see it. That is a stronger guarantee than `C2RS_SINK_CHAIN`'s
/// poison, which exists precisely because the sink *can* reach the emitter.
///
/// The set is published as `gap-metric cflow-residue-admit` on any scan that
/// sets it, and the key is **absent** rather than empty otherwise: a collector
/// that read a missing key as "no admissions" would be right, and one that read
/// an empty key as "the file has no arms" would not.
fn residue_admits(why: &str) -> bool {
    // PROV[N] not load-bearing — a `OnceLock` measurement sink.
    static ON: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    ON.get_or_init(|| {
        std::env::var("C2RS_CFRESIDUE_ADMIT")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
    .iter()
    .any(|s| s == why)
}

/// The admit set, for the scan report. Empty when the variable is unset.
pub(crate) fn residue_admit_set() -> String {
    std::env::var("C2RS_CFRESIDUE_ADMIT").unwrap_or_default()
}

fn residue_of(s: &Scan) -> CfResidue {
    if s.off_class {
        CfResidue::Expression
    } else {
        CfResidue::Modeled
    }
}

/// Classify the decoded sites. Order matters and is by **cost of lowering**, not
/// by frequency: a switch is a switch even if it also loops, and a loop is a loop
/// however few conditionals it has, because the back edge is the expensive fact.
fn shape_of(s: &Scan) -> CfShape {
    if s.switches > 0 {
        return CfShape::Switch;
    }
    // A back edge: some branch or jump names a label whose `29` already went past.
    // `3A` carries no direction (§8.1) — the target is a token and forward/backward
    // is decided by where the definition happens to sit — so this is the only way
    // to know, and it is why the scan records positions rather than counts.
    let backward = s
        .jumps
        .iter()
        .chain(s.conds.iter())
        .any(|b| s.labels.iter().any(|l| l.tok == b.tok && l.at < b.at));
    if backward {
        return CfShape::Loop;
    }
    if s.conds.is_empty() {
        // Every body has exactly one epilogue `3A` and one epilogue `29` (§9), so
        // one of each is the single-basic-block case and anything more is a second
        // exit path converging on it.
        if s.jumps.len() <= 1 && s.labels.len() <= 1 {
            return CfShape::Straight;
        }
        return CfShape::MultiExit;
    }
    CfShape::Forward(s.conds.len().min(255) as u8)
}

/// One `item` of `docs/IL_STMT_GRAMMAR.md` §0. `Ok(false)` ends the statement list
/// (the walk is standing on the function tail's `4F 12`).
fn step(s: &mut Scan) -> Result<bool, Block> {
    s.line_markers();
    let Some(b) = s.at(0) else {
        return Err(blk(s.seg, s.p, "cf-stmt"));
    };
    match b {
        // ---- the scope stack (§1) -----------------------------------------
        0x53 => {
            if s.depth >= MAX_DEPTH {
                return Err(blk(s.seg, s.p, "cf-scope-too-deep"));
            }
            s.p += 1;
            s.depth += 1;
        }
        0x54 => {
            let Some(k) = s.at(1) else {
                return Err(blk(s.seg, s.p, "cf-scope-close"));
            };
            let Some(d) = s.depth.checked_sub(1) else {
                return Err(blk(s.seg, s.p, "cf-scope-underflow"));
            };
            // **The falsification test.** `k` is the depth remaining after the pop,
            // so this is a free integrity check on every field width consumed since
            // the last close — and the reason it is a comparison rather than a
            // decode is §12.1: whether `k` is a plain byte or a varint is UNKNOWN,
            // and both readings agree on every value ever observed (max `0x2A`).
            if u32::from(k) != d {
                let seg_len = s.seg.len();
                let ctx = "cf-scope-depth";
                return Err(Block { ctx, byte: Some(k), off: s.p, seg_len, aux: 0 });
            }
            s.p += 2;
            s.depth = d;
        }
        // ---- control flow (§7, §8, §9, §11) --------------------------------
        0x29 => {
            let at = s.p;
            s.p += 1;
            let tok = s.tok("cf-label-tok")?;
            s.labels.push(Site { tok, at });
        }
        0x38 | 0x39 => {
            let at = s.p;
            s.p += 1;
            let tok = s.tok("cf-branch-tok")?;
            s.conds.push(Site { tok, at });
        }
        0x3A => {
            let at = s.p;
            s.p += 1;
            let tok = s.tok("cf-jump-tok")?;
            s.jumps.push(Site { tok, at });
        }
        0x3B => {
            s.p += 1;
            s.tok("cf-switch-tok")?;
            s.switches += 1;
        }
        0x3C => {
            s.p += 1;
            s.ty("cf-switch-type")?;
            s.tok("cf-switch-default")?;
            s.switches += 1;
        }
        0x3D => {
            s.p += 1;
            s.tok("cf-switch-case")?;
            s.switches += 1;
        }
        // ---- statement end (§2) --------------------------------------------
        // `4B` pops the expression stack to empty and discards whatever is left; it
        // is emitted for the last statement too. The `return` statement is the one
        // that has none (§9).
        0x4B => {
            s.p += 1;
            s.eh.stmt_ends += 1;
        }
        // `4F 12` opens the function tail. Any other `4F NN` at a statement
        // boundary is not statement layer (§12.6) and the tail check refuses it.
        0x4F => return Ok(false),
        _ => {
            operand(s)?;
        }
    }
    Ok(true)
}

/// Step over exactly one operand-stream token, by **width only**.
///
/// Every arm either (a) is a token the modeled expression class handles, in which
/// case it may also class-check its TYPE, or (b) calls [`Scan::off_class`] to
/// record that this body needs expression work beyond control flow. Nothing here
/// accepts anything: the widths come from `docs/IL_EXPR_LAYER.md`,
/// `docs/IL_CAST_CONVERT.md` and `docs/IL_TYPE_TAGS.md`, and a token whose width
/// this file does not know is an `Err`, never a guessed skip.
fn operand(s: &mut Scan) -> Result<(), Block> {
    let Some(b) = s.at(0) else {
        return Err(blk(s.seg, s.p, "cf-expr"));
    };
    match b {
        // LOAD `B9 <tok> <TYPE>` — modeled when the value is int4 or a 4-byte
        // pointer, which is exactly the gate `parse_expr` uses.
        0xB9 => {
            s.p += 1;
            s.tok("cf-load-tok")?;
            let (tag, kind) = s.ty("cf-load-type")?;
            if !(is_int4_type(tag, kind) || is_ptr_to_4(tag, kind)) {
                s.off_class("load-type");
            }
        }
        // LITERAL `33 <TYPE> <payload>`. The payload width is a function of the
        // type: a real is 8 IEEE bytes + a 2-byte size, an 8-byte integer's escape
        // is 8 bytes, everything else is the ordinary varint.
        0x33 => {
            s.p += 1;
            let (tag, kind) = s.ty("cf-lit-type")?;
            if !(is_int4_type(tag, kind) || is_ptr_to_4(tag, kind)) {
                s.off_class("lit-type");
            }
            lit_payload(s, tag, kind)?;
        }
        // SYMBOL PUSH `26 <tok>` — a designator. In the modeled vocabulary: it is
        // how every accepted call names its callee and how every accepted
        // assignment names its destination. What is *not* modeled is a designator
        // built ON one (an offset add, a subscript, a deref), and those are their
        // own opcodes below.
        0x26 => {
            s.p += 1;
            s.tok("cf-sym-tok")?;
        }
        // The plain additive/multiplicative chain — the whole of the modeled
        // operator vocabulary.
        0x02 | 0x03 | 0x04 => s.p += 1,
        // The rest of the payload-free operator table, and **only the entries a
        // capture has established**: `%` and `~` from `IL_STMT_GRAMMAR.md` §5's
        // `p4.cpp` one-function-per-operator probe, the shifts and bitwise trio
        // from the same, the short-circuit trio, and the six relations that
        // `docs/CODEGEN_W6_COMPARE.md` pins by compiling a probe per relation and
        // reading the emitted byte.
        //
        // The gaps are deliberate and are the point. `07`, `08`, `14`, `1D`,
        // `1E`, `25` are unwitnessed; `14` in particular has no C operator between
        // `%=` and `<<=` and §5 says in as many words not to fill it. Guessing
        // width 1 for them would be right most of the time and silently
        // desynchronize the rest — and a desync that lands on a plausible tail is
        // the failure this whole scanner is built to make impossible. They refuse,
        // and the size of the `cf-expr-0xNN` row they produce is what tells the
        // next rung whether establishing them is worth a probe.
        //
        // **`05` WAS in that unwitnessed list and is not any more** (`lane
        // w-divsplit`, board **#820**). This paragraph read "`05`, `07`, …" and
        // the row it names, `cf-expr-0x05`, had grown to **4,671 bodies** — the
        // largest thing this table was refusing. It was already witnessed when
        // that was written: `lane w-divmod` captured `B9 <tok> <T> B9 <tok> <T>
        // >05< 41 <T> 3A …` for all four cells of the divide/modulo leaf
        // (`div_mod_leaf`'s module header, read out of `c2rs census`'s own
        // blocking hexdump) and graded that shape **185 of 185** against real
        // `c2.dll`. `06` sat in the witnessed list beside it the whole time,
        // from the same §5 probe, which is the tell: `%` and `/` are one
        // production and only one of them had been carried across.
        //
        // Confirmed a second way before moving it, on the workload rather than
        // on a probe: at all **4,674** dc3 division sites the byte after the
        // opcode opens a new token — `32 <TYPE>` at 4,646 and `33 <TYPE>
        // <payload>` at 26 — so there is nowhere for a payload to be
        // (`work/w-divsplit/shape.py`).
        0x05 | 0x06 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x1A | 0x1B | 0x1C | 0x1F
        | 0x20 | 0x21 | 0x22 | 0x23 | 0x24 => {
            s.p += 1;
            s.off_class(match b {
                0x05 | 0x06 => "div-mod",
                0x09 | 0x0A => "shift",
                0x0B | 0x0C | 0x0D | 0x0E => "bitwise",
                0x1A | 0x1B | 0x1C => "logical",
                _ => "compare",
            });
        }
        // Compound assignment / inc-dec: `<op> <TYPE>`, the twelve witnessed
        // opcodes of §5. `0x14` is deliberately NOT here — it is unobserved, and it
        // is handled above as a payload-free operator only because a width guess in
        // the other direction desynchronizes; see the test that pins it.
        0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x35 | 0x36 => {
            s.p += 1;
            s.ty("cf-rmw-type")?;
            s.off_class("rmw");
        }
        // The three `<op> <TYPE>` tokens the modeled shapes DO consume, each with
        // the same class gate the accepting parser applies at that position: the
        // `41` result annotation and the `55` argument push take
        // `eat_int_like_or_ptr4`'s pair, and the `32` store takes `eat_int_like`.
        // A wider type here is a real lowering difference (a `stb`, an `stfs`, a
        // 64-bit pair), not an annotation, which is why the class is asked rather
        // than the opcode alone.
        0x32 | 0x41 | 0x55 => {
            s.p += 1;
            let (tag, kind) = s.ty(match b {
                0x32 => "cf-store-type",
                0x41 => "cf-result-type",
                _ => "cf-argpush-type",
            })?;
            let ok = if b == 0x32 {
                is_int4_type(tag, kind)
            } else {
                is_int4_type(tag, kind) || is_ptr_to_4(tag, kind)
            };
            if !ok {
                s.off_class("store-type");
            }
        }
        // …and the `<op> <TYPE>` tokens that are their own missing production:
        // indirect load, byte-offset add, virtual dispatch, and the rest.
        //
        // **One `ctx` per opcode, deliberately.** These arms shared a single
        // `cf-op-type` key for exactly one measurement, and it reported 112,389
        // bodies blocked on the byte `55` — a byte that cannot be a TYPE tag,
        // i.e. a desync, with no way to tell WHICH of the seven opcodes had
        // desynchronized. The cause was `44`, which is **payload-free**
        // (`docs/IL_EXPR_LAYER.md` §7: the following byte is `30`/`55`, whose bit
        // 7 is clear, so it cannot be a TYPE) and had been given a TYPE here on
        // `IL_CALL_GRAMMAR.md` §7's superseded reading. A shared key hid a live
        // width bug behind an opaque hex bucket, which is `GAPS.md` §6's
        // by-position mis-attribution in miniature — so the keys are split.
        0x27 => {
            s.p += 1;
            s.ty("cf-offadd-type")?;
            s.off_class("off-add");
        }
        0x30 => {
            s.p += 1;
            s.ty("cf-deref-type")?;
            s.off_class("deref");
        }
        // ---- the EH-state trailer family (WEH, `docs/EH_RECORDS.md` §7) -----
        //
        // `5C <TYPE> <varint>` — emitted at the end of a statement in which an
        // object with a destructor became live. `5D <varint> <varint>` and
        // `5E <varint> <varint>` — the constructor-side and destructor-side count
        // trailers, whose first field is how many such objects the body's EH state
        // tracks (`docs/EH_RECORDS.md` §6 measured `5E 02` for a two-member
        // destructor against `5E 01` for a one-member one).
        //
        // Widths MEASURED, not inferred, at the workload's own flags —
        // `work/WEH/probe/p1.cpp` and `p2.cpp`, fourteen hand-written functions:
        // the `5C` TYPE is a decoded type of varying width (`86 41 74`,
        // `86 46 AB 20`, `A6 43 8C 20` all occur) and every second field is a
        // varint that really does escape (`5D 01 80 A1 00 00 00`,
        // `5C 86 41 74 80 01 01 00 00`), so neither a fixed width nor a plain byte
        // read survives the corpus.
        //
        // This is the rung `cf-expr-0x5C` — **309,804 bodies, the largest single
        // row on the control-flow axis** — was measuring, and what it was
        // measuring was not a ctor/dtor row: `int userfn(int a){ MemA s; g(a);
        // return a+1; }` carries a `5C` too. See [`EhMarkers`].
        0x5C => {
            s.p += 1;
            s.ty("cf-eh-live-type")?;
            s.vint("cf-eh-live-state")?;
            s.eh.live_stmts += 1;
            // …and one more object is live from here on. MEASURED: the `5C` is the
            // last token of its statement (it stands immediately before the `4B`),
            // so a constructor call in the same statement is BEFORE it and is
            // correctly counted at the lower state — which is where c2 puts it
            // (`docs/EH_RECORDS.md` §9.1, "live from the instruction after its
            // constructor call returns").
            s.eh_live += 1;
            s.eh.any = true;
            s.off_class("eh-obj-live");
        }
        0x5D | 0x5E => {
            s.p += 1;
            let n = s.vint("cf-eh-count")?;
            s.vint("cf-eh-count-state")?;
            s.eh.count = s.eh.count.max(n.max(0) as u32);
            // …and this trailer's `<n>` objects stop being live. This is what
            // separates `{ SE s; } { SE t; }` (two objects, never live together,
            // NO EH record) from `{ SE s; SE t; }` (one is live across the
            // other's transfer, record) — probe `mF` against `mE`,
            // `docs/EH_RECORDS.md` §10. Without it the second scope's
            // constructor call reads as a transfer at a live set and the body is
            // filed on the expensive side, which is measurably wrong.
            s.eh_live = s.eh_live.saturating_sub(n.max(0) as u32);
            // A trailer standing immediately before a `4B` is a statement of its
            // own; one standing before anything else is an operand of the
            // statement it sits inside. Both spellings occur in one probe
            // (`5E 01 21 4B` in `??1One`, `5E 01 01 44` in `?userfn`), and
            // counting them alike is what would make a bare body read as one with
            // an extra statement.
            if s.at(0) == Some(0x4B) {
                s.eh.trailer_stmts += 1;
            }
            s.eh.any = true;
            s.off_class("eh-trailer");
        }
        // `31` is NOT here, and that is a result rather than an omission —
        // `IL_CALL_GRAMMAR.md` §7 lists it as unidentified. An opcode whose
        // payload no capture has established refuses here, at itself, as
        // `cf-expr-0xNN`, and the row is then an honest measurement of what
        // establishing it would buy. (`64` and `67` used to be in this sentence.
        // A first cut of this file had given `67` a TYPE on the strength of the
        // shape of its neighbours, and that read failed at a non-tag byte in
        // 29,687 bodies — the *visible* half of a wrong width; the invisible half
        // is the bodies where the guess lands on a legal type and the walk
        // carries on desynchronized, for which there is no counter. Both are
        // decoded below now, each from a capture that separates its reading from
        // the plausible alternative.)
        // `44` — PAYLOAD-FREE. Witnessed twice (`44 30 …` and `44 55 …`), and the
        // byte after it has bit 7 clear at both sites, so it cannot be carrying a
        // TYPE. Its meaning is UNKNOWN ("materialize / bind" is the obvious guess
        // and nothing tests it); its width is not.
        0x44 => {
            s.p += 1;
            s.off_class("materialize-44");
        }
        // `28 00 00` — the subscript byte-offset add. The two trailing bytes are
        // `00 00` at every captured site and are NOT understood
        // (`docs/IL_EXPR_LAYER.md` §4.1), so anything else is not this token and
        // must refuse rather than be skipped as "two bytes of something".
        0x28 => {
            s.p += 1;
            if !eat(s.seg, &mut s.p, &[0x00, 0x00]) {
                return Err(blk(s.seg, s.p, "cf-subscript-payload"));
            }
            s.off_class("subscript");
        }
        // CONVERT `2C <TYPE target> <varint>`. Modeled only when the target is
        // inside the int4/ptr4 class — the class-preserving case `parse_expr`
        // admits because c2 emits nothing for it. A conversion OUT of the class is
        // a real instruction (`extsb`, `rlwinm`, `fctiwz`), so it is residue.
        0x2C => {
            s.p += 1;
            let (tag, kind) = s.ty("cf-convert-type")?;
            s.vint("cf-convert-payload")?;
            if !(is_int4_type(tag, kind) || is_ptr_to_4(tag, kind)) {
                s.off_class("convert-out-of-class");
            }
        }
        // INTRINSIC CALL `40 <TYPE result>` — no trailing field
        // (`docs/IL_INTRINSIC_CALL.md` §1).
        0x40 => {
            s.p += 1;
            s.ty("cf-intrinsic-type")?;
            s.off_class("intrinsic");
        }
        // `43 <sub-opcode> [payload]` — an ESCAPE, and the payload width is a
        // function of the sub-opcode (`docs/IL_EXPR_LAYER.md` §8): `42` (the
        // conditional expression) carries two bytes, `37` (a bitfield designator)
        // carries none. Only those two are witnessed, so every other sub-opcode
        // refuses — a fixed four-byte read desynchronizes on every bitfield in the
        // corpus, which is precisely the census name `expr-ternary` was a
        // generalization from.
        0x43 => {
            let Some(sub) = s.at(1) else {
                return Err(blk(s.seg, s.p, "cf-escape-43"));
            };
            match sub {
                0x42 => s.p += 4,
                0x37 => s.p += 2,
                _ => {
                    let seg_len = s.seg.len();
                    let ctx = "cf-escape-43";
                    return Err(Block { ctx, byte: Some(sub), off: s.p, seg_len, aux: 0 });
                }
            }
            if s.p > s.seg.len() {
                return Err(blk(s.seg, s.seg.len(), "cf-escape-43"));
            }
            s.off_class("ternary");
        }
        // `66 <n> <n tokens>` — the class-pair descriptor of the 2113–2119
        // intrinsic family. Its second byte is an ARITY, not the constant `02`
        // (`docs/IL_INTRINSIC_CALL.md` §4.3), and the tokens are LEB-width; the ONE
        // decoder for it is `mcall`'s, imported rather than restated.
        0x66 => {
            if eat_class_descriptor(s.seg, &mut s.p).is_none() {
                return Err(blk(s.seg, s.p, "cf-class-descriptor"));
            }
            s.off_class("class-descriptor");
        }
        // ---- virtual dispatch, and the by-value return (WDR) ----------------
        //
        // `67 <varint vtable-byte-offset> <token>` — VIRTUAL DISPATCH. The whole
        // production is `67 <slot> <method-tok>  B9 <recv>  30 <TYPE>  30 <TYPE>
        // 9A <TYPE>  BD … 4C`: load the receiver, load its vtable pointer, load
        // the slot, bind, call. It is why a `99`-bind site is direct dispatch by
        // construction (`docs/IL_CALL_IN_EXPR.md` §3).
        //
        // **The first field is a signed varint, and that is MEASURED, not
        // assumed.** Every witness anyone had was `00`, `04`, `08`, `0C`, `34`,
        // `38` — all below `0x80`, where a plain byte and a varint agree, so the
        // two readings were indistinguishable. `work/WDR/probe/p3.cpp` separates
        // them with a class carrying forty virtuals: calling the 33rd emits
        // `67 80 80 00 00 00 04 0A` — the varint escape, at byte offset 128. A
        // plain-byte reading desynchronizes on every class with more than 32
        // virtual functions, which in this corpus is 315 bodies per 838k.
        0x67 => {
            s.p += 1;
            s.vint("cf-virtual-slot")?;
            s.tok("cf-virtual-tok")?;
            s.off_class("virtual-slot");
        }
        // `9A <TYPE>` — the vtable-slot bind, the virtual sibling of `99`. Its
        // width is separated from `99`'s `<TYPE> <varint>` by the corpus and not
        // by analogy: a trailing varint swallows the `BD` that follows at every
        // site, and the byte after `BD` is a TYPE tag, which is not an operand
        // opcode. Measured over 837,830 bodies, `9A <TYPE>` decodes **13,024**
        // that `9A <TYPE> <varint>` does not, and the latter decodes none that
        // the former does not.
        //
        // It is listed here rather than with `99` because it is what makes `67`
        // worth anything: alone, decoding `67` moves the 45,631-body row to a
        // 45,631-body `cf-expr-0x9A` row two tokens later and the decode reach by
        // **zero** — the §6n rule that a first-blocker row is not a population,
        // arriving as a measured prediction rather than a surprise.
        0x9A => {
            s.p += 1;
            s.ty("cf-vbind-type")?;
            s.off_class("vbind");
        }
        // `64 <TYPE>` — the by-value return's temporary MATERIALIZE, in the same
        // syntactic slot a `BD` call occupies and closed by the same `4C`.
        // Reproduced from hand-written source (`work/WDR/probe/p2.cpp`, a
        // 27-function battery of which exactly one emits it):
        //
        //     void c_val(B* b) { b->Val(); }        // Val() returns a class BY VALUE
        //     26 <Val>  B9 <b> <B*>  99 <TYPE> 00  BD <A*> 00 <id>
        //     9B <aggregate TYPE> <tok>             bind the temporary
        //     2C <A*> 00                            its address
        //     64 <A*>  4C                           materialize into it
        //     30 <aggregate TYPE>  4B               and read it back
        //
        // No trailing field, on the model of `40 <TYPE result>`: `4C` is an
        // opcode of this grammar in its own right (it closes a call's operand
        // region) and is therefore not `64`'s payload. That distinction is very
        // nearly unobservable — a `<TYPE> <varint>` reading swallows the `4C` and
        // reaches the same tail — and the corpus separates the two by exactly
        // **one body in 837,830**. Recorded as such rather than dressed up: this
        // is the same standing as `99`'s trailing `00`, which is
        // INDISTINGUISHABLE from a constant and is documented that way.
        0x64 => {
            s.p += 1;
            s.ty("cf-materialize-type")?;
            s.off_class("materialize-64");
        }
        // `99 <TYPE> <varint>` member bind and `9B <TYPE> <token>` temporary bind.
        // **Adjacent opcodes with different trailing-field encodings**, and neither
        // is inferable from the other (`docs/IL_EXPR_LAYER.md` §7) — reading `9B`'s
        // as a varint is the desync that produced `IL_STMT_GRAMMAR.md` §12.4's one
        // real-TU scope-depth counterexample.
        0x99 => {
            s.p += 1;
            s.ty("cf-bind-type")?;
            s.vint("cf-bind-payload")?;
            s.off_class("bind");
        }
        0x9B => {
            s.p += 1;
            s.ty("cf-temp-type")?;
            s.tok("cf-temp-tok")?;
            s.off_class("temp");
        }
        // CALL `BD <ret TYPE> <cc> <varint fn-type-id>`, and the `4C` that ends an
        // argument list. 8–13 bytes, every field self-delimiting. In the modeled
        // vocabulary — the port lowers calls — but only at the one calling
        // convention it has been graded on: `00` is cdecl/stdcall, `04` fastcall
        // and `40` varargs need argument passing the port does not implement, and
        // admitting one as the other is a mis-emit rather than a gap.
        0xBD => {
            s.p += 1;
            s.ty("cf-call-ret-type")?;
            let cc = s.at(0);
            s.p += 1;
            s.vint("cf-call-fn-type-id")?;
            if cc != Some(0x00) {
                s.off_class("call-cc");
            }
        }
        // **The EH predicate lives HERE, on `4C`, not on the `BD` above.** `4C`
        // closes a call's argument list, and `BD … 4C` brackets the call: the
        // descriptor is emitted BEFORE the arguments are evaluated. So a
        // destructible temporary materialized by a nested call goes live *between*
        // the two, and only `4C` is on the right side of it.
        //
        // MEASURED, and it cost a wrong answer to find: `int t1(int a){ return
        // gp(mkSE().m) + a; }` emits `BD`(gp) … `BD`(mkSE) `4C` … `5C` …
        // `4C`(gp). Counting at `BD` puts gp's transfer at the empty live set and
        // calls the body cheap; its obj has an `__ehfuncinfo$` and `Value = 8`.
        // Counting at `4C` gets it right and gets all 46 probe functions right.
        // A destructible temporary is one of the four things `docs/EH_RECORDS.md`
        // §9.10 lists as never probed, which is why it was the cell that broke it.
        0x4C => {
            s.p += 1;
            if s.eh_live > 0 {
                s.eh.calls_live += 1;
            }
        }
        _ => return Err(blk(s.seg, s.p, "cf-expr")),
    }
    Ok(())
}

/// The literal payload of `33 <TYPE> …`, the type triple already consumed.
///
/// One copy of the rule `mcall::eat_literal_payload` also owns; it is restated here
/// rather than imported because that one is private to a walk with different
/// failure semantics (it returns `bool` and leaves the cursor unspecified), and a
/// scanner whose whole contract is "fail closed on an offset" cannot use a reader
/// that does not say where it stopped.
fn lit_payload(s: &mut Scan, tag: u8, kind: u8) -> Result<(), Block> {
    // A real literal: 8 IEEE bytes then a 2-byte LE size. Not a varint at all.
    if kind & 0x0F == 0xA {
        s.p += 10;
        return if s.p <= s.seg.len() {
            Ok(())
        } else {
            Err(blk(s.seg, s.seg.len(), "cf-lit-real"))
        };
    }
    match s.at(0) {
        // The escape. Its payload is 8 bytes for an 8-byte scalar and 4 otherwise —
        // the `read_varint` note that a tag-`0x88` escape is not 4 bytes.
        Some(0x80) => {
            s.p += 1 + if tag == 0x88 { 8 } else { 4 };
            if s.p <= s.seg.len() {
                Ok(())
            } else {
                Err(blk(s.seg, s.seg.len(), "cf-lit-escape"))
            }
        }
        Some(_) => {
            s.p += 1;
            Ok(())
        }
        None => Err(blk(s.seg, s.p, "cf-lit-payload")),
    }
}

#[cfg(test)]
// **`pub(crate)` so `super::step5`'s tests can cite these segments rather than
// transcribe them again.** Four of the consts below are the tree's only pinned
// real-capture witnesses of a `29`/`38`/`3A` body, and a second copy in another
// file is a second thing to keep true — the same reason `chain_skip_form`'s
// widths are called and never restated.
pub(crate) mod tests {
    use super::*;
    use crate::func::bundle::LO_MARKER;
    use crate::func::readers::find_subslice;

    fn scan(seg: &[u8]) -> Result<CfBody, Block> {
        let lo = find_subslice(seg, &LO_MARKER).expect("a body marker");
        scan_full(seg, lo).body
    }

    /// The **superseded** statement-count EH axis's reading of the same body:
    /// `(key, markers)`. Kept because §7.3's published split is keyed on it.
    fn eh(seg: &[u8]) -> (&'static str, EhMarkers) {
        let lo = find_subslice(seg, &LO_MARKER).expect("a body marker");
        let s = scan_full(seg, lo);
        (s.eh.key(s.decoded), s.eh)
    }

    /// The **measured** `maxState` EH axis's reading: `(key, markers)`.
    fn eh_state(seg: &[u8]) -> (&'static str, EhMarkers) {
        let lo = find_subslice(seg, &LO_MARKER).expect("a body marker");
        let s = scan_full(seg, lo);
        (s.eh.state_key(s.decoded), s.eh)
    }

    /// **[CF] `il_stmt_seq.cpp` `void stmt_seq0() {}`** — the smallest body there
    /// is, and the calibration for every shape below: one epilogue jump, one
    /// epilogue label, nothing else.
    pub(crate) const EMPTY: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x3A, 0xE5, 0x09, // jump epilogue
        0x54, 0x02, // close the body scope
        0x29, 0xE5, 0x09, // epilogue:
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail
    ];

    #[test]
    fn empty_body_is_one_basic_block() {
        assert_eq!(
            scan(EMPTY),
            Ok(CfBody { shape: CfShape::Straight, residue: CfResidue::Modeled })
        );
    }

    /// **[CF] `il_stmt_if_else.cpp` `void stmt_if_else(int a){ if(a) g(); else h(); }`**,
    /// transcribed byte for byte from `docs/IL_STMT_GRAMMAR.md` §7. The shape is a
    /// diamond: one conditional, forward only.
    pub(crate) const IF_ELSE: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0D, 0x53, //
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // load a
        0x38, 0xE8, 0x09, // brFALSE -> else
        0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4C,
        0x4B, // then: g();
        0x4F, 0x01, 0x0E, 0x54, 0x04, // close the then scope
        0x3A, 0xE9, 0x09, // jump join
        0x29, 0xE8, 0x09, // else:
        0x53, 0x26, 0xE4, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4C,
        0x4B, // else: h();
        0x54, 0x04, // close the else scope
        0x29, 0xE9, 0x09, // join:
        0x54, 0x03, 0x4F, 0x01, 0x0F, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn if_else_is_a_forward_diamond_blocked_on_control_flow_alone() {
        // Every operand in it — the `B9` int load, the two `26 <callee>` pushes,
        // the two cdecl `BD` calls — is one the port already lowers. So this body
        // is `+expr-modeled`: the ONLY thing between it and an obj is the CFG. That
        // is the population the block-IR restructure is worth, and separating it
        // from the bodies that also need expression work is the whole point of the
        // residue field.
        assert_eq!(
            scan(IF_ELSE),
            Ok(CfBody { shape: CfShape::Forward(1), residue: CfResidue::Modeled })
        );
    }

    /// …and the counter-case, which must NOT read `+expr-modeled`: the same
    /// diamond with one indirect load (`30 <TYPE>`) spliced into its condition.
    /// A residue that could not separate these two would report the block IR as
    /// worth every branching body in the corpus.
    #[test]
    fn one_unmodeled_operand_takes_a_body_out_of_expr_modeled() {
        let mut with_deref = IF_ELSE.to_vec();
        let load = with_deref
            .windows(3)
            .position(|w| w == [0xB9, 0xE5, 0x09])
            .expect("the condition load");
        with_deref.splice(load + 6..load + 6, [0x30, 0x86, 0x41, 0x74]);
        assert_eq!(
            scan(&with_deref),
            Ok(CfBody { shape: CfShape::Forward(1), residue: CfResidue::Expression })
        );
    }

    /// **[CF] `il_stmt_while.cpp` `void stmt_while_call(int a){ while(a){ g(); a=a-1; } }`**,
    /// §8.1. The `3A E8 09` at the end of the body targets the `29 E8 09` that
    /// opened it — a BACK edge, and `3A` carries no direction, so nothing but the
    /// recorded positions can tell.
    pub(crate) const WHILE: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0D, 0x53, //
        0x29, 0xE8, 0x09, // TOP:
        0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, 0x38, 0xE9, 0x09, // brFALSE -> EXIT
        0x53, 0x4F, 0x01, 0x0E, //
        0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4C,
        0x4B, // g();
        0x4F, 0x01, 0x0F, //
        0x26, 0xE4, 0x09, 0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x03,
        0x32, 0x86, 0x41, 0x74, 0x4B, // a = a - 1;
        0x4F, 0x01, 0x10, 0x54, 0x04, //
        0x3A, 0xE8, 0x09, // jump TOP  <- the back edge
        0x29, 0xE9, 0x09, // EXIT:
        0x54, 0x03, 0x4F, 0x01, 0x11, 0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn while_is_a_loop_by_its_back_edge() {
        assert_eq!(
            scan(WHILE),
            Ok(CfBody { shape: CfShape::Loop, residue: CfResidue::Modeled })
        );
    }

    /// **[CF] `il_stmt_early_return.cpp` `int stmt_early_int(int a){ if(a) return 1; return 2; }`**,
    /// §9. Two returns, ONE epilogue label — the pattern that makes `MultiExit`
    /// worth separating from `Forward`, since here the second exit rides on a
    /// conditional and elsewhere it does not.
    pub(crate) const EARLY_RETURN: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0E, 0x53, //
        0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, 0x38, 0xE7, 0x09, //
        0x53, 0x33, 0x86, 0x41, 0x74, 0x01, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE6,
        0x09, // return 1;
        0x4F, 0x01, 0x0F, 0x54, 0x04, 0x29, 0xE7, 0x09, //
        0x54, 0x03, //
        0x33, 0x86, 0x41, 0x74, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE6, 0x09, // return 2;
        0x4F, 0x01, 0x10, 0x54, 0x02, 0x29, 0xE6, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn early_return_is_forward_with_two_jumps_to_one_label() {
        assert_eq!(
            scan(EARLY_RETURN),
            Ok(CfBody { shape: CfShape::Forward(1), residue: CfResidue::Modeled })
        );
    }

    /// A wrong `54 <k>` is the falsification test, and it must fire. Corrupting the
    /// depth operand of `EMPTY`'s single close is the minimal witness: every field
    /// width is still correct, so nothing but the invariant can catch it.
    #[test]
    fn a_wrong_scope_depth_refuses_rather_than_landing_on_the_tail() {
        let mut bad = EMPTY.to_vec();
        let close = bad.windows(2).position(|w| w == [0x54, 0x02]).expect("the body close");
        bad[close + 1] = 0x03;
        let err = scan(&bad).expect_err("a depth mismatch must refuse");
        assert_eq!(err.ctx, "cf-scope-depth");
    }

    /// …and so must a body that stops in the right neighbourhood for the wrong
    /// reason. Truncating the tail by one byte leaves the statement loop ending in
    /// exactly the same place; only requirement (1) separates them.
    #[test]
    fn landing_near_the_tail_is_not_landing_on_it() {
        let mut bad = EMPTY.to_vec();
        bad.pop();
        let err = scan(&bad).expect_err("a short tail must refuse");
        assert_eq!(err.ctx, "cf-tail");
    }

    /// `43` is an escape whose payload width is a function of its sub-opcode. A
    /// fixed four-byte read desynchronizes on every bitfield read in the corpus, so
    /// both witnessed widths are pinned and an unwitnessed sub-opcode refuses
    /// rather than being skipped at a guessed width.
    #[test]
    fn the_43_escape_is_sub_opcode_width_and_unknown_sub_opcodes_refuse() {
        // `43 42 00 00` (conditional) inside an otherwise empty body.
        let mut cond = EMPTY.to_vec();
        cond.splice(4..4, [0x43, 0x42, 0x00, 0x00]);
        assert_eq!(scan(&cond).map(|b| b.shape), Ok(CfShape::Straight));
        // `43 37` (bitfield designator) carries nothing.
        let mut bits = EMPTY.to_vec();
        bits.splice(4..4, [0x43, 0x37]);
        assert_eq!(scan(&bits).map(|b| b.shape), Ok(CfShape::Straight));
        // …and an unwitnessed sub-opcode is an honest refusal.
        let mut other = EMPTY.to_vec();
        other.splice(4..4, [0x43, 0x11, 0x00, 0x00]);
        let err = scan(&other).expect_err("an unwitnessed 43 sub-opcode must refuse");
        assert_eq!(err.ctx, "cf-escape-43");
    }

    /// The census rename is **1:1**: seven control-flow bytes, seven names, no two
    /// the same, and nothing outside the set renamed. A rename that merged two
    /// buckets would silently invalidate every recorded comparison in
    /// `docs/rungs/`, which is the one thing a key change must not do.
    #[test]
    fn the_control_flow_rename_is_one_to_one() {
        use crate::func::body::cflow_opcode_name;
        let mut names: Vec<&str> = Vec::new();
        for b in 0..=0xFFu8 {
            match cflow_opcode_name(b) {
                Some(n) => {
                    assert!(!names.contains(&n), "two opcodes share the name {n}");
                    names.push(n);
                }
                None => {}
            }
        }
        assert_eq!(names.len(), 7, "the statement layer has exactly seven: {names:?}");
        // …and the neighbours that are NOT control flow keep their hex, including
        // the two the grammar deliberately leaves alone: `3E`/`3F` are unwitnessed
        // and `2C`/`32` are expression-layer tokens that happen to sit next to the
        // range.
        for b in [0x28, 0x2C, 0x32, 0x37, 0x3E, 0x3F, 0x40, 0x41] {
            assert!(cflow_opcode_name(b).is_none(), "0x{b:02X} is not control flow");
        }
    }

    /// …and the key a real refusal renders. Two productions blocked on the *same*
    /// byte must stay two buckets, because the production they interrupted is
    /// different work: `body-cflow-label` is a `do`/`while`'s top label and
    /// `return-scope-close-cflow-label` is one met in the return plumbing.
    #[test]
    fn the_same_byte_in_two_productions_stays_two_buckets() {
        // `seg_len: 1` — a block that HAS a blocking byte at offset 0 came from a
        // segment with at least that byte in it. The rendering below does not
        // consult it (the byte path never can), but the block still has to be one
        // the parser could have produced.
        let label = |ctx| Block { ctx, byte: Some(0x29), off: 0, seg_len: 1, aux: 0 }.feature();
        assert_eq!(label("body"), "body-cflow-label");
        assert_eq!(label("return-scope-close"), "return-scope-close-cflow-label");
        assert_ne!(label("body"), label("return-scope-close"));
        // The `expr` production renders through its own table first and falls
        // through to this one, so a branch met as an operand is `expr-brfalse`
        // rather than `expr-cflow-brfalse` — one prefix per production, as every
        // other `expr-*` key has.
        assert_eq!(
            Block { ctx: "expr", byte: Some(0x38), off: 0, seg_len: 1, aux: 0 }.feature(),
            "expr-brfalse"
        );
        assert_eq!(
            Block { ctx: "call-ref", byte: Some(0x3A), off: 0, seg_len: 1, aux: 0 }.feature(),
            "call-ref-cflow-jump"
        );
    }

    // ---- the EH axis (WEH) --------------------------------------------------
    //
    // Five bodies, transcribed from two probes captured at the **workload's own**
    // flags (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`), never the fixture
    // profile — `docs/EH_RECORDS.md` §6.1 measured that the fixture profile
    // understates this exact production by a phase. Each one's obj was checked for
    // an `__ehfuncinfo$` and the axis agrees with the obj in every case; the whole
    // table is `docs/EH_RECORDS.md` §7.

    /// **[P] `work/WEH/probe/p1.cpp` `struct One { ~One(); MemA m; };  One::~One(){}`**
    /// — ONE sub-object statement and nothing else. `.text` is `b ??1MemA`, four
    /// bytes, and the obj carries **no** `__ehfuncinfo$`. The cheap side.
    const EH_ONE: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xE5, 0x09, 0xB9, 0x0A,
        0x0A, 0xA6, 0x43, 0x81, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0xA6, 0x43, 0x8B,
        0x20, 0x2C, 0xA6, 0x43, 0x8C, 0x20, 0x00, 0x99, 0x86, 0x43, 0x8E, 0x20, 0x00, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x0E, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0x86, 0x41, 0x74,
        0x01, 0x4B, 0x3A, 0x0B, 0x0A, 0x54, 0x02, 0x29, 0x0B, 0x0A, 0x5E, 0x01, 0x21, 0x4B,
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[P] `p1.cpp` `struct Two { ~Two(); MemA m; MemB n; };  Two::~Two(){}`** —
    /// TWO sub-object statements, `5E 02 21`, and an `__ehfuncinfo$??1Two@@QAA@XZ`
    /// in the obj.
    const EH_TWO: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xE5, 0x09, 0xB9, 0x1A,
        0x0A, 0xA6, 0x43, 0x93, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0xA6, 0x43, 0x8B,
        0x20, 0x2C, 0xA6, 0x43, 0x8C, 0x20, 0x00, 0x99, 0x86, 0x43, 0x8E, 0x20, 0x00, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x0E, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0x86, 0x41, 0x74,
        0x01, 0x4B, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xF1, 0x09, 0xB9, 0x1A, 0x0A, 0xA6,
        0x43, 0x93, 0x20, 0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0xA6, 0x43, 0x9B, 0x20, 0x2C,
        0xA6, 0x43, 0x9C, 0x20, 0x00, 0x99, 0x86, 0x43, 0x9E, 0x20, 0x00, 0xBD, 0x82, 0x07,
        0x03, 0x00, 0x80, 0x1E, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0x86, 0x41, 0x74, 0x01, 0x4B,
        0x3A, 0x1B, 0x0A, 0x54, 0x02, 0x29, 0x1B, 0x0A, 0x5E, 0x02, 0x21, 0x4B, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[P] `p1.cpp` `struct OneB { ~OneB(); void Fini(); MemA m; };  OneB::~OneB(){ Fini(); }`**
    /// — ONE sub-object statement, `5E 01 21`, **plus one body statement**, and an
    /// `__ehfuncinfo$??1OneB@@QAA@XZ`. This is the census key
    /// `expr-call-in-expr-recv-field-off0-then-chain-bind-whole` that
    /// `docs/EH_RECORDS.md` §6.2 sized at 2,666 functions.
    const EH_ONEB: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xE5, 0x09, 0xB9, 0x2A,
        0x0A, 0xA6, 0x43, 0xA3, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0xA6, 0x43, 0x8B,
        0x20, 0x2C, 0xA6, 0x43, 0x8C, 0x20, 0x00, 0x99, 0x86, 0x43, 0x8E, 0x20, 0x00, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x0E, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0x86, 0x41, 0x74,
        0x01, 0x4B, 0x26, 0x1E, 0x0A, 0xB9, 0x2A, 0x0A, 0xA6, 0x43, 0xA3, 0x20, 0x2C, 0xA6,
        0x43, 0xA3, 0x20, 0x00, 0x99, 0x86, 0x43, 0xA4, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x24, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0x2B, 0x0A, 0x54, 0x02, 0x29,
        0x2B, 0x0A, 0x5E, 0x01, 0x21, 0x4B, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[P] `p1.cpp` `int userfn(int a){ MemA s; g(a); return a + 1; }`** — and the
    /// reason this axis is not a ctor/dtor axis. An ordinary user function, no
    /// sub-object anywhere, and it carries a `5C` because a destructible local
    /// became live. Its obj has an `__ehfuncinfo$?userfn@@YAHH@Z`.
    ///
    /// It is also the witness for the trailer's two positions: `5E 01 01` here
    /// stands before a `44`, not a `4B`, so it is an operand of the statement it
    /// sits in rather than a statement of its own — and `5D 01 80 A1 00 00 00`
    /// is the escaped varint that rules out reading the second field as a byte.
    const EH_USERFN: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0x26, 0x58, 0x0A, 0x2C, 0xA6, 0x43, 0x8C,
        0x20, 0x00, 0x99, 0x86, 0x43, 0x8E, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x8C, 0x20, 0x00,
        0x80, 0x0D, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE5, 0x09, 0x26, 0x58, 0x0A, 0x2C, 0xA6,
        0x43, 0x8C, 0x20, 0x00, 0x99, 0x86, 0x43, 0x8E, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x0E, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0xA6, 0x43, 0x8C, 0x20, 0x01, 0x4B,
        0x26, 0xFC, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x40, 0x10, 0x00, 0x00, 0xB9,
        0x55, 0x0A, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x4B, 0x9B, 0x86, 0x41,
        0x74, 0x59, 0x0A, 0xB9, 0x55, 0x0A, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01,
        0x02, 0x32, 0x86, 0x41, 0x74, 0x5E, 0x01, 0x01, 0x44, 0x9B, 0x86, 0x41, 0x74, 0x59,
        0x0A, 0x30, 0x86, 0x41, 0x74, 0x44, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x57, 0x0A, 0x5D,
        0x01, 0x80, 0xA1, 0x00, 0x00, 0x00, 0x4B, 0x54, 0x02, 0x29, 0x57, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[P] `work/WEH/probe/p2.cpp` `void onlylocal(){ MemA s; }`** — the other
    /// half of the same point. An ordinary function, one destructible object, no
    /// other statement, and **no `__ehfuncinfo$`**: the cheap side is not a
    /// property of being a generated destructor, it is a property of the count.
    const EH_ONLYLOCAL: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0x26, 0x10, 0x0A, 0x2C, 0xA6, 0x43, 0x8C,
        0x20, 0x00, 0x99, 0x86, 0x43, 0x8E, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x8C, 0x20, 0x00,
        0x80, 0x0D, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE5, 0x09, 0x26, 0x10, 0x0A, 0x2C, 0xA6,
        0x43, 0x8C, 0x20, 0x00, 0x99, 0x86, 0x43, 0x8E, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x0E, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0xA6, 0x43, 0x8C, 0x20, 0x01, 0x4B,
        0x5E, 0x01, 0x21, 0x4B, 0x3A, 0x0F, 0x0A, 0x54, 0x02, 0x29, 0x0F, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **The axis, graded against the obj.** Every one of these five was compiled
    /// at the workload's flags and its obj inspected for an `__ehfuncinfo$`; the
    /// right-hand column is that inspection, not a prediction.
    #[test]
    fn the_eh_axis_agrees_with_whether_the_obj_carries_an_eh_record() {
        for (seg, want, ehfuncinfo, what) in [
            (EH_ONE, "eh-bare", false, "one sub-object, nothing else"),
            (EH_ONLYLOCAL, "eh-bare", false, "one destructible local, nothing else"),
            (EH_TWO, "eh-multi", true, "two sub-objects"),
            (EH_ONEB, "eh-plus-stmt", true, "one sub-object plus a body statement"),
            (EH_USERFN, "eh-plus-stmt", true, "a destructible local plus two statements"),
            (EMPTY, "eh-none", false, "no destructible object at all"),
        ] {
            let (key, _) = eh(seg);
            assert_eq!(key, want, "{what}");
            assert_eq!(
                key != "eh-bare" && key != "eh-none",
                ehfuncinfo,
                "{what}: the axis and the obj must agree about the EH record"
            );
        }
    }

    /// The counts behind the key, so a future change that keeps the key by luck
    /// still has to keep the arithmetic. `EH_ONE` and `EH_ONEB` differ by exactly
    /// one statement and nothing else — which is the boundary, in two numbers.
    #[test]
    fn the_boundary_is_one_statement_wide() {
        let (_, one) = eh(EH_ONE);
        assert_eq!((one.live_stmts, one.count, one.other_stmts()), (1, 1, 0));
        let (_, oneb) = eh(EH_ONEB);
        assert_eq!((oneb.live_stmts, oneb.count, oneb.other_stmts()), (1, 1, 1));
        let (_, two) = eh(EH_TWO);
        assert_eq!((two.live_stmts, two.count, two.other_stmts()), (2, 2, 0));
        // …and the trailer in operand position must not be counted as a statement:
        // `userfn` has three `4B`, one `5C`, and only ONE of its two `5D`/`5E`
        // trailers stands before a `4B`.
        let (_, u) = eh(EH_USERFN);
        assert_eq!((u.live_stmts, u.count, u.stmt_ends, u.trailer_stmts), (1, 1, 3, 1));
        assert_eq!(u.other_stmts(), 1);
    }

    /// **`eh-partial` is a positive claim, not an absence.** A body that carries a
    /// marker and then stops decoding is certainly not bare, because the bare shape
    /// decodes end to end by construction — which is what makes the undecoded
    /// residue of `cf-expr-0x5C` rankable at all rather than a hole in the census.
    #[test]
    fn a_marker_then_a_stop_is_not_bare() {
        // Splice an opcode this scanner refuses (`07`, unestablished — it sits in
        // the gap `IL_STMT_GRAMMAR.md` §5 leaves between `%` = `06` and the
        // shifts at `09`) after the sub-object statement's `4B`. This was `64`
        // until WDR established it and `05` until `lane w-divsplit` did (board
        // **#820**); the substitution is 1:1 in what the test asserts, which is
        // that SOME refusal after a marker reads `eh-partial`. A test whose
        // stand-in keeps getting established is a table that keeps growing.
        let mut seg = EH_ONE.to_vec();
        let at = seg
            .windows(6)
            .position(|w| w == [0x5C, 0x86, 0x41, 0x74, 0x01, 0x4B])
            .expect("the statement trailer")
            + 6;
        seg.splice(at..at, [0x07]);
        assert_eq!(eh(&seg).0, "eh-partial");
        // …and a body that stops BEFORE any marker claims nothing.
        let mut early = EH_ONE.to_vec();
        early.splice(4..4, [0x07]);
        assert_eq!(eh(&early).0, "eh-unknown");
    }

    // ---- the maxState axis (EHMS) -------------------------------------------
    //
    // `docs/EH_RECORDS.md` §9.4 refuted the statement-count predicate the five
    // bodies above are graded on, and §10 re-derived the axis on the measured one
    // (`maxState >= 1` iff a transfer occurs while a destructible object is live).
    // These four are the cells where the two predicates DISAGREE — the ones §7.2
    // did not contain, which is why it passed its own grading. All four were
    // compiled at the workload's own flags and their objs read for an
    // `__ehfuncinfo$` and the function symbol's `Value`; the expected column is
    // that reading, never a prediction. `work/EHMS/probe/m1.cpp`, `m3.cpp`.

    /// **[P] `m1.cpp` `int mA(int a){ SE s; int x=a*3; int y=x^7; return y+1; }`**
    /// — one object and THREE more statements, none of which calls anything.
    /// `maxState = 0`: no prefix, one `.pdata`, no `.rdata`, no funclet, symbol
    /// `Value = 0`. The statement rule calls this `eh-plus-stmt` and files a
    /// **cheap** body on the expensive side.
    const EH_ST_PLAINSTMTS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0x26, 0x01, 0x0A, 0x2C, 0xA6, 0x43, 0x81,
        0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x81, 0x20, 0x00,
        0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE6, 0x09, 0x26, 0x01, 0x0A, 0x2C, 0xA6,
        0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01, 0x4B,
        0x26, 0x02, 0x0A, 0xB9, 0xFE, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x03,
        0x04, 0x32, 0x86, 0x41, 0x74, 0x4B, 0x26, 0x03, 0x0A, 0xB9, 0x02, 0x0A, 0x86, 0x41,
        0x74, 0x33, 0x86, 0x41, 0x74, 0x07, 0x0D, 0x32, 0x86, 0x41, 0x74, 0x4B, 0x9B, 0x86,
        0x41, 0x74, 0x04, 0x0A, 0xB9, 0x03, 0x0A, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74,
        0x01, 0x02, 0x32, 0x86, 0x41, 0x74, 0x5E, 0x01, 0x01, 0x44, 0x9B, 0x86, 0x41, 0x74,
        0x04, 0x0A, 0x30, 0x86, 0x41, 0x74, 0x44, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x00, 0x0A,
        0x5D, 0x01, 0x80, 0xA1, 0x00, 0x00, 0x00, 0x4B, 0x54, 0x02, 0x29, 0x00, 0x0A, 0x4F,
        0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[P] `m1.cpp` `int mC(int a){ SE s; return gp(a); }`** — one object and NO
    /// other `4B` statement, because a `return` carries none
    /// (`docs/IL_STMT_GRAMMAR.md` §9). The statement rule calls it `eh-bare` and
    /// files an **EH** body on the cheap side: its obj has an
    /// `__ehfuncinfo$?mC@@YAHH@Z`, `maxState = 1`, two ip2state entries, symbol
    /// `Value = 8`. This is §9.4's `qB2`, and it is the hole in the "cheap side is
    /// a lower bound" reading.
    const EH_ST_RETCALL: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0x26, 0x08, 0x0A, 0x2C, 0xA6, 0x43, 0x81,
        0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x81, 0x20, 0x00,
        0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE6, 0x09, 0x26, 0x08, 0x0A, 0x2C, 0xA6,
        0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01, 0x4B,
        0x9B, 0x86, 0x41, 0x74, 0x09, 0x0A, 0x26, 0xFC, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00,
        0x80, 0x0A, 0x10, 0x00, 0x00, 0xB9, 0x05, 0x0A, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41,
        0x74, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x5E, 0x01, 0x01, 0x44, 0x9B, 0x86, 0x41, 0x74,
        0x09, 0x0A, 0x30, 0x86, 0x41, 0x74, 0x44, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x07, 0x0A,
        0x5D, 0x01, 0x80, 0xA1, 0x00, 0x00, 0x00, 0x4B, 0x54, 0x02, 0x29, 0x07, 0x0A, 0x4F,
        0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[P] `m1.cpp` `int mF(int a){ { SE s; } { SE t; } return a+1; }`** — TWO
    /// objects, and no EH record at all, because their lifetimes do not overlap:
    /// the second scope's constructor is a transfer at the EMPTY live set. The
    /// statement rule calls it `eh-multi`. This is the cell that makes the
    /// `5D`/`5E` decrement load-bearing — without it the second `4C` reads as a
    /// transfer at a live object and the body lands on the expensive side.
    const EH_ST_SCOPES: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x53, 0x26, 0xE5, 0x09, 0x26, 0x13, 0x0A, 0x2C, 0xA6, 0x43,
        0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x81, 0x20,
        0x00, 0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE6, 0x09, 0x26, 0x13, 0x0A, 0x2C,
        0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07,
        0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01,
        0x4B, 0x5E, 0x01, 0x21, 0x4B, 0x54, 0x03, 0x53, 0x26, 0xE5, 0x09, 0x26, 0x14, 0x0A,
        0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0xA6,
        0x43, 0x81, 0x20, 0x00, 0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE6, 0x09, 0x26,
        0x14, 0x0A, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0xA6, 0x43,
        0x81, 0x20, 0x01, 0x4B, 0x5E, 0x01, 0x21, 0x4B, 0x54, 0x03, 0xB9, 0x10, 0x0A, 0x86,
        0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x12,
        0x0A, 0x54, 0x02, 0x29, 0x12, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[P] `m3.cpp` `int t1(int a){ return gp(mkSE().m) + a; }`** — a
    /// destructible **temporary**, one of the four shapes `docs/EH_RECORDS.md`
    /// §9.10 lists as never probed. Its obj has an `__ehfuncinfo$?t1@@YAHH@Z`.
    ///
    /// It is here because it is the cell that moved the counting site. The stream
    /// is `BD`(gp) … `BD`(mkSE) `4C` … `5C` … `4C`(gp): the call descriptor is
    /// emitted before the arguments are evaluated, so the temporary goes live
    /// *between* gp's `BD` and gp's `4C`. Counting transfers at `BD` calls this
    /// body cheap and is wrong; counting at `4C` is right.
    const EH_ST_TEMP: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x9B, 0x86, 0x41, 0x74, 0x3D, 0x0A, 0x26, 0xFC, 0x09, 0xBD,
        0x86, 0x41, 0x74, 0x00, 0x80, 0x0B, 0x10, 0x00, 0x00, 0x26, 0xFE, 0x09, 0xBD, 0x86,
        0x43, 0xA5, 0x20, 0x00, 0x80, 0x26, 0x10, 0x00, 0x00, 0x9B, 0x86, 0x46, 0x80, 0x20,
        0x3C, 0x0A, 0x2C, 0x86, 0x43, 0xA5, 0x20, 0x00, 0x64, 0x86, 0x43, 0xA5, 0x20, 0x4C,
        0x26, 0xE6, 0x09, 0x9B, 0x86, 0x46, 0x80, 0x20, 0x3C, 0x0A, 0x2C, 0x86, 0x43, 0xA5,
        0x20, 0x00, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0x86, 0x43,
        0xA5, 0x20, 0x03, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0xF4, 0x08, 0x30,
        0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0xB9, 0x39, 0x0A, 0x86, 0x41, 0x74,
        0x02, 0x32, 0x86, 0x41, 0x74, 0x5E, 0x01, 0x23, 0x44, 0x9B, 0x86, 0x41, 0x74, 0x3D,
        0x0A, 0x30, 0x86, 0x41, 0x74, 0x44, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x3B, 0x0A, 0x54,
        0x02, 0x29, 0x3B, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **The maxState axis, graded against the obj — including the five bodies the
    /// statement axis was graded on.** The `ehfuncinfo` column is what the obj
    /// carries, read with `scripts/gt_eh.py`, not a prediction.
    ///
    /// The last column is the OTHER axis's answer, and four of the ten rows have
    /// the two disagreeing. That is the measurement: the statement rule is wrong
    /// in **both** directions, and the confirming cells of §7.2 could not have
    /// shown it because both rules agree on every one of them.
    #[test]
    fn the_maxstate_axis_agrees_with_whether_the_obj_carries_an_eh_record() {
        for (seg, want, ehfuncinfo, stmt_says, what) in [
            // the five §7.2 cells — both rules agree, so these confirm only
            (EH_ONE, "eh-state0", false, "eh-bare", "one sub-object, nothing else"),
            (EH_ONLYLOCAL, "eh-state0", false, "eh-bare", "one local, nothing else"),
            (EH_TWO, "eh-state1", true, "eh-multi", "two sub-objects"),
            (EH_ONEB, "eh-state1", true, "eh-plus-stmt", "sub-object plus a body call"),
            (EH_USERFN, "eh-state1", true, "eh-plus-stmt", "a local plus a call"),
            (EMPTY, "eh-none", false, "eh-none", "no destructible object at all"),
            // …and the four where they disagree, which is the whole point
            (EH_ST_PLAINSTMTS, "eh-state0", false, "eh-plus-stmt", "object + call-free statements"),
            (EH_ST_RETCALL, "eh-state1", true, "eh-bare", "object + a call in the return"),
            (EH_ST_SCOPES, "eh-state0", false, "eh-multi", "two objects, disjoint scopes"),
            (EH_ST_TEMP, "eh-state1", true, "eh-bare", "a destructible temporary"),
        ] {
            let (key, _) = eh_state(seg);
            assert_eq!(key, want, "{what}");
            assert_eq!(
                key == "eh-state1",
                ehfuncinfo,
                "{what}: the axis and the obj must agree about the EH record"
            );
            assert_eq!(eh(seg).0, stmt_says, "{what}: the superseded axis is unmoved");
        }
    }

    /// …and the counts behind it, so a change that keeps the key by luck still has
    /// to keep the arithmetic. The old axis orders `EH_ST_PLAINSTMTS` and
    /// `EH_ST_RETCALL` the wrong way round — 2 other statements against 0, and it
    /// is the one with **0** that carries the whole EH record set — while the new
    /// one separates them by exactly one transfer at a live object.
    #[test]
    fn the_boundary_is_one_transfer_wide() {
        assert_eq!(eh_state(EH_ST_PLAINSTMTS).1.calls_live, 0);
        assert_eq!(eh_state(EH_ST_RETCALL).1.calls_live, 1);
        assert_eq!(eh_state(EH_ST_SCOPES).1.calls_live, 0);
        assert_eq!(eh_state(EH_ST_TEMP).1.calls_live, 1);
        // The old axis reads them the other way round, which is the refutation in
        // two numbers.
        assert_eq!(eh(EH_ST_PLAINSTMTS).1.other_stmts(), 2);
        assert_eq!(eh(EH_ST_RETCALL).1.other_stmts(), 0);
    }

    /// **A transfer already seen at a live object is not un-seen by a later
    /// stop.** Splice a refused opcode after the call in `EH_ST_RETCALL` and the
    /// body stops decoding — but `maxState >= 1` was already proven, so it stays
    /// `eh-state1` rather than falling back to `eh-partial`. The converse holds
    /// too: stopping before the proof leaves `eh-partial`, which claims nothing.
    #[test]
    fn a_proven_state_survives_an_undecoded_tail() {
        let mut seg = EH_ST_RETCALL.to_vec();
        let at = seg.windows(4).position(|w| w == [0x5E, 0x01, 0x01, 0x44]).expect("the trailer");
        seg.splice(at..at, [0x07]);
        assert!(scan(&seg).is_err(), "the splice must stop the walk");
        assert_eq!(eh_state(&seg).0, "eh-state1");
        // …and a stop between the marker and any call claims nothing.
        let mut early = EH_ST_RETCALL.to_vec();
        let at = early
            .windows(6)
            .position(|w| w == [0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01])
            .expect("the 5C")
            + 7;
        early.splice(at..at, [0x07]);
        assert_eq!(eh_state(&early).0, "eh-partial");
    }

    /// **The widths are decoded, not guessed, and the corpus rules out both
    /// shortcuts.**
    ///
    /// * The `5C` TYPE is a real type of varying width — `86 41 74` in `??1One`,
    ///   `A6 43 8C 20` in `?userfn` — so a fixed read is impossible by inspection.
    /// * The trailers' state field escapes: `5D 01 80 A1 00 00 00` in `?userfn`.
    ///   Dropping the `80` marker leaves the walk standing on a `00`, which is not
    ///   a token this scanner knows, and it refuses instead of wandering.
    ///
    /// The standing falsification is bigger than this test and is what the widths
    /// actually rest on: the walk must land exactly on the seven-byte function
    /// tail with every `54 <k>` depth agreeing, over the whole workload.
    #[test]
    fn the_trailer_widths_are_decoded_not_guessed() {
        assert!(EH_ONE.windows(4).any(|w| w == [0x5C, 0x86, 0x41, 0x74]));
        assert!(EH_USERFN.windows(5).any(|w| w == [0x5C, 0xA6, 0x43, 0x8C, 0x20]));
        assert!(scan(EH_ONE).is_ok() && scan(EH_USERFN).is_ok());
        let mut bad = EH_USERFN.to_vec();
        let at = bad
            .windows(7)
            .position(|w| w == [0x5D, 0x01, 0x80, 0xA1, 0x00, 0x00, 0x00])
            .expect("the 5D trailer");
        bad.remove(at + 2); // the escape marker
        assert!(scan(&bad).is_err(), "a mis-read state field must not reach the tail");
    }

    // ---- WDR: virtual dispatch (`67`, `9A`) and the by-value return (`64`) ---
    //
    // Every constant below is transcribed byte for byte from a capture at the
    // 878-TU workload's own flags (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi
    // /EHsc`), `work/WDR/probe/p1.cpp` and `p3.cpp`.

    /// **[WDR p1] `struct W { virtual int A(); … }; int w_a(W* p){ return p->A(); }`**
    /// — virtual slot 0. The whole production: dispatch token, receiver, two
    /// indirect loads (vtable pointer, then the slot), the `9A` bind, the call.
    const VIRT_SLOT0: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, //
        0x67, 0x00, 0x01, 0x0A, // virtual dispatch, vtable byte offset 0
        0xB9, 0x0E, 0x0A, 0x86, 0x43, 0x9B, 0x20, // load p
        0x30, 0xA6, 0x43, 0x9E, 0x20, // -> the vtable pointer
        0x30, 0x86, 0x43, 0x99, 0x20, // -> the slot
        0x9A, 0x86, 0x43, 0x9F, 0x20, // bind it
        0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x1F, 0x10, 0x00, 0x00, 0x4C, //
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0x10, 0x0A, 0x54, 0x02, 0x29, 0x10, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[WDR p3] `int w31(Wide* p){ return p->v31(); }`** — the 32nd virtual, at
    /// byte offset `0x7C`. The last slot whose offset fits the varint short form.
    const VIRT_SLOT31: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, //
        0x67, 0x7C, 0x03, 0x0A, //
        0xB9, 0x16, 0x0A, 0x86, 0x43, 0x81, 0x20, //
        0x30, 0xA6, 0x43, 0x85, 0x20, 0x30, 0x86, 0x43, 0x94, 0x20, //
        0x9A, 0x86, 0x43, 0x86, 0x20, //
        0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, //
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0x18, 0x0A, 0x54, 0x02, 0x29, 0x18, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[WDR p3] `int w32(Wide* p){ return p->v32(); }`** — the 33rd virtual, at
    /// byte offset `0x80`, and **the separator this whole reading rests on**:
    /// `67 80 80 00 00 00`. Every witness before this file existed was below
    /// `0x80`, where a plain byte and a signed varint are indistinguishable.
    const VIRT_SLOT32: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, //
        0x67, 0x80, 0x80, 0x00, 0x00, 0x00, 0x04, 0x0A, // slot offset 128, escaped
        0xB9, 0x19, 0x0A, 0x86, 0x43, 0x81, 0x20, //
        0x30, 0xA6, 0x43, 0x85, 0x20, 0x30, 0x86, 0x43, 0x94, 0x20, //
        0x9A, 0x86, 0x43, 0x86, 0x20, //
        0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, //
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0x1B, 0x0A, 0x54, 0x02, 0x29, 0x1B, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[WDR p3] `void b_discard(Src* s){ s->Make(); }`**, `Src::Make()` returning
    /// a three-int class **by value**. The `64` production, whole: the call, the
    /// `9B` temporary, its address, `64 <TYPE>`, the `4C`, and the read-back.
    const BYVAL: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0x34, 0x0A, // push Make
        0xB9, 0x44, 0x0A, 0x86, 0x43, 0x96, 0x20, // load s
        0x99, 0x86, 0x43, 0x9B, 0x20, 0x00, // member bind
        0xBD, 0x86, 0x43, 0xA9, 0x20, 0x00, 0x80, 0x1B, 0x10, 0x00, 0x00, //
        0x9B, 0x86, 0xC6, 0x99, 0x20, 0x47, 0x0A, // bind the temporary
        0x2C, 0x86, 0x43, 0xA9, 0x20, 0x00, // its address
        0x64, 0x86, 0x43, 0xA9, 0x20, // MATERIALIZE
        0x4C, //
        0x30, 0x86, 0xC6, 0x99, 0x20, 0x4B, // read it back, end of statement
        0x3A, 0x46, 0x0A, 0x54, 0x02, 0x29, 0x46, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **[WDR p3] `void b_virt(Src* s){ s->VMake(); }`** — a **virtual** call
    /// returning by value, so `67`, `9A` and `64` all appear in one body. The
    /// composition is the check: three independently established widths that have
    /// to agree on one cursor to reach the tail.
    const BYVAL_VIRT: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, //
        0x67, 0x00, 0x35, 0x0A, //
        0xB9, 0x5A, 0x0A, 0x86, 0x43, 0x96, 0x20, //
        0x30, 0xA6, 0x43, 0x9A, 0x20, 0x30, 0x86, 0x43, 0x94, 0x20, //
        0x9A, 0x86, 0x43, 0x9B, 0x20, //
        0xBD, 0x86, 0x43, 0xA9, 0x20, 0x00, 0x80, 0x1B, 0x10, 0x00, 0x00, //
        0x9B, 0x86, 0xC6, 0x99, 0x20, 0x5D, 0x0A, //
        0x2C, 0x86, 0x43, 0xA9, 0x20, 0x00, //
        0x64, 0x86, 0x43, 0xA9, 0x20, 0x4C, //
        0x30, 0x86, 0xC6, 0x99, 0x20, 0x4B, //
        0x3A, 0x5C, 0x0A, 0x54, 0x02, 0x29, 0x5C, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// All five decode end to end, and every one is a single basic block — a
    /// virtual call is not control flow at this layer, which is the fact that
    /// keeps `cflow-straight` honest when 45,631 bodies join it.
    #[test]
    fn virtual_dispatch_and_byvalue_return_decode() {
        for seg in [VIRT_SLOT0, VIRT_SLOT31, VIRT_SLOT32, BYVAL, BYVAL_VIRT] {
            assert_eq!(
                scan(seg),
                Ok(CfBody { shape: CfShape::Straight, residue: CfResidue::Expression })
            );
        }
    }

    /// **The vtable slot is a signed varint, and only `w32` can say so.** `w31`
    /// (offset `0x7C`) and `w32` (offset `0x80`) are the same function with one
    /// digit changed in the source, and their encodings differ by four bytes.
    /// Truncating the escape payload by one byte must not reach the tail — a
    /// width that fails is visible, a width that succeeds by luck is not.
    #[test]
    fn the_vtable_slot_is_a_varint_not_a_byte() {
        assert!(VIRT_SLOT31.windows(2).any(|w| w == [0x67, 0x7C]));
        assert!(VIRT_SLOT32.windows(6).any(|w| w == [0x67, 0x80, 0x80, 0x00, 0x00, 0x00]));
        assert!(scan(VIRT_SLOT31).is_ok() && scan(VIRT_SLOT32).is_ok());
        let mut short = VIRT_SLOT32.to_vec();
        short.remove(5); // one byte out of the escape payload
        assert!(scan(&short).is_err(), "a mis-read slot field must not reach the tail");
    }

    /// **`9A` carries no trailing field, and `99`'s spelling proves it can.**
    /// Splicing `99`'s `<varint>` in after `9A <TYPE>` — the one plausible
    /// alternative reading, since the two opcodes are adjacent and both bind —
    /// swallows the `BD` and lands the walk on a TYPE tag, which is not an
    /// operand opcode. Over 837,830 real bodies the same separation is worth
    /// **13,024** decoded bodies to the reading without the field, and **0** to
    /// the reading with it.
    #[test]
    fn the_vtable_bind_has_no_trailing_field() {
        let at = VIRT_SLOT0
            .windows(5)
            .position(|w| w == [0x9A, 0x86, 0x43, 0x9F, 0x20])
            .expect("the 9A bind")
            + 5;
        let mut spliced = VIRT_SLOT0.to_vec();
        spliced.splice(at..at, [0x00]);
        assert!(scan(VIRT_SLOT0).is_ok());
        assert!(scan(&spliced).is_err(), "9A <TYPE> <varint> must not also decode");
    }

    /// **`64` takes a TYPE, and a byte that cannot be a tag refuses at the type
    /// rather than being stepped over.** The positive claim this pairs with is
    /// the composition test above; this one pins the failure direction, which is
    /// the half a decode-only scanner can get wrong silently.
    #[test]
    fn the_materialize_takes_a_type() {
        let at = BYVAL.windows(5).position(|w| w == [0x64, 0x86, 0x43, 0xA9, 0x20]).unwrap();
        let mut bad = BYVAL.to_vec();
        bad[at + 1] = 0x04; // bit 7 clear — cannot be a TYPE tag
        match scan(&bad) {
            Err(b) => assert_eq!(b.ctx, "cf-materialize-type"),
            Ok(_) => panic!("a non-tag byte after 64 must refuse"),
        }
    }


    // ---- the WIDE type tag (WVB, docs/IL_TYPE_WIDE_TAG.md) -------------------

    /// **[P] `work/WVB/probe/p3.cpp`, whole four-line TU, at the workload's own
    /// flags — `D::D()`'s complete segment.**
    ///
    /// ```cpp
    /// struct P { virtual void V(); int q; };   // 8 bytes, polymorphic
    /// struct N { int a, b; N(); };             // 8 bytes, NOT polymorphic
    /// struct D : P, N { D(); };
    /// D::D() {}
    /// ```
    ///
    /// Two base constructions in the same production, one source keyword apart:
    ///
    /// ```text
    ///   … 4C  30 c6 81 86 82 20  4B      P — WIDE tag, FIVE bytes
    ///   … 4C  30 86    86 93 20  4B      N — narrow,   four
    /// ```
    ///
    /// Under the pre-WVB three-byte reading the first of those resumes the walk on
    /// `82`, and this body refuses as `cf-expr-0x82` — the 23,254-body row that
    /// ranked second on the control-flow axis and contained no `82` opcode at all.
    const CTOR_TWO_BASES: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x01, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33,
        0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F,
        0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01,
        0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x53,
        0x53, 0x26, 0xF9, 0x09, 0xB9, 0x02, 0x0A, 0xA6, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43,
        0xA3, 0x20, 0x00, 0x46, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE7, 0x09, 0x33, 0x86, 0x41,
        0x74, 0x80, 0x41, 0x08, 0x00, 0x00, 0x40, 0x86, 0x43, 0xA4, 0x20, 0x66, 0x02, 0x80,
        0x20, 0x82, 0x20, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86,
        0x41, 0x74, 0xB9, 0x02, 0x0A, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20,
        0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00, 0xBD, 0xA6, 0x43, 0x86, 0x20, 0x00, 0x80,
        0x0C, 0x10, 0x00, 0x00, 0x4C, 0x30, 0xC6, 0x81, 0x86, 0x82, 0x20, 0x4B, 0x26, 0xF2,
        0x09, 0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00, 0x40, 0x86, 0x43, 0xA5,
        0x20, 0x66, 0x02, 0x80, 0x20, 0x93, 0x20, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41,
        0x74, 0x08, 0x55, 0x86, 0x41, 0x74, 0xB9, 0x02, 0x0A, 0xA6, 0x43, 0x81, 0x20, 0x55,
        0xA6, 0x43, 0x81, 0x20, 0x4C, 0x99, 0x86, 0x43, 0xA6, 0x20, 0x00, 0xBD, 0xA6, 0x43,
        0x94, 0x20, 0x00, 0x80, 0x15, 0x10, 0x00, 0x00, 0x4C, 0x30, 0x86, 0x86, 0x93, 0x20,
        0x4B, 0x33, 0x86, 0x41, 0x74, 0x80, 0x45, 0x08, 0x00, 0x00, 0x40, 0xA6, 0x43, 0xAA,
        0x20, 0x66, 0x02, 0x80, 0x20, 0x82, 0x20, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41,
        0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41,
        0x74, 0xB9, 0x02, 0x0A, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20, 0x4C,
        0x26, 0x00, 0x0A, 0x2C, 0x86, 0x43, 0xA9, 0x20, 0x00, 0x32, 0x86, 0x43, 0xA9, 0x20,
        0x4B, 0x3A, 0x03, 0x0A, 0x54, 0x02, 0x29, 0x03, 0x0A, 0xB9, 0x02, 0x0A, 0xA6, 0x43,
        0x81, 0x20, 0x41, 0xA6, 0x43, 0x81, 0x20, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The whole body decodes end to end — the falsification test (land exactly on
    /// the seven-byte tail, every `54 <k>` depth agreeing) applied to the segment
    /// that separates the two spellings. Its neighbour in the same function is the
    /// control: if the wide rule were a licence rather than a width, the *narrow*
    /// load would break too.
    #[test]
    fn the_polymorphic_base_load_is_five_bytes_and_the_body_closes_on_it() {
        assert!(scan(CTOR_TWO_BASES).is_ok(), "D::D() must decode end to end");
        // Both loads are present, and they differ by exactly the mark byte.
        assert!(CTOR_TWO_BASES.windows(6).any(|w| w == [0x30, 0xC6, 0x81, 0x86, 0x82, 0x20]));
        assert!(CTOR_TWO_BASES.windows(5).any(|w| w == [0x30, 0x86, 0x86, 0x93, 0x20]));
        // …and the fail-closed direction, which is the half a decode-only scanner
        // can get wrong silently: a wide tag whose mark has bit 7 CLEAR is not a
        // type, and the walk must refuse AT THE LOAD rather than step a guessed
        // width. (This is the case the workload has 36 of under the literal-`81`
        // rule, and it is why the mark is a bit test — see `readers.rs`.)
        let at = CTOR_TWO_BASES.windows(2).position(|w| w == [0xC6, 0x81]).unwrap();
        let mut bad = CTOR_TWO_BASES.to_vec();
        bad[at + 1] = 0x01;
        match scan(&bad) {
            Err(b) => assert_eq!(b.ctx, "cf-deref-type"),
            Ok(_) => panic!("a mark without bit 7 must refuse"),
        }
    }

    /// The scanner is decode-only, and this is the test that says so: a body it
    /// decodes completely is still refused by the accepting parser. If this ever
    /// fails, something has wired the scanner into acceptance.
    #[test]
    fn decoding_a_body_never_makes_it_in_class() {
        use crate::func::body::parse_segment_detail;
        use crate::func::sy::SyView;
        assert!(scan(IF_ELSE).is_ok());
        assert!(parse_segment_detail(IF_ELSE, SyView::UNKNOWN).is_err());
        assert!(scan(WHILE).is_ok());
        assert!(parse_segment_detail(WHILE, SyView::UNKNOWN).is_err());
    }
}

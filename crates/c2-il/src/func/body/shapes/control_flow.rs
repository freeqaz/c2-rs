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
const PRE_BODY_DEPTH: u32 = 2;
/// Deeper than any real function; a stream claiming more has desynchronized.
/// (The widest witness is 40 nested braces at depth 42 — **[P] `p6.cpp`**.)
const MAX_DEPTH: u32 = 96;
/// The function tail every decoded body lands exactly on.
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

    /// **The EH census axis.** Which side of `docs/EH_RECORDS.md` §6's boundary
    /// this body falls on:
    ///
    /// > Exactly one sub-object statement and nothing else is a bare branch. A
    /// > second sub-object, or any other statement beside it, is the WHOLE EH
    /// > RECORD.
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
}

/// One body's two decode-only readings: the control-flow verdict (or the byte
/// that stopped it) and the EH markers counted up to that point.
pub(crate) struct CfScan {
    pub(crate) body: Result<CfBody, Block>,
    pub(crate) eh: EhMarkers,
    /// Whether the walk reached the function tail — the same fact as
    /// `body.is_ok()`, named because [`EhMarkers::key`] reads it.
    pub(crate) decoded: bool,
}

/// A branch or label site, in stream order.
#[derive(Clone, Copy)]
struct Site {
    tok: u32,
    /// Offset of the *opcode*, so "defined before" is a plain comparison.
    at: usize,
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
    /// The EH-state trailer counts — see [`EhMarkers`].
    eh: EhMarkers,
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

    /// Note that the operand stream left the modeled class. Called with the token's
    /// own reason so the call sites read as a list of what is *not* modeled.
    fn off_class(&mut self) {
        self.off_class = true;
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
        p: lo + 3,
        depth: PRE_BODY_DEPTH,
        labels: Vec::new(),
        conds: Vec::new(),
        jumps: Vec::new(),
        switches: 0,
        off_class: false,
        eh: EhMarkers::default(),
    };
    let body = walk(&mut s);
    CfScan { decoded: body.is_ok(), body, eh: s.eh }
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
                return Err(Block { ctx: "cf-scope-depth", byte: Some(k), off: s.p, aux: 0 });
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
                s.off_class();
            }
        }
        // LITERAL `33 <TYPE> <payload>`. The payload width is a function of the
        // type: a real is 8 IEEE bytes + a 2-byte size, an 8-byte integer's escape
        // is 8 bytes, everything else is the ordinary varint.
        0x33 => {
            s.p += 1;
            let (tag, kind) = s.ty("cf-lit-type")?;
            if !(is_int4_type(tag, kind) || is_ptr_to_4(tag, kind)) {
                s.off_class();
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
        // The gaps are deliberate and are the point. `05`, `07`, `08`, `14`, `1D`,
        // `1E`, `25` are unwitnessed; `14` in particular has no C operator between
        // `%=` and `<<=` and §5 says in as many words not to fill it. Guessing
        // width 1 for them would be right most of the time and silently
        // desynchronize the rest — and a desync that lands on a plausible tail is
        // the failure this whole scanner is built to make impossible. They refuse,
        // and the size of the `cf-expr-0xNN` row they produce is what tells the
        // next rung whether establishing them is worth a probe.
        0x06 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x1A | 0x1B | 0x1C | 0x1F | 0x20
        | 0x21 | 0x22 | 0x23 | 0x24 => {
            s.p += 1;
            s.off_class();
        }
        // Compound assignment / inc-dec: `<op> <TYPE>`, the twelve witnessed
        // opcodes of §5. `0x14` is deliberately NOT here — it is unobserved, and it
        // is handled above as a payload-free operator only because a width guess in
        // the other direction desynchronizes; see the test that pins it.
        0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x35 | 0x36 => {
            s.p += 1;
            s.ty("cf-rmw-type")?;
            s.off_class();
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
                s.off_class();
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
            s.off_class();
        }
        0x30 => {
            s.p += 1;
            s.ty("cf-deref-type")?;
            s.off_class();
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
            s.eh.any = true;
            s.off_class();
        }
        0x5D | 0x5E => {
            s.p += 1;
            let n = s.vint("cf-eh-count")?;
            s.vint("cf-eh-count-state")?;
            s.eh.count = s.eh.count.max(n.max(0) as u32);
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
            s.off_class();
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
            s.off_class();
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
            s.off_class();
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
                s.off_class();
            }
        }
        // INTRINSIC CALL `40 <TYPE result>` — no trailing field
        // (`docs/IL_INTRINSIC_CALL.md` §1).
        0x40 => {
            s.p += 1;
            s.ty("cf-intrinsic-type")?;
            s.off_class();
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
                _ => return Err(Block { ctx: "cf-escape-43", byte: Some(sub), off: s.p, aux: 0 }),
            }
            if s.p > s.seg.len() {
                return Err(blk(s.seg, s.seg.len(), "cf-escape-43"));
            }
            s.off_class();
        }
        // `66 <n> <n tokens>` — the class-pair descriptor of the 2113–2119
        // intrinsic family. Its second byte is an ARITY, not the constant `02`
        // (`docs/IL_INTRINSIC_CALL.md` §4.3), and the tokens are LEB-width; the ONE
        // decoder for it is `mcall`'s, imported rather than restated.
        0x66 => {
            if eat_class_descriptor(s.seg, &mut s.p).is_none() {
                return Err(blk(s.seg, s.p, "cf-class-descriptor"));
            }
            s.off_class();
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
            s.off_class();
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
            s.off_class();
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
            s.off_class();
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
            s.off_class();
        }
        0x9B => {
            s.p += 1;
            s.ty("cf-temp-type")?;
            s.tok("cf-temp-tok")?;
            s.off_class();
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
                s.off_class();
            }
        }
        0x4C => s.p += 1,
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
mod tests {
    use super::*;
    use crate::func::bundle::LO_MARKER;
    use crate::func::readers::find_subslice;

    fn scan(seg: &[u8]) -> Result<CfBody, Block> {
        let lo = find_subslice(seg, &LO_MARKER).expect("a body marker");
        scan_full(seg, lo).body
    }

    /// The EH axis's reading of the same body: `(key, markers)`.
    fn eh(seg: &[u8]) -> (&'static str, EhMarkers) {
        let lo = find_subslice(seg, &LO_MARKER).expect("a body marker");
        let s = scan_full(seg, lo);
        (s.eh.key(s.decoded), s.eh)
    }

    /// **[CF] `il_stmt_seq.cpp` `void stmt_seq0() {}`** — the smallest body there
    /// is, and the calibration for every shape below: one epilogue jump, one
    /// epilogue label, nothing else.
    const EMPTY: &[u8] = &[
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
    const IF_ELSE: &[u8] = &[
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
    const WHILE: &[u8] = &[
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
    const EARLY_RETURN: &[u8] = &[
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
        let label = |ctx| Block { ctx, byte: Some(0x29), off: 0, aux: 0 }.feature();
        assert_eq!(label("body"), "body-cflow-label");
        assert_eq!(label("return-scope-close"), "return-scope-close-cflow-label");
        assert_ne!(label("body"), label("return-scope-close"));
        // The `expr` production renders through its own table first and falls
        // through to this one, so a branch met as an operand is `expr-brfalse`
        // rather than `expr-cflow-brfalse` — one prefix per production, as every
        // other `expr-*` key has.
        assert_eq!(
            Block { ctx: "expr", byte: Some(0x38), off: 0, aux: 0 }.feature(),
            "expr-brfalse"
        );
        assert_eq!(
            Block { ctx: "call-ref", byte: Some(0x3A), off: 0, aux: 0 }.feature(),
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
        // Splice an opcode this scanner refuses (`05`, still unestablished —
        // `IL_STMT_GRAMMAR.md` §5's operator table stops at `%` = `06` and does
        // not say what `05` is) after the sub-object statement's `4B`. This was
        // `64` until WDR established it; the substitution is 1:1 in what the test
        // asserts, which is that SOME refusal after a marker reads `eh-partial`.
        let mut seg = EH_ONE.to_vec();
        let at = seg
            .windows(6)
            .position(|w| w == [0x5C, 0x86, 0x41, 0x74, 0x01, 0x4B])
            .expect("the statement trailer")
            + 6;
        seg.splice(at..at, [0x05]);
        assert_eq!(eh(&seg).0, "eh-partial");
        // …and a body that stops BEFORE any marker claims nothing.
        let mut early = EH_ONE.to_vec();
        early.splice(4..4, [0x05]);
        assert_eq!(eh(&early).0, "eh-unknown");
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

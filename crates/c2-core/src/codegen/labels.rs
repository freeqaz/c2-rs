//! The label→offset map — `docs/CFG_SHAPE.md` §6.2 item **B**, built.
//!
//! > **B. Labels as first-class, resolved by a fixup pass.** `3A`/`38`/`39`
//! > carry no direction (§2.1), so the target's offset is unknown when the
//! > branch is emitted. The IR needs a label identity, a map from label to
//! > block, and a **fixup list** of (word offset, label, form). Even §4's
//! > single-branch minimal instance needs this — it is not an optimization for
//! > the many-block case.
//!
//! # Why this exists when `calls.rs` already resolved a branch
//!
//! Board row **Z-c** (W11) reads *"the port emits an intra-section `b` and
//! resolves a real label→offset map"*. What W11 actually built is a fixup list
//! with **one implicit target**: `early_fixups`, every entry resolved against
//! `epi_start`, with the epilogue's identity carried by the *shape of the tuple*
//! rather than by any label. `calls.rs` said so itself four lines above the
//! block that resolved it — *"there is no fixup list and no label map"*. That
//! lowering is byte-exact at 12 lanes and this module does not change one byte
//! of it; what it changes is that the target is now **named**, so a second
//! target can exist.
//!
//! # The two rules this map enforces, and where each was measured
//!
//! **1. Two encodings, chosen by target kind — never one "patch the branch"
//! path.** `bc` and the intra-section `b` carry a **true self-relative
//! displacement** and take **no relocation**; an external `b`/`bl` carries
//! `−(own offset)` and takes a `REL24` (`CFG_SHAPE.md` §3.3, board **#191**).
//! `48000008` and `4bffffec` are the same instruction. **This map holds only the
//! first kind.** An external branch is not a label reference — it is a
//! relocation — and admitting it here is exactly the corruption §3.3 warns
//! about, so [`Form`] has no variant for it and [`encode_tail_branch`] stays
//! where it is.
//!
//! [`encode_tail_branch`]: super::encode::encode_tail_branch
//!
//! **2. Every reference must be FORWARD — a backward reference is refused, and
//! the refusal is a `coff/` fact, not a `codegen/` preference.**
//!
//! This is the rule lane w-label measured before writing the code
//! (`work/w-label/PREREG.md` §1; `work/w-label/cflabels.py`, 24 seed-free in-TU
//! cells at `/O1 /GS- /c`, anchor controls held on every row):
//!
//! ```text
//!   every body with a BACKWARD intra-section branch charges the
//!   compiler-label counter >= +1                                11 of 11
//!   no body without one charges more than +1                    13 of 13
//! ```
//!
//! `coff::plan_labels` charges a framed function `label_lead + 5` and
//! `IlFunction::label_lead` returns non-zero only for the signed two-call
//! comparator and `eh_bare`. So a lowering that emitted a backward branch would
//! be **one label slot low for this function and every later one in the TU** —
//! six wrong bytes in a symbol table, in an obj that still links, which is the
//! defect class `docs/LABEL_COUNTER.md` exists for and which `coff/` has shipped
//! twice.
//!
//! The magnitude on the far side is **measured and not modelled**: the same 24
//! cells read +1 (`do/while`, an explicit backward `goto`, the exit-value
//! merge), +2 (`while`, `for`), +3 (`for(;;)`+`break`, two sequential
//! `do/while`s) and +4 (two `for`s, nested `for`) — four distinct magnitudes
//! over eleven cells with no rule that survives all of them. Two candidate rules
//! *were* fitted to that table and **both are refuted by it**: "one slot per
//! interior branch target" misses 15 of 24 rows and "one per interior join"
//! misses 6, one of them across the zero boundary. Interpolating any of that
//! would be `CFG_SHAPE.md` §3.5's declined fold model a second time, so the
//! refusal ships and the measurement ships with it.
//!
//! **What the rule does NOT say.** Forward-only is *necessary*, not
//! *sufficient*: two forward-only cells (`cf-ifelse`, `cf-merge-tail`) charge +1
//! anyway, and both are §3.4.1's code-motion shapes — a block c2 created by
//! tail-merging two paths. The port refuses those for an unrelated reason (a
//! body whose arms end in the same call is out of class), so the two refusals
//! are independent and **closing this one does not close that one**. Named here
//! because a lane that closed the backward case alone would still emit a wrong
//! `$M` on the other.
//!
//! # The consequence is CONDITIONAL, and the condition is measured
//!
//! **Correction, lane `w-loop` (board #741/#742).** Everything above stands and
//! the refusal stays, but the sentence *"the obj would carry a wrong `$M`"* is
//! **conditional on the obj carrying a `$M` at all**, and w-label's grid could
//! not see that because every one of its 33 probes is a framed Class-A body.
//! Two measurements, `work/w-loop/loopcost.py`:
//!
//! ```text
//!   Q1  a LEAF loop's seed-free stride, 17 cells, anchor control 5/5 on
//!       every row:   while +2  do/while +1  for +2  for(;;)+break +3
//!                    nested +4  two sequential +4  backward goto +1
//!       against `leaf-none` = 1. The SAME integers `LABEL_COUNTER.md` §4
//!       records for the framed probes — the control-flow surcharge does not
//!       key on frame class — and `minted` is 1 on every row, so it mints
//!       nothing.  ==> `plan_labels` charges 0 and c2 charges up to +4. The
//!                     refusal above is RIGHT.
//!
//!   Q2  34 leaf-only TUs over the same 17 bodies, 28 of them containing a
//!       backward branch:  ZERO `$M`/`$T` symbols, 34 of 34.
//!       Control: the same 17 bodies each followed by ONE framed function
//!       minted a triple, 17 of 17.
//!       ==> `coff::plan_labels` returns `Some([n,n+1,n+2])` only for a
//!           function with a `frame`. That is the ONLY channel by which the
//!           counter's VALUE reaches an object file, and a TU without a framed
//!           function does not have it.
//! ```
//!
//! So the wrong `$M` this method refuses to emit **needs a framed function in
//! the same TU to land on**. On the codegen frontier that is not a corner case:
//! `Sort.cpp`, `Primes.cpp` and `IPP_basicmath_xbox.cpp` — three of the six
//! `cflow-loop`-blocked frontier TUs, six loop functions between them — carry no
//! `$M` at all, which the scan now prints per TU as `label-free`
//! (`c2_obj::ObjImage::compiler_label_symbols`, board #742).
//! ⚠ **STALE as of 2026-08-08 — `Sort.cpp` converted and the count is now seven.
//! See the correction at the end of this header; the sentence stays as the
//! record of what was true when the refusal was justified.**
//!
//! **This is not a licence and nothing here was widened for it.** Three reasons,
//! and each is load-bearing:
//!
//! 1. **The counter is only the first refusal.** Every one of those three TUs
//!    prices at ≥ 4 independent refusals on its own bytes (`Sort.cpp`: a signed
//!    `%` expansion, two `twi` traps with a three-instruction predicate, an
//!    update-form `lbzu`, a record-form `mr.` on cr0, `mulli`, and a schedule
//!    that interleaves the trap predicate between `divw` and `mullw`). The
//!    standing decline clause fires on all three.
//! 2. **`Selected` still has no variant with a back edge**, so there is no
//!    caller that would pass a relaxation, and a `resolve` that accepted one
//!    would be an ungraded code path by construction — w-frame row **F-c**.
//!    ⚠ **REASON 2 HAS EXPIRED — see the 2026-08-08 correction below. Reasons 1
//!    and 3 have not, and the refusal is unchanged.**
//! 3. **The precondition is TU-level and this map is per-body.** A `LabelMap`
//!    cannot see whether a later function in the TU is framed. The existing
//!    mechanism that *can* is `IlBundle::functions`' gate, which already demands
//!    `label_slots(..) == 1` from every non-framed function in a TU containing a
//!    framed one and is three-valued so an unmeasured class refuses — a loop
//!    body returning `None` there is refused in exactly the TUs where the charge
//!    is observable, and admitted in exactly the TUs where Q2 says it is not.
//!    **That is where a loop rung's relaxation belongs; it is not here.**
//!
//! # Correction, lane `w-loop`, 2026-08-08 (boards #1393, #1394)
//!
//! Everything above is left as written; three of its facts have moved and one of
//! its three reasons is dead. Re-measured on the 878-TU scan at master
//! `2b1c89da`.
//!
//! **1. `Sort.cpp` CONVERTED and is no longer on the frontier.** The paragraph
//! above names it as one of "three of the six `cflow-loop`-blocked frontier TUs"
//! and reason 1 prices it at ≥ 4 refusals. It is a **match** — lane `w-hash`,
//! board **#761**, via [`super::ptr_walk_loop`]. The current figures are
//! **seven** `cflow-loop`-blocked frontier TUs, of which **three** are
//! label-free, and the three are `Primes.cpp`, `IPP_basicmath_xbox.cpp` and
//! `Pool.cpp`. The decline that priced `Sort.cpp` at ≥ 8 was a correct reading
//! of its bytes and a wrong prediction about the outcome; both stay on the page.
//!
//! **2. REASON 2 IS FALSE, and it is the one a relaxing lane would lean on.**
//! *"`Selected` still has no variant with a back edge"* was true when written
//! and is not now: [`super::ptr_walk_loop`] and
//! [`super::ptr_walk_chain_loop`] both emit a **backward** `bc` and both reach
//! [`super::select::Selected::Plain`]. There is no contradiction with invariant
//! 4 — neither carrier routes through this map; each computes its displacement
//! directly through `encode_bc`, so the map never sees the reference — but the
//! *argument* "no caller could pass a relaxation" no longer holds, because two
//! callers now emit exactly the thing the relaxation would admit.
//!
//! **The refusal is nevertheless unchanged, on reasons 1 and 3 alone**, and the
//! honest statement of why is narrower than the old one: relaxing invariant 4
//! would convert **nothing today**, because every remaining `cflow-loop` TU is
//! blocked ahead of codegen. `Primes.cpp` — the cheapest of them at 64 bytes —
//! does not reach a selector at all: the scan reads it `vocab-gap`, blocking
//! feature `expr-jump`, `il function decode failed`. A relaxation whose only
//! effect is on bodies no reader produces is w-frame row **F-c** by a different
//! route.
//!
//! **3. The instruction vocabulary was two words short, not eight refusals
//! deep.** `codegen::frontier_bytes` (`cfg(test)`) rebuilds all sixteen words of
//! `Primes.cpp`'s `?NextHashPrime@@YAHH@Z` from this crate's encoders —
//! **fourteen from encoders that already existed**, two from `encode_cmpw` and
//! `encode_lwzx` added with that module. Board **#1105**'s "eight codegen
//! refusals" is not thereby wrong; what it does not say, and what a lane sizing
//! this work needs, is that **none of the eight is an encoder**.
//!
//! # Correction, lane `w-fencea`, 2026-08-15 — **invariant 4's STATED RULE AND
//! ITS ENFORCING LINE QUANTIFY OVER DIFFERENT POPULATIONS, AND FOUR SHIPPED
//! CLASSES LIVE IN THE GAP**
//!
//! Everything above stays as written. What this adds is the reading `w-ir-g`
//! (#3114) and `w-item-d` (#3119) each paid for separately — **read a rule's
//! sentence and the line that enforces it apart** — applied to this module's own
//! rule 2.
//!
//! * The **stated rule**, four paragraphs up, quantifies over *bodies*: *"every
//!   body with a BACKWARD intra-section branch charges the compiler-label
//!   counter >= +1, 11 of 11"*.
//! * The **enforcing line**, [`LabelMap::resolve`] invariant 4, quantifies over
//!   *references routed through this map*.
//!
//! Those are not the same population, and the 2026-08-08 correction above
//! already records the gap without drawing its consequence: [`super::ptr_walk_loop`]
//! and [`super::ptr_walk_chain_loop`] *"both emit a backward `bc` … neither
//! carrier routes through this map"*. [`super::json_utf8_copy`] and
//! [`super::xtea_encrypt_loop`] do the same. **So the enforcing line has never
//! enforced the stated rule**: four byte-exact classes emit the very thing it
//! refuses, and it refuses none of them.
//!
//! What that leaves invariant 4 doing, from `w-layout` onwards, is **blocking 7
//! of the 8 residual `BodyLayout` sites** (board **#3144**) at zero counter
//! benefit — because the counter was never this module's to protect. **Reason
//! 3, above, says so in its own words**: *"That is where a loop rung's
//! relaxation belongs; it is not here."*
//!
//! ## What replaces it, and why the residual is excluded BY CONSTRUCTION
//!
//! Invariant 4 becomes a **per-map admission fixed at construction**, defaulting
//! to the refusal. [`LabelMap::new`] is unchanged and is [`BackEdge::Refused`],
//! so **all nine of item A's clients keep exactly the map they have today** and
//! not one byte of theirs moves. The only way past it is
//! [`LabelMap::admitting_back_edges`], which takes a [`ChargedClass`] — a
//! **closed** enum whose every variant is graded, in
//! [`tests::every_admitted_class_has_a_registered_control_flow_surcharge`],
//! against `c2_il`'s own three-valued counter gate:
//!
//! ```text
//!   label_slots(false) == None                    => IlBundle::functions refuses
//!                                                    EVERY TU in which this body's
//!                                                    $M could be observed at all
//!                                                    (board #742, Q2 above)
//!   label_slots(false) == Some(label_lead() + 1)
//!     and label_lead() >= 1                       => coff::plan_labels ALREADY
//!                                                    advances the surcharge
//! ```
//!
//! A class that is **neither** — `label_lead() == 0` with `label_slots ==
//! Some(1)` — is exactly the wrong-`$M` case invariant 4 was built for, and the
//! test refuses it. The closure is the compiler's: the grading test `match`es
//! [`ChargedClass`] exhaustively and walks [`ChargedClass::ALL`], so a variant
//! added without evidence does not compile, and one added without being listed
//! fails [`tests::the_admitted_class_list_is_complete`].
//!
//! ## The charge is a SERIES, not one obj reading (`#3147`)
//!
//! `w-slots` established that reading a charge off one cell's obj gives a number
//! that is right for that cell and wrong as a rule, and that only varying the
//! structural count separates them. The structural count here is **the number of
//! admitted-class loop functions in one TU**, and `work/w-fencea/cells/` varies
//! it over `n = 0, 1, 2, 3` against real `c2.dll`:
//!
//! ```text
//!   loops_0  $M2548  lead +0      ctl_plain_0  +0
//!   loops_1  $M2564  lead +2      ctl_plain_1  +0
//!   loops_2  $M2580  lead +4      ctl_plain_2  +0
//!   loops_3  $M2596  lead +6      ctl_plain_3  +0     ==> 2n, and the four
//!                                                         plain-leaf controls
//!                                                         say it is not the
//!                                                         function count
//! ```
//!
//! All eight cells `match` end to end at `/O1`. `w-fenceb` registered the `2`
//! from `n = 1` alone; this is the first grading of it at `n >= 2`.
//!
//! **What this does NOT do.** It does not model a loop charge — `#3127`'s
//! hold-out (5 of 15, the loop *kind* is a term no backward-branch feature
//! vector holds) stands untouched, and no rule of that shape is used or implied
//! here. It admits exactly the classes whose charge `c2_il` has already
//! registered and graded, and refuses every other body as before.

use super::select::out_of_class;
use super::{encode_b_intra, encode_bc};
use crate::BackendError;

/// A label identity, minted by [`LabelMap::mint`].
///
/// Opaque and `Copy`. It carries an index into the map that minted it, so a
/// label from one map used against another is caught by
/// [`LabelMap::resolve`]'s bounds check rather than silently reading a
/// neighbour's offset.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Label(usize);

/// Which of the two intra-section branch encodings a reference site wants.
///
/// There is deliberately **no external/relocated variant** — see this module's
/// header. The discriminator is the target, not the opcode, and the two live in
/// different places for that reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// A conditional branch. `BD` is a signed 14-bit field scaled by 4, so it
    /// reaches ±32764.
    Bc { bo: u8, bi: u8 },
    /// An unconditional intra-section `b`. `LI` is a signed 24-bit field scaled
    /// by 4.
    B,
}

impl Form {
    fn encode(self, disp: i32) -> Option<[u8; 4]> {
        match self {
            Form::Bc { bo, bi } => encode_bc(bo, bi, disp),
            Form::B => encode_b_intra(disp),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Form::Bc { .. } => "bc",
            Form::B => "b",
        }
    }
}

/// **The classes whose back edge may be resolved through the map**, and the only
/// values [`BackEdge::ChargedAtIl`] can carry.
///
/// This enum is the admission set for invariant 4, and it is **closed**: every
/// variant is graded against `c2_il`'s own three-valued counter gate by
/// [`tests::every_admitted_class_has_a_registered_control_flow_surcharge`],
/// whose `match` is exhaustive, and [`Self::ALL`] is checked complete by
/// [`tests::the_admitted_class_list_is_complete`]. A class that wants in has to
/// bring a registered surcharge or it does not compile; a class with
/// `label_lead() == 0` is refused by the grading test itself, because that is
/// precisely the wrong-`$M` case invariant 4 exists for.
///
/// **The variant does not carry a number**, deliberately. `#3148` records three
/// published label numbers that drifted because they lived in a second place;
/// the charge lives in `c2_il::IlFunction::label_lead` and is read from there by
/// the test, never copied here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChargedClass {
    /// `c2_il::PtrWalkModLoop` — `label_lead` **2**, board **#746** fence B
    /// lifted by lane `w-fenceb`, and the `2n` series above.
    PtrWalkModLoop,
    /// `c2_il::XteaEncryptLoop` — `label_lead` **4** (lane `w-xtea3`), and
    /// `label_slots` falls through to `label_lead() + 1`.
    XteaEncryptLoop,
    /// `c2_il::PtrWalkChainLoop` — `label_slots` is **`None`**, so
    /// `IlBundle::functions` refuses every TU in which this body's `$M` could be
    /// observed. Admitted by the second arm of the rule, not the first: the
    /// charge is *undetermined*, and the TU-level gate is what makes that safe.
    PtrWalkChainLoop,
    /// `c2_il::JsonUtf8CopyFn` — `label_lead` **4**, and the class is
    /// **`is_framed()`** even though its Class C prologue has no `stwu`: it
    /// carries a `.pdata` record and a `$M`/`$M`/`$T` triple, which is what that
    /// predicate means. So `label_slots(false)` is `Some(label_lead() + 4)` =
    /// **`Some(8)`**, arm 1.
    ///
    /// **Board `#3155` publishes `Some(5)` for this class and that is the
    /// non-framed formula.** Recorded here rather than corrected silently,
    /// because the reason it cost nothing is structural and worth keeping: the
    /// grading test below reads `label_slots` and `label_lead` off `c2_il`
    /// rather than off the row, so a wrong number on the board cannot become a
    /// wrong number in this crate (`#3148`).
    JsonUtf8Copy,
}

impl ChargedClass {
    /// Every variant. Kept beside the enum and checked complete by a test — a
    /// list nobody grades is how an admission set silently grows.
    pub const ALL: [ChargedClass; 4] = [
        ChargedClass::PtrWalkModLoop,
        ChargedClass::XteaEncryptLoop,
        ChargedClass::PtrWalkChainLoop,
        ChargedClass::JsonUtf8Copy,
    ];

    /// The `c2_il` shape this class is recognized by. Diagnostic and test text
    /// only.
    pub fn il_shape(self) -> &'static str {
        match self {
            ChargedClass::PtrWalkModLoop => "c2_il::PtrWalkModLoop",
            ChargedClass::XteaEncryptLoop => "c2_il::XteaEncryptLoop",
            ChargedClass::PtrWalkChainLoop => "c2_il::PtrWalkChainLoop",
            ChargedClass::JsonUtf8Copy => "c2_il::JsonUtf8CopyFn",
        }
    }
}

/// **Invariant 4's admission**, fixed when the map is built.
///
/// Three clients' worth of history is in this module's `w-fencea` correction.
/// The short form: the enforcing line quantifies over references routed through
/// this map, the stated rule quantifies over bodies, and the counter is
/// `c2_il::IlFunction::label_slots`' business and not this map's. So the map
/// stops pretending to be a counter gate and starts being what it is — a fixup
/// pass that will resolve a back edge **only** for a body whose class has
/// already been graded through the counter gate.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BackEdge {
    /// Invariant 4 as written and as measured. [`LabelMap::new`]'s value, and
    /// every existing client's.
    #[default]
    Refused,
    /// Admitted, for a body of this class and no other.
    ChargedAtIl(ChargedClass),
}

impl BackEdge {
    /// Whether a **strictly** backward reference may be patched. A
    /// zero-displacement self reference is refused under every value — see
    /// [`LabelMap::resolve`] invariant 4.
    pub fn admits(self) -> bool {
        matches!(self, BackEdge::ChargedAtIl(_))
    }
}

/// One pending reference: the `.text` offset of the placeholder word, the label
/// it names, and which encoding to patch it with.
struct Ref {
    at: usize,
    label: Label,
    form: Form,
}

/// The label→offset map for **one function body**.
///
/// A body's `text` is built in order; a branch whose target is not yet emitted
/// calls [`LabelMap::reference`], which appends a **zero placeholder word** and
/// records the site. [`LabelMap::define`] binds a label to the offset the body
/// has reached. [`LabelMap::resolve`] patches every site once, at the end, when
/// every offset is known.
///
/// It is scoped to one body on purpose. A label offset is a `.text`-section
/// offset and the port emits one COMDAT per function, so a map that outlived a
/// body would be holding offsets in two coordinate systems — which is the shape
/// of the `.pdata` mistake `docs/OBJ_GY_SHAPES.md` §3.3 records.
#[derive(Default)]
pub struct LabelMap {
    /// `defined[i]` is the offset bound to label `i`, or `None` while it is
    /// still forward.
    defined: Vec<Option<usize>>,
    /// A human name per label, used only in the error text. A refusal that
    /// cannot say *which* label it is about is a refusal somebody will have to
    /// re-derive.
    names: Vec<&'static str>,
    refs: Vec<Ref>,
    /// Invariant 4's admission for **this body**. `Refused` by default, which is
    /// what [`Self::new`] gives and what every client had before lane
    /// `w-fencea`.
    back_edge: BackEdge,
}

impl LabelMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// A map for a body of `class`, whose **strictly backward** references
    /// [`Self::resolve`] will patch instead of refusing.
    ///
    /// The argument is a [`ChargedClass`] rather than a `bool` because the
    /// question invariant 4 is really asking is *"has this body's control-flow
    /// label surcharge been through the counter gate"*, and that is a property
    /// of a class, graded in `c2_il`. See this module's `w-fencea` correction.
    pub fn admitting_back_edges(class: ChargedClass) -> Self {
        Self { back_edge: BackEdge::ChargedAtIl(class), ..Self::default() }
    }

    /// This map's admission. Read by [`super::block_ir::BodyLayout`] only to
    /// pass it on; nothing re-derives it.
    pub fn back_edge(&self) -> BackEdge {
        self.back_edge
    }

    /// Mint a fresh, undefined label. `name` appears in refusal text only.
    pub fn mint(&mut self, name: &'static str) -> Label {
        self.defined.push(None);
        self.names.push(name);
        Label(self.defined.len() - 1)
    }

    /// Bind `label` to the current end of `text`.
    ///
    /// Refuses a **second** definition rather than overwriting: two blocks
    /// claiming one label is a lowering bug, and silently keeping the last one
    /// would emit a legal-looking branch to the wrong place — the same failure
    /// mode as a truncated `BD`, which the encoders already refuse rather than
    /// round.
    pub fn define(&mut self, label: Label, text: &[u8]) -> Result<(), BackendError> {
        let at = text.len();
        let slot = self
            .defined
            .get_mut(label.0)
            .ok_or_else(|| out_of_class("a label from a different function's map"))?;
        if slot.is_some() {
            return Err(out_of_class(
                "a label defined twice in one body: two blocks claiming one \
                 target is a lowering defect, not a layout",
            ));
        }
        *slot = Some(at);
        Ok(())
    }

    /// Append a placeholder word for a branch to `label` and record the fixup.
    ///
    /// The placeholder is written **here** rather than by the caller, so the
    /// "the site is still zero when we patch it" invariant in [`Self::resolve`]
    /// is a real check on the caller and not a restatement of what the caller
    /// just did.
    pub fn reference(&mut self, text: &mut Vec<u8>, label: Label, form: Form) {
        let at = text.len();
        text.extend_from_slice(&[0; 4]);
        self.refs.push(Ref { at, label, form });
    }

    /// How many references are still outstanding. Used by the callers' own
    /// assertions and by the tests; a body that finishes with a non-empty map
    /// and never resolves it is the bug [`Self::resolve`] cannot catch, because
    /// it was never called.
    pub fn pending(&self) -> usize {
        self.refs.len()
    }

    /// Patch every recorded site, then consume the map.
    ///
    /// Five invariants, each an ordinary `Err` rather than a panic — the port
    /// must degrade to `NotImplemented` honestly, and a `debug_assert` is
    /// compiled out of the release build the gate actually runs.
    ///
    /// 1. **Every referenced label is defined.** An undefined one names itself.
    /// 2. **The site is in range** of the finished text.
    /// 3. **The site still holds the zero placeholder.** A caller that wrote
    ///    over its own fixup site would otherwise get a branch patched on top of
    ///    an instruction.
    /// 4. **The reference is FORWARD, unless this map was built for a
    ///    [`ChargedClass`]** — see the module header, and its `w-fencea`
    ///    correction for why the admission exists and why the default is
    ///    unchanged. A **zero-displacement** self reference is refused under
    ///    every admission: it is a branch to itself, not a back edge, and no
    ///    measured c2 body carries one.
    /// 5. **The displacement fits the form's field.** `CFG_SHAPE.md` §3.3.1's
    ///    long-branch expansion (invert the condition, branch over an
    ///    unconditional `b`) is measured and **not built** — no fixture body is
    ///    32 KB, so it has no bytes for the oracle to compare and building it
    ///    would be an ungraded code path by construction (w-frame row **F-c**).
    pub fn resolve(self, text: &mut [u8]) -> Result<(), BackendError> {
        for r in &self.refs {
            let target = match self.defined.get(r.label.0).copied().flatten() {
                Some(t) => t,
                None => {
                    return Err(out_of_class(&format!(
                        "a branch to label `{}`, which no block defined",
                        self.names.get(r.label.0).copied().unwrap_or("?")
                    )))
                }
            };
            if r.at + 4 > text.len() || target > text.len() {
                return Err(out_of_class(
                    "a label fixup outside the body it belongs to",
                ));
            }
            if text[r.at..r.at + 4] != [0; 4] {
                return Err(out_of_class(
                    "a label fixup site that is no longer the zero placeholder: \
                     something was emitted over a pending branch",
                ));
            }
            // **Invariant 4.** `target < r.at` is a back edge and is admitted
            // only for a [`ChargedClass`]; `target == r.at` is a
            // zero-displacement self reference and is refused under every
            // admission, which is what the `<=` here rather than `<` is for.
            if target <= r.at && !(self.back_edge.admits() && target < r.at) {
                // §1.4 of `work/w-label/PREREG.md`: >= +1 on the compiler-label
                // counter in 11 of 11 measured cells, and `coff::plan_labels`
                // charges 0. Emitting this would be a wrong `$M` for this
                // function and every later one in the TU.
                return Err(out_of_class(&format!(
                    "a BACKWARD branch to label `{}`: c2 charges the \
                     compiler-label counter at least one extra slot for every \
                     body with a backward intra-section branch (11 of 11 cells, \
                     work/w-label/PREREG.md §1.4; and +1..+4 on 17 LEAF cells, \
                     work/w-loop/loopcost.py Q1) and `coff::plan_labels` \
                     charges none, so a TU that mints any $M would carry a wrong \
                     one as well as a wrong block. The magnitude is measured \
                     (+1/+2/+3/+4) and NOT modelled. The $M consequence needs a \
                     FRAMED function in the TU to land on (Q2: 34 of 34 \
                     leaf-only TUs mint zero labels, 17 of 17 leaf+framed \
                     controls mint a triple) — that relaxation belongs in the \
                     TU-level gate, not in this per-body map",
                    self.names.get(r.label.0).copied().unwrap_or("?")
                )));
            }
            // Signed, and computed in `i64` before it narrows: with a back edge
            // admitted, `target - r.at` in `usize` is an underflow that would
            // wrap to a colossal positive displacement — which `encode_bc`
            // would then refuse for being out of field, so the bug would read
            // as a range refusal rather than as arithmetic. A `.text` section
            // cannot reach `i32::MAX`, so the narrowing is total in practice
            // and is written as a `try_from` rather than an `as` anyway.
            let disp = i32::try_from(target as i64 - r.at as i64).map_err(|_| {
                out_of_class("a label displacement outside a 32-bit .text offset")
            })?;
            let word = r.form.encode(disp).ok_or_else(|| {
                out_of_class(&format!(
                    "a `{}` past its displacement field: the long-branch \
                     expansion is measured in docs/CFG_SHAPE.md §3.3.1 and not \
                     built",
                    r.form.name()
                ))
            })?;
            text[r.at..r.at + 4].copy_from_slice(&word);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The map's own happy path: two references to one label, resolved after
    /// the label is defined, both carrying their true self-relative
    /// displacement and neither taking a relocation.
    #[test]
    fn two_references_to_one_forward_label_resolve_to_their_own_displacements() {
        let mut m = LabelMap::new();
        let epi = m.mint("epilogue");
        let mut t = Vec::new();
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x05]); // li r3,5
        m.reference(&mut t, epi, Form::B); // at 4
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x0b]); // li r3,11
        m.reference(&mut t, epi, Form::B); // at 12
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x00]); // li r3,0
        assert_eq!(m.pending(), 2);
        m.define(epi, &t).unwrap();
        m.resolve(&mut t).unwrap();
        // 4 -> 20 is +16; 12 -> 20 is +8.
        assert_eq!(&t[4..8], &[0x48, 0x00, 0x00, 0x10]);
        assert_eq!(&t[12..16], &[0x48, 0x00, 0x00, 0x08]);
    }

    #[test]
    fn a_conditional_reference_patches_the_bc_form() {
        let mut m = LabelMap::new();
        let join = m.mint("join");
        let mut t = Vec::new();
        m.reference(&mut t, join, Form::Bc { bo: 12, bi: 26 }); // at 0
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x05]);
        m.define(join, &t).unwrap();
        m.resolve(&mut t).unwrap();
        assert_eq!(&t[0..4], &[0x41, 0x9a, 0x00, 0x08]);
    }

    /// **The ordering rule this module exists for.** A backward reference is
    /// refused, and the refusal names the label and the counter.
    #[test]
    fn a_backward_reference_is_refused_because_it_moves_the_label_counter() {
        let mut m = LabelMap::new();
        let top = m.mint("loop-top");
        let mut t = Vec::new();
        m.define(top, &t).unwrap(); // at 0
        t.extend_from_slice(&[0x38, 0x60, 0x00, 0x05]);
        m.reference(&mut t, top, Form::Bc { bo: 4, bi: 26 }); // at 4, target 0
        let err = m.resolve(&mut t).unwrap_err();
        let s = format!("{err:?}");
        assert!(s.contains("BACKWARD"), "{s}");
        assert!(s.contains("loop-top"), "{s}");
        assert!(s.contains("plan_labels"), "{s}");
    }

    /// A self-reference is backward by the same rule (`target <= at`), which is
    /// the boundary case the `<=` rather than `<` is there for.
    #[test]
    fn a_self_reference_is_refused_by_the_same_rule() {
        let mut m = LabelMap::new();
        let l = m.mint("self");
        let mut t = Vec::new();
        m.define(l, &t).unwrap();
        m.reference(&mut t, l, Form::B);
        // The reference sits at 0 and the label is bound to 0.
        let err = m.resolve(&mut t).unwrap_err();
        assert!(format!("{err:?}").contains("BACKWARD"));
    }

    #[test]
    fn an_undefined_label_is_refused_and_names_itself() {
        let mut m = LabelMap::new();
        let l = m.mint("never-defined");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::B);
        let err = m.resolve(&mut t).unwrap_err();
        assert!(format!("{err:?}").contains("never-defined"));
    }

    #[test]
    fn defining_a_label_twice_is_refused() {
        let mut m = LabelMap::new();
        let l = m.mint("join");
        let mut t = Vec::new();
        m.define(l, &t).unwrap();
        t.extend_from_slice(&[0; 4]);
        let err = m.define(l, &t).unwrap_err();
        assert!(format!("{err:?}").contains("defined twice"));
    }

    /// A caller that overwrote its own pending site gets a refusal rather than a
    /// branch patched on top of an instruction.
    #[test]
    fn a_fixup_site_that_was_written_over_is_refused() {
        let mut m = LabelMap::new();
        let l = m.mint("epilogue");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::B);
        t[0..4].copy_from_slice(&[0x38, 0x60, 0x00, 0x05]); // the caller's bug
        t.extend_from_slice(&[0; 4]);
        m.define(l, &t).unwrap();
        let err = m.resolve(&mut t).unwrap_err();
        assert!(format!("{err:?}").contains("zero placeholder"));
    }

    /// The displacement-range refusal, per form. `CFG_SHAPE.md` §3.3.1 brackets
    /// the real transition between +32628 (direct) and +34148 (expanded); the
    /// encoders already refuse past the architectural limit and this checks the
    /// map propagates that refusal rather than truncating.
    #[test]
    fn a_bc_past_its_field_is_refused_with_the_expansion_named() {
        let mut m = LabelMap::new();
        let l = m.mint("far");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::Bc { bo: 12, bi: 26 });
        t.resize(40_000, 0x60); // well past the 14-bit BD field
        m.define(l, &t).unwrap();
        let err = m.resolve(&mut t).unwrap_err();
        let s = format!("{err:?}");
        assert!(s.contains("displacement field"), "{s}");
        assert!(s.contains("3.3.1"), "{s}");
    }

    /// …and the same body is *fine* for the wider `LI` field, which is what
    /// makes the previous test a statement about the field rather than about
    /// the length.
    #[test]
    fn the_same_distance_is_in_range_for_the_wider_b_field() {
        let mut m = LabelMap::new();
        let l = m.mint("far");
        let mut t = Vec::new();
        m.reference(&mut t, l, Form::B);
        t.resize(40_000, 0x60);
        m.define(l, &t).unwrap();
        m.resolve(&mut t).unwrap();
        assert_eq!(&t[0..4], &(0x4800_0000u32 | 40_000).to_be_bytes());
    }

    /// A label that is minted and defined but never referenced is not an error:
    /// the `/Ox` arm of an early return duplicates the epilogue instead of
    /// branching to it, so a body legitimately finishes with labels nobody
    /// named.
    #[test]
    fn an_unreferenced_label_is_not_an_error() {
        let mut m = LabelMap::new();
        let l = m.mint("epilogue");
        let mut t = vec![0x38, 0x60, 0x00, 0x05];
        m.define(l, &t).unwrap();
        assert_eq!(m.pending(), 0);
        m.resolve(&mut t).unwrap();
        assert_eq!(&t[..], &[0x38, 0x60, 0x00, 0x05]);
    }

    // ---- lane `w-fencea`: invariant 4's admission ---------------------------

    /// **The admission set is CLOSED, and this is the line that closes it.**
    ///
    /// For every [`ChargedClass`] — the `match` is exhaustive, so a new variant
    /// does not compile until it is answered here — build the `IlFunction` that
    /// carries only that shape and read `c2_il`'s **own** three-valued counter
    /// gate off it. One of exactly two things has to hold, and a class that is
    /// neither is the wrong-`$M` case invariant 4 was built for:
    ///
    /// 1. `label_slots(false) == None` — `IlBundle::functions` refuses every TU
    ///    in which this body's `$M` could be observed (board #742's Q2: 34 of 34
    ///    leaf-only TUs mint zero labels);
    /// 2. `label_slots(false) == Some(label_lead() + 1)` with `label_lead() >=
    ///    1` — `coff::plan_labels` already advances the surcharge.
    ///
    /// **No number is copied into this crate.** The lead is read from
    /// `IlFunction::label_lead`, which is the one place it lives — `#3148`
    /// records three published label numbers that drifted because a second copy
    /// existed.
    #[test]
    fn every_admitted_class_has_a_registered_control_flow_surcharge() {
        use c2_il::{
            ChainOp, ChainOpKind, ChainRhs, JsonUtf8CopyFn, PtrWalkChainLoop, PtrWalkModLoop,
            XteaEncryptLoop,
        };
        for class in ChargedClass::ALL {
            let mut f = crate::codegen::testutil::func_with(vec![0x09EA, 0x09EB], Vec::new());
            match class {
                ChargedClass::PtrWalkModLoop => {
                    f.ptr_walk_loop = Some(PtrWalkModLoop {
                        params: vec![0x09EA, 0x09EB],
                        acc_init: 0,
                        mul_k: 127,
                    })
                }
                ChargedClass::XteaEncryptLoop => {
                    f.xtea_encrypt_loop = Some(XteaEncryptLoop {
                        callee: "?Encipher@XTEABlockEncrypter@@AAA_K_KPAI@Z".to_string(),
                        key_off: 16,
                        nonce_off: 0,
                        trips: 2,
                    })
                }
                ChargedClass::PtrWalkChainLoop => {
                    f.ptr_walk_chain_loop = Some(PtrWalkChainLoop {
                        params: vec![0x09E3],
                        acc_init: 0,
                        elem_unsigned: false,
                        ops: vec![ChainOp { kind: ChainOpKind::Add, rhs: ChainRhs::Char }],
                    })
                }
                ChargedClass::JsonUtf8Copy => {
                    f.json_utf8_copy = Some(JsonUtf8CopyFn {
                        params: vec![0x09f3, 0x09f0, 0x09f1],
                        off_buffer: 0,
                        off_size: 4,
                        k_arg_err: 0x8007_0057u32 as i32,
                        k_size_err: 0x803F_0005u32 as i32,
                    })
                }
            }
            let slots = f.label_slots(false);
            let lead = f.label_lead();
            // `plan_labels` advances `label_lead + 4` for a framed function
            // (5 under `/Gy`, its own `$M`/`$M`/`$T` triple) and `label_lead +
            // 1` otherwise. **Both already contain the class's surcharge**, so
            // the frame class decides the constant and never whether the
            // surcharge is charged. `xtea_encrypt_loop` is framed and
            // `ptr_walk_loop` is not, which is why this arm is written over the
            // predicate rather than assuming one of them.
            let ok = match slots {
                // Arm 2: the TU-level gate refuses every observable TU.
                None => true,
                // Arm 1: `plan_labels` already advances the surcharge, and it
                // is a REAL surcharge — a lead of 0 is the defect, not the
                // evidence.
                Some(k) => lead >= 1 && k == lead + if f.is_framed() { 4 } else { 1 },
            };
            assert!(
                ok,
                "{} is admitted past invariant 4 with label_slots(false) = {slots:?}, \
                 label_lead() = {lead}, framed = {}: neither `None` (the TU gate refuses \
                 every observable TU) nor a NON-ZERO lead that `plan_labels` already \
                 advances. That is exactly the wrong-$M case invariant 4 exists for",
                class.il_shape(),
                f.is_framed()
            );
        }
    }

    /// **`json_utf8_copy` comes in on arm 1 as a FRAMED class**, and board
    /// `#3155`'s published `label_slots` `Some(5)` is the **non-framed**
    /// formula.
    ///
    /// The class has no `stwu` — its Class C prologue is two words — and
    /// `is_framed()` is still true, because what that predicate means is a
    /// `.pdata` record and a `$M`/`$M`/`$T` triple, both of which this class
    /// carries. So `coff::plan_labels` advances `label_lead() + 4`, not `+ 1`.
    ///
    /// **No number is written down here**, deliberately (`#3148`): the test
    /// asserts which *arm* the class comes in on and that the `+ 1` reading is
    /// impossible, so it stays true if `c2_il` re-measures the lead and goes red
    /// if the class stops being framed — which is the change that would actually
    /// matter.
    #[test]
    fn json_utf8_copy_is_admitted_as_a_framed_class_and_not_at_lead_plus_one() {
        use c2_il::JsonUtf8CopyFn;
        let mut f = crate::codegen::testutil::func_with(vec![0x09f3, 0x09f0, 0x09f1], Vec::new());
        f.json_utf8_copy = Some(JsonUtf8CopyFn {
            params: vec![0x09f3, 0x09f0, 0x09f1],
            off_buffer: 0,
            off_size: 4,
            k_arg_err: 0x8007_0057u32 as i32,
            k_size_err: 0x803F_0005u32 as i32,
        });
        assert!(f.is_framed(), "a .pdata record and a $M/$M/$T triple, with no stwu");
        let lead = f.label_lead();
        assert!(lead >= 1, "arm 1 needs a REAL surcharge; a lead of 0 is the defect");
        assert_eq!(f.label_slots(false), Some(lead + 4), "the FRAMED constant");
        assert_ne!(
            f.label_slots(false),
            Some(lead + 1),
            "board #3155 publishes the non-framed formula for this class"
        );
    }

    /// [`ChargedClass::ALL`] really is all of them. Written as an exhaustive
    /// `match` returning an index, so that a variant added without being listed
    /// fails here rather than quietly leaving the grading test above with a
    /// smaller set than the enum.
    #[test]
    fn the_admitted_class_list_is_complete() {
        fn index(c: ChargedClass) -> usize {
            match c {
                ChargedClass::PtrWalkModLoop => 0,
                ChargedClass::XteaEncryptLoop => 1,
                ChargedClass::PtrWalkChainLoop => 2,
                ChargedClass::JsonUtf8Copy => 3,
            }
        }
        assert_eq!(ChargedClass::ALL.len(), 4);
        for (i, c) in ChargedClass::ALL.iter().enumerate() {
            assert_eq!(index(*c), i, "{} is out of place in ALL", c.il_shape());
        }
    }

    /// **The admission actually patches a back edge, to its true negative
    /// displacement** — `ptr_walk_loop`'s own `-48`, which is the word
    /// `4082ffd0` that `work/w-hash/Sort.obj` carries.
    ///
    /// The displacement is `LabelMap`'s and nothing in this test computes it.
    #[test]
    fn an_admitted_back_edge_resolves_to_its_true_negative_displacement() {
        let mut m = LabelMap::admitting_back_edges(ChargedClass::PtrWalkModLoop);
        let top = m.mint("loop-top");
        let mut t = Vec::new();
        m.define(top, &t).unwrap(); // the loop top is at 0
        t.resize(48, 0x60); // twelve words of body
        m.reference(&mut t, top, Form::Bc { bo: 4, bi: 2 }); // at 48, target 0
        m.resolve(&mut t).unwrap();
        assert_eq!(&t[48..52], &[0x40, 0x82, 0xff, 0xd0]);
    }

    /// …and the **default** map is unchanged, on the identical body. This is the
    /// pair that says the admission is the only thing that moved.
    #[test]
    fn the_same_back_edge_through_a_default_map_is_still_refused() {
        let mut m = LabelMap::new();
        assert_eq!(m.back_edge(), BackEdge::Refused);
        let top = m.mint("loop-top");
        let mut t = Vec::new();
        m.define(top, &t).unwrap();
        t.resize(48, 0x60);
        m.reference(&mut t, top, Form::Bc { bo: 4, bi: 2 });
        let s = format!("{:?}", m.resolve(&mut t).unwrap_err());
        assert!(s.contains("BACKWARD"), "{s}");
        assert!(s.contains("plan_labels"), "{s}");
    }

    /// **A zero-displacement self reference is refused even under an
    /// admission.** A branch to its own word is not a back edge, and no measured
    /// c2 body carries one — this is what the `<=` rather than `<` in invariant
    /// 4 still buys after the lift.
    #[test]
    fn a_self_reference_is_refused_even_for_an_admitted_class() {
        let mut m = LabelMap::admitting_back_edges(ChargedClass::PtrWalkModLoop);
        let l = m.mint("self");
        let mut t = Vec::new();
        m.define(l, &t).unwrap();
        m.reference(&mut t, l, Form::B);
        let s = format!("{:?}", m.resolve(&mut t).unwrap_err());
        assert!(s.contains("BACKWARD"), "{s}");
    }

    /// The displacement range check reaches a **backward** reference too. A
    /// `bc` 40,000 bytes back is as far out of the `BD` field as one 40,000
    /// bytes forward, and the refusal still names §3.3.1's unbuilt expansion
    /// rather than truncating — the arithmetic being signed is what this asserts.
    #[test]
    fn a_backward_bc_past_its_field_is_refused_with_the_expansion_named() {
        let mut m = LabelMap::admitting_back_edges(ChargedClass::PtrWalkModLoop);
        let top = m.mint("far-back");
        let mut t = Vec::new();
        m.define(top, &t).unwrap();
        t.resize(40_000, 0x60);
        m.reference(&mut t, top, Form::Bc { bo: 4, bi: 2 });
        let s = format!("{:?}", m.resolve(&mut t).unwrap_err());
        assert!(s.contains("displacement field"), "{s}");
        assert!(s.contains("3.3.1"), "{s}");
        // …and the same distance IS in range for the wider `LI` field, which is
        // what makes this a statement about the field and not about the sign.
        let mut m2 = LabelMap::admitting_back_edges(ChargedClass::PtrWalkModLoop);
        let top2 = m2.mint("far-back");
        let mut t2 = Vec::new();
        m2.define(top2, &t2).unwrap();
        t2.resize(40_000, 0x60);
        m2.reference(&mut t2, top2, Form::B);
        m2.resolve(&mut t2).unwrap();
        assert_eq!(&t2[40_000..40_004], &(0x4800_0000u32 | (-40_000i32 as u32 & 0x03FF_FFFC)).to_be_bytes());
    }

    /// A label minted by one map and used against another is caught rather than
    /// silently reading the neighbouring label's offset.
    #[test]
    fn a_label_from_another_map_is_refused() {
        let mut a = LabelMap::new();
        let _ = a.mint("a0");
        let stray = a.mint("a1");
        let mut b = LabelMap::new();
        let mut t = Vec::new();
        let err = b.define(stray, &t).unwrap_err();
        assert!(format!("{err:?}").contains("different function's map"));
        // …and on the reference side, where the index is out of range.
        let mut b2 = LabelMap::new();
        b2.reference(&mut t, stray, Form::B);
        assert!(b2.resolve(&mut t).is_err());
    }
}

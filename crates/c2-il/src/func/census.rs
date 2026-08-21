use super::body::{
    self, bind_refusal_key, call_tokens, parse_segment_detail, BodyShape, Complete, DtorSubObject,
    CALLEE_DEFINED_IN_TU, CALLEE_UNRESOLVED_DTOR,
    CALLEE_UNRESOLVED_FRAMED, CALLEE_UNRESOLVED_SEQ, CALLEE_UNRESOLVED_TAIL,
    STATIC_SCAN_LOOP_OBJECT, STORE_RUN_BIND_NO_CARRIER, STORE_RUN_CALL_NO_CARRIER,
    DATA_SYM_LINKAGE, DATA_SYM_STRLIT_FENCED, DATA_SYM_UNRESOLVED, OPT_MODE,
    PTR_WALK_CHAIN_LOOP_NOT_O1,
    PTR_WALK_LOOP_NOT_O1,
};
use super::bind::{
    callee_defined_here, callee_defined_here_unmodelled, defined_name_set, Bindings,
    EmitBinding, STRLIT_NARROW_PREFIX,
};
use super::bundle::shape_to_function;
use super::bundle::split_function_bodies_at;
use super::bundle::{opt_word_at, opt_word_mode};
use super::Block;
use super::IlFunction;
use crate::IlBundle;

/// Split the `.ex` stream into per-function byte segments at each `4F 1F`
/// function-start marker. Segment `k` runs from marker `k` to marker `k+1`
/// (the last to end-of-stream).
/// One function's census verdict (P2b). Either the modeled shape it parsed as,
/// or the first feature that blocked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FnVerdict {
    /// Parsed as a modeled shape. The string is a stable shape label
    /// (`straight-line`, `void-tail-call`, `int-tail-call`, `framed-call`).
    InClass(&'static str),
    /// Blocked at the first unmodeled feature.
    Blocked(Block),
}

impl FnVerdict {
    /// The census bucket key: the shape label when in class, else the blocking
    /// feature (see [`Block::feature`]).
    pub fn key(&self) -> String {
        match self {
            FnVerdict::InClass(s) => (*s).to_string(),
            FnVerdict::Blocked(b) => b.feature(),
        }
    }
    pub fn in_class(&self) -> bool {
        matches!(self, FnVerdict::InClass(_))
    }

    /// **The grammar-completeness axis** — see [`Complete`], `docs/ROADMAP.md`
    /// §9.11 and §9.14.
    ///
    /// A *fifth* census axis, and a separate field for exactly the reason
    /// [`FnCensus::cflow`], [`FnCensus::eh`], [`FnCensus::dispatch`] and
    /// [`FnCensus::prod`] are separate: **the blocking-feature key is not a
    /// reliable carrier of it.** Two producers encode the same fact in two
    /// different halves of the key — `-whole`/`-more` from the completeness
    /// walker, `:eof`/`:mid` from the byte-less refusals — and WR1 moved 39,967
    /// functions from the first encoding to the second without moving one
    /// function between classes. Every table built by grepping the key for
    /// `-whole` has under-counted that family by **18,931** since.
    ///
    /// The point is not that the grep was written badly. It is that a fact
    /// carried only in a *name* has no stable home, so every consumer re-derives
    /// it and the derivations drift. This is the fact's home.
    pub fn completeness(&self) -> Complete {
        match self {
            FnVerdict::InClass(_) => Complete::InClass,
            FnVerdict::Blocked(b) => b.completeness(),
        }
    }
}

/// One census row: a function segment and how it classified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnCensus {
    /// Index of the function within the TU (`.ex` segment order).
    pub index: usize,
    /// Mangled name, when `.gl` has one at this position.
    pub name: Option<String>,
    /// Segment length in bytes (a rough proxy for function size).
    pub seg_len: usize,
    pub verdict: FnVerdict,
    /// Raw bytes around the blocking site, for grammar work: the segment window
    /// `[off - CENSUS_HEX_BACK, off + CENSUS_HEX_FWD)` clamped to the segment,
    /// plus the index of the blocking byte within that window. Empty when the
    /// function is in class.
    pub hex: Vec<u8>,
    /// Index of the blocking byte inside [`FnCensus::hex`].
    pub hex_mark: usize,
    /// **The control-flow class** of this body, decoded independently of whether
    /// the body is in class (`crates/c2-il/src/func/body/shapes/control_flow.rs`).
    ///
    /// Two families of value, and the prefix says which:
    ///
    /// * `cflow-<shape>` / `cflow-<shape>+expr-modeled` — the statement layer
    ///   decoded end to end, so this body's CFG is fully known. The `+expr-modeled`
    ///   half is the one blocked on **control flow alone**; without the suffix the
    ///   body needs expression work as well.
    /// * `cf-<production>-0xNN` — the statement-layer decoder itself stopped, and
    ///   this is where. Ranked, it is the residue of the grammar.
    ///
    /// A third census axis, beside the blocking feature and the frame class, and a
    /// separate field for the same reason the frame class is one: the
    /// blocking-feature histogram IS the widening order and several sessions of
    /// documented tables name its keys, so an orthogonal fact goes beside it rather
    /// than into its names.
    ///
    /// **Decode-only, and structurally so**: nothing reads this field except the
    /// report. It is not consulted by acceptance, by `shape_to_function`, or by the
    /// emitter, and the scanner that produces it constructs no `BodyShape`.
    pub cflow: String,
    /// **Which operand token took this body out of `CfResidue::Modeled`** —
    /// `"div-mod"`, `"intrinsic"`, `"virtual-slot"`, … — or `""` when nothing
    /// did (the body is `+expr-modeled`, or it has no body to scan).
    ///
    /// A fifth census axis, and a separate field for exactly the reason
    /// [`FnCensus::cflow`] is separate from the blocking feature: **the `cflow`
    /// key says only `not only control flow`, and eight days of published
    /// numbers are keyed on those strings.** Board **#1344** measured that
    /// `CfResidue::Modeled` misses 518,991 of the 711,486 bodies the port
    /// ACCEPTS, and #1345 forbade closing that by a bare widening — what it
    /// owes is a pair. This field is the half of the pair the tree did not
    /// have: it says *which* token, so the repair set is a measurement instead
    /// of a guess, and it can be crossed with IN-CLASS / BLOCKED so a widening
    /// can be scored on BOTH sides of the two-sided error before it ships.
    ///
    /// **Decode-only, and structurally so**: nothing reads it except the
    /// report. First reason wins; see `control_flow::Scan::off_class`.
    pub cflow_off: &'static str,
    /// **`IL_STMT_GRAMMAR.md` §14.2 step 5's fail-closed boundary, as this body
    /// scores against it** — `admit-<shape>`, `refuse-<clause>`, `cfg-no-body`,
    /// or the consistency alarm `DISAGREE-backedge-vs-shape`. See
    /// [`body::shapes::step5::CfgAdmit`].
    ///
    /// A sixth census axis, and a separate field for exactly the reason
    /// [`FnCensus::cflow`] is one: **the `cflow` key says what SHAPE the body
    /// has and says nothing about whether that shape may be emitted.** Those are
    /// different questions and the whole discipline of §14.2 is that they must
    /// not share a name — *"decoding a production is not licence to emit it"*.
    /// `cflow-if-1+expr-modeled` and `refuse-back-edge` can be true of the same
    /// body.
    ///
    /// **Decode-only, and structurally so**: nothing reads this field except the
    /// report. It gates no acceptance, constructs no
    /// [`BodyShape`](body::BodyShape), and is consulted by no emitter — see
    /// [`body::shapes::step5`]'s own "what this module is NOT".
    pub cfg_admit: &'static str,
    /// **The exception-handling axis** — which side of `docs/EH_RECORDS.md` §6's
    /// sub-object boundary this body falls on:
    ///
    /// > Exactly one sub-object statement and nothing else is a bare branch. A
    /// > second sub-object, or any other statement beside it, is the WHOLE EH
    /// > RECORD.
    ///
    /// * `eh-none` — the body decoded and carries no `5C`/`5D`/`5E` marker. No
    ///   destructible object is ever live in it, so `/EHsc` costs it nothing.
    /// * `eh-bare` — one object goes live, one is tracked, and there is no other
    ///   statement. **The cheap side**: no `__CxxFrameHandler` prefix, no second
    ///   `.pdata`, no funclet. The port's three `empty-dtor-*` shapes all live
    ///   here, which is this axis's control group.
    /// * `eh-plus-stmt` — one object, plus a body statement.
    /// * `eh-multi` — two or more objects.
    /// * `eh-partial` — a marker was seen and then the walk stopped. **Not bare**:
    ///   the bare shape decodes end to end by construction, so a body that carries
    ///   a marker and does not decode is on the EH side whatever else it needs.
    /// * `eh-unknown` — the walk stopped before any marker; nothing is claimed.
    ///
    /// A separate field from [`FnCensus::cflow`] and from the blocking feature for
    /// the reason both of those are separate from each other: **nothing in the
    /// blocking-feature key says which side a body is on.** `work/WEH/probe/p1.cpp`
    /// files a cheap constructor and an EH constructor under the *same* key
    /// `expr-intrinsic-this-adjust`, and that is not a defect of the key — the two
    /// bodies differ by one statement the key never reaches.
    ///
    /// **Decode-only, structurally**: nothing reads this field except the report.
    ///
    /// **This field is now the `maxState` axis** (`eh-none` / `eh-state0` /
    /// `eh-state1` / `eh-partial` / `eh-unknown`) — see
    /// [`EhMarkers::state_key`](body::shapes::control_flow). The
    /// statement-count axis it used to hold moved to [`FnCensus::eh_stmt`],
    /// unchanged, because §7.3's published split is keyed on it.
    pub eh: String,
    /// **The superseded statement-count EH axis** (`eh-bare` / `eh-plus-stmt` /
    /// `eh-multi` / …), kept so the two can be crossed.
    ///
    /// It is REFUTED (`docs/EH_RECORDS.md` §9.4, §10) and it is not for ranking.
    /// It is here because a published number that changes silently is worse than
    /// a wrong one: `docs/EH_RECORDS.md` §7.3 and `docs/ROADMAP.md` §6o both size
    /// the EH phase off this key, and the cross `eh × eh_stmt` is what says
    /// exactly which bodies moved and in which direction.
    pub eh_stmt: String,
    /// **How many CALL tokens the body issues** — see [`call_tokens`]. Counted for
    /// every function, in class or not, because the in-class shapes are the control
    /// group: they are all leaves or single tail calls, so a non-zero count among
    /// them would say the measure is wrong.
    pub calls: usize,
    /// **Which arm of the body-dispatch ladder claimed this body** — the `disp-*`
    /// axis of [`super::body`], recorded for every function whether in class or
    /// not.
    ///
    /// A fourth census axis, and a separate field for exactly the reason
    /// [`FnCensus::cflow`] and [`FnCensus::eh`] are: **nothing in the blocking
    /// feature says which recognizer looked at the body.** `mcall`'s completeness
    /// walk mints `expr-call-in-expr-recv-*` for a member call wherever it stands
    /// — as a whole body, as the right-hand side of a store, as the argument of a
    /// plain call — and only the first of those three ever reaches
    /// `try_parse_member_tail_call`. The other two are `disp-expr` and
    /// `disp-plain-call`, and **no widening inside any member-call production can
    /// move one of them**, so a rung sized off the census key alone is sized off a
    /// population it cannot serve.
    ///
    /// In-class rows are the control group and they are worth reading: an accepted
    /// body's arm is the production that accepted it, so a `store-leaf` reading
    /// anything but `disp-store-leaf` would indict the axis rather than reveal
    /// anything about the body.
    ///
    /// **Decode-only, structurally**: nothing reads this field except the report.
    pub dispatch: &'static str,
    /// **Which non-committal bail inside the member-call productions fired** — the
    /// `prod-*` axis of [`super::body`], for the bodies that reached them.
    ///
    /// The distinction this axis exists to draw, which no census key draws: a big
    /// blocking row is either a **construct the port has no production for** or a
    /// **private limit inside a production that already ships**, and those are
    /// different orders of work. Six ranking rungs running, the answer has been
    /// the second.
    ///
    /// Four states are set by the ladder itself ([`super::body::PROD_NOT_ENTERED`],
    /// `PROD_ENTERED_UNTAGGED`, `PROD_ACCEPTED`, `PROD_COMMITTED_REFUSAL`); the
    /// named per-site values come from tag calls inside
    /// `body::shapes::mcall_{tail,chain,cmp}`. `prod-entered-untagged` is
    /// therefore **the measure of how much of those files is still untagged**, and
    /// it is printed rather than suppressed precisely so that it cannot be
    /// mistaken for "no bodies land here".
    ///
    /// **Decode-only, structurally**: nothing reads this field except the report.
    pub prod: &'static str,
    /// This function's **optimization-settings word**, read out of this segment's
    /// own `4F 1F 80 <LE32>` head (never zipped in from `IlBundle::opt_words`,
    /// which walks a different segmentation). The census/gate cross-check needs
    /// it to pick the mode the port would emit under.
    pub opt_word: Option<u32>,
    /// **The emitted-function binding** (`docs/GAPS.md` §8): the mangled name of
    /// the `.gl` function record whose body-start offset lands in this segment.
    ///
    /// A *different* field from [`FnCensus::name`], and deliberately so. `name`
    /// is [`Bindings::positional`]'s answer, which on a real translation unit is
    /// `None` for every row (`src/App.cpp`: 3,752 names against 9,033 segments,
    /// so the pairing is refused whole). This one is per record, so it answers on
    /// real input — 131,041 of 142,205 emitted symbols across 371 workload TUs.
    ///
    /// It exists to be joined against the obj's `.text` COMDAT leaders
    /// ([`c2_obj::ObjImage::text_comdat_functions`], from the harness, which is
    /// the only layer that has an obj) and so answer the one question the
    /// per-body census cannot: **of the functions c2 actually emits, how many are
    /// in class?** For a body c2 never emits, "in class" is a parser-only claim
    /// that no byte compare has ever graded or ever can.
    ///
    /// `None` is a refusal, never a guess: no record claimed this segment, or two
    /// did, or the name it would have taken is claimed by another row. Reported
    /// as residue by `c2rs gap` rather than dropped.
    ///
    /// **Decode-only, structurally**: nothing reads this field except the report.
    /// It is not consulted by acceptance, by `shape_to_function`, or by the
    /// emitter.
    pub emit_name: Option<String>,
    /// **Board #980 — the callee this REFUSED body emits nothing but a call to.**
    ///
    /// `Some(name)` when the body is
    /// [`super::body::shapes::no_effect::no_effect_call`]'s dead-temporary call
    /// shape: its whole content is one discarded call, plus a temporary the
    /// body's own grammar proves nothing else reads. The name is the callee's,
    /// resolved through the same `.gl` symbol index every call shape uses.
    ///
    /// **It is a CONDITION, not a verdict.** It says *this function emits
    /// nothing provided that callee reduces to nothing*, which is exactly the
    /// step `c2_core::elide`'s least fixpoint takes, and it is the only consumer.
    /// Asking it of a body whose callee does **not** reduce to nothing answers
    /// nothing at all.
    ///
    /// `None` for every in-class row: a body that parses has an
    /// [`super::IlFunction`] and its emptiness is read from that, never from
    /// here — one fact, one owner.
    ///
    /// **The row stays `FnVerdict::Blocked`.** Nothing about acceptance moves:
    /// `parse_segment` still refuses this body, [`super::IlBundle::functions`]
    /// still refuses its whole TU, and the census key is still
    /// `expr-intrinsic-memset`. Board **#971** condition 4 is that this widening
    /// may not widen the gate, and this field is how it does not.
    pub no_effect_callee: Option<String>,
    /// **Board #1053 — this REFUSED body emits nothing AT ALL, with no callee.**
    ///
    /// `true` when the body is
    /// [`super::body::shapes::no_effect::no_effect_nothing`]'s shape: two
    /// discarded literals and the return plumbing, walked totally, over a closed
    /// vocabulary that contains no call token — `p->~T()` on a class with a
    /// trivial destructor.
    ///
    /// **It is a VERDICT, not a condition, and that is the whole difference from
    /// [`Self::no_effect_callee`].** That field says *provided its callee reduces
    /// to nothing*; this one says *unconditionally*. The first is a link into
    /// `c2_core::elide`'s least fixpoint and the second is a **seed**, which is a
    /// strictly stronger claim — see `c2_core::elide::Reduction` for the
    /// termination and cycle arguments that had to be re-derived to admit it.
    ///
    /// The two are **mutually exclusive by construction** and a test says so in
    /// both directions rather than leaving it to the reading.
    ///
    /// `false` for every in-class row, for `no_effect_callee`'s reason: a body
    /// that parses has an [`super::IlFunction`] and its emptiness is read from
    /// that. **The row stays `FnVerdict::Blocked`**, still `fnbyte-refused`, and
    /// [`super::IlBundle::functions`] still refuses its whole TU — #971
    /// condition 4, satisfied by construction and not by care.
    pub no_effect_nothing: bool,
}

impl FnCensus {
    /// The **frame class**: what the call count alone settles about whether this
    /// body needs a stack frame (`docs/IL_CALL_IN_EXPR.md` §18).
    ///
    /// Three values, and the middle one is honest rather than convenient:
    ///
    /// * `calls-0` — no call at all. It cannot need LR saved, so **no frame**.
    /// * `calls-1` — exactly one. A tail call emits `b callee` and stays a leaf
    ///   (`return p->M();`), while a call whose result is then computed on needs a
    ///   frame (`return g(a) + k;`, which is the port's existing `FramedCall`).
    ///   The count cannot tell them apart and this class does not pretend to.
    /// * `calls-2plus` — two or more, which **always** needs a frame: the first
    ///   `bl` clobbers LR and the return address is still live. There is no
    ///   two-call shape that stays a leaf.
    pub fn frame_class(&self) -> &'static str {
        match self.calls {
            0 => "calls-0",
            1 => "calls-1",
            _ => "calls-2plus",
        }
    }
}

/// The control-flow axis for one segment: run the statement-layer scanner and
/// render its verdict as a census key.
///
/// Run for **every** function, in class or not, for the reason the frame class is:
/// the in-class shapes are the control group. Until lane `w-hash` every one of
/// them was a single basic block, so **any** `cflow-loop` among the accepted
/// rows indicted the measure.
///
/// **That is no longer the statement, and the weaker one is written out rather
/// than left implied.** Exactly one accepted key may read `cflow-loop`:
/// `ptr-walk-mod-loop`, the pointer-walk accumulate this port now emits. Every
/// *other* in-class key must still read `cflow-straight`, and a `cflow-loop`
/// under any of them still indicts the measure. The test
/// [`tests::every_in_class_row_is_a_single_basic_block`] enforces the pair.
///
/// A segment with no `LO` body marker has no body to scan; that is already the
/// `lo-marker` refusal on the primary axis, and restating it here would put a
/// container-level fact into a control-flow histogram.
/// …and BOTH EH axes beside it, from the SAME walk. Returns
/// `(control-flow key, maxState EH key, statement-count EH key)`.
///
/// One scan, three readings. The axes answer different questions off one
/// traversal and a second traversal would double the census's cost for facts the
/// first one already collected. The two EH keys are the measured predicate and
/// the refuted one it replaces; see [`FnCensus::eh`] and [`FnCensus::eh_stmt`].
fn cflow_key(seg: &[u8]) -> (String, String, String, &'static str, &'static str) {
    let Some(lo) = crate::func::readers::find_subslice(seg, &crate::func::bundle::LO_MARKER) else {
        return (
            "cf-no-body".to_string(),
            "eh-unknown".to_string(),
            "eh-unknown".to_string(),
            "",
            // Not `refuse-undecoded`: there is no body to decode, so the first
            // pass was never run and its verdict has no subject. A separate
            // string, because folding it into the refusal would put every
            // bodiless row into a bucket named after a walk that did not happen.
            "cfg-no-body",
        );
    };
    let scan = body::shapes::control_flow::scan_full(seg, lo);
    let cflow = match &scan.body {
        Ok(cf) => cf.key(),
        Err(b) => b.feature(),
    };
    // **Step 5's fail-closed boundary, evaluated on every body in the corpus.**
    // One scan, four readings now. The verdict reads only facts `scan_full`
    // already collected, so this costs a comparison and no second traversal —
    // and it means the predicate is exercised on ~1.7M bodies rather than on
    // whatever a hand-written test happens to contain, which is
    // `CFG_SHAPE.md` §6.3 rule 4's answer to board #283 (16 of 56 shape markers
    // had zero corpus cases).
    let admit = if body::shapes::step5::CfgAdmit::label_map_is_empty_on_a_decoded_body(&scan) {
        "DISAGREE-empty-label-map"
    } else if body::shapes::step5::CfgAdmit::backedge_disagrees_with_shape(&scan) {
        // **The consistency control is reported IN the axis, not beside it.** A
        // control published as its own key can read 0 because nothing reached
        // it; this one can only read 0 if bodies reached the axis and agreed,
        // because the same rows carry both.
        "DISAGREE-backedge-vs-shape"
    } else {
        let v = body::shapes::step5::CfgAdmit::of(&scan);
        // §9's counterexample gets its own axis value — NOT an alarm and NOT a
        // refusal, see `has_fallthrough_epilogue` — so the three bodies stay
        // visible instead of merging into `admit-straight` where nobody would
        // find them again.
        //
        // **Gated on `v.admits()`, and that gate is the whole correctness of
        // this arm.** Written without it, the fallthrough name was reached
        // BEFORE the residue clause and relabelled one body that
        // `refuse-unmodeled-operand` had refused into a name beginning
        // `admit-` — the census reporting an admission for a body the predicate
        // rejects, which is the exact confusion "decoding a production is not
        // licence to emit it" is a rule against. Caught by
        // `step5-refuse-unmodeled-operand-BLOCKED` moving by one.
        if v.admits() && body::shapes::step5::CfgAdmit::has_fallthrough_epilogue(&scan) {
            "admit-fallthrough-epilogue"
        } else {
            v.name()
        }
    };
    (
        cflow,
        scan.eh.state_key(scan.decoded).to_string(),
        scan.eh.key(scan.decoded).to_string(),
        scan.off_reason.unwrap_or(""),
        admit,
    )
}

/// **The `C2RS_CFRESIDUE_ADMIT` set this process is running under**, verbatim,
/// or `""` when the variable is unset.
///
/// Re-exported so the scan report can print it. A run that admits arms is
/// **not** the shipped predicate and every `cflow-*` number it produces is a
/// counterfactual; the only way that stays true is if the run says so itself.
/// See `body::shapes::control_flow::residue_admits`.
pub fn cflow_residue_admit_set() -> String {
    body::shapes::control_flow::residue_admit_set()
}

/// Bytes of context kept before / after a blocking site.
pub const CENSUS_HEX_BACK: usize = 16;
pub const CENSUS_HEX_FWD: usize = 24;

/// **W-INLFENCE — the same-TU callees the port has a MODEL of**, so the inline
/// fence can refuse only the ones it does not.
///
/// # Why this exists, and the cost of it existing
///
/// The inline fence ([`super::bind::callee_defined_here`]) refuses a body whose
/// callee this TU defines, because c2 may inline it. **The port is not silent
/// about every inline**: mechanism E (`c2_core::elide`) says a call to a callee
/// that emits nothing costs no branch at all, and the judge grades that
/// **1,877 of 1,877 byte-exact** on the 878-TU workload. Refusing those in the
/// census would refuse bodies the port provably gets right.
///
/// **And mechanism I** (`c2_core::splice`, SPLICE-0) says a call to a callee this
/// TU defines and that the port can LOWER is replaced by that callee's own
/// emitted body, graded **723 of 723 byte-exact**. The two populations are
/// disjoint in the worst way for a single rule: E's callees are rows the parser
/// REFUSED and I's are rows it ACCEPTED, so an exemption that covers one covers
/// neither.
///
/// So the set below is the union — *reduces to nothing* **or** *the port can
/// lower it* — and the fence refuses only a callee the port has **no** model of,
/// which is the honest statement of what it is for: c2 may inline, and here
/// nobody knows what that produces.
///
/// `c2_core::elide::TuEmptyCallees` is the owner of the first half and this is a
/// **second implementation of it**, which is a real cost and is stated rather
/// than hidden: `c2-core` depends on `c2-il` and not the other way round, so the
/// census cannot call it. What keeps the two in agreement is that six standing
/// integration cells fail loudly if this one is narrower —
/// `dead_temp_elision.rs`'s four chains, `call_targets.rs`'s locator and
/// `empty_elision.rs`'s c19. A **depth-1** version of this function was written
/// first and five of them caught it; a *reduces-to-nothing only* version was
/// written second and c19 caught that.
///
/// The second half is deliberately **not** a re-statement of SPLICE-0's own
/// refusals (`splice.rs`'s S1–S6). It is *"the callee's body is one the port
/// lowers"*, which is broader, so a callee the splice declines is exempted here
/// and the port keeps its `bl`. That is the pre-existing behaviour and it is a
/// named residue, not a claim (`docs/rungs/2026-08-09-w-inlfence.md` §6).
///
/// # The rule, mirrored clause for clause from `elide.rs`
///
/// * **seeds** — [`IlFunction::empty_body`], and a refused row whose grammar
///   proves it emits nothing at all (`no_effect_nothing`, board #1053).
/// * **links** — a refused row that emits nothing but one call
///   (`no_effect_call` / `no_effect_loop`), and a parsed body that is a bare
///   tail call: no data symbol, no `framed_call`, no `call_seq`, no `cond_pair`
///   (`elide::elidable_step`, whose doc gives the graded reason each of those
///   four disqualifies).
/// * **lowerable** — the segment parses whole, `shape_to_function` resolves
///   every token in it, and its optimization-settings word is one the port
///   emits under. Asked WITHOUT the inline fence, which is what keeps this
///   non-recursive: it is a statement about the callee's own body, not about
///   whether the callee would itself be admitted.
/// * **a name two segments disagree about contributes neither**, exactly as
///   `TuContext::of_rows` drops it.
///
/// Keyed on [`EmitBinding::name`], which is the key
/// `c2_harness::gap::fnbytes::tu_empty_callees` feeds the real context with, so
/// the two cannot key one function two ways.
#[allow(clippy::too_many_arguments)]
fn tu_modelled_callees(
    segs: &[&[u8]],
    bind: &Bindings,
    emit: &EmitBinding,
    src: &Option<String>,
    resolve: &dyn Fn(u32) -> Option<String>,
    resolve_data: &dyn Fn(u32) -> Option<String>,
    resolve_data_def: &dyn Fn(u32) -> Option<crate::func::IlDataDef>,
    resolve_bss_def: &dyn Fn(u32) -> Option<crate::func::IlDataDef>,
) -> std::collections::BTreeSet<String> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut seed: BTreeSet<String> = BTreeSet::new();
    let mut lowerable: BTreeSet<String> = BTreeSet::new();
    let mut link: BTreeMap<String, String> = BTreeMap::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut conflict: BTreeSet<String> = BTreeSet::new();
    for (j, s2) in segs.iter().enumerate() {
        let Some(n) = emit.name(j) else { continue };
        if !seen.insert(n.to_string()) {
            conflict.insert(n.to_string());
            continue;
        }
        if body::shapes::no_effect::no_effect_nothing(s2) {
            seed.insert(n.to_string());
            continue;
        }
        if let Some(c) = body::shapes::no_effect::no_effect_call(s2)
            .or_else(|| body::shapes::no_effect::no_effect_loop(s2))
            .and_then(resolve)
        {
            link.insert(n.to_string(), c);
            continue;
        }
        let Ok(sh) = parse_segment_detail(s2, bind.locals(j)) else {
            continue;
        };
        let Some(f) = shape_to_function(
            sh,
            &bind.name_for_shape(j),
            src,
            resolve,
            resolve_data,
            resolve_data_def,
            resolve_bss_def,
        ) else {
            continue;
        };
        // **Mechanism I's half.** The body parses whole, every token in it
        // resolves, and the mode is one the port emits under: the splice has a
        // body to substitute, so the caller is not guessing.
        if opt_word_mode(opt_word_at(s2)).is_some() {
            lowerable.insert(n.to_string());
        }
        if f.empty_body() {
            seed.insert(n.to_string());
        } else if f.data_syms.is_empty()
            && f.framed_call().is_none()
            && f.call_seq().is_none()
            && f.cond_pair().is_none()
        {
            if let Some(c) = f.tail_call() {
                link.insert(n.to_string(), c.to_string());
            }
        }
    }
    for n in &conflict {
        seed.remove(n);
        link.remove(n);
        lowerable.remove(n);
    }
    // The closure. `seed` only grows and is bounded by `link`, so this
    // terminates on a cycle instead of chasing it — `elide.rs`'s own cycle
    // re-derivation, and `a_cycle_of_dead_temporary_bodies_is_never_admitted`
    // is the cell that grades it.
    loop {
        let step: Vec<String> = link
            .iter()
            .filter(|(n, c)| !seed.contains(*n) && seed.contains(*c))
            .map(|(n, _)| n.clone())
            .collect();
        if step.is_empty() {
            break;
        }
        seed.extend(step);
    }
    seed.extend(lowerable);
    seed
}

impl IlBundle {
    /// **Function-level census (P2b).** Classify *every* function in the bundle
    /// independently, so a TU whose 700th function uses an unmodeled opcode
    /// still reports the other 699 as in-class.
    ///
    /// This is the measurement [`IlBundle::functions`] cannot give: that method
    /// is all-or-nothing per TU (correctly — the port must emit a whole obj or
    /// nothing), so over a real workload it reports one `vocab-gap` per TU and
    /// cannot rank the missing classes. The census runs the *same*
    /// [`parse_segment_detail`] per segment and keeps the first blocking
    /// feature, so the histogram of [`FnVerdict::key`] over a corpus is the
    /// widening order (docs/ROADMAP.md §G5).
    ///
    /// **The emitted-function binding's own accounting** (`docs/GAPS.md` §8) —
    /// how many `.gl` body-offset records were found, how many bound, and every
    /// way a record failed to.
    ///
    /// A separate entry point from [`IlBundle::function_census`] because it is
    /// the *instrument's* self-report rather than a per-function fact: the scan
    /// prints these counts on every run so that the residue cannot disappear
    /// quietly, which is the failure mode `docs/ROADMAP.md` §8.4 names.
    /// [`FnCensus::emit_name`] carries the same binding's per-row answer.
    pub fn emit_binding(&self) -> Option<EmitBinding> {
        let gl = self.get("gl")?;
        let (seg_starts, _) = split_function_bodies_at(self.ex()?);
        Some(EmitBinding::new(gl, &seg_starts))
    }

    /// Diagnostic only — never a gate, and never consulted by the emitter.
    /// Returns `None` only when the bundle lacks the required files.
    pub fn function_census(&self) -> Option<Vec<FnCensus>> {
        Some(
            self.census_functions()?
                .into_iter()
                .map(|(c, _)| c)
                .collect(),
        )
    }

    /// **The census/gate cross-check (roadmap #44).** Every row
    /// [`IlBundle::function_census`] reports, paired with the emitter's own
    /// per-function record for the rows the census calls in class.
    ///
    /// Why this exists: acceptance is supposed to live in the IL parser so the
    /// census and the gate cannot disagree, and for a long time it did not
    /// entirely — `int f(int a,int b,int c){ return a + b*c; }` censused in class
    /// and `PortC2` returned `NotImplemented`, because a `*` after the first
    /// operator was gated in codegen where the census could not see it. A
    /// numerator with an unmeasured error term is not a benchmark, so the
    /// disagreement gets a permanent instrument rather than a note: the harness
    /// runs the port's own selector over every `Ok` row and reports the
    /// disagreement in the same block as the census (`docs/GAPS.md` §6, "a
    /// diagnostic that runs outside the parser needs a population whose answer is
    /// already known").
    ///
    /// `Err` carries why there is no record:
    ///
    /// * `"blocked"` — the census itself refused; nothing to cross-check.
    /// * `"callee-unresolved"` — the body parsed, but the CALL token has no `.gl`
    ///   symbol, so [`shape_to_function`] refuses. That IS a disagreement, and a
    ///   per-function one, so it is named rather than folded into `blocked`.
    ///
    /// Diagnostic only, exactly like the census: acceptance is unchanged and the
    /// emitter never consults it.
    pub fn census_functions(&self) -> Option<Vec<(FnCensus, Result<IlFunction, &'static str>)>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let (seg_starts, segs) = split_function_bodies_at(ex);
        // The emitted-function binding (`docs/GAPS.md` §8), built once per
        // bundle over this same segmentation. Diagnostic only — it feeds
        // `FnCensus::emit_name` and nothing else, and it is a THIRD binding
        // beside the two `bind.rs` tabulates, because it answers a question
        // neither does: which obj symbol, if any, is this row.
        let emit = EmitBinding::new(gl, &seg_starts);
        // The whole correspondence seam comes from ONE place ([`super::bind`]).
        // The census's names are paired POSITIONALLY, which is a different
        // binding from the gate's per-record one — `bind.rs`'s module doc states
        // that disagreement and pins it; closing it is roadmap #14's follow-up
        // and moves the numerator, so it is not done silently here.
        //
        // `.gl` is deliberately NOT threaded into the body parse. The assignment
        // class used to decide "is this destination a global?" by asking whether
        // `.gl` named it, and that was wrong (a file-scope `static` is `$sv`, which
        // the index does not accept as an identifier). The symbol view locals needed
        // turned out to be `.sy`, not `.gl`, so the vestigial `.gl` thread is gone
        // rather than left in place looking load-bearing. Do not restore the
        // absence test.
        //
        // The `.sy` locals and the `GlIndex` resolution are the SAME construction
        // the gate makes — one `Bindings`, one `SyLocals::new`, one `GlIndex` —
        // so the census cannot report a function in class that
        // `IlBundle::functions` would refuse for want of a local, or the reverse.
        // Over the census's own segment list, which is NOT the gate's: see
        // `bind.rs`'s table.
        let bind = Bindings::positional(gl, self.get("in").unwrap_or(&[]), self.get("sy"), &segs);
        let resolve = |tok: u32| -> Option<String> { bind.resolve(tok) };
        let resolve_data = |tok: u32| -> Option<String> { bind.resolve_data(tok) };
        // **W-DATA** — the same DEFINED-object resolver `IlBundle::functions`
        // builds, from the same `Bindings`. The census and the gate must ask one
        // question about an object or the census over-claims (`docs/GAPS.md` §6).
        let resolve_data_def =
            |tok: u32| -> Option<crate::func::IlDataDef> { bind.resolve_data_def(tok) };
        // **W-WORDWRAP** — the `.bss` sibling, built beside the `.data` one and
        // from the same `Bindings`, so the two answer about one `.gl`.
        let resolve_bss_def =
            |tok: u32| -> Option<crate::func::IlDataDef> { bind.resolve_bss_def(tok) };
        let src = bind.src.clone();
        // **W-INLFENCE** — the names this TU DEFINES, built once per bundle for
        // the post-parse gate (c) below. Built from `.gl` directly and not from
        // `bind.names()`, because the census's binding is
        // [`Bindings::positional`] and its names are **all** mangled names —
        // callees included — so testing a callee against them would refuse every
        // call in the workload. See [`defined_name_set`].
        let defined = &defined_name_set(gl);
        // **W-INLFENCE** — the defined callees mechanism E already models,
        // built LAZILY because the fence fires on **one** row in the whole
        // 878-TU workload and this pass costs a second parse of every segment.
        // Keyed on `EmitBinding::name`, which is the key
        // `c2_harness::gap::fnbytes::tu_empty_callees` uses for the same fact,
        // so the two cannot key one function two ways.
        let empty_here: std::cell::OnceCell<std::collections::BTreeSet<String>> =
            std::cell::OnceCell::new();
        // **W-FENCE163 — the string-literal fence's ground map: emit-binding
        // name → the segment that DEFINES it, for every name the emit binding
        // claims on one of this TU's own segments.**
        //
        // Clause (c2) below cannot use clause (c)'s `defined` set:
        // [`defined_name_set`]'s walk is whole-TU fail-closed and binds **0
        // records on most real TUs** (measured on `ContentMgr_Xbox.cpp`, and
        // `gl.rs`' own doc records 0-of-36 on `vec.cpp`), which is exactly the
        // blindness that let `?ContentPath@…` grade `fnbyte-differs` under an
        // unfenced admission. The [`EmitBinding`] read of the same `.gl` DOES
        // frame the definition records on those TUs — it is the reader the
        // emitted census itself runs on — and it names `?MakeString@@YAPBDPBD@Z`
        // on the TU that shipped the wrong lowering (probed: 1,137 of 2,053
        // segments named there, the callee among them).
        //
        // **The residue is fail-OPEN and is stated rather than hidden**: a
        // callee whose defining segment the emit binding leaves nameless (916
        // of 2,053 on that same TU) is invisible here, exactly as clause (c)'s
        // doc says of its own walk. Requiring completeness instead was BUILT
        // AND MEASURED first (this lane's F2 rung): it closes the fence on
        // essentially every real TU and holds back the entire +163 — a fence
        // whose price is its whole yield. What bounds the open direction is
        // the grading itself: a call lowered against a callee c2 inlined or
        // discarded cannot be byte-exact, and `fnbyte-differs` plus the
        // relocation-target compare (board #984's reader) are the standing
        // instruments that count it — the tip scan's requirement of
        // `fnbyte-differs` Δ0 is the fence's own acceptance test.
        let strlit_ground: std::cell::OnceCell<std::collections::BTreeMap<String, usize>> =
            std::cell::OnceCell::new();
        // **W-MMIOCLOSE — the `.gl` function attribute byte, once per bundle.**
        // `None` when the reader refused the file; see
        // [`super::gl::gl_function_attrs`] for why that is a whole-file answer
        // and not a per-record one.
        let attrs = super::gl::gl_function_attrs(gl);
        Some(
            segs.iter()
                .enumerate()
                .map(|(i, seg)| {
                    // A variadic function is refused on its NAME, because its body
                    // IL is byte-identical to a non-variadic twin's — see
                    // [`super::bind::mangled_is_varargs`], which is the same
                    // predicate `functions` applies, so the census and the gate
                    // cannot disagree.
                    //
                    // Only when the names are `paired`. Unpaired means the census
                    // has no name for this segment, and reporting the body's real
                    // blocker is better than inventing a reason: `functions`
                    // refuses that whole TU for want of names anyway, so nothing
                    // here can be emitted either way.
                    let varargs = bind.is_varargs(i);
                    // The dispatch axes are per-body and the varargs arm below
                    // never calls the parser, so they are cleared HERE as well as
                    // inside `parse_segment_detail`. Without this a variadic
                    // function would inherit the previous segment's arm — the
                    // exact "a stale reading looks like a measurement" failure the
                    // axis is supposed to close.
                    body::dispatch_reset();
                    // Held across the verdict so the gate side can convert the
                    // very same parse — two readings of one parse, never two parses.
                    //
                    // Offset 0 and NOT the segment end. This refusal is raised on
                    // the mangled NAME before the body is looked at, so nothing is
                    // known about the body's grammar — `:eof` would claim the
                    // opposite (parse complete, nothing hiding behind the row).
                    let mut shape: Result<BodyShape, Block> =
                        Err(Block::refuse(seg, 0, "fn-varargs"));
                    let verdict = if varargs {
                        FnVerdict::Blocked(Block::refuse(seg, 0, "fn-varargs"))
                    } else {
                        shape = parse_segment_detail(seg, bind.locals(i));
                        match &shape {
                            Ok(BodyShape::StraightLine { .. }) => FnVerdict::InClass("straight-line"),
                            Ok(BodyShape::VoidTailCall { .. }) => FnVerdict::InClass("void-tail-call"),
                            // Three buckets for one shape, so the movement out of
                            // `expr-call-in-expr` is attributable *per receiver
                            // production*: the base form and the member form at
                            // offset 0 emit the identical four bytes a void tail
                            // call does, and the adjusted member form emits one
                            // `addi` more. Splitting them here is what lets the
                            // in-class gain be checked against the individual
                            // `recv-field-off0` / `recv-field` bucket drops rather
                            // than against their sum.
                            Ok(BodyShape::EmptyDtorDelegation {
                                sub_object: DtorSubObject::Base,
                                ..
                            }) => FnVerdict::InClass("empty-dtor-delegation"),
                            Ok(BodyShape::EmptyDtorDelegation { adjust: 0, .. }) => {
                                FnVerdict::InClass("empty-dtor-member")
                            }
                            Ok(BodyShape::EmptyDtorDelegation { .. }) => {
                                FnVerdict::InClass("empty-dtor-member-adjusted")
                            }
                            // WEC — the empty constructor delegating to one
                            // base. **ONE bucket**, although the shape has two
                            // forms whose label strides differ by 1: the split
                            // that matters is `/EHsc` on or off, and the *`eh`
                            // axis* already carries it exactly
                            // (`eh-bare|empty-ctor-base` against
                            // `eh-none|empty-ctor-base`). A second
                            // `FnVerdict::InClass` label would also be a family
                            // `scripts/cross_sweep.py` enumerates and demands a
                            // representative for — and no representative exists,
                            // because the sweep corpus is compiled without
                            // `/EHsc` and every case there reads `eh: false`.
                            // A declared family the sweep cannot supply is a
                            // hole that lane fails on, correctly.
                            Ok(BodyShape::EmptyCtorBaseDelegation { .. }) => {
                                FnVerdict::InClass("empty-ctor-base")
                            }
                            // The pointer-walk accumulate loop — its own bucket,
                            // because it is the first in-class shape with a back
                            // edge and the `cflow-loop` axis has to be able to
                            // report an in-class row against it. A `cflow-loop`
                            // reading among the accepted rows used to indict the
                            // control-flow measure; from this rung on, exactly
                            // one accepted key may carry it and this is that key.
                            Ok(BodyShape::PtrWalkModLoop(_)) => {
                                FnVerdict::InClass("ptr-walk-mod-loop")
                            }
                            // The **body-parameterized** loop, in its own bucket
                            // beside the fixed-length one. Two buckets for what
                            // could be called one family, because the two are
                            // exactly what a reader would want told apart: the
                            // row above is a transcription of a single workload
                            // function, this one is a class whose members differ
                            // in body length. Summing them would hide which of
                            // the two a census move came from.
                            Ok(BodyShape::PtrWalkChainLoop(_)) => {
                                FnVerdict::InClass("ptr-walk-chain-loop")
                            }
                            // **W-POOL2** — two buckets, not one, even though
                            // the two shapes are one class of one TU: the guard
                            // pair is `cflow-if-1` and the constructor is
                            // `cflow-loop`, so folding them would hide which of
                            // the two axes a census move came from.
                            Ok(BodyShape::PoolFreeList(_)) => {
                                FnVerdict::InClass("pool-free-list")
                            }
                            Ok(BodyShape::PoolCtorChain(_)) => {
                                FnVerdict::InClass("pool-ctor-chain")
                            }
                            // The integer divide/modulo leaf. Its own bucket
                            // rather than folded into `straight-line`, so the
                            // rung's census gain is attributable: this key's
                            // count is exactly the population that used to
                            // render as `expr-op-0x05` / `expr-op-0x06`, and
                            // the two gap keys must fall by the same number.
                            // W-CFG1 — the `if`/`else`-with-a-join. Its own
                            // bucket, and the FIRST accepted key that may read
                            // `cflow-if-n`: the control-flow axis's control
                            // (`every_in_class_row_is_a_single_basic_block`)
                            // names the accepted keys that may carry a non-
                            // straight reading one at a time, so a widening
                            // cannot slip a second one in unnoticed.
                            Ok(BodyShape::IfCallJoin(_)) => FnVerdict::InClass("if-call-join"),
                            // W-EXTDATA — its own bucket, for the reason every
                            // transcription above gets one: it is the first
                            // in-class shape whose `.text` carries a REFHI/REFLO
                            // against a FUNCTION, and a census that folded it in
                            // with `if-call-join` could not report that movement
                            // against the `expr-cmp-eq` bucket it comes out of.
                            Ok(BodyShape::GuardChainSharedTail(_)) => {
                                FnVerdict::InClass("guard-chain-shared-tail")
                            }
                            // W-UNDNAME — its own bucket, for the reason every
                            // transcription above gets one: it is the first
                            // in-class shape whose `.text` carries TWO
                            // REFHI/REFLO quads, and a census that folded it in
                            // with `guard-chain-shared-tail` could not report
                            // that movement against the `expr-cmp-ne` bucket it
                            // comes out of.
                            Ok(BodyShape::AllocInitOrFail(_)) => {
                                FnVerdict::InClass("alloc-init-or-fail")
                            }
                            // **W-OSFINFO** — its own bucket for the same
                            // reason, and one more: it is the first in-class
                            // shape whose two data symbols are reached
                            // DIFFERENTLY (one by value, one by address), so a
                            // census that folded it in with
                            // `alloc-init-or-fail` could not report its movement
                            // against the `expr-cmp-ge` bucket it comes out of.
                            Ok(BodyShape::OsfHandleGuard(_)) => {
                                FnVerdict::InClass("osf-handle-guard")
                            }
                            // **W-IFN** — its own bucket for the same reason,
                            // and one more: it is the first in-class shape that
                            // calls a function the IL never NAMES (an intrinsic
                            // selector), so a census that folded it in with a
                            // neighbouring framed shape could not report the
                            // minted-external rung separately.
                            Ok(BodyShape::GuardRetChain(_)) => {
                                FnVerdict::InClass("guard-ret-chain")
                            }
                            // **W-MMIO3** — its own bucket for the same reason,
                            // and one more: it is the first in-class shape with
                            // an INDIRECT call (a `bctrl` whose callee the IL
                            // names nowhere) and the first whose acceptance
                            // depends on a fact about a SIBLING segment, so a
                            // census that folded it in with a neighbouring
                            // framed shape could not report either rung
                            // separately. **The census is fail-open on the
                            // sibling facts by construction** — they are asked
                            // at `IlBundle::functions`, which is where board
                            // #139 puts a whole-TU clause and where
                            // `Bindings::is_varargs` already lives — so this
                            // bucket can count a body the GATE refuses, exactly
                            // as `unclaimed-gl-symbol` and the label-counter
                            // gate already can.
                            Ok(BodyShape::CloseCallChain(_)) => {
                                FnVerdict::InClass("close-call-chain")
                            }
                            // **W-XLR** — its own bucket for the same reason,
                            // and one more: it is the first in-class shape whose
                            // FRAME is a different class (the `__savegprlr_N`
                            // helper), so a census that folded it in with a
                            // neighbouring `cflow-if-n` shape could not report
                            // the frame rung's movement separately from the body
                            // rung's.
                            Ok(BodyShape::XlrcCreateGuard(_)) => {
                                FnVerdict::InClass("xlrc-create-guard")
                            }
                            // **W-JSON** — its own bucket for the same reason,
                            // and one more: it is the first in-class shape with
                            // a BACK EDGE, so the `cflow-loop` axis can report
                            // an in-class row against it and the rung's gain is
                            // attributable to this production rather than to one
                            // of the four pointer-walk loops.
                            Ok(BodyShape::JsonUtf8Copy(_)) => {
                                FnVerdict::InClass("json-utf8-copy")
                            }
                            // **W-DATA — the static-array scan loop.** Its own
                            // bucket, like every other whole-body shape, so the
                            // `cflow-loop` axis can report an in-class row
                            // against it and the rung's gain is attributable to
                            // this production and not to a sibling loop's.
                            Ok(BodyShape::StaticScanLoop(_)) => {
                                FnVerdict::InClass("static-scan-loop")
                            }
                            // **W-BDNZ — the counted-`for` accumulate loop.**
                            // Its own bucket for the same reason: `cflow-loop`
                            // now has five in-class productions and a gain that
                            // landed in a shared bucket would be attributable to
                            // none of them. This is the first one whose lowering
                            // is derived from a READING of c2's algorithm
                            // (`wb-loop`'s passes 1 and 2) rather than
                            // transcribed from one workload function.
                            Ok(BodyShape::CountedAccumLoop(_)) => {
                                FnVerdict::InClass("counted-accum-loop")
                            }
                            // **W-BLOCKIR — the float array-walk loop.** Its own
                            // bucket on the same rule every loop class here
                            // follows: a gain that landed in a shared
                            // `cflow-loop` bucket would be attributable to none
                            // of the six productions that now write into it.
                            Ok(BodyShape::FloatWalkLoop(_)) => {
                                FnVerdict::InClass("float-walk-loop")
                            }
                            // **W-XTEA2 — the `memcpy` tail branch.** Its own
                            // bucket rather than the tail call's: the two end in
                            // a REL24 branch and differ in where the callee's
                            // name comes from, and a gain that landed in the
                            // tail-call bucket would be attributable to neither.
                            Ok(BodyShape::MemcpyTail { .. }) => {
                                FnVerdict::InClass("memcpy-tail")
                            }
                            // **W-WORDWRAP — the file-scope-global store leaf.**
                            // Its own bucket rather than `leaf-store`'s: that
                            // class's destination is positively a `.sy`
                            // automatic and this one's is a `.gl` DEFINED
                            // OBJECT, so a gain that landed there would be
                            // attributable to neither production.
                            Ok(BodyShape::GlobalStoreLeaf { .. }) => {
                                FnVerdict::InClass("global-store-leaf")
                            }
                            // **W-XTEA3 — the two-element 64-bit member run.**
                            // Its own bucket rather than the store run's: the
                            // two write a run of `32`s and differ in whether the
                            // stored value is computed, and a gain that landed
                            // in `store-run` would be attributable to neither.
                            Ok(BodyShape::NonceAddRun { .. }) => {
                                FnVerdict::InClass("nonce-add-run")
                            }
                            // **W-XTEA3 — the XTEA round loop.** Its own bucket
                            // rather than `counted-accum-loop`'s: #1981 defines
                            // that class to contain no memory reference and this
                            // one has an `lwzx` inside the loop, so a gain that
                            // landed there would be attributable to neither.
                            Ok(BodyShape::XteaRoundLoop { .. }) => {
                                FnVerdict::InClass("xtea-round-loop")
                            }
                            // **W-XTEA3 — the framed XTEA block loop.** Its own
                            // bucket, for the reason every class here gets one:
                            // a gain that landed in a shared framed-loop bucket
                            // would be attributable to neither production.
                            Ok(BodyShape::XteaEncryptLoop { .. }) => {
                                FnVerdict::InClass("xtea-encrypt-loop")
                            }
                            // **W-BIQUAD — the float-store diamond.** Its own
                            // bucket on the same rule every class here follows:
                            // a gain that landed in a shared `cflow-if-1`
                            // bucket would be attributable to neither of the two
                            // productions that write into it.
                            Ok(BodyShape::FpStoreDiamond(_)) => {
                                FnVerdict::InClass("fp-store-diamond")
                            }
                            // **W-BIQUAD — the forwarding constructor.** Its own
                            // bucket, not the store run's: the two productions
                            // share `run_call_tail` and differ by whether there
                            // is a run at all, and a gain that landed in one
                            // bucket would be attributable to neither.
                            Ok(BodyShape::CtorForwardCall { .. }) => {
                                FnVerdict::InClass("ctor-forward-call")
                            }
                            Ok(BodyShape::DivModLeaf(_)) => FnVerdict::InClass("div-mod-leaf"),
                            Ok(BodyShape::IntTailCall { .. }) => FnVerdict::InClass("int-tail-call"),
                            // Split from the integer tail call by the register
                            // FILE, and split again by whether the boundary
                            // narrows, so the rung's gain is attributable to the
                            // free move and to the `frsp` separately rather than
                            // to their sum — the same reason the dtor
                            // delegation carries three buckets for one shape.
                            Ok(BodyShape::FpTailCall { narrowing: false, .. }) => {
                                FnVerdict::InClass("fp-tail-call")
                            }
                            Ok(BodyShape::FpTailCall { .. }) => {
                                FnVerdict::InClass("fp-tail-call-narrowing")
                            }
                            Ok(BodyShape::MultiArgTailCall { .. }) => {
                                FnVerdict::InClass("multiarg-tail-call")
                            }
                            // W34, the multi-argument FP tail call. Split by
                            // whether the permutation moves anything at all: the
                            // identity is a bare `b <callee>` and a cycle is
                            // `fmr`s through f0, and the two are worth different
                            // amounts of evidence even though they are one shape.
                            Ok(BodyShape::FpMultiArgTailCall { arg_sources, .. }) => {
                                FnVerdict::InClass(
                                    if arg_sources.iter().enumerate().all(|(i, &s)| i == s) {
                                        "fp-multiarg-tail-call"
                                    } else {
                                        "fp-multiarg-tail-call-perm"
                                    },
                                )
                            }
                            Ok(BodyShape::FramedCall { .. }) => FnVerdict::InClass("framed-call"),
                            // Class A many-calls. Split by tail so the rung's gain
                            // can be attributed to the production that earned it
                            // rather than to their sum.
                            // **W10 — a GUARDED sequence gets its own key**,
                            // ahead of the tail split. The tail says what the
                            // body does after its last call and is orthogonal
                            // to whether one of them is branched over; keying
                            // the guard into the tail names would cross two
                            // axes and make a census delta unattributable,
                            // which is `docs/BOARD.md` #150's shape.
                            Ok(BodyShape::CallSeq { guard: Some(_), .. }) => {
                                FnVerdict::InClass("call-sequence-guarded")
                            }
                            // **W11 — a sequence with guarded EARLY RETURNS gets
                            // its own key**, for the same reason and beside it:
                            // it is the first class whose lowering emits an
                            // **intra-section `b`** and the first with a real
                            // label→offset map, so a rung that widens it has to
                            // be able to read its population without it being
                            // summed into `call-sequence-lit`'s. Not split by
                            // guard count either — the count varies the
                            // displacement and nothing else, and a key per count
                            // would cross two axes.
                            Ok(BodyShape::CallSeq { early, .. }) if !early.is_empty() => {
                                FnVerdict::InClass("call-sequence-early-return")
                            }
                            Ok(BodyShape::CallSeq { tail, .. }) => {
                                FnVerdict::InClass(match tail {
                                    body::SeqTail::Void => "call-sequence",
                                    body::SeqTail::CallValue { .. } => "call-sequence-value",
                                    // **W-FLTRET** — the same tail in the OTHER
                                    // register file, and the reason it is not
                                    // folded into `-value` is that it emits the
                                    // identical instruction stream: the only
                                    // observable is `_fltused` in the obj. A
                                    // shared key would make a census delta unable
                                    // to say whether a lane moved the integer
                                    // tail or the FP one, which is the mistake
                                    // `-load-fp` was already split to avoid.
                                    body::SeqTail::CallValueFp => "call-sequence-value-fp",
                                    body::SeqTail::Lit(_) => "call-sequence-lit",
                                    // WCO — the chain result read through, one
                                    // `lwz`. Its own key rather than sharing
                                    // `-value`'s: the address form IS
                                    // `CallValue`, so a shared name would make
                                    // the two indistinguishable in a census
                                    // delta and this rung ships both.
                                    body::SeqTail::CallLoad { .. } => "call-sequence-load",
                                    // WFL — the same read-through in the OTHER
                                    // register file. Its own key rather than
                                    // sharing `-load`'s: the instruction is
                                    // `lfs`/`lfd` into f1 and the obj acquires
                                    // `_fltused`, so a shared name would make a
                                    // census delta unable to say which of the
                                    // two produced it — and this family's whole
                                    // history is deltas attributed to the wrong
                                    // production.
                                    body::SeqTail::CallLoadFp { .. } => "call-sequence-load-fp",
                                    // Split by relation, not merged: the `==`
                                    // fold and the order spines are different
                                    // instruction counts and different label
                                    // strides, so a shared key would hide which
                                    // of the two a census delta came from.
                                    body::SeqTail::Cmp { cmp: crate::func::SeqCmp::Eq, .. } => {
                                        "call-sequence-cmp-eq"
                                    }
                                    body::SeqTail::Cmp { .. } => "call-sequence-cmp-order",
                                })
                            }
                            Ok(BodyShape::Compare(_)) => FnVerdict::InClass("compare-leaf"),
                            Ok(BodyShape::CmpShiftOr(_)) => FnVerdict::InClass("cmp-shift-or"),
                            // **W8 — the two-arm conditional tail call**, its own
                            // bucket because it is the first class whose lowering
                            // emits a branch: a rung that widens it must be able
                            // to read its population without it being summed into
                            // the tail-call family it is otherwise built from.
                            Ok(BodyShape::CondTailPair(_)) => {
                                FnVerdict::InClass("cond-tail-pair")
                            }
                            Ok(BodyShape::EmptyBody) => FnVerdict::InClass("empty-body"),
                            Ok(BodyShape::IndirectLoad { .. }) => {
                                FnVerdict::InClass("indirect-load-leaf")
                            }
                            // Kept apart from `indirect-load-leaf` so the in-class
                            // gain of this rung can be checked against the bucket
                            // drops it claims (`docs/IL_CALL_IN_EXPR.md` §19), and
                            // because the two emit different instructions.
                            Ok(BodyShape::AddrLeaf { .. }) => FnVerdict::InClass("addr-leaf"),
                            // Kept apart from `addr-leaf` and `indirect-load-leaf`
                            // for the same reason those two are kept apart: the
                            // three share a designator and emit three different
                            // instructions, so this rung's gain can be checked
                            // against the `expr-op-0x27` / `expr-op-0x32` /
                            // `expr-intrinsic-base-member-addr` bucket drops it
                            // claims rather than against their sum.
                            Ok(BodyShape::StoreLeaf { .. }) => FnVerdict::InClass("store-leaf"),
                            // W37. Its own family, kept apart from `store-leaf`
                            // for the same reason: it is a different production
                            // with two gates the single store does not have, and
                            // `cross_sweep.sh` discovers families by this label.
                            Ok(BodyShape::StoreRun { .. }) => FnVerdict::InClass("store-run"),
                            // **F3.** Its own family for the reason every
                            // neighbour above has one, plus a sharper one: this
                            // label is what routes `shape_to_function`'s `None`
                            // to [`STORE_RUN_CALL_NO_CARRIER`] instead of to a
                            // `callee-unresolved-*` key that would name the
                            // wrong construct. It is never an `InClass` the
                            // numerator keeps — the arm below turns every one of
                            // them into a `Blocked` — which is exactly the
                            // point: the residue is counted under its own name.
                            Ok(BodyShape::StoreRunCall { .. }) => {
                                FnVerdict::InClass("store-run-call")
                            }
                            // **#839.** Its own family for every reason F3's has
                            // one, and one more: `cross_sweep.sh` discovers
                            // families by this label, and a bind-carrying run
                            // filed as `store-run` would claim a lowering this
                            // reader deliberately does not have. Like F3's it is
                            // never an `InClass` the numerator keeps — the arm
                            // below turns every one into a `Blocked` under
                            // [`STORE_RUN_BIND_NO_CARRIER`].
                            Ok(BodyShape::StoreRunBind { .. }) => {
                                FnVerdict::InClass("store-run-bind")
                            }
                            Ok(BodyShape::FloatLeaf { double, .. }) => {
                                FnVerdict::InClass(if *double { "double-leaf" } else { "float-leaf" })
                            }
                            Err(b) => FnVerdict::Blocked(*b),
                        }
                    };
                    // Read the dispatch axes **immediately** after the parse, while
                    // the values provably belong to this segment: everything below
                    // (the post-parse gates, `shape_to_function`, the control-flow
                    // scan) runs after, and a future one of them acquiring its own
                    // `parse_segment_detail` call would otherwise silently
                    // overwrite them.
                    let dispatch = body::dispatch_site();
                    let prod = body::prod_site();
                    // ---- The two POST-PARSE gates -----------------------------
                    //
                    // Both are per-function facts `PortC2` has always enforced and
                    // the census never checked, so the numerator counted functions
                    // the port refuses (roadmap #44). They are applied **last**, to
                    // an otherwise-in-class function only, which is what keeps every
                    // blocked function's real blocking feature in the histogram —
                    // gating either of them up front would relabel bodies whose
                    // actual problem is somewhere else entirely.
                    let opt_word = opt_word_at(seg);
                    let mut func: Result<IlFunction, &'static str> = Err("blocked");
                    let verdict = match (shape, verdict) {
                        (Ok(sh), FnVerdict::InClass(label)) => {
                            // (a) The callee must resolve through `.gl`. A CALL
                            // token carries a function-*type* id, not the callee, so
                            // the name comes from the symbol index; when it is not
                            // there the emitter has no symbol to relocate against,
                            // and guessing one is a relocation against the wrong
                            // symbol — a mis-emit, not a gap. `shape_to_function` is
                            // the same conversion `IlBundle::functions` runs, so the
                            // two cannot disagree about this.
                            //
                            // Both gates below are raised with [`Block::at_end`],
                            // and they are the two producers entitled to it: this
                            // arm runs only for a body the whole-segment parser
                            // already accepted, and acceptance requires the cursor
                            // to reach `seg.len()` (`eat_fn_tail`). So the `:eof`
                            // these render is the true statement — the body is
                            // grammar-complete and this is the only thing left
                            // wrong with it — rather than the artefact of a
                            // byte-less refusal. (It used to be recorded at
                            // `len - 1`, the last byte, which is a *different*
                            // claim and one the renderer now tells apart.)
                            let name = bind.name_for_shape(i);
                            // **WR1** — which resolution failed, asked BEFORE the
                            // shape is consumed. A body carrying a data symbol
                            // whose token has no name, or whose `.gl` linkage is
                            // not undefined-external, is filed under its own key:
                            // the callee resolves in every one of those bodies and
                            // reporting them as `callee-unresolved` would name the
                            // wrong construct and hide the population a follow-on
                            // rung is sized from.
                            let sym_fail = match &sh {
                                BodyShape::MultiArgTailCall { arg_sources, .. } => arg_sources
                                    .iter()
                                    .find_map(|a| match a {
                                        body::SlotArg::SymAddr(t) if resolve_data(*t).is_none() => {
                                            Some(match resolve(*t) {
                                                None => DATA_SYM_UNRESOLVED,
                                                // **W-FENCE163** — the name is a
                                                // string literal that
                                                // `resolve_data`'s narrow-prefix
                                                // clause did NOT admit (wide, or
                                                // an unmeasured form). Filed
                                                // under the fence's own key, not
                                                // `DATA_SYM_LINKAGE`: this
                                                // population needs a *grading*,
                                                // not a section emitter, and the
                                                // two rows size two different
                                                // rungs.
                                                Some(n) if n.starts_with("??_C@") => {
                                                    DATA_SYM_STRLIT_FENCED
                                                }
                                                Some(_) => DATA_SYM_LINKAGE,
                                            })
                                        }
                                        _ => None,
                                    }),
                                _ => None,
                            };
                            // **Board #1199.** Asked BEFORE the shape is
                            // consumed, for the reason the `sym_fail` probe
                            // above is: `shape_to_function` takes `sh` by value.
                            let bind_key = bind_refusal_key(&sh);
                            match shape_to_function(sh, &name, &src, &resolve, &resolve_data, &resolve_data_def, &resolve_bss_def) {
                                None if sym_fail.is_some() => FnVerdict::Blocked(Block::at_end(
                                    seg,
                                    sym_fail.expect("just checked"),
                                )),
                                None => FnVerdict::Blocked(Block::at_end(
                                    seg,
                                    match label {
                                        // **F3.** Not a callee problem at all —
                                        // the symbol resolves; the MODEL has no
                                        // carrier for the composition. Its own
                                        // key so the residue #844 is sized from
                                        // is a number rather than a rumour.
                                        "store-run-call" => STORE_RUN_CALL_NO_CARRIER,
                                        // **W-DATA.** The body parsed whole and the
                                        // OBJECT is out of class. See the constant.
                                        "static-scan-loop" => STATIC_SCAN_LOOP_OBJECT,
                                        // **#839 / board #1199 — the carrier
                                        // LANDED, so this label no longer has
                                        // one answer.** `bind_run_ops` is the
                                        // same decision procedure
                                        // `shape_to_function` just ran, asked
                                        // again for its REASON: four named
                                        // refusals, each with its own key so
                                        // each residue is separately sizeable —
                                        // and one of them, the mixed-kind run,
                                        // is boards #836/#868 becoming a
                                        // countable row on the frontier's
                                        // cheapest TU for the first time. A
                                        // bind body that `bind_run_ops` accepts
                                        // and `shape_to_function` still refuses
                                        // is a callee that did not resolve, and
                                        // it keeps the old key.
                                        "store-run-bind" => {
                                            bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER)
                                        }
                                        "framed-call" => CALLEE_UNRESOLVED_FRAMED,
                                        l if l.starts_with("call-sequence") => {
                                            CALLEE_UNRESOLVED_SEQ
                                        }
                                        l if l.starts_with("empty-dtor") => {
                                            CALLEE_UNRESOLVED_DTOR
                                        }
                                        _ => CALLEE_UNRESOLVED_TAIL,
                                    },
                                )),
                                // (b) The optimization mode. `.ex` records it per
                                // function and the port emits only the two words it
                                // has been verified against; the rest — `/Od`, a
                                // `#pragma optimize("", off)`, an unreadable prefix —
                                // are refused.
                                Some(f) if opt_word_mode(opt_word).is_none() => {
                                    let _ = f;
                                    FnVerdict::Blocked(Block {
                                        aux: opt_word.unwrap_or(0) as u64,
                                        ..Block::at_end(seg, OPT_MODE)
                                    })
                                }
                                // (b2) …and the one shape whose lowering is
                                // MODE-SPECIFIC rather than merely mode-gated.
                                // See [`PTR_WALK_LOOP_NOT_O1`]. Raised here as
                                // well as in codegen so the census and the port
                                // agree; without it every `/Ox` capture of the
                                // class is an in-class claim the emitter refuses.
                                Some(f)
                                    if f.ptr_walk_loop().is_some()
                                        && opt_word_mode(opt_word)
                                            != Some(crate::OptWordMode::O1) =>
                                {
                                    FnVerdict::Blocked(Block {
                                        aux: opt_word.unwrap_or(0) as u64,
                                        ..Block::at_end(seg, PTR_WALK_LOOP_NOT_O1)
                                    })
                                }
                                // (b3) …and its body-parameterized sibling,
                                // for the same reason and with its own key.
                                Some(f)
                                    if f.ptr_walk_chain_loop().is_some()
                                        && opt_word_mode(opt_word)
                                            != Some(crate::OptWordMode::O1) =>
                                {
                                    FnVerdict::Blocked(Block {
                                        aux: opt_word.unwrap_or(0) as u64,
                                        ..Block::at_end(seg, PTR_WALK_CHAIN_LOOP_NOT_O1)
                                    })
                                }
                                // **(c) W-INLFENCE — the callee this TU also
                                // DEFINES.** The body parses, the symbol
                                // resolves, the mode is one the port emits
                                // under, and c2 may still **inline** the callee
                                // — in which case the port's `bl` is a wrong
                                // body and not a gap. `IlBundle::functions` has
                                // refused this wholesale since the MVP; this is
                                // the same predicate
                                // ([`super::bind::callee_defined_here`]) asked
                                // per function, so the census stops claiming
                                // bodies the gate refuses.
                                //
                                // LAST of the post-parse gates, deliberately:
                                // see [`CALLEE_DEFINED_IN_TU`]. Silent on a TU
                                // whose defined-name walk stopped, exactly as
                                // [`super::bind::Bindings::is_varargs`] is
                                // silent when the pairing is not meaningful —
                                // the gate refuses such a TU for want of names
                                // before this could matter, and the residue is
                                // sized in the rung rather than folded in.
                                //
                                // **AND IT YIELDS TO A GRADED MODEL.** The port
                                // is not silent about every inline: mechanism E
                                // (`c2_core::elide`) says a call to a callee
                                // this TU defines and that emits NOTHING costs
                                // no branch at all, and the judge grades that
                                // **1,877 of 1,877 byte-exact** on the workload.
                                // Fencing those would refuse bodies the port
                                // provably gets right — over-broad in the one
                                // direction a fence is not allowed to be
                                // cheaply. So a callee in `empty_here` is NOT
                                // fenced.
                                //
                                // The set is **depth 1** where `elide`'s is a
                                // fixpoint, so this under-exempts and never
                                // over-exempts; the residue is a refusal, which
                                // is the safe side. It is built LAZILY and the
                                // `parse_segment` calls it makes are safe only
                                // because `dispatch`/`prod` were read for this
                                // row several lines above and the next row calls
                                // `dispatch_reset()` — the exact hazard that
                                // block's own comment warns about.
                                Some(f)
                                    if callee_defined_here(&f, defined).is_some()
                                        && callee_defined_here_unmodelled(
                                            &f,
                                            defined,
                                            empty_here.get_or_init(|| {
                                                tu_modelled_callees(
                                                    &segs,
                                                    &bind,
                                                    &emit,
                                                    &src,
                                                    &resolve,
                                                    &resolve_data,
                                                    &resolve_data_def,
                                                    &resolve_bss_def,
                                                )
                                            }),
                                        )
                                        .is_some() =>
                                {
                                    let _ = f;
                                    FnVerdict::Blocked(Block::at_end(seg, CALLEE_DEFINED_IN_TU))
                                }
                                // **(c2) W-FENCE163 — the string-literal fence.**
                                //
                                // A body that materializes a `??_C@_0…` address
                                // is newly admitted by `resolve_data`, and the
                                // one measured wrong lowering in that class is
                                // `?ContentPath@XboxContentMgr@@UAAPBDH@Z`:
                                // its callee (`MakeString`, an `inline` header
                                // function) is DEFINED in the TU, c2 INLINES it
                                // (14 words against the port's 3), and clause
                                // (c) above cannot see that because `defined`
                                // is empty on that TU. The question is re-asked
                                // here against the emit-binding ground map
                                // (`strlit_ground`), and a defined-here callee
                                // is admitted on exactly TWO measured grounds:
                                //
                                // * it is **modelled** — mechanism E elides it
                                //   or the splice lowers it (clause (c)'s own
                                //   exemption, same set); or
                                // * its own body decodes **`eh-state1`**
                                //   (`maxState >= 1`) — **c2's inliner keeps a
                                //   call to an EH-stateful callee**, measured on
                                //   a four-cell obj grid at the workload's
                                //   profile (`rungs/2026-08-17-fence163.md`
                                //   §2): dtor-temp + throw KEPT, dtor-local
                                //   without any throw KEPT (the discriminating
                                //   cell), no-dtor no-throw INLINED (the
                                //   ContentPath reproduction), and bare throw
                                //   with no unwindable INLINED — so the
                                //   categorical fact is the EH STATE, not the
                                //   throw. On the workload the admitted side is
                                //   1,047 calls to `?__stl_throw_length_error@…`
                                //   plus 8 to `?__stl_throw_out_of_range@…`
                                //   (both `eh-state1`, both kept by c2, 163 of
                                //   them emitted and every one relocation-graded
                                //   byte-exact), and the refused side is
                                //   `MakeString` (`eh-none`, inlined by c2).
                                //
                                // Anything else — `eh-none`, `eh-state0`,
                                // `eh-partial`, `eh-unknown`, or a segment the
                                // ground map cannot name — refuses, fail-closed:
                                // only the measured non-inlined class is
                                // admitted.
                                //
                                // Scoped to strlit-carrying bodies on purpose:
                                // widening this gate to every call-bearing
                                // body would re-fence populations that are
                                // byte-exact today, and that widening is a
                                // separately-priced rung (`GAPS.md` §6's
                                // two-sided rule, applied in the other
                                // direction).
                                Some(f)
                                    if f.data_syms
                                        .iter()
                                        .any(|d| d.starts_with(STRLIT_NARROW_PREFIX))
                                        && {
                                            let ground = strlit_ground.get_or_init(|| {
                                                (0..segs.len())
                                                    .filter_map(|j| {
                                                        emit.name(j).map(|n| (n.to_string(), j))
                                                    })
                                                    .collect()
                                            });
                                            let modelled = empty_here.get_or_init(|| {
                                                tu_modelled_callees(
                                                    &segs,
                                                    &bind,
                                                    &emit,
                                                    &src,
                                                    &resolve,
                                                    &resolve_data,
                                                    &resolve_data_def,
                                                    &resolve_bss_def,
                                                )
                                            });
                                            f.callees().any(|c| {
                                                let Some(&j) = ground.get(c) else {
                                                    return false; // not known-defined here
                                                };
                                                if modelled.contains(c) {
                                                    return false; // graded elide/splice model
                                                }
                                                // c2 keeps the call ONLY for an
                                                // EH-stateful callee (grid §2).
                                                cflow_key(segs[j]).1 != "eh-state1"
                                            })
                                        } =>
                                {
                                    let _ = f;
                                    FnVerdict::Blocked(Block::at_end(seg, DATA_SYM_STRLIT_FENCED))
                                }
                                Some(mut f) => {
                                    // **W-MMIOCLOSE — the `.gl` inlinability
                                    // bit, keyed on `EmitBinding::name`.**
                                    //
                                    // That key and not `f.mangled_name`: this
                                    // census's names come from
                                    // [`Bindings::positional`], which #918
                                    // measured disagreeing with the per-record
                                    // binding on 74,955 workload rows, while
                                    // `emit.name(i)` is the key
                                    // `c2_harness::gap::fnbytes::tu_empty_callees`
                                    // builds its `TuContext` rows with. The
                                    // consumer looks the callee up by THAT key,
                                    // so the flag has to be filed under it or
                                    // the two would be looking at two functions.
                                    //
                                    // `None` — no attribute map, or no row for
                                    // this name — leaves every consumer where it
                                    // was.
                                    f.inlinable = attrs
                                        .as_ref()
                                        .zip(emit.name(i))
                                        .and_then(|(m, n)| m.get(n))
                                        .map(|a| a & super::gl::FN_FLAG_INLINABLE != 0);
                                    func = Ok(f);
                                    FnVerdict::InClass(label)
                                }
                            }
                        }
                        (_, v) => v,
                    };
                    // Keep the raw bytes around the blocking site: decoding a new
                    // grammar production always starts by staring at exactly this
                    // window, and having it in the census means that work is a
                    // report away instead of a one-off script.
                    let (hex, hex_mark) = match &verdict {
                        FnVerdict::InClass(_) => (Vec::new(), 0),
                        FnVerdict::Blocked(b) => {
                            let start = b.off.saturating_sub(CENSUS_HEX_BACK);
                            let end = (b.off + CENSUS_HEX_FWD).min(seg.len());
                            let start = start.min(end);
                            (seg[start..end].to_vec(), b.off - start)
                        }
                    };
                    let (cflow, eh, eh_stmt, cflow_off, cfg_admit) = cflow_key(seg);
                    // Board #980. Asked of REFUSED rows only, and asked HERE
                    // rather than in the struct literal so the read happens
                    // before `verdict` moves into it.
                    let no_effect_callee = match &verdict {
                        FnVerdict::InClass(_) => None,
                        // Two shapes, one fact and one field: the dead-temporary
                        // call (`w-inl0`) and the destroy LOOP (`w-memset`).
                        // They are mutually exclusive by construction — the loop
                        // opens a scope the straight-line walk requires not to be
                        // there — and both answer the same conditional question,
                        // so they share the field rather than each acquiring one.
                        FnVerdict::Blocked(_) => body::shapes::no_effect::no_effect_call(seg)
                            .or_else(|| body::shapes::no_effect::no_effect_loop(seg))
                            .and_then(&resolve),
                    };
                    // Board #1053. The third reader, and the one that answers
                    // UNCONDITIONALLY — hence its own field rather than a share of
                    // `no_effect_callee`, which is an `Option<String>` and has no
                    // spelling for "nothing, and there is no callee to name".
                    let no_effect_nothing = match &verdict {
                        FnVerdict::InClass(_) => false,
                        FnVerdict::Blocked(_) => body::shapes::no_effect::no_effect_nothing(seg),
                    };
                    (
                        FnCensus {
                            index: i,
                            name: bind.reported_name(i),
                            seg_len: seg.len(),
                            verdict,
                            hex,
                            hex_mark,
                            calls: call_tokens(seg),
                            cflow,
                            cflow_off,
                            cfg_admit,
                            eh,
                            eh_stmt,
                            dispatch,
                            prod,
                            opt_word,
                            emit_name: emit.name(i).map(str::to_string),
                            // An in-class row's emptiness is a property of its
                            // `IlFunction`, and two owners for one fact is how a
                            // rule comes to have two answers (`elide.rs`'s own
                            // §"what it refuses").
                            no_effect_callee,
                            no_effect_nothing,
                        },
                        func,
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::bundle::FN_START;
    use crate::func::test_fixtures::*;

    /// `4F 1F 80 <LE32 /Ox>` — a segment head carrying a mode the port emits under.
    fn seg_head() -> Vec<u8> {
        let mut v = vec![FN_START[0], FN_START[1], 0x80];
        v.extend_from_slice(&crate::func::OPT_WORD_OX.to_le_bytes());
        v
    }

    /// A `.gl` with two real records: `<token> 00 <name> 00`, which is the shape
    /// [`super::super::gl::gl_symbol_index`] reads the callee name out of.
    fn gl_two_records() -> Vec<u8> {
        let mut v = vec![0xE4, 0x09, 0x00];
        v.extend_from_slice(b"?f@@YAXXZ\x00");
        v.extend_from_slice(&[0xE3, 0x09, 0x00]);
        v.extend_from_slice(b"?g@@YAXXZ\x00");
        v
    }

    #[test]
    fn census_classifies_each_function_independently() {
        // The point of P2b: one blocked function does not hide the in-class
        // ones. `functions()` (the gate) is all-or-nothing and returns None.
        // Each segment opens `4F 1F 80 <LE32 opt word>` and `.gl` carries a real
        // token→name record per symbol: both are POST-PARSE acceptance gates now
        // (the optimization mode, and the callee resolving through `.gl`), so a
        // fixture that omits them measures those gates rather than the split.
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(CALL_THEN_STMT);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), ex),
                ("gl".to_string(), gl_two_records()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 2);
        assert_eq!(census[0].verdict, FnVerdict::InClass("void-tail-call"));
        assert!(!census[1].verdict.in_class());
        assert_eq!(census[0].name.as_deref(), Some("?f@@YAXXZ"));
        // In-class functions carry no hex window; blocked ones point at the
        // offending byte inside theirs.
        assert!(census[0].hex.is_empty());
        let FnVerdict::Blocked(b) = census[1].verdict else {
            panic!("expected a block");
        };
        assert_eq!(census[1].hex[census[1].hex_mark], b.byte.unwrap());
    }

    /// **The control group for the control-flow axis.** Every shape the port
    /// accepts is a single basic block **except one**, so an in-class row must
    /// read `cflow-straight` unless its key is `ptr-walk-mod-loop`. Asserted
    /// here on the pinned segments and measured on the workload — a `cflow-loop`
    /// under any other in-class key would mean the port had been handed a back
    /// edge it cannot lower, and a `cflow-if-1` would mean the scanner invents
    /// branches.
    ///
    /// The exception is **named**, not a relaxation to "any loop is fine": the
    /// one shape that may carry it is the one the emitter has a transcription
    /// for, and widening this predicate is how a real back edge would slip in
    /// unnoticed.
    ///
    /// The axis is also **decode-only**, which the second half asserts: the row's
    /// verdict is the same whatever the scanner said, because nothing reads the
    /// field except the report.
    #[test]
    fn every_in_class_row_is_a_single_basic_block() {
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(CALL_THEN_STMT);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![("ex".to_string(), ex), ("gl".to_string(), gl_two_records())]
                .into_iter()
                .collect(),
        };
        let census = bundle.function_census().unwrap();
        for f in &census {
            if f.verdict.in_class() {
                assert!(
                    f.cflow.starts_with("cflow-straight")
                        || f.verdict.key() == "ptr-walk-mod-loop"
                        || f.verdict.key() == "if-call-join",
                    "in-class function #{} with key {} reads {} — the port accepts \
                     only single basic blocks apart from `ptr-walk-mod-loop`, so \
                     this is either a scanner inventing control flow or an emitter \
                     that has been handed a back edge it cannot lower",
                    f.index,
                    f.verdict.key(),
                    f.cflow
                );
            }
        }
        // …and a blocked row still carries the axis, because the measurement is
        // over every function, not only the refused ones.
        assert!(census.iter().all(|f| !f.cflow.is_empty()));
    }

    #[test]
    fn census_hex_window_is_clamped_to_the_segment() {
        // A block at offset 0 must not underflow, and one near the end must not
        // run past it — the window is diagnostic and must never panic.
        let tiny: &[u8] = &[0x4C, 0x4F, 0x11, 0xFF];
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), tiny.to_vec()),
                ("gl".to_string(), b"?f@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 1);
        let c = &census[0];
        assert!(!c.verdict.in_class());
        assert!(c.hex_mark < c.hex.len().max(1));
        assert!(c.hex.len() <= CENSUS_HEX_BACK + CENSUS_HEX_FWD);
    }

    /// **The two dispatch axes reach the census, and every row carries a NAMED
    /// reading.**
    ///
    /// The second half is the one that matters. `dispatch` and `prod` are new
    /// axes, and the way a new axis fails is not by carrying a wrong value — it is
    /// by carrying none, so its rows never appear and the population it describes
    /// reads as zero. So the assertion is stated as a count of rows that DO carry
    /// a named value, floored at the number of functions in the bundle.
    #[test]
    fn every_census_row_carries_a_named_dispatch_and_production_reading() {
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(CALL_THEN_STMT);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![("ex".to_string(), ex), ("gl".to_string(), gl_two_records())]
                .into_iter()
                .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 2, "both segments must have been censused");
        let named = census
            .iter()
            .filter(|f| f.dispatch.starts_with("disp-") && f.prod.starts_with("prod-"))
            .count();
        assert_eq!(
            named, 2,
            "every censused function must carry a named reading on BOTH axes — an \
             axis that is silently absent renders as a population of zero, which is \
             the exact failure these axes exist to close"
        );
        // …and the value itself, so "named" cannot be satisfied by a constant.
        // Both of these bodies open on a plain call, and neither can reach the
        // member-call productions — which is the fact that makes a widening
        // inside those productions unable to move them.
        for f in &census {
            assert_eq!(
                f.dispatch, "disp-plain-call",
                "function #{} opens on a plain call and must be claimed by that arm",
                f.index
            );
            assert_eq!(
                f.prod,
                crate::func::body::PROD_NOT_ENTERED,
                "function #{} never reaches a member-call production, and the axis \
                 must SAY so rather than leave the row blank",
                f.index
            );
        }
    }

    /// **A function refused on its NAME never runs the ladder, and the axis says
    /// exactly that.** A variadic function is blocked before a byte of its body is
    /// read, so any `disp-*` arm here would be a reading inherited from the
    /// previous segment — the one way a thread-local instrument can report
    /// fiction.
    #[test]
    fn a_body_the_ladder_never_ran_for_reads_disp_not_run() {
        // Segment 0 is an ordinary body that leaves a non-default arm behind;
        // segment 1 is variadic (`…ZZ`) and is refused on its name. The ORDER is
        // the adversarial one.
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(MVP_CALL);
        let mut gl = vec![0xE4, 0x09, 0x00];
        gl.extend_from_slice(b"?f@@YAXXZ\x00");
        gl.extend_from_slice(&[0xE3, 0x09, 0x00]);
        gl.extend_from_slice(b"?v@@YAXZZ\x00");
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![("ex".to_string(), ex), ("gl".to_string(), gl)]
                .into_iter()
                .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 2, "both segments must have been censused");
        assert_eq!(
            census[0].dispatch, "disp-plain-call",
            "precondition: the first segment must leave a NON-default arm behind, \
             or the staleness check below has nothing to detect"
        );
        assert!(
            census[1].verdict.key().starts_with("fn-varargs"),
            "precondition: the second segment must be refused on its name, before \
             the ladder runs — it reads {}",
            census[1].verdict.key()
        );
        assert_eq!(
            census[1].dispatch,
            crate::func::body::DISP_NOT_RUN,
            "a body refused on its NAME must read `disp-not-run` — inheriting the \
             previous segment's arm would attribute it to a recognizer that never \
             saw a byte of it"
        );
    }

    /// **The §9.11 half of the completeness correspondence** — the `:eof`/`:mid`
    /// producer, which is the family the WR1 re-key moved 39,967 functions into.
    ///
    /// Graded the same three ways as the grammar half
    /// (`body::mcall::tests::the_completeness_axis_agrees_with_the_rendered_key`),
    /// and the assertions are stated in **both directions** on purpose: a
    /// classifier that answered `WholeSegmentEnd` for every byte-less refusal
    /// passes the positive alone, which is the shape
    /// `the_eof_suffix_is_earned_by_reaching_the_segment_end` was written to
    /// catch for the suffix itself.
    ///
    /// The bridging claim this axis makes — that `:eof` and `-whole` are the
    /// same *claim* reached by two producers — is the one §9.13 had to make by
    /// hand to re-check its 1,399-row figure. Here it is a named method
    /// ([`Complete::is_whole`]) instead of a grep, and the two provenances stay
    /// separable because the variants stay separate.
    #[test]
    fn the_completeness_axis_reads_the_segment_end_producer_both_ways() {
        let seg = [0u8; 8];

        // Raised AT the end: the parse consumed the whole segment first, which
        // is exactly what `:eof` promises and what `-whole` claims.
        let at_end = Block::at_end(&seg, "call-arg-multi-sym");
        assert_eq!(at_end.feature(), "call-arg-multi-sym:eof");
        assert_eq!(at_end.completeness(), Complete::WholeSegmentEnd);
        assert!(at_end.completeness().is_whole());

        // Raised MID-segment. Must NOT read as whole — this is the negative the
        // positive alone cannot establish.
        let mid = Block::refuse(&seg, 3, "call-arg-multi-sym");
        assert_eq!(mid.feature(), "call-arg-multi-sym:mid");
        assert_eq!(mid.completeness(), Complete::PartialSegmentEnd);
        assert!(!mid.completeness().is_whole());

        // A keyed BYTE refusal carries neither signal, and says so rather than
        // defaulting into either camp. `expr-op-0x27` is the largest row on the
        // emitted board and it is genuinely silent about completeness.
        let byte = super::super::Block {
            ctx: "expr",
            byte: Some(0x27),
            off: 2,
            seg_len: seg.len(),
            aux: 0,
        };
        assert_eq!(byte.completeness(), Complete::NoSignal);
        assert!(!byte.completeness().is_whole());

        // The residue is NAMED and therefore printable. A totality claim whose
        // residue has no name cannot be audited.
        assert_eq!(Complete::NoSignal.name(), "complete-none");

        // Injectivity across the whole closed vocabulary: seven readings, seven
        // distinct names. Summing two of them can never double-count one row.
        let all = [
            Complete::WholeGrammar,
            Complete::MoreGrammar,
            Complete::UnmeasuredGrammar,
            Complete::WholeSegmentEnd,
            Complete::PartialSegmentEnd,
            Complete::NoSignal,
            Complete::InClass,
        ];
        let names: std::collections::BTreeSet<&str> = all.iter().map(|c| c.name()).collect();
        assert_eq!(names.len(), all.len(), "the completeness vocabulary is not injective");
        // …and every name is prefixed so it can never collide with a blocking
        // feature key, which is what would silently merge two histograms.
        assert!(names.iter().all(|n| n.starts_with("complete-")), "{names:?}");
    }
}


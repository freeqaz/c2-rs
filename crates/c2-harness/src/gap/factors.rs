//! **The Phase 7 factor model** (`docs/ROADMAP.md` §10.19/§10.21, boards #160
//! and #179): the A/B/C/D/E predicates, the joints and known-answer control
//! taken over them, the section vocabulary and its greedy ladder, and the
//! machine-readable `GAP-METRICS` block. Split out of `gap.rs` unchanged; see
//! [`super`] for the module docs.

use std::collections::BTreeSet;

use super::fnbytes::MAX_BLR_STOP_LEVELS;
use super::{GapReport, TuClass, TuResult, PORT_WRITER_SECTIONS, WHOLE_TU_RECOGNIZERS};

/// **The control-flow shapes `c2_core::codegen::Selected` can encode**, and
/// there are exactly two (lane `w-tu4`, board **#720**).
///
/// `Selected` has seven variants — `Plain`, `Tail`, `Float`, `Framed`, `Seq`,
/// `CondPair` — and between them they cover **straight-line** bodies and **one
/// two-arm conditional**. None encodes a multi-way conditional.
///
/// This list is the screen's single assumption and it is the thing to re-check
/// when a variant is added: it is a **hand-maintained mirror of a `c2-core`
/// enum**, and nothing in the type system ties the two together. It is
/// deliberately spelled with the census's own `cflow-…` keys rather than the
/// variant names, because the census is what the screen actually reads.
///
/// # `cflow-loop` is DELIBERATELY ABSENT, and the asymmetry is the point
///
/// Since lane `w-hash` the port **does** emit one body with a backward branch —
/// `codegen::ptr_walk_loop`, the pointer-walk accumulate that converted
/// `src/system/math/Sort.cpp`. `cflow-loop` is still not in this list, and
/// adding it would be the screen's first over-claim: what shipped is a
/// **twenty-word transcription of one function class at `/O1`**, not a loop
/// lowering, and every other loop shape has exactly the representation it had
/// before, which is none.
///
/// So the list is now known to be **conservative in one named direction**: a
/// frontier TU whose only obstacle is *this* loop would read `NeedsClass`
/// wrongly. That costs nothing today — the one such TU is a `match` and a match
/// is not on the frontier — and it is written here rather than discovered,
/// because a screen that quietly widened to `cflow-loop` would report every
/// remaining loop TU as buildable. **Widen this entry only alongside a
/// `Selected` variant that can express loops in general.**
///
/// The `+expr-modeled` spellings are the same two classes with the statement
/// layer fully decoded — the census emits both forms and they are the same CFG.
///
/// # Every entry here is `Whole`, and that is the identity end of board #778
///
/// The list was a flat `&[&str]` until lane `w-subclass`. [`CfgSub::Whole`] is
/// exactly what a bare string meant, so this list is behaviourally the list it
/// replaced — measured, not asserted: `reach` over the 878-TU workload reads
/// **2 of 17** on both sides, the same two TUs by name.
const PORT_CFG_CLASSES: &[CfgClass<'static>] = &[
    CfgClass { class: "cflow-straight", sub: CfgSub::Whole },
    CfgClass { class: "cflow-straight+expr-modeled", sub: CfgSub::Whole },
    CfgClass { class: "cflow-if-1", sub: CfgSub::Whole },
    CfgClass { class: "cflow-if-1+expr-modeled", sub: CfgSub::Whole },
];

/// **One entry of [`PORT_CFG_CLASSES`]: a CFG class, optionally restricted to a
/// named sub-class** (lane `w-subclass`, board **#778**).
///
/// # The problem this exists for
///
/// The list used to be `&[&str]`, matched against the bare census class string.
/// Two lanes in a row — `w-rotate` §7 and `w-sched2` §8 — measured real,
/// honest, **partial** coverage of `cflow-loop` and had no way to record it.
/// The claim `w-sched2` could support was *"`cflow-loop`, restricted to the
/// sentinel walk at `/O1`, pointer formal at slot 0, chains of single-word
/// producers with no hoisted literal"*, and a list of strings can hold only the
/// wholesale claim, which is false. Both lanes correctly declined to widen the
/// list, and the second one filed the refusal as the *whole* remaining blocker.
///
/// # The one property that makes this safe: NARROWER OR EQUAL, NEVER WIDER
///
/// [`CfgClass::admits`] is `self.class == class && <sub>`. `Whole` contributes
/// `true`, so an unrestricted entry is precisely the old string comparison;
/// `Keys` contributes a **conjunct**. A restriction can therefore only ever
/// remove `(class, key)` pairs from the admitted set, never add one —
/// `admits(Keys(_)) ⟹ admits(Whole)` for the same class, for every input,
/// algebraically. The algebra is not the evidence: `GapReport::cfg_reach_bounds`
/// re-derives the nesting on the real workload every scan, and the scan prints
/// it.
///
/// # Matching is EXACT, and that is a load-bearing choice
///
/// `Keys` is an **enumeration** compared with `==`, never a prefix and never a
/// substring. A prefix match is the wrongly-permissive failure mode this design
/// is against: census keys nest densely (`expr-op-0x27` is a prefix of nothing
/// today, but `expr-cmp-eq` sits beside `expr-cmp-eq-and-branch-more` and 240
/// distinct `cflow-loop|…` keys share long stems), so a prefix restriction would
/// silently grow every time the census mints a neighbouring key and a lane would
/// report coverage it never measured. An enumeration cannot. The exactness is
/// graded by `a_prefix_of_a_listed_key_is_not_admitted`, which is the unit test
/// that fails under must-fail mutation **M2**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CfgClass<'a> {
    /// The census's `cflow-…` class string, matched exactly.
    pub class: &'a str,
    /// Whole class, or the named sub-class.
    pub sub: CfgSub<'a>,
}

/// How much of a [`CfgClass`] the port claims — see that type for why the two
/// cases cannot be collapsed into a `&[&str]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfgSub<'a> {
    /// **The whole class**, which is what a bare `&str` meant. Every shipped
    /// entry is this today.
    Whole,
    /// **Only these census keys**, by exact equality. The empty slice admits
    /// nothing and is a legal (and used) value: it is the `⊥` bound
    /// `cfg_reach_bounds` takes.
    Keys(&'a [&'a str]),
}

impl<'a> CfgClass<'a> {
    /// Does this entry admit a blocked function whose cross-tab row is
    /// `"<class>|<key>"`?
    ///
    /// **The class test is a conjunct of both arms**, which is what makes a
    /// restriction narrowing rather than merely different.
    pub fn admits(&self, class: &str, key: &str) -> bool {
        self.class == class
            && match self.sub {
                CfgSub::Whole => true,
                // `contains` on `&[&str]` is element-wise `==`. NOT
                // `starts_with`, NOT `contains(key)` on the string — see the
                // type docs, and `M2` in the rung.
                CfgSub::Keys(ks) => ks.contains(&key),
            }
    }

    /// Does this entry name `class` **at all**, restricted or not?
    ///
    /// Separate from [`Self::admits`] because the two answer different
    /// questions, and the difference is the whole content of a partial claim:
    /// `covers_class` is *"the port has something in this class"* and `admits`
    /// is *"the port has THIS body"*. `cfg_reach` uses both — the first decides
    /// how to NAME a miss, the second decides whether it is a miss.
    pub fn covers_class(&self, class: &str) -> bool {
        self.class == class
    }

    /// `true` when this entry is a partial claim.
    pub fn is_restricted(&self) -> bool {
        matches!(self.sub, CfgSub::Keys(_))
    }

    /// The keys a restricted entry lists, or `None` for a whole class. Used by
    /// the ledger to report a listed key **no scan ever witnessed**, which would
    /// otherwise be a claim that quietly does nothing.
    pub fn keys(&self) -> Option<&'a [&'a str]> {
        match self.sub {
            CfgSub::Whole => None,
            CfgSub::Keys(ks) => Some(ks),
        }
    }
}

/// **ONE FRONTIER TU's CODEGEN COLUMN, read off the judge's own predicate**
/// (lane `w-column`, board **#1474**) — see [`GapReport::frontier_codegen`].
///
/// Board **#1463** priced the sixteen frontier TUs on two reader ladders and
/// published `NO COLUMN` in the codegen cell of every row, and **#1464** gave
/// the reason: `ladder.py` reads `fn_blockers` (a reader column),
/// `emit_blockers` (the *same* reader column at a second population) and
/// `fn_gate_refusals` (an invariant defined to be zero). Every codegen price on
/// the board is therefore a **hand-count** — #1105's `>= 15`, #1418's 776 bytes,
/// `w-conv`'s tallies.
///
/// This is the instrument reading that replaces the hand-count, and it is
/// deliberately **smaller** than one, because the honest column has a hole in it
/// and the hole is named rather than filled:
///
/// | field | what it is | is it a codegen price? |
/// |---|---|---|
/// | [`exact`](Self::exact) | c2's bytes, produced | **done** — negative distance |
/// | [`wrong`](Self::wrong) | the reader accepted, the emitter LOWERED, and the bytes or relocations DIFFER | **YES — the only positive codegen price this project can measure per function** |
/// | [`cg_refused`](Self::cg_refused) | the reader accepted and the emitter DECLINED (`fnbyte-refused-codegen` + `fnbyte-partial`) | yes, but see [`super::fnbytes::Decline`] — three of four stages are zero by construction |
/// | [`reader`](Self::reader) | the IL parser refused; **no codegen question was asked and none can be** | **NO. This is the hole.** |
/// | [`ungraded`](Self::ungraded) | unbound / no bytes / relocations unreadable | no — instrument limits, printed so they cannot hide |
///
/// **The load-bearing row is `reader`.** A frontier TU's remaining codegen
/// distance is not `wrong + cg_refused`; it is `wrong + cg_refused` **plus an
/// unknown amount hiding behind `reader`**, and that unknown is unmeasurable
/// today by construction — there is no `IlFunction` to hand `select_function`,
/// so the question cannot be put. Any lane quoting this struct's positive
/// numbers as *the* codegen price of a frontier TU is quoting a **lower bound of
/// unknown tightness**, which is the shape `cflow-emitted-modeled`'s "718" had
/// for eight days (boards #1343/#1344).
///
/// The five fields partition [`denominator`](Self::denominator), and that is a
/// printed control ([`Self::partition_broken`]) rather than an assertion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrontierCodegen {
    /// Every `.text` COMDAT leader real c2 emitted for this TU — `fnbyte`'s own
    /// denominator, a function of **c2's** output alone.
    pub denominator: usize,
    /// Byte-exact **and** relocation-exact against c2's COMDAT.
    pub exact: usize,
    /// `fnbyte-differs` + `fnbyte-reloc-differs`. Lowered, and wrong.
    pub wrong: usize,
    /// `fnbyte-refused-codegen` + `fnbyte-partial`: the reader accepted and the
    /// emitter had nothing to emit.
    pub cg_refused: usize,
    /// `fnbyte-refused-parse`: the reader refused. **The unmeasurable half.**
    pub reader: usize,
    /// `fnbyte-unbound` + `fnbyte-nobytes` + `fnbyte-reloc-unknown` — the
    /// instrument could not put the question, for a reason that is neither the
    /// reader's nor the emitter's.
    pub ungraded: usize,
}

impl FrontierCodegen {
    /// **The measurable codegen price**: lowered-and-wrong, plus
    /// lowered-and-declined. A **lower bound** — see the struct docs.
    pub fn measured(self) -> usize {
        self.wrong + self.cg_refused
    }

    /// **The partition control, target 0.** Printed on every scan beside the row
    /// it grades. Non-zero means a bucket stopped being written and the row
    /// above it is short by an unknown amount.
    pub fn partition_broken(self) -> bool {
        self.exact + self.wrong + self.cg_refused + self.reader + self.ungraded
            != self.denominator
    }
}

/// One frontier TU's answer to *"can the port's emitter express this TU's
/// blocked functions at all?"* — see [`GapReport::frontier_cfg_reachability`].
///
/// Deliberately **not** a `bool`: the third state is the one that matters. A TU
/// whose census bailed before assigning a CFG class is neither reachable nor
/// blocked-by-a-named-class, and collapsing it into `false` would hide that the
/// screen does not actually know, while collapsing it into `true` would be
/// absence read as success.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CfgReach {
    /// Every blocked function's CFG class is in [`PORT_CFG_CLASSES`].
    Reachable,
    /// At least one blocked function needs a CFG class the port has no
    /// `Selected` variant for. Carries the class names, sorted.
    NeedsClass(BTreeSet<String>),
    /// `n` blocked functions have no CFG class at all in `fn_cflow` — the census
    /// bailed first. **Not reachable**, and not a named blocker either.
    Unclassified(usize),
}

impl CfgReach {
    /// `true` only for [`CfgReach::Reachable`]. The one reading acted on.
    pub fn is_reachable(&self) -> bool {
        matches!(self, CfgReach::Reachable)
    }

    /// **Is this verdict blocked on `class`** — wholly or in part (board
    /// **#778**)?
    ///
    /// [`CfgReach::NeedsClass`] holds the bare class string when nothing in
    /// [`PORT_CFG_CLASSES`] names the class, and `"<class>!<key>"` when a
    /// *restricted* entry names it but does not admit this body. A caller that
    /// tested set membership of the bare string directly would silently stop
    /// counting a TU the day a lane restricted that class — the count would
    /// fall and nothing would say why. Every reader goes through here.
    pub fn needs_class(&self, class: &str) -> bool {
        match self {
            CfgReach::NeedsClass(v) => v
                .iter()
                .any(|s| s == class || s.split_once('!').is_some_and(|(c, _)| c == class)),
            _ => false,
        }
    }

    /// A one-line rendering for the scan block.
    pub fn label(&self) -> String {
        match self {
            CfgReach::Reachable => "REACHABLE — every blocked fn is a port CFG class".into(),
            CfgReach::NeedsClass(v) => format!(
                "needs a CFG class the port lacks: {}",
                v.iter().cloned().collect::<Vec<_>>().join(", ")
            ),
            CfgReach::Unclassified(n) => {
                format!("{n} blocked fn with NO CFG class (census bailed first)")
            }
        }
    }
}

/// [`GapReport::cfg_reach_bounds`]'s four reachable sets and the nesting taken
/// over them. Every field is a set of TU source paths, sorted.
#[derive(Clone, Debug)]
pub struct CfgBounds<'a> {
    /// The frontier this was taken over — the denominator every count needs.
    pub frontier: usize,
    /// `⊥` — every shipped entry restricted to no keys. **Must be empty.**
    pub bottom: Vec<&'a str>,
    /// Every shipped entry rewritten as the enumeration of its observed keys.
    /// **Must equal `shipped`.**
    pub enumerated: Vec<&'a str>,
    /// The shipped list — today's answer, and the only one anyone acts on.
    pub shipped: Vec<&'a str>,
    /// `⊤` — every class the frontier mentions, wholesale. A hypothetical.
    pub top: Vec<&'a str>,
    /// The classes `⊤` was built from, so its width is readable.
    pub top_classes: Vec<&'a str>,
    /// How many `(class, key)` pairs `enumerated` listed — the size of the
    /// exercise the `Keys` path actually got on this scan.
    pub enumerated_keys: usize,
}

impl CfgBounds<'_> {
    /// **`⊥ ⊆ shipped ⊆ ⊤`, and `enumerated == shipped`**, checked as sets.
    ///
    /// Returns the violations, each a human-readable line. Empty is the pass,
    /// and callers print the count rather than the status — comparing a count
    /// and never a status is trap 5's standing mitigation.
    pub fn violations(&self) -> Vec<String> {
        let mut v = Vec::new();
        let subset = |a: &[&str], b: &[&str]| -> Vec<String> {
            a.iter().filter(|x| !b.contains(x)).map(|x| x.to_string()).collect()
        };
        if !self.bottom.is_empty() {
            v.push(format!(
                "BOTTOM is not empty ({} TUs: {}) — a list admitting no census key \
                 reached something, so the matcher is ignoring its key argument",
                self.bottom.len(),
                self.bottom.join(", ")
            ));
        }
        let esc = subset(&self.shipped, &self.top);
        if !esc.is_empty() {
            v.push(format!(
                "SHIPPED is not a subset of TOP ({}) — a restriction widened the \
                 admitted set, which is the one direction #778 forbids",
                esc.join(", ")
            ));
        }
        if self.enumerated != self.shipped {
            let a = subset(&self.enumerated, &self.shipped);
            let b = subset(&self.shipped, &self.enumerated);
            v.push(format!(
                "ENUMERATED != SHIPPED (only-in-enumerated: [{}], only-in-shipped: [{}]) \
                 — Keys and Whole disagree where they are built to agree",
                a.join(", "),
                b.join(", ")
            ));
        }
        v
    }
}

/// One row of [`GapReport::cfg_subclass_ledger`].
#[derive(Clone, Debug)]
pub struct CfgLedgerRow {
    /// The entry's CFG class.
    pub class: &'static str,
    /// How many census keys the entry lists, or `None` for a whole class.
    pub listed: Option<usize>,
    /// How many distinct census keys this scan observed for the class.
    pub observed_keys: usize,
    /// How many of those the entry admits.
    pub admitted_keys: usize,
    /// Listed keys this scan never saw — a claim doing nothing.
    pub unwitnessed: Vec<String>,
    /// Keys `admits` accepted that the entry does not list. **Must be empty**;
    /// `None` for a whole class, where there is no declaration to compare with.
    pub intruders: Option<Vec<String>>,
}

impl GapReport {
    /// **The five Phase 7 factors for one TU** (`docs/ROADMAP.md` §10.19 and
    /// §10.21, boards #160 and #179), in `[A, B, C, D, E]` order:
    ///
    /// | | predicate | key |
    /// |---|---|---|
    /// | **A** | `.ex` segments == obj `.text` COMDATs, on the anchor the port consumes | `emit-set-ceiling-gate` |
    /// | **B** | every emitted symbol binds | `emit-set-ceiling-today` |
    /// | **C** | obj section set ⊆ [`PORT_WRITER_SECTIONS`] | `emit-sec-reachable` |
    /// | **D** | every emitted COMDAT is in the port's **per-function** codegen class | `emit-class-complete` |
    /// | **E** | a registered **whole-TU** recognizer accepts this bundle | `emit-whole-tu-any` |
    ///
    /// Every one reads a key some *other* code path wrote, so this function
    /// re-derives no rule — it is a join, and that is the whole point (§10.14).
    ///
    /// # What the factorization is a factorization OF, and where D went wrong
    ///
    /// §10.19's four predicates are four questions the port must answer yes to
    /// before its output can be the reference's bytes: **A** do the port and the
    /// reference agree on *what set of things is emitted*; **B** can the port
    /// *name* everything in that set; **C** can the writer *write the containers*
    /// the obj needs; **D** does the port have an *accepted route to the
    /// contents*.
    ///
    /// A/B/C are properties of the obj and the binding. **D is the odd one out**:
    /// it is not a property of the obj at all but of the port's acceptance
    /// machinery — `emit-class-complete` is the *per-function* census's verdict,
    /// i.e. "`PortC2`'s per-function path takes every COMDAT here". §10.19 was
    /// measured when `PortC2::build` had exactly **one** acceptance path, so
    /// "the port has a route to the contents" and "the per-function path accepts
    /// every COMDAT" were the same sentence. They are not any more:
    /// `PortC2::build` tries `IlBundle::dyninit_tu()` *before* `functions()`.
    ///
    /// So D was never the general form of question 4 — it was the only reading of
    /// it that existed. **E is the whole-TU reading**, and the general form is the
    /// disjunction [`Self::emit_path`]:
    ///
    /// > A byte-exact obj requires **A ∧ B ∧ C ∧ (D ∨ E)**.
    ///
    /// Measured (2026-08-04, 871 graded TUs): the conjunction `A∧B∧C∧D` is 6
    /// while the differential grades 8, so **D alone is not necessary** and the
    /// old known-answer control was right to print `D 2`. E alone is not
    /// necessary either — it is false on all six per-function matches. The
    /// *disjunction* is what is claimed necessary, and it is what the control is
    /// taken over.
    ///
    /// # This is a disjunct on D, not a widening of D
    ///
    /// D's definition is byte-for-byte what it was: `emit-emitted ==
    /// emit-in-class`, from the per-function census. Nothing in `c2-il`'s
    /// `census.rs` is touched, so the scan's `census/gate disagreement: 0` line
    /// still tracks the symmetry w-r1c declined to break — teaching the
    /// per-function census a whole-TU fact is what a widening would have meant,
    /// and it is not what happened. D's own violation count is still printed, as
    /// a number, so §10.19's refutation stays a visible finding rather than an
    /// absorbed one.
    ///
    /// E is also deliberately **not** "the port emitted it" and **not** the class
    /// field: either would be circular and would make the model unfalsifiable. E
    /// is a class-membership predicate of the same *kind* as D — evaluated
    /// without running the emitter — just at whole-TU granularity. The accepted
    /// consequence is that, exactly like D, **E is not sufficient**:
    /// `PortC2::build_dyninit` carries the `/GF` fence, which lives in `c2-core`
    /// and not in the recognizer, so an E-true TU can still fail to emit. That
    /// would show as an over-prediction in the printed set-identity line, which
    /// is where it belongs.
    ///
    /// **A is gate-anchored** (`4F 1F`, what `PortC2::build` consumes) rather
    /// than `LO`-anchored: §10.18 settled that the two splitters disagree on 634
    /// of 871 TUs and that the port's anchor is the one its emitter has to
    /// satisfy. [`Self::factor_a_lo`] is the other reading, published beside it.
    pub fn factors(r: &TuResult) -> [bool; 5] {
        let has = |k: &str| r.emit.contains_key(k);
        [
            has("emit-set-ceiling-gate"),
            has("emit-set-ceiling-today"),
            has("emit-sec-reachable"),
            has("emit-class-complete"),
            has("emit-whole-tu-any"),
        ]
    }

    /// **The five factors of one TU as a fixed-width letter string**, e.g.
    /// `"AB-D-"` — the letter when the predicate holds, `-` when it does not.
    ///
    /// Fixed width and fixed order (`A B C D E`), so a consumer can index a
    /// column rather than parse a set, and so two runs' rows are byte-comparable
    /// even when a factor flips. See [`Self::factor_membership`] for why the
    /// per-TU form exists at all.
    pub fn factor_letters(r: &TuResult) -> String {
        let f = Self::factors(r);
        ['A', 'B', 'C', 'D', 'E']
            .iter()
            .zip(f.iter())
            .map(|(ch, on)| if *on { *ch } else { '-' })
            .collect()
    }

    /// **The per-TU factor membership, by name** — `(src, class, letters)` for
    /// every graded TU, in scan order (sorted by `src`).
    ///
    /// # Why a per-TU list and not another count
    ///
    /// Every joint the factorization publishes — `B∧C`, `A∧B∧C`, the FRONTIER —
    /// is a **count**, and a count cannot be intersected with anything. Lane
    /// `w-emitp` measured a per-TU emit-set model over a 850-TU corpus and could
    /// not price it in TU reach, because the number that does that is
    /// `|{TU : the model is exact} ∩ B∧C|` and *this report had no per-TU `B∧C`
    /// list to intersect against*. It declined to multiply `151 × 0.555`
    /// instead, which was right: multiplying a per-TU rate by a joint count is
    /// exactly the move that left `B∧C` published at **107** — a figure taken at
    /// `C = 114` and never re-measured when the writer's section vocabulary grew
    /// `C` to 169. The true answer was 151 and any number in `[107, 169]` would
    /// have looked consistent.
    ///
    /// So the membership is published rather than the joints alone. It is
    /// **written to a file** (`--factors-tsv`) rather than to stdout: `c2rs gap`
    /// also grades the generated case corpus through `scripts/mode_lane.sh` and
    /// `scripts/mode_cross.sh`, where "one line per graded TU" is tens of
    /// thousands of lines per lane. The counts stay on stdout; the membership
    /// they are counts *of* goes to the file a lane asks for.
    ///
    /// **Every joint in the `GAP-METRICS` block is re-derivable from these rows**
    /// — that is the property the unit test grades, and it is what makes the file
    /// a publication of the same measurement rather than a second one.
    pub fn factor_membership(&self) -> Vec<(&str, &'static str, String)> {
        self.graded()
            .map(|r| (r.src.as_str(), r.class.label(), Self::factor_letters(r)))
            .collect()
    }

    /// Render [`Self::factor_membership`] as the `--factors-tsv` file body.
    ///
    /// Pure (returns the text rather than writing it) so the unit test grades
    /// the bytes with no filesystem and no toolchain. The header names the
    /// columns, the population and — the part that matters — **what is NOT a
    /// row**: `capture-fail` TUs were never measured, so they are absent rather
    /// than false, and a consumer that read absence as `0 0 0 0 0` would make
    /// every factor look tighter than it is (`docs/STATUS.md` trap 5).
    pub fn factor_tsv(&self) -> String {
        let mut s = String::from(
            "# c2rs gap --factors-tsv — per-TU Phase 7 factor membership \
             (docs/ROADMAP.md §10.19/§10.21, boards #160/#179)\n\
             # columns: src<TAB>class<TAB>A<TAB>B<TAB>C<TAB>D<TAB>E<TAB>letters\n\
             #   A `.ex` segments == obj `.text` COMDATs (gate-anchored `4F 1F`)\n\
             #   B every emitted symbol binds\n\
             #   C obj section set subset of the port writer's section names\n\
             #   D every emitted COMDAT in the port's per-function codegen class\n\
             #   E a REGISTERED whole-TU recognizer accepts this bundle\n\
             # A byte-exact obj requires A and B and C and (D or E).\n\
             # ROWS ARE THE GRADED TUs ONLY. A `capture-fail` TU has no obj and no\n\
             # census, so it is NOT a row here — it was never measured, which is a\n\
             # different fact from every factor being false. Do not read its absence\n\
             # as a zero row.\n",
        );
        s.push_str(&format!("# graded-rows {}\n", self.graded().count()));
        for (src, class, letters) in self.factor_membership() {
            let f: Vec<&str> = letters
                .chars()
                .map(|c| if c == '-' { "0" } else { "1" })
                .collect();
            s.push_str(&format!(
                "{src}\t{class}\t{}\t{}\t{}\t{}\t{}\t{letters}\n",
                f[0], f[1], f[2], f[3], f[4]
            ));
        }
        s
    }

    /// **Question 4 in its general form: `D ∨ E`** — the port has an accepted
    /// route to this TU's contents, by *some* acceptance path.
    ///
    /// The term the model claims is necessary. Neither disjunct is necessary
    /// alone and both are measured not to be, which is the entire content of
    /// board #179: see [`Self::factors`].
    pub fn emit_path(f: &[bool; 5]) -> bool {
        f[3] || f[4]
    }

    /// Factor A on the **`LO`** anchor (`4C 4F 11`, the census's splitter) —
    /// the reading `emit_set_reachable_tus` filters on. Published beside the
    /// gate-anchored one because §10.18's whole finding is that they are two
    /// different numbers and only one is the port's.
    pub fn factor_a_lo(r: &TuResult) -> bool {
        r.fn_total == r.emit.get("emit-emitted").copied().unwrap_or(0)
    }

    /// The TUs the factorization is computed over: everything the harness
    /// graded, i.e. every TU that captured. `capture-fail` TUs have no obj and
    /// no census, so they are not "outside the factors" — they were never
    /// measured, and folding them in would make every factor look tighter.
    pub fn graded(&self) -> impl Iterator<Item = &TuResult> {
        self.results.iter().filter(|r| r.class != TuClass::CaptureFail)
    }

    /// `(|A|, |B|, |C|, |D|, |E|, |A_lo|, |B∧C|, |A∧B∧C|, |A∧B∧C∧D|,
    /// |A∧B∧C∧(D∨E)|)` over the graded TUs.
    ///
    /// `B∧C` is the plan's **near-term joint ceiling** — what a perfect emit-set
    /// model plus a perfect binding reaches while the writer's vocabulary is
    /// what it is (`PHASE7_PLAN.md` §1). It is a *joint*, measured per TU, and
    /// not a product of marginals: §8.6's standing rule, and the reason this
    /// function exists rather than a note telling readers to multiply.
    ///
    /// **`A∧B∧C∧D` is kept and reported** even though the model's joint is now
    /// `A∧B∧C∧(D∨E)`. §10.19's original conjunction is the thing board #179
    /// refutes; a refutation whose refuted quantity stops being measured is a
    /// claim nobody can re-check.
    pub fn factor_counts(&self) -> [usize; 10] {
        let mut c = [0usize; 10];
        for r in self.graded() {
            let f = Self::factors(r);
            for i in 0..5 {
                c[i] += usize::from(f[i]);
            }
            c[5] += usize::from(Self::factor_a_lo(r));
            c[6] += usize::from(f[1] && f[2]);
            let abc = f[0] && f[1] && f[2];
            c[7] += usize::from(abc);
            c[8] += usize::from(abc && f[3]);
            c[9] += usize::from(abc && Self::emit_path(&f));
        }
        c
    }

    /// **The model's joint, `A∧B∧C∧(D∨E)`**, by source path. The claim is that
    /// this set **is** the match set, so it is returned as a list of names rather
    /// than a count: a count could agree by coincidence, and two sets that differ
    /// by a swap would read as equal.
    pub fn factor_all_tus(&self) -> Vec<&str> {
        self.graded()
            .filter(|r| {
                let f = Self::factors(r);
                f[0] && f[1] && f[2] && Self::emit_path(&f)
            })
            .map(|r| r.src.as_str())
            .collect()
    }

    /// §10.19's **original** conjunction `A∧B∧C∧D`, by source path — the set
    /// board #179 refutes. Kept beside [`Self::factor_all_tus`] so the
    /// refutation stays checkable rather than becoming folklore: the difference
    /// between the two lists is exactly the TUs the fifth term accounts for.
    pub fn factor_abcd_tus(&self) -> Vec<&str> {
        self.graded()
            .filter(|r| {
                let f = Self::factors(r);
                f[0] && f[1] && f[2] && f[3]
            })
            .map(|r| r.src.as_str())
            .collect()
    }

    /// Per-recognizer marginals for [`WHOLE_TU_RECOGNIZERS`]: `(name, TUs it
    /// accepts)`, in registry order.
    ///
    /// Printed per entry rather than only as the union, because a registry entry
    /// that never fires and one that was never added are the same number in
    /// `|E|` and very different facts about the model.
    pub fn whole_tu_marginals(&self) -> Vec<(&'static str, usize)> {
        WHOLE_TU_RECOGNIZERS
            .iter()
            .map(|(name, _)| (*name, self.emit_total(&format!("emit-whole-tu|{name}"))))
            .collect()
    }

    /// **The known-answer control on the factorization**: how many byte-exact
    /// TUs fail each term, and how many `match` TUs there were to check.
    /// Returns `([A, B, C, D, E, D∨E] violations, matching TUs)`.
    ///
    /// # Which of these must be zero, and why that is not a relaxation
    ///
    /// **A, B, C and `D∨E` must be 0.** Those are the model's *necessary*
    /// conditions, which is the only thing that makes them a ceiling; nonzero
    /// anywhere means the term is not necessary and any bound drawn from it is
    /// void.
    ///
    /// **D and E individually must not be**, and it would be wrong to require it.
    /// Both are measured non-necessary on the 878-TU workload: D fails on the two
    /// `??__E` TUs (whole-TU emit path), E fails on all six per-function matches.
    /// They are the two readings of one question (see [`Self::factors`]), so
    /// their columns are **diagnostics**, printed with the label that says so.
    ///
    /// The distinction matters because moving a column from "must be 0" to
    /// "diagnostic" is exactly the move that a fitted control would make to go
    /// green. What makes it legitimate here is that the *replacement* column is
    /// strictly narrower than "anything the port emits": `D∨E` is D plus a
    /// **closed, named registry** ([`WHOLE_TU_RECOGNIZERS`]) of one entry, so an
    /// emit path nobody registered still turns it red. `E := decodes()` would
    /// have been the relaxation; this is not it.
    ///
    /// For **C** this is also the control on [`PORT_WRITER_SECTIONS`] itself: a
    /// matching obj is the port's own output, so a name missing from that list
    /// shows up here rather than in an argument about whether the list is
    /// complete.
    pub fn factor_control_on_match_tus(&self) -> ([usize; 6], usize) {
        let mut bad = [0usize; 6];
        let mut n = 0;
        for r in self.results.iter().filter(|r| r.class == TuClass::Match) {
            n += 1;
            let f = Self::factors(r);
            for (i, ok) in f.iter().enumerate() {
                bad[i] += usize::from(!ok);
            }
            bad[5] += usize::from(!Self::emit_path(&f));
        }
        (bad, n)
    }

    /// **The frontier**: TUs inside `A∧B∧C` that are not yet a `match` and that
    /// **no acceptance path the port has covers** — the emit set is reachable,
    /// every emitted symbol binds, the obj's sections are all writable, neither
    /// the per-function class (D) nor any registered whole-TU recognizer (E)
    /// takes the TU, and widening the accepted *function* class is the whole
    /// remaining distance.
    ///
    /// **Board #179 narrowed this from `¬D` to `¬(D∨E)`.** A TU some whole-TU
    /// recognizer already accepts but that is not a match is *not* on the
    /// codegen-breadth frontier: its blocker is that whole-TU emitter's own fence
    /// (for `dyninit`, the `/GF` fence in `c2-core`), which is different work
    /// from widening the function class. Leaving it in would have advertised
    /// per-function codegen as the route to a TU per-function codegen cannot
    /// reach.
    ///
    /// This is the one actionable list the factorization produces. Everything
    /// else it prints is a bound; these are TUs where no model, no section work
    /// and no binding repair is needed. Sorted by distance (emitted functions
    /// not in class), nearest first.
    ///
    /// **It is not a schedule** (`ROADMAP.md` §9.16.1): a TU one blocked
    /// function away can be one blocked function away from a construct nobody
    /// has modelled.
    pub fn factor_frontier(&self) -> Vec<(&TuResult, usize)> {
        let mut v: Vec<(&TuResult, usize)> = self
            .graded()
            .filter(|r| r.class != TuClass::Match)
            .filter(|r| {
                let f = Self::factors(r);
                f[0] && f[1] && f[2] && !Self::emit_path(&f)
            })
            .map(|r| {
                let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
                let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
                (r, e.saturating_sub(i))
            })
            .collect();
        v.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.src.cmp(&b.0.src)));
        v
    }

    /// **The counterfactual frontier**: what [`Self::factor_frontier`] would
    /// count if factor **A** were true on every graded TU — i.e. if a perfect
    /// emit-set model existed. Same clauses as the frontier with `f[0]` dropped.
    ///
    /// Board **#213** quotes this beside `B∧C` and both halves of that row's
    /// arithmetic went stale together: it was published as `17 → 99` when
    /// `A∧B∧C` was 25 and `B∧C` was 107. It is computed here rather than
    /// subtracted by hand because *that hand-subtraction is exactly the defect
    /// this function exists to prevent* — `99 − 17 == 107 − 25` only while
    /// every `match`-or-`D∨E` TU inside `B∧C` also satisfies A, which is a
    /// contingent fact about the corpus and not an identity.
    pub fn factor_frontier_if_a(&self) -> usize {
        self.graded()
            .filter(|r| r.class != TuClass::Match)
            .filter(|r| {
                let f = Self::factors(r);
                f[1] && f[2] && !Self::emit_path(&f)
            })
            .count()
    }

    /// **The TUs on which board #213's two arithmetics disagree**, by name.
    ///
    /// #213 states the value of a perfect emit predicate twice — as
    /// `B∧C − A∧B∧C` and as `frontier-if-A − FRONTIER` — and published one
    /// number for both (`+82`), because when it was written the two coincided.
    /// They coincide exactly when **no** TU inside `B∧C` fails A while already
    /// having an accepted route (D or E), and that is a contingent property of
    /// the corpus. These are the TUs in the difference: reachable if the emit
    /// set were modelled, but *not* additions to the codegen frontier, because
    /// the port already accepts their contents.
    ///
    /// Returned by name rather than as a count, for the reason
    /// [`Self::factor_all_tus`] gives: a count can agree by coincidence.
    pub fn factor_projection_divergence(&self) -> Vec<&str> {
        self.graded()
            .filter(|r| r.class != TuClass::Match)
            .filter(|r| {
                let f = Self::factors(r);
                !f[0] && f[1] && f[2] && Self::emit_path(&f)
            })
            .map(|r| r.src.as_str())
            .collect()
    }

    /// **The FRONTIER ranked by `.text` byte fraction** (lane `w-tu3`, board
    /// **#500**) — the third unit this project has tried for "which frontier TU
    /// is nearest", and the first with a conversion outcome behind it.
    ///
    /// * **#269** counted independent *refusals* and could not see what was
    ///   already emitted.
    /// * **#465** counted already-emitted *functions* and could not see how much
    ///   of the TU they were. It was **refuted by the very TU pre-registered to
    ///   confirm it** — `mmio` scores 72.7 % by function and 16.8 % by byte.
    /// * **This** counts `.text` **bytes** the port already produces a body for.
    ///   `xboxmem`, the one TU ever converted by codegen breadth, scored 50 % by
    ///   function and **54.5 %** by byte.
    ///
    /// `None` in the second slot means **no denominator** — a TU with no `.text`
    /// COMDAT bytes. It is ranked last and printed, never scored 100 %; see
    /// [`super::fnbytes::byte_fraction`] for why that specific zero matters.
    ///
    /// Descending by fraction, ties broken by source path so the order is
    /// stable across runs. Pure over `results`.
    pub fn frontier_byte_ranking(&self) -> Vec<(&TuResult, Option<(usize, usize)>)> {
        let mut rows: Vec<(&TuResult, Option<(usize, usize)>)> = self
            .factor_frontier()
            .into_iter()
            .map(|(r, _)| (r, super::fnbytes::byte_fraction(r)))
            .collect();
        rows.sort_by(|x, y| {
            // Integer key: no float comparator in a sort that must be stable
            // across machines. `1_000_000` is six significant figures, which is
            // four more than anything printed.
            let key = |v: &Option<(usize, usize)>| {
                v.map(|(n, d)| (1u8, (n as u128) * 1_000_000 / d as u128))
                    .unwrap_or((0, 0))
            };
            key(&y.1).cmp(&key(&x.1)).then_with(|| x.0.src.cmp(&y.0.src))
        });
        rows
    }

    /// **THE CODEGEN COLUMN ON THE FRONTIER** (lane `w-column`, board **#1474**)
    /// — one [`FrontierCodegen`] per frontier TU, sorted by source path.
    ///
    /// Read [`FrontierCodegen`]'s own docs first: the useful half of this table
    /// is the [`reader`](FrontierCodegen::reader) column, which says how much of
    /// each TU **cannot be priced at all** today.
    ///
    /// Computed for ONE TU by [`Self::codegen_column`], which is public and
    /// separate for the reason [`Self::cfg_reach`] is: a screen that can only be
    /// evaluated on the live frontier cannot be tested.
    pub fn frontier_codegen(&self) -> Vec<(&TuResult, FrontierCodegen)> {
        let mut v: Vec<(&TuResult, FrontierCodegen)> = self
            .factor_frontier()
            .into_iter()
            .map(|(r, _)| (r, Self::codegen_column(r)))
            .collect();
        v.sort_by(|a, b| a.0.src.cmp(&b.0.src));
        v
    }

    /// [`Self::frontier_codegen`]'s reading for one TU.
    ///
    /// **Every field comes from a key `super::fnbytes` writes in the same loop
    /// iteration that files the bucket** — nothing here subtracts two published
    /// totals to recover a third, which is precisely how `emit_blockers` came to
    /// be read as a codegen column it never was (board #1464).
    pub fn codegen_column(r: &TuResult) -> FrontierCodegen {
        let g = |k: &str| r.emit.get(k).copied().unwrap_or(0);
        FrontierCodegen {
            denominator: g("fnbyte-denominator"),
            exact: g("fnbyte-exact"),
            wrong: g("fnbyte-differs") + g("fnbyte-reloc-differs"),
            cg_refused: g("fnbyte-refused-codegen") + g("fnbyte-partial"),
            reader: g("fnbyte-refused-parse"),
            ungraded: g("fnbyte-unbound") + g("fnbyte-nobytes") + g("fnbyte-reloc-unknown"),
        }
    }

    /// **THE CFG-REACHABILITY SCREEN** (lane `w-tu4`, board **#720**) — the
    /// question every previous frontier ranking was structurally unable to ask:
    /// *is this TU buildable at all, or does it need a control-flow class the
    /// port does not have?*
    ///
    /// # Why the byte-fraction ranker cannot ask it
    ///
    /// #269 counted independent refusals, #465 counted already-emitted
    /// *functions*, #500 counted already-emitted **bytes**. All three are
    /// *quantities of progress* over the emitted population, and all three are
    /// computed from `codegen::select_function`, which answers per function and
    /// returns a `Selected` or a refusal. **None of them can distinguish "this
    /// function is refused because one expression token is unmodelled" from
    /// "this function is refused because it is a LOOP, and no variant of
    /// `Selected` encodes a backward branch".** The first is a rung; the second
    /// is a new CFG class, and the difference is not a quantity.
    ///
    /// # What it is
    ///
    /// It reads [`TuResult::fn_cflow`], takes the CFG class of every **blocked**
    /// function, and asks whether all of them are in [`PORT_CFG_CLASSES`]. A TU
    /// with even one blocked `cflow-loop`, `cflow-if-n` or `cflow-if-2` function
    /// cannot convert however small its remaining byte count, because the
    /// emitter has nowhere to put the body.
    ///
    /// # It is an INSTRUMENT and never a gate
    ///
    /// Pure over `results`, reads no obj, licenses no emit, appears in no
    /// accept/refuse path, moves no numerator. It does not rank — it
    /// **partitions** — and a negative verdict is a statement about the *port*,
    /// not about the TU.
    ///
    /// # The unclassified case is a `false`, never a `true`
    ///
    /// A body the census bailed on before assigning a CFG class (`cf-expr-…`)
    /// contributes to `fn_blockers` but to no `class|key` row in `fn_cflow`.
    /// Counting only the classified rows would let such a TU read "reachable" on
    /// the strength of the functions it *could* classify — absence read as
    /// success, this project's most-repeated defect. So the classified count is
    /// compared against `fn_blockers`' total and any shortfall is
    /// [`CfgReach::Unclassified`], which is **not** reachable.
    /// `src/system/utl/Pool.cpp` is the live instance: two `cflow-if-1`
    /// functions, and a constructor the census tags `cf-expr-0x05` whose obj is
    /// an `mtctr`/`bdnz` CTR loop.
    ///
    /// Returns one row per frontier TU, sorted by source path.
    pub fn frontier_cfg_reachability(&self) -> Vec<(&TuResult, CfgReach)> {
        let mut v: Vec<(&TuResult, CfgReach)> = self
            .factor_frontier()
            .into_iter()
            .map(|(r, _)| (r, Self::cfg_reach(r)))
            .collect();
        v.sort_by(|a, b| a.0.src.cmp(&b.0.src));
        v
    }

    /// [`Self::frontier_cfg_reachability`]'s verdict for one TU. Public and
    /// separate so the tests can call it on a TU that is not on the frontier.
    pub fn cfg_reach(r: &TuResult) -> CfgReach {
        Self::cfg_reach_with(PORT_CFG_CLASSES, r)
    }

    /// [`Self::cfg_reach`] against **an arbitrary class list** (board **#778**).
    ///
    /// The parameter is what makes the sub-class mechanism gradeable rather
    /// than merely present. Three callers depend on it:
    ///
    /// * [`Self::cfg_reach`] — the shipped list, the only answer anyone acts on.
    /// * [`Self::cfg_reach_bounds`] — the `⊥ ⊆ shipped ⊆ ⊤` nesting, re-derived
    ///   on the live workload every scan, which is how "narrower or equal" is
    ///   *measured* instead of argued.
    /// * A loop lane with a candidate restriction, which can now price it
    ///   against the real workload **before** proposing it — the move neither
    ///   `w-rotate` nor `w-sched2` had available.
    ///
    /// # Naming a miss under a partial claim
    ///
    /// When no entry covers the class at all, the missing class is named by its
    /// bare string — byte-identical to what the flat list produced. When an
    /// entry *does* cover the class but does not admit this key, the miss is
    /// named `"<class>!<key>"`: the port has part of the class and this body is
    /// outside the part. Collapsing that into the bare class would report a
    /// partially-covered class as wholly missing, which is the same
    /// over-statement as the wholesale claim, pointing the other way.
    ///
    /// **Nothing is restricted today, so the `!` form cannot occur on this
    /// tree** — which is precisely why the identity measurement is meaningful.
    pub fn cfg_reach_with(list: &[CfgClass<'_>], r: &TuResult) -> CfgReach {
        let blocked_total: usize = r.fn_blockers.values().sum();
        let mut classified = 0usize;
        let mut outside: BTreeSet<String> = BTreeSet::new();
        for (k, n) in &r.fn_cflow {
            // Only the CROSSED rows (`<cflow class>|<census key>`) name a
            // blocked function; a bare class row counts every function in the
            // TU, in-class ones included, and summing those would over-count.
            let Some((class, key)) = k.split_once('|') else {
                continue;
            };
            classified += n;
            if !list.iter().any(|e| e.admits(class, key)) {
                if list.iter().any(|e| e.covers_class(class)) {
                    outside.insert(format!("{class}!{key}"));
                } else {
                    outside.insert(class.to_string());
                }
            }
        }
        if !outside.is_empty() {
            return CfgReach::NeedsClass(outside);
        }
        if classified < blocked_total {
            return CfgReach::Unclassified(blocked_total - classified);
        }
        if blocked_total == 0 {
            // Nothing blocked: there is no reachability question to answer, and
            // answering `Reachable` would credit a TU the screen never tested.
            return CfgReach::Unclassified(0);
        }
        CfgReach::Reachable
    }

    /// **THE NARROWING MEASUREMENT** (lane `w-subclass`, board **#778**) — the
    /// sub-class mechanism's reachability figure bracketed on the live workload.
    ///
    /// [`CfgClass::admits`] makes "a restriction is narrower or equal" an
    /// algebraic fact. This project does not accept algebra as evidence about
    /// an instrument, so the property is **re-derived from the real frontier on
    /// every scan** by running [`Self::cfg_reach_with`] against four lists over
    /// the same `results`:
    ///
    /// | list | built from | admits |
    /// |---|---|---|
    /// | `⊥` | every shipped entry rewritten `Keys(&[])` | nothing |
    /// | `enumerated` | every shipped entry rewritten as the **exact set of census keys this scan observed for its class** | the same pairs the shipped list does |
    /// | `shipped` | [`PORT_CFG_CLASSES`] | today's answer |
    /// | `⊤` | every class observed in the frontier cross-tab, `Whole` | every class present |
    ///
    /// Returns the reachable TU paths for each, **as sets by name rather than
    /// as counts** — trap 4's shape: `|⊥| ≤ |shipped| ≤ |⊤}` is satisfied
    /// exactly by swapping one TU for another, so the counts cannot distinguish
    /// nesting from coincidence and the names can.
    ///
    /// # What each bound is actually load-bearing for
    ///
    /// * **`⊥` is the live exercise of the `Keys` path, and the M1 detector.**
    ///   It must be **empty**: a list admitting no `(class, key)` pair leaves
    ///   every frontier TU with a blocked function reading `NeedsClass`. A
    ///   matcher that ignored its key argument would make `⊥` equal `shipped`
    ///   instead, and that is the wrongly-permissive mutation the brief asks
    ///   for. Without this bound `Keys` would be a code path no run reaches,
    ///   which this project rates worse than an absent one (`w-rotate` §7.2).
    /// * **`enumerated` is the agreement check.** Re-expressing a `Whole` entry
    ///   as the enumeration of its own observed keys must reproduce `shipped`
    ///   **TU for TU**; a difference means `Keys` and `Whole` disagree where
    ///   they are constructed to agree.
    /// * **`⊤` is the width the screen is NOT claiming.** Its gap to `shipped`
    ///   is the honest size of the refusal — with an inert `⊤` the nesting
    ///   would be vacuously true and would demonstrate nothing.
    ///
    /// **An instrument, never a gate.** Pure over `results`, reads no obj,
    /// licenses no emit, and `⊤` in particular is a hypothetical the port has
    /// no claim to whatsoever.
    pub fn cfg_reach_bounds(&self) -> CfgBounds<'_> {
        let rows = self.factor_frontier();
        // `⊤`: every class the frontier's cross-tab mentions, wholesale. Owned
        // separately because the class strings are borrowed from `results`.
        let mut top_classes: BTreeSet<&str> = BTreeSet::new();
        // The per-class observed key sets, for `enumerated`.
        let mut observed: std::collections::BTreeMap<&str, BTreeSet<&str>> = Default::default();
        for (r, _) in &rows {
            for k in r.fn_cflow.keys() {
                if let Some((class, key)) = k.split_once('|') {
                    top_classes.insert(class);
                    observed.entry(class).or_default().insert(key);
                }
            }
        }
        let top_list: Vec<CfgClass<'_>> = top_classes
            .iter()
            .map(|c| CfgClass { class: c, sub: CfgSub::Whole })
            .collect();
        let bottom_list: Vec<CfgClass<'_>> = PORT_CFG_CLASSES
            .iter()
            .map(|e| CfgClass { class: e.class, sub: CfgSub::Keys(&[]) })
            .collect();
        // `enumerated` needs the key vectors to outlive the list that borrows
        // them, so they are materialised first and kept alive by `_keysets`.
        let keysets: Vec<(&str, Vec<&str>)> = PORT_CFG_CLASSES
            .iter()
            .map(|e| {
                let ks: Vec<&str> = match e.sub {
                    // A `Whole` entry enumerates exactly what this scan saw.
                    CfgSub::Whole => observed
                        .get(e.class)
                        .map(|s| s.iter().copied().collect())
                        .unwrap_or_default(),
                    // A restricted entry enumerates what it already lists.
                    CfgSub::Keys(ks) => ks.to_vec(),
                };
                (e.class, ks)
            })
            .collect();
        let enum_list: Vec<CfgClass<'_>> = keysets
            .iter()
            .map(|(c, ks)| CfgClass { class: c, sub: CfgSub::Keys(ks.as_slice()) })
            .collect();
        let reach_under = |list: &[CfgClass<'_>]| -> Vec<&str> {
            let mut v: Vec<&str> = rows
                .iter()
                .filter(|(r, _)| Self::cfg_reach_with(list, r).is_reachable())
                .map(|(r, _)| r.src.as_str())
                .collect();
            v.sort_unstable();
            v
        };
        CfgBounds {
            frontier: rows.len(),
            bottom: reach_under(&bottom_list),
            enumerated: reach_under(&enum_list),
            shipped: reach_under(PORT_CFG_CLASSES),
            top: reach_under(&top_list),
            top_classes: top_classes.into_iter().collect(),
            enumerated_keys: keysets.iter().map(|(_, ks)| ks.len()).sum(),
        }
    }

    /// **THE SUB-CLASS LEDGER** (lane `w-subclass`, board **#778**) — one row
    /// per shipped entry, so a partial claim is auditable against the workload
    /// that is supposed to justify it.
    ///
    /// A restriction is a claim about a named set of census keys. Two ways it
    /// can be quietly wrong, both of which this project has been bitten by:
    ///
    /// * **A listed key no scan witnesses** is a claim doing nothing — trap 5,
    ///   absence read as success, with the claim still on the page. The ledger
    ///   reports it as `unwitnessed` **with a count**, never as silence.
    /// * **The matcher and the declaration disagreeing.** `admitted` is
    ///   recomputed here by asking [`CfgClass::admits`] about every observed
    ///   key, and `declared` by literal membership in the listed slice. They
    ///   must be equal. Under must-fail mutation **M2** (exact → `starts_with`)
    ///   `admitted` exceeds `declared` and the row names the intruders.
    ///
    /// The cross-check is **`None` when the entry is `Whole`** — there is no
    /// declaration to cross-check against and printing `PASS` for it would be
    /// exactly the absence-read-as-success this row exists to forbid.
    pub fn cfg_subclass_ledger(&self) -> Vec<CfgLedgerRow> {
        self.cfg_subclass_ledger_with(PORT_CFG_CLASSES)
    }

    /// [`Self::cfg_subclass_ledger`] against an arbitrary list.
    ///
    /// **The intruder cross-check needs a RESTRICTED entry to have anything to
    /// say, and no shipped entry is restricted today**, so on this tree the
    /// shipped ledger reports `n/a` on every row and the check is untested by
    /// construction. That is the exact shape of an ungraded code path. This
    /// parameterized form is how the cross-check gets graded — a test builds a
    /// restricted list and asserts `intruders` is empty under exact matching,
    /// and must-fail mutation **M2** (exact → `starts_with`) makes it non-empty
    /// and fails that test. It is also what a loop lane calls to audit a
    /// candidate restriction against the workload before proposing it.
    pub fn cfg_subclass_ledger_with(&self, list: &[CfgClass<'static>]) -> Vec<CfgLedgerRow> {
        let mut observed: std::collections::BTreeMap<&str, BTreeSet<&str>> = Default::default();
        for r in &self.results {
            for k in r.fn_cflow.keys() {
                if let Some((class, key)) = k.split_once('|') {
                    observed.entry(class).or_default().insert(key);
                }
            }
        }
        list.iter()
            .map(|e| {
                let seen = observed.get(e.class).cloned().unwrap_or_default();
                let admitted: BTreeSet<&str> =
                    seen.iter().copied().filter(|k| e.admits(e.class, k)).collect();
                let (listed, unwitnessed, intruders) = match e.keys() {
                    None => (None, Vec::new(), None),
                    Some(ks) => {
                        let declared: BTreeSet<&str> = ks.iter().copied().collect();
                        let un: Vec<String> = declared
                            .iter()
                            .filter(|k| !seen.contains(*k))
                            .map(|k| k.to_string())
                            .collect();
                        let extra: Vec<String> = admitted
                            .difference(&declared)
                            .map(|k| k.to_string())
                            .collect();
                        (Some(ks.len()), un, Some(extra))
                    }
                };
                CfgLedgerRow {
                    class: e.class,
                    listed,
                    observed_keys: seen.len(),
                    admitted_keys: admitted.len(),
                    unwitnessed,
                    intruders,
                }
            })
            .collect()
    }

    /// **The known-answer control on the CFG screen** (board **#721**).
    ///
    /// `xboxmem.cpp` is the one TU this project ever converted from per-function
    /// codegen breadth. On a tree where it matches, `fn_blockers` is empty and
    /// the screen has nothing to answer — so what the control asserts is the
    /// standing fact the screen is built on: **every CFG class appearing in that
    /// TU at all is one of the port's two** (`cflow-if-1` × 3 +
    /// `cflow-straight` × 1, measured). If a future tree shows a matching TU
    /// carrying a class outside [`PORT_CFG_CLASSES`], the list is wrong and this
    /// returns `false`.
    ///
    /// Returns `None` when the TU is not in `results` — an absent control is
    /// never reported as a passing one.
    ///
    /// # It asks `covers_class`, NOT `admits`, and that is deliberate
    ///
    /// This control is **class-level** and stays class-level after board #778.
    /// The cross-tab key on a *matching* TU is the census's **in-class label**
    /// (`cond-tail-pair`, `cmp-shift-or`), not a blocker key — `fn_cflow` is
    /// written over every function and `FnVerdict::key` spells both populations
    /// into one namespace. A [`CfgSub::Keys`] restriction enumerates **blocker**
    /// keys, so asking it about an in-class label is a category error: it would
    /// report `FAIL` on a converted TU the moment any lane restricted a class,
    /// for a reason that has nothing to do with the TU. The question this
    /// control exists to ask — *"did a converted TU carry a class the list does
    /// not name at all?"* — is answerable at class level and is unchanged.
    pub fn cfg_reach_control(&self, src: &str) -> Option<bool> {
        let r = self.results.iter().find(|r| r.src == src)?;
        Some(r.fn_cflow.keys().all(|k| {
            let class = k.split('|').next().unwrap_or(k.as_str());
            PORT_CFG_CLASSES.iter().any(|e| e.covers_class(class))
        }))
    }

    /// **The known-answer control on the ranker** (board **#501**): a `match` TU
    /// is byte-identical to `c2`'s obj, so the port produced a body for every
    /// `.text` byte in it and its fraction **must** read 100 %.
    ///
    /// Returns `(at_100, no_denominator, shortfall)`, where each shortfall entry
    /// is `(explained_by_factor_e, tu, accepted, denominator)`.
    ///
    /// **Factor E is the one legitimate shortfall.** E is a *whole-TU*
    /// recognizer and the ranker's numerator is the *per-function* path
    /// (`codegen::select_function`), which structurally cannot answer for one —
    /// the identical shape as the factorization's own factor-D control, which
    /// `docs/STATUS.md` leaves red on purpose. Anything **not** explained by E is
    /// a defect in the numerator, and that count is the one that must be 0.
    ///
    /// **The FRONTIER is unaffected in either case**: it is defined as
    /// `A∧B∧C ∧ ¬(D∨E)`, so no factor-E TU can reach the ranking.
    #[allow(clippy::type_complexity)]
    pub fn byte_fraction_control(&self) -> (usize, usize, Vec<(bool, &TuResult, usize, usize)>) {
        let mut full = 0;
        let mut nodenom = 0;
        let mut short = Vec::new();
        for r in self.results.iter().filter(|r| r.class == TuClass::Match) {
            match super::fnbytes::byte_fraction(r) {
                None => nodenom += 1,
                Some((n, d)) if n == d => full += 1,
                Some((n, d)) => short.push((Self::factors(r)[4], r, n, d)),
            }
        }
        (full, nodenom, short)
    }

    /// **The stable machine-readable metric block**, one `key value` pair per
    /// entry, for `scripts/status.sh` and any other collector.
    ///
    /// # Why this exists
    ///
    /// Every figure here was already printed by a `gap` scan, in prose, and
    /// **three of them went stale twice in one day** (2026-08-04, lane
    /// `w-book4`): factor `C`, `A∧B∧C` and the `FRONTIER` live only in
    /// hand-written `STATUS.md`/`BOARD.md` paragraphs because the collector's
    /// five `sed` recipes cover only the six `TuClass` counters. `B∧C` was
    /// worse — it was published once, at `C = 114`, and then *silently
    /// invalidated by a dependency* when the writer's section vocabulary grew.
    /// A number a script cannot re-derive is a number that goes stale, and the
    /// project navigated by a stale one for two merges.
    ///
    /// # The two rules this block follows
    ///
    /// * **Keys are stable and values are bare integers or bare tokens**, so a
    ///   `sed`-based collector can take them without a parser. The keys are
    ///   part of the interface: renaming one silently returns `NO-RESULT`,
    ///   which is trap 5 (absence read as success) with the mask on.
    /// * **Derived quantities are derived HERE.** `emit-predicate-worth` is
    ///   `B∧C − A∧B∧C`; publishing the two halves and letting a reader subtract
    ///   is precisely how `+82` survived both of its inputs moving.
    ///
    /// Pure over `results`, so the unit test below grades it with no toolchain.
    pub fn metrics(&self) -> Vec<(&'static str, String)> {
        let [a, b, c, d, e, a_lo, bc, abc, abcd, joint] = self.factor_counts();
        let graded = self.graded().count();
        let frontier = self.factor_frontier().len();
        let ladder = self.section_ladder();
        let cfgb = self.cfg_reach_bounds();
        let cfgl = self.cfg_subclass_ledger();
        let mut m: Vec<(&'static str, String)> = vec![
            ("tu-total", self.results.len().to_string()),
            ("graded", graded.to_string()),
            ("match", self.count(TuClass::Match).to_string()),
            ("mismatch", self.count(TuClass::Mismatch).to_string()),
            ("codegen-gap", self.count(TuClass::CodegenGap).to_string()),
            ("vocab-gap", self.count(TuClass::VocabGap).to_string()),
            ("port-error", self.count(TuClass::PortError).to_string()),
            ("capture-fail", self.count(TuClass::CaptureFail).to_string()),
            ("factor-a", a.to_string()),
            ("factor-a-lo", a_lo.to_string()),
            ("factor-b", b.to_string()),
            ("factor-c", c.to_string()),
            ("factor-d", d.to_string()),
            ("factor-e", e.to_string()),
            ("b-and-c", bc.to_string()),
            ("a-and-b-and-c", abc.to_string()),
            ("a-and-b-and-c-and-d", abcd.to_string()),
            ("a-and-b-and-c-and-d-or-e", joint.to_string()),
            ("frontier", frontier.to_string()),
            ("frontier-if-a", self.factor_frontier_if_a().to_string()),
            // The headline projection, derived here so it cannot be assembled
            // from two independently-stale halves. Board #213.
            ("emit-predicate-worth", bc.saturating_sub(abc).to_string()),
            ("writer-sections", PORT_WRITER_SECTIONS.len().to_string()),
            // **The CFG sub-class mechanism** (board #778). `cfg-reach-shipped`
            // is the figure the screen prints; the other three are the bracket
            // that makes it a NARROWER-OR-EQUAL claim rather than a bare number,
            // and they are published beside it for the same reason
            // `emit-predicate-worth` is derived here: a reachability figure
            // quoted without the bound it sits inside is the shape #213's `+82`
            // had. `cfg-bounds-violations` MUST read 0 — it is a count and not a
            // status, which is trap 5's standing mitigation.
            ("cfg-reach-bottom", cfgb.bottom.len().to_string()),
            ("cfg-reach-enumerated", cfgb.enumerated.len().to_string()),
            ("cfg-reach-shipped", cfgb.shipped.len().to_string()),
            ("cfg-reach-top", cfgb.top.len().to_string()),
            ("cfg-bounds-violations", cfgb.violations().len().to_string()),
            ("cfg-subclass-entries", cfgl.len().to_string()),
            (
                "cfg-subclass-restricted",
                cfgl.iter().filter(|r| r.listed.is_some()).count().to_string(),
            ),
            (
                "cfg-subclass-unwitnessed",
                cfgl.iter().map(|r| r.unwitnessed.len()).sum::<usize>().to_string(),
            ),
            (
                "cfg-subclass-intruders",
                cfgl.iter()
                    .filter_map(|r| r.intruders.as_ref())
                    .map(|v| v.len())
                    .sum::<usize>()
                    .to_string(),
            ),
            ("workload-sections", self.section_vocabulary().len().to_string()),
            ("ladder-steps", ladder.len().to_string()),
            // **The control-flow counterfactual and its denominator, together.**
            // Four keys and not one, because the single number these replace —
            // "718" — was quoted for eight days as the price of the block-IR
            // restructure while being a LOWER bound of unknown tightness. The
            // pairing is the interface: `-modeled` is the counterfactual,
            // `-branchy` is the population it is a fraction of, and
            // `residue-inclass-offclass` over
            // (`-offclass` + `-modeled`) is how far the predicate that produced
            // it has fallen behind the class it mirrors. Boards #1343/#1344.
            (
                "cflow-emitted-branchy",
                self.cflow_emitted_counterfactual().0.to_string(),
            ),
            (
                "cflow-emitted-modeled",
                self.cflow_emitted_counterfactual().1.to_string(),
            ),
            (
                "cflow-residue-inclass-modeled",
                self.cflow_residue_control().0.to_string(),
            ),
            (
                "cflow-residue-inclass-offclass",
                self.cflow_residue_control().1.to_string(),
            ),
            // The error pointing the OTHER way, so the pair cannot be read as
            // "the residue is conservative". It is not; it is a different
            // predicate, and both differences are published.
            (
                "cflow-residue-straight-modeled-blocked",
                self.cflow_residue_overclaim().to_string(),
            ),
            // **The accounting control for the decomposition below** — the sum
            // of the per-reason IN-CLASS column, printed beside the total it
            // must equal rather than asserted against it. Board #1345.
            //
            // Two counts of the same population in the same unit (bodies), from
            // two different maps: `cflow-residue-inclass-offclass` reads the
            // `cflow` cross, this reads `fn_cflow_off`. They agree only if every
            // off-class body recorded a reason, which is `Scan::off_class`'s
            // `first reason wins` invariant. `w-tag02` is why it is printed and
            // not asserted: an identity whose two sides are counted in different
            // units is 0 forever and green for the wrong reason.
            (
                "cflow-offclass-accounted",
                self.cflow_offclass_reasons().1.to_string(),
            ),
        ];
        // **A COUNTERFACTUAL RUN MUST SAY SO ITSELF.** When
        // `C2RS_CFRESIDUE_ADMIT` is set, every `cflow-*` key above is a
        // what-if and not the shipped predicate — so the set is printed beside
        // them, and the key is ABSENT rather than empty when it is not set.
        // Absence reads as success unless something forbids it (trap 5): an
        // empty-string key would be indistinguishable from a default run in a
        // grep, and this is exactly the direction where confusing the two
        // publishes a counterfactual as a measurement.
        let admit = c2_il::func::cflow_residue_admit_set();
        if !admit.is_empty() {
            m.push(("cflow-residue-admit", admit));
        }
        // **The decomposition itself**, one key per reason, both populations.
        // Emitted as `cflow-offclass-<reason>-{inclass,blocked}` rather than one
        // packed string so each stays `sed`-able, and emitted only for reasons
        // that OCCUR — a key printed as 0 for a reason with no bodies would let
        // a collector read a vanished arm as an empty one.
        for (why, inc, blk) in self.cflow_offclass_reasons().0 {
            m.push((
                Box::leak(format!("cflow-offclass-{why}-inclass").into_boxed_str()),
                inc.to_string(),
            ));
            m.push((
                Box::leak(format!("cflow-offclass-{why}-blocked").into_boxed_str()),
                blk.to_string(),
            ));
        }
        // The ladder head, when there is one. Emitted as two keys rather than
        // one "name C=n" string so the numeric one stays `sed`-able, and
        // omitted entirely when the vocabulary is closed — a collector that
        // reads a missing key as 0 would then claim a closed ladder reaches
        // C = 0, so absence must be absence.
        if let Some((name, reach)) = ladder.first() {
            m.push(("ladder-head", name.clone()));
            m.push(("ladder-head-c", reach.to_string()));
        }
        // PROGRESS MASS — emitted only when the scan graded something, for the
        // reason `progress_mass` returns `Option`: a collector must read an
        // empty scan as NO-RESULT, never as any number. The `f`-term inputs are
        // published beside it so the value is never quotable without its
        // denominators.
        if let Some(p) = self.progress_mass() {
            m.push(("progress-mass", format!("{:.5}", p.value)));
            m.push(("progress-emitted-in-class", p.emitted_in_class.to_string()));
            m.push(("progress-emitted-total", p.emitted_total.to_string()));
            m.push(("progress-mismatch-zeroed", p.mismatch_zeroed.to_string()));
        }
        // FUNCTION BYTE MATCH — emitted only when at least one emitted function
        // was graded, for the reason `fn_byte_match` returns `Option`: a
        // collector must read "nothing graded" as NO-RESULT and never as a
        // number, least of all as 1.0. Every bucket rides along so the ratio is
        // never quotable without the partition it came from, and `fnbyte-partial`
        // — the size of the instrument's own under-report — is not optional.
        if let Some(f) = self.fn_byte_match() {
            m.push(("fnbyte-match", format!("{:.5}", f.value)));
            m.push(("fnbyte-exact", f.exact.to_string()));
            m.push(("fnbyte-denominator", f.denominator.to_string()));
            m.push(("fnbyte-differs", f.differs.to_string()));
            m.push(("fnbyte-partial", f.partial.to_string()));
            m.push(("fnbyte-refused", f.refused.to_string()));
            m.push(("fnbyte-unbound", f.unbound.to_string()));
            m.push(("fnbyte-partition-broken", f.partition_broken.to_string()));
            m.push(("fnbyte-census-disagree", f.census_disagree.to_string()));
            m.push(("fnbyte-exact-relocated", f.exact_relocated.to_string()));
            // **RELOC-EQ** (lane `w-relo`, board #884). `fnbyte-reloc-differs`
            // is published as its OWN key and never folded into `differs`: the
            // two are different repairs, and merging them would put one defect
            // in two work queues while making the widening unauditable.
            //
            // `fnbyte-exact-bytes` is the OLD `fnbyte-exact` predicate, kept so
            // the number this widening replaced stays derivable to the digit,
            // and `fnbyte-reloc-graded` / `-reloc-unknown` are the population
            // the compare could reach and its counted residue (trap 0).
            m.push(("fnbyte-reloc-differs", f.reloc_differs.to_string()));
            m.push(("fnbyte-reloc-unknown", f.reloc_unknown.to_string()));
            m.push(("fnbyte-reloc-graded", f.reloc_graded.to_string()));
            m.push(("fnbyte-exact-bytes", f.exact_bytes.to_string()));
            m.push((
                "fnbyte-reloc-partition-broken",
                f.reloc_partition_broken.to_string(),
            ));
            m.push((
                "fnbyte-match-tu-reloc-differs",
                f.match_tu_reloc_differs.to_string(),
            ));
            for k in [
                "fnbyte-reloc-table-unreadable",
                "fnbyte-reloc-index-desync",
                "fnbyte-reloc-graded-relocated",
            ] {
                m.push((k, self.emit_total(k).to_string()));
            }
            for kind in ["count", "offset", "type", "target", "section-target"] {
                m.push((
                    Box::leak(format!("fnbyte-reloc-differs-{kind}").into_boxed_str()),
                    self.emit_total(&format!("fnbyte-reloc-differs|{kind}"))
                        .to_string(),
                ));
            }
            m.push((
                "fnbyte-reloc-witnesses",
                self.fn_byte_reloc_witnesses().len().to_string(),
            ));
            m.push((
                "fnbyte-match-tu-differs",
                f.match_tu_differs.to_string(),
            ));
            m.push(("fnbyte-whole-tu", f.whole_tu.to_string()));
            m.push((
                "fnbyte-tus-full",
                self.fn_byte_by_tu()
                    .iter()
                    .filter(|(_, e, d)| e == d)
                    .count()
                    .to_string(),
            ));
            m.push(("fnbyte-tus", self.fn_byte_by_tu().len().to_string()));
            // **MECHANISM E's own counters** (`c2_core::elide`, lane `w-empty`).
            // `fnbyte-elided` is how many bodies the elision produced;
            // `fnbyte-elided-exact` how many of those the judge agrees with. The
            // pair is emitted unconditionally, zeros included, because a rule
            // that quietly stopped firing would otherwise read as "no news".
            //
            // `fnbyte-name-disagree` is the control on the elision's one input:
            // census rows whose positional `IlFunction::mangled_name` differs
            // from their per-record `FnCensus::emit_name`. It read **74,955** on
            // the dc3 workload, and keying the elision on the first of those two
            // names produced 14 wrong bodies and zero right ones. It is
            // published so no later lane keys another name-matched fact off the
            // positional binding without seeing the number first.
            // `fnbyte-elided-ref-reloc` is w-drop3's caveat closed for this
            // population: known answer **0**, because an elided body is one
            // `4e800020` and carries no relocation for a symbol to disagree
            // about. Printed, not inferred.
            for k in [
                "fnbyte-elided",
                "fnbyte-elided-exact",
                "fnbyte-elided-ref-reloc",
                "fnbyte-name-disagree",
            ] {
                m.push((k, self.emit_total(k).to_string()));
            }
            // **THE RELOCATION TARGET** (lane `w-drop3`, boards #984–#986) —
            // whom the port's body calls, against whom c2's does, by symbol
            // name, over the graded population.
            //
            // `fnbyte-exact-relocated` above says how many credited bodies carry
            // a relocation the byte test never checked (**#882**). These say how
            // many of them **point somewhere else**, which is the question that
            // number was standing in for. Emitted unconditionally, zeros
            // included: `-disagree-exact` is a control whose known answer is a
            // count, and a key that appeared only when nonzero would make a
            // wrong emit's absence read as success.
            for k in [
                "fnbyte-calltarget-graded",
                "fnbyte-calltarget-agree",
                "fnbyte-calltarget-disagree",
                "fnbyte-calltarget-disagree-exact",
                "fnbyte-calltarget-disagree-differs",
                "fnbyte-calltarget-disagree-name",
                "fnbyte-calltarget-disagree-count",
                "fnbyte-calltarget-ungraded",
                "fnbyte-call-targets-unreadable",
                // **THE CROSS-CHECK between the two readers** (lane `w-relo`).
                // `w-drop3`'s walk asks `REL24` targets by name; `compare_relocs`
                // asks every record's offset, packed type and target. Written by
                // two lanes from two sources, so their agreement is evidence and
                // their disagreement is a finding — in a DIRECTION, which is why
                // there are three keys and not one.
                //
                // `-calltarget-only` is the one with a known answer of **0**: a
                // `REL24` target disagreement is a record disagreement and the
                // full compare cannot miss it. `-reloc-only` may legitimately be
                // positive (a data-symbol target, a type, an offset — none of
                // which the call-target walk looks at) and is measured, not
                // predicted.
                "fnbyte-reloc-vs-calltarget-both",
                "fnbyte-reloc-vs-calltarget-reloc-only",
                "fnbyte-reloc-vs-calltarget-calltarget-only",
            ] {
                m.push((k, self.emit_total(k).to_string()));
            }
            // **Board #980's own counters** — the dead-temporary reader that
            // feeds mechanism E a callee it could not previously establish.
            // Emitted unconditionally, zeros included, for the same reason the
            // three above are: a reader that stopped firing would read as "no
            // news".
            //
            // `-ref-other` is the **alarm**: for every row the fixpoint admitted
            // on this reader's evidence, c2's own `.text` COMDAT is asserted to
            // be one `4e800020`. A nonzero here says the rule fired on a body c2
            // emits bytes for, and it is a positive count rather than a
            // subtraction so it cannot be mistaken for absence.
            for k in [
                "fnbyte-noeffect-rows",
                "fnbyte-noeffect-admitted",
                "fnbyte-noeffect-ref-blr",
                "fnbyte-noeffect-ref-other",
                "fnbyte-noeffect-ref-absent",
                "fnbyte-noeffect-callee-unbound",
                "fnbyte-noeffect-callee-parsed-live",
                "fnbyte-noeffect-callee-refused",
            ] {
                m.push((k, self.emit_total(k).to_string()));
            }
            // **Board #1053's counters** — the SEED. Same discipline as #980's
            // above and for the same reason, with one difference that is the whole
            // rung: `-nothing-ref-other` is a stronger alarm than
            // `-noeffect-ref-other`, because a seed asserts UNCONDITIONALLY that
            // c2 emits nothing for the body, so every row here has a known answer
            // and not just the ones whose callee happened to close.
            for k in [
                "fnbyte-nothing-rows",
                "fnbyte-nothing-ref-blr",
                "fnbyte-nothing-ref-other",
                "fnbyte-nothing-ref-absent",
                "fnbyte-nothing-not-admitted",
                "fnbyte-nothing-unnamed",
            ] {
                m.push((k, self.emit_total(k).to_string()));
            }
            // Which PRODUCTION the seeded rows are refused under. `expr-lit-type-8207`
            // is the whole graded population; a second key appearing here is a
            // body the grid never saw and is a finding, not a bonus.
            for (key, n) in self.fn_byte_blr_stops("fnbyte-nothing-key|").into_iter().take(6) {
                m.push((
                    Box::leak(format!("fnbyte-nothing-key-{key}").into_boxed_str()),
                    n.to_string(),
                ));
            }
            // The stop histogram's top rows — the widening order for board
            // #980's rule. `Box::leak` for the same reason the shape census
            // leaks: this function's signature is `&'static str` and the key
            // half is data-derived. Bounded at 8 rows per report.
            for (key, n) in self.fn_byte_noeffect_stops().into_iter().take(8) {
                m.push((
                    Box::leak(format!("fnbyte-noeffect-stop-{key}").into_boxed_str()),
                    n.to_string(),
                ));
            }
            // The residue of board #980's own cluster, at EVERY level of the
            // chain the collector walks. Top 6 each; a row here is a production
            // and a count of functions it holds, which is what a follow-on rung
            // is sized off.
            //
            // **Levels, not two hard-coded prefixes.** This read
            // `[("fnbyte-blr-stop|", …), ("fnbyte-blr-stop2|", …)]` until lane
            // `w-memset` read the loop at level 3 and the chain got a fourth
            // link. The collector emitted `fnbyte-blr-stop3|…` and this renderer
            // dropped it on the floor, so the scan reported `blr-stop2`
            // unchanged and looked exactly like a reader that had done nothing.
            // A key that is collected and not rendered is a key that does not
            // exist, and it cost this lane a debugging pass to notice.
            for level in 1..=MAX_BLR_STOP_LEVELS {
                let (prefix, tag) = if level == 1 {
                    ("fnbyte-blr-stop|".to_string(), "blr-stop".to_string())
                } else {
                    (format!("fnbyte-blr-stop{level}|"), format!("blr-stop{level}"))
                };
                for (key, n) in self.fn_byte_blr_stops(&prefix).into_iter().take(6) {
                    m.push((
                        Box::leak(format!("fnbyte-{tag}-{key}").into_boxed_str()),
                        n.to_string(),
                    ));
                }
            }
            // **Board #322's own keys.** The four shapes FBM used to decline are
            // graded now, so the collector needs to be able to say *which* shape
            // moved and *which* stage still declines — a corpus total cannot.
            // Emitted unconditionally (including the zeros) for the reason every
            // control on this page is: a key that appears only when nonzero
            // makes absence read as success.
            //
            // `Box::leak` because this function's signature is
            // `Vec<(&'static str, String)>` — an interface `scripts/status.sh`
            // and four tests parse — and the shape half of these keys is
            // data-derived. The leak is bounded by the number of distinct
            // (shape, verdict) pairs the `Selected` enum can produce, which is
            // at most 7 × 6, per call; `metrics()` is called once per report.
            for (shape, verdict, n) in self.fn_byte_shape_census() {
                m.push((
                    Box::leak(format!("fnbyte-shape-{shape}-{verdict}").into_boxed_str()),
                    n.to_string(),
                ));
            }
            // `parse` FIRST, and it is new (lane `w-column`, board #1473). Until
            // this lane there were four stages and the reader's refusal was
            // filed under `selector`, so `fnbyte-decline-selector` published
            // 130,575 — every digit of it a body the IL parser refused, under a
            // codegen name. The split moves that count to `-parse` and leaves
            // `-selector` reading **0**, which is what it has always been worth.
            for d in ["parse", "opt-mode", "selector", "gy-shape", "data-ref"] {
                m.push((
                    Box::leak(format!("fnbyte-decline-{d}").into_boxed_str()),
                    self.emit_total(&format!("fnbyte-decline|{d}")).to_string(),
                ));
            }
            // …and the same split at the bucket, so `fnbyte-refused` decomposes
            // without anyone subtracting two published totals. The two rows sum
            // to `fnbyte-refused` and that identity is a printed control
            // (`fnbyte-refused-split-broken`), not an assertion — a bucket that
            // silently stopped being written would otherwise shrink one side.
            let rparse = self.emit_total("fnbyte-refused-parse");
            let rcodegen = self.emit_total("fnbyte-refused-codegen");
            m.push(("fnbyte-refused-parse", rparse.to_string()));
            m.push(("fnbyte-refused-codegen", rcodegen.to_string()));
            m.push((
                "fnbyte-refused-split-broken",
                (self.emit_total("fnbyte-refused") != rparse + rcodegen)
                    .then_some(1usize)
                    .unwrap_or(0)
                    .to_string(),
            ));
            m.push((
                "fnbyte-differs-witnesses",
                self.fn_byte_differ_witnesses().len().to_string(),
            ));
        }
        // **THE BYTE-FRACTION RANKER** (board #500) and its control (#501).
        //
        // The head is emitted as three keys — name, numerator, denominator —
        // and NOT as a percentage, for the reason `byte_fraction` gives: a
        // ratio whose denominator is not beside it is the shape of objdiff's
        // `calc_fuzzy_match_percent` bug, and a collector that saw only
        // `frontier-bytefrac-top 0` could not tell "the top TU is 0 % emitted"
        // from "there is no top TU". The whole block is omitted when the
        // frontier is empty, so absence is absence.
        let ranking = self.frontier_byte_ranking();
        if let Some((top, frac)) = ranking.first() {
            m.push(("frontier-bytefrac-top-tu", top.src.clone()));
            match frac {
                Some((n, d)) => {
                    m.push(("frontier-bytefrac-top-accepted", n.to_string()));
                    m.push(("frontier-bytefrac-top-denominator", d.to_string()));
                }
                // A frontier whose BEST member has no `.text` at all. Emitted as
                // a token, never as a 0 that would read like a measured zero.
                None => {
                    m.push(("frontier-bytefrac-top-accepted", "NO-DENOMINATOR".into()));
                    m.push(("frontier-bytefrac-top-denominator", "0".into()));
                }
            }
            // How much of the frontier the port has NOTHING for. The headline of
            // the ranking on this corpus, and the number a future lane should
            // watch: it is the count of TUs where codegen breadth has not begun.
            m.push((
                "frontier-bytefrac-zero",
                ranking
                    .iter()
                    .filter(|(_, f)| matches!(f, Some((0, _))))
                    .count()
                    .to_string(),
            ));
            m.push((
                "frontier-bytefrac-no-denominator",
                ranking.iter().filter(|(_, f)| f.is_none()).count().to_string(),
            ));
        }
        // **THE CODEGEN COLUMN, in a form a collector can take** (lane
        // `w-column`, board **#1474**). Six keys and not one, for the reason the
        // `cflow-emitted-*` pair above carries: the measurable price is
        // meaningless without the population it is a fraction of, and here the
        // population that matters is `frontier-codegen-reader` — the part that
        // CANNOT be priced. A collector taking `frontier-codegen-measured` alone
        // would read `0` as *"the frontier needs no codegen work"*, which is the
        // exact inversion this block exists to prevent.
        //
        // Emitted unconditionally, including on an empty frontier, so absence
        // never reads as success — `frontier-codegen-denominator 0` is
        // distinguishable from the keys being missing.
        {
            let cols = self.frontier_codegen();
            let s = |f: fn(&FrontierCodegen) -> usize| -> usize {
                cols.iter().map(|(_, c)| f(c)).sum()
            };
            m.push(("frontier-codegen-denominator", s(|c| c.denominator).to_string()));
            m.push(("frontier-codegen-exact", s(|c| c.exact).to_string()));
            m.push(("frontier-codegen-wrong", s(|c| c.wrong).to_string()));
            m.push(("frontier-codegen-refused", s(|c| c.cg_refused).to_string()));
            m.push(("frontier-codegen-reader", s(|c| c.reader).to_string()));
            m.push(("frontier-codegen-ungraded", s(|c| c.ungraded).to_string()));
            m.push((
                "frontier-codegen-measured",
                s(|c| c.wrong + c.cg_refused).to_string(),
            ));
            // The control, target 0 — a count and not a status (trap 5).
            m.push((
                "frontier-codegen-partition-broken",
                cols.iter().filter(|(_, c)| c.partition_broken()).count().to_string(),
            ));
        }
        let (ctl_full, ctl_nodenom, ctl_short) = self.byte_fraction_control();
        m.push(("bytefrac-control-full", ctl_full.to_string()));
        m.push(("bytefrac-control-no-denominator", ctl_nodenom.to_string()));
        m.push((
            "bytefrac-control-shortfall-explained",
            ctl_short.iter().filter(|(e, ..)| *e).count().to_string(),
        ));
        // **The one that must be 0.** A matched TU below 100 % that factor E does
        // not explain means the ranker's numerator stopped crediting something.
        m.push((
            "bytefrac-control-unexplained",
            ctl_short.iter().filter(|(e, ..)| !*e).count().to_string(),
        ));
        // **W-PHASE7 — the tag-0x10 ALIAS channel** (`rungs/
        // _2026-08-04-w-emitp-findings.md` §6). Emitted unconditionally, zeroes
        // included, in four groups that answer four different questions and must
        // not be summed with each other:
        //
        // * the decode's invariants, with **both nulls** — a field position
        //   quoted without its shifted read is a field position that was
        //   searched for, and the null is shipped here rather than described;
        // * `alias-dom-emitted`, `dom(alias) ∩ E`, joined against the same
        //   `.text` COMDAT leader list `emit-emitted` counts. **KNOWN ANSWER
        //   0**;
        // * `alias-inref-*`, the reachable population at the `in` `02`-node
        //   resolution site, on every TU whatever any writer does;
        // * `alias-datatu-relocs-alias` and `alias-emit-names`, the **live**
        //   population — what the port would name today. **KNOWN ANSWER 0 for
        //   both**; a nonzero is board #232's shape and an alarm, not a gap.
        //
        // `alias-dom-with-body` is the precondition for §6 step 4 and rides on
        // every scan for the reason `fnbyte-partial` does: a guard whose safety
        // condition is checked once in a test is a guard nobody re-checks.
        for k in [
            "alias-runs",
            "alias-tag10",
            "alias-head-fail",
            "alias-rt-fail",
            "alias-unbound-target",
            "alias-self",
            "alias-dup",
            "alias-bound",
            "alias-shape-e-to-g",
            "alias-dom-with-body",
            "alias-null-m1-bound",
            "alias-null-m1-shape",
            "alias-null-p1-bound",
            "alias-null-p1-shape",
            "alias-dom-emitted",
            "alias-dom-emitted-tus",
            "alias-target-emitted",
            "alias-inref-total",
            "alias-inref-unbound",
            "alias-inref-alias",
            "alias-inref-records",
            "alias-inref-tus",
            "alias-datatu-relocs",
            "alias-datatu-relocs-alias",
            "alias-emit-names",
            "alias-obj-relocs",
            "alias-obj-reloc-alias",
            "alias-obj-reloc-alias-tus",
            "alias-obj-reloc-target",
            "alias-obj-reloc-unreadable",
            // **The alias's REAL obj-level observable** — a COFF weak
            // external, `??_E<X>` -> `??_G<X>`, `SEARCH_ALIAS`. This is a
            // per-RECORD grade of the decode against c2's own symbol table,
            // which no emit-set metric could be. `-default-disagree` and
            // `-not-search-library` are the two alarms and their known answer is
            // 0; `-unpredicted` and `-unrealized` are recall's and precision's
            // error terms and are NOT the same kind of thing as each other.
            "alias-weak-records",
            "alias-weak-predicted",
            "alias-weak-default-disagree",
            "alias-weak-unpredicted",
            "alias-weak-not-search-library",
            "alias-weak-unreadable",
            "alias-weak-tus",
            "alias-weak-exact-tus",
            "alias-unrealized",
            // **THE REALISATION RULE** — "c2 writes `a -> t` iff `t` is a
            // `.text` COMDAT leader of the same obj" — with its two error
            // terms kept apart, because a rule that promised a record c2 did
            // not write and a rule that missed one c2 did are different
            // mistakes and a single "disagreement" count lets them cancel.
            "alias-rule-predicted",
            "alias-rule-miss",
            "alias-rule-extra",
            "alias-rule-exact-tus",
        ] {
            m.push((k, self.emit_total(k).to_string()));
        }
        // **THE CONSEQUENCE, and it is the only number on this page that turns
        // the alias channel into a bound on TU match.**
        //
        // Rule R says a TU whose obj emits `??_G<X>` must ALSO carry a
        // `WEAK_EXTERNAL` symbol record `??_E<X>` and its undefined default.
        // The port's COFF writer has no weak-external record at all, so **every
        // TU with a realised alias is unreachable until it does** — and that is
        // a blocker of a kind no factor in §10.19 represents. It is not
        // codegen, not the section vocabulary and not the emit set: it is the
        // SYMBOL TABLE.
        //
        // Three keys, and the third is the load-bearing one:
        //
        // * `alias-weak-needed-tus` — graded TUs needing ≥ 1 weak external;
        // * `alias-weak-needed-in-b-and-c` — the same, intersected with `B∧C`.
        //   **This intersection IS computable here** and is published rather
        //   than estimated, which is the discipline `w-emitp` §5 asked for and
        //   could not satisfy for its own quantity;
        // * `alias-weak-needed-in-frontier` — the same over the FRONTIER.
        {
            let mut needed = 0usize;
            let mut in_bc = 0usize;
            for r in self.graded() {
                if r.emit.get("alias-rule-predicted").copied().unwrap_or(0) == 0 {
                    continue;
                }
                needed += 1;
                let f = Self::factors(r);
                if f[1] && f[2] {
                    in_bc += 1;
                }
            }
            let front: std::collections::BTreeSet<&str> = self
                .factor_frontier()
                .into_iter()
                .map(|(r, _)| r.src.as_str())
                .collect();
            let in_front = self
                .graded()
                .filter(|r| front.contains(r.src.as_str()))
                .filter(|r| r.emit.get("alias-rule-predicted").copied().unwrap_or(0) > 0)
                .count();
            m.push(("alias-weak-needed-tus", needed.to_string()));
            m.push(("alias-weak-needed-in-b-and-c", in_bc.to_string()));
            m.push(("alias-weak-needed-in-frontier", in_front.to_string()));
        }
        m
    }

    /// **The section vocabulary census**: every distinct section name in the
    /// workload with the number of objs carrying it, most common first.
    ///
    /// The whole of factor C's problem, enumerated. It is a *finite* list —
    /// which is what makes C the one factor in §10.19 with a short route to
    /// closure — so the count of rows is itself the headline and is printed.
    pub fn section_vocabulary(&self) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix("emit-sec-name|")?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// Per-TU set of section names **outside** the port's writer vocabulary, for
    /// the graded TUs whose obj decoded. The ladder's input.
    fn extra_section_sets(&self) -> Vec<Vec<&str>> {
        self.graded()
            .filter(|r| r.emit.contains_key("emit-sec-readable"))
            .map(|r| {
                r.emit
                    .keys()
                    .filter_map(|k| k.strip_prefix("emit-sec-extra|"))
                    .collect()
            })
            .collect()
    }

    /// **The greedy section ladder**: which name to teach the writer next, by
    /// the TUs it brings into reach. Each row is `(name, resulting |C|)`.
    ///
    /// Greedy by immediate gain, ties broken by name ascending, and it **does
    /// not stop at a zero-gain step** — it runs until every readable obj is
    /// reachable. That matters: two names that only ever co-occur each score 0
    /// alone, so a ladder that halted on no-progress would report the vocabulary
    /// as unclosable when it is one step from closed. A zero-gain row printed
    /// beside a gain is also the honest way to say "these two are one step".
    ///
    /// Greedy is not proven optimal, and the row order is a *route*, not a
    /// schedule (`ROADMAP.md` §9.16.1). What it establishes is an upper bound on
    /// the length of the route, which is the claim §10.19 makes.
    pub fn section_ladder(&self) -> Vec<(String, usize)> {
        let sets = self.extra_section_sets();
        let mut taught: std::collections::BTreeSet<&str> = Default::default();
        let reach = |taught: &std::collections::BTreeSet<&str>| -> usize {
            sets.iter()
                .filter(|s| s.iter().all(|n| taught.contains(n)))
                .count()
        };
        let mut out = Vec::new();
        while reach(&taught) < sets.len() {
            let mut candidates: std::collections::BTreeSet<&str> = Default::default();
            for s in &sets {
                for n in s {
                    if !taught.contains(n) {
                        candidates.insert(n);
                    }
                }
            }
            let mut best: Option<(usize, &str)> = None;
            for c in candidates {
                let mut t = taught.clone();
                t.insert(c);
                let got = reach(&t);
                // Ties by name ascending: `BTreeSet` iterates sorted and the
                // comparison is strict, so the first of a tie wins and the
                // ladder is reproducible run to run.
                let better = match best {
                    None => true,
                    Some((n, _)) => got > n,
                };
                if better {
                    best = Some((got, c));
                }
            }
            let Some((got, name)) = best else { break };
            taught.insert(name);
            out.push((name.to_string(), got));
        }
        out
    }
}

//! **Mechanism I, the replacement stratum** — the body c2 emits for a caller
//! whose one call it expanded, which is the callee's body and **not** the
//! caller's setup with the callee appended.
//!
//! # What this is, and what it is NOT
//!
//! `docs/INLINE_PREDICATE.md` §0 separates two reasons an IL call can be absent
//! from c2's output. [`crate::elide`] is **E** — the callee's body does nothing,
//! so the call is dropped — and this module is **I**, the inliner expanding it.
//! The two are kept apart there and they are kept apart here: nothing in this
//! file reads [`c2_il::IlFunction::empty_body`] as a licence, and
//! [`crate::elide::drops_tail_call`] is asked **first** so that one function
//! never gets two answers.
//!
//! The composition rule is `w-seq`'s measurement and not a guess
//! (`docs/rungs/2026-08-08-w-seq.md` §4.1, graded by real c2 on 2,470 workload
//! callers whose reference obj carries *both* COMDATs):
//!
//! | hypothesis | graded | exact |
//! |---|---:|---:|
//! | **SPLICE-0** — c2's body for the caller **is** c2's body for the callee | 2,470 | **1,967** |
//! | — `seq` | 816 | **816** |
//! | — `tail` | 1,531 | 1,151 |
//! | — `framed` | 123 | **0** |
//! | SPLICE-P — the port's setup ++ the callee's body | 2,470 | 578 — **578/578** with no setup, **0/953** with one |
//! | SPLICE-N — two or more callees, concatenated | 548 | **0** |
//!
//! **The argument setup does not survive, and neither does the frame.** That is
//! the whole difference between this rule and the concatenation the same lane
//! registered first and lost: of the 1,892 SPLICE-P failures, 1,890 diverge at
//! word 0, which is the port's own first setup word.
//!
//! # The predicate, exactly as it was registered
//!
//! [`splice_body`] is the whole rule; `work/w-splice/PREREG.md` §1 is the
//! registered form and `work/w-splice/cells/` is the frozen grid that grades
//! each clause's boundary against real c2.
//!
//! | clause | what it requires | the cell that grades it |
//! |---|---|---|
//! | **S1** | the selection is [`Selected::Tail`] or [`Selected::Seq`] | `t07` — a **framed** caller. SPLICE-0 is **0 of 123** there, every one a destination-register rename |
//! | **S2** | exactly **one** call site | `t08` — two calls. SPLICE-N is **0 of 548** |
//! | **S3** | the port emits **nothing around the call**: for a `Tail`, an empty setup; for a `Seq`, an **identity argument mapping read off the IL** and a tail that is the ABI identity on r3 | `t04` (register move), `t05` (arithmetic), `t06` (pointer offset). Every one of w-seq's 503 SPLICE-0 failures is a **field** of the callee's body rewritten — a source rename (286), a destination rename (123), a displacement fold (~92) — and a non-identity setup is the thing that rewrites it. **A `Seq`'s emitted setup is NOT the test**: 816 of 816 carry the frame's own `mr r31,r3` and none carries a real marshalling, so the clause reads the IL |
//! | **S4** | the callee is not the caller | `t12`. `INLINE_PREDICATE.md` §4 grades `recurse` **336/336** declined by c2 as well |
//! | **S5** | the callee is defined in **this bundle**, unambiguously | `t14`, the control: an undefined callee keeps its REL24 |
//! | **S6** | the port composes a complete body for the **chain's end**, with **no frame**, and the chain **ENDED** rather than being cut off — it names no callee this TU carries (`S6-chain-truncated`) and it carries no call at all (`S6-chain-open`) | `t09` — an unlowerable callee. `t11` — **the fixpoint**: when the callee itself splices, its emitted COMDAT is *its* callee's body, so the walk steps down. `t10` — a chain that ends in a call, **refused**, and the 3 verified-correct relocations that refusal costs are the price of the 1 verified-wrong one it removes |
//! | **S7** | that body is at most [`INLINE_UNBOUNDED_BYTES`] bytes, and the callee is not varargs | `t13`. See below — this is the inline decision, taken on the safe side of its own boundary |
//! | **S8** | the caller materializes no data symbol of its own | the caller's whole body is discarded and an `F`-side data reference would go with it |
//! | **S9** | mechanism **E** does not fire for the caller | `elide.rs` keeps its answer; one function, one rule |
//!
//! **Every clause is readable from the IL bundle or from the port's own
//! emitter. None of them reads the reference obj.** That is what makes this a
//! predicate rather than the enumeration `w-seq` §6 priced at 726.
//!
//! # S7 IS THE INLINE DECISION, and it is taken where the decision is categorical
//!
//! `INLINE_PREDICATE.md` §2 is a *cost model* — graded 0.9716 on a 9,993-callee
//! frozen hold-out, with a 2.84 % residual its own §7 leaves **NOT MODELLED**.
//! A shipped emitter may not consult a 97 % rule; 3 % of a guess is a wrong
//! emit. What it may consult is the **region where that rule is categorical**:
//!
//! ```text
//! index(G) = s - 4*(nparams-1) - 8*[inline] - 48*[leaf]        <=  s
//!
//! EXTERNAL: N_max = UNBOUNDED  if index <= 64          (§6.17.4)
//! STATIC:   i = index/4;  N_max = UNBOUNDED if i <= 16 (§6.18.9)   i.e. index <= 64
//! either:   N_max = 0          if varargs(G)           (§6.18.5)
//! ```
//!
//! Both linkage classes turn unbounded at the **same** `index <= 64`, and every
//! correction term is subtractive, so `s <= 64` implies `index <= 64` implies
//! `N_max` unbounded — **independently of linkage, of `inline`, of `nparams`, of
//! the `leaf` bit that §5 calls the model's one unreadable input, and of the
//! site count**. Varargs is the single categorical exception and it is checked.
//!
//! The site-side exceptions of §2 are excluded structurally rather than
//! modelled: an **indirect** site names no callee and the IL parser refuses both
//! of its productions (`elide.rs`'s standing hazard note); a **conditional**
//! site is a `cond_pair` or a `Seq` with a `guard`, and S1/S3 refuse both — and
//! in any case that exception moves a ceiling from 260 to 164 bytes, both far
//! above this bound.
//!
//! # What it emits, and the relocation that makes it different from every other rule here
//!
//! The caller's `/Gy` COMDAT becomes the callee's — text, **relocations** and
//! data references, at the same in-section offsets. The caller acquires **no**
//! REL24 against the callee; it acquires the callee's own.
//!
//! > `t10` is that stated as a compiled cell. `void ext(); void g(){ext();}
//! > void f(){g();}` — the port emits `b ?g` for `?f` and c2 emits `b ?ext`, and
//! > **both are the word `48000000`**. FUNCTION BYTE MATCH compares a `.text`
//! > COMDAT's raw data, which does not contain its relocations, so it calls that
//! > pair `exact` today: board **#882**, 4,664 credited functions, and `w-seq`
//! > §4.3's `s12` is the same four lines. This rule *fixes* that cell rather than
//! > adding to it, and `work/w-splice/relocheck.py` verifies the relocation set
//! > of every function it moves, per symbol, against the reference obj.
//!
//! # THE FIXPOINT — registered as a question, and both answers agree
//!
//! `work/w-splice/PREREG.md` §4 registered *"does c2 close a two-step splice
//! chain?"* as a **question**, with the port taking one level either way. It
//! shipped at one level, and two independent measurements said that was wrong:
//!
//! * **`t11`**, a compiled cell — `int h(int a){return a+1;} int g(int a){return
//!   h(a);} int f(int a){return g(a);}` — c2 emits **`?h`'s two words for all
//!   three functions**;
//! * **the workload's relocation check**, which the one-level rule forced into
//!   existence: **150 of 945** spliced functions relocated against the chain's
//!   *intermediate* where c2 relocates against its *end*.
//!   `??1length_error@stlpmtx_std@@` named `??1__Named_exception@stlpmtx_std@@`
//!   and c2 names `??1exception@std@@`, 145 times in that one shape; the other
//!   five are `??1?$_List_base@…` where c2 names `?clear@?$_List_base@…`. **All
//!   150 were this, and none was a different target.**
//!
//! So the walk follows the chain to the first link that does **not** splice and
//! takes that body. Termination is structural — a step either repeats a name,
//! which is the `S6-chain-cycle` refusal, or admits a new one, and the bundle
//! has finitely many — with a ceiling behind it so that an edit breaking that
//! argument refuses instead of walking forever. Mechanism E reached the same
//! shape from the other direction (`elide.rs`, board #946): a chain, and a cycle
//! that is never admitted.
//!
//! **S7 still needs only one size check.** `INLINE-P`'s `s` is the callee's own
//! *emitted* size, and an intermediate that splices emits exactly the chain-end
//! body — c2's COMDAT for it *is* that body — so `s` is the same number at every
//! edge of the chain.
//!
//! # Where it may NOT fire — [`crate::PortC2::build`]
//!
//! `IlBundle::functions()` refuses any TU that defines one of its own callees,
//! which is a strict superset of S5, so no bundle that can splice ever reaches
//! the whole-obj emitter. That refusal is **not** narrowed here. Independently,
//! `build` refuses a bundle in which this predicate fires, on **both** paths and
//! with the reason named: a spliced `Seq` loses its frame, and with it its
//! `.pdata` record and its `$M`/`$M`/`$T` label slots, while
//! `PortC2::frame_label_counter` is computed from the IL before any body exists.
//! `elide.rs` refuses the packed path for the narrower version of the same
//! reason; this one reaches the label counter, so it refuses both.

use c2_il::{IlFunction, SeqTail};

use crate::codegen::{OptMode, Selected, opt_mode_of_word};
use crate::comdat::{ComdatBody, ComdatDecline};
use crate::elide::{Reduction, TuEmptyCallees, drops_tail_call};

/// **S7's bound.** `INLINE_PREDICATE.md` §2's `N_max` is UNBOUNDED at
/// `index <= 64` in *both* linkage classes, and `index <= s`, so a callee whose
/// emitted body is at most this many bytes is inlined at every site whatever its
/// linkage, its parameter count, its `inline` keyword or the model's unreadable
/// `leaf` bit.
///
/// It is a **byte** count because `s` is: §6.5 reads it off the callee's own
/// emitted `.text`.
pub const INLINE_UNBOUNDED_BYTES: usize = 64;

/// **The bundle-level facts a `/Gy` body needs that are not properties of one
/// function** — mechanism E's callee set, and the callee bodies mechanism I
/// splices.
///
/// It carries [`TuEmptyCallees`] rather than replacing it, and [`Deref`]s to it,
/// so every existing caller of `drops_tail_call(f, tu)` and `tu.reduces_to_nothing(..)` reads the
/// same E context it always did. **E is still `elide.rs`'s and only its**; this
/// type is the carrier, not a second implementation.
///
/// # It is name-keyed, and *which* name is the whole problem
///
/// The same #918 that `elide.rs` documents: `IlFunction::mangled_name` is paired
/// **positionally** over `.ex` segments and disagrees with the per-record
/// `FnCensus::emit_name` on **74,955** rows of the dc3 workload. A name-keyed
/// fact read through the wrong binding is attached to the wrong function — for
/// E that turned 14 byte-exact bodies wrong, and for this rule it would splice
/// some *other* function's body into the caller. So the name is supplied by the
/// caller through [`TuContext::of_named`], exactly as E's is, and the callee
/// resolution rate under the two bindings on this very population is **3,702 vs
/// 546 of 3,928** (w-seq §8).
#[derive(Debug, Default, Clone)]
pub struct TuContext<'a> {
    empty: TuEmptyCallees,
    /// `(name, definition, the `.ex` optimization word)`, sorted by name.
    /// A name with more than one row is **refused** rather than resolved to the
    /// first — see [`TuContext::definition`].
    ///
    /// The definition is `None` for a name this bundle **defines and whose IL
    /// the parser refused**. Those rows are kept, and keeping them is not
    /// bookkeeping: [`TuContext::mentions`] is what tells a chain that *ended*
    /// from one the port could not *follow*, and a refused row that vanished
    /// from this vector would read as an external. See the type's docs.
    ///
    ///
    /// The definition is `None` for a name this bundle **defines and whose IL
    /// did not parse**. Carrying those rows costs nothing and buys the one
    /// distinction a refusal census has to make: *"this TU has no such
    /// function"* and *"this TU has one and the port cannot read it"* price two
    /// completely different rungs — the second is `w-seq` §5's production table,
    /// which is 1,774 differs deep. Folding them together is how
    /// `refused:blocked` got printed 1,774 times and named nothing.
    rows: Vec<(&'a str, Option<&'a IlFunction>, Option<u32>)>,
}

impl<'a> TuContext<'a> {
    /// The context for a caller with no bundle: nothing is elided and nothing is
    /// spliced.
    ///
    /// Required rather than defaulted, for [`crate::comdat`]'s reason: the two
    /// bundle-level mechanisms are the facts a per-function composition cannot
    /// derive, and a caller that *forgot* one would silently emit a call c2 does
    /// not emit. Saying "no bundle" out loud is the point.
    pub fn none() -> Self {
        Self::default()
    }

    /// Build both bundle-level contexts from one pass over `(name, definition,
    /// opt_word)` triples the caller has already bound.
    ///
    /// `opt_word` is the callee's own `.ex` optimization word. `None` means the
    /// caller does not track it per function — [`crate::PortC2::build`], where
    /// the TU has already been refused unless every function shares one mode —
    /// and then the caller's mode is used. A callee whose mode differs from the
    /// caller's is refused (`mode` decides a register field inside a chain, so
    /// splicing across a mode boundary would emit the wrong allocation).
    pub fn of_named(
        named: impl IntoIterator<Item = (&'a str, Option<&'a IlFunction>, Option<u32>)>,
    ) -> Self {
        Self::of_rows(
            named
                .into_iter()
                // `Some(f)` is a parsed body both mechanisms may use; `None`
                // is a name this TU defines whose IL the parser refused, which
                // neither can use and which must still be VISIBLE — see
                // `of_rows`. This wrapper has no `no_effect_callee` to offer,
                // so it never mints a `NoEffectCall` edge; the census-row
                // caller in `gap/fnbytes.rs` is the one that does (#980).
                .map(|(n, f, w)| (n, f.map(Reduction::Parsed), w)),
        )
    }

    /// **The general constructor** — one row per name this TU defines, carrying
    /// what each of the two mechanisms can make of it.
    ///
    /// # Three questions, three row sets, and only one of them is every row
    ///
    /// | asked by | which rows |
    /// |---|---|
    /// | mechanism **E**'s closure ([`TuEmptyCallees::of_rows`]) | the rows with a [`Reduction`] — parsed, or refused **with a readable `no_effect_callee`** (board **#980**). Every other refused row contributes nothing, which is that board's conservative direction and is preserved here exactly |
    /// | [`TuContext::definition`], i.e. the splice's S5/S6 | **parsed rows only**. A `NoEffectCall` row is a body the parser refused, so the port has no bytes for it and S6 cannot compose a chain end out of it |
    /// | [`TuContext::mentions`] | **every row**, parsed or not, qualifying or not |
    ///
    /// **The third is the one a careless merge loses.** `S6-chain-truncated`
    /// refuses a splice when the chain's last link still names a callee this TU
    /// carries; a refused row dropped from this vector would read as an
    /// *external*, the clause would stop firing, and the splice would run off
    /// the end of a chain it cannot see. That is not a missing count — it is
    /// the wrong-relocation defect (#1020's 72 witnesses) coming back.
    ///
    /// So `Option<Reduction>` is three-valued on purpose: `Some(Parsed)` is a
    /// body both mechanisms can use, `Some(NoEffectCall)` is one only E can use,
    /// and `None` is a name this TU defines that **neither** can use and that
    /// still has to be visible.
    pub fn of_rows(
        rows: impl IntoIterator<Item = (&'a str, Option<Reduction<'a>>, Option<u32>)>,
    ) -> Self {
        let mut rows: Vec<(&'a str, Option<Reduction<'a>>, Option<u32>)> =
            rows.into_iter().filter(|(n, _, _)| !n.is_empty()).collect();
        rows.sort_by_key(|(n, _, _)| *n);
        // Mechanism E sees exactly what board #980 gives it and nothing else.
        let empty = TuEmptyCallees::of_rows(
            rows.iter().filter_map(|(n, r, _)| Some((*n, (*r)?))),
        );
        let rows = rows
            .into_iter()
            .map(|(n, r, w)| {
                // **Matched EXHAUSTIVELY, with no wildcard arm.** A refused body
                // has no bytes, so the splice can never take one — but the arm
                // that says so used to be `_ => None`, and a wildcard is how a
                // variant added by a peer lane gets a silent answer instead of a
                // considered one. Four lanes this week erased each other's work
                // through shared semantics with no textual conflict; a wildcard
                // here is that failure mode with the compiler's help switched off.
                let def = match r {
                    Some(Reduction::Parsed(f)) => Some(f),
                    // E gets a LINK out of this row (board #980); the splice gets
                    // nothing, because there is no parsed body to compose from.
                    Some(Reduction::NoEffectCall(_)) => None,
                    // E gets a SEED out of this row (board #1053); the splice
                    // still gets nothing, for the same reason and no other.
                    Some(Reduction::NoEffectNothing) => None,
                    // A name this TU defines that NEITHER mechanism can use, and
                    // which must still be visible to `mentions` — see this
                    // function's doc.
                    None => None,
                };
                (n, def, w)
            })
            .collect();
        Self { empty, rows }
    }

    /// [`TuContext::of_named`] over `IlFunction::mangled_name` with no per
    /// function optimization word — the binding `IlBundle::functions()` hands
    /// the emitter, where the TU's mode is already known to be uniform.
    ///
    /// **Do not reach for this from a census-row caller**, for the reason
    /// [`TuEmptyCallees::of`] gives: the census pairs names positionally.
    pub fn of(funcs: impl IntoIterator<Item = &'a IlFunction>) -> Self {
        Self::of_named(
            funcs
                .into_iter()
                .map(|f| (f.mangled_name.as_str(), Some(f), None)),
        )
    }

    /// Mechanism E's context — [`elide`](crate::elide)'s and unchanged.
    pub fn empty_callees(&self) -> &TuEmptyCallees {
        &self.empty
    }

    /// **S5.** The one definition of `name` in this bundle, or `None` when zero
    /// or **more than one** row carries it.
    ///
    /// Two rows are refused rather than resolved to the first: the two spellings
    /// would have to be told apart by something this type does not have, and
    /// splicing the wrong one is a silent wrong-bytes emit. This is the same
    /// answer `TuEmptyCallees` gives a name whose definitions disagree.
    pub fn definition(&self, name: &str) -> Option<(&'a IlFunction, Option<u32>)> {
        let i = self.unique_row(name)?;
        Some((self.rows[i].1?, self.rows[i].2))
    }

    /// Does this bundle carry `name` **at all** — whether or not its IL parsed,
    /// and whether or not exactly one row claims it?
    ///
    /// The distinction [`TuContext::definition`] cannot make, and the one a
    /// refusal census needs: a callee this TU does not define is an external and
    /// no mechanism can be about it, while a callee it defines and the parser
    /// refuses is a *priced* rung — `w-seq` §5 ranks those productions and the
    /// largest blocks 573 differs on its own.
    ///
    /// **It deliberately does not require the row to be unique.** An ambiguous
    /// name is one this TU *does* define, twice; reading it as "not defined
    /// here" is how one function slipped past `S6-chain-truncated` and
    /// relocated against `??1?$_Rb_tree@…` where c2 relocates against
    /// `?clear@?$_Rb_tree@…` — a wrong relocation FUNCTION BYTE MATCH scores
    /// `exact` (board #882).
    pub fn mentions(&self, name: &str) -> bool {
        self.rows
            .binary_search_by(|(n, _, _)| (*n).cmp(name))
            .is_ok()
    }

    /// More than one row claims `name`, so no single definition can be resolved.
    pub fn ambiguous(&self, name: &str) -> bool {
        self.mentions(name) && self.unique_row(name).is_none()
    }

    /// The single row for `name`, or `None` when zero or **more than one**
    /// carries it. Two rows are refused rather than resolved to the first, for
    /// the reason [`TuContext::definition`]'s doc gives.
    fn unique_row(&self, name: &str) -> Option<usize> {
        let i = self.rows.binary_search_by(|(n, _, _)| (*n).cmp(name)).ok()?;
        if i > 0 && self.rows[i - 1].0 == name {
            return None;
        }
        if i + 1 < self.rows.len() && self.rows[i + 1].0 == name {
            return None;
        }
        Some(i)
    }

    /// How many definitions the context can splice from — printed by
    /// diagnostics rather than inferred, the same reason [`TuEmptyCallees::len`]
    /// exists.
    ///
    /// # It is NOT called `len`, and that is a defect this lane shipped and caught
    ///
    /// This type `Deref`s to [`TuEmptyCallees`] so that every existing caller of
    /// `drops_tail_call(f, &tu)` and `tu.reduces_to_nothing(..)` keeps working
    /// untouched. **An inherent method shadows the `Deref` target's**, so while
    /// this was spelled `len` the scan's `fnbyte-tu-empty-callees` key — which
    /// has always meant *"how many names mechanism E admits"* — silently began
    /// reporting the bundle's whole definition count instead: **88,894 →
    /// 1,474,755** on the dc3 workload, a sixteen-fold move in a key nobody was
    /// diffing, with no compile error and no test failure.
    ///
    /// It was found by a merge-time control that counts every *peer lane's* key
    /// family at both ends (`work/w-splice/peerkeys.py`), not by anything in
    /// this lane's own acceptance. The name is now unambiguous, so no future
    /// caller can reach the wrong `len` by writing the obvious thing —
    /// `docs/GAPS.md` §6's "one fact, one locator" applied to a *name* rather
    /// than to an implementation.
    pub fn definitions(&self) -> usize {
        self.rows.len()
    }

    /// True when the bundle contributed no definition at all.
    ///
    /// Named apart from [`TuEmptyCallees::is_empty`] for [`Self::definitions`]'s
    /// reason.
    pub fn has_no_definitions(&self) -> bool {
        self.rows.is_empty()
    }
}

impl<'a> std::ops::Deref for TuContext<'a> {
    type Target = TuEmptyCallees;

    fn deref(&self) -> &TuEmptyCallees {
        &self.empty
    }
}

/// **S1–S5, S8, S9** — the part of the predicate that is a property of the
/// caller's own selection, with the callee it names.
///
/// Separated from [`splice_body`] because it is the half that can be asserted
/// without lowering anything, so a test can walk the boundary cells without an
/// IL bundle for the callee — and because `PortC2::build`'s refusal needs the
/// same question asked without paying for the callee's body.
///
/// `None` is the refusal, and it is the answer for every shape this rule was not
/// graded on. Nothing here is an assertion about the parser's invariants: an
/// [`IlFunction`] is a parse result and this is a predicate over it.
pub fn splice_callee<'a>(
    f: &IlFunction,
    selected: &Selected,
    tu: &TuContext<'a>,
) -> Option<&'a str> {
    splice_callee_why(f, selected, tu).ok()
}

/// [`splice_callee`] with **the clause that refused**, for a diagnostic that has
/// to price the shortfall rather than report it.
///
/// One implementation, two readers — the emitter takes `Ok`/`Err` and the
/// instrument prints the `Err`. A separate "why did it refuse" routine would be
/// the one-fact-two-implementations drift `docs/GAPS.md` §6 keeps recording, and
/// here it would be worse than usual: the refusal reason is exactly what the
/// next widening rung is priced in, so a stale copy would price the wrong
/// clause.
pub fn splice_callee_why<'a>(
    f: &IlFunction,
    selected: &Selected,
    tu: &TuContext<'a>,
) -> Result<&'a str, &'static str> {
    // **S9.** Mechanism E answers first and keeps its `blr`. Asked before
    // anything else so that a reader meets the precedence at the top, and so
    // that the two rules can never both claim one function.
    if drops_tail_call(f, tu.empty_callees()) {
        return Err("S9-mechanism-e");
    }
    // **S8.** The caller's whole body is discarded; a data symbol of its own
    // would be discarded with it and no cell grades that.
    if f.data_sym.is_some() {
        return Err("S8-caller-data-sym");
    }
    // **S1 and S3.** `Framed` is 0 of 123 and `CondPair` is a conditional site
    // that was never graded; `Plain` and `Float` name no callee at all. What
    // survives is the two shapes whose whole emitted body is the call — which is
    // S3, and which is why the two clauses are one match.
    let callee: &str = match selected {
        // A tail call with a non-empty setup is SPLICE-P's `port_words > 1`
        // stratum: **0 of 953**, with 1,890 of them diverging at word 0.
        Selected::Tail(setup) if setup.is_empty() => match f.tail_call.as_deref() {
            Some(c) => c,
            None => return Err("S1-tail-without-callee"),
        },
        Selected::Tail(_) => return Err("S3-tail-setup"),
        Selected::Seq { setups, .. } => {
            let Some(seq) = f.call_seq.as_ref() else {
                return Err("S1-seq-without-call-seq");
            };
            // **S2.** One call site. SPLICE-N is 0 of 548.
            if seq.calls.len() != 1 || setups.len() != 1 {
                return Err("S2-multi-call");
            }
            // **S3.** Nothing around the call: no argument setup, no guarded
            // block (which is also §2's *conditional site*), no early return.
            if seq.guard.is_some() || !seq.early.is_empty() {
                return Err("S3-seq-guard-or-early");
            }
            // **S3 for a `Seq` reads the IL's argument mapping, NOT the emitted
            // setup bytes — and the two disagree on 816 of 816 workload cells.**
            //
            // The clause this lane registered required `setups[0]` to be empty,
            // generalizing the `tail` split (578/578 with no setup, 0/953 with
            // one). It fired on **zero** `seq` bodies, and the refusal census
            // said why: **816 of the 816** single-call `seq` differs are
            // `S3-seq-setup-frame-only` and **none** is `S3-seq-setup-args`.
            //
            // A `Seq` body is FRAMED, so `setups[0]` carries the frame's own
            // bookkeeping — the `mr r31,r3` that saves `this` across the `bl` —
            // as well as any real argument marshalling. c2's inlined body has no
            // frame at all (`w-seq` §4.4: the port emits nine words opening
            // `mflr r12` where c2 emits three and no frame), so a setup that is
            // *only* the save is an artefact of the port's own lowering, while a
            // setup that marshals a value is exactly the register rename or
            // displacement fold that makes SPLICE-0 fail on `tail`.
            //
            // The judge has already graded the widened population: SPLICE-0 is
            // exact on **816 of 816** single-callee `seq` differs (`w-seq`
            // §4.1), which is real c2 on 816 cells rather than a hand cell.
            if !identity_call_args(f, &seq.calls[0]) {
                return Err("S3-seq-setup-args");
            }
            // **S3, the tail half.** The body may do nothing to r3 after the
            // call, because c2's inlined body is the callee's and carries no
            // epilogue of the caller's. Two tails qualify and both are the ABI
            // identity:
            //
            // * `CallValue { add_k: 0 }` — return what the call left in r3,
            //   with the `addi` elided at 0. Class A, so nothing is saved.
            // * `SavedFormal { param: 0 }` — the WEC shape, an empty
            //   constructor delegating to one base: `mr r31,r3 ; bl ?B ;
            //   mr r3,r31`. The formal handed back is the one that was already
            //   in r3 at entry, so with an identity argument mapping the whole
            //   body IS the call. **This is the 634-function `seq` family**, and
            //   its witness is a workload obj rather than a hand cell —
            //   `??0?$_List_iterator@VString@@…` in `CharDriver.cpp` emits
            //   `stw r4,0(r3) ; blr`, word for word its base constructor's
            //   body, where the port's own lowering is three words inside a
            //   96-byte frame (`work/w-splice/caps/chardriver.excerpt.txt`).
            //   GRID-T's `t03` was meant to be the hand cell and is a **dud**:
            //   its base constructor is declared and never defined, so it has no
            //   `Seq` body to grade. Said rather than smoothed.
            //
            // Every other tail — a literal, a read-through, a comparison —
            // emits words of its own after the `bl`, and c2's inlined body would
            // have to carry them too. Not graded, so refused.
            match seq.tail {
                SeqTail::CallValue { add_k: 0 } if seq.saved.is_empty() => {}
                SeqTail::SavedFormal { param: 0 } if seq.saved.as_slice() == [0] => {}
                SeqTail::CallValue { .. } => return Err("S3-seq-tail-callvalue-k"),
                SeqTail::SavedFormal { .. } => return Err("S3-seq-tail-savedformal-other"),
                SeqTail::Void => return Err("S3-seq-tail-void"),
                SeqTail::Lit(_) => return Err("S3-seq-tail-lit"),
                SeqTail::CallLoad { .. } => return Err("S3-seq-tail-callload"),
                SeqTail::CallLoadFp { .. } => return Err("S3-seq-tail-callloadfp"),
                SeqTail::Cmp { .. } => return Err("S3-seq-tail-cmp"),
            }
            seq.calls[0].callee.as_str()
        }
        Selected::Framed { .. } => return Err("S1-framed"),
        Selected::CondPair(_) => return Err("S1-cond-pair"),
        Selected::Plain(_) | Selected::Float { .. } => return Err("S1-no-call"),
    };
    // **S4.** Self-recursion. c2 declines it too — `INLINE_PREDICATE.md` §4
    // grades the `recurse` family 336/336 — and a rule that took it would
    // splice a body into itself.
    if callee == f.mangled_name {
        return Err("S4-self-recursion");
    }
    // **S5.** Defined here, once. Returned as the context's own `&'a str` so the
    // spliced body's relocations outlive the caller's borrow.
    //
    // The two ways this fails are two different rungs and are named apart: an
    // **external** callee is nobody's mechanism, and a callee this TU defines
    // whose IL the parser refuses is `w-seq` §5's production table — 1,774
    // differs deep, and the largest single production blocks 573 of them.
    if !tu.mentions(callee) {
        return Err("S5-callee-extern");
    }
    if tu.ambiguous(callee) {
        return Err("S5-callee-ambiguous");
    }
    if tu.definition(callee).is_none() {
        return Err("S6-callee-parse-refused");
    }
    self_named(tu, callee).ok_or("S5-callee-extern")
}

/// **Every callee this body names**, from the decoded fields and never from a
/// byte offset (#644).
///
/// Used by `S6-chain-truncated` to ask the one question that separates a chain
/// that *ended* from one the port could not follow: does the chain's last link
/// still name a function this TU defines?
///
/// Deliberately a local copy of the shape `crates/c2-harness/src/gap/fnbytes.rs`
/// walks for its forensics rather than a shared one: that one is a diagnostic
/// over a *census row* and this is a clause of an emitter's predicate, and the
/// emitter must not depend on the instrument that grades it.
fn port_callees(f: &IlFunction) -> Vec<&str> {
    let mut v: Vec<&str> = Vec::new();
    if let Some(c) = f.tail_call.as_deref() {
        v.push(c);
    }
    if let Some(fc) = f.framed_call.as_ref() {
        v.push(fc.callee.as_str());
    }
    if let Some(cs) = f.call_seq.as_ref() {
        for c in &cs.calls {
            v.push(c.callee.as_str());
        }
    }
    if let Some(cp) = f.cond_pair.as_ref() {
        v.push(cp.then_arm.callee.as_str());
        v.push(cp.else_arm.callee.as_str());
    }
    v
}

/// **Is this call's argument mapping the IDENTITY** — every argument register
/// already holding, at the call, what it held at function entry?
///
/// Read off the IL and never off the emitted setup bytes, because a `Seq` body's
/// setup also carries the frame's callee-saved bookkeeping, which is the port's
/// lowering and not a transform of the callee's arguments.
///
/// Identity is: no chain link (its receiver is a previous call's result), no
/// slot permutation, and an operand stream that is either empty (a nullary call,
/// or one whose only argument is the implicit `this` already in r3) or a single
/// passthrough `Load` of the **first** formal.
fn identity_call_args(f: &IlFunction, call: &c2_il::SeqCall) -> bool {
    if call.link_args.is_some() || call.arg_sources.is_some() {
        return false;
    }
    match call.arg_ops.as_slice() {
        [] => true,
        [c2_il::IlOp::Load(t)] => f.params.first() == Some(t),
        _ => false,
    }
}

/// The context's own copy of `name`, with the context's lifetime.
///
/// [`TuContext::definition`] hands back the definition; the *name* has to come
/// from the same place, because a spliced [`ComdatBody`] carries `&str`
/// relocation targets that outlive the caller's borrow of its own callee field.
fn self_named<'a>(tu: &TuContext<'a>, name: &str) -> Option<&'a str> {
    let i = tu.rows.binary_search_by(|(n, _, _)| (*n).cmp(name)).ok()?;
    Some(tu.rows[i].0)
}

/// **THE MECHANISM.** The caller's `/Gy` COMDAT body, when it is the callee's.
///
/// `Ok(None)` is the refusal — the predicate declined, and the caller emits its
/// ordinary body. `Err` is the callee's own decline propagated: it cannot happen
/// through S6, which treats an uncomposable callee as a refusal, and the arm
/// exists so that a future edit which stops doing that goes red rather than
/// silent.
///
/// **The chain is walked, not stepped once.** `splice_body_why` follows it to
/// the first link whose own predicate declines, and takes *that* body — c2
/// closes the chain and `t11` plus 150 workload relocation witnesses say so
/// ("THE FIXPOINT" above). The composition of that final body is asked with
/// `allow_splice: false`, because the walk has already established that this
/// link does not splice and asking again through the composition would be the
/// same question with a second implementation.
pub fn splice_body<'a>(
    f: &IlFunction,
    selected: &Selected,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> Result<Option<ComdatBody<'a>>, ComdatDecline> {
    match splice_body_why(f, selected, mode, tu) {
        Ok(b) => Ok(Some(b)),
        Err(SpliceDecline::Refused(_)) => Ok(None),
        Err(SpliceDecline::Callee(d)) => Err(d),
    }
}

/// Why the mechanism did not fire, or the callee's own decline propagated.
///
/// [`SpliceDecline::Callee`] cannot happen through S6, which treats an
/// uncomposable callee as a refusal; the variant exists so that a future edit
/// which stops doing that goes red rather than silent.
#[derive(Debug)]
pub enum SpliceDecline {
    /// A clause of the predicate declined, named by its clause number.
    Refused(&'static str),
    /// The callee's own composition failed.
    Callee(ComdatDecline),
}

/// [`splice_body`] with **the clause that refused**, for the same reason
/// [`splice_callee_why`] exists: the refusal reason is what prices the next
/// widening, so there is one implementation of it and not two.
pub fn splice_body_why<'a>(
    f: &IlFunction,
    selected: &Selected,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> Result<ComdatBody<'a>, SpliceDecline> {
    let mut callee = splice_callee_why(f, selected, tu).map_err(SpliceDecline::Refused)?;
    // **THE CHAIN.** `S6-chain` below walks it; `seen` is what makes a cycle
    // terminate, and the ceiling is what makes a broken edit terminate too.
    let mut seen: Vec<&str> = vec![callee];
    let ceiling = tu.definitions() + 1;
    loop {
        // **S7, the varargs half.** `N_max = 0` categorically (§6.18.5). MSVC
        // terminates a varargs argument list with `Z` where an ordinary one ends
        // `@`, so the mangled name ends `ZZ` — read off the name because that is
        // where §2's table says it is readable, IL side and obj side alike.
        if callee.ends_with("ZZ") {
            return Err(SpliceDecline::Refused("S7-varargs"));
        }
        let Some((g, opt_word)) = tu.definition(callee) else {
            return Err(SpliceDecline::Refused("S5-callee-extern"));
        };
        // The callee's own mode. A callee under a different `#pragma optimize`
        // than its caller allocates a chain intermediate to a different register
        // (`OptMode`'s doc), so splicing across that boundary would emit the
        // wrong register field. `None` is "the caller does not track it per
        // function", and then the TU has already been refused unless every
        // function shares a mode.
        let g_mode = match opt_word {
            Some(_) => match opt_mode_of_word(opt_word) {
                Ok(m) => m,
                Err(_) => return Err(SpliceDecline::Refused("S6-callee-opt-mode")),
            },
            None => mode,
        };
        if g_mode != mode {
            return Err(SpliceDecline::Refused("S6-mode-mismatch"));
        }
        // **S6.** The port must have a body for the callee — `t09` is the cell
        // where it does not.
        let Ok(g_sel) = crate::codegen::select_function(g, g_mode) else {
            return Err(SpliceDecline::Refused("S6-callee-refused"));
        };
        // **S6-CHAIN — THE FIXPOINT, and it is measured rather than assumed.**
        //
        // If the callee ITSELF splices, its own emitted COMDAT is not the branch
        // the port would lower it to: it is *its* callee's body. So the caller
        // must take that one, and the walk steps down.
        //
        // `work/w-splice/PREREG.md` §4 registered this as a QUESTION with the
        // port taking one level either way. Both answers came back and they
        // agree:
        //
        // * `t11` — `int h(int a){return a+1;} int g(int a){return h(a);}
        //   int f(int a){return g(a);}` — c2 emits **`?h`'s two words for all
        //   three**, so `splice0(?f)` against `ref(?g)` grades `exact`;
        // * the workload, through the relocation check the one-level rule
        //   forced: **150 of 945** spliced functions named the chain's
        //   *intermediate* where c2 named its *end* —
        //   `??1length_error@stlpmtx_std@@` relocating against
        //   `??1__Named_exception@stlpmtx_std@@` where c2 relocates against
        //   `??1exception@std@@`, 145 times in that shape. Every one of the 150
        //   was this and none was a different target.
        //
        // The one-level rule therefore could not ship: `PREREG.md` §3 item 4
        // makes a relocation disagreement a decline-floor failure, and 150 of
        // them is not a rounding.
        if let Ok(next) = splice_callee_why(g, &g_sel, tu) {
            if seen.contains(&next) {
                // A cycle. `elide.rs`'s least fixpoint never *seeds* one and so
                // never admits one; this walk reaches the same answer by
                // construction, and c2 declines a recursive callee too
                // (`INLINE_PREDICATE.md` §4, `recurse` 336/336).
                return Err(SpliceDecline::Refused("S6-chain-cycle"));
            }
            if seen.len() > ceiling {
                // Unreachable while `seen` is checked above — a step either
                // repeats a name or admits a new one, and the bundle has
                // finitely many. Here so that an edit which breaks that
                // argument REFUSES instead of walking forever.
                return Err(SpliceDecline::Refused("S6-chain-ceiling"));
            }
            seen.push(next);
            callee = next;
            continue;
        }
        // **S6-CHAIN-TRUNCATED — the chain must END, not be CUT OFF.**
        //
        // The walk stops at the first link whose own predicate declines. There
        // are two completely different reasons that happens and only one of them
        // is an ending:
        //
        // * the link names **no same-TU callee** — an external, or nothing at
        //   all. c2 has no body to expand either, so this is where c2 stops too
        //   and the body is the answer;
        // * the link names a callee this TU **defines** and the port cannot
        //   follow — its IL is parse-refused, it has an argument setup, it is a
        //   multi-call body. c2 is under no such restriction, and **it does not
        //   stop there**: measured, 72 spliced functions relocated against
        //   `??1?$_List_base@…` where c2 relocates against
        //   `?clear@?$_List_base@…`, which is one link further down a chain the
        //   port cannot read. Every one of the 77 residual disagreements after
        //   the fixpoint landed was this.
        //
        // So a truncated chain **refuses**. The port does not know where c2
        // stopped, and a relocation against the wrong symbol is invisible to
        // FUNCTION BYTE MATCH — board **#882** — which makes guessing here the
        // one thing this lane must not do.
        if port_callees(g).into_iter().any(|c| tu.mentions(c)) {
            return Err(SpliceDecline::Refused("S6-chain-truncated"));
        }
        // The chain's end: this callee emits its own body, so that body is the
        // caller's. Composed with `allow_splice: false` — the walk above has
        // already established that this link does not splice, and asking again
        // through the composition would be the same question with a second
        // implementation.
        let Ok(body) = crate::comdat::body_of(g, g_sel, g_mode, tu, false) else {
            return Err(SpliceDecline::Refused("S6-callee-no-compose"));
        };
        // **S6, the frame half.** A framed callee carries a prologue, an
        // epilogue and a `.pdata` record whose association is a property of the
        // function it belongs to. Splicing one into a caller is a cell nobody
        // graded.
        if body.frame.is_some() {
            return Err(SpliceDecline::Refused("S6-callee-framed"));
        }
        // **S6-CHAIN-OPEN — the chain's end may carry NO CALL AT ALL, and this
        // clause is a measured retreat.**
        //
        // `S6-chain-truncated` above asks whether the end still names a callee
        // *this TU's census carries*. One workload function got past it:
        // `??1CharPollableSorter@@QAA@XZ` spliced to a body whose branch names
        // `??1?$_Rb_tree@PAVObject@Hmx@@…`, and c2 relocates against
        // `?clear@?$_Rb_tree@…` one link further down. That callee has **no
        // census row in its TU at all** — `c2rs census` on `Character.cpp`
        // prints zero rows matching it — so it sits in the `unbound` population
        // (9,225 rows of this workload) and the port cannot tell it from a
        // genuine external. c2 can: it inlined it, which is proof it is defined
        // in that obj.
        //
        // So a chain that ends *in a call* is refused. It costs **4** spliced
        // functions of 727 — **3 whose relocation the check verified CORRECT**
        // (`t10`'s shape: the callee really does call an external, and the port
        // really does name it) and **1 verified wrong**. Keeping the three would
        // mean keeping the one, because nothing the port can read separates
        // them, and a wrong relocation under byte-exact text is exactly board
        // **#882**'s 4,664.
        //
        // The rung that recovers the three is named and not taken: give the port
        // a defined-name set that does not come from the census — the `.gl`
        // knows which names this TU defines — and then re-grade the one.
        if !body.calls.is_empty() {
            return Err(SpliceDecline::Refused("S6-chain-open"));
        }
        // **S7.** The inline decision, on the side of its own boundary where it
        // is categorical in both linkage classes.
        //
        // **One check covers every link of the chain.** `INLINE-P`'s `s` is the
        // callee's own *emitted* size, and an intermediate that splices emits
        // exactly this body — c2's COMDAT for it *is* the chain's end body — so
        // `s` is the same number at every edge and testing it once tests it
        // everywhere.
        if body.text.is_empty() {
            return Err(SpliceDecline::Refused("S7-callee-empty-text"));
        }
        if body.text.len() > INLINE_UNBOUNDED_BYTES {
            return Err(SpliceDecline::Refused("S7-callee-over-64B"));
        }
        return Ok(body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::select_function;
    use crate::codegen::testutil::func_with;
    use c2_il::{CallSeq, IlOp, SeqCall};

    /// `int g(int a) { return a + 1; }` — a leaf the port lowers, 8 bytes.
    fn leaf(name: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add]);
        f.mangled_name = name.into();
        f.data_sym = None;
        f
    }

    /// `int f(int a) { return g(a); }` — an empty setup and one branch word.
    fn tail(name: &str, callee: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], Vec::new());
        f.mangled_name = name.into();
        f.data_sym = None;
        f.tail_call = Some(callee.into());
        f
    }

    fn ctx<'a>(funcs: &'a [IlFunction]) -> TuContext<'a> {
        TuContext::of(funcs)
    }

    fn fires(funcs: &[IlFunction], i: usize) -> bool {
        let tu = ctx(funcs);
        let sel = select_function(&funcs[i], OptMode::O1).unwrap();
        splice_callee(&funcs[i], &sel, &tu).is_some()
    }

    fn spliced(funcs: &[IlFunction], i: usize) -> Option<Vec<u8>> {
        let tu = ctx(funcs);
        let sel = select_function(&funcs[i], OptMode::O1).unwrap();
        splice_body(&funcs[i], &sel, OptMode::O1, &tu)
            .unwrap()
            .map(|b| b.text)
    }

    /// `t01` — the positive cell. The caller's body **is** the callee's, and the
    /// caller acquires no relocation of its own.
    #[test]
    fn an_empty_setup_tail_call_takes_the_callees_body() {
        let funcs = vec![leaf("?g@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        let body = splice_body(&funcs[1], &sel, OptMode::O1, &tu)
            .unwrap()
            .expect("S1-S9 hold: this is t01");
        let g_sel = select_function(&funcs[0], OptMode::O1).unwrap();
        let g_body = crate::comdat::comdat_body_from_selected(&funcs[0], g_sel, OptMode::O1, &tu)
            .unwrap();
        assert_eq!(body.text, g_body.text, "the caller's body is the callee's");
        assert!(
            body.calls.is_empty(),
            "THE CALLER ACQUIRED A RELOCATION AGAINST ITS CALLEE: the whole \
             mechanism is that it does not — board #882, and a leaf callee has \
             no relocation of its own to inherit"
        );
        assert!(body.frame.is_none());
    }

    /// **S3** — `t04`/`t05`/`t06`. A setup means c2 rewrote a field of the
    /// callee's body (a register rename, a displacement fold), and w-seq graded
    /// the concatenation **0 of 953** there.
    #[test]
    fn a_tail_call_with_an_argument_setup_is_refused() {
        // `int f(int a) { return g(a + 1); }` — the setup is one `addi`.
        let mut caller = tail("?f@@YAHH@Z", "?g@@YAHH@Z");
        caller.ops = vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add];
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        assert!(
            !fires(&funcs, 1),
            "SPLICE-P IS 0 OF 953 WITH A SETUP: 1,890 of those failures diverge \
             at word 0, which is the port's own first setup word"
        );
    }

    /// **S5** — `t14`, the control. A callee this bundle does not define is the
    /// ordinary tail call, relocation and all.
    #[test]
    fn a_callee_this_bundle_does_not_define_is_refused() {
        let funcs = vec![tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 0));
        // …and a definition of a DIFFERENT name must not admit it either.
        let funcs = vec![leaf("?other@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 1));
    }

    /// **S5, the ambiguity half.** Two definitions of one name admit neither:
    /// telling them apart needs something this context does not have.
    #[test]
    fn a_name_defined_twice_is_refused() {
        let mut g2 = leaf("?g@@YAHH@Z");
        g2.ops = vec![IlOp::Load(0xE309), IlOp::Lit(2), IlOp::Add];
        let funcs = vec![leaf("?g@@YAHH@Z"), g2, tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 2));
    }

    /// **S4** — `t12`. A body spliced into itself is not a fixpoint, it is a
    /// loop; c2 declines self-recursion too (`recurse`, 336/336).
    #[test]
    fn self_recursion_is_refused() {
        let funcs = vec![tail("?r@@YAHH@Z", "?r@@YAHH@Z")];
        assert!(!fires(&funcs, 0));
    }

    /// **S9** — mechanism E answers first. An empty-bodied callee is `elide`'s
    /// and this rule must not claim it, or one function has two rules.
    #[test]
    fn mechanism_e_keeps_its_own_callers() {
        let mut g = leaf("?g@@YAXXZ");
        g.ops = Vec::new();
        g.params = Vec::new();
        g.empty_body = true;
        let mut caller = tail("?f@@YAXXZ", "?g@@YAXXZ");
        caller.params = Vec::new();
        let funcs = vec![g, caller];
        let tu = ctx(&funcs);
        assert!(
            tu.reduces_to_nothing("?g@@YAXXZ"),
            "the E context still resolves through the composite"
        );
        assert!(
            !fires(&funcs, 1),
            "MECHANISM E AND MECHANISM I BOTH CLAIMED ONE FUNCTION: E is asked \
             first and keeps its blr"
        );
    }

    /// **S8** — a caller that materializes a data symbol. Its whole body is
    /// discarded and the data reference would go with it.
    #[test]
    fn a_caller_with_a_data_symbol_is_refused() {
        let mut caller = tail("?f@@YAHH@Z", "?g@@YAHH@Z");
        caller.data_sym = Some("?gv@@3HA".into());
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        assert!(!fires(&funcs, 1));
    }

    /// **S7, the varargs half.** `N_max = 0` categorically, and the mangled name
    /// is where §2's table says the bit is readable.
    #[test]
    fn a_varargs_callee_is_refused() {
        let mut g = leaf("?g@@YAHHZZ");
        g.mangled_name = "?g@@YAHHZZ".into();
        let funcs = vec![g, tail("?f@@YAHH@Z", "?g@@YAHHZZ")];
        // S1-S5 hold — the refusal is S7's and only S7's.
        assert!(fires(&funcs, 1), "the structural half of the predicate holds");
        assert!(
            spliced(&funcs, 1).is_none(),
            "A VARARGS CALLEE WAS SPLICED: N_max is 0 for it in both linkage \
             classes (INLINE_PREDICATE.md §2, §6.18.5)"
        );
    }

    /// **S6** — a callee the port cannot lower has no body to splice, so the
    /// caller keeps its branch. The structural half still holds, which is why
    /// the two halves are separate functions.
    #[test]
    fn an_unlowerable_callee_leaves_the_branch_alone() {
        let mut g = leaf("?g@@YAHH@Z");
        g.ops = vec![IlOp::Load(0xE309), IlOp::Load(0xE409), IlOp::Div];
        let funcs = vec![g, tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(fires(&funcs, 1));
        assert!(
            spliced(&funcs, 1).is_none(),
            "the port has no bytes for this callee; splicing would emit a body \
             it cannot produce"
        );
    }

    /// **BOARD #980's BOUNDARY — a `NoEffectCall` row is E's and NEVER the
    /// splice's.**
    ///
    /// Lane `w-inl0` feeds mechanism E edges from rows the IL parser **refused**:
    /// a refused body whose grammar still proves it emits nothing but a call to
    /// one callee contributes `Reduction::NoEffectCall`. E can close through
    /// such a node because closing needs only *"does this emit anything"*.
    ///
    /// The splice cannot, and the reason is S6 rather than a policy: its rule is
    /// *"the caller's body IS the callee's body"*, and there is no body — the
    /// parser refused it. So [`TuContext::definition`] returns `None` for those
    /// rows and the walk refuses with `S6-callee-parse-refused`.
    ///
    /// **But the row must still be VISIBLE.** `?g` is a name this TU defines,
    /// and `mentions` says so, which is what `S6-chain-truncated` reads to tell
    /// a chain that *ended* from one the port could not *follow*. A resolution
    /// that dropped refused rows from the context would make `?g` read as an
    /// external — and running a splice off the end of a chain the port cannot
    /// see is the wrong-relocation defect this lane already closed once.
    #[test]
    fn a_no_effect_call_row_feeds_e_and_never_the_splice() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let mut caller = tail("?f@@YAXXZ", "?g@@YAXXZ");
        caller.params = Vec::new();
        // `?g` is REFUSED by the parser — there is no `IlFunction` for it at
        // all — and carries only the edge board #980 reads out of its grammar.
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", Some(Reduction::NoEffectCall("?h@@YAXXZ")), None),
            ("?f@@YAXXZ", Some(Reduction::Parsed(&caller)), None),
        ]);

        // E closes through the refused node — that is #980's whole content.
        assert!(
            tu.reduces_to_nothing("?g@@YAXXZ"),
            "BOARD #980 REGRESSED: mechanism E must close through a refused row \
             that carries a NoEffectCall edge"
        );

        // The splice must not be able to reach it as a body...
        assert!(
            tu.definition("?g@@YAXXZ").is_none(),
            "A REFUSED BODY WAS OFFERED TO THE SPLICE: S6 needs a COMPOSED body \
             for the chain's end and the parser refused this one"
        );
        // ...and must still be CARRIED by the context. `mentions()` is what
        // reads this row, and it arrives with `S6-chain-truncated`; what this
        // commit can assert is that the row is in the table at all, which is
        // the property that clause will depend on.
        assert!(
            tu.mentions("?g@@YAXXZ"),
            "A REFUSED ROW VANISHED FROM THE CONTEXT: it reads as an external, \
             S6-chain-truncated stops firing, and the splice runs off the end \
             of a chain it cannot see"
        );
        assert_eq!(
            tu.definitions(),
            3,
            "every row this TU binds stays in the table, parsed or not"
        );

        // And the two mechanisms still do not both claim `?f`: E takes it.
        let sel = select_function(&caller, OptMode::O1).unwrap();
        assert!(
            crate::elide::drops_tail_call(&caller, tu.empty_callees()),
            "?f tail-calls a name that reduces to nothing, so E answers"
        );
        assert!(
            splice_callee(&caller, &sel, &tu).is_none(),
            "S9: mechanism E is asked first and keeps its blr"
        );
    }

    /// A refused row with **no** readable `no_effect_callee` feeds neither
    /// mechanism and is still visible. That is the third value of the
    /// constructor's `Option<Reduction>`, and it is the majority of the refused
    /// population.
    #[test]
    fn a_refused_row_with_no_edge_is_visible_and_nothing_else() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", None, None),
        ]);
        assert!(tu.mentions("?g@@YAXXZ"), "still a name this TU defines");
        assert_eq!(tu.definitions(), 2, "and still a row in the table");
        assert!(tu.definition("?g@@YAXXZ").is_none());
        assert!(
            !tu.reduces_to_nothing("?g@@YAXXZ"),
            "#980 IS CONSERVATIVE: a refused row with nothing readable \
             contributes NO edge to the closure"
        );
    }

    /// **THE `Deref` SHADOWING TRAP, pinned.**
    ///
    /// `TuContext` derefs to [`TuEmptyCallees`] so existing callers keep
    /// working, and an inherent method on the wrapper therefore **silently
    /// overrides** the target's. While this type spelled its row count `len`,
    /// the scan's `fnbyte-tu-empty-callees` reported the wrong quantity —
    /// 88,894 against 1,474,755 on the dc3 workload — with no compile error and
    /// no test failure. It also made one of the two tests above pass by
    /// coincidence, because that cell's E context happens to have as many
    /// members as the table has rows.
    ///
    /// So the two counts are asserted to be **different** on a bundle where
    /// they must be, which is a thing no rename can quietly undo.
    #[test]
    fn the_row_count_and_the_e_context_are_not_the_same_number() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let g = leaf("?g@@YAHH@Z"); // parses, non-empty: a row, never in E
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAHH@Z", Some(Reduction::Parsed(&g)), None),
            ("?x@@YAXXZ", None, None), // defined here, parser refused it
        ]);
        assert_eq!(tu.definitions(), 3, "three rows this TU binds");
        assert_eq!(tu.empty_callees().len(), 1, "only ?h reduces to nothing");
        assert_ne!(
            tu.definitions(),
            tu.empty_callees().len(),
            "A ROW COUNT WAS READ AS THE E-CONTEXT SIZE (or the reverse): these \
             are different facts, `TuContext` Derefs to `TuEmptyCallees`, and an \
             inherent `len` here would shadow the target's and move an existing \
             scan key by 16x without failing anything"
        );
    }

    /// **S2** — `t08`. Two call sites is SPLICE-N, **0 of 548**.
    #[test]
    fn a_two_call_body_is_refused() {
        let mut caller = func_with(Vec::new(), Vec::new());
        caller.mangled_name = "?f@@YAXXZ".into();
        caller.data_sym = None;
        caller.call_seq = Some(CallSeq {
            calls: vec![
                SeqCall {
                    callee: "?g@@YAXXZ".into(),
                    arg_ops: Vec::new(),
                    arg_sources: None,
                    link_args: None,
                },
                SeqCall {
                    callee: "?g@@YAXXZ".into(),
                    arg_ops: Vec::new(),
                    arg_sources: None,
                    link_args: None,
                },
            ],
            tail: SeqTail::Void,
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
        });
        let mut g = leaf("?g@@YAXXZ");
        g.params = Vec::new();
        g.ops = Vec::new();
        g.empty_body = false;
        g.compare = None;
        let funcs = vec![g, caller];
        let tu = ctx(&funcs);
        if let Ok(sel) = select_function(&funcs[1], OptMode::O1) {
            assert!(
                splice_callee(&funcs[1], &sel, &tu).is_none(),
                "SPLICE-N IS 0 OF 548: a two-call body is not the concatenation \
                 of its callees and it is not either of them"
            );
        }
    }

    /// **S3, the `Seq` tail half.** A `Seq` whose tail does work of its own
    /// after the `bl` is refused: c2's inlined body would have to carry those
    /// words too, and no cell graded that.
    #[test]
    fn a_seq_with_a_working_tail_is_refused() {
        let mut caller = func_with(vec![0xE309], Vec::new());
        caller.mangled_name = "?f@@YAHH@Z".into();
        caller.data_sym = None;
        caller.call_seq = Some(CallSeq {
            calls: vec![SeqCall {
                callee: "?g@@YAHH@Z".into(),
                arg_ops: Vec::new(),
                arg_sources: None,
                link_args: None,
            }],
            // `return g(a) + 5;` — one `addi` after the call.
            tail: SeqTail::CallValue { add_k: 5 },
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
        });
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        let tu = ctx(&funcs);
        if let Ok(sel) = select_function(&funcs[1], OptMode::O1) {
            assert!(splice_callee(&funcs[1], &sel, &tu).is_none());
        }
    }

    /// **THE FIXPOINT** — `t11`, and the 150 relocation witnesses that forced
    /// it. `h` is lowerable, `g` splices `h`, and `f` must get **`h`'s body**
    /// and not `g`'s one branch word.
    ///
    /// c2 closes the chain: `t11` compiles this exact source and c2 emits `?h`'s
    /// two words for all three functions. The one-level rule that shipped first
    /// named the intermediate in **150 of 945** spliced functions' relocations
    /// where c2 named the end.
    #[test]
    fn the_splice_closes_the_chain() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g@@YAHH@Z"),
        ];
        let h = spliced(&funcs, 1).expect("?g splices ?h");
        let f = spliced(&funcs, 2).expect("?f splices through ?g to ?h");
        assert_eq!(h.len(), 8, "?h is `addi r3,r3,1 ; blr`");
        assert_eq!(
            f, h,
            "THE CHAIN WAS NOT CLOSED: ?f must get ?h's BODY, not ?g's branch \
             word. GRID-T t11 grades c2 emitting ?h's two words for all three, \
             and the workload's relocation check found the one-level rule \
             naming the intermediate 150 times"
        );
    }

    /// A chain of four, and every member below the top gets the same body. The
    /// depth is not special-cased anywhere, which is the property this pins.
    #[test]
    fn the_chain_closes_at_every_depth() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g3@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?g2@@YAHH@Z", "?g3@@YAHH@Z"),
            tail("?g1@@YAHH@Z", "?g2@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g1@@YAHH@Z"),
        ];
        let want = spliced(&funcs, 1).expect("?g3 splices ?h");
        for i in 2..funcs.len() {
            assert_eq!(
                spliced(&funcs, i).as_ref(),
                Some(&want),
                "{} did not reach the chain's end",
                funcs[i].mangled_name
            );
        }
    }

    /// **A CYCLE TERMINATES AND REFUSES.** `?a` splices `?b` splices `?a`: the
    /// walk repeats a name, and a repeated name is the refusal rather than a
    /// deeper step. `elide.rs`'s least fixpoint never seeds a cycle and never
    /// admits one; this reaches the same answer from the other direction.
    #[test]
    fn a_chain_cycle_terminates_and_refuses() {
        let funcs = vec![
            tail("?a@@YAHH@Z", "?b@@YAHH@Z"),
            tail("?b@@YAHH@Z", "?a@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?a@@YAHH@Z"),
        ];
        for i in 0..funcs.len() {
            assert!(
                spliced(&funcs, i).is_none(),
                "A CYCLE WAS SPLICED: the walk must refuse, not recurse — {}",
                funcs[i].mangled_name
            );
        }
    }

    /// **S6-CHAIN-OPEN** — a chain whose end still calls something is refused,
    /// and this is the cell that used to be `t10`'s positive.
    ///
    /// `void ext(); void g(){ext();} void f(){g();}` — c2 emits `b ?ext` for
    /// `?f` and the port would emit `b ?ext` too, which is RIGHT. It is refused
    /// anyway: on the workload the identical shape produced 3 verified-correct
    /// relocations and **1 verified wrong** one, and nothing the port can read
    /// separates them — the wrong one's next link has no census row in its TU,
    /// so it reads as an external and is not. Board #882 is a wrong relocation
    /// under byte-exact text, and 3 right is not worth 1 wrong here.
    #[test]
    fn a_chain_that_ends_in_a_call_is_refused() {
        let funcs = vec![
            tail("?g@@YAXXZ", "?ext@@YAXXZ"),
            tail("?f@@YAXXZ", "?g@@YAXXZ"),
        ];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        // The structural half holds — the refusal is S6-chain-open's alone.
        assert!(splice_callee(&funcs[1], &sel, &tu).is_some());
        assert!(
            splice_body(&funcs[1], &sel, OptMode::O1, &tu)
                .unwrap()
                .is_none(),
            "A CHAIN ENDING IN A CALL WAS SPLICED: the port cannot tell that \
             callee from one this TU defines whose census row is unbound, and \
             the workload has one of each"
        );
    }

    /// …and the property that clause protects, stated where a test can hold it:
    /// **a spliced body never relocates against its own callee.** Every body the
    /// rule now emits carries the chain end's relocations, and the chain end has
    /// none.
    #[test]
    fn a_spliced_body_never_relocates_against_its_callee() {
        let funcs = vec![leaf("?g@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        let body = splice_body(&funcs[1], &sel, OptMode::O1, &tu)
            .unwrap()
            .expect("t01's shape");
        assert!(
            body.calls.is_empty(),
            "BOARD #882: the caller must acquire no REL24 at all here — not one \
             against ?g, which is the relocation the ordinary Tail arm emits and \
             the one c2 does not"
        );
        assert!(body.data_refs.is_empty());
    }

    /// A name defined once resolves; the ambiguity guard must not reject the
    /// *first* or *last* row of the sorted table by walking off it.
    #[test]
    fn definition_lookup_handles_the_table_edges() {
        let funcs = vec![
            leaf("?a@@YAHH@Z"),
            leaf("?m@@YAHH@Z"),
            leaf("?z@@YAHH@Z"),
        ];
        let tu = ctx(&funcs);
        for n in ["?a@@YAHH@Z", "?m@@YAHH@Z", "?z@@YAHH@Z"] {
            assert!(tu.definition(n).is_some(), "{n} must resolve");
        }
        assert!(tu.definition("?nope@@YAHH@Z").is_none());
    }

    /// An unnamed definition cannot be a callee — the same answer `elide` gives.
    #[test]
    fn an_unnamed_definition_is_never_a_splice_source() {
        let mut g = leaf("");
        g.mangled_name = String::new();
        let funcs = [g];
        let tu = TuContext::of(&funcs);
        assert_eq!(tu.definitions(), 0);
        assert!(tu.definition("").is_none());
    }
}

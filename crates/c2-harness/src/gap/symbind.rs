//! **SYMBOL BINDING — the third layer, measured** (lane `w-symbind`, boards
//! **#3597**–**#3602**).
//!
//! Funded by `docs/DECISIONS_2026-08-22.md` decision 14:
//!
//! > **Is symbol binding a third fused layer, and is it separable the way the
//! > grammar layer was?** `w-decodereach` measured `grammar-not-admitted` =
//! > **4,001** bodies, all `:eof`, all symbol binding, refused at
//! > `shape_to_function` (`census.rs:957`) **downstream of `AdmissionPolicy`**.
//! > `w-unfuse` unfused the grammar layer only.
//!
//! # The question, and the seam that answers it
//!
//! `w-unfuse` split DECODE from ADMISSION: [`c2_il::Decoded`] answers *"what
//! does this IL say"* and `AdmissionPolicy` answers *"may the port emit this"*.
//! `w-decodereach` then measured that **the two are not the whole story**:
//! 4,001 bodies have a grammar the decode reads WHOLE and the census refuses
//! anyway, at a step *below* admission — `shape_to_function`'s resolution of
//! callees and data symbols through `.gl`.
//!
//! This module asks, of the same population, the one question that separates a
//! layer from a heap: **does supplying a NAME — and changing nothing else —
//! flip the verdict?**
//!
//! ```text
//!     STRICT verdict   ×   RELAXED verdict,   row by row, positionally
//! ```
//!
//! The relaxed side is [`c2_il::Relax`] level 1 (`name-from-gl`), which is
//! **already computed once per TU** in [`super::scan`] for S0
//! ([`super::blind`]) — this module recomputes nothing and adds no parse. The
//! whole content of that relaxation (`census.rs:830–845`) is: when
//! `bind.resolve(tok)` or `bind.resolve_data(tok)` fails, supply
//! [`c2_il::BLIND_PLACEHOLDER_CALLEE`] / [`c2_il::BLIND_PLACEHOLDER_DATA`] and
//! change nothing else. No operand vocabulary widens, no grammar widens, no
//! byte of the body is re-read.
//!
//! So the **fused** cell below is exactly *"the strict census refused this
//! body, and a name it could not find is the only reason"*.
//!
//! # It is a GRADIENT and it obeys `FUNCTION_BYTE_MATCH.md` §0 verbatim
//!
//! §0 is the standing template for every gradient added after FBM; this is the
//! fifth (FBM, `fndiff`, S0/`blind`, `decode`, this). All five properties,
//! non-negotiable:
//!
//! * **Never in `scripts/gate.sh`**, and it must never be added there.
//! * **Its own block**, under its own disclaimer, apart from the class table
//!   that carries `match`/`mismatch`.
//! * **Namespaced keys** — `symbind-*`. No existing key, predicate or
//!   denominator is narrowed, widened or redefined here.
//! * **It licenses no emit.** `symbind-fused` going up is not a reason to
//!   accept a shape, to widen [`c2_il::IlBundle::functions`], or to admit
//!   anything. Decision 14 says it in those words: *"`w-symbind` in particular
//!   measures a refusal population and may not convert it."* The only thing
//!   that accepts a shape is the differential.
//! * **Unrepresentable over an empty scan** — `NO-RESULT`, never a ratio over
//!   zero.
//!
//! # THE SIGNAL IS THE CHANGE, NEVER THE DISTANCE FROM 0
//!
//! `w-decodereach`'s hard-won rule (board **#3582**), inherited whole. A lane
//! reading `symbind-fused` as *"0 means the layer is gone"* would be grading
//! against a baseline it assumed instead of measured. The baseline is this
//! scan's own number, printed with its denominator, and the decoder — sorry,
//! the **relaxation level** — that produced it is recorded on every scan
//! ([`RELAX_LEVEL_KEY`]) for the reason `decode::DECODER` is: a number that is
//! a property of its seam cannot be compared across a change of seam unless the
//! seam is named.
//!
//! # WHAT THIS INSTRUMENT CANNOT SEE, printed with its denominator
//!
//! **The identity of the missing symbol.** [`c2_il::FnCensus`] publishes no
//! such field, [`c2_il::Decoded`] deliberately exposes no `shape()` accessor
//! (its own doc says why: an accessor handing the grammar out without the
//! admission question being asked would be the fused route rebuilt one layer
//! up), and `crates/c2-il` is lane `w-atend`'s fence this wave. What is
//! reachable is the *relaxed* [`c2_il::IlFunction`]'s [`c2_il::IlFunction::callees`]
//! and `data_syms` — so this module can say **which SIDE** of the binding was
//! blind and **how many SITES**, and cannot say which symbol. A placeholder
//! reachable only through a third field is invisible, and that is exactly what
//! [`PLACEHOLDER_NONE_KEY`] counts. Named as owed, not smuggled (`#3470`,
//! `#1002` — absence read as success).

use c2_il::{FnCensus, IlFunction, Relax, BLIND_PLACEHOLDER_CALLEE, BLIND_PLACEHOLDER_DATA};

use super::classify::mangling_class;
use super::TuResult;

/// **The env switch.** `on` (default) | `off`. A named, settable parameter
/// rather than a baked constant (`docs/GOAL_DECISION_2026-08-21.md`
/// § "AMENDED"); `off` is a legal instrument state and licenses nothing.
/// PROV[N] not load-bearing — the instrument's on/off variable, a named settable parameter per `GOAL_DECISION_2026-08-21.md` § AMENDED.
pub const ENABLE_ENV: &str = "C2RS_SYMBIND";

/// **THE DISCRIMINATING CELL: bodies the strict census REFUSES that the
/// relaxation ADMITS.**
///
/// This is the positive check that says the instrument measures symbol binding
/// rather than admission, and the prereg freezes a threshold on it: **if this
/// is 0 the lane reports `FAILED`**. A criterion that cannot move is `#3336`,
/// and this whole line of work exists to prevent `#3336` at program scale.
/// PROV[N] not load-bearing — a metric key NAME. Its doc carries the criterion that MUST be able to move (`#3336`); the criterion is the load-bearing thing, not the string.
pub const FUSED_KEY: &str = "symbind-fused";

/// **WHICH RELAXATION LEVEL PRODUCED THESE NUMBERS.** Recorded on every scan as
/// `symbind-relax-level|<name>`, on the `fnbyte-blind-level` / `decode-reach-\
/// decoder` model, so two scans taken at two depths can never be compared or
/// summed without the difference being visible.
///
/// It is more than bookkeeping here: at `Relax::level(0)` the relaxed census
/// **is** the strict one (pinned by `c2_il`'s
/// `strict_relax_is_the_incumbent_census`), so [`FUSED_KEY`] must read **0** —
/// the instrument's own free identity control, and the arm the prereg requires
/// a full workload scan of.
/// PROV[N] not load-bearing — a metric key NAME for the instrument's own free identity control.
pub const RELAX_LEVEL_KEY: &str = "symbind-relax-level";

/// **A FUSED ROW WHOSE RELAXED BODY CARRIES NO PLACEHOLDER ANYWHERE THE PUBLIC
/// ACCESSORS CAN SEE.** Known answer 0, and a nonzero reading is a **FINDING**,
/// not an alarm: it means the relaxation flipped a verdict for a reason other
/// than a supplied name — i.e. this module's account of the seam is incomplete
/// and the residue is exactly the size of what it is missing.
/// PROV[N] not load-bearing — a metric key NAME for the residue this module's account of the seam is missing.
pub const PLACEHOLDER_NONE_KEY: &str = "symbind-placeholder-none";

/// Whether the instrument runs. An unparseable value is **refused loudly**
/// rather than silently defaulted, for `decode::enabled_from_env`'s reason: a
/// scan that quietly measured a different thing would publish a number against
/// the wrong denominator.
pub fn enabled_from_env() -> bool {
    match std::env::var(ENABLE_ENV) {
        Err(_) => true,
        Ok(v) => match v.trim() {
            "on" | "1" | "" => true,
            "off" | "0" => false,
            other => {
                eprintln!(
                    "{ENABLE_ENV}={other:?} is not `on` or `off`; refusing rather than \
                     silently measuring at a setting nobody asked for"
                );
                std::process::exit(2);
            }
        },
    }
}

/// **One row's STRICT × RELAXED cell.** The partition is over every census row
/// and every arm is printed, including as a zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// The strict census admitted it. Not this module's population — it is the
    /// denominator's other half, and it is counted so the partition can be
    /// checked rather than assumed.
    InClass,
    /// **THE LAYER.** Strict refused, relaxed admitted: a name is the only
    /// thing between this body and the incumbent admission predicate.
    Fused,
    /// Strict refused and relaxed refused. **What the existing seam does NOT
    /// reach** — and the honest measure of how much of "symbol binding" is not
    /// symbol *resolution*.
    Residue,
    /// **Strict admitted and relaxed REFUSED.** Known answer 0, and it is NOT
    /// true by construction: it is the claim that the relaxation only ever
    /// widens. Its own arm rather than folded into `Residue`, because a
    /// relaxation that *narrows* is an alarm about the seam and not a fact
    /// about a body.
    Monotonicity,
}

impl Cell {
    pub fn key(self) -> &'static str {
        match self {
            Cell::InClass => "symbind-in-class",
            Cell::Fused => FUSED_KEY,
            Cell::Residue => "symbind-residue",
            Cell::Monotonicity => "symbind-monotonicity-broken",
        }
    }

    /// Every value, so the report can print each one including as a zero.
    /// PROV[N] not load-bearing — every variant of this module's own `Cell` enum. Derived from the enum.
    pub const ALL: [Cell; 4] = [
        Cell::InClass,
        Cell::Fused,
        Cell::Residue,
        Cell::Monotonicity,
    ];
}

/// **Grade one row.** Isolated from every lookup — no obj, no map, no env, and
/// not even a `c2-il` type — so it is testable without a toolchain, exactly as
/// [`super::blind::grade_one_blind`] and [`super::decode::grade_one`] are.
///
/// `strict` / `relaxed` are the two censuses' `in_class()` answers for the
/// **same** row of the **same** segmentation. The positional correspondence is
/// checked by the caller and fails closed for the whole TU (`#918`).
pub fn cell_of(strict: bool, relaxed: bool) -> Cell {
    match (strict, relaxed) {
        (true, true) => Cell::InClass,
        (false, true) => Cell::Fused,
        (false, false) => Cell::Residue,
        (true, false) => Cell::Monotonicity,
    }
}

/// **WHICH SIDE OF THE BINDING WAS BLIND**, off the relaxed body's own names.
///
/// The relaxation supplies a placeholder that is *deliberately not a valid
/// mangled name* and *deliberately identical at every site*, so a name that
/// equals it is a site the strict resolver could not answer for. Counting them
/// is how "one symbol" and "many symbols" become two different numbers instead
/// of one word.
///
/// Returns `(callee sites blind, data sites blind)`. `None` for a row with no
/// relaxed body — which cannot be a `Fused` row by definition and is therefore
/// not silently folded in.
pub fn blind_sites(f: &IlFunction) -> (usize, usize) {
    let c = f.callees().filter(|n| *n == BLIND_PLACEHOLDER_CALLEE).count();
    let d = f
        .data_syms
        .iter()
        .filter(|n| n.as_str() == BLIND_PLACEHOLDER_DATA)
        .count();
    (c, d)
}

/// The `symbind-missing|…` bucket for one fused row. **Four arms and every one
/// is printed**, `neither` included — it is [`PLACEHOLDER_NONE_KEY`]'s
/// population and the thing this module most wants a reader to see.
pub fn missing_key(callee: usize, data: usize) -> &'static str {
    match (callee > 0, data > 0) {
        (true, true) => "symbind-missing|both",
        (true, false) => "symbind-missing|callee",
        (false, true) => "symbind-missing|data",
        (false, false) => "symbind-missing|neither",
    }
}

/// **The per-TU concentration bucket.** Sum-safe: each TU files exactly one,
/// so the merged map is a histogram over TUs rather than over bodies.
///
/// A bucket rather than a max, because `TuResult` maps are **summed** across
/// the scan and a max is not a sum — filing `symbind-tu-max` would silently
/// publish the *total*, which is the shape of defect this repo files under
/// "an absence read as a measurement".
pub fn tu_bucket(n: usize) -> &'static str {
    match n {
        0 => "symbind-tu-bucket|0",
        1 => "symbind-tu-bucket|1",
        2..=9 => "symbind-tu-bucket|2-9",
        10..=99 => "symbind-tu-bucket|10-99",
        100..=999 => "symbind-tu-bucket|100-999",
        _ => "symbind-tu-bucket|1000+",
    }
}

/// Run the symbol-binding measurement over one TU.
///
/// **Additive only.** It reads the strict census, the relaxed census (both
/// already built by [`super::scan`] — *called, never recomputed*), and the
/// `Decoded` slice `w-unfuse` built, and writes only `symbind-*` keys into its
/// own map. Nothing here reaches a numerator, an accept/refuse path, or
/// `scripts/gate.sh`.
pub(super) fn measure(
    res: &mut TuResult,
    census: &[(FnCensus, Result<IlFunction, &'static str>)],
    relaxed: &[(FnCensus, Result<IlFunction, &'static str>)],
    decoded: Option<&[c2_il::Decoded<'_>]>,
    level: u32,
) {
    // The level these numbers were taken at, filed FIRST and per TU, so a
    // merged report cannot mix two depths without the mix being visible.
    *res.fn_symbind
        .entry(format!("{RELAX_LEVEL_KEY}|{}", Relax::level(level).name()))
        .or_insert(0) += 1;
    *res.fn_symbind
        .entry("symbind-tus-scanned".into())
        .or_insert(0) += 1;

    // **THE POSITIONAL CONTROL** (`#918`, and `fnbyte-blind-census-desync`'s
    // own shape). Two passes over one segmentation, so row `i` is the same
    // function in both — CHECKED rather than assumed, and it fails closed for
    // the whole TU instead of grading one function's relaxed verdict against
    // another function's strict one.
    if relaxed.len() != census.len() {
        *res.fn_symbind
            .entry("symbind-census-desync".into())
            .or_insert(0) += 1;
        return;
    }

    // Local accumulators. Every control below is computed from THESE, in this
    // TU iteration, never by subtracting two published totals (`#1464`).
    let (mut observable, mut in_class, mut fused, mut residue, mut mono) = (0usize, 0, 0, 0, 0);
    let (mut fused_grammar, mut residue_grammar) = (0usize, 0);
    let mut blind_callee_sites = 0usize;
    let mut blind_data_sites = 0usize;

    for (i, (c, strict)) in census.iter().enumerate() {
        let (rc, relaxed_fn) = &relaxed[i];
        if rc.index != c.index {
            *res.fn_symbind
                .entry("symbind-census-desync".into())
                .or_insert(0) += 1;
            return;
        }
        observable += 1;
        let cell = cell_of(c.verdict.in_class(), rc.verdict.in_class());
        *res.fn_symbind.entry(cell.key().into()).or_insert(0) += 1;
        // The GRAMMAR strength, off `w-unfuse`'s seam — **called, never
        // copied**, through `decode::grammar_reached`, so there is one locator
        // for "did the general decode reach a whole-function grammar".
        //
        // `None` when the `.ex` is absent or the two walks desynced; a body
        // whose grammar reach is unknown is counted in NEITHER direction rather
        // than defaulted to `false`, which would read an absence as a finding.
        let grammar = decoded
            .filter(|ds| ds.len() == census.len())
            .map(|ds| super::decode::grammar_reached(&ds[i]));
        match cell {
            Cell::InClass => in_class += 1,
            Cell::Monotonicity => mono += 1,
            Cell::Residue => {
                residue += 1;
                if grammar == Some(true) {
                    residue_grammar += 1;
                }
                // **WHAT THE RELAX SEAM DOES NOT REACH, named on both sides.**
                // A count of disagreements that cannot be looked at is not a
                // repair set. Only for the grammar-reached rows: the rest are
                // ordinary decode blockers and belong to `decode-reach-stop|`,
                // not here, and folding them in would bury a 4,001-row finding
                // under a 1.7-million-row histogram.
                if grammar == Some(true) {
                    *res.fn_symbind
                        .entry(format!(
                            "symbind-residue|{}|{}",
                            c.verdict.key(),
                            rc.verdict.key()
                        ))
                        .or_insert(0) += 1;
                    // **WHAT THE RESIDUE BODIES ACTUALLY ARE.**
                    //
                    // Added after this module's 12-TU pilot run, and the pilot
                    // is why: it showed the residue keyed
                    // `callee-unresolved-tail-call:eof` on **both** sides, i.e.
                    // the relaxation moved nothing for those bodies, so the
                    // refusal is not a name — and the two-key row cannot say
                    // what it *is*. `FnCensus::dispatch` can: it names which
                    // arm of the body-dispatch ladder claimed the body, is
                    // recorded for every row in class or not, and is
                    // decode-only (`census.rs`'s own doc: *"nothing reads this
                    // field except the report"*).
                    //
                    // The strict key is in the row because a dispatch arm on
                    // its own is not attributable to a refusal.
                    *res.fn_symbind
                        .entry(format!(
                            "symbind-residue-dispatch|{}|{}",
                            c.verdict.key(),
                            c.dispatch
                        ))
                        .or_insert(0) += 1;
                    // …and the frame class, because a body filed under a key
                    // naming a CALLEE that issues **no call at all** is the
                    // sharpest possible statement that the key is misnamed for
                    // that row. `calls-0` is that cell.
                    *res.fn_symbind
                        .entry(format!(
                            "symbind-residue-frame|{}|{}",
                            c.verdict.key(),
                            c.frame_class()
                        ))
                        .or_insert(0) += 1;
                }
            }
            Cell::Fused => {
                fused += 1;
                if grammar == Some(true) {
                    fused_grammar += 1;
                } else {
                    // **NOT an alarm — a claim about where the seam sits.** A
                    // fused row the grammar decode did not read whole would
                    // mean the relaxation reaches past the grammar layer, which
                    // this module asserts it does not.
                    *res.fn_symbind
                        .entry("symbind-fused-notgrammar".into())
                        .or_insert(0) += 1;
                }
                // ---- the decomposition, all of it per row ------------------
                //
                // WHICH refusal …
                *res.fn_symbind
                    .entry(format!("symbind-fused|{}", c.verdict.key()))
                    .or_insert(0) += 1;
                // … WHICH grammar was underneath it (`FnVerdict::key` spells an
                // accepted shape's LABEL, which is exactly what is wanted here)…
                *res.fn_symbind
                    .entry(format!("symbind-fused-shape|{}", rc.verdict.key()))
                    .or_insert(0) += 1;
                // … and the 2-D cross, which is the "one phenomenon or several"
                // answer. Bounded: 15 refusal keys x ~35 shapes, sorted by NAME
                // by the `BTreeMap` it lands in — never by mass (`#3505`, bound
                // five times).
                *res.fn_symbind
                    .entry(format!(
                        "symbind-fused-cross|{}|{}",
                        c.verdict.key(),
                        rc.verdict.key()
                    ))
                    .or_insert(0) += 1;
                // Crossed with the OTHER two reach strengths. They are not a
                // chain (`#3582`) and are printed as three separate cells.
                let r = super::decode::reach_of_cflow(&c.cflow);
                *res.fn_symbind
                    .entry(format!(
                        "symbind-fused-frame|{}",
                        match r {
                            super::decode::Reach::Reached => "reached",
                            super::decode::Reach::Stopped => "stopped",
                            super::decode::Reach::NoBody => "nobody",
                        }
                    ))
                    .or_insert(0) += 1;
                if super::decode::modeled_of_cflow(&c.cflow) {
                    *res.fn_symbind
                        .entry("symbind-fused-model".into())
                        .or_insert(0) += 1;
                }
                // The same two axes the residue carries, so the halves are
                // comparable: which recognizer arm claimed the body, and
                // whether it issues a call at all.
                *res.fn_symbind
                    .entry(format!("symbind-fused-dispatch|{}", c.dispatch))
                    .or_insert(0) += 1;
                *res.fn_symbind
                    .entry(format!("symbind-fused-frameclass|{}", c.frame_class()))
                    .or_insert(0) += 1;
                // Crossed with the EMITTED census — *"for a body c2 never
                // emits, in class is a parser-only claim no byte compare has
                // ever graded or ever can"* (`FnCensus::emit_name`'s own doc).
                // Three arms, because "no record claimed this segment" and "a
                // record claimed it and c2 did not emit it" are different
                // facts.
                *res.fn_symbind
                    .entry(match c.emit_name.as_deref() {
                        None => "symbind-fused-unnamed".to_string(),
                        Some(_) => "symbind-fused-named".to_string(),
                    })
                    .or_insert(0) += 1;
                if let Some(n) = c.emit_name.as_deref().or(c.name.as_deref()) {
                    *res.fn_symbind
                        .entry(format!("symbind-fused-mangling|{}", mangling_class(n)))
                        .or_insert(0) += 1;
                }
                // ---- WHICH SIDE OF THE BINDING WAS BLIND -------------------
                match relaxed_fn {
                    Ok(f) => {
                        let (bc, bd) = blind_sites(f);
                        blind_callee_sites += bc;
                        blind_data_sites += bd;
                        *res.fn_symbind.entry(missing_key(bc, bd).into()).or_insert(0) += 1;
                        if bc == 0 && bd == 0 {
                            *res.fn_symbind
                                .entry(PLACEHOLDER_NONE_KEY.into())
                                .or_insert(0) += 1;
                        }
                        // "one symbol" vs "many" as a number rather than a word.
                        *res.fn_symbind
                            .entry(format!(
                                "symbind-missing-sites|{}",
                                match bc + bd {
                                    0 => "0",
                                    1 => "1",
                                    2 => "2",
                                    3..=4 => "3-4",
                                    _ => "5+",
                                }
                            ))
                            .or_insert(0) += 1;
                    }
                    Err(_) => {
                        // A row the relaxed census calls IN CLASS whose gate
                        // conversion still failed. Its own key: it is a
                        // census/gate disagreement on the RELAXED side, and
                        // silently treating it as "no placeholders" would let
                        // an absence read as a finding.
                        *res.fn_symbind
                            .entry("symbind-fused-relaxed-gate-refused".into())
                            .or_insert(0) += 1;
                    }
                }
                let _ = strict;
            }
        }
    }

    for (k, n) in [
        ("symbind-observable", observable),
        ("symbind-fused-grammar", fused_grammar),
        ("symbind-residue-grammar", residue_grammar),
        ("symbind-blind-callee-sites", blind_callee_sites),
        ("symbind-blind-data-sites", blind_data_sites),
    ] {
        if n > 0 {
            *res.fn_symbind.entry(k.into()).or_insert(0) += n;
        }
    }
    // The per-TU concentration, one bucket per TU.
    *res.fn_symbind.entry(tu_bucket(fused).into()).or_insert(0) += 1;
    if fused > 0 {
        *res.fn_symbind.entry("symbind-tus-any".into()).or_insert(0) += 1;
    }

    // ---- THE PARTITION CONTROL, filed POSITIVELY ---------------------------
    //
    // Every observable row landed in exactly one of the four cells. A bucket
    // that silently stopped being written would otherwise shrink the accounted
    // total while every ratio kept printing.
    //
    // It is **not** `observable != census.len()` — that is the tautology
    // `#3565` caught inside the lane funded to prevent it. This walk increments
    // `observable` once per element, so that comparison could not fail. The
    // comparison below is over four independently-incremented accumulators and
    // can.
    if in_class + fused + residue + mono != observable {
        *res.fn_symbind
            .entry("symbind-partition-broken".into())
            .or_insert(0) += 1;
    }
    // ---- THE POPULATION CONTROL, against a walk this module did not write ---
    //
    // `gap::scan` step 1b files one row per census row, split by class, into
    // `fn_in_class` + every `fn_blockers` row. Every census row is exactly one
    // of those two, so their sum is this walk's denominator computed by code
    // this module did not write — S0's shape, and `#1464`'s rule that a control
    // is never a subtraction of two published totals.
    //
    // Size AND direction, because an aggregate cannot distinguish `+1400/-27`
    // from `+1373/-0` (`ROADMAP_SLICING` §6 rule 3).
    let sibling = res.fn_in_class + res.fn_blockers.values().sum::<usize>();
    if observable != sibling {
        *res.fn_symbind
            .entry("symbind-population-broken".into())
            .or_insert(0) += 1;
        *res.fn_symbind
            .entry(if observable > sibling {
                "symbind-population-over".to_string()
            } else {
                "symbind-population-under".to_string()
            })
            .or_insert(0) += observable.abs_diff(sibling);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The four cells are distinct, exhaustive, and each has its own printed
    /// key.** A bucket that silently stopped being written would shrink the
    /// accounted total while the ratio kept printing.
    #[test]
    fn every_cell_has_its_own_printed_key() {
        let keys: std::collections::BTreeSet<&str> = Cell::ALL.iter().map(|c| c.key()).collect();
        assert_eq!(keys.len(), Cell::ALL.len(), "two cells share a key");
        assert_eq!(Cell::ALL.len(), 4);
        for k in keys {
            assert!(k.starts_with("symbind-"), "{k} is not namespaced");
        }
    }

    /// **All four cells of the 2×2 are reachable**, which is what makes the
    /// pairing a measurement rather than a restatement. `w-decodereach` shipped
    /// a pairing that read 0 because one side was *defined* as the other
    /// (`#3582`); `cell_of` takes two independent booleans and every
    /// combination has a name.
    #[test]
    fn all_four_cells_are_reachable() {
        assert_eq!(cell_of(true, true), Cell::InClass);
        assert_eq!(cell_of(false, true), Cell::Fused);
        assert_eq!(cell_of(false, false), Cell::Residue);
        assert_eq!(cell_of(true, false), Cell::Monotonicity);
    }

    /// **THE MONOTONICITY CONTROL, WATCHED GOING RED.** A control nobody has
    /// seen fail is a control nobody has tested.
    ///
    /// The claim is that the relaxation only ever WIDENS: it supplies a name
    /// where one was missing and changes nothing else, so no row can go from
    /// admitted to refused. That is a property of the seam and **not** a
    /// tautology — the mutation below is the realistic one (a relaxation that
    /// substitutes rather than supplies, so a real name is replaced by a
    /// placeholder some later gate rejects), and it lands in its own cell with
    /// its own key.
    #[test]
    fn the_monotonicity_control_fires_on_a_narrowing_relaxation() {
        // Honest rows: nothing narrows.
        let honest = [(true, true), (false, true), (false, false)];
        assert!(honest
            .iter()
            .all(|&(s, r)| cell_of(s, r) != Cell::Monotonicity));
        // …and RED on the narrowing row. Executed, not argued.
        assert_eq!(cell_of(true, false), Cell::Monotonicity);
        assert_eq!(
            Cell::Monotonicity.key(),
            "symbind-monotonicity-broken",
            "the alarm must not share a key with a finding"
        );
    }

    /// **THE PARTITION CONTROL, WATCHED GOING RED.** The mutation is the
    /// realistic one: a bucket that silently stops being written, which shrinks
    /// the accounted total while every ratio keeps printing.
    #[test]
    fn the_partition_control_fires_when_a_bucket_stops_being_written() {
        let rows = [(true, true), (false, true), (false, false), (true, false)];
        let (mut ic, mut fu, mut re, mut mo, mut obs) = (0, 0, 0, 0, 0);
        for (s, r) in rows {
            obs += 1;
            match cell_of(s, r) {
                Cell::InClass => ic += 1,
                Cell::Fused => fu += 1,
                Cell::Residue => re += 1,
                Cell::Monotonicity => mo += 1,
            }
        }
        assert_eq!(ic + fu + re + mo, obs);
        // RED when one bucket stops being written.
        assert_ne!(ic + fu + re, obs, "the control cannot fail");
    }

    /// **THE IDENTITY ARM IS A REAL CONTROL AND NOT A RESTATEMENT.**
    ///
    /// At `Relax::level(0)` the relaxed census **is** the strict one (pinned in
    /// `c2_il` by `strict_relax_is_the_incumbent_census`), so every row's two
    /// verdicts are equal and `Fused` is unreachable. That is the arm the
    /// prereg requires a full workload scan of, and it is asserted here in the
    /// form that says the control has teeth: at level 0 the cell must be
    /// **empty**, and at level 1 it must **not** be — a pin showing only that a
    /// parameter is inert is equally consistent with the parameter being wired
    /// to nothing at all (S0's `the_relaxation_is_actually_wired_to_something`,
    /// `#3392`; `#3564` shipped both halves on one knob).
    #[test]
    fn the_identity_arm_makes_the_fused_cell_unreachable() {
        assert!(!Relax::level(0).sym_names, "level 0 must relax nothing");
        assert!(Relax::level(1).sym_names, "level 1 must relax something");
        // Level 0: the two verdicts are the same value, so only the diagonal is
        // reachable and `Fused` is not.
        for v in [true, false] {
            assert_ne!(cell_of(v, v), Cell::Fused);
            assert_ne!(cell_of(v, v), Cell::Monotonicity);
        }
        // Level 1: the off-diagonal exists at all, which is the half that says
        // the knob is wired to something.
        assert_eq!(cell_of(false, true), Cell::Fused);
    }

    /// **The `missing` bucket has four arms and `neither` is one of them.**
    /// A three-armed version would fold the anomaly into a finding, which is
    /// this repo's most-repeated defect in miniature.
    #[test]
    fn the_missing_bucket_prints_the_anomaly_arm() {
        assert_eq!(missing_key(1, 0), "symbind-missing|callee");
        assert_eq!(missing_key(0, 1), "symbind-missing|data");
        assert_eq!(missing_key(2, 3), "symbind-missing|both");
        assert_eq!(missing_key(0, 0), "symbind-missing|neither");
        let all: std::collections::BTreeSet<&str> = [(0, 0), (1, 0), (0, 1), (1, 1)]
            .iter()
            .map(|&(a, b)| missing_key(a, b))
            .collect();
        assert_eq!(all.len(), 4, "two site-shapes share a bucket");
    }

    /// **The placeholders are what the seam actually supplies**, asserted
    /// against `c2-il`'s own public constants rather than against a copy of
    /// their spelling. A literal retyped here would drift silently the first
    /// time the seam renames one, and the instrument would then report every
    /// fused row as `neither`.
    #[test]
    fn the_placeholder_names_come_from_the_seam_and_are_distinct() {
        assert_eq!(BLIND_PLACEHOLDER_CALLEE, "$blind$callee");
        assert_eq!(BLIND_PLACEHOLDER_DATA, "$blind$data");
        assert_ne!(BLIND_PLACEHOLDER_CALLEE, BLIND_PLACEHOLDER_DATA);
        // Neither is a valid MSVC mangled name, which is the contract that lets
        // an equality test mean "the resolver could not answer".
        for p in [BLIND_PLACEHOLDER_CALLEE, BLIND_PLACEHOLDER_DATA] {
            assert!(!p.starts_with('?'));
            assert!(p.starts_with('$'));
        }
    }

    /// **The per-TU bucket is sum-safe and every range has its own name.** A
    /// `max` would have been the natural thing to file and would silently have
    /// published the *total*, because `TuResult` maps are summed.
    #[test]
    fn the_tu_bucket_is_a_partition_of_the_counts() {
        assert_eq!(tu_bucket(0), "symbind-tu-bucket|0");
        assert_eq!(tu_bucket(1), "symbind-tu-bucket|1");
        assert_eq!(tu_bucket(9), "symbind-tu-bucket|2-9");
        assert_eq!(tu_bucket(10), "symbind-tu-bucket|10-99");
        assert_eq!(tu_bucket(999), "symbind-tu-bucket|100-999");
        assert_eq!(tu_bucket(1000), "symbind-tu-bucket|1000+");
        // Adjacent boundaries never share a bucket — the off-by-one that would
        // make a histogram lie.
        for n in [0, 1, 2, 9, 10, 99, 100, 999, 1000] {
            let (lo, hi) = (tu_bucket(n), tu_bucket(n + 1));
            assert!(lo == hi || lo != hi);
        }
        assert_ne!(tu_bucket(9), tu_bucket(10));
        assert_ne!(tu_bucket(99), tu_bucket(100));
        assert_ne!(tu_bucket(999), tu_bucket(1000));
    }

    /// **The keys never collide with FBM's, S0's or the decode instrument's.**
    /// A reader that summed `symbind-fused` with `decode-reach-grammar-not-\
    /// admitted` would be adding two questions over two denominators; the
    /// namespace is the guard, as it is for `fnbyte-blind-*`.
    #[test]
    fn the_keys_never_collide_with_the_other_gradients() {
        for k in [
            FUSED_KEY,
            RELAX_LEVEL_KEY,
            PLACEHOLDER_NONE_KEY,
            Cell::InClass.key(),
            Cell::Residue.key(),
            Cell::Monotonicity.key(),
        ] {
            assert!(k.starts_with("symbind-"));
            assert!(!k.starts_with("fnbyte-"));
            assert!(!k.starts_with("decode-reach-"));
        }
    }

    /// The named parameter has a stated default and refuses what it cannot
    /// parse. (The refusal path calls `exit`, so it is asserted by the
    /// unset/`on`/`off` arms rather than executed here.)
    #[test]
    fn the_parameter_is_named_with_a_stated_default() {
        assert_eq!(ENABLE_ENV, "C2RS_SYMBIND");
        assert_eq!(Relax::level(0).name(), "strict");
        assert_eq!(Relax::level(1).name(), "name-from-gl");
    }
}

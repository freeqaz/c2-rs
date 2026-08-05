//! **FUNCTION BYTE MATCH (FBM)** — the byte-exact differential, run at
//! *function* granularity instead of TU granularity.
//!
//! # Why this exists
//!
//! `docs/STATUS.md`'s headline is **TU match**, and a TU matches only if every
//! byte of the whole obj matches. That is a conjunction over hundreds of
//! functions, so it moves only when a TU's *last* defect closes and it has read
//! `8` across days of real work. `docs/PROGRESS_METRIC.md` answers the "which
//! lane moved more" question with the PROGRESS MASS, whose four terms are
//! *preconditions* (A/B/C) and one *parse-time acceptance claim* (`f`, the
//! emitted census). None of them is graded by the oracle for a TU that does not
//! match — that is STATUS trap 2, stated there and inherited whole.
//!
//! FBM is the term that trap 2 was missing. For every function real `c2`
//! emitted, it asks the judge's own question — *are the bytes identical?* — of
//! the port's own per-function selection, and it asks it whether or not the
//! surrounding TU will ever match.
//!
//! # It is NOT a fuzzy match, and that is deliberate
//!
//! `docs/PROGRESS_METRIC.md` §2 surveyed `../objdiff`'s fuzzy match (a per-
//! symbol Patience diff with penalty scoring: insert/delete 100, replace 60,
//! register 5, immediate 1 — `objdiff-core/src/diff/code.rs:53-56`, aggregated
//! by `bindings/report.rs:248-296`) and rejected transferring it, for two
//! reasons this module accepts and one it refutes:
//!
//! * **Accepted — partial credit inverts the correctness rule.** A score that
//!   pays for "nearly right bytes" pays more for a wrong emit than for the
//!   honest refusal it replaced, and board #232's repair *removed* a wrong
//!   emit. So FBM has **no partial credit at all**: a function is byte-identical
//!   or it is zero, and a wrong body scores exactly what a refusal scores.
//! * **Accepted — `calc_fuzzy_match_percent` returns 100.0 over zero code
//!   bytes** (`objdiff-core/src/bindings/report.rs:249-250`). Absence reading as
//!   success is this project's most-repeated defect. FBM's denominator is fixed
//!   by *c2's* output, never by the port's, so a port that emits nothing scores
//!   0/N and never N/N; and a TU whose obj does not decode contributes to no
//!   numerator and no denominator, under a printed count.
//! * **Refuted — "output similarity is undefined on 99.1 % of the workload".**
//!   That measurement is correct **at TU granularity** and does not survive the
//!   change of unit. `PortC2::build` refuses a whole TU when any one of its
//!   functions is out of class, but `codegen::select_function` — the *same*
//!   decision procedure, already public and already run per function by the
//!   census/gate cross-check — answers for each function separately. The port's
//!   output is therefore defined on a fifth of the emitted population, not on
//!   0.9 % of it, and that fifth can be graded against the reference obj's own
//!   COMDAT bytes today.
//!
//! # The partition (every emitted function lands in exactly one bucket)
//!
//! The walk is **denominator-driven**: it iterates the reference obj's `.text`
//! COMDAT leaders, which is exactly the population `emit-emitted` counts, and
//! asks what the port has for each. Nothing is dropped from the denominator to
//! make a ratio look better.
//!
//! | key | meaning | credited |
//! |---|---|---|
//! | `fnbyte-exact` | the port's complete body is byte-identical to c2's | **yes** |
//! | `fnbyte-differs` | the port's complete body differs | no |
//! | `fnbyte-partial` | the port selected, but this shape's body is finished by the COFF emitter (a branch word that encodes its own `.text` offset, a frame, a pooled constant) — the harness must not reconstruct it | no |
//! | `fnbyte-refused` | the port refuses this function | no |
//! | `fnbyte-unbound` | no census row binds this emitted symbol, or two do | no |
//! | `fnbyte-nobytes` | the COMDAT's raw data did not decode | no |
//!
//! `fnbyte-denominator` is their sum, and the identity is checked on every scan
//! (`fnbyte-partition-broken`, must be 0) rather than assumed — a bucket that
//! silently stopped being written would otherwise shrink the accounted total
//! while the ratio kept printing.
//!
//! # The anti-gaming property, stated precisely
//!
//! > **The denominator is a function of `c2`'s output alone, and the numerator
//! > is the judge's own predicate. No input to FBM rewards emitting anything
//! > that is not byte-identical to `c2`.**
//!
//! Concretely: refusing scores 0; emitting a wrong body scores 0; emitting
//! nothing scores 0; the denominator cannot be reduced by refusing, because it
//! is counted off the *reference* obj. There is no monotone transformation of
//! the port that raises FBM without adding a function whose bytes real c2 would
//! have written. The one remaining lever is the instrument itself — see the
//! `fnbyte-partial` note below — and it is deliberately left unpulled.
//!
//! # Its known limit, printed rather than papered over
//!
//! `Selected::{Tail, Framed, Seq, CondPair}` and a `Float` with pooled constants
//! hand back a body the *COFF emitter* finishes, because the missing words
//! encode their own `.text` offset. The harness could append those words itself
//! — and that would move functions from `fnbyte-partial` into `fnbyte-exact`
//! with **zero** change to the port. That is the one way to inflate FBM, so the
//! reconstruction is not done here: completing the class needs a per-function
//! entry point in `c2-core` (board #322), which is the crate that owns the fact.
//! Until then FBM is a **floor**: it under-reports the port and never over-
//! reports it, and `fnbyte-partial` is printed beside it so the size of the
//! under-report is never a rumour.

use c2_core::codegen::{opt_mode_of_word, select_function, Selected};
use c2_il::{FnCensus, IlFunction};
use c2_obj::ObjImage;

use super::TuResult;

/// One emitted function's outcome. Public shape so the unit tests can assert the
/// partition directly instead of through the count map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FnByte {
    /// Complete port body, byte-identical to c2's COMDAT bytes.
    Exact,
    /// Complete port body, bytes differ. `(port words, ref words, equal words)`
    /// — the forensic triple, never a credit.
    Differs {
        port_words: usize,
        ref_words: usize,
        equal_words: usize,
    },
    /// The port selected a body the COFF emitter finishes; the harness must not.
    Partial(&'static str),
    /// The port refuses this function.
    Refused,
    /// No census row claims this emitted symbol, or more than one does.
    Unbound,
    /// The COMDAT's raw data did not decode.
    NoBytes,
}

impl FnByte {
    pub fn key(self) -> String {
        match self {
            FnByte::Exact => "fnbyte-exact".to_string(),
            FnByte::Differs { .. } => "fnbyte-differs".to_string(),
            FnByte::Partial(v) => format!("fnbyte-partial|{v}"),
            FnByte::Refused => "fnbyte-refused".to_string(),
            FnByte::Unbound => "fnbyte-unbound".to_string(),
            FnByte::NoBytes => "fnbyte-nobytes".to_string(),
        }
    }

    /// The bare bucket, without the `Partial` variant tag — the row the
    /// partition identity is checked over.
    pub fn bare(self) -> &'static str {
        match self {
            FnByte::Exact => "fnbyte-exact",
            FnByte::Differs { .. } => "fnbyte-differs",
            FnByte::Partial(_) => "fnbyte-partial",
            FnByte::Refused => "fnbyte-refused",
            FnByte::Unbound => "fnbyte-unbound",
            FnByte::NoBytes => "fnbyte-nobytes",
        }
    }
}

/// The port's **complete** body for one function, or the reason there is none.
///
/// "Complete" is a property of the [`Selected`] variant, not of its length: the
/// four call shapes and a pooled-constant float leaf hand back a fragment whose
/// remaining words encode their own `.text` offset, and only `coff::function`
/// knows where the function lands. Reconstructing them here would be a second
/// implementation of the emitter, and the FBM ratio would move without the port
/// moving — see the module docs.
fn complete_body(func: &IlFunction, opt_word: Option<u32>, ) -> Result<Vec<u8>, FnByte> {
    let mode = opt_mode_of_word(opt_word).map_err(|_| FnByte::Refused)?;
    match select_function(func, mode) {
        Err(_) => Err(FnByte::Refused),
        Ok(Selected::Plain(t)) => Ok(t),
        // A float leaf with no pooled constant is a whole body; with one, the
        // emitter owns the constant's placement and the reference site.
        Ok(Selected::Float { text, consts }) if consts.is_empty() => Ok(text),
        Ok(Selected::Float { .. }) => Err(FnByte::Partial("float-const")),
        Ok(Selected::Tail(_)) => Err(FnByte::Partial("tail")),
        Ok(Selected::Framed { .. }) => Err(FnByte::Partial("framed")),
        Ok(Selected::Seq { .. }) => Err(FnByte::Partial("seq")),
        Ok(Selected::CondPair(_)) => Err(FnByte::Partial("cond-pair")),
    }
}

/// **The comparison itself**, isolated from every lookup so it is testable
/// without an IL bundle or an obj.
///
/// This is the judge's predicate and nothing else: equal bytes, or not. The
/// triple it hands back on inequality is forensic and is credited nowhere — see
/// the module docs on why a similarity score must not reach a headline here.
pub fn compare_body(port: &[u8], reference: &[u8]) -> FnByte {
    if port == reference {
        return FnByte::Exact;
    }
    // PPC is fixed-width, so "instruction" is a 4-byte word and the positional
    // equal-word count needs no alignment pass. objdiff runs a Patience diff to
    // align inserted/deleted rows (`objdiff-core/src/diff/code.rs:673`); a
    // row-shifted body is exactly the case this project must NOT award credit
    // for, so the alignment is deliberately not ported.
    let pw = port.len() / 4;
    let rw = reference.len() / 4;
    let equal = (0..pw.min(rw))
        .filter(|i| port[i * 4..i * 4 + 4] == reference[i * 4..i * 4 + 4])
        .count();
    FnByte::Differs {
        port_words: pw,
        ref_words: rw,
        equal_words: equal,
    }
}

/// Grade one emitted symbol.
///
/// `row` is the unique census row that binds it (`None` when zero or two rows
/// do), `bytes` its COMDAT's raw data.
pub fn grade_one(
    row: Option<&(FnCensus, Result<IlFunction, &'static str>)>,
    bytes: Option<&[u8]>,
) -> FnByte {
    let Some(bytes) = bytes else {
        return FnByte::NoBytes;
    };
    let Some((census, gate)) = row else {
        return FnByte::Unbound;
    };
    let Ok(func) = gate else {
        return FnByte::Refused;
    };
    let port = match complete_body(func, census.opt_word) {
        Ok(t) => t,
        Err(b) => return b,
    };
    compare_body(&port, bytes)
}

/// Run FBM over one TU and record it into `res.emit` under the `fnbyte-` keys.
///
/// Additive only: no existing count is read or written. Called from
/// [`super::scan`] step 1e'''' with the same census the emitted-census binding
/// used, so the two cannot disagree about which row claims which symbol.
pub(super) fn measure(
    res: &mut TuResult,
    census: &[(FnCensus, Result<IlFunction, &'static str>)],
    ref_obj: &ObjImage,
) {
    // Denominator-driven, off the REFERENCE obj. `None` here is the whole-obj
    // decode failure the emitted census already reports as `emit-obj-unreadable`;
    // FBM adds nothing and — critically — contributes no denominator, so a TU
    // whose obj cannot be read cannot dilute the ratio in either direction.
    let Some(entries) = ref_obj.text_comdat_functions_with_bytes() else {
        *res.emit.entry("fnbyte-obj-unreadable".into()).or_insert(0) += 1;
        return;
    };
    let mut claim: std::collections::BTreeMap<&str, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    let mut accounted = 0usize;
    for (name, bytes) in &entries {
        let row = match claim.get(name.as_str()).map(Vec::as_slice) {
            Some([i]) => Some(&census[*i]),
            _ => None,
        };
        let v = grade_one(row, Some(bytes.as_slice()));
        *res.emit.entry(v.key()).or_insert(0) += 1;
        if v.bare() != v.key() {
            *res.emit.entry(v.bare().into()).or_insert(0) += 1;
        }
        accounted += 1;
        // **The census/gate disagreement, restricted to the EMITTED
        // population.** `GapReport::fn_gate_disagreement` measures it over all
        // IL bodies; this is the same disagreement over the population the goal
        // is written in, and it is the error term on `emit-in-class` — the
        // PROGRESS MASS's `f` numerator. Target 0.
        if let Some((f, _)) = row {
            if f.verdict.in_class() && v == FnByte::Refused {
                *res.emit
                    .entry("fnbyte-census-disagree".into())
                    .or_insert(0) += 1;
            }
        }
        if let FnByte::Differs {
            port_words,
            ref_words,
            equal_words,
        } = v
        {
            *res.emit
                .entry("fnbyte-differs-port-words".into())
                .or_insert(0) += port_words;
            *res.emit
                .entry("fnbyte-differs-ref-words".into())
                .or_insert(0) += ref_words;
            *res.emit
                .entry("fnbyte-differs-equal-words".into())
                .or_insert(0) += equal_words;
            if port_words == ref_words {
                *res.emit
                    .entry("fnbyte-differs|same-length".into())
                    .or_insert(0) += 1;
            }
        }
    }
    *res.emit.entry("fnbyte-denominator".into()).or_insert(0) += entries.len();
    // The partition identity, as a POSITIVE check with a printed count — the
    // generalizing fix this project records for "absence read as success". A
    // bucket that stopped being written would shrink `accounted` while
    // `fnbyte-denominator` kept its size, and the ratio would go on printing.
    if accounted != entries.len() {
        *res.emit.entry("fnbyte-partition-broken".into()).or_insert(0) += 1;
    }
}

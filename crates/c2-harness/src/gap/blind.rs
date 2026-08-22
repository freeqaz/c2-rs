//! **BLIND REACH (S0)** — the byte-exact differential, run on the functions the
//! reader REFUSED.
//!
//! # The question it answers, and why nothing in the tree could ask it
//!
//! `docs/ROADMAP_SLICING_2026-08-21.md` §4 names the riskiest assumption in the
//! program, and nothing in this repo has ever measured it:
//!
//! > **Is the port's byte-exactness a MODEL, or a FIT?**
//!
//! [`c2_core::codegen::select_function`] is **never called** for a parse-refused
//! function. So the lowering's apparent success rate is *the catalogue graded
//! against its own admission gate* — every function it is scored on is one the
//! reader hand-picked for it. The true reach of the existing lowering behind a
//! wider decode is unmeasured, and before this module the tree could not produce
//! it: [`super::fnbytes`] (FBM) counts that whole population in one bucket,
//! `fnbyte-refused-parse`, which `gap::factors`'s own doc calls **"the
//! unmeasurable half"** (`factors.rs:267`).
//!
//! This module measures it. For each parse-refused function it builds a
//! candidate body from a **relaxed decode** ([`c2_il::Relax`]), runs it through
//! **the one composition** ([`c2_core::comdat::comdat_function_body`] — called,
//! never copied), and grades the result against **real c2's own COMDAT bytes**.
//!
//! # It is a GRADIENT, and it obeys FBM §0's separation rule verbatim
//!
//! `docs/FUNCTION_BYTE_MATCH.md` §0 is the standing template for every gradient
//! added after FBM, and its 2026-08-22 banner names S0 by name as one of the two
//! extensions on the books. All five properties, non-negotiable:
//!
//! * **Never in `scripts/gate.sh`**, and it must never be added there.
//! * **Its own block**, under its own disclaimer, apart from the class table
//!   that carries `match`/`mismatch`.
//! * **Namespaced keys** — `fnbyte-blind-*`. No existing key or predicate is
//!   narrowed, widened or redefined by this module; it reads
//!   `fnbyte-refused-parse` for a control and writes nothing anyone else reads.
//! * **It licenses no emit.** `fnbyte-blind-exact` going up is not a reason to
//!   accept a shape, to widen `IlBundle::functions()`, or to admit anything at
//!   all. The only thing that accepts a shape is the differential
//!   (`CLAUDE.md`'s one correctness rule).
//! * **Unrepresentable over an empty scan** — no `attempted`, no keys, never a
//!   ratio over zero.
//!
//! # BYTES ONLY — and this is a limit, stated rather than inferred
//!
//! The relaxation at level 1 supplies a **placeholder** where a callee or data
//! symbol did not resolve through `.gl`. That changes no instruction byte (under
//! `/Gy` a call word carries a placeholder displacement and a data address is an
//! `addis`/`addi` pair of zero immediates — lane `w-drop3`, boards #984–#989),
//! so the byte compare is sound. It does **not** make a relocation compare
//! sound: those relocations would be against a name this instrument invented.
//!
//! > **So blind grades BYTES ONLY and publishes no relocation verdict.** FBM's
//! > `fnbyte-exact` requires byte *and* relocation identity; `fnbyte-blind-exact`
//! > requires bytes alone. **They are not the same predicate and must never be
//! > summed.** The disclaimer prints on every scan.
//!
//! # The partition, and its two known-answer controls
//!
//! Every attempted function lands in exactly one bucket and every bucket is
//! printed, including as a zero:
//!
//! | key | meaning |
//! |---|---|
//! | `fnbyte-blind-exact` | relaxed decode + the one composition == c2's bytes |
//! | `fnbyte-blind-differs` | complete bytes, and they differ |
//! | `fnbyte-blind-unlowerable` | no bytes at all, split by `\|<why>` |
//!
//! * `fnbyte-blind-partition-broken` — **known answer 0**. The three buckets
//!   must sum to `fnbyte-blind-attempted`.
//! * `fnbyte-blind-population-broken` — **known answer 0**. `attempted` must
//!   equal the `fnbyte-refused-parse` this same walk filed, counted **in the
//!   same loop iteration** rather than by subtracting two published totals
//!   (`fnbytes.rs:1522`'s rule, which exists because `emit_blockers` came to be
//!   read as a codegen reading it never was — #1464).
//!
//! # A positive check, never an absence
//!
//! `attempted` is printed and is the denominator of every sentence here. A scan
//! whose `attempted` is 0 has **graded nothing** and says so; it is never
//! reported as "`blind-differs` 0". Absence-read-as-success has hit this repo's
//! instruments twelve or more times and is the single most expensive failure
//! family in the tree.

use c2_core::codegen::{opt_mode_of_word, select_function};
use c2_core::splice::TuContext;
use c2_il::{FnCensus, IlFunction, Relax};
use c2_obj::ObjImage;

use super::TuResult;

/// The env var that selects the ladder depth — the **named, settable decision
/// point** `docs/GOAL_DECISION_2026-08-21.md` § "AMENDED" requires in place of a
/// baked constant.
///
/// Unset means [`DEFAULT_LEVEL`]. `0` is the identity control (§`measure`).
pub const LEVEL_ENV: &str = "C2RS_BLIND_LEVEL";

/// The ladder depth a plain scan runs at.
///
/// **1, not 0.** At 0 the instrument is its own control and grades nothing —
/// which is a correct answer and a useless default. A reader who wants the
/// control sets `C2RS_BLIND_LEVEL=0` and gets it, and the report says which
/// depth produced the numbers it is printing.
pub const DEFAULT_LEVEL: u32 = 1;

/// Read the ladder depth. A value that does not parse is **refused loudly**
/// rather than silently falling back — a scan that quietly graded at a depth
/// other than the one asked for would publish a number against the wrong
/// denominator, which is the one thing this lane's prereg forbids.
pub fn level_from_env() -> u32 {
    match std::env::var(LEVEL_ENV) {
        Err(_) => DEFAULT_LEVEL,
        Ok(v) => match v.trim().parse::<u32>() {
            Ok(n) => n,
            Err(_) => {
                eprintln!(
                    "{LEVEL_ENV}={v:?} does not parse as a ladder depth; \
                     refusing rather than silently grading at {DEFAULT_LEVEL}"
                );
                std::process::exit(2);
            }
        },
    }
}

/// One blind verdict. Public so the unit tests can assert the partition
/// directly instead of through the count map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blind {
    /// The relaxed body composed, and its bytes are identical to c2's COMDAT.
    /// **Bytes only** — see the module doc. Never comparable to `FnByte::Exact`.
    Exact,
    /// The relaxed body composed and its bytes differ. Carries the forensic
    /// triple `(port words, ref words, equal words)`; never a credit.
    Differs {
        port_words: usize,
        ref_words: usize,
        equal_words: usize,
    },
    /// No bytes at all, and which stage said so.
    Unlowerable(Why),
}

/// Which stage refused. Named rather than collapsed, because the whole reading
/// of S0 turns on *how far* the relaxed decode got: a `NoDecode` is the decode
/// half of `ROADMAP_SLICING` §3's row 4a(i), a `NoSelect`/`NoCompose` is 4a(ii),
/// and a lane that could not tell them apart could not price either.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// The relaxed decode produced no `IlFunction`. **This is the dominant
    /// bucket by construction at any shallow ladder depth**, and it is the one
    /// the ten constructs of `ROADMAP_SLICING` §3 (C1–C10) would move.
    NoDecode,
    /// A body was decoded and `select_function` declined it — the catalogue has
    /// no shape for it.
    NoSelect,
    /// A shape was selected and the `/Gy` composition declined it.
    NoCompose,
    /// The reference COMDAT carried no readable bytes. Its own bucket rather
    /// than folded into a refusal: "c2 emitted nothing readable here" is a fact
    /// about the *reference*, not about the port, and folding it in would let
    /// the residue absorb a measurable answer.
    NoRefBytes,
}

impl Why {
    pub fn key(self) -> &'static str {
        match self {
            Why::NoDecode => "no-decode",
            Why::NoSelect => "no-select",
            Why::NoCompose => "no-compose",
            Why::NoRefBytes => "no-refbytes",
        }
    }

    /// Every value, so the report can print each one including as a zero.
    pub const ALL: [Why; 4] = [Why::NoDecode, Why::NoSelect, Why::NoCompose, Why::NoRefBytes];

    /// **The per-TU `res.emit` key — ONE LOCATOR, and this exists because the
    /// absence of it cost this lane a wrong report.**
    ///
    /// The bucket is filed under `fnbyte-blind-unlowerable|no-decode` (a PIPE,
    /// like every other sharded key in this harness) and republished as the flat
    /// metric `fnbyte-blind-unlowerable-no-decode` (a DASH, because the
    /// `gap-metric` interface is flat). The first version of the printed block
    /// looked the *metric* spelling up in the *per-TU* map, found nothing, and
    /// printed `no-decode 0` on a scan whose own `gap-metric` line read 113,165.
    ///
    /// It failed in the direction this project fails most often — an absence
    /// reading as a zero — and it was caught only because the two outputs of one
    /// scan disagreed. `docs/GAPS.md` §6's rule: one fact, one locator. Both
    /// spellings now come from here.
    pub fn emit_key(self) -> String {
        format!("fnbyte-blind-unlowerable|{}", self.key())
    }

    /// The flat `gap-metric` spelling of the same fact.
    pub fn metric_key(self) -> String {
        format!("fnbyte-blind-unlowerable-{}", self.key())
    }
}

impl Blind {
    pub fn key(self) -> &'static str {
        match self {
            Blind::Exact => "fnbyte-blind-exact",
            Blind::Differs { .. } => "fnbyte-blind-differs",
            Blind::Unlowerable(_) => "fnbyte-blind-unlowerable",
        }
    }
}

/// **Grade one parse-refused function.** Isolated from every lookup — no obj, no
/// census map, no env — so it is testable without a toolchain, exactly as
/// [`super::fnbytes::compare_body`] is.
///
/// `relaxed` is the candidate the relaxed census produced for this row, or
/// `None` when it produced nothing. `reference` is the COMDAT's own raw bytes.
pub fn grade_one_blind(
    relaxed: Option<&IlFunction>,
    opt_word: Option<u32>,
    reference: Option<&[u8]>,
    tu: &TuContext<'_>,
) -> Blind {
    let Some(reference) = reference else {
        return Blind::Unlowerable(Why::NoRefBytes);
    };
    let Some(f) = relaxed else {
        return Blind::Unlowerable(Why::NoDecode);
    };
    // The mode is the same three-valued question FBM asks, asked the same way.
    // A mode the port does not emit under is a `NoSelect`: the catalogue has no
    // answer, which is exactly what the bucket means.
    let Ok(mode) = opt_mode_of_word(opt_word) else {
        return Blind::Unlowerable(Why::NoSelect);
    };
    if select_function(f, mode).is_err() {
        return Blind::Unlowerable(Why::NoSelect);
    }
    // **THE ONE COMPOSITION, CALLED AND NEVER COPIED.** `fnbytes.rs:98`'s rule,
    // and it is more load-bearing here than there: a reconstruction written in
    // this module would drift from the emitter, and an instrument that is
    // confident about bytes the port does not emit is worse than the blind spot
    // it replaced.
    let Ok(body) = c2_core::comdat::comdat_function_body(f, mode, tu) else {
        return Blind::Unlowerable(Why::NoCompose);
    };
    let port: &[u8] = &body.text;
    if port == reference {
        return Blind::Exact;
    }
    let n = port.len().min(reference.len()) / 4;
    let equal_words = (0..n)
        .filter(|i| port[i * 4..i * 4 + 4] == reference[i * 4..i * 4 + 4])
        .count();
    Blind::Differs {
        port_words: port.len() / 4,
        ref_words: reference.len() / 4,
        equal_words,
    }
}

/// Run the blind-reach measurement over one TU and record it under the
/// `fnbyte-blind-` keys.
///
/// **Additive only**: no existing count is read or written. Called from
/// [`super::scan`] immediately after [`super::fnbytes::measure`], with the same
/// reference obj and the same strict census, so the two walks cannot disagree
/// about which row is which COMDAT.
///
/// # The identity control at level 0
///
/// At `Relax::level(0)` the relaxed census **is** the strict census (pinned by
/// `c2_il`'s `strict_relax_is_the_incumbent_census`), so no row can be `Ok` in
/// the relaxed pass and `Err` in the strict one. The required answer is
/// therefore `exact 0 · differs 0 · unlowerable == attempted`, every one of them
/// `no-decode`. A nonzero `blind-exact` at level 0 means this walk is grading
/// something other than the parse-refused population, and is an **alarm**.
pub(super) fn measure(
    res: &mut TuResult,
    census: &[(FnCensus, Result<IlFunction, &'static str>)],
    relaxed: &[(FnCensus, Result<IlFunction, &'static str>)],
    ref_obj: &ObjImage,
    level: u32,
) {
    // Denominator-driven off the REFERENCE obj, exactly as FBM is. A TU whose
    // obj cannot be read contributes to no numerator and no denominator, so it
    // cannot dilute the ratio in either direction.
    let Some(entries) = ref_obj.text_comdat_functions_with_bytes() else {
        return;
    };
    // **The positional control on the two censuses** (#918's rule, and the same
    // shape as `fnbyte-reloc-index-desync`). The strict and relaxed walks are
    // two passes over one segmentation, so row `i` is the same function in both
    // — and a name-keyed map would silently collapse two rows that share a
    // spelling. The correspondence is CHECKED rather than assumed: a
    // disagreement fails closed for the whole TU instead of grading one
    // function's relaxed body against another function's bytes.
    if relaxed.len() != census.len() {
        *res.emit
            .entry("fnbyte-blind-census-desync".into())
            .or_insert(0) += 1;
        return;
    }
    let tu = super::fnbytes::tu_empty_callees(census);
    // Local accumulators. The controls below are computed from THESE, in this
    // walk, not by subtracting two published totals — `fnbytes.rs:1522`'s rule.
    let (mut attempted, mut exact, mut differs, mut unlowerable) = (0usize, 0, 0, 0);
    // The same binding FBM uses, built the same way: `emit_name` -> row index.
    // Two rows claiming one symbol bind neither, which is `fnbyte-unbound` on
    // FBM's side and is simply not an attempted row here.
    let mut claim: std::collections::BTreeMap<&str, Vec<usize>> = Default::default();
    for (i, (c, _)) in census.iter().enumerate() {
        if let Some(n) = c.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    for (name, bytes) in entries.iter() {
        let Some([i]) = claim.get(name.as_str()).map(Vec::as_slice) else {
            continue;
        };
        let (c, strict) = &census[*i];
        // **THE POPULATION, and it is defined by the STRICT census.** A row the
        // strict reader accepted is FBM's business and not this module's; a row
        // it refused is the "unmeasurable half" and is exactly what is attempted
        // here. This predicate is the same one `fnbytes.rs` files as
        // `fnbyte-refused-parse`, read off the same census in the same walk —
        // which is what makes `blind-population-broken` a real cross-check
        // rather than a restatement.
        if strict.is_ok() {
            continue;
        }
        // The positional cross-check, per row: the relaxed pass must be looking
        // at the same function.
        let (rc, relaxed_fn) = &relaxed[*i];
        if rc.index != c.index {
            *res.emit
                .entry("fnbyte-blind-census-desync".into())
                .or_insert(0) += 1;
            continue;
        }
        attempted += 1;
        *res.emit.entry("fnbyte-blind-attempted".into()).or_insert(0) += 1;
        let v = grade_one_blind(
            relaxed_fn.as_ref().ok(),
            c.opt_word,
            Some(bytes.as_slice()),
            &tu,
        );
        *res.emit.entry(v.key().into()).or_insert(0) += 1;
        match v {
            Blind::Unlowerable(w) => {
                unlowerable += 1;
                *res.emit.entry(w.emit_key()).or_insert(0) += 1;
            }
            Blind::Differs {
                port_words,
                ref_words,
                equal_words,
            } => {
                differs += 1;
                // The forensic triple, summed. Never a credit — it is what makes
                // a `differs` population sizeable as work rather than as a count.
                *res.emit
                    .entry("fnbyte-blind-differs-port-words".into())
                    .or_insert(0) += port_words;
                *res.emit
                    .entry("fnbyte-blind-differs-ref-words".into())
                    .or_insert(0) += ref_words;
                *res.emit
                    .entry("fnbyte-blind-differs-equal-words".into())
                    .or_insert(0) += equal_words;
            }
            Blind::Exact => {
                exact += 1;
                // **WHICH REFUSAL CLASS the catalogue reached past.** The single
                // most important thing a nonzero `blind-exact` can carry: "the
                // lowering generalises" is unreadable without knowing which gate
                // it generalised past, and this repo has been wrong four times
                // about a population whose classes were never enumerated
                // (`ranking instruments measure themselves`).
                *res.emit
                    .entry(format!("fnbyte-blind-exact|{}", c.verdict.key()))
                    .or_insert(0) += 1;
            }
        }
        // The same axis on the differing side, because the two-sided price
        // §6 rule 2 requires is per-class or it is not actionable.
        if matches!(v, Blind::Differs { .. }) {
            *res.emit
                .entry(format!("fnbyte-blind-differs|{}", c.verdict.key()))
                .or_insert(0) += 1;
        }
    }
    // ---- THE TWO KNOWN-ANSWER CONTROLS ---------------------------------------
    //
    // Both are computed from THIS walk's own accumulators, in the same TU
    // iteration that filed the buckets — never by subtracting two published
    // totals, which is how `emit_blockers` came to be read as a codegen reading
    // it never was (#1464). Both are DEFECT counts whose known answer is 0, and
    // both are filed positively so that an absence cannot read as agreement.

    // (1) PARTITION. Every attempted function landed in exactly one bucket. A
    //     bucket that silently stopped being written would otherwise shrink the
    //     accounted total while the ratio kept printing.
    if exact + differs + unlowerable != attempted {
        *res.emit
            .entry("fnbyte-blind-partition-broken".into())
            .or_insert(0) += 1;
    }

    // (2) POPULATION. The blind walk must have offered the relaxed decode
    //     exactly the functions FBM files as `fnbyte-refused-parse` — the
    //     "unmeasurable half" — and no others. This is the check that says the
    //     instrument is pointed at the population it claims, and it is the
    //     difference between a measurement and a coincidence.
    //
    //     `fnbytes::measure` ran immediately before this on the same `res`, the
    //     same census and the same obj, so the comparison is against a count
    //     filed by the sibling walk over the identical entry list rather than
    //     against a doc's remembered figure.
    let refused_parse = res
        .emit
        .get("fnbyte-refused-parse")
        .copied()
        .unwrap_or_default();
    if attempted != refused_parse {
        *res.emit
            .entry("fnbyte-blind-population-broken".into())
            .or_insert(0) += 1;
        // The size and direction of the disagreement, not merely its existence:
        // an aggregate cannot distinguish `+1400/-27` from `+1373/-0`
        // (`ROADMAP_SLICING` §6 rule 3, `w-empty`'s first attempt).
        *res.emit
            .entry(if attempted > refused_parse {
                "fnbyte-blind-population-over".into()
            } else {
                "fnbyte-blind-population-under".to_string()
            })
            .or_insert(0) += attempted.abs_diff(refused_parse);
    }
    // The ladder depth this TU was graded at, recorded per TU so a report cannot
    // sum two scans taken at different depths without the sum being visible.
    *res.emit
        .entry(format!("fnbyte-blind-level|{}", Relax::level(level).name()))
        .or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty TU context. Every case below refuses before the composition, so
    /// the context is never consulted — but it is real rather than mocked, for
    /// the reason `CLAUDE.md` gives about never faking either side.
    fn ctx() -> TuContext<'static> {
        TuContext::of(&[])
    }

    /// **The known-answer control on the reference side.** A COMDAT with no
    /// readable bytes is its own bucket and never a refusal — folding it in
    /// would let the residue absorb a measurable answer.
    #[test]
    fn a_reference_with_no_bytes_is_its_own_bucket() {
        assert_eq!(
            grade_one_blind(None, None, None, &ctx()),
            Blind::Unlowerable(Why::NoRefBytes)
        );
    }

    /// **No candidate is `no-decode`, and it is NOT `differs`.** The distinction
    /// is the whole reading of S0: `no-decode` is `ROADMAP_SLICING` §3's 4a(i)
    /// half and `differs` is a wrong emit the next `functions()` widening would
    /// ship. An instrument that reported the first as the second would price the
    /// two-sided hazard at a number it invented.
    #[test]
    fn no_candidate_is_no_decode_and_never_differs() {
        let v = grade_one_blind(None, None, Some(&[0x4e, 0x80, 0x00, 0x20]), &ctx());
        assert_eq!(v, Blind::Unlowerable(Why::NoDecode));
        assert_eq!(v.key(), "fnbyte-blind-unlowerable");
        assert_ne!(v.key(), "fnbyte-blind-differs");
    }

    /// Every `Why` prints a distinct key, and `ALL` really does enumerate them.
    /// A bucket that silently stopped being written would otherwise shrink the
    /// accounted total while the ratio kept printing.
    #[test]
    fn every_unlowerable_reason_has_its_own_printed_key() {
        let keys: std::collections::BTreeSet<&str> = Why::ALL.iter().map(|w| w.key()).collect();
        assert_eq!(keys.len(), Why::ALL.len(), "two reasons share a key");
        assert_eq!(Why::ALL.len(), 4);
    }

    /// **`fnbyte-blind-exact` is not `fnbyte-exact`, and the key names say so.**
    /// The two predicates differ — FBM requires byte AND relocation identity,
    /// blind requires bytes alone — so a reader that summed them would be adding
    /// two different questions. The namespace is the guard.
    #[test]
    fn the_keys_are_namespaced_and_never_collide_with_fbm() {
        for k in [
            Blind::Exact.key(),
            Blind::Differs {
                port_words: 0,
                ref_words: 0,
                equal_words: 0,
            }
            .key(),
            Blind::Unlowerable(Why::NoDecode).key(),
        ] {
            assert!(k.starts_with("fnbyte-blind-"), "{k} is not namespaced");
            assert_ne!(k, "fnbyte-exact");
            assert_ne!(k, "fnbyte-differs");
            assert_ne!(k, "fnbyte-refused-parse");
        }
    }

    /// **THE TWO SPELLINGS COME FROM ONE PLACE.** This test exists because they
    /// did not: the printed block looked the flat `gap-metric` spelling up in
    /// the per-TU map, found nothing, and printed `no-decode 0` on a scan whose
    /// own machine line read 113,165 for the same fact. An absence read as a
    /// zero, caught only because two outputs of one scan disagreed.
    #[test]
    fn the_per_tu_key_and_the_metric_key_agree_on_the_reason() {
        for w in Why::ALL {
            assert_eq!(w.emit_key(), format!("fnbyte-blind-unlowerable|{}", w.key()));
            assert_eq!(w.metric_key(), format!("fnbyte-blind-unlowerable-{}", w.key()));
            // The two spellings differ — that is the hazard — but they differ
            // ONLY in the separator, so neither can drift into a new name.
            assert_ne!(w.emit_key(), w.metric_key());
            assert_eq!(w.emit_key().replace('|', "-"), w.metric_key());
        }
    }

    /// The ladder depth is a **named parameter with a stated default**, and an
    /// unparseable value is refused rather than silently defaulted.
    #[test]
    fn the_ladder_depth_is_a_named_parameter() {
        assert_eq!(DEFAULT_LEVEL, 1);
        assert_eq!(LEVEL_ENV, "C2RS_BLIND_LEVEL");
        // Level 0 is the identity control and is reachable.
        assert_eq!(Relax::level(0), Relax::STRICT);
        assert_ne!(Relax::level(DEFAULT_LEVEL), Relax::STRICT);
    }
}

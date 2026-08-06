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
//! | `fnbyte-partial` | the port selected, and the **port's own `/Gy` composition** declined this body (a pooled FP constant; a frame or call sequence the port cannot lay out). Not a harness limit — see below | no |
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
//! # Its known limit — CLOSED for four of the five shapes (board #322)
//!
//! `Selected::{Tail, Framed, Seq, CondPair}` and a `Float` with pooled constants
//! hand back a body the *COFF emitter* finishes, because the missing words
//! encode their own `.text` offset. FBM declined to grade all five, and lane
//! `w-seam` measured the size of that blind spot: **9,375 functions**
//! (`tail 7098 · seq 2150 · framed 123 · cond-pair 4`) in which a wrong emit
//! read as `differs 0`.
//!
//! **The decline reason was a statement about the PACKED emitter, and this
//! instrument's denominator is the `/Gy` COMDAT population.** Under
//! function-level linking every function starts at offset **0** of its own
//! section, so the `.text` offset the harness "cannot know" is a constant, and
//! `PortC2::build`'s `/Gy` branch has always composed these bodies completely.
//! Lane `w-fnbyte` lifted that composition into
//! [`c2_core::comdat::comdat_function_body`] and calls **it** — never a copy.
//! That is the load-bearing part: a reconstruction written here could drift from
//! the emitter, and an alarm that is green about bytes the port does not emit is
//! worse than the blind one it replaced.
//!
//! What is still declined is `Float` with pooled constants, and it is declined
//! because **the port itself refuses it under `/Gy`** (`docs/OBJ_GY_SHAPES.md`
//! §2, the reverse-order `.rdata` append) — not because the harness will not
//! reconstruct it. Zero functions in the dc3 workload are in that bucket.
//!
//! FBM is still a **floor**, for the reason §7.1 of the doc gives and not this
//! one: a `.text` COMDAT's bytes are a subset of the obj.

use c2_core::codegen::{opt_mode_of_word, select_function};
use c2_core::comdat::{comdat_body_from_selected, selected_tag, ComdatDecline};
use c2_core::elide::TuEmptyCallees;
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

/// Why a graded function has no port body — the *stage* that declined, kept
/// apart from the bucket it lands in so the per-shape census can print a
/// reason and not just a count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decline {
    /// The `.ex` optimization word is not one this port emits for.
    OptMode,
    /// `codegen::select_function` refused the function outright.
    Selector,
    /// The selector lowered it, but the `/Gy` composition has no obj model for
    /// this shape (today only a pooled floating-point constant, which the port
    /// refuses under `/Gy` — so this is the *port's* limit, not the harness's).
    GyShape,
    /// The body exists and the data-symbol relocation halves cannot be located
    /// inside it, so `PortC2::build` refuses the whole obj. The `.text` bytes
    /// are real; the function is still one the port cannot emit.
    DataRef,
}

impl Decline {
    fn key(self) -> &'static str {
        match self {
            Decline::OptMode => "opt-mode",
            Decline::Selector => "selector",
            Decline::GyShape => "gy-shape",
            Decline::DataRef => "data-ref",
        }
    }
}

/// The port's **complete** `/Gy` COMDAT body for one function, its shape tag, or
/// the reason there is none.
///
/// **It runs the port's own emitter** — [`comdat_body_from_selected`] is the
/// same composition `PortC2::build` uses under function-level linking, called
/// from `c2-core` rather than reimplemented here. Before board #322 this
/// function mapped four of the six `Selected` variants straight to
/// `FnByte::Partial` and compared no bytes at all; the module docs give the
/// reason that argument was wrong for this denominator.
fn complete_body(
    func: &IlFunction,
    opt_word: Option<u32>,
    tu: &TuEmptyCallees,
) -> Result<(&'static str, Vec<u8>), (&'static str, Decline)> {
    let mode = opt_mode_of_word(opt_word).map_err(|_| ("opt-mode", Decline::OptMode))?;
    let selected = select_function(func, mode).map_err(|_| ("refused", Decline::Selector))?;
    let shape = selected_tag(&selected);
    match comdat_body_from_selected(func, selected, mode, tu) {
        Ok(b) => {
            debug_assert_eq!(b.shape, shape);
            Ok((shape, b.text))
        }
        Err(ComdatDecline::Shape(_)) => Err((shape, Decline::GyShape)),
        Err(ComdatDecline::DataRef(_)) => Err((shape, Decline::DataRef)),
        // Unreachable: the selection already succeeded above and is handed in.
        Err(ComdatDecline::Selector(_)) => Err((shape, Decline::Selector)),
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

/// The `fnbyte-partial|…` tag for a shape whose `/Gy` composition declined.
///
/// `float-const` keeps its historical name — it is the one shape whose decline
/// is documented in `docs/FUNCTION_BYTE_MATCH.md` §3.1 and quoted elsewhere.
/// Everything else gets `-compose`, because `seq` alone would read as "the
/// instrument does not grade `seq`", which is the opposite of true.
fn compose_tag(shape: &'static str) -> &'static str {
    match shape {
        "float-const" => "float-const",
        "seq" => "seq-compose",
        "framed" => "framed-compose",
        "tail" => "tail-compose",
        "cond-pair" => "cond-pair-compose",
        other => other,
    }
}

/// One emitted symbol's verdict, with the **shape** that produced it and the
/// stage that declined when there is no body.
///
/// The shape is not decoration: board #322 is a lane about a blind spot that was
/// invisible because the instrument printed a bucket without the shape behind
/// it, and `partial by shape` was the one line that made its size legible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Graded {
    pub verdict: FnByte,
    /// The `Selected` variant tag (`plain`/`tail`/`framed`/`seq`/`cond-pair`/
    /// `float`/`float-const`), or a pseudo-tag when the port never got that far
    /// (`refused`, `opt-mode`, `unbound`, `no-bytes`).
    pub shape: &'static str,
    pub decline: Option<Decline>,
}

/// **The bundle-level context the composition needs**, built from the same
/// census rows FBM grades — see [`c2_core::elide`].
///
/// Mechanism E is the one fact in a `/Gy` body that is not a property of the
/// function alone, so it is derived from **every** census row that parsed, not
/// from the emitted subset: a callee that reduces to nothing is a callee whether
/// or not it is itself in FBM's denominator. A row that did not parse
/// contributes nothing, which is the conservative direction — the port keeps its
/// branch and the function keeps whatever verdict it had.
///
/// **It is a FIXPOINT and so the whole-bundle input matters more than it did**
/// (board #946): a row that fails to parse no longer costs only its own
/// elision, it breaks every chain that runs through it. That is still the
/// conservative direction, and it is why the row's `IlFunction` — not just its
/// `empty_body` flag — is what this hands over.
///
/// # It keys on `emit_name`, and keying on `mangled_name` was MEASURED WRONG
///
/// A census row carries two names from two bindings. `IlFunction::mangled_name`
/// is paired **positionally** over `.ex` segments; `FnCensus::emit_name` is the
/// per-record emitted-symbol binding — the one the walk below uses to decide
/// which row *is* which `.text` COMDAT. They disagree on **74,955** rows of the
/// dc3 workload, which this module now counts as `fnbyte-name-disagree` on every
/// scan rather than leaving as a paragraph in `bind.rs`.
///
/// Built from `mangled_name`, the elision fired **14** times on the workload:
/// fourteen previously byte-exact `tail` bodies turned wrong, and **not one** of
/// family A's 1,886 was reached. The names it matched were other functions'.
/// Built from `emit_name` — the binding this walk already trusts for exactly
/// this population — the elision and the instrument that grades it cannot be
/// looking at two different functions.
///
/// A row with no `emit_name` contributes nothing: it binds no emitted symbol, so
/// nothing can reach it under that name either.
pub fn tu_empty_callees(census: &[(FnCensus, Result<IlFunction, &'static str>)]) -> TuEmptyCallees {
    TuEmptyCallees::of_named(
        census
            .iter()
            .filter_map(|(c, g)| Some((c.emit_name.as_deref()?, g.as_ref().ok()?))),
    )
}

/// Grade one emitted symbol.
///
/// `row` is the unique census row that binds it (`None` when zero or two rows
/// do), `bytes` its COMDAT's raw data, `tu` the bundle's empty-bodied callees
/// (see [`tu_empty_callees`]).
pub fn grade_one(
    row: Option<&(FnCensus, Result<IlFunction, &'static str>)>,
    bytes: Option<&[u8]>,
    tu: &TuEmptyCallees,
) -> Graded {
    let g = |verdict, shape, decline| Graded {
        verdict,
        shape,
        decline,
    };
    let Some(bytes) = bytes else {
        return g(FnByte::NoBytes, "no-bytes", None);
    };
    let Some((census, gate)) = row else {
        return g(FnByte::Unbound, "unbound", None);
    };
    let Ok(func) = gate else {
        return g(FnByte::Refused, "parse-refused", Some(Decline::Selector));
    };
    match complete_body(func, census.opt_word, tu) {
        Ok((shape, port)) => g(compare_body(&port, bytes), shape, None),
        // The `/Gy` composition has no obj model for the shape. It keeps the
        // `partial` bucket because the body's bytes were never compared — and
        // the tag says `-compose` for every shape but the pooled FP constant, so
        // `partial by shape: seq 2` cannot be misread as "the instrument is
        // still blind to `seq`". It is blind to two `seq` bodies whose own
        // composition the port declined; the other 222 are graded.
        Err((shape, Decline::GyShape)) => {
            g(FnByte::Partial(compose_tag(shape)), shape, Some(Decline::GyShape))
        }
        // Everything else is a refusal: the port does not emit this function.
        Err((shape, d)) => g(FnByte::Refused, shape, Some(d)),
    }
}

/// **The byte-level witness for one differing function**: the index of the first
/// disagreeing word and the two words themselves, as `first@<i>:port=<hex>,ref=
/// <hex>`.
///
/// Recomputes the port body — this runs only on the `differs` path, whose known
/// answer is 0, so the cost is paid exactly when something is wrong and the
/// evidence is worth more than the cycles. `-` when there is no body at all
/// (unreachable from the `Differs` arm; representable rather than a panic).
fn differ_witness(
    row: Option<&(FnCensus, Result<IlFunction, &'static str>)>,
    reference: &[u8],
    tu: &TuEmptyCallees,
) -> String {
    let Some((census, Ok(func))) = row else {
        return "first@-".to_string();
    };
    let Ok((_, port)) = complete_body(func, census.opt_word, tu) else {
        return "first@-".to_string();
    };
    let hex = |b: &[u8]| -> String {
        b.iter()
            .map(|x| format!("{x:02x}"))
            .collect::<Vec<_>>()
            .join("")
    };
    let n = port.len().min(reference.len()) / 4;
    for i in 0..n {
        let (a, b) = (&port[i * 4..i * 4 + 4], &reference[i * 4..i * 4 + 4]);
        if a != b {
            return format!("first@{i}:port={},ref={}", hex(a), hex(b));
        }
    }
    // Every common word agrees: the bodies differ only in LENGTH, which is a
    // distinct failure and must not be reported as a word mismatch.
    format!(
        "len:port={}B,ref={}B",
        port.len(),
        reference.len()
    )
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
    // **Mechanism E's one bundle-level input**, resolved once per TU over every
    // census row that parsed — never per function, and never from the emitted
    // subset alone. See [`tu_empty_callees`] and `c2_core::elide`.
    let tu = tu_empty_callees(census);
    *res.emit.entry("fnbyte-tu-empty-callees".into()).or_insert(0) += tu.len();
    // **The control on the input the elision reads.** A census row carries TWO
    // names from TWO different bindings: `IlFunction::mangled_name`, paired
    // POSITIONALLY over `.ex` segments (`bind.rs`'s own module doc pins that
    // disagreement), and `FnCensus::emit_name`, the per-record emitted-symbol
    // binding this walk uses to decide which row IS which COMDAT. Where they
    // differ, a name-keyed fact read off the first one is attached to the wrong
    // function — so the size of the disagreement is counted on every scan rather
    // than assumed to be zero.
    for (c, g) in census {
        if let (Some(en), Ok(f)) = (c.emit_name.as_deref(), g.as_ref()) {
            if !f.mangled_name.is_empty() && f.mangled_name != en {
                *res.emit.entry("fnbyte-name-disagree".into()).or_insert(0) += 1;
            }
        }
    }
    let mut claim: std::collections::BTreeMap<&str, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    // **The gap between "the bytes match" and "the function matches".** A
    // `.text` COMDAT's raw data does not contain its relocations, so two bodies
    // that load the address of two DIFFERENT globals are byte-identical here and
    // differ in the obj. FBM's `exact` bucket therefore credits a body whose
    // relocation targets it never checked, and the size of that gap is the
    // number of credited functions that relocate at all.
    //
    // Measured on every scan rather than left as a caveat, and counted only for
    // the CREDITED bucket, because that is the only place it can mislead. `None`
    // is a decode failure and is its own printed row — never a zero.
    let relocs: std::collections::BTreeMap<String, usize> = match ref_obj.text_comdat_reloc_counts()
    {
        Some(v) => v.into_iter().collect(),
        None => {
            *res.emit
                .entry("fnbyte-reloc-counts-unreadable".into())
                .or_insert(0) += 1;
            Default::default()
        }
    };
    let mut accounted = 0usize;
    // **The byte-fraction ranker's accumulators** (lane w-tu3, board #500). Same
    // walk, same denominator source, one extra unit: `.text` COMDAT *bytes*
    // rather than COMDAT *count*. See `byte_fraction` below for what the ratio
    // is and what it may not be used for.
    let mut byte_den = 0usize;
    let mut byte_accepted = 0usize;
    let mut byte_exact = 0usize;
    let mut byte_differs = 0usize;
    let mut byte_refused = 0usize;
    let mut byte_unaccounted = 0usize;
    for (name, bytes) in &entries {
        let row = match claim.get(name.as_str()).map(Vec::as_slice) {
            Some([i]) => Some(&census[*i]),
            _ => None,
        };
        let graded = grade_one(row, Some(bytes.as_slice()), &tu);
        let v = graded.verdict;
        *res.emit.entry(v.key()).or_insert(0) += 1;
        if v.bare() != v.key() {
            *res.emit.entry(v.bare().into()).or_insert(0) += 1;
        }
        // **The per-shape census** (board #322). Two rows per function: what the
        // port selected, and what the judge said about it. Before this lane the
        // first row existed only for the `partial` bucket, so "which shapes is
        // the alarm blind to" could be answered and "which shapes is it now
        // GRADING, and with what verdict" could not.
        *res.emit
            .entry(format!("fnbyte-shape|{}", graded.shape))
            .or_insert(0) += 1;
        *res.emit
            .entry(format!("fnbyte-shape|{}|{}", graded.shape, v.bare()))
            .or_insert(0) += 1;
        if let Some(d) = graded.decline {
            *res.emit
                .entry(format!("fnbyte-decline|{}", d.key()))
                .or_insert(0) += 1;
        }
        // **MECHANISM E, counted where it fires and split by the judge's own
        // verdict** (`c2_core::elide`). A delta between two scans is not a
        // measurement of how often a rule fired — it is a measurement of the net
        // — so the count is positive, printed, and carries the verdict beside
        // it: `fnbyte-elided` is how many bodies the elision produced and
        // `fnbyte-elided-exact` how many of those c2 agrees with. The two being
        // equal is the claim; the pair being printed is what makes a future
        // divergence visible instead of arithmetic.
        //
        // Calls the port's OWN predicate, never a copy of it — the same rule
        // `comdat_body_from_selected` just applied, for the same reason that
        // composition is called rather than reimplemented here.
        if let Some((_, Ok(f))) = row {
            if c2_core::elide::drops_tail_call(f, &tu) && graded.shape == "tail" {
                *res.emit.entry("fnbyte-elided".into()).or_insert(0) += 1;
                if v == FnByte::Exact {
                    *res.emit.entry("fnbyte-elided-exact".into()).or_insert(0) += 1;
                }
            }
        }
        accounted += 1;
        byte_den += bytes.len();
        match v {
            FnByte::Exact => {
                byte_exact += bytes.len();
                byte_accepted += bytes.len();
            }
            FnByte::Partial(_) => byte_accepted += bytes.len(),
            FnByte::Differs { .. } => byte_differs += bytes.len(),
            FnByte::Refused => byte_refused += bytes.len(),
            FnByte::Unbound | FnByte::NoBytes => byte_unaccounted += bytes.len(),
        }
        if v == FnByte::Exact && relocs.get(name.as_str()).copied().unwrap_or(0) > 0 {
            *res.emit.entry("fnbyte-exact-relocated".into()).or_insert(0) += 1;
        }
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
            // **THE WITNESS.** A count cannot be acted on; a differ has to be
            // reproducible from the scan's own output, by name and by word.
            // Board #232/#259/#263/#276 were each closed from a named
            // reproducer, and the first thing a lane needs is which function and
            // which word. One key per differing function, value 1.
            *res.emit
                .entry(format!(
                    "fnbyte-differs-fn|{}|w{port_words}/{ref_words}/eq{equal_words}|{}|{name}",
                    graded.shape,
                    differ_witness(row, bytes, &tu),
                ))
                .or_insert(0) += 1;
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
    // **The byte-fraction ranker's per-TU record** (board #500).
    //
    // Written unconditionally, including the zeros: a TU whose every emitted
    // function the port refuses records `bytefrac-accepted 0` beside a positive
    // `bytefrac-denominator`, and that is a 0 % score rather than an absent one.
    // A TU with **no** `.text` bytes at all records neither, and is counted under
    // `bytefrac-no-denominator` — see `byte_fraction`, which returns `None` for
    // it rather than a ratio.
    if byte_den == 0 {
        *res.emit
            .entry("bytefrac-no-denominator".into())
            .or_insert(0) += 1;
    } else {
        *res.emit.entry("bytefrac-denominator".into()).or_insert(0) += byte_den;
        *res.emit.entry("bytefrac-accepted".into()).or_insert(0) += byte_accepted;
        *res.emit.entry("bytefrac-exact".into()).or_insert(0) += byte_exact;
        *res.emit.entry("bytefrac-differs".into()).or_insert(0) += byte_differs;
        *res.emit.entry("bytefrac-refused".into()).or_insert(0) += byte_refused;
        *res.emit
            .entry("bytefrac-unaccounted".into())
            .or_insert(0) += byte_unaccounted;
    }
    // The byte partition identity, as a POSITIVE check with a printed count —
    // the same discipline `fnbyte-partition-broken` applies to the COMDAT count.
    // `accepted` here includes `exact`, so `exact` is deliberately NOT a summand.
    if byte_accepted + byte_differs + byte_refused + byte_unaccounted != byte_den {
        *res.emit
            .entry("bytefrac-partition-broken".into())
            .or_insert(0) += 1;
    }
    if byte_exact > byte_accepted {
        *res.emit
            .entry("bytefrac-exact-exceeds-accepted".into())
            .or_insert(0) += 1;
    }
}

/// **THE BYTE-FRACTION RANKER** (lane `w-tu3`, board **#500**) — how much of a
/// TU's `.text`, *by byte*, the port already produces a body for.
///
/// # Why the unit is bytes and not functions
///
/// Board **#465** priced a frontier TU by how many of its emitted **functions**
/// the port already covers, and it was **refuted by the TU pre-registered to
/// confirm it** (`rungs/_2026-08-05-w-tu2.md` §3.1). `src/xdk/nuispeech/mmio.cpp`
/// scores **8 of 11 = 72.7 %** by function — the best on the frontier — and
/// **64 of 380 = 16.8 %** by byte, because those eight functions are 8-byte
/// `li r3,0 ; blr` stubs. `src/xdk/nuispeech/xboxmem.cpp`, the one TU ever
/// converted from per-function codegen breadth, scores **50 % by function** and
/// **54.5 % by byte**. The function metric ranks `mmio` *above* `xboxmem`; the
/// byte metric ranks `xboxmem` **3.2× above** `mmio`, **and the byte metric is
/// the one that got the outcome right**. #465 was right in instinct and wrong in
/// unit — the same defect as #269, which counted refusals and could not see what
/// was already emitted, one level along.
///
/// **n = 2.** Two hand-counted cells and two outcomes. This function reproduces
/// them from the objs rather than from prose, which is a different claim from
/// validating them.
///
/// # Returns `None`, never 100 %, on an empty denominator
///
/// `objdiff`'s `calc_fuzzy_match_percent` returns `100.0` over zero code bytes
/// (`objdiff-core/src/bindings/report.rs:249-250`) — absence read as success,
/// this project's most-repeated defect, refused by construction in the module
/// docs above and refused again here. A TU with no `.text` COMDAT bytes has no
/// byte fraction; it is counted under `bytefrac-no-denominator` and this returns
/// `None`. Every caller prints the denominator beside the ratio.
///
/// # It is an INSTRUMENT and never a gate
///
/// It licenses no emit, appears in no accept/refuse path, and its numerator is
/// **not** raisable by emitting bytes `c2` would not have written: the
/// denominator is a function of the *reference* obj alone, and a body the judge
/// has already called wrong (`FnByte::Differs`) is credited **nowhere** — it
/// lands in `bytefrac-differs`, which is an alarm, so a wrong emit *lowers* this
/// score. The one remaining lever is `FnByte::Partial`, whose bodies the COFF
/// emitter finishes and which the harness deliberately does not reconstruct
/// (module docs); `bytefrac-exact` is printed beside every ratio as the
/// oracle-graded floor under it.
///
/// Returns `(accepted, denominator)` in bytes.
pub fn byte_fraction(res: &TuResult) -> Option<(usize, usize)> {
    let den = res.emit.get("bytefrac-denominator").copied().unwrap_or(0);
    if den == 0 {
        return None;
    }
    Some((
        res.emit.get("bytefrac-accepted").copied().unwrap_or(0),
        den,
    ))
}

/// The oracle-graded floor under [`byte_fraction`]'s numerator: bytes the port
/// reproduces **byte-identically**, as opposed to bytes it merely selects a
/// shape for. Quoted with the ratio so the size of the ungraded part is never a
/// rumour — the same role `fnbyte-partial` plays for FBM.
pub fn byte_fraction_exact(res: &TuResult) -> usize {
    res.emit.get("bytefrac-exact").copied().unwrap_or(0)
}

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
use c2_core::elide::Reduction;
use c2_core::splice::TuContext;
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
    tu: &TuContext,
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

/// **Does a spliced body's relocation set agree with the reference obj's?**
///
/// The check FBM does not do and cannot: a `.text` COMDAT's raw data does not
/// contain its relocations, so two bodies that are both the word `48000000`
/// against two different targets compare `exact` (board **#882**,
/// `fnbyte-exact-relocated` = 4,664). Lane `w-splice` ships a rule that replaces
/// a caller's relocations with its callee's, so for the functions it moves that
/// gap is the whole question.
///
/// Compared as **(target name, in-section offset)** sets, per symbol:
///
/// * the port side is the spliced [`c2_core::comdat::ComdatBody`]'s `calls` and
///   `data_refs` — the relocation sites `PortC2::build` would register;
/// * the reference side is the reference obj's own relocation records for the
///   **same COMDAT**, `PAIR` records excluded because a `PAIR`'s
///   `SymbolTableIndex` is a displacement rather than an index and it always
///   accompanies the `REFHI`/`REFLO` already counted.
///
/// The verdict is a short stable string so a scan can histogram it. `no-relocs`
/// is printed rather than folded into `ok`: "both sides are empty" and "both
/// sides carry the same three targets" are different observations and one of
/// them is a much weaker statement.
fn reloc_verdict(
    body: &c2_core::comdat::ComdatBody<'_>,
    reference: Option<&Vec<(u32, u16, Option<String>)>>,
) -> (String, String) {
    let Some(reference) = reference else {
        // The reference obj's relocation table did not decode. Never `ok`.
        return ("ref-unreadable".to_string(), String::new());
    };
    let mut port: Vec<(String, u32)> = body
        .calls
        .iter()
        .map(|c| (c.callee.to_string(), c.reloc_offset))
        .chain(
            body.data_refs
                .iter()
                .flat_map(|d| data_ref_sites(d)),
        )
        .collect();
    let mut refs: Vec<(String, u32)> = reference
        .iter()
        .filter(|(_, ty, target)| {
            *ty & c2_obj::IMAGE_REL_PPC_TYPEMASK != c2_obj::IMAGE_REL_PPC_PAIR
                && target.is_some()
        })
        .map(|(va, _, target)| (target.clone().unwrap_or_default(), *va))
        .collect();
    port.sort();
    refs.sort();
    if port == refs {
        return if port.is_empty() {
            ("no-relocs".to_string(), String::new())
        } else {
            (format!("ok|n{}", port.len()), String::new())
        };
    }
    // A disagreement is named by WHAT disagrees, because the two failures need
    // different work: a target-name mismatch is the #882 hazard landing, and an
    // offset mismatch is a body whose relocation sites moved.
    let pn: Vec<&str> = port.iter().map(|(n, _)| n.as_str()).collect();
    let rn: Vec<&str> = refs.iter().map(|(n, _)| n.as_str()).collect();
    let witness = format!("port={}|ref={}", pn.join(","), rn.join(","));
    if pn == rn {
        (format!("offset-differs|n{}", port.len()), witness)
    } else {
        (format!("target-differs|port{}|ref{}", pn.len(), rn.len()), witness)
    }
}

/// The relocation **sites** one data reference registers, as `(symbol, offset)`.
///
/// A named data symbol's address is materialized by a `lis`/`addi` pair and
/// takes a REFHI/PAIR/REFLO/PAIR quad, of which two records name the symbol.
/// Only those two are compared, for the reason [`reloc_verdict`] gives about
/// `PAIR`.
fn data_ref_sites(d: &c2_core::coff::DataRef<'_>) -> Vec<(String, u32)> {
    vec![(d.name.to_string(), d.hi_off), (d.name.to_string(), d.lo_off)]
}

/// The census row's `.ex` optimization word, or `None` when no row binds this
/// emitted symbol — the same three-valued answer `grade_one` reads, kept in one
/// place so the splice counter and the composition cannot disagree about which
/// mode they asked under.
fn census_opt_word(
    row: Option<&(FnCensus, Result<IlFunction, &'static str>)>,
) -> Option<u32> {
    row.and_then(|(c, _)| c.opt_word)
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
pub fn tu_empty_callees<'a>(
    census: &'a [(FnCensus, Result<IlFunction, &'static str>)],
) -> TuContext<'a> {
    // **One row per emitted symbol this TU binds, and the row says what each
    // mechanism can make of it** (`c2_core::splice::TuContext::of_rows`).
    //
    // `Some(Reduction::Parsed)`      both mechanisms: E may close through it,
    //                                the splice may compose a chain end out of
    //                                it.
    // `Some(Reduction::NoEffectCall)` **board #980**, lane `w-inl0` — a row the
    //                                parser REFUSED whose grammar still proves
    //                                it emits nothing but a call. E gets the
    //                                edge; the splice gets nothing, because
    //                                there are no bytes to splice.
    // `None`                         a refused row neither mechanism can use —
    //                                and it is still passed, because
    //                                `TuContext::mentions` is what tells a
    //                                chain that ENDED from one the port could
    //                                not FOLLOW. A refused row missing from the
    //                                context reads as an external, and
    //                                `S6-chain-truncated` would stop firing.
    //
    // #980's conservative direction is preserved exactly: only a refused row
    // with a readable `no_effect_callee` contributes an E edge, its verdict
    // does not move — still `Blocked`, still `fnbyte-refused` — and
    // `IlBundle::functions` still refuses its TU.
    TuContext::of_rows(census.iter().filter_map(|(c, g)| {
        let name = c.emit_name.as_deref()?;
        let reduction = match g.as_ref().ok() {
            Some(f) => Some(Reduction::Parsed(f)),
            None => c.no_effect_callee.as_deref().map(Reduction::NoEffectCall),
        };
        Some((name, reduction, c.opt_word))
    }))
}

/// Grade one emitted symbol.
///
/// `row` is the unique census row that binds it (`None` when zero or two rows
/// do), `bytes` its COMDAT's raw data, `tu` the bundle's empty-bodied callees
/// (see [`tu_empty_callees`]).
pub fn grade_one(
    row: Option<&(FnCensus, Result<IlFunction, &'static str>)>,
    bytes: Option<&[u8]>,
    tu: &TuContext,
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
/// At most this many `fnbyte-calltarget-witness|…` keys per TU. A witness is
/// worth its bytes only until the population repeats; the *counts* beside it are
/// unbounded and are what a lane sizes work off.
const MAX_CALLTARGET_WITNESSES: usize = 4;

/// **Whom the PORT's body would call**, in emitted order, as mangled names —
/// `(offset, callee)` per `REL24` site.
///
/// The other side of `ObjImage::text_comdat_call_targets`, and the whole point
/// is that bytes cannot carry it: a `/Gy` placeholder displacement is
/// `-(offset)` for every callee alike.
///
/// **Taken from [`c2_core::comdat::comdat_function_body`]'s own `calls` list,
/// never re-derived from the `IlFunction`.** A second walk over
/// `tail_call` / `call_seq` / `cond_pair` / `framed_call` would be a *copy* of
/// the writer's relocation rule, and the two would drift — mechanism E alone
/// (`Selected::Tail` with `drops_tail_call`) emits no branch and no `REL24` at
/// all, so a copy that forgot it would report 1,516 calls that are not there.
/// `docs/GAPS.md` §6 "one fact, one locator"; the same discipline
/// `comdat_body_from_selected` is called for rather than reimplemented above.
///
/// `None` is the port having no body here (refused / declined), which is a
/// different answer from "calls nothing" and is filed as `ungraded`.
fn port_call_targets(
    f: &IlFunction,
    opt_word: Option<u32>,
    tu: &TuContext,
) -> Option<Vec<(u32, String)>> {
    let mode = opt_mode_of_word(opt_word).ok()?;
    let body = c2_core::comdat::comdat_function_body(f, mode, tu).ok()?;
    Some(
        body.calls
            .iter()
            .map(|c| (c.reloc_offset, c.callee.to_string()))
            .collect(),
    )
}

fn differ_witness(
    row: Option<&(FnCensus, Result<IlFunction, &'static str>)>,
    reference: &[u8],
    tu: &TuContext,
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

// ---------------------------------------------------------------------------
// THE RESIDUAL-DIFFER FORENSICS (lane `w-seq`, boards #966–#970)
//
// `fnbyte-differs` is 3,195 and the witness key says *what* the bytes are. It
// does not say *why*, and "why" here has exactly two candidate mechanisms
// (`docs/INLINE_PREDICATE.md` §0), which need different rungs:
//
//   I  c2 expanded a same-TU callee into this caller, so the reference body
//      CONTAINS the callee's code and the port emits a branch to it;
//   E  the callee reduces to nothing in c2 and the port cannot say so, because
//      the callee's IL body is refused by a named production (§1.4's 370).
//
// Both are statements about **the callee**, so the forensics below resolve the
// port's own callee set against this TU's census rows and publish the
// disposition of each. Everything here is additive: it writes only new keys and
// reads no existing one.
// ---------------------------------------------------------------------------

/// Every callee the port's own selection names, in selection order.
///
/// #644: read off the decoded fields, never off a byte offset. A shape that
/// carries no callee returns empty, which is a *printed* class (`no-callee`)
/// rather than an absence — `docs/STATUS.md` trap 5.
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

/// One callee's **disposition** in the TU that names it.
///
/// `extern` is the honest answer for a callee no census row of this TU binds —
/// neither mechanism can be about it, because c2 has no body to expand or to
/// find empty. `refused:<production>` is family (b)'s whole content: the
/// production is the price, and naming it is what makes the family a work list.
fn callee_disposition(
    callee: &str,
    claim: &std::collections::BTreeMap<&str, Vec<usize>>,
    census: &[(FnCensus, Result<IlFunction, &'static str>)],
    tu: &TuContext,
    refbytes: &std::collections::BTreeMap<&str, &[u8]>,
) -> String {
    let Some([i]) = claim.get(callee).map(Vec::as_slice) else {
        // Zero rows bind it (external), or two do — the second is `unbound`'s
        // own ambiguity and is kept apart from it rather than folded in.
        return match claim.get(callee).map(Vec::len) {
            None => "extern".to_string(),
            Some(_) => "ambiguous".to_string(),
        };
    };
    match &census[*i].1 {
        // The gate's own `&'static str` is the coarse `blocked`; the PRODUCTION
        // is `FnVerdict::key()`, the blocking-feature key the census histogram
        // is ranked by — which is what a widening rung is priced in. Reading the
        // coarse one printed `refused:blocked` 1,774 times and named nothing.
        Err(_) => format!("refused:{}", census[*i].0.verdict.key()),
        Ok(g) => {
            if tu.reduces_to_nothing(callee) {
                "reduces".to_string()
            } else if g.empty_body {
                "empty".to_string()
            } else {
                // **Can the port lower the callee?** This is the question that
                // prices family (a): a splice is only available if the port has
                // bytes to splice. Graded by the judge — the port's own `/Gy`
                // body for the callee against c2's COMDAT for it in this obj.
                match (
                    complete_body(g, census[*i].0.opt_word, tu),
                    refbytes.get(callee).copied(),
                ) {
                    (Ok((_, p)), Some(r)) if p == r => "body:exact".to_string(),
                    (Ok(_), Some(_)) => "body:differs".to_string(),
                    (Ok(_), None) => "body:no-comdat".to_string(),
                    (Err(_), _) => "body:nocompose".to_string(),
                }
            }
        }
    }
}

/// **SPLICE-P**, evaluated against real c2's own obj.
///
/// > For a caller the port lowers to a body ending in one branch word, the
/// > emission c2 produces is the port's setup with that branch word replaced by
/// > **c2's own emitted body for the callee**:
/// > `splice = port[..len-4] ++ ref_body(callee)`.
///
/// The hypothesis is graded by the sole judge on the whole workload rather than
/// on hand cells, because the reference obj carries *both* COMDATs. What it
/// cannot say is whether the **port** could produce those bytes — that is the
/// callee's own FBM verdict, published beside this one.
///
/// Returns `None` when no splice can be formed (no single callee, or the
/// reference obj has no COMDAT for it), which is a counted class and never a
/// silent skip.
fn splice_of(port: &[u8], callee_ref: &[u8]) -> Option<Vec<u8>> {
    if port.len() < 4 {
        return None;
    }
    let mut v = port[..port.len() - 4].to_vec();
    v.extend_from_slice(callee_ref);
    Some(v)
}

/// Does `hay` contain `needle` as a **contiguous word run**?
///
/// The weaker question SPLICE-P's concatenation cannot answer for a multi-call
/// body: *is the callee's code in there at all*. Word-aligned by construction —
/// PPC is fixed width and a byte-aligned match would be an artefact.
fn contains_words(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > hay.len() {
        return false;
    }
    (0..=(hay.len() - needle.len()))
        .step_by(4)
        .any(|i| &hay[i..i + needle.len()] == needle)
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
    // Cloned once: the diff-signature rows name their TU, and `res` is mutably
    // borrowed for the whole walk below.
    let src_name = res.src.clone();
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
    // **Board #980 — the dead-temporary reader, counted where it fires and
    // graded against the judge's own bytes.** Three positive counts, because
    // "the reader recognized N bodies" and "the fixpoint admitted N" and "c2
    // agrees about N" are three different claims and a lane that prints only the
    // last cannot say which of the first two moved:
    //
    // * `-noeffect-rows` — refused census rows the reader read.
    // * `-noeffect-admitted` — of those, the ones whose callee actually reduces
    //   to nothing, i.e. the ones the fixpoint took.
    // * `-noeffect-ref-blr` / `-noeffect-ref-other` — **the known answer.** For
    //   an admitted row, c2's own `.text` COMDAT must be the single word
    //   `4e800020`. `-ref-other` is the alarm, and it is printed rather than
    //   inferred from a subtraction. A row c2 emits no COMDAT for at all is in
    //   neither: it is `-noeffect-ref-absent`.
    for (c, _) in census {
        let Some(callee) = c.no_effect_callee.as_deref() else {
            continue;
        };
        *res.emit.entry("fnbyte-noeffect-rows".into()).or_insert(0) += 1;
        if !tu.reduces_to_nothing(callee) {
            // **Why the chain stopped**, split so the residue is attributable
            // rather than a single number. The reader firing and the fixpoint
            // taking it are two different events, and the gap between them is
            // the callee's own disposition — exactly the axis `w-seq` §2 had to
            // add to make "1,774 name a refused callee" mean anything.
            let found = census
                .iter()
                .find(|(c2, _)| c2.emit_name.as_deref() == Some(callee));
            let key = match found {
                None => "fnbyte-noeffect-callee-unbound",
                Some((_, Ok(_))) => "fnbyte-noeffect-callee-parsed-live",
                Some((_, Err(_))) => "fnbyte-noeffect-callee-refused",
            };
            *res.emit.entry(key.into()).or_insert(0) += 1;
            // **The road ahead, by production.** When the chain stops at a
            // refused callee, name the production it stopped at — the same
            // widening-order histogram the census itself is, restricted to the
            // one population this rule is blocked on. Without it the residue is
            // a single number and the next rung has nothing to aim at.
            if let Some((c2, Err(_))) = found {
                *res.emit
                    .entry(format!("fnbyte-noeffect-stop|{}", c2.verdict.key()))
                    .or_insert(0) += 1;
            }
            continue;
        }
        *res.emit.entry("fnbyte-noeffect-admitted".into()).or_insert(0) += 1;
        let key = match c
            .emit_name
            .as_deref()
            .and_then(|n| entries.iter().find(|(e, _)| e == n))
        {
            Some((_, b)) if b.as_slice() == [0x4E, 0x80, 0x00, 0x20] => "fnbyte-noeffect-ref-blr",
            Some(_) => "fnbyte-noeffect-ref-other",
            None => "fnbyte-noeffect-ref-absent",
        };
        *res.emit.entry(key.into()).or_insert(0) += 1;
    }
    let mut claim: std::collections::BTreeMap<&str, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    // **The reference obj's own body for every emitted symbol**, by name (#644:
    // resolved through the COMDAT table, never by position). This is what makes
    // SPLICE-P gradeable by the sole judge on the whole workload instead of on
    // hand cells: when c2 expanded a callee into a caller, both COMDATs are
    // sitting in the same obj and the hypothesis is a byte compare.
    let refbytes: std::collections::BTreeMap<&str, &[u8]> = entries
        .iter()
        .map(|(n, b)| (n.as_str(), b.as_slice()))
        .collect();
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
    // **The relocation SITES**, for the diff signature (board #976). The counts
    // above answer "does this body relocate at all"; a signature has to answer
    // "is *this* mismatched word the one the linker owns", which needs the
    // offsets. Same fail-closed contract, same printed-not-inferred residue.
    let reloc_sites: std::collections::BTreeMap<String, Vec<(u32, u16)>> =
        match ref_obj.text_comdat_reloc_sites() {
            Some(v) => v.into_iter().collect(),
            None => {
                *res.emit
                    .entry("fndiff-reloc-sites-unreadable".into())
                    .or_insert(0) += 1;
                Default::default()
            }
        };
    // **The reference obj's relocation RECORDS, by COMDAT** — the same walk one
    // field wider: target name and in-section offset, not just a count. Read for
    // the splice's own check (`reloc_verdict`), which is the one question FBM's
    // byte compare structurally cannot answer about the bodies this port now
    // moves. `None` is a decode failure and is its own printed row, never an
    // empty map read as agreement.
    //
    // **Three relocation readers, three questions, and they are not
    // interchangeable**: `relocs` (counts) says *whether* a body relocates,
    // `reloc_sites` says *which word* the linker owns, `call_targets` says
    // *what a REL24 points at*, and this one carries the whole record —
    // offset, raw type and target — because the splice compares a body's
    // entire relocation SET against the reference's, data references included.
    #[allow(clippy::type_complexity)]
    let refrelocs: std::collections::BTreeMap<String, Vec<(u32, u16, Option<String>)>> =
        match ref_obj.text_comdat_relocs() {
            Some(v) => v.into_iter().collect(),
            None => {
                *res.emit
                    .entry("fnbyte-reloc-records-unreadable".into())
                    .or_insert(0) += 1;
                Default::default()
            }
        };
    // **The relocation TARGETS** (lane `w-drop3`, board #984) — `REL24` only, by
    // name. `reloc_sites` above says *which word* the linker owns; this says
    // *what it points at*, and nothing else in this file can tell two
    // byte-identical branch words apart. Same fail-closed contract, same
    // printed-not-inferred residue.
    let call_targets: std::collections::BTreeMap<String, Vec<(u32, String)>> =
        match ref_obj.text_comdat_call_targets() {
            Some(v) => v.into_iter().collect(),
            None => {
                *res.emit
                    .entry("fnbyte-call-targets-unreadable".into())
                    .or_insert(0) += 1;
                Default::default()
            }
        };
    let tu_label = src_name.clone();
    let mut witnesses = 0usize;
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
                    // **The relocation-target caveat, answered rather than
                    // argued** (lane `w-drop3`, boards #984-#986): a `/Gy` call
                    // word carries the same placeholder displacement whatever it
                    // calls, so byte equality on a call word says nothing about
                    // WHOM it calls, and 861 bodies FBM credits as exact
                    // relocate against the wrong symbol.
                    //
                    // An elided body cannot be one of them, and this is the
                    // positive count that says so: mechanism E's whole output is
                    // the single word `4e800020`, which is not a call word and
                    // takes no relocation. **Known answer 0** - a nonzero here
                    // means the port credited an elision for a c2 body that
                    // still relocates, which would be a wrong emit of exactly
                    // the kind `-calltarget-disagree` was built to see.
                    if relocs.get(name.as_str()).copied().unwrap_or(0) > 0 {
                        *res.emit
                            .entry("fnbyte-elided-ref-reloc".into())
                            .or_insert(0) += 1;
                    }
                }
            }
            // **MECHANISM I, counted the same way and for the same reason**
            // (`c2_core::splice`, lane w-splice). A delta between two scans
            // measures the NET; what a rule did is a positive count, printed
            // with the judge's verdict beside it. `fnbyte-spliced` is how many
            // bodies the splice produced and `fnbyte-spliced-exact` how many of
            // those c2 agrees with — the two being equal is the claim, and the
            // pair being printed is what makes a future divergence visible
            // instead of arithmetic.
            //
            // Calls the port's OWN predicate against the same context the
            // composition just used, never a copy of it.
            if let Ok(sel) = opt_mode_of_word(census_opt_word(row))
                .and_then(|m| select_function(f, m).map(|s| (m, s)))
            {
                let (m, s) = sel;
                match c2_core::splice::splice_body_why(f, &s, m, &tu) {
                    Ok(b) => {
                        *res.emit.entry("fnbyte-spliced".into()).or_insert(0) += 1;
                        *res.emit
                            .entry(format!("fnbyte-spliced|{}", graded.shape))
                            .or_insert(0) += 1;
                        // **THE RELOCATION CHECK, and it is not FBM's.**
                        //
                        // A spliced body inherits the CALLEE's relocations,
                        // resolved in the callee's context. FBM compares a
                        // `.text` COMDAT's raw bytes, which do not contain
                        // relocations, so it calls two bodies that are both
                        // `48000000` against different targets `exact` — board
                        // **#882**, 4,664 credited functions. This mechanism
                        // moves 945 functions' relocation sets at once, so the
                        // one thing FBM cannot see is exactly the thing it
                        // changes, and it is checked here per symbol rather
                        // than argued in a rung.
                        //
                        // Verdict per spliced function: the port's REL24 sites
                        // and data-symbol references against the reference
                        // obj's own relocation records for the SAME COMDAT, by
                        // target name and by in-section offset.
                        let rv = reloc_verdict(&b, refrelocs.get(name.as_str()));
                        *res.emit
                            .entry(format!("fnbyte-spliced-reloc|{}", rv.0))
                            .or_insert(0) += 1;
                        if !rv.0.starts_with("ok|") && rv.0 != "no-relocs" {
                            // A disagreement is a NAMED function with both
                            // target lists beside it, never a count: #882 was
                            // credited for 4,664 functions on a count, and the
                            // first thing a lane needs is which symbol and
                            // which target.
                            *res.emit
                                .entry(format!(
                                    "fnbyte-spliced-reloc-fn|{}|{}|{name}",
                                    rv.0, rv.1
                                ))
                                .or_insert(0) += 1;
                        }
                        if v == FnByte::Exact {
                            *res.emit.entry("fnbyte-spliced-exact".into()).or_insert(0) += 1;
                        } else {
                            // Named, never a remainder: a splice the judge
                            // rejects is the one row a net count would hide.
                            *res.emit
                                .entry(format!(
                                    "fnbyte-spliced-differs-fn|{}|{name}",
                                    graded.shape
                                ))
                                .or_insert(0) += 1;
                        }
                    }
                    // **WHICH CLAUSE REFUSED**, on the `differs` path only —
                    // where a refusal is a function the port still gets wrong,
                    // and therefore the price of the next widening. On the
                    // `exact` path a refusal is the rule correctly standing
                    // aside and counting it would drown the signal.
                    Err(d) => {
                        if matches!(v, FnByte::Differs { .. }) {
                            let why = match &d {
                                c2_core::splice::SpliceDecline::Refused(w) => *w,
                                c2_core::splice::SpliceDecline::Callee(_) => "callee-decline",
                            };
                            *res.emit
                                .entry(format!("fnbyte-splice-refused|{}|{why}", graded.shape))
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        // **Board #980's residue, priced where it lives.** A `differs` whose
        // whole reference body is one `4e800020` is a function c2 emits nothing
        // for and the port emits a call for — the cluster this lane is about.
        // For each one still standing, name the CALLEE's own blocking
        // production, and when that callee is itself a recognized no-effect body
        // name its callee's. Two levels, because the chain in this family is
        // three deep (`_Destroy_Range` → `__destroy_range` → the tag-dispatch
        // leaf) and a one-level count would say "memset" for every row and price
        // nothing.
        if matches!(v, FnByte::Differs { .. }) && bytes.as_slice() == [0x4E, 0x80, 0x00, 0x20] {
            let callee = match row {
                Some((_, Ok(f))) => f.tail_call.as_deref(),
                _ => None,
            };
            let key = |n: Option<&str>| -> String {
                match n.and_then(|n| census.iter().find(|(c, _)| c.emit_name.as_deref() == Some(n)))
                {
                    None => "callee-unbound".to_string(),
                    Some((c, _)) => c.verdict.key(),
                }
            };
            *res.emit
                .entry(format!("fnbyte-blr-stop|{}", key(callee)))
                .or_insert(0) += 1;
            let grand = callee
                .and_then(|n| census.iter().find(|(c, _)| c.emit_name.as_deref() == Some(n)))
                .and_then(|(c, _)| c.no_effect_callee.as_deref());
            if grand.is_some() {
                *res.emit
                    .entry(format!("fnbyte-blr-stop2|{}", key(grand)))
                    .or_insert(0) += 1;
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
        // **WHOM DOES THE BODY CALL** — the relocation TARGET, not the
        // relocation count (lane `w-drop3`, boards #984–#986).
        //
        // Everything above compares `.text` bytes. A `/Gy` call to a symbol
        // outside the COMDAT is emitted with the placeholder displacement
        // `-(offset of the branch word)` **whatever the callee is**, so two
        // bodies calling two different functions from the same word index carry
        // the same four bytes and every byte test above scores that word
        // `equal`. That is board **#882** — the 4,664 credited functions whose
        // relocations FBM does not check — and until this key existed it was a
        // caveat with no number.
        //
        // Compared as an ORDERED LIST OF NAMES: the port's own call set (its
        // `IlFunction`, which is what its emitter relocates from) against the
        // reference COMDAT's `REL24` targets (real c2's own symbol table).
        // Filtering to `REL24` is what makes the two sides the same question —
        // the port's list is calls, so a data reference on either side would be
        // a category error rather than a disagreement.
        //
        // **Additive and diagnostic only.** No FBM bucket moves, nothing here
        // reaches an accept path, and no emitter consults it: it is read off the
        // judge's output, so an emitter that used it would be grading itself on
        // the answer (`text_comdat_label_symbols`'s standing rule). What it
        // buys is that a *substitution under a relocation* can no longer present
        // as a *deletion* — which is exactly how 140 mechanism-I bodies came to
        // be filed as "the port omits a call c2 makes".
        if matches!(v, FnByte::Exact | FnByte::Differs { .. }) {
            // `port_call_targets` recomposes the body, so it is called ONCE and
            // the "the port has no body here" case falls through to `ungraded`
            // with the two lookups — a guard that called it and then called it
            // again in the arm would pay for every graded function twice.
            let pt = match row {
                Some((c, Ok(f))) => port_call_targets(f, c.opt_word, &tu),
                _ => None,
            };
            match (pt, call_targets.get(name.as_str())) {
                (Some(pt), Some(reftargets)) => {
                    // Compared as `(offset, name)` pairs, in emitted order. The
                    // offset is carried because a call at the right site to the
                    // wrong symbol and a call to the right symbol from the wrong
                    // site are different defects, and a name-only compare would
                    // score the second one green.
                    let port: Vec<String> =
                        pt.iter().map(|(o, n)| format!("{o:#x}:{n}")).collect();
                    let refs: Vec<String> = reftargets
                        .iter()
                        .map(|(o, n)| format!("{o:#x}:{n}"))
                        .collect();
                    *res.emit.entry("fnbyte-calltarget-graded".into()).or_insert(0) += 1;
                    if port == refs {
                        *res.emit.entry("fnbyte-calltarget-agree".into()).or_insert(0) += 1;
                    } else {
                        *res.emit
                            .entry("fnbyte-calltarget-disagree".into())
                            .or_insert(0) += 1;
                        // Split by the byte verdict, because only one half can
                        // mislead: a `differs` body was already called wrong, and
                        // an **`exact`** body with a different callee is a wrong
                        // emit the numerator is crediting.
                        let bucket = if v == FnByte::Exact { "exact" } else { "differs" };
                        *res.emit
                            .entry(format!("fnbyte-calltarget-disagree-{bucket}"))
                            .or_insert(0) += 1;
                        // Count-vs-name, because they price different work: a
                        // count disagreement is a call the port did not emit at
                        // all (mechanism I or E), a same-count name disagreement
                        // is a relocation against the wrong symbol.
                        let kind = if port.len() == refs.len() { "name" } else { "count" };
                        *res.emit
                            .entry(format!("fnbyte-calltarget-disagree-{kind}"))
                            .or_insert(0) += 1;
                        // **Witnessed only in the `exact` bucket.** A `differs`
                        // body is already named by `fnbyte-differs-fn|` and
                        // clustered by `DIFF_STRUCTURE.md`; the news here is the
                        // body the judge's byte test **credits** while its
                        // relocation points elsewhere, and mixing the two would
                        // let 3,195 already-reported rows crowd the cap.
                        if v == FnByte::Exact && witnesses < MAX_CALLTARGET_WITNESSES {
                            witnesses += 1;
                            *res.emit
                                .entry(format!(
                                    "fnbyte-calltarget-witness|{}|{}|port={}|ref={}",
                                    tu_label,
                                    name,
                                    port.join(","),
                                    refs.join(",")
                                ))
                                .or_insert(0) += 1;
                        }
                    }
                }
                // Printed rather than skipped (STATUS trap 5): a body the port
                // parsed but whose reference targets did not decode, or the
                // reverse, is not an agreement.
                _ => {
                    *res.emit
                        .entry("fnbyte-calltarget-ungraded".into())
                        .or_insert(0) += 1;
                }
            }
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
            // ---- THE RESIDUAL-DIFFER FORENSICS (lane w-seq) ----------------
            //
            // Why does this body differ? The answer is a statement about the
            // CALLEE, so the port's own callee set is resolved against this
            // TU's census rows and every disposition is published with a count.
            // Runs only on the `differs` path, like the witness above.
            if let Some((c, Ok(f))) = row {
                let callees = port_callees(f);
                let dispos: Vec<String> = callees
                    .iter()
                    .map(|cal| callee_disposition(cal, &claim, census, &tu, &refbytes))
                    .collect();
                let mut uniq: Vec<&str> = dispos.iter().map(String::as_str).collect();
                uniq.sort_unstable();
                uniq.dedup();
                let summary = if uniq.is_empty() {
                    "no-callee".to_string()
                } else {
                    uniq.join(",")
                };
                // Is c2's whole body the single word `blr`? That is mechanism
                // E's signature from the caller's side, and it is the fact that
                // separates family (b) from family (a) without an inference.
                let refblr = if bytes.as_slice() == [0x4e, 0x80, 0x00, 0x20] {
                    "refblr"
                } else {
                    "refbody"
                };
                *res.emit
                    .entry(format!(
                        "fnbyte-differs-why|{}|{}|{summary}|{refblr}|{name}",
                        graded.shape,
                        callees.len(),
                    ))
                    .or_insert(0) += 1;
                *res.emit
                    .entry(format!("fnbyte-why|{}|{summary}|{refblr}", graded.shape))
                    .or_insert(0) += 1;

                // **The #918 control, as a positive count.** The same callee
                // set is resolved a second time through the POSITIONAL name
                // binding. `emit` must resolve at least as many as `mangled`;
                // a tie means this population cannot see the disagreement and
                // that has to be said rather than assumed.
                for cal in &callees {
                    *res.emit.entry("fnbyte-callee-total".into()).or_insert(0) += 1;
                    if claim.contains_key(*cal) {
                        *res.emit
                            .entry("fnbyte-callee-resolved-emit".into())
                            .or_insert(0) += 1;
                    }
                    if census
                        .iter()
                        .any(|(_, g)| g.as_ref().is_ok_and(|x| x.mangled_name == **cal))
                    {
                        *res.emit
                            .entry("fnbyte-callee-resolved-mangled".into())
                            .or_insert(0) += 1;
                    }
                }

                // ---- SPLICE-P, graded by the reference obj's own bytes -----
                if let Ok((_, port_body)) = complete_body(f, c.opt_word, &tu) {
                    // The callee's own emitted body, from THIS obj. Resolved by
                    // name through the COMDAT table (#644), never by position.
                    let one = if callees.len() == 1 {
                        refbytes.get(callees[0]).copied()
                    } else {
                        None
                    };
                    match one {
                        None if callees.len() == 1 => {
                            *res.emit
                                .entry(format!("fnbyte-splice|{}|no-callee-comdat", graded.shape))
                                .or_insert(0) += 1;
                        }
                        None => {
                            *res.emit
                                .entry(format!("fnbyte-splice|{}|not-single-call", graded.shape))
                                .or_insert(0) += 1;
                            // **SPLICE-N** — the multi-call generalization of
                            // SPLICE-0: c2's body for the caller is the callees'
                            // own bodies laid end to end, each but the last with
                            // its trailing `blr` removed. Asked of every body
                            // that names two or more callees, so the `seq`
                            // population's other half is a measurement and not a
                            // remainder.
                            let bodies: Option<Vec<&[u8]>> =
                                callees.iter().map(|c| refbytes.get(*c).copied()).collect();
                            match bodies {
                                None => {
                                    *res.emit
                                        .entry(format!(
                                            "fnbyte-spliceN|{}|no-callee-comdat",
                                            graded.shape
                                        ))
                                        .or_insert(0) += 1;
                                }
                                Some(bs) => {
                                    let mut cat: Vec<u8> = Vec::new();
                                    let last = bs.len() - 1;
                                    for (i, b) in bs.iter().enumerate() {
                                        let t = if i != last
                                            && b.len() >= 8
                                            && b[b.len() - 4..] == [0x4e, 0x80, 0x00, 0x20]
                                        {
                                            &b[..b.len() - 4]
                                        } else {
                                            b
                                        };
                                        cat.extend_from_slice(t);
                                    }
                                    let v = if cat == *bytes {
                                        format!("exact|n{}", bs.len())
                                    } else {
                                        format!(
                                            "differs|n{}|len:cat={}w,ref={}w",
                                            bs.len(),
                                            cat.len() / 4,
                                            bytes.len() / 4
                                        )
                                    };
                                    *res.emit
                                        .entry(format!("fnbyte-spliceN|{}|{v}", graded.shape))
                                        .or_insert(0) += 1;
                                }
                            }
                        }
                        Some(cb) => {
                            // **SPLICE-0** — the degenerate hypothesis SPLICE-P
                            // reduces to when the setup is empty: c2's body for
                            // the caller IS c2's body for the callee. Asked of
                            // every single-callee shape, including `seq` and
                            // `framed`, whose port bodies carry a frame the
                            // concatenation rule cannot subtract. When it fails
                            // the first disagreeing word is what says whether
                            // inlining renamed a register or moved a
                            // displacement.
                            {
                                let v0 = if cb == bytes.as_slice() {
                                    "exact".to_string()
                                } else {
                                    let m = cb.len().min(bytes.len()) / 4;
                                    let hx = |b: &[u8]| {
                                        b.iter().map(|x| format!("{x:02x}")).collect::<String>()
                                    };
                                    match (0..m)
                                        .find(|i| cb[i * 4..i * 4 + 4] != bytes[i * 4..i * 4 + 4])
                                    {
                                        Some(i) => format!(
                                            "differs|first@{i}:callee={},ref={}",
                                            hx(&cb[i * 4..i * 4 + 4]),
                                            hx(&bytes[i * 4..i * 4 + 4])
                                        ),
                                        None => format!(
                                            "differs|len:callee={}w,ref={}w",
                                            cb.len() / 4,
                                            bytes.len() / 4
                                        ),
                                    }
                                };
                                *res.emit
                                    .entry(format!("fnbyte-splice0|{}|{v0}", graded.shape))
                                    .or_insert(0) += 1;
                                *res.emit
                                    .entry(format!(
                                        "fnbyte-splice0-fn|{}|{}|{name}",
                                        graded.shape,
                                        v0.split('|').next().unwrap_or("?")
                                    ))
                                    .or_insert(0) += 1;
                            }
                            let spl = splice_of(&port_body, cb);
                            let verdict = match &spl {
                                Some(s) if s.as_slice() == bytes.as_slice() => "exact",
                                Some(_) => "differs",
                                None => "no-body",
                            };
                            // **What the splice PERTURBS.** A count of failures
                            // cannot be acted on; the first disagreeing word and
                            // the two lengths can. Printed only on failure, the
                            // same discipline `differ_witness` follows.
                            if verdict == "differs" {
                                if let Some(s) = &spl {
                                    let m = s.len().min(bytes.len()) / 4;
                                    let at = (0..m).find(|i| {
                                        s[i * 4..i * 4 + 4] != bytes[i * 4..i * 4 + 4]
                                    });
                                    let hx = |b: &[u8]| {
                                        b.iter().map(|x| format!("{x:02x}")).collect::<String>()
                                    };
                                    let w = match at {
                                        Some(i) => format!(
                                            "first@{i}:spl={},ref={}",
                                            hx(&s[i * 4..i * 4 + 4]),
                                            hx(&bytes[i * 4..i * 4 + 4])
                                        ),
                                        None => format!(
                                            "len:spl={}w,ref={}w",
                                            s.len() / 4,
                                            bytes.len() / 4
                                        ),
                                    };
                                    *res.emit
                                        .entry(format!(
                                            "fnbyte-splice-why|{}|{w}",
                                            graded.shape
                                        ))
                                        .or_insert(0) += 1;
                                }
                            }
                            *res.emit
                                .entry(format!("fnbyte-splice|{}|{verdict}", graded.shape))
                                .or_insert(0) += 1;
                            *res.emit
                                .entry(format!(
                                    "fnbyte-splice|{}|{verdict}|pw{port_words}",
                                    graded.shape
                                ))
                                .or_insert(0) += 1;
                            *res.emit
                                .entry(format!(
                                    "fnbyte-splice-fn|{}|{verdict}|pw{port_words}/rw{ref_words}/cw{}|{name}",
                                    graded.shape,
                                    cb.len() / 4
                                ))
                                .or_insert(0) += 1;
                        }
                    }
                }
                // The weaker containment question, asked of EVERY callee and of
                // every shape: is the callee's code in the reference body at
                // all, minus its own trailing `blr`? A `seq` body's
                // concatenation is not a splice, and this is what can still be
                // measured about it.
                for cal in &callees {
                    let Some(cb) = refbytes.get(*cal).copied() else {
                        *res.emit
                            .entry(format!("fnbyte-contains|{}|no-comdat", graded.shape))
                            .or_insert(0) += 1;
                        continue;
                    };
                    let trimmed = if cb.len() >= 8 && cb[cb.len() - 4..] == [0x4e, 0x80, 0x00, 0x20]
                    {
                        &cb[..cb.len() - 4]
                    } else {
                        cb
                    };
                    let hit = contains_words(bytes.as_slice(), trimmed);
                    *res.emit
                        .entry(format!(
                            "fnbyte-contains|{}|{}",
                            graded.shape,
                            if hit { "yes" } else { "no" }
                        ))
                        .or_insert(0) += 1;
                }
            }
            // **THE DIFF SIGNATURE** (board #976, [`super::fndiff`]). The
            // witness key above names one word; this names the *structure* —
            // word-granular alignment, per-substitution field class, the
            // same-multiset bit, and whether the disagreement sits under a
            // relocation. Additive: `fndiff-` keys only, and it runs on the
            // `differs` path alone, so a scan with no differs pays nothing.
            //
            // The body is recomposed rather than threaded down from the grading
            // call, for the same reason `differ_witness` recomposes it: the cost
            // is paid exactly when something is already known to be wrong.
            if let Some((census, Ok(func))) = row {
                match complete_body(func, census.opt_word, &tu) {
                    Ok((_, port)) => {
                        let sites = reloc_sites
                            .get(name.as_str())
                            .map(Vec::as_slice)
                            .unwrap_or(&[]);
                        let sig = super::fndiff::signature(
                            &src_name,
                            name,
                            graded.shape,
                            &port,
                            bytes,
                            sites,
                        );
                        for (k, n) in sig.keys() {
                            *res.emit.entry(k).or_insert(0) += n;
                        }
                        res.fndiff.push(sig.to_json());
                    }
                    // Unreachable from this arm — the grading above produced a
                    // body. Counted rather than panicked, so a future refactor
                    // that made it reachable would show up as a number instead
                    // of a crash or a silently short census.
                    Err(_) => {
                        *res.emit
                            .entry("fndiff-body-unavailable".into())
                            .or_insert(0) += 1;
                    }
                }
            } else {
                *res.emit.entry("fndiff-body-unavailable".into()).or_insert(0) += 1;
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

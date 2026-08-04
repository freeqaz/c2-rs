//! `c2-harness::gap` — **real-workload gap scan**: run the whole pipeline
//! (capture → port → byte compare) over a list of *real* project TUs with the
//! project's *real* compile flags, and aggregate exactly where and why the
//! port falls short. This is the measuring tool that turns "the port is a toy"
//! into a ranked list of gaps to close.
//!
//! Each TU lands in exactly one class, ordered from farthest-from-goal to
//! done:
//!
//! * **`capture-fail`** — the reference pipeline itself couldn't compile the
//!   TU here (usually front-end: missing headers, flags we don't replicate).
//!   A gap in the *harness*, not the port; until it captures, the TU can't
//!   even be measured.
//! * **`vocab-gap`** — the bundle captured, but `c2-il` cannot decode its
//!   functions (unknown IL vocabulary). Gap in the IL *model*.
//! * **`codegen-gap`** — functions decode, but `PortC2` returns
//!   `NotImplemented` (feature outside the ported class). Gap in the *port*,
//!   with the codegen's own reason string as the key.
//! * **`mismatch`** — the port *emitted an obj and it differs* from real c2.
//!   The alarming class; anything here is a correctness bug, not a gap.
//! * **`match`** — byte-exact against real c2 (timestamp-normalized RAW
//!   compare; in fact the S_OBJNAME path is threaded in, so RAW-identical).
//!
//! An optional soundness lane replays every Nth captured bundle through
//! standalone c2 (`--replay-every N`) — extending the P0.1 byte-exactness
//! claim from fixtures to real workloads.
//!
//! Nothing here is a gate: the scan always reports; only harness errors (bad
//! list file etc.) fail. The scan writes one JSONL record per TU when asked,
//! for longitudinal diffing of successive scans (are we closing gaps?).

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use c2_core::{Backend, BackendError, PortC2};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

use crate::capture_cache::CaptureCache;
use crate::provenance::Provenance;

/// Scan configuration (see `c2rs gap --help` for the CLI mapping).
pub struct GapConfig {
    /// Source arguments, passed to `cl.exe` verbatim (relative to `cwd`).
    pub sources: Vec<String>,
    /// Compile flags (should include `/c`; `/Bd` is added by the capture).
    pub flags: Vec<String>,
    /// Working directory for the compiles (project root for relative paths).
    pub cwd: Option<PathBuf>,
    /// Scan at most this many TUs.
    pub limit: Option<usize>,
    /// Concurrency (worker threads; each TU gets its own work subdir).
    pub jobs: usize,
    /// Replay every Nth captured bundle through standalone c2 (0 = never).
    pub replay_every: usize,
    /// Write one JSON record per TU here.
    pub jsonl: Option<PathBuf>,
    /// Scratch root; per-TU subdirs are created below it.
    pub work: PathBuf,
    /// Reference-capture cache root (`None` = `--no-cache`). See
    /// [`crate::capture_cache`] — the key is content-addressed over source
    /// bytes, flags, toolchain and workload-tree identity, never mtimes.
    pub cache: Option<PathBuf>,
    /// Re-capture and byte-compare every Nth cache **hit** (0 = never). The
    /// bypass-and-compare validator that makes a poisoned cache detectable.
    pub validate_cache: usize,
}

/// Outcome class for one TU (see module docs for the ladder).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TuClass {
    CaptureFail,
    VocabGap,
    CodegenGap,
    PortError,
    Mismatch,
    Match,
}

impl TuClass {
    pub fn label(self) -> &'static str {
        match self {
            TuClass::CaptureFail => "capture-fail",
            TuClass::VocabGap => "vocab-gap",
            TuClass::CodegenGap => "codegen-gap",
            TuClass::PortError => "port-error",
            TuClass::Mismatch => "mismatch",
            TuClass::Match => "match",
        }
    }
}

/// Per-TU scan record.
#[derive(Clone, Debug)]
pub struct TuResult {
    /// Source argument as scanned (relative path, as passed to cl).
    pub src: String,
    pub class: TuClass,
    /// Normalized aggregation key (short, stable): cl error code for
    /// capture-fail, codegen reason for codegen-gap, etc.
    pub reason: String,
    /// Full detail line (first error line / mismatch offsets) for the JSONL.
    pub detail: String,
    /// `.ex` size in bytes (0 when capture failed).
    pub ex_len: usize,
    /// Function count per `.gl` mangled names (0 when capture failed).
    pub fn_names: usize,
    /// Standalone-c2 replay soundness check: `None` = not run this TU.
    pub replay_ok: Option<bool>,
    /// **P2b function-level census**: `.ex` function segments in this TU.
    pub fn_total: usize,
    /// How many of those parse as a modeled shape (the true in-class numerator;
    /// `fn_total - fn_in_class` are blocked).
    pub fn_in_class: usize,
    /// Blocking-feature counts for this TU's out-of-class functions.
    pub fn_blockers: BTreeMap<String, usize>,
    /// **The D6 frame measure** (`docs/IL_CALL_IN_EXPR.md` §18): census key crossed
    /// with the body's CALL-token count class, `"<calls-class>|<census key>"`, over
    /// **every** function including the in-class ones.
    ///
    /// A separate map rather than a suffix on [`TuResult::fn_blockers`]'s keys,
    /// deliberately: the ranked histogram is the widening order and four sessions
    /// of documented tables name its keys, so renaming all of them to carry an
    /// orthogonal fact would break every recorded comparison for no gain. This is
    /// the second axis, kept beside the first.
    pub fn_frames: BTreeMap<String, usize>,
    /// **The control-flow axis** (roadmap #25/#61): the body's decoded CFG shape,
    /// and — for the bodies whose statement layer decodes end to end — that shape
    /// crossed with the census key, `"<cflow class>|<census key>"`.
    ///
    /// This is the **sizing of the block-IR restructure**. Every previous estimate
    /// of that work came from summing blocker rows named after the byte a
    /// straight-line parser stopped on, which `docs/GAPS.md` §6's
    /// unstable-attribution rule says is not the shape's population: a row's size is
    /// not its yield, and a first-blocker attribution is not a shape. The cross
    /// product is, because it says of each shape *how many* bodies have it and
    /// *what else* those bodies are waiting on.
    ///
    /// The cross-tab is emitted only for a decoded body. An undecoded one
    /// contributes just the bare `cf-…` key naming where the statement-layer walk
    /// stopped — crossing "we could not read this body's control flow" with a
    /// blocker would be a product of two ignorances.
    pub fn_cflow: BTreeMap<String, usize>,
    /// **The exception-handling axis** (`docs/EH_RECORDS.md` §9.4, §10): which
    /// side of the `maxState` boundary each body falls on. Four row shapes, and
    /// the shape is in the key so no two populations can share a row:
    ///
    /// * `"<eh class>"` — the total, over every function, in class or not.
    /// * `"<eh class>|BLOCKED"` — the blocked subtotal. **This is the row to
    ///   size a rung off.**
    /// * `"<eh class>|BLOCKED|<blocker key>"` / `"<eh class>|INCLASS|<shape>"` —
    ///   the cross, with the population named. It used to be `"<eh>|<key>"` for
    ///   both, and since `FnVerdict::key` spells accepted shapes and blockers
    ///   into one namespace, the largest row of the whole cross was an
    ///   **accepted** shape (`eh-bare|empty-dtor-delegation`, 27,501) that read
    ///   exactly like a blocker.
    /// * `"eh-migrate|<maxState key>|<statement-count key>"` — the measured axis
    ///   against the refuted one it replaces, so §7.3's published split can be
    ///   reconciled rather than silently overwritten.
    ///
    /// A third axis for the same reason there is a second: **nothing in the
    /// blocking-feature key says which side a body is on.** The cheap side is a
    /// bare branch the port already emits; the EH side mints a
    /// `__CxxFrameHandler` prefix, a second `.pdata`, a `Selection = 5` `.rdata`
    /// and an unwind funclet — a whole phase of work — and the two are filed
    /// under the *same* census key in the largest population on the board
    /// (`expr-intrinsic-this-adjust`, 141,800). Ranking either without this axis
    /// is ranking the sum of two different rungs.
    ///
    /// The cross is emitted for **every** function, decoded or not, unlike
    /// [`TuResult::fn_cflow`]'s. An undecoded body is not always an ignorance
    /// here: a call already seen at a non-empty live set proves `maxState >= 1`
    /// whatever stopped the walk afterwards, so those rows read `eh-state1`.
    /// `eh-partial` is what is left — a marker, no such call yet, and the walk
    /// stopped — and it claims **nothing**, in either direction.
    pub fn_eh: BTreeMap<String, usize>,
    /// **The body-dispatch axis**: which arm of the IL parser's dispatch ladder
    /// claimed each body (`c2_il`'s `disp-*` tags). Row shapes, and the shape is in
    /// the key so no two populations can share a row — the same discipline
    /// [`TuResult::fn_eh`] had to be retrofitted with:
    ///
    /// * `"<disp>"` — the total, over every function, in class or not.
    /// * `"<disp>|BLOCKED"` — the blocked subtotal. **The row to size a rung off.**
    /// * `"<disp>|BLOCKED|<blocker key>"` / `"<disp>|INCLASS|<shape>"` — the cross,
    ///   with the population named.
    ///
    /// **What this axis says that no census key can.** A member-call construct
    /// gets an `expr-call-in-expr-recv-*` key wherever it stands, and only the
    /// bodies that *begin* with the member call ever reach the three member-call
    /// productions. The rest are a store's right-hand side (`disp-expr`) or a plain
    /// call's argument (`disp-plain-call`), and **no widening inside any of those
    /// productions can move one of them**. Ranked without this cross, those rows
    /// are indistinguishable from the ones a widening would serve — which is how a
    /// production-first-blocker table can decompose 41,292 of a 71,767-function
    /// family and leave 30,475 in a single row reading "none of the three
    /// productions was entered".
    pub fn_dispatch: BTreeMap<String, usize>,
    /// **The grammar-completeness axis** (`c2_il::Complete`, roadmap §9.11 /
    /// §9.14): is anything hiding behind this row, or is its count directly a
    /// widening estimate?
    ///
    /// Rows are `"<complete>"` totals, `"<complete>|BLOCKED"` subtotals, and
    /// `"<complete>|<census key>"` crosses.
    ///
    /// **A field, because the key is not a reliable carrier.** Two producers
    /// encode this fact in two different halves of the rendered key —
    /// `-whole`/`-more` and `:eof`/`:mid` — and WR1 moved 39,967 functions from
    /// the first encoding to the second. A ranking table built by grepping
    /// `-whole` has under-counted that family by **18,931** ever since, and
    /// §9.13 had to re-derive the join by hand to re-check a 1,399-row figure.
    /// Every consumer re-deriving a fact from a *name* is how the derivations
    /// drift; this is the fact's home.
    pub fn_complete: BTreeMap<String, usize>,
    /// **The member-call production first-blocker axis**: for the bodies that
    /// reached `try_parse_member_tail_call`, which non-committal bail inside it (or
    /// inside the chain / comparison productions it delegates to) fired.
    ///
    /// Rows are `"<prod>"` totals, `"<prod>|BLOCKED"` subtotals, and — for the
    /// bodies that actually entered a production — the `"<prod>|BLOCKED|<key>"`
    /// cross. The bare totals are emitted for **every** function including
    /// `prod-not-entered`, so the axis always sums to the census.
    ///
    /// **This is the only instrument on the board that tells a missing construct
    /// apart from a private limit inside a recognizer that already ships**, and
    /// "a private limit inside a shipping recognizer" has been the answer to
    /// "what is this big blocking row" six rungs running. A ranking made without
    /// it is a guess about which of the two a row is.
    ///
    /// `prod-entered-untagged` is the **tag-coverage residue**: bodies that
    /// entered a production, declined non-committally, and hit no tagged bail. It
    /// is printed like any other row rather than suppressed, because an
    /// unattributed population that renders as an absence is precisely the failure
    /// this axis exists to close. Its target is 0.
    pub fn_prod: BTreeMap<String, usize>,
    /// **The census/gate cross-check** (roadmap #44): of the functions this TU's
    /// census calls in class, how many does `PortC2`'s own per-function selector
    /// **refuse**, keyed by the refusal.
    ///
    /// It must be empty. Acceptance is supposed to live in the IL parser
    /// precisely so the census and the gate cannot disagree; anything here is the
    /// census over-claiming, and the headline numerator is an upper bound by the
    /// sum of these counts. `docs/GAPS.md` §6: a diagnostic that runs outside the
    /// parser needs a population whose answer is already known, and this is that
    /// population — every in-class function, whose answer should be "accepted".
    pub fn_gate_refusals: BTreeMap<String, usize>,
    /// **The `.gl` binding invariants** (D14). A binding decides *which symbol* a
    /// token names, and the oracle cannot grade a correspondence — a green
    /// differential only says the binding chosen and the binding c2 chose agreed
    /// on the IL tested (`docs/GAPS.md` §6, the `.sy` bullet). So the two facts
    /// the container itself settles are counted on every scan:
    ///
    /// * `"gl-token-ambiguous-dropped"` / `"gl-token-conflict-mangled"` — operand
    ///   tokens `.gl` claims for two different names, dropped rather than resolved
    ///   to the first ([`c2_il::gl_symbol_conflicts`]). Only the second has a known
    ///   answer of 0: `.gl` assigns one token per symbol, so a `?`-mangled name can
    ///   never be one of a disagreeing pair. The first is the type table brushing
    ///   against this reader's record shape, and costs nothing.
    /// * `"dtor-callee-<class>"` — the mangling class of the callee every
    ///   in-class generated empty destructor resolves to. The shape is a
    ///   destructor delegating to a sub-object's destructor, so every one of them
    ///   must be a destructor mangling (`??1` / `??_G` / `??_E` / `??_D`);
    ///   `dtor-callee-other` is the count that says the binding names something
    ///   the shape cannot delegate to, and its known answer is 0.
    pub bind_checks: BTreeMap<String, usize>,
    /// **The emitted-function census** (`docs/GAPS.md` §8, `docs/ROADMAP.md`
    /// §8.2) — the per-TU join between the census's rows and the *reference
    /// obj's* `.text` COMDAT leaders.
    ///
    /// The per-body census counts **IL bodies**, and c2 emits 7.23 % of them. A
    /// body it never emits needs the port to *skip* it, not to lower it, and no
    /// byte compare has ever graded such a body or ever can — the differential
    /// grades whole objs and those objs do not contain it. So the numerator's
    /// overlap with emitted code is the softest number in the project, and this
    /// map is the measurement of it.
    ///
    /// Rows, all of which print on every scan:
    ///
    /// * `emit-emitted` — `.text*` COMDAT leaders in the reference obj. **The
    ///   denominator.**
    /// * `emit-bound` — of those, symbols exactly one census row claims.
    /// * `emit-in-class` — of *those*, rows the census calls in class. **The
    ///   read-out.**
    /// * `emit-residue-generated` — unbound, and the name is a compiler-generated
    ///   form (`??_G` scalar-deleting destructor, `??_E` vector-deleting, `??_D`
    ///   vector destructor iterator, `??__E`/`??__F` dynamic init/atexit thunks).
    ///   c2 synthesizes these; **they have no `.ex` body at all**, so nothing
    ///   could bind them, and they are separated from the unexplained residue for
    ///   that reason and not to make the number look better.
    /// * `emit-residue-unbound` — unbound and *not* explained. The honest residue.
    /// * `emit-obj-unreadable` — the obj's COFF headers did not decode, so this
    ///   TU contributes **no** denominator rather than a short one.
    /// * `emit-record-*`, `emit-row-conflict`, `emit-name-conflict` — the
    ///   binding's own self-report ([`c2_il::EmitBinding`]).
    /// * `emit-accounting-broken` — the totality identity failed. Known answer 0.
    /// * `emit-match-tu-residue` — on a TU the port compiles **byte-exact**, an
    ///   emitted symbol the binding did not bind to an in-class row. The port's
    ///   obj *is* c2's there, so the answer is known exactly and the known answer
    ///   is 0. This is the one place the binding can be graded against ground
    ///   truth rather than against its own invariants.
    pub emit: BTreeMap<String, usize>,
    /// **The emitted-only blocking histogram**: the census key of every
    /// out-of-class row that binds to a symbol c2 actually emitted.
    ///
    /// Separate from [`TuResult::fn_blockers`] and never merged into it. That
    /// histogram is the widening order over all 2.46 M IL bodies; this one is the
    /// widening order over the 178 k the compiler emits, and the two rank
    /// differently by construction — a row made of header-inline bodies is large
    /// in the first and can be empty in the second.
    pub emit_blockers: BTreeMap<String, usize>,
    /// **The witness list for this TU's emitted-symbol residue** (board #159) —
    /// empty unless `C2RS_WITNESS` is set. See [`witness_path`] for why the
    /// names are emitted from here and not read back out of the obj by a second
    /// reader.
    ///
    /// One row per symbol counted into `emit-unbound-no-record|<class>` or
    /// `emit-unbound-has-record`, pushed from the same loop iteration that
    /// increments the counter, so the rows and the counts cannot disagree.
    pub emit_witness: Vec<WitnessRow>,
}

///
/// These appear as emitted `.text` COMDATs and can never bind to a census row,
/// because there is no row: the front end did not hand c2 a body for them.
/// Counted in their own residue bucket so the unexplained residue stays
/// unexplained instead of being diluted by a population that is explained.
///
/// The prefixes are MSVC's documented generated-symbol forms: `??_G` scalar
/// deleting destructor, `??_E` vector deleting destructor, `??_D` vector
/// destructor iterator, `??__E` dynamic initializer, `??__F` dynamic atexit
/// destructor.
/// **The port's COFF writer vocabulary**, imported from its published home.
///
/// This used to be a six-name hand-written mirror whose own doc comment said the
/// list "should be `c2-core`" and was duplicated only because that crate
/// belonged to another lane. **The mirror was accurate exactly as long as
/// `emit_dyninit_obj` had no caller** — w-r1's rung filed that as "left in place,
/// with the trigger named". W-R1c is that trigger: the port emits `.text$yc`,
/// `.bss` and `.CRT$XCU` now, and a stale six-name list would put the two
/// converted license TUs *outside* factor C while they are byte-exact matches.
///
/// So there is one list, next to the writers, and this is a `use` rather than a
/// copy.
use c2_core::coff::PORT_WRITER_SECTIONS;

/// The MSVC mangling class of `name`, for naming the unbound residue.
///
/// Coarse on purpose — it separates the populations that would be explained by
/// different stories, and nothing finer is measured.
fn mangling_class(name: &str) -> &'static str {
    match name {
        n if n.starts_with("??1") => "dtor",
        n if n.starts_with("??0") => "ctor",
        n if n.starts_with("??_") => "special-generated",
        n if n.starts_with("??$") => "template-operator",
        n if n.starts_with("??") => "operator",
        n if n.starts_with("?$") => "template",
        n if n.starts_with('?') => "ordinary",
        _ => "undecorated",
    }
}

/// Whether `name` is a function **c2 synthesizes**, with no `.ex` body behind it.
fn is_compiler_generated(name: &str) -> bool {
    ["??_G", "??_E", "??_D", "??__E", "??__F"]
        .iter()
        .any(|p| name.starts_with(p))
}

/// **W-EMITSET scratch read-out** — one TSV line per emitted symbol the census
/// did not bind, appended to `C2RS_WALL_DUMP`. Off, and free, when unset.
///
/// `src · has-record|no-record · mangled name`
///
/// It exists because `mangling_class` is a *prefix* rule and prefix rules have
/// lied four times this week: `special-generated` is every `??_…`, which is
/// `??_G`/`??_E`/`??_D` (real synthesized functions) **and** `??_7` (vftable),
/// `??_R0`…`??_R4` (RTTI) and `??_C` (string literals), which are data. A
/// decomposition that reports 47.7 % `special-generated` and never prints a name
/// cannot tell those apart, and the whole reading of the wall rests on which it
/// is. Read-only: it changes no count.
fn wall_dump(src: &str, name: &str, kind: &str) {
    static OUT: std::sync::OnceLock<Option<Mutex<std::fs::File>>> = std::sync::OnceLock::new();
    let out = OUT.get_or_init(|| {
        let p = std::env::var("C2RS_WALL_DUMP").ok()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
            .map(Mutex::new)
    });
    let Some(out) = out else { return };
    if let Ok(mut g) = out.lock() {
        let _ = g.write_all(format!("{src}\t{kind}\t{name}\n").as_bytes());
    }
}

/// **The witness list** (board #159) — the mangled names behind the
/// emitted-symbol residue, emitted by the code that *classifies* them.
///
/// `C2RS_WITNESS=<path>` turns it on and writes two artifacts at the end of the
/// scan:
///
/// * `<path>` — the ranked summary: per bucket, the symbol total, the distinct
///   name count, the TU count, and the top [`WITNESS_CAP`] names by frequency
///   with an example TU for each.
/// * `<path>.rows.tsv` — every row, `src · bucket · in-gl · name`, for slicing
///   per TU.
///
/// **Why this exists rather than a private reader.** `ROADMAP.md` §10.14 is the
/// record of the alternative: a standalone COFF reader was written to answer
/// "what is an `emit-unbound-no-record|ordinary` symbol", it keyed on *no `.gl`
/// run* where the instrument keys on *no framed `.gl` body record*, and it
/// missed the harness's known answer on the first witness TU. A diagnostic that
/// needs a classification the harness already computes must be **emitted by the
/// harness**; a second implementation is a second rule that agrees until the
/// moment it matters.
///
/// **Why an environment variable and not a CLI flag.** The classification lives
/// here, and so does the precedent: [`wall_dump`] and [`row_dump`] are already
/// env-gated scratch instruments in this file. Off by default, and when off the
/// rows are never built — [`witness_path`] is consulted once per process.
///
/// The two `in-gl` columns are **third and fourth** predicates and are labelled
/// as such wherever they are read. Neither is "binds to a census row" and
/// neither is "has a framed body record": they ask whether the symbol's name is
/// in `.gl` **at all**, which separates "c2 invented this symbol" from "the name
/// is right there and only the framed body record is missing".
///
/// **There are two because one of them cannot see half the residue.**
/// [`c2_il::mangled_names`] requires the run's second byte to be alphabetic and
/// therefore **silently drops every `??`-prefixed name** — its own doc comment
/// says so — which is every `dtor` and every `special-generated` row in this
/// list. Read alone it reports `0 of 947` for `??_G…` and that zero is an
/// artifact of the predicate, not a fact about `.gl`. So the second column is
/// [`c2_il::gl_symbol_index`], the binding's own token→name index, which does
/// carry `??`-names. Reporting both, and naming which is which, is the whole
/// discipline `ROADMAP.md` §10.11/§10.14 was written about.
fn witness_path() -> Option<&'static std::path::Path> {
    static P: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    P.get_or_init(|| std::env::var_os("C2RS_WITNESS").map(PathBuf::from))
        .as_deref()
}

/// How many names each bucket prints in the ranked summary. The remainder is
/// printed as a count of names *and* a count of symbols, never elided — a tail
/// that renders as nothing is the failure mode `docs/GAPS.md` §7 is about.
const WITNESS_CAP: usize = 40;

/// One witness row: which residue bucket, the mangled name, and whether that
/// name appears as a mangled run in `.gl` at all.
#[derive(Clone, Debug)]
pub struct WitnessRow {
    /// The residue bucket, spelled exactly as the counter key it accompanies:
    /// `emit-unbound-no-record|<mangling class>` or `emit-unbound-has-record`.
    pub bucket: String,
    pub name: String,
    /// `c2_il::mangled_names` contains this name — a **different predicate**
    /// from the one that put the row in its bucket, and one that cannot see a
    /// `??`-prefixed name at all.
    pub in_gl_runs: bool,
    /// `c2_il::gl_symbol_index` binds this name to some operand token — the
    /// predicate that *can* see `??`-names. Also not the bucketing predicate.
    pub in_gl_index: bool,
}

/// Aggregated witness numbers for one bucket. Every field is a count, so a
/// bucket that collected nothing prints zeros beside a nonzero grand total
/// rather than vanishing.
pub struct WitnessBucket {
    pub bucket: String,
    pub symbols: usize,
    pub tus: usize,
    /// Rows whose name `c2_il::mangled_names` finds (blind to `??`-names).
    pub in_gl_runs: usize,
    /// Rows whose name `c2_il::gl_symbol_index` binds to a token.
    pub in_gl_index: usize,
    /// `(name, occurrences, TUs it appears in, an example TU)`, ranked by
    /// occurrences descending then name ascending.
    pub names: Vec<(String, usize, usize, String)>,
}

/// Aggregated scan report.
pub struct GapReport {
    pub results: Vec<TuResult>,
    /// What produced these numbers (roadmap #46/#48): both trees' git HEADs, the
    /// resolved toolchain paths, the wibo version. `None` only when a report is
    /// built by hand in a test.
    pub provenance: Option<Provenance>,
    /// Capture-cache counters for this scan (all zero when `--no-cache`).
    pub cache: crate::capture_cache::CacheStats,
}

impl GapReport {
    pub fn count(&self, class: TuClass) -> usize {
        self.results.iter().filter(|r| r.class == class).count()
    }

    /// Reasons for `class`, most frequent first, with TU counts.
    pub fn top_reasons(&self, class: TuClass) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in self.results.iter().filter(|r| r.class == class) {
            *map.entry(r.reason.as_str()).or_insert(0) += 1;
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **P2b headline**: (functions in class, functions total) across the scan.
    /// Unlike the TU classes this is monotone and fine-grained — it moves on
    /// every widening step, where TU-level `match` stays 0 until a whole TU
    /// happens to be in class.
    pub fn fn_coverage(&self) -> (usize, usize) {
        self.results
            .iter()
            .fold((0, 0), |(a, b), r| (a + r.fn_in_class, b + r.fn_total))
    }

    /// Blocking features across all scanned functions, most frequent first.
    /// **This histogram is the widening order** (docs/ROADMAP.md §G5/P2b).
    pub fn fn_blocker_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_blockers))
    }

    /// **The D6 frame measure**, aggregated: `"<calls-class>|<census key>"` counts
    /// over every scanned function, most frequent first.
    pub fn fn_frame_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_frames))
    }

    /// **The control-flow axis**, aggregated, most frequent first. Rows are either
    /// a bare class (`cflow-…` decoded, `cf-…` the decoder's own residue) or a
    /// `"<cflow class>|<census key>"` cross-tab; see [`TuResult::fn_cflow`].
    pub fn fn_cflow_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_cflow))
    }

    /// **The EH axis**, aggregated, most frequent first. Rows are either a bare
    /// class (`eh-…`) or an `"<eh class>|<census key>"` cross-tab; see
    /// [`TuResult::fn_eh`].
    pub fn fn_eh_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_eh))
    }

    /// How many scanned functions the statement-layer scanner decoded end to end,
    /// and how many it did not — `(decoded, undecoded)`.
    ///
    /// The ratio is the honest bound on everything the control-flow axis claims: a
    /// shape histogram over half the corpus is a shape histogram over half the
    /// corpus, and the other half's CFG is simply not known yet.
    pub fn cflow_decoded_totals(&self) -> (usize, usize) {
        let mut d = 0;
        let mut u = 0;
        for r in &self.results {
            for (k, n) in &r.fn_cflow {
                if k.contains('|') {
                    continue; // a cross-tab row, already counted in its bare class
                }
                if k.starts_with("cflow-") {
                    d += n;
                } else {
                    u += n;
                }
            }
        }
        (d, u)
    }

    /// The three frame classes' totals across the scan, in `calls-0`, `calls-1`,
    /// `calls-2plus` order.
    pub fn frame_class_totals(&self) -> [usize; 3] {
        let mut t = [0usize; 3];
        for r in &self.results {
            for (k, n) in &r.fn_frames {
                let i = match k.split('|').next() {
                    Some("calls-0") => 0,
                    Some("calls-1") => 1,
                    _ => 2,
                };
                t[i] += n;
            }
        }
        t
    }

    /// **The body-dispatch axis**, aggregated, most frequent first. See
    /// [`TuResult::fn_dispatch`] for the row shapes.
    pub fn fn_complete_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_complete))
    }

    pub fn fn_dispatch_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_dispatch))
    }

    /// **The member-call production first-blocker axis**, aggregated, most frequent
    /// first. See [`TuResult::fn_prod`].
    pub fn fn_prod_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_prod))
    }

    /// The **tag-coverage residue** of the production axis: bodies that entered a
    /// member-call production, declined non-committally, and reached no tagged
    /// bail — so their refusal is inside a shipping recognizer and is **not yet
    /// attributed to a site**.
    ///
    /// Reported as a number on every scan rather than inferred from the absence of
    /// rows. It is an upper bound on what the 37 tag sites in
    /// `body::shapes::mcall_{tail,chain,cmp}` have left to explain, and it reaches
    /// 0 when they are all placed.
    pub fn prod_untagged_residue(&self) -> usize {
        self.results
            .iter()
            .map(|r| {
                r.fn_prod
                    .get("prod-entered-untagged")
                    .copied()
                    .unwrap_or(0)
            })
            .sum()
    }

    /// How many functions each dispatch axis saw in total. Both must equal the
    /// census's own function total: every body takes exactly one arm and reaches
    /// exactly one production state, so a short count means a body slipped through
    /// untagged and the axis is under-reporting rather than the population being
    /// small.
    pub fn dispatch_axis_totals(&self) -> (usize, usize) {
        let bare = |m: &BTreeMap<String, usize>| -> usize {
            m.iter()
                .filter(|(k, _)| !k.contains('|'))
                .map(|(_, n)| *n)
                .sum()
        };
        self.results.iter().fold((0, 0), |(a, b), r| {
            (a + bare(&r.fn_dispatch), b + bare(&r.fn_prod))
        })
    }

    /// **The census/gate disagreement**, aggregated: how many censused-in-class
    /// functions `PortC2` refuses, per refusal, most frequent first.
    ///
    /// Every entry is an error term on [`GapReport::fn_coverage`]'s numerator.
    /// The target is an empty list.
    pub fn fn_gate_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_gate_refusals))
    }

    /// Total censused-in-class functions the port refuses across the scan.
    pub fn fn_gate_disagreement(&self) -> usize {
        self.results
            .iter()
            .map(|r| r.fn_gate_refusals.values().sum::<usize>())
            .sum()
    }

    /// **The `.gl` binding invariants**, aggregated (see [`TuResult::bind_checks`]).
    pub fn bind_check_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.bind_checks))
    }

    /// **The emitted-function census**, aggregated (see [`TuResult::emit`]).
    pub fn emit_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.emit))
    }

    /// One aggregated emitted-census row.
    pub fn emit_total(&self, key: &str) -> usize {
        self.results
            .iter()
            .map(|r| r.emit.get(key).copied().unwrap_or(0))
            .sum()
    }

    /// **The read-out**: (in class ∩ emitted, emitted). The ratio is what
    /// `docs/ROADMAP.md` §8.2 ranks the plan by, and it is a **floor** — every
    /// emitted symbol the binding could not claim is residue, never a numerator.
    pub fn emit_coverage(&self) -> (usize, usize) {
        (self.emit_total("emit-in-class"), self.emit_total("emit-emitted"))
    }

    /// The unbound residue, split: (compiler-generated with no IL body,
    /// unexplained). The second number is the one that has to shrink; the first
    /// is a population no binding could ever claim.
    pub fn emit_residue(&self) -> (usize, usize) {
        (
            self.emit_total("emit-residue-generated"),
            self.emit_total("emit-residue-unbound") + self.emit_total("emit-name-two-rows"),
        )
    }

    /// **The emitted-only widening order**: blocking features restricted to rows
    /// that bind to a symbol c2 emitted, most frequent first.
    pub fn emit_blocker_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.emit_blockers))
    }

    /// **Ground truth.** On a TU the port compiles byte-exact, c2's emitted set
    /// *is* the port's, which came from the gate's own per-record binding — so
    /// the emitted census must read `in-class == emitted` with an empty residue.
    /// Returns how many emitted symbols on `match` TUs the binding failed to
    /// bind to an in-class row. **Known answer: 0.**
    ///
    /// This is the only check on the binding that is not a self-invariant. The
    /// oracle cannot grade a correspondence in general — but on a byte-exact TU
    /// it has already graded the whole symbol table, so the answer is known and
    /// the binding can be held to it.
    pub fn emit_match_tu_residue(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .map(|r| {
                let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
                let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
                e.saturating_sub(i)
            })
            .sum()
    }

    /// TUs ordered by **distance to matching** — how many of their functions are
    /// blocked — keeping only those at or below `max_blocked`, nearest first.
    ///
    /// `docs/ROADMAP.md` §8.2 makes TU match the payoff metric and this
    /// distribution its leading indicator; the emitted census is what says
    /// whether a given TU's remaining distance is real work or bookkeeping.
    /// `capture-fail` TUs are excluded: they have no census at all, so a
    /// distance of 0 there means "never measured", not "nearly done".
    pub fn near_match_tus(&self, max_blocked: usize) -> Vec<&TuResult> {
        let mut v: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| r.class != TuClass::CaptureFail && r.fn_total > 0)
            .filter(|r| r.fn_total - r.fn_in_class <= max_blocked)
            .collect();
        v.sort_by_key(|r| (r.fn_total - r.fn_in_class, r.src.clone()));
        v
    }

    /// The same distribution measured on the population the **goal** is written
    /// in: blocked *emitted* functions, not blocked IL bodies.
    ///
    /// [`Self::near_match_tus`] counts `.ex` bodies, and the workload carries
    /// 2,462,571 of those against 178,968 emitted functions (`ROADMAP.md` §8.1).
    /// The two distances are not the same number and not even the same order:
    /// `src/system/math/Rand2.cpp` is 8 blocked bodies but **2** blocked emitted
    /// functions, and `src/system/math/vec.cpp` is 565 blocked bodies with
    /// **zero** blocked emitted functions. Published side by side because
    /// neither one alone is "distance to a byte-exact TU" — see
    /// [`Self::emit_set_reachable_tus`] for the third constraint that binds
    /// both.
    pub fn near_match_tus_emitted(&self, max_blocked: usize) -> Vec<&TuResult> {
        let blocked = |r: &TuResult| {
            let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
            let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
            e.saturating_sub(i)
        };
        let mut v: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| {
                r.class != TuClass::CaptureFail
                    && r.emit.get("emit-emitted").copied().unwrap_or(0) > 0
            })
            .filter(|r| blocked(r) <= max_blocked)
            .collect();
        v.sort_by_key(|r| (blocked(r), r.src.clone()));
        v
    }

    /// TUs for which the port could emit the **right set of `.text` COMDATs at
    /// all**, however good its codegen becomes — a hard ceiling on TU match that
    /// no widening can lift.
    ///
    /// `PortC2::build` takes `il.functions()`, one entry per `.ex` function
    /// segment, and under `/Gy` pushes exactly one `.text` COMDAT per entry.
    /// **There is no emit-set model anywhere in the port** (`ROADMAP.md` §8.3
    /// Phase 7 is where one would go). So when a TU's `.ex` segment count
    /// differs from its reference obj's `.text` COMDAT-leader count, the port
    /// emits the wrong number of sections and the obj diverges regardless of
    /// what any function lowers to. `emit-emitted` is exactly that leader count
    /// and `fn_total` is exactly that segment count, so the predicate is a
    /// comparison of two numbers the scan already has.
    ///
    /// This is a **necessary** condition, not a sufficient one — the bodies
    /// still have to lower byte-exact. Its value is as a ceiling: on the dc3
    /// workload it holds for 25 of 871 graded TUs, which bounds TU match at
    /// 25/878 until Phase 7 exists, against a terminal target of 871.
    pub fn emit_set_reachable_tus(&self) -> Vec<&TuResult> {
        let mut v: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| r.class != TuClass::CaptureFail)
            .filter(|r| r.fn_total == r.emit.get("emit-emitted").copied().unwrap_or(0))
            .collect();
        v.sort_by_key(|r| (r.fn_total - r.fn_in_class, r.src.clone()));
        v
    }

    /// The invariant behind [`Self::emit_set_reachable_tus`], as a count that
    /// must be **zero**: a TU that the differential graded `match` and whose
    /// `.ex` segment count nevertheless disagrees with its obj's `.text`
    /// COMDAT-leader count.
    ///
    /// A byte-exact obj cannot have a different number of `.text` COMDATs than
    /// the port emitted, so a nonzero here means `fn_total` and `emit-emitted`
    /// are not counting the things this reading says they count, and the
    /// ceiling above is void. It is the control that makes the ceiling a
    /// measurement rather than an argument: on this workload the agreement rate
    /// is 25/871 = 2.9 %, so six matching TUs agreeing by accident is ~10⁻⁹.
    pub fn emit_set_violations(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .filter(|r| r.fn_total != r.emit.get("emit-emitted").copied().unwrap_or(0))
            .count()
    }

    /// [`Self::emit_set_violations`] against the **gate-anchored** segment count
    /// (`4F 1F`) instead of the census's (`4C 4F 11`) — see step 1g in
    /// [`scan_one`].
    ///
    /// Returns `(violations, matching TUs the gate count is KNOWN for)`. The
    /// second number is not decoration: this control can only go red on a TU
    /// where `functions()` returned a count, and reporting the violation total
    /// without the population it was taken over is the shape that lets a green
    /// control mean "nothing was checked".
    pub fn emit_set_violations_gate(&self) -> (usize, usize) {
        let m: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .filter(|r| r.emit.contains_key("emit-gate-segments-known"))
            .collect();
        let bad = m
            .iter()
            .filter(|r| {
                r.emit.get("emit-gate-segments").copied().unwrap_or(0)
                    != r.emit.get("emit-emitted").copied().unwrap_or(0)
            })
            .count();
        (bad, m.len())
    }

    /// The splitter disagreement as counts (step 1g): `(TUs the gate count is
    /// known for, unknown, agree, disagree, gate sees more, census sees more,
    /// gate-anchored ceiling, entering the ceiling, leaving it)`.
    #[allow(clippy::type_complexity)]
    pub fn splitter_disagreement(&self) -> (usize, usize, usize, usize, usize, usize, usize, usize, usize) {
        let t = |k: &str| self.emit_total(k);
        (
            t("emit-gate-segments-known"),
            t("emit-gate-segments-unknown"),
            t("emit-splitter-agree"),
            t("emit-splitter-disagree"),
            t("emit-splitter-gate-sees-more"),
            t("emit-splitter-census-sees-more"),
            t("emit-set-ceiling-gate"),
            t("emit-set-ceiling-gate-enter"),
            t("emit-set-ceiling-gate-leave"),
        )
    }

    /// **The four Phase 7 factors for one TU** (`docs/ROADMAP.md` §10.19,
    /// board #160), in `[A, B, C, D]` order:
    ///
    /// | | predicate | key |
    /// |---|---|---|
    /// | **A** | `.ex` segments == obj `.text` COMDATs, on the anchor the port consumes | `emit-set-ceiling-gate` |
    /// | **B** | every emitted symbol binds | `emit-set-ceiling-today` |
    /// | **C** | obj section set ⊆ [`PORT_WRITER_SECTIONS`] | `emit-sec-reachable` |
    /// | **D** | every emitted COMDAT is in the port's codegen class | `emit-class-complete` |
    ///
    /// Each factor is **necessary** for a byte-exact obj and none is sufficient;
    /// what §10.19 measured is that their conjunction is exactly the observed
    /// match set. Every one reads a key some *other* code path wrote, so this
    /// function re-derives no rule — it is a join, and that is the whole point
    /// (§10.14).
    ///
    /// **A is gate-anchored** (`4F 1F`, what `PortC2::build` consumes) rather
    /// than `LO`-anchored: §10.18 settled that the two splitters disagree on 634
    /// of 871 TUs and that the port's anchor is the one its emitter has to
    /// satisfy. [`Self::factor_a_lo`] is the other reading, published beside it.
    pub fn factors(r: &TuResult) -> [bool; 4] {
        let has = |k: &str| r.emit.contains_key(k);
        [
            has("emit-set-ceiling-gate"),
            has("emit-set-ceiling-today"),
            has("emit-sec-reachable"),
            has("emit-class-complete"),
        ]
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

    /// `(|A|, |B|, |C|, |D|, |A_lo|, |B∧C|, |A∧B∧C∧D|)` over the graded TUs.
    ///
    /// `B∧C` is the plan's **near-term joint ceiling** — what a perfect emit-set
    /// model plus a perfect binding reaches while the writer's vocabulary is
    /// what it is (`PHASE7_PLAN.md` §1). It is a *joint*, measured per TU, and
    /// not a product of marginals: §8.6's standing rule, and the reason this
    /// function exists rather than a note telling readers to multiply.
    pub fn factor_counts(&self) -> [usize; 7] {
        let mut c = [0usize; 7];
        for r in self.graded() {
            let f = Self::factors(r);
            for i in 0..4 {
                c[i] += usize::from(f[i]);
            }
            c[4] += usize::from(Self::factor_a_lo(r));
            c[5] += usize::from(f[1] && f[2]);
            c[6] += usize::from(f.iter().all(|&b| b));
        }
        c
    }

    /// The TUs satisfying all four factors, by source path. §10.19's claim is
    /// that this set **is** the match set, so it is returned as a list of names
    /// rather than a count: a count could agree by coincidence, and two sets
    /// that differ by a swap would read as equal.
    pub fn factor_all_tus(&self) -> Vec<&str> {
        self.graded()
            .filter(|r| Self::factors(r).iter().all(|&b| b))
            .map(|r| r.src.as_str())
            .collect()
    }

    /// **The known-answer control on the factorization**: how many byte-exact
    /// TUs fail each factor, and how many `match` TUs there were to check.
    /// Returns `([A, B, C, D] violations, matching TUs)`.
    ///
    /// Every factor is a *necessary* condition for a byte-exact obj, which is
    /// the only thing that makes it a ceiling — so on a `match` TU all four must
    /// hold. Nonzero anywhere means the factor is not necessary and any bound
    /// drawn from it is void. For **C** this is also the control on
    /// [`PORT_WRITER_SECTIONS`] itself: a matching obj is the port's own output,
    /// so a name missing from that list shows up here rather than in an argument
    /// about whether the list is complete.
    pub fn factor_control_on_match_tus(&self) -> ([usize; 4], usize) {
        let mut bad = [0usize; 4];
        let mut n = 0;
        for r in self.results.iter().filter(|r| r.class == TuClass::Match) {
            n += 1;
            for (i, ok) in Self::factors(r).iter().enumerate() {
                bad[i] += usize::from(!ok);
            }
        }
        (bad, n)
    }

    /// **The frontier**: TUs inside `A∧B∧C` that are not yet a `match` — i.e.
    /// the emit set is reachable, every emitted symbol binds, the obj's sections
    /// are all writable, and the *only* factor left is **D**, codegen breadth.
    ///
    /// This is the one actionable list the factorization produces. Everything
    /// else it prints is a bound; these are TUs where no model, no section work
    /// and no binding repair is needed — widening the accepted function class is
    /// the whole remaining distance. Sorted by that distance (emitted functions
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
                f[0] && f[1] && f[2] && !f[3]
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

    /// The binding invariant that must be **zero**: a generated destructor bound to
    /// a callee that is not a destructor. Nonzero means the `.gl` reader is naming
    /// the wrong symbol in a way no obj comparison over this corpus could have
    /// shown, because these bodies rarely reach an emitter.
    ///
    /// The ambiguity counts are deliberately **not** in here. A token two records
    /// disagree about is dropped, so it is an over-refusal with a measurable cost,
    /// not a wrong binding; the workload's residual is 7, all of them one `.gl`
    /// record form this reader does not model (`$…$initializer$` local statics), and
    /// their measured cost is 0 functions.
    pub fn bind_violations(&self) -> usize {
        self.results
            .iter()
            .map(|r| {
                r.bind_checks
                    .iter()
                    .filter(|(k, _)| {
                        k.as_str() == "dtor-callee-other" || k.as_str() == "dtor-callee-none"
                    })
                    .map(|(_, n)| *n)
                    .sum::<usize>()
            })
            .sum()
    }

    /// Replay soundness: (checked, diverged).
    pub fn replay_stats(&self) -> (usize, usize) {
        let checked = self.results.iter().filter(|r| r.replay_ok.is_some()).count();
        let bad = self
            .results
            .iter()
            .filter(|r| r.replay_ok == Some(false))
            .count();
        (checked, bad)
    }
}

/// Sum a per-TU count map across the scan and rank it, most frequent first with
/// ties broken by key. The six axis histograms above differ only in which map they
/// read, and each used to spell this same fold out longhand.
fn merge_counts<'a>(
    maps: impl Iterator<Item = &'a BTreeMap<String, usize>>,
) -> Vec<(String, usize)> {
    let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    for m in maps {
        for (k, n) in m {
            *map.entry(k.as_str()).or_insert(0) += n;
        }
    }
    let mut v: Vec<(String, usize)> = map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

/// Pull a normalized headline out of a cl.exe failure blob: the first line
/// containing `error C`, else the first non-empty line, truncated.
fn normalize_cl_error(blob: &str) -> (String, String) {
    let detail = blob
        .lines()
        .map(str::trim)
        .find(|l| l.contains("error C"))
        .or_else(|| blob.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("(no output)")
        .to_string();
    // Aggregation key = the `Cnnnn` code when present, else a clipped line.
    let key = detail
        .split_whitespace()
        .find(|t| {
            let t = t.trim_end_matches(':');
            t.len() >= 4
                && t.starts_with('C')
                && t[1..].chars().all(|c| c.is_ascii_digit())
        })
        .map(|t| t.trim_end_matches(':').to_string())
        .unwrap_or_else(|| clip(&detail, 60));
    (key, clip(&detail, 200))
}

/// A stable bucket key for one of the port's per-function refusals.
///
/// The refusal messages are prose (they are what a `codegen-gap` TU reports), so
/// the key is the leading clause — everything before the first `:` — clipped.
/// That is deliberately the message's own words rather than a hand-maintained
/// enum: a key nobody has to remember to add is a key that cannot go stale, and
/// `docs/GAPS.md` §6's rule against guessed names applies here too. The keys are
/// meant to reach zero, not to be ranked forever.
fn gate_key(msg: &str) -> String {
    let head = msg.split(':').next().unwrap_or(msg).trim();
    clip(head, 72)
}

/// Which destructor mangling a generated empty destructor's resolved callee is —
/// `"other"` when it is not one at all, which is the count that must stay 0.
///
/// MSVC spells the four: `??1` an ordinary destructor, `??_G` the scalar deleting
/// destructor, `??_E` the vector deleting one, `??_D` the vbase destructor. The
/// shape [`c2_il`] parses is a destructor whose whole body delegates to a
/// sub-object's destructor, so the callee is one of these by construction of the
/// *source*, independently of how `.gl` was read — which is what makes this a
/// grader for the binding rather than a restatement of it.
pub fn dtor_callee_class(name: &str) -> &'static str {
    for (p, k) in [
        ("??1", "1"),
        ("??_G", "G"),
        ("??_E", "E"),
        ("??_D", "D"),
    ] {
        if name.starts_with(p) {
            return k;
        }
    }
    "other"
}

fn clip(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        let mut end = n;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

/// **The per-row read-out (W-ADJUST, boards #127/#128/#131).** One TSV line per
/// census row whose key is named in `C2RS_ROW_DUMP` (or `*` for all), appended to
/// `C2RS_ROW_DUMP_OUT`; `C2RS_ROW_DUMP_EMITTED` restricts it to rows that bind to
/// a symbol c2 actually emitted. Off — and free — when the variable is unset.
///
/// ```text
/// src · index · key · EMITTED|not-emitted · mangled name · frame · cflow · eh
///     · dispatch · production · completeness · hex_mark · the blocking-byte window
/// ```
///
/// **Every axis this scan prints is a histogram, and a histogram cannot answer a
/// question about a JOINT.** `docs/ROADMAP.md` §8.6's standing rule — never
/// multiply marginals for an intersection, measure the joint per TU — has no tool
/// behind it without this: the EH, frame and control-flow crosses are each a
/// separate `BTreeMap`, so "how many emitted rows of THIS key are straight *and*
/// EH-free *and* single-call" is unanswerable from the report. It is answerable
/// from one pass over this file, and that is where the 3,062-clean figure for
/// `expr-intrinsic-this-adjust` and the 9,111-clean figure for the whole
/// receiver-designator site came from.
///
/// Two further questions it exists for, both of which changed a ranking:
///
/// * **which production site actually refused** — `expr-intrinsic-this-adjust`
///   names the byte the *assignment* parser stopped on, while 99.99 % of the row
///   declines one reader earlier at the receiver designator;
/// * **is a row N distinct source functions or one replicated across TUs** —
///   `…recv-object-then-type-ptr-whole` is 1,380 emitted functions and **four**
///   mangled names, which is a fact about the differential coverage a rung can
///   claim, and no aggregate can see it.
///
/// **Read-only over the census: it changes no count and no verdict.** Asserted by
/// running the whole 878-TU scan with the dump armed and comparing all five
/// published numbers against the un-armed scan — 703,875 / 2,462,571 bodies,
/// 34,674 / 178,968 emitted, 6 match, 0 mismatch, disagreement 0, identical. An
/// instrument whose inertness is argued rather than run is this project's
/// dominant failure mode (`docs/GAPS.md` §6).
fn row_dump(
    src: &str,
    census: &[(c2_il::FnCensus, Result<c2_il::IlFunction, &'static str>)],
    emitted: Option<&[String]>,
) {
    static OUT: std::sync::OnceLock<Option<Mutex<std::fs::File>>> = std::sync::OnceLock::new();
    let Ok(want) = std::env::var("C2RS_ROW_DUMP") else {
        return;
    };
    let wanted: Vec<&str> = want.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let out = OUT.get_or_init(|| {
        let p = std::env::var("C2RS_ROW_DUMP_OUT").ok()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
            .map(Mutex::new)
    });
    let Some(out) = out else { return };
    let emitted_set: std::collections::BTreeSet<&str> =
        emitted.unwrap_or(&[]).iter().map(String::as_str).collect();
    let emitted_only = std::env::var_os("C2RS_ROW_DUMP_EMITTED").is_some();
    let mut buf = String::new();
    for (f, _) in census {
        let key = f.verdict.key();
        if !wanted.iter().any(|w| key == *w || *w == "*") {
            continue;
        }
        let name = f.emit_name.as_deref().unwrap_or("-");
        let is_emitted = f.emit_name.as_deref().is_some_and(|n| emitted_set.contains(n));
        if emitted_only && !is_emitted {
            continue;
        }
        let hex: String = f.hex.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
        buf.push_str(&format!(
            "{src}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            f.index,
            key,
            if is_emitted { "EMITTED" } else { "not-emitted" },
            name,
            f.frame_class(),
            f.cflow,
            f.eh,
            f.dispatch,
            f.prod,
            f.verdict.completeness().name(),
            f.hex_mark,
            hex,
        ));
    }
    if buf.is_empty() {
        return;
    }
    if let Ok(mut g) = out.lock() {
        let _ = g.write_all(buf.as_bytes());
    }
}

/// Scan one TU. `work` must be a private (per-TU) directory.
fn scan_one(
    tc: &Toolchain,
    cfg: &GapConfig,
    cache: Option<&CaptureCache>,
    src: &str,
    work: &Path,
    do_replay: bool,
) -> TuResult {
    let mut res = TuResult {
        src: src.to_string(),
        class: TuClass::CaptureFail,
        reason: String::new(),
        detail: String::new(),
        ex_len: 0,
        fn_names: 0,
        replay_ok: None,
        fn_total: 0,
        fn_in_class: 0,
        fn_blockers: BTreeMap::new(),
        fn_frames: BTreeMap::new(),
        fn_cflow: BTreeMap::new(),
        fn_eh: BTreeMap::new(),
        fn_dispatch: BTreeMap::new(),
        fn_complete: BTreeMap::new(),
        fn_prod: BTreeMap::new(),
        fn_gate_refusals: BTreeMap::new(),
        bind_checks: BTreeMap::new(),
        emit: BTreeMap::new(),
        emit_blockers: BTreeMap::new(),
        emit_witness: Vec::new(),
    };

    // 1. Capture: real flags, real cwd, strace keeps bundle + obj. Served from
    //    the content-addressed cache when one is configured — the cache dir IS
    //    the capture dir, so the `-Fo` path c2 bakes into the obj is a function
    //    of the key and a hit is byte-identical to the capture that filled it
    //    (`crate::capture_cache`).
    let capture_result = match cache {
        Some(c) => c.capture(tc, src, &cfg.flags, cfg.cwd.as_deref(), work).0,
        None => tc.capture_reference_with(src, work, &cfg.flags, cfg.cwd.as_deref()),
    };
    let captured =
        match capture_result {
            Ok(c) => c,
            Err(e) => {
                let (key, detail) = normalize_cl_error(&e.to_string());
                res.reason = key;
                res.detail = detail;
                return res;
            }
        };
    // The obj's shape depends on argv the IL bundle does not record: /Gy (implied
    // by /O1 and /O2) puts each function in its own COMDAT .text. Two of the
    // port's per-function refusals are /Gy-only, so the cross-check below needs
    // the same flag the emitter gets.
    let gy = PortC2::flags_imply_function_level_linking(&cfg.flags);
    res.ex_len = captured.bundle.ex().map(|b| b.len()).unwrap_or(0);
    res.fn_names = captured
        .bundle
        .get("gl")
        .map(|gl| c2_il::mangled_names(gl).len())
        .unwrap_or(0);

    // 1b. P2b function-level census — runs regardless of the TU class below, so
    //     even a `vocab-gap` TU contributes its per-function ranking. This is
    //     the only measurement that moves before whole TUs come in class.
    if let Some(census) = captured.bundle.census_functions() {
        res.fn_total = census.len();
        for (f, gate) in &census {
            if f.verdict.in_class() {
                res.fn_in_class += 1;
                // 1c. The cross-check: run the port's own per-function selector
                //     over every function the census claims. A refusal here is a
                //     census/gate disagreement, and it is recorded under its own
                //     key rather than left as a rumour — the numerator is the
                //     public claim, so its error term has to be measured on every
                //     scan (roadmap #44, `docs/GAPS.md` §6).
                let key = match gate {
                    Err(e) => Some((*e).to_string()),
                    Ok(func) => match c2_core::codegen::opt_mode_of_word(f.opt_word) {
                        Err(_) => Some("opt-mode".to_string()),
                        Ok(mode) => c2_core::codegen::function_gate(func, mode, gy)
                            .err()
                            .map(|e| gate_key(&e.to_string())),
                    },
                };
                if let Some(k) = key {
                    *res.fn_gate_refusals.entry(k).or_insert(0) += 1;
                }
            } else {
                *res.fn_blockers.entry(f.verdict.key()).or_insert(0) += 1;
            }
            // The D6 frame axis, over *every* function: the in-class shapes are
            // the control group (all of them are leaves or single tail calls, so
            // a `calls-2plus` reading among them would indict the measure).
            *res.fn_frames
                .entry(format!("{}|{}", f.frame_class(), f.verdict.key()))
                .or_insert(0) += 1;
            // The control-flow axis, likewise over every function — the in-class
            // shapes are the control group here too, and they must all read
            // `cflow-straight`.
            *res.fn_cflow.entry(f.cflow.clone()).or_insert(0) += 1;
            if f.cflow.starts_with("cflow-") {
                *res.fn_cflow
                    .entry(format!("{}|{}", f.cflow, f.verdict.key()))
                    .or_insert(0) += 1;
            }
            // The EH axis, likewise over every function — and here the in-class
            // shapes are more than a control group: the `empty-dtor-*` buckets
            // ARE the cheap side of the boundary, so any of them reading
            // anything but the cheap key would say the axis is wrong.
            //
            // **The cross says which population a row is in, and it must.**
            // `FnVerdict::key` spells IN-CLASS labels and BLOCKER keys into one
            // namespace, and this cross used to be `"<eh>|<key>"` for both. On
            // one scan that made `eh-bare|empty-dtor-delegation` — 27,501 —
            // the largest row of the whole EH cross, and `empty-dtor-delegation`
            // is an ACCEPTED shape. Anyone ranking off the table ranked a control
            // group, and one nearly got scheduled as a rung. The population is
            // now in the key, and there is a per-class `|BLOCKED` subtotal so a
            // blocked stock can be sized without knowing the in-class label
            // strings by heart.
            *res.fn_eh.entry(f.eh.clone()).or_insert(0) += 1;
            let pop = if f.verdict.in_class() { "INCLASS" } else { "BLOCKED" };
            if !f.verdict.in_class() {
                *res.fn_eh.entry(format!("{}|BLOCKED", f.eh)).or_insert(0) += 1;
            }
            *res.fn_eh
                .entry(format!("{}|{pop}|{}", f.eh, f.verdict.key()))
                .or_insert(0) += 1;
            // The two DISPATCH axes, over every function — same row shapes as the
            // EH cross above and for the same reason: `FnVerdict::key` spells
            // accepted shapes and blockers into one namespace, so the population
            // has to be in the key or an accepted control group reads like a rung.
            //
            // The bare totals are emitted for EVERY function, so both axes sum to
            // the census and a body that reached no tagged site is a printed row
            // rather than a hole. That is the whole discipline here: this axis
            // exists because 30,475 functions were previously reported only as
            // "none of the three productions was entered", which is an absence,
            // and an absence cannot be ranked.
            *res.fn_dispatch.entry(f.dispatch.to_string()).or_insert(0) += 1;
            *res.fn_prod.entry(f.prod.to_string()).or_insert(0) += 1;
            let complete = f.verdict.completeness().name();
            *res.fn_complete.entry(complete.to_string()).or_insert(0) += 1;
            if !f.verdict.in_class() {
                *res.fn_complete
                    .entry(format!("{complete}|BLOCKED"))
                    .or_insert(0) += 1;
                *res.fn_complete
                    .entry(format!("{complete}|{}", f.verdict.key()))
                    .or_insert(0) += 1;
            }
            if !f.verdict.in_class() {
                *res.fn_dispatch
                    .entry(format!("{}|BLOCKED", f.dispatch))
                    .or_insert(0) += 1;
                *res.fn_prod
                    .entry(format!("{}|BLOCKED", f.prod))
                    .or_insert(0) += 1;
            }
            *res.fn_dispatch
                .entry(format!("{}|{pop}|{}", f.dispatch, f.verdict.key()))
                .or_insert(0) += 1;
            // The production cross is emitted only for the bodies that actually
            // reached a member-call production. Crossing `prod-not-entered` with a
            // census key would restate the dispatch axis under a second name, and
            // it is the dispatch axis that owns that population.
            if f.prod != "prod-not-entered" {
                *res.fn_prod
                    .entry(format!("{}|{pop}|{}", f.prod, f.verdict.key()))
                    .or_insert(0) += 1;
            }
            // …and the migration cross: the measured `maxState` axis against the
            // refuted statement-count one it replaces (`docs/EH_RECORDS.md` §9.4,
            // §10). This is what reconciles §7.3's published split with the real
            // one instead of silently replacing it.
            *res.fn_eh
                .entry(format!("eh-migrate|{}|{}", f.eh, f.eh_stmt))
                .or_insert(0) += 1;
            // 1d. The binding invariant (D14): what did the `.gl` symbol index
            //     say a generated destructor delegates to? A destructor, always —
            //     anything else is a binding the oracle would have had no chance
            //     to catch, because these bodies rarely reach an emitter.
            if f.verdict.key().starts_with("empty-dtor") {
                if let Ok(func) = gate {
                    let k = match &func.tail_call {
                        Some(c) => dtor_callee_class(c),
                        None => "none",
                    };
                    *res.bind_checks
                        .entry(format!("dtor-callee-{k}"))
                        .or_insert(0) += 1;
                }
            }
        }
        // 1e. **The emitted-function census** (`docs/GAPS.md` §8). The census
        //     above counts IL bodies; this counts the functions c2 *emitted*, and
        //     says how many of those the port's accepted class covers. It is the
        //     only reading of the numerator that a byte compare could ever have
        //     graded, because it is the only one restricted to code that appears
        //     in an obj.
        //
        //     The join is: census row --(`.gl` body-offset record)--> mangled name
        //     --(`.text` COMDAT leader)--> emitted. Both halves fail closed, and
        //     every failure lands in a printed residue row rather than adjusting
        //     a denominator downwards, which would inflate the ratio.
        match captured.ref_obj.text_comdat_functions() {
            None => {
                *res.emit.entry("emit-obj-unreadable".into()).or_insert(0) += 1;
            }
            Some(emitted) => {
                // Which rows claim which emitted symbol. A symbol two rows claim
                // is not bound to either — `EmitBinding` already drops those, so
                // this can only see a repeat when two DISTINCT sections carry one
                // name, which is itself a thing to count rather than to average.
                let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
                for (i, (f, _)) in census.iter().enumerate() {
                    if let Some(n) = f.emit_name.as_deref() {
                        claim.entry(n).or_default().push(i);
                    }
                }
                *res.emit.entry("emit-emitted".into()).or_insert(0) += emitted.len();
                // **W-EMITSET — the residue split that decides the ceiling.**
                // §9.16.3 ceilings TU match at 25/871 because the port emits one
                // COMDAT per `.ex` segment with no emit-set model. The ceiling on
                // any *model* over `.ex` segments is different and harder: the
                // port can only ever emit a COMDAT for a body it HAS, under the
                // name the binding gives it. So an emitted symbol no row claims
                // is one of two completely different things, and they had been
                // reported as one number:
                //
                // * it has a framed `.gl` body record — the body IS in this
                //   bundle and `EmitBinding` merely lost the row. **Instrument
                //   defect**, closable in `bind.rs`.
                // * it has none — no body, so a segment-driven port must
                //   SYNTHESIZE the COMDAT. **A wall**, and a different phase.
                //
                // `emit-set-ceiling-*` below turns that into a per-TU predicate.
                let body_records = captured
                    .bundle
                    .get("gl")
                    .map(c2_il::gl_body_record_names)
                    .unwrap_or_default();
                let mut unbound_with_body = 0usize;
                let mut unbound_no_body = 0usize;
                // The witness list's third predicate (board #159, `witness_path`):
                // is the name in `.gl` AT ALL? Built only when witnesses are on,
                // and deliberately NOT used by any counter above — a name present
                // as a run with no framed body record is a different fact from
                // both "binds to a row" and "has a body record", and §10.14 is the
                // record of what conflating two of the three costs.
                let (gl_runs, gl_index): (
                    std::collections::BTreeSet<String>,
                    std::collections::BTreeSet<String>,
                ) = match (witness_path(), captured.bundle.get("gl")) {
                    (Some(_), Some(gl)) => (
                        c2_il::mangled_names(gl).into_iter().collect(),
                        c2_il::gl_symbol_index(gl).into_values().collect(),
                    ),
                    _ => Default::default(),
                };
                let witness = |res: &mut TuResult, bucket: String, name: &str| {
                    if witness_path().is_some() {
                        res.emit_witness.push(WitnessRow {
                            bucket,
                            name: name.to_string(),
                            in_gl_runs: gl_runs.contains(name),
                            in_gl_index: gl_index.contains(name),
                        });
                    }
                };
                for name in &emitted {
                    if matches!(claim.get(name.as_str()).map(Vec::as_slice), Some([_])) {
                        // The CONTROL population. Whatever story the residue's
                        // names suggest has to be false of the symbols that DO
                        // bind, or it is a story about mangled names in general
                        // and not about the residue.
                        wall_dump(src, name, "bound");
                        continue;
                    }
                    if body_records.contains(name) {
                        unbound_with_body += 1;
                        *res.emit.entry("emit-unbound-has-record".into()).or_insert(0) += 1;
                        wall_dump(src, name, "has-record");
                        witness(&mut res, "emit-unbound-has-record".into(), name);
                    } else {
                        unbound_no_body += 1;
                        *res.emit.entry("emit-unbound-no-record".into()).or_insert(0) += 1;
                        let key = format!("emit-unbound-no-record|{}", mangling_class(name));
                        *res.emit.entry(key.clone()).or_insert(0) += 1;
                        wall_dump(src, name, "no-record");
                        witness(&mut res, key, name);
                    }
                }
                // The three per-TU ceilings, as counts of TUs (0 or 1 each here;
                // summed over the scan by the report). Stated as *nested* bounds
                // so the order of attack is legible:
                //
                //  * `today`     — every emitted symbol already binds to a row.
                //                  The ceiling on a model built on today's binding.
                //  * `repaired`  — every emitted symbol at least HAS a body record.
                //                  The ceiling if `bind.rs` were perfect.
                //  * `wall`      — this TU has an emitted symbol with no body at
                //                  all, so no binding repair can reach it.
                if unbound_with_body == 0 && unbound_no_body == 0 {
                    *res.emit.entry("emit-set-ceiling-today".into()).or_insert(0) += 1;
                }
                if unbound_no_body == 0 {
                    *res.emit.entry("emit-set-ceiling-repaired".into()).or_insert(0) += 1;
                } else {
                    *res.emit.entry("emit-set-ceiling-wall".into()).or_insert(0) += 1;
                }
                for name in &emitted {
                    match claim.get(name.as_str()).map(Vec::as_slice) {
                        Some([row]) => {
                            *res.emit.entry("emit-bound".into()).or_insert(0) += 1;
                            let f = &census[*row].0;
                            if f.verdict.in_class() {
                                *res.emit.entry("emit-in-class".into()).or_insert(0) += 1;
                            } else {
                                *res.emit_blockers.entry(f.verdict.key()).or_insert(0) += 1;
                            }
                        }
                        Some(_) => {
                            *res.emit.entry("emit-name-two-rows".into()).or_insert(0) += 1;
                        }
                        None if is_compiler_generated(name) => {
                            *res.emit
                                .entry("emit-residue-generated".into())
                                .or_insert(0) += 1;
                        }
                        None => {
                            *res.emit.entry("emit-residue-unbound".into()).or_insert(0) += 1;
                            // …and WHAT it is, by mangling class. A residue
                            // reported only as a number is a rumour; these rows
                            // are what a follow-up lane would attack, and they
                            // are also the check on the story: if the residue
                            // were really "c2 synthesized it", it would be
                            // concentrated in the special-member classes, and if
                            // it is spread across ordinary `?…` functions then
                            // the BINDING is losing them and the reader is what
                            // needs work.
                            *res.emit
                                .entry(format!("emit-residue-unbound|{}", mangling_class(name)))
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        // 1e'. SCRATCH INSTRUMENT (W-ADJUST, boards #127/#128) — see [`row_dump`].
        //      Off unless `C2RS_ROW_DUMP` is set; changes no count either way.
        row_dump(
            src,
            &census,
            captured.ref_obj.text_comdat_functions().as_deref(),
        );
    }

    // 1f. The emit binding's own self-report. Printed on every scan, whether or
    //     not the obj was readable, so a residue cannot disappear by the route
    //     `prod-entered-untagged` had to be dragged out of: an absence.
    if let Some(b) = captured.bundle.emit_binding() {
        for (key, n) in [
            ("emit-records", b.records),
            ("emit-record-outside", b.records_outside),
            ("emit-record-nameless", b.records_nameless),
            ("emit-row-conflict", b.dropped_row_conflict),
            ("emit-name-conflict", b.dropped_name_conflict),
        ] {
            if n > 0 {
                *res.emit.entry(key.into()).or_insert(0) += n;
            }
        }
        let (found, accounted) = b.accounting();
        if found != accounted {
            *res.emit.entry("emit-accounting-broken".into()).or_insert(0) += 1;
        }
        // **ARITY (#144, W-VGL).** `emit-records` counts records as entities and
        // the totality identity above balances them against a residue; neither
        // can see a change that keeps every record and loses something *inside*
        // one. `record_offsets` is the contents — a property of the FRAMING only
        // — so it is reported as its own row and its own broken-invariant count.
        // A naming change (W-VGL's `26` separator) must move `emit-record-nameless`
        // and leave `emit-record-offsets` alone; a framing change moves both, and
        // a report carrying only the first cannot tell them apart.
        let (records, offsets) = b.arity();
        *res.emit.entry("emit-record-offsets".into()).or_insert(0) += offsets;
        if records != offsets {
            *res.emit.entry("emit-arity-broken".into()).or_insert(0) += 1;
        }
    }

    if let Some(gl) = captured.bundle.get("gl") {
        let (dropped, mangled) = c2_il::gl_symbol_conflicts(gl);
        if dropped > 0 {
            *res.bind_checks
                .entry("gl-token-ambiguous-dropped".to_string())
                .or_insert(0) += dropped;
        }
        if mangled > 0 {
            *res.bind_checks
                .entry("gl-token-conflict-mangled".to_string())
                .or_insert(0) += mangled;
        }
    }

    // 1g. **THE TWO SPLITTERS DISAGREE, AND THE CEILING IS COMPUTED WITH THE
    //     WRONG ONE** (ROADMAP §10.11 / §10.12, W-PHASE6).
    //
    //     `emit_set_reachable_tus` — the "25 of 871" emit-set ceiling, and the
    //     `at most 19 TUs, ever` claim §10 builds its re-plan on — filters on
    //     `fn_total == emit-emitted`, and its doc comment asserts that
    //     `fn_total` "is exactly that segment count", meaning the count
    //     `PortC2::build` consumes. **It is not.**
    //
    //     | count | comes from | anchored on |
    //     |---|---|---|
    //     | `fn_total` | `census_functions()` / `split_function_bodies_at` | `LO_MARKER` = `4C 4F 11` |
    //     | what the port consumes | `IlBundle::functions()` / `split_functions_at` | `FN_START` = `4F 1F` |
    //
    //     §10.12 named the population that separates them: a `??__E`/`??__F`
    //     dynamic-initializer thunk carries a **bare `4C`** with no `4F 11`, so
    //     the census sees 0 segments where the gate sees 1.
    //
    //     So this counts the ceiling BOTH ways and publishes the disagreement.
    //     It does **not** replace `fn_total` or `emit-emitted` — a ceiling
    //     silently recomputed under the same name would be indistinguishable
    //     from the old one being right, and which of the two is the ceiling is
    //     not something this instrument gets to decide.
    //
    //     **This used to read `functions().map(|f| f.len())`, and that made the
    //     whole block near-vacuous**: `functions()` is an *acceptance* decision,
    //     so it returns `None` for every `vocab-gap` TU — 865 of 871 — and the
    //     gate-anchored ceiling was knowable for exactly the 6 that already
    //     match, five of which define zero functions. The lane that added this
    //     block measured that and declined to report a number, correctly.
    //     `IlBundle::ex_segment_count` is the pure reader that closes it: the
    //     `4F 1F` split with no acceptance decision attached, available on a
    //     bundle `functions()` refuses.
    //
    //     Its `None` means **no `.ex` at all**, which is not "zero functions" —
    //     so it keeps feeding `emit-gate-segments-unknown` rather than being
    //     unwrapped to 0. Every key here is NEW; none of the existing ones is
    //     touched.
    let gate_segments = captured.bundle.ex_segment_count();
    {
        let comdats = res.emit.get("emit-emitted").copied().unwrap_or(0);
        match gate_segments {
            None => {
                *res.emit
                    .entry("emit-gate-segments-unknown".into())
                    .or_insert(0) += 1;
            }
            Some(n) => {
                *res.emit.entry("emit-gate-segments-known".into()).or_insert(0) += 1;
                *res.emit.entry("emit-gate-segments".into()).or_insert(0) += n;
                if n == res.fn_total {
                    *res.emit.entry("emit-splitter-agree".into()).or_insert(0) += 1;
                } else {
                    *res.emit.entry("emit-splitter-disagree".into()).or_insert(0) += 1;
                    // Signed, in two keys rather than one absolute value: §10.12
                    // predicts the gate seeing MORE segments than the census
                    // (`??__E` with a bare `4C`), and a count that cannot tell
                    // that from the opposite is not evidence for it.
                    let key = if n > res.fn_total {
                        "emit-splitter-gate-sees-more"
                    } else {
                        "emit-splitter-census-sees-more"
                    };
                    *res.emit.entry(key.into()).or_insert(0) += 1;
                }
                // The ceiling, gate-anchored, and how it moves against the
                // `LO`-anchored one. `enter`/`leave` are the deliverable: the
                // net is not enough, because two TUs swapping sides is a
                // different fact from nothing happening.
                let (lo_reach, gate_reach) = (res.fn_total == comdats, n == comdats);
                if gate_reach {
                    *res.emit.entry("emit-set-ceiling-gate".into()).or_insert(0) += 1;
                }
                match (gate_reach, lo_reach) {
                    (true, false) => {
                        *res.emit.entry("emit-set-ceiling-gate-enter".into()).or_insert(0) += 1
                    }
                    (false, true) => {
                        *res.emit.entry("emit-set-ceiling-gate-leave".into()).or_insert(0) += 1
                    }
                    _ => {}
                }
            }
        }
    }

    // 1h. **FACTORS C AND D** (`docs/ROADMAP.md` §10.19, board #160).
    //
    //     §10.19 factored Phase 7 into four predicates over the graded TUs and
    //     found `A∧B∧C∧D` reproduces the match set exactly. **A and B were
    //     already keys here; C and D were not** — they lived in a one-off
    //     analysis, which means the project's central planning model could not
    //     regress-detect and the next reader re-derives it by hand. §10.14 is
    //     the record of what a by-hand re-derivation of a rule the harness owns
    //     costs.
    //
    //     | factor | predicate | key |
    //     |---|---|---|
    //     | A | `.ex` segments == obj `.text` COMDATs | `emit-set-ceiling-gate` (1g) |
    //     | B | every emitted symbol binds | `emit-set-ceiling-today` (1e) |
    //     | **C** | obj section set ⊆ [`PORT_WRITER_SECTIONS`] | `emit-sec-reachable` |
    //     | **D** | every emitted COMDAT in the port's codegen class | `emit-class-complete` |
    //
    //     **C reads the obj afresh and shares no variable with 1e, 1g or step
    //     3.** That is not fussiness: §10.18 is this file's own record of a
    //     variable with two consumers being changed for one of them, moving 865
    //     TUs between classes with nothing red. The question here — *what
    //     sections does this obj have* — is not the question `text_comdat_*`
    //     asks (*which COMDAT `.text` leaders are there*), and a `.data` or
    //     `.bss` is invisible to the second.
    //
    //     **D is built from the keys 1e already computed, never from a second
    //     in-class rule.** `emit-in-class` is the census's own
    //     `FnVerdict::in_class()` joined through the binding, so an emitted
    //     symbol that fails to bind is *not* counted in class — D fails closed,
    //     which is the direction that cannot flatter it.
    {
        match captured.ref_obj.section_names() {
            None => {
                // Fail closed and SAY SO. An unreadable obj contributes no
                // section vocabulary and is outside C — never "carries nothing
                // outside the writer's set", which would put it inside.
                *res.emit.entry("emit-sec-unreadable".into()).or_insert(0) += 1;
            }
            Some(names) => {
                *res.emit.entry("emit-sec-readable".into()).or_insert(0) += 1;
                *res.emit.entry("emit-sec-count".into()).or_insert(0) += names.len();
                let distinct: std::collections::BTreeSet<&str> =
                    names.iter().map(String::as_str).collect();
                *res.emit.entry("emit-sec-distinct".into()).or_insert(0) += distinct.len();
                let mut extra = 0usize;
                for n in &distinct {
                    // One per DISTINCT name per TU, so the aggregated row reads
                    // "objs carrying this section" and not "sections named this"
                    // — under `/Gy` the second would count 158 `.text`s in one
                    // obj and no reader of the table would know which it was.
                    *res.emit.entry(format!("emit-sec-name|{n}")).or_insert(0) += 1;
                    if !PORT_WRITER_SECTIONS.contains(n) {
                        extra += 1;
                        *res.emit.entry(format!("emit-sec-extra|{n}")).or_insert(0) += 1;
                    }
                }
                let key = if extra == 0 {
                    "emit-sec-reachable"
                } else {
                    "emit-sec-blocked"
                };
                *res.emit.entry(key.into()).or_insert(0) += 1;
            }
        }
        // Factor D. Its population is **"1e's join actually ran"**, which the
        // presence of the `emit-emitted` key states exactly: 1e writes it
        // unconditionally once the census decoded and the obj's emitted set
        // read, including when the value is 0. A TU whose obj did not decode,
        // or that has no census at all, is therefore *outside* D rather than
        // vacuously inside it on `0 == 0` — the flattering direction, and the
        // one §9.18.8 records twelve times.
        if let Some(&emitted) = res.emit.get("emit-emitted") {
            let in_class = res.emit.get("emit-in-class").copied().unwrap_or(0);
            *res.emit.entry("emit-class-known".into()).or_insert(0) += 1;
            if emitted == in_class {
                *res.emit.entry("emit-class-complete".into()).or_insert(0) += 1;
            }
            // The vacuous half, counted separately because §10.19 says "6 of
            // those 8 emit nothing" and a factor that is mostly satisfied by
            // empty objs is a different fact from one that is not.
            if emitted == 0 {
                *res.emit.entry("emit-class-empty".into()).or_insert(0) += 1;
            }
        }
    }

    // 2. Optional soundness lane: standalone-c2 replay must reproduce the
    //    pipeline obj on this real bundle.
    if do_replay {
        let ref_obj_path = captured.ref_obj_path.clone();
        res.replay_ok = Some(
            match tc.replay(&captured, &work.join("replay_il"), &ref_obj_path) {
                Ok(obj) => {
                    matches!(ObjImage::diff(&captured.ref_obj, &obj), ObjDiff::Identical)
                }
                Err(_) => false,
            },
        );
        // The replay must write to the reference's own `-Fo` path (that string
        // is inside the obj), which under a cache is the cache entry itself —
        // so restore the captured bytes afterwards. Without this a diverging
        // replay would leave its own output behind as the "cached capture",
        // i.e. the scan would poison its own cache with the thing it was
        // checking for.
        if cache.is_some() {
            let _ = std::fs::write(&ref_obj_path, captured.ref_obj.as_bytes());
        }
    }

    // 3. Vocabulary: can the IL model even decode this bundle's functions?
    //
    //    **This calls `functions()` itself, and must never be folded back into
    //    1g's `gate_segments`.** It was, briefly, on the true observation that
    //    `functions()` is pure and was being evaluated twice. Then 1g's reader
    //    changed from `functions()` to `ex_segment_count()` — correctly, because
    //    the ceiling needs the `4F 1F` split on TUs the gate refuses — and the
    //    shared variable silently carried that change into the class decision.
    //    `ex_segment_count` is `None` only when there is no `.ex` at all, so the
    //    vocab-gap test stopped firing: **`vocab-gap 865 -> 0` and
    //    `codegen-gap 0 -> 865` in one run**, with `mismatch` still 0 and
    //    `match` still 6.
    //
    //    That is the FALSE-GREEN direction and it is why this comment is here:
    //    a report reading "the port now decodes 865 TUs and merely declines to
    //    lower them" is a headline, and nothing in the run was red. The two
    //    predicates answer different questions —
    //
    //      | question | reader |
    //      |---|---|
    //      | how many `.ex` segments are there | `ex_segment_count` — pure, always answers |
    //      | will the gate ACCEPT this bundle  | `functions()` — an acceptance decision |
    //
    //    — and sharing one call between them makes the second silently inherit
    //    whatever the first is changed to.
    //
    //    `the_class_predicate_is_not_the_segment_counter` pins the *premise*
    //    (the two readers really do disagree on a bundle the gate refuses), so
    //    the paragraph above is executable rather than folklore. **It does not
    //    catch a re-fold** — that shows up only as the class counts moving, and
    //    `classify_one` needs a toolchain, so no portable test reaches it. The
    //    evidence on record is a 3-TU scan run both ways: folded gives
    //    `codegen-gap 2 / vocab-gap 0`, unfolded `codegen-gap 0 / vocab-gap 2`.
    //    Stated plainly so the next reader does not take the test for a guard it
    //    is not.
    //    **W-R1c: the acceptance question now has TWO paths and must be asked
    //    through one predicate.** `IlBundle::decodes()` is
    //    `functions().is_some() || dyninit_tu().is_some()`. Calling `functions()`
    //    alone here would file every converted `??__E` dynamic-initializer TU as
    //    `vocab-gap` — "the port could not decode it" — while the port emitted a
    //    byte-exact obj for it, which is the same mis-attribution this comment
    //    block already warns about in the other direction.
    if !captured.bundle.decodes() {
        res.class = TuClass::VocabGap;
        res.reason = "il function decode failed".to_string();
        res.detail = format!(
            ".ex {} B, {} .gl names — c2_il::functions() and dyninit_tu() both None",
            res.ex_len, res.fn_names
        );
        return res;
    }

    // 4. The port, threaded with the reference's exact -Fo path (S_OBJNAME).
    let obj_name = c2_reference::to_wibo_path(&captured.ref_obj_path);
    // The obj's shape depends on argv the IL bundle does not record: /Gy
    // (implied by /O1 and /O2) puts each function in its own COMDAT .text.
    // Pass the project's real flags so the port can refuse rather than emit a
    // packed .text against a per-function-COMDAT reference.
    let port = PortC2::new(obj_name.clone()).with_function_level_linking(gy);
    match port.compile_to(&captured.bundle, &obj_name) {
        Ok(obj) => match ObjImage::diff(&captured.ref_obj, &obj) {
            ObjDiff::Identical => {
                res.class = TuClass::Match;
                res.reason = "byte-exact".to_string();
            }
            ObjDiff::Differs { first_offset, .. } => {
                res.class = TuClass::Mismatch;
                res.reason = "bytes diverge".to_string();
                res.detail = format!(
                    "first divergence at {first_offset} (ref {} B, port {} B)",
                    captured.ref_obj.len(),
                    obj.len()
                );
            }
        },
        Err(BackendError::NotImplemented(msg)) => {
            res.class = TuClass::CodegenGap;
            res.reason = clip(&msg, 80);
            res.detail = clip(&msg, 200);
        }
        Err(e) => {
            res.class = TuClass::PortError;
            res.reason = clip(&e.to_string(), 80);
            res.detail = clip(&e.to_string(), 200);
        }
    }
    res
}

/// Run the scan: worker pool over the source list, per-TU work subdirs.
/// `progress` is called per finished TU (from worker threads, serialized).
///
/// The scan also records **what produced the numbers** — see
/// [`Provenance`]: a scan whose corpus moved and whose `fn_total` matched anyway
/// is a scan that lied, and the denominator guard alone is proven insufficient.
pub fn gap_scan(
    tc: &Toolchain,
    cfg: &GapConfig,
    progress: &(dyn Fn(usize, usize, &TuResult) + Sync),
) -> std::io::Result<GapReport> {
    let provenance = Provenance::collect(tc, cfg.cwd.as_deref());
    let cache = match &cfg.cache {
        Some(root) => Some(CaptureCache::new(
            root.clone(),
            tc,
            cfg.cwd.as_deref(),
            cfg.validate_cache,
        )?),
        None => None,
    };
    let sources: Vec<&str> = cfg
        .sources
        .iter()
        .map(|s| s.as_str())
        .take(cfg.limit.unwrap_or(usize::MAX))
        .collect();
    let total = sources.len();
    std::fs::create_dir_all(&cfg.work)?;

    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<TuResult>> = Mutex::new(Vec::with_capacity(total));
    let jobs = cfg.jobs.max(1).min(total.max(1));

    std::thread::scope(|scope| {
        for worker in 0..jobs {
            let sources = &sources;
            let next = &next;
            let done = &done;
            let results = &results;
            let cache = cache.as_ref();
            scope.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= sources.len() {
                    break;
                }
                let src = sources[i];
                let work = cfg.work.join(format!("tu{i:05}"));
                let _ = std::fs::create_dir_all(&work);
                let do_replay = cfg.replay_every > 0 && i % cfg.replay_every == 0;
                let r = scan_one(tc, cfg, cache, src, &work, do_replay);
                // Bound scratch usage: captured bundles/objs for huge scans
                // add up; the JSONL is the durable record.
                let _ = std::fs::remove_dir_all(&work);
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                progress(n, total, &r);
                results.lock().unwrap().push(r);
                let _ = worker;
            });
        }
    });

    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| a.src.cmp(&b.src));

    let cache_stats = cache
        .as_ref()
        .map(|c| c.stats())
        .unwrap_or_default();

    if let Some(path) = &cfg.jsonl {
        let mut f = std::fs::File::create(path)?;
        // Record 0 is the provenance header (roadmap #46). Per-TU rows below are
        // unchanged and carry no `record` field, so two scans' rows stay
        // byte-comparable; a consumer skips this one with
        // `if r.get("record"): continue`.
        let extra: Vec<(&str, String)> = vec![
            (
                "cache_root",
                match &cfg.cache {
                    Some(p) => crate::jstr(&p.display().to_string()),
                    None => "null".to_string(),
                },
            ),
            (
                "cache_context",
                match cache.as_ref() {
                    Some(c) => crate::jstr(&c.context_digest()),
                    None => "null".to_string(),
                },
            ),
            ("cache_hits", cache_stats.hits.to_string()),
            ("cache_misses", cache_stats.misses.to_string()),
            ("cache_validated", cache_stats.validated.to_string()),
            ("cache_poisoned", cache_stats.poisoned.to_string()),
            ("tu_count", results.len().to_string()),
            ("replay_every", cfg.replay_every.to_string()),
            ("flags", crate::jstr(&cfg.flags.join(" "))),
        ];
        writeln!(f, "{}", provenance.to_json(&extra))?;
        for r in &results {
            let blockers = r
                .fn_blockers
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let frames = r
                .fn_frames
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let cflow = r
                .fn_cflow
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let eh = r
                .fn_eh
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let dispatch = r
                .fn_dispatch
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let complete = r
                .fn_complete
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let prod = r
                .fn_prod
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let gate = r
                .fn_gate_refusals
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let binds = r
                .bind_checks
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let emit = r
                .emit
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let emit_blockers = r
                .emit_blockers
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            writeln!(
                f,
                "{{\"src\":{},\"class\":{},\"reason\":{},\"detail\":{},\"ex_len\":{},\"fn_names\":{},\"replay_ok\":{},\"fn_total\":{},\"fn_in_class\":{},\"fn_blockers\":{{{}}},\"fn_frames\":{{{}}},\"fn_cflow\":{{{}}},\"fn_eh\":{{{}}},\"fn_dispatch\":{{{}}},\"fn_complete\":{{{}}},\"fn_prod\":{{{}}},\"fn_gate_refusals\":{{{}}},\"bind_checks\":{{{}}},\"emit\":{{{}}},\"emit_blockers\":{{{}}}}}",
                crate::jstr(&r.src),
                crate::jstr(r.class.label()),
                crate::jstr(&r.reason),
                crate::jstr(&r.detail),
                r.ex_len,
                r.fn_names,
                match r.replay_ok {
                    None => "null".to_string(),
                    Some(b) => b.to_string(),
                },
                r.fn_total,
                r.fn_in_class,
                blockers,
                frames,
                cflow,
                eh,
                dispatch,
                complete,
                prod,
                gate,
                binds,
                emit,
                emit_blockers,
            )?;
        }
    }

    let report = GapReport {
        results,
        provenance: Some(provenance),
        cache: cache_stats,
    };
    // Board #159's step one: print the names the scan just classified. Written
    // last, from the collected results, so the ranking is over the whole scan
    // and not over whatever one worker happened to see.
    if let Some(p) = witness_path() {
        write_witness(&report, p)?;
    }
    // **The two segment counts, side by side** (step 1g). Printed here rather
    // than in the caller's report because the classification lives in this file;
    // printed ALWAYS, as counts, because a disagreement nobody prints is the
    // absence-reads-as-success shape this project has paid for repeatedly.
    // Neither number is presented as "the" ceiling: which anchor the ceiling
    // should use is a decision, and this is the measurement it needs.
    let (known, unknown, agree, disagree, gate_more, census_more, gate_ceil, enter, leave) =
        report.splitter_disagreement();
    let (viol, viol_pop) = report.emit_set_violations_gate();
    println!(
        "\nSPLITTER ANCHORS (ROADMAP §10.11/§10.12) — the census splits `.ex` on `4C 4F 11`, \
         the port on `4F 1F`\n\
         \x20 gate-side segment count KNOWN for {known} of {} captured TUs; UNKNOWN for {unknown} \
         (no `.ex` at all). Read through `IlBundle::ex_segment_count`, a PURE reader of the \
         `4F 1F` split — not `functions()`, which is an acceptance decision and returns None \
         for every vocab-gap TU, leaving this knowable for only the 6 that already match.\n\
         \x20 of the {known} known: {agree} agree with `fn_total`, {disagree} disagree \
         ({gate_more} where the gate sees MORE segments, {census_more} where the census does)\n\
         \x20 emit-set ceiling, LO-anchored, over ALL graded TUs: {}\n\
         \x20 emit-set ceiling, GATE-anchored (`4F 1F`, what the port consumes), over the \
         {known} known: {gate_ceil} (+{enter} entering, -{leave} leaving vs the LO-anchored set)\n\
         \x20 gate-anchored control on matching TUs: {viol} violations over {viol_pop} matching \
         TUs whose gate count is known",
        known + unknown,
        report.emit_set_reachable_tus().len(),
    );
    print_factorization(&report);
    Ok(report)
}

/// **The Phase 7 factorization, printed on every scan** (`docs/ROADMAP.md`
/// §10.19, board #160).
///
/// Printed from here rather than from `main.rs` for the same reason the splitter
/// block above is: the predicates live in this file, and a report assembled
/// somewhere else is a second place the definitions can drift.
///
/// **Everything here is a count.** There is no "factorization OK" line: a joint
/// that reproduced nothing would print zeros against a nonzero match count,
/// which is visible, where a status would not be (`docs/GAPS.md` §7).
fn print_factorization(report: &GapReport) {
    let graded = report.graded().count();
    let [a, b, c, d, a_lo, bc, abcd] = report.factor_counts();
    let (bad, match_tus) = report.factor_control_on_match_tus();
    let matched: Vec<&str> = report
        .results
        .iter()
        .filter(|r| r.class == TuClass::Match)
        .map(|r| r.src.as_str())
        .collect();
    let all = report.factor_all_tus();
    let vocab = report.section_vocabulary();
    let unreadable = report.emit_total("emit-sec-unreadable");
    let readable = report.emit_total("emit-sec-readable");

    println!(
        "\nPHASE 7 FACTORS (ROADMAP §10.19, board #160) — four NECESSARY conditions on a \
         byte-exact obj, over {graded} graded TUs\n\
         \x20 A  emit set reachable   `.ex` segments == obj `.text` COMDATs   {a:>5}  \
         (gate-anchored `4F 1F`; {a_lo} on the census's `4C 4F 11` anchor)\n\
         \x20 B  binding complete     every emitted symbol binds              {b:>5}\n\
         \x20 C  section shape        obj sections subset of the writer's {}   {c:>5}\n\
         \x20 D  codegen breadth      every emitted COMDAT in class           {d:>5}  \
         ({} of them emit nothing at all)\n\
         \x20 B and C jointly (the near-term ceiling, measured per TU — NOT a product of \
         marginals, §8.6): {bc}\n\
         \x20 A and B and C and D: {abcd}   |   TUs the differential graded `match`: {}\n\
         \x20 section headers: {readable} objs read, {unreadable} did not decode (outside C, \
         fail-closed)",
        PORT_WRITER_SECTIONS.len(),
        report.emit_total("emit-class-empty"),
        matched.len(),
    );
    // The set identity, by name. §10.19's claim is that the joint IS the match
    // set; two sets of the same size that differ by a swap would read as equal
    // if this printed only counts.
    if all == matched {
        println!(
            "\x20 the joint is EXACTLY the match set ({} TUs, by name): {}",
            all.len(),
            all.join(", ")
        );
    } else {
        let only_joint: Vec<&&str> = all.iter().filter(|s| !matched.contains(s)).collect();
        let only_match: Vec<&&str> = matched.iter().filter(|s| !all.contains(s)).collect();
        println!(
            "\x20 the joint is NOT the match set — {} in the joint only ({:?}), {} matching but \
             outside some factor ({:?}). The second list is the ALARM: every factor is meant \
             to be NECESSARY, so a matching TU outside one voids the bound it carries.",
            only_joint.len(),
            only_joint,
            only_match.len(),
            only_match,
        );
    }
    println!(
        "\x20 known-answer control — matching TUs failing each factor (all must be 0, over \
         {match_tus} matching TUs): A {} B {} C {} D {}",
        bad[0], bad[1], bad[2], bad[3]
    );
    // The vocabulary, in full. It is finite, and its size is the headline: it is
    // what makes C the one factor with a short route to closure.
    println!(
        "\x20 SECTION VOCABULARY — {} distinct names across the workload (objs carrying each):",
        vocab.len()
    );
    for (name, objs) in &vocab {
        let mine = if PORT_WRITER_SECTIONS.contains(&name.as_str()) {
            "writer"
        } else {
            "  ---"
        };
        println!("\x20   {objs:>5} objs  [{mine}]  {name}");
    }
    // The one ACTIONABLE row of the whole block: TUs whose only remaining
    // factor is D. Printed as a list with each one's distance, because a
    // count would say "16" and name nothing to work on.
    let frontier = report.factor_frontier();
    println!(
        "\x20 FRONTIER — {} graded TUs satisfy A and B and C and are NOT a match, so codegen \
         breadth (D) is the whole remaining distance (blocked emitted | emitted | src):",
        frontier.len()
    );
    for (r, blocked) in frontier.iter().take(40) {
        println!(
            "\x20   {blocked:>4} | {:>4} | {}",
            r.emit.get("emit-emitted").copied().unwrap_or(0),
            r.src
        );
    }
    if frontier.len() > 40 {
        println!("\x20   … and {} more", frontier.len() - 40);
    }
    // …and the route: which name to teach next, by TUs brought into reach.
    let ladder = report.section_ladder();
    println!(
        "\x20 GREEDY LADDER — next section name to teach the writer, and the resulting C \
         ({} steps from {c} to {readable}):",
        ladder.len()
    );
    let mut prev = c;
    for (name, reach) in &ladder {
        println!("\x20   +{name:<12} C = {reach:>5}   (+{})", reach - prev);
        prev = *reach;
    }
}

/// Rank one scan's [`WitnessRow`]s per bucket. Pure over `results`, so the unit
/// test below grades it without a toolchain.
pub fn witness_buckets(results: &[TuResult]) -> Vec<WitnessBucket> {
    // bucket -> (symbols, TUs, in-gl, name -> (occurrences, TUs, example TU))
    type PerName = BTreeMap<String, (usize, std::collections::BTreeSet<String>, String)>;
    #[allow(clippy::type_complexity)]
    let mut agg: BTreeMap<
        String,
        (usize, std::collections::BTreeSet<String>, usize, usize, PerName),
    > = BTreeMap::new();
    for r in results {
        for w in &r.emit_witness {
            let e = agg.entry(w.bucket.clone()).or_insert_with(|| {
                (0, std::collections::BTreeSet::new(), 0, 0, BTreeMap::new())
            });
            e.0 += 1;
            e.1.insert(r.src.clone());
            e.2 += usize::from(w.in_gl_runs);
            e.3 += usize::from(w.in_gl_index);
            let n = e
                .4
                .entry(w.name.clone())
                .or_insert_with(|| (0, std::collections::BTreeSet::new(), r.src.clone()));
            n.0 += 1;
            n.1.insert(r.src.clone());
        }
    }
    let mut out: Vec<WitnessBucket> = agg
        .into_iter()
        .map(|(bucket, (symbols, tus, in_gl_runs, in_gl_index, names))| {
            let mut ranked: Vec<(String, usize, usize, String)> = names
                .into_iter()
                .map(|(name, (count, tus, example))| (name, count, tus.len(), example))
                .collect();
            // Frequency descending, then name ascending — a total order, so two
            // runs of the same scan print the same table.
            ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            WitnessBucket {
                bucket,
                symbols,
                tus: tus.len(),
                in_gl_runs,
                in_gl_index,
                names: ranked,
            }
        })
        .collect();
    out.sort_by(|a, b| b.symbols.cmp(&a.symbols).then_with(|| a.bucket.cmp(&b.bucket)));
    out
}

/// Write the ranked summary to `path` and every row to `<path>.rows.tsv`.
///
/// Every line is a **count**. There is no "no witnesses" status: a scan that
/// collected nothing prints `0 rows` against the scan's own residue totals, and
/// those totals disagreeing with the row count is the check that the list is
/// complete (`docs/GAPS.md` §7 — absence must not read as success).
fn write_witness(report: &GapReport, path: &std::path::Path) -> std::io::Result<()> {
    let buckets = witness_buckets(&report.results);
    let rows: usize = report.results.iter().map(|r| r.emit_witness.len()).sum();

    let mut raw = std::io::BufWriter::new(std::fs::File::create(path.with_extension("rows.tsv"))?);
    writeln!(raw, "src\tbucket\tin_gl_runs\tin_gl_index\tname")?;
    for r in &report.results {
        for w in &r.emit_witness {
            writeln!(
                raw,
                "{}\t{}\t{}\t{}\t{}",
                r.src,
                w.bucket,
                u8::from(w.in_gl_runs),
                u8::from(w.in_gl_index),
                w.name
            )?;
        }
    }
    raw.flush()?;

    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(
        f,
        "WITNESS LIST — the emitted-symbol residue, named (board #159)\n\
         scan: {} TUs, {rows} witness rows over {} buckets\n",
        report.results.len(),
        buckets.len()
    )?;
    // The cross-check that makes the list evidence rather than a plausible
    // sample: the rows must sum, per bucket, to the counter the same loop
    // incremented. Printed per bucket, as counts, both sides.
    for b in &buckets {
        let counted = report.emit_total(&b.bucket);
        writeln!(
            f,
            "== {} — {} symbols / {} TUs / {} distinct names\n\
             \x20  name present in `.gl`: {} by `mangled_names` (BLIND to `??`-names), \
             {} by `gl_symbol_index` — two predicates, neither is this bucket's\n\
             \x20  cross-check vs the scan's own counter: rows {} vs counter {} — agree: {}",
            b.bucket,
            b.symbols,
            b.tus,
            b.names.len(),
            b.in_gl_runs,
            b.in_gl_index,
            b.symbols,
            counted,
            b.symbols == counted
        )?;
        for (i, (name, count, tus, example)) in b.names.iter().take(WITNESS_CAP).enumerate() {
            writeln!(f, "  {:>4}. {count:>6} sym {tus:>4} TU  {name}  [{example}]", i + 1)?;
        }
        if b.names.len() > WITNESS_CAP {
            let shown: usize = b.names.iter().take(WITNESS_CAP).map(|(_, c, _, _)| *c).sum();
            writeln!(
                f,
                "  … and {} more distinct names covering {} symbols (top {WITNESS_CAP} cover {shown})",
                b.names.len() - WITNESS_CAP,
                b.symbols - shown
            )?;
        }
        writeln!(f)?;
    }
    f.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cl_error_normalization_extracts_code() {
        let blob = "capture failed\n  stdout:\n    x.cpp\n    src/x.h(12): fatal error C1083: Cannot open include file: 'foo.h': No such file\n";
        let (key, detail) = normalize_cl_error(blob);
        assert_eq!(key, "C1083");
        assert!(detail.contains("foo.h"));
    }

    #[test]
    fn cl_error_normalization_survives_codeless_blobs() {
        let (key, _) = normalize_cl_error("wibo: something exploded\n");
        assert_eq!(key, "wibo: something exploded");
    }

    fn mk_report(results: Vec<TuResult>) -> GapReport {
        GapReport {
            results,
            provenance: None,
            cache: crate::capture_cache::CacheStats::default(),
        }
    }

    fn mk(reason: &str) -> TuResult {
        TuResult {
            src: "s".into(),
            class: TuClass::CodegenGap,
            reason: reason.into(),
            detail: String::new(),
            ex_len: 0,
            fn_names: 0,
            replay_ok: None,
            fn_total: 0,
            fn_in_class: 0,
            fn_blockers: BTreeMap::new(),
            fn_frames: BTreeMap::new(),
            fn_cflow: BTreeMap::new(),
            fn_eh: BTreeMap::new(),
            fn_dispatch: BTreeMap::new(),
            fn_complete: BTreeMap::new(),
            fn_prod: BTreeMap::new(),
            fn_gate_refusals: BTreeMap::new(),
            bind_checks: BTreeMap::new(),
            emit: BTreeMap::new(),
            emit_blockers: BTreeMap::new(),
            emit_witness: Vec::new(),
        }
    }

    #[test]
    fn report_ranks_reasons_by_count() {
        let rep = mk_report(vec![mk("b"), mk("a"), mk("b")]);
        assert_eq!(
            rep.top_reasons(TuClass::CodegenGap),
            vec![("b".to_string(), 2), ("a".to_string(), 1)]
        );
        assert_eq!(rep.count(TuClass::Match), 0);
    }

    /// **The witness list ranks by frequency across TUs, and its per-bucket
    /// symbol total is the number the scan's own counter must equal** (board
    /// #159). Both halves matter: a list ranked per TU would name the largest
    /// TU's symbols rather than the workload's, and a list whose total does not
    /// reconcile with `emit-unbound-*` is a second measurement of the residue —
    /// which is exactly the defect `ROADMAP.md` §10.14 records.
    #[test]
    fn the_witness_list_ranks_across_tus_and_reconciles_with_its_counter() {
        let row = |bucket: &str, name: &str, in_gl: bool| WitnessRow {
            bucket: bucket.into(),
            name: name.into(),
            in_gl_runs: in_gl,
            in_gl_index: in_gl,
        };
        let ord = "emit-unbound-no-record|ordinary";
        let spc = "emit-unbound-no-record|special-generated";
        let mut a = mk("x");
        a.src = "a.cpp".into();
        a.emit_witness = vec![row(ord, "?rare@@YAXXZ", false), row(ord, "?common@@YAXXZ", true)];
        a.emit.insert(ord.into(), 2);
        let mut b = mk("x");
        b.src = "b.cpp".into();
        b.emit_witness = vec![row(ord, "?common@@YAXXZ", true), row(spc, "??_7C@6B@", false)];
        b.emit.insert(ord.into(), 1);
        b.emit.insert(spc.into(), 1);

        let rep = mk_report(vec![a, b]);
        let buckets = witness_buckets(&rep.results);
        assert_eq!(buckets.len(), 2, "one entry per bucket that collected a row");
        let o = &buckets[0];
        assert_eq!(o.bucket, ord, "buckets rank by symbol count, largest first");
        assert_eq!((o.symbols, o.tus, o.names.len(), o.in_gl_runs, o.in_gl_index), (3, 2, 2, 2, 2));
        assert_eq!(
            o.names[0],
            ("?common@@YAXXZ".to_string(), 2, 2, "a.cpp".to_string()),
            "the name seen in two TUs outranks the name seen once, and carries an example TU"
        );
        assert_eq!(o.names[1].1, 1);
        assert_eq!(buckets[1].symbols, 1);

        // The reconciliation the report prints: rows summed per bucket equal the
        // counter the same loop incremented. This is the check §10.14's reader
        // could not have passed, because it had no counter to reconcile against.
        for b in &buckets {
            assert_eq!(
                b.symbols,
                rep.emit_total(&b.bucket),
                "{}: witness rows must equal the scan's own counter",
                b.bucket
            );
        }
        let rows: usize = buckets.iter().map(|b| b.symbols).sum();
        assert_eq!(rows, 4, "every row lands in exactly one bucket");
    }

    /// A TU whose emitted census is spelled out: `emitted` symbols, of which
    /// `bound` bound, `in_class` in class, and `gen`/`other` in the two residue
    /// buckets.
    fn mk_emit(
        class: TuClass,
        emitted: usize,
        bound: usize,
        in_class: usize,
        gen: usize,
        other: usize,
    ) -> TuResult {
        let mut r = mk("x");
        r.class = class;
        r.fn_total = emitted;
        r.fn_in_class = in_class;
        for (k, n) in [
            ("emit-emitted", emitted),
            ("emit-bound", bound),
            ("emit-in-class", in_class),
            ("emit-residue-generated", gen),
            ("emit-residue-unbound", other),
        ] {
            if n > 0 {
                r.emit.insert(k.into(), n);
            }
        }
        r
    }

    /// The read-out and its residue aggregate across TUs, and the denominator is
    /// the emitted count — never reduced by what failed to bind, which would
    /// inflate the ratio.
    #[test]
    fn the_emitted_census_aggregates_and_keeps_its_denominator_whole() {
        let a = mk_emit(TuClass::VocabGap, 100, 90, 20, 4, 6);
        let b = mk_emit(TuClass::VocabGap, 50, 45, 5, 1, 4);
        let rep = mk_report(vec![a, b]);
        assert_eq!(
            rep.emit_coverage(),
            (25, 150),
            "the read-out is in-class over EMITTED, not over bound"
        );
        assert_eq!(
            rep.emit_residue(),
            (5, 10),
            "the residue splits into generated-with-no-body and unexplained"
        );
        assert_eq!(
            rep.emit_total("emit-bound") + rep.emit_residue().0 + rep.emit_residue().1,
            150,
            "bound + residue must account for every emitted symbol"
        );
    }

    /// GROUND TRUTH, and its NEGATIVE CONTROL. On a byte-exact TU the oracle has
    /// already graded the whole symbol table, so the binding's answer there is
    /// checkable: every emitted symbol must bind to an in-class row.
    ///
    /// The guard's quantity — one `match` TU with 40 emitted functions — is held
    /// FIXED across the two halves; only how many of them the binding claimed
    /// moves. Without that, the second half could pass by the TU no longer being
    /// a `match` at all, and the assertion under test would never run.
    #[test]
    fn a_match_tu_whose_emitted_symbols_do_not_all_bind_is_a_binding_defect() {
        let good = mk_emit(TuClass::Match, 40, 40, 40, 0, 0);
        let rep = mk_report(vec![good, mk_emit(TuClass::VocabGap, 100, 50, 10, 20, 30)]);
        assert_eq!(rep.count(TuClass::Match), 1, "control: one byte-exact TU");
        assert_eq!(
            rep.emit_match_tu_residue(),
            0,
            "control: a byte-exact TU with every symbol bound and in class reads 0"
        );

        let bad = mk_emit(TuClass::Match, 40, 37, 37, 0, 3);
        let rep = mk_report(vec![bad, mk_emit(TuClass::VocabGap, 100, 50, 10, 20, 30)]);
        assert_eq!(
            rep.count(TuClass::Match),
            1,
            "the mutation must not change the number of byte-exact TUs — otherwise \
             this control tests the class filter, not the binding"
        );
        assert_eq!(
            rep.emit_match_tu_residue(),
            3,
            "three emitted symbols the port provably emitted correctly did not bind \
             to an in-class row: the binding is wrong there, and it must say so"
        );
    }

    /// The near-match table is the payoff metric's leading indicator, and a
    /// `capture-fail` TU must not appear in it: it has no census, so its distance
    /// of 0 means "never measured", not "nearly done".
    #[test]
    fn the_near_match_table_excludes_the_tus_that_were_never_measured() {
        let mut near = mk_emit(TuClass::VocabGap, 10, 10, 9, 0, 0);
        near.src = "near.cpp".into();
        near.fn_total = 10;
        near.fn_in_class = 9;
        let mut far = mk_emit(TuClass::VocabGap, 500, 400, 10, 0, 0);
        far.src = "far.cpp".into();
        far.fn_total = 500;
        far.fn_in_class = 10;
        let mut unmeasured = mk("c1083");
        unmeasured.class = TuClass::CaptureFail;
        unmeasured.src = "never-captured.cpp".into();
        let rep = mk_report(vec![near, far, unmeasured]);
        let got: Vec<&str> = rep.near_match_tus(100).iter().map(|r| r.src.as_str()).collect();
        assert_eq!(
            got,
            vec!["near.cpp"],
            "only the measured TU within 100 blocked functions may appear"
        );
    }

    /// The two distances measure different populations and must be allowed to
    /// disagree — the whole reason for publishing both. Modelled on the real
    /// `src/system/math/Rand2.cpp`: 13 `.ex` bodies, 5 in class (8 blocked
    /// bodies), but only 2 emitted functions of which 1 is in class, so **2**
    /// by the measure the goal is written in. A leading indicator that ranked
    /// this TU at 8 while another at 8-blocked-bodies-and-8-blocked-emitted also
    /// read 8 is ranking two very different amounts of work the same.
    #[test]
    fn the_two_distances_are_different_populations_and_may_disagree() {
        let mut rand2 = mk_emit(TuClass::VocabGap, 2, 2, 1, 0, 0);
        rand2.src = "Rand2.cpp".into();
        rand2.fn_total = 13;
        rand2.fn_in_class = 5;
        let mut even = mk_emit(TuClass::VocabGap, 9, 9, 1, 0, 0);
        even.src = "even.cpp".into();
        even.fn_total = 9;
        even.fn_in_class = 1;
        let rep = mk_report(vec![rand2, even]);

        let by_body: Vec<&str> = rep.near_match_tus(8).iter().map(|r| r.src.as_str()).collect();
        assert_eq!(
            by_body,
            vec!["Rand2.cpp", "even.cpp"],
            "by blocked BODIES both TUs are 8 away and the measure cannot tell them apart"
        );
        let by_emit: Vec<&str> = rep
            .near_match_tus_emitted(2)
            .iter()
            .map(|r| r.src.as_str())
            .collect();
        assert_eq!(
            by_emit,
            vec!["Rand2.cpp"],
            "by blocked EMITTED functions Rand2 is 2 away and the other is 8 — if this \
             ever equals the body measure, one of the two is not reading what it says"
        );
    }

    /// The emit-set ceiling, and the control that makes it a measurement.
    ///
    /// `PortC2` emits one `.text` COMDAT per `.ex` function segment and has no
    /// emit-set model, so a TU whose segment count differs from its obj's
    /// COMDAT-leader count cannot be byte-exact however good its codegen is.
    /// The invariant that keeps that reading honest is that **no matching TU may
    /// violate it** — a byte-exact obj cannot carry a different number of
    /// `.text` COMDATs than the port wrote. The mutation below is exactly that
    /// violation and it must be counted, otherwise the ceiling is an argument
    /// rather than a control.
    #[test]
    fn the_emit_set_ceiling_is_bounded_by_an_invariant_that_can_go_red() {
        // A matching TU: 2 bodies, 2 emitted COMDATs, both in class.
        let mut ok = mk_emit(TuClass::Match, 2, 2, 2, 0, 0);
        ok.src = "Spew.cpp".into();
        ok.fn_total = 2;
        ok.fn_in_class = 2;
        // Reachable but not there yet: counts agree, one body still blocked.
        let mut near = mk_emit(TuClass::VocabGap, 1, 1, 0, 0, 0);
        near.src = "xboxheap.cpp".into();
        near.fn_total = 1;
        near.fn_in_class = 0;
        // UNREACHABLE: 802 `.ex` bodies against 2 emitted COMDATs. Every emitted
        // function is already in class, so BOTH distance measures call it near;
        // the port would still write 802 sections against c2's 2.
        let mut vec_cpp = mk_emit(TuClass::VocabGap, 2, 2, 2, 0, 0);
        vec_cpp.src = "vec.cpp".into();
        vec_cpp.fn_total = 802;
        vec_cpp.fn_in_class = 237;
        let rep = mk_report(vec![ok, near, vec_cpp]);

        let reach: Vec<&str> = rep
            .emit_set_reachable_tus()
            .iter()
            .map(|r| r.src.as_str())
            .collect();
        assert_eq!(
            reach,
            vec!["Spew.cpp", "xboxheap.cpp"],
            "vec.cpp has zero blocked EMITTED functions and is still unreachable — \
             that is the point of the ceiling"
        );
        assert_eq!(
            rep.emit_set_violations(),
            0,
            "a matching TU whose counts disagree would mean fn_total and emit-emitted \
             are not counting what the ceiling says they count"
        );

        // The control: make a MATCHING TU violate it. If this does not go red the
        // invariant cannot see the defect it exists for (#145).
        let mut bad = mk_emit(TuClass::Match, 2, 2, 2, 0, 0);
        bad.src = "Spew.cpp".into();
        bad.fn_total = 5;
        bad.fn_in_class = 2;
        let rep = mk_report(vec![bad]);
        assert_eq!(
            rep.count(TuClass::Match),
            1,
            "the mutation must not change the number of byte-exact TUs — otherwise this \
             control tests the class filter, not the emit-set reading"
        );
        assert_eq!(
            rep.emit_set_violations(),
            1,
            "a byte-exact obj with 5 `.ex` segments and 2 `.text` COMDATs is impossible; \
             the invariant must say so"
        );
    }

    /// A TU with the four Phase 7 factors set explicitly, through the same keys
    /// `scan_one` writes.
    fn mk_factors(class: TuClass, src: &str, a: bool, b: bool, c: bool, d: bool) -> TuResult {
        let mut r = mk("x");
        r.class = class;
        r.src = src.into();
        // `emit-gate-segments-known` and `emit-emitted` are the populations the
        // factors are defined over; a TU missing them is UNMEASURED, not false.
        r.emit.insert("emit-gate-segments-known".into(), 1);
        r.emit.insert("emit-emitted".into(), 0);
        r.emit.insert("emit-sec-readable".into(), 1);
        for (k, on) in [
            ("emit-set-ceiling-gate", a),
            ("emit-set-ceiling-today", b),
            ("emit-sec-reachable", c),
            ("emit-class-complete", d),
        ] {
            if on {
                r.emit.insert(k.into(), 1);
            }
        }
        r
    }

    /// **The factorization is a JOINT, and the joint is not the product of its
    /// marginals** (`ROADMAP.md` §8.6 — the standing rule this report had no tool
    /// for until the per-row dump, and now has one for at TU level).
    ///
    /// The four TUs below give marginals A = B = C = D = 3 of 4, which multiplied
    /// against 4 TUs would "predict" ≈1.3 — and the measured joint is **0**,
    /// because each TU fails a different factor. A report that printed only the
    /// four counts would let a reader do that multiplication and be wrong in the
    /// flattering direction.
    #[test]
    fn the_factorization_is_a_joint_and_not_a_product_of_marginals() {
        let rep = mk_report(vec![
            mk_factors(TuClass::VocabGap, "a.cpp", false, true, true, true),
            mk_factors(TuClass::VocabGap, "b.cpp", true, false, true, true),
            mk_factors(TuClass::VocabGap, "c.cpp", true, true, false, true),
            mk_factors(TuClass::VocabGap, "d.cpp", true, true, true, false),
        ]);
        let [a, b, c, d, _a_lo, bc, abcd] = rep.factor_counts();
        assert_eq!([a, b, c, d], [3, 3, 3, 3], "each marginal is 3 of 4");
        assert_eq!(bc, 2, "B and C jointly is measured per TU, not B*C/n");
        assert_eq!(
            abcd, 0,
            "no TU satisfies all four — the joint can be 0 while every marginal \
             is 3/4, which is the whole reason this is measured and not multiplied"
        );
        assert!(rep.factor_all_tus().is_empty());
    }

    /// **The known-answer control**, and it must be able to go red. Each factor
    /// is a *necessary* condition for a byte-exact obj, so a `match` TU outside
    /// one means the factor is not necessary and every bound drawn from it is
    /// void. For **C** this is also the only executable check on
    /// [`PORT_WRITER_SECTIONS`]: a matching obj is the port's own output, so a
    /// name missing from that list surfaces here.
    ///
    /// The guard's quantity — one `match` TU — is held fixed across both halves,
    /// so the second half cannot pass by the TU ceasing to be a `match`.
    #[test]
    fn a_matching_tu_outside_any_factor_is_a_red_control() {
        let ok = mk_factors(TuClass::Match, "Spew.cpp", true, true, true, true);
        let rep = mk_report(vec![ok, mk_factors(TuClass::VocabGap, "z.cpp", false, false, false, false)]);
        assert_eq!(rep.factor_control_on_match_tus(), ([0, 0, 0, 0], 1));
        assert_eq!(rep.factor_all_tus(), vec!["Spew.cpp"]);

        // The mutation: the same matching TU, now carrying a section the writer
        // cannot emit. That is impossible — the port wrote that obj — so it must
        // be counted, and against factor C specifically.
        let bad = mk_factors(TuClass::Match, "Spew.cpp", true, true, false, true);
        let rep = mk_report(vec![bad]);
        assert_eq!(
            rep.count(TuClass::Match),
            1,
            "the mutation must not change the number of byte-exact TUs — otherwise \
             this control tests the class filter, not the factor"
        );
        assert_eq!(
            rep.factor_control_on_match_tus(),
            ([0, 0, 1, 0], 1),
            "a byte-exact obj outside the port writer's section vocabulary is \
             impossible; C must say so, and name itself"
        );
    }

    /// **The frontier is `A∧B∧C ∧ ¬D ∧ ¬match`**, and each of those four clauses
    /// is load-bearing: a byte-exact TU in the list would be work already done,
    /// and a TU missing A, B or C is not one widening away from anything.
    #[test]
    fn the_frontier_is_the_tus_whose_only_remaining_factor_is_codegen() {
        let mut near = mk_factors(TuClass::VocabGap, "near.cpp", true, true, true, false);
        near.emit.insert("emit-emitted".into(), 5);
        near.emit.insert("emit-in-class".into(), 4);
        let mut far = mk_factors(TuClass::VocabGap, "far.cpp", true, true, true, false);
        far.emit.insert("emit-emitted".into(), 11);
        far.emit.insert("emit-in-class".into(), 8);
        let rep = mk_report(vec![
            far,
            near,
            // Already done — must not appear.
            mk_factors(TuClass::Match, "done.cpp", true, true, true, true),
            // Blocked on a factor codegen cannot move — must not appear.
            mk_factors(TuClass::VocabGap, "sections.cpp", true, true, false, false),
            mk_factors(TuClass::VocabGap, "emitset.cpp", false, true, true, false),
        ]);
        let got: Vec<(&str, usize)> = rep
            .factor_frontier()
            .into_iter()
            .map(|(r, n)| (r.src.as_str(), n))
            .collect();
        assert_eq!(
            got,
            vec![("near.cpp", 1), ("far.cpp", 3)],
            "nearest first by blocked EMITTED functions, and only the TUs where D is \
             the whole remaining distance"
        );
    }

    /// An obj whose section headers did not decode is **outside** C, never
    /// inside it. An empty section list would read as "carries nothing beyond
    /// the writer's set", which is the flattering direction and the shape
    /// §9.18.8 records twelve times.
    #[test]
    fn an_unreadable_obj_is_outside_factor_c_rather_than_vacuously_inside_it() {
        let mut r = mk("x");
        r.class = TuClass::VocabGap;
        r.src = "broken.cpp".into();
        r.emit.insert("emit-sec-unreadable".into(), 1);
        let rep = mk_report(vec![r]);
        assert_eq!(rep.factor_counts()[2], 0, "no section list means no C");
        assert!(rep.factor_all_tus().is_empty());
    }

    /// **The greedy ladder must run through a zero-gain step.** Two names that
    /// only ever co-occur each score 0 alone, so a ladder that stopped on
    /// no-progress would report the vocabulary as unclosable when it is one step
    /// from closed — which is exactly the workload's `.CRT$XCU`/`.text$yc` pair
    /// (126 objs each, never apart).
    #[test]
    fn the_greedy_ladder_runs_through_a_zero_gain_step() {
        let tu = |src: &str, extras: &[&str]| {
            let mut r = mk("x");
            r.class = TuClass::VocabGap;
            r.src = src.into();
            r.emit.insert("emit-sec-readable".into(), 1);
            for e in extras {
                r.emit.insert(format!("emit-sec-extra|{e}"), 1);
            }
            if extras.is_empty() {
                r.emit.insert("emit-sec-reachable".into(), 1);
            }
            r
        };
        let rep = mk_report(vec![
            tu("in.cpp", &[]),
            tu("one.cpp", &[".data"]),
            tu("two.cpp", &[".data"]),
            tu("pair1.cpp", &[".CRT$XCU", ".text$yc"]),
            tu("pair2.cpp", &[".CRT$XCU", ".text$yc"]),
        ]);
        assert_eq!(rep.factor_counts()[2], 1, "one TU is already reachable");
        assert_eq!(
            rep.section_ladder(),
            vec![
                (".data".to_string(), 3),
                (".CRT$XCU".to_string(), 3),
                (".text$yc".to_string(), 5),
            ],
            "greedy takes the +2 first, then must push through the zero-gain half \
             of the co-occurring pair to reach the whole workload"
        );
    }

    /// The vocabulary census counts **objs carrying a section**, not sections.
    /// Under `/Gy` one obj holds one COMDAT `.text` per emitted function, so the
    /// second reading would report 158 for `src/App.cpp` alone and no reader of
    /// the table could tell which number it was looking at.
    #[test]
    fn the_section_vocabulary_counts_objs_and_not_sections() {
        let tu = |src: &str, names: &[&str]| {
            let mut r = mk("x");
            r.class = TuClass::VocabGap;
            r.src = src.into();
            r.emit.insert("emit-sec-readable".into(), 1);
            // 158 `.text` sections in this obj — one row, because the key is
            // written once per DISTINCT name per TU.
            r.emit.insert("emit-sec-count".into(), 158);
            for n in names {
                r.emit.insert(format!("emit-sec-name|{n}"), 1);
            }
            r
        };
        let rep = mk_report(vec![tu("a.cpp", &[".text", ".data"]), tu("b.cpp", &[".text"])]);
        assert_eq!(
            rep.section_vocabulary(),
            vec![(".text".to_string(), 2), (".data".to_string(), 1)],
            "two objs carry `.text` and one carries `.data`, ranked most common first"
        );
    }

    #[test]
    fn fn_census_aggregates_across_tus() {
        // Two TUs: 10 functions each, 3 + 4 in class, blockers summed by key.
        // The point of P2b: coverage is measurable (7/20) even though NO whole
        // TU is in class, so both TUs classify as `codegen-gap` above.
        let mut a = mk("x");
        a.fn_total = 10;
        a.fn_in_class = 3;
        a.fn_blockers.insert("expr-cmp-gt".into(), 5);
        a.fn_blockers.insert("expr-shift".into(), 2);
        let mut b = mk("x");
        b.fn_total = 10;
        b.fn_in_class = 4;
        b.fn_blockers.insert("expr-cmp-gt".into(), 6);
        let rep = mk_report(vec![a, b]);
        assert_eq!(rep.fn_coverage(), (7, 20));
        assert_eq!(
            rep.fn_blocker_histogram(),
            vec![("expr-cmp-gt".to_string(), 11), ("expr-shift".to_string(), 2)]
        );
    }

    /// **The dispatch axes aggregate, and the residue is a NUMBER.**
    ///
    /// The rows below are the ones a ranking reads: two dispatch arms, one of
    /// which (`disp-expr`) can never reach a member-call production, and the
    /// tag-coverage residue on the production axis. Each is asserted as a positive
    /// count with its own message, because the way this report fails is by
    /// printing a short table that looks complete.
    #[test]
    fn dispatch_axes_aggregate_across_tus() {
        let mut a = mk("x");
        a.fn_total = 10;
        a.fn_in_class = 1;
        // Six bodies took the expression arm; four of those are blocked on a
        // member-call construct they can never reach a member-call production
        // with, which is the whole point of the axis.
        a.fn_dispatch.insert("disp-expr".into(), 6);
        a.fn_dispatch.insert("disp-expr|BLOCKED".into(), 6);
        a.fn_dispatch
            .insert("disp-expr|BLOCKED|expr-call-in-expr-recv-field-whole".into(), 4);
        a.fn_dispatch.insert("disp-assign".into(), 4);
        a.fn_dispatch.insert("disp-assign|BLOCKED".into(), 3);
        a.fn_prod.insert("prod-not-entered".into(), 6);
        a.fn_prod.insert("prod-entered-untagged".into(), 3);
        a.fn_prod.insert("prod-accepted".into(), 1);
        let mut b = mk("x");
        b.fn_total = 5;
        b.fn_dispatch.insert("disp-expr".into(), 5);
        b.fn_dispatch.insert("disp-expr|BLOCKED".into(), 5);
        b.fn_prod.insert("prod-not-entered".into(), 3);
        b.fn_prod.insert("prod-entered-untagged".into(), 2);
        let rep = mk_report(vec![a, b]);

        let disp = rep.fn_dispatch_histogram();
        let get = |h: &[(String, usize)], k: &str| -> usize {
            h.iter().find(|(a, _)| a == k).map(|(_, n)| *n).unwrap_or(0)
        };
        assert_eq!(
            get(&disp, "disp-expr"),
            11,
            "the expression arm must sum across TUs — this is the arm that CANNOT \
             reach a member-call production, so its size is the part of a \
             member-call row no widening there can serve"
        );
        assert_eq!(
            get(&disp, "disp-expr|BLOCKED|expr-call-in-expr-recv-field-whole"),
            4,
            "the arm x census-key cross must survive aggregation: it is the only \
             row that says a member-call CONSTRUCT arrived in an arm the member-call \
             productions never see"
        );
        let prod = rep.fn_prod_histogram();
        assert_eq!(
            get(&prod, "prod-not-entered"),
            9,
            "`prod-not-entered` is a measured population and must aggregate like \
             any other row, not be suppressed as a default"
        );
        assert_eq!(
            rep.prod_untagged_residue(),
            5,
            "the tag-coverage residue must be reported as a NUMBER — it is what the \
             tag sites in mcall_*.rs have left to explain, and inferring it from \
             missing rows is the mistake this axis exists to stop"
        );
        // Both axes must sum to the same population the census counted. A short
        // count means bodies went untagged and every row above is a lower bound.
        assert_eq!(
            rep.dispatch_axis_totals(),
            (15, 15),
            "both axes must account for all 15 functions: a body takes exactly one \
             arm and reaches exactly one production state, so a short total is an \
             under-reporting instrument rather than a small population"
        );
    }

    /// **A scan in which nothing reached a tagged site still reports numbers.**
    ///
    /// This is the state of the board before the 37 tag sites in
    /// `body::shapes::mcall_{tail,chain,cmp}` are placed: every body that entered
    /// a production lands in `prod-entered-untagged`. The residue must read as
    /// that population's exact size — not as 0, and not as an empty histogram,
    /// either of which would be indistinguishable from "no bodies enter a
    /// production at all".
    #[test]
    fn an_entirely_untagged_scan_reports_its_residue_rather_than_nothing() {
        let mut a = mk("x");
        a.fn_total = 7;
        a.fn_prod.insert("prod-not-entered".into(), 3);
        a.fn_prod.insert("prod-entered-untagged".into(), 4);
        a.fn_prod.insert("prod-entered-untagged|BLOCKED".into(), 4);
        a.fn_dispatch.insert("disp-assign".into(), 4);
        a.fn_dispatch.insert("disp-expr".into(), 3);
        let rep = mk_report(vec![a]);
        assert_eq!(
            rep.prod_untagged_residue(),
            4,
            "with no tag site placed, the residue IS the whole entered population \
             and must be printed as such"
        );
        assert!(
            rep.fn_prod_histogram()
                .iter()
                .any(|(k, n)| k == "prod-entered-untagged" && *n == 4),
            "the residue must appear as a ranked row too, so a reader of the table \
             sees it beside the named sites rather than having to know it is missing"
        );
        assert_eq!(
            rep.dispatch_axis_totals(),
            (7, 7),
            "and the axes still account for every function"
        );
    }
}

#[cfg(test)]
mod splitter_predicate_guard {
    use c2_il::IlBundle;

    /// **The two `.ex` readers must disagree on a bundle the gate refuses**, or
    /// `gap.rs` step 3 can be fed by step 1g's variable without anything going
    /// red.
    ///
    /// That substitution actually happened, and the run it produced was green in
    /// every field a reviewer scans: `mismatch 0`, `match 6`, `0 failed`. What it
    /// did was move **865 TUs from `vocab-gap` to `codegen-gap`** — a report
    /// claiming the port decodes the whole workload. The failure direction is
    /// flattering, which is exactly the shape ROADMAP §9.18.8 records twelve
    /// times.
    ///
    /// So this pins the *difference*, not either reader: a bundle whose `.ex`
    /// carries function-start markers the gate cannot accept must give
    /// `ex_segment_count() = Some(n > 0)` and `functions() = None`. A test that
    /// only asserted each reader's own value would pass with the two wired
    /// together.
    ///
    /// **What it does NOT do**, said here so it is not mistaken for a guard:
    /// re-folding `gap.rs` step 3 onto step 1g's variable still passes this
    /// test. The re-fold is only visible in the class counts, and `classify_one`
    /// needs a toolchain. This makes the invariant executable; the 3-TU
    /// both-ways scan in the rung doc is the evidence for the consequence.
    #[test]
    fn the_class_predicate_is_not_the_segment_counter() {
        // `.ex` with two `4F 1F` starts and nothing the body parser can read;
        // no `.gl` records, so the binding refuses. The point is only that the
        // gate says NO while segments exist.
        let mut ex = vec![0x11u8; 8];
        ex.extend_from_slice(&[0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00]);
        ex.extend_from_slice(&[0x22; 16]);
        ex.extend_from_slice(&[0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00]);
        ex.extend_from_slice(&[0x33; 16]);

        let mut b = IlBundle::default();
        b.base_name = "_CL_guard".to_string();
        b.files.insert("ex".to_string(), ex);
        b.files.insert("gl".to_string(), vec![0u8; 32]);

        let segs = b.ex_segment_count();
        assert_eq!(
            segs,
            Some(2),
            "the pure reader must count both `4F 1F` starts, whatever the gate thinks"
        );
        assert!(
            b.functions().is_none(),
            "control: the gate must REFUSE this bundle, or the pair below proves nothing"
        );
        assert_ne!(
            segs.is_none(),
            b.functions().is_none(),
            "the segment counter and the acceptance decision must not agree here — \
             if they do, gap.rs step 3 can be fed by step 1g's variable and \
             `vocab-gap` silently becomes `codegen-gap`"
        );
    }
}

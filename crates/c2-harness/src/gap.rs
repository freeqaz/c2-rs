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
    if captured.bundle.functions().is_none() {
        res.class = TuClass::VocabGap;
        res.reason = "il function decode failed".to_string();
        res.detail = format!(
            ".ex {} B, {} .gl names — c2_il::functions() = None",
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

    Ok(GapReport {
        results,
        provenance: Some(provenance),
        cache: cache_stats,
    })
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

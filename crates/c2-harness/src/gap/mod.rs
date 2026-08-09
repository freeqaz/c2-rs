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
use std::path::PathBuf;

use crate::provenance::Provenance;

mod classify;
mod factors;
pub mod fnbytes;
pub mod fndiff;
mod render;
mod report;
mod scan;
pub mod sets;
mod witness;

#[cfg(test)]
mod splitter_predicate_guard;
#[cfg(test)]
mod tests;

pub use classify::{cflow_needs_block_ir, dtor_callee_class};
pub use factors::{CfgBounds, CfgClass, CfgLedgerRow, CfgReach, CfgSub};
pub use scan::gap_scan;
pub use witness::{witness_buckets, WitnessBucket, WitnessRow};

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
    /// **The diff-signature sink** (lane `w-bytes`, board #976): one JSON record
    /// per `fnbyte-differs` **function**, not per TU — see [`fndiff`].
    ///
    /// Off by default and a file rather than stdout, for the same reason
    /// [`GapConfig::factors_tsv`] is: the population is thousands of rows on the
    /// dc3 workload and would swamp a report meant to be read. The **counts**
    /// derived from the same signatures are printed on every scan regardless
    /// (`fndiff-*` keys), so the cluster census is never conditional on somebody
    /// having passed a flag.
    pub fndiff_jsonl: Option<PathBuf>,
    /// Write the **per-TU Phase 7 factor membership** here, one row per graded
    /// TU (`src`, class, A/B/C/D/E). See [`GapReport::factor_membership`] for
    /// why the joints alone are not enough: a count cannot be intersected with
    /// another lane's per-TU set, and the last lane that needed to do so had to
    /// decline the measurement rather than multiply a rate by a count.
    ///
    /// Off by default and a file rather than stdout: `c2rs gap` also grades the
    /// generated case corpus (`scripts/mode_lane.sh`, `scripts/mode_cross.sh`),
    /// where one line per TU is tens of thousands of lines per lane.
    pub factors_tsv: Option<PathBuf>,
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
    /// **WHICH TOKEN took each off-class body out of `CfResidue::Modeled`**,
    /// crossed with the population: `"<reason>|IN-CLASS"` / `"<reason>|BLOCKED"`.
    /// Board **#1345**.
    ///
    /// **Its own map and not a row of [`TuResult::fn_cflow`], and that is not
    /// tidiness.** `GapReport::cflow_residue_control` counts every `fn_cflow`
    /// row ending in `|IN-CLASS` that does not end in `+expr-modeled|IN-CLASS`
    /// as off-class. A `"div-mod|IN-CLASS"` row added to that map would have
    /// been silently folded into the 518,991 and the published number would
    /// have roughly doubled with nothing in git to show for it — the exact
    /// shared-predicate collision that produces no merge conflict. Two maps
    /// cannot collide.
    ///
    /// The `|IN-CLASS` half sums to `cflow_residue_control().1`; that identity
    /// is published as `gap-metric cflow-offclass-accounted` rather than
    /// asserted, because a totality control counted in two different units is
    /// trap 0's own instance (`w-tag02`, `records` vs `values`).
    pub fn_cflow_off: BTreeMap<String, usize>,
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
    /// **The GATE's own first refusal, by name** — `IlBundle::decode_causes().first`
    /// (lane `w-vec`, board **#2500**).
    ///
    /// `None` when the bundle decodes. Otherwise the cause
    /// [`c2_il::IlBundle::functions`] actually stops on: `gl-stop-26-introduced`,
    /// `bind-record-count-ne-segments`, `body-out-of-class`,
    /// `unclaimed-gl-symbol`, … — the closed string set in `c2_il::func::diag`.
    ///
    /// # Why this field exists, and it is CEILING §11.4 item 8 made checkable
    ///
    /// Before this, every `vocab-gap` TU rendered one string —
    /// *"il function decode failed"*, 851 of 878 of them — and the `detail`
    /// beside it named only two sizes (`.ex` bytes, `.gl` names) and the fact
    /// that both acceptance paths said `None`. **Neither says which of eleven
    /// gates fired.** `IlBundle::decode_causes` had answered exactly that
    /// question since lane `w-vocab` and **no caller in `c2-harness` ever
    /// called it**, so a lane that wanted the gate's own refusal had to write a
    /// scratch patch — which is how `src/system/math/vec.cpp` came to be
    /// commissioned as *"`_fltused` plus seven non-instruction sections"* when
    /// the gate's first stop on it is `gl-stop-26-introduced`, four mechanisms
    /// upstream of either.
    ///
    /// This is a **diagnostic and never a gate**: it is read from the same
    /// predicate `decodes()` already decided the class with, after the class is
    /// decided, and no verdict anywhere depends on it. `c2_il`'s own
    /// `causes.is_empty() == decodes` invariant is what keeps it from drifting.
    pub gate_cause: Option<String>,
    /// Every cause that fires on this TU, not just the first — the same list,
    /// ascending and deduplicated. Empty iff the bundle decodes.
    ///
    /// The first cause is what `functions()` **stops** on and is therefore the
    /// only one a repair is guaranteed to move; the rest are what a lane would
    /// still owe after repairing it, which is the number a conversion price
    /// needs and the first cause alone cannot give.
    pub gate_causes: Vec<String>,
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
    /// **One rendered JSON row per `fnbyte-differs` function** ([`fndiff`]), in
    /// the reference obj's COMDAT order.
    ///
    /// Rendered here rather than at the sink so the scan's worker threads do the
    /// work and the writer only concatenates — and so the rows exist whether or
    /// not `--fnbyte-diff-jsonl` was passed, which is what lets the `fndiff-*`
    /// counters be unconditional.
    pub fndiff: Vec<String>,
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
    /// empty unless `C2RS_WITNESS` is set. See [`witness_path`](crate::gap::witness::witness_path) for why the
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

/// **The registry of WHOLE-TU RECOGNIZERS** — the fifth term's whole population,
/// by name (board #179, `docs/ROADMAP.md` §10.21/§10.22).
///
/// A *whole-TU recognizer* is an acceptance predicate over an entire
/// [`c2_il::IlBundle`] that no per-function predicate can express, and which
/// `PortC2::build` consults as its own arm before the per-function path. There
/// is exactly one today: `IlBundle::dyninit_tu`, the `??__E` dynamic-initializer
/// shape lane w-r1c landed.
///
/// # Why a closed list and not `IlBundle::decodes()`
///
/// This is the whole degradation argument for **factor E**, so it is written
/// here rather than in a rung nobody re-reads.
///
/// The one-line alternative is `E := bundle.decodes() && functions().is_none()`.
/// It needs no registry and it is **wrong on purpose-built grounds**:
/// `decodes()` is defined as `functions().is_some() || dyninit_tu().is_some()`
/// and its own doc comment says *"adding a third path means adding it here"*. So
/// a third acceptance path would enter `decodes()` in `c2-il` and E would
/// **silently absorb it** — the factorization would stay green through an emit
/// path it does not model, which is exactly the false-green this term exists to
/// prevent. That is the open-world definition.
///
/// This is the closed-world one. A new arm in `PortC2::build` does **not** enter
/// this table; adding it is a separate, deliberate edit in this file. Until that
/// edit happens, a TU converted by the new path is a `match` with **D false and
/// E false**, so [`GapReport::factor_control_on_match_tus`]'s `D∨E` column goes
/// red and names it — which is precisely the event of 2026-08-04, when
/// `dyninit_tu` landed and the `D` column went to 2.
///
/// **There is no static guard that this list is complete, and none is claimed.**
/// `gap.rs` cannot enumerate `c2-core`'s match arms, and a test asserting
/// `decodes() == functions().is_some() || <this table>` would pass vacuously on
/// every bundle that exercises no new path. The guard is **empirical and is the
/// scan's own known-answer control**; `the_control_goes_red_for_an_unregistered_whole_tu_path`
/// is the executable demonstration that it can fire.
///
/// Each entry carries its own key (`emit-whole-tu|<name>`) so the marginal of
/// each recognizer is separately visible: a registry that grew an entry which
/// never fires would otherwise be indistinguishable from one that did not.
pub const WHOLE_TU_RECOGNIZERS: &[(&str, fn(&c2_il::IlBundle) -> bool)] = &[
    ("dyninit-??__E", |b| b.dyninit_tu().is_some()),
    // **W-SECT, board #174 — the functionless data TU.** Registered in the same
    // commit that gave `PortC2::build` the arm, which is what this table's own
    // doc asks for: an unregistered arm turns the `D∨E` control red the moment
    // it converts anything, and that red is the design working.
    //
    // It fires on **0 of the 871 graded TUs** and is registered anyway, which is
    // exactly why each entry carries its own marginal: a registry entry that
    // never fires and one that was never added are the same number in `|E|` and
    // very different facts. The workload census carries no TU whose section set
    // is the shell plus data — measured, not assumed — so this entry is a
    // statement about the *model*, not about today's corpus.
    //
    // **It over-approximates on purpose.** `data_tu` is the DECODE bound; the
    // LAYOUT bound (at most two objects per non-COMDAT section,
    // `OBJ_DATA_BSS_SHAPE.md` §8.1) lives in `coff::emit_data_obj` and this
    // predicate cannot see it, so a TU with three `.bss` objects reads as E-true
    // and refuses. That is the same approximation `dyninit-??__E` makes — the
    // recognizer names the path, not the emitter's every gate — and it errs
    // toward counting a TU the port declines, never toward missing one it takes.
    ("data-only-tu", |b| b.data_tu().is_some()),
];

/// Aggregated scan report.
/// The computed PROGRESS MASS and every input it was computed from — see
/// [`GapReport::progress_mass`]. Inputs travel with the value so a consumer can
/// never quote `value` without its denominators, which is the positive-claim
/// rule (STATUS trap 5: compare a count, never a status).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressMass {
    /// TUs the scan graded (everything that captured, mismatches included).
    pub graded: usize,
    /// Factor A TUs among the non-mismatch graded (emit set reachable).
    pub a: usize,
    /// Factor B TUs among the non-mismatch graded (every emitted symbol binds).
    pub b: usize,
    /// Factor C TUs among the non-mismatch graded (sections within the writer).
    pub c: usize,
    /// Emitted functions the census calls in class, non-mismatch TUs only.
    pub emitted_in_class: usize,
    /// Emitted functions across ALL graded TUs — the `f` denominator is never
    /// reduced by a mismatch, so zeroing a TU's numerator always costs.
    pub emitted_total: usize,
    /// TUs graded `mismatch`, whose contributions were zeroed. Printed, so a
    /// scan with wrong emits cannot present a quietly-reduced P as clean.
    pub mismatch_zeroed: usize,
    /// `mean(a, b, c, f)` — in `[0, 1]`.
    pub value: f64,
}

/// **FUNCTION BYTE MATCH** and the full partition it was computed from — see
/// [`fnbytes`] for the design and [`GapReport::fn_byte_match`] for the
/// aggregation.
///
/// Every field travels with `value` for the same reason [`ProgressMass`]'s do:
/// the ratio is not quotable without its denominator, and the buckets are the
/// only place the size of the instrument's own under-report is stated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FnByteMatch {
    /// Emitted `.text` COMDATs across the graded workload — the denominator,
    /// counted off the **reference** obj and therefore not a function of the
    /// port's output.
    pub denominator: usize,
    /// Emitted functions whose port body is byte-identical to c2's, graded
    /// through the port's **per-function** route (`codegen::select_function`).
    pub exact: usize,
    /// Emitted functions on a TU the differential graded `match` that the
    /// per-function route did not credit. The whole-obj byte compare has
    /// already certified them, and the judge's verdict supersedes the
    /// instrument's route — see [`GapReport::fn_byte_match`]. Credited.
    pub whole_tu: usize,
    /// Complete port body, bytes differ. Forensic, never credited.
    pub differs: usize,
    /// The port selected a shape the COFF emitter finishes. Not credited, and
    /// deliberately not reconstructed here — see [`fnbytes`].
    pub partial: usize,
    /// The port refuses this function.
    pub refused: usize,
    /// No census row binds this emitted symbol, or two do.
    pub unbound: usize,
    /// The COMDAT's raw data did not decode.
    pub nobytes: usize,
    /// TUs whose obj did not decode at all — no denominator taken from them.
    pub obj_unreadable: usize,
    /// Buckets that did not sum to the denominator. **Known answer 0.**
    pub partition_broken: usize,
    /// Instruction words in the `differs` class: `(port, reference, equal)`.
    pub differ_words: (usize, usize, usize),
    /// Emitted functions the census calls in class that the port refuses — the
    /// census/gate disagreement restricted to the emitted population, which is
    /// the error term on [`ProgressMass`]'s `f` numerator.
    ///
    /// **Not a target of 0 as a total** — see
    /// [`FnByteMatch::census_disagree_expressible`], which is. Lane
    /// `w-inlfence2`.
    pub census_disagree: usize,
    /// The half of [`FnByteMatch::census_disagree`] board #139's rule reaches:
    /// every stage the IL parser **could** have refused. **Target 0.**
    ///
    /// The complement is the post-lowering stages — `gy-shape`, `data-ref` and
    /// `inlined-callee` — which are not a function of the IL body alone, so no
    /// parser clause can express them. Both of the first two read 0 on the dc3
    /// workload, which is why the total could be read as the alarm until
    /// `inlined-callee` existed. Splitting it keeps the alarm sharp instead of
    /// re-defining "nonzero is fine now".
    pub census_disagree_expressible: usize,
    /// Of the `exact` bucket, how many carry at least one relocation in c2's
    /// obj.
    ///
    /// **Retired into a graded number by lane `w-relo`.** It used to be the size
    /// of a blind spot — credited on bytes whose relocation targets FBM never
    /// checked (`FUNCTION_BYTE_MATCH.md` §7.6, board #884). Since RELOC-EQ every
    /// one of these has had its records compared and passed, so it is now the
    /// *denominator of a verdict*: how much of the credit rests on relocations
    /// that were actually graded.
    pub exact_relocated: usize,
    /// **Bytes identical, relocations differ** — the class board #884 named.
    /// Its own bucket, never merged into `differs`: two bodies branching to two
    /// different functions are byte-identical and are not the same function.
    /// Never credited.
    pub reloc_differs: usize,
    /// Bytes identical, and the reference obj's relocation table did not decode,
    /// so RELOC-EQ could not be asked. **The counted residue of the population
    /// the relocation compare can reach** (`docs/STATUS.md` trap 0). Never
    /// credited.
    pub reloc_unknown: usize,
    /// Byte-exact functions that got a RELOC-EQ verdict, either way.
    /// `reloc_graded + reloc_unknown == exact_bytes` is checked per TU.
    pub reloc_graded: usize,
    /// The **old** `fnbyte-exact`: bytes identical, relocations unexamined.
    /// Published so the number this widening replaced stays derivable to the
    /// digit — `exact + reloc_differs + reloc_unknown`.
    pub exact_bytes: usize,
    /// The reach identity above failing on some TU. **Known answer 0.**
    pub reloc_partition_broken: usize,
    /// Emitted functions on a `match` TU for which the per-function route
    /// produced a body that DIFFERS from c2's. **Known answer 0** — see
    /// [`GapReport::fn_byte_match_tu_differs`].
    pub match_tu_differs: usize,
    /// Emitted functions on a `match` TU whose port body is byte-exact and whose
    /// RELOCATIONS differ. **Known answer 0, and a five-alarm if not**: a
    /// byte-exact obj means every relocation record in it is c2's own, so a
    /// positive count is a live disagreement between `select_function` plus
    /// `comdat::text_reloc_plan` and the COFF writer on a body the oracle has
    /// already certified.
    pub match_tu_reloc_differs: usize,
    /// `(exact + whole_tu) / denominator` — in `[0, 1]`.
    pub value: f64,
}

pub struct GapReport {
    pub results: Vec<TuResult>,
    /// What produced these numbers (roadmap #46/#48): both trees' git HEADs, the
    /// resolved toolchain paths, the wibo version. `None` only when a report is
    /// built by hand in a test.
    pub provenance: Option<Provenance>,
    /// Capture-cache counters for this scan (all zero when `--no-cache`).
    pub cache: crate::capture_cache::CacheStats,
}

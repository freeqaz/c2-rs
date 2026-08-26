//! `c2-reference::stage` — **the stage oracle**: observe real `c2.dll`'s
//! per-function phase boundaries while it compiles, without changing what it
//! compiles to.
//!
//! # Why this exists
//!
//! The sole judge of the port is a byte compare of the finished obj. It says
//! *"differs"* and nothing about **where**, so every divergence costs a
//! whole-object byte archaeology session — which is what makes lanes here
//! expensive (a single alignment nibble cost a lane; `dag.c`'s lowering order
//! cost two). The prize this module is chasing is **divergence localization**:
//! a divergence attributable to a *pass* rather than to an obj.
//!
//! # What already existed, so nothing here is re-shipped
//!
//! The `/FAsc` + `/QXSTALLS` narration seam ([`crate::Toolchain::capture_listing_with`],
//! [`crate::cod`], `c2rs listing-scan`, boards #132/#134/#136) already makes c2
//! narrate its own output: label counter, section order, EH layout,
//! relocations by name, per-instruction issue cycles. That is an **end-state**
//! observation — after all four scheduler runs, after COLOR, after lowering —
//! and it structurally cannot separate COLOR's output from the scheduler's.
//! `/QXSTALLS`'s cycles are worse than merely late: they come from **K4**
//! (`0x10c1ce93`), which builds its *own* whole-function DAG read-only and
//! tears it down (`docs/whitebox/WB_DAGCLIENTS_FINDINGS.md` §4.4), so they are
//! a re-derivation and not the schedule the scheduler produced.
//!
//! This module observes c2 **between** passes instead, by detouring its own
//! per-function phase call sites. The machinery is in `c2host/stagetap.c`; this
//! file is the Rust seam that arms it, runs a replay through it, and parses
//! what it reported.
//!
//! # The standing bound on everything here
//!
//! **A snapshot is a development instrument.** It never gates an emit, never
//! appears in a refusal predicate, and no rule enters `crates/` on snapshot
//! equality alone. The obj byte compare against real `c2.dll` remains the sole
//! judge — which is exactly why [`TapReport`] is delivered *beside* an
//! [`ObjImage`] that the caller is expected to compare against the untapped
//! one.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use c2_core::ObjImage;

use crate::{CapturedReference, Toolchain};

/// Every site `c2host/stagetap.c` knows how to arm, in table order.
///
/// **This list is duplicated on purpose and the duplication is tested.**
/// `crates/c2-reference/tests/stage.rs::the_two_site_tables_are_one_table`
/// parses `c2host/stagetap.c`'s own table and asserts equality, the pattern
/// `docs/ARCHITECTURE_SEAMS.md` §0 used for the `..base` and `bind.rs` moves:
/// two readers of one definition, with a test standing where a shared header
/// cannot (the C side is not in the Rust workspace and never will be).
/// `after0` is the **eighth** site and the only one that observes state after
/// the final schedule. `0x10b7df57` is run 4 (mode 0) and its site
/// `0x10b7e00c` lives inside it; `0x10b7e701` is the first call in the
/// per-function orchestrator `0x10b7e6af` after `0x10b7df57` returns, with
/// `ecx` still holding the function record. Without it the run that fixes
/// emitted instruction order has its output observed nowhere — every `sched0`
/// region block is run 4's INPUT, because the region tap fires at region-finder
/// entry and run 4 has no successor run
/// (`docs/ARCH_REVIEW_2026-08-21.md` finding 1).
/// PROV[R] DISCLOSURE `W-STAGETAP-1` and `W-STAGETAP-3` — eight call-site addresses read out of `c2.dll` (`0x10b7dc9f`, `0x10b7dcb7`, `0x10b7dcde`, `0x10b7dcf6`, `0x10b7dd1d`, `0x10b7e00c`, `0x10be643e`, `0x10b7e701`). This array carries the NAMES only; the addresses live in `c2host/stagetap.c`'s `g_sites[]`, which the rows also name. Fail-closed at run time: `tap_arm` refuses a site whose byte is not still `0xE8` at the recorded target plus the measured slide.
pub const STAGE_SITES: &[&str] = &[
    "sched1", "globregs", "sched2", "color", "sched3", "sched0", "region", "after0",
];

/// The **four** scheduler sites that the optimizer-on flag `DAT_10c2e2fc`
/// gates. `globregs` and `color` are NOT in this list and are not
/// optimizer-gated: they are reached unconditionally at `0x10b7dcaa` and
/// `0x10b7dce9`.
///
/// # The gate is NECESSARY, not sufficient — corrected in the fix round
///
/// An earlier revision of this doc, and of the rung, said the four runs are
/// gated *"only"* by `DAT_10c2e2fc`. **The disassembly refutes it**, and the
/// lane's own deliverable includes whitebox record corrections, so it does not
/// get to introduce one:
///
/// * `0x10b7dc83`/`0x10b7dcc2`/`0x10b7dd01` are each
///   `cmp DWORD PTR ds:0x10c2e2fc,edi` with `edi == 0` — the optimizer gate,
///   and it is checked first, which is all the `/Od` → 0 direction needs.
/// * **But each is followed by a SECOND per-function gate**:
///   `0x10b7dc8b`/`0x10b7dcca`/`0x10b7dd09` are `test BYTE PTR [esi+0x1c],bl`
///   with `bl == 1` (`0x10b7dc53 xor ebx,ebx` / `0x10b7dc58 inc ebx`), i.e.
///   bit 0 of the function record's `+0x1c`.
/// * `sched0` at `0x10b7e00c` carries three more beyond the optimizer gate at
///   `0x10b7dfd9`: `test DWORD PTR [eax+0x20],0x1000` (`0x10b7dfe3`, taken
///   ⇒ skip), `test eax,0x400000` (`0x10b7dff2`) and `test al,0x8`
///   (`0x10b7dff9`) over the function record's `+0x94`.
///
/// **Consequence, and it is the one that matters:** `/Od` ⇒ 0 hits still holds
/// (the optimizer `je` fires first), so the G3 null control is as grounded as
/// it ever was. What is NOT structural is the converse — `hits_at(site) ==
/// procs` at `/O1` is an EMPIRICAL property of the fixtures measured here, not
/// a property of the code, because a function with `[esi+0x1c] & 1 == 0` would
/// be skipped by three of the four sites.
/// PROV[O] DISCLOSURE `W-STAGETAP-1`'s fix round supplied the `[R]` half (the optimizer flag `0x10c2e2fc`, the three `cmp` sites, and the SECOND per-function gate at `0x10b7dc8b`/`0x10b7dcca`/`0x10b7dd09`); WHICH four sites are gated is graded against the live tap. The doc above states the bound explicitly — `hits_at(site) == procs` at `/O1` is empirical over these fixtures, not a property of the code.
pub const OPT_GATED_SITES: &[&str] = &["sched1", "sched2", "sched3", "sched0"];

/// What one armed run reported.
///
/// Deliberately **not** a snapshot digest type: at this revision the payload is
/// counts only, and a type that promised more than the mechanism delivers is
/// precisely the "deterministic and vacuous" failure this lane registered as
/// its most-likely-misreported outcome.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TapReport {
    /// `true` iff c2.dll loaded at its preferred base (slide 0). Recorded as a
    /// boolean, never as an address: an address in the stream would make the
    /// canonical form environment-dependent.
    pub slide_zero: bool,
    /// Sites armed, in table order.
    pub armed: Vec<String>,
    /// `(site, reason)` for every site that refused. A refusal is fail-closed:
    /// the image was **not** written.
    pub refused: Vec<(String, String)>,
    /// Hit count per armed site.
    pub hits: BTreeMap<String, u64>,
    /// Every `[stagetap]` line, verbatim, so a result table can be re-derived
    /// from the log rather than accumulated (#3231 F2).
    pub lines: Vec<String>,
    /// The payload's `TU <idx> <opcode> <cat> <flags> <cc>` rows, in order,
    /// with the leading `TU ` stripped. Empty on a counts-only run.
    pub tuples: Vec<String>,
    /// `REFUSE …` lines emitted by the bounded walk (overrun, span, arena
    /// full, implausible pointer). A non-empty list means the payload is
    /// TRUNCATED and the caller must not read absence as a terminus.
    pub walk_refusals: Vec<String>,
    /// Number of `SITE region ENTER` blocks in the payload.
    pub regions: usize,
    /// Raw tuple windows, when requested: same order as [`TapReport::tuples`].
    /// **Never** part of [`TapReport::canonical_bytes`].
    pub raw: Vec<String>,
    /// One entry per region block: `(phase, function-ordinal, tuple rows)`.
    ///
    /// The phase is the per-function site that was last entered before this
    /// region — `sched2` is immediately BEFORE the register allocator and
    /// `sched3` immediately after (`P_DAG.md` §1's order
    /// `sched1 -> globregs -> sched2 -> COLOR -> sched3`), so a pre/post-COLOR
    /// pair falls out of the region tap without ever needing the
    /// function-record → tuple-list-head offset.
    pub blocks: Vec<StageBlock>,
    /// One entry per `FN … END-FN` payload block: a WHOLE-function walk taken
    /// from the function record, when `C2RS_STAGE_FUNCWALK` is on. Unlike
    /// [`TapReport::blocks`] these visit each tuple exactly once, and they are
    /// the only observable that exists at the `after0` site.
    pub funcs: Vec<FuncWalk>,
}

/// One whole function, walked from its record at one phase.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FuncWalk {
    /// The per-function site that was entered: `sched1` … `sched0`, `after0`.
    pub phase: String,
    /// Function ordinal within the TU.
    ///
    /// **This is c2's processing order and NOT, by itself, `.text` address
    /// order** — board **#3459**. It is `g_fn`, a count of `sched1` entries,
    /// and `sched1` is one of the four optimizer-gated sites. Pair on
    /// [`FuncWalk::identity`] and check the ordinal with
    /// [`TapReport::pair_by_identity`]; never assume it.
    pub func: u32,
    /// **The function IDENTITY**, verbatim from c2's own symbol record
    /// (`func+0x00` → symbol, `symbol+0x04` → `char *`; `W-STAGETAP-6`).
    ///
    /// Empty when the payload predates #3459 or carried no `nm` field at all.
    /// Otherwise either a name or one of the tap's four refusal spellings —
    /// use [`FuncWalk::identity`], which knows the difference, rather than
    /// reading this field raw.
    pub sym: String,
    /// `(kind, sub-kind)` — the symbol record's `+0x30` / `+0x31`. Published
    /// because c2's own name path decorates by sub-kind (`"__unwind$"`,
    /// `"__catch$"`, `"$M"`) and a future consumer that needs the decorated
    /// form should not have to re-tap to get the discriminator.
    pub sym_kind: Option<(u8, u8)>,
    /// One `Vec` of `<opcode> <cat> <flags> <cc>[ | OP …]` rows per block, in
    /// block-chain order, **already reversed** back into list order (the C
    /// walk runs backward down `tuple+0x10`).
    pub blocks: Vec<Vec<String>>,
}

/// The tap's refusal spellings for [`FuncWalk::sym`]. None of them is a legal
/// identifier or mangled name, so a refusal can never be mistaken for one —
/// which is the whole reason they are words and not an empty string.
/// PROV[N] refusal spellings, not a derived value — chosen so a refusal cannot be mistaken for an identifier. Reaches no graded byte.
const SYM_REFUSALS: [&str; 4] = ["<unread>", "<empty>", "<nonascii>", "<toolong>"];

impl FuncWalk {
    /// The identity, or `None` when the tap refused the read or the payload
    /// carried none.
    ///
    /// **Absence and refusal are the same answer to a consumer** — neither may
    /// be paired on — but they are different facts about the run, and
    /// [`TapReport::pair_by_identity`] reports them as different verdicts.
    pub fn identity(&self) -> Option<&str> {
        if self.sym.is_empty() || SYM_REFUSALS.contains(&self.sym.as_str()) {
            None
        } else {
            Some(self.sym.as_str())
        }
    }
}

/// What [`TapReport::pair_by_identity`] concluded about one phase's walks.
///
/// **Positive by construction**: `Verified` demands that a bijection was
/// exhibited, and every other arm names what was missing. An enumeration of
/// failure modes with a catch-all "ok" would let a new failure arrive as a
/// pass, which is this project's signature defect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrdinalVerdict {
    /// Every expected name matched exactly one walk and every walk was
    /// matched. `ordinal_order_agrees` additionally reports whether the walk
    /// ordinals happen to run in the order the caller supplied — i.e. whether
    /// the pre-#3459 assumption held on THIS TU. It is a measurement, never a
    /// requirement.
    Verified { functions: usize, ordinal_order_agrees: bool },
    /// No walk at this phase at all. Not an error here; the caller decides.
    Empty,
    /// At least one walk carried no readable identity — the pre-#3459 state.
    /// A consumer must fall back to the ordinal *and say that it did*.
    NoIdentity { walks: usize, without: usize },
    /// A name appears twice, on one side or the other, so no pairing is
    /// well-defined. Refused rather than resolved by position.
    Duplicate { name: String, side: &'static str },
    /// Identities present, but the two name sets differ. The partial pairing
    /// is still returned — matched functions are correctly paired — and the
    /// two difference lists say exactly which are not.
    Unmatched { walks: usize, expected: usize, only_in_tap: Vec<String>, only_in_expected: Vec<String> },
}

impl OrdinalVerdict {
    /// `true` only for [`OrdinalVerdict::Verified`].
    pub fn is_verified(&self) -> bool {
        matches!(self, OrdinalVerdict::Verified { .. })
    }
    /// A short, stable label for a report table.
    pub fn label(&self) -> &'static str {
        match self {
            OrdinalVerdict::Verified { .. } => "verified",
            OrdinalVerdict::Empty => "empty",
            OrdinalVerdict::NoIdentity { .. } => "no-identity",
            OrdinalVerdict::Duplicate { .. } => "duplicate",
            OrdinalVerdict::Unmatched { .. } => "unmatched",
        }
    }
}

impl FuncWalk {
    /// Every row of every block, concatenated in block order.
    pub fn rows(&self) -> Vec<String> {
        self.blocks.iter().flat_map(|b| b.iter().cloned()).collect()
    }

    /// The rows with everything from the first `" | "` stripped — the tuple
    /// SPINE alone, without the operand records. Comparing both is how a
    /// difference gets attributed to the spine or to the operands rather than
    /// reported as one undifferentiated "DIFFERS".
    pub fn spine(&self) -> Vec<String> {
        self.rows()
            .iter()
            .map(|r| match r.split_once(" | ") {
                Some((s, _)) => s.to_string(),
                None => r.clone(),
            })
            .collect()
    }
}

/// One scheduling region as observed at one phase.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageBlock {
    /// The per-function site last entered: `sched1`/`globregs`/`sched2`/
    /// `color`/`sched3`/`sched0`.
    pub phase: String,
    /// Function ordinal within the TU (counts `sched1` entries).
    pub func: u32,
    /// `<idx> <opcode> <cat> <flags> <cc>` rows, in walk order.
    pub tuples: Vec<String>,
    /// Raw hex windows, same order, when a raw window was requested.
    pub raw: Vec<String>,
}

/// The result of [`TapReport::distinct_rows`] — a payload size beside the
/// coverage it actually represents.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DistinctRows {
    /// Total `TU` rows in the payload — what `stage-snap-tuples` publishes.
    pub rows: usize,
    /// Distinct tuple positions observed, summed over `(phase, function)`
    /// groups. A FLOOR whenever `suffix_violations > 0`.
    pub distinct: usize,
    /// Number of `(phase, function)` groups.
    pub groups: usize,
    /// Blocks whose rows are NOT a tail of the longest block in their group.
    /// Nonzero means the nesting model above is wrong for this payload and
    /// `distinct` must be read as a floor.
    pub suffix_violations: usize,
}

impl TapReport {
    /// The region blocks observed at one phase, for one function.
    pub fn blocks_at(&self, phase: &str, func: u32) -> Vec<&StageBlock> {
        self.blocks
            .iter()
            .filter(|b| b.phase == phase && b.func == func)
            .collect()
    }

    /// Every function walk at one phase, in ordinal order.
    pub fn walks_at(&self, phase: &str) -> Vec<&FuncWalk> {
        let mut v: Vec<&FuncWalk> = self.funcs.iter().filter(|f| f.phase == phase).collect();
        v.sort_by_key(|f| f.func);
        v
    }

    /// **Board #3459 — pair the walks at `phase` to `expected` BY NAME, and say
    /// so, instead of pairing by ordinal and hoping.**
    ///
    /// `expected` is the caller's own list of function names in whatever order
    /// the caller indexes by — for the obj-side consumers that is `.text`
    /// address order. The return is `(pairing, verdict)` where `pairing[i]` is
    /// the walk whose identity equals `expected[i]`, or `None`.
    ///
    /// # Why this is not just `zip`
    ///
    /// [`FuncWalk::func`] is `g_fn`, a count of `sched1` entries in
    /// `c2host/stagetap.c`. `sched1` is one of the four sites the optimizer
    /// flag gates, and each carries a second per-function gate on the function
    /// record's `+0x1c` bit 0 ([`OPT_GATED_SITES`]). A function c2 processes
    /// without entering `sched1` therefore inherits the PREVIOUS function's
    /// ordinal, and every downstream site — `after0` included — is then
    /// off by one for the rest of the TU. `w-pwords` §5 measured that live:
    /// six "failures" on one fixture with mismatches in *both* directions,
    /// `T=73` against `W=14` beside `T=6` against `W=72`, which is a permuted
    /// pairing and not a compiler that sometimes emits 59 extra words.
    ///
    /// # The two questions this answers, and they are different
    ///
    /// 1. *Which walk is this function's?* — the pairing. Order-independent by
    ///    construction, so it is right even when the ordinals are permuted.
    /// 2. *Did the ordinal happen to equal the caller's order anyway?* —
    ///    `Verified { ordinal_order_agrees }`. That is the pre-#3459
    ///    assumption, now **measured on every TU instead of assumed on all of
    ///    them**. It is reported and never required: a `false` is a fact about
    ///    c2, not a failure.
    ///
    /// A partial pairing is returned even on
    /// [`OrdinalVerdict::Unmatched`] — a function whose name matched IS
    /// correctly paired, and throwing the whole TU away would re-create the
    /// quarantine this exists to remove.
    pub fn pair_by_identity<'a>(
        &'a self,
        phase: &str,
        expected: &[String],
    ) -> (Vec<Option<&'a FuncWalk>>, OrdinalVerdict) {
        let none = || vec![None; expected.len()];
        let walks = self.walks_at(phase);
        if walks.is_empty() {
            return (none(), OrdinalVerdict::Empty);
        }
        let without = walks.iter().filter(|w| w.identity().is_none()).count();
        if without > 0 {
            return (none(), OrdinalVerdict::NoIdentity { walks: walks.len(), without });
        }
        // Duplicates are REFUSED, not resolved by position: resolving one by
        // position is exactly the assumption this function exists to retire.
        let mut by_name: BTreeMap<&str, &FuncWalk> = BTreeMap::new();
        for w in &walks {
            let n = w.identity().expect("checked above");
            if by_name.insert(n, w).is_some() {
                return (none(), OrdinalVerdict::Duplicate { name: n.to_string(), side: "tap" });
            }
        }
        let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
        for e in expected {
            if seen.insert(e.as_str(), ()).is_some() {
                return (
                    none(),
                    OrdinalVerdict::Duplicate { name: e.clone(), side: "expected" },
                );
            }
        }
        let pairing: Vec<Option<&FuncWalk>> =
            expected.iter().map(|e| by_name.get(e.as_str()).copied()).collect();
        let only_in_expected: Vec<String> = expected
            .iter()
            .filter(|e| !by_name.contains_key(e.as_str()))
            .cloned()
            .collect();
        let only_in_tap: Vec<String> = by_name
            .keys()
            .filter(|n| !seen.contains_key(**n))
            .map(|n| (*n).to_string())
            .collect();
        if !only_in_expected.is_empty() || !only_in_tap.is_empty() {
            return (
                pairing,
                OrdinalVerdict::Unmatched {
                    walks: walks.len(),
                    expected: expected.len(),
                    only_in_tap,
                    only_in_expected,
                },
            );
        }
        // Sets agree and both are duplicate-free, so `walks.len() ==
        // expected.len()` and the ordinal question is well posed.
        let agrees = walks
            .iter()
            .zip(expected.iter())
            .all(|(w, e)| w.identity() == Some(e.as_str()));
        (
            pairing,
            OrdinalVerdict::Verified { functions: expected.len(), ordinal_order_agrees: agrees },
        )
    }

    /// [`TapReport::pair_by_identity`]'s verdict alone.
    pub fn verify_ordinals(&self, phase: &str, expected: &[String]) -> OrdinalVerdict {
        self.pair_by_identity(phase, expected).1
    }

    /// **The pre/post-COLOR pair.** Returns `(before, after)` as the
    /// concatenated tuple rows of every region observed at `sched2` and at
    /// `sched3` for one function.
    ///
    /// An EMPTY difference is a finding, not a green: it means the walk is
    /// reading a list COLOR does not write, and it is reported in those words.
    pub fn color_pair(&self, func: u32) -> (Vec<String>, Vec<String>) {
        let cat = |phase: &str| -> Vec<String> {
            self.blocks_at(phase, func)
                .into_iter()
                .flat_map(|b| b.tuples.iter().cloned())
                .collect()
        };
        (cat("sched2"), cat("sched3"))
    }

    /// **How much of `stage-snap-tuples` is a re-read, and how much is new.**
    ///
    /// The bounded walk terminates on `next == 0` — the end of the FUNCTION's
    /// tuple list, not the end of the region it was handed (`stagetap.c`'s
    /// `tap_walk_tuples`). So the walk launched at region 1 emits the whole
    /// list, region 2's walk emits the same list minus its first region, and so
    /// on: within one `(phase, function)` the blocks are nested suffixes and
    /// every published tuple count is inflated by the nesting.
    ///
    /// **Compare a count, never a status** (arch review 2026-08-21, finding 1).
    /// `stage-snap-tuples` is a payload size; this is the number of distinct
    /// tuple positions actually observed, which is what a coverage claim needs.
    ///
    /// The suffix structure is **checked, not assumed**: each block's rows,
    /// with the walk index stripped, must equal the tail of the longest block
    /// in its group. A group where that fails contributes a
    /// `suffix_violations` count and its `distinct` term is then a FLOOR — the
    /// blocks are not nested and the union could be larger.
    pub fn distinct_rows(&self) -> DistinctRows {
        // Group in first-seen order; a BTreeMap over (phase, func) would work
        // too but the order is the diagnostic one.
        let mut groups: Vec<((String, u32), Vec<&StageBlock>)> = Vec::new();
        for b in &self.blocks {
            let key = (b.phase.clone(), b.func);
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => v.push(b),
                None => groups.push((key, vec![b])),
            }
        }
        let strip = |row: &String| -> String {
            match row.split_once(' ') {
                Some((_idx, rest)) => rest.to_string(),
                None => row.clone(),
            }
        };
        let mut out = DistinctRows {
            rows: self.tuples.len(),
            distinct: 0,
            groups: groups.len(),
            suffix_violations: 0,
        };
        for (_, blocks) in &groups {
            let Some(longest) = blocks.iter().max_by_key(|b| b.tuples.len()) else {
                continue;
            };
            let reference: Vec<String> = longest.tuples.iter().map(strip).collect();
            out.distinct += reference.len();
            for b in blocks {
                let rows: Vec<String> = b.tuples.iter().map(strip).collect();
                if rows.len() > reference.len() {
                    out.suffix_violations += 1;
                    continue;
                }
                let tail = &reference[reference.len() - rows.len()..];
                if tail != rows.as_slice() {
                    out.suffix_violations += 1;
                }
            }
        }
        out
    }

    /// Did the tap actually arm? **The environment control's predicate.**
    ///
    /// A run where nothing armed produces zero hits, and zero hits is
    /// indistinguishable from "the pass never ran" unless something asserts
    /// this. That is #3219/#3231's failure — a registered RED reading GREEN
    /// with a clean suite and the right exit code — in a new place.
    pub fn armed_ok(&self) -> bool {
        !self.armed.is_empty() && self.refused.is_empty()
    }

    /// Total hits over all armed sites.
    pub fn total_hits(&self) -> u64 {
        self.hits.values().copied().sum()
    }

    /// **Did the tap arm AND execute?** The predicate a neutrality grade must
    /// be conditioned on.
    ///
    /// [`TapReport::armed_ok`] says the bytes were written; it says nothing
    /// about whether c2 ever reached them. On a TU that emits no function body
    /// c2 runs no per-function phase, every detour is dead code, and an
    /// armed-vs-disarmed obj compare is byte-identical **for free** — the
    /// identity is a fact about the fixture, not about the tap. Counting such
    /// an obj in a required-zero's denominator inflates it with cells that
    /// could not have failed, which is this project's signature defect
    /// (absence read as success) wearing the required-zero's own clothes.
    ///
    /// Positive by construction: it demands evidence that something happened,
    /// and is never an enumeration of the ways a run can be empty.
    pub fn armed_and_fired(&self) -> bool {
        self.armed_ok() && self.total_hits() > 0
    }

    /// The payload's canonical bytes: the exact stream a digest is taken over.
    ///
    /// **SCHEMA RULE, and it is the whole of G2b:** no address, no pointer, no
    /// path, no PID, no timestamp and no allocation count may appear. Only
    /// walk indices and values read out of c2's own records. Without this a
    /// digest is stable only because the environment was.
    pub fn canonical_bytes(&self) -> String {
        let mut out = String::new();
        // SCHEMA 2 — #3459 added the function identity to the `FN` row. The
        // bump is honest bookkeeping and costs nothing: nothing in the tree
        // persists a canonical stream or pins a digest, and `stage snap`'s
        // rerun comparison is within one process.
        out.push_str("SCHEMA 2\n");
        out.push_str(if self.slide_zero { "SLIDE 0\n" } else { "SLIDE nonzero\n" });
        for s in &self.armed {
            out.push_str("ARMED ");
            out.push_str(s);
            out.push('\n');
        }
        for (k, v) in &self.hits {
            out.push_str("HITS ");
            out.push_str(k);
            out.push(' ');
            out.push_str(&v.to_string());
            out.push('\n');
        }
        for t in &self.tuples {
            out.push_str("TU ");
            out.push_str(t);
            out.push('\n');
        }
        for f in &self.funcs {
            out.push_str("FN ");
            out.push_str(&f.phase);
            out.push(' ');
            out.push_str(&f.func.to_string());
            // The identity IS canonical: it is read out of c2's own record and
            // contains no address, path, PID or timestamp, so it satisfies the
            // schema rule above — and a stream that omitted it would call two
            // runs identical while the function-to-ordinal binding moved.
            out.push(' ');
            out.push_str(if f.sym.is_empty() { "<none>" } else { f.sym.as_str() });
            out.push('\n');
            for (i, b) in f.blocks.iter().enumerate() {
                out.push_str("BLK ");
                out.push_str(&i.to_string());
                out.push('\n');
                for row in b {
                    out.push_str("FT ");
                    out.push_str(row);
                    out.push('\n');
                }
            }
        }
        for w in &self.walk_refusals {
            out.push_str(w);
            out.push('\n');
        }
        out
    }

    /// FNV-1a over [`TapReport::canonical_bytes`]. Hand-rolled: std-only, zero
    /// external crates, and nothing here needs a cryptographic digest — this
    /// compares a stream against itself across runs.
    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.canonical_bytes().as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        h
    }

    /// Hits at one site (0 when unarmed or never reached — the caller is
    /// expected to have checked [`TapReport::armed_ok`] first).
    pub fn hits_at(&self, site: &str) -> u64 {
        self.hits.get(site).copied().unwrap_or(0)
    }

    /// Parse c2host's stderr. Tolerant of interleaved c2 output by construction:
    /// only lines carrying the `[stagetap] ` marker are read.
    pub fn parse(stderr: &str) -> TapReport {
        let mut r = TapReport::default();
        // Which payload context an `OP` line belongs to. `OP` rows follow
        // either a region-walk `TU` row or a function-walk `FT` row, and
        // attaching one to the wrong owner would silently mix two
        // observables.
        let mut in_funcwalk = false;
        for raw in stderr.lines() {
            let Some(rest) = raw.trim_start().strip_prefix("[stagetap] ") else {
                continue;
            };
            r.lines.push(rest.to_string());
            let mut it = rest.split_whitespace();
            match it.next() {
                Some("ARM") => {
                    if let Some(name) = it.next() {
                        r.armed.push(name.to_string());
                    }
                }
                Some("REFUSE") => {
                    // Two different refusals share the word. An ARMING refusal
                    // names a site and means nothing was patched; a WALK
                    // refusal means the payload is truncated. Conflating them
                    // would let a truncated payload read as a failed arm (or,
                    // worse, the reverse).
                    if rest.contains("walk-") || rest.contains("arena-full") {
                        r.walk_refusals.push(rest.to_string());
                    } else {
                        let name = it.next().unwrap_or("?").to_string();
                        r.refused.push((name, rest.to_string()));
                    }
                }
                Some("END") => {
                    let name = it.next().unwrap_or("?").to_string();
                    let n = it
                        .find_map(|t| t.strip_prefix("hits="))
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);
                    r.hits.insert(name, n);
                }
                Some("TAP") => {
                    r.slide_zero = rest.contains("slide=0");
                }
                Some("ARENA") => {
                    // TRUNCATION HAS A SECOND SPELLING, and the first version
                    // of this parser could not see it. `ARENA bytes=N full=1`
                    // is the arena's own end-of-buffer flag; the `REFUSE …
                    // arena-full` line is written from a reserved tail so it
                    // can always be emitted, but a payload that filled the
                    // arena OUTSIDE the walk (in a SITE header, say) sets only
                    // this flag. Reading it as a walk refusal is what keeps
                    // `walk_refusals.is_empty()` meaning "the payload is
                    // complete" rather than "the walk happened not to say so".
                    if rest.contains("full=1") {
                        r.walk_refusals.push(rest.to_string());
                    }
                }
                Some("RAW") => {
                    let t: Vec<&str> = rest.split_whitespace().collect();
                    if let Some(h) = t.get(2) {
                        r.raw.push((*h).to_string());
                        if let Some(b) = r.blocks.last_mut() {
                            b.raw.push((*h).to_string());
                        }
                    }
                }
                Some("TU") => {
                    let row = rest["TU ".len()..].to_string();
                    if let Some(b) = r.blocks.last_mut() {
                        b.tuples.push(row.clone());
                    }
                    r.tuples.push(row);
                }
                Some("FN") => {
                    // FN <phase> fn <n> [sk <kk> <ss> nm <name>]
                    //
                    // The identity tail is parsed as OPTIONAL, and not for
                    // forward compatibility — it is how a payload produced by a
                    // tap without #3459's field reads as `NoIdentity` (a named
                    // verdict) instead of as a name that happens to be empty.
                    let t: Vec<&str> = rest.split_whitespace().collect();
                    let phase = t.get(1).copied().unwrap_or("?").to_string();
                    let func = t.get(3).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                    let sym_kind = match (t.get(4), t.get(5), t.get(6)) {
                        (Some(&"sk"), Some(k), Some(s)) => {
                            match (u8::from_str_radix(k, 16), u8::from_str_radix(s, 16)) {
                                (Ok(k), Ok(s)) => Some((k, s)),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    // The name is the TAIL of the line, taken whole. The C side
                    // refuses any name containing a byte outside `0x21..=0x7e`,
                    // so a name can never re-split here — and a name that could
                    // have arrives as `<nonascii>`, which is a refusal.
                    let sym = match rest.split_once(" nm ") {
                        Some((_, n)) => n.trim().to_string(),
                        None => String::new(),
                    };
                    r.funcs.push(FuncWalk { phase, func, sym, sym_kind, blocks: Vec::new() });
                    in_funcwalk = true;
                }
                Some("BLK") => {
                    if let Some(f) = r.funcs.last_mut() {
                        f.blocks.push(Vec::new());
                    }
                }
                Some("FT") => {
                    // FT <i> <opcode> <cat> <flags> <cc>. The walk index is
                    // dropped: the C side emits rows in REVERSE list order, so
                    // the index descends and carries no information the
                    // position does not.
                    let row = match rest["FT ".len()..].split_once(' ') {
                        Some((_i, v)) => v.to_string(),
                        None => String::new(),
                    };
                    if let Some(b) = r.funcs.last_mut().and_then(|f| f.blocks.last_mut()) {
                        b.push(row);
                    }
                }
                Some("END-FN") => {
                    // The C walk runs backward down `tuple+0x10` (prev), so
                    // each block's rows arrive last-first. Reverse them here
                    // rather than in every consumer — an observable whose
                    // order is the finding must not be published inverted.
                    if let Some(f) = r.funcs.last_mut() {
                        for b in &mut f.blocks {
                            b.reverse();
                        }
                    }
                    in_funcwalk = false;
                }
                Some("OP") => {
                    // Appended INLINE to the row it belongs to, so every
                    // existing comparison over tuple rows covers the operand
                    // records for free when the lever is on, and covers
                    // exactly what it used to when it is off.
                    let op = rest.to_string();
                    let target = if in_funcwalk {
                        r.funcs.last_mut().and_then(|f| f.blocks.last_mut()).and_then(|b| b.last_mut())
                    } else {
                        r.blocks.last_mut().and_then(|b| b.tuples.last_mut())
                    };
                    if let Some(row) = target {
                        row.push_str(" | ");
                        row.push_str(&op);
                    }
                    if !in_funcwalk {
                        if let Some(last) = r.tuples.last_mut() {
                            last.push_str(" | ");
                            last.push_str(&op);
                        }
                    }
                }
                Some("SITE") => {
                    r.regions += 1;
                    // SITE region ENTER <phase> fn <n> r <n>
                    let t: Vec<&str> = rest.split_whitespace().collect();
                    let phase = t.get(3).copied().unwrap_or("?").to_string();
                    let func = t
                        .get(5)
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    r.blocks.push(StageBlock { phase, func, tuples: Vec::new(), raw: Vec::new() });
                }
                _ => {}
            }
        }
        r
    }
}

impl Toolchain {
    /// **P0.1 replay with the stage tap armed.**
    ///
    /// Identical to [`Toolchain::replay`] in every respect except two
    /// environment variables, and that is the point: the neutrality claim this
    /// whole instrument stands on is *"the same command, plus arming, produces
    /// the same obj"*, and it is only a real claim if the command really is the
    /// same one. `build_replay_command` is reused unchanged.
    ///
    /// Pass an empty `taps` to get the **disarmed** control through the exact
    /// same code path — which is how the neutrality test avoids comparing two
    /// different functions and calling the result a measurement.
    pub fn replay_tapped(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
    ) -> io::Result<(ObjImage, TapReport)> {
        self.replay_tapped_with(captured, bundle_dir, out_obj, taps, false)
    }

    /// [`Toolchain::replay_tapped`] with the bounded tuple-walk payload
    /// switchable.
    ///
    /// Counts and payload are the SAME mechanism at two settings, deliberately:
    /// it means the expensive content run and the cheap neutrality sweep are
    /// not two different instruments whose agreement would have to be argued.
    /// It also means **G1 can be re-run at the full table WITH the payload**
    /// rather than extrapolated from a counts-only run — the payload is the
    /// half that touches c2's own memory, so extrapolating would be assuming
    /// the answer.
    pub fn replay_tapped_with(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
        payload: bool,
    ) -> io::Result<(ObjImage, TapReport)> {
        self.replay_tapped_raw(captured, bundle_dir, out_obj, taps, payload, 0)
    }

    /// [`Toolchain::replay_tapped_with`] plus a **raw window**: `raw` bytes of
    /// every tuple, dumped as hex beside the decoded row.
    ///
    /// This is how "which fields does COLOR write?" gets ANSWERED instead of
    /// guessed — diff the `sched2` and `sched3` dumps and the offsets the
    /// register allocator touches name themselves. It is **excluded from
    /// [`TapReport::canonical_bytes`] by construction**: a raw window can
    /// contain pointers, and a digest over pointers is stable only because the
    /// allocator happened to be.
    pub fn replay_tapped_raw(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
        payload: bool,
        raw: u32,
    ) -> io::Result<(ObjImage, TapReport)> {
        let (obj, rep) =
            self.replay_tapped_inner(captured, bundle_dir, out_obj, taps, payload, raw, None, true, None)?;
        // `require_obj` was true, so the missing-obj case returned `Err` above.
        let obj = obj.expect("replay_tapped_inner(require_obj = true) returned no obj");
        Ok((obj, rep))
    }

    /// [`Toolchain::replay_tapped_raw`] with the load slide DELIBERATELY WRONG
    /// by `force_slide` bytes — **the fail-closed check's own test lever.**
    ///
    /// Why this exists as an API instead of an environment variable set by the
    /// test: the tests in one integration binary run as threads of one process,
    /// so a `set_var` would leak into every other test's child. The lever has
    /// to travel with the command.
    ///
    /// Why it exists at all: `tap_arm`'s check is *"the byte is still `0xE8`
    /// **and** the decoded target equals the recorded target plus the MEASURED
    /// SLIDE"*, and the `+ slide` half — the half the lane's first plan defect
    /// was about — had never executed against a live image, because c2.dll
    /// loads at its preferred base on every run here (slide 0 always). A guard
    /// nobody has watched fire is a guard nobody has tested.
    ///
    /// The only correct outcome is **every requested site refusing and the obj
    /// coming out unchanged anyway**.
    /// Returns `None` for the obj when the replay produced none — c2 crashed
    /// or aborted mid-pass. That is NOT an error on this path, and the reason
    /// is the ordering of failures: the interesting outcome of a wrong slide is
    /// **what got armed**, and if a missing obj were an `Err` the arming
    /// assertion could never be reached. Measured, not anticipated: with the
    /// target check disabled, a `+0x18` slide arms five sites and c2 SIGSEGVs,
    /// and the first version of this signature reported that as
    /// *"forced-slide replay failed"* — a true sentence that names the wrong
    /// defect (work/oracle/fixround/mutation_failclosed.log).
    pub fn replay_tapped_forced_slide(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
        force_slide: u32,
    ) -> io::Result<(Option<ObjImage>, TapReport)> {
        self.replay_tapped_inner(
            captured,
            bundle_dir,
            out_obj,
            taps,
            false,
            0,
            Some(force_slide),
            false,
            None,
        )
    }

    /// [`Toolchain::replay_tapped_with`] with the two probe levers set
    /// EXPLICITLY rather than read from the ambient environment.
    ///
    /// `ops` walks each tuple's operand and symbol/candidate records; `funcwalk`
    /// walks whole functions from the function record (and is the only way to
    /// observe anything at the `after0` site). Both default off through every
    /// other entry point, so a caller that does not ask for them measures
    /// exactly what it measured before this lane.
    pub fn replay_tapped_probe(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
        ops: bool,
        funcwalk: bool,
    ) -> io::Result<(ObjImage, TapReport)> {
        let (obj, rep) = self.replay_tapped_inner(
            captured,
            bundle_dir,
            out_obj,
            taps,
            true,
            0,
            None,
            true,
            Some((ops, funcwalk)),
        )?;
        let obj = obj.expect("replay_tapped_inner(require_obj = true) returned no obj");
        Ok((obj, rep))
    }

    #[allow(clippy::too_many_arguments)]
    fn replay_tapped_inner(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
        payload: bool,
        raw: u32,
        force_slide: Option<u32>,
        require_obj: bool,
        probe: Option<(bool, bool)>,
    ) -> io::Result<(Option<ObjImage>, TapReport)> {
        let (mut cmd, out_abs) = self.build_replay_command(captured, bundle_dir, out_obj)?;
        match force_slide {
            Some(v) => {
                cmd.env("C2RS_STAGE_FORCE_SLIDE", format!("{v:x}"));
            }
            None => {
                cmd.env_remove("C2RS_STAGE_FORCE_SLIDE");
            }
        }
        if raw > 0 {
            cmd.env("C2RS_STAGE_RAW", raw.to_string());
        } else {
            cmd.env_remove("C2RS_STAGE_RAW");
        }
        // THE TWO PROBE LEVERS, FORWARDED DELIBERATELY.
        //
        // `C2RS_STAGE_OPS` walks each tuple's operand and symbol/candidate
        // records; `C2RS_STAGE_FUNCWALK` walks the whole function from the
        // function record instead of relying on the region tap. Both are read
        // from THIS process's environment and either forwarded or REMOVED —
        // never left to silent inheritance, which is the same inertness rule
        // `C2RS_STAGE_TAPS` gets: an ambient value in a caller's environment
        // must not be able to change what a "default" run measures.
        for (k, explicit) in [("C2RS_STAGE_OPS", probe.map(|p| p.0)), ("C2RS_STAGE_FUNCWALK", probe.map(|p| p.1))] {
            let on = match explicit {
                // An EXPLICIT setting wins over the ambient one. A test cannot
                // use `set_var` here: the integration binary runs its tests as
                // threads of one process, so a `set_var` would leak into every
                // other test's child (the same reason `replay_tapped_forced_slide`
                // is an API and not a variable).
                Some(v) => v,
                None => matches!(std::env::var(k), Ok(v) if !v.is_empty() && v != "0"),
            };
            if on {
                cmd.env(k, "1");
            } else {
                cmd.env_remove(k);
            }
        }
        if payload {
            cmd.env("C2RS_STAGE_PAYLOAD", "1");
        } else {
            cmd.env_remove("C2RS_STAGE_PAYLOAD");
        }
        if taps.is_empty() {
            // Inertness, asserted at the seam and not only in C: with nothing
            // requested the variable is REMOVED, so an ambient value in the
            // caller's environment cannot silently arm a "disarmed" control.
            cmd.env_remove("C2RS_STAGE_TAPS");
        } else {
            cmd.env("C2RS_STAGE_TAPS", taps.join(","));
        }
        let output = cmd.output()?;
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let report = TapReport::parse(&stderr);
        if !out_abs.exists() || std::fs::metadata(&out_abs)?.len() == 0 {
            if !require_obj {
                return Ok((None, report));
            }
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "tapped replay produced no (or an empty) obj at {} — \
                     c2 crashed or aborted mid-pass.\nstderr:\n{stderr}",
                    out_abs.display(),
                ),
            ));
        }
        Ok((Some(ObjImage::new(std::fs::read(&out_abs)?)), report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_an_armed_run() {
        let s = "[c2host] noise\n\
                 [stagetap] hmodule=00000018 invoke=10bebffd slide=00000000\n\
                 [stagetap] ARM sched1 site=10b7dc9f -> 10be6382 (x)\n\
                 [stagetap] armed=1 of 7 requested-list=sched1\n\
                 c2 chatter\n\
                 [stagetap] TAP 1 c80981c0 slide=0\n\
                 [stagetap] END sched1 hits=12\n";
        let r = TapReport::parse(s);
        assert!(r.armed_ok());
        assert_eq!(r.armed, vec!["sched1".to_string()]);
        assert_eq!(r.hits_at("sched1"), 12);
        assert_eq!(r.total_hits(), 12);
        assert!(r.slide_zero);
    }

    #[test]
    fn a_refusal_is_not_an_armed_run() {
        let s = "[stagetap] REFUSE sched1 site=10b7dc9f opcode=00 (expected e8)\n\
                 [stagetap] armed=0 of 7 requested-list=sched1\n";
        let r = TapReport::parse(s);
        assert!(!r.armed_ok());
        assert_eq!(r.refused.len(), 1);
        assert_eq!(r.refused[0].0, "sched1");
    }

    #[test]
    fn an_armed_run_that_never_fired_is_not_a_graded_cell() {
        // The population G1's denominator must NOT count: the sites were
        // patched, c2 never reached one of them, and the armed/disarmed objs
        // are therefore identical for free.
        let s = "[stagetap] ARM sched1 site=10b7dc9f -> 10be6382 (x)\n\
                 [stagetap] TAP 1 c80981c0 slide=0\n\
                 [stagetap] END sched1 hits=0\n";
        let r = TapReport::parse(s);
        assert!(r.armed_ok(), "the bytes WERE written");
        assert!(!r.armed_and_fired(), "but nothing executed, so nothing was graded");
        assert_eq!(r.total_hits(), 0);
    }

    /// The payload's size and its COVERAGE are different numbers, and the
    /// difference is not small: the walk runs to the end of the LIST, so the
    /// walk launched at region 1 re-reads every later region.
    #[test]
    fn the_tuple_count_is_a_payload_size_and_the_distinct_count_is_the_coverage() {
        // One function, one phase, three regions of one tuple each. Region 1's
        // walk sees all three, region 2's sees two, region 3's sees one:
        // 6 rows published for 3 distinct tuple positions.
        let s = "[stagetap] SITE region ENTER sched2 fn 1 r 1\n\
                 [stagetap] TU 0 0000000b 0d 01 00\n\
                 [stagetap] TU 1 000000d6 0f 01 00\n\
                 [stagetap] TU 2 0000017a 12 01 04\n\
                 [stagetap] END-REGION\n\
                 [stagetap] SITE region ENTER sched2 fn 1 r 2\n\
                 [stagetap] TU 0 000000d6 0f 01 00\n\
                 [stagetap] TU 1 0000017a 12 01 04\n\
                 [stagetap] END-REGION\n\
                 [stagetap] SITE region ENTER sched2 fn 1 r 3\n\
                 [stagetap] TU 0 0000017a 12 01 04\n\
                 [stagetap] END-REGION\n";
        let d = TapReport::parse(s).distinct_rows();
        assert_eq!(d.rows, 6, "the payload size");
        assert_eq!(d.distinct, 3, "the tuple positions actually observed");
        assert_eq!(d.groups, 1);
        assert_eq!(d.suffix_violations, 0, "the blocks ARE nested suffixes here");
    }

    /// The nesting is CHECKED. If a block is not a tail of its group's longest
    /// block the union may exceed the longest, so `distinct` is a floor and the
    /// instrument has to say so rather than publish an exact-looking number.
    #[test]
    fn a_block_that_is_not_a_suffix_makes_the_distinct_count_a_floor() {
        let s = "[stagetap] SITE region ENTER sched2 fn 1 r 1\n\
                 [stagetap] TU 0 0000000b 0d 01 00\n\
                 [stagetap] TU 1 000000d6 0f 01 00\n\
                 [stagetap] END-REGION\n\
                 [stagetap] SITE region ENTER sched2 fn 1 r 2\n\
                 [stagetap] TU 0 deadbeef 19 01 00\n\
                 [stagetap] END-REGION\n";
        let d = TapReport::parse(s).distinct_rows();
        assert_eq!(d.rows, 3);
        assert_eq!(
            d.suffix_violations, 1,
            "`deadbeef` is not the tail of the longest block, so the nesting model fails"
        );
    }

    /// Two phases are two independent observations of the same list, so they
    /// group separately — a distinct count that collapsed them would under-report
    /// coverage by exactly the factor the pre/post-COLOR pair depends on.
    #[test]
    fn phases_and_functions_are_separate_groups() {
        let s = "[stagetap] SITE region ENTER sched2 fn 1 r 1\n\
                 [stagetap] TU 0 0000000b 0d 01 00\n\
                 [stagetap] END-REGION\n\
                 [stagetap] SITE region ENTER sched3 fn 1 r 1\n\
                 [stagetap] TU 0 0000000b 0d 01 00\n\
                 [stagetap] END-REGION\n\
                 [stagetap] SITE region ENTER sched2 fn 2 r 1\n\
                 [stagetap] TU 0 0000000b 0d 01 00\n\
                 [stagetap] END-REGION\n";
        let d = TapReport::parse(s).distinct_rows();
        assert_eq!(d.groups, 3);
        assert_eq!(d.distinct, 3);
        assert_eq!(d.rows, 3);
    }

    #[test]
    fn a_full_arena_is_a_truncated_payload_even_without_a_refuse_line() {
        // `ARENA … full=1` is truncation's second spelling. Before the fix
        // round the parser read only `REFUSE … arena-full`, and the C side
        // could not emit that line once the arena was full (the announcement
        // used the same appender that had just stopped appending).
        let s = "[stagetap] TU 0 00000272 0d 01 04\n\
                 [stagetap] ARENA bytes=4194303 full=1\n";
        let r = TapReport::parse(s);
        assert_eq!(r.tuples.len(), 1);
        assert_eq!(
            r.walk_refusals.len(),
            1,
            "a full arena must read as a REFUSAL: the tuple count above it is a \
             floor, not a measurement"
        );
        // And the healthy spelling must NOT be read as truncation.
        let ok = TapReport::parse("[stagetap] ARENA bytes=17 full=0\n");
        assert!(ok.walk_refusals.is_empty());
    }

    // -----------------------------------------------------------------------
    // #3459 — the function identity, and the fence around it
    // -----------------------------------------------------------------------

    /// Three functions at `after0`, ordinals 1..3, names `?a@` `?b@` `?c@`.
    /// **The ordinals are deliberately NOT in the same order as the names**
    /// in one of the tests below; here they are.
    fn idpayload(rows: &[(u32, &str)]) -> String {
        let mut s = String::new();
        for (ord, name) in rows {
            s.push_str(&format!("[stagetap] FN after0 fn {ord} sk 03 54 nm {name}\n"));
            s.push_str("[stagetap] BLK 0 rev\n");
            s.push_str("[stagetap] FT 0 0000000b 0d 01 00\n");
            s.push_str("[stagetap] END-FN\n");
        }
        s
    }

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn the_fn_row_carries_a_function_identity() {
        let r = TapReport::parse(&idpayload(&[(1, "?a@@YAHXZ"), (2, "?b@@YAHXZ")]));
        assert_eq!(r.funcs.len(), 2);
        assert_eq!(r.funcs[0].identity(), Some("?a@@YAHXZ"));
        assert_eq!(r.funcs[0].sym_kind, Some((0x03, 0x54)));
        // And the identity is in the canonical stream, so two runs whose
        // ordinal→function binding moved cannot digest the same.
        let swapped = TapReport::parse(&idpayload(&[(1, "?b@@YAHXZ"), (2, "?a@@YAHXZ")]));
        assert_ne!(
            r.digest(),
            swapped.digest(),
            "the canonical stream is blind to WHICH function each walk belongs to"
        );
    }

    /// A payload from a tap without #3459's field must read as
    /// `NoIdentity` — a named verdict — and never as a name that is empty.
    #[test]
    fn a_payload_without_the_identity_field_says_so() {
        let s = "[stagetap] FN after0 fn 1\n\
                 [stagetap] BLK 0 rev\n\
                 [stagetap] FT 0 0000000b 0d 01 00\n\
                 [stagetap] END-FN\n";
        let r = TapReport::parse(s);
        assert_eq!(r.funcs[0].identity(), None);
        assert_eq!(
            r.verify_ordinals("after0", &names(&["?a@@YAHXZ"])),
            OrdinalVerdict::NoIdentity { walks: 1, without: 1 }
        );
    }

    /// The tap's refusal spellings are REFUSALS, not names. If `<unread>` were
    /// read as an identity a whole TU would pair against a fiction.
    #[test]
    fn a_refusal_spelling_is_not_an_identity() {
        for spell in SYM_REFUSALS {
            let r = TapReport::parse(&idpayload(&[(1, spell)]));
            assert_eq!(r.funcs[0].identity(), None, "{spell} was read as a name");
        }
    }

    /// The healthy case: names agree, and the ordinal happens to run in the
    /// caller's order too.
    #[test]
    fn a_matching_identity_set_verifies_and_reports_the_ordinal_agreement() {
        let r = TapReport::parse(&idpayload(&[(1, "?a@"), (2, "?b@"), (3, "?c@")]));
        let (pairing, v) = r.pair_by_identity("after0", &names(&["?a@", "?b@", "?c@"]));
        assert_eq!(v, OrdinalVerdict::Verified { functions: 3, ordinal_order_agrees: true });
        assert_eq!(pairing[1].unwrap().func, 2);
    }

    /// **THE FENCE, half one: a PERMUTED pairing is caught.**
    ///
    /// This is `w-pwords` §5's live failure, reproduced synthetically: c2's
    /// ordinals run `?c@ ?a@ ?b@` while the caller indexes `.text` address
    /// order `?a@ ?b@ ?c@`. The pairing must still be RIGHT — that is the
    /// deliverable — and `ordinal_order_agrees` must go **false**, which is the
    /// part the pre-#3459 consumers had no way to learn.
    #[test]
    fn a_permuted_ordinal_is_paired_correctly_and_reported_as_disagreeing() {
        let r = TapReport::parse(&idpayload(&[(1, "?c@"), (2, "?a@"), (3, "?b@")]));
        let (pairing, v) = r.pair_by_identity("after0", &names(&["?a@", "?b@", "?c@"]));
        assert_eq!(
            v,
            OrdinalVerdict::Verified { functions: 3, ordinal_order_agrees: false },
            "a permuted pairing read as agreeing — the ordinal check is a rubber stamp"
        );
        // The pairing itself is correct, which the old `func == i+1` rule was
        // not: index 0 is `?a@`, which c2 walked SECOND.
        assert_eq!(pairing[0].unwrap().func, 2);
        assert_eq!(pairing[1].unwrap().func, 3);
        assert_eq!(pairing[2].unwrap().func, 1);
    }

    /// **THE FENCE, half two: a WRONG name list is refused.**
    ///
    /// One expected name replaced by a fiction. The verdict must be
    /// `Unmatched` and must name both sides of the difference — a bare
    /// `false` would leave the caller unable to say what went wrong.
    #[test]
    fn a_wrong_expected_name_is_refused_and_both_sides_are_named() {
        let r = TapReport::parse(&idpayload(&[(1, "?a@"), (2, "?b@")]));
        let (pairing, v) = r.pair_by_identity("after0", &names(&["?a@", "?NOT_A_REAL_NAME@"]));
        match v {
            OrdinalVerdict::Unmatched { only_in_tap, only_in_expected, .. } => {
                assert_eq!(only_in_tap, names(&["?b@"]));
                assert_eq!(only_in_expected, names(&["?NOT_A_REAL_NAME@"]));
            }
            other => panic!("a fabricated name was ABSORBED: {other:?}"),
        }
        // …and the function that DID match is still paired. Discarding it
        // would re-create the whole-TU quarantine #3459 exists to remove.
        assert_eq!(pairing[0].unwrap().func, 1);
        assert!(pairing[1].is_none());
    }

    /// A short list is a difference, not a truncation to be tolerated.
    #[test]
    fn a_short_expected_list_is_refused() {
        let r = TapReport::parse(&idpayload(&[(1, "?a@"), (2, "?b@")]));
        let v = r.verify_ordinals("after0", &names(&["?a@"]));
        assert!(matches!(v, OrdinalVerdict::Unmatched { walks: 2, expected: 1, .. }), "{v:?}");
    }

    /// Two walks with one name: no pairing is well-defined, and resolving it
    /// by position would be the assumption this whole API retires.
    #[test]
    fn a_duplicate_identity_is_refused_rather_than_resolved_by_position() {
        let r = TapReport::parse(&idpayload(&[(1, "?a@"), (2, "?a@")]));
        assert_eq!(
            r.verify_ordinals("after0", &names(&["?a@", "?b@"])),
            OrdinalVerdict::Duplicate { name: "?a@".to_string(), side: "tap" }
        );
        let r2 = TapReport::parse(&idpayload(&[(1, "?a@"), (2, "?b@")]));
        assert_eq!(
            r2.verify_ordinals("after0", &names(&["?a@", "?a@"])),
            OrdinalVerdict::Duplicate { name: "?a@".to_string(), side: "expected" }
        );
    }

    /// No walk at the phase is `Empty` and not a silent verified-zero.
    #[test]
    fn no_walk_at_a_phase_is_empty_and_never_a_vacuous_pass() {
        let r = TapReport::parse(&idpayload(&[(1, "?a@")]));
        let v = r.verify_ordinals("sched2", &names(&["?a@"]));
        assert_eq!(v, OrdinalVerdict::Empty);
        assert!(!v.is_verified());
    }

    #[test]
    fn an_unarmed_run_reports_nothing_and_is_not_ok() {
        // The inert path: no [stagetap] line at all. `armed_ok` must be false,
        // because "no output" is exactly what an unprovisioned environment
        // produces and it must never read as a green.
        let r = TapReport::parse("[c2host] returned 0\n");
        assert!(!r.armed_ok());
        assert_eq!(r.total_hits(), 0);
        assert!(r.lines.is_empty());
    }
}

//! **PER-SUBSYSTEM METRICS** — the scoreboard decision 15 restructured the goal
//! onto, one 4-tuple per `docs/whitebox/ref/SUBSYS.md` §1 subsystem.
//!
//! Lane `w-submetric`, board **#3617**–**#3622**. Funded by
//! `docs/DECISIONS_2026-08-22.md` decision 15 (the owner: *"lets restructure our
//! goal so that we can get each submodule in shape and have measurements for
//! that. the overall TU goal is too broad because it is binary."*).
//!
//! # It is a GRADIENT and it obeys `FUNCTION_BYTE_MATCH.md` §0 verbatim
//!
//! All five properties, non-negotiable — §0 is the standing template for every
//! gradient added after FBM, and `decode-reach-*` and `symbind-*` adopted it
//! before this one did:
//!
//! * **Never in `scripts/gate.sh`'s verdict**, and it must never be added there.
//!   Nothing here reaches an accept/refuse path.
//! * **Its own block**, under its own disclaimer, apart from the class table
//!   that carries `match`/`mismatch`. It does not print inside a `c2rs gap`
//!   scan at all — it is a separate subcommand, so it cannot move the gate's
//!   21-row count table even by accident.
//! * **Namespaced keys** — `subsys-metric <key> <value>`. No existing key,
//!   predicate or denominator is narrowed, widened or redefined here.
//! * **It licenses no emit.** A subsystem row going green is not a reason to
//!   accept a shape, to widen the admitted set, or to admit anything. The only
//!   thing that accepts a shape is the differential — real `c2.dll` under wibo,
//!   `CLAUDE.md`'s one correctness rule.
//! * **Unrepresentable over an empty scan** — a strength with no data prints a
//!   **named residue**, never `0`, never silence, never a ratio over zero.
//!
//! # `#1406` placement, and why it is not in tension with the line above
//!
//! `#1406` binds any instrument whose output is quoted as evidence to run under
//! `cargo test` or `scripts/gate.sh`. §0 forbids the second. `decode-reach`
//! resolved this by putting the instrument's **logic and its controls** in
//! `crates/`, where `cargo test --workspace` runs them; this module does the
//! same. The verdict this contributes to is `cargo test`'s — that every
//! denominator in [`SUBSYSTEMS`] still reproduces from the tree — never the
//! differential's.
//!
//! # THE SIGNAL IS THE CHANGE IN EACH STRENGTH, NEVER ITS DISTANCE FROM 0
//!
//! Decision 15's own words. A row reading `read 16/93` is not "17 % done"; it
//! is a statement about the population the instrument can reach, on the tree
//! the denominator was taken on. Three traps ride with every number here and
//! they are printed with every render:
//!
//! 1. **The signal is the CHANGE, never the distance from 0 or 100.**
//! 2. **A green row is a statement about the population the instrument can
//!    reach** — every denominator says which tree and which enumeration it came
//!    from, because *the same subsystem has more than one defensible
//!    denominator and they differ by up to 3.8×* (§ [`Denominator`]).
//! 3. **These keys license no emit.**
//!
//! # The four strengths, and what each one actually is here
//!
//! | strength | this module's answer |
//! |---|---|
//! | **read** | a **containment**, never a ratio: `sites ⊇ read ⊇ ported`. `sites` is the subsystem's enumerable population (recomputed from `FUNCS.tsv` where it is a band), `read` is what the `P_*.md` page says it read, `ported` is **measured on `encode` and `section`** and a **named residue** on the other eight — see [`PortedRecount`] |
//! | **agreement** | the page's own **evidence-mark census** — `[O]` (obj-confirmed) against `[R]`+`[O]`+`[I]` — plus, where a page carries a real differential, that differential quoted with its own denominator. **A mark is a page annotation, not a site**; the caveat prints beside the number |
//! | **exercised** | a labelled **workload-output proxy** where one exists, from real-`c2` section census of the 878-TU workload; a named residue otherwise. **Per-SITE exercise is unmeasurable on this tree for all ten** — nothing traces `c2.dll`'s own addresses over the workload |
//! | **byte-owned** | **CITED, never re-measured.** Board **#3534** measured it 2026-08-25 at port tree `a8593651b`. Re-funding that read is what this repo calls *"check the board before dispatching"* |

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Where the whitebox reference index lives, relative to the repo root.
pub const REF_DIR: &str = "docs/whitebox/ref";

/// The encoder's arm enumeration, inside [`REF_DIR`] — 79 rows, one per distinct
/// arm target of the jump table at `0x10bfae2d`, each naming the encode-forms
/// that arm serves. Dumped from the pinned image by
/// `docs/whitebox/scripts/dump_opcode_tables.py --arms` (lane `w-read-r2`,
/// board `#3376`); **re-measured on this tree by lane `w-encmap`**, see
/// [`recount_encode_ported`].
pub const ENCODE_ARMS_TXT: &str = "ENCODE_ARMS.txt";

/// The real-`c2` section census of the workload, relative to the repo root.
/// Committed (lane `w-bss`); regenerating it needs the `dc3` tree and ~102 MB
/// of objs, which is why it is in the tree rather than rebuilt.
pub const SECTIONS_JSONL: &str = "work/w-bss/census/sections.jsonl";

/// The `.gl` **record dispatcher**'s arm enumeration, relative to the repo root
/// — 27 tag rows over 16 jump-table slots, decoded from the pinned image by
/// `work/w-secported/dump_glrec.py` (lane `w-secported`, board `#3661`).
///
/// Root-relative rather than inside [`REF_DIR`] for the same reason
/// [`SECTIONS_JSONL`] is: it is a lane's committed data artifact, and this
/// lane's fence does not include the reference index. One consequence is worth
/// stating rather than discovering — **`scripts/subsys_metrics.sh --self-test`
/// corrupts a copy of `REF_DIR` and therefore cannot reach this file**, so the
/// control that binds it is the `cargo test` one
/// (`control_a_fabricated_section_ported_is_caught`), not the shell self-test.
pub const GLREC_ARMS_TSV: &str = "work/w-secported/GLREC_ARMS.tsv";

/// The one subtree the [`PortedRecount::GlRecArms`] scan **must not read** — its
/// own crate.
///
/// **An instrument that can move its own number is not an instrument.**
/// `#3641` is the standing case: writing prose *about* mark letters on a
/// `P_*.md` page moved that page's agreement census, because the counter could
/// not tell a mark from a mention. The `ported` scan below has exactly that
/// shape — it counts source files naming an arm address, and this file must
/// name those addresses to explain itself. Excluding the metric crate is what
/// keeps the numerator a statement about the **port** rather than about its
/// own documentation, and
/// `the_observer_crate_cannot_move_its_own_ported` holds it.
pub const PORTED_SCAN_EXCLUDES_CRATE: &str = "c2-harness";

/// **BYTE-OWNED IS CITED AND NOT RE-MEASURED** — board `#3534`, lane
/// `w-permeasure`, 2026-08-25, port tree `a8593651b`, the 878-TU workload.
/// Decision 15 says so in its own words: *"Not re-measured this wave"*.
pub const BYTE_OWNED_CITATION: &str = concat!(
    "#3534 (w-permeasure, 2026-08-25, port tree a8593651b, 878-TU workload): ",
    "the port's wrong bodies are 1,968 bodies / 7,912 substituted words, ",
    "opcode 7,902 = 99.87 %, 0 pure reorderings, 92.78 % wrong at word 0. ",
    "docs/DIFF_STRUCTURE.md, docs/PERMUTER_POPULATION.md §3"
);

/// **THE BYTE-OWNED STRENGTH HAS NO PER-SUBSYSTEM SPLIT AND SAYING SO IS THE
/// POINT.** `#3534` measured the *shape* of the port's wrong bytes over the
/// whole workload; it did not attribute a single byte to `coff.c` rather than
/// `color.c`, and no instrument in this tree does. A per-subsystem
/// byte-ownership column would therefore be invented, so this module publishes
/// the one measurement that exists, once, with its citation — and a residue
/// naming what an attributed column would need.
pub const BYTE_OWNED_RESIDUE: &str = concat!(
    "no per-subsystem attribution of graded bytes exists: #3534 measured the ",
    "SHAPE of the port's wrong bytes workload-wide (99.87 % opcode ",
    "substitutions), not which subsystem authored them. Attributing a byte ",
    "would need the port's emit path to carry a subsystem tag through ",
    "codegen::select_function to the COFF writer; nothing does today"
);

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// Which endpoint convention reproduces a page's stated band denominator.
///
/// **The ten pages do not share one**, and this is measured rather than
/// assumed: `P_COFF`'s 120 and `P_SECTION`'s 137 reproduce only when the high
/// address is **inclusive**, `P_REGALLOC`'s 70 only when it is **exclusive**
/// (71 inclusive). Recording the convention per row is what makes the recount
/// reproducible instead of a coincidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// `lo <= addr <= hi`
    Inclusive,
    /// `lo <= addr < hi`
    HalfOpen,
}

/// One address band of a subsystem, in absolute VAs of
/// `compilers/X360/16.00.11886.00/c2.dll`.
#[derive(Clone, Copy, Debug)]
pub struct Band {
    pub lo: u32,
    pub hi: u32,
    pub end: End,
}

impl Band {
    fn holds(&self, a: u32) -> bool {
        match self.end {
            End::Inclusive => a >= self.lo && a <= self.hi,
            End::HalfOpen => a >= self.lo && a < self.hi,
        }
    }
}

/// How a subsystem's site denominator was obtained — and therefore whether this
/// module can **recount** it or can only **verify it is still on the page**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basis {
    /// Ghidra function entries inside [`Subsystem::bands`]. Recounted from
    /// `FUNCS.tsv` on every run and on every `cargo test`.
    Band,
    /// A call/callee set, not an address range — `P_SYMBOL`'s *"`FUN_10b28a9b`
    /// and its four callees"*, `P_GLOBREGS`'s *"the target plus its 18
    /// callees"*. **Not recountable from a band**; verified by requiring the
    /// page still carries the sentence it came from.
    CallSet,
    /// A closed site population read directly out of the image — `P_LABEL`'s
    /// 163 charging sites (31 direct + 132 constructor). Not a band; verified
    /// against the page's own words.
    SitePopulation,
}

/// How a row's `ported` number is **recomputed from the tree on every run**,
/// so that a carried number cannot rot and a fabricated one cannot pass.
///
/// This is `ported`'s analogue of [`Basis::Band`]: the band denominators
/// recount from `FUNCS.tsv`, and a `ported` cell with a recount recomputes from
/// the **port's own public tables**. A row with `None` here has no recount and
/// must therefore carry a [`Cell::Residue`] — [`verify`] enforces exactly that,
/// which is what stops the next lane from typing a number into a `ported` cell
/// and shipping it green.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortedRecount {
    /// **The encoder.** For each arm in `ENCODE_ARMS.txt`, ask `c2-core`'s own
    /// [`c2_core::codegen::mop`] whether the port can produce a word through
    /// that arm — see [`recount_encode_ported`] for the predicate, which is
    /// stated there rather than here because it is the load-bearing definition.
    EncodeArms,
    /// **The section model.** For each **live** arm of the `.gl` record
    /// dispatcher `0x10b9b8e9` in [`GLREC_ARMS_TSV`], ask whether any source
    /// file under `crates/` outside the metric crate names that arm's address —
    /// see [`recount_section_ported`], where the predicate and the reason it is
    /// sound **on this unit and not on the entry unit** are both stated.
    GlRecArms,
}

/// A measurable strength: a number with its denominator and where it came from,
/// a **named** residue, or a peer lane's pending work. There is no fourth
/// variant and in particular **there is no silent zero**.
#[derive(Clone, Debug)]
pub enum Cell {
    /// `num` of `den` `unit`, from `source`.
    Measured {
        num: u64,
        den: u64,
        unit: &'static str,
        source: &'static str,
        /// Printed beside the number, always. A proxy that is not the thing
        /// says so here.
        caveat: &'static str,
    },
    /// **A named residue.** Never empty — [`verify`] fails a blank one.
    Residue(&'static str),
    /// A peer lane is building this. Cited, never waited on.
    Pending(&'static str),
}

impl Cell {
    fn render(&self) -> String {
        match self {
            Cell::Measured {
                num, den, unit, ..
            } => {
                if *den == 0 {
                    "NO-RESULT (denominator 0)".to_string()
                } else {
                    format!(
                        "{} / {} {} ({:.2} %)",
                        commas(*num),
                        commas(*den),
                        unit,
                        100.0 * (*num as f64) / (*den as f64)
                    )
                }
            }
            Cell::Residue(r) => format!("RESIDUE — {r}"),
            Cell::Pending(p) => format!("PENDING — {p}"),
        }
    }

    fn note(&self) -> &'static str {
        match self {
            Cell::Measured { caveat, .. } => caveat,
            Cell::Residue(r) => r,
            Cell::Pending(p) => p,
        }
    }

    fn source(&self) -> &'static str {
        match self {
            Cell::Measured { source, .. } => source,
            _ => "—",
        }
    }
}

/// One `SUBSYS.md` §1 row, with every denominator carried as data beside the
/// document reference that states it — so staleness is visible rather than
/// inferred.
#[derive(Clone, Debug)]
pub struct Subsystem {
    /// Stable key for the `subsys-metric` lines.
    pub key: &'static str,
    pub title: &'static str,
    /// c2's own translation unit(s), from its C1001 path (Tier 1 — `strings`).
    pub tus: &'static str,
    /// The reference page, in [`REF_DIR`].
    pub page: &'static str,

    /// The address band(s), empty when [`Subsystem::basis`] is not [`Basis::Band`].
    pub bands: &'static [Band],
    /// How the site denominator was obtained.
    pub basis: Basis,
    /// **The site denominator.** Recounted from `FUNCS.tsv` when
    /// `basis == Band`; otherwise verified against [`Subsystem::den_probe`].
    pub sites: u64,
    pub sites_unit: &'static str,
    /// The doc reference that states the denominator — printed beside it.
    pub sites_doc: &'static str,
    /// A substring that **must still appear verbatim** in [`Subsystem::page`].
    /// This is what makes a stale number fail rather than rot quietly.
    pub den_probe: &'static str,

    /// What the page says it read, in the page's own unit.
    pub read: u64,
    pub read_unit: &'static str,
    pub read_doc: &'static str,

    /// The **other** defensible denominator: `FUNCS.tsv`'s `subsys` column, a
    /// TU-level attribution rather than a band. `None` where that column has no
    /// value for this subsystem (`globregs`, `encode`, `label` — `build_ref.py`
    /// has no `TU_PAGE`/`PAGE_SUBSYS` entry for their pages).
    pub tu_population: Option<u64>,

    /// Strength 1's third level.
    ///
    /// **Eight of ten rows are still a named residue** — see [`Cell::Residue`]
    /// and the module docs. The `encode` and `section` rows are measured, and both
    /// are measured by [`PortedRecount`] rather than carried.
    pub ported: Cell,
    /// When `Some`, [`verify`] **recomputes** `ported` from the tree and fails
    /// if the carried cell disagrees. When `None`, `ported` **must** be a
    /// residue or a pending — a bare number with nothing able to recount it is
    /// exactly the fabrication this scoreboard exists to make impossible.
    pub ported_recount: Option<PortedRecount>,
    /// Strength 2, beyond the mark census that every row gets.
    pub agreement_extra: Option<Cell>,
    /// Strength 3.
    pub exercised: Cell,

    /// Where `SUBSYS.md` §1's own `entries / band` cell disagrees with the
    /// page — in value or in **unit**. Empty when they agree.
    pub subsys_cell_note: &'static str,
}

// ---------------------------------------------------------------------------
// THE TABLE — ten rows, one per `SUBSYS.md` §1 row
// ---------------------------------------------------------------------------

/// Every number here was **re-measured on this tree** (lane `w-submetric`,
/// 2026-08-26, base `6c753ead0`) and is re-verified by [`verify`] on every
/// `cargo test`. The `*_doc` fields are the reference beside each number so a
/// reader can tell a carried figure from a computed one.
pub const SUBSYSTEMS: &[Subsystem] = &[
    Subsystem {
        key: "coff",
        title: "obj writer",
        tus: "coff.c (model), coffemit.c (every fwrite)",
        page: "P_COFF.md",
        bands: &[Band { lo: 0x10b281af, hi: 0x10b2b0dd, end: End::Inclusive }],
        basis: Basis::Band,
        sites: 120,
        sites_unit: "Ghidra function entries in the band",
        sites_doc: "P_COFF.md:16 \"21 of the 120 functions in the coff.c/coffemit.c band\"",
        den_probe: "21 of the 120 functions in the `coff.c`/`coffemit.c` band",
        read: 21,
        read_unit: "entries",
        read_doc: "P_COFF.md:16",
        tu_population: Some(129),
        ported: Cell::Residue(
            "no port<->image site map for the obj writer. crates/c2-obj writes COFF \
             by a route derived from the format, not from these 21 addresses; \
             counting which of them the port implements needs the derived-vs-fitted \
             provenance census, which lane w-provenance owns this wave",
        ),
        ported_recount: None,
        agreement_extra: None,
        exercised: Cell::Measured {
            num: 871,
            den: 871,
            unit: "workload TUs whose obj real c2 wrote",
            source: "work/w-bss/census/sections.jsonl (871 records)",
            caveat: "OUTPUT PROXY, NOT A SITE COUNT. Every obj in the workload went \
                     through this writer, so the proxy is 100 % by construction and \
                     carries no information about WHICH of the 120 functions ran. \
                     393,236 section headers over the 871",
        },
        subsys_cell_note: "",
    },
    Subsystem {
        key: "section",
        title: "section & symbol model",
        tus: "p2symtab.c, emit.cpp",
        page: "P_SECTION.md",
        bands: &[
            Band { lo: 0x10b97dfb, hi: 0x10b9b8e9, end: End::Inclusive },
            Band { lo: 0x10be71c9, hi: 0x10be7e81, end: End::Inclusive },
        ],
        basis: Basis::Band,
        sites: 137,
        sites_unit: "Ghidra function entries in the two bands (102 + 35)",
        sites_doc: "P_SECTION.md:11 \"24 entries against a denominator of 137\"",
        den_probe: "24 entries against a denominator of 137",
        read: 24,
        read_unit: "entries",
        read_doc: "P_SECTION.md:11",
        tu_population: Some(327),
        // RECOUNTED, never carried — `verify` recomputes this from
        // GLREC_ARMS.tsv plus a scan of the port's own sources on every run and
        // every `cargo test`. See `recount_section_ported` for the predicate and
        // for the measurement that makes a citation reading sound on THIS unit.
        ported: Cell::Measured {
            num: 1,
            den: 15,
            unit: "live .gl record-dispatcher arms the port has a decoder for",
            source: "lane w-secported, board #3661-#3666: work/w-secported/GLREC_ARMS.tsv \
                     (decoded from the pinned image on this tree) x a scan of crates/ \
                     outside c2-harness",
            caveat: "THE DENOMINATOR IS THE 15 LIVE ARMS OF THE .gl RECORD DISPATCHER \
                     0x10b9b8e9, AND `27 arms` -- which this row itself used to say and \
                     decision 17 repeats -- IS NOT AN ARM COUNT. Re-measured from the \
                     image: 27 TAG VALUES (0x01..0x1B) index a 27-entry byte table into \
                     16 jump slots, and ONE slot is the fatal C1001 path 0x10b9c5ca \
                     serving EIGHT tags (0x0C 0x0F 0x11 0x13 0x14 0x15 0x16 0x17). So \
                     the population is 15 live arms over 19 live tags plus one refusal \
                     over 8, and calling it 27 overstates the arm count by 1.8x. FOUR \
                     RIVAL DENOMINATORS WERE MEASURED AND ARE PUBLISHED RATHER THAN ONE \
                     CHOSEN SILENTLY: 16 arms counting the refusal as a site (a port \
                     that also refuses agrees with c2 by doing nothing, so this is \
                     rejected); 27 tags, on which the port names 5 (0x01 0x02 0x04 0x0E \
                     0x10, all in c2_il::func::glalias, and three of them as pattern \
                     LOCATORS rather than decoders); the page's 24/25 read ENTRIES, on \
                     which an address grep gives 2 and is WRONG -- see \
                     recount_section_ported for the two rules the port implements while \
                     citing nothing; and the band's 137 / the TU-level 327, neither of \
                     which the port maps onto at all. CONTAINMENT: the 15 arms all live \
                     INSIDE 0x10b9b8e9, which is one of the 24 read entries, which is \
                     one of the 137 sites -- so `sites superset-of read superset-of \
                     ported` holds as a containment of SITE SETS, while the three counts \
                     are in three granularities and their RATIOS must not be compared. \
                     THE ONE ARM IS 0x10b9bdcf, the shared tag-0x04/0x0E/0x10 handler, \
                     decoded by c2_il::func::glalias under DISCLOSURE W-ALIAS-1. The 14 \
                     unported arms include 0x10b9c212 -- TAG 0x09, THE SECTION \
                     DEFINITION RECORD, every field of which P_SECTION marks obj-checked \
                     by .gl mutation. The port does not read it: it carries 17 \
                     fully-resolved (name, Characteristics) constants where c2 has a \
                     kind switch, a remapper, a base resolver and an alignment chooser. \
                     See P_SECTION.md section 7",
        },
        ported_recount: Some(PortedRecount::GlRecArms),
        agreement_extra: None,
        exercised: Cell::Measured {
            num: 14,
            den: 14,
            unit: "distinct section names real c2 emits over the workload",
            source: "work/w-bss/census/sections.jsonl (393,236 sections, 871 TUs)",
            caveat: "OUTPUT PROXY, NOT A SITE COUNT — and its denominator is the \
                     observed set, so it is 14/14 by construction. The names: \
                     .drectve .debug$S .XBLD$W:C1 .XBLD$W:C2 .text .text$yc .text$yd \
                     .pdata .xdata$x .rdata .rdata$r .data .bss .CRT$XCU",
        },
        subsys_cell_note: "THE PAGE'S OWN COVERAGE LINE DOES NOT REPRODUCE, AND NEVER \
                           DID. `read 24` is carried verbatim from P_SECTION.md:11 \
                           (`24 entries against a denominator of 137`) and recounting \
                           the page's §1 table on this tree gives 25 ROWS, of which 22 \
                           are Ghidra function entries (three -- 0x10b9bdcf, 0x10b9c212, \
                           0x10b9c5ca -- are addresses INSIDE 0x10b9b8e9 and the page \
                           says so), of which 20 are inside the two bands that give the \
                           137 (0x10b805b3 is misc.c, 0x10c27b56 is smdmisc.c). 24 \
                           reproduces under NONE of the three, and `git log -S` puts \
                           the line at 25 rows in the file's FIRST commit -- wrong when \
                           written, not rotted since, the same family as #3643. NOT \
                           CORRECTED HERE: the line is this row's den_probe and the \
                           standing convention is that an amendment stands beside the \
                           original reading (P_SECTION §5's own retraction is the \
                           model); it is flagged on the page at §7.5 instead. `ported` \
                           is in a THIRD unit again -- 15 live dispatcher arms -- and \
                           the containment holds as a nesting of SITE SETS, not as \
                           comparable ratios",
    },
    Subsystem {
        key: "regalloc",
        title: "register allocator",
        tus: "color.c (+ globregs.c, regasg.c)",
        page: "P_REGALLOC.md",
        bands: &[Band { lo: 0x10b2c21d, hi: 0x10b3219f, end: End::HalfOpen }],
        basis: Basis::Band,
        sites: 70,
        sites_unit: "Ghidra function entries in color.c's span",
        sites_doc: "P_REGALLOC.md:25 \"18 code entries + 15 data entries against a denominator of 70\"",
        den_probe: "18 code entries + 15 data entries against a denominator of 70",
        read: 33,
        read_unit: "entries (18 code + 15 data)",
        read_doc: "P_REGALLOC.md:25",
        tu_population: Some(230),
        ported: Cell::Residue(
            "the port has no register allocator of this shape at all — the byte-exact \
             classes are one-function bodies whose registers are assigned by \
             codegen::select_function's own rules, not by a colouring pass. A \
             site-level numerator is not merely unmeasured, it is not yet defined",
        ),
        ported_recount: None,
        agreement_extra: None,
        exercised: Cell::Residue(
            "per-site exercise is unmeasurable: nothing traces c2.dll's own addresses \
             over the workload. The nearest measured thing is P_REGALLOC's own [O] \
             evidence on 6 frozen grid cells (G1-G4, L3, P1), which is a 6-cell probe \
             grid and not the 878-TU workload",
        ),
        subsys_cell_note: "SUBSYS.md §1 prints `33 / 70`; the page's 33 is 18 code + 15 \
                           data entries, and the 15 data entries are TABLES, not \
                           functions, so the numerator and the denominator are in \
                           different units. Read as entries-against-functions, not as \
                           a fraction. Also: the band reproduces 70 only HALF-OPEN \
                           (71 inclusive) — 0x10b3219f is dag.c's anchor",
    },
    Subsystem {
        key: "globregs",
        title: "globregs: the candidate SET, its ORDER, and the tie key",
        tus: "globregs.c (+ the symbol-table arena in p2symtab.c)",
        page: "P_GLOBREGS.md",
        bands: &[],
        basis: Basis::CallSet,
        sites: 19,
        sites_unit: "the R4 target plus its 18 callees",
        sites_doc: "P_GLOBREGS.md:20-21 \"The denominator R4 registered was *the target plus its 18 callees = 19*\"",
        den_probe: "the target plus its 18 callees = 19",
        read: 26,
        read_unit: "entries (16 code + 10 data)",
        read_doc: "P_GLOBREGS.md:20",
        tu_population: None,
        ported: Cell::Residue(
            "the port does no global register promotion; there is no site to count. \
             P_GLOBREGS §2's order and tie key are read but unadopted",
        ),
        ported_recount: None,
        agreement_extra: None,
        exercised: Cell::Residue(
            "per-site exercise unmeasurable (no address trace). P_GLOBREGS's own [O] \
             is 262 formal->register assignments over 62 GRID objs — a probe grid, \
             not the 878-TU workload",
        ),
        subsys_cell_note: "THE READ IS LARGER THAN ITS OWN DENOMINATOR (26 against 19) \
                           and the page says why in its own words: the read went \
                           OUTSIDE the registered denominator on purpose, because the \
                           three functions that decide the order are not callees of \
                           the target at all. The page's honest statement is `6 of 18 \
                           callees read to policy level, plus 7 functions outside the \
                           target's subtree`. SUBSYS.md §1's cell `16 code + 10 data` \
                           prints no denominator at all",
    },
    Subsystem {
        key: "dag",
        title: "DAG build + scheduler",
        tus: "dag.c, and an unnamed TU with no ICE site",
        page: "P_DAG.md",
        bands: &[
            Band { lo: 0x10b3219f, hi: 0x10b3433f, end: End::HalfOpen },
            Band { lo: 0x10be5cce, hi: 0x10be663f, end: End::HalfOpen },
        ],
        basis: Basis::Band,
        sites: 61,
        sites_unit: "Ghidra function entries in the two bands (48 + 13)",
        sites_doc: "P_DAG.md:9-10 \"24 code entries + 8 data/table entries against a denominator of 61\"",
        den_probe: "24 code entries + 8 data/table entries against a denominator of",
        read: 32,
        read_unit: "entries (24 code + 8 data/table)",
        read_doc: "P_DAG.md:9",
        tu_population: Some(83),
        ported: Cell::Residue(
            "the port schedules nothing — emission order is tuple-list order \
             (P_BLOCKORDER §5.2, #3437-#3441) and the port's bodies are built \
             straight-line. No site-level numerator is defined",
        ),
        ported_recount: None,
        agreement_extra: None,
        exercised: Cell::Residue(
            "per-site exercise unmeasurable (no address trace). The scheduler band \
             0x10be5cce-0x10be663f is a TU with NO ICE SITE, so even its attribution \
             is a hypothesis rather than a fact (SUBSYS.md's own blind-spot box)",
        ),
        subsys_cell_note: "",
    },
    Subsystem {
        key: "inline",
        title: "inliner",
        tus: "inline.c",
        page: "P_INLINE.md",
        bands: &[Band { lo: 0x10b5b86d, hi: 0x10b62b00, end: End::Inclusive }],
        basis: Basis::Band,
        sites: 93,
        sites_unit: "Ghidra function entries in the inliner band",
        sites_doc: "P_INLINE.md:9 \"16 entries against a denominator of 93\"",
        den_probe: "16 entries against a denominator of 93",
        read: 16,
        read_unit: "entries",
        read_doc: "P_INLINE.md:9",
        tu_population: Some(350),
        ported: Cell::Residue(
            "the port carries a FITTED inline predicate (INLINE_PREDICATE.md's 0.9716 \
             model), not an implementation of these 93 sites. The clause-by-clause \
             port-state column is lane w-inlmetric's deliverable this wave and is not \
             built here",
        ),
        ported_recount: None,
        agreement_extra: Some(Cell::Pending(
            "the inliner's clause-by-clause differential is being built by lane \
             w-inlmetric (decision 15, boards #3623-#3628), in flight at this \
             render. Cited, not waited on, and its worktree is not read",
        )),
        exercised: Cell::Residue(
            "per-site exercise unmeasurable (no address trace). P_INLINE's own worked \
             case is one anchor (keygen_xbox.cpp) where the read predicts six inlines \
             and gets one [O] — a single TU, not a workload count",
        ),
        subsys_cell_note: "",
    },
    Subsystem {
        key: "encode",
        title: "instruction encoder (tuple -> one PPC word, plus .text relocation requests)",
        tus: "code.c",
        page: "P_ENCODE.md",
        bands: &[Band { lo: 0x10bf96d0, hi: 0x10bfae2a, end: End::Inclusive }],
        basis: Basis::Band,
        sites: 14,
        sites_unit: "Ghidra function entries in the encoder band",
        sites_doc: "SUBSYS.md §1 cell `14 / 14`; recounted from FUNCS.tsv on this tree",
        den_probe: "79 of the 79 distinct arms read",
        read: 79,
        read_unit: "distinct encode arms (covering 660 of 660 machine opcodes)",
        read_doc: "P_ENCODE.md:27",
        tu_population: None,
        // RECOUNTED, never carried — `verify` recomputes this from
        // ENCODE_ARMS.txt plus c2-core's own public tables on every run and
        // every `cargo test`. See `recount_encode_ported` for the predicate.
        ported: Cell::Measured {
            num: 30,
            den: 79,
            unit: "encode arms the port can produce a word through",
            source: "lane w-encmap, board #3636-#3641 (27/79); lane w-encarms, board \
                     #3756-#3761 (29/79, wave 18); lane w-fmadd, board #3790-#3795 \
                     (30/79, wave 19): ENCODE_ARMS.txt (79 rows, \
                     re-measured on this tree) x c2_core::codegen::mop::{plan, OPCODES}",
            caveat: "THE DENOMINATOR IS THE 79 ARMS, NOT THE BAND'S 14 AND NOT THE 111 \
                     JUMP-TABLE ENTRIES, and the choice is published rather than \
                     silent: `read` on this row is already 79 arms, so `read \
                     superset-of ported` is well formed only in the arm unit. The 111 \
                     entries are 111 FORMS, each belonging to exactly one arm \
                     (re-measured: 111 -> 79, no form served by two arms), so the \
                     entry unit would count the same arm up to 12 times. The band's \
                     14 is Ghidra function entries -- a different population \
                     entirely. AN ARM COUNTS ON ONE OF ITS FORMS, so this OVER-states \
                     partial arms: the strict reading (every form of the arm \
                     reachable) is 27, and the loose reading (a FieldPlan exists, \
                     whether or not an opcode reaches it) is 31 -- the extra arm is \
                     10bfa26c, form 2, a plan no opcode reaches. BOTH MOVED BY ONE ON \
                     2026-08-29 and both were re-derived, not incremented: arm \
                     10bfa49a serves form 24 ALONE, so an arm that counts published \
                     also counts strict, which is why w-fmadd's adoption is the first \
                     that moves all three readings together. \
                     27 -> 29 ON 2026-08-28, lane w-encarms: arms 10bfa285 (form 7, \
                     `bl`) and 10bfa76a (form 54, `mfspr`) were read at their \
                     addresses in the pinned image and adopted, discharging all three \
                     of codegen::word_seam's armed refusals; DISCLOSURE W-ENCARMS-1. \
                     THE SAME LANE REFUTED THE `25` THIS CAVEAT USED TO CARRY FOR THE \
                     STRICT READING: measured directly at master 4b79bf46a it is 24, \
                     so the strict reading moved 24 -> 26 and the published 25 never \
                     reproduced (board #3759). The 50 still-unmapped arms are NOT \
                     uniform, and neither is what the WORKLOAD reaches: over 861 \
                     real-c2 objs of the 878-TU workload (3,192,747 non-zero \
                     executable .text words) only 10 of the 52 pre-adoption unmapped \
                     arms are reached unambiguously and 31 are reached ZERO times, \
                     including the ICE arm 10bfa81d and 30 others; the 104-opcode \
                     default arm 10bf9f91 is reached by at most 10 words, 0.0003 %. \
                     work/w-encarms/armhist.py. \
                     29 -> 30 ON 2026-08-29, lane w-fmadd: arm 10bfa49a (form 24, the \
                     FUSED multiply-adds) was read at its address and adopted, and \
                     unlike w-encarms's two this one had NO field plan and NO opcode \
                     row before the lane -- the port could not compose the word at \
                     all. It is the fourth-most-reached unmapped arm on the same \
                     861-obj histogram (7,995 words, unambiguous), and the adoption \
                     is an EMIT and not a fold: `a*b+c` was a refusal and is now \
                     byte-exact (fixtures/cpp/w13c_fma.cpp, 14 functions). \
                     DISCLOSURE W-FMADD-1. See P_ENCODE.md section 10",
        },
        ported_recount: Some(PortedRecount::EncodeArms),
        agreement_extra: Some(Cell::Measured {
            num: 630_548,
            den: 634_457,
            unit: "executable .text words explained by the page's own arm masks",
            source: "P_ENCODE.md §8.2 [O], 500 dc3-decomp reference objs",
            caveat: "THE STRICT-MASK PASS IS THE ONE WITH TEETH — the page says so \
                     itself: a second pass with every read form masked reads 99.8060 % \
                     and MUST NOT be quoted as stronger, because sixteen VMX128 forms \
                     are masked at 0x03FFFFFF and a generous mask cannot fail. \
                     Denominator is 500 objs, NOT the 878-TU workload. The 3,909 \
                     residuals are unmasked forms, not disagreements; 0 unexplained \
                     at any of 124,700 relocation sites",
        }),
        exercised: Cell::Measured {
            num: 863,
            den: 871,
            unit: "workload TUs with any .text section",
            source: "work/w-bss/census/sections.jsonl",
            caveat: "OUTPUT PROXY, NOT A SITE COUNT — 178,104 .text COMDATs over the \
                     863. Says nothing about which of the 79 arms the workload takes",
        },
        subsys_cell_note: "SUBSYS.md §1 prints `14 / 14`, which is the BAND (14 Ghidra \
                           entries, recounted here and correct). The page's own \
                           coverage line is `79 of the 79 distinct arms`, covering \
                           `660 of 660` opcodes. THE TWO CELLS ARE IN DIFFERENT UNITS \
                           and neither is wrong; a reader taking `14 / 14` for the \
                           coverage statement is off by a factor of 5.6 in the \
                           numerator and 47 in the opcode denominator",
    },
    Subsystem {
        key: "eh",
        title: "EH state synthesis",
        tus: "ehexcept.c, except.c (+ the .pdata drivers)",
        page: "P_EH.md",
        bands: &[Band { lo: 0x10be04e7, hi: 0x10be3800, end: End::Inclusive }],
        basis: Basis::Band,
        sites: 47,
        sites_unit: "Ghidra function entries in the EH band",
        sites_doc: "P_EH.md:9 \"19 entries against a denominator of 47\"",
        den_probe: "19 entries against a denominator of 47",
        read: 19,
        read_unit: "entries",
        read_doc: "P_EH.md:9",
        tu_population: Some(127),
        ported: Cell::Residue(
            "P_EH marks two entries `[O] port` — the port reproduces the deferred \
             unwind-word pass's OUTPUT — but the page's marks are per-claim, not \
             per-site, so they do not compose into a `sites implemented` numerator. \
             Building one is the same missing port<->image map as every other row",
        ),
        ported_recount: None,
        agreement_extra: None,
        exercised: Cell::Measured {
            num: 849,
            den: 871,
            unit: "workload TUs carrying .pdata",
            source: "work/w-bss/census/sections.jsonl",
            caveat: "OUTPUT PROXY, NOT A SITE COUNT — 103,128 .pdata records over the \
                     849. Its value is the INDEPENDENT CORROBORATION beside it: this \
                     census counts .xdata$x in exactly 67 of 871 TUs, reproducing \
                     P_EH's own `67 workload objs, all STLport` from a different \
                     instrument",
        },
        subsys_cell_note: "",
    },
    Subsystem {
        key: "label",
        title: "compiler-label numbering (the $M/$T/$L* counter and its charges)",
        tus: "p2symtab.c (allocator + ctor), vlines.c (the $M minter), plus 21 more files",
        page: "P_LABEL.md",
        bands: &[],
        basis: Basis::SitePopulation,
        sites: 163,
        sites_unit: "charging sites (31 direct calls of the allocator + 132 of the generic ctor)",
        sites_doc: "P_LABEL.md:0 \"its 31 direct call sites are the entire population of charges, plus the 132 sites\"",
        den_probe: "direct call sites are the entire population of charges",
        read: 163,
        read_unit: "sites (the population is CLOSED by construction — the allocator's address is never taken)",
        read_doc: "P_LABEL.md §0",
        tu_population: None,
        ported: Cell::Residue(
            "the port mints labels from its own counter; no mapping exists from its \
             mint points to these 163 charging sites. LABEL_COUNTER.md's own finding \
             is that stride == minted fails both ways, so a naive site count would be \
             wrong even if it were built",
        ),
        ported_recount: None,
        agreement_extra: None,
        exercised: Cell::Residue(
            "per-site exercise unmeasurable, and WORSE HERE THAN ELSEWHERE: 42 of the \
             163 sites sit on LOOP BACK EDGES, so a TU's charge is a data-dependent \
             sum over whatever population the loop walks, not a per-construct \
             constant. A site-hit count would not be a charge count even if we had \
             one (P_LABEL §0; LABEL_SEED_GAP is not a constant either)",
        ),
        subsys_cell_note: "SUBSYS.md §1's cell reads `163 sites / 86+25 callers`. The 86 \
                           reproduces on the page (`All 132 are direct E8 calls from 86 \
                           distinct functions`, P_LABEL:445/471). THE `25` DOES NOT \
                           REPRODUCE ANYWHERE ON THE PAGE — the nearest figure is 85, \
                           the PLACEMENT population that calls FUN_10bd415e (P_LABEL:505), \
                           and the nearest literal 25 on the page is `fitted from 25 TUs` \
                           in an unrelated sentence at :222. Reported, not corrected: \
                           P_LABEL/SUBSYS.md are not this lane's to edit",
    },
    Subsystem {
        key: "symbol",
        title: "symbol records: storage class, section number, WEAK EXTERNALS",
        tus: "coff.c (FUN_10b28a9b) + coffemit.c's three appenders",
        page: "P_SYMBOL.md",
        bands: &[],
        basis: Basis::CallSet,
        sites: 5,
        sites_unit: "functions (FUN_10b28a9b and its four callees)",
        sites_doc: "P_SYMBOL.md:25 \"27 addresses in FUN_10b28a9b and its four callees\"",
        den_probe: "27 addresses in `FUN_10b28a9b` and its four callees",
        read: 27,
        read_unit: "addresses",
        read_doc: "P_SYMBOL.md:25",
        tu_population: Some(5),
        ported: Cell::Residue(
            "P_SYMBOL §2 marks several addresses `[O]` via the port's own \
             ObjImage::weak_externals with KNOWN-ANSWER 0 alarms, so parts of this \
             subsystem ARE implemented and graded — but per-ADDRESS, and the page's \
             27 addresses do not map onto port functions one-for-one. The numerator \
             is undefined rather than zero",
        ),
        ported_recount: None,
        agreement_extra: None,
        exercised: Cell::Measured {
            num: 675,
            den: 871,
            unit: "workload TUs needing a weak external",
            source: "CITED from the existing gap key `alias-weak-needed-tus` \
                     (SUBSYS.md §4 row 5); NOT recomputed here",
            caveat: "OUTPUT PROXY, NOT A SITE COUNT, and it is CITED from another \
                     instrument's key rather than measured by this one — one fact, \
                     one locator (docs/GAPS.md §6). It counts TUs that NEED a weak \
                     external, not sites of the record writer that ran",
        },
        subsys_cell_note: "SUBSYS.md §1 prints `27 / 5`, a ratio greater than 1: the \
                           numerator is ADDRESSES and the denominator is FUNCTIONS. \
                           Recounted here, the page's own address band \
                           0x10b28a9b-0x10b28d6f holds exactly ONE Ghidra function \
                           entry, so there is no band reading under which `5` is a \
                           function count of that span — the 5 is FUN_10b28a9b plus \
                           four callees that live elsewhere in coff.c's gap",
    },
];

// ---------------------------------------------------------------------------
// Reading the tree
// ---------------------------------------------------------------------------

/// The evidence-mark census of one reference page: `[R]` read, `[O]`
/// obj-confirmed, `[I]` inferred, as defined by `docs/whitebox/ref/README.md`
/// §2.
///
/// **A MARK IS A PAGE ANNOTATION, NOT A SITE.** A page may mark one sentence
/// `[O]` and cover twenty addresses with it, or mark the same fact twice. This
/// is a census of the page's own claims about its own evidence tier, and it is
/// published as the `agreement` strength because it is the only quantity that
/// is (a) uniform across all ten pages and (b) mechanically recomputable. It is
/// not a differential and it must never be quoted as one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    pub read: u64,
    pub obj: u64,
    pub inferred: u64,
}

impl Marks {
    pub fn total(&self) -> u64 {
        self.read + self.obj + self.inferred
    }
}

/// Count evidence marks in a page body.
///
/// **The rule, stated so it is reproducible**: everything up to and including
/// the page's first line consisting of exactly `---` is the provenance banner
/// and mark legend, and is skipped; every occurrence of `[R]`, `[O]`, `[I]`
/// after it counts. Every one of the ten pages has such a line.
pub fn count_marks(page: &str) -> Option<Marks> {
    let mut lines = page.lines();
    let mut found = false;
    for l in lines.by_ref() {
        if l.trim_end() == "---" {
            found = true;
            break;
        }
    }
    if !found {
        return None;
    }
    let body: String = lines.collect::<Vec<_>>().join("\n");
    Some(Marks {
        read: occurrences(&body, "[R]"),
        obj: occurrences(&body, "[O]"),
        inferred: occurrences(&body, "[I]"),
    })
}

fn occurrences(hay: &str, needle: &str) -> u64 {
    hay.matches(needle).count() as u64
}

/// One function row of `FUNCS.tsv`, reduced to what this module needs.
fn funcs_addresses(tsv: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for (i, line) in tsv.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let first = line.split('\t').next().unwrap_or("");
        if i == 0 || first == "addr" {
            continue;
        }
        if let Ok(a) = u32::from_str_radix(first.trim_start_matches("0x"), 16) {
            out.push(a);
        }
    }
    out
}

/// Recount a subsystem's band denominator from `FUNCS.tsv`.
pub fn recount_band(addrs: &[u32], bands: &[Band]) -> u64 {
    addrs
        .iter()
        .filter(|a| bands.iter().any(|b| b.holds(**a)))
        .count() as u64
}

/// The workload-side stamp, read out of the section census's `.prov` sidecar.
#[derive(Clone, Debug, Default)]
pub struct WorkloadStamp {
    pub records: Option<u64>,
    pub generated_utc: Option<String>,
    pub corpus_head: Option<String>,
    pub corpus_dirty: Option<bool>,
    pub data_sha256: Option<String>,
    pub present: bool,
}

fn json_str(src: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\"");
    let i = src.find(&pat)? + pat.len();
    let rest = &src[i..];
    let c = rest.find(':')? + 1;
    let rest = rest[c..].trim_start();
    if let Some(r) = rest.strip_prefix('"') {
        let e = r.find('"')?;
        Some(r[..e].to_string())
    } else {
        let e = rest
            .find(|c: char| c == ',' || c == '\n' || c == '}')
            .unwrap_or(rest.len());
        Some(rest[..e].trim().to_string())
    }
}

/// Read the workload stamp. Absent file ⇒ `present: false`, never a panic.
pub fn workload_stamp(root: &Path) -> WorkloadStamp {
    let p = root.join(format!("{SECTIONS_JSONL}.prov"));
    let Ok(s) = std::fs::read_to_string(&p) else {
        return WorkloadStamp::default();
    };
    WorkloadStamp {
        records: json_str(&s, "data_records").and_then(|v| v.parse().ok()),
        generated_utc: json_str(&s, "generated_utc"),
        corpus_head: json_str(&s, "head"),
        corpus_dirty: json_str(&s, "dirty").map(|v| v == "true"),
        data_sha256: json_str(&s, "data_sha256"),
        present: true,
    }
}

// ---------------------------------------------------------------------------
// Verification — the part `cargo test` runs, and the part a fabrication reddens
// ---------------------------------------------------------------------------

/// Everything [`verify`] checked, whether or not it held.
#[derive(Clone, Debug)]
pub struct Verified {
    pub marks: BTreeMap<&'static str, Marks>,
    pub recounted: BTreeMap<&'static str, u64>,
    /// `key -> (ported, denominator)` for every row [`PortedRecount`] could
    /// recompute. A row absent here has no recount and carries a residue.
    pub ported_recounted: BTreeMap<&'static str, (u64, u64)>,
    pub failures: Vec<String>,
}

impl Verified {
    pub fn ok(&self) -> bool {
        self.failures.is_empty()
    }
}

// ---------------------------------------------------------------------------
// `ported`, for the one row where the question is well formed
// ---------------------------------------------------------------------------

/// One row of `ENCODE_ARMS.txt`: an arm address and the encode-forms it serves.
#[derive(Clone, Debug)]
pub struct EncodeArm {
    pub arm: String,
    /// c2's own opcode count for the arm, as the dump computed it. Carried for
    /// the caveat text, never used in the ratio.
    pub opcodes: u64,
    pub forms: Vec<u16>,
}

/// Parse `ENCODE_ARMS.txt`. `#` comments and blank lines are skipped; a row is
/// `<arm> <nforms> <nopcodes> <comma-separated forms>`.
pub fn parse_encode_arms(txt: &str) -> Vec<EncodeArm> {
    let mut out = Vec::new();
    for line in txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        let (arm, nforms, nops, forms) = match (it.next(), it.next(), it.next(), it.next()) {
            (Some(a), Some(nf), Some(no), Some(f)) => (a, nf, no, f),
            _ => continue,
        };
        let forms: Vec<u16> = forms.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        // A row whose declared form count disagrees with its own list is a
        // corrupt dump, not a zero — drop it and let the arm-count check below
        // notice the shortfall.
        if forms.len() != nforms.parse::<usize>().unwrap_or(usize::MAX) {
            continue;
        }
        out.push(EncodeArm {
            arm: arm.to_string(),
            opcodes: nops.parse().unwrap_or(0),
            forms,
        });
    }
    out
}

/// **`ported` for the `encode` row: `(arms the port can produce a word through,
/// arms enumerated)`.**
///
/// # The predicate, stated once, because the number means nothing without it
///
/// An arm counts as **ported** iff there is some encode-form `f` that arm serves
/// for which **both** hold:
///
/// 1. `c2_core::codegen::mop::plan(f)` is `Some` — the port has transcribed
///    that arm's **field placement**; and
/// 2. some row of `c2_core::codegen::mop::OPCODES` carries form `f` — the port
///    actually has **an instruction that goes through it**.
///
/// **Both halves are required, and requiring the second is the conservative
/// choice.** A `FieldPlan` no opcode reaches composes nothing; counting it would
/// inflate `ported` by rules the port cannot execute. On this tree that is worth
/// exactly one arm (`10bfa26c`, form 2, which `plan` answers and no `OPCODES`
/// row names), and the looser reading is published beside the number rather than
/// instead of it.
///
/// # It is a LOWER bound on understanding and an UPPER bound on nothing
///
/// The rule grants an arm on **one** of its forms, so an arm the port serves
/// partially still counts. The strict variant — every form of the arm planned —
/// is [`recount_encode_ported_strict`] and is printed in the caveat.
///
/// # Why this is live rather than transcribed
///
/// Both inputs are `c2-core`'s **public** tables, read through the crate rather
/// than copied into this one. Nothing here transcribes a whitebox value into
/// `crates/`: the denominator is a file read (exactly as the band denominators
/// read `FUNCS.tsv`) and the numerator is a query against the port. A lane that
/// adds an opcode or a form plan moves this number without touching this file,
/// and a lane that types a wrong number into the table reddens `cargo test`.
pub fn recount_encode_ported(arms: &[EncodeArm]) -> (u64, u64) {
    let n = arms
        .iter()
        .filter(|a| a.forms.iter().any(|&f| port_reaches_form(f)))
        .count() as u64;
    (n, arms.len() as u64)
}

/// The strict variant: **every** form the arm serves is reachable by the port.
pub fn recount_encode_ported_strict(arms: &[EncodeArm]) -> u64 {
    arms.iter()
        .filter(|a| !a.forms.is_empty() && a.forms.iter().all(|&f| port_reaches_form(f)))
        .count() as u64
}

/// The looser variant: the port has a `FieldPlan` for the form, whether or not
/// any opcode reaches it. Published beside the number, never as it.
pub fn recount_encode_ported_planned(arms: &[EncodeArm]) -> u64 {
    arms.iter()
        .filter(|a| {
            a.forms
                .iter()
                .any(|&f| c2_core::codegen::mop::plan(c2_core::codegen::mop::Form(f)).is_some())
        })
        .count() as u64
}

/// Can the port produce a word whose placement is this form's arm?
fn port_reaches_form(f: u16) -> bool {
    use c2_core::codegen::mop::{plan, Form, OPCODES};
    plan(Form(f)).is_some() && OPCODES.iter().any(|r| r.form.0 == f)
}

// ---------------------------------------------------------------------------
// `ported` for the `section` row — the `.gl` record dispatcher's arms
// ---------------------------------------------------------------------------

/// One row of [`GLREC_ARMS_TSV`]: a record tag, the jump-table slot its
/// byte-index entry selects, and the arm that slot holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlRecArm {
    pub tag: u8,
    pub slot: u8,
    pub arm: u32,
}

/// Everything [`parse_glrec_arms`] recovers, including the **fatal** arm, which
/// the file names on its own `# fatal` line so a consumer never has to infer
/// which of the sixteen slots is the refusal.
#[derive(Clone, Debug, Default)]
pub struct GlRecTables {
    pub rows: Vec<GlRecArm>,
    pub fatal: Option<u32>,
}

impl GlRecTables {
    /// The **live** arm targets — distinct, fatal excluded, in first-tag order.
    ///
    /// This is the denominator, and excluding the fatal arm is a choice with a
    /// reason: `0x10b9c5ca` is `mov edx,0x7ba; jmp` — a `C1001` on
    /// `p2symtab.c:1978`, confirmed live by a one-byte desync. It is c2
    /// **refusing** eight tag values, and a port that also refuses them agrees
    /// with c2 by doing nothing. Counting a refusal as an implemented site is
    /// how a `ported` number would drift upward without a line of work, and
    /// this repo's scoring rule (`docs/PROGRESS_METRIC.md`) runs the other way.
    /// The 16-slot reading is published beside the 15 rather than instead.
    pub fn live_arms(&self) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for r in &self.rows {
            if Some(r.arm) != self.fatal && !out.contains(&r.arm) {
                out.push(r.arm);
            }
        }
        out
    }

    /// The tag values that reach a live arm — 19 of the 27 on the pinned image.
    pub fn live_tags(&self) -> Vec<u8> {
        self.rows
            .iter()
            .filter(|r| Some(r.arm) != self.fatal)
            .map(|r| r.tag)
            .collect()
    }
}

/// Parse [`GLREC_ARMS_TSV`]. `# fatal <hex>` names the refusal arm; every other
/// `#` line is a comment; the body is `<tag>\t<slot>\t<arm>`, all hex but the
/// slot.
pub fn parse_glrec_arms(txt: &str) -> GlRecTables {
    let mut t = GlRecTables::default();
    for line in txt.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("# fatal ") {
            t.fatal = parse_hex(rest.trim());
            continue;
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with("tag\t") {
            continue;
        }
        let mut it = line.split('\t');
        let (tag, slot, arm) = match (it.next(), it.next(), it.next()) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => continue,
        };
        match (parse_hex(tag.trim()), slot.trim().parse::<u8>(), parse_hex(arm.trim())) {
            (Some(tag), Ok(slot), Some(arm)) if tag <= 0xFF => {
                t.rows.push(GlRecArm { tag: tag as u8, slot, arm })
            }
            // A malformed row is dropped rather than defaulted; the row-count
            // check in `verify` then notices the shortfall, exactly as
            // `parse_encode_arms` does.
            _ => continue,
        }
    }
    t
}

fn parse_hex(s: &str) -> Option<u32> {
    u32::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}

/// **`ported` for the `section` row: `(live `.gl` record arms the port has a
/// decoder for, live arms enumerated)`.**
///
/// # The predicate, stated once, because the number means nothing without it
///
/// A live arm counts as **ported** iff its address appears in some `.rs` file
/// under `<root>/crates/` that is neither in the metric crate
/// ([`PORTED_SCAN_EXCLUDES_CRATE`]) nor a test file.
///
/// # This is a CITATION predicate, and here is the measurement that makes it
/// sound
///
/// A citation predicate normally measures **documentation discipline**, not
/// implementation — and on the section subsystem's *entry* unit it demonstrably
/// does. Two of `P_SECTION.md`'s rules are implemented in the port and cite
/// nothing, because they were derived **black-box** and only later found to
/// agree with the read: the alignment-nibble ladder (`§2` step 3;
/// `c2_core`'s `coff::container::align_nibble`, fitted from
/// `OBJ_DYNINIT_SHAPE.md` §4.2) and the `.bss` reversal rule (`§5`; Rule Y1 in
/// `coff::data`, which `§5` itself records as *"independently confirmed
/// black-box by lane `w-bss` from the IL alone"*). An address grep scores both
/// **0** where the honest answer is **1**. That is why the entry unit is
/// **not** this row's denominator, and it is published as a rival reading
/// rather than suppressed.
///
/// **On the ARM unit the divergence was measured and it is zero.** Lane
/// `w-secported` checked all fifteen live arms by hand against the port on the
/// tree that shipped this: the port has no `.gl` record-stream decoder at all —
/// `c2_il::func::gl` scans name runs and TYPE tags without ever consuming a
/// record tag, and `c2_il::func::glalias` is the single module that decodes a
/// record grammar, on the shared tag-`0x04`/`0x0E`/`0x10` handler `0x10b9bdcf`
/// (adopted under `DISCLOSURE.md` **W-ALIAS-1**). So the citing set and the
/// implementing set coincide, 15 of 15 cells.
///
/// **The direction it will break is named**: a lane that decodes a `.gl` record
/// without citing its arm moves the truth without moving this number, and the
/// number is then LOW. It cannot break the other way — an address in live
/// source with no decoder behind it would be a citation of a site the port does
/// not implement, which is the thing `DISCLOSURE.md` exists to prevent.
///
/// # Why this is live rather than transcribed
///
/// Both inputs are files on the tree: the denominator is
/// [`GLREC_ARMS_TSV`], re-derivable from the pinned image by
/// `work/w-secported/dump_glrec.py`, and the numerator is a scan of the port's
/// own sources. Nothing here transcribes a whitebox value into `crates/`
/// beyond the addresses this function is given at run time. **A lane that ports
/// a `.gl` record arm moves this number without touching this file**, which is
/// the property `recount_encode_ported` has and the property that makes a
/// `ported` cell worth printing.
pub fn recount_section_ported(root: &Path, t: &GlRecTables) -> (u64, u64) {
    let live = t.live_arms();
    let cited = crates_cited(root, &live);
    (cited.len() as u64, live.len() as u64)
}

/// Which of `addrs` appear in the port's own sources. Separate from
/// [`recount_section_ported`] so a test can assert **which** arm was found,
/// never only how many — a count that is right for the wrong reason is the
/// failure `#3336` names.
pub fn crates_cited(root: &Path, addrs: &[u32]) -> Vec<u32> {
    let needles: Vec<(u32, String)> = addrs.iter().map(|&a| (a, format!("{a:08x}"))).collect();
    let mut hit: Vec<u32> = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        let rd = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if p.is_dir() {
                // `target/` is build output, `tests/` is the integration-test
                // tree, and the metric crate is the observer.
                if name == "target" || name == "tests" || name == PORTED_SCAN_EXCLUDES_CRATE {
                    continue;
                }
                stack.push(p);
            } else if name.ends_with(".rs") && name != "tests.rs" && name != "testutil.rs" {
                let s = std::fs::read_to_string(&p).unwrap_or_default();
                for (a, n) in &needles {
                    if !hit.contains(a) && s.contains(n.as_str()) {
                        hit.push(*a);
                    }
                }
            }
        }
    }
    hit.sort_unstable();
    hit
}

/// Re-verify **every denominator in [`SUBSYSTEMS`] against the tree**, plus the
/// structural invariants that make the table a scoreboard rather than prose.
///
/// Checks, each of which a fabrication must be able to redden — and all three
/// fabrications are exercised by this module's own `mod tests`:
///
/// 1. **Enumeration.** Every `SUBSYS.md` §1 row has exactly one [`SUBSYSTEMS`]
///    row, matched by page name, and there are no extras. A dropped subsystem
///    fails here.
/// 2. **Band recount.** Every `Basis::Band` denominator recomputes from
///    `FUNCS.tsv` under the row's declared endpoint convention. A wrong
///    denominator fails here.
/// 3. **Probe.** Every row's `den_probe` still appears verbatim in its page. A
///    page whose coverage line moved fails here rather than rotting.
/// 4. **No silence.** Every residue and pending string is non-empty, and every
///    strength is one of measured / residue / pending.
/// 5. **Marks.** Every page yields a mark census (i.e. has the `---` the rule
///    depends on) and a non-zero total.
///
/// `root` is the repo root and `ref_dir` is the reference index, which the shell
/// self-test points at a corrupted copy. **They are separate arguments on
/// purpose**: [`PortedRecount::GlRecArms`] reads its denominator from
/// [`GLREC_ARMS_TSV`] and its numerator from `root/crates`, neither of which
/// lives under `ref_dir` — so a self-test that corrupts the index leaves the
/// section recount reading the real tree, and says so rather than silently
/// grading a copy.
pub fn verify(root: &Path, ref_dir: &Path, table: &[Subsystem]) -> Verified {
    let mut v = Verified {
        marks: BTreeMap::new(),
        recounted: BTreeMap::new(),
        ported_recounted: BTreeMap::new(),
        failures: Vec::new(),
    };

    // ---- 1. enumeration against SUBSYS.md §1 -------------------------------
    match std::fs::read_to_string(ref_dir.join("SUBSYS.md")) {
        Err(e) => v.failures.push(format!("SUBSYS.md unreadable: {e}")),
        Ok(s) => {
            let section = subsys_section_1(&s);
            let mut listed: Vec<&str> = Vec::new();
            for line in section.lines() {
                for sub in table.iter() {
                    if line.contains(sub.page) && !listed.contains(&sub.page) {
                        listed.push(sub.page);
                    }
                }
            }
            for sub in table.iter() {
                if !section.contains(sub.page) {
                    v.failures.push(format!(
                        "{}: SUBSYS.md §1 has no row naming this page",
                        sub.key
                    ));
                }
            }
            // The reverse direction — a §1 row with no table row — is the
            // fabrication `control_a_dropped_subsystem_is_caught` drives.
            for pg in section_pages(&section) {
                if !table.iter().any(|s| s.page == pg) {
                    v.failures.push(format!(
                        "SUBSYS.md §1 lists {pg} and the metric table has no row for it \
                         — the scoreboard is missing a subsystem"
                    ));
                }
            }
        }
    }

    // ---- 2/3. per-page checks ---------------------------------------------
    let funcs = std::fs::read_to_string(ref_dir.join("FUNCS.tsv")).unwrap_or_default();
    if funcs.is_empty() {
        v.failures.push("FUNCS.tsv unreadable or empty".to_string());
    }
    let addrs = funcs_addresses(&funcs);

    for sub in table.iter() {
        let page = match std::fs::read_to_string(ref_dir.join(sub.page)) {
            Ok(p) => p,
            Err(e) => {
                v.failures.push(format!("{}: {} unreadable: {e}", sub.key, sub.page));
                continue;
            }
        };

        if sub.basis == Basis::Band {
            if sub.bands.is_empty() {
                v.failures
                    .push(format!("{}: Basis::Band with no bands", sub.key));
            } else {
                let n = recount_band(&addrs, sub.bands);
                v.recounted.insert(sub.key, n);
                if n != sub.sites {
                    v.failures.push(format!(
                        "{}: band denominator DOES NOT REPRODUCE — table says {}, \
                         FUNCS.tsv gives {n} over {} band(s)",
                        sub.key,
                        sub.sites,
                        sub.bands.len()
                    ));
                }
            }
        } else if !sub.bands.is_empty() {
            v.failures.push(format!(
                "{}: non-band basis {:?} must declare no bands",
                sub.key, sub.basis
            ));
        }

        if sub.den_probe.is_empty() {
            v.failures
                .push(format!("{}: empty den_probe — the denominator is unverifiable", sub.key));
        } else if !page.contains(sub.den_probe) {
            v.failures.push(format!(
                "{}: den_probe not found verbatim in {} — the page moved and the \
                 carried denominator is now unsourced: {:?}",
                sub.key, sub.page, sub.den_probe
            ));
        }

        match count_marks(&page) {
            None => v.failures.push(format!(
                "{}: {} has no `---` line, so the mark census rule cannot be applied",
                sub.key, sub.page
            )),
            Some(m) if m.total() == 0 => v.failures.push(format!(
                "{}: {} yielded 0 evidence marks — NO-RESULT, not agreement 0",
                sub.key, sub.page
            )),
            Some(m) => {
                v.marks.insert(sub.key, m);
            }
        }

        // ---- 4. no silence -------------------------------------------------
        for (name, cell) in [
            ("ported", &sub.ported),
            ("exercised", &sub.exercised),
        ] {
            check_cell(&mut v, sub.key, name, cell);
        }

        // ---- 6. `ported` recount -------------------------------------------
        // A measured `ported` must be recomputable, and a row with no recount
        // must not carry a number. Both directions, because either one alone
        // lets a fabrication through.
        match (&sub.ported, sub.ported_recount) {
            (Cell::Measured { num, den, .. }, Some(PortedRecount::EncodeArms)) => {
                let txt = std::fs::read_to_string(ref_dir.join(ENCODE_ARMS_TXT))
                    .unwrap_or_default();
                let arms = parse_encode_arms(&txt);
                if arms.is_empty() {
                    v.failures.push(format!(
                        "{}: {ENCODE_ARMS_TXT} unreadable or empty — ported is \
                         NO-RESULT, not 0",
                        sub.key
                    ));
                } else {
                    let (rn, rd) = recount_encode_ported(&arms);
                    v.ported_recounted.insert(sub.key, (rn, rd));
                    if (rn, rd) != (*num, *den) {
                        v.failures.push(format!(
                            "{}: ported DOES NOT REPRODUCE — table says {num}/{den}, \
                             the tree gives {rn}/{rd} (ENCODE_ARMS.txt x \
                             c2_core::codegen::mop)",
                            sub.key
                        ));
                    }
                }
            }
            (Cell::Measured { num, den, .. }, Some(PortedRecount::GlRecArms)) => {
                let txt = std::fs::read_to_string(root.join(GLREC_ARMS_TSV))
                    .unwrap_or_default();
                let t = parse_glrec_arms(&txt);
                if t.rows.is_empty() || t.fatal.is_none() {
                    v.failures.push(format!(
                        "{}: {GLREC_ARMS_TSV} unreadable, empty, or missing its \
                         `# fatal` line — ported is NO-RESULT, not 0",
                        sub.key
                    ));
                } else {
                    let (rn, rd) = recount_section_ported(root, &t);
                    v.ported_recounted.insert(sub.key, (rn, rd));
                    if (rn, rd) != (*num, *den) {
                        v.failures.push(format!(
                            "{}: ported DOES NOT REPRODUCE — table says {num}/{den}, \
                             the tree gives {rn}/{rd} (GLREC_ARMS.tsv live arms x a \
                             scan of crates/ outside {PORTED_SCAN_EXCLUDES_CRATE})",
                            sub.key
                        ));
                    }
                }
            }
            (Cell::Measured { .. }, None) => v.failures.push(format!(
                "{}: ported carries a NUMBER with no recount — a ported cell nothing \
                 can recompute is a fabrication waiting to happen",
                sub.key
            )),
            (_, Some(_)) => v.failures.push(format!(
                "{}: ported declares a recount but is not measured",
                sub.key
            )),
            (_, None) => {}
        }
        if let Some(c) = &sub.agreement_extra {
            check_cell(&mut v, sub.key, "agreement", c);
        }
        if sub.sites_doc.is_empty() || sub.read_doc.is_empty() {
            v.failures.push(format!(
                "{}: a denominator with no doc reference beside it",
                sub.key
            ));
        }
    }

    v
}

fn check_cell(v: &mut Verified, key: &str, name: &str, cell: &Cell) {
    match cell {
        Cell::Residue(r) if r.trim().is_empty() => v
            .failures
            .push(format!("{key}: {name} is an EMPTY residue — silence, not a name")),
        Cell::Pending(p) if p.trim().is_empty() => v
            .failures
            .push(format!("{key}: {name} is an EMPTY pending — silence, not a name")),
        Cell::Measured { den: 0, .. } => v.failures.push(format!(
            "{key}: {name} is measured over a denominator of 0 — that is NO-RESULT"
        )),
        Cell::Measured { num, den, .. } if num > den => v.failures.push(format!(
            "{key}: {name} numerator {num} exceeds denominator {den}"
        )),
        _ => {}
    }
}

/// The text of `SUBSYS.md` between the `## 1.` heading and the next `## `.
fn subsys_section_1(s: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in s.lines() {
        if line.starts_with("## 1.") {
            inside = true;
            continue;
        }
        if inside && line.starts_with("## ") {
            break;
        }
        if inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Every `P_*.md` page named by a §1 table row (a line beginning with `|` and
/// containing a `.md` link).
fn section_pages(section: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in section.lines() {
        if !line.starts_with('|') {
            continue;
        }
        let mut rest = line;
        while let Some(i) = rest.find("P_") {
            rest = &rest[i..];
            let end = rest.find(".md").map(|e| e + 3);
            match end {
                Some(e) => {
                    let name = &rest[..e];
                    if name
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '.' || c == 'm' || c == 'd')
                        && !out.iter().any(|x| x == name)
                    {
                        out.push(name.to_string());
                    }
                    rest = &rest[e..];
                }
                None => break,
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The workload-output census (strength 3's proxy source)
// ---------------------------------------------------------------------------

/// Section-name census of the real-`c2` objs of the workload, recomputed from
/// the committed `sections.jsonl`.
///
/// Carries its own known-answer control: each record states `nsec`, and the
/// parsed `order` array must have exactly that many entries. A parser that
/// silently drops names fails the control instead of reporting a small number.
#[derive(Clone, Debug, Default)]
pub struct SectionCensus {
    pub tus: u64,
    pub sections: u64,
    /// section name -> (TUs carrying it, total sections)
    pub by_name: BTreeMap<String, (u64, u64)>,
    /// Records whose `order` length disagreed with their own `nsec`. Non-zero
    /// is an alarm, not a gap.
    pub nsec_disagree: u64,
    pub present: bool,
}

pub fn section_census(root: &Path) -> SectionCensus {
    let Ok(s) = std::fs::read_to_string(root.join(SECTIONS_JSONL)) else {
        return SectionCensus::default();
    };
    let mut c = SectionCensus {
        present: true,
        ..Default::default()
    };
    for line in s.lines() {
        if line.trim().is_empty() {
            continue;
        }
        c.tus += 1;
        let nsec: u64 = json_str(line, "nsec").and_then(|v| v.parse().ok()).unwrap_or(0);
        let names = order_names(line);
        if names.len() as u64 != nsec {
            c.nsec_disagree += 1;
        }
        c.sections += names.len() as u64;
        let mut seen: Vec<&String> = Vec::new();
        for n in &names {
            let e = c.by_name.entry(n.clone()).or_insert((0, 0));
            e.1 += 1;
        }
        for n in &names {
            if !seen.iter().any(|x| *x == n) {
                seen.push(n);
                c.by_name.entry(n.clone()).or_insert((0, 0)).0 += 1;
            }
        }
    }
    c
}

fn order_names(line: &str) -> Vec<String> {
    let Some(i) = line.find("\"order\"") else {
        return Vec::new();
    };
    let rest = &line[i..];
    let Some(open) = rest.find('[') else {
        return Vec::new();
    };
    let Some(close) = rest[open..].find(']') else {
        return Vec::new();
    };
    let inner = &rest[open + 1..open + close];
    let mut out = Vec::new();
    let mut it = inner.char_indices();
    while let Some((i, ch)) = it.next() {
        if ch != '"' {
            continue;
        }
        let start = i + 1;
        let mut end = start;
        for (j, c2) in inner[start..].char_indices() {
            if c2 == '"' {
                end = start + j;
                break;
            }
        }
        out.push(inner[start..end].to_string());
        // advance past the closing quote
        while let Some((k, _)) = it.next() {
            if k >= end {
                break;
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

fn commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The three standing traps, printed verbatim with every render. Decision 15's
/// own words plus `FUNCTION_BYTE_MATCH.md` §0's.
pub const TRAPS: &[&str] = &[
    "THE SIGNAL IS THE CHANGE IN EACH STRENGTH, NEVER ITS DISTANCE FROM 0 OR 100. \
     A subsystem can go from 20 % to 90 % understood with `match` unchanged; that \
     movement is what this table exists to make visible, and a row's absolute \
     height is not a grade.",
    "A GREEN ROW IS A STATEMENT ABOUT THE POPULATION THE INSTRUMENT CAN REACH. \
     Every denominator here says which tree and which enumeration it came from, \
     because the same subsystem has more than one defensible denominator: the \
     band and the TU-level attribution differ by up to 3.8x (inliner, 93 vs 350). \
     A ratio without its denominator's basis is not a reading.",
    "THESE KEYS LICENSE NO EMIT. They are progress instruments under \
     docs/FUNCTION_BYTE_MATCH.md §0 — never in scripts/gate.sh's verdict, their \
     own block under their own disclaimer, namespaced, NO-RESULT rather than a \
     ratio over zero. The sole judge of the port is real c2.dll under wibo plus a \
     byte-exact obj compare, and a wrong emit still scores strictly below the \
     refusal it replaced.",
];

/// Everything one render needs, gathered from the tree.
pub struct Rendered {
    pub text: String,
    pub markdown: String,
    pub verified: Verified,
    pub census: SectionCensus,
    pub stamp: WorkloadStamp,
}

/// Build the report. `root` is the repo root; `ref_dir` defaults to
/// `root/docs/whitebox/ref` and is overridable so the shell self-test can point
/// at a deliberately corrupted copy.
pub fn render(root: &Path, ref_dir: Option<PathBuf>, table: &[Subsystem]) -> Rendered {
    let ref_dir = ref_dir.unwrap_or_else(|| root.join(REF_DIR));
    let verified = verify(root, &ref_dir, table);
    let census = section_census(root);
    let stamp = workload_stamp(root);

    let mut t = String::new();
    let mut m = String::new();

    // ---- the disclaimer, first, in both renders ---------------------------
    let _ = writeln!(
        t,
        "\n PER-SUBSYSTEM METRICS (w-submetric, decision 15) — a PROGRESS \
         instrument under\n   docs/FUNCTION_BYTE_MATCH.md §0. Ten subsystems, four \
         strengths, every denominator\n   printed beside its numerator."
    );
    for trap in TRAPS {
        let _ = writeln!(t, "   * {}", wrap(trap, 74, "     "));
    }

    let _ = writeln!(t, "\n   WORKLOAD STAMP");
    let _ = writeln!(t, "     ref index  : {}", ref_dir.display());
    if stamp.present {
        let _ = writeln!(
            t,
            "     workload   : {} TUs, real c2 section census generated {} \
             (corpus {} dirty={})",
            stamp.records.map(commas).unwrap_or_else(|| "?".into()),
            stamp.generated_utc.clone().unwrap_or_else(|| "?".into()),
            stamp
                .corpus_head
                .clone()
                .map(|h| h[..h.len().min(9)].to_string())
                .unwrap_or_else(|| "?".into()),
            stamp
                .corpus_dirty
                .map(|d| d.to_string())
                .unwrap_or_else(|| "?".into()),
        );
        let _ = writeln!(
            t,
            "     recomputed : {} TUs, {} sections, {} distinct names, \
             nsec-disagree {} (known answer 0)",
            commas(census.tus),
            commas(census.sections),
            census.by_name.len(),
            census.nsec_disagree
        );
    } else {
        let _ = writeln!(
            t,
            "     workload   : ABSENT — {SECTIONS_JSONL} not in this tree. Every \
             workload-output proxy below reads NO-DATA rather than 0."
        );
    }
    let _ = writeln!(
        t,
        "     byte-owned : CITED, NOT RE-MEASURED — {}",
        wrap(BYTE_OWNED_CITATION, 70, "                  ")
    );

    // ---- the table --------------------------------------------------------
    let _ = writeln!(t, "\n   THE TUPLE, PER SUBSYSTEM\n");
    for sub in table.iter() {
        let marks = verified.marks.get(sub.key).copied().unwrap_or_default();
        let _ = writeln!(t, "   [{}] {} — {}", sub.key, sub.title, sub.page);
        let _ = writeln!(t, "       TU(s): {}", sub.tus);
        let _ = writeln!(
            t,
            "     1 read      : sites {} ({}) ⊇ read {} ({}) ⊇ ported {}",
            commas(sub.sites),
            sub.sites_unit,
            commas(sub.read),
            sub.read_unit,
            match &sub.ported {
                Cell::Residue(_) => "RESIDUE".to_string(),
                c => c.render(),
            }
        );
        let _ = writeln!(t, "                   denominator doc: {}", sub.sites_doc);
        if let Some(tp) = sub.tu_population {
            let _ = writeln!(
                t,
                "                   SECOND DENOMINATOR: {} functions under FUNCS.tsv's \
                 TU-level `subsys` column ({:.1}x the band)",
                commas(tp),
                tp as f64 / sub.sites.max(1) as f64
            );
        }
        match &sub.ported {
            Cell::Residue(r) => {
                let _ = writeln!(
                    t,
                    "                   ported RESIDUE — {}",
                    wrap(r, 62, "                     ")
                );
            }
            // A MEASURED `ported` MUST PRINT ITS CAVEAT. The number alone is the
            // trap this row already documents: `SUBSYS.md` prints `14 / 14` for
            // the same subsystem whose coverage line is `79 / 79`, 5.6x apart,
            // and neither cell is wrong. Which denominator was used, and why, is
            // not an optional footnote on a `ported` cell.
            Cell::Measured { source, caveat, .. } => {
                let _ = writeln!(
                    t,
                    "                   ported RECOUNTED on this tree — {}",
                    wrap(source, 62, "                     ")
                );
                let _ = writeln!(
                    t,
                    "                   {}",
                    wrap(caveat, 62, "                     ")
                );
            }
            Cell::Pending(p) => {
                let _ = writeln!(
                    t,
                    "                   ported PENDING — {}",
                    wrap(p, 62, "                     ")
                );
            }
        }
        let _ = writeln!(
            t,
            "     2 agreement : marks [O] {} of {} ({:.1} %) — [R] {} [I] {}",
            marks.obj,
            marks.total(),
            if marks.total() == 0 {
                0.0
            } else {
                100.0 * marks.obj as f64 / marks.total() as f64
            },
            marks.read,
            marks.inferred
        );
        let _ = writeln!(
            t,
            "                   A MARK IS A PAGE ANNOTATION, NOT A SITE — this is the \
             page's own"
        );
        let _ = writeln!(
            t,
            "                   evidence-tier census, not a differential."
        );
        match &sub.agreement_extra {
            None => {
                let _ = writeln!(
                    t,
                    "                   RESIDUE — no differential exists for {} beyond \
                     the mark census",
                    sub.key
                );
            }
            Some(c) => {
                let _ = writeln!(t, "                   {}", c.render());
                let _ = writeln!(
                    t,
                    "                   source: {}",
                    wrap(c.source(), 62, "                     ")
                );
                let _ = writeln!(
                    t,
                    "                   {}",
                    wrap(c.note(), 62, "                     ")
                );
            }
        }
        let ex = exercised_cell(sub, &census);
        let _ = writeln!(t, "     3 exercised : {}", ex.render());
        let _ = writeln!(
            t,
            "                   {}",
            wrap(ex.note(), 62, "                     ")
        );
        let _ = writeln!(
            t,
            "     4 byte-owned: CITED #3534 — no per-subsystem split exists"
        );
        let _ = writeln!(t);
        if !sub.subsys_cell_note.is_empty() {
            let _ = writeln!(
                t,
                "       ! SUBSYS.md §1 CELL: {}\n",
                wrap(sub.subsys_cell_note, 66, "         ")
            );
        }
    }

    // ---- machine-readable keys -------------------------------------------
    let _ = writeln!(
        t,
        "   MACHINE-READABLE (namespaced; NOT gap-metric, NOT read by gate.sh)"
    );
    for line in keys(table, &verified, &census) {
        let _ = writeln!(t, "     subsys-metric {line}");
    }

    // ---- verification verdict --------------------------------------------
    let _ = writeln!(t, "\n   SELF-VERIFICATION (the same checks `cargo test` runs)");
    if verified.ok() {
        let _ = writeln!(
            t,
            "     VERIFY: PASS — {} band denominators recounted from FUNCS.tsv, {} \
             pages' coverage probes found verbatim, {} mark censuses, 0 empty \
             residues.",
            verified.recounted.len(),
            table.len(),
            verified.marks.len()
        );
    } else {
        let _ = writeln!(
            t,
            "     VERIFY: FAIL — {} problem(s):",
            verified.failures.len()
        );
        for f in &verified.failures {
            let _ = writeln!(t, "       - {f}");
        }
    }

    markdown(&mut m, table, &verified, &census, &stamp, &ref_dir);

    Rendered {
        text: t,
        markdown: m,
        verified,
        census,
        stamp,
    }
}

/// Strength 3's cell, with the workload census actually consulted so an absent
/// census reads `NO-DATA` rather than a stale carried number.
fn exercised_cell(sub: &Subsystem, census: &SectionCensus) -> Cell {
    match &sub.exercised {
        Cell::Measured { .. } if !census.present && sub.key != "symbol" => Cell::Residue(
            "NO-DATA — the workload section census is not in this tree, so this \
             row's output proxy cannot be recomputed. Not 0.",
        ),
        other => other.clone(),
    }
}

/// The `subsys-metric` lines. Namespaced, sorted by key NAME (never by mass —
/// this repo's standing rule against dispatching off a size ranking, which has
/// now bound five times, `#3505`).
pub fn keys(table: &[Subsystem], v: &Verified, census: &SectionCensus) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for sub in table.iter() {
        let k = sub.key;
        out.push(format!("{k}-sites {}", sub.sites));
        out.push(format!("{k}-read {}", sub.read));
        if let Some(n) = v.recounted.get(k) {
            out.push(format!("{k}-sites-recounted {n}"));
        }
        if let Some(tp) = sub.tu_population {
            out.push(format!("{k}-sites-tu-level {tp}"));
        }
        let m = v.marks.get(k).copied().unwrap_or_default();
        out.push(format!("{k}-marks-obj {}", m.obj));
        out.push(format!("{k}-marks-total {}", m.total()));
        match &sub.ported {
            Cell::Measured { num, den, .. } => {
                out.push(format!("{k}-ported {num}"));
                // The denominator prints beside the numerator, always — this
                // row's own `subsys_cell_note` exists because two defensible
                // denominators on one page differ by 5.6x.
                out.push(format!("{k}-ported-den {den}"));
            }
            _ => out.push(format!("{k}-ported RESIDUE")),
        }
        match exercised_cell(sub, census) {
            Cell::Measured { num, den, .. } => {
                out.push(format!("{k}-exercised-proxy {num}"));
                out.push(format!("{k}-exercised-proxy-den {den}"));
            }
            _ => out.push(format!("{k}-exercised-proxy RESIDUE")),
        }
    }
    out.push(format!("subsystems {}", table.len()));
    out.push(format!("verify-failures {}", v.failures.len()));
    out.push(format!(
        "workload-census-nsec-disagree {}",
        census.nsec_disagree
    ));
    out.push("byte-owned CITED-3534".to_string());
    out.sort();
    out
}

fn wrap(s: &str, width: usize, indent: &str) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    for w in s.split_whitespace() {
        if col > 0 && col + 1 + w.len() > width {
            out.push('\n');
            out.push_str(indent);
            col = 0;
        } else if col > 0 {
            out.push(' ');
            col += 1;
        }
        out.push_str(w);
        col += w.len();
    }
    out
}

/// The reference index path as it may appear in a **committed** file: the
/// repo-relative form when it is the tree's own index, and only the trailing
/// components otherwise. Never an absolute machine path (`CLAUDE.md` § Commits).
fn rel_ref_dir(ref_dir: &Path) -> String {
    let s = ref_dir.to_string_lossy();
    match s.find(REF_DIR) {
        Some(i) => s[i..].to_string(),
        None => ref_dir
            .file_name()
            .map(|f| format!("(non-default index)/{}", f.to_string_lossy()))
            .unwrap_or_else(|| "(non-default index)".to_string()),
    }
}

fn md_escape(s: &str) -> String {
    s.replace('|', "\\|").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn markdown(
    m: &mut String,
    table: &[Subsystem],
    v: &Verified,
    census: &SectionCensus,
    stamp: &WorkloadStamp,
    ref_dir: &Path,
) {
    let _ = writeln!(
        m,
        "# SUBSYS METRICS — the per-subsystem scoreboard\n\n\
         **Status: adopted 2026-08-26 (lane `w-submetric`, boards `#3617`–`#3622`).**\n\
         Funded by [`DECISIONS_2026-08-22.md`](DECISIONS_2026-08-22.md) § Decision 15,\n\
         the owner's restructuring of the working goal: *\"the overall TU goal is too\n\
         broad because it is binary. we need a smarter goal … focus on building tools\n\
         we can use to measure our progress for each unit.\"*\n\n\
         One 4-tuple per [`whitebox/ref/SUBSYS.md`](whitebox/ref/SUBSYS.md) §1\n\
         subsystem — **read**, **agreement**, **exercised**, **byte-owned** — with\n\
         **every denominator printed beside its numerator**.\n"
    );

    let _ = writeln!(
        m,
        "## 0. The separation rule (read this even if you read nothing else)\n\n\
         > **These keys are PROGRESS instruments and never correctness criteria.**\n\
         > The real `c2` under wibo plus the byte-exact whole-obj compare is the SOLE\n\
         > judge of the port (`CLAUDE.md`). A `subsys-metric` row going green while\n\
         > `mismatch` reads 1 is a FAILING tree.\n\n\
         [`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md) §0 is the standing\n\
         template for every gradient added after FBM, and this one adopts all five\n\
         properties verbatim, as `decode-reach-*` and `symbind-*` did before it:\n\n\
         * **Never in `scripts/gate.sh`'s verdict**, and it must never be added\n\
           there. It does not print inside a `c2rs gap` scan at all — it is a\n\
           separate offline subcommand, so it cannot move the gate's 21-row count\n\
           table even by accident.\n\
         * **Its own block**, under its own disclaimer, apart from the class table\n\
           that carries `match`/`mismatch`.\n\
         * **Namespaced keys** — `subsys-metric <key> <value>`. No existing key,\n\
           predicate or denominator is narrowed, widened or redefined here.\n\
         * **It licenses no emit.** A subsystem row going green is not a reason to\n\
           accept a shape or to widen the admitted set.\n\
         * **Unrepresentable over an empty scan** — a strength with no data prints a\n\
           **named residue**, never `0`, never silence.\n"
    );

    let _ = writeln!(
        m,
        "## 1. The three standing traps, verbatim\n"
    );
    for (i, trap) in TRAPS.iter().enumerate() {
        let _ = writeln!(m, "{}. **{}**\n", i + 1, md_escape(trap));
    }

    let _ = writeln!(
        m,
        "## 2. What each strength actually is here\n\n\
         | strength | this instrument's answer |\n|---|---|\n\
         | **1 read** | a **containment, never a ratio**: `sites ⊇ read ⊇ ported`. \
         `sites` is the subsystem's enumerable population, **recomputed from \
         `FUNCS.tsv` on this tree** where it is a band; `read` is what the `P_*.md` \
         page says it read, in the page's own unit; `ported` is **measured on two rows \
         and a named residue on the other eight** — see §4, and note that each \
         measured row's `ported` is in its OWN unit (encode arms, `.gl` arms), so \
         the containment nests SITE SETS and the three counts' RATIOS must not be \
         compared |\n\
         | **2 agreement** | the page's own **evidence-mark census** — `[O]` \
         obj-confirmed against `[R]`+`[O]`+`[I]` — plus, where a page carries a real \
         differential, that differential quoted with its own denominator. **A mark is \
         a page annotation, not a site.** Two rows carry more: `encode` has a \
         measured differential, `inline`'s is being built by lane `w-inlmetric` and \
         prints `PENDING` |\n\
         | **3 exercised** | a **labelled workload-output proxy** where one exists, \
         from the committed real-`c2` section census of the workload; a named residue \
         otherwise. **Per-SITE exercise is unmeasurable on this tree for all ten** — \
         nothing traces `c2.dll`'s own addresses over the workload, so no row can say \
         which of its functions the workload entered |\n\
         | **4 byte-owned** | **CITED, NEVER RE-MEASURED.** Board `#3534` measured it \
         2026-08-25. Decision 15 says so in its own words; re-funding that read is \
         what this repo calls *\"check the board before dispatching\"* |\n\n\
         **The mark census's honest limit, stated before its numbers are read:** it \
         counts a page's claims about its own evidence tier, not sites and not \
         agreements. A page may mark one sentence `[O]` and cover twenty addresses \
         with it. It is published as strength 2 because it is the only quantity that \
         is both uniform across all ten pages and mechanically recomputable — and \
         because the alternative was ten rows of silence.\n\n\
         **The counting rule, so it is reproducible:** everything up to and including \
         a page's first line consisting of exactly `---` is the provenance banner and \
         mark legend and is skipped; every occurrence of `[R]`/`[O]`/`[I]` after it \
         counts.\n"
    );

    let _ = writeln!(m, "## 3. The tuple table\n");
    let _ = writeln!(
        m,
        "| subsystem | page | 1 read — `sites ⊇ read ⊇ ported` | 2 agreement | 3 exercised | 4 byte-owned |"
    );
    let _ = writeln!(m, "|---|---|---|---|---|---|");
    for sub in table.iter() {
        let marks = v.marks.get(sub.key).copied().unwrap_or_default();
        let second = sub
            .tu_population
            .map(|tp| format!("<br>**second denominator** {tp} (TU-level, {:.1}×)", tp as f64 / sub.sites.max(1) as f64))
            .unwrap_or_default();
        let agree = match &sub.agreement_extra {
            None => format!(
                "`[O] {}` of `{}` marks ({:.1} %)<br>**RESIDUE — no differential exists for {}** beyond the page's own mark census",
                marks.obj,
                marks.total(),
                if marks.total() == 0 { 0.0 } else { 100.0 * marks.obj as f64 / marks.total() as f64 },
                sub.key
            ),
            Some(c) => format!(
                "`[O] {}` of `{}` marks ({:.1} %)<br>{}<br>*{}*",
                marks.obj,
                marks.total(),
                if marks.total() == 0 { 0.0 } else { 100.0 * marks.obj as f64 / marks.total() as f64 },
                md_escape(&c.render()),
                md_escape(c.note())
            ),
        };
        let ex = exercised_cell(sub, census);
        // `Cell::Residue`'s render IS its note; printing both duplicates the
        // whole sentence in the cell. Only a Measured cell has a caveat that
        // is distinct from its number, and a measured number without its
        // caveat is the thing this table exists to prevent.
        let ex_cell = match &ex {
            Cell::Measured { .. } => format!("{}<br>*{}*", md_escape(&ex.render()), md_escape(ex.note())),
            _ => md_escape(&ex.render()),
        };
        // **THIS CELL USED TO BE THE LITERAL STRING `ported RESIDUE`, ON ALL TEN
        // ROWS, WHATEVER THE ROW SAID.** `w-submetric` hard-coded it when every
        // row really was a residue; `w-encmap` then measured `encode` at 27/79
        // and the tuple table went on printing RESIDUE beside a §4 that said
        // *"encode is measured: 27 of 79 arms"* — the two sections of one
        // generated file disagreeing, with the machine-readable keys siding
        // with §4. No control could see it: every control here fabricates a
        // NUMBER, and this fabricated a WORD. Repaired by lane `w-secported`
        // (`#3665`) in the commit that added the second measured row, because
        // shipping a second one under the same bug would have hidden it twice.
        // A Residue keeps its one-word cell and its reason stays in §4's list —
        // pasting the whole sentence into a table column is what §4 exists to
        // avoid. A Measured cell prints its number AND its denominator, which
        // is this file's first rule.
        let ported_cell = match &sub.ported {
            Cell::Measured { .. } => format!("**ported {}**", md_escape(&sub.ported.render())),
            Cell::Residue(_) => "**ported RESIDUE**".to_string(),
            Cell::Pending(_) => "**ported PENDING**".to_string(),
        };
        let _ = writeln!(
            m,
            "| **{}**<br>`{}` | [`{}`]({}) | **{} sites** ({})<br>⊇ **read {}** ({})<br>⊇ {}{} | {} | {} | CITED `#3534` |",
            sub.title,
            sub.key,
            sub.page,
            format!("whitebox/ref/{}", sub.page),
            commas(sub.sites),
            md_escape(sub.sites_unit),
            commas(sub.read),
            md_escape(sub.read_unit),
            ported_cell,
            second,
            agree,
            ex_cell,
        );
    }

    let _ = writeln!(
        m,
        "\n## 4. `ported` — two rows measured, eight still residue\n\n\
         Decision 15 asks strength 1 for *\"how many the port implements\"*. Lane\n\
         `w-submetric` shipped it as a **named residue on all ten rows** (`#3617`),\n\
         because no port↔image site map existed. Lane `w-encmap` (`#3636`–`#3641`,\n\
         decision 16) converted the cheapest one; lane `w-secported`\n\
         (`#3661`–`#3666`, decision 17) converted the second.\n\n\
         **`encode` is measured: 27 of 79 arms.** The predicate is\n\
         `subsys::recount_encode_ported` and it is **recomputed on every run and\n\
         every `cargo test`** from `ENCODE_ARMS.txt` × `c2_core::codegen::mop`'s\n\
         public tables — a carried number could rot, and a fabricated one is caught\n\
         by `control_a_fabricated_ported_is_caught`. **The denominator is the 79\n\
         arms and the choice is published rather than silent** — see the caveat in\n\
         §3, which also carries the strict (25) and plan-only (28) readings and the\n\
         shape of the 52 unmapped.\n\n\
         **`section` is measured: 1 of 15 live `.gl` record-dispatcher arms.** The\n\
         predicate is `subsys::recount_section_ported`, recomputed from\n\
         `work/w-secported/GLREC_ARMS.tsv` (decoded from the pinned image) × a scan\n\
         of `crates/` outside this crate, and a fabrication is caught by\n\
         `control_a_fabricated_section_ported_is_caught`. **Its denominator is\n\
         published with four rivals**, and the first thing it corrects is a phrase\n\
         this file itself used to print: **there are not 27 arms.** 27 is a count of\n\
         TAG VALUES; they index 16 jump slots, one of which is the fatal `C1001`\n\
         path serving eight tags. The population is **15 live handlers over 19 live\n\
         tags plus one refusal over 8** — `P_SECTION.md` §7.\n\n\
         **THE PROPERTY THAT MAKES A ROW CONVERTIBLE IS NOT \"ITS SITES ARE\n\
         RULES\".** `#3636` named that property and `#3661` tested it on the only\n\
         other row it predicted for. What both convertible rows actually share is a\n\
         **key the port carries on its own side**: an encode-form number, adopted\n\
         from c2's table under `DISCLOSURE.md` W-MOP-2, and a `.gl` arm address,\n\
         adopted under W-ALIAS-1. Where no adoption exists there is nothing to join\n\
         on, and `P_SECTION.md` §7.4 makes that concrete — the port implements the\n\
         alignment-nibble ladder and the `.bss` reversal rule, **derived black-box\n\
         and citing nothing**, so on a site unit an address grep scores them 0 where\n\
         the honest answer is 1. **The two rules agree with c2 and are joinable to\n\
         it by nothing.**\n\n\
         **The other eight stay residue and the reason is structural, not a gap**: the\n\
         port is **I/O-behavioral by construction** (`CLAUDE.md`'s one correctness\n\
         rule — AVX, restructured CFGs, anything, so long as the *output obj*\n\
         matches), so *\"the port implements site `0x10b2e7f8`\"* has no truth value\n\
         for most of these addresses. **A row where the question is not well formed\n\
         keeps its residue rather than getting an invented number**, and `verify`\n\
         refuses a `ported` number that nothing can recount.\n\n\
         Per residue row, with the reason rather than a blank:\n"
    );
    for sub in table.iter() {
        if let Cell::Residue(r) = &sub.ported {
            let _ = writeln!(m, "* **`{}`** — {}", sub.key, md_escape(r));
        }
    }

    // **AND EVERY MEASURED ROW'S CAVEAT, VERBATIM, IN THE PUBLISHED DOC.**
    // `#3665`'s sibling, found in the same pass: the `ported` caveat — which
    // for both measured rows IS the published denominator choice and its
    // rivals, the thing decision 16 and decision 17 each demanded be stated
    // out loud rather than picked silently — reached the CONSOLE render and
    // `subsys.rs`'s source and **nothing else**. `docs/SUBSYS_METRICS.md` has
    // carried `encode`'s number since `w-encmap` without the paragraph that
    // says what its 79 means or why it is not 14 or 111. A denominator
    // published only in the source of the tool that prints it is not
    // published.
    let measured: Vec<&Subsystem> = table
        .iter()
        .filter(|s| matches!(s.ported, Cell::Measured { .. }))
        .collect();
    if !measured.is_empty() {
        let _ = writeln!(
            m,
            "\nAnd for each **measured** row, the caveat that carries its \
             denominator choice and the rivals it was chosen against — verbatim, \
             because a denominator published only in the source of the tool that \
             prints it is not published:\n"
        );
        for sub in measured {
            if let Cell::Measured { num, den, unit, source, caveat } = &sub.ported {
                let _ = writeln!(
                    m,
                    "* **`{}` — {} of {} {}**\n  * source: {}\n  * {}\n",
                    sub.key,
                    commas(*num),
                    commas(*den),
                    md_escape(unit),
                    md_escape(source),
                    md_escape(caveat)
                );
            }
        }
    }

    let _ = writeln!(m, "\n## 5. Where `SUBSYS.md` §1's own cell needs reading twice\n\n\
         Found by re-measuring every denominator on this tree rather than carrying it.\n\
         **None of these is corrected here, and that is a rule rather than a fence** —\n\
         a disagreement recorded beside a page beats a silent rewrite of it\n\
         (`#3538`). It held even where a lane DID own the page: `w-secported` owns\n\
         `P_SECTION.md`, found its coverage line unreproducible under all three\n\
         readings of its own table and wrong from the file's first commit, and left\n\
         the line as written with the amendment beside it at §7.5.\n");
    let mut any = false;
    for sub in table.iter() {
        if !sub.subsys_cell_note.is_empty() {
            any = true;
            let _ = writeln!(m, "* **`{}`** — {}\n", sub.key, md_escape(sub.subsys_cell_note));
        }
    }
    if !any {
        let _ = writeln!(m, "*(none on this tree)*\n");
    }

    let _ = writeln!(m, "\n## 6. Workload stamp\n");
    let _ = writeln!(m, "| what | value |");
    let _ = writeln!(m, "|---|---|");
    // RELATIVE, never absolute: this string is committed into
    // `docs/SUBSYS_METRICS.md` and `CLAUDE.md` forbids machine paths in
    // tracked files.
    let _ = writeln!(m, "| whitebox ref index | `{}` |", rel_ref_dir(ref_dir));
    if stamp.present {
        let _ = writeln!(
            m,
            "| workload section census | `{}` records, generated `{}` |",
            stamp.records.map(commas).unwrap_or_else(|| "?".into()),
            stamp.generated_utc.clone().unwrap_or_else(|| "?".into())
        );
        let _ = writeln!(
            m,
            "| corpus | `{}` dirty=`{}` |",
            stamp.corpus_head.clone().unwrap_or_else(|| "?".into()),
            stamp.corpus_dirty.map(|d| d.to_string()).unwrap_or_else(|| "?".into())
        );
        let _ = writeln!(
            m,
            "| recomputed here | {} TUs, {} sections, {} distinct names, `nsec-disagree {}` (known answer 0) |",
            commas(census.tus),
            commas(census.sections),
            census.by_name.len(),
            census.nsec_disagree
        );
    } else {
        let _ = writeln!(m, "| workload section census | **ABSENT** — proxies read `NO-DATA`, never 0 |");
    }
    let _ = writeln!(m, "| byte-owned | **CITED, NOT RE-MEASURED** — {} |", md_escape(BYTE_OWNED_CITATION));

    let _ = writeln!(m, "\n## 7. Machine-readable keys\n\n\
         Namespaced, and **sorted by key NAME rather than by mass** — this repo's\n\
         standing rule against dispatching off a blocked-key size ranking, which has\n\
         now bound five times (`#3505`, and *\"ranking instruments measure\n\
         themselves\"*, four for four).\n");
    let _ = writeln!(m, "```text");
    for line in keys(table, v, census) {
        let _ = writeln!(m, "subsys-metric {line}");
    }
    let _ = writeln!(m, "```");

    let _ = writeln!(m, "\n## 8. Self-verification\n");
    if v.ok() {
        let _ = writeln!(
            m,
            "`VERIFY: PASS` — {} band denominators recounted from `FUNCS.tsv`, {} pages' \
             coverage probes found verbatim, {} mark censuses, 0 empty residues.",
            v.recounted.len(),
            table.len(),
            v.marks.len()
        );
    } else {
        let _ = writeln!(m, "`VERIFY: FAIL` — {} problem(s):\n", v.failures.len());
        for f in &v.failures {
            let _ = writeln!(m, "* {f}");
        }
    }

    let _ = writeln!(
        m,
        "\n### 8.1 The controls, and that they were watched failing\n\n\
         `#3336`: **a control never seen failing is decoration.** Seven fabrications\n\
         run on every `cargo test -p c2-harness --lib subsys`, each asserting the\n\
         verifier *refuses*, and each pinned to the check that must own the refusal so\n\
         a case cannot pass by being caught for the wrong reason:\n\n\
         | control | fabrication | must be caught by |\n|---|---|---|\n\
         | `control_a_fabricated_denominator_is_caught` | the inliner's `93` → `94` | the `FUNCS.tsv` recount |\n\
         | `control_a_dropped_subsystem_is_caught` | the `eh` row deleted from the table | the `SUBSYS.md` §1 enumeration |\n\
         | `control_an_empty_residue_is_caught` | `dag`'s `ported` residue set to `\"   \"` | the no-silence check |\n\
         | `control_a_moved_coverage_line_is_caught` | `P_COFF`'s probe pointed at a line that is not on the page | the verbatim probe |\n\
         | `control_a_fabricated_ported_is_caught` | `encode`'s `ported` `27` → `28` | the `ENCODE_ARMS.txt` × `mop` recount |\n\
         | `control_a_ported_number_with_no_recount_is_caught` | a number typed into `coff`'s `ported`, which has no recount | the recount-or-residue rule |\n\
         | `control_a_fabricated_section_ported_is_caught` | `section`'s `ported` `1` → `2` | the `GLREC_ARMS.tsv` × `crates/` scan |\n\n\
         **Every `ported` control was watched failing against the SHIPPED table,\n\
         not only against a copy** (lanes `w-encmap`, `w-secported`): editing the real\n\
         `encode` cell `27` → `28` reddens four tests with\n\
         `encode: ported DOES NOT REPRODUCE — table says 28/79, the tree gives 27/79`,\n\
         and deleting the `OPCODES` half of `port_reaches_form` reddens five,\n\
         including `the_three_ported_readings_are_distinct` (`published` collapses\n\
         onto `planned` at 28). Editing the real `section` cell `1` → `2` likewise\n\
         reddens four with `section: ported DOES NOT REPRODUCE — table says 2/15, the\n\
         tree gives 1/15`. A recount that only ever grades a mutated copy is a\n\
         recount that has never been shown to bind the number the doc prints.\n\n\
         **Two further checks are not fabrications and are worth naming separately.**\n\
         `the_section_ported_arm_is_the_one_the_port_actually_decodes` pins *which*\n\
         arm the numerator found — a count that is right for the wrong reason is\n\
         `#3336`'s other failure — and pins the dispatcher's shape (27 tags, 16\n\
         slots, 15 live arms, 19 live tags, 8 fatal) so a re-dump that disagrees\n\
         reddens instead of shipping a moved denominator quietly.\n\
         `the_observer_crate_cannot_move_its_own_ported` is **`#3641`'s family, caught\n\
         by construction rather than by review**: the `section` numerator is a scan of\n\
         source text for arm addresses, and `subsys.rs` must name those addresses to\n\
         explain itself — so the scan excludes its own crate. **The hazard was\n\
         measured, not assumed**: disabling that one exclusion moves the shipped\n\
         number from `1/15` to `2/15`.\n\n\
         And `scripts/subsys_metrics.sh --self-test` drives the **binary** against\n\
         three deliberately corrupted copies of the reference index — a function moved\n\
         out of the inliner band, `P_EH.md`'s coverage line edited, a subsystem\n\
         deleted from `SUBSYS.md` §1 — requiring each to exit non-zero *and* proving\n\
         each mutation applied first, because a `sed` that matched nothing leaves a\n\
         clean copy and the case then \"passes\" by testing the control twice\n\
         (`#3516`'s mutation-not-applied failure, named in the same words by\n\
         `scripts/gate_identity_diff.sh --self-test`).\n\n\
         **And one control is not a fabrication at all**, because the defect it\n\
         guards is not a number.\n\
         `control_a_measured_ported_must_reach_the_rendered_table` asserts that §3's\n\
         own row carries each measured `ported`. That check did **not** exist while\n\
         §3's cell was the hard-coded string `ported RESIDUE` on all ten rows — so\n\
         for two waves this file printed `RESIDUE` in the table and *\"encode is\n\
         measured: 27 of 79 arms\"* four paragraphs below it, with the\n\
         machine-readable keys siding with the prose. **Every other control here\n\
         fabricates a NUMBER; that defect fabricated a WORD**, which is the blind\n\
         spot `#3641` and `#3643` also sit in. `#3665`, repaired and watched failing\n\
         by restoring the literal.\n"
    );

    let _ = writeln!(
        m,
        "## 9. How to regenerate\n\n\
         ```sh\n\
         scripts/subsys_metrics.sh              # console report\n\
         scripts/subsys_metrics.sh --write      # regenerate THIS FILE\n\
         scripts/subsys_metrics.sh --keys       # only the subsys-metric lines\n\
         scripts/subsys_metrics.sh --self-test  # prove the verifier CAN go red\n\
         \n\
         cargo test -p c2-harness --lib subsys  # the same checks, plus the 4 controls\n\
         cargo run -p c2-harness --bin c2rs -- subsys\n\
         ```\n\n\
         **No toolchain, no capture, no scan.** The instrument reads\n\
         `docs/whitebox/ref/` and the committed workload section census and prints, so\n\
         it degrades cleanly by construction: an absent census makes every output\n\
         proxy read `NO-DATA`, never `0`.\n\n\
         ### 9.1 `#1406`, and why this is not in `gate.sh`\n\n\
         `#1406` binds any instrument whose output is quoted as evidence to run under\n\
         `cargo test` or `scripts/gate.sh`. §0 forbids the second. The resolution is\n\
         `decode-reach`'s, and it is the reason this file's numbers are trustworthy\n\
         without the gate grading them: **the logic and the controls live in\n\
         `crates/c2-harness/src/subsys.rs` and run under `cargo test --workspace`,\n\
         which is a `gate.sh` row.** The verdict they contribute to is `cargo test`'s\n\
         — that every denominator here still reproduces from the tree — never the\n\
         differential's. `scripts/subsys_metrics.sh` is a thin wrapper over the same\n\
         code, so there is **one producer** of the table and it cannot drift from the\n\
         tests that grade it.\n"
    );
}

// ---------------------------------------------------------------------------
// Tests — including the POSITIVE CONTROLS, watched failing (#3336)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        crate::provenance::repo_root()
    }

    fn ref_dir() -> PathBuf {
        root().join(REF_DIR)
    }

    /// THE CHECK. Every denominator in the shipped table still reproduces from
    /// this tree. This is the `#1406` placement: the instrument's own grading
    /// runs under `cargo test`, never in `gate.sh`'s verdict.
    #[test]
    fn every_denominator_reproduces_on_this_tree() {
        let v = verify(&root(), &ref_dir(), SUBSYSTEMS);
        assert!(
            v.ok(),
            "per-subsystem metric table no longer reproduces:\n  {}",
            v.failures.join("\n  ")
        );
        assert_eq!(SUBSYSTEMS.len(), 10, "SUBSYS.md §1 has ten rows");
        assert_eq!(
            v.recounted.len(),
            SUBSYSTEMS.iter().filter(|s| s.basis == Basis::Band).count()
        );
    }

    /// POSITIVE CONTROL 1 — **a fabricated denominator must go RED.**
    /// A control never seen failing is decoration (`#3336`).
    #[test]
    fn control_a_fabricated_denominator_is_caught() {
        let mut table: Vec<Subsystem> = SUBSYSTEMS.to_vec();
        let i = table.iter().position(|s| s.key == "inline").unwrap();
        assert_eq!(table[i].sites, 93);
        table[i].sites = 94; // off by one, the cheapest lie
        let v = verify(&root(), &ref_dir(), &table);
        assert!(!v.ok(), "an off-by-one denominator was NOT caught");
        assert!(
            v.failures.iter().any(|f| f.contains("DOES NOT REPRODUCE")),
            "caught, but not by the recount: {:?}",
            v.failures
        );
    }

    /// POSITIVE CONTROL 2 — **a dropped subsystem row must go RED.** This is
    /// the failure decision 15's scoreboard is most exposed to: a subsystem
    /// silently stops being tracked and the table still looks complete.
    #[test]
    fn control_a_dropped_subsystem_is_caught() {
        let table: Vec<Subsystem> = SUBSYSTEMS
            .iter()
            .filter(|s| s.key != "eh")
            .cloned()
            .collect();
        assert_eq!(table.len(), 9);
        let v = verify(&root(), &ref_dir(), &table);
        assert!(!v.ok(), "a missing subsystem row was NOT caught");
        assert!(
            v.failures
                .iter()
                .any(|f| f.contains("P_EH.md") && f.contains("missing a subsystem")),
            "caught, but not as a missing subsystem: {:?}",
            v.failures
        );
    }

    /// POSITIVE CONTROL 3 — **an empty residue must go RED.** Decision 15's
    /// rule is that a strength this lane cannot measure prints a *named*
    /// residue: never silence, never 0.
    #[test]
    fn control_an_empty_residue_is_caught() {
        let mut table: Vec<Subsystem> = SUBSYSTEMS.to_vec();
        let i = table.iter().position(|s| s.key == "dag").unwrap();
        table[i].ported = Cell::Residue("   ");
        let v = verify(&root(), &ref_dir(), &table);
        assert!(!v.ok(), "an empty residue was NOT caught");
        assert!(
            v.failures.iter().any(|f| f.contains("EMPTY residue")),
            "{:?}",
            v.failures
        );
    }

    /// POSITIVE CONTROL 4 — **a coverage line that moved must go RED.** The
    /// numbers here are carried from the pages; the probe is what makes a
    /// carried number fail instead of rot.
    #[test]
    fn control_a_moved_coverage_line_is_caught() {
        let mut table: Vec<Subsystem> = SUBSYSTEMS.to_vec();
        let i = table.iter().position(|s| s.key == "coff").unwrap();
        table[i].den_probe = "21 of the 121 functions in the `coff.c`/`coffemit.c` band";
        let v = verify(&root(), &ref_dir(), &table);
        assert!(!v.ok(), "a moved coverage line was NOT caught");
        assert!(
            v.failures.iter().any(|f| f.contains("den_probe not found")),
            "{:?}",
            v.failures
        );
    }

    /// POSITIVE CONTROL 5 — **a fabricated `ported` must go RED.** The strength
    /// decision 15 named *first* was a residue on all ten rows until this lane
    /// (`#3617`); the moment one row carries a number, the number needs the same
    /// thing every other denominator here has — something that recomputes it.
    /// `#3336`: a control never seen failing is decoration, and `w-submetric`'s
    /// own self-test caught its author's bad `sed` on run one.
    #[test]
    fn control_a_fabricated_ported_is_caught() {
        let mut table: Vec<Subsystem> = SUBSYSTEMS.to_vec();
        let i = table.iter().position(|s| s.key == "encode").unwrap();
        let (num, den, unit, source, caveat) = match &table[i].ported {
            Cell::Measured { num, den, unit, source, caveat } => {
                (*num, *den, *unit, *source, *caveat)
            }
            other => panic!("the encode row's ported is no longer measured: {other:?}"),
        };
        // **27 -> 29 on 2026-08-28, lane `w-encarms`** (board #3758): arms
        // `10bfa285` (form 7, `bl`) and `10bfa76a` (form 54, `mfspr`) were
        // adopted into `mop`. The pin is deliberate and stays a pin — it is
        // what makes this control a statement about a KNOWN cell rather than
        // about whatever the cell happens to be, and it went red on exactly the
        // commit that moved the number, which is the behaviour asked for.
        assert_eq!((num, den), (30, 79), "the shipped ported cell moved");
        // The cheapest lie: one more arm than the port implements.
        table[i].ported = Cell::Measured { num: num + 1, den, unit, source, caveat };
        let v = verify(&root(), &ref_dir(), &table);
        assert!(!v.ok(), "an inflated `ported` was NOT caught");
        assert!(
            v.failures
                .iter()
                .any(|f| f.contains("ported DOES NOT REPRODUCE")),
            "caught, but not by the recount: {:?}",
            v.failures
        );
    }

    /// POSITIVE CONTROL 6 — **a `ported` NUMBER with nothing able to recount it
    /// must go RED.** Control 5 only bites a row that declares a recount; this
    /// is the other door, and it is the one the next lane will walk through when
    /// it wants to put a number on the `coff` row.
    #[test]
    fn control_a_ported_number_with_no_recount_is_caught() {
        let mut table: Vec<Subsystem> = SUBSYSTEMS.to_vec();
        let i = table.iter().position(|s| s.key == "coff").unwrap();
        assert!(table[i].ported_recount.is_none());
        table[i].ported = Cell::Measured {
            num: 21,
            den: 120,
            unit: "functions",
            source: "invented",
            caveat: "invented",
        };
        let v = verify(&root(), &ref_dir(), &table);
        assert!(!v.ok(), "an unrecountable `ported` number was NOT caught");
        assert!(
            v.failures.iter().any(|f| f.contains("with no recount")),
            "caught, but not as unrecountable: {:?}",
            v.failures
        );
    }

    /// POSITIVE CONTROL 7 — **a fabricated `section` `ported` must go RED.**
    /// Control 5's twin, on the row this lane converted, and pinned to the check
    /// that must own the refusal so it cannot pass by being caught for the wrong
    /// reason.
    #[test]
    fn control_a_fabricated_section_ported_is_caught() {
        let mut table: Vec<Subsystem> = SUBSYSTEMS.to_vec();
        let i = table.iter().position(|s| s.key == "section").unwrap();
        let (num, den, unit, source, caveat) = match &table[i].ported {
            Cell::Measured { num, den, unit, source, caveat } => {
                (*num, *den, *unit, *source, *caveat)
            }
            other => panic!("the section row's ported is no longer measured: {other:?}"),
        };
        assert_eq!((num, den), (1, 15), "the shipped section ported cell moved");
        table[i].ported = Cell::Measured { num: num + 1, den, unit, source, caveat };
        let v = verify(&root(), &ref_dir(), &table);
        assert!(!v.ok(), "an inflated section `ported` was NOT caught");
        assert!(
            v.failures
                .iter()
                .any(|f| f.contains("section: ported DOES NOT REPRODUCE")),
            "caught, but not by the GLREC recount: {:?}",
            v.failures
        );
    }

    /// The section recount is **not** a count that is right for the wrong
    /// reason: it must name the arm it found, and that arm must be the shared
    /// tag-`0x04`/`0x0E`/`0x10` handler `0x10b9bdcf`, which
    /// `c2_il::func::glalias` decodes under `DISCLOSURE.md` W-ALIAS-1.
    ///
    /// This is where the dispatcher's real shape is pinned too — a lane that
    /// re-dumps the table and gets a different population reddens here rather
    /// than shipping a moved denominator quietly.
    #[test]
    fn the_section_ported_arm_is_the_one_the_port_actually_decodes() {
        let txt = std::fs::read_to_string(root().join(GLREC_ARMS_TSV))
            .expect("GLREC_ARMS.tsv");
        let t = parse_glrec_arms(&txt);
        assert_eq!(t.rows.len(), 27, "the 27 tag values 0x01..0x1B moved");
        assert_eq!(t.fatal, Some(0x10b9c5ca), "the fatal arm moved");
        assert_eq!(t.live_arms().len(), 15, "15 live arms + 1 refusal = 16 slots");
        assert_eq!(t.live_tags().len(), 19, "19 live tags, 8 fatal");
        // The refusal really does serve eight tags -- the fact that makes `27
        // arms` wrong.
        assert_eq!(t.rows.iter().filter(|r| Some(r.arm) == t.fatal).count(), 8);
        // Tag 0x09, the SECTION DEFINITION record, is live and is NOT the arm
        // the port decodes. If a future lane ports it, this line moves and the
        // number moves with it -- which is the whole point.
        let tag9 = t.rows.iter().find(|r| r.tag == 0x09).expect("tag 0x09");
        assert_eq!(tag9.arm, 0x10b9c212);
        assert_eq!(crates_cited(&root(), &[0x10b9c212]), Vec::<u32>::new());

        assert_eq!(crates_cited(&root(), &t.live_arms()), vec![0x10b9bdcf]);
    }

    /// **THE OBSERVER MUST NOT BE ABLE TO MOVE ITS OWN NUMBER.** `#3641`'s
    /// family: an instrument whose input is source text, writing about the
    /// addresses it counts. This file names every live arm in its own doc
    /// comments and in the control above; if the scan read `c2-harness` those
    /// mentions would score as implementations and the row would read 15 of 15.
    ///
    /// Asserted on the real tree rather than a fixture, because the mentions are
    /// really here: `subsys.rs` contains `10b9c212` (three times) and the scan
    /// must still report it uncited.
    #[test]
    fn the_observer_crate_cannot_move_its_own_ported() {
        let me = std::fs::read_to_string(
            root().join("crates/c2-harness/src/subsys.rs"),
        )
        .expect("this file");
        assert!(
            me.contains("10b9c212"),
            "this control is vacuous unless this file really names the address"
        );
        assert_eq!(
            crates_cited(&root(), &[0x10b9c212]),
            Vec::<u32>::new(),
            "the metric crate's own mention of an arm was counted as a port decoder"
        );
    }

    /// **CONTROL 8 — THE RENDERED TABLE MUST CARRY EVERY MEASURED NUMBER.**
    ///
    /// `#3665`: §3's `ported` column was the hard-coded string `ported RESIDUE`
    /// on all ten rows for two waves, so when `w-encmap` measured `encode` at
    /// 27/79 the table went on printing RESIDUE beside a §4 that said the
    /// opposite. **Every other control in this module fabricates a NUMBER; that
    /// one fabricated a WORD**, and nothing here could see it — which is the
    /// same blind spot `#3643` and `#3641` sit in.
    ///
    /// So this asserts the property rather than the string: for each measured
    /// row, the rendered markdown carries its numerator, its denominator and
    /// its key. Watched failing by reverting the cell to the literal.
    #[test]
    fn control_a_measured_ported_must_reach_the_rendered_table() {
        let r = render(&root(), None, SUBSYSTEMS);
        let measured: Vec<&Subsystem> = SUBSYSTEMS
            .iter()
            .filter(|s| matches!(s.ported, Cell::Measured { .. }))
            .collect();
        assert_eq!(measured.len(), 2, "encode and section are measured");
        // §3's tuple table is everything between its heading and §4's.
        let table = r
            .markdown
            .split("## 3. The tuple table")
            .nth(1)
            .and_then(|s| s.split("## 4. ").next())
            .expect("§3 exists");
        for sub in measured {
            let (num, den) = match sub.ported {
                Cell::Measured { num, den, .. } => (num, den),
                _ => unreachable!(),
            };
            let row = table
                .lines()
                .find(|l| l.contains(&format!("`{}` |", sub.key)))
                .unwrap_or_else(|| panic!("no §3 row for {}", sub.key));
            assert!(
                row.contains(&format!("ported {num} / {den}")),
                "{}: §3 does not carry the measured ported {num}/{den} -- \
                 the tuple table and §4 disagree, which is #3665: {row}",
                sub.key
            );
            // ...and §4 must carry the caveat VERBATIM. A denominator
            // published only in this file's source is not published, and
            // `w-encmap`'s 79-arm justification lived exactly there for a
            // whole wave.
            let caveat = match sub.ported {
                Cell::Measured { caveat, .. } => caveat,
                _ => unreachable!(),
            };
            let head: String = caveat.split_whitespace().take(8).collect::<Vec<_>>().join(" ");
            assert!(
                r.markdown.contains(&head),
                "{}: the measured ported caveat does not reach the published doc \
                 -- looked for {head:?}",
                sub.key
            );
        }
    }

    /// The `ported` predicate is **not** vacuous: the three readings it
    /// publishes must actually differ, or the caveat is decoration. Strict < the
    /// published number < planned-only, on this tree.
    #[test]
    fn the_three_ported_readings_are_distinct() {
        let txt = std::fs::read_to_string(ref_dir().join(ENCODE_ARMS_TXT))
            .expect("ENCODE_ARMS.txt");
        let arms = parse_encode_arms(&txt);
        assert_eq!(arms.len(), 79, "the arm enumeration moved");
        let total_forms: usize = arms.iter().map(|a| a.forms.len()).sum();
        assert_eq!(total_forms, 111, "the 111 jump-table entries moved");
        let (n, d) = recount_encode_ported(&arms);
        assert_eq!(d, 79);
        let strict = recount_encode_ported_strict(&arms);
        let planned = recount_encode_ported_planned(&arms);
        assert!(
            strict < n && n < planned,
            "the three readings collapsed: strict {strict}, published {n}, planned {planned} \
             — if they are equal the caveat is telling the reader nothing"
        );
    }

    /// The endpoint convention is not decoration: two rows only reproduce
    /// under one convention and flipping it must move the count.
    #[test]
    fn the_endpoint_convention_is_load_bearing() {
        let funcs =
            std::fs::read_to_string(ref_dir().join("FUNCS.tsv")).expect("FUNCS.tsv");
        let addrs = funcs_addresses(&funcs);
        let regalloc = Band { lo: 0x10b2c21d, hi: 0x10b3219f, end: End::HalfOpen };
        let flipped = Band { end: End::Inclusive, ..regalloc };
        assert_eq!(recount_band(&addrs, &[regalloc]), 70);
        assert_eq!(
            recount_band(&addrs, &[flipped]),
            71,
            "P_REGALLOC's 70 reproduces ONLY half-open"
        );
        let coff = Band { lo: 0x10b281af, hi: 0x10b2b0dd, end: End::Inclusive };
        let flipped = Band { end: End::HalfOpen, ..coff };
        assert_eq!(recount_band(&addrs, &[coff]), 120);
        assert_eq!(
            recount_band(&addrs, &[flipped]),
            119,
            "P_COFF's 120 reproduces ONLY inclusive"
        );
    }

    /// The section census's own known-answer control: `nsec` against the
    /// parsed `order` length, on every record. A parser that drops names must
    /// not be able to report a plausible small number.
    #[test]
    fn the_section_census_control_holds_or_the_file_is_absent() {
        let root = crate::provenance::repo_root();
        let c = section_census(&root);
        if !c.present {
            eprintln!("SKIP: {SECTIONS_JSONL} absent from this tree");
            return;
        }
        assert_eq!(
            c.nsec_disagree, 0,
            "{} record(s) whose parsed section list disagrees with their own nsec",
            c.nsec_disagree
        );
        assert_eq!(c.tus, 871);
        assert_eq!(c.sections, 393_236);
        assert_eq!(c.by_name.len(), 14);
        assert_eq!(c.by_name.get(".pdata").copied(), Some((849, 103_128)));
        // Independent corroboration of P_EH's own "67 workload objs, all STLport".
        assert_eq!(c.by_name.get(".xdata$x").map(|v| v.0), Some(67));
    }

    /// The mark census rule reproduces the numbers the shipped doc quotes.
    #[test]
    fn the_mark_census_reproduces() {
        let v = verify(&root(), &ref_dir(), SUBSYSTEMS);
        assert_eq!(v.marks.len(), 10);
        let eh = v.marks["eh"];
        assert_eq!((eh.read, eh.obj, eh.inferred), (27, 14, 0));
        let gr = v.marks["globregs"];
        assert_eq!((gr.read, gr.obj, gr.inferred), (49, 21, 4));
    }

    /// §0's fifth property: no key may be a ratio over zero, and the renderer
    /// must never print a bare `0` for an unmeasured strength.
    #[test]
    fn no_strength_prints_a_bare_zero() {
        let root = crate::provenance::repo_root();
        let r = render(&root, None, SUBSYSTEMS);
        assert!(r.verified.ok(), "{:?}", r.verified.failures);
        for sub in SUBSYSTEMS {
            let ex = exercised_cell(sub, &r.census);
            match ex {
                Cell::Measured { den, .. } => assert!(den > 0),
                Cell::Residue(s) | Cell::Pending(s) => assert!(!s.trim().is_empty()),
            }
        }
        assert!(r.text.contains("LICENSE NO EMIT") || r.text.contains("LICENSE NO EMIT"));
        for trap in TRAPS {
            let first = trap.split_whitespace().take(4).collect::<Vec<_>>().join(" ");
            assert!(
                r.text.contains(&first),
                "trap missing from the render: {first}"
            );
        }
    }

    /// The keys are namespaced and sorted by NAME, never by mass (`#3505`).
    #[test]
    fn keys_are_namespaced_and_name_sorted() {
        let root = crate::provenance::repo_root();
        let r = render(&root, None, SUBSYSTEMS);
        let k = keys(SUBSYSTEMS, &r.verified, &r.census);
        let mut sorted = k.clone();
        sorted.sort();
        assert_eq!(k, sorted, "keys must be sorted by name");
        assert!(k.iter().any(|l| l.starts_with("byte-owned CITED-3534")));
        assert!(k.iter().any(|l| l == "verify-failures 0"));
    }
}

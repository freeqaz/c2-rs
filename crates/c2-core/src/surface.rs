//! **The decision-surface registry** — the checkable form of board `#3723`.
//!
//! Lane `w-doctrine`, wave 17 (`docs/DECISIONS_2026-08-22.md` § Decision 21 §2).
//!
//! # What this exists to catch, in one paragraph
//!
//! `docs/rungs/README.md` grades a construct rung on a **required-zero byte
//! delta**, diffed line-for-line over the gate's 21 per-lane rows (board
//! `#290`'s pattern). **That criterion passes a real emit widening.** Lane
//! `w-regsel` planted control **C6** — the caller's allowed register set opened
//! from the volatiles to `r0..r31`, so c2's callee-saved tail becomes reachable
//! from a production path — and measured the result: **471 of 475 crate tests
//! still passed, no encoder row moved, `GATE: PASS` at both ends, and the
//! identity diff read 0 lines over 21 rows.** The widening would have shipped.
//!
//! The reason is not subtle and it is not fixable inside the byte delta. **The
//! gate can only see emissions the corpus exercises.** A widening whose new
//! emissions are unexercised is invisible to it — board `#1236`'s shape, a
//! guard green precisely because the offender is out of scope.
//!
//! # What this module is
//!
//! A registry of the port's **decision surfaces**: the places where the port
//! chooses among alternatives or refuses. Each surface enumerates a domain that
//! deliberately **extends past what the corpus reaches** and renders one
//! canonical row per point. The rendered whole is committed as
//! `surface/DOMAIN.txt` and compared by a test.
//!
//! So a change to a registered surface cannot land silently. Either the domain
//! is unchanged — and the test is green for a reason, not by tautology — or it
//! moved, the test is red, and the only way forward is to re-bless the baseline,
//! **which puts the widening in the diff as text a reviewer can read**. That is
//! the whole mechanism: the instrument does not decide whether a widening is
//! right, it makes one impossible to make by accident.
//!
//! # The four things that are asserted, and why each one exists
//!
//! | # | assertion | the failure it exists for |
//! |---|---|---|
//! | **E1** | the rendered domain equals `surface/DOMAIN.txt` | `#3723` itself — a byte-neutral widening |
//! | **E2** | the source markers and the registry are a **bijection** | `#3641` — a rename silently emptying the population. A registry nothing points at grades nothing and is green |
//! | **E3** | every surface meets a **minimum cell and refusal count** | `#3470` — a check over zero cells is green and says nothing. A surface that refuses nothing is not a refusal boundary |
//! | **E4** | every boundary-named `const` in this crate is **covered or listed** | the only hole E1 cannot reach: the registry's own completeness. E4 does not close it — it makes it unable to grow quietly (`#3689`'s ratchet) |
//!
//! # What this module is NOT, and the line is `docs/FUNCTION_BYTE_MATCH.md` §0
//!
//! **A progress/method instrument, never a judge.** Nothing here is in
//! `scripts/gate.sh`'s verdict, nothing here licenses an emit, and a green
//! domain is not evidence that the port is right about anything. The sole judge
//! of the port stays real `c2.dll` under wibo plus a byte-exact obj compare.
//! The byte delta is **not weakened** by any of this: it stays necessary, and
//! what `#3723` established is that it is not sufficient.
//!
//! # Where the row generators live, and why they are not here
//!
//! Each surface's `rows` function lives **in the module it characterizes**,
//! next to the code and next to its marker. Two reasons, and the second is
//! load-bearing: the rows are a statement about that module, and
//! `codegen::regalloc`'s cost fence
//! (`the_only_cost_array_the_port_constructs_is_zero_and_the_call_sites_are_enumerated`)
//! scans every other file in the crate for `regalloc::select` call sites and
//! for the non-default order names. Enumerating the orders from here would trip
//! it — correctly, because from here it would look exactly like a new consumer.

use std::collections::BTreeSet;

/// One point of a surface's domain, and what the port does there.
///
/// `outcome` is either the refusal marker [`REFUSE`] (optionally followed by a
/// reason) or a comma-separated list of the **values the port would emit** at
/// that point. The token list is what makes the summary line able to say which
/// values are reachable at all, which is the question `#3723` is about.
pub struct Row {
    /// The domain point, rendered so it sorts and reads stably.
    pub point: String,
    /// `REFUSE`, `REFUSE <reason>`, or a comma-separated emitted-value list.
    pub outcome: String,
}

impl Row {
    /// Build a row. `outcome` is passed through verbatim.
    pub fn new(point: impl Into<String>, outcome: impl Into<String>) -> Row {
        Row { point: point.into(), outcome: outcome.into() }
    }

    fn is_refusal(&self) -> bool {
        self.outcome == REFUSE || self.outcome.starts_with("REFUSE ")
    }
}

/// The refusal marker. A surface that never emits this is not a refusal
/// boundary and [`Surface::min_refusals`] is what says so.
///
/// PROV[N] an instrument token; reaches no emitted byte.
pub const REFUSE: &str = "REFUSE";

/// A registered decision surface.
pub struct Surface {
    /// Stable identity. Also the token the source marker must carry.
    pub name: &'static str,
    /// The file the marker must appear in, relative to `crates/c2-core/src`.
    pub site: &'static str,
    /// One line: which boundary this is.
    pub boundary: &'static str,
    /// Where the boundary comes from — a doc section, a board row, an address.
    pub cite: &'static str,
    /// The boundary-named `const`s this surface's domain actually exercises.
    /// E4 reads this as the coverage claim, so an entry here that the domain
    /// does not reach is a false coverage claim and the control set is what
    /// keeps it honest.
    pub guards: &'static [&'static str],
    /// The domain.
    pub rows: fn() -> Vec<Row>,
    /// E3's floor on the number of cells.
    pub min_cells: usize,
    /// E3's floor on the number of refusals.
    pub min_refusals: usize,
}

/// **The registry.**
///
/// Four surfaces, deliberately from three unrelated families — register
/// allocation, frame layout and branch reach — because a registry that only
/// covered `w-regsel`'s own grid would be that lane's test moved to a shared
/// file rather than a general instrument.
///
/// PROV[N] an instrument registry; reaches no emitted byte.
pub const SURFACES: &[Surface] = &[
    Surface {
        name: "alloc.allocate",
        site: "codegen/alloc.rs",
        boundary: "which registers a store run's producers get, and on which \
                   (producer count, pool floor) the allocation REFUSES",
        cite: "docs/ALLOC.md; board #543, #541; the C6 control that motivated #3723",
        guards: &["MAX_MODELLED_PRODUCERS", "VOLATILE_GPR_TOP"],
        rows: crate::codegen::alloc::surface_rows,
        min_cells: 256,
        min_refusals: 100,
    },
    Surface {
        name: "regalloc.select",
        site: "codegen/regalloc.rs",
        boundary: "c2's minimum-cost selector over an ordered register list — \
                   which register wins, and when the allowed set is exhausted",
        cite: "docs/whitebox/ref/P_REGALLOC.md §3; c2 `0x10b2e7f8`; DISCLOSURE W-REGSEL-1",
        guards: &[],
        rows: crate::codegen::regalloc::surface_rows,
        min_cells: 224,
        min_refusals: 25,
    },
    Surface {
        name: "frame.out_of_class",
        site: "codegen/frame.rs",
        boundary: "which frame layouts the prologue/epilogue emitter admits, \
                   and the named reason it refuses the rest",
        cite: "docs/CODEGEN_FRAMED_CALLS.md §1.2; the helper thresholds 3 (GPR) and 4 (FPR)",
        guards: &["FRAME_MAX_SAVED_NO_SPILL", "FRAME_MIN_OUT_SLOTS"],
        rows: crate::codegen::frame::surface_rows,
        min_cells: 504,
        min_refusals: 200,
    },
    Surface {
        name: "reach.branch",
        site: "codegen/reach.rs",
        boundary: "whether a branch displacement is emitted direct, expanded, \
                   or refused as unmeasured",
        cite: "docs/CFG_SHAPE.md §3.3.1; board #290, #3119",
        guards: &["BC_MAX_DISP", "B_MAX_DISP"],
        rows: crate::codegen::reach::surface_rows,
        min_cells: 75,
        min_refusals: 15,
    },
    // -- APPENDED 2026-08-28 by lane `w-encarms`, the registry's FIRST OUTSIDE
    //    CONSUMER. Kept as one additive block at the end of the array so the
    //    merge against `w-inlbudget`, which owns this file the same wave, is a
    //    trivial one. Nothing above this line was touched.
    Surface {
        name: "mop.encode_form",
        site: "codegen/mop.rs",
        boundary: "which of c2's 113 encode FORMS the port can place fields for, \
                   the word each one composes at a canonical operand assignment, \
                   and the displacement domain where form 7's `bl` mask diverges \
                   from c2's",
        cite: "docs/whitebox/ref/P_ENCODE.md §5, §10; c2 arms `0x10bfa285` (form 7) \
               and `0x10bfa76a` (form 54); DISCLOSURE W-ENCARMS-1; board #3760, #3761",
        // No guard named. `MAX_FIELDS` is the obvious candidate and is the WRONG
        // one: it is a capacity bound on this port's own `FieldPlan` array, not a
        // c2 boundary, `UNCOVERED` already says so, and claiming it here would be
        // a false coverage claim of exactly the shape `POOL_TOP` was found to be.
        guards: &[],
        rows: crate::codegen::mop::surface_rows,
        min_cells: 113,
        min_refusals: 60,
    },
];

/// Boundary-named `const`s this crate carries that **no registered surface
/// covers**, each with the reason it is not covered.
///
/// This is E4's hole, written down. It is not an allowlist of things that do
/// not matter — several of these are real emit boundaries with no enumerated
/// domain yet, and saying so is the point. [`UNCOVERED_RATCHET`] is what stops
/// the list growing without anyone noticing (`#3689`: a number printed on every
/// run drifted 16 to 18 inside one wave because printing is not gating).
///
/// PROV[N] an instrument's own coverage table; reaches no emitted byte.
pub const UNCOVERED: &[(&str, &str)] = &[
    ("DS_MAX", "the `ds`-form displacement field; a real boundary, no enumerated domain yet"),
    ("HEAD_SLOTS_MAX", "a shape-recogniser cap, not an emit boundary"),
    ("K_ASCII_MAX", "an XTEA-fixture constant; a value, not a decision"),
    ("K_TWO_MAX", "an XTEA-fixture constant; a value, not a decision"),
    ("LITERAL_TEXT_BYTE_LIMIT", "a real refusal boundary in the string-literal path; not enumerated yet"),
    ("MAX_C2_OPCODE", "the opcode table's length; a denominator, not a decision"),
    ("MAX_FIELDS", "a COFF parse cap; not an emit boundary"),
    ("MAX_MULTISYM_PRODUCERS", "a real refusal boundary in the multi-symbol run path; not enumerated yet"),
    ("MAX_OBJECTS_PER_SECTION", "a COFF layout cap; not enumerated yet"),
    ("MAX_SYMBOL_CROSSINGS", "a real refusal boundary in the symbol-binding path; not enumerated yet"),
    ("POOL_TOP", "MEASURED, not assumed: a DERIVED alias of `GPR_DEFAULT.regs[0]` with no production use outside tests. Re-spelling it as a literal `9` moves ZERO domain lines, so naming it a guard was a false coverage claim. What it aliases — the order's head — is covered by `regalloc.select`"),
    ("R_BOUND", "FALSE POSITIVE of the name screen — a REGISTER NUMBER whose name ends in `_BOUND`"),
    ("TOP", "FALSE POSITIVE of the name screen — a loop-top byte OFFSET named `TOP`"),
];

/// The ceiling on [`UNCOVERED`]. Raising it is a one-line edit and that is
/// fine; what is now impossible is raising it **silently**, which is the only
/// thing that ever actually happens (`#3689`).
///
/// PROV[N] an instrument ceiling; reaches no emitted byte.
pub const UNCOVERED_RATCHET: usize = 13;

/// Render the whole registry to the canonical text that `surface/DOMAIN.txt`
/// holds.
///
/// Deterministic by construction: the registry order is source order, each
/// surface's rows are emitted in the order its generator produced them, and the
/// summary's value set is sorted.
pub fn render() -> String {
    let mut out = String::new();
    out.push_str(
        "# c2-rs DECISION-SURFACE DOMAIN — GENERATED, DO NOT HAND-EDIT.\n\
         #\n\
         # Regenerate:  cargo test -p c2-core --lib surface::tests::bless -- --ignored\n\
         #\n\
         # This file is the answer to board #3723: a required-zero BYTE delta\n\
         # passes an emit widening whose new emissions the corpus never\n\
         # exercises. Every line below is a point of a decision surface that the\n\
         # corpus does NOT have to reach, so a widening moves lines here even\n\
         # when it moves no byte and no gate row.\n\
         #\n\
         # A line that changed is not automatically a defect. It is a claim that\n\
         # has to be read: the port now chooses, or refuses, differently.\n\
         #\n\
         # `REFUSE` is a refusal; anything else is the comma-separated list of\n\
         # values the port would emit at that point.\n",
    );
    for s in SURFACES {
        let rows = (s.rows)();
        out.push_str("\n## surface  ");
        out.push_str(s.name);
        out.push('\n');
        out.push_str("##   site      ");
        out.push_str(s.site);
        out.push('\n');
        out.push_str("##   boundary  ");
        out.push_str(&squash(s.boundary));
        out.push('\n');
        out.push_str("##   cite      ");
        out.push_str(&squash(s.cite));
        out.push('\n');
        out.push_str("##   guards    ");
        let guards = s.guards.join(" ");
        out.push_str(if guards.is_empty() { "(none named)" } else { &guards });
        out.push('\n');
        let mut values: BTreeSet<&str> = BTreeSet::new();
        let mut refusals = 0usize;
        for r in &rows {
            if r.is_refusal() {
                refusals += 1;
            } else {
                for tok in r.outcome.split(',') {
                    values.insert(tok);
                }
            }
            out.push_str(s.name);
            out.push_str("  ");
            out.push_str(&r.point);
            out.push_str("  ");
            out.push_str(&r.outcome);
            out.push('\n');
        }
        out.push_str(&format!(
            "## summary  {}  cells={} admit={} refuse={} values={{{}}}\n",
            s.name,
            rows.len(),
            rows.len() - refusals,
            refusals,
            values.into_iter().collect::<Vec<_>>().join(","),
        ));
    }
    out
}

/// Collapse a Rust string continuation's whitespace so a multi-line literal
/// renders as one line.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    /// The committed baseline.
    const BASELINE: &str = include_str!("surface/DOMAIN.txt");

    fn src_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    /// Every `.rs` under `crates/c2-core/src`, path relative to that root.
    ///
    /// **This module is IN the population and is not excluded**, which is
    /// deliberate: `codegen::regalloc`'s cost fence had to exclude itself
    /// because it greps for a token it must contain to do the grepping, and an
    /// exclusion is a hole. Here the marker token is assembled at run time from
    /// two halves (see [`marker`]), so this file contains no literal marker and
    /// needs no exclusion — the property is guaranteed by construction rather
    /// than asserted about a list.
    fn crate_sources() -> Vec<(String, String)> {
        let root = src_root();
        let mut out = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).expect("c2-core/src readable").flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    let rel = p.strip_prefix(&root).unwrap().to_string_lossy().into_owned();
                    out.push((rel, std::fs::read_to_string(&p).expect("source readable")));
                }
            }
        }
        out.sort();
        out
    }

    /// The source marker, assembled so that this file does not contain it.
    fn marker() -> String {
        format!("{}{}", "SURFA", "CE[")
    }

    // -- E1 -----------------------------------------------------------------

    /// **E1 — the domain is what the baseline says it is.**
    ///
    /// This is the assertion `#3723` asks for. `w-regsel`'s control C6 is
    /// byte-neutral, gate-neutral and identity-diff-neutral; it is **not**
    /// neutral here, and `work/w-doctrine/controls_red.txt` records the run.
    #[test]
    fn the_decision_surface_domain_matches_the_committed_baseline() {
        let live = render();
        if live == BASELINE {
            return;
        }
        let l: Vec<&str> = live.lines().collect();
        let b: Vec<&str> = BASELINE.lines().collect();
        let mut moved = 0usize;
        let mut shown = String::new();
        for i in 0..l.len().max(b.len()) {
            let (a, c) = (b.get(i).copied().unwrap_or("<absent>"), l.get(i).copied().unwrap_or("<absent>"));
            if a != c {
                moved += 1;
                if moved <= 40 {
                    shown.push_str(&format!("  - {a}\n  + {c}\n"));
                }
            }
        }
        panic!(
            "THE DECISION-SURFACE DOMAIN MOVED — {moved} line(s) of {} differ from \
             crates/c2-core/src/surface/DOMAIN.txt.\n\n\
             This is board #3723's check. A required-zero byte delta and a 0-line \
             gate identity diff are BOTH silent about the lines below: they are \
             points the corpus does not reach.\n\n\
             If the change is intended, re-bless the baseline\n\
             (cargo test -p c2-core --lib surface::tests::bless -- --ignored)\n\
             and the widening becomes a reviewable text diff, which is the whole \
             point. If it is not intended, you have just been told about an emit \
             widening no fixture would have caught.\n\n{shown}",
            b.len().max(l.len())
        );
    }

    /// Rewrite the baseline. `#[ignore]`d so it never runs in the suite: a
    /// blessing has to be an explicit act, and it lands as a diff.
    #[test]
    #[ignore = "regenerates the committed baseline; run deliberately"]
    fn bless_the_domain_baseline() {
        let path = src_root().join("surface/DOMAIN.txt");
        std::fs::write(&path, render()).expect("baseline writable");
        eprintln!("blessed {}", path.display());
    }

    // -- E2 -----------------------------------------------------------------

    /// **E2 — the markers and the registry are a bijection.**
    ///
    /// A registry entry whose marker was renamed away grades nothing and is
    /// green; a marker naming no entry is a surface somebody meant to register
    /// and did not. `#3641`'s shape, in both directions.
    #[test]
    fn every_surface_marker_names_a_registered_surface_and_back() {
        let m = marker();
        let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut scanned = 0usize;
        for (path, src) in crate_sources() {
            scanned += 1;
            let mut rest = src.as_str();
            while let Some(i) = rest.find(&m) {
                rest = &rest[i + m.len()..];
                let Some(j) = rest.find(']') else {
                    panic!("{path}: an unterminated surface marker");
                };
                found.entry(rest[..j].to_string()).or_default().push(path.clone());
            }
        }
        assert!(scanned > 50, "the source scan found only {scanned} files — it is not reading the crate");

        let registered: BTreeSet<&str> = SURFACES.iter().map(|s| s.name).collect();
        for (name, paths) in &found {
            assert!(
                registered.contains(name.as_str()),
                "{paths:?} carries a surface marker for `{name}`, which is not in \
                 c2_core::surface::SURFACES. A marked surface that is not \
                 registered is enumerated nowhere and graded by nothing."
            );
        }
        for s in SURFACES {
            let paths = found.get(s.name).unwrap_or_else(|| {
                panic!(
                    "surface `{}` is registered but NO source carries its marker. \
                     A registry nothing points at grades nothing and is green \
                     (#3641).",
                    s.name
                )
            });
            assert!(
                paths.iter().any(|p| p.replace('\\', "/") == s.site),
                "surface `{}` declares site `{}` but its marker is in {paths:?}",
                s.name,
                s.site
            );
        }
        assert_eq!(found.len(), SURFACES.len(), "marker set and registry disagree in size");
    }

    // -- E3 -----------------------------------------------------------------

    /// **E3 — no surface grades zero, and no surface refuses nothing.**
    ///
    /// `#3470`: only a denominator catches an absence. A domain that silently
    /// became empty renders as an empty block and would compare equal to a
    /// baseline blessed in the same state.
    #[test]
    fn every_surface_has_a_nonempty_domain_and_a_real_refusal_boundary() {
        assert!(!SURFACES.is_empty(), "an empty registry passes everything");
        for s in SURFACES {
            let rows = (s.rows)();
            assert!(
                rows.len() >= s.min_cells,
                "surface `{}`: {} cells, floor is {}",
                s.name,
                rows.len(),
                s.min_cells
            );
            let refusals = rows.iter().filter(|r| r.is_refusal()).count();
            assert!(
                refusals >= s.min_refusals,
                "surface `{}`: {refusals} refusals, floor is {}. A surface that \
                 refuses (almost) nothing is not a refusal boundary — either the \
                 domain no longer reaches past what the port admits, or the \
                 boundary moved.",
                s.name,
                s.min_refusals
            );
            let admits = rows.len() - refusals;
            assert!(admits > 0, "surface `{}` admits nothing at all", s.name);
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for r in &rows {
                assert!(
                    seen.insert(r.point.as_str()),
                    "surface `{}` renders the point `{}` twice — the domain is not \
                     a set and a diff over it is not readable",
                    s.name,
                    r.point
                );
            }
        }
    }

    // -- E4 -----------------------------------------------------------------

    /// Every `const NAME` in the crate whose name is boundary-shaped.
    ///
    /// The screen is a NAME screen and it is therefore both incomplete (a
    /// boundary called `POOL_CEIL_UNSPELLED` is missed) and noisy (two entries
    /// of [`UNCOVERED`] are false positives, and say so). It is a ratchet on a
    /// hole, not a proof that the hole is closed.
    fn boundary_named_consts() -> BTreeSet<String> {
        const WORDS: &[&str] = &["MAX", "MIN", "TOP", "LIMIT", "CEILING", "FLOOR", "THRESHOLD", "BOUND", "CAP"];
        let mut out = BTreeSet::new();
        for (_, src) in crate_sources() {
            for line in src.lines() {
                let t = line.trim();
                if t.starts_with("//") {
                    continue;
                }
                let Some(i) = ["const ", "pub const ", "pub(crate) const ", "pub(super) const "]
                    .iter()
                    .find(|p| t.starts_with(**p))
                    .map(|p| p.len() - 6)
                else {
                    continue;
                };
                let name: String = t[i + 6..]
                    .chars()
                    .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                    .collect();
                if name.len() < 2 || !t[i + 6 + name.len()..].starts_with(':') {
                    continue;
                }
                if name.split('_').any(|w| WORDS.contains(&w)) {
                    out.insert(name);
                }
            }
        }
        out
    }

    /// **E4 — the registry's coverage hole cannot grow quietly.**
    #[test]
    fn every_boundary_named_const_is_covered_by_a_surface_or_listed_uncovered() {
        let found = boundary_named_consts();
        assert!(
            found.len() >= 15,
            "the boundary-name screen found only {} consts — it has stopped \
             reading the crate, and a screen over nothing is green (#3470)",
            found.len()
        );

        let covered: BTreeSet<&str> = SURFACES.iter().flat_map(|s| s.guards.iter().copied()).collect();
        let listed: BTreeSet<&str> = UNCOVERED.iter().map(|(n, _)| *n).collect();
        assert_eq!(listed.len(), UNCOVERED.len(), "UNCOVERED names a const twice");

        for n in &covered {
            assert!(!listed.contains(n), "`{n}` is both covered and listed uncovered");
            assert!(found.contains(*n), "surface guard `{n}` is not a const this crate defines");
        }
        for n in &listed {
            assert!(found.contains(*n), "UNCOVERED names `{n}`, which this crate no longer defines — delete the row and lower the ratchet");
        }
        for (_, why) in UNCOVERED {
            assert!(!why.trim().is_empty(), "an UNCOVERED row with no reason is an allowlist entry pretending to be a finding");
        }

        let mut unaccounted: Vec<&str> = found
            .iter()
            .map(|s| s.as_str())
            .filter(|n| !covered.contains(n) && !listed.contains(n))
            .collect();
        unaccounted.sort();
        assert!(
            unaccounted.is_empty(),
            "NEW BOUNDARY-NAMED CONST(S) {unaccounted:?} in c2-core, covered by no \
             registered decision surface and listed in no UNCOVERED row.\n\n\
             A boundary constant is where an emit widening is spelled — #3723's \
             C6 was one token of one. Either register a surface whose domain \
             reaches it, or add it to c2_core::surface::UNCOVERED with the \
             reason it is not covered and raise UNCOVERED_RATCHET, which puts \
             the decision in the diff."
        );

        assert!(
            UNCOVERED.len() <= UNCOVERED_RATCHET,
            "UNCOVERED is {} rows against a ratchet of {UNCOVERED_RATCHET}",
            UNCOVERED.len()
        );
        eprintln!(
            "surface coverage: {} boundary-named consts, {} covered by {} surfaces, {} listed uncovered (ratchet {UNCOVERED_RATCHET})",
            found.len(),
            covered.len(),
            SURFACES.len(),
            UNCOVERED.len()
        );
    }

    /// The rendered artifact is text, is stable across two calls, and carries
    /// nothing `CLAUDE.md` forbids in a committed file.
    #[test]
    fn the_rendered_domain_is_deterministic_and_committable() {
        assert_eq!(render(), render(), "the renderer is not deterministic");
        let r = render();
        assert!(!r.contains('\0'), "a NUL byte in a committed text artifact");
        assert!(!r.contains("/home/"), "an absolute machine path in a committed artifact");
        assert!(r.lines().count() > 700, "the artifact is implausibly short: {} lines", r.lines().count());
    }
}

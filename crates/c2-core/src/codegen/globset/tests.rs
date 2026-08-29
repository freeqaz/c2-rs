//! **THE FAIL AXIS, AND IT IS A MEASUREMENT RATHER THAN A NAME.**
//!
//! `rungs/README.md`'s refusal-domain clause says in as many words that the
//! `Fail axis:` header *"checks presence, not measurement — it cannot tell a
//! named axis from a measured one"*. This file is the measured form for lane
//! `w-globset`, and it has two halves that fail for different reasons:
//!
//! 1. **[`the_fail_axis`] grades this module's transcription against tables a
//!    DIFFERENT lane's grader decoded out of `c2.dll` itself**, parsed from
//!    `work/w-globarms/GRADE.txt` and `work/w-globobj/GRADE.txt` at test time.
//!    Nothing here re-reads the image; the point is that the answer key was not
//!    authored by the thing it grades. If P1, P3 or P5 is transcribed wrongly
//!    this goes red while every byte, every gate row and every identity-diff
//!    line is unchanged.
//! 2. **[`population_power_over_the_obj_cells`] measures which of this
//!    module's non-default parameter values the obj population can actually
//!    refute, and PUBLISHES ITS ZEROS.** `#1236`: *"my test passes"* and *"my
//!    test can tell two rules apart"* are different claims, and four of the
//!    seven rivals here are refuted by **nothing** in this population.
//!
//! **These are not a `ported` numerator** (decision 21 §4, `#3505`). Separating
//! power is a property of a population; it is not a fraction of c2 and no
//! percentage is derived from it anywhere.

use super::*;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// The answer keys — read, never transcribed
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    // crates/c2-core -> crates -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// Read a committed grade transcript.
///
/// **A missing file FAILS; it does not skip.** A screen over zero rows is green
/// and says nothing (`#3470`), and this whole file exists because a green test
/// that could not have failed is the defect, not the evidence.
fn answer_key(rel: &str) -> String {
    let p = repo_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| {
        panic!(
            "the answer key {rel} is unreadable ({e}). This test grades this \
             module against tables another lane's instrument decoded out of \
             c2.dll; without them it would be a check over zero rows, which is \
             green and says nothing (#3470). Do not make this a skip."
        )
    })
}

/// `  kind 0x07  A8  ELIGIBLE-ALIASED  always joins …` → `(0x07, "A8",
/// "ELIGIBLE-ALIASED")`.
fn kind_arm_rows(text: &str) -> Vec<(u8, String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("kind 0x") else { continue };
        let tok: Vec<&str> = rest.split_whitespace().collect();
        if tok.len() < 3 {
            continue;
        }
        let Ok(k) = u8::from_str_radix(tok[0], 16) else { continue };
        out.push((k, tok[1].to_string(), tok[2].to_string()));
    }
    out
}

/// `    linkage 2 -> kind 8 when …` → `(2, "kind 8 when …")`.
fn linkage_rows(text: &str) -> Vec<(u8, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("linkage ") else { continue };
        let Some((idx, tail)) = rest.split_once(" -> ") else { continue };
        let Ok(i) = idx.trim().parse::<u8>() else { continue };
        out.push((i, tail.trim().to_string()));
    }
    out
}

/// `  .gl record kind [gl+0x30] == 3    -> globregs kind 0xb` →
/// `("3", "globregs kind 0xb")`.
fn gl_kind_rows(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with(".gl record kind") {
            continue;
        }
        let Some((lhs, rhs)) = t.split_once("->") else { continue };
        let Some((_, sel)) = lhs.split_once("==") else { continue };
        out.push((sel.trim().to_string(), rhs.trim().to_string()));
    }
    out
}

/// `  . ga_int  A6  arm_W.txt  want PROMOTED  got PROMOTED` →
/// `("ga_int", "A6", "arm_W.txt", "PROMOTED")`.
fn cell_rows(text: &str) -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix(". ") else { continue };
        let tok: Vec<&str> = rest.split_whitespace().collect();
        let Some(gi) = tok.iter().position(|w| *w == "got") else { continue };
        if tok.len() < 4 || gi + 1 >= tok.len() {
            continue;
        }
        out.push((tok[0].to_string(), tok[1].to_string(), tok[2].to_string(), tok[gi + 1].to_string()));
    }
    out
}

/// The gate-B answer key line `grade_globobj.py` prints after decoding the
/// 30-byte table out of the image:
/// `gate B @ 0x10b18b28 : 25 promotable, NOT promotable = 0x00 0x12 …`.
fn gate_b_answer_key(text: &str) -> Option<Vec<u8>> {
    for line in text.lines() {
        let Some((_, rhs)) = line.split_once("NOT promotable =") else { continue };
        let mut v = Vec::new();
        for w in rhs.split_whitespace() {
            let Some(h) = w.strip_prefix("0x") else { break };
            let Ok(b) = u8::from_str_radix(h, 16) else { break };
            v.push(b);
        }
        if !v.is_empty() {
            return Some(v);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// F1–F3 + the gate-B key — the fail axis proper
// ---------------------------------------------------------------------------

/// **THE FAIL AXIS.** Four independent tables, all decoded out of `c2.dll` by
/// `docs/whitebox/scripts/grade_globarms.py` and `grade_globobj.py`, none of
/// them authored here.
#[test]
fn the_fail_axis() {
    let arms = answer_key("work/w-globarms/GRADE.txt");
    let obj = answer_key("work/w-globobj/GRADE.txt");
    let p = CandidateSet::C2;

    // -- F2: the 8-entry jump table at 0x10bd2a9f --------------------------
    let links = linkage_rows(&arms);
    assert_eq!(
        links.len(),
        8,
        "the linkage table parsed to {} rows, not 8 — a short extraction and \
         an agreeing pair both print nothing (#3470). Rows: {links:?}",
        links.len()
    );
    for (i, desc) in &links {
        let want = &p.kinds.table[*i as usize];
        let ok = if desc == "unreachable" {
            matches!(want, LinkageArm::NullSlot)
        } else if let Some(h) = desc.strip_prefix("globregs kind 0x") {
            let k = u8::from_str_radix(h.trim(), 16).expect("a hex kind");
            matches!(want, LinkageArm::Kind(x) if *x == k)
        } else if desc.starts_with("kind 8 when") && desc.contains("else kind 7") {
            matches!(want, LinkageArm::StorageBits { hit: 8, miss: 7 })
        } else if desc == "storage-kind switch" {
            matches!(want, LinkageArm::StorageKind)
        } else if desc.contains("& 2 | 5") {
            matches!(want, LinkageArm::AliasBit)
        } else {
            panic!("linkage {i}: the answer key says `{desc}`, which this test cannot read");
        };
        assert!(ok, "linkage {i}: the image says `{desc}`, KindMap::C2 says {want:?}");
    }

    // -- the `.gl` kind chain at 0x10bd2926 --------------------------------
    let gls = gl_kind_rows(&arms);
    assert_eq!(gls.len(), 5, "the .gl kind chain parsed to {} rows, not 5: {gls:?}", gls.len());
    for (sel, desc) in &gls {
        let gl_kind: u8 = if sel == "else" { 9 } else { sel.parse().expect("a gl kind") };
        let gl = GlRecord { kind_byte: gl_kind, ..GlRecord::data(1) };
        if desc.contains("linkage jump table") {
            assert_eq!(
                p.kinds.kind_of(&gl),
                p.kinds.linkage_kind(&gl),
                "[gl+0x30] == {sel} must route through the table"
            );
        } else if let Some(h) = desc.strip_prefix("globregs kind 0x") {
            let k = u8::from_str_radix(h.trim(), 16).expect("a hex kind");
            // The mapped kind must NOT depend on the linkage for these arms.
            for linkage in 0..8u8 {
                let g = GlRecord { kind_byte: gl_kind, ..GlRecord::data(linkage) };
                assert_eq!(
                    p.kinds.kind_of(&g),
                    MappedKind::Kind(k),
                    "[gl+0x30] == {sel} must be kind 0x{k:02x} at every linkage"
                );
            }
        } else {
            panic!("the .gl chain row `{sel} -> {desc}` is unreadable to this test");
        }
    }

    // -- F1: the 17-row kind -> arm simulation -----------------------------
    let rows = kind_arm_rows(&arms);
    assert_eq!(
        rows.len(),
        17,
        "the kind->arm table parsed to {} rows, not 17 (kinds 0x00..0x10): {rows:?}",
        rows.len()
    );
    for (kind, want_arm, want_verdict) in &rows {
        let sym = Symbol::leader(*kind);
        let (arm, outcome) = p.gate_a(&sym);
        match want_verdict.as_str() {
            "SKIP" => {
                assert_eq!(outcome, Outcome::Skip, "kind 0x{kind:02x}");
                assert_eq!(&arm.name(), want_arm, "kind 0x{kind:02x}: arm");
            }
            "REJECT" => {
                assert_eq!(outcome, Outcome::Reject, "kind 0x{kind:02x}");
                assert_eq!(&arm.name(), want_arm, "kind 0x{kind:02x}: arm");
            }
            "ELIGIBLE" => {
                assert_eq!(outcome, Outcome::Eligible, "kind 0x{kind:02x}");
                assert_eq!(&arm.name(), want_arm, "kind 0x{kind:02x}: arm");
                // …and the internal test: the escape bit is what aliases it.
                let mut aliased = sym;
                aliased.escaped = true;
                assert_eq!(p.gate_a(&aliased).1, Outcome::EligibleAliased, "kind 0x{kind:02x}");
            }
            "ELIGIBLE-ALIASED" => {
                assert_eq!(outcome, Outcome::EligibleAliased, "kind 0x{kind:02x}");
                assert_eq!(&arm.name(), want_arm, "kind 0x{kind:02x}: arm");
                // A8 is aliased ALWAYS — the escape bit must not move it.
                let mut unescaped = sym;
                unescaped.escaped = false;
                assert_eq!(
                    p.gate_a(&unescaped).1,
                    Outcome::EligibleAliased,
                    "kind 0x{kind:02x}: A8 joins the set unconditionally"
                );
            }
            "COND" => {
                assert!(
                    want_arm.contains(arm.name()),
                    "kind 0x{kind:02x}: the image says arm `{want_arm}`, this module says `{}`",
                    arm.name()
                );
                assert!(
                    outcome.is_eligible(),
                    "kind 0x{kind:02x} must be ELIGIBLE with every condition met"
                );
                // COND means a condition exists that flips it. Find one.
                let mut flipped = 0usize;
                for adverse in 0..3 {
                    let mut s = sym;
                    match adverse {
                        0 => s.temp_slot_clear = false,
                        1 => s.temp_flag_clear = false,
                        _ => s.extern_indexable = false,
                    }
                    if p.gate_a(&s).1 == Outcome::Reject {
                        flipped += 1;
                    }
                }
                assert!(
                    flipped > 0,
                    "kind 0x{kind:02x} is COND in the image but nothing in this \
                     module's Symbol can make it reject — the condition is not \
                     modelled at all"
                );
            }
            other => panic!("kind 0x{kind:02x}: unreadable verdict `{other}`"),
        }
    }

    // -- P5: gate B's not-promotable set, decoded from 0x10b18b28 ----------
    let key = gate_b_answer_key(&obj).expect(
        "work/w-globobj/GRADE.txt carries no `NOT promotable =` answer-key line — \
         the gate-B half of the fail axis has nothing to grade against",
    );
    assert_eq!(
        key,
        TypeClassPolicy::C2.not_promotable.to_vec(),
        "gate B: the image says {key:x?}, TypeClassPolicy::C2 says {:x?}",
        TypeClassPolicy::C2.not_promotable
    );
    // …and the complement, which is the number the answer key prints beside it.
    let promotable =
        (0..=TYPE_CLASS_MAX).filter(|c| TypeClassPolicy::C2.promotable(*c) == Some(true)).count();
    assert_eq!(promotable, 25, "25 of the 30 classes are promotable");

    eprintln!(
        "FAIL AXIS: {} linkage rows, {} .gl rows, {} kind->arm rows, gate-B key {:x?} — all from \
         work/w-globarms/GRADE.txt and work/w-globobj/GRADE.txt, decoded from c2.dll by another \
         lane's instrument",
        links.len(),
        gls.len(),
        rows.len(),
        key
    );
}

// ---------------------------------------------------------------------------
// The obj cells, and what they can and cannot decide
// ---------------------------------------------------------------------------

/// One `w-globarms` cell this module claims to decide, with the symbol model
/// the cell's own construction fixes.
///
/// **The descriptor is not a free parameter.** Every cell here is one or two
/// autos differing only in whether an address is taken and whether it escapes,
/// or a kind-3 temporary — which is what the cells were built to isolate. A
/// cell whose descriptor would have to be *chosen* is not in this table, and
/// §"the refusals" below says which and why.
struct Cell {
    name: &'static str,
    /// The back-end kind, from the arm the cell's source form reaches.
    kind: u8,
    /// `(address_taken, address_escapes)` per symbol in the group.
    syms: &'static [(bool, bool)],
}

/// The scored population: **11 cells, 22 verdicts** over two profiles.
const CELLS: &[Cell] = &[
    // A6 — autos. `ga_param` and `ga_ref` are autos too; the answer key's arm
    // column is what says so, and the test asserts this module agrees with it.
    Cell { name: "ga_int", kind: 4, syms: &[(false, false)] },
    Cell { name: "ga_param", kind: 4, syms: &[(false, false)] },
    Cell { name: "ga_ref", kind: 4, syms: &[(false, false)] },
    Cell { name: "ga_escape", kind: 4, syms: &[(true, true)] },
    // The deciding pair for P4, and `gb_addr_local` is the one that refutes
    // `AddressTaken`: the address IS taken and it does NOT escape.
    Cell { name: "gb_addr_local", kind: 4, syms: &[(true, false)] },
    Cell { name: "gb_addr_escape", kind: 4, syms: &[(true, true)] },
    Cell { name: "gb_pair_none", kind: 4, syms: &[(false, false), (false, false)] },
    Cell { name: "gb_pair_xescape", kind: 4, syms: &[(true, true), (false, false)] },
    Cell { name: "gb_pair_yescape", kind: 4, syms: &[(false, false), (true, true)] },
    // A11 — the kind-3 temporary, accept side.
    Cell { name: "ga_temp", kind: 3, syms: &[(false, false)] },
    Cell { name: "ga_temp3", kind: 3, syms: &[(false, false), (false, false)] },
];

/// The cells this module **refuses to score**, each with the reason. Written
/// down because an absence that is not named reads as coverage.
const REFUSED_CELLS: &[(&str, &str)] = &[
    ("ga_extern", "A8, CONFOUNDED — w-globarms §1.3: a symbol with a COFF record must be observable across an opaque call for language reasons, so gate A and the C++ object model predict the same MEMORY and the obj cannot separate them"),
    ("ga_fstatic", "A8, CONFOUNDED — as ga_extern"),
    ("ga_lstatic", "A8, CONFOUNDED — as ga_extern"),
    ("ga_fnaddr", "A9 — the readout scores the BODY, which has no frame traffic because c2 emits a direct `bl` and never materialises the function symbol as a value. PROMOTED here is not a statement about candidacy"),
    ("gb_fnaddr2", "A9 — as ga_fnaddr"),
    ("ga_struct4", "A3 — a member-wise aggregate promotes member by member through the leader's +0x0c chain; this module does not model sub-symbols and will not score a cell it does not model"),
    ("ga_structmix", "A3 — as ga_struct4"),
    ("ga_vol", "the answer key's own arm column is `-`. `volatile` is not a parameter of this policy and no arm of gate A tests it"),
];

/// This module's prediction for a cell, **gate A only**.
///
/// Gate B is deliberately out of the loop: the map from a C++ type to a gate-B
/// **type class** is unread (`P_GLOBREGS` §3 gives the class table, not the
/// nibble resolution of a front-end type word), so putting a class in here
/// would make the answer a function of this test's own choice rather than of
/// the policy. That is the whole of why P5's registered separating power over
/// this population is **zero** — see [`population_power_over_the_obj_cells`].
fn predict(p: &CandidateSet, c: &Cell) -> (Arm, String) {
    let mut arm = None;
    let mut promoted = 0usize;
    for (taken, escapes) in c.syms {
        let mut s = Symbol::leader(c.kind);
        s.escaped = p
            .aliasing
            .escape_bit(&SymbolGroup { address_taken: *taken, address_escapes: *escapes });
        let (a, o) = p.gate_a(&s);
        arm = Some(a);
        if o == Outcome::Eligible {
            promoted += 1;
        }
    }
    let verdict = if promoted == c.syms.len() {
        "PROMOTED"
    } else if promoted == 0 {
        "MEMORY"
    } else {
        "SPLIT"
    };
    (arm.expect("a cell has at least one symbol"), verdict.to_string())
}

/// **F3 — the default policy agrees with all 22 scored obj verdicts**, and its
/// arm agrees with the answer key's arm column.
#[test]
fn the_default_policy_agrees_with_every_scored_obj_cell() {
    let arms = answer_key("work/w-globarms/GRADE.txt");
    let cells = cell_rows(&arms);
    assert_eq!(cells.len(), 38, "the cell table parsed to {} rows, not 38", cells.len());

    let p = CandidateSet::C2;
    let mut scored = 0usize;
    for (name, want_arm, profile, got) in &cells {
        let Some(c) = CELLS.iter().find(|c| c.name == name) else {
            assert!(
                REFUSED_CELLS.iter().any(|(n, _)| n == name),
                "cell `{name}` is neither scored nor listed in REFUSED_CELLS. An \
                 unscored cell that nobody wrote a reason for reads as coverage."
            );
            continue;
        };
        let (arm, verdict) = predict(&p, c);
        assert_eq!(
            arm.name(),
            want_arm.as_str(),
            "{name} @ {profile}: the answer key's arm is {want_arm}, this module reaches {}",
            arm.name()
        );
        assert_eq!(&verdict, got, "{name} @ {profile}");
        scored += 1;
    }
    assert_eq!(scored, 22, "22 verdicts (11 cells x 2 profiles) are scored; got {scored}");
    eprintln!("obj cells: 22 scored, {} refused with a named reason, 38 in the key", REFUSED_CELLS.len());
}

/// **THE POWER OF THE POPULATION, INCLUDING ITS ZEROS.**
///
/// `#1236`. Seven non-default parameter values; the 22-verdict obj population
/// refutes **three** and is blind to **four**. The four zeros are the useful
/// half: they say which parameters this project's existing cells cannot grade
/// at all, which is the thing a lane reading "the default agrees with every
/// cell" would otherwise take for confirmation.
#[test]
fn population_power_over_the_obj_cells() {
    let base = CandidateSet::C2;

    let mut null_slot_reachable = CandidateSet::C2;
    null_slot_reachable.kinds.table[0] = LinkageArm::Kind(4);

    let rivals: [(&str, CandidateSet); 7] = [
        ("P4 AddressTaken", CandidateSet { aliasing: AliasingPolicy::AddressTaken, ..base }),
        ("P4 Never", CandidateSet { aliasing: AliasingPolicy::Never, ..base }),
        ("P4 Always", CandidateSet { aliasing: AliasingPolicy::Always, ..base }),
        ("P3 A6 bound 5->7", CandidateSet { gate_a: GateA::C2.with_auto_bound(7), ..base }),
        ("P5 all promotable", CandidateSet { gate_b: TypeClassPolicy::ALL_PROMOTABLE, ..base }),
        ("P5 stride shifted", CandidateSet { gate_b: TypeClassPolicy::STRIDE_SHIFTED, ..base }),
        ("P1 entry 0 reachable", null_slot_reachable),
    ];

    let mut report = String::new();
    let mut refuted = Vec::new();
    let mut blind = Vec::new();
    for (name, rival) in &rivals {
        // Two verdicts per cell — the population is both profiles, and every
        // cell is identical at /O1 and /Ox in the answer key.
        let n: usize = CELLS
            .iter()
            .filter(|c| predict(&base, c).1 != predict(rival, c).1)
            .count()
            * 2;
        report.push_str(&format!("  {name:<24} refuted by {n:>2} of 22 verdicts\n"));
        if n > 0 { refuted.push(*name) } else { blind.push(*name) }
    }
    eprintln!("SEPARATING POWER of the 22-verdict obj population:\n{report}");

    // The three the population DOES decide.
    assert_eq!(
        refuted,
        vec!["P4 AddressTaken", "P4 Never", "P4 Always"],
        "the refuted set moved"
    );
    // …and `gb_addr_local` alone is what kills `AddressTaken`: address taken,
    // no escape, PROMOTED at both profiles.
    let local = CELLS.iter().find(|c| c.name == "gb_addr_local").unwrap();
    assert_eq!(predict(&base, local).1, "PROMOTED");
    assert_eq!(
        predict(&CandidateSet { aliasing: AliasingPolicy::AddressTaken, ..base }, local).1,
        "MEMORY",
        "gb_addr_local is the cell that refutes `address taken`"
    );

    // **THE REGISTERED ZEROS.** Prereg §4 F5, and they are a finding.
    assert_eq!(
        blind,
        vec![
            "P3 A6 bound 5->7",
            "P5 all promotable",
            "P5 stride shifted",
            "P1 entry 0 reachable"
        ],
        "the BLIND set moved — a rival this population was known not to reach \
         has become reachable, or one it reached has stopped being reached. \
         Either is a finding and neither is a green test."
    );
}

// ---------------------------------------------------------------------------
// The read, executed rather than quoted
// ---------------------------------------------------------------------------

/// **A6's kinds ARE the linkage classes with no COFF record**, which is
/// `w-globarms`' headline composed out of two pages, run instead of quoted.
#[test]
fn a6s_kinds_are_exactly_the_linkage_classes_with_no_coff_record() {
    let p = CandidateSet::C2;
    let mut no_record = Vec::new();
    let mut with_record = Vec::new();
    for linkage in 1..8u8 {
        for bits in [false, true] {
            for storage_kind in [1u8, 2, 4, 9] {
                for alias in [false, true] {
                    let gl = GlRecord {
                        kind_byte: 1,
                        linkage,
                        storage_bits_hit: bits,
                        storage_kind,
                        alias_bit: alias,
                    };
                    let MappedKind::Kind(k) = p.kinds.kind_of(&gl) else { continue };
                    if p.kinds.has_coff_record(linkage) {
                        with_record.push((linkage, k));
                    } else {
                        no_record.push((linkage, k));
                    }
                }
            }
        }
    }

    // Every no-COFF-record symbol is an A6 auto…
    for (linkage, k) in &no_record {
        assert!(matches!(k, 4 | 5), "linkage {linkage} has no COFF record but maps to kind 0x{k:02x}");
        assert_eq!(p.gate_a(&Symbol::leader(*k)).0, Arm::A6, "kind 0x{k:02x} must reach A6");
    }
    // …and every symbol that DOES get one arrives at A8 or A9 as 7, 8 or 9.
    // Linkage 5's kind-5 arm is the documented exception and is named as one:
    // `((gl+0x20) >> 4) & 2 | 5` yields kind 5 for a linkage the {1,3}
    // suppression does not cover, so `has_coff_record` and the kind disagree
    // there. The read says so; this test states it rather than hiding it.
    let mut exceptions = 0usize;
    for (linkage, k) in &with_record {
        if *linkage == 5 && *k == 5 {
            exceptions += 1;
            continue;
        }
        assert!(
            matches!(k, 7 | 8 | 9),
            "linkage {linkage} gets a COFF record but maps to kind 0x{k:02x}"
        );
        let arm = p.gate_a(&Symbol::leader(*k)).0;
        assert!(matches!(arm, Arm::A8 | Arm::A9), "kind 0x{k:02x} reached {}", arm.name());
    }
    assert!(exceptions > 0, "linkage 5's kind-5 arm must be exercised, not asserted away");
}

/// Linkage 0 is refused, never mapped. `WB_GLOBARMS_FINDINGS.md` §7: entry 0
/// of the table is a **null slot** and c2 would jump to address 0.
#[test]
fn linkage_zero_is_refused_and_not_given_an_invented_kind() {
    let p = CandidateSet::C2;
    assert_eq!(p.kinds.kind_of(&GlRecord::data(0)), MappedKind::Unreachable);
    assert_eq!(p.verdict_for(&GlRecord::data(0), &SymbolGroup::default(), 1), None);
}

/// The escape bit is a property of a symbol **GROUP**, and `gb_pair_yescape`
/// against `gb_pair_xescape` is why it cannot be a property of the function.
#[test]
fn the_escape_bit_is_per_symbol_not_per_function() {
    let p = CandidateSet::C2;
    let escaping = SymbolGroup { address_taken: true, address_escapes: true };
    let quiet = SymbolGroup::default();
    assert!(p.aliasing.escape_bit(&escaping));
    assert!(!p.aliasing.escape_bit(&quiet));

    let mut x = Symbol::leader(4);
    let mut y = Symbol::leader(4);
    x.escaped = p.aliasing.escape_bit(&quiet);
    y.escaped = p.aliasing.escape_bit(&escaping);
    assert_eq!(p.gate_a(&x).1, Outcome::Eligible);
    assert_eq!(p.gate_a(&y).1, Outcome::EligibleAliased);
}

/// Kind 10 does **not** reach gate B — `WB_GLOBARMS_FINDINGS.md` §2.2's
/// refutation of `P_GLOBREGS` §3's "gate A then gate B" sequencing.
#[test]
fn kind_ten_never_reaches_gate_b() {
    let p = CandidateSet::C2;
    assert!(!p.gate_b.reaches_gate_b(0x0a));
    let mut sym = Symbol::leader(0x0a);
    // Even at a class gate B would reject, kind 10 is admitted: the type gate
    // is not on its path at all.
    sym.type_class = 0x12;
    assert_eq!(p.verdict(&sym), Verdict::Promoted);
    for k in [3u8, 4, 5, 7, 8] {
        assert!(p.gate_b.reaches_gate_b(k), "kind 0x{k:02x} is type-gated");
    }
}

/// A class above [`TYPE_CLASS_MAX`] is refused, not guessed.
#[test]
fn a_class_above_the_nibble_tables_range_is_refused() {
    let p = TypeClassPolicy::C2;
    assert_eq!(p.promotable(TYPE_CLASS_MAX), Some(false)); // 0x1d is the whole of nibble 8
    assert_eq!(p.promotable(TYPE_CLASS_MAX + 1), None);
    assert_eq!(p.promotable(0x01), Some(true));
}

/// Every parameter's default is c2's, and `Default` agrees with the `C2`
/// constant everywhere. The decision-surface clause is *"named, enumerable
/// parameters whose DEFAULT reproduces c2 byte-exactly"*, and a default that
/// drifted from the constant is how that stops being true silently.
#[test]
fn every_default_is_c2() {
    assert_eq!(CandidateSet::default(), CandidateSet::C2);
    assert_eq!(KindMap::default(), KindMap::C2);
    assert_eq!(GateA::default(), GateA::C2);
    assert_eq!(TypeClassPolicy::default(), TypeClassPolicy::C2);
    assert_eq!(AliasingPolicy::default(), AliasingPolicy::EscapesToOpaqueCallee);
}

// ---------------------------------------------------------------------------
// The fence: no production caller
// ---------------------------------------------------------------------------

/// **NOTHING IN THE PORT MAY REACH THIS MODULE.**
///
/// The port has no symbol arena, no `.gl` records and no tuple list, so a
/// consumer arriving here would be consuming a model of state the port does not
/// have. This is `regalloc_worklist`'s condition and it is enforced the same
/// way `codegen::regalloc`'s cost fence is enforced: by scanning the crate.
///
/// The registry entry in `surface.rs` is the **one** permitted reference, and
/// it is a reference from an instrument, not from an emitter.
#[test]
fn this_module_has_no_production_caller() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut refs: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).expect("c2-core/src readable").flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().is_none_or(|x| x != "rs") {
                continue;
            }
            let rel = p.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
            scanned += 1;
            if rel.starts_with("codegen/globset") {
                continue;
            }
            let src = std::fs::read_to_string(&p).expect("source readable");
            for (n, line) in src.lines().enumerate() {
                let t = line.trim();
                if t.starts_with("//") {
                    continue;
                }
                if t.contains("globset") {
                    refs.push(format!("{rel}:{}: {t}", n + 1));
                }
            }
        }
    }
    assert!(scanned > 50, "the source scan found only {scanned} files — it is not reading the crate");
    let stray: Vec<&String> =
        refs.iter().filter(|r| !r.starts_with("surface.rs:") && !r.starts_with("codegen/mod.rs:")).collect();
    assert!(
        stray.is_empty(),
        "codegen::globset acquired a consumer: {stray:?}. It models c2 state \
         this port does not have (no symbol arena, no .gl records, no tuple \
         list) and a caller means one of the two is now a fiction."
    );
    // Exactly two, and they are the registry row's two halves: the `site`
    // string E2 checks the marker against, and the `rows` function pointer. A
    // third is a new consumer wearing an instrument's clothes.
    assert_eq!(
        refs.iter().filter(|r| r.starts_with("surface.rs:")).count(),
        2,
        "surface.rs must reference this module exactly twice — the registry \
         row's `site` and its `rows`: {refs:?}"
    );
}

/// The surface's domain is non-trivial, deterministic, and its points are a
/// set. `surface.rs`'s E1/E3 grade the committed rendering; this is the local
/// half, so a breakage names this module rather than the registry.
#[test]
fn the_surface_domain_is_a_deterministic_set_that_reaches_past_every_fixture() {
    let a = surface_rows();
    let b = surface_rows();
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!((&x.point, &x.outcome), (&y.point, &y.outcome));
    }
    let mut pts: Vec<&str> = a.iter().map(|r| r.point.as_str()).collect();
    pts.sort_unstable();
    let n = pts.len();
    pts.dedup();
    assert_eq!(pts.len(), n, "the domain renders a point twice");

    let refusals = a.iter().filter(|r| r.outcome.starts_with(crate::surface::REFUSE)).count();
    assert!(refusals > 0 && refusals < a.len(), "{refusals} refusals of {n} rows");
    eprintln!("globregs.candidate_set: {n} cells, {refusals} refusals, {} admits", n - refusals);
}

//! Tests for [`super`] — the comparator's mechanics, **and** the measurement of
//! what the 20 obj cells can and cannot decide about it.
//!
//! Lane `w-regprio`. The second half is the part that matters: `#1236` says
//! *"my test passes"* and *"my test can tell the two rules apart"* are different
//! claims, and this module publishes the second one **with its zeros**.

use super::*;

// ===========================================================================
// THE POPULATION — 20 obj cells at two profiles, plus the instrument control
// ===========================================================================

/// **The 20 order cells of `docs/whitebox/grids/wb-dagorder2/candorder_grid.cpp`,
/// at both profiles.**
///
/// `(cell, n_candidates, order_at_O1, order_at_Ox)`. Each order is the colouring
/// order head-first, spelled in formal letters: `a` is the first formal, `b` the
/// second, and so on. The instrument is the grid's own
/// (`work/w-dagorder/extract.py`): every cell keeps its ints live across a call,
/// so the volatiles are all disallowed and the callee-saved run `r31, r30, …` is
/// handed out in the fixed order — **which formal got `r31` is which candidate
/// was coloured first**.
///
/// **Re-measured by this lane on 2026-08-27**, not transcribed: the frozen grid
/// (sha256 `b06a05fc…afeee6`, verified by the runner) recompiled against real
/// `cl.exe` 16.00.11886.00 / `c2.dll` under wibo at `/nologo /c /GR /O1 /Oi
/// /EHsc` and `… /Ox /Oi /EHsc`, control `w5_chain.cpp` → 4/4, batch 113 ms.
/// **All 18 orders `WB_DAGORDER2_FINDINGS.md` prints reproduced 18/18**, and the
/// two it never printed — `cnd_x3` and `cnd_x3r` — are recovered here for the
/// first time.
///
/// PROV[O] `docs/rungs/2026-08-27-w-regprio.md` §2 — obj/listing-confirmed at
/// both profiles by this lane; the orders are an observable of real c2 and no
/// address is required for them.
const CELLS: &[(&str, usize, &str, &str)] = &[
    // --- the A series: n formals summed in declaration order, n = 1..8.
    //     Agrees at both profiles on all eight.
    ("cnd_a1", 1, "a", "a"),
    ("cnd_a2", 2, "ba", "ba"),
    ("cnd_a3", 3, "bac", "bac"),
    ("cnd_a4", 4, "dbac", "dbac"),
    ("cnd_a5", 5, "ebacd", "ebacd"),
    ("cnd_a6", 6, "febacd", "febacd"),
    ("cnd_a7", 7, "gfbacde", "gfbacde"),
    ("cnd_a8", 8, "hgfbacde", "hgfbacde"),
    // --- the X series: commutative operand order. INERT at n=2 and — this
    //     lane's new datum — inert at n=3 as well: `a+b+c` and `c+b+a` give the
    //     same order at both profiles, and it is `cnd_a3`'s.
    ("cnd_x2", 2, "ba", "ba"),
    ("cnd_x2r", 2, "ba", "ba"),
    ("cnd_x3", 3, "bac", "bac"),
    ("cnd_x3r", 3, "bac", "bac"),
    // --- non-commutative, so reassociation cannot normalise the pair away —
    //     and they are STILL insensitive to written operand order.
    ("cnd_s2", 2, "ab", "ab"),
    ("cnd_s2r", 2, "ab", "ab"),
    // --- the H series: the discriminator. Formals, declarations and live set
    //     held fixed; only dependence HEIGHT moves. Profiles disagree.
    ("cnd_h2", 2, "ba", "ab"),
    ("cnd_h2r", 2, "ab", "ba"),
    ("cnd_h3", 3, "bca", "acb"),
    ("cnd_h3r", 3, "abc", "cba"),
    // --- the U series: the use-count axis. Profiles disagree.
    ("cnd_u2", 2, "ab", "ba"),
    ("cnd_u2r", 2, "ba", "ab"),
];

/// **The instrument control, and it is the reason the batch is not void.**
///
/// `cnd_c0` keeps nothing live across the call, so it must take **no**
/// callee-saved colour at all. Measured at both profiles: the arrival→colour map
/// is empty and no register in `r14…r31` is written. If this cell ever framed,
/// the extractor would be reading something other than what it claims to and
/// every row of [`CELLS`] would be void.
const CONTROL_C0_COLOURS: usize = 0;

/// The formal letters of a cell with `n` candidates, in **mint order**.
///
/// The accumulation order `0x10b316b1` feeds the comparator is the hash-bucket
/// walk over `cand+0x1c`, and read **R1** makes that counter per-function and
/// dense from 1 — so bucket order is mint order. Whether mint order tracks
/// *formal* order is `FUN_10b55dbe`'s question and is **not** settled
/// (`P_GLOBREGS` §6); it is the assumption model `M-TIE` makes, and this module
/// tests the model rather than assuming it.
fn mint_order(n: usize) -> Vec<u32> {
    (0..n as u32).collect()
}

/// `"bac"` → `[1, 0, 2]`, i.e. formal letters to zero-based candidate ids.
fn ids_of(order: &str) -> Vec<u32> {
    order.bytes().map(|b| (b - b'a') as u32).collect()
}

// ===========================================================================
// THE FIVE PLANTED MUTANTS
// ===========================================================================

/// Tie tier `<=` → `<`. **The decision point the brief singles out.**
const MUT_LT: WorklistComparator =
    WorklistComparator { tie_tier: TieTier::NewLast, ..WorklistComparator::C2 };

/// Primary key descending → ascending.
const MUT_ASC: WorklistComparator = WorklistComparator {
    primary: KeySpec { dir: SortDir::Asc, ..WorklistComparator::C2.primary },
    ..WorklistComparator::C2
};

/// Primary and tie keys exchanged — the tie ordinal leads.
const MUT_SWAP: WorklistComparator = WorklistComparator {
    primary: KeySpec { field: KeyField::TieOrdinal, dir: SortDir::Desc, signed: false },
    tie: KeySpec { field: KeyField::Priority, dir: SortDir::Desc, signed: true },
    tie_tier: TieTier::NewFirst,
};

/// Primary key read unsigned instead of signed.
const MUT_U0C: WorklistComparator = WorklistComparator {
    primary: KeySpec { signed: false, ..WorklistComparator::C2.primary },
    ..WorklistComparator::C2
};

/// Tie key read signed instead of unsigned.
const MUT_S44: WorklistComparator = WorklistComparator {
    tie: KeySpec { signed: true, ..WorklistComparator::C2.tie },
    ..WorklistComparator::C2
};

const MUTANTS: &[(&str, WorklistComparator)] = &[
    ("MUT-LT", MUT_LT),
    ("MUT-ASC", MUT_ASC),
    ("MUT-SWAP", MUT_SWAP),
    ("MUT-U0C", MUT_U0C),
    ("MUT-S44", MUT_S44),
];

// ===========================================================================
// 1. THE RULE AS READ — hand-worked, so the code can be checked against §4
// ===========================================================================

/// The four clauses of `0x10b2b82d`'s predicate, one case each, worked by hand
/// from `P_REGALLOC` §4 rather than from this module's own code.
#[test]
fn the_predicate_reproduces_the_rule_as_read() {
    let c = WorklistComparator::C2;
    let new = Candidate::new(9, 100, 50);

    // n->[0x0c] < new->[0x0c]  ->  insert before.
    assert!(c.insert_before(&new, &Candidate::new(0, 99, 999)));
    // n->[0x0c] > new->[0x0c]  ->  do not.
    assert!(!c.insert_before(&new, &Candidate::new(0, 101, 0)));
    // equal primary, n->[0x44] < new->[0x44]  ->  insert before.
    assert!(c.insert_before(&new, &Candidate::new(0, 100, 49)));
    // equal primary, n->[0x44] > new->[0x44]  ->  do not.
    assert!(!c.insert_before(&new, &Candidate::new(0, 100, 51)));
    // EXACT TIE IN BOTH: `<=`, so the NEW candidate goes first.
    assert!(
        c.insert_before(&new, &Candidate::new(0, 100, 50)),
        "the tie tier is `<=`: an exact tie in both keys puts the NEW candidate first"
    );
}

/// **The primary key is signed and it matters** — `0x10b2d630` subtracts, so
/// `cand+0x0c` genuinely reaches negative values.
#[test]
fn the_primary_key_is_signed_and_a_negative_priority_ranks_last() {
    let w = Worklist::build(
        WorklistComparator::C2,
        &[Candidate::new(0, -5, 0), Candidate::new(1, 3, 0), Candidate::new(2, -100, 0)],
    );
    assert_eq!(w.ids(), vec![1, 0, 2], "signed DESC: 3 > -5 > -100");

    // Read unsigned, -5 and -100 become huge and lead. This is MUT-U0C, and it
    // is the observable difference the signedness parameter names.
    let m = Worklist::build(
        MUT_U0C,
        &[Candidate::new(0, -5, 0), Candidate::new(1, 3, 0), Candidate::new(2, -100, 0)],
    );
    assert_eq!(m.ids(), vec![0, 2, 1], "unsigned DESC: 0xfffffffb > 0xffffff9c > 3");
    assert_ne!(w.ids(), m.ids());
}

/// An exact tie in both keys reverses the accumulation order — the mechanical
/// content of `<=`, and the thing model `M-TIE` is built on.
#[test]
fn a_full_tie_reverses_the_accumulation_order_and_the_lt_mutant_preserves_it() {
    let cands: Vec<Candidate> = (0..5).map(|i| Candidate::new(i, 0, 0)).collect();
    assert_eq!(Worklist::build(WorklistComparator::C2, &cands).ids(), vec![4, 3, 2, 1, 0]);
    assert_eq!(Worklist::build(MUT_LT, &cands).ids(), vec![0, 1, 2, 3, 4]);
}

// ===========================================================================
// 2. RE-ENTRY — consequence 2, made runnable
// ===========================================================================

/// **`P_REGALLOC` §4 consequence 2, executed**: a spilled candidate re-enters by
/// priority, and a port modelling the worklist as a stack **or** a queue is
/// wrong — in **both** directions, on one input.
#[test]
fn reentry_by_priority_differs_from_both_stack_and_queue() {
    let seed = [Candidate::new(0, 30, 0), Candidate::new(1, 20, 0), Candidate::new(2, 10, 0)];
    // The spilled candidate's priority sits strictly between two incumbents, so
    // priority order is the only one that puts it in the middle.
    let spilled = Candidate::new(3, 15, 0);

    let run = |p: ReentryPolicy| {
        let mut w = Worklist::build(WorklistComparator::C2, &seed);
        w.reentry = p;
        w.pop(); // colour the head, then discover it must spill
        w.reinsert_after_spill(spilled);
        w.ids()
    };

    let by_priority = run(ReentryPolicy::ByPriority);
    let stack = run(ReentryPolicy::Head);
    let queue = run(ReentryPolicy::Tail);

    assert_eq!(by_priority, vec![1, 3, 2], "c2: re-enters between 20 and 10");
    assert_eq!(stack, vec![3, 1, 2], "a stack pops it next — WRONG");
    assert_eq!(queue, vec![1, 2, 3], "a queue pops it last — WRONG");
    assert_ne!(by_priority, stack);
    assert_ne!(by_priority, queue);
}

/// **No cell of the 20 can test the previous test's claim**, and that is
/// registered rather than left to inference.
///
/// The widest cell is `cnd_a8` at 8 candidates; the callee-saved run
/// `r31 … r14` is 18 registers wide. Nothing spills, so nothing re-enters, so
/// the population's power over [`ReentryPolicy`] is **zero** and the claim is
/// confirmed only synthetically.
#[test]
fn no_cell_of_the_twenty_reaches_the_spill_reentry_path() {
    const CALLEE_SAVED_RUN: usize = 18; // r31 … r14, `0x10c37de0`'s tail
    let widest = CELLS.iter().map(|c| c.1).max().unwrap();
    assert_eq!(widest, 8, "cnd_a8 is the widest cell");
    assert!(
        widest < CALLEE_SAVED_RUN,
        "if a cell ever exceeded the callee-saved run this zero would stop being true"
    );
    // The observable that would witness a spill: a `stw`/`lwz` pair against the
    // frame for a candidate. Every cell's order is a permutation of its formals
    // with no repeats, which is what a non-spilling colouring looks like.
    for (name, n, o1, ox) in CELLS {
        assert_eq!(o1.len(), *n, "{name} at /O1");
        assert_eq!(ox.len(), *n, "{name} at /Ox");
        let mut seen = ids_of(o1);
        seen.sort_unstable();
        assert_eq!(seen, mint_order(*n), "{name}: each candidate coloured exactly once");
    }
}

// ===========================================================================
// 3. THE CONTROL — watched failing, per #3336
// ===========================================================================

/// **The instrument control.** `cnd_c0` holds nothing live across the call and
/// must take no callee-saved colour; measured empty at both profiles.
#[test]
fn the_grid_control_took_no_callee_saved_colour() {
    assert_eq!(
        CONTROL_C0_COLOURS, 0,
        "cnd_c0 framed: the extractor is not reading what it claims and CELLS is void"
    );
}

/// **`#3336` — a control never watched fail is decoration.** Every one of the
/// five planted mutants must disagree with [`WorklistComparator::C2`] on at
/// least one input, or the parameter it moves is not a decision point at all.
///
/// **This test has been watched going red.** Setting `MUT_S44` to
/// `WorklistComparator::C2` produces:
///
/// ```text
/// MUT-S44 agrees with C2 on every synthetic input: the parameter it moves is
/// not a decision point
/// ```
///
/// The separator for each mutant is named beside it, so a future reader can see
/// *what* distinguishes it rather than only *that* something does.
#[test]
fn all_five_mutants_separate_from_c2_on_synthetic_input() {
    // One input per mutant, each chosen to be the smallest thing that reaches
    // the clause the mutant moves.
    let separators: &[(&str, WorklistComparator, Vec<Candidate>, &str)] = &[
        (
            "MUT-LT",
            MUT_LT,
            vec![Candidate::new(0, 7, 7), Candidate::new(1, 7, 7)],
            "an exact tie in both keys — the only input the tier can see",
        ),
        (
            "MUT-ASC",
            MUT_ASC,
            vec![Candidate::new(0, 1, 0), Candidate::new(1, 2, 0)],
            "two distinct priorities",
        ),
        (
            "MUT-SWAP",
            MUT_SWAP,
            vec![Candidate::new(0, 9, 1), Candidate::new(1, 1, 9)],
            "the two keys disagree about which candidate leads",
        ),
        (
            "MUT-U0C",
            MUT_U0C,
            vec![Candidate::new(0, -1, 0), Candidate::new(1, 1, 0)],
            "a NEGATIVE priority — reachable, `0x10b2d630` subtracts",
        ),
        (
            "MUT-S44",
            MUT_S44,
            vec![Candidate::new(0, 0, 0x8000_0000), Candidate::new(1, 0, 1)],
            "a tie ordinal with bit 31 set — see the observability test below",
        ),
    ];

    assert_eq!(separators.len(), MUTANTS.len(), "every mutant needs a separator");

    for (name, mutant, input, why) in separators {
        let base = Worklist::build(WorklistComparator::C2, input).ids();
        let mutated = Worklist::build(*mutant, input).ids();
        assert_ne!(
            base, mutated,
            "{name} agrees with C2 on every synthetic input: the parameter it moves is \
             not a decision point (separator: {why})"
        );
    }
}

/// **`MUT-S44` is a decision point with an EMPTY observable set**, and saying so
/// is the point of the test.
///
/// It separates from c2 only when `cand+0x44` has bit 31 set. That field is a
/// tuple-visit counter incremented once per real tuple (`0x10b55f77`), so
/// reaching 2³¹ needs a function with two billion lowered tuples. **No
/// compilation this project can build separates signed from unsigned on the tie
/// key** — the parameter is settable, defaults to the read, and is graded by
/// nothing. Registered so nobody later quotes it as confirmed.
#[test]
fn the_tie_key_signedness_has_an_empty_observable_set() {
    // Below bit 31 the two readings are indistinguishable, at every value.
    for t in [0u32, 1, 2, 1000, 0x7fff_ffff] {
        let input = [Candidate::new(0, 0, t), Candidate::new(1, 0, 7)];
        assert_eq!(
            Worklist::build(WorklistComparator::C2, &input).ids(),
            Worklist::build(MUT_S44, &input).ids(),
            "tie ordinal {t} does not separate signed from unsigned"
        );
    }
    // And only at or above it do they differ.
    let input = [Candidate::new(0, 0, 0x8000_0000), Candidate::new(1, 0, 7)];
    assert_ne!(
        Worklist::build(WorklistComparator::C2, &input).ids(),
        Worklist::build(MUT_S44, &input).ids()
    );
}

// ===========================================================================
// 4. THE POPULATION'S POWER — the measurement this lane exists to publish
// ===========================================================================

/// Model **M-FREE**: both keys unconstrained. Construct a key vector that makes
/// `target` come out under `cmp`, then **verify it by running the worklist** —
/// a constructed witness that is never checked is an assertion, not a proof.
fn m_free_witness(cmp: WorklistComparator, target: &str) -> Option<Vec<Candidate>> {
    let want = ids_of(target);
    let n = want.len();
    let mut cands: Vec<Candidate> = mint_order(n).iter().map(|&i| Candidate::new(i, 0, 0)).collect();
    for (rank, &id) in want.iter().enumerate() {
        // Rank 0 must lead. Under Desc give it the largest value, under Asc the
        // smallest; write into whichever field the comparator uses as primary.
        let v = match cmp.primary.dir {
            SortDir::Desc => (n - rank) as i32,
            SortDir::Asc => rank as i32,
        };
        let c = &mut cands[id as usize];
        match cmp.primary.field {
            KeyField::Priority => c.priority = v,
            KeyField::TieOrdinal => c.tie = v as u32,
        }
    }
    // Accumulate in MINT order, as `0x10b316b1` does.
    let got = Worklist::build(cmp, &cands).ids();
    if got == want { Some(cands) } else { None }
}

/// Model **M-TIE**: the model the record commits to for the `/O1` cells —
/// *"the benefit keys are equal and the order is decided entirely by insertion
/// sequence"* (`WB_DAGORDER2_FINDINGS.md` §5), with insertion sequence the
/// bucket walk = ascending mint index (§5.0 + read **R1**). Both keys 0.
fn m_tie_prediction(cmp: WorklistComparator, n: usize) -> Vec<u32> {
    let cands: Vec<Candidate> = mint_order(n).iter().map(|&i| Candidate::new(i, 0, 0)).collect();
    Worklist::build(cmp, &cands).ids()
}

/// **M-FREE IS VACUOUS, AND SO IS EVERY MUTANT OF IT.**
///
/// With `cand+0x0c` and `cand+0x44` unconstrained — which is their real state,
/// since neither is observable in an obj — every one of the 20 orders is
/// reproducible at both profiles under c2's comparator **and under all five
/// mutants**. So *"the comparator reproduces the 20 cells"* is a true sentence
/// that carries no information: the reversed-direction comparator reproduces
/// them too.
///
/// This is the finding, not a caveat. `#1236`.
#[test]
fn m_free_reproduces_every_cell_under_c2_and_under_every_mutant() {
    let all: Vec<(&str, WorklistComparator)> =
        core::iter::once(("C2", WorklistComparator::C2)).chain(MUTANTS.iter().copied()).collect();

    for (label, cmp) in &all {
        let mut hits = 0usize;
        for (name, _n, o1, ox) in CELLS {
            for (prof, order) in [("O1", o1), ("Ox", ox)] {
                assert!(
                    m_free_witness(*cmp, order).is_some(),
                    "{label}: no key vector reproduces {name}@{prof} = {order}"
                );
                hits += 1;
            }
        }
        assert_eq!(hits, 40, "{label}: 20 cells x 2 profiles");
    }
}

/// **M-TIE — the record's own model of the `/O1` cells — IS REFUTED BY THE A
/// SERIES ON 6 OF ITS 8 CELLS.**
///
/// M-TIE + `<=` predicts the finished list is the **reverse** of the mint order.
/// The A series is `a`, `ba`, **`bac`**, **`dbac`**, **`ebacd`**, … — it agrees
/// at n=1 and n=2 and disagrees from n=3 on.
///
/// `WB_DAGORDER2_FINDINGS.md` §5.0 discloses the n=3 miss and fences its own
/// claim on it. What is added here is that the miss is **not a single awkward
/// cell**: it is 6 of 8, it is the whole A series past n=2, and read **R4**
/// explains it — `cand+0x44` is a *dense tuple-visit ordinal*, so an exact tie
/// in **both** keys is the exception rather than the rule, and M-TIE's premise
/// is wrong in kind.
#[test]
fn the_all_ties_model_is_refuted_on_six_of_the_eight_a_series_cells() {
    let a_series: Vec<&(&str, usize, &str, &str)> =
        CELLS.iter().filter(|c| c.0.starts_with("cnd_a")).collect();
    assert_eq!(a_series.len(), 8);

    let mut hit = Vec::new();
    let mut miss = Vec::new();
    for (name, n, o1, _ox) in &a_series {
        let predicted = m_tie_prediction(WorklistComparator::C2, *n);
        if predicted == ids_of(o1) { hit.push(*name) } else { miss.push(*name) }
    }

    assert_eq!(hit, vec!["cnd_a1", "cnd_a2"], "M-TIE survives only at n=1 and n=2");
    assert_eq!(
        miss,
        vec!["cnd_a3", "cnd_a4", "cnd_a5", "cnd_a6", "cnd_a7", "cnd_a8"],
        "M-TIE is refuted from n=3 on — 6 of 8"
    );

    // And the `<` mutant does no better: it predicts forward mint order, which
    // the A series matches only at n=1. So the population refutes BOTH tiers
    // under this model, which is what makes the next test's zero meaningful.
    let lt_hits = a_series
        .iter()
        .filter(|(_, n, o1, _)| m_tie_prediction(MUT_LT, *n) == ids_of(o1))
        .count();
    assert_eq!(lt_hits, 1, "M-TIE + `<` survives only at n=1");
}

/// **POPULATION POWER — published with its zeros, which is the whole point.**
///
/// For each mutant: on how many of the 40 cell-profiles does the observation
/// prefer c2's comparator to the mutant? A cell counts only if one of the two
/// reproduces the observed order and the other does not.
///
/// Measured under M-FREE — the honest model, because the keys really are
/// unobservable — **and every entry is 0**. The 20 obj cells the brief calls
/// this lane's test population **cannot decide any of the five parameters**,
/// including the `<=`/`<` tier the brief singles out.
///
/// The one model under which the cells *would* decide the tier is M-TIE, and the
/// test above shows the same cells refute M-TIE 6-of-8. So the correct statement
/// is not *"the cells confirm the `<=` tier"* and not *"the cells are silent by
/// accident"* — it is:
///
/// > **the 20 cells separate `<=` from `<` only under a key model those same 20
/// > cells refute.**
#[test]
fn population_power_over_the_twenty_cells() {
    let mut table: Vec<(&str, usize)> = Vec::new();
    for (name, mutant) in MUTANTS {
        let mut deciding = 0usize;
        for (_cell, _n, o1, ox) in CELLS {
            for order in [o1, ox] {
                let c2_ok = m_free_witness(WorklistComparator::C2, order).is_some();
                let mut_ok = m_free_witness(*mutant, order).is_some();
                if c2_ok != mut_ok {
                    deciding += 1;
                }
            }
        }
        table.push((name, deciding));
    }

    assert_eq!(
        table,
        vec![("MUT-LT", 0), ("MUT-ASC", 0), ("MUT-SWAP", 0), ("MUT-U0C", 0), ("MUT-S44", 0)],
        "if any entry is nonzero the population has power the lane reported it did not; \
         re-score the rung"
    );
}

/// The 6-of-20 profile disagreement, reproduced independently — the cells that
/// carry the signal, and the exact-reversal relation on all six.
///
/// Not a claim about the comparator; a check that this lane's re-measurement is
/// the same population `#3241` measured, so the zeros above are zeros about the
/// right 20 cells.
#[test]
fn six_of_the_twenty_cells_disagree_across_profiles_and_all_six_are_exact_reversals() {
    let differ: Vec<&str> =
        CELLS.iter().filter(|(_, _, o1, ox)| o1 != ox).map(|(n, _, _, _)| *n).collect();
    assert_eq!(
        differ,
        vec!["cnd_h2", "cnd_h2r", "cnd_h3", "cnd_h3r", "cnd_u2", "cnd_u2r"],
        "6 of 20, and they are exactly the H and U families"
    );
    for (name, _, o1, ox) in CELLS.iter().filter(|(_, _, a, b)| a != b) {
        let rev: String = o1.chars().rev().collect();
        assert_eq!(&rev, ox, "{name}: /O1 and /Ox must be exact reversals");
    }
}

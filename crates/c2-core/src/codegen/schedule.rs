//! **SCHED** — the store/producer schedule, as real `c2` performs it.
//!
//! The single recurring blocker of this project. Twelve candidate rules had
//! been refuted by four lanes before this file existed: `w-pair` §4's six
//! *placement* rules, `leaf_store.rs`'s four *allocation* rules, and
//! `w-dclass`/B's `F4a` (fitted 6/6, refuted by `o7` and by `xboxheap`) and
//! `F4b` (declared unfitted). `w-conv` recorded the axis as "unpriceable".
//!
//! It is two rules and one constant. See `docs/STORE_SCHEDULE.md` for the
//! grid, the holdout protocol and the numbers.
//!
//! **The read that would replace this module's fit — comment only, nothing
//! here changes.** Added 2026-08-22 under read-before-probe
//! (`docs/WHITEBOX_LEVERAGE_2026-08-21.md` §1;
//! `docs/whitebox/READ_PLAN_2026-08-21.md` §2, the fitted-constant index).
//! The rules below came out of a preregistered **13,104-configuration** list-
//! scheduler search that returned a *negative* result — 89/146, residual
//! exactly the two-producer tier, no member of the family expressing the
//! shipped rule. c2's actual scheduler has been read: driver `0x10be6382`
//! (4 runs/function at `/O1`), region finder `0x10be5d4b`, ready-list compare
//! `0x10be5cea` (priority DESC then `node+0x44` ASC), priority formula
//! `(height<<13)+(fanout<<8)+(symdest<<10)`, and the 11×11 latency matrix at
//! `0x10c3c1a8` — **all `[R]`, none `[O]`**. Read **R7** (3–5 d) is exactly
//! the promotion: *no new reading*, just confronting that read model against
//! the live tap. Note the scope difference honestly — this module schedules a
//! **store run**, c2's is a machine scheduler over tuple regions, and
//! `schedule.rs:38` says so; R7 grades the latter, not this. `MAX_MODELLED_PRODUCERS`
//! (board #541) is a shipped constant with unread provenance.
//!
//! 1. **Store order.** Walk the source statements in order and emit the
//!    earliest store that is *allowed*. A store whose value needs a new
//!    instruction to materialise it — a **produced** store — may not occupy
//!    store position 0 or 1: it may not be the first or the second store.
//!    Stores through different base **symbols** may not be reordered past each
//!    other (they may alias). If every remaining store is blocked, source
//!    order wins.
//! 2. **Producer placement.** The producers, in source order, are inserted
//!    immediately *before* the stores at store positions 0, 1, 2, … — one
//!    producer per store slot, from the top of the block. **Scope condition**
//!    (board #542, measured by `w-alloc`, shipped by `w-order2`): that holds
//!    only while there are unproduced stores to slot against. With
//!    `u = min(2, #unproduced)` head slots, producers fill those one apiece
//!    and every *remaining* producer is emitted **contiguously** immediately
//!    before store slot `u` — `{a=1;b=2;c=3;}` is `P P P S S S`.
//!
//! ## Superseded, in part
//!
//! Rule 1 is the special case of [`order`](super::order)'s single floor, and
//! rule 1's silence about what fills the head when *every* store is produced
//! is what [`order`](super::order) fills in. This module is kept because it
//! carries the **may-alias** axis, which `order` refuses rather than models:
//! `xboxheap.cpp` stores through two symbols and emits its producers in a
//! different order from a single-symbol run of the same shape.
//!
//! ## What this is NOT
//!
//! It is **not a machine scheduler**, and that is measured rather than
//! asserted. A `mulli` producer — several cycles slower than an `addi` on this
//! part — yields the byte-identical permutation to an `li`; so do `addi` from
//! a base, `addi` from a formal, and `rlwinm`. A preregistered search over
//! 13,104 list schedulers (forward/backward × latency 1..6 × a lexicographic
//! priority over six DAG features) tops out at 89 of 146 fit cells and its
//! residual is *exactly* the two-producer tier, 0 of 48. Rule 2 is not a
//! priority function, so no member of that family can express it.
//!
//! ## The one thing it does not cover
//!
//! When the register allocator hands a producer a register that is still the
//! data source of a store the schedule has not emitted yet, the resulting
//! write-after-read anti-dependence perturbs the order. That is an
//! **allocation** fact — `leaf_store.rs` already records four refuted rules
//! for it — and it is out of scope here. It **never** happens with 0 or 1
//! producers (154 of 154 measured cells) and is rare with 2. [`schedule`]
//! therefore describes the emitted order exactly whenever the caller can show
//! the allocation is clean, and callers that cannot must refuse.

/// One source statement of a store run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stmt {
    /// `Some(id)` when the value needs a materialising instruction. Two
    /// statements sharing an `id` share one producer (c2 CSEs equal
    /// constants and equal address binds); `None` when the value is already
    /// in a register — a formal, or `this`.
    pub producer: Option<u32>,
    /// The base **symbol** of the store's address expression — not the
    /// machine register. Two stores with different symbols may alias and are
    /// never reordered past each other. This is the axis `w-pair`'s H4/H5
    /// mistook for a register-number superstition.
    pub base: u32,
}

/// One emitted slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// The materialising instruction for producer id `n`.
    Producer(u32),
    /// The store for source statement `n`.
    Store(usize),
}

/// Rule 1's only constant: a produced store may not be the first or second
/// store of the run.
pub const BLOCKED_STORE_POSITIONS: usize = 2;

/// The schedule. Returns one [`Slot`] per emitted instruction, in order.
pub fn schedule(stmts: &[Stmt]) -> Vec<Slot> {
    // Producers, in source order, deduplicated by id — a shared producer is
    // one instruction with two consumers.
    let mut producers: Vec<u32> = Vec::new();
    for s in stmts {
        if let Some(p) = s.producer {
            if !producers.contains(&p) {
                producers.push(p);
            }
        }
    }

    // Rule 1.
    let mut order: Vec<usize> = Vec::with_capacity(stmts.len());
    let mut left: Vec<usize> = (0..stmts.len()).collect();
    while !left.is_empty() {
        let pos = order.len();
        let pick = left.iter().copied().position(|k| {
            if stmts[k].producer.is_some() && pos < BLOCKED_STORE_POSITIONS {
                return false;
            }
            // may-alias: cannot pass an earlier store on a different symbol
            !left
                .iter()
                .any(|&j| j < k && stmts[j].base != stmts[k].base)
        });
        // Everything blocked ⇒ source order. Reached whenever every store of
        // the run is produced (`{a=9;b=9;c=9;}`), which is exactly the shape
        // the port already emits byte-exact.
        order.push(left.remove(pick.unwrap_or(0)));
    }

    // Rule 2, with `w-alloc`'s scope condition (board #542, now closed by
    // `w-order2`). "One producer per store slot" holds only while there are
    // UNPRODUCED stores to slot against: rule 1 keeps store positions 0 and 1
    // free of produced stores, so there are `u = min(2, #unproduced)` such
    // slots. Producers fill those one apiece and every REMAINING producer is
    // emitted contiguously immediately before store slot `u`.
    //
    // Measured through the `/FAsc` seam at the workload's flags:
    //   `{a=1;b=2;c=3;}`     is  P P P S S S    — not P S P S P S
    //   `{a=1;b=2;c=3;d=f;}` is  P S P P S S S
    // `w-sched`'s grid always had at least three formals and at most three
    // producers, so it never ran out of slots and never saw the difference.
    let u = stmts
        .iter()
        .filter(|s| s.producer.is_none())
        .count()
        .min(BLOCKED_STORE_POSITIONS);
    let mut out = Vec::with_capacity(stmts.len() + producers.len());
    let mut next = 0usize;
    for (store_pos, &k) in order.iter().enumerate() {
        while next < producers.len() && (store_pos == next || (store_pos == u && next >= u))
        {
            out.push(Slot::Producer(producers[next]));
            next += 1;
        }
        out.push(Slot::Store(k));
    }
    while next < producers.len() {
        out.push(Slot::Producer(producers[next]));
        next += 1;
    }
    out
}

/// True when [`schedule`] leaves every store where the source put it. The
/// store emitters currently accept only runs for which this holds, and they
/// refuse when it does not — so a future widening of the *parser* cannot
/// silently turn a clean refusal into a wrong instruction order, which is
/// exactly how board #232 became a live wrong emit for 255 commits.
pub fn is_source_order(stmts: &[Stmt]) -> bool {
    let mut seen = 0usize;
    for slot in schedule(stmts) {
        if let Slot::Store(k) = slot {
            if k != seen {
                return false;
            }
            seen += 1;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compact spelling: `.` = a formal-valued store, a digit = a producer id,
    /// all through one base symbol.
    fn stmts(spec: &str) -> Vec<Stmt> {
        spec.chars()
            .map(|c| Stmt {
                producer: c.to_digit(10),
                base: 0,
            })
            .collect()
    }

    fn render(slots: &[Slot]) -> String {
        slots
            .iter()
            .map(|s| match s {
                Slot::Producer(p) => format!("P{p}"),
                Slot::Store(k) => format!("S{k}"),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn check(spec: &str, want: &str) {
        assert_eq!(render(&schedule(&stmts(spec))), want, "cell {spec}");
    }

    /// Every one of these is a **measurement**, taken from real `c2.dll` under
    /// wibo at the workload's own flags via the `/FAsc` listing seam. The
    /// grid is `work/w-sched/`; the cell names are the published ones.
    #[test]
    fn measured_cells_from_real_c2() {
        // --- w-pair §4, whose six placement rules each died on one of these
        check("...", "S0 S1 S2"); // C0 control: no producer, no reorder
        check(".0", "P0 S0 S1"); // C1
        check(".0.", "P0 S0 S2 S1"); // C2f
        check("0.", "P0 S1 S0"); // D1
        check("0..", "P0 S1 S2 S0"); // D2
        check("0...", "P0 S1 S2 S0 S3"); // D3
        check(".0..", "P0 S0 S2 S1 S3"); // D7
        check("0....", "P0 S1 S2 S0 S3 S4"); // D8
        check("......0", "P0 S0 S1 S2 S3 S4 S5 S6"); // C7: a 7-slot hoist
        check("0......", "P0 S1 S2 S0 S3 S4 S5 S6"); // C8
        check("0...", "P0 S1 S2 S0 S3"); // D6, an `addi` producer
        check("01..", "P0 S2 P1 S3 S0 S1"); // E5, two producers

        // --- w-dclass/B §3.4, including the cell F4b called UNFITTED
        check(".0..", "P0 S0 S2 S1 S3"); // o1
        check("...0", "P0 S0 S1 S2 S3"); // o2
        check("0...", "P0 S1 S2 S0 S3"); // o3
        check(".00.", "P0 S0 S3 S1 S2"); // o4: ONE shared `li`
        check(".01.", "P0 S0 P1 S3 S1 S2"); // o5
        check("..0..", "P0 S0 S1 S2 S3 S4"); // o6
        check(".012.", "P0 S0 P1 S4 P2 S1 S2 S3"); // o7 — F4b's cell
        check("...0.", "P0 S0 S1 S2 S3 S4"); // o8

        // --- three producers, tier 5 (o7's regime), from the HOLDOUT
        check("012..", "P0 S3 P1 S4 P2 S0 S1 S2");
        check("...012", "P0 S0 P1 S1 P2 S2 S3 S4 S5");

        // --- every store produced ⇒ source order (the port's accepted shape)
        check("00", "P0 S0 S1");
        check("000", "P0 S0 S1 S2");
        check("0", "P0 S0");
    }

    /// Rule 2's **scope condition** — board #542, measured by `w-alloc` and
    /// shipped here. "One producer per store slot" runs out of slots when the
    /// unproduced stores do, and every remaining producer is then emitted
    /// contiguously. Read off real `c2.dll` through the `/FAsc` seam at the
    /// workload's flags; the grid rows are named.
    #[test]
    fn rule_2_runs_out_of_slots_and_the_rest_go_contiguous() {
        check("01", "P0 P1 S0 S1"); // w-order2 t1_01
        check("012", "P0 P1 P2 S0 S1 S2"); // t2_012 — NOT P S P S P S
        check("0120", "P0 P1 P2 S0 S1 S2 S3"); // t2_0120
        // one unproduced store: exactly ONE producer is interleaved, and the
        // other two come out together.
        check("012.", "P0 S3 P1 P2 S0 S1 S2"); // t9_012_tail1
        // two unproduced stores: the head is full, nothing is left over.
        check("012..", "P0 S3 P1 S4 P2 S0 S1 S2"); // t9_012_tail2
    }

    /// The may-alias axis, isolated. `w-pair`'s H4/H5 died on E1/F1/F2 and
    /// concluded the surviving rules were "register-number superstitious".
    /// They were not: the axis is the base SYMBOL, and the controlled cells
    /// that separate it are tier 6's split destinations.
    #[test]
    fn two_base_symbols_pin_the_order_and_that_is_what_killed_h4_h5() {
        let sym = |spec: &str, bases: &str| -> Vec<Stmt> {
            spec.chars()
                .zip(bases.chars())
                .map(|(c, b)| Stmt {
                    producer: c.to_digit(10),
                    base: b as u32,
                })
                .collect()
        };
        // E2 — all four stores through ONE symbol: free to reorder, gap 3.
        assert_eq!(render(&schedule(&sym("0..0", "hhhh"))), "P0 S1 S2 S0 S3");
        // E1 — the stores alternate between two symbols: pinned, gap 1.
        assert_eq!(render(&schedule(&sym("0..0", "ghhg"))), "P0 S0 S1 S2 S3");
        // F1/F2 — w-pair's controlled swap. BOTH gap 1, and the reason is the
        // two symbols, not which architectural register the producer reads.
        assert_eq!(render(&schedule(&sym("0..0", "lbbl"))), "P0 S0 S1 S2 S3");
        assert_eq!(render(&schedule(&sym("0..0", "laal"))), "P0 S0 S1 S2 S3");
        // D5 — one machine base register, TWO symbols (`h` and the bound
        // reference `l`): pinned. This is the cell that refuted w-pair's H3.
        assert_eq!(render(&schedule(&sym("0..0", "lhhl"))), "P0 S0 S1 S2 S3");
    }

    /// `src/xdk/nuispeech/xboxheap.cpp`'s constructor — the FRONTIER's only
    /// branch-free TU, and the object this whole rule was built to predict.
    /// Statements: `mSize = size` (formal), `mFreeHead = this`, `mCount = 0`
    /// (producer 0), `mUsedHead = this`, `listHead.mNext = &listHead`,
    /// `listHead.mPrev = &listHead` (producer 1, shared, through the bound
    /// reference's own symbol).
    #[test]
    fn xboxheap_constructor_is_derived_not_fitted() {
        let s = |p: Option<u32>, b: u32| Stmt { producer: p, base: b };
        let body = [
            s(None, 0),
            s(None, 0),
            s(Some(0), 0),
            s(None, 0),
            s(Some(1), 1),
            s(Some(1), 1),
        ];
        // Read off the REAL obj at the workload's flags:
        //   li r10,0 ; stw r5,16(r3) ; addi r11,r3,8 ; stw r3,0(r3) ;
        //   stw r10,20(r3) ; [mr r31,r3] ; stw r3,4(r3) ; stw r11,8(r3) ;
        //   stw r11,12(r3)
        // The `mr r31,r3` is the live-range save across the call, not a store
        // producer — counting it as one is what made this TU look like a
        // "third regime at 0, 2, 5".
        assert_eq!(
            render(&schedule(&body)),
            "P0 S0 P1 S1 S2 S3 S4 S5"
        );
    }

    #[test]
    fn source_order_predicate_agrees_with_the_schedule() {
        assert!(is_source_order(&stmts("...")));
        assert!(is_source_order(&stmts("000")));
        assert!(is_source_order(&stmts("...0")));
        assert!(!is_source_order(&stmts("0...")));
        assert!(!is_source_order(&stmts(".0..")));
    }
}

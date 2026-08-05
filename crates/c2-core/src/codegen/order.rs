//! **ORDER** — the store order of a run when the head slots are contested.
//!
//! [`schedule`](super::schedule) settles the order of a store run whenever
//! *some* store of the run needs no producer, and names what it does not
//! cover: what fills store positions 0 and 1 when **every** store is produced.
//! [`alloc`](super::alloc) settles the register. `docs/ORDER.md` is the
//! write-up; the grid is `work/w-order2/` (gitignored — the generators are
//! committed, the `.cod` and `.obj` are not).
//!
//! # The rule
//!
//! > Rank the run's distinct value-producers by
//! > **(use count descending, first-use source index ascending)**.
//! > Let `u = min(2, number of unproduced stores)`.
//! > A store whose producer has rank `j` may not occupy store position
//! > `< u + j`; an unproduced store is never blocked. Walk the source
//! > statements in order and emit the earliest allowed store.
//! >
//! > Producers are **emitted in rank order**. The first `u` go one apiece
//! > immediately before store slots `0 … u-1`; every remaining producer is
//! > emitted **contiguously** immediately before store slot `u`.
//!
//! One constant, the `2`, and it is [`schedule`](super::schedule)'s own.
//!
//! The rank is **not** the register order. [`alloc`](super::alloc) breaks a
//! use-count tie among constants by **reverse** source order; the rank breaks
//! it by **forward** source order. `{a=1;b=2;c=2}` puts `2` in `r11` and emits
//! its producer **second**. That two orders over the same producers disagree
//! in sign is why fitting either alone kept failing.
//!
//! # What this subsumes
//!
//! * [`schedule`](super::schedule)'s rule 1 — *"a produced store may not
//!   occupy store position 0 or 1"* — is rank `j = 0` with `u = 2`.
//! * [`alloc`](super::alloc)'s measured hoist — *"the first consumer of the
//!   strictly greatest use count moves into the head"* — is rank `j = 0` with
//!   `u < 2`: the rank-0 producer is the strictly-greatest-count one, and it
//!   is the only producer whose floor is below `u + 1`.
//!
//! Both are recomputed in the tests from this rule, not transcribed.
//!
//! # What it refuses, and why the refusals are not conservatism
//!
//! * **More than one base symbol.** `xboxheap.cpp`'s constructor stores
//!   through two symbols and emits its producers in **first-consumption**
//!   order where a single-symbol run of the same shape emits them in **rank**
//!   order — measured on 8 cells of this lane's grid, 2 fit and 6 held out.
//!   The composition of two symbol-runs is board **#564**, and it is open.
//! * **More than [`MAX_MODELLED_PRODUCERS`] producers**, matching
//!   [`alloc`](super::alloc)'s own domain. The store order alone is exact on
//!   four-producer runs (822 of 822 cells), but the register is not, and a
//!   caller needs both.
//!
//! # Evidence
//!
//! | population | cells | in domain | exact | wrong |
//! |---|---:|---:|---:|---:|
//! | discovery (`w-alloc`'s grid, both partitions) | 526 | 479 | **479** | 0 |
//! | fit | 250 | 248 | **248** | 0 |
//! | **holdout** | 572 | 561 | **561** | **0** |
//! | holdout, shapes absent from discovery | — | 223 | **223** | 0 |
//! | store order alone, fit + holdout | 822 | 822 | **822** | 0 |
//!
//! A preregistered exhaustive search over **1,048,576 per-store release-time
//! schedulers** — 2 counters × `4^9` thresholds over
//! `{unproduced} ∪ {count 1,2,3,≥4} × {first use?}` × 2 tiebreaks — tops out
//! at **196 of 250** fit cells, and **50 of its 54 misses are cells whose rank
//! order is not the producers' source order** against 4 of 157 where it is.
//! The axis is the rank, and no release time keyed on a store's own use count
//! can carry it: two producers can tie on the count and take different ranks.
//!
//! The holdout partition was declared in
//! `docs/rungs/_2026-08-05-w-order2-prereg.md` §6 **before** `grid.py` was
//! written, written by the generator into a file the fitter refuses to open,
//! and scored only after this rule was frozen at commit `980e42e`.

use super::schedule::{Slot, Stmt};

/// [`schedule`](super::schedule)'s rule 1 constant, reused: the number of head
/// store slots a producer may be interleaved into, and the ceiling on `u`.
pub const HEAD_SLOTS_MAX: usize = 2;

/// Matches [`alloc`](super::alloc)'s domain. The order alone is exact past
/// this; the register is not.
pub const MAX_MODELLED_PRODUCERS: usize = 3;

/// The distinct producers of a run, **in rank order** — use count descending,
/// first-use source index ascending. Returns `None` outside the domain.
pub fn rank_order(stmts: &[Stmt]) -> Option<Vec<u32>> {
    if !single_symbol(stmts) {
        return None;
    }
    let mut ps: Vec<(u32, usize, usize)> = Vec::new(); // (id, uses, first)
    for (i, s) in stmts.iter().enumerate() {
        if let Some(id) = s.producer {
            match ps.iter_mut().find(|p| p.0 == id) {
                Some(p) => p.1 += 1,
                None => ps.push((id, 1, i)),
            }
        }
    }
    if ps.len() > MAX_MODELLED_PRODUCERS {
        return None;
    }
    ps.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
    Some(ps.into_iter().map(|p| p.0).collect())
}

fn single_symbol(stmts: &[Stmt]) -> bool {
    stmts.windows(2).all(|w| w[0].base == w[1].base)
}

/// The number of head store slots a producer is interleaved into:
/// `min(2, #unproduced)`. This is `w-alloc`'s scope condition for
/// [`schedule`](super::schedule)'s rule 2 and the `u` of the floor.
pub fn head_slots(stmts: &[Stmt]) -> usize {
    stmts
        .iter()
        .filter(|s| s.producer.is_none())
        .count()
        .min(HEAD_SLOTS_MAX)
}

/// The store order: source indices, in emitted order. `None` outside the
/// domain.
pub fn store_order(stmts: &[Stmt]) -> Option<Vec<usize>> {
    let ranks = rank_order(stmts)?;
    let floor = |k: usize| -> usize {
        match stmts[k].producer {
            None => 0,
            Some(id) => {
                head_slots(stmts) + ranks.iter().position(|&r| r == id).unwrap_or(0)
            }
        }
    };
    let mut left: Vec<usize> = (0..stmts.len()).collect();
    let mut out: Vec<usize> = Vec::with_capacity(stmts.len());
    while !left.is_empty() {
        let q = out.len();
        // The earliest source-order store whose floor this slot clears.
        //
        // The fallback is UNREACHABLE on every cell measured — 822 of 822,
        // fit and holdout, plus the 479 discovery cells — because the stores
        // of ranks `0..=j` always number at least `j + 1`. It is written as a
        // saturating pick rather than a panic so that a widening reaches a
        // wrong ORDER through the guard below, never through a crash.
        let pick = left
            .iter()
            .position(|&k| q >= floor(k))
            .unwrap_or(0);
        out.push(left.remove(pick));
    }
    Some(out)
}

/// The full emitted schedule — producers and stores, in order. `None` outside
/// the domain.
pub fn schedule(stmts: &[Stmt]) -> Option<Vec<Slot>> {
    let ranks = rank_order(stmts)?;
    let order = store_order(stmts)?;
    let u = head_slots(stmts);
    let mut out = Vec::with_capacity(stmts.len() + ranks.len());
    let mut next = 0usize;
    for (slot, &k) in order.iter().enumerate() {
        // one producer apiece into the first `u` slots, then the remainder
        // contiguously immediately before slot `u`
        while next < ranks.len() && (slot == next || (slot == u && next >= u)) {
            out.push(Slot::Producer(ranks[next]));
            next += 1;
        }
        out.push(Slot::Store(k));
    }
    while next < ranks.len() {
        out.push(Slot::Producer(ranks[next]));
        next += 1;
    }
    Some(out)
}

/// `Some(true)` when ORDER leaves every store where the source put it,
/// `Some(false)` when it moves one, `None` when the run is outside the domain
/// and this module has nothing to say.
///
/// The store emitters call this as a **positive check**: they emit in source
/// order and refuse on `Some(false)`. It is inert today by construction — the
/// parser admits an all-unproduced run and an all-one-producer run, and ORDER
/// returns source order for both — and the point is board **#232**, where a
/// parser widening turned a clean refusal into a live wrong emit that survived
/// 255 commits. A widening that reaches this guard gets a refusal instead of a
/// wrong order.
pub fn is_source_order(stmts: &[Stmt]) -> Option<bool> {
    Some(store_order(stmts)?.iter().enumerate().all(|(q, &k)| q == k))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compact spelling, as `schedule`'s tests use it: `.` = a store whose
    /// value needs no instruction, a digit = a producer id. One base symbol.
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
        let got = schedule(&stmts(spec)).expect("in domain");
        assert_eq!(render(&got), want, "cell {spec}");
    }

    /// Every cell below is a **measurement**, read off real `c2.dll` under
    /// wibo at the workload's own flags through the `/FAsc` listing seam. The
    /// grid is `work/w-order2/`; `docs/ORDER.md` §6 reproduces it.
    ///
    /// These are the cells `docs/STORE_SCHEDULE.md` publishes, recomputed from
    /// ORDER rather than transcribed from `schedule`. Every one of them was a
    /// consequence of rule 1; they must stay consequences of the rule that
    /// replaces it.
    #[test]
    fn schedules_published_cells_stay_derived() {
        check("...", "S0 S1 S2"); // C0: no producer, no reorder
        check(".0", "P0 S0 S1"); // C1
        check(".0.", "P0 S0 S2 S1"); // C2f
        check("0.", "P0 S1 S0"); // D1
        check("0..", "P0 S1 S2 S0"); // D2
        check("0...", "P0 S1 S2 S0 S3"); // D3
        check(".0..", "P0 S0 S2 S1 S3"); // D7 / o1
        check("0....", "P0 S1 S2 S0 S3 S4"); // D8
        check("......0", "P0 S0 S1 S2 S3 S4 S5 S6"); // C7
        check("0......", "P0 S1 S2 S0 S3 S4 S5 S6"); // C8
        check("01..", "P0 S2 P1 S3 S0 S1"); // E5
        check("...0", "P0 S0 S1 S2 S3"); // o2
        check(".00.", "P0 S0 S3 S1 S2"); // o4, one shared `li`
        check(".01.", "P0 S0 P1 S3 S1 S2"); // o5
        check("..0..", "P0 S0 S1 S2 S3 S4"); // o6
        check("...0.", "P0 S0 S1 S2 S3 S4"); // o8
        check("012..", "P0 S3 P1 S4 P2 S0 S1 S2");
        check("...012", "P0 S0 P1 S1 P2 S2 S3 S4 S5");
        // o7 — the cell `w-dclass`/B declared UNFITTED, `STORE_SCHEDULE` §1.2
        check(".012.", "P0 S0 P1 S4 P2 S1 S2 S3");
    }

    /// `w-alloc`'s hoist — *"the first consumer of the STRICTLY greatest use
    /// count moves into the head, and nothing moves on a tie"* — is rank
    /// `j = 0`, not a separate mechanism. `docs/ALLOC.md` §6's three published
    /// cells, recomputed.
    #[test]
    fn the_hoist_is_rank_zero() {
        check("011", "P1 P0 S1 S0 S2"); // counts 1,2 -> stores 1,0,2
        check("0101", "P0 P1 S0 S1 S2 S3"); // counts 2,2 -> tie, no hoist
        check("00111", "P1 P0 S2 S0 S1 S3 S4"); // counts 2,3 -> 2,0,1,3,4
    }

    /// Family A of board #544's residual: counts (2,2,1), **no** unproduced
    /// store. The count-1 store is displaced out of the head, and the two
    /// count-2 producers take ranks 0 and 1 by first use even though `alloc`
    /// hands them `r11`/`r10` the other way round.
    #[test]
    fn family_a_the_all_produced_tie() {
        check("01022", "P0 P2 P1 S0 S2 S1 S3 S4");
        check("01122", "P1 P2 P0 S1 S2 S0 S3 S4");
        check("01202", "P0 P2 P1 S0 S2 S1 S3 S4");
        check("01212", "P1 P2 P0 S1 S2 S0 S3 S4");
        check("00122", "P0 P2 P1 S0 S1 S2 S3 S4");
        // the singleton already sits at position 2 -> nothing moves
        check("01201", "P0 P1 P2 S0 S1 S2 S3 S4");
        // and with only two producers the count-1 store is NOT displaced,
        // which is the cell a use-count release time gets wrong
        check("010", "P0 P1 S0 S1 S2");
    }

    /// Family B: counts (2,1) with exactly two unproduced stores. The hoist
    /// fires with the head slots already full — which is what `w-alloc`'s
    /// gating on "the unproduced stores ran out" got wrong.
    #[test]
    fn family_b_the_hoist_fires_with_a_full_head() {
        check("..011", "P1 S0 P0 S1 S3 S2 S4");
        check("..001", "P0 S0 P1 S1 S2 S3 S4"); // shared value first: no move
        check("0.1.1", "P1 S1 P0 S3 S2 S0 S4");
        check("0..11", "P1 S1 P0 S2 S3 S0 S4");
    }

    /// The cell that corrected the frozen rule ON FIT. Three unproduced
    /// stores: the floor is `u + j` with `u = min(2, 3) = 2`, so the rank-1
    /// producer's store is free at position 3 and source order stands. The
    /// version frozen at `7ee557e` counted PRODUCED stores instead and moved
    /// it. Discovery could not contain this shape — its runs are five
    /// statements long, so three fillers leave two producers that can never
    /// differ in rank by more than source order already gives.
    #[test]
    fn three_fillers_do_not_displace_the_rank_one_store() {
        check("...011", "P1 S0 P0 S1 S2 S3 S4 S5");
        check("0...11", "P1 S1 P0 S2 S3 S0 S4 S5");
        check("..0.11", "P1 S0 P0 S1 S3 S2 S4 S5");
        check("...0111", "P1 S0 P0 S1 S2 S3 S4 S5 S6");
    }

    /// `w-alloc`'s scope condition for rule 2, which is the layout clause
    /// here: with no unproduced store to slot against, the producers come out
    /// **contiguously**. `{a=1;b=2;c=3;}` is `P P P S S S`.
    #[test]
    fn producers_are_contiguous_when_there_is_nothing_to_slot_against() {
        check("01", "P0 P1 S0 S1");
        check("012", "P0 P1 P2 S0 S1 S2");
        check("0120", "P0 P1 P2 S0 S1 S2 S3");
        // one unproduced store: exactly one producer is interleaved
        check("012.", "P0 S3 P1 P2 S0 S1 S2");
    }

    /// Two base symbols: REFUSED, and the refusal is measured, not cautious.
    /// `xboxheap.cpp`'s constructor emits its producers in first-consumption
    /// order; eight single-symbol cells of the same shape emit them in rank
    /// order. Board **#564**.
    #[test]
    fn two_base_symbols_are_refused_because_xboxheap_disagrees() {
        let s = |p: Option<u32>, b: u32| Stmt { producer: p, base: b };
        let xboxheap = [
            s(None, 0),
            s(None, 0),
            s(Some(0), 0),
            s(None, 0),
            s(Some(1), 1),
            s(Some(1), 1),
        ];
        assert_eq!(store_order(&xboxheap), None);
        assert_eq!(rank_order(&xboxheap), None);
        assert_eq!(is_source_order(&xboxheap), None);
        // The same statement shape through ONE symbol is in domain, and c2
        // emits the rank-0 producer first — the opposite of xboxheap.
        check("...011", "P1 S0 P0 S1 S2 S3 S4 S5");
    }

    /// Four producers: the store order is exact (822 of 822 grid cells) but
    /// `alloc` cannot supply the registers, so the module refuses rather than
    /// answer half the question.
    #[test]
    fn four_producers_are_refused_with_alloc() {
        assert_eq!(store_order(&stmts("0123")), None);
        assert_eq!(is_source_order(&stmts("0123")), None);
    }

    /// The guard's own contract, on exactly the shapes the parser admits.
    #[test]
    fn source_order_predicate_is_inert_on_what_the_parser_admits() {
        // an all-unproduced run
        assert_eq!(is_source_order(&stmts("...")), Some(true));
        assert_eq!(is_source_order(&stmts("......")), Some(true));
        // an all-one-producer run: u = 0, rank 0, floor 0
        assert_eq!(is_source_order(&stmts("0")), Some(true));
        assert_eq!(is_source_order(&stmts("00")), Some(true));
        assert_eq!(is_source_order(&stmts("0000")), Some(true));
        // and the shapes it would refuse if the parser widened
        assert_eq!(is_source_order(&stmts("0...")), Some(false));
        assert_eq!(is_source_order(&stmts(".0..")), Some(false));
        assert_eq!(is_source_order(&stmts("011")), Some(false));
        assert_eq!(is_source_order(&stmts("01022")), Some(false));
    }

    /// The relaxation branch of `store_order` is unreachable on every shape
    /// the grid contains. This is the positive check with a printed count that
    /// `GAPS.md` asks for: it enumerates every run up to six statements over
    /// up to three producers and one filler and asserts the floor is always
    /// clearable.
    #[test]
    fn the_fallback_never_fires_on_any_enumerable_run() {
        let alphabet = ['.', '0', '1', '2'];
        let mut checked = 0usize;
        for n in 1..=6 {
            let mut idx = vec![0usize; n];
            loop {
                let spec: String = idx.iter().map(|&i| alphabet[i]).collect();
                let st = stmts(&spec);
                if let Some(ranks) = rank_order(&st) {
                    let u = head_slots(&st);
                    let order = store_order(&st).unwrap();
                    for (q, &k) in order.iter().enumerate() {
                        let floor = match st[k].producer {
                            None => 0,
                            Some(id) => {
                                u + ranks.iter().position(|&r| r == id).unwrap()
                            }
                        };
                        assert!(q >= floor, "fallback fired on {spec} at slot {q}");
                    }
                    checked += 1;
                }
                // odometer
                let mut i = n;
                loop {
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                    idx[i] += 1;
                    if idx[i] < alphabet.len() {
                        break;
                    }
                    idx[i] = 0;
                    if i == 0 {
                        idx = vec![alphabet.len(); n]; // sentinel: done
                        break;
                    }
                }
                if idx.iter().any(|&i| i >= alphabet.len()) {
                    break;
                }
            }
        }
        assert!(checked >= 5000, "only {checked} runs enumerated");
    }
}

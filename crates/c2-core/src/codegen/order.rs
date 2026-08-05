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
//! # More than one base symbol — board #564/#582, and it is now half open
//!
//! Lane `w-sym` measured 7,589 probe cells through one, two and three base
//! symbols. Three separable facts came out of it and only two are modelled:
//!
//! 1. **The cross-symbol PIN is exact.** The emitted symbol pattern equals the
//!    source symbol pattern on **7,589 of 7,589** cells, model-free. The store
//!    order only ever permutes *within* a symbol group. Board **#601**.
//! 2. **The STORE order** generalises with one change — [`store_order`]'s
//!    lowered `u` — and is exact at up to [`MAX_MULTISYM_PRODUCERS`]
//!    producers. **This module now answers instead of refusing.**
//! 3. **The producer EMISSION order** is a case split, [`producer_order`],
//!    board **#582**.
//!
//! 4. **The LAYOUT** — which store slots the producers are interleaved into —
//!    is modelled by [`layout_slots`] since `w-frame2`, on the domain where it
//!    is exact. Board **#602**.
//!
//! What is still refused, and why the refusal is not conservatism:
//!
//! * **A producer that crosses more than [`MAX_SYMBOL_CROSSINGS`] symbol-group
//!   boundaries before it is first consumed** — see [`layout_slots`]. This is
//!   the gate that makes the layout exact rather than 98.6 % correct, and
//!   removing it is a wrong-bytes emit, not a widening.
//! * **More than two producers through more than one symbol.** The store order
//!   is 97 % fit / 88 % holdout there and the residual is not one family.
//! * **More than [`MAX_MODELLED_PRODUCERS`] producers** on one symbol,
//!   matching [`alloc`](super::alloc)'s own domain. The store order alone is
//!   exact on four-producer runs (822 of 822 cells), but the register is not,
//!   and a caller needs both.
//! * **Producers of MIXED KIND.** Every ≤2-producer store-order miss on the
//!   holdout — 12 of 12 — is a cell mixing a constant with an address
//!   producer, which is board **#581**'s population. [`Stmt`] cannot express
//!   the kind and the parser only ever builds constant producers, so the
//!   domain holds today by construction; board **#603** records that a parser
//!   widening to address-valued producers invalidates it.
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

/// The domain of the **multi-symbol** store order, and it is smaller.
///
/// At two producers the store order is exact on 1867 fit and 1501 holdout
/// cells with 0 wrong; at three it is 97 % and 88 % and the residual is not one
/// family. `w-sym` measured both — `docs/SYMBOL.md` §2.3.
pub const MAX_MULTISYM_PRODUCERS: usize = 2;

/// The domain of the **LAYOUT**, and it is an axis nothing else in this module
/// uses: the number of symbol-group boundaries a producer's value crosses in
/// the final store order before it is first consumed.
///
/// `w-frame2` measured 62,365 cells and this is the constant that separates the
/// layout's exact half from its 98.6 % half. See [`layout_slots`].
pub const MAX_SYMBOL_CROSSINGS: usize = 2;

/// `(id, use count, first-use source index)` per distinct producer, in
/// first-use order.
fn distinct(stmts: &[Stmt]) -> Vec<(u32, usize, usize)> {
    let mut ps: Vec<(u32, usize, usize)> = Vec::new();
    for (i, s) in stmts.iter().enumerate() {
        if let Some(id) = s.producer {
            match ps.iter_mut().find(|p| p.0 == id) {
                Some(p) => p.1 += 1,
                None => ps.push((id, 1, i)),
            }
        }
    }
    ps
}

/// The run's distinct producers by **(use count descending, first-use
/// ascending)** — the rank, with no symbol gate.
fn global_rank(stmts: &[Stmt]) -> Vec<u32> {
    let mut ps = distinct(stmts);
    ps.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
    ps.into_iter().map(|p| p.0).collect()
}

/// The **producer EMISSION order**. `None` outside the domain.
///
/// * **One symbol** — the rank order, [`rank_order`], board **#561**.
/// * **More than one** — the order of **first consumption in the final store
///   order**, board **#582**.
///
/// The two are a **case split and not a unification**, and that is the
/// measurement rather than a simplification: `xboxheap`'s statement word emits
/// the count-2 producer first through ONE symbol and the count-1 producer
/// first through TWO, with the same statements, the same producers and the
/// same registers. `w-sym` searched the whole class of "sort the producers on
/// their own features" — 8,420 lexicographic keys over 10 signed features —
/// and no member covers both sides; nor does any of the four merge rules it
/// then built. `docs/SYMBOL.md` §3.
///
/// Nothing consumes this yet: [`schedule`] still refuses a multi-symbol run
/// because the **layout** is not modelled (see its docs). It is here because
/// it is the answer to #582 and because a future emitter needs it.
pub fn producer_order(stmts: &[Stmt]) -> Option<Vec<u32>> {
    let order = store_order(stmts)?;
    if single_symbol(stmts) {
        return Some(global_rank(stmts));
    }
    let mut out: Vec<u32> = Vec::new();
    for &k in &order {
        if let Some(id) = stmts[k].producer {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    Some(out)
}

/// The distinct producers of a run, **in rank order** — use count descending,
/// first-use source index ascending. Returns `None` outside the domain.
///
/// Deliberately still **single-symbol only**: this is the *emission* order,
/// and on more than one symbol the emission order is [`producer_order`]'s
/// first-consumption order instead. [`schedule`] gates on this function, which
/// is what keeps it refusing a multi-symbol run.
pub fn rank_order(stmts: &[Stmt]) -> Option<Vec<u32>> {
    if !single_symbol(stmts) {
        return None;
    }
    if distinct(stmts).len() > MAX_MODELLED_PRODUCERS {
        return None;
    }
    Some(global_rank(stmts))
}

fn single_symbol(stmts: &[Stmt]) -> bool {
    stmts.windows(2).all(|w| w[0].base == w[1].base)
}

/// The rank a store's producer takes among the producers of **its own base
/// symbol**, in the global rank order. With one symbol this is the global
/// rank, which is why the whole module reduces to `ORDER` there.
fn group_ranks(stmts: &[Stmt]) -> Vec<usize> {
    let rank = global_rank(stmts);
    stmts
        .iter()
        .map(|s| match s.producer {
            None => 0,
            Some(id) => rank
                .iter()
                .filter(|&&j| {
                    stmts
                        .iter()
                        .any(|t| t.base == s.base && t.producer == Some(j))
                })
                .position(|&j| j == id)
                .unwrap_or(0),
        })
        .collect()
}

/// One pass of the walk at a given `u`. `None` when some slot has **no**
/// allowed store — the relaxation `w-sched` rule 1 spelled as "if every
/// remaining store is blocked, source order wins".
fn walk(stmts: &[Stmt], ranks: &[usize], u: usize) -> Option<Vec<usize>> {
    let mut left: Vec<usize> = (0..stmts.len()).collect();
    let mut out: Vec<usize> = Vec::with_capacity(stmts.len());
    while !left.is_empty() {
        let q = out.len();
        let mut pick = None;
        for (i, &k) in left.iter().enumerate() {
            if stmts[k].producer.is_some() && q < u + ranks[k] {
                continue;
            }
            // The cross-symbol PIN. Measured model-free by `w-sym` on 7,589
            // cells: the emitted symbol pattern equals the source symbol
            // pattern, always. Board **#601**.
            if left[..i].iter().any(|&j| stmts[j].base != stmts[k].base) {
                continue;
            }
            pick = Some(i);
            break;
        }
        out.push(left.remove(pick?));
    }
    Some(out)
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
///
/// **One walk covers both regimes**, and that is the reduction proof rather
/// than a claim about it: a store of group rank `j` may not occupy position
/// `< u + j`, two stores through different symbols may not be reordered past
/// each other, and `u` is the **largest** value the run can afford. With one
/// symbol the pin is vacuous, the group rank is the global rank, and the
/// largest affordable `u` is `min(2, #unproduced)` — every single-symbol cell
/// this module was fitted on goes through this code and comes out unchanged.
///
/// `w-sched` rule 1's relaxation ("if every remaining store is blocked,
/// source order wins") is **deleted, not carried**, exactly as `w-order2`
/// deleted it: instead of relaxing a floor, lower `u` until no floor has to
/// be relaxed. On one symbol nothing changes, because the relaxation never
/// fires there — the enumerating test below still walks 5,460 runs to say so.
/// On more than one symbol it is worth **86.9 % → 98.4 %**, and it is what
/// makes the one-producer multi-symbol case exact (`docs/SYMBOL.md` §2).
pub fn store_order(stmts: &[Stmt]) -> Option<Vec<usize>> {
    let limit = if single_symbol(stmts) {
        MAX_MODELLED_PRODUCERS
    } else {
        MAX_MULTISYM_PRODUCERS
    };
    if distinct(stmts).len() > limit {
        return None;
    }
    let ranks = group_ranks(stmts);
    for u in (0..=head_slots(stmts)).rev() {
        if let Some(out) = walk(stmts, &ranks, u) {
            return Some(out);
        }
    }
    None
}

/// The **leading run of unproduced stores in the FINAL store order**, capped at
/// [`HEAD_SLOTS_MAX`] — `w-parse`'s board **#584** correction to `u`.
///
/// Distinct from [`head_slots`], which counts the unproduced stores of the run
/// wherever they land. The two agree on every single-symbol run this module was
/// fitted on (the enumerating test below walks 5,460 of them to say so) and
/// disagree on multi-symbol runs, where the pin can strand an unproduced store
/// behind a produced one. Measured by `w-frame2` over 62,365 cells: as the
/// layout's `u`, the leading run is **98.6 %** and the count is **62.9 %**.
fn lead_slots(stmts: &[Stmt], order: &[usize]) -> usize {
    let mut u = 0usize;
    for &k in order {
        if u >= HEAD_SLOTS_MAX || stmts[k].producer.is_some() {
            break;
        }
        u += 1;
    }
    u
}

/// The **LAYOUT** — the store slot immediately before which each producer is
/// emitted, indexed by [`producer_order`]. `None` outside the domain.
///
/// > Let `u` be the leading run of unproduced stores in the final store order,
/// > capped at 2. The producer at emission index `i` is emitted immediately
/// > before store slot `min(i, u)`.
///
/// # The domain gate is the whole finding — board #602
///
/// The clause above is `docs/ORDER.md`'s with #584's `u`, and on its own it is
/// **98.59 %**, not a rule. `w-frame2` swept the symbol mask exhaustively over
/// every word (62,365 cells; `w-sym`'s grid contained the counterexample family
/// six times, this one contains it 255 times in FIT alone) and the residual
/// named the axis:
///
/// > **`nsw`** — the number of **symbol-group transitions in the final store
/// > order, up to and including the store that first consumes this producer**.
///
/// It is exactly what separates `x_2sym` from `x_split`: the same statements,
/// the same store order, the same producer order and the same registers, one
/// store moved to the other symbol, `nsw = 1` against `nsw = 3` for the second
/// producer — and the second producer lands one slot later. Restricted to
/// `nsw <= `[`MAX_SYMBOL_CROSSINGS`], the clause is **exact**:
///
/// | population | in domain | exact | wrong |
/// |---|---:|---:|---:|
/// | fit | 30,271 | **30,271** | 0 |
/// | **holdout** (3 symbols, mixed kinds, ≥3 producers, length 7 — held out *wholesale by shape*) | 24,891 | **24,891** | **0** |
/// | external (`xboxheap`'s own word at every mask) | 54 | **54** | 0 |
///
/// A rival that answers on the *whole* population — `min(max(i, nsw−2), u)` —
/// reaches 99.44 % fit and 97.30 % holdout and is **deliberately not shipped**.
/// 99 % is a rule with a residual, and an emitter fed a 99 % layout emits wrong
/// bytes on the other 1 %. Board **#621** records it as measured and refused.
pub fn layout_slots(stmts: &[Stmt]) -> Option<Vec<usize>> {
    let order = store_order(stmts)?;
    let prods = producer_order(stmts)?;
    let u = lead_slots(stmts, &order);
    for &id in &prods {
        // the first store of `order` that consumes this producer
        let fc = order.iter().position(|&k| stmts[k].producer == Some(id))?;
        let nsw = (1..=fc)
            .filter(|&q| stmts[order[q]].base != stmts[order[q - 1]].base)
            .count();
        if nsw > MAX_SYMBOL_CROSSINGS {
            return None;
        }
    }
    Some((0..prods.len()).map(|i| i.min(u)).collect())
}

/// `Some(true)` when the LAYOUT puts **every** producer ahead of the first
/// store, `Some(false)` when it puts one later, `None` when the run is outside
/// [`layout_slots`]'s domain.
///
/// The store emitters call this as a **positive check**: they hoist their one
/// producer ahead of the whole run and refuse on `Some(false)`. Inert today by
/// construction — `leaf_store` accepts a produced run only when *every* store
/// takes the same literal, so there is no unproduced store, `u` is 0 and the
/// model puts the producer at slot 0 — and the point is board **#232**, where a
/// parser widening turned a clean refusal into a live wrong emit that survived
/// 255 commits. A widening that reaches this guard gets a refusal instead of a
/// producer in the wrong place.
///
/// **Additive-refusal by construction.** `Some(false)` is the only reading the
/// caller acts on, so a new answer here can *add* a refusal and can never turn
/// one into an accept.
pub fn producers_lead(stmts: &[Stmt]) -> Option<bool> {
    Some(layout_slots(stmts)?.iter().all(|&s| s == 0))
}

/// The full emitted schedule — producers and stores, in order. `None` outside
/// the domain.
///
/// Since `w-frame2` this answers a **multi-symbol** run wherever
/// [`layout_slots`] does; it used to refuse every one of them through
/// [`rank_order`], because the layout was the unmodelled third fact. Board
/// **#602**.
///
/// **This widening is additive-ACCEPT, and that is said plainly rather than
/// blurred into the guard's property.** `schedule` has no caller under
/// `crates/` — [`producers_lead`] is what `leaf_store` consumes — so the
/// widening moves no byte today. A future caller inherits [`layout_slots`]'s
/// domain gate, which is what makes the answer exact rather than 98.6 %
/// correct.
pub fn schedule(stmts: &[Stmt]) -> Option<Vec<Slot>> {
    let order = store_order(stmts)?;
    let prods = producer_order(stmts)?;
    let at = layout_slots(stmts)?;
    let mut out = Vec::with_capacity(stmts.len() + prods.len());
    let mut next = 0usize;
    for (slot, &k) in order.iter().enumerate() {
        while next < prods.len() && at[next] == slot {
            out.push(Slot::Producer(prods[next]));
            next += 1;
        }
        out.push(Slot::Store(k));
    }
    while next < prods.len() {
        out.push(Slot::Producer(prods[next]));
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

    fn s(p: Option<u32>, b: u32) -> Stmt {
        Stmt {
            producer: p,
            base: b,
        }
    }

    /// Multi-symbol cells, spelled `<producer><base>` per statement with `.`
    /// for an unproduced store: `".0" -> ".0 00"`. Every cell asserted below
    /// was compiled by real `c2.dll` under wibo at the workload's own flags
    /// and read out of a `/FAsc` listing — the grid is `work/w-sym/`.
    fn msym(spec: &str) -> Vec<Stmt> {
        spec.split_whitespace()
            .map(|t| {
                let mut c = t.chars();
                let p = c.next().unwrap();
                let b = c.next().unwrap().to_digit(10).unwrap();
                s(p.to_digit(10), b)
            })
            .collect()
    }

    /// `xboxheap.cpp`'s constructor — the cell `#564` is named after. Two
    /// symbols, and BOTH halves are now answered.
    #[test]
    fn xboxheap_two_symbols_is_answered_not_refused() {
        let xboxheap = msym(".0 .0 00 .0 11 11");
        assert_eq!(store_order(&xboxheap), Some(vec![0, 1, 2, 3, 4, 5]));
        assert_eq!(is_source_order(&xboxheap), Some(true));
        // #582: the count-1 producer comes out FIRST here …
        assert_eq!(producer_order(&xboxheap), Some(vec![0, 1]));
        assert_eq!(rank_order(&xboxheap), None);
        // … and #602: the LAYOUT is now answered too, with the sequence real
        // `c2` emits for this cell (`work/w-frame2/external.tsv`, `x_2sym_LL`:
        // `P0 S0.0 P1 S0.1 S0.2 S0.3 S1.4 S1.5`). Both crossings are 0 and 1,
        // inside `MAX_SYMBOL_CROSSINGS`.
        assert_eq!(layout_slots(&xboxheap), Some(vec![0, 1]));
        assert_eq!(
            schedule(&xboxheap),
            Some(vec![
                Slot::Producer(0),
                Slot::Store(0),
                Slot::Producer(1),
                Slot::Store(1),
                Slot::Store(2),
                Slot::Store(3),
                Slot::Store(4),
                Slot::Store(5),
            ])
        );

        // The SAME statement word through ONE symbol: the store order moves
        // and the count-2 producer comes out first. Same producers, same
        // counts, opposite answer — this pair is the whole of #582.
        let one = stmts("..0.11");
        assert_eq!(store_order(&one), Some(vec![0, 1, 3, 2, 4, 5]));
        assert_eq!(producer_order(&one), Some(vec![1, 0]));
        check("..0.11", "P1 S0 P0 S1 S3 S2 S4 S5");
    }

    /// **Board #602, the LAYOUT — and its boundary.** The three cells below are
    /// `xboxheap`'s statement word at three symbol masks, all four of them
    /// compiled by real `c2.dll` at the workload's flags
    /// (`work/w-frame2/external.tsv`). Same statements, same store order, same
    /// producer order, same registers; only the mask moves.
    ///
    /// The `split` mask is the cell that refutes the layout clause — c2 emits
    /// `P0 S0 S1 P1 S2 …`, layout `[0, 2]`, where `min(i, u)` says `[0, 1]` —
    /// and it is refused here rather than answered wrongly.
    #[test]
    fn the_layout_answers_two_masks_and_refuses_the_third() {
        // `x_2sym` and `x_late`: both producers cross <= 2 group boundaries.
        for cell in [".0 .0 00 .0 11 11", ".0 .0 00 .1 11 11"] {
            let v = msym(cell);
            assert_eq!(layout_slots(&v), Some(vec![0, 1]), "cell {cell}");
        }
        // `x_split`: one store moved to the other symbol. RECOMPUTED from the
        // rule rather than transcribed — the second producer's value crosses
        // three group boundaries before it is consumed, so the cell is outside
        // the domain and every entry point refuses.
        let split = msym(".0 .0 01 .0 11 11");
        let order = store_order(&split).expect("the STORE order is in domain");
        let fc = order
            .iter()
            .position(|&k| split[k].producer == Some(1))
            .expect("producer 1 is consumed");
        let nsw = (1..=fc)
            .filter(|&q| split[order[q]].base != split[order[q - 1]].base)
            .count();
        assert_eq!(nsw, 3, "the crossings are what put this cell out of domain");
        assert!(nsw > MAX_SYMBOL_CROSSINGS);
        assert_eq!(layout_slots(&split), None);
        assert_eq!(schedule(&split), None);
        assert_eq!(producers_lead(&split), None);
        // The STORE order and the PRODUCER order are still answered — `w-sym`
        // owns those two facts and this lane did not narrow them.
        assert_eq!(order, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(producer_order(&split), Some(vec![0, 1]));
    }

    /// The guard `leaf_store` consumes is **inert on what the parser admits**,
    /// which is the same sentence the ORDER and ALLOC guards carry. A run whose
    /// stores all take one literal has no unproduced store, so `u` is 0 and the
    /// producer belongs at slot 0 — which is where the emitter hoists it.
    #[test]
    fn the_layout_guard_is_inert_on_what_the_parser_admits() {
        for cell in ["00 00 00", "00 01 00", "00 01 01 00", "00 00 01 01"] {
            assert_eq!(producers_lead(&msym(cell)), Some(true), "cell {cell}");
        }
        // An unproduced head is what moves a producer off slot 0, and the
        // parser does not admit one beside a produced store today.
        assert_eq!(producers_lead(&msym(".0 .0 00 11")), Some(false));
    }

    /// `u` is the LEADING RUN of unproduced stores in the final store order
    /// (#584), not [`head_slots`]'s count of them. The two agree on every
    /// single-symbol run — the enumerating test below says so — and the cell
    /// that separates them is one where the pin strands an unproduced store
    /// behind a produced one.
    #[test]
    fn the_layout_u_is_the_leading_run_not_the_count() {
        let v = msym("00 01 .1 00 .0 01");
        let order = store_order(&v).expect("in domain");
        assert!(
            lead_slots(&v, &order) < head_slots(&v),
            "this cell exists to separate the two readings of u"
        );
    }

    /// **The reduction, as code rather than as a claim beside it.** Swapping
    /// the layout's `u` from [`head_slots`] to [`lead_slots`] must move nothing
    /// on ONE symbol, which is the whole population `ORDER` was fitted on.
    /// Enumerated over every run of length 1..=6 with up to 3 producers.
    #[test]
    fn the_two_readings_of_u_agree_on_every_single_symbol_run() {
        let mut n = 0usize;
        for len in 1..=6usize {
            for code in 0..4u32.pow(len as u32) {
                let v: Vec<Stmt> = (0..len)
                    .map(|i| {
                        let d = (code >> (2 * i)) & 3;
                        s(if d == 0 { None } else { Some(d - 1) }, 0)
                    })
                    .collect();
                let Some(order) = store_order(&v) else { continue };
                n += 1;
                assert_eq!(
                    lead_slots(&v, &order),
                    head_slots(&v),
                    "the readings of u disagree on a SINGLE-symbol run: {v:?}"
                );
            }
        }
        assert!(n >= 5000, "the enumeration must be a population, got {n}");
    }

    /// The lowered `u`. `{V0 V0 T V0 T}` with the FIRST store through the
    /// second symbol: `min(2, #unproduced)` is 2, the two unproduced stores
    /// are pinned behind produced ones by the cross-symbol clause, and
    /// `w-parse`'s SYMORDER relaxes a floor and moves a store. Real `c2`
    /// leaves the run in source order, which is `u = 0`.
    #[test]
    fn u_is_lowered_rather_than_a_floor_relaxed() {
        for cell in [
            "01 00 .0 00 .0",
            "00 01 .1 00 .0 01",
            "00 01 .1 .1 00 00",
            "00 00 .1 00 00 .1",
        ] {
            let st = msym(cell);
            let n = st.len();
            assert_eq!(
                store_order(&st),
                Some((0..n).collect::<Vec<_>>()),
                "cell {cell}"
            );
            assert_eq!(is_source_order(&st), Some(true), "cell {cell}");
        }
    }

    /// The PIN: two stores through different symbols are never reordered past
    /// each other, so the emitted symbol pattern is the source one. Measured
    /// model-free on 7,589 of 7,589 cells — board #601. Enumerated here as a
    /// positive check with a printed count.
    #[test]
    fn the_emitted_symbol_pattern_is_always_the_source_pattern() {
        let alphabet = ['.', '0', '1', '2'];
        let mut checked = 0usize;
        for n in 2..=5 {
            for mask in 0..(1u32 << n) {
                for w in 0..alphabet.len().pow(n as u32) {
                    let st: Vec<Stmt> = (0..n)
                        .map(|i| {
                            let c = alphabet[(w / alphabet.len().pow(i as u32))
                                % alphabet.len()];
                            s(c.to_digit(10), (mask >> i) & 1)
                        })
                        .collect();
                    if let Some(order) = store_order(&st) {
                        let emitted: Vec<u32> =
                            order.iter().map(|&k| st[k].base).collect();
                        let source: Vec<u32> =
                            st.iter().map(|t| t.base).collect();
                        assert_eq!(emitted, source, "pin broken");
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked >= 3000, "only {checked} runs enumerated");
    }

    /// Three symbols: the same rule, no new constant. `w-sym`'s whole
    /// three-symbol tier was HELD OUT by the preregistered partition and
    /// scored **406 of 406** out of sample at two producers.
    #[test]
    fn three_symbols_need_no_new_constant() {
        assert_eq!(store_order(&msym("00 01 02")), Some(vec![0, 1, 2]));
        assert_eq!(store_order(&msym(".0 01 12 02")), Some(vec![0, 1, 2, 3]));
        assert_eq!(producer_order(&msym("00 01 12 02")), Some(vec![0, 1]));
    }

    /// Past two producers a multi-symbol run is refused: the store order is
    /// 97 % there and the residual is not one family.
    #[test]
    fn three_producers_through_two_symbols_are_refused() {
        assert_eq!(store_order(&msym("00 10 21 20 01")), None);
        assert_eq!(is_source_order(&msym("00 10 21 20 01")), None);
        assert_eq!(producer_order(&msym("00 10 21 20 01")), None);
        // …and the same word through ONE symbol is still in domain.
        assert!(store_order(&stmts("01220")).is_some());
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

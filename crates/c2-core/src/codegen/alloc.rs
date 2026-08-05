//! **ALLOC** — which register c2 gives each value-producer of a store run.
//!
//! [`schedule`](super::schedule) settles the ORDER of a store run and says so
//! explicitly: *"the allocation is a SECOND INPUT and it is open"*. This module
//! is that second input. `docs/ALLOC.md` is the write-up; the grid is
//! `work/w-alloc/` (gitignored — the generators are committed, the `.cod` and
//! `.obj` are not).
//!
//! Before this module, `crates/c2-il/src/func/body/shapes/leaf_store.rs`
//! carried **four fitted allocation rules, each refuted by one of the others**
//! (use count by `A1`, live-range length by `A2`, last-use by `B6`, first-use
//! by `B4`/`B7`). All four killer cells are derived consequences of the single
//! rule below and are reproduced in the tests.
//!
//! # The rule
//!
//! Enumerate the run's distinct producers. Order them by
//!
//! 1. **use count, descending** — the number of stores that consume the value;
//! 2. on a tie, **register-derived** producers before **constant** ones;
//! 3. on a tie within the register-derived, **source order**;
//! 4. on a tie within the constants, **REVERSE source order**.
//!
//! and hand out the pool registers **descending** — `r11`, `r10`, `r9`, `r8`,
//! … — in that order. The pool is the free volatile registers taken
//! highest-first, minus those holding live-in formals. **`r12` is never used**
//! (board #543 — recorded, not explained).
//!
//! Clauses 3 and 4 carry **opposite signs inside one sort**, which is why the
//! rule is not a priority function. A preregistered exhaustive search over
//! **52,416 priority-function allocators** — 4 scan directions × 3 assignment
//! points × 2 pool walks × 2,184 lexicographic keys over 7 base features —
//! tops out at **179 of 236** fit cells with its residual **exactly** the tie
//! tier, 0 misses at every non-tie count. That negative is the same shape as
//! `w-sched`'s 13,104-configuration result and it is the reason this rule is
//! believable rather than merely fitted.
//!
//! # What this module refuses, and why the refusals are not conservatism
//!
//! * **A multiply producer.** `mulli` is not held live beside another
//!   producer at all: it is materialised one at a time, in `r11`, immediately
//!   before the stores that consume it. `{a=u*3; b=u*5;}` is
//!   `mulli r11 ; stw ; mulli r11 ; stw` — a different regime, measured, not a
//!   counterexample.
//! * **More than [`MAX_MODELLED_PRODUCERS`] producers.** Past three, c2 starts
//!   REUSING a freed register in preference to taking a fresh one, and the two
//!   probed four-producer runs with identical statement structure **disagree**
//!   (`li`-valued reuses `r11`; `addi`-valued takes a fresh `r8`). That is
//!   board #541 and it is open.
//! * **A run mixing constant and register-derived producers.** This refusal is
//!   **load-bearing, and clause 2 is REFUTED on the mixed run** — see
//!   "Clause 2 is refuted" below. It used to read *"clause 2 is measured only
//!   on the supplementary probe, never on the held-out partition, so it is not
//!   shipped"*, which understated it: the clause is not merely untested, it is
//!   wrong.
//! * **A pool too small to serve the run.** Once the volatiles run out c2
//!   descends into registers freed by already-emitted stores — including `r4`
//!   and even `r3`, the base pointer itself — and then into `r30`/`r31` with a
//!   save/restore pair. Open.
//!
//! # Evidence
//!
//! | population | cells | in domain | exact | wrong |
//! |---|---:|---:|---:|---:|
//! | fit | 242 | 236 | **236** | 0 |
//! | **holdout** | 284 | 257 | **250** | **0** (7 refused) |
//! | killer cells | 6 | 6 | **6** | 0 |
//!
//! The holdout partition was declared in
//! `docs/rungs/_2026-08-05-w-alloc-prereg.md` §6 **before** the grid was
//! generated, written by the generator into a file the fitter refuses to open,
//! and scored only after this rule was frozen at commit `8973ffc`.
//!
//! # Clause 2 is REFUTED, and three of the four clauses are unreachable
//!
//! **Board #836.** Lane `w-next` measured 24 mixed-kind cells and fitted a
//! single key — *`uses + (register-derived ? 1 : 0)`, descending* — with 0
//! misses, and deliberately left it unshipped. Lane `w-alloc2` took it to a
//! **fresh** holdout (`work/w-alloc2/freshgrid.py`, 60 cells / 56 graded) and
//! **refuted it on 7**. All 24 fitted cells spell the register-derived producer
//! the same way, `(int)&q`, and the bonus is a property of that spelling rather
//! than of the kind:
//!
//! ```text
//!   addi rX,3,K   (&s->inner, stored INTO s->inner)   1 use   BEATS   li 1 use
//!   add  rX,4,5   (u + v)                             1 use   loses to li 1 use
//!   addi rX,4,5   (u + 5)                             1 use   loses to li 1 use
//!   slwi rX,4,3   (u << 3)                            1 use   loses to li 1 use
//! ```
//!
//! The emitted instruction ORDER is identical in the deciding pair, so this is
//! allocation and not [`super::schedule`]. So **clause 2 as written above — "on
//! a tie, register-derived producers before constant ones" — is false**:
//! `B-notself-1v1` is a register-derived producer at 1 use losing a tie to a
//! constant at 1 use.
//!
//! Over 81 mixed cells graded against real `c2.dll`
//! (`work/w-alloc2/mutate.py`): clause 1 alone is wrong on **29**, clause 2
//! alone on **35**, w-next's key on **20**, and **this module's refusal on 0**,
//! because a refusal is never wrong. That is the whole argument for the refusal
//! staying.
//!
//! **The surviving candidate is not shipped either.** `H-self` — the bonus is
//! worth ~1.5 uses and attaches to a producer whose value is stored *into the
//! object it points at* — is wrong on **1** of the 81, and it is **fitted on
//! the cells that produced it**, which is exactly where w-next's key stood
//! before its fresh holdout killed it. Its one miss, `F4-shift-r2k1`, sits on a
//! third axis nobody has modelled: `work/w-alloc2/bisect.py` shows the C++
//! reference binding `L& q = s->inner;` moves both the schedule and the
//! allocation, and **all 24 of w-next's cells carry it and none varies it**.
//!
//! **And clauses 2, 3 and 4-for-register-derived are unreachable from the
//! emitter today**, which is why none of this moves a byte:
//! `super::super::leaf::store` builds every [`Producer`] with
//! [`ProducerKind::Constant`], hard-coded, because a store's value there is
//! either a literal or a formal already live in a register. Only clause 1 and
//! clause 4 ever execute. A lane that widens the parser to admit an interior
//! address as a store value makes the mixed run reachable and inherits every
//! paragraph above.

/// How a producer's value is materialised. The distinction is read off the IL,
/// never off the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerKind {
    /// Reads no register: `li`, `lis`+`ori`.
    Constant,
    /// Reads a register: `addi`, `rlwinm`, `add`, …
    RegisterDerived,
    /// A multiply. Its own regime — see the module docs.
    Multiply,
}

/// One distinct value-producer of a store run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Producer {
    /// Identity. Two statements sharing an `id` share one producer, because c2
    /// CSEs equal constants and equal address binds.
    pub id: u32,
    pub kind: ProducerKind,
    /// How many stores consume this value. **Clause 1 sorts on this**, and it
    /// is the field every one of the four refuted rules got wrong.
    pub uses: usize,
    /// Source index of the first statement naming this value — clauses 3 and 4.
    pub first: usize,
}

/// The top of the pool. `r12` is never allocated (board #543).
pub const POOL_TOP: u8 = 11;

/// Past three producers c2 begins reusing freed registers and the probed cells
/// disagree. Board #541.
pub const MAX_MODELLED_PRODUCERS: usize = 3;

/// The allocation, or `None` when the run is outside the modelled regime.
///
/// `pool_floor` is the lowest register number free for the whole run — one
/// above the highest register holding a live-in formal. For a run in a
/// function taking `this` plus `n` integer formals that is `4 + n`.
///
/// Returns `(producer id, register number)` pairs, in the order the registers
/// were handed out, so the caller can see both the assignment and the rank.
pub fn allocate(producers: &[Producer], pool_floor: u8) -> Option<Vec<(u32, u8)>> {
    if producers.is_empty() || producers.len() > MAX_MODELLED_PRODUCERS {
        return None;
    }
    if producers.iter().any(|p| p.kind == ProducerKind::Multiply) {
        return None;
    }
    // **A mixed run refuses, and clause 2 is REFUTED rather than merely
    // untested** (board #836). Over 81 mixed cells graded against real `c2`,
    // clause 1 alone is wrong on 29 and clause 2 alone on 35; this refusal is
    // wrong on 0. `the_mixed_refusal_covers_the_measured_refutations` below
    // pins the seven cells any future mixed rule has to reproduce.
    let constant = producers[0].kind == ProducerKind::Constant;
    if producers
        .iter()
        .any(|p| (p.kind == ProducerKind::Constant) != constant)
    {
        return None;
    }
    if pool_floor > POOL_TOP {
        return None;
    }
    if ((POOL_TOP - pool_floor + 1) as usize) < producers.len() {
        return None;
    }

    let mut order: Vec<&Producer> = producers.iter().collect();
    order.sort_by(|a, b| {
        // Clause 1: use count, descending.
        b.uses.cmp(&a.uses).then_with(|| {
            // Clauses 3/4. The tiebreak REVERSES only for a SHARED constant; a
            // count-1 tie runs forward whatever the kind. That sign flip,
            // inside one sort, is what puts the rule outside every
            // priority-key class.
            if constant && a.uses >= 2 {
                b.first.cmp(&a.first)
            } else {
                a.first.cmp(&b.first)
            }
        })
    });
    Some(
        order
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, POOL_TOP - i as u8))
            .collect(),
    )
}

/// True when every producer of the run lands in `reg`.
///
/// The store emitters put every materialised value in `r11` today, which
/// [`allocate`] confirms is right for a run with **one** producer and wrong for
/// every run with two or more. This is the positive check the emitters call
/// before emitting, so a widening of the *parser* cannot silently turn a clean
/// refusal into a wrong register — board **#232** is the precedent, a parser
/// widening that became a live wrong emit and survived 255 commits.
pub fn all_in(producers: &[Producer], pool_floor: u8, reg: u8) -> bool {
    match allocate(producers, pool_floor) {
        Some(a) => a.iter().all(|&(_, r)| r == reg),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compact spelling: one char per statement, the char naming the value.
    /// `k` picks the kind for every producer in the run.
    fn run(spec: &str, k: ProducerKind) -> Vec<Producer> {
        let mut out: Vec<Producer> = Vec::new();
        for (i, c) in spec.chars().enumerate() {
            let id = c as u32;
            match out.iter_mut().find(|p| p.id == id) {
                Some(p) => p.uses += 1,
                None => out.push(Producer {
                    id,
                    kind: k,
                    uses: 1,
                    first: i,
                }),
            }
        }
        out
    }

    fn regs(spec: &str, k: ProducerKind) -> Vec<(char, u8)> {
        let mut a: Vec<(char, u8)> = allocate(&run(spec, k), 4)
            .unwrap()
            .iter()
            .map(|&(id, r)| (char::from_u32(id).unwrap(), r))
            .collect();
        a.sort();
        a
    }

    /// The four allocation rules `leaf_store.rs` records as refuted are all
    /// derived consequences of ALLOC. MEASURED through the real c2 at the
    /// workload's flags — `work/w-alloc/external.py` recompiles every one.
    #[test]
    fn the_four_refuted_rules_killer_cells() {
        // B4  {a=1;b=2;c=3;d=1}  refuted "first-use order"
        assert_eq!(
            regs("1231", ProducerKind::Constant),
            vec![('1', 11), ('2', 10), ('3', 9)]
        );
        // B7  {a=1;b=2;c=3;d=2;e=1}  refuted "use count by A1"
        assert_eq!(
            regs("12321", ProducerKind::Constant),
            vec![('1', 10), ('2', 11), ('3', 9)]
        );
        // A1  {a=1;b=2;c=1;d=2}
        assert_eq!(
            regs("1212", ProducerKind::Constant),
            vec![('1', 10), ('2', 11)]
        );
        // B6  {a=1;b=1;c=2;d=2;e=2}  refuted "last-use"
        assert_eq!(
            regs("11222", ProducerKind::Constant),
            vec![('1', 10), ('2', 11)]
        );
    }

    /// Clause 1 is the USE COUNT and it outranks every tiebreak. These are the
    /// unequal-count patterns that refuted the SHARED-vs-SIMPLE framing the
    /// prereg's H1 was stated in.
    #[test]
    fn clause_one_is_the_use_count() {
        for k in [ProducerKind::Constant, ProducerKind::RegisterDerived] {
            // 0 used twice, 1 used three times -> the busier value takes r11.
            assert_eq!(regs("00111", k), vec![('0', 10), ('1', 11)]);
            assert_eq!(regs("01111", k), vec![('0', 10), ('1', 11)]);
            assert_eq!(regs("011110", k), vec![('0', 10), ('1', 11)]);
        }
    }

    /// Clauses 3 and 4 carry OPPOSITE SIGNS. A count-1 tie runs forward for
    /// both kinds; a count>=2 tie reverses for constants only.
    #[test]
    fn the_tiebreak_sign_flips_on_the_count_and_on_the_kind() {
        // count-1 tie: forward, both kinds.
        assert_eq!(
            regs("012", ProducerKind::Constant),
            vec![('0', 11), ('1', 10), ('2', 9)]
        );
        assert_eq!(
            regs("012", ProducerKind::RegisterDerived),
            vec![('0', 11), ('1', 10), ('2', 9)]
        );
        // count-2 tie: constants REVERSE, register-derived do not.
        assert_eq!(
            regs("0101", ProducerKind::Constant),
            vec![('0', 10), ('1', 11)]
        );
        assert_eq!(
            regs("0101", ProducerKind::RegisterDerived),
            vec![('0', 11), ('1', 10)]
        );
        assert_eq!(
            regs("0011", ProducerKind::Constant),
            vec![('0', 10), ('1', 11)]
        );
        assert_eq!(
            regs("0011", ProducerKind::RegisterDerived),
            vec![('0', 11), ('1', 10)]
        );
    }

    /// The pool is walked highest-first and starts below the live-in formals.
    #[test]
    fn the_pool_starts_below_the_live_in_formals() {
        let r = run("012", ProducerKind::Constant);
        assert_eq!(
            allocate(&r, 4),
            Some(vec![(48, 11), (49, 10), (50, 9)]),
            "with one formal the pool is r11..r5"
        );
        // Six formals hold r4..r9, so only r11 and r10 are free: three
        // producers do not fit and the allocator REFUSES rather than guessing.
        assert_eq!(allocate(&r, 10), None);
        // Two producers do fit.
        assert!(allocate(&run("01", ProducerKind::Constant), 10).is_some());
    }

    /// The refusals, each one a measured different regime rather than caution.
    #[test]
    fn refusals_are_measured_regimes_not_caution() {
        // a multiply is never held live beside another producer
        assert_eq!(allocate(&run("01", ProducerKind::Multiply), 4), None);
        // past three producers c2 reuses a freed register (board #541)
        assert_eq!(allocate(&run("0123", ProducerKind::Constant), 4), None);
        // a mixed run: clause 2 was never held out, so it is not shipped
        let mixed = vec![
            Producer {
                id: 0,
                kind: ProducerKind::Constant,
                uses: 2,
                first: 0,
            },
            Producer {
                id: 1,
                kind: ProducerKind::RegisterDerived,
                uses: 2,
                first: 1,
            },
        ];
        assert_eq!(allocate(&mixed, 4), None);
        assert_eq!(allocate(&[], 4), None);
    }

    /// **The seven cells any future mixed-kind rule has to reproduce.**
    ///
    /// Board **#836**. Each row is a mixed run measured against real `c2.dll`
    /// at the workload's own flags (`work/w-alloc2/freshgrid.py`,
    /// `opgrid.py`), and each is a cell where the obvious rule emits the WRONG
    /// register rather than a refusal:
    ///
    /// ```text
    ///   cell              producer            uses  c2 gives r11 to
    ///   F4-add-r1k1       add  rX,4,5   (u+v)  1/1   the CONSTANT
    ///   F4-add-r1k2       add  rX,4,5          1/2   the CONSTANT
    ///   F4-addi-r1k1      addi rX,4,5   (u+5)  1/1   the CONSTANT
    ///   F4-addi-r1k2      addi rX,4,5          1/2   the CONSTANT
    ///   F4-shift-r1k1     slwi rX,4,3   (u<<3) 1/1   the CONSTANT
    ///   F4-shift-r1k2     slwi rX,4,3          1/2   the CONSTANT
    ///   F4-shift-r2k1     slwi rX,4,3          2/1   the CONSTANT  <- clause 1 too
    /// ```
    ///
    /// w-next's key (`uses + register-derived ? 1 : 0`) says the
    /// register-derived producer takes `r11` in all seven. **Shipping it would
    /// have produced wrong bytes, not a refusal** — which is why the refusal
    /// below is the shipped answer.
    ///
    /// This test fails the moment [`allocate`] answers a mixed run, so a lane
    /// that ships one has to come here and state what it measured.
    #[test]
    fn the_mixed_refusal_covers_the_measured_refutations() {
        // (register-derived uses, constant uses) for the seven cells above.
        for &(ru, cu) in &[(1, 1), (1, 2), (1, 1), (1, 2), (1, 1), (1, 2), (2, 1)] {
            let mixed = vec![
                Producer {
                    id: 0,
                    kind: ProducerKind::Constant,
                    uses: cu,
                    first: 0,
                },
                Producer {
                    id: 1,
                    kind: ProducerKind::RegisterDerived,
                    uses: ru,
                    first: 1,
                },
            ];
            assert_eq!(
                allocate(&mixed, 4),
                None,
                "a mixed run at (reg {ru}, const {cu}) must REFUSE: real c2 \
                 gives r11 to the constant here, and every fitted rule gives \
                 it to the register-derived producer"
            );
            // …and the guard the emitters actually call must decline too.
            assert!(!all_in(&mixed, 4, 11));
        }
    }

    /// Three of the four clauses are unreachable from the emitter, and this
    /// pins the reason rather than leaving it to prose: a pure-constant run —
    /// the only kind `leaf::store` can build — never consults clause 2 or
    /// clause 3, so widening the parser is what makes them live.
    #[test]
    fn a_pure_constant_run_is_the_only_kind_the_emitter_can_build() {
        // Same use counts, both kinds. The pure runs answer; the mix refuses.
        assert!(allocate(&run("0011", ProducerKind::Constant), 4).is_some());
        assert!(allocate(&run("0011", ProducerKind::RegisterDerived), 4).is_some());
        let mixed = vec![
            Producer {
                id: 0,
                kind: ProducerKind::Constant,
                uses: 2,
                first: 0,
            },
            Producer {
                id: 1,
                kind: ProducerKind::RegisterDerived,
                uses: 2,
                first: 1,
            },
        ];
        assert_eq!(allocate(&mixed, 4), None);
    }

    /// The guard the store emitters call. One producer is r11 — which is what
    /// the port emits today and why this is inert; two or more never are.
    #[test]
    fn all_in_r11_holds_for_one_producer_and_no_more() {
        assert!(all_in(&run("000", ProducerKind::Constant), 4, 11));
        assert!(all_in(&run("0", ProducerKind::Constant), 4, 11));
        assert!(!all_in(&run("01", ProducerKind::Constant), 4, 11));
        assert!(!all_in(&run("0101", ProducerKind::Constant), 4, 11));
        // out of the modelled regime => not provable => refuse
        assert!(!all_in(&run("0123", ProducerKind::Constant), 4, 11));
        assert!(!all_in(&run("00", ProducerKind::Multiply), 4, 11));
    }
}

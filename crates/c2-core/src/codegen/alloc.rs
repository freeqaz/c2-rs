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
//! **`H-self` is REFUTED too** (board #857, lane w-refbind) — the bonus worth
//! ~1.5 uses attaching to a producer whose value is stored *into the object it
//! points at* scored 1 wrong of 81 on the cells that produced it, and then
//! **11 wrong of 72** on a frozen never-fitted holdout
//! (`work/w-refbind/holdout_pred.tsv`, predictions committed before any cell
//! was compiled). It dies on its *negative* side — `extsh` and a `lwz` load
//! take the bonus register at 1-vs-1 where H-self forbids it everywhere — so
//! it dies independently of the reference-binding axis it was suspected of
//! mismodelling. No allocation key on record survives off its own cells.
//!
//! **The NARROW lift is refused too, and it was measured rather than
//! assumed.** Board **#868**, lane `w-seam`. The remaining way to open
//! `xboxheap.cpp` was to lift the refusal only where **clause 1 decides with no
//! tie** — the register-derived producer at *strictly* more uses than the
//! constant, so no tie-break, no kind bonus and neither refuted key is
//! consulted. 36 cells at the workload's own flags, **36 graded, 0 out of
//! regime, 12 MISS**, every miss a `slwi` cell and the row losing at a
//! use-count advantage of **three** as flatly as at one:
//!
//! ```text
//!   addi-interior  12 / 0 / 0      add  12 / 0 / 0      slwi  0 / 12 / 0
//! ```
//!
//! So there is no threshold to narrow around, and the separating axis is the
//! **spelling** — which [`ProducerKind`] cannot represent.
//! `the_strict_use_count_subcase_is_refused_too` pins the six gaps.
//!
//! # RULE BIND is REFUTED — the seventh, and the first one that was not a key
//!
//! **Board #1067**, lane `w-alloc3`. Every entry above answers *which of two
//! live producers gets `r11`*. This one asked a different question and died
//! anyway, which is why it is worth its own paragraph rather than a row.
//!
//! `w-seq` (#969) dissected 503 splice failures and found every one is a
//! **field** perturbation with no reordering anywhere: 286 source renames
//! `r3 → r4`, 123 destination renames `r3 → r11`, ~92 displacement folds.
//! **RULE BIND** was the obvious reading of that:
//!
//! ```text
//!   BIND  every SOURCE register field still holding a callee formal is
//!         rewritten to the register the caller's actual already lives in.
//!   TEMP  the destination of the instruction producing the callee's return
//!         value stays r3 iff that value is the caller's returned value, and
//!         otherwise becomes POOL_TOP = r11.
//! ```
//!
//! It reproduces, from published bytes and with no toolchain, **all five**
//! recorded witnesses — `w-seq`'s 123 (`?back@?$vector` against `?end@`), its
//! 286 (`?Release@Object@Hmx@@` against `?Release@ObjRef@@`), and its hand
//! cells `s01`, `s03` and `s11` — and it is **33 of 33** on a fit grid.
//!
//! On a **frozen, never-fitted** holdout of 46 cells it is **5 WRONG of 38**
//! in domain (`work/w-alloc3/gridH.tsv`; sources and their `sha256` committed
//! at `5832dd14` before a cell was compiled, the rule frozen at `245945c2`).
//! **The shipped refusal is wrong on 0 of the same 71 cells.**
//!
//! **It dies because c2 does not rename a body — it RECOMPILES an
//! expression.** Four of the five misses are one three-formal callee at the
//! permutations that put its commutative pair the other way round:
//!
//! ```text
//!   int g(int a, int b, int c) { return a - b + c; }   ; sub r11,r3,r4
//!                                                     ; add r3,r11,r5
//!   int f(int x0,int x1,int x2){ return g(x2,x1,x0); } ; H-perm-210
//!      RULE BIND   sub r11,r5,r4 ; add r3,r11,r3      (a renaming)
//!      c2          sub r11,r3,r4 ; add r3,r11,r5      (the callee's own bytes)
//! ```
//!
//! and the fifth is sharper still — `int g(int a){return -a;}` at a site
//! `g(x1) + 4` becomes **`subfic r3,r4,4`**, one word, an opcode that appears
//! nowhere in the callee:
//!
//! ```text
//!      RULE BIND   neg r11,r4 ; addi r3,r11,4         7d6400d0 386b0004
//!      c2          subfic r3,r4,4                     20640004
//! ```
//!
//! So `w-seq` §10.2's caution is now measured rather than argued: **the field
//! diff says WHAT changed and not WHAT DECIDES IT**, and a rule stated as a
//! field edit is a description of the output. The two clauses are not equally
//! dead, and the split is the useful part:
//!
//! * **TEMP survived everything this lane could throw at it.** The result of
//!   an inlined callee lands in `POOL_TOP` = `r11` and nowhere else, at caller
//!   formal counts **1 through 8** (16 of 16 on the holdout's `H-wide`), at
//!   every bound position, with the caller's `r3` provably dead, and even when
//!   the callee already holds a temp in `r11` (4 of 4 on `H-temp`). The rival
//!   *"the temp is the lowest free volatile"* is refuted: at five caller
//!   formals the lowest free volatile is `r8` and c2 emits `lwz r11,4(r7)`.
//!   That is a **direct measurement of `POOL_TOP` in a regime this module has
//!   never been exercised in**, and it agrees with #543/#605.
//! * **BIND is what died**, and only where the caller's expression admits a
//!   different-but-equal encoding.
//!
//! A successor may not restate BIND as a field edit. What it owes first is a
//! decision procedure for c2's **operand canonicalisation** — every one of the
//! five misses is one — and that is a fresh frozen grid, not another pass over
//! these cells (#912's standing lesson).
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

    /// **The NARROW lift is refused too, and this pins the grid that killed
    /// it** — lane `w-seam`, board **#868**, `work/w-seam/grida.out`.
    ///
    /// The obvious way to open `xboxheap.cpp`'s configuration is to lift the
    /// mixed refusal only for the sub-case *clause 1 decides with no tie*:
    /// two producers, one register-derived and one single-word constant, with
    /// the register-derived one at **strictly more uses**. No tie-break clause
    /// runs, no kind bonus, neither refuted key is consulted — it looks like
    /// pure clause 1 and therefore like conservatism.
    ///
    /// **It is not.** 36 cells compiled at the workload's own flags and graded
    /// against real `c2.dll` — three spellings × six use-count gaps × two body
    /// kinds (leaf, and a run before a trailing call) — **36 graded, 0 out of
    /// regime, 12 MISS**:
    ///
    /// ```text
    ///   spelling         hit / miss / out-of-regime
    ///   addi-interior    12 /  0 / 0     (int)&q   — xboxheap's own spelling
    ///   add              12 /  0 / 0     (u + v)
    ///   slwi              0 / 12 / 0     (u << 3)  — the CONSTANT takes r11
    /// ```
    ///
    /// The `slwi` row loses at a use-count advantage of **three** (reg 4 uses
    /// against const 1) exactly as flatly as at one, so there is no threshold
    /// the lift could be narrowed around, and both body kinds agree cell for
    /// cell, so a frame does not rescue it either. The separating axis is the
    /// **spelling**, which is [`ProducerKind::RegisterDerived`]'s own blind
    /// spot — the enum cannot represent the distinction the answer turns on.
    ///
    /// A lane that ships the strict-gap sub-case has to come here and say what
    /// it measured that these 36 cells did not.
    #[test]
    fn the_strict_use_count_subcase_is_refused_too() {
        // Every (reg uses, const uses) gap of `work/w-seam/grida.py`, each one
        // a cell where clause 1 alone decides and 12 of 36 graded objs
        // disagree with it.
        for &(ru, cu) in &[(2, 1), (3, 1), (3, 2), (4, 1), (4, 2), (4, 3)] {
            assert!(ru > cu, "the sub-case is a STRICT use-count advantage");
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
                "the strict-gap mixed run at (reg {ru}, const {cu}) must \
                 REFUSE: real c2 gives r11 to the CONSTANT for a `slwi` \
                 producer at every one of these gaps (w-seam GRID A, 12 of 36)"
            );
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

    /// **`POOL_TOP` MEASURED IN A REGIME THIS MODULE HAS NEVER BEEN EXERCISED
    /// IN** — lane `w-alloc3`, board **#1068**, the surviving half of the
    /// refuted RULE BIND.
    ///
    /// Every grid behind this module so far has been a *store run* in a leaf.
    /// `w-alloc3`'s `H-wide` family is a different shape entirely — the single
    /// value an **inlined callee** returns, consumed by one more instruction —
    /// and it lands in `r11` at **every** caller formal count from 1 to 8, at
    /// the first and last bound position, with the caller's `r3` provably
    /// dead. 16 of 16 on a frozen holdout, graded against real `c2.dll`:
    ///
    /// ```text
    ///   int* g(V* v) { return v->b; }                      lwz  r3, 4(r3)
    ///   int* f(int,int,int,int,V* x4){ return g(x4)-1; }   lwz  r11,4(r7)
    ///                                                      addi r3, r11,-4
    /// ```
    ///
    /// With five formals live the **lowest** free volatile is `r8`, so this
    /// separates *"the pool is walked highest-first"* from *"the pool is
    /// walked lowest-first"* on a population that is not a store run at all.
    /// [`allocate`] already answers that way for one producer at every legal
    /// floor, and this pins it so a future edit cannot quietly invert the walk
    /// and stay green — the walk direction has no other test that varies the
    /// floor.
    #[test]
    fn one_producer_takes_pool_top_at_every_floor() {
        for floor in 4..=POOL_TOP {
            let a = allocate(&run("0", ProducerKind::Constant), floor)
                .expect("one producer fits at every floor up to POOL_TOP");
            assert_eq!(
                a,
                vec![(48, POOL_TOP)],
                "a single value takes r11 at pool floor r{floor}, not the \
                 lowest free volatile — w-alloc3 H-wide, 16 of 16"
            );
            assert!(all_in(&run("0", ProducerKind::Constant), floor, POOL_TOP));
        }
        // …and above the top there is no pool, so it refuses rather than
        // reaching for r12 (#543).
        assert_eq!(allocate(&run("0", ProducerKind::Constant), POOL_TOP + 1), None);
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

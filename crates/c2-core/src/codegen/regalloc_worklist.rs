//! **REGALLOC WORKLIST** — c2's priority worklist comparator, `0x10b2b82d`,
//! expressed as executable code with every decision it makes exposed as a
//! named, settable parameter.
//!
//! Lane `w-regprio`, the L2 row of `docs/REGALLOC_BRIEF_2026-08-27.md`, funded
//! by owner decision 20 (`docs/DECISIONS_2026-08-22.md`). Board **#3700**.
//! Write-up: `docs/rungs/2026-08-27-w-regprio.md`.
//!
//! # What this is, and the four things it is NOT
//!
//! c2's register allocator is **priority colouring** (Chow–Hennessy) — no
//! interference graph, no simplify/select stack. `FUN_10b316b1` walks the
//! candidate hash and threads every candidate of the class through
//! `FUN_10b2b82d`, a **sorted insert into a doubly-linked list**; the driver
//! `FUN_10b31c9a` then pops the head. This module is that sorted insert and
//! nothing else. `docs/whitebox/ref/P_REGALLOC.md` §1 §4 is the read.
//!
//! It is **not** a register allocator, and that is an instruction rather than
//! an omission. Decision 20 §2: the comparator **consumes** its two keys and
//! computes neither. `cand+0x0c` is accumulated by `0x10b2d630` over the code
//! *the scheduler produced*, `cand+0x44` is stamped by `0x10b55fac` over the
//! *lowered* tuple list, and the port schedules nothing and has no tuple list.
//! So:
//!
//! * **No production caller.** Nothing in [`crate::PortC2::build`] reaches this
//!   module, no byte the judge grades can move, and no refusal consults it.
//!   The rung that lands it is additive by construction — see
//!   `docs/rungs/README.md` § "Lane kinds", the construct-rung corollary.
//! * **No `ported` numerator.** `#3505`. A site-level numerator for regalloc is
//!   *not yet defined*; inventing one to make a percentage move is that row's
//!   failure, four for four.
//! * **No claim to be `[O]`.** The comparator is `[R]` — read from the
//!   disassembly. What is `[O]` is the *order* on 20 obj cells, and
//!   [`tests`] measures that those 20 cells **do not confirm the comparator**
//!   (§ "Population power" below).
//! * **No cost model.** The selector's cost arithmetic is a different function
//!   (`0x10b2e7f8`) and a different lane (`w-regsel`).
//!
//! # The rule, as read
//!
//! ```text
//! insert new before n  iff  n->[0x0c] <  new->[0x0c]                          /* signed   */
//!                      or  (n->[0x0c] == new->[0x0c] && n->[0x44] <= new->[0x44]) /* unsigned */
//! ```
//!
//! Primary key `cand+0x0c` **descending, signed**; tie-break `cand+0x44`
//! **descending, unsigned**; and the tie tier compares **`<=`, not `<`**, so an
//! exact tie in *both* keys puts the **newly inserted** candidate FIRST.
//! [`WorklistComparator::C2`] is exactly that and is the default everywhere.
//!
//! # The decision points, exposed
//!
//! `docs/GOAL_DECISION_2026-08-21.md` § AMENDED, propagated into
//! `docs/rungs/README.md` § "Lane kinds" as the DECISION-SURFACE CLAUSE: a
//! general layer ships its arbitrary choices as **named, enumerable parameters
//! whose default reproduces c2**, because a named decision point serves the
//! permuter and the training pipeline at the same correctness cost as a baked
//! constant. Seven live here:
//!
//! | parameter | default (= the read) | why it is a decision point |
//! |---|---|---|
//! | [`KeySpec::field`] ×2 | `Priority` then `TieOrdinal` | which of the two `0x48`-record fields leads |
//! | [`KeySpec::dir`] ×2 | [`SortDir::Desc`] both | the list is highest-priority-first |
//! | [`KeySpec::signed`] ×2 | primary **signed**, tie **unsigned** | two different widths of the same 32 bits |
//! | [`WorklistComparator::tie_tier`] | [`TieTier::NewFirst`] (`<=`) | **the one the brief singles out** |
//! | [`Worklist::reentry`] | [`ReentryPolicy::ByPriority`] | a spilled candidate re-enters *sorted* |
//!
//! `tie_tier` is the sharpest of them. A permuter that flips one bit of a
//! close-but-wrong allocation wants exactly this bit, and baking it as a
//! constant would have spent the read for nothing.
//!
//! # Re-entry is the second falsifiable prediction, and it is about a port
//!
//! The driver re-inserts a spilled candidate with
//! `DAT_10c43b7c = FUN_10b2b82d(cand, DAT_10c43b7c)` — **the same comparator**.
//! So a spilled candidate re-enters **by priority**, not at the head:
//!
//! > **a port modelling the worklist as a stack or a queue is wrong in both
//! > directions** (`P_REGALLOC` §4 consequence 2).
//!
//! [`ReentryPolicy`] makes all three executable so the claim can be run rather
//! than quoted. [`tests::reentry_by_priority_differs_from_both_stack_and_queue`]
//! is the executed form of "wrong in both directions"; it is also the *only*
//! place the claim can be tested, because **no cell of the 20-cell population
//! spills** (§ "Population power").
//!
//! # Population power — read this before quoting a green test
//!
//! `#1236`: *"my test passes"* and *"my test can tell two rules apart"* are
//! different claims. The 20 obj cells at two profiles are this module's
//! population, and their measured power over the five planted mutants is
//! published by [`tests::population_power_over_the_twenty_cells`] **including
//! its zeros**. The headline, measured and not asserted:
//!
//! * With the two keys **unconstrained**, every one of the 20 orders is
//!   reproducible under the default **and under all five mutants** — the keys
//!   are not observable in an obj, so an unconstrained fit is decoration.
//! * The one key model the record does commit to for the `/O1` cells — *"the
//!   benefit keys are equal and the order is decided entirely by insertion
//!   sequence"* (`WB_DAGORDER2_FINDINGS.md` §5) — **is refuted by the A series
//!   on 6 of its 8 cells**, and read **R4** says why: `cand+0x44` is a dense
//!   tuple-visit ordinal, so an exact tie in *both* keys is the exception.
//!
//! PROV-BLOCK[R] DISCLOSURE `W-REGPRIO-1` — the comparator's decision rule, its
//! two key offsets, their directions and signednesses, and the `<=` tie tier,
//! all read at `0x10b2b82d`. Adopted as an *algorithm*, which the ledger's rule
//! names as adoption. The read is inherited from `P_REGALLOC` §4 /
//! `WB_DAGORDER2_FINDINGS.md` §5 and was **not re-taken** by this lane.

use core::cmp::Ordering;

/// Which field of c2's `0x48`-byte candidate record a key reads.
///
/// The offsets are the record's, and `P_REGALLOC` §4.1 is the layout. They are
/// carried here as *names*, never as a struct offset the port dereferences —
/// nothing in this crate lays out a c2 candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyField {
    /// **`cand+0x0c`** — the priority.
    ///
    /// Accumulated by `0x10b2d630`: `+= cand[0x18] * n_live` where live, `-=
    /// n_live` where not, scaled by a block weight. It is a **spill
    /// cost/benefit measure**, *not* a use count — which is exactly why the
    /// black-box readings that looked like a use count were refuted
    /// (`P_REGALLOC` §4 consequence 1, `[I]`). Signed, and the subtraction is
    /// why: it can go negative.
    Priority,
    /// **`cand+0x44`** — the tie ordinal.
    ///
    /// Read **R4** (`P_GLOBREGS` §7): a **tuple-visit ordinal**, sole
    /// origination site `0x10b55fac`, zeroed per function at `0x10b55eb7` and
    /// incremented once per real tuple at `0x10b55f77`; three further writes
    /// are verbatim inheritance onto a split child, and the destructor's
    /// `memset` makes its default a hard 0. Its **sole reader in the image is
    /// this comparator**, six reads.
    ///
    /// Compared **unsigned** by c2 — and that choice has no observable
    /// consequence on anything this project can compile, because a tuple count
    /// cannot reach 2³¹. See [`tests::the_tie_key_signedness_has_an_empty_observable_set`].
    TieOrdinal,
}

/// Ascending or descending. c2 sorts **both** keys descending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDir {
    /// Larger key ranks **earlier** — c2's choice on both keys.
    Desc,
    /// Smaller key ranks earlier.
    Asc,
}

/// One key of the two-key comparator: which field, which direction, and which
/// of the two readings of the same 32 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeySpec {
    /// Which candidate field this key reads.
    pub field: KeyField,
    /// Whether a larger value ranks earlier.
    pub dir: SortDir,
    /// **Whether the 32 bits are read as `i32` or `u32`.** c2 reads the primary
    /// key signed and the tie key unsigned — two different instructions over
    /// the same field width, and the asymmetry is in the disassembly, not an
    /// inference.
    pub signed: bool,
}

impl KeySpec {
    /// Rank two candidates on this key alone: `Less` means `a` ranks **earlier**.
    fn rank(&self, a: &Candidate, b: &Candidate) -> Ordering {
        let (x, y) = match self.field {
            KeyField::Priority => (a.priority, b.priority),
            KeyField::TieOrdinal => (a.tie as i32, b.tie as i32),
        };
        let raw = if self.signed {
            x.cmp(&y)
        } else {
            (x as u32).cmp(&(y as u32))
        };
        match self.dir {
            // Desc: the LARGER value ranks earlier, so reverse the numeric order.
            SortDir::Desc => raw.reverse(),
            SortDir::Asc => raw,
        }
    }
}

/// **The `<=`-versus-`<` decision, and the reason this module exists as a
/// parameterised thing rather than a function.**
///
/// The two tiers differ on exactly one input class: an exact tie in **both**
/// keys. c2 compares `<=`, so the newly inserted candidate goes first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TieTier {
    /// **`<=` — c2's.** On an exact tie in both keys the **new** candidate is
    /// inserted before the incumbent, so the finished list is the *reverse* of
    /// the accumulation order over any run of fully-tied candidates.
    NewFirst,
    /// **`<`.** On an exact tie in both keys the new candidate goes *after* the
    /// incumbent — the finished list preserves accumulation order over a tied
    /// run. **Not c2**, kept because it is the mutant that makes the default a
    /// measurement rather than a transcription.
    NewLast,
}

/// What the driver does with a candidate that has just been spilled.
///
/// `P_REGALLOC` §4 consequence 2, made executable: the driver's re-insert call
/// is the same `FUN_10b2b82d`, so the answer is [`ReentryPolicy::ByPriority`]
/// and the two obvious ports are both wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReentryPolicy {
    /// **c2's.** Re-insert through the comparator; the candidate lands wherever
    /// its keys put it.
    ByPriority,
    /// A **stack**: push at the head, so the spilled candidate is popped next.
    Head,
    /// A **queue**: push at the tail, so it is popped last.
    Tail,
}

/// A candidate, reduced to exactly the two fields the comparator reads.
///
/// This is deliberately **not** a model of c2's `0x48`-byte record. The port
/// does not build candidates and cannot compute either key (Decision 20 §2);
/// what it can do is consume a supplied pair, which is what the comparator
/// itself does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// An identity for the caller's own bookkeeping. c2's `cand+0x1c` is a
    /// per-function counter dense from 1 (read **R1**); nothing here depends on
    /// that and the comparator never reads it.
    pub id: u32,
    /// `cand+0x0c`, the priority. **Signed** — `0x10b2d630` subtracts.
    pub priority: i32,
    /// `cand+0x44`, the tie ordinal. Held as `u32` because that is how the
    /// comparator reads it.
    pub tie: u32,
}

impl Candidate {
    /// A candidate with both keys given.
    pub fn new(id: u32, priority: i32, tie: u32) -> Self {
        Candidate { id, priority, tie }
    }
}

/// The comparator at `0x10b2b82d`, as a settable object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorklistComparator {
    /// The leading key. c2: `cand+0x0c`, descending, **signed**.
    pub primary: KeySpec,
    /// The tie-break key. c2: `cand+0x44`, descending, **unsigned**.
    pub tie: KeySpec,
    /// What happens on an exact tie in both. c2: [`TieTier::NewFirst`].
    pub tie_tier: TieTier,
}

impl WorklistComparator {
    /// **c2's comparator, and the default everywhere in this module.**
    ///
    /// PROV[R] DISCLOSURE `W-REGPRIO-1` — read at `0x10b2b82d`; the six
    /// `cand+0x44` reads are `0x10b2b84d/850/860/863/87c/87f`.
    pub const C2: WorklistComparator = WorklistComparator {
        primary: KeySpec { field: KeyField::Priority, dir: SortDir::Desc, signed: true },
        tie: KeySpec { field: KeyField::TieOrdinal, dir: SortDir::Desc, signed: false },
        tie_tier: TieTier::NewFirst,
    };

    /// Rank `a` against `b` on both keys. `Equal` means an **exact tie in both**
    /// — the only input on which [`TieTier`] can matter.
    pub fn rank(&self, a: &Candidate, b: &Candidate) -> Ordering {
        match self.primary.rank(a, b) {
            Ordering::Equal => self.tie.rank(a, b),
            other => other,
        }
    }

    /// The predicate c2's loop evaluates at each list node: *does `new` go
    /// before the incumbent `n`?*
    ///
    /// This is the whole of `0x10b2b82d`'s decision, and reading it beside the
    /// disassembly is the point of writing it out:
    ///
    /// ```text
    /// n->[0x0c] <  new->[0x0c]                            -> rank(new, n) == Less
    /// n->[0x0c] == new->[0x0c] && n->[0x44] <= new->[0x44] -> Less, or Equal under NewFirst
    /// ```
    pub fn insert_before(&self, new: &Candidate, n: &Candidate) -> bool {
        match self.rank(new, n) {
            Ordering::Less => true,
            Ordering::Equal => self.tie_tier == TieTier::NewFirst,
            Ordering::Greater => false,
        }
    }
}

impl Default for WorklistComparator {
    fn default() -> Self {
        WorklistComparator::C2
    }
}

/// c2's priority worklist — `DAT_10c43b7c`, built by `0x10b316b1` and consumed
/// head-first by `0x10b31c9a`.
///
/// c2 threads it as a doubly-linked list through `cand+0x14` (next) and
/// `cand+0x18` (prev); this is a `Vec`, which is the same *order* and is the
/// point (`CLAUDE.md`: I/O-behavioral, never binary-faithful — the port may use
/// any representation whose observable behaviour agrees).
///
/// **`cand+0x18` is phase-overloaded and this module does not model it.**
/// `P_REGALLOC` §4.1 correction 1: it is an accumulation weight during
/// `0x10b2d630` and the list's `prev` pointer during the colouring loop, and *a
/// port reading it as one thing is wrong in one of the two phases*. Not
/// modelling it is how that trap is avoided rather than stepped in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worklist {
    /// The comparator. Default [`WorklistComparator::C2`].
    pub cmp: WorklistComparator,
    /// What a spill re-entry does. Default [`ReentryPolicy::ByPriority`].
    pub reentry: ReentryPolicy,
    list: Vec<Candidate>,
}

impl Worklist {
    /// An empty worklist with c2's comparator and c2's re-entry policy.
    pub fn new() -> Self {
        Worklist {
            cmp: WorklistComparator::C2,
            reentry: ReentryPolicy::ByPriority,
            list: Vec::new(),
        }
    }

    /// An empty worklist with a chosen comparator — the permuter's entry point.
    pub fn with_comparator(cmp: WorklistComparator) -> Self {
        Worklist { cmp, reentry: ReentryPolicy::ByPriority, list: Vec::new() }
    }

    /// `FUN_10b2b82d` itself: walk from the head and splice `new` in before the
    /// first node the predicate accepts; append if none does.
    pub fn insert(&mut self, new: Candidate) {
        let at = self
            .list
            .iter()
            .position(|n| self.cmp.insert_before(&new, n))
            .unwrap_or(self.list.len());
        self.list.insert(at, new);
    }

    /// `0x10b31e97` — the head-first pop the colouring loop performs.
    pub fn pop(&mut self) -> Option<Candidate> {
        if self.list.is_empty() { None } else { Some(self.list.remove(0)) }
    }

    /// Put a **spilled** candidate back, honouring [`Self::reentry`].
    ///
    /// c2 calls the comparator, so the default lands it by priority. The other
    /// two arms exist to make "a port modelling the worklist as a stack or a
    /// queue is wrong in both directions" a thing that can be **run**.
    pub fn reinsert_after_spill(&mut self, cand: Candidate) {
        match self.reentry {
            ReentryPolicy::ByPriority => self.insert(cand),
            ReentryPolicy::Head => self.list.insert(0, cand),
            ReentryPolicy::Tail => self.list.push(cand),
        }
    }

    /// The list in colouring order, head first.
    pub fn order(&self) -> &[Candidate] {
        &self.list
    }

    /// The ids in colouring order — the shape every test compares.
    pub fn ids(&self) -> Vec<u32> {
        self.list.iter().map(|c| c.id).collect()
    }

    /// Is the worklist empty? (`while (DAT_10c43b7c)`.)
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Build a finished worklist by accumulating `cands` in the order given —
    /// `0x10b316b1`'s loop, with the accumulation order supplied by the caller.
    pub fn build(cmp: WorklistComparator, cands: &[Candidate]) -> Worklist {
        let mut w = Worklist::with_comparator(cmp);
        for c in cands {
            w.insert(*c);
        }
        w
    }
}

impl Default for Worklist {
    fn default() -> Self {
        Worklist::new()
    }
}

#[cfg(test)]
mod tests;

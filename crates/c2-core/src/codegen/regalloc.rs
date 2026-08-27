//! **`codegen::regalloc`** — c2's register SELECTOR, `0x10b2e7f8`, as executable
//! code, with the allocation ORDER as a named, settable parameter.
//!
//! Wave 16 lane `w-regsel`, `docs/DECISIONS_2026-08-22.md` § Decision 20, from
//! `docs/REGALLOC_BRIEF_2026-08-27.md` §3 L1. Reference page:
//! [`docs/whitebox/ref/P_REGALLOC.md`] §2.1, §3. Provenance ledger row:
//! **`DISCLOSURE W-REGSEL-1`**.
//!
//! # THIS IS NOT A REGISTER ALLOCATOR, AND IT MUST NOT GROW INTO ONE
//!
//! Decision 20 §2, in its own words: *a candidate is a `(symbol, live-range
//! version)` pair whose versions need the backward walk over the **lowered**
//! tuple list, and F5 is not separable from F0 because `cand+0x0c` accumulates
//! over the code the scheduler produced* — while the port **schedules
//! nothing**. What is separable, and all that is here, is the part of c2's
//! allocator that *consumes* a priority rather than computing one:
//!
//! ```text
//!   inputs GIVEN to the selector      the candidate set, each candidate's
//!                                     ALLOWED register set, the cost array
//!   inputs the selector COMPUTES      none
//! ```
//!
//! [`select`] is therefore a pure function of three arguments the scheduler
//! does not supply. Everything upstream of it — the worklist, the priority
//! accumulation at `0x10b2d630`, the interference recomputation at
//! `0x10b30517`, the spiller — is **out of scope and stays out**.
//!
//! # THE COST ARITHMETIC IS `[R]` AND STAYS `[R]` — READ THIS BEFORE QUOTING
//!   THIS MODULE
//!
//! `P_REGALLOC.md` §3's correction box records that on all **10** cells of
//! `wb-live`'s grid and all **15** of `wb-regalloc`'s, **every cost array is
//! uniformly zero over its allowed set**, so the answer is decided entirely by
//! list order. The consequence for this module is exact and is not a hedge:
//!
//! * [`Costs`] and the `cost[reg]` comparison in [`select`] are a
//!   transcription of instructions that were **read**, never a confirmed
//!   model. Nothing in this repo has ever observed c2 break a tie on a cost.
//! * The **ORDER** is the `[O]` part — obj-confirmed on cells G1–G4 and P1
//!   (`WB_REGALLOC_FINDINGS.md` §7.1, 6/6, with three rival rules refuted by
//!   cell count), and it is what this module makes executable and testable.
//! * **The port supplies exactly one cost array and it is [`Costs::ZERO`]**
//!   (pinned by `the_only_cost_array_the_port_constructs_is_zero_and_the_call_
//!   sites_are_enumerated`). A lane
//!   that reports "the cost model is confirmed" on the strength of this file
//!   has confirmed nothing.
//!
//! # The decision surface (decision 15 / `rungs/README.md`'s decision-surface
//!   clause)
//!
//! The allocation order ships as an **enumerable parameter** — [`ORDERS`] —
//! whose **default reproduces c2 byte-exactly**. Every non-default entry is a
//! legal *instrument* state and licenses no emit; the only production call
//! site passes [`GPR_DEFAULT`], pinned by a test. This is what turns a
//! close-but-wrong register into something a permuter can search.
//!
//! # Two numberings, and they are not the same
//!
//! c2's ordered arrays hold **c2's own register indices** — the index into the
//! name table at `0x10b181c0`, where `0` is *noreg*, `1` is `r0`, `2` is `sp`,
//! `3` is `toc`, `4…13` are `r3…r12`, `14` is `r13` and `15…32` are `r14…r31`.
//! Over the whole GPR file that collapses to `ppc = c2_index - 1`, which is
//! what [`gpr_from_c2_index`] does. This module's public surface is in **PPC
//! architectural numbers**, because that is what the port's encoders take; the
//! **raw c2 index arrays are kept** ([`C2_GPR_DEFAULT`] &c.) so the
//! transcription is re-derivable rather than a second magic list.

use std::fmt;

/// A register, in **PPC architectural numbering within its class** — `r0…r31`
/// for the GPRs, `fp0…fp31` for the FPRs. Not c2's index; see the module
/// header.
pub type Reg = u8;

/// The selector's cost, one signed int per register.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1` — `0x10c435e8` is `0x594` bytes = **357
/// ints**, one per c2 register number `0…356`; `0x10b2e7f8` `memset`s it, adds
/// interference and constraint penalties, then subtracts copy preferences, so
/// **negative means preferred**. The *width* is read; no cost VALUE in this
/// repo has ever been observed non-zero (module header).
pub type Cost = i32;

// ---------------------------------------------------------------------------
// The ordered lists — c2's `0x10c385c4[class]`
// ---------------------------------------------------------------------------

/// c2 index → PPC GPR number. `1 → r0`, `2 → sp` (`r1`), `3 → toc` (`r2`),
/// `4…13 → r3…r12`, `14 → r13`, `15…32 → r14…r31`.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1` — the name table at `0x10b181c0`, whose
/// index *is* c2's register number (`P_REGALLOC.md` §2.1).
const fn gpr_from_c2_index(i: u8) -> Reg {
    // `0` is *noreg* and appears in no ordered array (they are zero-terminated,
    // which is how c2 finds the end); a `0` here is a transcription error.
    assert!(i >= 1 && i <= 32);
    i - 1
}

const fn map_gpr<const N: usize>(src: [u8; N]) -> [Reg; N] {
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N {
        out[i] = gpr_from_c2_index(src[i]);
        i += 1;
    }
    out
}

/// **The default GPR allocation order, in c2's own index numbering**, exactly
/// as the zero-terminated array at `0x10c37de0` holds it.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1` — `0x10c37de0`, transcribed in
/// `docs/whitebox/WB_REGALLOC_FINDINGS.md` §3.1. The *decoded* order is
/// additionally **`[O]`** on cells G1–G4 and P1 with no disassembly.
pub const C2_GPR_DEFAULT: [u8; 27] = [
    12, 11, 10, 9, 8, 7, 6, 5, 4, //
    32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15,
];

/// The `-QGPRReserve` variant at `0x10c37e50` — the same head, and the tail
/// stops at c2 index `18` (`r17`), reserving `r14`,`r15`,`r16`.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1`. **No obj cell exercises it** — the
/// workload does not pass `-QGPRReserve`.
pub const C2_GPR_QGPRRESERVE: [u8; 24] = [
    12, 11, 10, 9, 8, 7, 6, 5, 4, //
    32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18,
];

/// The POGO-instrumented variant at `0x10c37eb8` — tail stops at c2 index `17`
/// (`r16`), reserving `r14`,`r15`.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1`. **No obj cell exercises it.**
pub const C2_GPR_POGO: [u8; 25] = [
    12, 11, 10, 9, 8, 7, 6, 5, 4, //
    32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
];

/// PROV-BLOCK[N] `DISCLOSURE W-REGSEL-1` — not load-bearing on their own:
/// each is [`map_gpr`] applied to the marked raw array one screen up, so the
/// value is derived from another marked constant and carries no independent
/// claim. Decoding here rather than transcribing a second list is what keeps
/// them that way.
const GPR_DEFAULT_REGS: [Reg; 27] = map_gpr(C2_GPR_DEFAULT);
const GPR_QGPRRESERVE_REGS: [Reg; 24] = map_gpr(C2_GPR_QGPRRESERVE);
const GPR_POGO_REGS: [Reg; 25] = map_gpr(C2_GPR_POGO);

/// The FPR order at `0x10c37f20` — `fp0`, then `fp13…fp1`, then `fp31…fp14`.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1`, and this one is **`[R]` with no obj cell
/// in existence at all** — `P_REGALLOC.md` §7: *"read and never obj-checked;
/// no cell in any grid uses floating point"*. Closing that cell is lane
/// `w-regcells`' deliverable (decision 20 §1), not this one's. Held here so
/// that lane has an executable order to grade against; **no production path
/// reaches it**, pinned by the call-site enumeration in
/// `the_only_cost_array_the_port_constructs_is_zero_and_the_call_sites_are_enumerated`.
///
/// Kept in **fp numbering directly**, not c2 indices: the FPR block starts at
/// c2 index `34` (`fp0`), so the mapping is `fp = c2_index - 34` and re-deriving
/// it here would state the same transcription twice.
pub const FPR_DEFAULT_REGS: [Reg; 32] = [
    0, //
    13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, //
    31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14,
];

/// A per-class ordered register list — c2's `0x10c385c4[class]`, the array the
/// selector walks from index 0 forward.
///
/// **This is the lane's settable parameter.** It is a value, not a constant:
/// [`select`] takes it by reference and has no default of its own.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RegOrder {
    /// A stable name, so a permuter or a training run can say which order it
    /// used without printing 27 numbers.
    pub name: &'static str,
    /// The registers, **in the order c2 walks them**. Earliest wins a tie.
    pub regs: &'static [Reg],
}

impl fmt::Debug for RegOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegOrder({}, {} regs)", self.name, self.regs.len())
    }
}

impl RegOrder {
    /// The rank of `r` in this order, or `None` when `r` is not allocatable in
    /// this class at all (`r12`, `r13`, `r0`, `sp`, `toc` for the GPRs).
    pub fn rank(&self, r: Reg) -> Option<usize> {
        self.regs.iter().position(|&x| x == r)
    }

    /// Whether `r` is allocatable under this order.
    pub fn allocatable(&self, r: Reg) -> bool {
        self.rank(r).is_some()
    }
}

/// **THE DEFAULT, and the only order any production path uses.** `r11, r10, …,
/// r3` then `r31, r30, …, r14`.
///
/// PROV[O] `DISCLOSURE W-REGSEL-1` — the ORDER is obj-confirmed on cells
/// G1–G4 and P1 with no disassembly (`WB_REGALLOC_FINDINGS.md` §7.1, 6/6),
/// with three rival rules refuted by cell count: first-free ascending from
/// `r3` (6 cells), first-free descending from `r12` (9 cells), and
/// no-preference (4 cells). **This is the mark that separates this module
/// from its cost arithmetic**, which is `[R]` and stays `[R]`.
pub const GPR_DEFAULT: RegOrder = RegOrder { name: "gpr-default", regs: &GPR_DEFAULT_REGS };

/// `-QGPRReserve`. Instrument only.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1` — `0x10c37e50`. **No obj cell exercises
/// it**, so it is `[R]` and not `[O]`: the workload does not pass the flag.
pub const GPR_QGPRRESERVE: RegOrder =
    RegOrder { name: "gpr-qgprreserve", regs: &GPR_QGPRRESERVE_REGS };

/// POGO-instrumented. Instrument only.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1` — `0x10c37eb8`. **No obj cell exercises it.**
pub const GPR_POGO: RegOrder = RegOrder { name: "gpr-pogo", regs: &GPR_POGO_REGS };

/// The FPR order. Instrument only.
///
/// PROV[R] `DISCLOSURE W-REGSEL-1` — `0x10c37f20`, and this one has **no obj
/// cell in existence at all**; closing it is lane `w-regcells`' deliverable.
/// See [`FPR_DEFAULT_REGS`].
pub const FPR_DEFAULT: RegOrder = RegOrder { name: "fpr-default", regs: &FPR_DEFAULT_REGS };

/// PROV[N] `DISCLOSURE W-REGSEL-1` — not load-bearing: a list of the four
/// marked orders above, carrying no value of its own.
///
/// **The enumerable parameter space** — decision 15's *"named, enumerable
/// parameters whose DEFAULT reproduces c2 byte-exactly"*. Index 0 is the
/// default.
pub const ORDERS: &[RegOrder] = &[GPR_DEFAULT, GPR_QGPRRESERVE, GPR_POGO, FPR_DEFAULT];

// ---------------------------------------------------------------------------
// The allowed set
// ---------------------------------------------------------------------------

/// A candidate's ALLOWED register set — c2's `cand+0x20`.
///
/// The constructor at `0x10b54d32` starts every candidate with **the whole
/// class allowed** and *"allocation only ever removes"* (`P_REGALLOC.md` §2.1);
/// an empty set means spill. A bitset over register numbers `0…63` covers both
/// register files in this port's numbering.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct RegSet(u64);

impl RegSet {
    /// The empty set — c2's "spill".
    ///
    /// PROV[N] not load-bearing — the identity of a bitset this crate defines.
    pub const EMPTY: RegSet = RegSet(0);

    /// `lo..=hi`, empty when `hi < lo`.
    pub const fn range_inclusive(lo: Reg, hi: Reg) -> RegSet {
        if hi < lo || lo > 63 {
            return RegSet(0);
        }
        let hi = if hi > 63 { 63 } else { hi };
        // `1<<64` is UB-adjacent; build the mask by subtraction on the top bit.
        let width = (hi - lo + 1) as u32;
        let mask = if width >= 64 { u64::MAX } else { (1u64 << width) - 1 };
        RegSet(mask << lo)
    }

    pub const fn contains(self, r: Reg) -> bool {
        r < 64 && (self.0 >> r) & 1 == 1
    }

    #[must_use]
    pub const fn insert(self, r: Reg) -> RegSet {
        if r >= 64 { self } else { RegSet(self.0 | (1u64 << r)) }
    }

    /// The removal c2 performs when a neighbour takes a register.
    #[must_use]
    pub const fn remove(self, r: Reg) -> RegSet {
        if r >= 64 { self } else { RegSet(self.0 & !(1u64 << r)) }
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }
}

// ---------------------------------------------------------------------------
// The cost array
// ---------------------------------------------------------------------------

/// The selector's cost array — c2's `DAT_10c435e8`.
///
/// **Read the module header before using anything but [`Costs::ZERO`].** The
/// arithmetic here is `[R]`; every array this project has ever observed is
/// uniformly zero over its allowed set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Costs {
    c: [Cost; 64],
}

impl Costs {
    /// What `memset(&DAT_10c435e8, 0, 0x594)` leaves, and the only array any
    /// production path in this port constructs.
    ///
    /// PROV[R] `DISCLOSURE W-REGSEL-1` — the `memset` at the head of
    /// `0x10b2e7f8`. It is `[R]` and not `[O]` on purpose: what 25 measured
    /// cells establish is that every array they saw was *uniform over its
    /// allowed set*, which is consistent with zero and does not pin it, since
    /// a uniform non-zero array selects identically.
    pub const ZERO: Costs = Costs { c: [0; 64] };

    pub const fn get(&self, r: Reg) -> Cost {
        if r >= 64 { 0 } else { self.c[r as usize] }
    }

    /// `cost[reg] += w` — c2's interference/constraint penalty (`[R]`).
    pub fn penalize(&mut self, r: Reg, w: Cost) {
        if r < 64 {
            self.c[r as usize] = self.c[r as usize].saturating_add(w);
        }
    }

    /// `cost[reg] -= w` — c2's copy preference; **negative cost = preferred**
    /// (`[R]`).
    pub fn prefer(&mut self, r: Reg, w: Cost) {
        if r < 64 {
            self.c[r as usize] = self.c[r as usize].saturating_sub(w);
        }
    }

    /// Whether this array is uniform over `allowed` — the regime in which
    /// every cell this project has measured sits, and the regime in which
    /// [`select_sequence`]'s single-array simplification is exact.
    pub fn is_uniform_over(&self, allowed: RegSet) -> bool {
        let mut seen: Option<Cost> = None;
        for r in 0..64u8 {
            if allowed.contains(r) {
                match seen {
                    None => seen = Some(self.get(r)),
                    Some(v) if v == self.get(r) => {}
                    Some(_) => return false,
                }
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// The selector
// ---------------------------------------------------------------------------

/// **THE SELECTOR — `0x10b2e7f8`.**
///
/// PROV[R] `DISCLOSURE W-REGSEL-1` for the walk, PROV[O] for the ORDER the
/// walk is handed (`GPR_DEFAULT`, cells G1–G4/P1). `P_REGALLOC.md` §3 step 5:
///
/// ```text
///   best = none;
///   for (i = 0; list[i] != 0; i++)
///       if (candidate_set has list[i] && (best == none || cost[list[i]] < best_cost))
///           { best = list[i]; best_cost = cost[best]; }
/// ```
///
/// **The `<` is strict, so ties go to the EARLIEST register in list order.**
/// That single character is the whole `[O]` claim; control `C3` plants `<=`
/// and watches the tie test go red.
///
/// Returns `None` when nothing in the order survives the allowed set — c2's
/// spill.
pub fn select(order: &RegOrder, allowed: RegSet, cost: &Costs) -> Option<Reg> {
    let mut best: Option<(Reg, Cost)> = None;
    for &r in order.regs {
        if !allowed.contains(r) {
            continue;
        }
        let c = cost.get(r);
        match best {
            None => best = Some((r, c)),
            // STRICT `<`. See the doc above; this is control C3's site.
            Some((_, bc)) if c < bc => best = Some((r, c)),
            Some(_) => {}
        }
    }
    best.map(|(r, _)| r)
}

/// `n` mutually-interfering candidates over one shared allowed set.
///
/// This is c2's driver loop (`0x10b31c9a`) **collapsed to the one shape the
/// port can reach**: every candidate interferes with every other, so each
/// colouring removes the chosen register from what is left
/// (`0x10b54d32`: *allocation only ever removes*), and the priority order
/// among them is decided by the CALLER, not here — this function colours the
/// candidates in the order it is given them.
///
/// **It is not the allocator.** It computes no priority, builds no worklist and
/// models no live range; those need the scheduler (decision 20 §2).
///
/// `cost` is held fixed across the `n` colourings, which is exact exactly when
/// the array is uniform over `allowed` — the regime of every measured cell —
/// and an approximation otherwise. [`Costs::is_uniform_over`] tests it; the
/// port's only call passes [`Costs::ZERO`].
///
/// Returns `None` if fewer than `n` registers of the order survive — which is
/// where every "the pool is too small" refusal now comes from.
pub fn select_sequence(order: &RegOrder, allowed: RegSet, cost: &Costs, n: usize) -> Option<Vec<Reg>> {
    let mut left = allowed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let r = select(order, left, cost)?;
        left = left.remove(r);
        out.push(r);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- the transcription, re-derived -----------------------------------

    /// The decoded default order is `WB_REGALLOC_FINDINGS.md` §3.1's sentence,
    /// spelled out. This is the `[O]` claim (cells G1–G4, P1).
    #[test]
    fn the_default_gpr_order_decodes_to_r11_down_then_r31_down() {
        assert_eq!(
            GPR_DEFAULT.regs,
            &[
                11, 10, 9, 8, 7, 6, 5, 4, 3, //
                31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14
            ]
        );
        assert_eq!(GPR_DEFAULT.regs.len(), 27);
    }

    /// **BOARD #543 IS EXPLAINED, NOT MERELY RECORDED.** `alloc.rs` carries
    /// *"`r12` is never used (board #543 — recorded, not explained)"*. The
    /// read explains it: c2 index `13` is in **none** of the three GPR arrays,
    /// so `r12` is excluded by the ORDER and not by any cap. The same is true
    /// of `r13`, `r0`, `sp` (`r1`) and `toc` (`r2`).
    ///
    /// Control **C2** plants `r12` at the head of the order and watches this
    /// test — and the equivalence test — go red.
    #[test]
    fn r12_r13_r0_sp_toc_are_in_no_gpr_order_which_is_board_543() {
        for order in [GPR_DEFAULT, GPR_QGPRRESERVE, GPR_POGO] {
            for r in [0u8, 1, 2, 12, 13] {
                assert!(
                    !order.allocatable(r),
                    "{}: r{r} must not be allocatable — #543",
                    order.name
                );
            }
            for c2ix in [1u8, 2, 3, 13, 14] {
                assert!(
                    !order.regs.contains(&gpr_from_c2_index(c2ix)),
                    "{}: c2 index {c2ix} must be absent",
                    order.name
                );
            }
        }
    }

    #[test]
    fn the_variant_orders_reserve_exactly_what_the_read_says() {
        // -QGPRReserve drops r14, r15, r16 and nothing else.
        let d: Vec<Reg> = GPR_DEFAULT.regs.to_vec();
        let q: Vec<Reg> = GPR_QGPRRESERVE.regs.to_vec();
        let dropped: Vec<Reg> = d.iter().copied().filter(|r| !q.contains(r)).collect();
        assert_eq!(dropped, vec![16, 15, 14], "-QGPRReserve reserves r14..r16");
        // POGO drops r14, r15.
        let p: Vec<Reg> = GPR_POGO.regs.to_vec();
        let dropped: Vec<Reg> = d.iter().copied().filter(|r| !p.contains(r)).collect();
        assert_eq!(dropped, vec![15, 14], "POGO reserves r14..r15");
        // Every variant is a subsequence of the default: the head is shared and
        // only the tail is truncated.
        for v in [&q, &p] {
            let mut it = d.iter();
            for r in v.iter() {
                assert!(it.any(|x| x == r), "variant order is not a subsequence");
            }
        }
    }

    #[test]
    fn the_fpr_order_is_fp0_then_volatiles_down_then_callee_saved_down() {
        assert_eq!(FPR_DEFAULT.regs[0], 0);
        assert_eq!(&FPR_DEFAULT.regs[1..14], &[13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(FPR_DEFAULT.regs[14], 31);
        assert_eq!(FPR_DEFAULT.regs[31], 14);
        assert_eq!(FPR_DEFAULT.regs.len(), 32);
    }

    #[test]
    fn every_order_is_duplicate_free_and_named() {
        for o in ORDERS {
            let mut seen = RegSet::EMPTY;
            for &r in o.regs {
                assert!(!seen.contains(r), "{}: r{r} twice", o.name);
                seen = seen.insert(r);
            }
            assert!(!o.name.is_empty());
        }
        assert_eq!(ORDERS[0], GPR_DEFAULT, "index 0 is the default");
    }

    // ---- the walk ---------------------------------------------------------

    /// **THE `[O]` RULE.** With a uniformly-zero cost array the selector
    /// returns the EARLIEST entry of the order that survives the allowed set —
    /// which is the whole of what the 25 measured cells decide.
    ///
    /// Control **C3** flips `<` to `<=` and watches this go red.
    #[test]
    fn a_zero_cost_array_selects_the_earliest_surviving_entry() {
        let all = RegSet::range_inclusive(0, 31);
        assert_eq!(select(&GPR_DEFAULT, all, &Costs::ZERO), Some(11));
        assert_eq!(select(&GPR_DEFAULT, all.remove(11), &Costs::ZERO), Some(10));
        // Take the whole volatile head away and the tail's head appears.
        let mut s = all;
        for r in 3..=12 {
            s = s.remove(r);
        }
        assert_eq!(
            select(&GPR_DEFAULT, s, &Costs::ZERO),
            Some(31),
            "with the volatiles gone the callee-saved tail starts at r31 — \
             `WB_LIVE_PREREG.md` P5.1's prediction, and it needs no preference \
             for callee-saved registers"
        );
    }

    #[test]
    fn an_empty_allowed_set_is_a_spill() {
        assert_eq!(select(&GPR_DEFAULT, RegSet::EMPTY, &Costs::ZERO), None);
        // …and so is a set holding only registers the order excludes.
        let only_r12 = RegSet::EMPTY.insert(12).insert(13).insert(0);
        assert_eq!(select(&GPR_DEFAULT, only_r12, &Costs::ZERO), None);
    }

    /// The cost comparison, transcribed. **This tests the WALK, not the cost
    /// model** — see the module header. Nothing here is evidence that c2 ever
    /// produces one of these arrays.
    #[test]
    fn a_negative_cost_beats_the_order_and_is_the_preference_term() {
        let all = RegSet::range_inclusive(0, 31);
        let mut c = Costs::ZERO;
        c.prefer(5, 1);
        assert_eq!(select(&GPR_DEFAULT, all, &c), Some(5), "negative cost = preferred");
        let mut c = Costs::ZERO;
        c.penalize(11, 1);
        assert_eq!(select(&GPR_DEFAULT, all, &c), Some(10), "a penalty moves past r11");
        // A tie between two negatives still goes to the earlier one.
        let mut c = Costs::ZERO;
        c.prefer(5, 7);
        c.prefer(9, 7);
        assert_eq!(select(&GPR_DEFAULT, all, &c), Some(9), "r9 precedes r5 in the order");
    }

    #[test]
    fn is_uniform_over_sees_the_regime_the_measured_cells_sit_in() {
        let all = RegSet::range_inclusive(3, 11);
        assert!(Costs::ZERO.is_uniform_over(all));
        let mut c = Costs::ZERO;
        c.penalize(4, 3);
        assert!(!c.is_uniform_over(all));
        assert!(c.is_uniform_over(all.remove(4)));
        assert!(c.is_uniform_over(RegSet::EMPTY));
    }

    // ---- the sequence -----------------------------------------------------

    /// Control **C4** stops the removal and watches this go red.
    #[test]
    fn a_sequence_hands_out_distinct_registers_in_order() {
        let pool = RegSet::range_inclusive(4, 12);
        let seq = select_sequence(&GPR_DEFAULT, pool, &Costs::ZERO, 3).unwrap();
        assert_eq!(seq, vec![11, 10, 9]);
        let mut seen = RegSet::EMPTY;
        for r in &seq {
            assert!(!seen.contains(*r));
            seen = seen.insert(*r);
        }
    }

    /// Control **C5** drops the allowed-set check and watches this go red.
    #[test]
    fn a_sequence_longer_than_the_surviving_order_refuses() {
        // {4..=12} ∩ order = {4..=11}: nine registers, so nine is the ceiling
        // and r12 does NOT count even though it is in the set.
        let pool = RegSet::range_inclusive(4, 12);
        assert_eq!(pool.len(), 9, "the SET holds nine including r12");
        assert!(select_sequence(&GPR_DEFAULT, pool, &Costs::ZERO, 8).is_some());
        assert!(
            select_sequence(&GPR_DEFAULT, pool, &Costs::ZERO, 9).is_none(),
            "eight allocatable registers in {{4..=12}} (r12 is excluded by the ORDER), \
             so nine candidates spill"
        );
        assert_eq!(select_sequence(&GPR_DEFAULT, pool, &Costs::ZERO, 0), Some(vec![]));
    }

    /// **THE SECOND FAIL AXIS of `work/w-regsel/PREREG.md` §4.** The read order
    /// continues past `r3` into `r31…r14`; that tail must stay unreachable from
    /// any allowed set the port builds, or the port would silently start
    /// emitting callee-saved registers with no fixture to catch it.
    #[test]
    fn the_callee_saved_tail_is_unreachable_from_a_volatile_only_allowed_set() {
        for lo in 0..=13u8 {
            for n in 0..=9usize {
                let pool = RegSet::range_inclusive(lo, 12);
                if let Some(seq) = select_sequence(&GPR_DEFAULT, pool, &Costs::ZERO, n) {
                    for r in seq {
                        assert!(
                            (3..=11).contains(&r),
                            "a volatile-capped allowed set selected r{r}"
                        );
                    }
                }
            }
        }
    }

    // ---- the parameter is a parameter -------------------------------------

    /// Decision 15: the default reproduces c2, every other entry is an
    /// instrument state. A non-default order must actually CHANGE the answer,
    /// or it is not a decision surface.
    #[test]
    fn a_non_default_order_changes_the_answer_which_is_what_makes_it_a_surface() {
        let callee_only = {
            let mut s = RegSet::range_inclusive(14, 31);
            s = s.remove(16);
            s
        };
        assert_eq!(select(&GPR_DEFAULT, callee_only, &Costs::ZERO), Some(31));
        // -QGPRReserve stops at r17, so with only r14..r15 allowed it spills
        // where the default would not.
        let low = RegSet::range_inclusive(14, 15);
        assert_eq!(select(&GPR_DEFAULT, low, &Costs::ZERO), Some(15));
        assert_eq!(select(&GPR_QGPRRESERVE, low, &Costs::ZERO), None);
        assert_eq!(select(&GPR_POGO, low, &Costs::ZERO), None);
        let reversed_regs: Vec<Reg> = GPR_DEFAULT.regs.iter().rev().copied().collect();
        assert_ne!(
            reversed_regs.first(),
            GPR_DEFAULT.regs.first(),
            "the order is orientation-bearing"
        );
    }

    // ---- the sets ---------------------------------------------------------

    // ---- the seam control ---------------------------------------------------

    /// Every `.rs` under `crates/c2-core/src`, EXCEPT this file. The scanner
    /// cannot scan itself: a control that greps for a token it must contain to
    /// do the grepping cannot pass, and pretending otherwise is how a control
    /// becomes decoration.
    fn crate_sources_excluding_this_module() -> Vec<(String, String)> {
        fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
            let Ok(rd) = std::fs::read_dir(dir) else { return };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().and_then(|x| x.to_str()) == Some("rs")
                    && p.file_name().and_then(|x| x.to_str()) != Some("regalloc.rs")
                {
                    if let Ok(s) = std::fs::read_to_string(&p) {
                        out.push((p.display().to_string(), s));
                    }
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut out = Vec::new();
        walk(&root, &mut out);
        assert!(out.len() > 40, "the walk found only {} files", out.len());
        out
    }

    /// **P5 of `work/w-regsel/PREREG.md`, and it is the pin that keeps
    /// `P_REGALLOC.md` §3's correction box TRUE OF `crates/`.**
    ///
    /// Every cost array the port constructs is [`Costs::ZERO`]. Nothing in
    /// this repo has ever observed c2 break a tie on a cost, so nothing in the
    /// port may ship one — a non-zero array would be a **fitted** value
    /// wearing a read's provenance, which is the exact failure
    /// `WHITEBOX_LEVERAGE`'s doctrine exists to prevent.
    ///
    /// It also enumerates the call sites, so a second consumer of the selector
    /// cannot appear without this test naming it.
    #[test]
    fn the_only_cost_array_the_port_constructs_is_zero_and_the_call_sites_are_enumerated() {
        let mut call_sites: Vec<String> = Vec::new();
        let mut non_default: Vec<String> = Vec::new();
        for (path, src) in crate_sources_excluding_this_module() {
            for (n, line) in src.lines().enumerate() {
                let t = line.trim();
                if t.starts_with("//") || t.starts_with("///") {
                    continue;
                }
                if t.contains("regalloc::select") {
                    call_sites.push(format!("{path}:{}", n + 1));
                }
                for instrument in ["GPR_QGPRRESERVE", "GPR_POGO", "FPR_DEFAULT"] {
                    if t.contains(instrument) {
                        non_default.push(format!("{path}:{}: {t}", n + 1));
                    }
                }
                if let Some(i) = t.find("Costs") {
                    assert!(
                        t[i..].starts_with("Costs::ZERO"),
                        "{path}:{}: every cost array outside this module must \
                         be the ZERO constant — found `{t}`",
                        n + 1
                    );
                }
            }
        }
        // Two: `alloc::allocate`'s production call, and `alloc`'s own
        // supply-divergence probe. A third means a new consumer arrived and
        // this test is the thing that has to be re-read.
        assert_eq!(call_sites.len(), 2, "call sites moved: {call_sites:?}");
        assert!(
            call_sites.iter().all(|s| s.contains("alloc.rs")),
            "the only consumer is codegen::alloc: {call_sites:?}"
        );
        assert!(
            non_default.is_empty(),
            "a NON-DEFAULT order reached a production path — every non-default              entry of ORDERS is an instrument state and licenses no emit              (rungs/README.md's decision-surface clause): {non_default:?}"
        );
    }

    /// **HOW MUCH OF THE PARAMETER IS ACTUALLY EXERCISED — a denominator,
    /// because only a denominator catches an absence (`#3470`, `#1002`).**
    ///
    /// The order has 27 entries. Production reaches the first **three** and no
    /// more, because [`crate::codegen::alloc::MAX_MODELLED_PRODUCERS`] is 3 and the
    /// allowed set is capped at the volatiles. So `w-regsel` makes 27 entries
    /// *executable* and 3 of them *exercised by an emitted byte* — **11.1 %**
    /// — and this test is where that number lives so nobody has to infer it
    /// from a rung.
    ///
    /// Raising [`crate::codegen::alloc::MAX_MODELLED_PRODUCERS`] is board `#541`, which
    /// is open for reasons that have nothing to do with this module.
    #[test]
    fn only_the_first_three_entries_of_the_order_are_reachable_from_an_emitted_byte() {
        let mut reachable: Vec<Reg> = Vec::new();
        for pool_floor in 0..=20u8 {
            for n in 1..=crate::codegen::alloc::MAX_MODELLED_PRODUCERS {
                let pool = RegSet::range_inclusive(pool_floor, crate::codegen::alloc::VOLATILE_GPR_TOP);
                if let Some(seq) = select_sequence(&GPR_DEFAULT, pool, &Costs::ZERO, n) {
                    for r in seq {
                        if !reachable.contains(&r) {
                            reachable.push(r);
                        }
                    }
                }
            }
        }
        reachable.sort_unstable();
        reachable.reverse();
        assert_eq!(
            reachable,
            vec![11, 10, 9],
            "exactly the first three entries of a 27-entry order are reachable"
        );
        assert_eq!(GPR_DEFAULT.regs.len(), 27, "the denominator");
        assert_eq!(&GPR_DEFAULT.regs[..3], &reachable[..], "and they are its PREFIX");
    }

    #[test]
    fn regset_ranges_are_exact_including_the_degenerate_ones() {
        assert_eq!(RegSet::range_inclusive(4, 11).len(), 8);
        assert!(RegSet::range_inclusive(12, 11).is_empty());
        assert!(RegSet::range_inclusive(13, 12).is_empty());
        assert_eq!(RegSet::range_inclusive(0, 63).len(), 64);
        assert_eq!(RegSet::range_inclusive(0, 0).len(), 1);
        assert!(RegSet::range_inclusive(0, 0).contains(0));
        assert!(!RegSet::range_inclusive(4, 11).contains(12));
        assert!(!RegSet::range_inclusive(4, 11).contains(3));
    }
}

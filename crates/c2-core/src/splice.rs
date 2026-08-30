//! **Mechanism I, the replacement stratum** — the body c2 emits for a caller
//! whose one call it expanded, which is the callee's body and **not** the
//! caller's setup with the callee appended.
//!
//! # What this is, and what it is NOT
//!
//! `docs/INLINE_PREDICATE.md` §0 separates two reasons an IL call can be absent
//! from c2's output. [`crate::elide`] is **E** — the callee's body does nothing,
//! so the call is dropped — and this module is **I**, the inliner expanding it.
//! The two are kept apart there and they are kept apart here: nothing in this
//! file reads [`c2_il::IlFunction::empty_body`] as a licence, and
//! [`crate::elide::drops_tail_call`] is asked **first** so that one function
//! never gets two answers.
//!
//! The composition rule is `w-seq`'s measurement and not a guess
//! (`docs/rungs/2026-08-08-w-seq.md` §4.1, graded by real c2 on 2,470 workload
//! callers whose reference obj carries *both* COMDATs):
//!
//! | hypothesis | graded | exact |
//! |---|---:|---:|
//! | **SPLICE-0** — c2's body for the caller **is** c2's body for the callee | 2,470 | **1,967** |
//! | — `seq` | 816 | **816** |
//! | — `tail` | 1,531 | 1,151 |
//! | — `framed` | 123 | **0** |
//! | SPLICE-P — the port's setup ++ the callee's body | 2,470 | 578 — **578/578** with no setup, **0/953** with one |
//! | SPLICE-N — two or more callees, concatenated | 548 | **0** |
//!
//! **The argument setup does not survive, and neither does the frame.** That is
//! the whole difference between this rule and the concatenation the same lane
//! registered first and lost: of the 1,892 SPLICE-P failures, 1,890 diverge at
//! word 0, which is the port's own first setup word.
//!
//! # The predicate, exactly as it was registered
//!
//! [`splice_body`] is the whole rule; `work/w-splice/PREREG.md` §1 is the
//! registered form and `work/w-splice/cells/` is the frozen grid that grades
//! each clause's boundary against real c2.
//!
//! | clause | what it requires | the cell that grades it |
//! |---|---|---|
//! | **S1** | the selection is [`Terminator::TailCall`] or [`Selected::Seq`] | `t07` — a **framed** caller. SPLICE-0 is **0 of 123** there, every one a destination-register rename |
//! | **S2** | exactly **one** call site | `t08` — two calls. SPLICE-N is **0 of 548** |
//! | **S3** | the port emits **nothing around the call**: for a `Tail`, an empty setup; for a `Seq`, an **identity argument mapping read off the IL** and a tail that is the ABI identity on r3 | `t04` (register move), `t05` (arithmetic), `t06` (pointer offset). Every one of w-seq's 503 SPLICE-0 failures is a **field** of the callee's body rewritten — a source rename (286), a destination rename (123), a displacement fold (~92) — and a non-identity setup is the thing that rewrites it. **A `Seq`'s emitted setup is NOT the test**: 816 of 816 carry the frame's own `mr r31,r3` and none carries a real marshalling, so the clause reads the IL |
//! | **S4** | the callee is not the caller | `t12`. `INLINE_PREDICATE.md` §4 grades `recurse` **336/336** declined by c2 as well |
//! | **S5** | the callee is defined in **this bundle**, unambiguously | `t14`, the control: an undefined callee keeps its REL24 |
//! | **S6** | the port composes a complete body for the **chain's end**, with **no frame**, and the chain **ENDED** rather than being cut off — it names no callee this TU carries (`S6-chain-truncated`) and it carries no call at all (`S6-chain-open`) | `t09` — an unlowerable callee. `t11` — **the fixpoint**: when the callee itself splices, its emitted COMDAT is *its* callee's body, so the walk steps down. `t10` — a chain that ends in a call, **refused**, and the 3 verified-correct relocations that refusal costs are the price of the 1 verified-wrong one it removes |
//! | **S7** | that body is at most [`INLINE_UNBOUNDED_BYTES`] bytes, and the callee is not varargs | `t13`. See below — this is the inline decision, taken on the safe side of its own boundary |
//! | **S8** | the caller materializes no data symbol of its own | the caller's whole body is discarded and an `F`-side data reference would go with it |
//! | **S9** | mechanism **E** does not fire for the caller | `elide.rs` keeps its answer; one function, one rule |
//!
//! **Every clause is readable from the IL bundle or from the port's own
//! emitter. None of them reads the reference obj.** That is what makes this a
//! predicate rather than the enumeration `w-seq` §6 priced at 726.
//!
//! # S7 IS THE INLINE DECISION, and it is taken where the decision is categorical
//!
//! `INLINE_PREDICATE.md` §2 is a *cost model* — graded 0.9716 on a 9,993-callee
//! frozen hold-out, with a 2.84 % residual its own §7 leaves **NOT MODELLED**.
//! A shipped emitter may not consult a 97 % rule; 3 % of a guess is a wrong
//! emit. What it may consult is the **region where that rule is categorical**:
//!
//! ```text
//! index(G) = s - 4*(nparams-1) - 8*[inline] - 48*[leaf]        <=  s
//!
//! EXTERNAL: N_max = UNBOUNDED  if index <= 64          (§6.17.4)
//! STATIC:   i = index/4;  N_max = UNBOUNDED if i <= 16 (§6.18.9)   i.e. index <= 64
//! either:   N_max = 0          if varargs(G)           (§6.18.5)
//! ```
//!
//! Both linkage classes turn unbounded at the **same** `index <= 64`, and every
//! correction term is subtractive, so `s <= 64` implies `index <= 64` implies
//! `N_max` unbounded — **independently of linkage, of `inline`, of `nparams`, of
//! the `leaf` bit that §5 calls the model's one unreadable input, and of the
//! site count**. Varargs is the single categorical exception and it is checked.
//!
//! The site-side exceptions of §2 are excluded structurally rather than
//! modelled: an **indirect** site names no callee and the IL parser refuses both
//! of its productions (`elide.rs`'s standing hazard note); a **conditional**
//! site is a `cond_pair` or a `Seq` with a `guard`, and S1/S3 refuse both — and
//! in any case that exception moves a ceiling from 260 to 164 bytes, both far
//! above this bound.
//!
//! # What it emits, and the relocation that makes it different from every other rule here
//!
//! The caller's `/Gy` COMDAT becomes the callee's — text, **relocations** and
//! data references, at the same in-section offsets. The caller acquires **no**
//! REL24 against the callee; it acquires the callee's own.
//!
//! > `t10` is that stated as a compiled cell. `void ext(); void g(){ext();}
//! > void f(){g();}` — the port emits `b ?g` for `?f` and c2 emits `b ?ext`, and
//! > **both are the word `48000000`**. FUNCTION BYTE MATCH compares a `.text`
//! > COMDAT's raw data, which does not contain its relocations, so it calls that
//! > pair `exact` today: board **#882**, 4,664 credited functions, and `w-seq`
//! > §4.3's `s12` is the same four lines. This rule *fixes* that cell rather than
//! > adding to it, and `work/w-splice/relocheck.py` verifies the relocation set
//! > of every function it moves, per symbol, against the reference obj.
//!
//! # THE FIXPOINT — registered as a question, and both answers agree
//!
//! `work/w-splice/PREREG.md` §4 registered *"does c2 close a two-step splice
//! chain?"* as a **question**, with the port taking one level either way. It
//! shipped at one level, and two independent measurements said that was wrong:
//!
//! * **`t11`**, a compiled cell — `int h(int a){return a+1;} int g(int a){return
//!   h(a);} int f(int a){return g(a);}` — c2 emits **`?h`'s two words for all
//!   three functions**;
//! * **the workload's relocation check**, which the one-level rule forced into
//!   existence: **150 of 945** spliced functions relocated against the chain's
//!   *intermediate* where c2 relocates against its *end*.
//!   `??1length_error@stlpmtx_std@@` named `??1__Named_exception@stlpmtx_std@@`
//!   and c2 names `??1exception@std@@`, 145 times in that one shape; the other
//!   five are `??1?$_List_base@…` where c2 names `?clear@?$_List_base@…`. **All
//!   150 were this, and none was a different target.**
//!
//! So the walk follows the chain to the first link that does **not** splice and
//! takes that body. Termination is structural — a step either repeats a name,
//! which is the `S6-chain-cycle` refusal, or admits a new one, and the bundle
//! has finitely many — with a ceiling behind it so that an edit breaking that
//! argument refuses instead of walking forever. Mechanism E reached the same
//! shape from the other direction (`elide.rs`, board #946): a chain, and a cycle
//! that is never admitted.
//!
//! **S7 still needs only one size check.** `INLINE-P`'s `s` is the callee's own
//! *emitted* size, and an intermediate that splices emits exactly the chain-end
//! body — c2's COMDAT for it *is* that body — so `s` is the same number at every
//! edge of the chain.
//!
//! # Where it may NOT fire — [`crate::PortC2::build`]
//!
//! `IlBundle::functions()` refuses any TU that defines one of its own callees,
//! which is a strict superset of S5, so no bundle that can splice ever reaches
//! the whole-obj emitter. That refusal is **not** narrowed here. Independently,
//! `build` refuses a bundle in which this predicate fires, on **both** paths and
//! with the reason named: a spliced `Seq` loses its frame, and with it its
//! `.pdata` record and its `$M`/`$M`/`$T` label slots, while
//! `PortC2::frame_label_counter` is computed from the IL before any body exists.
//! `elide.rs` refuses the packed path for the narrower version of the same
//! reason; this one reaches the label counter, so it refuses both.

use c2_il::{IlFunction, SeqTail};

use crate::codegen::{OptMode, Selected, Terminator, opt_mode_of_word};
use crate::comdat::{ComdatBody, ComdatDecline};
use crate::elide::{Reduction, TuEmptyCallees, drops_tail_call};

/// **S7's bound.** `INLINE_PREDICATE.md` §2's `N_max` is UNBOUNDED at
/// `index <= 64` in *both* linkage classes, and `index <= s`, so a callee whose
/// emitted body is at most this many bytes is inlined at every site whatever its
/// linkage, its parameter count, its `inline` keyword or the model's unreadable
/// `leaf` bit.
///
/// It is a **byte** count because `s` is: §6.5 reads it off the callee's own
/// emitted `.text`.
pub const INLINE_UNBOUNDED_BYTES: usize = 64;

// ===========================================================================
// REGION `w-inlbudget` BEGINS — c2's inline budget model, adopted from a read.
// Everything down to "REGION `w-inlbudget` ENDS" is this lane's; the rest of
// the file is unchanged except for the three call sites named in
// [`splice_body_why`]'s doc.
// ===========================================================================

/// **c2's growth budget, as an executable model.**
///
/// # What this is, and what it is not
///
/// `P_INLINE.md` §6.6.2 (lane `w-inlfit`, board `#3719`/`#3720`) read c2's
/// recursive inline expansion end to end and published a budget model that the
/// port had **no counterpart to at all** — no level, no budget, no site count,
/// no division. The port was nonetheless right on its admitted set, and that
/// lane said exactly why and exactly how far it goes:
///
/// > *"a soundness argument for a fit, not a derivation of one"*
///
/// This module is that derivation, and the arithmetic is the whole of it: the
/// port admits only chains in which **every link has exactly one call site**
/// (`S2`, and `S6-chain-open` requires the end to have none), so `n = 1` at
/// every expansion, so c2's divisor `n − i + 1` is **1** and the nested budget
/// is the parent's, *whatever the parent's budget is*. The port therefore never
/// has to know `B` on the set it admits — and **must refuse the moment it
/// would**, which is `n ≥ 2`. That refusal is the point of the whole file.
///
/// **Nothing here licenses an emit.** The default model reproduces the port's
/// current behaviour exactly, every other entry of [`BUDGET_MODELS`] is an
/// instrument state, and the sole judge stays real `c2.dll` under wibo plus a
/// byte-exact obj compare.
///
/// # Every field is read, and the addresses are in `DISCLOSURE.md`
///
/// Re-derived from the image by this lane rather than relayed —
/// `work/w-inlbudget/IMAGE_READ.md` carries the listings, and the seven claims
/// it re-derives (V1–V7) all confirm. Two things in it are **not** in §6.6.2:
///
/// * **`BYTE [site+0x18]` — the level increment — is `1`**, written into every
///   site record at `0x10b602ce`, and is overridden *only* for a callee
///   carrying `[sym+0x4c] & 0x10` (the bit the driver sets on the function it
///   is expanding, `0x10b61f56`) by that callee's occurrence count among the
///   sites. So on a chain with no recursion the level advances by exactly one
///   per expansion and C14's `0x10` is a true **16-level** cap. §6.6.2 left the
///   field unexplained, and without it the level is uninterpretable.
/// * `0x10b6240f`'s `__forceinline` skip does **not** leave c2's global state
///   untouched: `mov ds:0x10c3f5d0,eax` at `0x10b6240a` runs before the test.
///   Recorded in [`BudgetModel::forceinline_charged`]'s doc so nobody reads the
///   flag as "no trace".
///
/// PROV[R] `DISCLOSURE W-INLBUDGET-1`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BudgetModel {
    /// A stable name, so a permuter or a training run can say which model it
    /// used without printing nine fields.
    pub name: &'static str,
    /// `B = clamp(multiplier × caller_instrs, floor, ceiling)`.
    /// PROV[R] `add eax,eax` at `0x10b62708`.
    pub seed_multiplier: i64,
    /// PROV[R] `mov ebx,0x3e8` at `0x10b6270a`. See [`INLINE_BUDGET_FLOOR`].
    pub seed_floor: i64,
    /// PROV[R] `mov eax,0x88b8` at `0x10b62715`. See [`INLINE_BUDGET_CEILING`].
    pub seed_ceiling: i64,
    /// **THE §6.6.2 FINDING.** Divide the remaining budget evenly among the
    /// remaining call sites: at site *i* of *n* (1-based) the nested pass gets
    /// `remaining / (n − i + 1)`.
    ///
    /// PROV[R] `idiv DWORD PTR [ebp+0x14]` at `0x10b623ec`, divisor traced
    /// through four frames to the site collector's out-parameter
    /// (`lea edx,[ebp-0xc]` `0x10b61f99`, zeroed `0x10b600f7`, incremented per
    /// site `0x10b60374`, decremented per site `0x10b620c8`).
    pub divide_among_remaining_sites: bool,
    /// The level increment per expansion, `BYTE [site+0x18]`.
    ///
    /// PROV[R] `mov BYTE PTR [eax+0x18],0x1` at `0x10b602ce` — **this lane's
    /// read, not §6.6.2's**; see the type's doc.
    pub site_level_delta: i64,
    /// Decline when `level − base` exceeds this. C14.
    ///
    /// PROV[R] `cmp ecx,0x10` / `jg` at `0x10b60a1c`. See
    /// [`INLINE_LEVEL_DEPTH_CAP`].
    pub depth_cap: i64,
    /// c2's second parameter to the accept/decline predicate — `[ebp+0xc]`,
    /// loaded at `0x10b60a10`. **C15**, and it feeds *two* arms rather than
    /// one; see [`BudgetModel::declines_at_depth`] and
    /// [`BudgetModel::declines_at_maxlevel`].
    ///
    /// PROV[R] `cmp edx,0xff` at `0x10b60a2f` and `cmp ecx,edx` at
    /// `0x10b60a21`. See [`INLINE_MAXLEVEL_UNBOUNDED`].
    pub max_level: i64,
    /// A callee at or below this instruction count is **not charged to the
    /// local budget** — but *is* still added to the global growth total. C18.
    ///
    /// PROV[R] `cmp eax,0x28` / `jbe` at `0x10b625b6`, whose `jbe` skips
    /// `0x10b625bb` (local) and NOT `0x10b625c1` (global). See
    /// [`INLINE_CHARGE_EXEMPT_MAX`].
    pub charge_exempt_at_or_below: i64,
    /// Whether a `__forceinline` callee is charged at all.
    ///
    /// PROV[R] `test DWORD PTR [esi+0x4c],0x2000` / `jne` at `0x10b625a6`,
    /// which skips **both** stores, and `0x10b6240f`, which skips both stores
    /// of the nested pass's consumed budget. **It is `false` in c2 and it does
    /// not mean "leaves no trace"**: `mov ds:0x10c3f5d0,eax` at `0x10b6240a`
    /// runs unconditionally, before the test.
    pub forceinline_charged: bool,
    /// **C16's ceiling on the running growth total.** c2 declines the site when
    /// the total exceeds this.
    ///
    /// It is numerically the same 35,000 as [`INLINE_BUDGET_CEILING`] and it is
    /// a **different constant at a different site**, so it is a separate field
    /// and a separate name: a permuter that moves the seed's clamp must be able
    /// to leave the growth ceiling where it is, and a reader must not infer one
    /// from the other.
    ///
    /// PROV[R] `cmp DWORD PTR ds:0x10c3f5cc,0x88b8` at `0x10b60a63`. See
    /// [`INLINE_GROWTH_TOTAL_MAX`].
    pub growth_total_max: i64,
}

/// **THE DEFAULT** — c2's model as read. Index 0 of [`BUDGET_MODELS`], the only
/// model any production path uses, pinned by
/// [`tests::the_only_budget_model_on_a_production_path_is_the_default`].
///
/// PROV[R] `DISCLOSURE W-INLBUDGET-1` — every field's address is on its own
/// doc line above.
pub const BUDGET_C2: BudgetModel = BudgetModel {
    name: "c2-read",
    seed_multiplier: 2,
    seed_floor: INLINE_BUDGET_FLOOR,
    seed_ceiling: INLINE_BUDGET_CEILING,
    divide_among_remaining_sites: true,
    site_level_delta: 1,
    depth_cap: INLINE_LEVEL_DEPTH_CAP,
    max_level: INLINE_MAXLEVEL_UNBOUNDED,
    charge_exempt_at_or_below: INLINE_CHARGE_EXEMPT_MAX,
    forceinline_charged: false,
    growth_total_max: INLINE_GROWTH_TOTAL_MAX,
};

/// **The model the port implicitly held before this lane** — no division, so
/// the nested budget is always the parent's and `n ≥ 2` is never refused.
///
/// It is here because it is the thing `#1020`'s hazard is *about*: with this
/// model selected the surface's `n ≥ 2` rows stop refusing, which is what a
/// widening of `S2` would silently produce. **Instrument only.**
///
/// PROV[N] a counterfactual; reaches no emitted byte.
pub const BUDGET_UNDIVIDED: BudgetModel =
    BudgetModel { name: "undivided", divide_among_remaining_sites: false, ..BUDGET_C2 };

/// `__forceinline` charged like any other callee — §6.6.2's finding negated.
/// **Instrument only.**
///
/// PROV[N] a counterfactual; reaches no emitted byte.
pub const BUDGET_CHARGE_FORCEINLINE: BudgetModel =
    BudgetModel { name: "charge-forceinline", forceinline_charged: true, ..BUDGET_C2 };

/// A flat level — `BYTE [site+0x18]` read as 0 rather than 1, which is what a
/// reader of §6.6.2 alone would have had to guess. With it the depth cap never
/// binds. **Instrument only**, and it is the control for this lane's own read:
/// if `0x10b602ce` had said `0`, this is the model that would be the default.
///
/// PROV[N] a counterfactual; reaches no emitted byte.
pub const BUDGET_FLAT_LEVEL: BudgetModel =
    BudgetModel { name: "flat-level", site_level_delta: 0, ..BUDGET_C2 };

/// c2's `maxlevel` set to 2 — what `#pragma inline_depth(2)` hands the
/// predicate. **Instrument only**, and it is the state that makes
/// [`BudgetModel::max_level`] *reachable*: at the default
/// [`INLINE_MAXLEVEL_UNBOUNDED`] both of C15's arms are vacuous, so a domain
/// rendered over the default alone would cover the field's name and not its
/// boundary — `#3746`'s *"a `guards` entry whose domain cannot reach its const
/// is a false coverage claim"*, one field over.
///
/// `#pragma inline_depth` appears in **0 of the 100** hold-out TUs
/// (`P_INLINE.md` §6.1, C15's `exercised` cell), so nothing in this project
/// compiles in this state and nothing may select it.
///
/// PROV[N] a counterfactual; reaches no emitted byte.
pub const BUDGET_MAXLEVEL_2: BudgetModel =
    BudgetModel { name: "maxlevel-2", max_level: 2, ..BUDGET_C2 };

/// **The enumerable parameter space** — decision 15's *"named, enumerable
/// parameters whose DEFAULT reproduces c2 byte-exactly"*, `regalloc::ORDERS`'
/// shape. Index 0 is the default.
///
/// PROV[N] a list of the four models above, carrying no value of its own.
pub const BUDGET_MODELS: &[BudgetModel] = &[
    BUDGET_C2,
    BUDGET_UNDIVIDED,
    BUDGET_CHARGE_FORCEINLINE,
    BUDGET_FLAT_LEVEL,
    BUDGET_MAXLEVEL_2,
];

/// §2.2's lower clamp on the growth budget, in c2's pre-codegen instruction
/// units.
///
/// PROV[R] `DISCLOSURE W-INLBUDGET-1` — `mov ebx,0x3e8` at `0x10b6270a`.
pub const INLINE_BUDGET_FLOOR: i64 = 1000;

/// §2.2's upper clamp on the growth budget.
///
/// PROV[R] `DISCLOSURE W-INLBUDGET-1` — `mov eax,0x88b8` at `0x10b62715`.
pub const INLINE_BUDGET_CEILING: i64 = 35_000;

/// C14's depth cap: c2 declines when `level − DAT_10c3f50c` **exceeds** this.
///
/// PROV[R] `DISCLOSURE W-INLBUDGET-1` — `cmp ecx,0x10` / `jg 0x10b609f3` at
/// `0x10b60a1c`.
pub const INLINE_LEVEL_DEPTH_CAP: i64 = 16;

/// **C15's guard value.** c2's `maxlevel` parameter at this value switches both
/// of its arms off: `cmp edx,0xff` / `je 0x10b60a3c` at `0x10b60a2f` jumps past
/// the absolute test, and the relative test at `0x10b60a21` cannot fire because
/// `level − base > 255` is unreachable wherever C14's `> 16` has not already
/// declined.
///
/// **It is the value every compilation this project runs is in.**
/// `#pragma inline_depth` appears in 0 of the 100 hold-out TUs, so nothing
/// moves it, which is why adopting C15 changes no emitted byte *by
/// construction* rather than by measurement — see
/// [`BudgetModel::declines_at_maxlevel`].
///
/// PROV[R] `DISCLOSURE W-INLCLAUSE-1` — `cmp edx,0xff` at `0x10b60a2f`.
pub const INLINE_MAXLEVEL_UNBOUNDED: i64 = 255;

/// C18's exemption: a callee at or below this many instructions is not charged
/// to the **local** budget. The global growth total is added regardless.
///
/// PROV[R] `DISCLOSURE W-INLBUDGET-1` — `cmp eax,0x28` / `jbe 0x10b625bd` at
/// `0x10b625b6`.
pub const INLINE_CHARGE_EXEMPT_MAX: i64 = 40;

/// **C16's ceiling on c2's running growth total `DAT_10c3f5cc`.** c2 declines
/// the site when the total is **strictly greater** than this — `jg`, so the
/// value itself still admits.
///
/// The total is seeded from the caller's own instruction count
/// (`0x10b62703`, C2) and added to by every expanded callee (`0x10b625c1`,
/// C19) — and that `add` is **not** gated by C18's 40 test, which the
/// `sub` one instruction above it is. `CLAUSES.tsv` C19 states the two as one
/// clause and records neither asymmetry.
///
/// **Numerically 35,000, like [`INLINE_BUDGET_CEILING`], and deliberately not
/// spelled as it.** They are two immediates at two addresses governing two
/// quantities — a clamp on the seed and a cap on the accumulated total — and
/// the decision-surface clause is about making each separately settable.
///
/// PROV[R] `DISCLOSURE W-BUDGET-1` — `cmp DWORD PTR ds:0x10c3f5cc,0x88b8` /
/// `jg 0x10b609f3` at `0x10b60a63`.
pub const INLINE_GROWTH_TOTAL_MAX: i64 = 35_000;

/// **The nested pass's budget — a divisor when `B` is unknown, a NUMBER when it
/// is.**
///
/// # THE STATE THIS TYPE WAS IN BEFORE LANE `w-budget`, and what changed
///
/// This type used to carry the **divisor and never a number**, because the port
/// had no `B`: `splice.rs`'s own words were *"The port has no honest
/// `caller_instrs` to pass."* `B` is `clamp(2 × WORD [[fn]+0x50], 1000, 35000)`,
/// and `WORD [[fn]+0x50]` is the `.gl` function record's `SIZE` field — **which
/// the port already decoded and threw away** (`P_INLINE.md` §6.2 item 3 and
/// C24's own note). `w-instrcount` read where it comes from; the port now reads
/// the value ([`c2_il::func::gl_function_instr_counts`]).
///
/// **The objection that stood against consuming it, and why it does not reach
/// here.** §6.2 item 3 says consuming `SIZE` *"would be adopting a bound as
/// though it were the quantity"*, on §2.1b's matched pair `arith_012` /
/// `mix_008` — identical `SIZE` 115, opposite verdicts. That measurement is
/// about **C8, the candidacy size test**, and `WB_INSTRCOUNT_FINDINGS` §2.4
/// settled what it shows: **every identified input to that predicate is
/// identical across the pair**, including `ATTR`, so the separator is provably
/// *downstream* of the candidacy predicate `FUN_10b5fb5f`'s size compare and is
/// not this field being wrong. (The address is named on that page and
/// deliberately not repeated here: it is C8's, and a new citation of another
/// row's address moves that row's frozen `cites` footprint for a **mention**.)
/// Meanwhile
/// §2.1a/§2.2's census establishes that `[sym+0x50]` has **one writer in the
/// image** and **no reducer**, so between the `.gl` and `0x10b626f7` the value
/// is unchanged. For the **budget seed** the field is not a bound on the
/// quantity — it *is* the quantity, read instruction by instruction.
///
/// So: at `k = 1` the nested budget is the parent's for every possible `B`, and
/// that was already actionable. At `k ≥ 2` it is `B / k`, which is now a number
/// whenever the caller's count was readable and a refusal when it was not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NestedBudget {
    /// The divisor is 1 **and no `B` is in hand**: the nested budget is the
    /// parent's, undivided, **independently of `B`**.
    Parent,
    /// `parent / k`, `k ≥ 2`, **and no `B` is in hand** — not evaluable.
    Divided {
        /// c2's `n − i + 1` at this site.
        k: i64,
    },
    /// **The budget as a number**, in c2's pre-codegen instruction units.
    ///
    /// Reachable only when the caller's `.gl` `SIZE` was readable, which is what
    /// makes this variant the whole of lane `w-budget`: with it, `k ≥ 2` divides
    /// (`idiv` at `0x10b623ec`) instead of refusing.
    Amount(i64),
}

impl NestedBudget {
    /// True when the port can name the nested budget without knowing `B`.
    ///
    /// **Deliberately still false for [`NestedBudget::Amount`]** — an `Amount`
    /// is a budget the port knows *because* it knows `B`, which is the opposite
    /// of this predicate's question. [`NestedBudget::is_evaluable`] is the one a
    /// refusal should ask.
    pub fn is_evaluable_without_b(&self) -> bool {
        matches!(self, NestedBudget::Parent)
    }

    /// True when the port can act on this budget at all — either because the
    /// divisor is the identity, or because it holds the number.
    pub fn is_evaluable(&self) -> bool {
        !matches!(self, NestedBudget::Divided { .. })
    }

    /// The budget as a number, when there is one. `Parent` has none: it is a
    /// statement *about* an unknown, not a value.
    pub fn amount(&self) -> Option<i64> {
        match self {
            NestedBudget::Amount(b) => Some(*b),
            _ => None,
        }
    }
}

/// One level of c2's recursive expansion, as the port models it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Expansion {
    /// c2's `level`, the driver's `edx`. `1` at the pass entry.
    pub level: i64,
    /// c2's `DAT_10c3f50c`, the base C14 measures the level against. `0` at the
    /// pass entry (`mov ds:0x10c3f50c,ebp`, `0x10b6274c`, `ebp = 0`), and `0`
    /// means the cap does not bind (`je 0x10b60a25`, `0x10b60a15`).
    pub level_base: i64,
    /// What the nested pass's budget is, relative to this one's.
    pub budget: NestedBudget,
    /// **c2's running growth total, `DAT_10c3f5cc`** — C2 seeds it from the
    /// caller's own instruction count at `0x10b62703`, C19 adds every expanded
    /// callee's count at `0x10b625c1`, and C16 declines against it at
    /// `0x10b60a63`.
    ///
    /// **`None` is UNASKED**, and it is the state every `Expansion` was in
    /// before lane `w-budget`: no count was readable, so the port carries no
    /// total and C16 is inert. Every consumer must read it that way.
    pub growth_total: Option<i64>,
}

impl BudgetModel {
    /// §2.2 / C3's seed. `B = clamp(multiplier × caller_instrs, floor,
    /// ceiling)`.
    ///
    /// **AMENDED 2026-08-30, lane `w-budget`: this IS on a production path
    /// now.** It used to say *"the port has no honest `caller_instrs` to pass —
    /// so nothing in `crates/` calls this on a production path"*, and that was
    /// true until `w-instrcount` resolved where the number comes from. It is the
    /// `.gl` function record's `SIZE` field, which the port already decoded and
    /// discarded; [`c2_il::func::gl_function_instr_counts`] returns it and
    /// [`Expansion::at_pass_entry_seeded`] passes it here.
    ///
    /// It is still rendered by the decision surface, which is what makes
    /// [`INLINE_BUDGET_FLOOR`] and [`INLINE_BUDGET_CEILING`] covered rather
    /// than merely named. PROV[R] the load `0x10b626f5`/`0x10b626f7` and the
    /// seed store `0x10b62703`; DISCLOSURE `W-BUDGET-1`.
    pub fn seed(&self, caller_instrs: i64) -> i64 {
        let doubled = self.seed_multiplier.saturating_mul(caller_instrs);
        // c2's order: `max` against the floor first (`0x10b6270f`), then `min`
        // against the ceiling (`0x10b6271a`). The order is visible because the
        // two clamps use different registers and different jumps, and it
        // matters when floor > ceiling — which no c2 configuration produces,
        // but which a permuter setting these fields certainly can.
        let floored = if doubled > self.seed_floor { doubled } else { self.seed_floor };
        if floored < self.seed_ceiling { floored } else { self.seed_ceiling }
    }

    /// c2's divisor at site `site_index` (0-based) of `n_sites`.
    ///
    /// c2's counter is 1-based and decremented at the **bottom** of the loop,
    /// so at the 0-based index `i` it still reads `n − i`.
    pub fn divisor(&self, n_sites: i64, site_index: i64) -> i64 {
        if !self.divide_among_remaining_sites {
            return 1;
        }
        n_sites - site_index
    }

    /// **The model, applied.** What c2 hands the nested pass at this site.
    ///
    /// `Err` is a **malformed** question, not a refusal: a site index outside
    /// its own run, or a run of no sites. The refusals the port owes are
    /// [`port_enter_site`]'s, and they are deliberately a different function so
    /// that the model can still be *asked* about the region the port refuses.
    ///
    /// `callee_instrs` is the count of the callee **being entered**, or `None`
    /// when it was not readable. It feeds C18/C19's charge and nothing else
    /// here; the accept/decline arms it participates in are
    /// [`Self::declines_at_growth_total`] and [`Self::declines_unaffordable`],
    /// which [`port_enter_site`] asks in c2's own order.
    pub fn enter_site(
        &self,
        at: Expansion,
        n_sites: i64,
        site_index: i64,
        callee_instrs: Option<i64>,
    ) -> Result<Expansion, &'static str> {
        if n_sites < 1 {
            return Err("budget-no-sites");
        }
        if !(0..n_sites).contains(&site_index) {
            return Err("budget-site-index-out-of-run");
        }
        let k = self.divisor(n_sites, site_index);
        // **C18/C19's charge, applied before the division** — c2's order:
        // `FUN_10b6242a` subtracts from `*budget` (`0x10b625bb`) and adds to
        // `DAT_10c3f5cc` (`0x10b625c1`) as it expands the site, and the nested
        // pass then divides what is left (`idiv` at `0x10b623ec`).
        //
        // `forceinline = false` for the same reason [`port_enter_site`] passes
        // it: the port cannot read `[sym+0x4c] & 0x2000`, so it takes the
        // charge rather than assuming the bypass. That is the direction that
        // charges MORE, which is the safe one for a growth cap.
        let (local, global) = match callee_instrs {
            Some(ci) => self.charge(ci, false),
            None => (0, 0),
        };
        let budget = match at.budget {
            NestedBudget::Amount(b) => NestedBudget::Amount((b - local) / k),
            // No `B`: the divisor is all the port can say, exactly as before.
            _ if k == 1 => NestedBudget::Parent,
            _ => NestedBudget::Divided { k },
        };
        Ok(Expansion {
            level: at.level + self.site_level_delta,
            level_base: at.level_base,
            budget,
            // `None + anything` stays `None`: no count was readable, so the port
            // has no total and C16 stays inert — the pre-`w-budget` behaviour,
            // preserved by construction rather than by care.
            growth_total: at.growth_total.map(|t| t + global),
        })
    }

    /// **C16.** `35000 < DAT_10c3f5cc ⇒ decline`, asked of the total *as it
    /// stands before this site is charged* — which is where c2 asks it
    /// (`FUN_10b60930` runs before `FUN_10b6242a`).
    ///
    /// `None` is UNASKED and can never decline: without a count there is no
    /// total, and a port that refused on the absence of a number would be
    /// refusing on its own ignorance rather than on c2's rule.
    ///
    /// PROV[R] `cmp DWORD PTR ds:0x10c3f5cc,0x88b8` / `jg 0x10b609f3` at
    /// `0x10b60a63`. The comparison is `jg`, so the ceiling value itself
    /// admits. See [`INLINE_GROWTH_TOTAL_MAX`].
    pub fn declines_at_growth_total(&self, at: Expansion) -> bool {
        at.growth_total.is_some_and(|t| t > self.growth_total_max)
    }

    /// **C17.** `budget < instrs && instrs > 0x28 ⇒ decline`.
    ///
    /// Both operands are needed and both can be missing: without a `B` the
    /// budget is not a number, and without the callee's `.gl` `SIZE` the count
    /// is not either. Either absence is UNASKED and admits.
    ///
    /// PROV[R] `cmp DWORD PTR [ebp+0x10],eax` / `jge 0x10b60a81` at
    /// `0x10b60a73`, then `cmp eax,0x28` / `ja 0x10b609f3` at `0x10b60a78`.
    /// The second comparison is C18's constant at its **first** copy; C18's row
    /// cites the second, `0x10b625b6`, and they are one number at two sites.
    ///
    /// # Why the port may evaluate this at all, and exactly how far
    ///
    /// `[ebp+0x10]` is the budget **threaded through c2's driver recursion**,
    /// and `WB_INSTRCOUNT_FINDINGS` §7 records C17 as *"blocker removed, still
    /// not adoptable"* on the grounds that *"the port has no driver to thread it
    /// through"*. The port's chain walk **is** that threading — `Expansion`
    /// steps down the chain — but only for a chain, and c2's driver has
    /// **fan-out**. On the set the port admits the two coincide exactly: `S2`
    /// requires one call site per link, so c2 expands precisely the port's
    /// chain and charges precisely the port's charges. Off that set the port
    /// has already refused.
    pub fn declines_unaffordable(&self, at: Expansion, callee_instrs: Option<i64>) -> bool {
        let (Some(b), Some(ci)) = (at.budget.amount(), callee_instrs) else {
            return false;
        };
        b < ci && ci > self.charge_exempt_at_or_below
    }

    /// **The base-relative depth arms — C14, and one clause the 24-row table
    /// does not name.** `true` when c2 declines here.
    ///
    /// A `level_base` of 0 means neither arm binds — `je 0x10b60a25` at
    /// `0x10b60a15` jumps past both comparisons. That is the state every
    /// compilation this project runs is in, because the base is seeded to 0 at
    /// the pass entry and is only set non-zero for a function carrying
    /// `[fn+0x4c] & 0x10` (`0x10b61f77`).
    ///
    /// # There are TWO arms behind that `je`, and only one was modelled
    ///
    /// PROV[R] `DISCLOSURE W-INLCLAUSE-1` — `0x10b60a1c` and `0x10b60a21`,
    /// re-derived in `work/w-inlclause/IMAGE_READ.md` §2:
    ///
    /// ```text
    /// 10b60a1c:  cmp ecx,0x10     <- C14, `level - base > 16`
    /// 10b60a1f:  jg  0x10b609f3      decline
    /// 10b60a21:  cmp ecx,edx      <- `level - base > maxlevel` — NOT ANY CLAUSE
    /// 10b60a23:  jg  0x10b609f3      decline
    /// ```
    ///
    /// The second arm is covered by **no row of the 24**: C14 is the `0x10`
    /// comparison and C15 is the *absolute* `level > maxlevel` test at
    /// `0x10b60a2f`–`0x10b60a3a`, which is downstream of the `__forceinline`
    /// bypass and guarded by `maxlevel != 0xff`. This one is neither. So the
    /// port's model of c2's `base != 0` branch was **incomplete**: it admitted
    /// where c2 declines, in a region no byte can reach.
    ///
    /// Novelty checked, not assumed — `0x10b60a21` appears in the frozen
    /// corpus only as an unannotated listing line inside another row's context
    /// window (`work/w-inlclause/read_scan.py`).
    pub fn declines_at_depth(&self, at: Expansion) -> bool {
        if at.level_base == 0 {
            return false;
        }
        let rel = at.level - at.level_base;
        rel > self.depth_cap || rel > self.max_level
    }

    /// **C15.** `maxlevel != 0xff && maxlevel < level ⇒ decline`.
    ///
    /// PROV[R] `cmp edx,0xff` / `je 0x10b60a3c` at `0x10b60a2f` is the `!= 0xff`
    /// guard; `cmp DWORD PTR [ebp+0x8],edx` / `jg 0x10b609f3` at `0x10b60a37` is
    /// the comparison, written the other way round from the clause. `0x10b609f3`
    /// is `xor eax,eax` into the epilogue — the DECLINE sink
    /// (`IMAGE_READ.md` §1).
    ///
    /// # `forceinline` is a parameter because c2's bypass lands BETWEEN the arms
    ///
    /// `and eax,0x2000` at `0x10b60a28` / `jne 0x10b60a3c` skips exactly this
    /// test and reaches `cmp eax,ebx` / `jne 0x10b60a59` = accept. It does
    /// **not** skip [`declines_at_depth`], which is upstream at `0x10b60a1c`.
    /// Modelling that as one flat "forceinline bypasses everything" would get
    /// the precedence wrong, and precedence is a failure a byte delta cannot
    /// see.
    ///
    /// **The port passes `false`** — it cannot read `[sym+0x4c] & 0x2000`, so it
    /// evaluates the test rather than assuming the bypass. At
    /// [`INLINE_MAXLEVEL_UNBOUNDED`] the guard makes that identically `false`
    /// either way.
    pub fn declines_at_maxlevel(&self, at: Expansion, forceinline: bool) -> bool {
        !forceinline && self.max_level != INLINE_MAXLEVEL_UNBOUNDED && at.level > self.max_level
    }

    /// C18/C19's charge, as the pair of numbers c2 actually applies: what comes
    /// off the **local** budget, and what goes onto the **global** growth
    /// total. They are not the same number and the difference is the whole
    /// content of §6.6.2's orthogonality claim.
    ///
    /// Not on any production path — the port has no `callee_instrs`. Rendered
    /// by the surface, which is what covers [`INLINE_CHARGE_EXEMPT_MAX`].
    pub fn charge(&self, callee_instrs: i64, forceinline: bool) -> (i64, i64) {
        if forceinline && !self.forceinline_charged {
            return (0, 0);
        }
        let local = if callee_instrs > self.charge_exempt_at_or_below { callee_instrs } else { 0 };
        (local, callee_instrs)
    }
}

impl Expansion {
    /// The pass entry's state — `FUN_10b61ee1(fn, level = 1, budget = B, 0,
    /// 1e8, 0)`, `0x10b6276e`, with `DAT_10c3f50c` zeroed at `0x10b6274c`.
    pub fn at_pass_entry() -> Expansion {
        Expansion {
            level: 1,
            level_base: 0,
            budget: NestedBudget::Parent,
            // No count in hand. This is the entry state for every caller whose
            // `.gl` `SIZE` was unreadable, and it is exactly the state the port
            // was in everywhere before lane `w-budget`.
            growth_total: None,
        }
    }

    /// **The pass entry WITH c2's seed** — `FUN_10b61ee1(fn, level = 1,
    /// budget = B, …)` with `B` an actual number and `DAT_10c3f5cc` seeded.
    ///
    /// `caller_instrs` is the caller's own `.gl` `SIZE`, which is what c2 loads
    /// at `0x10b626f5`/`0x10b626f7`. c2 does two things with it in six
    /// instructions and this constructor is both of them:
    ///
    /// ```text
    /// 10b62703:  mov ds:0x10c3f5cc,eax   <- C2, the running total's SEED
    /// 10b62708:  add eax,eax             <- C3, the budget's
    /// ```
    ///
    /// PROV[R] DISCLOSURE `W-BUDGET-1`.
    pub fn at_pass_entry_seeded(model: &BudgetModel, caller_instrs: i64) -> Expansion {
        Expansion {
            level: 1,
            level_base: 0,
            budget: NestedBudget::Amount(model.seed(caller_instrs)),
            growth_total: Some(caller_instrs),
        }
    }
}

/// **THE PORT'S OBLIGATION, and it is to REFUSE.**
///
/// [`BudgetModel::enter_site`] answers what c2 does. This answers what the
/// **port** may do with that answer, and the two are different exactly where
/// `#1020`'s hazard lives:
///
/// > `w-inlfit`, `P_INLINE.md` §6.6.2 — *"the moment a lane widens `S2` to two
/// > call sites, `n = 2`, c2's divisor stops being 1, and the port has nothing
/// > to divide."*
///
/// So: `n = 1` gives [`NestedBudget::Parent`], which is evaluable for every
/// possible `B` and is the state the port's whole admitted set is in; anything
/// else refuses by name. The refusal is **unreachable from any fixture today**
/// — `S2` refuses a two-call body long before the walk starts — which is
/// precisely why it is registered as a decision surface instead of trusted to a
/// byte delta (board `#3723`).
pub fn port_enter_site(
    model: &BudgetModel,
    at: Expansion,
    n_sites: i64,
    site_index: i64,
    callee_instrs: Option<i64>,
) -> Result<Expansion, &'static str> {
    let next = model.enter_site(at, n_sites, site_index, callee_instrs).map_err(|e| match e {
        "budget-no-sites" => "S6-budget-no-sites",
        _ => "S6-budget-site-index",
    })?;
    if !next.budget.is_evaluable() {
        // `n ≥ 2` **and no `B`**. c2 divides; the port cannot, because the
        // caller's count was not readable.
        //
        // **THIS IS NO LONGER A BLANKET REFUSAL** (lane `w-budget`). Before, it
        // fired at every `n ≥ 2` because `B` was unknowable; now it fires only
        // where the caller's `.gl` `SIZE` was unreadable — a whole-file refusal
        // of the reader, the `0x81..=0xff` encoding, or a name with no record.
        // Where the count IS in hand the divide is `idiv`'s arithmetic and the
        // port evaluates c2's own arms below instead of declining to look.
        return Err("S6-budget-divided");
    }
    if model.declines_at_depth(next) {
        return Err("S6-budget-depth-cap");
    }
    // C15, in c2's own order: the base-relative arms above are upstream of the
    // `__forceinline` bypass at `0x10b60a28`, this one is downstream of it. The
    // port cannot see that bit, so it passes `false` and takes the test.
    if model.declines_at_maxlevel(next, false) {
        return Err("S6-budget-maxlevel");
    }
    // **C16 and C17, asked of the state BEFORE this site is charged** — which
    // is where c2 asks them. `FUN_10b60930`'s accept/decline runs on the
    // caller's current `DAT_10c3f5cc` and current `*budget`; `FUN_10b6242a`
    // charges afterwards, and `enter_site` above has already applied that
    // charge to `next`. So both arms read `at`, not `next`, and getting that
    // backwards would decline one site early — a precedence error a byte delta
    // cannot see, which is this module's standing reason for spelling orders
    // out (see [`BudgetModel::declines_at_maxlevel`]'s own note).
    if model.declines_at_growth_total(at) {
        return Err("S6-budget-caller-huge");
    }
    if model.declines_unaffordable(at, callee_instrs) {
        return Err("S6-budget-unaffordable");
    }
    Ok(next)
}

/// **SURFACE[splice.budget]** — the registered decision surface's domain
/// (`crate::surface`, board `#3762`, lane `w-inlbudget`).
///
/// Two blocks, because the model has two halves the port relates to
/// differently:
///
/// * the **seed and charge** rows, which no production path evaluates and which
///   exist so that [`INLINE_BUDGET_FLOOR`], [`INLINE_BUDGET_CEILING`] and
///   [`INLINE_CHARGE_EXEMPT_MAX`] are *covered* rather than merely named — the
///   `#3746` trap being a `guards` entry whose domain cannot reach it;
/// * the **site** rows, which are the port's own refusal boundary, enumerated
///   over `n = 1..=6` — five sixths of which the corpus can never reach,
///   because `S2` refuses a two-call body before the walk begins.
///
/// All four [`BUDGET_MODELS`] are rendered, not just the default:
/// `regalloc::surface_rows`' reason, and a stronger one here, since
/// [`BUDGET_UNDIVIDED`] is exactly the model a `S2` widening would silently
/// install.
pub fn surface_rows() -> Vec<crate::surface::Row> {
    let mut rows = Vec::new();
    for m in BUDGET_MODELS {
        for &c in &[0i64, 1, 400, 499, 500, 501, 17_499, 17_500, 20_000] {
            rows.push(crate::surface::Row::new(
                format!("model={} seed caller_instrs={c:06}", m.name),
                format!("B={}", m.seed(c)),
            ));
        }
        for &ci in &[0i64, 1, 39, 40, 41, 1000] {
            for fi in [false, true] {
                let (local, global) = m.charge(ci, fi);
                rows.push(crate::surface::Row::new(
                    format!("model={} charge instrs={ci:06} forceinline={}", m.name, fi as u8),
                    format!("local={local},global={global}"),
                ));
            }
        }
        for n in 1..=6i64 {
            for i in 0..n {
                for &(level, base) in
                    &[(1i64, 0i64), (2, 0), (16, 0), (17, 0), (2, 1), (5, 1), (17, 1), (18, 1)]
                {
                    let at = Expansion {
                        level,
                        level_base: base,
                        budget: NestedBudget::Parent,
                        growth_total: None,
                    };
                    let outcome = match port_enter_site(m, at, n, i, None) {
                        Ok(e) => format!("level={},budget=parent", e.level),
                        Err(why) => format!("{} {why}", crate::surface::REFUSE),
                    };
                    rows.push(crate::surface::Row::new(
                        format!(
                            "model={} n={n} i={i} level={level:02} base={base}",
                            m.name
                        ),
                        outcome,
                    ));
                }
            }
        }
    }
    // ---- C15, lane `w-inlclause` ------------------------------------------
    //
    // A SWEEP OVER `max_level` ITSELF, and it is not decoration: the guard in
    // `declines_at_maxlevel` compares the field against
    // `INLINE_MAXLEVEL_UNBOUNDED`, and `BUDGET_C2.max_level` IS that constant —
    // so mutating the constant moves both sides of the comparison together and
    // a domain rendered over the named models alone would not move one line.
    // That is `#3746`'s false-coverage trap exactly ("a `guards` entry whose
    // domain cannot reach its const"), and the only thing that escapes it is
    // asking the predicate at values the constant does not follow.
    //
    // `forceinline` is swept because `port_enter_site` can only ever pass
    // `false` — the port cannot read `[sym+0x4c] & 0x2000`. The bypass at
    // `0x10b60a28` lands BETWEEN c2's two depth arms, and a domain that showed
    // only the reachable half would hide that precedence.
    for &ml in &[0i64, 2, 254, INLINE_MAXLEVEL_UNBOUNDED, 256] {
        let m = BudgetModel { name: "sweep", max_level: ml, ..BUDGET_C2 };
        for &level in &[1i64, 3, 255, 300] {
            for fi in [false, true] {
                let at = Expansion {
                    level,
                    level_base: 0,
                    budget: NestedBudget::Parent,
                    growth_total: None,
                };
                rows.push(crate::surface::Row::new(
                    format!("maxlevel={ml:03} level={level:03} forceinline={}", fi as u8),
                    if m.declines_at_maxlevel(at, fi) {
                        format!("{} C15-decline", crate::surface::REFUSE)
                    } else {
                        "admit".to_string()
                    },
                ));
            }
        }
    }
    // ---- THE THREADED COUNT, lane `w-budget` ------------------------------
    //
    // Everything above renders with **no count in hand**, which is the state
    // the port was in everywhere before this lane — so every line above is
    // unchanged by it, and the block below is the whole of what moved.
    //
    // THIS IS THE `#3723` BLOCK. A required-zero byte delta cannot see any of
    // it, because `S2` refuses a two-call body before the walk begins and the
    // `.gl` reader refuses the `0x81..=0xff` encoding before a count above
    // 65,408 can reach the port. Both regions are enumerated here anyway, at
    // values no fixture can produce, which is the whole mechanism: a widening
    // has one way forward and it is a text diff somebody reads.
    //
    // The caller counts are chosen to straddle every boundary the seed and the
    // growth cap have:
    //
    //   0, 1            the floor's far side
    //   499/500/501     `2 x c` crossing the 1000 floor
    //   17_499/17_500   `2 x c` crossing the 35_000 ceiling
    //   35_000/35_001   `DAT_10c3f5cc`'s OWN cap, C16 — the seed alone puts a
    //                   caller over it, which is c2 declining the caller's very
    //                   first site because the caller is huge
    //   65_409, 65_535  **c2's `0x81..=0xff` sign-extension band**
    //                   (`WB_INSTRCOUNT_FINDINGS` §4). c2 reads a caller in this
    //                   state as ~65k instructions and declines everything. The
    //                   port's `.gl` reader REFUSES that encoding whole-file, so
    //                   no such count can arrive — and the model is asked here
    //                   anyway, because "unreachable" is a claim about the
    //                   reader and this file is about the decision.
    for &c in &[0i64, 1, 499, 500, 501, 17_499, 17_500, 35_000, 35_001, 65_409, 65_535] {
        for n in 1..=3i64 {
            for i in 0..n {
                // Callee counts across C18's 40 and past the 1000 floor, so
                // C17's arm is reachable in the domain and not merely named.
                for &ci in &[None, Some(0i64), Some(40), Some(41), Some(1_200)] {
                    let at = Expansion::at_pass_entry_seeded(&BUDGET_C2, c);
                    let outcome = match port_enter_site(&BUDGET_C2, at, n, i, ci) {
                        Ok(e) => format!(
                            "level={},budget={},total={}",
                            e.level,
                            e.budget.amount().map_or("parent".to_string(), |b| b.to_string()),
                            e.growth_total.map_or("-".to_string(), |t| t.to_string()),
                        ),
                        Err(why) => format!("{} {why}", crate::surface::REFUSE),
                    };
                    rows.push(crate::surface::Row::new(
                        format!(
                            "seeded caller={c:06} n={n} i={i} callee={}",
                            ci.map_or("none".to_string(), |v| format!("{v:06}")),
                        ),
                        outcome,
                    ));
                }
            }
        }
    }
    // The growth cap ITSELF, swept independently of the seed. `#3746`'s trap
    // one field over: `BUDGET_C2.growth_total_max` IS `INLINE_GROWTH_TOTAL_MAX`,
    // so a sweep that only moved the caller count would move the compared value
    // and the constant together on the seeded rows above. Here the model's field
    // is held at named values the constant does not follow, which is what makes
    // the constant a reachable boundary rather than a covered name.
    for &cap in &[0i64, 1_000, INLINE_GROWTH_TOTAL_MAX, 65_535] {
        let m = BudgetModel { name: "cap-sweep", growth_total_max: cap, ..BUDGET_C2 };
        for &total in &[0i64, 999, 1_000, 1_001, 35_000, 35_001, 65_535] {
            let at = Expansion {
                level: 1,
                level_base: 0,
                budget: NestedBudget::Amount(1_000),
                growth_total: Some(total),
            };
            rows.push(crate::surface::Row::new(
                format!("growthcap={cap:06} total={total:06}"),
                if m.declines_at_growth_total(at) {
                    format!("{} C16-decline", crate::surface::REFUSE)
                } else {
                    "admit".to_string()
                },
            ));
        }
    }
    rows
}

// ===========================================================================
// REGION `w-inlbudget` ENDS.
// ===========================================================================

/// **The bundle-level facts a `/Gy` body needs that are not properties of one
/// function** — mechanism E's callee set, and the callee bodies mechanism I
/// splices.
///
/// It carries [`TuEmptyCallees`] rather than replacing it, and [`Deref`]s to it,
/// so every existing caller of `drops_tail_call(f, tu)` and `tu.reduces_to_nothing(..)` reads the
/// same E context it always did. **E is still `elide.rs`'s and only its**; this
/// type is the carrier, not a second implementation.
///
/// # It is name-keyed, and *which* name is the whole problem
///
/// The same #918 that `elide.rs` documents: `IlFunction::mangled_name` is paired
/// **positionally** over `.ex` segments and disagrees with the per-record
/// `FnCensus::emit_name` on **74,955** rows of the dc3 workload. A name-keyed
/// fact read through the wrong binding is attached to the wrong function — for
/// E that turned 14 byte-exact bodies wrong, and for this rule it would splice
/// some *other* function's body into the caller. So the name is supplied by the
/// caller through [`TuContext::of_named`], exactly as E's is, and the callee
/// resolution rate under the two bindings on this very population is **3,702 vs
/// 546 of 3,928** (w-seq §8).
#[derive(Debug, Default, Clone)]
pub struct TuContext<'a> {
    empty: TuEmptyCallees,
    /// `(name, definition, the `.ex` optimization word)`, sorted by name.
    /// A name with more than one row is **refused** rather than resolved to the
    /// first — see [`TuContext::definition`].
    ///
    /// The definition is `None` for a name this bundle **defines and whose IL
    /// the parser refused**. Those rows are kept, and keeping them is not
    /// bookkeeping: [`TuContext::mentions`] is what tells a chain that *ended*
    /// from one the port could not *follow*, and a refused row that vanished
    /// from this vector would read as an external. See the type's docs.
    ///
    ///
    /// The definition is `None` for a name this bundle **defines and whose IL
    /// did not parse**. Carrying those rows costs nothing and buys the one
    /// distinction a refusal census has to make: *"this TU has no such
    /// function"* and *"this TU has one and the port cannot read it"* price two
    /// completely different rungs — the second is `w-seq` §5's production table,
    /// which is 1,774 differs deep. Folding them together is how
    /// `refused:blocked` got printed 1,774 times and named nothing.
    rows: Vec<(&'a str, Option<&'a IlFunction>, Option<u32>)>,
    /// **c2's pre-codegen instruction count per name**, sorted, from the `.gl`
    /// function record's `SIZE` field ([`c2_il::func::gl_function_instr_counts`]).
    ///
    /// Carried here rather than on [`IlFunction`] because it is a **whole-bundle
    /// fact keyed by the record binding**, which is what this type already is —
    /// and because the binding is the whole hazard. The count is looked up
    /// through the same names [`TuContext::definition`] resolves, so the callee
    /// side cannot get one function's body and another's count.
    ///
    /// **Empty is the pre-`w-budget` state and is a legal, silent one**: every
    /// consumer treats a missing count as UNASKED and behaves exactly as it did
    /// before this vector existed.
    counts: Vec<(&'a str, i64)>,
}

impl<'a> TuContext<'a> {
    /// The context for a caller with no bundle: nothing is elided and nothing is
    /// spliced.
    ///
    /// Required rather than defaulted, for [`crate::comdat`]'s reason: the two
    /// bundle-level mechanisms are the facts a per-function composition cannot
    /// derive, and a caller that *forgot* one would silently emit a call c2 does
    /// not emit. Saying "no bundle" out loud is the point.
    pub fn none() -> Self {
        Self::default()
    }

    /// Build both bundle-level contexts from one pass over `(name, definition,
    /// opt_word)` triples the caller has already bound.
    ///
    /// `opt_word` is the callee's own `.ex` optimization word. `None` means the
    /// caller does not track it per function — [`crate::PortC2::build`], where
    /// the TU has already been refused unless every function shares one mode —
    /// and then the caller's mode is used. A callee whose mode differs from the
    /// caller's is refused (`mode` decides a register field inside a chain, so
    /// splicing across a mode boundary would emit the wrong allocation).
    pub fn of_named(
        named: impl IntoIterator<Item = (&'a str, Option<&'a IlFunction>, Option<u32>)>,
    ) -> Self {
        Self::of_rows(
            named
                .into_iter()
                // `Some(f)` is a parsed body both mechanisms may use; `None`
                // is a name this TU defines whose IL the parser refused, which
                // neither can use and which must still be VISIBLE — see
                // `of_rows`. This wrapper has no `no_effect_callee` to offer,
                // so it never mints a `NoEffectCall` edge; the census-row
                // caller in `gap/fnbytes.rs` is the one that does (#980).
                .map(|(n, f, w)| (n, f.map(Reduction::Parsed), w)),
        )
    }

    /// **The general constructor** — one row per name this TU defines, carrying
    /// what each of the two mechanisms can make of it.
    ///
    /// # Three questions, three row sets, and only one of them is every row
    ///
    /// | asked by | which rows |
    /// |---|---|
    /// | mechanism **E**'s closure ([`TuEmptyCallees::of_rows`]) | the rows with a [`Reduction`] — parsed, or refused **with a readable `no_effect_callee`** (board **#980**). Every other refused row contributes nothing, which is that board's conservative direction and is preserved here exactly |
    /// | [`TuContext::definition`], i.e. the splice's S5/S6 | **parsed rows only**. A `NoEffectCall` row is a body the parser refused, so the port has no bytes for it and S6 cannot compose a chain end out of it |
    /// | [`TuContext::mentions`] | **every row**, parsed or not, qualifying or not |
    ///
    /// **The third is the one a careless merge loses.** `S6-chain-truncated`
    /// refuses a splice when the chain's last link still names a callee this TU
    /// carries; a refused row dropped from this vector would read as an
    /// *external*, the clause would stop firing, and the splice would run off
    /// the end of a chain it cannot see. That is not a missing count — it is
    /// the wrong-relocation defect (#1020's 72 witnesses) coming back.
    ///
    /// So `Option<Reduction>` is three-valued on purpose: `Some(Parsed)` is a
    /// body both mechanisms can use, `Some(NoEffectCall)` is one only E can use,
    /// and `None` is a name this TU defines that **neither** can use and that
    /// still has to be visible.
    pub fn of_rows(
        rows: impl IntoIterator<Item = (&'a str, Option<Reduction<'a>>, Option<u32>)>,
    ) -> Self {
        let mut rows: Vec<(&'a str, Option<Reduction<'a>>, Option<u32>)> =
            rows.into_iter().filter(|(n, _, _)| !n.is_empty()).collect();
        rows.sort_by_key(|(n, _, _)| *n);
        // Mechanism E sees exactly what board #980 gives it and nothing else.
        let empty = TuEmptyCallees::of_rows(
            rows.iter().filter_map(|(n, r, _)| Some((*n, (*r)?))),
        );
        let rows = rows
            .into_iter()
            .map(|(n, r, w)| {
                // **Matched EXHAUSTIVELY, with no wildcard arm.** A refused body
                // has no bytes, so the splice can never take one — but the arm
                // that says so used to be `_ => None`, and a wildcard is how a
                // variant added by a peer lane gets a silent answer instead of a
                // considered one. Four lanes this week erased each other's work
                // through shared semantics with no textual conflict; a wildcard
                // here is that failure mode with the compiler's help switched off.
                let def = match r {
                    Some(Reduction::Parsed(f)) => Some(f),
                    // E gets a LINK out of this row (board #980); the splice gets
                    // nothing, because there is no parsed body to compose from.
                    Some(Reduction::NoEffectCall(_)) => None,
                    // E gets a SEED out of this row (board #1053); the splice
                    // still gets nothing, for the same reason and no other.
                    Some(Reduction::NoEffectNothing) => None,
                    // A name this TU defines that NEITHER mechanism can use, and
                    // which must still be visible to `mentions` — see this
                    // function's doc.
                    None => None,
                };
                (n, def, w)
            })
            .collect();
        Self { empty, rows, counts: Vec::new() }
    }

    /// **Attach c2's per-function instruction counts** — the `.gl` `SIZE` field,
    /// keyed by the same record names this context's rows are keyed by.
    ///
    /// Additive and chained rather than a new constructor, so that every
    /// existing caller keeps the behaviour it had: a context with no counts is
    /// the pre-`w-budget` context exactly, and the budget model then refuses
    /// `n ≥ 2` as it always did.
    ///
    /// A name supplied twice with two different counts is **dropped**, not
    /// resolved to either — [`TuContext::definition`]'s rule, one field over,
    /// and for the same reason: two spellings of one name would have to be told
    /// apart by something this type does not have.
    pub fn with_instr_counts(
        mut self,
        counts: impl IntoIterator<Item = (&'a str, i64)>,
    ) -> Self {
        let mut v: Vec<(&'a str, i64)> = counts.into_iter().collect();
        v.sort_by_key(|(n, _)| *n);
        v.dedup_by(|b, a| a.0 == b.0 && a.1 == b.1);
        // Any name still repeated disagrees with itself; remove every copy.
        let mut out: Vec<(&'a str, i64)> = Vec::with_capacity(v.len());
        let mut i = 0usize;
        while i < v.len() {
            let j = v[i..].iter().take_while(|(n, _)| *n == v[i].0).count();
            if j == 1 {
                out.push(v[i]);
            }
            i += j;
        }
        self.counts = out;
        self
    }

    /// **c2's instruction count for `name`**, or `None` when it was not
    /// readable — no `.gl`, a `.gl` the reader refused, the `0x81..=0xff`
    /// encoding, no record, or two records disagreeing.
    ///
    /// `None` is UNASKED and never a zero: a function of zero instructions and
    /// a function whose count nobody could read are different facts, and only
    /// the first may seed a budget.
    pub fn instr_count(&self, name: &str) -> Option<i64> {
        let i = self.counts.binary_search_by(|(n, _)| (*n).cmp(name)).ok()?;
        Some(self.counts[i].1)
    }

    /// [`TuContext::of_named`] over `IlFunction::mangled_name` with no per
    /// function optimization word — the binding `IlBundle::functions()` hands
    /// the emitter, where the TU's mode is already known to be uniform.
    ///
    /// **Do not reach for this from a census-row caller**, for the reason
    /// [`TuEmptyCallees::of`] gives: the census pairs names positionally.
    pub fn of(funcs: impl IntoIterator<Item = &'a IlFunction>) -> Self {
        Self::of_named(
            funcs
                .into_iter()
                .map(|f| (f.mangled_name.as_str(), Some(f), None)),
        )
    }

    /// Mechanism E's context — [`elide`](crate::elide)'s and unchanged.
    pub fn empty_callees(&self) -> &TuEmptyCallees {
        &self.empty
    }

    /// **S5.** The one definition of `name` in this bundle, or `None` when zero
    /// or **more than one** row carries it.
    ///
    /// Two rows are refused rather than resolved to the first: the two spellings
    /// would have to be told apart by something this type does not have, and
    /// splicing the wrong one is a silent wrong-bytes emit. This is the same
    /// answer `TuEmptyCallees` gives a name whose definitions disagree.
    pub fn definition(&self, name: &str) -> Option<(&'a IlFunction, Option<u32>)> {
        let i = self.unique_row(name)?;
        Some((self.rows[i].1?, self.rows[i].2))
    }

    /// Does this bundle carry `name` **at all** — whether or not its IL parsed,
    /// and whether or not exactly one row claims it?
    ///
    /// The distinction [`TuContext::definition`] cannot make, and the one a
    /// refusal census needs: a callee this TU does not define is an external and
    /// no mechanism can be about it, while a callee it defines and the parser
    /// refuses is a *priced* rung — `w-seq` §5 ranks those productions and the
    /// largest blocks 573 differs on its own.
    ///
    /// **It deliberately does not require the row to be unique.** An ambiguous
    /// name is one this TU *does* define, twice; reading it as "not defined
    /// here" is how one function slipped past `S6-chain-truncated` and
    /// relocated against `??1?$_Rb_tree@…` where c2 relocates against
    /// `?clear@?$_Rb_tree@…` — a wrong relocation FUNCTION BYTE MATCH scores
    /// `exact` (board #882).
    pub fn mentions(&self, name: &str) -> bool {
        self.rows
            .binary_search_by(|(n, _, _)| (*n).cmp(name))
            .is_ok()
    }

    /// More than one row claims `name`, so no single definition can be resolved.
    pub fn ambiguous(&self, name: &str) -> bool {
        self.mentions(name) && self.unique_row(name).is_none()
    }

    /// The single row for `name`, or `None` when zero or **more than one**
    /// carries it. Two rows are refused rather than resolved to the first, for
    /// the reason [`TuContext::definition`]'s doc gives.
    fn unique_row(&self, name: &str) -> Option<usize> {
        let i = self.rows.binary_search_by(|(n, _, _)| (*n).cmp(name)).ok()?;
        if i > 0 && self.rows[i - 1].0 == name {
            return None;
        }
        if i + 1 < self.rows.len() && self.rows[i + 1].0 == name {
            return None;
        }
        Some(i)
    }

    /// How many definitions the context can splice from — printed by
    /// diagnostics rather than inferred, the same reason [`TuEmptyCallees::len`]
    /// exists.
    ///
    /// # It is NOT called `len`, and that is a defect this lane shipped and caught
    ///
    /// This type `Deref`s to [`TuEmptyCallees`] so that every existing caller of
    /// `drops_tail_call(f, &tu)` and `tu.reduces_to_nothing(..)` keeps working
    /// untouched. **An inherent method shadows the `Deref` target's**, so while
    /// this was spelled `len` the scan's `fnbyte-tu-empty-callees` key — which
    /// has always meant *"how many names mechanism E admits"* — silently began
    /// reporting the bundle's whole definition count instead: **88,894 →
    /// 1,474,755** on the dc3 workload, a sixteen-fold move in a key nobody was
    /// diffing, with no compile error and no test failure.
    ///
    /// It was found by a merge-time control that counts every *peer lane's* key
    /// family at both ends (`work/w-splice/peerkeys.py`), not by anything in
    /// this lane's own acceptance. The name is now unambiguous, so no future
    /// caller can reach the wrong `len` by writing the obvious thing —
    /// `docs/GAPS.md` §6's "one fact, one locator" applied to a *name* rather
    /// than to an implementation.
    pub fn definitions(&self) -> usize {
        self.rows.len()
    }

    /// True when the bundle contributed no definition at all.
    ///
    /// Named apart from [`TuEmptyCallees::is_empty`] for [`Self::definitions`]'s
    /// reason.
    pub fn has_no_definitions(&self) -> bool {
        self.rows.is_empty()
    }
}

impl<'a> std::ops::Deref for TuContext<'a> {
    type Target = TuEmptyCallees;

    fn deref(&self) -> &TuEmptyCallees {
        &self.empty
    }
}

/// **S1–S5, S8, S9** — the part of the predicate that is a property of the
/// caller's own selection, with the callee it names.
///
/// Separated from [`splice_body`] because it is the half that can be asserted
/// without lowering anything, so a test can walk the boundary cells without an
/// IL bundle for the callee — and because `PortC2::build`'s refusal needs the
/// same question asked without paying for the callee's body.
///
/// `None` is the refusal, and it is the answer for every shape this rule was not
/// graded on. Nothing here is an assertion about the parser's invariants: an
/// [`IlFunction`] is a parse result and this is a predicate over it.
pub fn splice_callee<'a>(
    f: &IlFunction,
    selected: &Selected,
    tu: &TuContext<'a>,
) -> Option<&'a str> {
    splice_callee_why(f, selected, tu).ok()
}

/// [`splice_callee`] with **the clause that refused**, for a diagnostic that has
/// to price the shortfall rather than report it.
///
/// One implementation, two readers — the emitter takes `Ok`/`Err` and the
/// instrument prints the `Err`. A separate "why did it refuse" routine would be
/// the one-fact-two-implementations drift `docs/GAPS.md` §6 keeps recording, and
/// here it would be worse than usual: the refusal reason is exactly what the
/// next widening rung is priced in, so a stale copy would price the wrong
/// clause.
pub fn splice_callee_why<'a>(
    f: &IlFunction,
    selected: &Selected,
    tu: &TuContext<'a>,
) -> Result<&'a str, &'static str> {
    // **S9.** Mechanism E answers first and keeps its `blr`. Asked before
    // anything else so that a reader meets the precedence at the top, and so
    // that the two rules can never both claim one function.
    if drops_tail_call(f, tu.empty_callees()) {
        return Err("S9-mechanism-e");
    }
    // **S8.** The caller's whole body is discarded; a data symbol of its own
    // would be discarded with it and no cell grades that.
    if !f.data_syms.is_empty() {
        return Err("S8-caller-data-sym");
    }
    // **S1 and S3.** `Framed` is 0 of 123 and `CondPair` is a conditional site
    // that was never graded; `Plain` and `Float` name no callee at all. What
    // survives is the two shapes whose whole emitted body is the call — which is
    // S3, and which is why the two clauses are one match.
    let callee: &str = match selected {
        // A tail call with a non-empty setup is SPLICE-P's `port_words > 1`
        // stratum: **0 of 953**, with 1,890 of them diverging at word 0.
        // **S1b keeps this clause EXACTLY as it was, and that is load-bearing.**
        // `setup.is_empty()` is a *semantic* stratum, not a spelling: SPLICE-P's
        // `port_words > 1` bucket is **0 of 953**, with 1,890 of them diverging
        // at word 0. Collapsing `Terminator::TailCall` into a terminator had to keep
        // the emptiness test on the same bytes it always tested — the body
        // before the branch — and it does: `text` here is what `setup` was.
        Selected::Body { text: setup, term: Terminator::TailCall } if setup.is_empty() => {
            match f.tail_call() {
                Some(c) => c,
                None => return Err("S1-tail-without-callee"),
            }
        }
        Selected::Body { term: Terminator::TailCall, .. } => return Err("S3-tail-setup"),
        // **S1/S3 — W-CFG1 names TWO callees and has a conditional site**, so it
        // is out on both of the clauses that exclude `CondPair`. Refused
        // explicitly rather than by a catch-all, so a later shape cannot fall
        // into a splice path nobody graded.
        Selected::IfCallJoin => return Err("S3-if-call-join"),
        // **W-BIQUAD: thirty-five words, two branches and no call at all.**
        // Mechanism I replaces a body that is NOTHING BUT one call, and this
        // one names no callee, so it can be neither source nor target. Refused
        // explicitly rather than by a catch-all, for the same reason every
        // clause here is: a later shape must not fall into a splice path nobody
        // graded.
        Selected::FpStoreDiamond { .. } => return Err("S3-fp-store-diamond"),
        // **W-BIQUAD's constructor IS "nothing but one call"**, which is exactly
        // mechanism I's shape — and it is refused all the same, because S3's
        // measured stratum is a body whose emitted words are the call and
        // nothing else. This one parks `this` first and carries a frame, so
        // splicing the callee in would drop the park, the frame, the `.pdata`
        // record and the label triple with it. Named rather than caught by a
        // catch-all, because it is the near miss.
        Selected::CtorForwardCall => return Err("S3-ctor-forward-call-has-a-frame"),
        // W-EXTDATA: a framed multi-call body is not a splice source or target
        // for the same reason W-CFG1 is not — mechanism I replaces a body that
        // is NOTHING BUT one call, and this one is thirty words.
        Selected::GuardChainSharedTail => return Err("S3-guard-chain-shared-tail"),
        // W-UNDNAME: same clause, same reason — twenty-four words, one call,
        // and a body that is not "nothing but that call". Refused explicitly so
        // a later shape cannot fall into a splice path nobody graded.
        Selected::AllocInitOrFail => return Err("S3-alloc-init-or-fail"),
        // W-OSFINFO: same clause, same reason — thirty-one words and TWO calls.
        Selected::OsfHandleGuard => return Err("S3-osf-handle-guard"),
        Selected::GuardRetChain => return Err("S3-guard-ret-chain"),
        // W-MMIO3: same clause, same reason — thirty-one words, THREE call
        // statements of which one is elided, and a body that is not "nothing
        // but that call". Refused explicitly so a later shape cannot fall
        // into a splice path nobody graded — and here that matters more than
        // usual, because a splice of this body would have to decide the
        // elision again, at a seam that has no access to the sibling.
        Selected::CloseCallChain => return Err("S3-close-call-chain"),
        // W-XLR: same clause, same reason — thirty-eight words and FOUR
        // relocations, two of which are its own frame's helpers. A splice would
        // have to reproduce a prologue that calls out of the function.
        Selected::XlrcCreateGuard => return Err("S3-xlrc-create-guard"),
        Selected::JsonUtf8Copy => return Err("S3-json-utf8-copy"),
        Selected::Body { term: Terminator::MemcpyCall, .. } => return Err("S3-memcpy-tail"),
        Selected::XteaEncryptLoop => return Err("S3-xtea-encrypt-loop"),
        Selected::Seq { setups, .. } => {
            let Some(seq) = f.call_seq() else {
                return Err("S1-seq-without-call-seq");
            };
            // **S2.** One call site. SPLICE-N is 0 of 548.
            if seq.calls.len() != 1 || setups.len() != 1 {
                return Err("S2-multi-call");
            }
            // **S3.** Nothing around the call: no argument setup, no guarded
            // block (which is also §2's *conditional site*), no early return.
            if seq.guard.is_some() || !seq.early.is_empty() {
                return Err("S3-seq-guard-or-early");
            }
            // **S3 for a `Seq` reads the IL's argument mapping, NOT the emitted
            // setup bytes — and the two disagree on 816 of 816 workload cells.**
            //
            // The clause this lane registered required `setups[0]` to be empty,
            // generalizing the `tail` split (578/578 with no setup, 0/953 with
            // one). It fired on **zero** `seq` bodies, and the refusal census
            // said why: **816 of the 816** single-call `seq` differs are
            // `S3-seq-setup-frame-only` and **none** is `S3-seq-setup-args`.
            //
            // A `Seq` body is FRAMED, so `setups[0]` carries the frame's own
            // bookkeeping — the `mr r31,r3` that saves `this` across the `bl` —
            // as well as any real argument marshalling. c2's inlined body has no
            // frame at all (`w-seq` §4.4: the port emits nine words opening
            // `mflr r12` where c2 emits three and no frame), so a setup that is
            // *only* the save is an artefact of the port's own lowering, while a
            // setup that marshals a value is exactly the register rename or
            // displacement fold that makes SPLICE-0 fail on `tail`.
            //
            // The judge has already graded the widened population: SPLICE-0 is
            // exact on **816 of 816** single-callee `seq` differs (`w-seq`
            // §4.1), which is real c2 on 816 cells rather than a hand cell.
            if !identity_call_args(f, &seq.calls[0]) {
                return Err("S3-seq-setup-args");
            }
            // **S3, the tail half.** The body may do nothing to r3 after the
            // call, because c2's inlined body is the callee's and carries no
            // epilogue of the caller's. Two tails qualify and both are the ABI
            // identity:
            //
            // * `CallValue { add_k: 0 }` — return what the call left in r3,
            //   with the `addi` elided at 0. Class A, so nothing is saved.
            // * `SavedFormal { param: 0 }` — the WEC shape, an empty
            //   constructor delegating to one base: `mr r31,r3 ; bl ?B ;
            //   mr r3,r31`. The formal handed back is the one that was already
            //   in r3 at entry, so with an identity argument mapping the whole
            //   body IS the call. **This is the 634-function `seq` family**, and
            //   its witness is a workload obj rather than a hand cell —
            //   `??0?$_List_iterator@VString@@…` in `CharDriver.cpp` emits
            //   `stw r4,0(r3) ; blr`, word for word its base constructor's
            //   body, where the port's own lowering is three words inside a
            //   96-byte frame (`work/w-splice/caps/chardriver.excerpt.txt`).
            //   GRID-T's `t03` was meant to be the hand cell and is a **dud**:
            //   its base constructor is declared and never defined, so it has no
            //   `Seq` body to grade. Said rather than smoothed.
            //
            // Every other tail — a literal, a read-through, a comparison —
            // emits words of its own after the `bl`, and c2's inlined body would
            // have to carry them too. Not graded, so refused.
            match seq.tail {
                SeqTail::CallValue { add_k: 0 } if seq.saved.is_empty() => {}
                SeqTail::SavedFormal { param: 0 } if seq.saved.as_slice() == [0] => {}
                SeqTail::CallValue { .. } => return Err("S3-seq-tail-callvalue-k"),
                // **W-FLTRET.** The ABI identity holds — the callee leaves the
                // value in `f1` and the caller's return reads `f1` — but splicing
                // the callee's body in removes the `bl`, and whether the
                // caller's TU still owes `_fltused` once its only FP-returning
                // call is gone has **not been captured**. Refused by name rather
                // than folded into the `add_k: 0` arm it otherwise matches: an
                // obj one symbol long is the same defect as one symbol short.
                SeqTail::CallValueFp => return Err("S3-seq-tail-callvaluefp"),
                SeqTail::SavedFormal { .. } => return Err("S3-seq-tail-savedformal-other"),
                SeqTail::Void => return Err("S3-seq-tail-void"),
                SeqTail::Lit(_) => return Err("S3-seq-tail-lit"),
                SeqTail::CallLoad { .. } => return Err("S3-seq-tail-callload"),
                SeqTail::CallLoadFp { .. } => return Err("S3-seq-tail-callloadfp"),
                SeqTail::Cmp { .. } => return Err("S3-seq-tail-cmp"),
            }
            seq.calls[0].callee.as_str()
        }
        Selected::Framed { .. } => return Err("S1-framed"),
        Selected::CondPair(_) => return Err("S1-cond-pair"),
        // `Terminator::None` is the retired `Terminator::None`: a body that owes
        // no branch names no callee, which is this clause's whole content.
        Selected::Body { term: Terminator::None, .. } | Selected::Float { .. } => {
            return Err("S1-no-call")
        }
    };
    // **S4.** Self-recursion. c2 declines it too — `INLINE_PREDICATE.md` §4
    // grades the `recurse` family 336/336 — and a rule that took it would
    // splice a body into itself.
    if callee == f.mangled_name {
        return Err("S4-self-recursion");
    }
    // **S5.** Defined here, once. Returned as the context's own `&'a str` so the
    // spliced body's relocations outlive the caller's borrow.
    //
    // The two ways this fails are two different rungs and are named apart: an
    // **external** callee is nobody's mechanism, and a callee this TU defines
    // whose IL the parser refuses is `w-seq` §5's production table — 1,774
    // differs deep, and the largest single production blocks 573 of them.
    if !tu.mentions(callee) {
        return Err("S5-callee-extern");
    }
    if tu.ambiguous(callee) {
        return Err("S5-callee-ambiguous");
    }
    if tu.definition(callee).is_none() {
        return Err("S6-callee-parse-refused");
    }
    self_named(tu, callee).ok_or("S5-callee-extern")
}

/// **Every callee this body names**, from the decoded fields and never from a
/// byte offset (#644).
///
/// Used by `S6-chain-truncated` to ask the one question that separates a chain
/// that *ended* from one the port could not follow: does the chain's last link
/// still name a function this TU defines?
///
/// Deliberately a local copy of the shape `crates/c2-harness/src/gap/fnbytes.rs`
/// walks for its forensics rather than a shared one: that one is a diagnostic
/// over a *census row* and this is a clause of an emitter's predicate, and the
/// emitter must not depend on the instrument that grades it.
fn port_callees(f: &IlFunction) -> Vec<&str> {
    let mut v: Vec<&str> = Vec::new();
    if let Some(c) = f.tail_call() {
        v.push(c);
    }
    if let Some(fc) = f.framed_call() {
        v.push(fc.callee.as_str());
    }
    if let Some(cs) = f.call_seq() {
        for c in &cs.calls {
            v.push(c.callee.as_str());
        }
    }
    if let Some(cp) = f.cond_pair() {
        v.push(cp.then_arm.callee.as_str());
        v.push(cp.else_arm.callee.as_str());
    }
    v
}

/// **Is this call's argument mapping the IDENTITY** — every argument register
/// already holding, at the call, what it held at function entry?
///
/// Read off the IL and never off the emitted setup bytes, because a `Seq` body's
/// setup also carries the frame's callee-saved bookkeeping, which is the port's
/// lowering and not a transform of the callee's arguments.
///
/// Identity is: no chain link (its receiver is a previous call's result), no
/// slot permutation, and an operand stream that is either empty (a nullary call,
/// or one whose only argument is the implicit `this` already in r3) or a single
/// passthrough `Load` of the **first** formal.
fn identity_call_args(f: &IlFunction, call: &c2_il::SeqCall) -> bool {
    if call.link_args.is_some() || call.arg_slots.is_some() {
        return false;
    }
    match call.arg_ops.as_slice() {
        [] => true,
        [c2_il::IlOp::Load(t)] => f.params.first() == Some(t),
        _ => false,
    }
}

/// **The number of call sites the predicate has already established** for a
/// body it admitted — c2's `n`, the site collector's out-parameter
/// (`0x10b600f7` zeroes it, `0x10b60374` increments it once per site).
///
/// Read off the clause that admitted the body rather than recounted, because
/// `S2` has *already* decided this question and a second count could disagree
/// with the first. A `Tail` body's whole emitted content is one branch, so its
/// site count is 1 by construction; a `Seq`'s is `seq.calls.len()`, which `S2`
/// required to be 1.
///
/// The catch-all arm returns 1 and is unreachable through
/// [`splice_callee_why`]'s `Ok` path — every other `Selected` is refused by
/// name there. It is 1 rather than 0 so that a future arm which *does* reach it
/// takes the identity divisor rather than `budget-no-sites`, which would be a
/// refusal blamed on the wrong clause.
fn predicate_site_count(f: &IlFunction, selected: &Selected) -> i64 {
    match selected {
        Selected::Seq { .. } => f.call_seq().map_or(1, |s| s.calls.len() as i64),
        _ => 1,
    }
}

/// The context's own copy of `name`, with the context's lifetime.
///
/// [`TuContext::definition`] hands back the definition; the *name* has to come
/// from the same place, because a spliced [`ComdatBody`] carries `&str`
/// relocation targets that outlive the caller's borrow of its own callee field.
fn self_named<'a>(tu: &TuContext<'a>, name: &str) -> Option<&'a str> {
    let i = tu.rows.binary_search_by(|(n, _, _)| (*n).cmp(name)).ok()?;
    Some(tu.rows[i].0)
}

/// **THE MECHANISM.** The caller's `/Gy` COMDAT body, when it is the callee's.
///
/// `Ok(None)` is the refusal — the predicate declined, and the caller emits its
/// ordinary body. `Err` is the callee's own decline propagated: it cannot happen
/// through S6, which treats an uncomposable callee as a refusal, and the arm
/// exists so that a future edit which stops doing that goes red rather than
/// silent.
///
/// **The chain is walked, not stepped once.** `splice_body_why` follows it to
/// the first link whose own predicate declines, and takes *that* body — c2
/// closes the chain and `t11` plus 150 workload relocation witnesses say so
/// ("THE FIXPOINT" above). The composition of that final body is asked with
/// `allow_splice: false`, because the walk has already established that this
/// link does not splice and asking again through the composition would be the
/// same question with a second implementation.
pub fn splice_body<'a>(
    f: &IlFunction,
    selected: &Selected,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> Result<Option<ComdatBody<'a>>, ComdatDecline> {
    match splice_body_why(f, selected, mode, tu) {
        Ok(b) => Ok(Some(b)),
        Err(SpliceDecline::Refused(_)) => Ok(None),
        Err(SpliceDecline::Callee(d)) => Err(d),
    }
}

/// Why the mechanism did not fire, or the callee's own decline propagated.
///
/// [`SpliceDecline::Callee`] cannot happen through S6, which treats an
/// uncomposable callee as a refusal; the variant exists so that a future edit
/// which stops doing that goes red rather than silent.
#[derive(Debug)]
pub enum SpliceDecline {
    /// A clause of the predicate declined, named by its clause number.
    Refused(&'static str),
    /// The callee's own composition failed.
    Callee(ComdatDecline),
}

/// [`splice_body`] with **the clause that refused**, for the same reason
/// [`splice_callee_why`] exists: the refusal reason is what prices the next
/// widening, so there is one implementation of it and not two.
pub fn splice_body_why<'a>(
    f: &IlFunction,
    selected: &Selected,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> Result<ComdatBody<'a>, SpliceDecline> {
    let mut callee = splice_callee_why(f, selected, tu).map_err(SpliceDecline::Refused)?;
    // **THE CHAIN.** `S6-chain` below walks it; `seen` is what makes a cycle
    // terminate, and the ceiling is what makes a broken edit terminate too.
    let mut seen: Vec<&str> = vec![callee];
    let ceiling = tu.definitions() as i64 + 2;
    // **THE BUDGET MODEL, entered.** `P_INLINE.md` §6.6.2 and this file's
    // "REGION `w-inlbudget`". c2 re-enters its whole decision at each level; the
    // port's walk is the same recursion with `n = 1` at every step, and
    // [`port_enter_site`] is what makes that a *derivation* instead of the
    // coincidence `w-inlfit` found. It refuses `n ≥ 2` — unreachable from any
    // fixture, because `S2` already refused a two-call body, and registered as a
    // decision surface for exactly that reason (board `#3723`).
    let model = &BUDGET_MODELS[0];
    // **THE SEED, and it is the whole of lane `w-budget`.** c2's pass entry
    // loads the CALLER's own count and makes two things out of it — the running
    // growth total (C2, `0x10b62703`) and the budget (C3, `0x10b62708`). The
    // port now has that count: it is the `.gl` `SIZE` field it already decoded
    // and used to discard. When it is unreadable the entry state is the one
    // this walk always had, and every arm keyed on a number stays inert.
    let mut exp = match tu.instr_count(&f.mangled_name) {
        Some(c) => Expansion::at_pass_entry_seeded(model, c),
        None => Expansion::at_pass_entry(),
    };
    exp = port_enter_site(
        model,
        exp,
        predicate_site_count(f, selected),
        0,
        tu.instr_count(callee),
    )
    .map_err(SpliceDecline::Refused)?;
    loop {
        // **S7, the varargs half.** `N_max = 0` categorically (§6.18.5). MSVC
        // terminates a varargs argument list with `Z` where an ordinary one ends
        // `@`, so the mangled name ends `ZZ` — read off the name because that is
        // where §2's table says it is readable, IL side and obj side alike.
        if callee.ends_with("ZZ") {
            return Err(SpliceDecline::Refused("S7-varargs"));
        }
        let Some((g, opt_word)) = tu.definition(callee) else {
            return Err(SpliceDecline::Refused("S5-callee-extern"));
        };
        // **S7-noinline — `__declspec(noinline)`, read off the `.gl`.**
        //
        // `crates/c2-harness/tests/noinline_boundary.rs` cell `w10` is a
        // **shipped, demonstrated wrong emit**: the splice puts the callee's
        // body where c2 emits `b ?g`, and that file's own note says the port
        // *"cannot read the attribute"*. It can now —
        // `c2_il::func::gl::FN_FLAG_INLINABLE`, board **#1039**'s undecoded
        // field — so the splice stops expanding what c2 keeps a call to.
        //
        // Asked **per link and inside the loop**, not once before it: `S6-chain`
        // steps `callee` down the chain and every intermediate is a body the
        // splice would take, so an attribute on link three has to refuse at link
        // three. Placed after `tu.definition` because the attribute belongs to a
        // function this TU DEFINES; an external's is unreadable and unneeded,
        // since `S5-callee-extern` has already refused it.
        //
        // `None` (unasked) and `Some(true)` behave exactly as before.
        if g.inlinable == Some(false) {
            return Err(SpliceDecline::Refused("S7-callee-noinline"));
        }
        // The callee's own mode. A callee under a different `#pragma optimize`
        // than its caller allocates a chain intermediate to a different register
        // (`OptMode`'s doc), so splicing across that boundary would emit the
        // wrong register field. `None` is "the caller does not track it per
        // function", and then the TU has already been refused unless every
        // function shares a mode.
        let g_mode = match opt_word {
            Some(_) => match opt_mode_of_word(opt_word) {
                Ok(m) => m,
                Err(_) => return Err(SpliceDecline::Refused("S6-callee-opt-mode")),
            },
            None => mode,
        };
        if g_mode != mode {
            return Err(SpliceDecline::Refused("S6-mode-mismatch"));
        }
        // **S6.** The port must have a body for the callee — `t09` is the cell
        // where it does not.
        let Ok(g_sel) = crate::codegen::select_function(g, g_mode) else {
            return Err(SpliceDecline::Refused("S6-callee-refused"));
        };
        // **S6-CHAIN — THE FIXPOINT, and it is measured rather than assumed.**
        //
        // If the callee ITSELF splices, its own emitted COMDAT is not the branch
        // the port would lower it to: it is *its* callee's body. So the caller
        // must take that one, and the walk steps down.
        //
        // `work/w-splice/PREREG.md` §4 registered this as a QUESTION with the
        // port taking one level either way. Both answers came back and they
        // agree:
        //
        // * `t11` — `int h(int a){return a+1;} int g(int a){return h(a);}
        //   int f(int a){return g(a);}` — c2 emits **`?h`'s two words for all
        //   three**, so `splice0(?f)` against `ref(?g)` grades `exact`;
        // * the workload, through the relocation check the one-level rule
        //   forced: **150 of 945** spliced functions named the chain's
        //   *intermediate* where c2 named its *end* —
        //   `??1length_error@stlpmtx_std@@` relocating against
        //   `??1__Named_exception@stlpmtx_std@@` where c2 relocates against
        //   `??1exception@std@@`, 145 times in that shape. Every one of the 150
        //   was this and none was a different target.
        //
        // The one-level rule therefore could not ship: `PREREG.md` §3 item 4
        // makes a relocation disagreement a decline-floor failure, and 150 of
        // them is not a rounding.
        if let Ok(next) = splice_callee_why(g, &g_sel, tu) {
            if seen.contains(&next) {
                // A cycle. `elide.rs`'s least fixpoint never *seeds* one and so
                // never admits one; this walk reaches the same answer by
                // construction, and c2 declines a recursive callee too
                // (`INLINE_PREDICATE.md` §4, `recurse` 336/336).
                return Err(SpliceDecline::Refused("S6-chain-cycle"));
            }
            // **The chain steps down: one more of c2's expansions.** The level
            // advances by `BYTE [site+0x18]`, which this lane read as `1`
            // (`0x10b602ce`), so `exp.level` counts the walk's depth in exactly
            // c2's units.
            exp = port_enter_site(
                model,
                exp,
                predicate_site_count(g, &g_sel),
                0,
                // The count of the callee this step ENTERS — `next`, not `g`.
                // `g` is the link already entered and already charged.
                tu.instr_count(next),
            )
            .map_err(SpliceDecline::Refused)?;
            if exp.level > ceiling {
                // Unreachable while `seen` is checked above — a step either
                // repeats a name or admits a new one, and the bundle has
                // finitely many. Here so that an edit which breaks that
                // argument REFUSES instead of walking forever.
                //
                // **Re-expressed through the budget model** (`w-inlbudget`,
                // construct rung): this was `seen.len() > tu.definitions() + 1`
                // and is now the same bound on `exp.level`, which starts at 1
                // and advances by one per expansion — hence the `+ 2`. The
                // guard is unreachable before and after, which is what makes it
                // a safe thing to re-express; what it buys is that the walk's
                // depth is now c2's `level` and not a private counter.
                return Err(SpliceDecline::Refused("S6-chain-ceiling"));
            }
            seen.push(next);
            callee = next;
            continue;
        }
        // **S6-CHAIN-TRUNCATED — the chain must END, not be CUT OFF.**
        //
        // The walk stops at the first link whose own predicate declines. There
        // are two completely different reasons that happens and only one of them
        // is an ending:
        //
        // * the link names **no same-TU callee** — an external, or nothing at
        //   all. c2 has no body to expand either, so this is where c2 stops too
        //   and the body is the answer;
        // * the link names a callee this TU **defines** and the port cannot
        //   follow — its IL is parse-refused, it has an argument setup, it is a
        //   multi-call body. c2 is under no such restriction, and **it does not
        //   stop there**: measured, 72 spliced functions relocated against
        //   `??1?$_List_base@…` where c2 relocates against
        //   `?clear@?$_List_base@…`, which is one link further down a chain the
        //   port cannot read. Every one of the 77 residual disagreements after
        //   the fixpoint landed was this.
        //
        // So a truncated chain **refuses**. The port does not know where c2
        // stopped, and a relocation against the wrong symbol is invisible to
        // FUNCTION BYTE MATCH — board **#882** — which makes guessing here the
        // one thing this lane must not do.
        if port_callees(g).into_iter().any(|c| tu.mentions(c)) {
            return Err(SpliceDecline::Refused("S6-chain-truncated"));
        }
        // The chain's end: this callee emits its own body, so that body is the
        // caller's. Composed with `allow_splice: false` — the walk above has
        // already established that this link does not splice, and asking again
        // through the composition would be the same question with a second
        // implementation.
        let Ok(body) = crate::comdat::body_of(g, g_sel, g_mode, tu, false) else {
            return Err(SpliceDecline::Refused("S6-callee-no-compose"));
        };
        // **S6, the frame half.** A framed callee carries a prologue, an
        // epilogue and a `.pdata` record whose association is a property of the
        // function it belongs to. Splicing one into a caller is a cell nobody
        // graded.
        if body.frame.is_some() {
            return Err(SpliceDecline::Refused("S6-callee-framed"));
        }
        // **S6-CHAIN-OPEN — the chain's end may carry NO CALL AT ALL, and this
        // clause is a measured retreat.**
        //
        // `S6-chain-truncated` above asks whether the end still names a callee
        // *this TU's census carries*. One workload function got past it:
        // `??1CharPollableSorter@@QAA@XZ` spliced to a body whose branch names
        // `??1?$_Rb_tree@PAVObject@Hmx@@…`, and c2 relocates against
        // `?clear@?$_Rb_tree@…` one link further down. That callee has **no
        // census row in its TU at all** — `c2rs census` on `Character.cpp`
        // prints zero rows matching it — so it sits in the `unbound` population
        // (9,225 rows of this workload) and the port cannot tell it from a
        // genuine external. c2 can: it inlined it, which is proof it is defined
        // in that obj.
        //
        // So a chain that ends *in a call* is refused. It costs **4** spliced
        // functions of 727 — **3 whose relocation the check verified CORRECT**
        // (`t10`'s shape: the callee really does call an external, and the port
        // really does name it) and **1 verified wrong**. Keeping the three would
        // mean keeping the one, because nothing the port can read separates
        // them, and a wrong relocation under byte-exact text is exactly board
        // **#882**'s 4,664.
        //
        // The rung that recovers the three is named and not taken: give the port
        // a defined-name set that does not come from the census — the `.gl`
        // knows which names this TU defines — and then re-grade the one.
        if !body.calls.is_empty() {
            return Err(SpliceDecline::Refused("S6-chain-open"));
        }
        // **S7.** The inline decision, on the side of its own boundary where it
        // is categorical in both linkage classes.
        //
        // **One check covers every link of the chain.** `INLINE-P`'s `s` is the
        // callee's own *emitted* size, and an intermediate that splices emits
        // exactly this body — c2's COMDAT for it *is* the chain's end body — so
        // `s` is the same number at every edge and testing it once tests it
        // everywhere.
        if body.text.is_empty() {
            return Err(SpliceDecline::Refused("S7-callee-empty-text"));
        }
        if body.text.len() > INLINE_UNBOUNDED_BYTES {
            return Err(SpliceDecline::Refused("S7-callee-over-64B"));
        }
        return Ok(body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::select_function;
    use crate::codegen::testutil::func_with;
    use c2_il::{CallSeq, IlOp, SeqCall};

    /// `int g(int a) { return a + 1; }` — a leaf the port lowers, 8 bytes.
    fn leaf(name: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add]);
        f.mangled_name = name.into();
        f.data_syms.clear();
        f
    }

    /// `int f(int a) { return g(a); }` — an empty setup and one branch word.
    fn tail(name: &str, callee: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], Vec::new());
        f.mangled_name = name.into();
        f.data_syms.clear();
        f.body = c2_il::BodyShape::Tail(callee.into());
        f
    }

    fn ctx<'a>(funcs: &'a [IlFunction]) -> TuContext<'a> {
        TuContext::of(funcs)
    }

    fn fires(funcs: &[IlFunction], i: usize) -> bool {
        let tu = ctx(funcs);
        let sel = select_function(&funcs[i], OptMode::O1).unwrap();
        splice_callee(&funcs[i], &sel, &tu).is_some()
    }

    fn spliced(funcs: &[IlFunction], i: usize) -> Option<Vec<u8>> {
        let tu = ctx(funcs);
        let sel = select_function(&funcs[i], OptMode::O1).unwrap();
        splice_body(&funcs[i], &sel, OptMode::O1, &tu)
            .unwrap()
            .map(|b| b.text)
    }

    /// `t01` — the positive cell. The caller's body **is** the callee's, and the
    /// caller acquires no relocation of its own.
    #[test]
    fn an_empty_setup_tail_call_takes_the_callees_body() {
        let funcs = vec![leaf("?g@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        let body = splice_body(&funcs[1], &sel, OptMode::O1, &tu)
            .unwrap()
            .expect("S1-S9 hold: this is t01");
        let g_sel = select_function(&funcs[0], OptMode::O1).unwrap();
        let g_body = crate::comdat::comdat_body_from_selected(&funcs[0], g_sel, OptMode::O1, &tu)
            .unwrap();
        assert_eq!(body.text, g_body.text, "the caller's body is the callee's");
        assert!(
            body.calls.is_empty(),
            "THE CALLER ACQUIRED A RELOCATION AGAINST ITS CALLEE: the whole \
             mechanism is that it does not — board #882, and a leaf callee has \
             no relocation of its own to inherit"
        );
        assert!(body.frame.is_none());
    }

    /// **S3** — `t04`/`t05`/`t06`. A setup means c2 rewrote a field of the
    /// callee's body (a register rename, a displacement fold), and w-seq graded
    /// the concatenation **0 of 953** there.
    #[test]
    fn a_tail_call_with_an_argument_setup_is_refused() {
        // `int f(int a) { return g(a + 1); }` — the setup is one `addi`.
        let mut caller = tail("?f@@YAHH@Z", "?g@@YAHH@Z");
        caller.ops = vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add];
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        assert!(
            !fires(&funcs, 1),
            "SPLICE-P IS 0 OF 953 WITH A SETUP: 1,890 of those failures diverge \
             at word 0, which is the port's own first setup word"
        );
    }

    /// **S5** — `t14`, the control. A callee this bundle does not define is the
    /// ordinary tail call, relocation and all.
    #[test]
    fn a_callee_this_bundle_does_not_define_is_refused() {
        let funcs = vec![tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 0));
        // …and a definition of a DIFFERENT name must not admit it either.
        let funcs = vec![leaf("?other@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 1));
    }

    /// **S5, the ambiguity half.** Two definitions of one name admit neither:
    /// telling them apart needs something this context does not have.
    #[test]
    fn a_name_defined_twice_is_refused() {
        let mut g2 = leaf("?g@@YAHH@Z");
        g2.ops = vec![IlOp::Load(0xE309), IlOp::Lit(2), IlOp::Add];
        let funcs = vec![leaf("?g@@YAHH@Z"), g2, tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 2));
    }

    /// **S4** — `t12`. A body spliced into itself is not a fixpoint, it is a
    /// loop; c2 declines self-recursion too (`recurse`, 336/336).
    #[test]
    fn self_recursion_is_refused() {
        let funcs = vec![tail("?r@@YAHH@Z", "?r@@YAHH@Z")];
        assert!(!fires(&funcs, 0));
    }

    /// **S9** — mechanism E answers first. An empty-bodied callee is `elide`'s
    /// and this rule must not claim it, or one function has two rules.
    #[test]
    fn mechanism_e_keeps_its_own_callers() {
        let mut g = leaf("?g@@YAXXZ");
        g.ops = Vec::new();
        g.params = Vec::new();
        g.body = c2_il::BodyShape::EmptyBody;
        let mut caller = tail("?f@@YAXXZ", "?g@@YAXXZ");
        caller.params = Vec::new();
        let funcs = vec![g, caller];
        let tu = ctx(&funcs);
        assert!(
            tu.reduces_to_nothing("?g@@YAXXZ"),
            "the E context still resolves through the composite"
        );
        assert!(
            !fires(&funcs, 1),
            "MECHANISM E AND MECHANISM I BOTH CLAIMED ONE FUNCTION: E is asked \
             first and keeps its blr"
        );
    }

    /// **S8** — a caller that materializes a data symbol. Its whole body is
    /// discarded and the data reference would go with it.
    #[test]
    fn a_caller_with_a_data_symbol_is_refused() {
        let mut caller = tail("?f@@YAHH@Z", "?g@@YAHH@Z");
        caller.data_syms = vec!["?gv@@3HA".into()];
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        assert!(!fires(&funcs, 1));
    }

    /// **S7, the varargs half.** `N_max = 0` categorically, and the mangled name
    /// is where §2's table says the bit is readable.
    #[test]
    fn a_varargs_callee_is_refused() {
        let mut g = leaf("?g@@YAHHZZ");
        g.mangled_name = "?g@@YAHHZZ".into();
        let funcs = vec![g, tail("?f@@YAHH@Z", "?g@@YAHHZZ")];
        // S1-S5 hold — the refusal is S7's and only S7's.
        assert!(fires(&funcs, 1), "the structural half of the predicate holds");
        assert!(
            spliced(&funcs, 1).is_none(),
            "A VARARGS CALLEE WAS SPLICED: N_max is 0 for it in both linkage \
             classes (INLINE_PREDICATE.md §2, §6.18.5)"
        );
    }

    /// **S6** — a callee the port cannot lower has no body to splice, so the
    /// caller keeps its branch. The structural half still holds, which is why
    /// the two halves are separate functions.
    #[test]
    fn an_unlowerable_callee_leaves_the_branch_alone() {
        let mut g = leaf("?g@@YAHH@Z");
        g.ops = vec![IlOp::Load(0xE309), IlOp::Load(0xE409), IlOp::Div];
        let funcs = vec![g, tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(fires(&funcs, 1));
        assert!(
            spliced(&funcs, 1).is_none(),
            "the port has no bytes for this callee; splicing would emit a body \
             it cannot produce"
        );
    }

    /// **BOARD #980's BOUNDARY — a `NoEffectCall` row is E's and NEVER the
    /// splice's.**
    ///
    /// Lane `w-inl0` feeds mechanism E edges from rows the IL parser **refused**:
    /// a refused body whose grammar still proves it emits nothing but a call to
    /// one callee contributes `Reduction::NoEffectCall`. E can close through
    /// such a node because closing needs only *"does this emit anything"*.
    ///
    /// The splice cannot, and the reason is S6 rather than a policy: its rule is
    /// *"the caller's body IS the callee's body"*, and there is no body — the
    /// parser refused it. So [`TuContext::definition`] returns `None` for those
    /// rows and the walk refuses with `S6-callee-parse-refused`.
    ///
    /// **But the row must still be VISIBLE.** `?g` is a name this TU defines,
    /// and `mentions` says so, which is what `S6-chain-truncated` reads to tell
    /// a chain that *ended* from one the port could not *follow*. A resolution
    /// that dropped refused rows from the context would make `?g` read as an
    /// external — and running a splice off the end of a chain the port cannot
    /// see is the wrong-relocation defect this lane already closed once.
    #[test]
    fn a_no_effect_call_row_feeds_e_and_never_the_splice() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.body = c2_il::BodyShape::EmptyBody;
        let mut caller = tail("?f@@YAXXZ", "?g@@YAXXZ");
        caller.params = Vec::new();
        // `?g` is REFUSED by the parser — there is no `IlFunction` for it at
        // all — and carries only the edge board #980 reads out of its grammar.
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", Some(Reduction::NoEffectCall("?h@@YAXXZ")), None),
            ("?f@@YAXXZ", Some(Reduction::Parsed(&caller)), None),
        ]);

        // E closes through the refused node — that is #980's whole content.
        assert!(
            tu.reduces_to_nothing("?g@@YAXXZ"),
            "BOARD #980 REGRESSED: mechanism E must close through a refused row \
             that carries a NoEffectCall edge"
        );

        // The splice must not be able to reach it as a body...
        assert!(
            tu.definition("?g@@YAXXZ").is_none(),
            "A REFUSED BODY WAS OFFERED TO THE SPLICE: S6 needs a COMPOSED body \
             for the chain's end and the parser refused this one"
        );
        // ...and must still be CARRIED by the context. `mentions()` is what
        // reads this row, and it arrives with `S6-chain-truncated`; what this
        // commit can assert is that the row is in the table at all, which is
        // the property that clause will depend on.
        assert!(
            tu.mentions("?g@@YAXXZ"),
            "A REFUSED ROW VANISHED FROM THE CONTEXT: it reads as an external, \
             S6-chain-truncated stops firing, and the splice runs off the end \
             of a chain it cannot see"
        );
        assert_eq!(
            tu.definitions(),
            3,
            "every row this TU binds stays in the table, parsed or not"
        );

        // And the two mechanisms still do not both claim `?f`: E takes it.
        let sel = select_function(&caller, OptMode::O1).unwrap();
        assert!(
            crate::elide::drops_tail_call(&caller, tu.empty_callees()),
            "?f tail-calls a name that reduces to nothing, so E answers"
        );
        assert!(
            splice_callee(&caller, &sel, &tu).is_none(),
            "S9: mechanism E is asked first and keeps its blr"
        );
    }

    /// A refused row with **no** readable `no_effect_callee` feeds neither
    /// mechanism and is still visible. That is the third value of the
    /// constructor's `Option<Reduction>`, and it is the majority of the refused
    /// population.
    #[test]
    fn a_refused_row_with_no_edge_is_visible_and_nothing_else() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.body = c2_il::BodyShape::EmptyBody;
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", None, None),
        ]);
        assert!(tu.mentions("?g@@YAXXZ"), "still a name this TU defines");
        assert_eq!(tu.definitions(), 2, "and still a row in the table");
        assert!(tu.definition("?g@@YAXXZ").is_none());
        assert!(
            !tu.reduces_to_nothing("?g@@YAXXZ"),
            "#980 IS CONSERVATIVE: a refused row with nothing readable \
             contributes NO edge to the closure"
        );
    }

    /// **THE `Deref` SHADOWING TRAP, pinned.**
    ///
    /// `TuContext` derefs to [`TuEmptyCallees`] so existing callers keep
    /// working, and an inherent method on the wrapper therefore **silently
    /// overrides** the target's. While this type spelled its row count `len`,
    /// the scan's `fnbyte-tu-empty-callees` reported the wrong quantity —
    /// 88,894 against 1,474,755 on the dc3 workload — with no compile error and
    /// no test failure. It also made one of the two tests above pass by
    /// coincidence, because that cell's E context happens to have as many
    /// members as the table has rows.
    ///
    /// So the two counts are asserted to be **different** on a bundle where
    /// they must be, which is a thing no rename can quietly undo.
    #[test]
    fn the_row_count_and_the_e_context_are_not_the_same_number() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.body = c2_il::BodyShape::EmptyBody;
        let g = leaf("?g@@YAHH@Z"); // parses, non-empty: a row, never in E
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAHH@Z", Some(Reduction::Parsed(&g)), None),
            ("?x@@YAXXZ", None, None), // defined here, parser refused it
        ]);
        assert_eq!(tu.definitions(), 3, "three rows this TU binds");
        assert_eq!(tu.empty_callees().len(), 1, "only ?h reduces to nothing");
        assert_ne!(
            tu.definitions(),
            tu.empty_callees().len(),
            "A ROW COUNT WAS READ AS THE E-CONTEXT SIZE (or the reverse): these \
             are different facts, `TuContext` Derefs to `TuEmptyCallees`, and an \
             inherent `len` here would shadow the target's and move an existing \
             scan key by 16x without failing anything"
        );
    }

    /// **S2** — `t08`. Two call sites is SPLICE-N, **0 of 548**.
    #[test]
    fn a_two_call_body_is_refused() {
        let mut caller = func_with(Vec::new(), Vec::new());
        caller.mangled_name = "?f@@YAXXZ".into();
        caller.data_syms.clear();
        caller.body = c2_il::BodyShape::Seq(CallSeq {
            calls: vec![
                SeqCall {
                    callee: "?g@@YAXXZ".into(),
                    arg_ops: Vec::new(),
                    arg_slots: None,
                    link_args: None,
                },
                SeqCall {
                    callee: "?g@@YAXXZ".into(),
                    arg_ops: Vec::new(),
                    arg_slots: None,
                    link_args: None,
                },
            ],
            tail: SeqTail::Void,
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
            store_run: None,
        });
        let mut g = leaf("?g@@YAXXZ");
        g.params = Vec::new();
        g.ops = Vec::new();
        g.body = c2_il::BodyShape::Plain;
        let funcs = vec![g, caller];
        let tu = ctx(&funcs);
        if let Ok(sel) = select_function(&funcs[1], OptMode::O1) {
            assert!(
                splice_callee(&funcs[1], &sel, &tu).is_none(),
                "SPLICE-N IS 0 OF 548: a two-call body is not the concatenation \
                 of its callees and it is not either of them"
            );
        }
    }

    /// **S3, the `Seq` tail half.** A `Seq` whose tail does work of its own
    /// after the `bl` is refused: c2's inlined body would have to carry those
    /// words too, and no cell graded that.
    #[test]
    fn a_seq_with_a_working_tail_is_refused() {
        let mut caller = func_with(vec![0xE309], Vec::new());
        caller.mangled_name = "?f@@YAHH@Z".into();
        caller.data_syms.clear();
        caller.body = c2_il::BodyShape::Seq(CallSeq {
            calls: vec![SeqCall {
                callee: "?g@@YAHH@Z".into(),
                arg_ops: Vec::new(),
                arg_slots: None,
                link_args: None,
            }],
            // `return g(a) + 5;` — one `addi` after the call.
            tail: SeqTail::CallValue { add_k: 5 },
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
            store_run: None,
        });
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        let tu = ctx(&funcs);
        if let Ok(sel) = select_function(&funcs[1], OptMode::O1) {
            assert!(splice_callee(&funcs[1], &sel, &tu).is_none());
        }
    }

    /// **THE FIXPOINT** — `t11`, and the 150 relocation witnesses that forced
    /// it. `h` is lowerable, `g` splices `h`, and `f` must get **`h`'s body**
    /// and not `g`'s one branch word.
    ///
    /// c2 closes the chain: `t11` compiles this exact source and c2 emits `?h`'s
    /// two words for all three functions. The one-level rule that shipped first
    /// named the intermediate in **150 of 945** spliced functions' relocations
    /// where c2 named the end.
    #[test]
    fn the_splice_closes_the_chain() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g@@YAHH@Z"),
        ];
        let h = spliced(&funcs, 1).expect("?g splices ?h");
        let f = spliced(&funcs, 2).expect("?f splices through ?g to ?h");
        assert_eq!(h.len(), 8, "?h is `addi r3,r3,1 ; blr`");
        assert_eq!(
            f, h,
            "THE CHAIN WAS NOT CLOSED: ?f must get ?h's BODY, not ?g's branch \
             word. GRID-T t11 grades c2 emitting ?h's two words for all three, \
             and the workload's relocation check found the one-level rule \
             naming the intermediate 150 times"
        );
    }

    /// A chain of four, and every member below the top gets the same body. The
    /// depth is not special-cased anywhere, which is the property this pins.
    #[test]
    fn the_chain_closes_at_every_depth() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g3@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?g2@@YAHH@Z", "?g3@@YAHH@Z"),
            tail("?g1@@YAHH@Z", "?g2@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g1@@YAHH@Z"),
        ];
        let want = spliced(&funcs, 1).expect("?g3 splices ?h");
        for i in 2..funcs.len() {
            assert_eq!(
                spliced(&funcs, i).as_ref(),
                Some(&want),
                "{} did not reach the chain's end",
                funcs[i].mangled_name
            );
        }
    }

    /// **A CYCLE TERMINATES AND REFUSES.** `?a` splices `?b` splices `?a`: the
    /// walk repeats a name, and a repeated name is the refusal rather than a
    /// deeper step. `elide.rs`'s least fixpoint never seeds a cycle and never
    /// admits one; this reaches the same answer from the other direction.
    #[test]
    fn a_chain_cycle_terminates_and_refuses() {
        let funcs = vec![
            tail("?a@@YAHH@Z", "?b@@YAHH@Z"),
            tail("?b@@YAHH@Z", "?a@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?a@@YAHH@Z"),
        ];
        for i in 0..funcs.len() {
            assert!(
                spliced(&funcs, i).is_none(),
                "A CYCLE WAS SPLICED: the walk must refuse, not recurse — {}",
                funcs[i].mangled_name
            );
        }
    }

    /// **S6-CHAIN-OPEN** — a chain whose end still calls something is refused,
    /// and this is the cell that used to be `t10`'s positive.
    ///
    /// `void ext(); void g(){ext();} void f(){g();}` — c2 emits `b ?ext` for
    /// `?f` and the port would emit `b ?ext` too, which is RIGHT. It is refused
    /// anyway: on the workload the identical shape produced 3 verified-correct
    /// relocations and **1 verified wrong** one, and nothing the port can read
    /// separates them — the wrong one's next link has no census row in its TU,
    /// so it reads as an external and is not. Board #882 is a wrong relocation
    /// under byte-exact text, and 3 right is not worth 1 wrong here.
    #[test]
    fn a_chain_that_ends_in_a_call_is_refused() {
        let funcs = vec![
            tail("?g@@YAXXZ", "?ext@@YAXXZ"),
            tail("?f@@YAXXZ", "?g@@YAXXZ"),
        ];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        // The structural half holds — the refusal is S6-chain-open's alone.
        assert!(splice_callee(&funcs[1], &sel, &tu).is_some());
        assert!(
            splice_body(&funcs[1], &sel, OptMode::O1, &tu)
                .unwrap()
                .is_none(),
            "A CHAIN ENDING IN A CALL WAS SPLICED: the port cannot tell that \
             callee from one this TU defines whose census row is unbound, and \
             the workload has one of each"
        );
    }

    /// …and the property that clause protects, stated where a test can hold it:
    /// **a spliced body never relocates against its own callee.** Every body the
    /// rule now emits carries the chain end's relocations, and the chain end has
    /// none.
    #[test]
    fn a_spliced_body_never_relocates_against_its_callee() {
        let funcs = vec![leaf("?g@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        let body = splice_body(&funcs[1], &sel, OptMode::O1, &tu)
            .unwrap()
            .expect("t01's shape");
        assert!(
            body.calls.is_empty(),
            "BOARD #882: the caller must acquire no REL24 at all here — not one \
             against ?g, which is the relocation the ordinary Tail arm emits and \
             the one c2 does not"
        );
        assert!(body.data_refs.is_empty());
    }

    /// A name defined once resolves; the ambiguity guard must not reject the
    /// *first* or *last* row of the sorted table by walking off it.
    #[test]
    fn definition_lookup_handles_the_table_edges() {
        let funcs = vec![
            leaf("?a@@YAHH@Z"),
            leaf("?m@@YAHH@Z"),
            leaf("?z@@YAHH@Z"),
        ];
        let tu = ctx(&funcs);
        for n in ["?a@@YAHH@Z", "?m@@YAHH@Z", "?z@@YAHH@Z"] {
            assert!(tu.definition(n).is_some(), "{n} must resolve");
        }
        assert!(tu.definition("?nope@@YAHH@Z").is_none());
    }

    /// An unnamed definition cannot be a callee — the same answer `elide` gives.
    #[test]
    fn an_unnamed_definition_is_never_a_splice_source() {
        let mut g = leaf("");
        g.mangled_name = String::new();
        let funcs = [g];
        let tu = TuContext::of(&funcs);
        assert_eq!(tu.definitions(), 0);
        assert!(tu.definition("").is_none());
    }

    /// **`S7-callee-noinline` — the shipped wrong emit, closed.**
    ///
    /// `crates/c2-harness/tests/noinline_boundary.rs` cell `w10` is
    /// `__declspec(noinline) int g(int a){return a+1;} int f(int a){return g(a);}`
    /// and it records what the port does today: the splice puts `?g`'s body into
    /// `?f` where c2 emits `b ?g`. That file's note says the port *"cannot read
    /// the attribute"*; `c2_il::func::gl::FN_FLAG_INLINABLE` is the attribute,
    /// and this is the refusal it buys.
    ///
    /// The control is the same TU with the flag left `None`, which still
    /// splices — so the cell measures the attribute and not the shape.
    #[test]
    fn a_noinline_callee_is_not_spliced() {
        let g = leaf("?g@@YAHH@Z");
        let caller = tail("?f@@YAHH@Z", "?g@@YAHH@Z");
        let sel = select_function(&caller, OptMode::O1).expect("the caller lowers");

        let tu_ok = TuContext::of_rows(vec![("?g@@YAHH@Z", Some(Reduction::Parsed(&g)), None)]);
        assert!(
            splice_body_why(&caller, &sel, OptMode::O1, &tu_ok).is_ok(),
            "the CONTROL must splice, or the cell below is measuring the shape"
        );

        let mut g_ni = leaf("?g@@YAHH@Z");
        g_ni.inlinable = Some(false);
        let tu_ni =
            TuContext::of_rows(vec![("?g@@YAHH@Z", Some(Reduction::Parsed(&g_ni)), None)]);
        assert!(
            matches!(
                splice_body_why(&caller, &sel, OptMode::O1, &tu_ni),
                Err(SpliceDecline::Refused("S7-callee-noinline"))
            ),
            "c2 emits `b ?g` here; the port must not emit ?g's body — and the \
             clause KEY is asserted, not merely that something refused, because \
             a cell that passes on the wrong clause is a cell that tests nothing"
        );

        // `Some(true)` is a positive permission and `None` is UNASKED, and
        // neither may move the splice — the must-fail half of the pair.
        for flag in [None, Some(true)] {
            let mut g2 = leaf("?g@@YAHH@Z");
            g2.inlinable = flag;
            let tu = TuContext::of_rows(vec![("?g@@YAHH@Z", Some(Reduction::Parsed(&g2)), None)]);
            assert!(
                splice_body_why(&caller, &sel, OptMode::O1, &tu).is_ok(),
                "inlinable = {flag:?} must leave the splice exactly where it was"
            );
        }
    }

    // -- REGION `w-inlbudget` TESTS ----------------------------------------

    /// **THE PIN** — every non-default [`BUDGET_MODELS`] entry is an instrument
    /// state and licenses no emit, so none of them may be named outside this
    /// module. `codegen::regalloc`'s cost fence, one seam over, and for the
    /// same reason: the decision-surface clause makes the *default*
    /// reproduce c2 byte-exactly and says nothing at all about the others.
    ///
    /// The production call site is asserted by count as well as by name,
    /// because a second consumer is exactly the thing this must not acquire
    /// quietly.
    #[test]
    fn the_only_budget_model_on_a_production_path_is_the_default() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![root.clone()];
        let mut offenders: Vec<String> = Vec::new();
        let mut call_sites: Vec<String> = Vec::new();
        let mut scanned = 0usize;
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).expect("src readable").flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().is_none_or(|x| x != "rs") {
                    continue;
                }
                let rel = p.strip_prefix(&root).unwrap().to_string_lossy().into_owned();
                scanned += 1;
                // This module defines them; `surface.rs` may not even name them.
                let is_this_module = rel.replace('\\', "/") == "splice.rs";
                for (n, line) in std::fs::read_to_string(&p).unwrap().lines().enumerate() {
                    let t = line.trim();
                    if t.starts_with("//") {
                        continue;
                    }
                    if t.contains("port_enter_site(") && !is_this_module {
                        call_sites.push(format!("{rel}:{}", n + 1));
                    }
                    if is_this_module {
                        continue;
                    }
                    for m in [
                        "BUDGET_UNDIVIDED",
                        "BUDGET_CHARGE_FORCEINLINE",
                        "BUDGET_FLAT_LEVEL",
                        "BUDGET_MAXLEVEL_2",
                    ] {
                        if t.contains(m) {
                            offenders.push(format!("{rel}:{}: {t}", n + 1));
                        }
                    }
                }
            }
        }
        assert!(scanned > 50, "the source scan found only {scanned} files — it is not reading the crate");
        assert!(
            offenders.is_empty(),
            "A NON-DEFAULT BUDGET MODEL REACHED A PRODUCTION PATH. Every entry \
             of BUDGET_MODELS except index 0 is an instrument state and \
             licenses no emit (rungs/README.md's decision-surface clause): \
             {offenders:?}"
        );
        assert!(
            call_sites.is_empty(),
            "port_enter_site acquired a consumer outside splice.rs: {call_sites:?}"
        );
    }

    /// **`#1020`'s HAZARD, EXECUTED.** `w-inlfit`'s words: *"the moment a lane
    /// widens `S2` to two call sites, `n = 2`, c2's divisor stops being 1, and
    /// the port has nothing to divide."* This is that sentence as an assertion.
    ///
    /// It is unreachable from any fixture — `S2` refuses a two-call body long
    /// before the walk starts — which is why it is *also* a registered decision
    /// surface. A test can only assert the point it was written for; the domain
    /// enumerates the region.
    ///
    /// **AMENDED by lane `w-budget`: the refusal is now conditional on there
    /// being no count.** `#1020`'s hazard is unchanged *where the hazard is* —
    /// a caller whose count the port could not read still has nothing to
    /// divide — and the sibling test below is the other half.
    #[test]
    fn two_call_sites_refuse_by_name_and_do_not_guess() {
        let m = &BUDGET_MODELS[0];
        let at = Expansion::at_pass_entry();
        assert_eq!(at.growth_total, None, "the unseeded entry state carries no total");
        assert_eq!(port_enter_site(m, at, 1, 0, None).map(|e| e.budget), Ok(NestedBudget::Parent));
        for n in 2..=6i64 {
            for i in 0..n {
                let got = port_enter_site(m, at, n, i, None);
                if m.divisor(n, i) == 1 {
                    // The LAST site of a run: c2's counter has reached 1 and the
                    // division is the identity there too. It is admitted, and
                    // saying so is the difference between modelling the rule and
                    // refusing on the shape of the question.
                    assert!(got.is_ok(), "n={n} i={i}: the last site divides by 1");
                    continue;
                }
                assert_eq!(
                    got,
                    Err("S6-budget-divided"),
                    "n={n} i={i}: the port must REFUSE where c2 divides — it has no B"
                );
            }
        }
    }

    /// **THE ADOPTION, ASSERTED WHERE IT HAPPENS.** With a count in hand,
    /// `n ≥ 2` is a *computed verdict* rather than a blanket refusal — which is
    /// the whole of lane `w-budget`.
    ///
    /// The arithmetic is c2's: `B = clamp(2 × 600, 1000, 35000) = 1200`, and at
    /// site 0 of 3 the nested pass gets `1200 / 3 = 400` (`idiv` at
    /// `0x10b623ec`, divisor `n − i` at the 0-based index).
    #[test]
    fn a_readable_caller_count_turns_the_divided_refusal_into_a_number() {
        let m = &BUDGET_MODELS[0];
        let at = Expansion::at_pass_entry_seeded(m, 600);
        assert_eq!(at.budget, NestedBudget::Amount(1_200), "C3's clamp on 2 x 600");
        assert_eq!(at.growth_total, Some(600), "C2 seeds the total from the same count");

        for n in 1..=6i64 {
            for i in 0..n {
                let got = port_enter_site(m, at, n, i, None);
                assert!(
                    got.is_ok(),
                    "n={n} i={i}: with B in hand the port DIVIDES; it does not refuse"
                );
                assert_eq!(
                    got.unwrap().budget,
                    NestedBudget::Amount(1_200 / m.divisor(n, i)),
                    "n={n} i={i}: the nested budget is B / (n - i)"
                );
            }
        }
        // And the refusal is still there for the case that is still honestly
        // unevaluable — no count.
        assert_eq!(
            port_enter_site(m, Expansion::at_pass_entry(), 3, 0, None),
            Err("S6-budget-divided")
        );
    }

    /// **C16, and the `0x81..=0xff` hazard as a modelled point.**
    ///
    /// `WB_INSTRCOUNT_FINDINGS` §4: a `.gl` `SIZE` byte in `0x81..=0xff` is one
    /// signed byte in c2, so the caller reads as 65,409..65,535, the seed puts
    /// `DAT_10c3f5cc` above 35,000, and **C16 declines the caller's very first
    /// site**. That is c2's behaviour and the model reproduces it.
    ///
    /// It is asserted here and it cannot arrive through the port: the `.gl`
    /// reader refuses that encoding whole-file. Two different statements, and
    /// the test says which is which.
    #[test]
    fn a_caller_over_the_growth_cap_declines_its_first_site() {
        let m = &BUDGET_MODELS[0];
        assert_eq!(m.growth_total_max, INLINE_GROWTH_TOTAL_MAX);
        // At the cap exactly, `jg` still admits.
        let at = Expansion::at_pass_entry_seeded(m, INLINE_GROWTH_TOTAL_MAX);
        assert!(port_enter_site(m, at, 1, 0, None).is_ok(), "35000 is not > 35000");
        // One past it, and anywhere in c2's sign-extension band, it declines.
        for c in [INLINE_GROWTH_TOTAL_MAX + 1, 65_409, 65_535] {
            assert_eq!(
                port_enter_site(m, Expansion::at_pass_entry_seeded(m, c), 1, 0, None),
                Err("S6-budget-caller-huge"),
                "caller count {c}"
            );
        }
        // Without a count there is no total and C16 cannot fire — the port must
        // never refuse on its own ignorance.
        assert!(!m.declines_at_growth_total(Expansion::at_pass_entry()));
    }

    /// **C17, and the first-site theorem it obeys.**
    ///
    /// `WB_INSTRCOUNT_FINDINGS` §5.2: `B ≥ 1000` for every caller, so at an
    /// undrained budget C17 cannot decline a callee counting below 1000 — and
    /// the caller's own size only scales `B` upward. The arm is reachable only
    /// once the budget has been drained or the callee is very large, and both
    /// are asserted rather than argued.
    #[test]
    fn the_affordability_arm_obeys_the_first_site_theorem() {
        let m = &BUDGET_MODELS[0];
        let entry = Expansion::at_pass_entry_seeded(m, 0);
        assert_eq!(entry.budget, NestedBudget::Amount(INLINE_BUDGET_FLOOR));
        for ci in [0i64, 40, 41, 999, 1_000] {
            assert!(
                port_enter_site(m, entry, 1, 0, Some(ci)).is_ok(),
                "callee {ci} is affordable at the floor: the first site cannot decline"
            );
        }
        // Past the floor it declines — and only when the callee is ALSO over
        // C18's 40, which is the `&&` in c2's pair of jumps.
        assert_eq!(
            port_enter_site(m, entry, 1, 0, Some(1_001)),
            Err("S6-budget-unaffordable")
        );
        let broke = Expansion {
            budget: NestedBudget::Amount(10),
            ..Expansion::at_pass_entry_seeded(m, 0)
        };
        assert!(
            port_enter_site(m, broke, 1, 0, Some(40)).is_ok(),
            "40 is not > 40: C18's exemption is what stops C17 firing on a small callee"
        );
        assert_eq!(
            port_enter_site(m, broke, 1, 0, Some(41)),
            Err("S6-budget-unaffordable")
        );
        // Either operand missing is UNASKED and admits.
        assert!(!m.declines_unaffordable(broke, None));
        assert!(!m.declines_unaffordable(Expansion::at_pass_entry(), Some(10_000)));
    }

    /// **C19's charge, on the production walk's own state.** The local
    /// subtraction is gated by C18's 40 and the global add is not — so a run of
    /// small callees drains nothing and still accumulates growth, which is the
    /// asymmetry `CLAUSES.tsv` C19 states as one clause.
    #[test]
    fn the_charge_moves_the_budget_and_the_total_by_different_amounts() {
        let m = &BUDGET_MODELS[0];
        let at = Expansion::at_pass_entry_seeded(m, 600);
        let after = port_enter_site(m, at, 1, 0, Some(40)).unwrap();
        assert_eq!(after.budget, NestedBudget::Amount(1_200), "at 40 the budget is untouched");
        assert_eq!(after.growth_total, Some(640), "and the total is charged anyway");
        let after = port_enter_site(m, at, 1, 0, Some(41)).unwrap();
        assert_eq!(after.budget, NestedBudget::Amount(1_159), "past 40 both move");
        assert_eq!(after.growth_total, Some(641));
    }

    /// **The count reaches the walk through `TuContext`, and a name it does not
    /// carry gets no answer.** The lookup is the binding hazard (#918), so it is
    /// asserted rather than assumed: one name in, one count out, and a
    /// disagreeing duplicate silently yields nothing rather than either value.
    #[test]
    fn the_context_answers_a_count_only_for_a_name_it_unambiguously_carries() {
        let tu = TuContext::none().with_instr_counts(vec![
            ("?a@@YAXXZ", 7i64),
            ("?b@@YAXXZ", 11),
            ("?dup@@YAXXZ", 3),
            ("?dup@@YAXXZ", 4),
            ("?same@@YAXXZ", 5),
            ("?same@@YAXXZ", 5),
        ]);
        assert_eq!(tu.instr_count("?a@@YAXXZ"), Some(7));
        assert_eq!(tu.instr_count("?b@@YAXXZ"), Some(11));
        assert_eq!(tu.instr_count("?dup@@YAXXZ"), None, "two counts, no answer");
        assert_eq!(tu.instr_count("?same@@YAXXZ"), Some(5), "agreeing rows are one fact");
        assert_eq!(tu.instr_count("?missing@@YAXXZ"), None);
        // A context with no counts at all is the pre-`w-budget` context.
        assert_eq!(TuContext::none().instr_count("?a@@YAXXZ"), None);
    }

    /// **THE PRODUCTION REACH, and the reason this is not `#3336`'s
    /// decoration.** A criterion that cannot fail abstains rather than passes —
    /// so the seed has to be shown arriving through the walk the emitter runs,
    /// not only through a hand-built `Expansion`.
    ///
    /// `t01`'s two functions, with a count attached to the caller: the splice
    /// still fires (byte-neutrality) **and** the count is what the caller's own
    /// name resolves to, which is the binding the walk uses.
    #[test]
    fn the_walk_seeds_from_the_callers_own_count() {
        let funcs = vec![leaf("?g@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        let tu = TuContext::of(&funcs)
            .with_instr_counts(vec![("?f@@YAHH@Z", 600i64), ("?g@@YAHH@Z", 12)]);
        assert_eq!(tu.instr_count(&funcs[1].mangled_name), Some(600));
        assert!(
            splice_body_why(&funcs[1], &sel, OptMode::O1, &tu).is_ok(),
            "the seed must not move the splice: this is the byte-neutrality claim"
        );
        // And with a caller over the growth cap it declines — the same body, the
        // same context, one number different. This is the refusal-domain control
        // in miniature: no fixture can produce it, and the port answers.
        let huge = TuContext::of(&funcs)
            .with_instr_counts(vec![("?f@@YAHH@Z", 65_535i64), ("?g@@YAHH@Z", 12)]);
        assert!(
            matches!(
                splice_body_why(&funcs[1], &sel, OptMode::O1, &huge),
                Err(SpliceDecline::Refused("S6-budget-caller-huge"))
            ),
            "a caller c2 reads as 65,535 instructions inlines nothing"
        );
    }

    /// The divisor is c2's `n − i + 1` on a 1-based index — the counter is
    /// initialised to `n` by the collector (`0x10b600f7`/`0x10b60374`) and
    /// decremented at the **bottom** of the loop (`0x10b620c8`), so at the
    /// 0-based `i` it still reads `n − i`.
    #[test]
    fn the_divisor_is_the_remaining_site_count() {
        let m = &BUDGET_MODELS[0];
        assert_eq!(m.divisor(1, 0), 1);
        assert_eq!(m.divisor(3, 0), 3);
        assert_eq!(m.divisor(3, 1), 2);
        assert_eq!(m.divisor(3, 2), 1);
        // The counterfactual: the model the port implicitly held before this
        // lane, which is what a widened `S2` would silently install.
        assert_eq!(BUDGET_UNDIVIDED.divisor(3, 0), 1);
    }

    /// §2.2 / C3's clamp, at both ends and on both sides of each.
    /// PROV[R] `0x10b62708` (`add eax,eax`), `0x10b6270a` (1000), `0x10b62715`
    /// (35000).
    #[test]
    fn the_seed_is_c2s_clamp() {
        let m = &BUDGET_MODELS[0];
        assert_eq!(m.seed(0), 1000, "the floor holds at zero instructions");
        assert_eq!(m.seed(499), 1000, "2 × 499 = 998 is under the floor");
        assert_eq!(m.seed(500), 1000, "2 × 500 = 1000 is the floor exactly, and `jle` keeps it");
        assert_eq!(m.seed(501), 1002, "past the floor the doubling is the answer");
        assert_eq!(m.seed(17_499), 34_998);
        assert_eq!(m.seed(17_500), 35_000, "the ceiling exactly");
        assert_eq!(m.seed(20_000), 35_000, "and it does not move past it");
    }

    /// **C18 AND THE `__forceinline` SKIP ARE DIFFERENT IN EXTENT, NOT ONLY IN
    /// CONDITION** — the sharpest form of §6.6.2's orthogonality claim, and the
    /// half that is only visible with both listings side by side.
    ///
    /// `jbe 0x10b625bd` at `0x10b625b9` skips **only** the local charge at
    /// `0x10b625bb`; `jne 0x10b625c7` at `0x10b625b0` skips that **and** the
    /// global growth total at `0x10b625c1`.
    #[test]
    fn the_forty_instruction_exemption_is_local_only_and_forceinline_is_both() {
        let m = &BUDGET_MODELS[0];
        assert_eq!(m.charge(40, false), (0, 40), "at 40: local exempt, GLOBAL STILL CHARGED");
        assert_eq!(m.charge(41, false), (41, 41), "past 40 both are charged");
        assert_eq!(m.charge(41, true), (0, 0), "__forceinline: NEITHER is charged");
        assert_eq!(m.charge(1000, true), (0, 0), "and its size does not matter");
        assert_eq!(
            BUDGET_CHARGE_FORCEINLINE.charge(41, true),
            (41, 41),
            "the counterfactual model charges it like anything else"
        );
    }

    /// **THE SOUNDNESS ARGUMENT, EXECUTED RATHER THAN ARGUED.**
    ///
    /// `w-inlfit` closed §6.6.2 with *"the port admits only chains in which
    /// every link has exactly one call site, so `n = 1` and c2's division is the
    /// identity — a soundness argument for a fit, not a derivation of one"*.
    /// This asserts the premise on the cells that actually fire, so that a lane
    /// which widens `S2` finds out here rather than in a workload relocation.
    #[test]
    fn every_admitted_link_has_exactly_one_call_site() {
        // `t01` — the tail-call cell.
        let funcs = vec![leaf("?g@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        assert_eq!(predicate_site_count(&funcs[1], &sel), 1);
        // `t11` — the fixpoint cell, all three links.
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g@@YAHH@Z"),
        ];
        for i in 1..3 {
            let sel = select_function(&funcs[i], OptMode::O1).unwrap();
            assert_eq!(predicate_site_count(&funcs[i], &sel), 1, "link {i}");
        }
        // And the chain still splices, with the model in the walk: the whole
        // point of the adoption is that this did not change.
        assert_eq!(
            spliced(&funcs, 2),
            spliced(&funcs, 1),
            "the fixpoint still closes: `?f` takes `?h`'s body, as `t11` measured"
        );
    }

    /// The `Seq` arm reads the IL's own call count rather than assuming 1, so a
    /// widening of `S2` reaches the budget model instead of walking past it.
    ///
    /// This is `t08`'s body — two calls — and the assertion is on the count the
    /// model would be handed, not on the refusal `S2` produces first. Both
    /// matter and they are different facts: `S2` is where the two-call body is
    /// refused **today**, and this is what the budget model would see if it
    /// were not.
    #[test]
    fn a_seq_bodys_site_count_comes_from_the_il() {
        let mut caller = func_with(Vec::new(), Vec::new());
        caller.mangled_name = "?f@@YAXXZ".into();
        caller.data_syms.clear();
        let call = |c: &str| SeqCall {
            callee: c.into(),
            arg_ops: Vec::new(),
            arg_slots: None,
            link_args: None,
        };
        caller.body = c2_il::BodyShape::Seq(CallSeq {
            calls: vec![call("?g@@YAXXZ"), call("?h@@YAXXZ")],
            tail: SeqTail::Void,
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
            store_run: None,
        });
        assert_eq!(caller.call_seq().map(|s| s.calls.len()), Some(2));
        if let Ok(sel) = select_function(&caller, OptMode::O1) {
            assert_eq!(
                predicate_site_count(&caller, &sel),
                2,
                "two calls, two sites — the count is READ, not assumed"
            );
            // …and at `n = 2` the model's first site divides by 2, which the
            // port cannot evaluate. This is `#1020`'s hazard reached through
            // the IL rather than through a hand-written `n`.
            assert_eq!(
                port_enter_site(
                    &BUDGET_MODELS[0],
                    Expansion::at_pass_entry(),
                    predicate_site_count(&caller, &sel),
                    0,
                    None
                ),
                Err("S6-budget-divided")
            );
        }
    }

    /// C14's depth cap, and the fact that makes it inert here: the base is 0 for
    /// every compilation this project runs (`mov ds:0x10c3f50c,ebp` at
    /// `0x10b6274c` with `ebp = 0`), and a zero base jumps past the whole
    /// comparison (`je 0x10b60a25` at `0x10b60a15`).
    ///
    /// Said as a test rather than a comment because "the cap cannot bind" is the
    /// kind of claim that stops being true without anyone noticing.
    #[test]
    fn the_depth_cap_is_read_but_inert_at_the_base_this_project_compiles_with() {
        let m = &BUDGET_MODELS[0];
        for level in [1i64, 17, 100, 10_000] {
            assert!(
                !m.declines_at_depth(Expansion { level, level_base: 0, budget: NestedBudget::Parent, growth_total: None }),
                "base 0: c2 jumps past the comparison entirely"
            );
        }
        assert!(!m.declines_at_depth(Expansion { level: 17, level_base: 1, budget: NestedBudget::Parent, growth_total: None }));
        assert!(m.declines_at_depth(Expansion { level: 18, level_base: 1, budget: NestedBudget::Parent, growth_total: None }));
    }

    /// **C15, and the arm no clause names.** Lane `w-inlclause`,
    /// `work/w-inlclause/IMAGE_READ.md` §2.
    ///
    /// The three assertions are three different statements and only the first
    /// is C15:
    ///
    /// 1. the absolute arm (`0x10b60a2f`–`0x10b60a3a`) is **vacuous at c2's
    ///    default**, which is why adopting it moved no byte;
    /// 2. the `__forceinline` bypass (`0x10b60a28`) skips **only** that arm;
    /// 3. the *relative* arm at `0x10b60a21` — `level − base > maxlevel`, which
    ///    the 24-clause table does not cover — declines where the port used to
    ///    admit, in the `base != 0` region no compilation here reaches.
    #[test]
    fn maxlevel_is_two_arms_the_bypass_reaches_one_and_the_default_switches_both_off() {
        let c2 = &BUDGET_MODELS[0];
        assert_eq!(c2.max_level, INLINE_MAXLEVEL_UNBOUNDED, "c2's read default");

        // 1. Vacuous at the default, at every level, either way on the bypass.
        for level in [1i64, 2, 255, 256, 100_000] {
            let at = Expansion {
                level,
                level_base: 0,
                budget: NestedBudget::Parent,
                growth_total: None,
            };
            for fi in [false, true] {
                assert!(
                    !c2.declines_at_maxlevel(at, fi),
                    "the 0xff guard makes C15 vacuous — this is the byte-neutrality argument"
                );
            }
        }

        // 2. With maxlevel set, the arm fires — and `__forceinline` skips it.
        let m2 = &BUDGET_MAXLEVEL_2;
        let deep = Expansion {
            level: 3,
            level_base: 0,
            budget: NestedBudget::Parent,
            growth_total: None,
        };
        assert!(m2.declines_at_maxlevel(deep, false), "3 > 2: c2 declines");
        assert!(!m2.declines_at_maxlevel(deep, true), "`jne 0x10b60a3c` skips exactly this test");
        assert_eq!(
            port_enter_site(
                m2,
                Expansion {
                    level: 2,
                    level_base: 0,
                    budget: NestedBudget::Parent,
                    growth_total: None,
                },
                1,
                0,
                None,
            ),
            Err("S6-budget-maxlevel"),
            "the port passes forceinline = false: it cannot read the bit"
        );

        // 3. The relative arm, which is upstream of the bypass and of the
        //    `!= 0xff` guard, and which no clause of the 24 names.
        let rel = Expansion {
            level: 6,
            level_base: 1,
            budget: NestedBudget::Parent,
            growth_total: None,
        };
        assert!(
            !c2.declines_at_depth(rel),
            "level - base = 5, under both 16 and 255"
        );
        assert!(
            m2.declines_at_depth(rel),
            "0x10b60a21: level - base = 5 > maxlevel 2, and C14's 16 has NOT bound. \
             The port admitted here before this lane read the second `jg`."
        );
    }

    /// **`#3746`, one field over.** `INLINE_MAXLEVEL_UNBOUNDED` is both the
    /// guard's comparand *and* `BUDGET_C2`'s field value, so mutating it moves
    /// both sides together and every named model renders identically. The
    /// sweep in [`surface_rows`] is what makes the constant reachable, and this
    /// asserts that the sweep actually separates the sentinel from its
    /// neighbours rather than merely mentioning it.
    #[test]
    fn the_maxlevel_sentinel_is_reachable_in_the_domain_and_not_merely_named() {
        let rows = surface_rows();
        let at = |ml: i64, level: i64| {
            let key = format!("maxlevel={ml:03} level={level:03} forceinline=0");
            rows.iter().find(|r| r.point == key).unwrap_or_else(|| panic!("no row {key}")).outcome.clone()
        };
        // At the sentinel the guard switches the whole clause off; one below and
        // one above, the same level declines. If those three ever agree, the
        // constant has stopped being a boundary and the domain will say so.
        assert!(at(INLINE_MAXLEVEL_UNBOUNDED, 300).starts_with("admit"));
        assert!(at(254, 300).starts_with(crate::surface::REFUSE));
        assert!(at(256, 300).starts_with(crate::surface::REFUSE));
    }

    /// The registry's floors, restated where the generator lives, so that a
    /// domain which quietly shrank is caught by this module's own suite and not
    /// only by `surface`'s.
    #[test]
    fn the_budget_surface_domain_refuses_far_more_than_it_admits() {
        let rows = surface_rows();
        assert!(rows.len() >= 500, "{} cells", rows.len());
        let refusals = rows.iter().filter(|r| r.outcome.starts_with(crate::surface::REFUSE)).count();
        assert!(refusals >= 300, "{refusals} refusals of {}", rows.len());
        assert!(
            rows.iter().any(|r| r.outcome.contains("S6-budget-divided")),
            "the n >= 2 region is the reason this surface exists"
        );
    }
}

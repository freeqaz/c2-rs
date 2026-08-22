//! The compiler-label counter — the `$M`/`$T` numbers c2 stamps into the symbol
//! table. A wrong `$M` is six wrong bytes in an obj that still links, which is
//! why `docs/LABEL_COUNTER.md` exists.
//!
//! **The read that would replace the fitted constants below — comment only,
//! nothing here changes.** Added 2026-08-22 under read-before-probe
//! (`docs/WHITEBOX_LEVERAGE_2026-08-21.md` §1;
//! `docs/whitebox/READ_PLAN_2026-08-21.md` §2/§3). `LABEL_SEED_GAP = 9` and
//! the `/Gy` `+3` are fitted from objs and their *identities* are unread —
//! which nine allocations make the nine, which three slots the `/Gy` three.
//! **c2's mechanism is fully read**: TU-global counter `DAT_10c2edd0` with a
//! **sole increment instruction at `0x10b97de5`**, allocator `FUN_10b97dd0`
//! (28 B, **31** call sites), generic constructor `FUN_10b9a455` (54 B, **132**
//! sites / 86 functions), name formatter `FUN_10b99dfe`, and a second
//! per-function counter `DAT_10c2e918` reset in `FUN_10b7e113`.
//!
//! **R3 IS DONE (2026-08-22, lane `w-read-r3`) AND IT SPLIT THE CLAIM ABOVE IN
//! HALF.** The **site population** is closed by construction, with the argument
//! the read-plan never supplied: a direct `call` encodes a *relative*
//! displacement, and neither `FUN_10b97dd0` nor `FUN_10b9a455` has its absolute
//! VA appear **anywhere** in the image as data, so no indirect call can reach
//! them — the 31 + 132 enumeration is exhaustive, not a sample. **The CHARGE is
//! not closed**: 42 of the 163 sites sit on **loop back edges** (decisively
//! `0x10b5cee1`, a nested loop over a 1,024-bucket symbol table), so the rule is
//! `charge(TU) = the number of objects c2 constructs itself` — data-dependent,
//! not a constant to be substituted. Anything priced as *"replace the fitted
//! +9/+3"* inherits *"reproduce c2's object population"* instead.
//! See `docs/whitebox/ref/P_LABEL.md`.
//!
//! **`LABEL_SEED_GAP` IS REPAIRED (2026-08-22, lane `w-seedgap`, board
//! `#3402`–`#3405`) AND THE `/Gy` `+3` IS NOT.** The gap is no longer a literal:
//! it is [`SeedGapModel::READ`] — R3's read formula, coefficients named —
//! evaluated at [`SeedGapInputs::PORT_ADMITTED`], whose two fields cite the two
//! upstream refusals that make `9` right today. The `+3` below is still fitted
//! from 11 TUs and still has an unread identity; it is a **charge**, so it is
//! exactly what R3 says is not a constant to be substituted, and this lane did
//! not touch it.
//!
//! **Two honest limits.** R3 gives the *charge*, not the *order*; a charge
//! rule without an order rule still cannot place a label, and the other half
//! is **R8** (block emission order, 5–10 d and the only read with no known
//! address for its rule — `CEILING` §6.1 phase 1, the one UNSERVED phase).
//! And `docs/LABEL_COUNTER.md:3-18`'s own banner says four lanes measured
//! label strides wrong by reading counterfactual displacements as charges
//! (board #3368) — read the banner before reusing any stride from that page.

use super::*;

/// How far past the `.gl` label counter ([`c2_il::label_counter`]) the first
/// compiler label of a TU sits.
///
/// **THIS IS NOT A COMPILATION-INDEPENDENT CONSTANT, AND `9` IS ONE CELL OF
/// IT** — which is now expressed in the type rather than only in this comment.
/// Read **R3** (2026-08-22, lane `w-read-r3`, board #3387–#3390;
/// `docs/whitebox/ref/P_LABEL.md`, `docs/whitebox/WB_LABELCHARGE_FINDINGS.md`)
/// measured it over 22 cells as
///
/// ```text
/// gap = 7 + 2·[/Og] + 1·[/GF ∧ a string literal pooled in the data phase]
/// ```
///
/// so `/Od` reads **7**, and `/Od` is one of `scripts/lanes.txt`'s 18 graded
/// lanes. **The defect is LATENT, not live, and that was checked rather than
/// argued**: `mode_lane.sh /Od` reads `match=21 mismatch=0` and all 21 matching
/// TUs contain **zero function definitions**, so no `$M` is ever emitted there;
/// the `/O1`+string shape returns `Port=NotImplemented` with the reference
/// replay byte-exact. What is live is the **licence** — this constant reads as
/// compilation-independent, and every caller inherits that reading.
///
/// **This is no longer written as a literal.** It is [`SeedGapModel::READ`]
/// evaluated at [`SeedGapInputs::PORT_ADMITTED`], and it is `9` because — and
/// only because — of the two upstream refusals that constant names.
///
/// So the old warning here — *"do not widen the emit set into a configuration
/// where a framed function meets `/Od`, `/Og` or `/GF`-with-a-pooled-string
/// without replacing this constant"* — is **discharged for `/Od` and left
/// standing for the string.** A widening that admits `/Od` now trips a
/// `debug_assert` in [`crate::PortC2::build`] (and the gate's DEBUG-lane row runs
/// every fixture through it) and has [`plan_labels_with_gap`] to call. A widening
/// that admits a **data-phase pooled string literal** has neither: the second
/// conjunct is not derivable from the port's inputs, so that caller must supply
/// [`SeedGapInputs::pooled_data_phase_string`] itself and must not guess it. A
/// wrong `$M` is six wrong bytes in an obj that still links, which is the whole
/// reason `docs/LABEL_COUNTER.md` exists.
pub const LABEL_SEED_GAP: u32 = SeedGapModel::READ.gap(&SeedGapInputs::PORT_ADMITTED);

/// **The compilation facts the seed gap is a function of** — the settable half of
/// a decision point that used to be a baked constant
/// (`docs/GOAL_DECISION_2026-08-21.md` § AMENDED: general layers expose decision
/// points as named, settable parameters).
///
/// Both fields are **read**, not fitted — R3 measured the gap over 22 cells and
/// these are the two indicators the measurement resolved it into. Neither is a
/// property of the IL alone: see each field for where it can and cannot be
/// derived from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeedGapInputs {
    /// `[/Og]` — **the global optimizer runs.**
    ///
    /// `/Og`, `/Ox`, `/O1` and `/O2` all imply it and read **9**; `/Od`, `/Os`,
    /// `/Ot`, `/Oy` and `/Ob2` do not and read **7**. **Derivable from the port's
    /// own inputs** — the per-function optimization word is in the IL, and
    /// [`global_optimizer_of_opt_word`] is the derivation.
    pub global_optimizer: bool,
    /// `[/GF ∧ a string literal is pooled in the data phase]` — **both**
    /// conjuncts, and neither one alone.
    ///
    /// * `/Ox` with a file-scope `const char* g = "x";` reads **9** — no `/GF`,
    ///   no pooling, no charge.
    /// * `/Ox /GF` with the same line reads **10**, and so do `/O1` and `/O2`,
    ///   which imply `/GF`.
    /// * `/O1` **without** the line reads **9**: `/GF` alone charges nothing.
    /// * `const char g[] = "x"` is an array *copy* with no separate string
    ///   object and reads **9** even under `/GF`.
    /// * A literal returned from a *function* is not in the data phase at all
    ///   and costs **0** (`docs/LABEL_COUNTER.md` §8.1).
    ///
    /// **This is NOT derivable from the port's inputs today, and the obvious
    /// detector is wrong.** `/GF` is an argv fact that appears nowhere in the IL
    /// — not a bundle field, not a bit of the optimization word, and not a member
    /// of [`crate::plan::PlanInputs`] — exactly as `/Gy` is. And the second
    /// conjunct cannot be read off `c2_il`'s `??_C@…` record set, because that
    /// set **over-approximates**: it also contains function-body literals, which
    /// charge 0. A detector built on it would replace a fitted constant with a
    /// fitted rule, which is strictly worse — the constant is at least visibly
    /// wrong. So this stays an input, supplied by the caller.
    pub pooled_data_phase_string: bool,
}

impl SeedGapInputs {
    /// **The configuration every TU the port admits today is in — and the two
    /// refusals that make that true.** `LABEL_SEED_GAP`'s `9` is this and nothing
    /// else, which is the fact the shipped literal `9` concealed.
    ///
    /// * `global_optimizer: true` — [`crate::PortC2::build`] resolves a whole-TU
    ///   mode through `codegen::opt_mode_of_word`, which admits exactly `/Ox`
    ///   ([`c2_il::OPT_WORD_OX`]) and `/O1` ([`c2_il::OPT_WORD_O1`]) plus their
    ///   `fp_contract`-off spellings. **Both imply `/Og`.**
    ///   [`c2_il::OPT_WORD_OD`] and [`c2_il::OPT_WORD_PRAGMA_OFF`] are refused
    ///   there, so no `/Od` TU with a function ever reaches [`plan_labels`].
    ///   This is asserted, not just asserted-in-prose: `build` carries a
    ///   `debug_assert` tying the resolved mode to this field, and the gate's
    ///   DEBUG-profile row runs every lane through it.
    /// * `pooled_data_phase_string: false` — a data-phase pooled literal needs a
    ///   TU-scope object holding a relocation against a `??_C@…` COMDAT, and
    ///   every path that reaches the two writers refuses that shape first:
    ///   `c2_il::IlBundle::data_tu` refuses any TU whose `.gl` carries a `??_C@…`
    ///   record at all, and `build`'s catch-all refuses "an initialized or
    ///   uninitialized namespace-scope object, a `const` pool, a string literal,
    ///   a thread-local, or a COMDAT". R3 checked it rather than argued it: a
    ///   file-scope `const char* g3 = "x";` ahead of two framed functions returns
    ///   `Port=NotImplemented` with the reference replay byte-exact.
    ///
    /// **Neither bullet is a promise about the future.** They are the reasons the
    /// value is right *today*, written down so the rung that widens the emit set
    /// past one of them finds the dependency instead of inheriting it.
    pub const PORT_ADMITTED: SeedGapInputs = SeedGapInputs {
        global_optimizer: true,
        pooled_data_phase_string: false,
    };
}

/// **The three coefficients of the seed-gap formula, as read.** Separated from
/// [`SeedGapInputs`] so the *model* and the *compilation* are two things: a
/// permuter searching this decision point moves the inputs, and a lane that
/// re-measures the gap moves the coefficients.
///
/// ```text
/// gap = 7 + 2·[/Og] + 1·[/GF ∧ a string literal pooled in the data phase]
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeedGapModel {
    /// The gap with neither term — `/Od`, `/Os`, `/Ot`, `/Oy`, `/Ob2`.
    pub base: u32,
    /// What the global optimizer adds.
    pub global_optimizer: u32,
    /// What a data-phase pooled string literal adds under `/GF`.
    pub pooled_data_phase_string: u32,
}

impl SeedGapModel {
    /// **The measured model**: `7 + 2·[/Og] + 1·[/GF ∧ pooled data-phase
    /// string]`, over 22 cells, seed read straight out of the captured `.gl` as
    /// `u32_le(.gl[7..11])` so it cannot hide inside the answer. Lane
    /// `w-read-r3`, board **#3388**; `docs/whitebox/ref/P_LABEL.md` §4.1,
    /// `docs/whitebox/WB_LABELCHARGE_FINDINGS.md` §5.1, `docs/LABEL_COUNTER.md`
    /// §8.1. Instrument: `scripts/gt_label_seedgap.py`.
    ///
    /// **What is read here is the arithmetic, not the mechanism.** Which
    /// allocations make the 7, the 9 or the 10 is still not enumerated —
    /// `P_LABEL.md` §4 bounds the candidates to five once-per-TU sites and says
    /// attributing each unit needs a live tap on `0x10b97de5`, which no lane has
    /// built. A `SeedGapModel` is therefore a *fit to a read grid*, one level
    /// better than the literal `9` and one level short of the mechanism.
    pub const READ: SeedGapModel = SeedGapModel {
        base: 7,
        global_optimizer: 2,
        pooled_data_phase_string: 1,
    };

    /// Evaluate the model. `const fn`, so [`LABEL_SEED_GAP`] stays a `const` and
    /// nothing that used it pays for the change.
    pub const fn gap(&self, inputs: &SeedGapInputs) -> u32 {
        self.base
            + if inputs.global_optimizer { self.global_optimizer } else { 0 }
            + if inputs.pooled_data_phase_string { self.pooled_data_phase_string } else { 0 }
    }
}

/// **`[/Og]` from one function's IL optimization word** — the derivation that
/// makes the first term a read of the compilation rather than an assumption
/// about it.
///
/// * `Some(true)` — `/Ox` and `/O1` (and their `fp_contract`-off spellings), the
///   only words [`c2_il::opt_word_mode`] admits. **Both imply `/Og`**: `/Ox` is
///   `/Og /Oi /Ot /Oy /Ob2` and `/O1` is `/Og /Os /Oy /Ob2 /GF /Gy`.
/// * `Some(false)` — [`c2_il::OPT_WORD_OD`] and [`c2_il::OPT_WORD_PRAGMA_OFF`],
///   both of which mean the global optimizer did not run. Their gap is **7**.
/// * `None` — any other word, including one nobody has captured. **Fail-closed
///   by construction**: a caller that cannot get a `bool` here must refuse, and
///   `None` is not `Some(false)`.
///
/// [`c2_il::OPT_WORD_SPECIAL_MEMBER`] is masked off first, for the reason its own
/// doc gives — the constructor/destructor bit is orthogonal to the mode, and not
/// masking it is what once censused every ctor and dtor as a `codegen-gap`.
///
/// **`/Os`, `/Ot`, `/Oy` and `/Ob2` are `None`, not `Some(false)`**, even though
/// R3 measured their gap at 7. Their words have never been captured, so mapping
/// them would be inventing a reading of a word this project has not read; the
/// gap they produce is known and the word that carries it is not.
pub fn global_optimizer_of_opt_word(word: Option<u32>) -> Option<bool> {
    // The accepted set first, through the one existing decoder, so this cannot
    // drift from what `build` actually admits.
    if c2_il::opt_word_mode(word).is_some() {
        return Some(true);
    }
    match word.map(|v| v & !c2_il::OPT_WORD_SPECIAL_MEMBER) {
        Some(c2_il::OPT_WORD_OD) | Some(c2_il::OPT_WORD_PRAGMA_OFF) => Some(false),
        // The same pragma under `/O1` is the **short varint** `04` rather than an
        // escaped word — `opt_word_at`'s `b < 0x80` branch, documented in its own
        // doc as `4f 1f 04 …  = 0x00000004`. Same state, different spelling.
        Some(0x0000_0004) => Some(false),
        _ => None,
    }
}

/// The `$M`/`$T` label numbers c2 gives each function, or `None` for a function
/// that is not framed (it consumes counter slots but emits no label).
///
/// The allocator, measured against real objs over 25 TUs — see
/// `docs/OBJ_GY_SHAPES.md` §3.4/§3.5:
///
/// * the first label of a TU is `.gl` counter + [`LABEL_SEED_GAP`];
/// * under `/Gy` a flat surcharge of **3 per function in the TU** is paid
///   up front, before any function's own labels — even for functions that emit
///   no label at all;
/// * then, in `.text` order, each function consumes **1** if it is a leaf and
///   **4** (packed) / **5** (`/Gy`) if it is framed, of which the framed
///   function emits the first three as `$M(n)` (prologue end), `$M(n+1)`
///   (function end) and `$T(n+2)` (its `.pdata` record).
///
/// The "1 per leaf" holds for every function class this port emits and **not**
/// for every function class: a signed-relational comparison leaf (`a < b`)
/// consumes 3, and each **newly pooled** FP constant a further 2. Those are
/// refused upstream ([`crate::PortC2::build`]) rather than modeled, because a
/// wrong stride is a wrong `$M` number and a wrong `$M` number is a wrong-bytes
/// obj — the whole point of the counter.
///
/// **A constant-free floating-point leaf is 1, not 2**, and this comment used to
/// say 2. The 2 is a whole-TU reading of a leaf that is itself the TU's first FP
/// function — `_fltused`'s slot, which the `+1` below already charges once per
/// TU. `docs/LABEL_COUNTER.md` §1: `leaf-float` = 2, `leaf-float-led` = 1,
/// `leaf-double-led` = 1. Charging it twice was what kept every (FP leaf, framed
/// function) pair out of class.
pub fn plan_labels(counter: u32, funcs: &[Function], comdat: bool) -> Vec<Option<[u32; 3]>> {
    plan_labels_with_gap(counter, funcs, comdat, LABEL_SEED_GAP)
}

/// [`plan_labels`] with the seed gap **supplied** instead of assumed — the
/// settable form of the decision point.
///
/// [`plan_labels`] is this at [`LABEL_SEED_GAP`], i.e. at
/// [`SeedGapInputs::PORT_ADMITTED`]. A caller in a different compilation —
/// a widened emit set that admits `/Od`, or a permuter searching this decision
/// point — computes its own with [`SeedGapModel::gap`] and passes it here rather
/// than editing a constant.
///
/// The gap is the **only** thing this takes; everything after it (the `/Gy`
/// pre-pass, the `_fltused` slot, the `memcpy` slot, the FP-pool surcharge, the
/// per-function stride) is a charge measured **seed-free, in-TU**, so it is
/// unaffected by the gap and is not a second parameter.
pub fn plan_labels_with_gap(
    counter: u32,
    funcs: &[Function],
    comdat: bool,
    seed_gap: u32,
) -> Vec<Option<[u32; 3]>> {
    let mut cur = counter + seed_gap;
    if comdat {
        // Measured exactly, on 11 TUs of 2 to 5 functions: the `/Gy` pre-pass is
        // three slots per function, whatever kind, and it is **not** affected by
        // floating point. Every row below is `packed + 3 * funcs.len()`.
        cur += 3 * funcs.len() as u32;
    }
    // **One extra slot for the TU's first FP-touching function** — the `_fltused`
    // external's slot, and the same field decides where that symbol goes
    // (`Function::is_float`), so the two are one fact and cannot drift.
    //
    // This corrects a rule that was wrong from two FP functions on. It read
    // "anything that touches floating point consumes 2", which fits one FP
    // function and predicts 4 slots for two where c2 gives 3, and 6 for three
    // where c2 gives 4. Measured seed-free as the *difference* between two framed
    // functions' labels in one TU, so nothing depends on the `.gl` seed; the
    // table is on `c2_il::IlFunction::label_slots`.
    //
    // This `+1` was once explained as "one slot per TU-level external", the same
    // rule as `docs/CODEGEN_FRAMED_CALLS.md` §4.4's `__savegprlr_N`/
    // `__restgprlr_N` pair consuming two slots for its two externals.
    // **The explanation is refuted** (`docs/LABEL_COUNTER.md` §2.1): a pooled FP
    // constant costs +2 and mints no external, a string literal costs 0 and
    // mints one. The `+1` and the `+2` are both still exact — see §1.1 for the
    // surcharge table that actually fits — but no new class may be added here on
    // the strength of counting its externals.
    let mut fltused_slot_taken = !funcs.iter().any(|f| f.is_float);
    // **W-IFN — one extra slot for the TU's first `memcpy`-minting function**,
    // the same shape as the `_fltused` slot above and measured the same way:
    // seed-free, as in-TU strides, on three cells at the workload's own flags
    // (`work/w-ifn/probe/lab_{x,y,z}.cpp`).
    //
    // ```text
    //   [framed, sub(memcpy)]                            stride 6
    //   [framed, sub1(memcpy), sub2(memcpy), framed]     strides 6, 5, 5
    // ```
    //
    // so the charge is per TU and not per function — which is also what
    // `src/xdk/nuispeech/mmio.cpp`'s own obj says, its two `memcpy` users
    // sitting at `$M3381` and `$M3386`, five apart.
    //
    // **The third cell is the one worth carrying**: `[sub(memcpy), framed]`
    // reads stride **5**, because a slot taken before the FIRST function's own
    // triple moves that function's labels and every later one's equally and is
    // therefore invisible to every in-TU stride. This rule was measured wrong by
    // exactly that cell before the differential caught it — see
    // [`super::Function::mints_memcpy`].
    let mut memcpy_slot_taken = !funcs.iter().any(|f| f.mints_memcpy);
    // **W-BIQUAD — `+2` per newly pooled FP constant**, `docs/LABEL_COUNTER.md`
    // §1.1's fourth surcharge row: *"a newly pooled FP constant — each distinct
    // `(bits,width)` first introduced — **+2**"*, measured on `const2-led` at
    // both `/Gy` and `/O x` (§1.2: *"every surcharge is byte-for-byte the same
    // integer"*).
    //
    // **It has been unobservable until now, and that is why it was not here.**
    // Only a FRAMED function has labels, so a surcharge taken by a leaf is
    // visible only when a framed function follows it in the same TU. Every
    // pool-bearing obj this port has emitted was leaves alone
    // (`w13b_fconst.cpp`, `w13b_fdedup.cpp`, `w13b_fpool.cpp`), where the whole
    // counter is dead. `Biquad.cpp` is the first TU with both: a two-pool leaf
    // and then a framed constructor, and without this the constructor's triple
    // came out `$M2570`/`$M2571`/`$T2572` against the reference's
    // `$M2574`/`$M2575`/`$T2576` — **exactly four low, which is 2 + 2**.
    //
    // TU-wide first-introduction, deduped on the same `(bits, double)` key the
    // writer pools on and read off the same `fp_refs` list, so the surcharge and
    // the `.rdata` section it pays for cannot disagree about which constants are
    // new. §1.1's last row — *"a helper width / FP constant an earlier function
    // already introduced: **0**, at any count"* — is that dedup.
    //
    // **Order against `_fltused` is NOT determined by any capture here.** In
    // `Biquad.cpp` both surcharges are taken by the same function, which is also
    // the first, so every later label moves by their SUM and no obj separates
    // them. Stated rather than left implicit.
    let mut pooled: Vec<(u64, bool)> = Vec::new();
    funcs
        .iter()
        .map(|f| {
            if f.is_float && !fltused_slot_taken {
                fltused_slot_taken = true;
                cur += 1;
            }
            if f.mints_memcpy && !memcpy_slot_taken {
                memcpy_slot_taken = true;
                cur += 1;
            }
            for r in &f.fp_refs {
                let key = (r.bits, r.double);
                if !pooled.contains(&key) {
                    pooled.push(key);
                    cur += 2;
                }
            }
            // **The leading surcharge is taken before the function's own triple**,
            // so it moves this function's `$M` numbers as well as every later
            // one's. Measured seed-free and in-TU (`scripts/gt_cmp_rr.py
            // --stride`, with the in-TU anchor control holding on every row):
            // a signed `>`/`<` two-call comparator is stride 7 / lead 2 under
            // `/Gy` and 6 / 2 packed, against 5 / 0 and 4 / 0 for its `==`,
            // unsigned and arithmetic-tailed siblings. Same shape as the
            // `__savegprlr_N` pair's, from `docs/LABEL_COUNTER.md` §1.1's
            // surcharge table and not from counting anything's externals — the
            // rule that once explained the `+1` above is refuted.
            cur += f.label_lead;
            match f.frame {
                Some(_) => {
                    let n = cur;
                    cur += if comdat { 5 } else { 4 };
                    Some([n, n + 1, n + 2])
                }
                None => {
                    cur += 1;
                    None
                }
            }
        })
        .collect()
}

/// Render a compiler label name (`$M2545`, `$T2547`). Kept as one function so
/// the 8-byte short-name limit is checked in one place: the numbers observed run
/// to four digits, and a five-digit counter would still fit (`$M12345`).
pub(crate) fn label_name(prefix: char, n: u32) -> String {
    format!("${prefix}{n}")
}

// `emit_framed_obj` used to live here: a second whole-obj emitter for the one
// single-function framed TU, with a hardcoded 20-symbol table and the label
// names `$M2545/$M2546/$T2547` written out literally. It is gone. A framed
// function is now a `Function` with a `frame`, and the same two emitters
// (`emit_obj` packed, `emit_comdat_obj` under `/Gy`) build every obj — because
// this file already carries two bugs whose whole cause was one rule
// implemented in two emitters and fixed in one.

/// Emit the `$T…` label that sits on a `.pdata` record. Same shape as
/// [`emit_label_symbol`] but storage class **3 (STATIC)**, not 6 (LABEL) — a
/// one-byte difference between two symbols emitted four slots apart, and the
/// reason this is its own function rather than a boolean argument.
pub(crate) fn emit_pdata_label_symbol(b: &mut Buf, name: &str, value: u32, sec_num: i16) {
    b.name8(name);
    b.u32(value);
    b.i16(sec_num);
    b.u16(0x0000); // Type
    b.u8(3); // IMAGE_SYM_CLASS_STATIC
    b.u8(0); // no aux
}

/// Emit a compiler-generated **label** symbol (storage class 6, no aux) with an
/// inline short name, e.g. `$M2545`/`$M2546`. `value` is its `.text` offset.
pub(crate) fn emit_label_symbol(b: &mut Buf, name: &str, value: u32, sec_num: i16) {
    b.name8(name);
    b.u32(value);
    b.i16(sec_num);
    b.u16(0x0000); // Type
    b.u8(6); // IMAGE_SYM_CLASS_LABEL
    b.u8(0); // no aux
}

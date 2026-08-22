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
//! per-function counter `DAT_10c2e918` reset in `FUN_10b7e113`. Read **R3**
//! (2–4 d) enumerates those 31 + 132 sites, which makes the charge rule
//! **closed by construction** — one increment instruction means the
//! enumeration is exhaustive, not a sample — and replaces both fitted numbers
//! with named identities.
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
pub const LABEL_SEED_GAP: u32 = 9;

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
    let mut cur = counter + LABEL_SEED_GAP;
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

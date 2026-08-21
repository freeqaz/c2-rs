//! **The static-array scan loop** — `?NextHashPrime@@YAHH@Z`'s shape, and the
//! first body this port emits that **references a data object the same TU
//! defines**.
//!
//! ```c
//!   int P(int i) {
//!       static int a[N] = { …, 0 };
//!       for (int j = 0; a[j] != 0; j++)
//!           if (a[j] >= i)
//!               return a[j];
//!       return i;
//!   }
//! ```
//!
//! # This is a TRANSCRIPTION, and saying so is the point
//!
//! `super::ptr_walk_loop`'s standard, unchanged: one named function class,
//! `/O1` only, `NotImplemented` outside. Every register number and every block
//! displacement below was **read off `c2`'s own output** — the obj for
//! `src/system/math/Primes.cpp` at the workload's own flags — and this module
//! *rebuilds* them from [`super::encode`] rather than storing a byte array.
//! What it does not do is derive them from a register allocator, because there
//! is no allocator here and pretending otherwise would be the claim
//! `super::frontier_bytes`' header spends a page refusing.
//!
//! **The class has ZERO free immediate fields**, which is the sharpest thing
//! about it and the reason the fence has to do all the work. `ptr_walk_loop`
//! has two (`K0` and `K`); `if_call_join` has two; this one has **none** —
//! sixteen words, every field fixed, and the only thing that varies across the
//! class is the **object**: its symbol name, its size, its alignment and its
//! 248 bytes of initializer. So a body that matches the recognizer emits these
//! exact sixty-four bytes, and any body that would need a different word must
//! be refused by the *reader*, never bent to fit here.
//!
//! # Where this came from
//!
//! `super::frontier_bytes::primes_next_hash_prime_text` is the same sixteen
//! words, built the same way, under `cfg(test)`. That file states its own status
//! plainly — *"it converts no TU, moves no numerator and appears in no accept
//! path"* — and this module is what gives it a caller. The two are kept as one
//! text rather than two: `frontier_bytes`' tests assert **this** builder against
//! `C2_TEXT`, the sixty-four bytes transcribed from real `c2`, so the
//! measurement and the emitter cannot come apart.
//!
//! # The label counter
//!
//! Same answer as `ptr_walk_loop`'s, for the same measured reason
//! (`super::labels`' header, `w-loop`'s Q1/Q2): a **leaf** loop charges the
//! compiler-label counter `+1..+4`, `plan_labels` charges 0, and the charge is
//! unobservable in a TU with no framed function because `$M`/`$T` short names
//! are the only channel to the obj. So [`c2_il::IlFunction::label_slots`]
//! returns `None` for this shape too, and `IlBundle::functions` refuses any TU
//! that pairs one with a framed function.
//!
//! `Primes.cpp` is `label-free`: its one emitted function is this class, and
//! the obj carries no `$M` or `$T` at all.
//!
//! # `super::labels` invariant 4 is NOT relaxed
//!
//! The body has a **backward** branch (`bf 26,.-24` at `0x2c`, to the loop top
//! at `0x14`) and the label map refuses one. This module never asks it: the
//! displacement is computed from the block layout directly through
//! [`super::encode::encode_bc`], exactly as `ptr_walk_loop` and
//! `ptr_walk_chain_loop` already do. `frontier_bytes`'
//! `the_back_edge_is_still_a_backward_reference_the_label_map_refuses` still
//! passes, with its forward-reference positive control, and is the pin that
//! this route was not opened by widening that one.

use super::encode::{
    BO_FALSE, CR_BIT_EQ, CR_BIT_LT, CR_COMPARE, cr_bi, encode_addi, encode_addis, encode_b_intra,
    encode_bc, encode_blr, encode_cmpw, encode_cmpwi, encode_lwz, encode_lwzx, encode_rlwinm,
};
use crate::BackendError;
use c2_il::IlFunction;

/// The five basic blocks, by `.text` byte offset. Named rather than inlined so
/// the three branch displacements are computed from the **layout** — the
/// rotation is then visible as ENTRY's branch going *forward* to `TEST`, which
/// sits *below* `TOP`.
const TOP: i32 = 0x14;
const BODY: i32 = 0x1c;
const TEST: i32 = 0x28;
const FALLOUT: i32 = 0x30;
const VALUE: i32 = 0x34;

/// The sixteen words, rebuilt from [`super::encode`].
///
/// Free-standing (it takes nothing) because the class has no immediate fields;
/// the caller supplies only the *fence*. Returns `None` never in practice — the
/// two `encode_b*` calls are in range by construction and the constants above
/// are what make that true — but the `?`s are kept rather than `expect`ed so a
/// future edit to a block offset produces a refusal and not a panic.
pub(crate) fn static_scan_loop_words() -> Option<Vec<u8>> {
    let bit_lt = cr_bi(CR_COMPARE, CR_BIT_LT); // 24
    let bit_eq = cr_bi(CR_COMPARE, CR_BIT_EQ); // 26
    // `slwi rA,rS,2` is `rlwinm rA,rS,2,0,29`.
    let slwi2 = |ra: u8, rs: u8| encode_rlwinm(ra, rs, 2, 0, 29);

    let words: [[u8; 4]; 16] = [
        // -- ENTRY: materialize &a, peel a[0], rotate into the test ----------
        encode_addis(10, 0, 0), // lis  r10,0        + REFHI
        encode_addi(11, 0, 0),  // li   r11,0        j = 0
        encode_addi(9, 10, 0),  // addi r9,r10,0     + REFLO   r9 = &a
        encode_lwz(10, 10, 0),  // lwz  r10,0(r10)   + REFLO   r10 = a[0]
        encode_b_intra(TEST - 0x10)?,
        // -- TOP -------------------------------------------------------------
        encode_cmpw(CR_COMPARE, 10, 3),
        encode_bc(BO_FALSE, bit_lt, VALUE - 0x18)?,
        // -- BODY ------------------------------------------------------------
        encode_addi(11, 11, 1), // j++
        slwi2(10, 11),
        encode_lwzx(10, 10, 9), // r10 = a[j]
        // -- TEST (the rotation's landing pad) -------------------------------
        encode_cmpwi(CR_COMPARE, 10, 0),
        // the BACK EDGE
        encode_bc(BO_FALSE, bit_eq, TOP - 0x2c)?,
        // -- FALLOUT ---------------------------------------------------------
        encode_blr(),
        // -- VALUE: c2 recomputes what r10 already holds ---------------------
        //
        // **The rematerialization is the class, not a peephole to remove.**
        // `r10` already holds `a[j]` at every entry to this block, and c2 still
        // spends two words recomputing it. Board **#1400** recorded exactly this
        // on this function and wrote it as a warning; `negate_test.cpp`'s
        // `mr r3,r3` is the second instance and w-cfgclass made that one a
        // `#[test]`. Collapsing either is six wrong bytes in an obj that links.
        slwi2(11, 11),
        encode_lwzx(3, 11, 9),
        encode_blr(),
    ];
    let mut out = Vec::with_capacity(64);
    for w in words {
        out.extend_from_slice(&w);
    }
    debug_assert_eq!(out.len(), 64);
    debug_assert_eq!(BODY, 0x1c, "the block table is read by the displacements above");
    debug_assert_eq!(FALLOUT, 0x30);
    Some(out)
}

/// The body for `func`, or `None` if it is not this class.
///
/// A three-valued reading of "is this the class" is deliberately **not**
/// offered: the only fact that decides it is `func.static_scan_loop`, set by
/// exactly one parser production.
pub(crate) fn static_scan_loop_text(func: &IlFunction) -> Option<Vec<u8>> {
    func.static_scan_loop()?;
    static_scan_loop_words()
}

/// The selector's arm. `mode` is the per-function optimization word.
///
/// # The mode fence is asked HERE and in the PARSER, and that is not a
/// duplication
///
/// Board **#1638** / w-cfgclass §5.3: a gate that lives only in the emitter is
/// a fact the **census** cannot ask, so the census counts a function in class
/// that `PortC2` refuses — `docs/GAPS.md` §6's one-fact-two-locators defect, and
/// `crates/c2-harness/tests/census_gate.rs` fails on it in the words it was
/// written to fail in. The recognizer asks the optimization word **first**,
/// before any body byte is read. This clause stays because `select_function` is
/// what `function_gate` runs, and a shape arriving here under the wrong mode
/// must refuse rather than emit.
///
/// `/Ox`, `/O2` and `/Od` are refused and **not measured**: this lane graded the
/// class at the workload's own `/O1` and nowhere else, and `/O1` is the only
/// mode `Primes.cpp` is ever compiled at. A lane that wants another needs its
/// own cells — `ptr_walk_loop` carries the identical clause for the identical
/// reason.
pub fn static_scan_loop_emit(
    func: &IlFunction,
    mode: super::OptMode,
) -> Result<Vec<u8>, BackendError> {
    if mode != super::OptMode::O1 {
        return Err(BackendError::NotImplemented(
            "the static-array scan loop is graded at `/O1` only: the workload's \
             own mode and the only one any cell of this class was compiled at"
                .to_string(),
        ));
    }
    static_scan_loop_text(func).ok_or_else(|| {
        BackendError::NotImplemented(
            "not a static-array scan loop (the parser sets `static_scan_loop`)".to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The oracle pin.** The sixteen words this emitter builds are the
    /// sixty-four bytes real `c2.dll` emitted for `?NextHashPrime@@YAHH@Z`,
    /// transcribed in `super::frontier_bytes::C2_TEXT`.
    ///
    /// One text, two consumers: `frontier_bytes`' own tests assert the *same*
    /// builder, so the measurement and the emitter cannot drift.
    #[test]
    fn the_sixteen_words_are_the_bytes_real_c2_emitted() {
        let got = static_scan_loop_words().expect("in range");
        assert_eq!(
            got.as_slice(),
            &super::super::frontier_bytes::C2_TEXT[..],
            "the emitter must be byte-identical to c2's own `.text`"
        );
    }

    /// The rematerialization at `VALUE` is two words and is **not** the same
    /// pair as `BODY`'s: they write different registers (`r10` vs `r11`, `r10`
    /// vs `r3`). A peephole that unified them would change four bytes.
    #[test]
    fn the_value_block_rematerializes_into_different_registers() {
        let t = static_scan_loop_words().unwrap();
        assert_ne!(&t[0x20..0x28], &t[0x34..0x3c], "BODY and VALUE are distinct");
    }
}

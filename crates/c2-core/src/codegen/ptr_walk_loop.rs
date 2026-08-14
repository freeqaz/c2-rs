//! **The pointer-walk accumulate loop** — the port's first lowering with a
//! backward branch, and the whole of `src/system/math/Sort.cpp`.
//!
//! ```text
//!   0000  lbz    r11,0(r3)      the PEELED load: iteration 0's character,
//!   0004  mr     r9,r3          before the loop is entered at all
//!   0008  li     r10,<K0>
//!   000c  cmplwi cr0,r11,0
//!   0010  bt     2,+0x38        -> the exit
//!   0014  mulli  r8,r10,<K>     LOOP:
//!   0018  lbzu   r10,1(r9)      iteration k+1's character, and the induction
//!   001c  add    r8,r8,r11      the accumulate, over iteration k's character
//!   0020  mr.    r11,r10        carry k+1 forward AND set cr0 — the loop test
//!   0024  rotlwi r10,r8,1       \
//!   0028  divw   r7,r8,r4        |
//!   002c  addi   r10,r10,-1      |  the signed `%` with its two trap guards,
//!   0030  mullw  r7,r7,r4        |  the predicate INTERLEAVED between `divw`
//!   0034  andc   r6,r4,r10       |  and `mullw`
//!   0038  twi    6,r4,0          |
//!   003c  subf   r10,r7,r8       |
//!   0040  twi    5,r6,-1        /
//!   0044  bf     2,-0x30        the BACK EDGE, on cr0
//!   0048  mr     r3,r10
//!   004c  blr
//! ```
//!
//! # What is a rule here and what is a transcription
//!
//! **This is a transcription, and saying so is the point.** Twenty words, two
//! immediate fields, no free parameters. It is not a loop lowering — there is no
//! scheduler here, no register allocator, no CFG builder. It reproduces one
//! function class byte for byte because that class was measured byte for byte.
//!
//! The two immediates, and their axes' cross product, are graded against real
//! `c2` by `work/w-hash/hashgrid.py`:
//!
//! * `<K0>` — the accumulator's initial `li`, any `simm16`;
//! * `<K>` — the `mulli`'s multiplier, any positive `mulli`-eligible literal.
//!
//! Everything else in the twenty words is a constant of the class, including
//! both branch displacements: the body has a fixed length, so `+0x38` and
//! `-0x30` are not computed from a layout, they *are* the layout. They are still
//! emitted through [`encode_bc`] rather than as literal words, so a future body
//! of a different length cannot silently keep them.
//!
//! # Why the register assignment is not a rule either
//!
//! `c2` re-plans it the moment the class is left. Measured, one axis at a time
//! (`work/w-hash/hashgrid.py`):
//!
//! ```text
//!   base                     C=r11 U=r9  A=r10 N=r10   guard `bt`,  tail `mr r3,r10 ; blr`
//!   pointer formal at slot 1 C=r11 U=r11 A=r3  N=r9    guard `beqlr`, NO tail move
//!   a third formal ahead     C=r11 U=r10 A=r3  N=r9    guard `beqlr`, NO tail move
//!   the formal walked direct C=r11 U=r10 A=r3  N=r9    guard `beqlr`, NO tail move
//! ```
//!
//! Those are three different **block plans**, not three register fields: the
//! accumulator coalesces into `r3`, the guard becomes a `beqlr`, and the closing
//! move disappears. Fitting a rule to four cells is how ten placement rules have
//! already been refuted in this project (`w-pair` §4's six, `leaf_store.rs`'s
//! four), so the parser refuses every one of them by construction —
//! `params.len() == 2`, pointer at slot 0, divisor at slot 1 — and the cost is a
//! number rather than an argument.

use c2_il::PtrWalkModLoop;

use crate::codegen::cond::{producer_at, Cond, CR0};
use crate::codegen::encode::{
    encode_add, encode_addi, encode_andc, encode_bc, encode_blr, encode_cmplwi, encode_divw,
    encode_lbz, encode_lbzu, encode_mr, encode_mr_record, encode_mulli, encode_mullw,
    encode_rlwinm, encode_subf, encode_twi, BO_FALSE, BO_TRUE, CR_BIT_EQ,
};
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;

// The private `const CR_RECORD: u8 = 0;` that used to sit here is gone, and the
// **reason it had to go is inside this file** — lane `w-ir-e`, `CFG_SHAPE.md`
// §6.2 item **E**. Its doc said *"the `cr` field a RECORD-FORM producer
// writes"*, and it was correct about the back edge; but this class's entry guard
// is `cmplwi cr0,CHAR,0`, **an explicit compare**, and it used the same
// constant. One name, one number, two of §3.2's two producers — which is
// precisely the distinction item E exists to make. Both producers are now read
// off the emitted words by [`cond_source`], the field comes from the producer,
// and [`CR0`] names the register rather than one of the ways of reaching it.
// Board #188's defect is unchanged and still the thing being prevented.

/// The peeled character, live across the back edge.
const R_CHAR: u8 = 11;
/// The walked pointer.
const R_PTR: u8 = 9;
/// The accumulator.
const R_ACC: u8 = 10;
/// The accumulate's intermediate, and the modulo's dividend.
const R_DIVIDEND: u8 = 8;
/// The quotient.
const R_QUOT: u8 = 7;
/// The overflow predicate's result.
const R_OVF: u8 = 6;
/// The pointer formal, slot 0.
const R_SRC: u8 = 3;
/// The divisor formal, slot 1.
const R_DIV: u8 = 4;

/// Emit the whole body. Twenty words, no relocation, no pooled constant, no
/// label — so the caller takes it as an ordinary `Selected::Plain`.
pub(crate) fn ptr_walk_loop_text(
    l: &PtrWalkModLoop,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    // **`/O1` ONLY, and this is the sharpest refusal in the file.**
    //
    // `docs/OPT_MODE.md`'s register-field rule — the modes "differ in exactly
    // one rule … only a register field" — is already recorded there as REFUTED
    // once a body has more than one block. This class is the strongest witness
    // yet: the identical source at `/Ox` and `/O2` is **84 bytes and twenty-one
    // words**, against `/O1`'s 80 and twenty, and it is a different *body*, not
    // a different allocation (`work/w-hash/hashgrid.py --mode '/Ox /GS- /c'`):
    //
    // ```text
    //   /O1  … bc · mulli lbzu add mr.  · rotlwi divw addi mullw andc twi subf twi · bc · mr blr
    //   /Ox  … bclr· rlwinm lbzu twi subf add mr. · rotlwi divw addi mullw andc subf twi · cmpli bc bclr
    // ```
    //
    // `/Ox` strength-reduces `ret * 127` into `rlwinm` + `subf` where `/O1`
    // emits one `mulli`, hoists the zero-divisor `twi` to the third slot, and
    // closes the loop on an explicit `cmpli` instead of the record form — so the
    // branch reads a *different condition register field*. Emitting the `/O1`
    // body under `/Ox` would be four wrong words and a wrong `BI`; refusing is
    // the only honest answer this rung has, and widening it means grading the
    // `/Ox` body as its own measured class.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "ptr-walk loop outside /O1: /Ox and /O2 emit a different 84-byte body \
             (strength-reduced multiply, hoisted trap, `cmpli` loop close). See \
             `codegen::ptr_walk_loop`.",
        ));
    }
    // Re-asserted here even though `try_parse_ptr_walk_loop` already required
    // both: `select_function` is what `function_gate` runs, so a shape that
    // reached codegen with a different arity would be a census/gate
    // disagreement, and that counter reading 0 is the only thing keeping the
    // census honest about what the port accepts.
    if l.params.len() != 2 {
        return Err(out_of_class(
            "ptr-walk loop with other than two formals: the register plan is measured at two",
        ));
    }
    let k0 = i16::try_from(l.acc_init).map_err(|_| {
        out_of_class("ptr-walk loop accumulator init outside simm16 (a `lis`/`ori` pair)")
    })?;
    let k = i16::try_from(l.mul_k)
        .map_err(|_| out_of_class("ptr-walk loop multiplier outside simm16"))?;

    let mut t: Vec<u8> = Vec::with_capacity(80);
    // --- the entry guard, over the PEELED character --------------------------
    t.extend_from_slice(&encode_lbz(R_CHAR, R_SRC, 0));
    t.extend_from_slice(&encode_mr(R_PTR, R_SRC));
    t.extend_from_slice(&encode_addi(R_ACC, 0, k0));
    t.extend_from_slice(&encode_cmplwi(CR0, R_CHAR, 0));
    // Filled once the body's length is known; asserted below against the
    // measured constant rather than trusted.
    let guard_at = t.len();
    t.extend_from_slice(&[0; 4]);

    let loop_top = t.len();
    // --- the accumulate ------------------------------------------------------
    t.extend_from_slice(&encode_mulli(R_DIVIDEND, R_ACC, k));
    t.extend_from_slice(&encode_lbzu(R_ACC, R_PTR, 1));
    t.extend_from_slice(&encode_add(R_DIVIDEND, R_DIVIDEND, R_CHAR));
    t.extend_from_slice(&encode_mr_record(R_CHAR, R_ACC));
    // --- the signed `%`, with the trap predicate interleaved -----------------
    //
    // The order is `c2`'s and reproduces in a **straight-line** leaf as well as
    // here: `int P(int a,int b){ return a%b; }` emits these same eight words in
    // this same order (`work/w-hash/divgrid.py`, row `s-mod-var`), which is why
    // refusal "the schedule" is a property of the modulo lowering and not of the
    // loop. It is *not* universal — the same expression with a computed dividend
    // in a straight-line body hoists `twi 6` to the front — so the order is
    // emitted as measured for this class and claimed for no other.
    t.extend_from_slice(&encode_rlwinm(R_ACC, R_DIVIDEND, 1, 0, 31));
    t.extend_from_slice(&encode_divw(R_QUOT, R_DIVIDEND, R_DIV));
    t.extend_from_slice(&encode_addi(R_ACC, R_ACC, -1));
    t.extend_from_slice(&encode_mullw(R_QUOT, R_QUOT, R_DIV));
    t.extend_from_slice(&encode_andc(R_OVF, R_DIV, R_ACC));
    t.extend_from_slice(&encode_twi(6, R_DIV, 0));
    t.extend_from_slice(&encode_subf(R_ACC, R_QUOT, R_DIVIDEND));
    t.extend_from_slice(&encode_twi(5, R_OVF, -1));
    // --- the BACK EDGE, on cr0 ----------------------------------------------
    //
    // The producer is `mr. CHAR,ACC` nine words above — a RECORD FORM — and the
    // field is read off it rather than named again here (§6.2 item E). The scan
    // walks back over `twi`, `subf`, `andc`, `mullw`, `addi`, `divw`, `rlwinm`
    // and `add`, none of which touches a condition register, and stops at the
    // first word that does.
    let back_at = t.len();
    let back_cond = Cond::new(producer_at(&t[..back_at], "ptr-walk loop back edge")?, BO_FALSE, CR_BIT_EQ);
    let back = encode_bc(
        back_cond.bo(),
        back_cond.bi(),
        loop_top as i32 - back_at as i32,
    )
    .ok_or_else(|| out_of_class("ptr-walk loop back edge past the `bc` field"))?;
    t.extend_from_slice(&back);
    // --- the exit ------------------------------------------------------------
    let exit_at = t.len();
    t.extend_from_slice(&encode_mr(3, R_ACC));
    t.extend_from_slice(&encode_blr());

    // The guard's producer is the `cmplwi cr0` immediately above its own site —
    // a COMPARE, and not into cr6. The two branches of this one class are
    // §3.2's two producers, which is why one constant could never have said
    // both.
    let guard_cond = Cond::new(producer_at(&t[..guard_at], "ptr-walk loop entry guard")?, BO_TRUE, CR_BIT_EQ);
    let guard = encode_bc(
        guard_cond.bo(),
        guard_cond.bi(),
        exit_at as i32 - guard_at as i32,
    )
    .ok_or_else(|| out_of_class("ptr-walk loop entry guard past the `bc` field"))?;
    t[guard_at..guard_at + 4].copy_from_slice(&guard);

    debug_assert_eq!(t.len(), 80, "the class's body length is a constant");
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loop_of(acc_init: i32, mul_k: i32) -> PtrWalkModLoop {
        PtrWalkModLoop { params: vec![0x09EA, 0x09EB], acc_init, mul_k }
    }

    /// **`?HashString@@YAHPBDH@Z`, word for word**, transcribed from
    /// `work/w-hash/Sort.obj` — the reference obj for
    /// `src/system/math/Sort.cpp` at the dc3 workload's own
    /// `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …`, and byte-identical at
    /// `/O1 /GS- /c` (`work/w-hash/loopshape.py`'s anchor control).
    #[test]
    fn hash_string_is_reproduced_word_for_word() {
        let want: [u32; 20] = [
            0x8963_0000, // lbz    r11,0(r3)
            0x7c69_1b78, // mr     r9,r3
            0x3940_0000, // li     r10,0
            0x280b_0000, // cmplwi cr0,r11,0
            0x4182_0038, // bt     2,+56
            0x1d0a_007f, // mulli  r8,r10,127
            0x8d49_0001, // lbzu   r10,1(r9)
            0x7d08_5a14, // add    r8,r8,r11
            0x7d4b_5379, // mr.    r11,r10
            0x550a_083e, // rotlwi r10,r8,1
            0x7ce8_23d6, // divw   r7,r8,r4
            0x394a_ffff, // addi   r10,r10,-1
            0x7ce7_21d6, // mullw  r7,r7,r4
            0x7c86_5078, // andc   r6,r4,r10
            0x0cc4_0000, // twi    6,r4,0
            0x7d47_4050, // subf   r10,r7,r8
            0x0ca6_ffff, // twi    5,r6,-1
            0x4082_ffd0, // bf     2,-48
            0x7d43_5378, // mr     r3,r10
            0x4e80_0020, // blr
        ];
        let got = ptr_walk_loop_text(&loop_of(0, 127), OptMode::O1).unwrap();
        let words: Vec<u32> = got
            .chunks_exact(4)
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        assert_eq!(words, want, "the reference obj's own twenty words");
    }

    /// **This class's two branches have two DIFFERENT producers, and one
    /// constant could not say so** — `CFG_SHAPE.md` §6.2 item **E**, lane
    /// `w-ir-e`.
    ///
    /// The entry guard at 0x10 reads the explicit `cmplwi cr0,r11,0` above it —
    /// an *explicit compare*, into cr0 and not into `CR_COMPARE`'s cr6. The back
    /// edge at 0x44 reads `mr. r11,r10` — a *record form*, nine words earlier,
    /// with eight condition-register-silent words in between. Both write cr0,
    /// which is exactly why the private `CR_RECORD` constant this file used to
    /// carry survived: it was right about the field for the whole life of the
    /// class and wrong about half of the producers.
    #[test]
    fn the_entry_guard_and_the_back_edge_read_two_different_producers() {
        use crate::codegen::cond::{cond_source, CondProducer, CondSource};
        const GUARD_AT: usize = 0x10;
        const BACK_AT: usize = 0x44;
        let t = ptr_walk_loop_text(&loop_of(0, 127), OptMode::O1).unwrap();
        assert_eq!(t.len(), 80);
        // The two branch words really are at those offsets — the offsets are
        // asserted, not assumed, so this test cannot silently scan the wrong run.
        assert_eq!(&t[GUARD_AT..GUARD_AT + 4], &[0x41, 0x82, 0x00, 0x38]); // bt 2,+56
        assert_eq!(&t[BACK_AT..BACK_AT + 4], &[0x40, 0x82, 0xff, 0xd0]); // bf 2,-48

        assert_eq!(
            cond_source(&t[..GUARD_AT]),
            CondSource::InBlock(CondProducer::Compare { crf: 0 }),
            "the entry guard's producer is an explicit compare into cr0"
        );
        assert_eq!(
            cond_source(&t[..BACK_AT]),
            CondSource::InBlock(CondProducer::RecordForm),
            "the back edge's producer is `mr.`, a record form"
        );
        // Different producers; same field. That pair is the whole of item E.
        assert_ne!(
            cond_source(&t[..GUARD_AT]),
            cond_source(&t[..BACK_AT])
        );
        assert_eq!(t[GUARD_AT + 1] & 0xfc, t[BACK_AT + 1] & 0xfc, "both BI = 2");
    }

    /// The two immediate fields are the **only** things that move, over the axes
    /// `work/w-hash/hashgrid.py` graded against real `c2`. Everything else —
    /// both displacements included — is a constant of the class, and a change
    /// that made any other word depend on `K` or `K0` would fail here before it
    /// reached an obj.
    #[test]
    fn only_the_two_immediates_move_across_the_graded_axes() {
        let base = ptr_walk_loop_text(&loop_of(0, 127), OptMode::O1).unwrap();
        for (k0, k) in [
            (0, 3),
            (0, 5),
            (0, 31),
            (0, 1000),
            (0, 32767),
            (1, 127),
            (7, 127),
            (-1, 127),
            (1000, 127),
            (-1, 32767),
        ] {
            let got = ptr_walk_loop_text(&loop_of(k0, k), OptMode::O1).unwrap();
            assert_eq!(got.len(), 80);
            for (i, (a, b)) in base.chunks_exact(4).zip(got.chunks_exact(4)).enumerate() {
                // word 2 is `li r10,K0`, word 5 is `mulli r8,r10,K`
                if i == 2 || i == 5 {
                    continue;
                }
                assert_eq!(a, b, "word {i} moved with (K0={k0}, K={k})");
            }
            assert_eq!(&got[8..12], &encode_addi(R_ACC, 0, k0 as i16));
            assert_eq!(&got[20..24], &encode_mulli(R_DIVIDEND, R_ACC, k as i16));
        }
    }

    /// The arity gate is re-asserted in codegen, so a shape that ever reached
    /// here with a different formals list refuses rather than emitting the
    /// two-formal register plan over three registers.
    /// `/Ox` and `/O2` emit a **different body** for this source, so the shape
    /// refuses outside `/O1` rather than emitting four wrong words and a wrong
    /// branch condition register.
    #[test]
    fn ox_refuses_because_c2_emits_a_different_body_there() {
        assert!(ptr_walk_loop_text(&loop_of(0, 127), OptMode::O1).is_ok());
        assert!(ptr_walk_loop_text(&loop_of(0, 127), OptMode::Ox).is_err());
    }

    #[test]
    fn a_different_arity_refuses_in_codegen_too() {
        let mut l = loop_of(0, 127);
        l.params = vec![0x09EA];
        assert!(ptr_walk_loop_text(&l, OptMode::O1).is_err());
        l.params = vec![0x09EA, 0x09EB, 0x09EC];
        assert!(ptr_walk_loop_text(&l, OptMode::O1).is_err());
    }
}

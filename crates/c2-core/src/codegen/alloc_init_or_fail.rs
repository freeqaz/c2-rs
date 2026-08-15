//! **W-UNDNAME — the emitter for a guarded allocation with a shared error
//! store.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::alloc_init_or_fail`]; this file is the
//! twenty-four words and nothing else. Everything variable in them is named in
//! [`c2_il::AllocInitOrFailFn`]: three names and **ten** immediates.
//!
//! ```text
//!    off  word       instruction               why it is this word
//!   ----  --------   -----------------------   ---------------------------------
//!   0x00  7d8802a6   mflr  r12                 FrameLayout{saved_gprs:2}: 112
//!   0x04  9181fff8   stw   r12,-8(r1)          bytes and two callee-saved GPRs,
//!   0x08  fbc1ffe8   std   r30,-24(r1)         byte for byte what the shipped
//!   0x0c  fbe1fff0   std   r31,-16(r1)         `FrameLayout` emits at 2 — the
//!   0x10  9421ff90   stwu  r1,-112(r1)         frame is free, and `out_slots`
//!                                              is 3 because the call takes the
//!                                              object's address plus two args
//!
//!   0x14  7c7f1b78   mr    r31,r3              ┐ BOTH parks. `this` is stored
//!   0x18  7c9e2378   mr    r30,r4              ┘ through at 0x60 and 0x70 and
//!                                              `node` at 0x44, all after a `bl`
//!                                              has clobbered every volatile —
//!                                              which is why the frame saves two
//!   0x1c  2b040000   cmplwi cr6,r4,0           `node != 0`, on cr6 …
//!   0x20  419a004c   bt    26,-> Lerr
//!   0x24  3d600000   lis   r11,0        REFHI  ┐ the OBJECT's address. Hoisted
//!   0x28  38a00000   li    r5,<k_flag>         │ TWO words above its low half,
//!   0x2c  386b0000   addi  r3,r11,0     REFLO  │ with the third argument's `li`
//!   0x30  38800010   li    r4,<k_size>         ┘ between them — and the `addi`
//!                                              takes r3's slot because the
//!                                              object's address IS the member
//!                                              call's `this`
//!   0x34  4bxxxxxx   bl    <alloc>      REL24
//!   0x38  28030000   cmplwi cr0,r3,0           … `p != 0` on **cr0** …
//!   0x3c  41820024   bt    2,-> Llink
//!   0x40  3d600000   lis   r11,0        REFHI  ┐ the VTABLE's address, hoisted
//!   0x44  93c30008   stw   r30,A(r3)           │ THREE words this time — no
//!   0x48  3940ffff   li    r10,<k_neg>         │ fixed hoist distance holds even
//!   0x4c  396b0000   addi  r11,r11,0   REFLO   ┘ inside one body, which is why
//!                                              `data_refs_of` pairs positionally
//!                                              — and this low half writes the
//!                                              SCRATCH register itself
//!   0x50  9143000c   stw   r10,B(r3)
//!   0x54  91630000   stw   r11,C(r3)           the vtable pointer
//!   0x58  817f0000   lwz   r11,D(r31)          ┐ the list splice: the old head
//!   0x5c  91630004   stw   r11,E(r3)           ┘ becomes the new node's `next`
//!  Llink:
//!   0x60  907f0000   stw   r3,D(r31)           …and the new node becomes the
//!                                              head. REACHED FROM BOTH PATHS —
//!                                              a null `p` is stored too
//!   0x64  2b030000   cmplwi cr6,r3,0           … and `p == 0` on cr6 again
//!   0x68  409a000c   bf    26,-> epilogue
//!  Lerr:
//!   0x6c  39600003   li    r11,<k_status>      ┐ the error block, reached THREE
//!   0x70  997f0004   stb   r11,F(r31)          ┘ ways and SUNK to the end. A
//!                                              BYTE store: a `stw` here writes
//!                                              three neighbouring fields to zero
//!   0x74  38210070   addi  r1,r1,112           the epilogue, also free
//!   …     …          lwz/mtlr/ld r30/ld r31/blr
//! ```
//!
//! **Zero words are chosen by a scheduler or a register allocator.** The two
//! hoists are constants of the class, not the output of a list scheduler — which
//! is exactly why the reader pins the statement count and the argument arity:
//! anything that would move a hoist is refused before it reaches here (board
//! **#1706**).
//!
//! Every branch here is **self-relative** and therefore independent of where the
//! function lands in `.text`; only the one `bl` encodes its own offset, so it is
//! the only word that needs `base_off`.

use crate::codegen::block_ir::{BlockOrder, BodyLayout, Terminator};
use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::{
    cr_bi, encode_addi, encode_addis, encode_cmplwi, encode_lwz, encode_mr, encode_stb,
    encode_stw, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{fits_i16, out_of_class, ARG_REGS, RET_REG, SCRATCH_REG};
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::AllocInitOrFailFn;

/// The callee-saved register the RECEIVER is parked in — `this`, read at 0x58,
/// 0x60 and 0x70.
const THIS_REG: u8 = 31;
/// The callee-saved register the FORMAL is parked in — `node`, read at 0x44.
/// Two parks is what makes the frame `saved_gprs: 2`.
const NODE_REG: u8 = 30;
/// The volatile register the `k_neg` literal lives in for two words. Not the
/// scratch: r11 is holding the vtable's high half across exactly that span.
const TEMP_REG: u8 = 10;
/// `cr0`, which the MIDDLE of the three tests reads where the outer two read
/// `CR_COMPARE` (cr6). Nothing in the source distinguishes them and the emitter
/// has no way to vary it — see the class doc's fact 2.
const CR_MIDDLE: u8 = 0;

/// `li rD,k` — `addi rD,0,k`. The same two-line helper
/// [`super::guard_chain_shared_tail`] carries, and for the same reason:
/// `encode_addi` with `ra = 0` is `li` and spelling that at every call site
/// hides it.
fn encode_li(rd: u8, k: i16) -> [u8; 4] {
    encode_addi(rd, 0, k)
}

/// This class's emitted body: the bytes plus the offsets the writers need.
pub struct AllocInitOrFailBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offset of the one `bl` word.
    pub bl_offset: u32,
    /// Prologue length in bytes: the `$M(n)` label's value and the `.pdata`
    /// record's `PrologLen`.
    pub prolog_len: u32,
}

/// Emit the twenty-four words.
///
/// `base_off` is the function's own offset within `.text` — zero under `/Gy`,
/// where each function is its own COMDAT. It reaches only the `bl`.
pub fn alloc_init_or_fail_text(
    a: &AllocInitOrFailFn,
    base_off: u32,
    mode: OptMode,
) -> Result<AllocInitOrFailBody, BackendError> {
    // **`/O1` only.** The reader asks this first, before any body byte is read
    // (board #1638); this is the emitter's own copy, kept for the reason
    // `guard_chain_shared_tail` keeps its: the two must not be able to disagree
    // silently, and `select_function` is what `function_gate` runs.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "a guarded allocation with a shared error store at /Ox or /O2: the \
             error block is reached from three places and its duplication \
             threshold has not been fitted (board row X-b)",
        ));
    }
    // Range-checked here as well as in the reader, because this is where a
    // truncation would happen: each of these lands in one signed 16-bit field.
    for k in [a.k_size, a.k_flag, a.k_neg, a.k_status, a.off_a, a.off_b, a.off_c, a.off_d, a.off_e,
        a.off_f]
    {
        if !fits_i16(k) {
            return Err(out_of_class(
                "a guarded allocation whose literal or member offset is outside simm16",
            ));
        }
    }
    if a.params.len() != 2 {
        return Err(out_of_class(
            "a guarded allocation without exactly `this` and one formal: the two \
             parks and the argument setup are both functions of that arity",
        ));
    }

    // `out_slots` 3 — the object's address plus the two literal arguments. It is
    // what makes `size()` 112 rather than 96, and the reference's `stwu` is the
    // check.
    let frame = FrameLayout {
        saved_gprs: 2,
        out_slots: 3,
        ..Default::default()
    };
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;

    // **The body is six blocks in `BodyLayout`** — `CFG_SHAPE.md` §6.2 item A,
    // board **#3124**. Nothing below computes a displacement: the three the
    // class needs are `LabelMap`'s (item B, #290), derived from where the blocks
    // landed. The two positions this class publishes — the `bl`'s `REL24` site
    // and the prologue's length — are stated as offsets **inside one block's own
    // run**, which is the property that makes them survive a re-layout.
    let mut l = BodyLayout::new(BlockOrder::IlStatement);
    let b_entry = l.declare("entry");
    let b_call = l.declare("call");
    let b_init = l.declare("init");
    let b_link = l.declare("link");
    let b_err = l.declare("err");
    let b_epi = l.declare("epilogue");

    // ---- the entry block: the prologue, both parks, then the `node != 0`
    //      guard, whose `bc` names the error block ---------------------------
    let mut run = prologue;
    run.extend_from_slice(&encode_mr(THIS_REG, ARG_REGS[0]));
    run.extend_from_slice(&encode_mr(NODE_REG, ARG_REGS[1]));
    run.extend_from_slice(&encode_cmplwi(CR_COMPARE, ARG_REGS[1], 0));
    l.place(
        b_entry,
        run,
        Terminator::Bc {
            bo: BO_TRUE,
            bi: cr_bi(CR_COMPARE, CR_BIT_EQ),
            taken: b_err,
        },
    )?;

    // ---- the call: the object's address into r3, two literals above it -----
    //
    // The `lis` is emitted TWO words above its `addi`, with `li r5` between
    // them. Written out in this order rather than as "arguments descending with
    // the high half hoisted N" because N is 2 here and 3 at 0x40: there is no
    // rule at n = 1, so the class transcribes rather than generalizes.
    //
    // The `bl` goes down as a **zero placeholder**: its word encodes its own
    // `.text` offset (§3.3, board #191) and that offset is the layout's answer,
    // not a running count. `BL_IN_CALL` is where it sits in **this block's** run.
    const BL_IN_CALL: u32 = 16;
    let mut run = Vec::new();
    run.extend_from_slice(&encode_addis(SCRATCH_REG, 0, 0));
    run.extend_from_slice(&encode_li(ARG_REGS[2], a.k_flag as i16));
    run.extend_from_slice(&encode_addi(RET_REG, SCRATCH_REG, 0));
    run.extend_from_slice(&encode_li(ARG_REGS[1], a.k_size as i16));
    debug_assert_eq!(run.len() as u32, BL_IN_CALL);
    run.extend_from_slice(&[0; 4]);
    // `if (p != 0) { four stores }` — the middle test reads cr0, and it is the
    // last condition-register writer in this block's own run, which is what
    // `BodyLayout::place` checks the terminator against (item E, board #188).
    run.extend_from_slice(&encode_cmplwi(CR_MIDDLE, RET_REG, 0));
    l.place(
        b_call,
        run,
        Terminator::Bc {
            bo: BO_TRUE,
            bi: cr_bi(CR_MIDDLE, CR_BIT_EQ),
            taken: b_link,
        },
    )?;

    // ---- the initialisation the middle guard skips -------------------------
    let mut run = Vec::new();
    run.extend_from_slice(&encode_addis(SCRATCH_REG, 0, 0));
    run.extend_from_slice(&encode_stw(NODE_REG, RET_REG, a.off_a as i16));
    run.extend_from_slice(&encode_li(TEMP_REG, a.k_neg as i16));
    run.extend_from_slice(&encode_addi(SCRATCH_REG, SCRATCH_REG, 0));
    run.extend_from_slice(&encode_stw(TEMP_REG, RET_REG, a.off_b as i16));
    run.extend_from_slice(&encode_stw(SCRATCH_REG, RET_REG, a.off_c as i16));
    run.extend_from_slice(&encode_lwz(SCRATCH_REG, THIS_REG, a.off_d as i16));
    run.extend_from_slice(&encode_stw(SCRATCH_REG, RET_REG, a.off_e as i16));
    l.place(b_init, run, Terminator::FallThrough)?;

    // ---- the link store, reached from both paths ---------------------------
    let mut run = Vec::new();
    run.extend_from_slice(&encode_stw(RET_REG, THIS_REG, a.off_d as i16));
    run.extend_from_slice(&encode_cmplwi(CR_COMPARE, RET_REG, 0));
    l.place(
        b_link,
        run,
        Terminator::Bc {
            bo: BO_FALSE,
            bi: cr_bi(CR_COMPARE, CR_BIT_EQ),
            taken: b_epi,
        },
    )?;

    // ---- the error block, SUNK here from the middle of the IL body ---------
    let mut run = Vec::new();
    run.extend_from_slice(&encode_li(SCRATCH_REG, a.k_status as i16));
    run.extend_from_slice(&encode_stb(SCRATCH_REG, THIS_REG, a.off_f as i16));
    l.place(b_err, run, Terminator::FallThrough)?;

    // The epilogue's last word IS the return, so it is the terminator and not
    // four more bytes of run — `Terminator::Blr` emits exactly that word.
    l.place(b_epi, epilogue_run(&epilogue)?, Terminator::Blr)?;

    let body = l.finish()?;
    // ---- the two positions this class publishes ----------------------------
    //
    // Both are the LAYOUT's, and both are block-relative: the `bl` is 16 bytes
    // into the call block's own run, the prologue is the head of the entry
    // block's. A change to any block's length moves them together, and a
    // re-layout that inserted a word would move them without either line here
    // being edited — which is the whole of board #3124.
    let bl_at = body.at(b_call, BL_IN_CALL)?;
    let bl_offset = base_off + bl_at;
    debug_assert_eq!(body.start_of(b_entry)?, 0);
    let mut t = body.text;
    t[bl_at as usize..bl_at as usize + 4].copy_from_slice(&encode_call_branch(bl_offset));

    Ok(AllocInitOrFailBody { text: t, bl_offset, prolog_len })
}

/// An epilogue's run **without** its closing `blr`, which is
/// [`Terminator::Blr`]'s word and not part of the straight-line run.
///
/// Shared by every class in this crate that materialises a common epilogue and
/// lays its body out through [`BodyLayout`]. It refuses rather than trims blind:
/// an epilogue that does not end in `blr` is a different frame class, and
/// silently dropping its last word would emit a body four bytes short with a
/// return the frame never wrote.
pub(super) fn epilogue_run(epilogue: &[u8]) -> Result<Vec<u8>, BackendError> {
    let n = epilogue.len();
    if n < 4 || epilogue[n - 4..] != crate::codegen::encode::encode_blr()[..] {
        return Err(out_of_class(
            "a materialised epilogue whose last word is not `blr`: \
             `Terminator::Blr` is that word, and trimming a word that is not it \
             would emit a body short of its own return",
        ));
    }
    Ok(epilogue[..n - 4].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::encode::encode_bc;

    /// `?append@DName@@QAAXPAVDNameNode@@@Z`'s parse, as the reader produces it
    /// from the workload's own IL. Tokens are the capture's; the names are the
    /// obj's.
    fn undname() -> AllocInitOrFailFn {
        AllocInitOrFailFn {
            params: vec![0xa08, 0xa06],
            alloc: "?getMemory@_HeapManager@@QAAPAXHH@Z".into(),
            object: "?gHeapManager@@3V_HeapManager@@A".into(),
            vtable: "?pairNode_vtable@@3PAXA".into(),
            k_size: 16,
            k_flag: 0,
            k_neg: -1,
            k_status: 3,
            off_a: 8,
            off_b: 12,
            off_c: 0,
            off_d: 0,
            off_e: 4,
            off_f: 4,
        }
    }

    /// **The thirty-five words, against the real obj.**
    ///
    /// Transcribed from `work/w-extdata/ref/undname/dis.txt`. The `bl` at 0x34
    /// carries `-52` because the function is its own COMDAT under `/Gy` and the
    /// placeholder displacement is `-(offset of the branch word)`.
    const REF_TEXT: [u32; 35] = [
        0x7d8802a6, 0x9181fff8, 0xfbc1ffe8, 0xfbe1fff0, 0x9421ff90,
        0x7c7f1b78, 0x7c9e2378, 0x2b040000, 0x419a004c,
        0x3d600000, 0x38a00000, 0x386b0000, 0x38800010, 0x4bffffcd,
        0x28030000, 0x41820024,
        0x3d600000, 0x93c30008, 0x3940ffff, 0x396b0000,
        0x9143000c, 0x91630000, 0x817f0000, 0x91630004,
        0x907f0000, 0x2b030000, 0x409a000c,
        0x39600003, 0x997f0004,
        0x38210070, 0x8181fff8, 0x7d8803a6, 0xebc1ffe8, 0xebe1fff0, 0x4e800020,
    ];

    fn ref_bytes() -> Vec<u8> {
        REF_TEXT.iter().flat_map(|w| w.to_be_bytes()).collect()
    }

    #[test]
    fn the_body_is_the_reference_obj_word_for_word() {
        let b = alloc_init_or_fail_text(&undname(), 0, OptMode::O1).expect("in class");
        let want = ref_bytes();
        assert_eq!(b.text.len(), want.len(), "140 bytes");
        for (i, (g, w)) in b.text.chunks_exact(4).zip(want.chunks_exact(4)).enumerate() {
            assert_eq!(g, w, "word {i} at .text+{:#04x}", i * 4);
        }
        assert_eq!(b.prolog_len, 0x14, "the $M(n) label and the .pdata PrologLen");
        assert_eq!(b.bl_offset, 0x34);
    }

    /// **The middle test reads a DIFFERENT condition register from the outer
    /// two**, and nothing in the source says so. A class using `CR_COMPARE`
    /// throughout emits the right program with one wrong `bc` operand and one
    /// wrong `cmplwi`, and every branch still resolves.
    #[test]
    fn the_three_tests_do_not_share_one_condition_register() {
        let b = alloc_init_or_fail_text(&undname(), 0, OptMode::O1).expect("in class");
        assert_eq!(&b.text[0x1c..0x20], &encode_cmplwi(CR_COMPARE, 4, 0));
        assert_eq!(&b.text[0x38..0x3c], &encode_cmplwi(CR_MIDDLE, 3, 0));
        assert_eq!(&b.text[0x64..0x68], &encode_cmplwi(CR_COMPARE, 3, 0));
        assert_ne!(&b.text[0x1c..0x20], &b.text[0x38..0x3c]);
    }

    /// **The two `lis`es are hoisted by DIFFERENT distances** — two words and
    /// three — so no fixed-distance derivation of the relocation sites can be
    /// right about both. This is the test behind `crate::data_refs_of`'s
    /// positional walk.
    #[test]
    fn the_two_high_halves_have_different_hoist_distances() {
        let b = alloc_init_or_fail_text(&undname(), 0, OptMode::O1).expect("in class");
        let lis = encode_addis(SCRATCH_REG, 0, 0);
        let his: Vec<usize> = b
            .text
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, w)| *w == lis)
            .map(|(i, _)| i * 4)
            .collect();
        assert_eq!(his, vec![0x24, 0x40], "two high halves");
        // …and the low halves, derived the way `data_refs_of` derives them.
        assert_eq!(&b.text[0x2c..0x30], &encode_addi(RET_REG, SCRATCH_REG, 0));
        assert_eq!(&b.text[0x4c..0x50], &encode_addi(SCRATCH_REG, SCRATCH_REG, 0));
        assert_ne!(0x2c - 0x24, 0x4c - 0x40, "the two hoists differ");
    }

    /// **The status store is a BYTE store.** A `stw` here writes three
    /// neighbouring fields to zero, links, and is a different program.
    #[test]
    fn the_status_store_is_a_byte_store() {
        let b = alloc_init_or_fail_text(&undname(), 0, OptMode::O1).expect("in class");
        assert_eq!(&b.text[0x70..0x74], &encode_stb(SCRATCH_REG, THIS_REG, 4));
        assert_ne!(&b.text[0x70..0x74], &encode_stw(SCRATCH_REG, THIS_REG, 4));
    }

    /// The link store is reached from BOTH the initialized and the uninitialized
    /// path: the middle branch targets it, not the epilogue.
    #[test]
    fn the_middle_branch_lands_on_the_link_store_not_the_epilogue() {
        let b = alloc_init_or_fail_text(&undname(), 0, OptMode::O1).expect("in class");
        assert_eq!(
            &b.text[0x3c..0x40],
            &encode_bc(BO_TRUE, cr_bi(CR_MIDDLE, CR_BIT_EQ), 0x60 - 0x3c).expect("in range")
        );
        assert_eq!(&b.text[0x60..0x64], &encode_stw(RET_REG, THIS_REG, 0));
    }

    /// Every immediate is a field, and moving one moves exactly the words that
    /// carry it. Ten fields, and the guard against a class that hardcoded one.
    #[test]
    fn the_ten_immediates_are_fields() {
        let base = alloc_init_or_fail_text(&undname(), 0, OptMode::O1).expect("in class");
        let mut a = undname();
        a.k_size = 24;
        a.k_flag = 1;
        a.k_neg = -2;
        a.k_status = 7;
        a.off_a = 16;
        a.off_b = 20;
        a.off_c = 24;
        a.off_d = 28;
        a.off_e = 32;
        a.off_f = 36;
        let moved = alloc_init_or_fail_text(&a, 0, OptMode::O1).expect("in class");
        assert_eq!(moved.text.len(), base.text.len(), "the same block plan");
        let differing = base
            .text
            .chunks_exact(4)
            .zip(moved.text.chunks_exact(4))
            .filter(|(x, y)| x != y)
            .count();
        // 0x28 0x30 0x44 0x48 0x50 0x54 0x58 0x5c 0x60 0x6c 0x70 — eleven words,
        // because `off_d` reaches both the `lwz` and the link `stw`.
        assert_eq!(differing, 11);
    }

    /// `/Ox` (which is also `/O2`) refuses in the emitter as well as in the reader
    /// — one
    /// fact, two locators, deliberately, so they cannot disagree silently.
    #[test]
    fn a_mode_other_than_o1_refuses() {
        for m in [OptMode::Ox] {
            assert!(alloc_init_or_fail_text(&undname(), 0, m).is_err(), "{m:?}");
        }
    }
}

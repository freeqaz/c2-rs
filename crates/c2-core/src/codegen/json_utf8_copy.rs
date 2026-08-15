//! **W-JSON — the UTF-16 → UTF-8 copy loop, in the port's FIRST framed body
//! with a back edge and its first FRAMELESS `__savegprlr_N` frame.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::json_utf8_copy`]; this file is the
//! **seventy-six** body words and nothing else. Everything variable in them is
//! named in [`c2_il::JsonUtf8CopyFn`]: **four** values, two member offsets and
//! two wide status constants. Nothing else moves.
//!
//! ## The frame: a helper-saving LEAF
//!
//! ```text
//!   0x000  7d8802a6  mflr r12          TWO prologue words, not three: the body
//!   0x004  4bfffffd  bl __savegprlr_28 makes NO call, needs no outgoing
//!                                      parameter area and has no addressed
//!                                      local, so c2 allocates **no frame at
//!                                      all** — there is no `stwu`, and the
//!                                      helper's stores land below r1.
//!   …
//!   0x12c  4bfffed4  b __restgprlr_28  ONE epilogue word: no `addi r1,r1,F`
//!                                      either, and no `blr`.
//! ```
//!
//! `$M(prologue)` is therefore **8**, and the `.pdata` unwind word is
//! `0x40004C02` — `PrologLen 2`, `FuncLen 76` — which `coff::pdata_record`
//! produces from those two lengths with nothing special-cased.
//!
//! ## The seventy-six words
//!
//! ```text
//!    off  word       instruction              why it is this word
//!   ----  --------   ----------------------   --------------------------------
//!   0x008 38e00000   li    r7,0               THE ZERO. One register holds the
//!                                             literal 0 for the whole body and
//!                                             every later `= 0` is an `mr` off
//!                                             it — including the two NUL stores
//!   0x00c 2b050000   cmplwi cr6,r5,0          `!pSize`
//!   0x010 419a0110   bt    cr6.EQ -> Lfail
//!   0x014 2b040000   cmplwi cr6,r4,0          `!pBuffer`
//!   0x018 409a0010   bf    cr6.EQ -> Lelse
//!   0x01c 81650000   lwz   r11,0(r5)          `*pSize != 0`
//!   0x020 2b0b0000   cmplwi cr6,r11,0
//!   0x024 409a00fc   bf    cr6.EQ -> Lfail
//!  Lelse:
//!   0x028 81630004   lwz   r11,OFF_SIZE(r3)   `mBufferSize`
//!   0x02c 7ce83b78   mr    r8,r7              outputSize = 0
//!   0x030 7cff3b78   mr    r31,r7             index      = 0
//!   0x034 2b0b0000   cmplwi cr6,r11,0
//!   0x038 409900c8   bf    cr6.GT -> Lafter   `mBufferSize > 0`
//!   0x03c 7ce63b78   mr    r6,r7              offset = 0
//!   0x040 3bc00001   li    r30,1              **HOISTED OUT OF THE LOOP** — the
//!                                             `1` the two `rlwimi`s rotate to
//!                                             0x80, live across the back edge
//!                                             and therefore callee-saved
//!  Lloop:
//!   0x044 81630000   lwz   r11,OFF_BUF(r3)    `mBuffer`, reloaded every trip
//!   0x048 3bff0001   addi  r31,r31,1          index++
//!   0x04c 7d6b322e   lhzx  r11,r11,r6         `*(unsigned short *)((char *)mBuffer + offset)`
//!   0x050 38c60002   addi  r6,r6,2            offset += 2
//!   0x054 7d6a5b78   mr    r10,r11            the SECOND copy of wc, live only
//!                                             into the >0x7F arm's compare
//!   0x058 2b0b007f   cmplwi cr6,r11,127
//!   0x05c 41990020   bt    cr6.GT -> Lwide
//!   0x060 81450000   lwz   r10,0(r5)          `*pSize`
//!   0x064 39080001   addi  r8,r8,1            outputSize++
//!   0x068 7f085040   cmplw cr6,r8,r10
//!   0x06c 40980088   bf    cr6.LT -> Lcont
//!   0x070 556b067e   clrlwi r11,r11,25        `wc & 0x7F`
//!   0x074 b1640000   sth   r11,0(r4)
//!   0x078 48000034   b     Lnul               the ONE-byte arm joins the
//!                                             two-byte arm's trailing NUL
//!  Lwide:
//!   0x07c 81250000   lwz   r9,0(r5)           `maxSize = *pSize`
//!   0x080 2b0a07ff   cmplwi cr6,r10,2047
//!   0x084 41990030   bt    cr6.GT -> Lthree
//!   0x088 39080002   addi  r8,r8,2
//!   0x08c 7f084840   cmplw cr6,r8,r9
//!   0x090 40980064   bf    cr6.LT -> Lcont
//!   0x094 394000c0   li    r10,192
//!   0x098 7d695b78   mr    r9,r11
//!   0x09c 516ad6fe   rlwimi r10,r11,26,27,31  `0xC0 | ((wc >> 6) & 0x1F)`
//!   0x0a0 53c93832   rlwimi r9,r30,7,0,25     `0x80 | (wc & 0x3F)` — the 0x80
//!                                             comes from the HOISTED r30
//!   0x0a4 b1440000   sth   r10,0(r4)
//!   0x0a8 b5240002   sthu  r9,2(r4)
//!  Lnul:
//!   0x0ac b4e40002   sthu  r7,2(r4)           the shared trailing NUL
//!   0x0b0 48000044   b     Lcont
//!  Lthree:
//!   0x0b4 39080003   addi  r8,r8,3
//!   0x0b8 7f084840   cmplw cr6,r8,r9
//!   0x0bc 40980038   bf    cr6.LT -> Lcont
//!   0x0c0 392000e0   li    r9,224
//!   0x0c4 3ba00080   li    r29,128
//!   0x0c8 39440002   addi  r10,r4,2           a POINTER CHAIN, not folded: the
//!   0x0cc 5169a73e   rlwimi r9,r11,20,28,31   source writes `pBuffer++` twice,
//!   0x0d0 7d7c5b78   mr    r28,r11            and c2 spells the second bump
//!   0x0d4 517dd6be   rlwimi r29,r11,26,26,31  `r11 = r10 + 2` off the first
//!   0x0d8 b1240000   sth   r9,0(r4)           rather than `r4 + 4`
//!   0x0dc 396a0002   addi  r11,r10,2
//!   0x0e0 53dc3832   rlwimi r28,r30,7,0,25
//!   0x0e4 b3a40004   sth   r29,4(r4)          **index 2, not 1** — the source
//!                                             stores through `*(pBuffer + 1)`
//!   0x0e8 b78b0002   sthu  r28,2(r11)
//!   0x0ec 7d645b78   mr    r4,r11
//!   0x0f0 b0eb0000   sth   r7,0(r11)          this arm's NUL is its OWN word:
//!                                             it is at a different displacement
//!                                             from `Lnul`'s and cannot share it
//!  Lcont:
//!   0x0f4 81630004   lwz   r11,OFF_SIZE(r3)
//!   0x0f8 7f1f5840   cmplw cr6,r31,r11
//!   0x0fc 4198ff48   bt    cr6.LT -> Lloop    **THE BACK EDGE** — the only one
//!  Lafter:
//!   0x100 81650000   lwz   r11,0(r5)
//!   0x104 7f085840   cmplw cr6,r8,r11
//!   0x108 4198000c   bt    cr6.LT -> Lstore
//!   0x10c 3ce0803f   lis   r7,HI(K_SIZE_ERR)  **r7 STOPS BEING ZERO HERE** and
//!   0x110 60e70005   ori   r7,r7,LO(K_SIZE_ERR)  becomes the returned status,
//!                                             which is why the two NUL stores
//!                                             above are inside the loop and
//!                                             this is after it
//!  Lstore:
//!   0x114 39680001   addi  r11,r8,1           `*pSize = outputSize + 1`
//!   0x118 91650000   stw   r11,0(r5)
//!   0x11c 4800000c   b     Lret
//!  Lfail:
//!   0x120 3ce08007   lis   r7,HI(K_ARG_ERR)
//!   0x124 60e70057   ori   r7,r7,LO(K_ARG_ERR)
//!  Lret:
//!   0x128 7ce33b78   mr    r3,r7
//!   0x12c 4bfffed4   b     __restgprlr_28  REL24
//! ```
//!
//! **Zero words are chosen by a scheduler or a register allocator.** The whole
//! assignment — r7 the zero-then-status, r8 the running output size, r6 the byte
//! offset, r31 the loop index, r30 the hoisted `1`, r29/r28 the three-byte arm's
//! two continuation bytes, r9/r10/r11 the scratch — is **transcribed**, and the
//! reader refuses any body whose shape would move one of them. Board **#1706**.
//!
//! Every branch is **self-relative** and therefore independent of where the
//! function lands in `.text`; only the two REL24 words encode their own offsets,
//! so they are the only ones that need `base_off`.

//! # ✔ 2026-08-15, lane `w-json` — **twenty blocks, and the three published
//! positions are the LAYOUT's** (board **#3155**)
//!
//! The last body in `w-item-d`'s **#3124** migration that a lane could move.
//! `w-layout` took fifteen of the crate's twenty-three [`reach::direct`] sites
//! into [`BodyLayout`] and could not take these two; `w-fencea` lifted fence A
//! and took five of the remaining eight; this takes **two of the last three**.
//! `pool_ctor_chain`'s one is not a budget item — its back edge is `bdnz`,
//! [`Terminator`] has no variant for it, and `CFG_SHAPE.md` §6.3 declines the
//! CTR-loop discovery that would justify one (**#3146**).
//!
//! **This class is a `#3142` shape in full force, and that is what made it a
//! lane rather than an arm.** It publishes **three** positions off the same
//! running `t.len()` — the `bl __savegprlr_28` `REL24` site, `prolog_len`
//! (`$M(prologue)` and the `.pdata` `PrologLen`), and the
//! `b __restgprlr_28` `REL24` site — so its branches could never have moved
//! alone. `ptr_walk_chain_loop` is the one class in this residue that publishes
//! nothing (`#3154`); this is the other end of that scale. All three are now
//! [`FinishedBody::at`] / [`FinishedBody::start_of`] / `tail_sites`, and
//! [`FrameLayout`] stays the **single producer** of both frameless Class C
//! words: it is asked again after `finish`, at the offsets the layout gave.
//!
//! **The budget in `#3155` is not this body's.** The row prices the lift at
//! *"76 words, ten branch sites, four block labels (`Lelse`/`Lloop`/`Lwide`/
//! `Lnul`)"*. The emitter has **fourteen** branch words — thirteen forward
//! patches and one inline back edge, and its own comment said *"the nine
//! forward branches"* — and **ten** block labels, of which the row names four.
//! The `2` in `#3144`'s residue table is a different and **correct** count:
//! [`reach::direct`] *textual call sites*. Three numbers in one row, two wrong,
//! and no instrument in this repo compares a board figure with the code it
//! prices — `#3151`'s disease, one level out from a doc comment.
//!
//! **The admission is `ChargedClass::JsonUtf8Copy`, and `#3155`'s
//! `label_slots` `Some(5)` is `Some(8)`.** This class is `is_framed()` — it
//! carries a `.pdata` record and a `$M`/`$M`/`$T` triple even with no `stwu` —
//! so `label_slots(false)` is `Some(label_lead() + 4)`, not `+ 1`. It comes in
//! on **arm 1** either way, and the reason the wrong figure cost nothing is
//! that `labels.rs`' grading test reads the number out of `c2-il` rather than
//! from the board (**#3148**).
//!
//! **Nothing here relaxes anything and no byte moved.** The criterion is a
//! required-zero delta.
//!
//! [`BodyLayout`]: super::block_ir::BodyLayout
//! [`Terminator`]: super::block_ir::Terminator
//! [`FinishedBody::at`]: super::block_ir::FinishedBody::at
//! [`FinishedBody::start_of`]: super::block_ir::FinishedBody::start_of
//! [`reach::direct`]: super::reach::direct

use crate::codegen::block_ir::{BlockId, BlockOrder, BodyLayout, Terminator};
use crate::codegen::encode::{
    cr_bi, encode_addi, encode_addis, encode_cmplw, encode_cmplwi,
    encode_lhzx, encode_lwz, encode_mr, encode_ori, encode_rlwimi, encode_rlwinm, encode_sth,
    encode_sthu, encode_stw, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_BIT_GT, CR_BIT_LT, CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::labels::ChargedClass;
use crate::codegen::select::out_of_class;
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::JsonUtf8CopyFn;

/// `this`, the buffer and the size, in r3/r4/r5. r4 is **written** by the
/// three-byte arm, which is why it is not parked anywhere.
const R_THIS: u8 = 3;
const R_BUF: u8 = 4;
const R_SIZE: u8 = 5;
/// The byte offset into `mBuffer`, live across the back edge but **volatile** —
/// the body makes no call, so nothing can clobber it.
const R_OFF: u8 = 6;
/// The literal zero for the whole body, and the returned status after `Lafter`.
const R_ZERO: u8 = 7;
/// The running output size.
const R_OUT: u8 = 8;
const R_T9: u8 = 9;
const R_T10: u8 = 10;
const R_T11: u8 = 11;
/// The three-byte arm's third continuation byte.
const R_C3: u8 = 28;
/// The three-byte arm's second continuation byte.
const R_C2: u8 = 29;
/// The hoisted literal `1`, rotated to `0x80` by two `rlwimi`s.
const R_ONE: u8 = 30;
/// The loop index.
const R_IX: u8 = 31;

/// Four callee-saved GPRs — r28…r31 — which is `__savegprlr_28`, and the whole
/// reason this class needs a frame emitter of its own.
const JSON_SAVED_GPRS: u8 = 4;

/// The frame this class always builds: **no locals, no outgoing slots**, four
/// saved GPRs. [`FrameLayout::out_of_class_ctx_gpr_helper_leaf`] is what turns
/// that into a two-word prologue and a one-word epilogue.
pub fn json_frame() -> FrameLayout {
    FrameLayout { locals: 0, out_slots: 0, saved_gprs: JSON_SAVED_GPRS, saved_fprs: 0 }
}

/// The emitted body plus everything the obj writer needs from it.
pub struct JsonUtf8CopyBody {
    pub text: Vec<u8>,
    /// The two REL24 sites, in ascending `.text` order: `__savegprlr_28` then
    /// `__restgprlr_28`. **This class has no other call site at all.**
    pub bl_offsets: [u32; 2],
    /// Prologue length in bytes — `$M(n)` and the `.pdata` `PrologLen`. **8**,
    /// i.e. two words: there is no `stwu`.
    pub prolog_len: u32,
}

/// Emit the seventy-six words, as **twenty basic blocks** in
/// [`BlockOrder::IlStatement`].
///
/// Every branch is a [`Terminator`] naming a [`BlockId`] and every displacement
/// is [`super::labels::LabelMap`]'s — including the one **back edge**, which
/// reaches the map through [`ChargedClass::JsonUtf8Copy`]'s admission. The three
/// positions this class publishes come back out of the finished layout; not one
/// of them is counted off a running byte vector any more.
pub fn json_utf8_copy_text(
    g: &JsonUtf8CopyFn,
    base_off: u32,
    mode: OptMode,
) -> Result<JsonUtf8CopyBody, BackendError> {
    // The mode is already gated in the parser (board #1638) — this is the second
    // lock, and it is the one that would fire if a future dispatch reached this
    // emitter from somewhere else.
    if mode != OptMode::O1 {
        return Err(out_of_class("json-utf8-copy is /O1 only"));
    }
    let off_size = i16::try_from(g.off_size)
        .map_err(|_| out_of_class("json member offset wider than an lwz displacement"))?;
    let off_buf = i16::try_from(g.off_buffer)
        .map_err(|_| out_of_class("json member offset wider than an lwz displacement"))?;

    let lo16 = |k: i32| (k as u32 & 0xFFFF) as u16;
    let hi16 = |k: i32| ((k as u32 >> 16) as u16) as i16;

    let frame = json_frame();

    // **The admission, and it is the map's.** This class emits one backward
    // intra-section branch — `Lcont`'s `bt cr6.LT -> Lloop` — and before lane
    // `w-fencea` no map would resolve one, so the whole body was fenced out of
    // a layout by its single back edge (the fence is per BODY: `finish`
    // resolves every branch through the one map). `ChargedClass::JsonUtf8Copy`
    // is graded in `labels.rs` against `c2_il`'s own counter gate — this class
    // is framed, its `label_lead` is 4, and `coff::plan_labels` already
    // advances `lead + 4` for its `$M`/`$M`/`$T` triple. No number is copied
    // here.
    let mut l =
        BodyLayout::admitting_back_edges(BlockOrder::IlStatement, ChargedClass::JsonUtf8Copy);

    // ---- the twenty identities, minted before any of them has a position ----
    //
    // Declaration order is NOT emission order and nothing here depends on it
    // being: `declare` mints an identity and says nothing about where the block
    // goes. They are listed in emission order only because that is how the
    // seventy-six words read.
    let entry = l.declare("json prologue and the !pSize guard");
    let guard_buf = l.declare("json !pBuffer guard");
    let guard_size = l.declare("json *pSize != 0 guard");
    let b_else = l.declare("Lelse");
    let preheader = l.declare("json loop preheader");
    let b_loop = l.declare("Lloop");
    let narrow = l.declare("the one-byte arm's size test");
    let narrow_store = l.declare("the one-byte arm's store");
    let b_wide = l.declare("Lwide");
    let two = l.declare("the two-byte arm's size test");
    let two_store = l.declare("the two-byte arm's store");
    let b_nul = l.declare("Lnul");
    let b_three = l.declare("Lthree");
    let three_store = l.declare("the three-byte arm's store");
    let b_cont = l.declare("Lcont");
    let b_after = l.declare("Lafter");
    let size_err = l.declare("the size-error status");
    let b_store = l.declare("Lstore");
    let b_fail = l.declare("Lfail");
    let b_ret = l.declare("Lret and the one-word epilogue");

    // The two branch senses this body uses, spelled once. `CR_COMPARE` is cr6
    // and every terminator below reads a field its own block's run wrote —
    // `BodyLayout::place` refuses it otherwise (item E, board #188).
    let bt = |bit: u8, taken: BlockId| Terminator::Bc {
        bo: BO_TRUE,
        bi: cr_bi(CR_COMPARE, bit),
        taken,
    };
    let bf = |bit: u8, taken: BlockId| Terminator::Bc {
        bo: BO_FALSE,
        bi: cr_bi(CR_COMPARE, bit),
        taken,
    };
    let run = |w: &[[u8; 4]]| -> Vec<u8> { w.concat() };

    // ---- the frameless Class C prologue: two words, one of them a relocation
    //
    // `BL_SAVE_IN_ENTRY` is a constant of THIS BLOCK'S OWN RUN, which is the
    // whole of #3124: stated as `at(entry, 4)` it grows with the block, where
    // the `base_off + 4` it replaces was a constant of the whole body.
    const BL_SAVE_IN_ENTRY: u32 = 4;
    let mut entry_run = frame.prologue_gpr_helper_leaf(base_off)?;
    let prolog_len = entry_run.len() as u32;

    // ---- the parameter guards ----------------------------------------------
    entry_run.extend_from_slice(&encode_addi(R_ZERO, 0, 0));
    entry_run.extend_from_slice(&encode_cmplwi(CR_COMPARE, R_SIZE, 0));
    l.place(entry, entry_run, bt(CR_BIT_EQ, b_fail))?;

    l.place(
        guard_buf,
        run(&[encode_cmplwi(CR_COMPARE, R_BUF, 0)]),
        bf(CR_BIT_EQ, b_else),
    )?;

    l.place(
        guard_size,
        run(&[encode_lwz(R_T11, R_SIZE, 0), encode_cmplwi(CR_COMPARE, R_T11, 0)]),
        bf(CR_BIT_EQ, b_fail),
    )?;

    // ---- Lelse, and the preheader that falls into the loop -----------------
    l.place(
        b_else,
        run(&[
            encode_lwz(R_T11, R_THIS, off_size),
            encode_mr(R_OUT, R_ZERO),
            encode_mr(R_IX, R_ZERO),
            encode_cmplwi(CR_COMPARE, R_T11, 0),
        ]),
        bf(CR_BIT_GT, b_after),
    )?;
    l.place(
        preheader,
        run(&[encode_mr(R_OFF, R_ZERO), encode_addi(R_ONE, 0, 1)]),
        Terminator::FallThrough,
    )?;

    // ---- Lloop -------------------------------------------------------------
    l.place(
        b_loop,
        run(&[
            encode_lwz(R_T11, R_THIS, off_buf),
            encode_addi(R_IX, R_IX, 1),
            encode_lhzx(R_T11, R_T11, R_OFF),
            encode_addi(R_OFF, R_OFF, 2),
            encode_mr(R_T10, R_T11),
            encode_cmplwi(CR_COMPARE, R_T11, K_ASCII_MAX),
        ]),
        bt(CR_BIT_GT, b_wide),
    )?;

    // ---- the one-byte arm --------------------------------------------------
    l.place(
        narrow,
        run(&[
            encode_lwz(R_T10, R_SIZE, 0),
            encode_addi(R_OUT, R_OUT, 1),
            encode_cmplw(CR_COMPARE, R_OUT, R_T10),
        ]),
        bf(CR_BIT_LT, b_cont),
    )?;
    l.place(
        narrow_store,
        run(&[
            encode_rlwinm(R_T11, R_T11, 0, ASCII_MASK_MB, 31),
            encode_sth(R_T11, R_BUF, 0),
        ]),
        Terminator::B { target: b_nul },
    )?;

    // ---- Lwide: the two-byte arm -------------------------------------------
    l.place(
        b_wide,
        run(&[
            encode_lwz(R_T9, R_SIZE, 0),
            encode_cmplwi(CR_COMPARE, R_T10, K_TWO_MAX),
        ]),
        bt(CR_BIT_GT, b_three),
    )?;
    l.place(
        two,
        run(&[encode_addi(R_OUT, R_OUT, 2), encode_cmplw(CR_COMPARE, R_OUT, R_T9)]),
        bf(CR_BIT_LT, b_cont),
    )?;
    l.place(
        two_store,
        run(&[
            encode_addi(R_T10, 0, K_LEAD2),
            encode_mr(R_T9, R_T11),
            encode_rlwimi(R_T10, R_T11, 26, 27, 31),
            encode_rlwimi(R_T9, R_ONE, 7, 0, 25),
            encode_sth(R_T10, R_BUF, 0),
            encode_sthu(R_T9, R_BUF, 2),
        ]),
        Terminator::FallThrough,
    )?;

    // ---- Lnul: the trailing NUL the one- and two-byte arms share -----------
    l.place(
        b_nul,
        run(&[encode_sthu(R_ZERO, R_BUF, 2)]),
        Terminator::B { target: b_cont },
    )?;

    // ---- Lthree: the three-byte arm ----------------------------------------
    l.place(
        b_three,
        run(&[encode_addi(R_OUT, R_OUT, 3), encode_cmplw(CR_COMPARE, R_OUT, R_T9)]),
        bf(CR_BIT_LT, b_cont),
    )?;
    l.place(
        three_store,
        run(&[
            encode_addi(R_T9, 0, K_LEAD3),
            encode_addi(R_C2, 0, K_CONT),
            encode_addi(R_T10, R_BUF, 2),
            encode_rlwimi(R_T9, R_T11, 20, 28, 31),
            encode_mr(R_C3, R_T11),
            encode_rlwimi(R_C2, R_T11, 26, 26, 31),
            encode_sth(R_T9, R_BUF, 0),
            encode_addi(R_T11, R_T10, 2),
            encode_rlwimi(R_C3, R_ONE, 7, 0, 25),
            encode_sth(R_C2, R_BUF, 4),
            encode_sthu(R_C3, R_T11, 2),
            encode_mr(R_BUF, R_T11),
            encode_sth(R_ZERO, R_T11, 0),
        ]),
        Terminator::FallThrough,
    )?;

    // ---- Lcont: the loop's own test, and **THE BACK EDGE** -----------------
    //
    // `taken` is `b_loop`, which is behind this block in the emission order.
    // The displacement is the map's and nothing here computes it — which is the
    // difference between this and the `reach::direct` call it replaces.
    l.place(
        b_cont,
        run(&[encode_lwz(R_T11, R_THIS, off_size), encode_cmplw(CR_COMPARE, R_IX, R_T11)]),
        bt(CR_BIT_LT, b_loop),
    )?;

    // ---- Lafter ------------------------------------------------------------
    l.place(
        b_after,
        run(&[encode_lwz(R_T11, R_SIZE, 0), encode_cmplw(CR_COMPARE, R_OUT, R_T11)]),
        bt(CR_BIT_LT, b_store),
    )?;
    l.place(
        size_err,
        run(&[
            encode_addis(R_ZERO, 0, hi16(g.k_size_err)),
            encode_ori(R_ZERO, R_ZERO, lo16(g.k_size_err)),
        ]),
        Terminator::FallThrough,
    )?;

    // ---- Lstore ------------------------------------------------------------
    l.place(
        b_store,
        run(&[encode_addi(R_T11, R_OUT, 1), encode_stw(R_T11, R_SIZE, 0)]),
        Terminator::B { target: b_ret },
    )?;

    // ---- Lfail -------------------------------------------------------------
    l.place(
        b_fail,
        run(&[
            encode_addis(R_ZERO, 0, hi16(g.k_arg_err)),
            encode_ori(R_ZERO, R_ZERO, lo16(g.k_arg_err)),
        ]),
        Terminator::FallThrough,
    )?;

    // ---- Lret, and the one-word epilogue ------------------------------------
    //
    // `b __restgprlr_28` is an **external** branch — a relocation and not a
    // label reference (board #191) — so it is `Terminator::TailCall`: a zero
    // placeholder whose offset the layout reports and whose word only
    // `FrameLayout` may write.
    l.place(b_ret, run(&[encode_mr(3, R_ZERO)]), Terminator::TailCall)?;

    let finished = l.finish()?;

    // ---- the three positions this class publishes, all three the layout's ---
    //
    // `prolog_len` is `$M(prologue)` and the `.pdata` `PrologLen`, and both are
    // offsets from the FUNCTION's own `.text` start rather than from a block's.
    // It is a run-local length above; this is the check that makes it a body
    // offset too, and it is a refusal rather than a `debug_assert` because the
    // release build the gate runs compiles those out.
    if finished.start_of(entry)? != 0 {
        return Err(out_of_class(
            "the frameless Class C prologue is not the first block in the \
             emission order: `$M(prologue)` and the .pdata PrologLen are \
             offsets from the function's own .text start, so a prologue placed \
             anywhere else makes both of them wrong",
        ));
    }
    if finished.tail_sites.len() != 1 {
        return Err(out_of_class(
            "the json body reported a number of tail-branch sites other than \
             one: this class has exactly one external branch, its epilogue",
        ));
    }
    let at_save = finished.at(entry, BL_SAVE_IN_ENTRY)?;
    let at_rest = finished.tail_sites[0];
    let bl_offsets = [base_off + at_save, base_off + at_rest];

    // `FrameLayout` stays the SINGLE producer of both frameless Class C words.
    // Each is asked again, at the offset the layout gave it, and the answer
    // overwrites the one placed before the positions were known — the second
    // write is the one that is true by construction.
    let mut t = finished.text;
    let pro_here = frame.prologue_gpr_helper_leaf(base_off)?;
    debug_assert_eq!(&t[..4], &pro_here[..4]);
    t[..8].copy_from_slice(&pro_here);
    let epi_here = frame.epilogue_gpr_helper_leaf(bl_offsets[1])?;
    t[at_rest as usize..at_rest as usize + 4].copy_from_slice(&epi_here);

    debug_assert_eq!(t.len(), 304);
    Ok(JsonUtf8CopyBody { text: t, bl_offsets, prolog_len })
}

/// `wc <= 0x7F` — the one-byte bound, in a `cmplwi` immediate.
///
/// **PINNED, not a field.** It is one of five constants that together are the
/// UTF-8 encoding: changing it without changing [`ASCII_MASK_MB`],
/// [`K_TWO_MAX`], the two `rlwimi` rotate/mask triples and the lead bytes gives
/// a body that is no longer this program, and the reader has no witness of any
/// other combination. Board **#1706** — anything the emitter cannot vary is
/// refused by the reader, and here the reader refuses every one of them.
const K_ASCII_MAX: u16 = 0x7F;

/// `clrlwi r11,r11,25` = `rlwinm r11,r11,0,25,31` — `wc & 0x7F`. PINNED with
/// [`K_ASCII_MAX`], whose mask it is.
const ASCII_MASK_MB: u8 = 25;

/// `wc <= 0x7FF` — the two-byte bound. PINNED.
const K_TWO_MAX: u16 = 0x7FF;

/// `0xC0`, the two-byte lead. PINNED.
const K_LEAD2: i16 = 0xC0;

/// `0xE0`, the three-byte lead. PINNED.
const K_LEAD3: i16 = 0xE0;

/// `0x80`, the continuation-byte mark, materialized as an `li` in the three-byte
/// arm and as the **hoisted `li r30,1` rotated by 7** everywhere else. PINNED —
/// and the two spellings are exactly why: which one c2 uses is not something
/// this emitter chooses.
const K_CONT: i16 = 0x80;

#[cfg(test)]
mod tests {
    use super::*;

    /// `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` exactly as `jsonwriter.cpp`'s IL
    /// decodes it.
    fn jsonwriter() -> JsonUtf8CopyFn {
        JsonUtf8CopyFn {
            params: vec![0x09f3, 0x09f0, 0x09f1],
            off_buffer: 0,
            off_size: 4,
            k_arg_err: 0x8007_0057u32 as i32,
            k_size_err: 0x803F_0005u32 as i32,
        }
    }

    /// The seventy-six reference words, `work/w-json/probe/ref.obj` disassembled
    /// at the workload's own flags before this file existed.
    const REF: [u32; 76] = [
        0x7d8802a6, 0x4bfffffd, 0x38e00000, 0x2b050000, 0x419a0110, 0x2b040000, 0x409a0010,
        0x81650000, 0x2b0b0000, 0x409a00fc, 0x81630004, 0x7ce83b78, 0x7cff3b78, 0x2b0b0000,
        0x409900c8, 0x7ce63b78, 0x3bc00001, 0x81630000, 0x3bff0001, 0x7d6b322e, 0x38c60002,
        0x7d6a5b78, 0x2b0b007f, 0x41990020, 0x81450000, 0x39080001, 0x7f085040, 0x40980088,
        0x556b067e, 0xb1640000, 0x48000034, 0x81250000, 0x2b0a07ff, 0x41990030, 0x39080002,
        0x7f084840, 0x40980064, 0x394000c0, 0x7d695b78, 0x516ad6fe, 0x53c93832, 0xb1440000,
        0xb5240002, 0xb4e40002, 0x48000044, 0x39080003, 0x7f084840, 0x40980038, 0x392000e0,
        0x3ba00080, 0x39440002, 0x5169a73e, 0x7d7c5b78, 0x517dd6be, 0xb1240000, 0x396a0002,
        0x53dc3832, 0xb3a40004, 0xb78b0002, 0x7d645b78, 0xb0eb0000, 0x81630004, 0x7f1f5840,
        0x4198ff48, 0x81650000, 0x7f085840, 0x4198000c, 0x3ce0803f, 0x60e70005, 0x39680001,
        0x91650000, 0x4800000c, 0x3ce08007, 0x60e70057, 0x7ce33b78, 0x4bfffed4,
    ];

    fn words(b: &[u8]) -> Vec<u32> {
        b.chunks_exact(4).map(|c| u32::from_be_bytes(c.try_into().unwrap())).collect()
    }

    /// **The whole body, against the reference obj** — every word, in order,
    /// including the two relocation placeholders and the one BACK EDGE.
    #[test]
    fn body_matches_the_reference_words() {
        let b = json_utf8_copy_text(&jsonwriter(), 0, OptMode::O1).unwrap();
        assert_eq!(words(&b.text), REF.to_vec(), "the 76 words");
        assert_eq!(b.text.len(), 304);
    }

    /// The two offsets the obj writer reads out of the body, each pinned to a
    /// field of the reference obj rather than to this emitter's arithmetic:
    /// `.pdata` says `PrologLen 2` and `$M2594` sits at `0x8`, and the two REL24
    /// records are at `0x4` and `0x12c`.
    #[test]
    fn relocation_sites_and_prologue_length_match_the_reference() {
        let b = json_utf8_copy_text(&jsonwriter(), 0, OptMode::O1).unwrap();
        assert_eq!(b.prolog_len, 8, "$M(prologue) — TWO words, there is no stwu");
        assert_eq!(b.bl_offsets, [4, 0x12c]);
        let fr = json_frame();
        assert_eq!(fr.save_gpr_helper_name(), Some("__savegprlr_28"));
        assert_eq!(fr.rest_gpr_helper_name(), Some("__restgprlr_28"));
    }

    /// The four free fields each move **exactly one instruction field** and
    /// nothing else — #1767's rule, asserted rather than asserted-about. Every
    /// other word is identical to the reference on every row.
    #[test]
    fn each_field_moves_exactly_the_words_it_names() {
        let base = words(&json_utf8_copy_text(&jsonwriter(), 0, OptMode::O1).unwrap().text);
        let moved = |g: JsonUtf8CopyFn| -> Vec<usize> {
            let w = words(&json_utf8_copy_text(&g, 0, OptMode::O1).unwrap().text);
            (0..76).filter(|&i| w[i] != base[i]).collect()
        };
        // `off_size` is read twice — the guard at 0x28 and the loop test at 0xf4.
        assert_eq!(moved(JsonUtf8CopyFn { off_size: 8, ..jsonwriter() }), vec![10, 61]);
        // `off_buffer` once, at the top of the loop.
        assert_eq!(moved(JsonUtf8CopyFn { off_buffer: 12, ..jsonwriter() }), vec![17]);
        // Each status constant moves its own `lis`+`ori` pair and nothing else.
        assert_eq!(moved(JsonUtf8CopyFn { k_arg_err: 0x8009_1234u32 as i32, ..jsonwriter() }),
                   vec![72, 73]);
        assert_eq!(moved(JsonUtf8CopyFn { k_size_err: 0x8004_5678u32 as i32, ..jsonwriter() }),
                   vec![67, 68]);
    }


    // ---- lane `w-json`: what the LAYOUT owns ------------------------------

    /// **Exactly TWO words move with the function, and they are the two
    /// frameless Class C branches** — the test that grades `w-json`'s actual
    /// claim, which is that both are re-asked of [`FrameLayout`] at the offsets
    /// the layout gave them rather than counted off a running `t.len()`.
    ///
    /// It did not exist before: every previous test here ran at `base_off = 0`,
    /// where the layout's answer and a body-length counter agree by
    /// construction and the whole question is invisible.
    #[test]
    fn exactly_two_words_move_with_the_function() {
        let b = json_utf8_copy_text(&jsonwriter(), 0x200, OptMode::O1).unwrap();
        assert_eq!(b.bl_offsets, [0x204, 0x32c]);
        assert_eq!(b.prolog_len, 8, "a body offset, and it does NOT move with base_off");
        let w = words(&b.text);
        let moved: Vec<usize> = (0..76).filter(|&i| w[i] != REF[i]).collect();
        assert_eq!(moved, vec![1, 75], "bl __savegprlr_28 and b __restgprlr_28");
        // …and each carries `-(its own .text offset)` in MSVC's placeholder
        // convention, which is what makes the site the layout's business.
        // opcode 18, AA = 0; `LK = 1` on the `bl` and 0 on the tail `b`.
        assert_eq!(w[1], 0x4800_0000 | ((-0x204i32 as u32) & 0x03FF_FFFC) | 1);
        assert_eq!(w[75], 0x4800_0000 | ((-0x32ci32 as u32) & 0x03FF_FFFC));
    }

    /// **The two published `REL24` sites and the published `prolog_len` are one
    /// consistent story**, not three counters that happen to agree. The `bl
    /// __savegprlr_28` is the *second* word of the prologue, so the prologue
    /// ends one word after it — stated over the published numbers rather than
    /// over the emitter's arithmetic, which is the point of #3124.
    #[test]
    fn the_published_prologue_length_and_the_published_save_site_agree() {
        for base in [0u32, 4, 0x100, 0x2f0] {
            let b = json_utf8_copy_text(&jsonwriter(), base, OptMode::O1).unwrap();
            assert_eq!(b.bl_offsets[0] - base + 4, b.prolog_len, "base {base:#x}");
            assert_eq!(b.bl_offsets[1] - base + 4, b.text.len() as u32, "base {base:#x}");
        }
    }

    /// **Every branch in the body lands on a block boundary**, decoded out of
    /// the emitted words rather than read off [`REF`].
    ///
    /// This is the assertion the byte compare cannot make. `REF` pins the
    /// seventy-six words, so a displacement that is wrong *and* transcribed into
    /// `REF` is invisible to it; this walks the fourteen branch words, adds each
    /// displacement to its own site, and checks the answer is the start of one
    /// of the twenty blocks. **Fourteen, not the ten `#3155` prices, and not
    /// the "nine forward branches" this file's own comment claimed** — thirteen
    /// forward and one back edge.
    #[test]
    fn every_branch_lands_on_a_block_boundary_and_there_are_fourteen_of_them() {
        let b = json_utf8_copy_text(&jsonwriter(), 0, OptMode::O1).unwrap();
        let w = words(&b.text);
        // Where each of the twenty blocks starts, in words, from the listing in
        // this module's header. Written out so the test states the layout rather
        // than re-deriving it from the thing under test.
        const STARTS: [usize; 20] = [
            0x000 / 4, 0x014 / 4, 0x01c / 4, 0x028 / 4, 0x03c / 4, 0x044 / 4, 0x060 / 4,
            0x070 / 4, 0x07c / 4, 0x088 / 4, 0x094 / 4, 0x0ac / 4, 0x0b4 / 4, 0x0c0 / 4,
            0x0f4 / 4, 0x100 / 4, 0x10c / 4, 0x114 / 4, 0x120 / 4, 0x128 / 4,
        ];
        let mut sites: Vec<usize> = Vec::new();
        for (i, &word) in w.iter().enumerate() {
            let op = word >> 26;
            // 16 = `bc`, 18 = `b`. The two REL24 words are `b`/`bl` at word 1
            // and word 75 and are NOT label references (board #191) — they are
            // relocations, and their placeholder is a negative self offset.
            if i == 1 || i == 75 {
                continue;
            }
            let disp: i32 = match op {
                16 => ((word & 0xFFFC) as u16 as i16) as i32,
                18 => (((word & 0x03FF_FFFC) << 6) as i32) >> 6,
                _ => continue,
            };
            let target = i as i32 + disp / 4;
            assert!(
                STARTS.contains(&(target as usize)),
                "the branch at word {i} ({word:#010x}) targets word {target}, \
                 which is not the start of any block"
            );
            sites.push(i);
        }
        assert_eq!(sites.len(), 14, "branch words: {sites:?}");
        // One of them is the BACK EDGE, and exactly one.
        let back: Vec<usize> = sites
            .iter()
            .copied()
            .filter(|&i| {
                let word = w[i];
                let disp = if word >> 26 == 16 {
                    ((word & 0xFFFC) as u16 as i16) as i32
                } else {
                    (((word & 0x03FF_FFFC) << 6) as i32) >> 6
                };
                disp < 0
            })
            .collect();
        assert_eq!(back, vec![0x0fc / 4], "Lcont's bt cr6.LT -> Lloop");
    }

    /// **The admission is load-bearing.** The same back-edge shape through
    /// [`BodyLayout::new`] refuses, in `labels.rs`' own words, naming the block.
    ///
    /// `w-fencea` lifted fence A on a **closed** admission and `BodyLayout::new`
    /// is unchanged, so a lane that reached for a default layout here would get
    /// the refusal every one of item A's other clients still gets. This is the
    /// line that says so rather than leaving it to be believed.
    #[test]
    fn the_back_edge_through_a_default_layout_is_refused_in_the_maps_own_words() {
        let mut l = BodyLayout::new(BlockOrder::IlStatement);
        let head = l.declare("Lloop");
        let tail = l.declare("Lcont");
        l.place(head, encode_mr(R_T10, R_T11).to_vec(), Terminator::FallThrough).unwrap();
        l.place(
            tail,
            encode_cmplw(CR_COMPARE, R_IX, R_T11).to_vec(),
            Terminator::Bc { bo: BO_TRUE, bi: cr_bi(CR_COMPARE, CR_BIT_LT), taken: head },
        )
        .unwrap();
        let s = format!("{:?}", l.finish().unwrap_err());
        assert!(s.contains("BACKWARD"), "{s}");
        assert!(s.contains("plan_labels"), "{s}");
        assert!(s.contains("Lloop"), "{s}");
    }

    /// The mode gate's second lock (#1638). The parser asks first; this is the
    /// one that fires if a future dispatch reaches the emitter another way.
    #[test]
    fn the_emitter_refuses_every_mode_but_o1() {
        for m in [OptMode::Ox] {
            assert!(json_utf8_copy_text(&jsonwriter(), 0, m).is_err(), "{m:?}");
        }
    }
}

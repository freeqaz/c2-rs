//! **The body-parameterized pointer-walk loop** — the port's first lowering
//! whose emitted body has no fixed length.
//!
//! [`super::ptr_walk_loop`] opens with the sentence *"this is a transcription,
//! and saying so is the point"*. This module is the other thing. Nothing here
//! is transcribed: the body's **length**, the induction load's **slot**, the
//! record form's **slot**, every **register field**, both branch
//! **displacements** and the entry form itself are computed from
//! [`PtrWalkChainLoop::ops`] by rules that were measured against real `c2` and
//! held out before this file existed.
//!
//! ```text
//!   M = 1, pv = 0 -> TWO            M = 4, pv = 0 -> SAME
//!   lbz    r11,0(r3)                lbz    r11,0(r3)
//!   mr     r10,r3                   mr     r10,r3
//!   li     r3,K0                    li     r3,K0
//!   extsb. r11,r11                  b      .+24            <- JUMPIN, into the
//!   bclr   12,2                     add    r9,r11,r3          record form
//!   lbzu   r9,1(r10)                lbzu   r11,1(r10)      <- writes CHAR
//!   add    r3,r11,r3                xori   r9,r9,3
//!   extsb. r11,r9                   addi   r9,r9,5
//!   bf     2,-12                    ori    r3,r9,9
//!   blr                             extsb. r11,r11
//!                                   bf     2,-24
//!                                   blr
//! ```
//!
//! One source operation added and the entry form, the allocation, the record
//! form's slot and the function's length all move together.
//!
//! # The rules, and where each was graded
//!
//! All five body rules are `docs/rungs/2026-08-05-w-sched2.md`'s, held out
//! there against real `c2`; the guard form is
//! `docs/rungs/2026-08-05-w-rotate.md` §3's **P2**, 46 of 46.
//!
//! | | rule | held out |
//! |---|---|---:|
//! | **S3m** | SAME regime iff `pv == 0` **and** `M >= 4` | 84 of 84 |
//! | **S1** | the load's slot `a` is 1, except 0 when `M <= 2` and `pv == 0` | 84 of 84 for `a <= 1` |
//! | **S2** | record slot: `max(p+1, a+2)` in TWO, `M+1` in SAME | 82 of 84 |
//! | **S4r/S4n** | roles by position about the record form; `T1 = r8`, `T2 = r9`, home `= r3` | 81 of 84 |
//! | **S5** | a commutative op's `RA` takes the operand of higher **recency** | 27 of 27 |
//! | **P2** | the guard folds to `bclr` iff the fall-out block is a bare `blr` | 46 of 46 |
//!
//! Restricted to the signature class this module admits, w-sched2 measured
//! **67 of 67** on S1·S2·S3m·S4r·S4n and its reconstruction rebuilt `c2`'s own
//! body bytes **104 of 104**.
//!
//! # The preamble and the tail were NOT measured by any of that
//!
//! w-sched2's reconstruction rebuilds the **body and the back edge only**. It
//! hard-codes the walked pointer as r10 and the accumulator home as r3, and it
//! never generates or compares a single preamble or tail word. Those words are
//! this lane's own measurement (`work/w-varloop/probe.py`, PREREG §2's V1 and
//! V2), and they are the reason the function's total length is `M + 9` in the
//! TWO regime and `M + 8` in SAME:
//!
//! ```text
//!   TWO    lbz · mr · li · <entry test> · bclr   |  body M+2  |  bf · blr
//!   SAME   lbz · mr · li · b .+4*(M+2)           |  body M+2  |  bf · blr
//! ```
//!
//! SAME is one word shorter because the peeled character and the induction load
//! share r11, so the record form *is* the entry test and is reached by jumping
//! into it rather than by emitting a second copy.
//!
//! # Why the branches are computed and not written down
//!
//! Both displacements are functions of `M`. `bclr` has none at all, which is
//! P2's practical content: the entry guard of a variable-length body needs no
//! forward fixup. The back edge is `-4*(M+2)` and the JUMPIN branch is
//! `+4*(M+2)`.
//!
//! ## ✔ 2026-08-15, lane `w-fencea` — **they are functions of the LAYOUT now,
//! and `M` is not written down at either site**
//!
//! Neither displacement is computed here. The back edge names the body block and
//! the JUMPIN `b` names the block the body's last word is in, and
//! [`super::labels::LabelMap`] derives both from where those blocks landed. The
//! SAME regime is why the body is **two** blocks: the preamble jumps *into* the
//! record form, which S2 puts at `M + 1` — the body's last slot — so the split
//! is at `4 * (M + 1)` and the jump gets a block identity instead of a count.
//!
//! **The admission is `labels.rs`' SECOND arm, not its first**, and the
//! difference matters: this class's `c2_il::IlFunction::label_slots` is `None`,
//! so `IlBundle::functions` refuses every TU in which its `$M` could be observed
//! (board #742). Its charge is **undetermined and stays undetermined** — nothing
//! here claims a lead for it, board **#746**'s fence B is untouched and is
//! peer-held, and the TU-level gate is precisely what makes routing this back
//! edge through the map safe.

use c2_il::{ChainOpKind, ChainRhs, PtrWalkChainLoop};

use crate::codegen::block_ir::{BlockOrder, BodyLayout, Terminator};
use crate::codegen::cond::{producer_at, Cond, CR0};
use crate::codegen::encode::{
    encode_add, encode_addi, encode_cmplwi,
    encode_extsb_record, encode_lbz, encode_lbzu, encode_mr, encode_mr_record, encode_mulli,
    encode_mullw, encode_or, encode_ori, encode_xor, encode_xori, BO_FALSE, BO_TRUE, CR_BIT_EQ,
};
use crate::codegen::labels::ChargedClass;
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;

// The private `const CR_RECORD: u8 = 0;` that used to sit here — the second of
// two identical copies, the other in [`super::ptr_walk_loop`] — is gone. Lane
// `w-ir-e`, `CFG_SHAPE.md` §6.2 item **E**: the condition-register field now
// comes from the **producer**, which this class varies by cell and its own
// `entry_test`/`record_form` doc already spells out — `extsb.` and `mr.` are
// record forms, and the SAME cell emits `cmplwi cr0` instead, an explicit
// compare. One constant could name the field; it could not name which of §3.2's
// two producers wrote it. [`CR0`] names the register itself.

/// The carried character, live across the back edge.
const R_CHAR: u8 = 11;
/// The walked pointer.
const R_PTR: u8 = 10;
/// The accumulator's home — **S4n**. It is also r3, the return register, which
/// is why the fall-out block is a bare `blr` and why P2 folds the guard.
const R_HOME: u8 = 3;
/// **S4n's `T1`** — the chain temp for producers scheduled before the record.
const R_T1: u8 = 8;
/// **S4n's `T2`** — the chain temp for producers scheduled after the record,
/// and the induction load's destination in the TWO regime.
const R_T2: u8 = 9;
/// The pointer formal, slot 0.
const R_SRC: u8 = 3;

/// The schedule: where chain step `i` lands in the body, given the load's slot
/// `a` and the record form's slot `r`.
///
/// The body is the chain in order with two words inserted into it, so a step's
/// slot is its index pushed right once by each insertion that precedes it. This
/// is the whole of the interleave, and it is the function w-sched2's `slot_of`
/// grades.
fn slot_of(i: usize, a: usize, r: usize) -> usize {
    let s = i + usize::from(i >= a);
    s + usize::from(s >= r)
}

/// Emit the whole function.
///
/// No relocation, no pooled constant, no label and no symbol — every branch is
/// self-relative — so the caller takes it as an ordinary `Selected::Plain`.
pub(crate) fn ptr_walk_chain_loop_text(
    l: &PtrWalkChainLoop,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    // **`/O1` only.** Every cell behind every rule in this file was captured at
    // `/O1 /GS- /c`. `docs/OPT_MODE.md`'s register-field reading is already
    // recorded as refuted once a body has more than one block, and
    // `super::ptr_walk_loop` is the strongest witness: the same source is a
    // different *body* at `/Ox`, not a different allocation. Emitting this body
    // there would be a guess with no oracle behind it.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "ptr-walk chain loop outside /O1: every cell behind the schedule, the \
             allocation and the entry form was captured at /O1. See \
             `codegen::ptr_walk_chain_loop`.",
        ));
    }
    // Re-asserted although the recognizer already required it: `select_function`
    // is what `function_gate` runs, so a shape reaching codegen with a different
    // arity would be a census/gate disagreement, and that counter reading 0 is
    // the only thing keeping the census honest about what the port accepts.
    if l.params.len() != 1 {
        return Err(out_of_class(
            "ptr-walk chain loop with other than one formal: the block plan is measured at one",
        ));
    }
    let m = l.producers();
    if m == 0 {
        return Err(out_of_class("ptr-walk chain loop with an empty chain"));
    }
    let pv = l
        .pv()
        .ok_or_else(|| out_of_class("ptr-walk chain loop whose chain never reads the character"))?;
    let k0 = i16::try_from(l.acc_init)
        .map_err(|_| out_of_class("ptr-walk chain loop accumulator init outside simm16"))?;

    // ---- S3m: the regime, from two IL facts --------------------------------
    let same = l.regime_same();
    // ---- S1: the induction load's slot -------------------------------------
    //
    // `a <= 1` at every body length w-sched2's axis reaches — the schedule does
    // not grow a prologue as the body grows, which is why an emitter never has
    // to search for this position.
    let a = usize::from(!(m <= 2 && pv == 0 && !same));

    // The induction load's destination. In SAME it *is* the character's
    // register, which is the entire content of the regime.
    let ld = if same { R_CHAR } else { R_T2 };

    // **The CHAR-reuse clause of S4r.** When the load sits at slot 1 and the
    // character dies at the chain's first step, that step is allocated to
    // CHAR itself — so the character's *physical register* stays read one
    // producer longer than its *value* lives, and S2 needs the later index.
    let p0_char = !same && a == 1 && pv == 0;
    let p_chain = if p0_char && m > 1 { 1 } else { pv };

    // ---- S2: the record form's slot ----------------------------------------
    let rec = if same {
        // SAME: the last word of the body. The load wrote CHAR, so the record
        // form has nothing to carry forward and only has to set the CR bit.
        m + 1
    } else {
        // TWO: the earliest slot at which the character's register is dead and
        // the load's result has landed. Both terms of w-sched2's `max` are
        // load-bearing; searching for the earliest legal slot states them at
        // once and cannot disagree with the schedule it is indexed against.
        (a + 2..=m + 1)
            .find(|&cand| slot_of(p_chain, a, cand) < cand)
            .ok_or_else(|| out_of_class("ptr-walk chain loop with no legal record slot"))?
    };

    // ---- S4r / S4n: the allocation -----------------------------------------
    //
    // Roles first, names second — the split is w-sched2 §3.4's finding. The
    // last producer writes the accumulator's home; in SAME every other producer
    // shares one register; in TWO the role is decided by which side of the
    // record form the producer sits on, and never by a register number.
    let mut regs: Vec<u8> = Vec::with_capacity(m);
    for i in 0..m {
        regs.push(if i == m - 1 {
            R_HOME
        } else if same {
            R_T2
        } else if i == 0 && p0_char {
            R_CHAR
        } else if slot_of(i, a, rec) > rec {
            R_T2
        } else {
            R_T1
        });
    }

    // ---- lay the body out --------------------------------------------------
    let body_len = m + 2;
    let mut body: Vec<Option<[u8; 4]>> = vec![None; body_len];
    body[a] = Some(encode_lbzu(ld, R_PTR, 1));
    body[rec] = Some(record_form(l.elem_unsigned, same, ld));
    for i in 0..m {
        let prev = if i == 0 { R_HOME } else { regs[i - 1] };
        body[slot_of(i, a, rec)] = Some(chain_word(&l.ops[i], regs[i], prev, i)?);
    }

    // **Blocks, and the two displacements are the LAYOUT's** — lane `w-fencea`,
    // board **#3144**. Both were computed here until then, because
    // `BodyLayout::finish` resolves *every* branch through the one map and this
    // body's back edge closed it to all of them — so even the SAME regime's
    // forward `b` was fenced out by a branch at the other end of the function.
    //
    // The admission is [`ChargedClass::PtrWalkChainLoop`], and it is the
    // **second** arm of `labels.rs`' rule rather than the first: this class's
    // `IlFunction::label_slots` is `None`, so `IlBundle::functions` refuses
    // every TU in which its `$M` could be observed at all (board #742). The
    // charge is *undetermined* and stays undetermined; **nothing here claims a
    // lead for it**, and the TU-level gate is exactly what makes routing its
    // back edge safe. That fence is peer-held (#746 fence B) and is untouched.
    let mut lay = BodyLayout::admitting_back_edges(
        BlockOrder::IlStatement,
        ChargedClass::PtrWalkChainLoop,
    );
    let pre = lay.declare("chain-loop preamble");
    // In SAME the preamble jumps **into** the body's last word, so the body is
    // two blocks and the jump names the second. In TWO there is one body block
    // and `chain_tail` is it.
    let chain_head = lay.declare("chain-loop body");
    let chain_tail = if same { lay.declare("chain-loop entry test") } else { chain_head };
    let exit = lay.declare("chain-loop fall-out");

    // ---- the preamble ------------------------------------------------------
    let mut pre_run: Vec<u8> = Vec::with_capacity(16);
    pre_run.extend_from_slice(&encode_lbz(R_CHAR, R_SRC, 0));
    pre_run.extend_from_slice(&encode_mr(R_PTR, R_SRC));
    pre_run.extend_from_slice(&encode_addi(R_HOME, 0, k0));
    if same {
        // **JUMPIN.** The record form is the entry test, so the preamble jumps
        // into it — and *into it* is now a block identity rather than
        // `4 * body_len` counted off the body's length.
        lay.place(pre, pre_run, Terminator::B { target: chain_tail })?;
    } else {
        // The peeled character's own test, then **P2's `bclr`**: the block the
        // loop falls out to is a bare `blr`, so the guard folds and carries no
        // displacement at all.
        pre_run.extend_from_slice(&entry_test(l.elem_unsigned));
        // The producer is the word immediately above, and WHICH producer it is
        // depends on the cell: `cmplwi cr0` for an unsigned element, `extsb.`
        // for a signed one. Read off the bytes rather than assumed (§6.2 item
        // E); both write cr0, which is why one constant survived this long.
        let guard = Cond::new(
            producer_at(&pre_run, "ptr-walk chain loop entry guard")?,
            BO_TRUE,
            CR_BIT_EQ,
        );
        lay.place(pre, pre_run, Terminator::bclr(guard))?;
    }
    // ---- the body ----------------------------------------------------------
    let mut body_run: Vec<u8> = Vec::with_capacity(4 * body_len);
    for w in body {
        // Every slot is filled: `a`, `rec` and the `m` chain slots are `m + 2`
        // distinct indices by construction. A `None` here would be a schedule
        // that dropped or doubled a word, so it refuses rather than emitting a
        // zero — which would be a legal-looking instruction.
        body_run.extend_from_slice(&w.ok_or_else(|| {
            out_of_class("ptr-walk chain loop schedule left a body slot unfilled")
        })?);
    }
    // ---- the back edge -----------------------------------------------------
    //
    // The record form is somewhere in the body — its slot is S2's answer and
    // moves with the schedule — so the scan finds it rather than the emitter
    // knowing where it put it. It is read off the **whole** body run and not off
    // whichever of the two blocks the run was split into, because the schedule's
    // answer is a property of the body and the SAME split is a property of the
    // entry jump.
    let back = Cond::new(
        producer_at(&body_run, "ptr-walk chain loop back edge")?,
        BO_FALSE,
        CR_BIT_EQ,
    );
    if same {
        // The last word is the record form (S2: `rec == m + 1`), and it is the
        // word the preamble jumps to. Splitting there is what lets one block
        // identity name it.
        let cut = 4 * (body_len - 1);
        lay.place(chain_head, body_run[..cut].to_vec(), Terminator::FallThrough)?;
        lay.place(chain_tail, body_run[cut..].to_vec(), Terminator::bc(back, chain_head))?;
    } else {
        lay.place(chain_head, body_run, Terminator::bc(back, chain_head))?;
    }
    // ---- the fall-out block, which P2 is a claim about ---------------------
    lay.place(exit, Vec::new(), Terminator::Blr)?;

    let t = lay.finish()?.text;
    debug_assert_eq!(
        t.len(),
        4 * (m + if same { 8 } else { 9 }),
        "the function's length is M + 8 (SAME) or M + 9 (TWO)"
    );
    Ok(t)
}

/// The **entry test** over the peeled character, in the TWO regime.
///
/// Measured both ways (`work/w-varloop/probe.py --sig 'const unsigned char* s'`):
/// a signed element is tested by the same `extsb.` that widens it, an unsigned
/// one by an explicit `cmplwi` — because `lbz` already zero-extends and there
/// is nothing to widen.
fn entry_test(elem_unsigned: bool) -> [u8; 4] {
    if elem_unsigned {
        encode_cmplwi(CR0, R_CHAR, 0)
    } else {
        encode_extsb_record(R_CHAR, R_CHAR)
    }
}

/// The **record form** — the body word that writes the CR bit the back edge
/// reads, and carries the next iteration's character forward.
///
/// Three forms, not two, and the third is a fact w-sched2's reconstruction
/// never had to derive because it copied `c2`'s opcode:
///
/// ```text
///   signed              extsb. CHAR,LD        widen, carry and test in one
///   unsigned, TWO       mr.    CHAR,LD        carry and test; no widening needed
///   unsigned, SAME      cmplwi cr0,CHAR,0     LD *is* CHAR, so `mr. CHAR,CHAR`
///                                             would be a move to itself — `c2`
///                                             emits the compare instead
/// ```
fn record_form(elem_unsigned: bool, same: bool, ld: u8) -> [u8; 4] {
    match (elem_unsigned, same) {
        (false, _) => encode_extsb_record(R_CHAR, ld),
        (true, false) => encode_mr_record(R_CHAR, ld),
        (true, true) => encode_cmplwi(CR0, R_CHAR, 0),
    }
}

/// One chain step, selected and allocated.
///
/// `dest` is S4r's answer, `prev` the register holding the value this step
/// reads (the accumulator's home at the chain's head, the previous step's
/// destination after that), and `i` the step's index — which **S5** needs,
/// because recency is what orders a commutative operator's operands:
///
/// ```text
///   a chain temp from this iteration  >  CHAR  >  the loop-carried accumulator
/// ```
///
/// So `r = r + c` is `add rD,CHAR,HOME` at the chain's head and
/// `add rD,T1,CHAR` at its tail — the same source operation, the operands the
/// other way round. S5 was the one fact the four position rules did not
/// contain, and it was found only because w-sched2 *generated* bytes instead of
/// predicting positions.
fn chain_word(
    op: &c2_il::ChainOp,
    dest: u8,
    prev: u8,
    i: usize,
) -> Result<[u8; 4], BackendError> {
    Ok(match op.rhs {
        ChainRhs::Lit(k) => {
            // A literal is not an operand S5 orders: the immediate field has
            // only one place to go.
            match op.kind {
                ChainOpKind::Add => encode_addi(dest, prev, lit16(k)?),
                ChainOpKind::Mul => encode_mulli(dest, prev, lit16(k)?),
                ChainOpKind::Or => encode_ori(dest, prev, ulit16(k)?),
                ChainOpKind::Xor => encode_xori(dest, prev, ulit16(k)?),
            }
        }
        ChainRhs::Char => {
            // **S5.** At the head the accumulator is the loop-carried value and
            // the character is the more recent operand; from the second step on
            // the chain temp is more recent than the character.
            let (first, second) = if i == 0 { (R_CHAR, prev) } else { (prev, R_CHAR) };
            match op.kind {
                ChainOpKind::Add => encode_add(dest, first, second),
                ChainOpKind::Mul => encode_mullw(dest, first, second),
                ChainOpKind::Or => encode_or(dest, first, second),
                ChainOpKind::Xor => encode_xor(dest, first, second),
            }
        }
    })
}

/// A `simm16` immediate, refusing rather than truncating. The recognizer has
/// already checked the range; this is the second gate, at the encoder, for the
/// reason `encode_bc` returns `Option` — a truncated field is a legal-looking
/// instruction that computes the wrong thing.
fn lit16(k: i32) -> Result<i16, BackendError> {
    i16::try_from(k).map_err(|_| out_of_class("ptr-walk chain loop literal outside simm16"))
}

/// A `uimm16` immediate, for the two logical operations.
fn ulit16(k: i32) -> Result<u16, BackendError> {
    u16::try_from(k).map_err(|_| out_of_class("ptr-walk chain loop literal outside uimm16"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2_il::ChainOp;

    fn loop_of(ops: &[(ChainOpKind, ChainRhs)]) -> PtrWalkChainLoop {
        PtrWalkChainLoop {
            params: vec![0x09E3],
            acc_init: 0,
            elem_unsigned: false,
            ops: ops.iter().map(|&(kind, rhs)| ChainOp { kind, rhs }).collect(),
        }
    }

    fn words(t: &[u8]) -> Vec<u32> {
        t.chunks_exact(4)
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    fn emit(ops: &[(ChainOpKind, ChainRhs)]) -> Vec<u32> {
        words(&ptr_walk_chain_loop_text(&loop_of(ops), OptMode::O1).unwrap())
    }

    use ChainOpKind::{Add, Mul, Or, Xor};
    use ChainRhs::{Char, Lit};

    /// **`M = 1`, word for word**, transcribed from real `c2`'s obj for
    /// `int P(const char* s){ int r=0; while (*s) { int c=*s; r=r+c; s++; } return r; }`
    /// at `/O1 /GS- /c` (`work/w-varloop/probe.py n1`).
    #[test]
    fn one_step_chain_is_reproduced_word_for_word() {
        assert_eq!(
            emit(&[(Add, Char)]),
            vec![
                0x8963_0000, // lbz    r11,0(r3)
                0x7c6a_1b78, // mr     r10,r3
                0x3860_0000, // li     r3,0
                0x7d6b_0775, // extsb. r11,r11
                0x4d82_0020, // bclr   12,2
                0x8d2a_0001, // lbzu   r9,1(r10)
                0x7c6b_1a14, // add    r3,r11,r3
                0x7d2b_0775, // extsb. r11,r9
                0x4082_fff4, // bf     2,-12
                0x4e80_0020, // blr
            ]
        );
    }

    /// **`M = 3`, the cell w-sched2 §3.5 publishes**, and the one that shows the
    /// load is *not* at slot 0: `a` becomes 1 the moment the chain has three
    /// steps, and the record form moves with it.
    #[test]
    fn three_step_chain_puts_the_load_at_slot_one() {
        assert_eq!(
            emit(&[(Add, Char), (Xor, Lit(3)), (Add, Lit(5))]),
            vec![
                0x8963_0000, // lbz    r11,0(r3)
                0x7c6a_1b78, // mr     r10,r3
                0x3860_0000, // li     r3,0
                0x7d6b_0775, // extsb. r11,r11
                0x4d82_0020, // bclr   12,2
                0x7d6b_1a14, // add    r11,r11,r3   chain0 -> CHAR (S4r's clause)
                0x8d2a_0001, // lbzu   r9,1(r10)    a = 1
                0x6968_0003, // xori   r8,r11,3     chain1 -> T1
                0x7d2b_0775, // extsb. r11,r9       R = 3
                0x3868_0005, // addi   r3,r8,5      chain2 -> home
                0x4082_ffec, // bf     2,-20
                0x4e80_0020, // blr
            ]
        );
    }

    /// **`M = 4` — the regime flips**, and with it the entry form, the
    /// allocation, the record form's slot and the function's total length. This
    /// is S3m's threshold and #773's `JUMPIN`, in one pair of asserts with the
    /// cell above.
    #[test]
    fn four_step_chain_flips_to_the_same_regime_and_jumps_in() {
        assert_eq!(
            emit(&[(Add, Char), (Xor, Lit(3)), (Add, Lit(5)), (Or, Lit(9))]),
            vec![
                0x8963_0000, // lbz    r11,0(r3)
                0x7c6a_1b78, // mr     r10,r3
                0x3860_0000, // li     r3,0
                0x4800_0018, // b      .+24          JUMPIN, into the record form
                0x7d2b_1a14, // add    r9,r11,r3     chain0 -> T2
                0x8d6a_0001, // lbzu   r11,1(r10)    writes CHAR itself
                0x6929_0003, // xori   r9,r9,3
                0x3929_0005, // addi   r9,r9,5
                0x6123_0009, // ori    r3,r9,9       last -> home
                0x7d6b_0775, // extsb. r11,r11       R = M+1, the body's last word
                0x4082_ffe8, // bf     2,-24
                0x4e80_0020, // blr
            ]
        );
    }

    /// **S5, both poles in one test.** The same source operation `r = r + c`
    /// takes its operands in opposite orders depending on where in the chain it
    /// sits, because recency decides and the accumulator is the stalest operand
    /// at the head. A rule that predicted the right slot and the wrong register
    /// passes every position test and fails here.
    #[test]
    fn s5_orders_a_commutative_operand_pair_by_recency() {
        // `c` first: the character outranks the loop-carried accumulator.
        let head = emit(&[(Add, Char), (Xor, Lit(3))]);
        assert_eq!(head[6], 0x7d0b_1a14, "add r8,r11,r3 — CHAR in RA");
        // `c` last: this iteration's chain temp outranks the character.
        let tail = emit(&[(Xor, Lit(3)), (Add, Char)]);
        assert_eq!(tail[7], 0x7c68_5a14, "add r3,r8,r11 — CHAR in RB");
    }

    /// The unsigned element changes **two** words in the TWO regime and **one**
    /// in SAME, and never anything else. Measured with
    /// `work/w-varloop/probe.py --sig 'const unsigned char* s'`.
    #[test]
    fn the_element_signedness_moves_only_the_record_and_entry_forms() {
        let mut l = loop_of(&[(Add, Char), (Xor, Lit(3))]);
        let signed = words(&ptr_walk_chain_loop_text(&l, OptMode::O1).unwrap());
        l.elem_unsigned = true;
        let unsigned = words(&ptr_walk_chain_loop_text(&l, OptMode::O1).unwrap());
        assert_eq!(unsigned[3], 0x280b_0000, "cmplwi cr0,r11,0 as the entry test");
        assert_eq!(unsigned[7], 0x7d2b_4b79, "mr. r11,r9 as the record form");
        for i in 0..signed.len() {
            if i == 3 || i == 7 {
                continue;
            }
            assert_eq!(signed[i], unsigned[i], "word {i} moved with the signedness");
        }

        // SAME: the load already wrote CHAR, so the record form degenerates to
        // the compare and `mr.` never appears.
        let mut l = loop_of(&[(Add, Char), (Xor, Lit(3)), (Add, Lit(5)), (Or, Lit(9))]);
        l.elem_unsigned = true;
        let u = words(&ptr_walk_chain_loop_text(&l, OptMode::O1).unwrap());
        assert_eq!(u[9], 0x280b_0000, "cmplwi, not a move to itself");
    }

    /// **The length is a function of the chain**, which is the whole difference
    /// between this shape and `super::ptr_walk_loop`'s fixed eighty bytes. Both
    /// arms of the regime are exercised, and the back edge tracks the length.
    #[test]
    fn the_emitted_length_follows_the_chain_and_the_regime() {
        for m in 1..=3 {
            let ops: Vec<_> = std::iter::once((Add, Char))
                .chain((1..m).map(|k| (Xor, Lit(k as i32 + 2))))
                .collect();
            let w = emit(&ops);
            assert_eq!(w.len(), m + 9, "TWO regime is M+9 words at M={m}");
            // The back edge reaches exactly the body's first word.
            let disp = ((w[w.len() - 2] & 0xFFFC) as i32) - 0x1_0000;
            assert_eq!(disp, -4 * (m as i32 + 2), "back edge at M={m}");
        }
        for m in 4..=8 {
            let ops: Vec<_> = std::iter::once((Add, Char))
                .chain((1..m).map(|k| (Xor, Lit(k as i32 + 2))))
                .collect();
            let w = emit(&ops);
            assert_eq!(w.len(), m + 8, "SAME regime is M+8 words at M={m}");
            assert_eq!(w[3] & 0x03FF_FFFC, 4 * (m as u32 + 2), "JUMPIN at M={m}");
        }
    }

    /// A chain whose character is read late stays in the TWO regime however
    /// long it is — S3m is `pv == 0` **and** `M >= 4`, and the conjunction is
    /// load-bearing. w-sched2's S3 stated the same rule over `N` and was
    /// refuted at 125 of 131.
    #[test]
    fn a_late_character_read_stays_in_the_two_regime_at_every_length() {
        for m in 4..=8 {
            let mut ops: Vec<_> = (1..m).map(|k| (Xor, Lit(k as i32 + 2))).collect();
            ops.push((Add, Char));
            let w = emit(&ops);
            assert_eq!(w.len(), m + 9, "TWO regime at M={m} despite M>=4");
            assert_eq!(w[4], 0x4d82_0020, "the `bclr` guard, not a JUMPIN `b`");
        }
    }

    /// `/Ox` refuses: every cell behind these rules was captured at `/O1`.
    #[test]
    fn ox_refuses_because_nothing_graded_it_there() {
        assert!(ptr_walk_chain_loop_text(&loop_of(&[(Add, Char)]), OptMode::O1).is_ok());
        assert!(ptr_walk_chain_loop_text(&loop_of(&[(Add, Char)]), OptMode::Ox).is_err());
    }

    /// The arity gate is re-asserted in codegen, so a shape that ever reached
    /// here with a different formals list refuses rather than emitting the
    /// one-formal register plan over two.
    #[test]
    fn a_different_arity_refuses_in_codegen_too() {
        let mut l = loop_of(&[(Add, Char)]);
        l.params = vec![0x09E3, 0x09E4];
        assert!(ptr_walk_chain_loop_text(&l, OptMode::O1).is_err());
        l.params = vec![];
        assert!(ptr_walk_chain_loop_text(&l, OptMode::O1).is_err());
    }

    /// A chain that never reads the character refuses. Every rule here is
    /// stated in terms of `pv`, and w-sched2's reconstruction refuses the same
    /// population with the reason printed.
    #[test]
    fn a_chain_that_never_reads_the_character_refuses() {
        let l = loop_of(&[(Xor, Lit(3)), (Add, Lit(5))]);
        assert!(l.pv().is_none());
        assert!(ptr_walk_chain_loop_text(&l, OptMode::O1).is_err());
    }

    /// The schedule is a **bijection** onto the body's slots at every length
    /// and in both regimes: `M` chain steps plus the load plus the record form
    /// fill `M + 2` distinct indices. This is the property the `None` check in
    /// the emitter guards, asserted directly so that a schedule which doubled a
    /// slot is caught here rather than by an obj compare.
    #[test]
    fn the_schedule_is_a_bijection_onto_the_body_slots() {
        for m in 1..=10usize {
            for pv in 0..m {
                for &a in &[0usize, 1] {
                    let same = pv == 0 && m >= 4;
                    let p_chain = if !same && a == 1 && pv == 0 && m > 1 { 1 } else { pv };
                    let rec = if same {
                        m + 1
                    } else {
                        match (a + 2..=m + 1).find(|&c| slot_of(p_chain, a, c) < c) {
                            Some(r) => r,
                            None => continue,
                        }
                    };
                    let mut seen = vec![false; m + 2];
                    let mut mark = |s: usize| {
                        assert!(s < m + 2, "slot {s} outside a body of {} words", m + 2);
                        assert!(!seen[s], "slot {s} filled twice (M={m} pv={pv} a={a})");
                        seen[s] = true;
                    };
                    mark(a);
                    mark(rec);
                    for i in 0..m {
                        mark(slot_of(i, a, rec));
                    }
                    assert!(seen.iter().all(|&x| x), "M={m} pv={pv} a={a} left a hole");
                }
            }
        }
    }
}

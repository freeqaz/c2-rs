//! PPC (Xbox 360, big-endian) instruction selection for the MVP add-chain
//! function class. Scope: straight-line integer add + return only — exactly
//! what `int add3(int,int,int){return a+b+c;}` needs. See the `CODEGEN` spec.
//!
//! `.text` payload is stored **big-endian** (unlike the little-endian COFF
//! struct fields). Bit numbering below is IBM/PPC convention (bit 0 = MSB).

use c2_il::{IlFunction, IlOp};

use crate::BackendError;

/// Encode `add rD, rA, rB` (rD = rA + rB): primary opcode 31, XO 266, OE=0,
/// Rc=0. Returns the 4-byte big-endian instruction word.
///
/// `word = (31<<26) | (rd<<21) | (ra<<16) | (rb<<11) | (266<<1)`.
pub fn encode_add(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((rd as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | ((rb as u32 & 0x1F) << 11)
        | (266 << 1);
    word.to_be_bytes()
}

/// Encode `mullw rD, rA, rB` (rD = rA * rB): primary opcode 31, XO 235, OE=0,
/// Rc=0. Commutative in rA/rB (like `add`), so operand order is match-neutral.
pub fn encode_mullw(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((rd as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | ((rb as u32 & 0x1F) << 11)
        | (235 << 1);
    word.to_be_bytes()
}

/// Encode `subf rD, rA, rB`: primary opcode 31, XO 40, OE=0, Rc=0.
///
/// **Non-commutative — operand order is load-bearing.** `subf` computes
/// `rD = rB - rA` (the *first* register operand is the subtrahend). To realize
/// a source `lhs - rhs`, the caller must pass `ra = rhs` (subtrahend) and
/// `rb = lhs` (minuend). Swapping `ra`/`rb` silently negates the result — a
/// corruption invisible to `fuzzy%` (it is a valid `subf`, just the wrong one),
/// exactly the non-commutative hazard the CLAUDE.md correctness boundary names.
/// This encoder is deliberately separate from `encode_add` and its single
/// caller ([`select_text`]'s `Sub` arm) documents the mapping at the call site.
pub fn encode_subf(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((rd as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | ((rb as u32 & 0x1F) << 11)
        | (40 << 1);
    word.to_be_bytes()
}

/// Encode `addi rD, rA, SI` (rD = rA + sign-extended SI): primary opcode 14.
/// `SI` is a 16-bit signed immediate. Note `addi` special-cases `rA = 0` to
/// mean the literal 0 (not the contents of r0), so `addi rD, 0, k` is the
/// canonical `li rD, k`. Used for `reg ± small-constant` and constant loads.
pub fn encode_addi(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    let word: u32 =
        (14 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
    word.to_be_bytes()
}

/// Encode `addis rD, rA, SI` (rD = rA + (SI << 16)): primary opcode 15. The
/// high half of a wide constant / immediate (with rA=0 for the `lis` idiom).
pub fn encode_addis(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    let word: u32 =
        (15 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
    word.to_be_bytes()
}

/// Encode `ori rA, rS, UI` (rA = rS | UI): primary opcode 24. The low half of
/// a wide **constant load** (`lis`+`ori`); `UI` is a zero-extended 16-bit field.
pub fn encode_ori(ra: u8, rs: u8, ui: u16) -> [u8; 4] {
    let word: u32 =
        (24 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (ui as u32);
    word.to_be_bytes()
}

/// `blr` — branch to link register (function return). `bclr` with BO=20
/// ("always"), BI=0, LK=0 → the fixed word `0x4E800020`.
pub fn encode_blr() -> [u8; 4] {
    0x4E80_0020u32.to_be_bytes()
}

/// Emit `dest = reg + k` as one `addi` (16-bit) or an `addis`+`addi` pair for a
/// wide immediate. The pair splits `k` into a sign-compensated high half and a
/// sign-extended low half: `lo = (i16)k`, `hi = (k − lo) >> 16` (so the `addi`'s
/// sign extension is absorbed). Verified: `a+70000` → `addis r3,r3,1 ; addi
/// r3,r3,4464`; `a-70000` → `addis r3,r3,-1 ; addi r3,r3,-4464`.
fn emit_add_imm(text: &mut Vec<u8>, dest: u8, reg: u8, k: i32) {
    if fits_i16(k) {
        text.extend_from_slice(&encode_addi(dest, reg, k as i16));
    } else {
        let lo = (k & 0xFFFF) as u16 as i16;
        let hi = ((k - lo as i32) >> 16) as i16;
        text.extend_from_slice(&encode_addis(dest, reg, hi));
        text.extend_from_slice(&encode_addi(dest, dest, lo));
    }
}

/// Emit a constant load `dest = k`: `li` (`addi dest,r0,k`) for a 16-bit value,
/// else the `lis`+`ori` idiom (`addis dest,r0,hi ; ori dest,dest,lo`, unsigned
/// halves). Verified: `return 70000` → `addis r3,r0,1 ; ori r3,r3,4464`.
fn emit_load_imm(text: &mut Vec<u8>, dest: u8, k: i32) -> Result<(), BackendError> {
    if fits_i16(k) {
        text.extend_from_slice(&encode_addi(dest, 0, k as i16));
    } else if k >= 0 {
        let hi = ((k >> 16) & 0xFFFF) as i16;
        let lo = (k & 0xFFFF) as u16;
        text.extend_from_slice(&encode_addis(dest, 0, hi));
        text.extend_from_slice(&encode_ori(dest, dest, lo));
    } else {
        return Err(out_of_class("negative wide constant load not yet modeled"));
    }
    Ok(())
}

/// Encode an unconditional relative branch `b` (primary opcode 18, AA=0, LK=0)
/// used for a **tail call**, to be paired with a REL24 relocation.
///
/// MSVC's PPC convention stores the displacement field as `−(the instruction's
/// own byte offset within .text)`, so the pre-link value references `.text`
/// offset 0; the linker then patches the 24-bit field from the target symbol.
/// Verified: a tail-call `b` at offset 0 → `0x48000000`; at offset 8 →
/// `0x4BFFFFF8` (displacement −8).
pub fn encode_tail_branch(text_offset: u32) -> [u8; 4] {
    let disp = -(text_offset as i32);
    let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC);
    word.to_be_bytes()
}

/// An operand on the selection stack: a physical register or an integer
/// literal not yet materialized (folded into an immediate instruction where
/// c2 does the same, e.g. `a + 5` → `addi`).
#[derive(Clone, Copy, Debug)]
enum Operand {
    Reg(u8),
    Imm(i32),
}

/// True iff `k` fits PPC's 16-bit signed immediate field (`addi`/`subf` imm).
fn fits_i16(k: i32) -> bool {
    (-0x8000..=0x7FFF).contains(&k)
}

fn out_of_class(msg: &str) -> BackendError {
    BackendError::NotImplemented(msg.to_string())
}

/// Integer argument registers, left-to-right (Xbox 360 PPC / MSVC ABI).
const ARG_REGS: [u8; 8] = [3, 4, 5, 6, 7, 8, 9, 10];
/// Integer return register.
const RET_REG: u8 = 3;
/// First allocatable volatile scratch (r12 is reserved; COLOR picks r11 next).
const SCRATCH_REG: u8 = 11;

/// Select `.text` bytes for a straight-line integer-arithmetic function
/// (`+`, `-`, `*`; no branches/calls/relocations).
///
/// Params are pre-colored to the incoming ABI argument registers by position
/// (a→r3, b→r4, c→r5, …). The postfix `LOAD`/binary-op stream is walked over an
/// operand stack of physical registers: each binary op pops rhs then lhs and
/// emits its instruction into `dest` — the **final** binary op targets the
/// return register r3, every earlier one targets the running scratch r11. A
/// trailing `blr` returns.
///
/// Operand-order handling per op (the correctness-critical part):
/// * `Add` → `add dest, lhs, rhs` — commutative, order match-neutral.
/// * `Mul` → `mullw dest, lhs, rhs` — commutative, order match-neutral.
/// * `Sub` → `subf dest, rhs, lhs` — **non-commutative**. `subf` computes
///   `rB - rA`, so realizing `lhs - rhs` requires `rA = rhs`, `rB = lhs`; this
///   is the exact reversed mapping the reference c2 emits (`a-b-c` →
///   `subf r11,r4,r3 ; subf r3,r5,r11`). A swap here would be a fuzzy-invisible
///   sign inversion (CLAUDE.md correctness boundary) — see [`encode_subf`].
pub fn select_text(func: &IlFunction) -> Result<Vec<u8>, BackendError> {
    if func.params.len() > ARG_REGS.len() {
        return Err(BackendError::Pass {
            pass: "codegen".into(),
            msg: format!(
                "codegen supports up to {} register args, got {}",
                ARG_REGS.len(),
                func.params.len()
            ),
        });
    }

    // token -> incoming ABI register, by declaration order.
    let reg_of = |tok: u16| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .map(|i| ARG_REGS[i])
    };

    let n_binops = func
        .ops
        .iter()
        .filter(|op| matches!(op, IlOp::Add | IlOp::Sub | IlOp::Mul))
        .count();

    let mut stack: Vec<Operand> = Vec::new();
    let mut text: Vec<u8> = Vec::new();
    let mut binop_idx = 0usize;

    for op in &func.ops {
        match op {
            IlOp::Load(tok) => {
                let reg = reg_of(*tok).ok_or_else(|| BackendError::Pass {
                    pass: "codegen".into(),
                    msg: format!("LOAD of unknown token 0x{tok:04X} (not a parameter)"),
                })?;
                stack.push(Operand::Reg(reg));
            }
            IlOp::Lit(k) => stack.push(Operand::Imm(*k)),
            IlOp::Add | IlOp::Sub | IlOp::Mul => {
                // Binary op: pop rhs then lhs.
                let rhs = stack.pop().ok_or_else(|| out_of_class("binary op: empty stack (rhs)"))?;
                let lhs = stack.pop().ok_or_else(|| out_of_class("binary op: empty stack (lhs)"))?;
                binop_idx += 1;
                let dest = if binop_idx == n_binops { RET_REG } else { SCRATCH_REG };
                let result = emit_binop(*op, dest, lhs, rhs, &mut text)?;
                stack.push(result);
            }
        }
        // Single-scratch (r11) selection is correct only for a **serial
        // accumulator chain** (operand stack depth ≤ 2: one running result +
        // one fresh operand). A tree like `(a+b)*(c+d)` reaches depth 3 and
        // needs a second scratch; emitting it with one would silently clobber
        // the first result. Reject as out-of-class rather than mis-emit.
        if stack.len() > 2 {
            return Err(out_of_class(
                "expression is not a serial accumulator chain (operand stack \
                 depth > 2 → needs more than one scratch register); outside the \
                 current straight-line class",
            ));
        }
    }

    // Materialize the single remaining operand into the return register r3.
    match stack.as_slice() {
        [Operand::Reg(RET_REG)] => {} // chain already ended in r3
        [Operand::Reg(other)] => {
            // A bare `return param` where the value is not already in r3 (e.g.
            // `return b;`) needs a register move — not yet modeled.
            return Err(out_of_class(&format!(
                "result is in r{other}, not the return register r3 (bare \
                 non-first-param return not yet handled)"
            )));
        }
        [Operand::Imm(k)] => {
            // Bare constant return, e.g. `return 42;` → `li r3,k`; wide → lis+ori.
            emit_load_imm(&mut text, RET_REG, *k)?;
        }
        _ => {
            return Err(out_of_class(
                "expression did not reduce to a single value (malformed or out of class)",
            ))
        }
    }

    text.extend_from_slice(&encode_blr());
    Ok(text)
}

/// Emit one binary op into `text`, returning the result operand. Handles the
/// register/immediate operand combinations c2 folds into a single instruction;
/// rejects (as out-of-class) the shapes that need a different instruction than
/// this class models (immediate multiply → strength reduction; `imm - reg` →
/// `subfic`; out-of-range immediates → `addis`+`addi`).
fn emit_binop(
    op: IlOp,
    dest: u8,
    lhs: Operand,
    rhs: Operand,
    text: &mut Vec<u8>,
) -> Result<Operand, BackendError> {
    use Operand::{Imm, Reg};
    match (op, lhs, rhs) {
        // add: commutative; reg+reg → add; reg+imm (either order) → addi / addis+addi.
        (IlOp::Add, Reg(a), Reg(b)) => text.extend_from_slice(&encode_add(dest, a, b)),
        (IlOp::Add, Reg(a), Imm(k)) | (IlOp::Add, Imm(k), Reg(a)) => emit_add_imm(text, dest, a, k),
        // mul: commutative; reg*reg only (reg*const strength-reduces — later rung).
        (IlOp::Mul, Reg(a), Reg(b)) => text.extend_from_slice(&encode_mullw(dest, a, b)),
        (IlOp::Mul, _, _) => {
            return Err(out_of_class(
                "multiply by a constant strength-reduces (shift/add); out of class",
            ))
        }
        // sub `lhs - rhs`: reg-reg → subf (rA=rhs); reg-imm → add of the negated imm.
        (IlOp::Sub, Reg(a), Reg(b)) => text.extend_from_slice(&encode_subf(dest, b, a)),
        (IlOp::Sub, Reg(a), Imm(k)) => {
            let neg = k
                .checked_neg()
                .ok_or_else(|| out_of_class("subtract immediate overflow (INT_MIN)"))?;
            emit_add_imm(text, dest, a, neg);
        }
        (IlOp::Sub, Imm(_), Reg(_)) => {
            return Err(out_of_class("`const - reg` needs subfic; out of class"))
        }
        // Two literals should have been constant-folded by the front-end.
        (_, Imm(_), Imm(_)) => {
            return Err(out_of_class("binary op on two literals (unexpected; c1xx folds these)"))
        }
        (IlOp::Load(_) | IlOp::Lit(_), _, _) => unreachable!("not a binary op"),
    }
    Ok(Operand::Reg(dest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2_il::IlOp;

    #[test]
    fn encode_add_matches_reference_words() {
        assert_eq!(encode_add(11, 3, 4), [0x7D, 0x63, 0x22, 0x14]);
        assert_eq!(encode_add(3, 11, 5), [0x7C, 0x6B, 0x2A, 0x14]);
    }

    #[test]
    fn encode_blr_is_fixed() {
        assert_eq!(encode_blr(), [0x4E, 0x80, 0x00, 0x20]);
    }

    #[test]
    fn encode_tail_branch_stores_negative_self_offset() {
        // Tail-call `b` displacement = −(own .text offset): offset 0 → 0x48000000,
        // offset 8 → 0x4BFFFFF8 (the REL24 reloc patches the target).
        assert_eq!(encode_tail_branch(0), [0x48, 0x00, 0x00, 0x00]);
        assert_eq!(encode_tail_branch(8), [0x4B, 0xFF, 0xFF, 0xF8]);
    }

    #[test]
    fn encode_mullw_matches_reference_words() {
        // a*b*c → mullw r11,r3,r4 ; mullw r3,r11,r5
        assert_eq!(encode_mullw(11, 3, 4), [0x7D, 0x63, 0x21, 0xD6]);
        assert_eq!(encode_mullw(3, 11, 5), [0x7C, 0x6B, 0x29, 0xD6]);
    }

    #[test]
    fn encode_subf_matches_reference_words() {
        // a-b-c → subf r11,r4,r3 ; subf r3,r5,r11 (rA = subtrahend).
        assert_eq!(encode_subf(11, 4, 3), [0x7D, 0x64, 0x18, 0x50]);
        assert_eq!(encode_subf(3, 5, 11), [0x7C, 0x65, 0x58, 0x50]);
    }

    #[test]
    fn select_text_sub_uses_reversed_operands() {
        // `a - b - c`: LOAD a, LOAD b, SUB, LOAD c, SUB. The subf operand order
        // (rA=rhs, rB=lhs) must reproduce c2's `subf r11,r4,r3 ; subf r3,r5,r11`.
        let func = IlFunction {
            mangled_name: "?sub3@@YAHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            params: vec![0xE309, 0xE409, 0xE509],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Sub,
                IlOp::Load(0xE509),
                IlOp::Sub,
            ],
        };
        assert_eq!(
            select_text(&func).unwrap(),
            vec![
                0x7D, 0x64, 0x18, 0x50, // subf r11,r4,r3  (= r3-r4 = a-b)
                0x7C, 0x65, 0x58, 0x50, // subf r3,r5,r11  (= r11-r5 = (a-b)-c)
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    #[test]
    fn encode_addi_matches_reference_words() {
        assert_eq!(encode_addi(3, 3, 5), [0x38, 0x63, 0x00, 0x05]); // a+5
        assert_eq!(encode_addi(3, 3, -5), [0x38, 0x63, 0xFF, 0xFB]); // a-5
        assert_eq!(encode_addi(3, 0, 42), [0x38, 0x60, 0x00, 0x2A]); // li r3,42
    }

    fn func_with(params: Vec<u16>, ops: Vec<IlOp>) -> IlFunction {
        IlFunction {
            mangled_name: "?f@@YAHH@Z".into(),
            source_path: None,
            tail_call: None,
            params,
            ops,
        }
    }

    #[test]
    fn select_text_add_immediate() {
        // `a + 5` → addi r3,r3,5 ; blr
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(5), IlOp::Add]);
        assert_eq!(
            select_text(&f).unwrap(),
            vec![0x38, 0x63, 0x00, 0x05, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_sub_immediate_folds_to_addi_neg() {
        // `a - 5` → addi r3,r3,-5 ; blr
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(5), IlOp::Sub]);
        assert_eq!(
            select_text(&f).unwrap(),
            vec![0x38, 0x63, 0xFF, 0xFB, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_bare_constant_return_is_li() {
        // `return 42;` → addi r3,r0,42 (li) ; blr
        let f = func_with(vec![], vec![IlOp::Lit(42)]);
        assert_eq!(
            select_text(&f).unwrap(),
            vec![0x38, 0x60, 0x00, 0x2A, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_rejects_immediate_multiply() {
        // `a * 3` strength-reduces (out of class) — must reject, not mis-emit.
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(3), IlOp::Mul]);
        assert!(matches!(select_text(&f), Err(BackendError::NotImplemented(_))));
    }

    #[test]
    fn select_text_wide_add_immediate_uses_addis_addi() {
        // `a + 70000` → addis r3,r3,1 ; addi r3,r3,4464 ; blr.
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(70000), IlOp::Add]);
        assert_eq!(
            select_text(&f).unwrap(),
            vec![
                0x3C, 0x63, 0x00, 0x01, // addis r3,r3,1
                0x38, 0x63, 0x11, 0x70, // addi r3,r3,4464
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    #[test]
    fn select_text_wide_constant_load_uses_lis_ori() {
        // `return 70000;` → addis r3,r0,1 ; ori r3,r3,4464 ; blr.
        let f = func_with(vec![], vec![IlOp::Lit(70000)]);
        assert_eq!(
            select_text(&f).unwrap(),
            vec![
                0x3C, 0x60, 0x00, 0x01, // addis r3,r0,1
                0x60, 0x63, 0x11, 0x70, // ori r3,r3,4464
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    #[test]
    fn select_text_rejects_tree_expression() {
        // `(a+b)*(c+d)` is postfix LOAD a,b,ADD,LOAD c,d,ADD,MUL — the operand
        // stack reaches depth 3, needing a second scratch. Must be rejected
        // (NotImplemented), NOT silently mis-emitted with one scratch.
        let func = IlFunction {
            mangled_name: "?t@@YAHHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            params: vec![0xE309, 0xE409, 0xE509, 0xE609],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Load(0xE609),
                IlOp::Add,
                IlOp::Mul,
            ],
        };
        assert!(matches!(
            select_text(&func),
            Err(BackendError::NotImplemented(_))
        ));
    }

    #[test]
    fn select_text_mul_is_commutative_order() {
        // `a * b * c` → mullw r11,r3,r4 ; mullw r3,r11,r5 ; blr.
        let func = IlFunction {
            mangled_name: "?mul3@@YAHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            params: vec![0xE309, 0xE409, 0xE509],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Mul,
            ],
        };
        assert_eq!(
            select_text(&func).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7C, 0x6B, 0x29, 0xD6, // mullw r3,r11,r5
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    #[test]
    fn select_text_for_add3() {
        let func = IlFunction {
            mangled_name: "?add3@@YAHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            params: vec![0xE309, 0xE409, 0xE509],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Add,
            ],
        };
        let text = select_text(&func).unwrap();
        assert_eq!(
            text,
            vec![
                0x7D, 0x63, 0x22, 0x14, // add r11,r3,r4
                0x7C, 0x6B, 0x2A, 0x14, // add r3,r11,r5
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }
}

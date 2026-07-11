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

/// `blr` — branch to link register (function return). `bclr` with BO=20
/// ("always"), BI=0, LK=0 → the fixed word `0x4E800020`.
pub fn encode_blr() -> [u8; 4] {
    0x4E80_0020u32.to_be_bytes()
}

/// Integer argument registers, left-to-right (Xbox 360 PPC / MSVC ABI).
const ARG_REGS: [u8; 8] = [3, 4, 5, 6, 7, 8, 9, 10];
/// Integer return register.
const RET_REG: u8 = 3;
/// First allocatable volatile scratch (r12 is reserved; COLOR picks r11 next).
const SCRATCH_REG: u8 = 11;

/// Select `.text` bytes for an MVP add-chain function.
///
/// Params are pre-colored to the incoming ABI argument registers by position
/// (a→r3, b→r4, c→r5, …). The postfix `LOAD`/`ADD` stream is walked over an
/// operand stack of physical registers: each `ADD` pops rhs then lhs, emits
/// `add dest, lhs, rhs`, and pushes dest — the **final** ADD targets the return
/// register r3, every earlier ADD targets the running scratch r11. A trailing
/// `blr` returns. For `add3` this yields exactly
/// `add r11,r3,r4 ; add r3,r11,r5 ; blr`.
///
/// Restricted to commutative integer `add` on purpose: operand order for `add`
/// is match-neutral (rA↔rB), so no silent non-commutative corruption is
/// possible here (see CLAUDE.md correctness boundary). Non-commutative ops
/// (`-`, shifts, compares) are intentionally NOT handled.
pub fn select_text(func: &IlFunction) -> Result<Vec<u8>, BackendError> {
    if func.params.len() > ARG_REGS.len() {
        return Err(BackendError::Pass {
            pass: "codegen".into(),
            msg: format!(
                "MVP codegen supports up to {} register args, got {}",
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

    let n_adds = func.ops.iter().filter(|op| matches!(op, IlOp::Add)).count();
    if n_adds == 0 {
        return Err(BackendError::Pass {
            pass: "codegen".into(),
            msg: "MVP codegen requires at least one ADD in the body".into(),
        });
    }

    let mut stack: Vec<u8> = Vec::new();
    let mut text: Vec<u8> = Vec::new();
    let mut add_idx = 0usize;

    for op in &func.ops {
        match op {
            IlOp::Load(tok) => {
                let reg = reg_of(*tok).ok_or_else(|| BackendError::Pass {
                    pass: "codegen".into(),
                    msg: format!("LOAD of unknown token 0x{tok:04X} (not a parameter)"),
                })?;
                stack.push(reg);
            }
            IlOp::Add => {
                let rhs = stack.pop().ok_or_else(|| BackendError::Pass {
                    pass: "codegen".into(),
                    msg: "ADD with empty operand stack (rhs)".into(),
                })?;
                let lhs = stack.pop().ok_or_else(|| BackendError::Pass {
                    pass: "codegen".into(),
                    msg: "ADD with empty operand stack (lhs)".into(),
                })?;
                add_idx += 1;
                let dest = if add_idx == n_adds { RET_REG } else { SCRATCH_REG };
                text.extend_from_slice(&encode_add(dest, lhs, rhs));
                stack.push(dest);
            }
        }
    }

    text.extend_from_slice(&encode_blr());
    Ok(text)
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
    fn select_text_for_add3() {
        let func = IlFunction {
            mangled_name: "?add3@@YAHHHH@Z".into(),
            source_path: None,
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

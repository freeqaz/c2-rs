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

/// Encode a **linking** relative branch `bl` (primary opcode 18, AA=0, **LK=1**)
/// used for a non-leaf `bl <callee>` inside a framed function, paired with a
/// REL24 relocation. Same MSVC displacement convention as [`encode_tail_branch`]
/// (`disp = −(own .text offset)`) plus the link bit. Verified: `bl` at offset
/// 0xC → `0x4BFFFFF5` (disp −0xC, LK=1).
pub fn encode_call_branch(text_offset: u32) -> [u8; 4] {
    let disp = -(text_offset as i32);
    let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC) | 1;
    word.to_be_bytes()
}

/// The `.text` byte offset of the `bl` instruction inside the constant framed
/// body (after the 3-word prologue). The caller returns this as the REL24
/// relocation site. Constant for the `return g(a) + k` frame class.
pub const FRAMED_BL_OFFSET: u32 = 0x0C;

/// Emit the `.text` for a **framed non-leaf call** `return g(a) + k` (W4b2).
///
/// The whole body is byte-constant except the post-call `addi r3,r3,k`
/// immediate and the `bl` target (patched by the REL24 relocation the caller
/// registers at [`FRAMED_BL_OFFSET`]). Verified anatomy (0x24 bytes, 9 words),
/// constant across 1/2/4 callee args — the frame is always 96 bytes:
///
/// ```text
/// 7d8802a6  mflr r12                prologue (3 words): save LR
/// 9181fff8  stw  r12,-8(r1)
/// 9421ffa0  stwu r1,-96(r1)         allocate the fixed 96-byte frame
/// 4bfffff5  bl   <callee>           REL24 reloc site at .text+0xC
/// 3863kkkk  addi r3,r3,k            the post-call op (+k)
/// 38210060  addi r1,r1,96           epilogue (4 words): free frame
/// 8181fff8  lwz  r12,-8(r1)         restore LR
/// 7d8803a6  mtlr r12
/// 4e800020  blr
/// ```
///
/// `k` must fit the signed-16-bit `addi` immediate (the IL parser guarantees
/// this before constructing the [`c2_il::FramedCall`]).
pub fn framed_call_text(add_k: i32) -> Vec<u8> {
    let k = add_k as i16; // range-checked upstream (c2_il::func::parse_segment)
    let mut text = Vec::with_capacity(0x24);
    // Prologue.
    text.extend_from_slice(&0x7D88_02A6u32.to_be_bytes()); // mflr r12
    text.extend_from_slice(&0x9181_FFF8u32.to_be_bytes()); // stw  r12,-8(r1)
    text.extend_from_slice(&0x9421_FFA0u32.to_be_bytes()); // stwu r1,-96(r1)
    // Call (LK=1); the REL24 reloc at FRAMED_BL_OFFSET patches the target.
    text.extend_from_slice(&encode_call_branch(FRAMED_BL_OFFSET)); // bl <callee>
    // Post-call op.
    text.extend_from_slice(&encode_addi(RET_REG, RET_REG, k)); // addi r3,r3,k
    // Epilogue.
    text.extend_from_slice(&0x3821_0060u32.to_be_bytes()); // addi r1,r1,96
    text.extend_from_slice(&0x8181_FFF8u32.to_be_bytes()); // lwz  r12,-8(r1)
    text.extend_from_slice(&0x7D88_03A6u32.to_be_bytes()); // mtlr r12
    text.extend_from_slice(&encode_blr()); // blr
    debug_assert_eq!(text.len(), 0x24);
    text
}

/// Emit the `.text` for an **integer tail call** `return g(<arg>)` (and the
/// identity-fold `g(a) + 0`): the single call argument computed into r3 by the
/// leaf arithmetic selector, then a `b <callee>` tail branch (paired with a
/// REL24 relocation the caller registers at the returned offset). Returns the
/// `.text` bytes and the branch's byte offset within `.text` (= `base_off` +
/// the arg-setup length; the reloc site).
///
/// The argument setup reuses [`select_text`] (params → r3, the exact same
/// instruction class as a leaf `return <arg>`) and drops its trailing `blr`:
/// the argument value stays live in r3 across the tail branch. A passthrough
/// `g(a)` selects to just `blr` → an empty prefix → a bare `b g` at `base_off`,
/// byte-identical to the void tail call; `g(a + 1)` selects `addi r3,r3,1 ; blr`
/// → the prefix `addi r3,r3,1` and the branch at `base_off + 4`. Non-commutative
/// arg-setup (`k - a`, shifts) is rejected inside `select_text` (fail closed),
/// honoring the CLAUDE.md correctness boundary; `a + k`/`a - k` fold to `addi`.
pub fn int_tail_call_text(
    func: &IlFunction,
    base_off: u32,
) -> Result<(Vec<u8>, u32), BackendError> {
    let mut text = select_text(func)?; // arg-setup + trailing blr
    let blr = encode_blr();
    debug_assert!(
        text.ends_with(&blr),
        "select_text always terminates in blr; the arg-setup is everything before it"
    );
    text.truncate(text.len() - blr.len()); // drop blr; the value stays live in r3
    let branch_off = base_off + text.len() as u32;
    text.extend_from_slice(&encode_tail_branch(branch_off));
    Ok((text, branch_off))
}

/// The base of an affine selection-stack value: either a concrete physical
/// register (a loaded parameter) or `Prev` — the running result of the most
/// recent emitted reg-reg instruction. `Prev` resolves to the scratch register
/// r11: any reg-reg result that is *read again* is by construction not the final
/// instruction (the final one lands in r3), so every consumed intermediate lives
/// in r11 (the single-scratch serial-chain invariant).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Base {
    Phys(u8),
    Prev,
}

/// An operand on the selection stack. c2 constant-folds a chain of immediate
/// additions/subtractions (`a + 5 + 5` → `a + 10`, one `addi`), so a value is
/// modeled **affinely** as an optional base register plus a pending immediate
/// offset — the offset accumulates for free and is materialized as a single
/// `addi` (or `addis`+`addi`) only when the value is finalized.
#[derive(Clone, Copy, Debug)]
enum Operand {
    /// A pure integer literal (no register component), not yet materialized.
    Imm(i32),
    /// `base + off`: a register value plus a folded constant offset (`off == 0`
    /// is a bare register). The offset materializes lazily; a reg-reg op
    /// requires `off == 0` (a pending offset there is out of the serial-chain
    /// class → fail closed).
    RegOff { base: Base, off: i32 },
}

/// One planned emission, in evaluation order. The **destination** register is
/// resolved by position at emit time — the last plan entry targets the return
/// register r3, every earlier one the scratch r11 — so folding that removes an
/// emission automatically re-targets the survivor (e.g. the single folded
/// `addi r3,r3,10` for `a + 5 + 5`) without a separate counter.
#[derive(Clone, Copy, Debug)]
enum PlanOp {
    /// A commutative/register binary op with both source registers resolved
    /// (`Base::Prev` → r11); `Sub` keeps its load-bearing operand order.
    Bin { op: IlOp, lhs: u8, rhs: u8 },
    /// Materialize a pending offset: `dest = src + k` (`addi`, or `addis`+`addi`
    /// when wide). The final flush of an affine `reg + off` value.
    AddImm { src: u8, k: i32 },
    /// Materialize a bare constant return: `dest = k` (`li`, or `lis`+`ori`).
    LoadImm { k: i32 },
}

impl Base {
    /// Resolve to the physical register a *read* of this base uses.
    fn read_reg(self) -> u8 {
        match self {
            Base::Phys(r) => r,
            Base::Prev => SCRATCH_REG,
        }
    }
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

    let mut stack: Vec<Operand> = Vec::new();
    let mut plan: Vec<PlanOp> = Vec::new();

    for op in &func.ops {
        match op {
            IlOp::Load(tok) => {
                let reg = reg_of(*tok).ok_or_else(|| BackendError::Pass {
                    pass: "codegen".into(),
                    msg: format!("LOAD of unknown token 0x{tok:04X} (not a parameter)"),
                })?;
                stack.push(Operand::RegOff { base: Base::Phys(reg), off: 0 });
            }
            IlOp::Lit(k) => stack.push(Operand::Imm(*k)),
            IlOp::Add | IlOp::Sub | IlOp::Mul => {
                // Binary op: pop rhs then lhs.
                let rhs = stack.pop().ok_or_else(|| out_of_class("binary op: empty stack (rhs)"))?;
                let lhs = stack.pop().ok_or_else(|| out_of_class("binary op: empty stack (lhs)"))?;
                let result = combine(*op, lhs, rhs, &mut plan)?;
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

    // Finalize the single remaining value into the return register r3. A pending
    // offset (or a bare literal) becomes the last plan entry, so it materializes
    // into r3 (see [`PlanOp`] dest resolution).
    match stack.as_slice() {
        [Operand::RegOff { base, off }] => {
            if *off != 0 {
                plan.push(PlanOp::AddImm { src: base.read_reg(), k: *off });
            } else {
                match base {
                    // Chain already ended in r3 (the last reg-reg op targets it),
                    // or a bare `return a` where the parameter is already in r3.
                    Base::Prev | Base::Phys(RET_REG) => {}
                    // A bare `return param` whose value is not in r3 (e.g.
                    // `return b;`) needs a register move — not yet modeled.
                    Base::Phys(other) => {
                        return Err(out_of_class(&format!(
                            "result is in r{other}, not the return register r3 \
                             (bare non-first-param return not yet handled)"
                        )));
                    }
                }
            }
        }
        [Operand::Imm(k)] => {
            // Bare constant return, e.g. `return 42;` → `li r3,k`; wide → lis+ori.
            plan.push(PlanOp::LoadImm { k: *k });
        }
        _ => {
            return Err(out_of_class(
                "expression did not reduce to a single value (malformed or out of class)",
            ))
        }
    }

    // Emit the plan: the **last** entry targets the return register r3, every
    // earlier one the scratch r11 (the single-scratch serial-chain invariant).
    let mut text: Vec<u8> = Vec::new();
    let last = plan.len().saturating_sub(1);
    for (i, entry) in plan.iter().enumerate() {
        let dest = if i == last { RET_REG } else { SCRATCH_REG };
        match *entry {
            PlanOp::Bin { op, lhs, rhs } => match op {
                IlOp::Add => text.extend_from_slice(&encode_add(dest, lhs, rhs)),
                IlOp::Mul => text.extend_from_slice(&encode_mullw(dest, lhs, rhs)),
                // `subf` computes rB − rA, so realizing `lhs − rhs` needs rA=rhs,
                // rB=lhs (the load-bearing reversed order — see [`encode_subf`]).
                IlOp::Sub => text.extend_from_slice(&encode_subf(dest, rhs, lhs)),
                IlOp::Load(_) | IlOp::Lit(_) => unreachable!("not a binary op"),
            },
            PlanOp::AddImm { src, k } => emit_add_imm(&mut text, dest, src, k),
            PlanOp::LoadImm { k } => emit_load_imm(&mut text, dest, k)?,
        }
    }

    text.extend_from_slice(&encode_blr());
    Ok(text)
}

/// Fold one binary op over the affine operand stack, recording a [`PlanOp`] only
/// when a register instruction is actually needed. Immediate accumulations fold
/// for free (`a + 5 + 5` → `a + 10`, matching c2's constant folding); a reg-reg
/// op requires both operands to be bare registers (`off == 0`) — a pending
/// offset there is outside the serial-chain class and fails closed. Rejects the
/// shapes needing an instruction this class does not model (immediate multiply →
/// strength reduction; `imm - reg` → `subfic`).
fn combine(
    op: IlOp,
    lhs: Operand,
    rhs: Operand,
    plan: &mut Vec<PlanOp>,
) -> Result<Operand, BackendError> {
    use Operand::{Imm, RegOff};

    // Emit a reg-reg instruction and return its running result (r11 via `Prev`).
    let mut emit_reg_reg = |op: IlOp, a: Base, b: Base| -> Result<Operand, BackendError> {
        plan.push(PlanOp::Bin { op, lhs: a.read_reg(), rhs: b.read_reg() });
        Ok(RegOff { base: Base::Prev, off: 0 })
    };

    match (op, lhs, rhs) {
        // ---- Add (commutative) ------------------------------------------------
        (IlOp::Add, Imm(a), Imm(b)) => Ok(Imm(a
            .checked_add(b)
            .ok_or_else(|| out_of_class("constant add overflow"))?)),
        (IlOp::Add, RegOff { base, off }, Imm(k)) | (IlOp::Add, Imm(k), RegOff { base, off }) => {
            let off = off
                .checked_add(k)
                .ok_or_else(|| out_of_class("folded add-immediate overflow"))?;
            Ok(RegOff { base, off })
        }
        (IlOp::Add, RegOff { base: a, off: 0 }, RegOff { base: b, off: 0 }) => {
            emit_reg_reg(IlOp::Add, a, b)
        }
        (IlOp::Add, RegOff { .. }, RegOff { .. }) => Err(out_of_class(
            "reg+reg add with a pending immediate offset (non-serial chain); out of class",
        )),

        // ---- Sub (`lhs - rhs`, NON-commutative) -------------------------------
        (IlOp::Sub, Imm(a), Imm(b)) => Ok(Imm(a
            .checked_sub(b)
            .ok_or_else(|| out_of_class("constant sub overflow"))?)),
        // reg − imm folds by *subtracting* into the running offset (no negate,
        // no INT_MIN hazard — `emit_add_imm` handles the sign at materialization).
        (IlOp::Sub, RegOff { base, off }, Imm(k)) => {
            let off = off
                .checked_sub(k)
                .ok_or_else(|| out_of_class("folded sub-immediate overflow"))?;
            Ok(RegOff { base, off })
        }
        (IlOp::Sub, Imm(_), RegOff { .. }) => {
            Err(out_of_class("`const - reg` needs subfic; out of class"))
        }
        (IlOp::Sub, RegOff { base: a, off: 0 }, RegOff { base: b, off: 0 }) => {
            emit_reg_reg(IlOp::Sub, a, b)
        }
        (IlOp::Sub, RegOff { .. }, RegOff { .. }) => Err(out_of_class(
            "reg-reg subtract with a pending immediate offset (non-serial chain); out of class",
        )),

        // ---- Mul (commutative) ------------------------------------------------
        (IlOp::Mul, RegOff { base: a, off: 0 }, RegOff { base: b, off: 0 }) => {
            emit_reg_reg(IlOp::Mul, a, b)
        }
        // reg*const strength-reduces, and const*const is unexpected (c1xx folds).
        (IlOp::Mul, _, _) => Err(out_of_class(
            "multiply by a constant strength-reduces (shift/add); out of class",
        )),

        (IlOp::Load(_) | IlOp::Lit(_), _, _) => unreachable!("not a binary op"),
    }
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
    fn encode_call_branch_sets_link_bit() {
        // `bl` at offset 0xC → disp −0xC, LK=1 → 0x4BFFFFF5 (reference `bl g`).
        assert_eq!(encode_call_branch(0x0C), [0x4B, 0xFF, 0xFF, 0xF5]);
    }

    #[test]
    fn framed_call_text_matches_reference_body() {
        // `int f(int a){ return g(a) + 1; }` — the verified 0x24-byte body.
        assert_eq!(
            framed_call_text(1),
            vec![
                0x7D, 0x88, 0x02, 0xA6, // mflr r12
                0x91, 0x81, 0xFF, 0xF8, // stw  r12,-8(r1)
                0x94, 0x21, 0xFF, 0xA0, // stwu r1,-96(r1)
                0x4B, 0xFF, 0xFF, 0xF5, // bl   g (REL24 @ 0xC)
                0x38, 0x63, 0x00, 0x01, // addi r3,r3,1
                0x38, 0x21, 0x00, 0x60, // addi r1,r1,96
                0x81, 0x81, 0xFF, 0xF8, // lwz  r12,-8(r1)
                0x7D, 0x88, 0x03, 0xA6, // mtlr r12
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
        // `+ 2` differs only in the addi immediate.
        assert_eq!(framed_call_text(2)[19], 0x02);
        assert_eq!(framed_call_text(1).len(), 0x24);
    }

    #[test]
    fn int_tail_call_passthrough_is_bare_branch() {
        // `return g(a)` — a is already in r3, so no arg setup: a bare `b g` at
        // offset 0 (`48000000`), reloc site 0 — byte-identical to the void
        // tail call. Verified against the live obj (.text=48000000, REL24 @0x0).
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309)]);
        let (text, reloc) = int_tail_call_text(&f, 0).unwrap();
        assert_eq!(text, vec![0x48, 0x00, 0x00, 0x00]);
        assert_eq!(reloc, 0);
    }

    #[test]
    fn int_tail_call_arg_setup_prepends_addi() {
        // `return g(a + 1)` — the argument a+1 computed into r3 (`addi r3,r3,1`),
        // then `b g` at offset 4 (`4bfffffc`, disp −4), reloc site 0x4. Verified
        // against the live obj (.text=386300014bfffffc, REL24 @0x4).
        let f = func_with(
            vec![0xE309],
            vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add],
        );
        let (text, reloc) = int_tail_call_text(&f, 0).unwrap();
        assert_eq!(
            text,
            vec![
                0x38, 0x63, 0x00, 0x01, // addi r3,r3,1
                0x4B, 0xFF, 0xFF, 0xFC, // b g (disp −4)
            ]
        );
        assert_eq!(reloc, 4);
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
            framed_call: None,
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
            framed_call: None,
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
    fn select_text_folds_consecutive_add_immediates() {
        // `a + 5 + 5` → the two literal adds fold to a single `addi r3,r3,10`
        // (c2 constant-folds `5 + 5` → `10`), NOT two chained addi. Verified
        // against the live obj (mvp_edit_addk2: .text = 3863000a 4e800020).
        let f = func_with(
            vec![0xE309],
            vec![
                IlOp::Load(0xE309),
                IlOp::Lit(5),
                IlOp::Add,
                IlOp::Lit(5),
                IlOp::Add,
            ],
        );
        assert_eq!(
            select_text(&f).unwrap(),
            vec![0x38, 0x63, 0x00, 0x0A, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_folds_mixed_add_sub_immediates() {
        // `a + 5 - 3` folds to `a + 2` → `addi r3,r3,2 ; blr`.
        let f = func_with(
            vec![0xE309],
            vec![
                IlOp::Load(0xE309),
                IlOp::Lit(5),
                IlOp::Add,
                IlOp::Lit(3),
                IlOp::Sub,
            ],
        );
        assert_eq!(
            select_text(&f).unwrap(),
            vec![0x38, 0x63, 0x00, 0x02, 0x4E, 0x80, 0x00, 0x20]
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
            framed_call: None,
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
            framed_call: None,
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
            framed_call: None,
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

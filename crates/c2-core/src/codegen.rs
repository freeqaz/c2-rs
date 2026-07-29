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

// ---- W6: comparison → boolean materialization encoders ---------------------
//
// c2 materializes integer comparisons **branchlessly** — it emits no
// `cmpw`/`cmplw` at all for a `return a <rel> k` leaf, but instead carry-bit and
// bit-extraction idioms (see docs/CODEGEN_W6_COMPARE.md, where every word below
// is matched against a live capture). Several of these are non-commutative and
// their operand order is load-bearing exactly like [`encode_subf`]'s.

/// `addic rD, rA, SIMM` (rD = rA + SIMM, **setting CA**): primary opcode 12.
/// The carry-out is the point: `addic rD,rX,-1` sets CA iff `rX != 0`.
pub fn encode_addic(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    let word: u32 =
        (12 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
    word.to_be_bytes()
}

/// `subfic rD, rA, SIMM` (rD = SIMM − rA, setting CA): primary opcode 8.
/// **Non-commutative**: the immediate is the minuend, the register the
/// subtrahend. CA is set iff `rA <= SIMM` unsigned.
pub fn encode_subfic(rd: u8, ra: u8, si: i16) -> [u8; 4] {
    let word: u32 =
        (8 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (si as u16 as u32);
    word.to_be_bytes()
}

/// `subfc rD, rA, rB` (rD = rB − rA, setting CA): opcode 31, XO 8.
/// **Non-commutative — same reversed mapping as [`encode_subf`]**: to realize
/// `lhs − rhs` pass `ra = rhs`, `rb = lhs`.
pub fn encode_subfc(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    xo31(rd, ra, rb, 8)
}

/// `subfe rD, rA, rB` (rD = ¬rA + rB + CA): opcode 31, XO 136.
/// **Non-commutative.** With `rA == rB` the register terms cancel to −1, so the
/// result is `CA − 1` — the don't-care-source idiom (§3.5 of the W6 doc), where
/// the source register number is still byte-visible and must be reproduced.
pub fn encode_subfe(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    xo31(rd, ra, rb, 136)
}

/// `addze rD, rA` (rD = rA + CA): opcode 31, XO 202.
pub fn encode_addze(rd: u8, ra: u8) -> [u8; 4] {
    xo31(rd, ra, 0, 202)
}

/// `neg rD, rA` (rD = −rA): opcode 31, XO 104.
pub fn encode_neg(rd: u8, ra: u8) -> [u8; 4] {
    xo31(rd, ra, 0, 104)
}

/// `andc rA, rS, rB` (rA = rS & ¬rB): opcode 31, XO 60. Not symmetric in
/// rS/rB — the complement applies to rB only.
pub fn encode_andc(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
    xo31(rs, ra, rb, 60)
}

/// `orc rA, rS, rB` (rA = rS | ¬rB): opcode 31, XO 412. Not symmetric.
pub fn encode_orc(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
    xo31(rs, ra, rb, 412)
}

/// `eqv rA, rS, rB` (rA = ¬(rS ^ rB)): opcode 31, XO 284. Logically symmetric,
/// but c2's emitted rS/rB order is reproduced rather than chosen.
pub fn encode_eqv(ra: u8, rs: u8, rb: u8) -> [u8; 4] {
    xo31(rs, ra, rb, 284)
}

/// `cntlzw rA, rS` (count leading zero bits): opcode 31, XO 26. Yields exactly
/// 32 iff rS is zero — the basis of the `== 0` idiom.
pub fn encode_cntlzw(ra: u8, rs: u8) -> [u8; 4] {
    xo31(rs, ra, 0, 26)
}

/// `xori rA, rS, UIMM` (rA = rS ^ UIMM): primary opcode 26.
pub fn encode_xori(ra: u8, rs: u8, ui: u16) -> [u8; 4] {
    let word: u32 =
        (26 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (ui as u32);
    word.to_be_bytes()
}

/// `rlwinm rA, rS, SH, MB, ME` — rotate left word immediate then AND with mask:
/// primary opcode 21, Rc=0. The workhorse of bit extraction here.
pub fn encode_rlwinm(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
    let word: u32 = (21 << 26)
        | ((rs as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | ((sh as u32 & 0x1F) << 11)
        | ((mb as u32 & 0x1F) << 6)
        | ((me as u32 & 0x1F) << 1);
    word.to_be_bytes()
}

/// `srwi rA, rS, 31` — extract the sign bit. The `rlwinm rA,rS,1,31,31` form.
pub fn encode_srwi31(ra: u8, rs: u8) -> [u8; 4] {
    encode_rlwinm(ra, rs, 1, 31, 31)
}

/// `clrlwi rA, rS, 31` — keep only bit 31. The `rlwinm rA,rS,0,31,31` form.
pub fn encode_clrlwi31(ra: u8, rs: u8) -> [u8; 4] {
    encode_rlwinm(ra, rs, 0, 31, 31)
}

// ---- W13a: floating-point leaf encoders ------------------------------------
//
// Single precision is primary opcode 59 (`0xEC…`) and double is 63 (`0xFC…`),
// with *identical* XO and register fields — so one encoder parameterised by
// precision covers both. Verified bit-exact against live captures
// (docs/CODEGEN_W13_FLOAT.md §3).
//
// Two traps that the integer path would walk straight into:
//   * `fmuls` puts the multiplier in the **C** field (bits 6..10), not B.
//   * `fsubs fD,fA,fB` computes `fA − fB` — the **opposite** of [`encode_subf`]'s
//     load-bearing reversal. Reusing the integer convention silently negates
//     every FP subtraction.

/// Primary opcode for the A-form FP ops: 59 single-precision, 63 double.
fn fp_primary(double: bool) -> u32 {
    if double {
        63
    } else {
        59
    }
}

/// A-form FP encode: `<op> fD, fA, fB, fC` with the given XO.
fn fp_a_form(double: bool, fd: u8, fa: u8, fb: u8, fc: u8, xo: u32) -> [u8; 4] {
    let word: u32 = (fp_primary(double) << 26)
        | ((fd as u32 & 0x1F) << 21)
        | ((fa as u32 & 0x1F) << 16)
        | ((fb as u32 & 0x1F) << 11)
        | ((fc as u32 & 0x1F) << 6)
        | (xo << 1);
    word.to_be_bytes()
}

/// `fadds`/`fadd` — XO 21. Commutative.
pub fn encode_fadd(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
    fp_a_form(double, fd, fa, fb, 0, 21)
}

/// `fsubs`/`fsub` — XO 20. **`fD = fA − fB`**, i.e. the operands are in source
/// order, unlike the integer [`encode_subf`]. Swapping them negates the result.
pub fn encode_fsub(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
    fp_a_form(double, fd, fa, fb, 0, 20)
}

/// `fmuls`/`fmul` — XO 25, with the multiplier in the **C** field.
pub fn encode_fmul(double: bool, fd: u8, fa: u8, fc: u8) -> [u8; 4] {
    fp_a_form(double, fd, fa, 0, fc, 25)
}

/// `fdivs`/`fdiv` — XO 18.
pub fn encode_fdiv(double: bool, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
    fp_a_form(double, fd, fa, fb, 0, 18)
}

/// FP scratch pool, in allocation order: `f0` first, then descending from `f13`,
/// wrapping. Deliberately NOT the integer shape — `f0` is allocatable and comes
/// first, and the result register `f1` is last.
const FP_POOL: [u8; 14] = [0, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
/// FP return register.
const FP_RET: u8 = 1;

/// Select `.text` for a **W13a floating-point leaf**: a straight-line chain over
/// float (or double) *parameters* with no constants and no conversions.
///
/// Register model, which differs from the integer one in every particular
/// (docs/CODEGEN_W13_FLOAT.md §2):
/// * parameters occupy `f1…f13` in float-parameter order; the result is `f1`;
/// * temporaries come from a rotating cursor over [`FP_POOL`] — `f0` first, then
///   down from `f13` — skipping registers that still hold a live value;
/// * an FP `+` chain does **not** collapse to a single accumulator the way the
///   integer one does.
///
/// Verified: `float fmul3(float a,float b,float c){return a*b*c;}` selects
/// `fmuls f0,f1,f2 ; fmuls f1,f0,f3 ; blr`.
pub fn float_leaf_text(func: &IlFunction, double: bool) -> Result<Vec<u8>, BackendError> {
    if func.params.len() > 13 {
        return Err(out_of_class(
            "more than 13 FP parameters: the 14th is stack-homed; out of class",
        ));
    }
    // Parameter n → f(n+1).
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .map(|i| (i + 1) as u8)
    };

    // Which ops appear — A5/A7 gating happens in the IL parser, but the mix is
    // re-checked here because a contraction mis-emit is silent.
    let has_mul = func.ops.iter().any(|o| matches!(o, IlOp::Mul));
    let has_addsub = func
        .ops
        .iter()
        .any(|o| matches!(o, IlOp::Add | IlOp::Sub));
    if has_mul && has_addsub {
        return Err(out_of_class(
            "FP expression mixes `*` with `+`/`-`: c2 contracts these to \
             fmadds/fmsubs/fnmsubs, which is not modeled; out of class",
        ));
    }

    // Evaluate the postfix stream over a stack of physical FP registers.
    let n_ops = func
        .ops
        .iter()
        .filter(|o| !matches!(o, IlOp::Load(_) | IlOp::Lit(_)))
        .count();
    let mut emitted = 0usize;
    let mut cursor = 0usize;
    let mut live: Vec<u8> = (1..=func.params.len() as u8).collect();
    let mut stack: Vec<u8> = Vec::new();
    let mut text: Vec<u8> = Vec::new();

    for op in &func.ops {
        match op {
            IlOp::Load(tok) => {
                let r = reg_of(*tok).ok_or_else(|| {
                    out_of_class("FP LOAD of a token that is not a parameter")
                })?;
                stack.push(r);
            }
            IlOp::Lit(_) => {
                return Err(out_of_class(
                    "FP literal needs an .rdata COMDAT, a REFHI/REFLO relocation \
                     pair and a GPR (W13b); out of class",
                ))
            }
            binop => {
                let rhs = stack
                    .pop()
                    .ok_or_else(|| out_of_class("FP binary op: empty stack (rhs)"))?;
                let lhs = stack
                    .pop()
                    .ok_or_else(|| out_of_class("FP binary op: empty stack (lhs)"))?;
                emitted += 1;
                // The final value lands in f1; earlier ones take the next free
                // pool slot, skipping anything still live.
                let dest = if emitted == n_ops {
                    FP_RET
                } else {
                    let mut d = None;
                    for _ in 0..FP_POOL.len() {
                        let cand = FP_POOL[cursor % FP_POOL.len()];
                        cursor += 1;
                        if !live.contains(&cand) {
                            d = Some(cand);
                            break;
                        }
                    }
                    d.ok_or_else(|| {
                        out_of_class("no free FP scratch register (would spill f31/f30)")
                    })?
                };
                // Both sources die here unless they are still-live parameters.
                for s in [lhs, rhs] {
                    if s as usize > func.params.len() || s == 0 {
                        live.retain(|&x| x != s);
                    }
                }
                match binop {
                    IlOp::Add => text.extend_from_slice(&encode_fadd(double, dest, lhs, rhs)),
                    // Source order, NOT the integer reversal.
                    IlOp::Sub => text.extend_from_slice(&encode_fsub(double, dest, lhs, rhs)),
                    IlOp::Mul => text.extend_from_slice(&encode_fmul(double, dest, lhs, rhs)),
                    IlOp::Div => text.extend_from_slice(&encode_fdiv(double, dest, lhs, rhs)),
                    IlOp::Load(_) | IlOp::Lit(_) => unreachable!("not a binary op"),
                }
                if dest != FP_RET {
                    live.push(dest);
                }
                stack.push(dest);
            }
        }
    }
    if stack.len() != 1 {
        return Err(out_of_class(
            "FP expression did not reduce to a single value; out of class",
        ));
    }
    text.extend_from_slice(&encode_blr());
    Ok(text)
}

/// Shared encoder for the opcode-31 X-form used above: the first register field
/// (bits 6..11) is rD for arithmetic forms and rS for logical ones — callers
/// pass them in that slot accordingly.
fn xo31(first: u8, second: u8, rb: u8, xo: u32) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((first as u32 & 0x1F) << 21)
        | ((second as u32 & 0x1F) << 16)
        | ((rb as u32 & 0x1F) << 11)
        | (xo << 1);
    word.to_be_bytes()
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

/// Select `.text` for a **W6 comparison leaf** (`return a <rel> k;`), returning
/// the spine plus its trailing `blr`.
///
/// c2 materializes these branchlessly — it emits no `cmpw`/`cmplw` at all for
/// this shape — using carry-bit and bit-extraction idioms. Every sequence below
/// is transcribed from a live capture (`docs/CODEGEN_W6_COMPARE.md` §3–§4) and
/// each instruction word is re-encoded from its fields here.
///
/// **The `k == 0` folds are dispatched first and are not optional.** c2 does not
/// run a zero literal through the general spine; it folds, sometimes to a
/// shorter sequence and sometimes to a constant. Two of the six fixture
/// functions land in that table, and emitting the general spine for them would
/// be a wrong-length, wrong-bytes mis-emit. This mirrors the `g(a) + 0` identity
/// fold in W4b2-vi: a zero operand changes the *shape*, not just an immediate.
///
/// Temporaries are allocated descending from r11 in emission order, one physical
/// register per temp with no reuse — including two kinds of slot consumed by
/// values that are never read (a `subfe u,v,v` don't-care source, and a
/// `subfc`/`subfic` destination whose only live output is the carry). Those
/// register numbers are byte-visible, so they must be allocated, not elided.
///
/// Outside this leaf class c2's allocator is demonstrably richer (it reuses dead
/// registers, and it schedules — numbering order is not emission order), so this
/// function accepts exactly the characterized shapes and returns
/// `NotImplemented` for the rest.
pub fn compare_leaf_text(cmp: &c2_il::CompareLeaf) -> Result<Vec<u8>, BackendError> {
    use c2_il::Rel;
    let mut t: Vec<u8> = Vec::with_capacity(28);
    let a = RET_REG; // the compared formal is the first argument, r3

    if cmp.k == 0 {
        // ---- mandatory `k == 0` folds (W6 doc §4.6) ----
        match (cmp.rel, cmp.signed) {
            // `a == 0` — same bytes signed and unsigned.
            (Rel::Eq, _) => {
                t.extend_from_slice(&encode_cntlzw(11, a));
                t.extend_from_slice(&encode_rlwinm(RET_REG, 11, 27, 31, 31));
            }
            // `a != 0` — same bytes signed and unsigned. `~(x-1) == -x`, so the
            // register terms cancel and r3 is exactly the carry.
            (Rel::Ne, _) => {
                t.extend_from_slice(&encode_addic(11, a, -1));
                t.extend_from_slice(&encode_subfe(RET_REG, 11, a));
            }
            // signed `a > 0` → (-a) & ~a, sign bit.
            (Rel::Gt, true) => {
                t.extend_from_slice(&encode_neg(11, a));
                t.extend_from_slice(&encode_andc(10, 11, a));
                t.extend_from_slice(&encode_srwi31(RET_REG, 10));
            }
            // unsigned `a > 0` is exactly `a != 0`.
            (Rel::Gt, false) => {
                t.extend_from_slice(&encode_addic(11, a, -1));
                t.extend_from_slice(&encode_subfe(RET_REG, 11, a));
            }
            // signed `a < 0` is just the sign bit.
            (Rel::Lt, true) => t.extend_from_slice(&encode_srwi31(RET_REG, a)),
            // unsigned `a < 0` is constant false.
            (Rel::Lt, false) => t.extend_from_slice(&encode_addi(RET_REG, 0, 0)),
            // signed `a <= 0` → a | ~(-a), sign bit.
            (Rel::Le, true) => {
                t.extend_from_slice(&encode_neg(11, a));
                t.extend_from_slice(&encode_orc(10, a, 11));
                t.extend_from_slice(&encode_srwi31(RET_REG, 10));
            }
            // unsigned `a <= 0` is exactly `a == 0`.
            (Rel::Le, false) => {
                t.extend_from_slice(&encode_cntlzw(11, a));
                t.extend_from_slice(&encode_rlwinm(RET_REG, 11, 27, 31, 31));
            }
            // signed `a >= 0` → !sign.
            (Rel::Ge, true) => {
                t.extend_from_slice(&encode_srwi31(11, a));
                t.extend_from_slice(&encode_xori(RET_REG, 11, 1));
            }
            // unsigned `a >= 0` is constant true.
            (Rel::Ge, false) => t.extend_from_slice(&encode_addi(RET_REG, 0, 1)),
        }
        t.extend_from_slice(&encode_blr());
        return Ok(t);
    }

    // ---- general spines, non-zero literal ----
    let k16 = i16::try_from(cmp.k).map_err(|_| {
        out_of_class(
            "comparison against a wide literal needs lis+ori materialization and \
             the extra temp slot it consumes; not characterized",
        )
    })?;

    match (cmp.rel, cmp.signed) {
        // `a == k` → difference, then "is it zero".
        (Rel::Eq, _) => {
            t.extend_from_slice(&encode_addi(11, a, -k16));
            t.extend_from_slice(&encode_cntlzw(10, 11));
            t.extend_from_slice(&encode_rlwinm(RET_REG, 10, 27, 31, 31));
        }
        // `a != k` → the `!= 0` spine applied to the difference.
        (Rel::Ne, _) => {
            t.extend_from_slice(&encode_addi(11, a, -k16));
            t.extend_from_slice(&encode_addic(10, 11, -1));
            t.extend_from_slice(&encode_subfe(RET_REG, 10, 11));
        }
        // unsigned `a > k`: CA of `k - a` is `a <= k`, so the answer is !CA.
        // `subfe r9,r10,r10` reads r10, which is never defined — the register
        // terms cancel so the value is a don't-care, but the register NUMBER is
        // byte-visible and must be reproduced.
        (Rel::Gt, false) => {
            t.extend_from_slice(&encode_subfic(11, a, k16));
            t.extend_from_slice(&encode_subfe(9, 10, 10));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, 9));
        }
        // signed `a > k`: the 5-instruction spine. p = a (the greater side),
        // q = k. The final clrlwi exists solely to kill the `2` case.
        (Rel::Gt, true) => {
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_subfc(10, a, 11)); // r10 dead; CA is the point
            t.extend_from_slice(&encode_eqv(9, a, 11));
            t.extend_from_slice(&encode_srwi31(8, 9));
            t.extend_from_slice(&encode_addze(7, 8));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, 7));
        }
        // `a < k`, `a >= k`, `a <= k` against a non-zero literal are NOT
        // characterized: `<`/`<=` are the `>`/`>=` spines with the operand roles
        // swapped, and the `>=`/`<=` spine's instruction ORDER for a literal
        // left-hand side is explicitly unresolved in the W6 characterization
        // (c2 schedules: numbering order is not emission order). Guessing the
        // order would be a silent wrong-bytes emit.
        (Rel::Lt, _) | (Rel::Ge, _) | (Rel::Le, _) => {
            return Err(out_of_class(
                "comparison relation against a non-zero literal whose spine \
                 instruction order is not yet pinned (see docs/CODEGEN_W6_COMPARE.md \
                 §4.5); out of class",
            ))
        }
    }
    t.extend_from_slice(&encode_blr());
    Ok(t)
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
    /// A binary op over unresolved operand *bases*; `Sub` keeps its load-bearing
    /// operand order. Bases stay symbolic until emission because `Base::Prev`
    /// resolves to whichever register the previous result was placed in, and
    /// that is no longer always r11 (see [`select_text`]'s allocator).
    Bin { op: IlOp, lhs: Base, rhs: Base },
    /// Materialize a pending offset: `dest = src + k` (`addi`, or `addis`+`addi`
    /// when wide). The final flush of an affine `reg + off` value.
    AddImm { src: Base, k: i32 },
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
/// Try to select a **depth-2 expression tree** `(a op b) root (c op d)` over
/// four distinct parameter leaves (W5 trees).
///
/// The operand stack reaches depth 3 here, so the serial-chain selector cannot
/// express it. c2 lowers it as an actual tree: left child into one scratch,
/// right child into another, then the root into r3.
///
/// ```text
///   (a+b)*(c+d)   add   r11,r3,r4 ; add   r10,r5,r6 ; mullw r3,r11,r10
///   (a*b)-(c*d)   mullw r11,r3,r4 ; mullw r10,r5,r6 ; subf  r3,r10,r11
///   (a*b)+(c*d)   mullw r10,r3,r4 ; mullw r11,r5,r6 ; add   r3,r10,r11
/// ```
///
/// Note the third line: **when the root is `+` the two children's registers are
/// swapped** relative to every other root operator. That is reproducible and
/// order-independent — `(a*b)+(c*d)` and `(c*d)+(a*b)` are byte-identical, so c2
/// canonicalizes the commutative root by parameter order and then gives the
/// first term r10. The mechanism is not understood, only characterized, which is
/// why the `+` root is accepted at *exactly* this depth and nowhere else.
///
/// Four gates, each a shape where c2 does **not** lower the source tree as a
/// tree and a post-order selector would emit plausible, wrong bytes:
///
/// * a `*` node with a `*` child collapses into one n-ary product and is
///   re-linearized into a chain — `(a*b)*(c*d)`, `(a+b)*(c*d)` and `a*(b*(c*d))`
///   all compile to the *same* chain, none of them the source pairing;
/// * a `+`/`-` node with a `+`/`-` child collects into one n-ary sum whose terms
///   are reordered (subtracted first) — `(a+b)-(c+d)` emits its leaves in the
///   order `a, c, d, b`;
/// * any immediate on an additive node (its register order is unexplained);
/// * anything but four distinct parameter leaves.
fn try_select_depth2_tree(
    func: &IlFunction,
    reg_of: &dyn Fn(u32) -> Option<u8>,
) -> Option<Vec<u8>> {
    let (l0, l1, op1, l2, l3, op2, root) = match func.ops.as_slice() {
        [IlOp::Load(a), IlOp::Load(b), o1, IlOp::Load(c), IlOp::Load(d), o2, r]
            if o1.is_tree_binop() && o2.is_tree_binop() && r.is_tree_binop() =>
        {
            (*a, *b, *o1, *c, *d, *o2, *r)
        }
        _ => return None,
    };
    // Four distinct parameter leaves, nothing else.
    let toks = [l0, l1, l2, l3];
    for (i, t) in toks.iter().enumerate() {
        if toks[..i].contains(t) {
            return None;
        }
    }
    let regs: Vec<u8> = toks.iter().map(|t| reg_of(*t)).collect::<Option<_>>()?;

    let is_additive = |o: IlOp| matches!(o, IlOp::Add | IlOp::Sub);
    // N1 — product flattening.
    if root == IlOp::Mul && (op1 == IlOp::Mul || op2 == IlOp::Mul) {
        return None;
    }
    // N2 — additive canonicalization.
    if is_additive(root) && (is_additive(op1) || is_additive(op2)) {
        return None;
    }
    // Integer division is not modeled at all.
    if root == IlOp::Div || op1 == IlOp::Div || op2 == IlOp::Div {
        return None;
    }

    // The `+`-root swap.
    let (left_reg, right_reg) = if root == IlOp::Add {
        (SCRATCH_REG - 1, SCRATCH_REG) // r10, r11
    } else {
        (SCRATCH_REG, SCRATCH_REG - 1) // r11, r10
    };

    let emit = |out: &mut Vec<u8>, op: IlOp, dest: u8, lhs: u8, rhs: u8| match op {
        IlOp::Add => out.extend_from_slice(&encode_add(dest, lhs, rhs)),
        IlOp::Mul => out.extend_from_slice(&encode_mullw(dest, lhs, rhs)),
        // `subf` computes rB − rA, so `lhs − rhs` needs rA=rhs, rB=lhs.
        IlOp::Sub => out.extend_from_slice(&encode_subf(dest, rhs, lhs)),
        _ => unreachable!("gated above"),
    };

    let mut text = Vec::with_capacity(16);
    // Left child first, always — only the register assignment swaps.
    emit(&mut text, op1, left_reg, regs[0], regs[1]);
    emit(&mut text, op2, right_reg, regs[2], regs[3]);
    emit(&mut text, root, RET_REG, left_reg, right_reg);
    text.extend_from_slice(&encode_blr());
    Some(text)
}

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
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .map(|i| ARG_REGS[i])
    };

    // A depth-2 tree is not a serial chain and the affine selector below cannot
    // express it; try the dedicated tree shape first.
    if let Some(text) = try_select_depth2_tree(func, &reg_of) {
        return Ok(text);
    }

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
            // Integer division is not modeled (`divw`/`divwu`, and a constant
            // divisor strength-reduces to a multiply-high). FP division reaches
            // `float_leaf_text` instead and never gets here.
            IlOp::Div => return Err(out_of_class("integer division; out of class")),
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
                plan.push(PlanOp::AddImm { src: *base, k: *off });
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

    // Emit the plan. The **last** entry targets the return register r3. For the
    // earlier ones the destination depends on the op, because c2 does NOT use a
    // single scratch for every chain (verified against live captures):
    //
    //   a+b+c+d  ->  add   r11,r3,r4 ; add   r11,r11,r5 ; add   r3,r11,r6
    //   a*b*c*d  ->  mullw r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
    //   a-b-c-d  ->  subf  r11,r4,r3 ; subf  r10,r5,r11 ; subf  r3,r6,r10
    //
    // An additive chain collapses to one running accumulator (r11 reused), while
    // a `*`/`-` chain gives every intermediate its own register, descending from
    // r11. The two rules coincide at exactly one intermediate — which is why
    // every fixture up to `a-b-c` matched while `a-b-c-d` silently mis-emitted.
    //
    // `Base::Prev` therefore resolves to the previous entry's ACTUAL destination
    // rather than to a fixed r11; that is why plan operands stay symbolic until
    // here.
    let mut text: Vec<u8> = Vec::new();
    let last = plan.len().saturating_sub(1);
    let mut next_scratch: u8 = SCRATCH_REG;
    let mut prev_reg: u8 = SCRATCH_REG;
    for (i, entry) in plan.iter().enumerate() {
        let dest = if i == last {
            RET_REG
        } else {
            match entry {
                // Additive accumulation reuses the accumulator register.
                PlanOp::Bin { op: IlOp::Add, .. } | PlanOp::AddImm { .. } => SCRATCH_REG,
                _ => {
                    let d = next_scratch;
                    // Observed descending allocation covers r11, r10, r9 (the
                    // deepest characterized chain is `a*b*c*d*e`). Below that is
                    // extrapolation, and c2's allocator is demonstrably richer
                    // outside this class — it reuses dead registers and it
                    // schedules — so refuse rather than guess.
                    if d < 9 {
                        return Err(out_of_class(
                            "expression chain needs more scratch registers than the \
                             characterized descending range r11..r9; out of class",
                        ));
                    }
                    next_scratch = d - 1;
                    d
                }
            }
        };
        let resolve = |b: Base| -> u8 {
            match b {
                Base::Phys(r) => r,
                Base::Prev => prev_reg,
            }
        };
        match *entry {
            PlanOp::Bin { op, lhs, rhs } => {
                let (l, r) = (resolve(lhs), resolve(rhs));
                match op {
                    IlOp::Add => text.extend_from_slice(&encode_add(dest, l, r)),
                    IlOp::Mul => text.extend_from_slice(&encode_mullw(dest, l, r)),
                    // `subf` computes rB − rA, so realizing `lhs − rhs` needs
                    // rA=rhs, rB=lhs (the load-bearing reversed order — see
                    // [`encode_subf`]).
                    IlOp::Sub => text.extend_from_slice(&encode_subf(dest, r, l)),
                    // `combine` never records a Div plan entry (it rejects
                    // first), so reaching here would be an internal error.
                    IlOp::Div | IlOp::Load(_) | IlOp::Lit(_) => {
                        unreachable!("not a modeled integer binary op")
                    }
                }
            }
            PlanOp::AddImm { src, k } => emit_add_imm(&mut text, dest, resolve(src), k),
            PlanOp::LoadImm { k } => emit_load_imm(&mut text, dest, k)?,
        }
        prev_reg = dest;
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
        plan.push(PlanOp::Bin { op, lhs: a, rhs: b });
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

        (IlOp::Div, _, _) => Err(out_of_class("integer division; out of class")),
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
            compare: None,
            empty_body: false,
            float_leaf: None,
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

    #[test]
    fn mul_chain_of_three_ops_uses_descending_scratch_registers() {
        // REGRESSION (w5_chain.cpp): `a*b*c*d`. c2 gives every intermediate of a
        // `*` chain its own register; the port used to reuse r11 and silently
        // mis-emitted. Reference `.text` (live capture):
        //   7d6321d6 mullw r11,r3,r4 ; 7d4b29d6 mullw r10,r11,r5
        //   7c6a31d6 mullw r3,r10,r6
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Mul,
                IlOp::Load(0xE609),
                IlOp::Mul,
            ],
        );
        assert_eq!(
            select_text(&f).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x4B, 0x29, 0xD6, // mullw r10,r11,r5
                0x7C, 0x6A, 0x31, 0xD6, // mullw r3,r10,r6
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn sub_chain_of_three_ops_descends_and_keeps_operand_order() {
        // `a-b-c-d`. Descending destinations AND the load-bearing reversed subf
        // operand order at every step. Reference:
        //   7d641850 subf r11,r4,r3 ; 7d455850 subf r10,r5,r11
        //   7c665050 subf r3,r6,r10
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Sub,
                IlOp::Load(0xE509),
                IlOp::Sub,
                IlOp::Load(0xE609),
                IlOp::Sub,
            ],
        );
        assert_eq!(
            select_text(&f).unwrap(),
            vec![
                0x7D, 0x64, 0x18, 0x50, // subf r11,r4,r3
                0x7D, 0x45, 0x58, 0x50, // subf r10,r5,r11
                0x7C, 0x66, 0x50, 0x50, // subf r3,r6,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn add_chain_reuses_one_accumulator_register() {
        // The contrast that makes the rule non-obvious: an ADDITIVE chain
        // collapses to a single accumulator, so `a+b+c+d` keeps r11 throughout
        // where the `*`/`-` chains above descend. Reference:
        //   7d632214 add r11,r3,r4 ; 7d6b2a14 add r11,r11,r5
        //   7c6b3214 add r3,r11,r6
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Add,
                IlOp::Load(0xE609),
                IlOp::Add,
            ],
        );
        assert_eq!(
            select_text(&f).unwrap(),
            vec![
                0x7D, 0x63, 0x22, 0x14, // add r11,r3,r4
                0x7D, 0x6B, 0x2A, 0x14, // add r11,r11,r5
                0x7C, 0x6B, 0x32, 0x14, // add r3,r11,r6
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    // ---- W13a floating-point leaves ----------------------------------------

    fn fpfunc(params: Vec<u32>, ops: Vec<IlOp>) -> IlFunction {
        let mut f = func_with(params, ops);
        f.float_leaf = Some(false);
        f
    }

    #[test]
    fn float_chain_matches_the_reference() {
        // `float fmul3(float a,float b,float c){ return a*b*c; }` — the live
        // capture is `ec0100b2 ec2000f2 4e800020`:
        //   fmuls f0,f1,f2   (first temp is f0 — the pool's FIRST slot)
        //   fmuls f1,f0,f3   (result forced to f1)
        // Note the multiplier sits in the C field, not B.
        let f = fpfunc(
            vec![0xE309, 0xE409, 0xE509],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Mul,
            ],
        );
        assert_eq!(
            float_leaf_text(&f, false).unwrap(),
            vec![
                0xEC, 0x01, 0x00, 0xB2, // fmuls f0,f1,f2
                0xEC, 0x20, 0x00, 0xF2, // fmuls f1,f0,f3
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn fp_subtract_uses_source_order_not_the_integer_reversal() {
        // `fsubs fD,fA,fB` computes fA − fB — the OPPOSITE of encode_subf's
        // load-bearing reversal. Reusing the integer convention here would
        // silently negate every FP subtraction, so pin the operand order.
        assert_eq!(encode_fsub(false, 1, 1, 2), [0xEC, 0x21, 0x10, 0x28]);
        assert_eq!(encode_fadd(false, 1, 1, 2), [0xEC, 0x21, 0x10, 0x2A]);
        assert_eq!(encode_fdiv(false, 1, 1, 2), [0xEC, 0x21, 0x10, 0x24]);
        // Double precision is the same fields under primary opcode 63.
        assert_eq!(encode_fadd(true, 1, 1, 2), [0xFC, 0x21, 0x10, 0x2A]);
    }

    #[test]
    fn fp_rejects_the_shapes_that_would_mis_emit() {
        // A `*` mixed with `+`/`-` CONTRACTS to fmadds/fmsubs in c2, so emitting
        // two instructions would be a silent wrong-bytes emit, not a gap.
        let mixed = fpfunc(
            vec![0xE309, 0xE409, 0xE509],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Add,
            ],
        );
        assert!(matches!(
            float_leaf_text(&mixed, false),
            Err(BackendError::NotImplemented(_))
        ));
        // An FP literal needs an .rdata COMDAT plus a REFHI/REFLO pair (W13b).
        let lit = fpfunc(
            vec![0xE309],
            vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Mul],
        );
        assert!(matches!(
            float_leaf_text(&lit, false),
            Err(BackendError::NotImplemented(_))
        ));
    }

    // ---- W6 comparison spines (bytes from live captures) --------------------

    fn cmp(rel: c2_il::Rel, signed: bool, k: i32) -> Vec<u8> {
        compare_leaf_text(&c2_il::CompareLeaf { param: 0xE309, rel, signed, k }).unwrap()
    }

    #[test]
    fn compare_zero_folds_match_the_reference() {
        use c2_il::Rel;
        // `x != 0` (unsigned) — 2 instructions, the carry trick.
        assert_eq!(
            cmp(Rel::Ne, false, 0),
            vec![0x31, 0x63, 0xFF, 0xFF, 0x7C, 0x6B, 0x19, 0x10, 0x4E, 0x80, 0x00, 0x20]
        );
        // signed `x > 0` — a FOLD, not the general 5-instruction spine.
        assert_eq!(
            cmp(Rel::Gt, true, 0),
            vec![
                0x7D, 0x63, 0x00, 0xD0, // neg r11,r3
                0x7D, 0x6A, 0x18, 0x78, // andc r10,r11,r3
                0x55, 0x43, 0x0F, 0xFE, // srwi r3,r10,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `x < 0` folds to constant false; `x >= 0` to constant true.
        assert_eq!(cmp(Rel::Lt, false, 0)[..4], [0x38, 0x60, 0x00, 0x00]);
        assert_eq!(cmp(Rel::Ge, false, 0)[..4], [0x38, 0x60, 0x00, 0x01]);
    }

    #[test]
    fn compare_general_spines_match_the_reference() {
        use c2_il::Rel;
        // `x == 1` (3 instructions).
        assert_eq!(
            cmp(Rel::Eq, false, 1),
            vec![
                0x39, 0x63, 0xFF, 0xFF, // addi r11,r3,-1
                0x7D, 0x6A, 0x00, 0x34, // cntlzw r10,r11
                0x55, 0x43, 0xDF, 0xFE, // rlwinm r3,r10,27,31,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // `x != 1` (3 instructions).
        assert_eq!(
            cmp(Rel::Ne, false, 1),
            vec![
                0x39, 0x63, 0xFF, 0xFF, // addi r11,r3,-1
                0x31, 0x4B, 0xFF, 0xFF, // addic r10,r11,-1
                0x7C, 0x6A, 0x59, 0x10, // subfe r3,r10,r11
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `x > 7` — note the `subfe r9,r10,r10` don't-care SOURCE r10,
        // which is never defined but is byte-visible and must be reproduced.
        assert_eq!(
            cmp(Rel::Gt, false, 7),
            vec![
                0x21, 0x63, 0x00, 0x07, // subfic r11,r3,7
                0x7D, 0x2A, 0x51, 0x10, // subfe r9,r10,r10
                0x55, 0x23, 0x07, 0xFE, // clrlwi r3,r9,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // signed `x > 7` — the 6-word spine.
        assert_eq!(
            cmp(Rel::Gt, true, 7),
            vec![
                0x39, 0x60, 0x00, 0x07, // li r11,7
                0x7D, 0x43, 0x58, 0x10, // subfc r10,r3,r11 (r10 dead; CA is the point)
                0x7C, 0x69, 0x5A, 0x38, // eqv r9,r3,r11
                0x55, 0x28, 0x0F, 0xFE, // srwi r8,r9,31
                0x7C, 0xE8, 0x01, 0x94, // addze r7,r8
                0x54, 0xE3, 0x07, 0xFE, // clrlwi r3,r7,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn compare_uncharacterized_relations_fail_closed() {
        use c2_il::Rel;
        // `<`, `<=`, `>=` against a NON-zero literal: the spine's instruction
        // order for a literal left-hand side is unresolved (c2 schedules, so
        // numbering order is not emission order). Guessing it would be a silent
        // wrong-bytes emit, so these must refuse.
        for rel in [Rel::Lt, Rel::Le, Rel::Ge] {
            assert!(matches!(
                compare_leaf_text(&c2_il::CompareLeaf { param: 0xE309, rel, signed: true, k: 7 }),
                Err(BackendError::NotImplemented(_))
            ));
        }
        // A wide literal needs lis+ori and the extra temp slot it consumes.
        assert!(matches!(
            compare_leaf_text(&c2_il::CompareLeaf {
                param: 0xE309,
                rel: Rel::Gt,
                signed: false,
                k: 70000,
            }),
            Err(BackendError::NotImplemented(_))
        ));
    }

    fn func_with(params: Vec<u32>, ops: Vec<IlOp>) -> IlFunction {
        IlFunction {
            mangled_name: "?f@@YAHH@Z".into(),
            source_path: None,
            tail_call: None,
            framed_call: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
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

    fn tree4(op1: IlOp, op2: IlOp, root: IlOp) -> IlFunction {
        func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                op1,
                IlOp::Load(0xE509),
                IlOp::Load(0xE609),
                op2,
                root,
            ],
        )
    }

    #[test]
    fn depth2_tree_matches_the_reference() {
        // `(a+b)*(c+d)` — the operand stack reaches depth 3, so this is a tree
        // rather than a serial chain: left child into r11, right into r10, root
        // into r3.
        assert_eq!(
            select_text(&tree4(IlOp::Add, IlOp::Add, IlOp::Mul)).unwrap(),
            vec![
                0x7D, 0x63, 0x22, 0x14, // add   r11,r3,r4
                0x7D, 0x45, 0x32, 0x14, // add   r10,r5,r6
                0x7C, 0x6B, 0x51, 0xD6, // mullw r3,r11,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // `(a*b)-(c*d)` — same register assignment; subf keeps its reversed
        // operand order (rA=rhs, rB=lhs).
        assert_eq!(
            select_text(&tree4(IlOp::Mul, IlOp::Mul, IlOp::Sub)).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x45, 0x31, 0xD6, // mullw r10,r5,r6
                0x7C, 0x6A, 0x58, 0x50, // subf  r3,r10,r11
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn depth2_tree_with_an_add_root_swaps_the_child_registers() {
        // The one exception: a `+` ROOT swaps the two children's registers
        // relative to every other root operator. `(a*b)+(c*d)` puts the left
        // child in r10 and the right in r11 — reproducible and order
        // independent, but not mechanistically understood, which is why the
        // `+` root is accepted at exactly this depth and nowhere else.
        assert_eq!(
            select_text(&tree4(IlOp::Mul, IlOp::Mul, IlOp::Add)).unwrap(),
            vec![
                0x7D, 0x43, 0x21, 0xD6, // mullw r10,r3,r4   <-- swapped
                0x7D, 0x65, 0x31, 0xD6, // mullw r11,r5,r6   <-- swapped
                0x7C, 0x6A, 0x5A, 0x14, // add   r3,r10,r11
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn tree_shapes_c2_does_not_lower_as_trees_fail_closed() {
        // These are tree-shaped SOURCE that c2 re-linearizes, so a post-order
        // selector emits plausible wrong bytes rather than running out of range.
        //
        // N1: a `*` with a `*` child becomes one n-ary product — `(a*b)*(c*d)`,
        //     `(a+b)*(c*d)` and `a*(b*(c*d))` all compile to the SAME chain,
        //     none of them the source's pairing.
        for (op1, op2) in [(IlOp::Mul, IlOp::Mul), (IlOp::Add, IlOp::Mul), (IlOp::Mul, IlOp::Add)]
        {
            assert!(
                matches!(
                    select_text(&tree4(op1, op2, IlOp::Mul)),
                    Err(BackendError::NotImplemented(_))
                ),
                "N1: {op1:?} / {op2:?} under a `*` root must reject"
            );
        }
        // N2: an additive node with an additive child collects into one n-ary
        //     sum whose terms are REORDERED — `(a+b)-(c+d)` emits its leaves in
        //     the order a, c, d, b.
        for root in [IlOp::Add, IlOp::Sub] {
            for (op1, op2) in [(IlOp::Add, IlOp::Add), (IlOp::Sub, IlOp::Mul), (IlOp::Mul, IlOp::Sub)]
            {
                assert!(
                    matches!(
                        select_text(&tree4(op1, op2, root)),
                        Err(BackendError::NotImplemented(_))
                    ),
                    "N2: {op1:?} / {op2:?} under a {root:?} root must reject"
                );
            }
        }
    }

    #[test]
    fn select_text_mul_is_commutative_order() {
        // `a * b * c` → mullw r11,r3,r4 ; mullw r3,r11,r5 ; blr.
        let func = IlFunction {
            mangled_name: "?mul3@@YAHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            framed_call: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
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
            compare: None,
            empty_body: false,
            float_leaf: None,
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

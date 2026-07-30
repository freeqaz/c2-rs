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

/// `lwz rD, D(rA)` — load a 32-bit word: primary opcode 32.
///
/// The constants are transcribed from raw captures rather than derived:
/// `int f(int* p){return *p;}` is `80630000`, `int f(int a,int* p){return *p;}`
/// is `80640000`, `s->d` (offset 16) is `80630010`, `p[-1]` is `8063fffc` and
/// `p[8000]` is `80637d00`. See `docs/IL_EXPR_LAYER.md` §3.
pub fn encode_lwz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (32 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `lbz rD, D(rA)` — load a zero-extended byte: primary opcode 34. Transcribed
/// from captures: `char f(char* p){return *p;}` is `88630000`, `s->c` at offset 4
/// is `88630004`, and the r11 target an `extsb` consumes is `89630000`.
pub fn encode_lbz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (34 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `lhz rD, D(rA)` — load a zero-extended halfword: primary opcode 40.
/// Captured: `short f(short* p){return *p;}` is `a0630000` (**never `lha`** —
/// see [`indirect_load_text`]), `s->h` at offset 6 is `a0630006`.
pub fn encode_lhz(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (40 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `ld rD, DS(rA)` — load a doubleword: primary opcode 58, **DS-form**. The low
/// two bits of the 16-bit field are the form selector (0 for `ld`), so the
/// displacement is only representable when it is a multiple of 4; callers gate on
/// that rather than letting it round. Captured: `long long f(long long* p)` is
/// `e8630000`, `s->q` at offset 16 is `e8630010`.
pub fn encode_ld(rd: u8, ra: u8, ds: i16) -> [u8; 4] {
    let word: u32 = (58 << 26)
        | ((rd as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | ((ds as u16 as u32) & 0xFFFC);
    word.to_be_bytes()
}

/// `extsb rA, rS` — sign-extend byte: opcode 31, XO 954. Captured as
/// `7d630774` = `extsb r3,r11` (the r11-then-r3 rule; see
/// [`indirect_load_text`]).
pub fn encode_extsb(ra: u8, rs: u8) -> [u8; 4] {
    xo31(rs, ra, 0, 954)
}

/// `extsh rA, rS` — sign-extend halfword: opcode 31, XO 922. Captured as
/// `7d630734` = `extsh r3,r11`. Emitted by no shape the port accepts today: the
/// one construct that produces it (`int f(short* p){return *p;}` under `/Ox`) is
/// refused because the same source is one `lha` under `/O1`, and this path has no
/// mode parameter. Kept, with its pinning test, because the *encoder* is measured
/// and the missing piece is the mode plumbing, not the word.
pub fn encode_extsh(ra: u8, rs: u8) -> [u8; 4] {
    xo31(rs, ra, 0, 922)
}

/// Lower an **indirect-load leaf** — `return *p;` / `return s->m;` /
/// `return p[k];` / `return mMember;` — to one load + `blr`.
///
/// Recognized by an **exact** two-op stream `[Load(base), LoadInd { off }]` or
/// `[Load(base), LoadIndSized { … }]`, which `c2_il::try_parse_indirect_load_leaf`
/// is the only producer of. Returns `None` for anything else so the ordinary
/// selector keeps its behaviour unchanged; the pattern is deliberately not a prefix
/// match, because c2 does *not* lower a load that feeds arithmetic this way —
/// `*p + 1` puts the loaded value in the scratch register
/// (`lwz r11,0(r3) ; addi r3,r11,1`, and for a `char*`
/// `lbz r11 ; extsb r11,r11 ; addi r3,r11,1`) and `*p * 3` is strength-reduced.
///
/// The measured lowering table (`/Ox /GS-` and the workload's `/O1`, identical
/// unless noted; `docs/IL_LOAD_TYPES.md` §3 plus this project's own re-capture):
///
/// ```text
///   T f(T*)                          int f(T*)   (an IL `2C … 00` to int)
///   char/schar   lbz r3              lbz r11 ; extsb r3,r11   <- the r11 rule
///   uchar/bool   lbz r3              lbz r3      (the widening is free)
///   short        lhz r3, NEVER lha    /O1: lha r3   /Ox,/O2: lhz r11 ; extsh r3,r11
///   ushort/wchar lhz r3              lhz r3
///   int/unsigned lwz r3              lwz r3
///   long long    ld r3 (DS-form)     — not captured
/// ```
///
/// The signed-halfword widening is the one row this function cannot emit: it is the
/// only shape in the table whose *instruction count* depends on the optimization
/// mode, and this path takes no mode. The parser refuses it, so it never arrives
/// here; the `Err` below is the second lock, not the primary one.
///
/// `func.params` maps the base token to its incoming argument register by
/// declaration order, with a member function's `this` already at index 0.
pub fn indirect_load_text(func: &IlFunction) -> Option<Result<Vec<u8>, BackendError>> {
    let (base_tok, off, width, sext) = match func.ops.as_slice() {
        [IlOp::Load(t), IlOp::LoadInd { off }] => (*t, *off, 4u8, false),
        [IlOp::Load(t), IlOp::LoadIndSized { off, width, sext }] => (*t, *off, *width, *sext),
        _ => return None,
    };
    let d = match i16::try_from(off) {
        Ok(d) => d,
        // The parser gates this; if it ever changed, refuse rather than truncate.
        Err(_) => return Some(Err(out_of_class("indirect load offset exceeds a 16-bit displacement"))),
    };
    let base = match func.params.iter().position(|&t| t == base_tok) {
        Some(i) if i < ARG_REGS.len() => ARG_REGS[i],
        _ => {
            return Some(Err(out_of_class(
                "indirect load whose base is not a register argument",
            )))
        }
    };
    let mut text = Vec::with_capacity(12);
    // A load that feeds a sign-extension targets r11 and the `exts*` produces r3;
    // an unextended load targets r3 directly.
    let dest = if sext { SCRATCH_REG } else { RET_REG };
    match (width, sext) {
        (1, _) => text.extend_from_slice(&encode_lbz(dest, base, d)),
        (2, false) => text.extend_from_slice(&encode_lhz(dest, base, d)),
        (4, false) => text.extend_from_slice(&encode_lwz(dest, base, d)),
        (8, false) if d % 4 == 0 => text.extend_from_slice(&encode_ld(dest, base, d)),
        (8, false) => {
            return Some(Err(out_of_class(
                "8-byte indirect load whose offset is not a multiple of 4 (ld is DS-form)",
            )))
        }
        // Only `width == 1` is ever sign-extended here (see `IlOp::LoadIndSized`).
        _ => {
            return Some(Err(out_of_class(
                "indirect load of an unmodeled width/extension combination",
            )))
        }
    }
    if sext {
        text.extend_from_slice(&encode_extsb(RET_REG, dest));
    }
    text.extend_from_slice(&encode_blr());
    Some(Ok(text))
}

/// Lower an **address leaf** — `return &s->m;` / `return &p->Base::m;` /
/// `return s->arr;` / `return &p->t[2];` — to one `addi` + `blr`, or to a bare
/// `blr` when the offset is zero.
///
/// Recognized by an **exact** two-op stream `[Load(base), AddrOf { off }]`, which
/// `c2_il::try_parse_addr_leaf` is the only producer of. Returns `None` for
/// anything else so the ordinary selector keeps its behaviour unchanged, and the
/// pattern is deliberately not a prefix match: an address that feeds arithmetic
/// is a construct with no capture behind it.
///
/// The measured lowering (`work/bma/probes/p1.cpp`, `p2.cpp`, `p3.cpp`, every
/// word read off the reference obj at `/Ox /GS- /c`):
///
/// ```text
///   int* f(S* s){ return &s->b; }         38630004  addi r3,r3,4
///   int* f(int x, S* s){ return &s->b; }  38640004  addi r3,r4,4
///   int* D::pb1(){ return &b1; }          3863000c  addi r3,r3,12   (2117, 8+4)
///   int* f(S* s){ return &s->a; }         —                          (off 0)
/// ```
///
/// The zero-offset case emits **nothing at all**, and is only reachable with the
/// base in r3: from any other argument register c2 emits `mr r3,rN`, and the
/// parser refuses that rather than have this function guess. The `Err` below is
/// the second lock on it, not the primary one.
///
/// `func.params` maps the base token to its incoming argument register by
/// declaration order, with a member function's `this` already at index 0.
pub fn addr_leaf_text(func: &IlFunction) -> Option<Result<Vec<u8>, BackendError>> {
    let (base_tok, off) = match func.ops.as_slice() {
        [IlOp::Load(t), IlOp::AddrOf { off }] => (*t, *off),
        _ => return None,
    };
    let d = match i16::try_from(off) {
        Ok(d) => d,
        // The parser gates this; if it ever changed, refuse rather than truncate —
        // a displacement over 32767 is `addis` + `addi`, two instructions.
        Err(_) => {
            return Some(Err(out_of_class(
                "sub-object address offset exceeds a 16-bit displacement",
            )))
        }
    };
    let base = match func.params.iter().position(|&t| t == base_tok) {
        Some(i) if i < ARG_REGS.len() => ARG_REGS[i],
        _ => {
            return Some(Err(out_of_class(
                "sub-object address whose base is not a register argument",
            )))
        }
    };
    let mut text = Vec::with_capacity(8);
    if d != 0 {
        text.extend_from_slice(&encode_addi(RET_REG, base, d));
    } else if base != RET_REG {
        // A zero-offset address from a non-first argument is the same one
        // register move `select_text` makes for `return b;` — `int* f(int k,
        // S* s){ return &s->a; }` is `mr r3,r4`, measured, the same word as the
        // pointer identity beside it. Two spellings, one instruction.
        text.extend_from_slice(&encode_mr(RET_REG, base));
    }
    text.extend_from_slice(&encode_blr());
    Some(Ok(text))
}

/// `stw rS, D(rA)` — store a 32-bit word: primary opcode 36.
///
/// Transcribed from captures (`work/lf/probes/p1.cpp`), not derived:
/// `void f(S* s,int v){ s->a = v; }` is `90830000`, `s->b` (offset 4) is
/// `90830004`, `s->arr[2]` (offset 48) is `90830030`, and
/// `void f(int x,S* s,int v){ s->b = v; }` is `90a40004` — value r5, base r4.
pub fn encode_stw(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (36 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `stb rS, D(rA)` — store a byte: primary opcode 38. Captured: a `char` member
/// at offset 12 is `9883000c`, an `unsigned char` at 16 is `98830010`, a `bool`
/// at 56 is `98830038`, and the literal form's `stb r11` is `99630000`.
pub fn encode_stb(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (38 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `sth rS, D(rA)` — store a halfword: primary opcode 44. Captured: a `short`
/// member at offset 14 is `b083000e`.
pub fn encode_sth(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (44 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

// `encode_std` is deliberately NOT defined beside its three siblings: the frame
// model added the byte-identical encoder for the callee-saved GPR prologue
// (captured as `fbe1fff0` = `std r31,-16(r1)`), and one function with two
// independent captures beats two functions with one each. This rung's own
// witness for it — a `long long` member at offset 32, `f8830020` — is in
// `store_leaf_text`'s table below.

/// Lower a **store leaf** — `void f(S* s, int v){ s->m = v; }` /
/// `void D::set(int v){ Base::m = v; }` / `void f(S* s){ s->m = 7; }` — to one
/// store instruction + `blr`, or to `li` + store + `blr` when the value is a
/// literal.
///
/// Recognized by an **exact** three-op stream `[Load(base), Load(value) | Lit(k),
/// StoreInd { off, width }]`, which `c2_il::try_parse_store_leaf` is the only
/// producer of. Returns `None` for anything else so the ordinary selector keeps
/// its behaviour unchanged, and the pattern is deliberately not a prefix match:
/// a store whose value is *computed* puts the computation in the scratch
/// register first (`s->m = a + b` is `add r11,r3,r4 ; stw r11,0(r3)`), which is
/// a different shape with no capture behind it here.
///
/// The measured lowering (`work/lf/probes/p1.cpp`, `p3.cpp`, every word read off
/// the reference obj at `/Ox /GS- /c`):
///
/// ```text
///   width 1  stb    s->c = v   (char, off 12)      9883000c
///   width 2  sth    s->s = v   (short, off 14)     b083000e
///   width 4  stw    s->a = v   (int, off 0)        90830000
///   width 8  std    s->q = v   (long long, off 32) f8830020   DS-form
///   literal         s->a = 7                       39600007 91630000   li r11,7 ; stw r11
///   literal         s->f = true  (bool)            39600001 99630000   li r11,1 ; stb r11
///   two regs        f(int x,S* s,int v){s->b=v;}   90a40004            stw r5,4(r4)
/// ```
///
/// **The literal goes through the scratch register r11, never r3.** That is the
/// same r11 rule [`indirect_load_text`] follows for a load feeding an extension,
/// and it is read off the capture rather than assumed — a `void` function's r3
/// holds nothing the ABI cares about, so `li r3,7` would have been just as
/// plausible and is not what c2 emits.
///
/// `func.params` maps both tokens to their incoming argument registers by
/// declaration order, with a member function's `this` already at index 0.
pub fn store_leaf_text(func: &IlFunction) -> Option<Result<Vec<u8>, BackendError>> {
    // The **floating-point** store, `void f(S* s, float v){ s->f = v; }` — one
    // `stfs`/`stfd` + `blr`. Two ops rather than three: the value's register is
    // already resolved, because the FP argument file is numbered over the FP
    // parameters alone and only the IL layer has the `.sy` view that says which
    // parameters those are ([`c2_il::IlOp::StoreIndFp`]). The base is the ordinary
    // GPR argument, and its index *is* its register number even with FP formals in
    // the list — an FP parameter fills no GPR but still consumes its slot, so the
    // two effects cancel exactly (`docs/ABI_EDGES.md` §2, and the capture
    // `void s_arg2(int x, S* s, float v){ s->f = v; }` → `stfs f1,4(r4)`).
    if let [IlOp::Load(b), IlOp::StoreIndFp { off, double, src }] = func.ops.as_slice() {
        let d = match i16::try_from(*off) {
            Ok(d) => d,
            Err(_) => {
                return Some(Err(out_of_class(
                    "FP store offset exceeds a 16-bit displacement",
                )))
            }
        };
        let Some(base) = func
            .params
            .iter()
            .position(|&t| t == *b)
            .filter(|&i| i < ARG_REGS.len())
            .map(|i| ARG_REGS[i])
        else {
            return Some(Err(out_of_class(
                "FP store whose base is not a register argument",
            )));
        };
        let mut text = Vec::with_capacity(8);
        text.extend_from_slice(&encode_stfs(*double, *src, base, d));
        text.extend_from_slice(&encode_blr());
        return Some(Ok(text));
    }
    let (base_tok, value, off, width) = match func.ops.as_slice() {
        [IlOp::Load(b), v @ (IlOp::Load(_) | IlOp::Lit(_)), IlOp::StoreInd { off, width }] => {
            (*b, v, *off, *width)
        }
        _ => return None,
    };
    let d = match i16::try_from(off) {
        Ok(d) => d,
        // The parser gates this; if it ever changed, refuse rather than truncate.
        Err(_) => {
            return Some(Err(out_of_class(
                "store offset exceeds a 16-bit displacement",
            )))
        }
    };
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .filter(|&i| i < ARG_REGS.len())
            .map(|i| ARG_REGS[i])
    };
    let Some(base) = reg_of(base_tok) else {
        return Some(Err(out_of_class(
            "store whose base is not a register argument",
        )));
    };
    let mut text = Vec::with_capacity(12);
    let src = match value {
        IlOp::Load(t) => match reg_of(*t) {
            Some(r) => r,
            None => {
                return Some(Err(out_of_class(
                    "store whose value is not a register argument",
                )))
            }
        },
        IlOp::Lit(k) => {
            if let Err(e) = emit_load_imm(&mut text, SCRATCH_REG, *k) {
                return Some(Err(e));
            }
            SCRATCH_REG
        }
        _ => return None,
    };
    match width {
        1 => text.extend_from_slice(&encode_stb(src, base, d)),
        2 => text.extend_from_slice(&encode_sth(src, base, d)),
        4 => text.extend_from_slice(&encode_stw(src, base, d)),
        8 if d % 4 == 0 => text.extend_from_slice(&encode_std(src, base, d)),
        8 => {
            return Some(Err(out_of_class(
                "8-byte store whose offset is not a multiple of 4 (std is DS-form)",
            )))
        }
        _ => return Some(Err(out_of_class("store of an unmodeled width"))),
    }
    text.extend_from_slice(&encode_blr());
    Some(Ok(text))
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

/// `adde rD, rA, rB` (rD = rA + rB + CA): opcode 31, XO 138. The two-sided
/// counterpart of [`encode_addze`], used by the signed `>=`/`<=` spines to add
/// the two sign terms and the borrow in one instruction.
pub fn encode_adde(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    xo31(rd, ra, rb, 138)
}

/// `subfze rD, rA` (rD = ~rA + CA): opcode 31, XO 200. Against a preloaded
/// `rA = -1` this is exactly "materialize CA", which is how the unsigned
/// `>=`/`<=` spines turn a borrow into a 0/1 boolean.
pub fn encode_subfze(rd: u8, ra: u8) -> [u8; 4] {
    xo31(rd, ra, 0, 200)
}

/// `srawi rA, rS, SH` (arithmetic shift right immediate, setting CA): opcode 31,
/// XO 824. At `SH = 31` this broadcasts the sign bit, giving 0 or −1 — the
/// signed relational spines' "sign of the operand" term. Note this is *not*
/// [`encode_srwi31`], which yields 0 or 1 via `rlwinm`; the signed `>=`/`<=`
/// spines use one of each and the pair is not interchangeable.
pub fn encode_srawi(ra: u8, rs: u8, sh: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((rs as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | ((sh as u32 & 0x1F) << 11)
        | (824 << 1);
    word.to_be_bytes()
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

/// `stfs fS, d(rA)` / `stfd fS, d(rA)` — store a floating-point register.
///
/// Primary **52** single, **54** double, both plain D-form. Note the asymmetry
/// with the integer family: `std` is DS-form and cannot encode a displacement
/// that is not a multiple of 4, while `stfd` owns all sixteen bits — so the
/// alignment gate `try_parse_store_leaf` applies to a `width == 8` integer store
/// deliberately has no counterpart on the FP path. Verified: `d0230004` is
/// `stfs f1,4(r3)` and `d8230008` is `stfd f1,8(r3)`
/// (`docs/CODEGEN_FP_ARGS.md` §3).
pub fn encode_stfs(double: bool, fs: u8, ra: u8, d: i16) -> [u8; 4] {
    let primary: u32 = if double { 54 } else { 52 };
    let word: u32 = (primary << 26)
        | ((fs as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | (d as u16 as u32);
    word.to_be_bytes()
}

/// `fmr fD, fB` — the FP register move: X-form, primary **63**, XO 72.
///
/// **Primary 63 whatever the operand width.** There is no `fmrs`: the
/// single-precision A-form ops use primary 59, but a register move is a bit copy
/// and the FPRs hold double internally, so the same encoding serves `float` and
/// `double`. Captured both ways — `float t2(float a,float b){ return g1f(b); }`
/// and its `double` twin both emit `fc201090`, `fmr f1,f2`
/// (`docs/CODEGEN_FP_ARGS.md` §1) — which is why this takes no `double` flag and
/// the A-form encoders above do.
pub fn encode_fmr(fd: u8, fb: u8) -> [u8; 4] {
    let word: u32 =
        (63u32 << 26) | ((fd as u32 & 0x1F) << 21) | ((fb as u32 & 0x1F) << 11) | (72u32 << 1);
    word.to_be_bytes()
}

/// `frsp fD, fB` — round to single precision: X-form, primary 63, XO 12.
///
/// The `double` → `float` narrowing, and it is a **real instruction** where the
/// widening `float` → `double` is nothing at all. Captured as the pair, which is
/// the only way to establish that the asymmetry is c2's and not the C standard's:
/// `double wid(float a){ return gd1(a); }` is a bare `b`, while
/// `float nar(double a){ return gf1(a); }` is `fc200818 ; b` —
/// `frsp f1,f1` (`docs/CODEGEN_FP_ARGS.md` §2).
pub fn encode_frsp(fd: u8, fb: u8) -> [u8; 4] {
    let word: u32 =
        (63u32 << 26) | ((fd as u32 & 0x1F) << 21) | ((fb as u32 & 0x1F) << 11) | (12u32 << 1);
    word.to_be_bytes()
}

/// FP scratch pool, in allocation order: `f0` first, then descending from `f13`,
/// wrapping. Deliberately NOT the integer shape — `f0` is allocatable and comes
/// first, and the result register `f1` is last.
const FP_POOL: [u8; 14] = [0, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
/// FP return register.
const FP_RET: u8 = 1;

/// `lfs fD, d(rA)` — load float single: primary opcode 48. The `lfd` (double)
/// form is primary 50. Both are D-form with a signed 16-bit displacement, which
/// the REFLO relocation rewrites, so `d` is emitted as 0.
pub fn encode_lfs(double: bool, fd: u8, ra: u8, d: i16) -> [u8; 4] {
    let primary: u32 = if double { 50 } else { 48 };
    let word: u32 = (primary << 26)
        | ((fd as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | (d as u16 as u32);
    word.to_be_bytes()
}

/// A **floating-point constant reference site** produced by [`float_leaf_text`].
///
/// c2 never materializes an FP constant with immediates — there is no FP
/// equivalent of `li`. It pools the value into its own `.rdata` COMDAT and loads
/// it through a two-instruction high/low address pair:
///
/// ```text
/// addis r11,r0,0     <- REFHI(__real@…) + PAIR
/// lfs   f0,0(r11)    <- REFLO(__real@…) + PAIR
/// ```
///
/// Both immediates are emitted as 0; the linker patches them. `hi_off` is the
/// `addis` byte offset **relative to the start of this function's text** — the
/// caller rebases it by the function's `.text` offset. The `lfs`/`lfd` always
/// immediately follows, so the REFLO site is `hi_off + 4`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FpConstRef {
    /// The constant's value as raw IEEE-754 **binary64** bits (as the IL carries
    /// it), regardless of the reference width.
    pub bits: u64,
    /// True for a `double` (8-byte `.rdata`, `lfd`); false for `float`.
    pub double: bool,
    /// Byte offset of the `addis` within this function's text.
    pub hi_off: u32,
}

/// Select `.text` for a **W13a/W13b floating-point leaf**: a straight-line chain
/// over float (or double) *parameters* and pooled constants, with no conversions.
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
///
/// Returns the text plus one [`FpConstRef`] per constant **reference site**, in
/// emission order; the caller pools them into `.rdata` COMDATs and turns each
/// into a REFHI/PAIR/REFLO/PAIR relocation quad.
pub fn float_leaf_text(
    func: &IlFunction,
    double: bool,
) -> Result<(Vec<u8>, Vec<FpConstRef>), BackendError> {
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
        .filter(|o| !matches!(o, IlOp::Load(_) | IlOp::Lit(_) | IlOp::FpLit { .. }))
        .count();
    let mut emitted = 0usize;
    let mut cursor = 0usize;
    let mut live: Vec<u8> = (1..=func.params.len() as u8).collect();
    let mut stack: Vec<u8> = Vec::new();
    let mut text: Vec<u8> = Vec::new();
    let mut consts: Vec<FpConstRef> = Vec::new();
    // Address GPRs for constant loads come off the integer scratch cursor,
    // descending from r11 exactly as the integer selector's do.
    let mut next_addr_gpr: u8 = SCRATCH_REG;

    // Pull the next free FP register off the rotating pool cursor.
    let take_fp = |cursor: &mut usize, live: &[u8]| -> Result<u8, BackendError> {
        for _ in 0..FP_POOL.len() {
            let cand = FP_POOL[*cursor % FP_POOL.len()];
            *cursor += 1;
            if !live.contains(&cand) {
                return Ok(cand);
            }
        }
        Err(out_of_class(
            "no free FP scratch register (would spill f31/f30)",
        ))
    };

    // W13b gate. With **one** pooled constant the address setup and the load sit
    // adjacently, immediately before the use — verified byte-exact on six
    // distinct bodies (`w13b_fconst`, and `ka`/`kb`/`kc`/`kd`/`ke`/`kdiv` in
    // `w13b_fpool`). With two, c2 stops doing that: it hoists *every* `addis`
    // into the function prologue as a group, then schedules each `lfs` at its
    // first use and recycles the FP register once a constant dies. See the `p1`
    // and `p5` captures in `docs/CODEGEN_W13_FLOAT.md` §5.3 — `p5` loads its
    // second constant back into `f0`. That scheduler is not modeled, and the
    // REFLO site stops being `hi_off + 4`, so refuse rather than mis-emit.
    let n_consts = func
        .ops
        .iter()
        .filter(|o| matches!(o, IlOp::FpLit { .. }))
        .count();
    if n_consts > 1 {
        return Err(out_of_class(
            "more than one pooled FP constant in one body: c2 hoists the `addis` \
             address setup into the prologue and schedules the loads at first \
             use; that scheduler is not modeled; out of class",
        ));
    }
    // A constant divisor does not survive as a division: **c2** — not c1xx —
    // strength-reduces it to a reciprocal multiply (`a/3.0f/7.0f` reaches the
    // backend with both literals and leaves it having pooled `__real@3d430c31`,
    // i.e. 1/21, and emitted one `fmuls`). That is the whole reason this gate
    // exists: the IL still holds the division, so seeing one here is expected,
    // and lowering it as `fdivs` would be the mis-emit.
    if n_consts > 0 && func.ops.iter().any(|o| matches!(o, IlOp::Div)) {
        return Err(out_of_class(
            "FP division involving a pooled constant: c2 strength-reduces a \
             constant divisor to a reciprocal multiply; out of class",
        ));
    }
    // Constants claim their FP register **before** any interior temporary does,
    // in IL order. Verified by `ke` (`a*2.0f*b*3.0f`, folded to `(a*b)*6.0f`):
    // c2 emits `fmuls f13,f1,f2` and puts the constant in `f0`, so the constant
    // took pool slot 0 even though the multiply is emitted first.
    let mut const_fp: Vec<u8> = Vec::new();
    for _ in 0..n_consts {
        let r = take_fp(&mut cursor, &live)?;
        live.push(r);
        const_fp.push(r);
    }
    let mut next_const = 0usize;

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
                    "integer literal in an FP expression implies a conversion; \
                     out of class",
                ))
            }
            // W13b: a pooled constant. `addis rA,r0,0` + `lfs/lfd fD,0(rA)`,
            // with both immediates left 0 for the REFHI/REFLO relocations.
            IlOp::FpLit { bits, double: lit_double } => {
                if *lit_double != double {
                    return Err(out_of_class(
                        "FP constant width differs from the expression width \
                         (implies a conversion); out of class",
                    ));
                }
                // A `float` constant must survive the binary64 → binary32
                // narrowing exactly, or the pooled 4 bytes would not be the
                // value c2 pooled.
                if !double {
                    let v = f64::from_bits(*bits);
                    if f64::from(v as f32).to_bits() != *bits {
                        return Err(out_of_class(
                            "float constant is not exactly representable in \
                             binary32; out of class",
                        ));
                    }
                }
                let gpr = next_addr_gpr;
                if gpr < 9 {
                    return Err(out_of_class(
                        "FP constant pool needs more address registers than the \
                         characterized descending range r11..r9; out of class",
                    ));
                }
                next_addr_gpr = gpr - 1;
                // Pre-assigned above; `live` already reflects it.
                let fd = const_fp[next_const];
                next_const += 1;
                consts.push(FpConstRef {
                    bits: *bits,
                    double,
                    hi_off: text.len() as u32,
                });
                text.extend_from_slice(&encode_addis(gpr, 0, 0));
                text.extend_from_slice(&encode_lfs(double, fd, gpr, 0));
                stack.push(fd);
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
                    take_fp(&mut cursor, &live)?
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
                    IlOp::Load(_)
                    | IlOp::Lit(_)
                    | IlOp::FpLit { .. }
                    | IlOp::LoadInd { .. }
                    | IlOp::LoadIndSized { .. }
                    | IlOp::AddrOf { .. }
                    | IlOp::StoreInd { .. }
                    | IlOp::StoreIndFp { .. } => {
                        unreachable!("not a binary op")
                    }
                }
                if dest != FP_RET {
                    live.push(dest);
                }
                stack.push(dest);
            }
        }
    }
    match stack.as_slice() {
        // Every binary op targets `FP_RET` when it is the last one, so a value
        // sitting anywhere else means the body is a bare `return <param>` whose
        // parameter is not the first — `float f(float a, float b){ return b; }`,
        // which c2 emits as `fmr f1,f2`. Emitting nothing there is wrong bytes,
        // and it *was*: this branch is the second lock on it, matching the one
        // [`select_text`] has carried for the integer identity since that class
        // was written. The parser refuses the shape first (`try_parse_float_leaf`
        // requires every formal to be an FP operand of the body), so nothing
        // should reach here.
        // A bare `return <FP parameter>` whose parameter is not the first FP one:
        // one `fmr` into the result register. `float f(float a, float b)
        // { return b; }` is `fmr f1,f2 ; blr` (captured, `fc201090`), and this
        // branch used to emit **nothing** at all — `GAPS.md` §6's seventh live
        // wrong-bytes emit, the integer identity's `straight_line_out_of_class_ctx`
        // gate missing from the other register file.
        //
        // Reachable only through the parameter list this shape now carries in
        // FP-register order; nothing else can leave a value outside `FP_RET`,
        // because every binary op targets it when it is the last one.
        [r] if *r != FP_RET => {
            text.extend_from_slice(&encode_fmr(FP_RET, *r));
        }
        [_] => {}
        _ => {
            return Err(out_of_class(
                "FP expression did not reduce to a single value; out of class",
            ))
        }
    }
    text.extend_from_slice(&encode_blr());
    Ok((text, consts))
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

// ---------------------------------------------------------------------------
// The X360 stack frame — a model, not a constant
// ---------------------------------------------------------------------------

/// Fixed head of every MSVC X360 stack frame, in bytes: 16 bytes of linkage
/// (the back chain at `0(r1)` and one reserved doubleword) plus a 64-byte
/// outgoing-parameter home area — 8 slots, the ABI floor. Measured: every local
/// this project has captured is addressed at `80(r1)` or above, and the frame of
/// a body with no locals and no saved registers is `align16(80 + 8) = 96`.
///
/// It is a *floor on the parameter area*, not a floor on the frame: a function
/// whose widest call passes more than eight arguments pushes the locals up
/// (`FrameLayout::locals_base`).
pub const FRAME_HEAD: u32 = 80;

/// The ABI floor on the outgoing-parameter home area, in 8-byte slots.
pub const FRAME_MIN_OUT_SLOTS: u32 = 8;

/// The largest `saved_gprs + saved_fprs` for which the frame-size rule is exact.
/// Past it the allocator spills to slots the rule does not model and the frame
/// grows by an unmeasured amount (39 of 480 designed compiles, all at
/// `nSaved ≥ 18` — `docs/CODEGEN_FRAMED_CALLS.md` §1.3). Refused, not guessed.
pub const FRAME_MAX_SAVED_NO_SPILL: u8 = 17;

/// The page the prologue's stack probes step by, and the unit of the
/// `_RtlCheckStack12` threshold. Measured: the probes are `ld r12,-4096(r1)`,
/// `ld r12,-8192(r1)`, … .
pub const FRAME_PAGE: u32 = 4096;

/// `stw r12,-8(r1)` — spill the just-`mflr`'d link register into the caller's
/// frame. The LR slot is the topmost doubleword of *this* function's frame
/// (`F-8(r1)` after the `stwu`), which is why it is written before the frame is
/// allocated and read back after it is freed.
const FRAME_LR_STORE: u32 = 0x9181_FFF8;
/// `lwz r12,-8(r1)` — the matching reload.
const FRAME_LR_LOAD: u32 = 0x8181_FFF8;
/// `mflr r12` (`mfspr r12,8`).
const FRAME_MFLR_R12: u32 = 0x7D88_02A6;
/// `mtlr r12` (`mtspr 8,r12`).
const FRAME_MTLR_R12: u32 = 0x7D88_03A6;
/// `stwux r1,r1,r12` — opcode 31, XO 183. The variable-size frame allocation
/// c2 emits immediately after `bl _RtlCheckStack12`, which takes `−F` in r12.
/// Captured, and pinned by a test, for a shape [`FrameLayout`] refuses: keeping
/// the measured word beside the threshold that gates it is what stops the next
/// implementer from guessing it.
pub const FRAME_STWUX: u32 = 0x7C21_616E;
/// `lwz r1,0(r1)` — deallocate through the back chain, used when `+F` does not
/// fit an `addi` immediate.
const FRAME_BACKCHAIN: u32 = 0x8021_0000;

/// The **measured X360 frame layout** of one function: how much local/spill
/// space it needs above the fixed head, and how many callee-saved GPRs and FPRs
/// it keeps live across its calls.
///
/// Every rule below was read out of reference objs compiled by the real
/// toolchain at `/Ox /GS- /c`; the probe sources are one-liners of the form
/// `int g(…); T f(…){ … g(…) … }` and the byte evidence is in
/// `docs/CODEGEN_PPC_MVP.md` §"The frame model".
///
/// **Sizing.**
///
/// ```text
///   locals_base = align16(16 + 8 × max(out_slots, 8))
///   size        = align16( max(16 + 8 × max(out_slots, 8),
///                              locals_base + locals)
///                          + 8 × (saved_gprs + saved_fprs) + 8 )
/// ```
///
/// — the linkage + outgoing-parameter area, the locals above it, one 8-byte slot
/// per saved register, and the 8-byte LR slot, rounded to 16. **Two independent
/// derivations agree on it**: 44 witnesses here (which is where the
/// stack-probing and `_RtlCheckStack12` rules below come from, `locals` up to
/// 200,000) and the 441-of-480 designed refutation sweep in
/// `docs/CODEGEN_FRAMED_CALLS.md` §1.2, which is where the `out_slots` term
/// comes from — every probe of this rung had `out_slots ≤ 8`, where the two
/// forms coincide at `align16(80 + locals + 8 + 8×saved)`. Exact while the
/// allocator does not spill; see [`FRAME_MAX_SAVED_NO_SPILL`].
///
/// Every row the roadmap had recorded as "96 B for one by-value temporary, 112 B
/// for two" is really the *saved-register* count, not a temporary count:
///
/// ```text
///   saved GPRs 0 1 2 3 4 5 6 7   frame 96 96 112 112 128 128 144 144
///   locals 1 → 96   locals 9 → 112   locals 64 → 160   locals 3600 → 3696
/// ```
///
/// **Register file.** Callee-saved GPRs are `r(32−n)…r31` and FPRs
/// `f(32−n)…f31` — always a contiguous run ending at the top of the file. They
/// share one descending array of 8-byte slots directly under the LR slot, GPRs
/// first: with two GPRs and one FPR, `r31` is at `−16(r1)`, `r30` at `−24` and
/// `f31` at `−32`. GPRs are stored with `std` (64-bit) and FPRs with `stfd`.
///
/// **Helpers.** Above a measured threshold c2 calls a save/restore helper
/// instead of open-coding the stores: **3 or more GPRs** →
/// `bl __savegprlr_(32−n)` / `b __restgprlr_(32−n)` (which save and restore the
/// LR too, so the `stw r12,-8(r1)` disappears and the epilogue *tail-branches*
/// into the restore helper), and **4 or more FPRs** →
/// `addi r12,r1,−(8 + 8×gprs)` + `bl __savefpr_(32−n)` /
/// `bl __restfpr_(32−n)`. Both are REL24 calls to externals.
///
/// **Stack probing.** A frame smaller than five pages is probed inline, one
/// `ld r12,−4096k(r1)` per page boundary crossed (`floor((F−1)/4096)` of them),
/// then `stwu r1,−F(r1)`. From five pages up it is
/// `li r12,−F` (or `lis`+`ori` past 32768) + `bl _RtlCheckStack12` +
/// `stwux r1,r1,r12`.
///
/// The emitter below covers only the layouts that need **no external helper and
/// no stack check** — everything else is refused by name, because those shapes
/// need a second REL24 site per function that the obj writer does not model.
/// The thresholds are therefore load-bearing gates, not decoration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameLayout {
    /// Bytes of addressed locals + compiler temporaries, above
    /// [`Self::locals_base`].
    pub locals: u32,
    /// **The argument count of the widest call this function makes** (0 for a
    /// leaf), floored at [`FRAME_MIN_OUT_SLOTS`]. Measured to be the maximum
    /// over the body's calls and not the last or first one — two calls of
    /// different arity in either order give the same frame
    /// (`docs/CODEGEN_FRAMED_CALLS.md` §1.2).
    pub out_slots: u8,
    /// Callee-saved GPRs: `r(32−n)…r31`.
    pub saved_gprs: u8,
    /// Callee-saved FPRs: `f(32−n)…f31`.
    pub saved_fprs: u8,
}

impl FrameLayout {
    /// The number of 8-byte register save slots, including the LR slot.
    fn save_slots(&self) -> u32 {
        1 + self.saved_gprs as u32 + self.saved_fprs as u32
    }

    /// Total callee-saved registers, the input to the spill boundary.
    fn n_saved(&self) -> u8 {
        self.saved_gprs.saturating_add(self.saved_fprs)
    }

    /// Where addressed locals start, relative to the new SP: the linkage area
    /// plus the outgoing-parameter home area, **16-aligned**. The alignment is
    /// measured, not assumed — with 9 outgoing slots the parameter area ends at
    /// SP+88 and the locals still start at SP+96, which an 8-aligned model
    /// mispredicts (`docs/CODEGEN_FRAMED_CALLS.md` §1.2).
    pub fn locals_base(&self) -> u32 {
        self.param_area_end().div_ceil(16) * 16
    }

    fn param_area_end(&self) -> u32 {
        16 + 8 * (self.out_slots as u32).max(FRAME_MIN_OUT_SLOTS)
    }

    /// The allocated frame size in bytes (the `stwu` displacement, negated).
    pub fn size(&self) -> u32 {
        let body = self.param_area_end().max(self.locals_base() + self.locals);
        (body + 8 * self.save_slots()).div_ceil(16) * 16
    }

    /// `-8` for the LR slot, then `-16, -24, …` for the saved registers: GPRs
    /// from `r31` downwards, then FPRs from `f31` downwards.
    fn gpr_slot(&self, i: u8) -> i16 {
        -16 - 8 * i as i16
    }
    fn fpr_slot(&self, i: u8) -> i16 {
        -16 - 8 * (self.saved_gprs as i16 + i as i16)
    }

    /// Page boundaries the frame crosses, i.e. how many inline probes the
    /// prologue emits. `F = 4096` crosses none; `F = 4112` crosses one.
    pub fn probe_pages(&self) -> u32 {
        self.size().saturating_sub(1) / FRAME_PAGE
    }

    /// True when the frame is allocated through `_RtlCheckStack12` rather than
    /// inline probes + `stwu`. Measured boundary: `F = 20464` is inline and
    /// `F = 20480 = 5 × 4096` is the helper.
    pub fn needs_stack_check(&self) -> bool {
        self.size() >= 5 * FRAME_PAGE
    }

    /// True when the GPR saves go through `__savegprlr_N` / `__restgprlr_N`.
    /// Measured: 2 saved GPRs are open-coded `std`s, 3 are the helper.
    pub fn needs_gpr_helper(&self) -> bool {
        self.saved_gprs >= 3
    }

    /// True when the FPR saves go through `__savefpr_N` / `__restfpr_N`.
    /// Measured: 3 saved FPRs are open-coded `stfd`s, 4 are the helper — a
    /// *different* threshold from the GPR one, which is why they are two
    /// predicates and not one.
    pub fn needs_fpr_helper(&self) -> bool {
        self.saved_fprs >= 4
    }

    /// The refusal reason for a layout this emitter cannot produce, or `None`.
    /// Each arm is a shape whose prologue contains a second REL24 call site.
    pub fn out_of_class_ctx(&self) -> Option<&'static str> {
        if self.needs_gpr_helper() {
            return Some("frame-savegprlr-helper");
        }
        if self.needs_fpr_helper() {
            return Some("frame-savefpr-helper");
        }
        if self.needs_stack_check() {
            return Some("frame-rtlcheckstack12");
        }
        // Unreachable behind the two helper thresholds today (3 and 4), and kept
        // as the second lock because the *sizing* rule stops being exact here
        // and a wrong `stwu` immediate is one silent byte.
        if self.n_saved() > FRAME_MAX_SAVED_NO_SPILL {
            return Some("frame-allocator-spill");
        }
        None
    }

    /// The prologue: `mflr`, the LR + register saves, the probes, the `stwu`.
    /// Its byte length is the function's `$M(n)` label value and, divided by
    /// four, the `PrologLen` field of its `.pdata` record.
    pub fn prologue(&self) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx() {
            return Err(out_of_class(ctx));
        }
        let f = self.size();
        // A frame this emitter can build always fits the `stwu` immediate: the
        // stack-check threshold (5 pages) is well under 32768. Assert rather
        // than truncate, because a silent wrap is a valid `stwu` of the wrong
        // size — exactly the fuzzy-invisible corruption the boundary rule is
        // about.
        let neg = i32::try_from(f)
            .ok()
            .and_then(|v| i16::try_from(-v).ok())
            .ok_or_else(|| out_of_class("frame larger than a stwu immediate"))?;
        let mut w: Vec<u8> = Vec::with_capacity(4 * (3 + self.save_slots() as usize));
        w.extend_from_slice(&FRAME_MFLR_R12.to_be_bytes());
        w.extend_from_slice(&FRAME_LR_STORE.to_be_bytes());
        // GPRs ascending in slot address: r(32-n) lowest, r31 at -16.
        for i in (0..self.saved_gprs).rev() {
            w.extend_from_slice(&encode_std(31 - i, 1, self.gpr_slot(i)));
        }
        // Then the FPRs, again ascending in address — and BELOW the GPRs.
        for i in (0..self.saved_fprs).rev() {
            w.extend_from_slice(&encode_stfd(31 - i, 1, self.fpr_slot(i)));
        }
        for k in 1..=self.probe_pages() {
            let d = -((k * FRAME_PAGE) as i32) as i16;
            w.extend_from_slice(&encode_ld(12, 1, d));
        }
        w.extend_from_slice(&encode_stwu(1, 1, neg));
        Ok(w)
    }

    /// The epilogue: free the frame, restore LR, restore the saved registers in
    /// ascending slot address (so FPRs, which sit lower, come first), `blr`.
    pub fn epilogue(&self) -> Result<Vec<u8>, BackendError> {
        if let Some(ctx) = self.out_of_class_ctx() {
            return Err(out_of_class(ctx));
        }
        let f = self.size();
        let mut w: Vec<u8> = Vec::with_capacity(4 * (4 + self.save_slots() as usize));
        if let Ok(pos) = i16::try_from(f) {
            w.extend_from_slice(&encode_addi(1, 1, pos));
        } else {
            w.extend_from_slice(&FRAME_BACKCHAIN.to_be_bytes());
        }
        w.extend_from_slice(&FRAME_LR_LOAD.to_be_bytes());
        w.extend_from_slice(&FRAME_MTLR_R12.to_be_bytes());
        for i in (0..self.saved_fprs).rev() {
            w.extend_from_slice(&encode_lfd(31 - i, 1, self.fpr_slot(i)));
        }
        for i in (0..self.saved_gprs).rev() {
            w.extend_from_slice(&encode_ldr(31 - i, 1, self.gpr_slot(i)));
        }
        w.extend_from_slice(&encode_blr());
        Ok(w)
    }
}

/// `std rS, DS(rA)` — store doubleword, primary opcode 62, DS-form (the low two
/// bits select the form, so the displacement must be a multiple of 4). Captured
/// as `fbe1fff0` = `std r31,-16(r1)` in every callee-saved GPR prologue.
pub fn encode_std(rs: u8, ra: u8, ds: i16) -> [u8; 4] {
    let word: u32 =
        (62 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | ((ds as u16 as u32) & 0xFFFC);
    word.to_be_bytes()
}

/// `ld rD, DS(rA)` with a **GPR** destination — the epilogue's reload. Same
/// encoder as [`encode_ld`]; named separately only where the frame code reads
/// better for it. Captured as `ebe1fff0` = `ld r31,-16(r1)`.
fn encode_ldr(rd: u8, ra: u8, ds: i16) -> [u8; 4] {
    encode_ld(rd, ra, ds)
}

/// `stfd frS, d(rA)` — store float double, primary opcode 54 (D-form, so any
/// 16-bit displacement). Captured as `dbe1fff0` = `stfd f31,-16(r1)`.
pub fn encode_stfd(frs: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (54 << 26) | ((frs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `lfd frD, d(rA)` — load float double, primary opcode 50. Captured as
/// `cbe1fff0` = `lfd f31,-16(r1)`.
pub fn encode_lfd(frd: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (50 << 26) | ((frd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `stwu rS, d(rA)` — store word with update, primary opcode 37: the frame
/// allocation. Captured as `9421ffa0` = `stwu r1,-96(r1)`.
pub fn encode_stwu(rs: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (37 << 26) | ((rs as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// A framed non-leaf call's emitted body: the bytes, and the `.text` offsets the
/// caller needs — the REL24 site of the `bl` and the prologue length that
/// becomes the `$M(n)` label and the `.pdata` `PrologLen`.
pub struct FramedBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offset of the `bl <callee>` (already includes
    /// `base_off`): the REL24 relocation site.
    pub bl_offset: u32,
    /// Prologue length in bytes, relative to the function start.
    pub prolog_len: u32,
}

/// Emit the `.text` for a **framed non-leaf call** `return g(<formal>) + k`
/// (W4b2).
///
/// ```text
/// 7d8802a6  mflr r12                prologue: save LR
/// 9181fff8  stw  r12,-8(r1)
/// 9421ffa0  stwu r1,-96(r1)         allocate the 96-byte frame
/// [7c832378  or  r3,rN,rN]          argument setup — ONLY when the argument is
///                                   not already the formal in r3
/// 4bfffff5  bl   <callee>           REL24 reloc site
/// 3863kkkk  addi r3,r3,k            the post-call op (+k)
/// 38210060  addi r1,r1,96           epilogue: free frame
/// 8181fff8  lwz  r12,-8(r1)         restore LR
/// 7d8803a6  mtlr r12
/// 4e800020  blr
/// ```
///
/// **The argument-setup word was missing and that was a live wrong-bytes emit.**
/// This function used to emit one byte-constant 0x24-byte body, on the tacit
/// assumption that the call's argument is always the formal already in r3. The
/// parser only ever required the argument to be *a* formal, so
/// `int f(int a,int b){ return g(b) + 1; }` emitted 9 words where c2 emits 10 —
/// `or r3,r4,r4` at `.text+0xC` — with the `.pdata` `FuncLen` and both `$M`
/// labels wrong to match. 37 of 47 probes around the accepted class mismatched,
/// including every member function (`this` occupies r3, so a one-parameter
/// member's argument is in r4) and every free function with a leading `float`,
/// `double`, `long long`, pointer or 8-byte aggregate parameter. The sweep never
/// separated it because every generated framed case is `int F(int a){ return
/// g(a) + 1; }` — one parameter, in r3. `docs/GAPS.md` §6: a corpus holding only
/// the safe half of a pair cannot see the dangerous half.
///
/// The setup is computed by the caller through [`select_text`], the *same*
/// locator the integer tail call uses for the same job, so the formal → argument
/// register mapping has one implementation and not two.
///
/// `k` must fit the signed-16-bit `addi` immediate (the IL parser guarantees
/// this before constructing the [`c2_il::FramedCall`]).
///
/// `base_off` is the function's start within the `.text` section it lands in —
/// 0 under `/Gy` (its own COMDAT) and its packed offset otherwise. It exists
/// because the `bl` displacement follows MSVC's `disp = −(own .text offset)`
/// convention, so a framed function that is **not first** in a packed `.text`
/// needs a different branch word: `?f` at 0x08 with the `bl` at 0x14 gets
/// `4BFFFFED`, not the `4BFFFFF5` this function emitted unconditionally. That
/// was unreachable while a framed TU was gated to one function and became a
/// live wrong-bytes emit the moment the gate came off.
pub fn framed_call_text(
    setup: &[u8],
    add_k: i32,
    base_off: u32,
    frame: FrameLayout,
) -> Result<FramedBody, BackendError> {
    let k = add_k as i16; // range-checked upstream (c2_il::func::parse_segment)
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;
    let bl_offset = base_off + prolog_len + setup.len() as u32;
    let mut text = Vec::with_capacity(prologue.len() + setup.len() + 8 + epilogue.len());
    text.extend_from_slice(&prologue);
    text.extend_from_slice(setup);
    // Call (LK=1); the REL24 reloc at `bl_offset` patches the target.
    text.extend_from_slice(&encode_call_branch(bl_offset));
    // Post-call op.
    text.extend_from_slice(&encode_addi(RET_REG, RET_REG, k)); // addi r3,r3,k
    text.extend_from_slice(&epilogue);
    Ok(FramedBody { text, bl_offset, prolog_len })
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
    mode: OptMode,
) -> Result<(Vec<u8>, u32), BackendError> {
    let mut text = select_text(func, mode)?; // arg-setup + trailing blr
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

/// `mr rA, rS` — the `or rA, rS, rS` idiom c2 uses for a register-to-register
/// move (opcode 31, XO 444).
pub fn encode_mr(ra: u8, rs: u8) -> [u8; 4] {
    xo31(rs, ra, rs, 444)
}

/// Lower an **argument permutation** for a multi-argument tail call: emit the
/// register moves that put each argument register's wanted value in place, with
/// `r11` as the single break temp.
///
/// `sources[i]` is the parameter index whose value argument slot `i` wants, so
/// slot `i` must end up holding what `ARG_REGS[sources[i]]` holds on entry.
/// `sources` being the identity is the passthrough case and emits nothing.
///
/// The rule, from the captures in `fixtures/cpp/il_call_multi.cpp`: decompose the
/// permutation into cycles; for each cycle save the source of its **lowest**
/// destination into the temp, then assign along the cycle in the order forced by
/// clobbering, filling that lowest destination from the temp last.
///
/// Only a **single** non-trivial cycle is accepted. Two disjoint cycles do both
/// saves up front (r11 then r10) and then have several clobber-free orders to
/// choose between, and the one capture available does not determine which — see
/// `rev4` in that fixture. A repeated argument is also refused: `dup3` emits a
/// *dead* `mr r11,r4`, which no live-value-driven solver would produce.
pub fn permute_args_text(sources: &[usize]) -> Result<Vec<u8>, BackendError> {
    if sources.len() > ARG_REGS.len() {
        return Err(out_of_class(
            "more arguments than the eight register slots: the rest are \
             stack-homed; out of class",
        ));
    }
    // A value wanted by two slots is a duplicate, which emits a dead move.
    for (i, s) in sources.iter().enumerate() {
        if sources[..i].contains(s) {
            return Err(out_of_class(
                "an argument value is passed twice: c2 emits a dead `mr` through \
                 the temp for this shape; out of class",
            ));
        }
        if *s >= ARG_REGS.len() {
            return Err(out_of_class("argument sources a stack-homed parameter"));
        }
    }

    // Cycle-decompose. `sources[i] == i` is a fixed point and needs no move.
    let n = sources.len();
    let mut seen = vec![false; n];
    let mut cycles: Vec<Vec<usize>> = Vec::new();
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        // Walk destination -> its source, collecting the cycle.
        let mut cycle = Vec::new();
        let mut at = start;
        while !seen[at] {
            seen[at] = true;
            cycle.push(at);
            at = sources[at];
            // A source outside `sources`' own index range cannot close a cycle;
            // that is a permutation of a larger set than we were given.
            if at >= n {
                return Err(out_of_class(
                    "argument permutation references a slot outside the argument \
                     list; out of class",
                ));
            }
        }
        cycles.push(cycle);
    }

    if cycles.is_empty() {
        return Ok(Vec::new()); // passthrough
    }
    if cycles.len() > 1 {
        return Err(out_of_class(
            "argument permutation has two or more disjoint cycles: c2 hoists both \
             saves (r11 then r10) and then has several clobber-free orders to pick \
             from, which one capture does not determine; out of class",
        ));
    }

    // One cycle. Its lowest destination is filled from the temp, last.
    let cycle = &cycles[0];
    let lowest = *cycle.iter().min().expect("non-empty cycle");
    let reg = |slot: usize| ARG_REGS[slot];
    let mut t = Vec::new();
    t.extend_from_slice(&encode_mr(SCRATCH_REG, reg(sources[lowest])));
    // Walk backwards from `lowest`: each step writes a destination whose old
    // value has already been consumed. This is the unique clobber-free order,
    // and it runs in whichever direction the cycle happens to go — which is why
    // `rot3` emits r4-then-r5 and `rot3b` emits r5-then-r4.
    let mut dst = sources[lowest];
    while dst != lowest {
        t.extend_from_slice(&encode_mr(reg(dst), reg(sources[dst])));
        dst = sources[dst];
    }
    t.extend_from_slice(&encode_mr(reg(lowest), SCRATCH_REG));
    Ok(t)
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
pub fn compare_leaf_text(
    cmp: &c2_il::CompareLeaf,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    use c2_il::Rel;
    // The relational spines below are `/Ox` shapes. `/O1` reallocates them — 14 of
    // `w6_rel_k`'s 19 leaves differ in their register fields (never in an opcode) —
    // and unlike the chain allocator the rule has not been enumerated, so this
    // refuses rather than emitting `/Ox` registers. `docs/OPT_MODE.md` §4.1.
    // `/O1` emits the SAME spines — same opcodes, operand order, immediates and
    // schedule — and reallocates only the temporaries: a temp whose defining
    // instruction makes the last use of the value in r11 is written to r11 instead
    // of taking a fresh descending number. 34 of the 108 matrix cells are therefore
    // byte-identical and the other 74 differ only in register fields.
    // `docs/CODEGEN_W6_O1.md` has the full side-by-side table; each arm below names
    // its own substitution, because which temps can collapse depends on what is
    // still live at that point in that spine.
    let o1 = mode == OptMode::O1;
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
                // /O1: the `andc` is the last use of the `neg` result, so it takes
                // r11. Ten of the twelve zero folds are mode-identical; this and
                // `<=` are the two that are not, for exactly that reason.
                let d = if o1 { 11 } else { 10 };
                t.extend_from_slice(&encode_neg(11, a));
                t.extend_from_slice(&encode_andc(d, 11, a));
                t.extend_from_slice(&encode_srwi31(RET_REG, d));
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
                // /O1: as for `>` above — the `orc` consumes the dying `neg`.
                let d = if o1 { 11 } else { 10 };
                t.extend_from_slice(&encode_neg(11, a));
                t.extend_from_slice(&encode_orc(d, a, 11));
                t.extend_from_slice(&encode_srwi31(RET_REG, d));
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
    // Only the `==`/`!=` spines form `a - k` as `addi r11,a,-k`, so only they need
    // `-k` to fit the immediate — and at `k == i16::MIN` it does not, because
    // negating it overflows. The port emitted a wrong immediate there.
    //
    // Scoped to those two relations deliberately: `<`, `<=`, `>` and `>=` reach
    // spines that never negate `k` and are correct at the boundary. `w6_rel_k.cpp`
    // tests `a <= -32768` and passes, which is exactly why the bug survived — that
    // fixture probes every relation, and both i16 boundaries, but never a
    // vulnerable relation *at* a boundary. A generated sweep over the cross product
    // found it at once.
    let negatable = k16.checked_neg().is_some();
    let needs_negation = matches!(cmp.rel, Rel::Eq | Rel::Ne);
    // **Two different immediate-eligibility predicates are in play, and they are
    // not interchangeable.** The carry spines (`<`, `<=`, `>`, `>=`) gate on raw
    // SIMM16 encodability, so `a > 4294967291u` is a legitimate
    // `subfic r11,r3,-5`. The `==`/`!=` difference spines gate on the literal's
    // **unsigned value** lying in `[0, 32767]`; against a large unsigned c2
    // materializes the constant and subtracts instead, one instruction more.
    //
    // Sharing one predicate was a live wrong-bytes emit in **both** modes:
    // `int f(unsigned a){return a == 4294967295u;}` and its `!=`, `-5` and
    // `4294967291u` siblings each came out 4 bytes short of the reference
    // (divergence at obj offset 8). Four of the 108 cells of the comparison
    // matrix, and none of them reachable from `w6_rel_k.cpp` or from
    // `scripts/expr_sweep.sh`, whose unsigned literals are all small — found only
    // by enumerating the matrix `docs/CODEGEN_W6_O1.md` tabulates.
    //
    // Refused rather than lowered: the materialize-and-subtract form is the wide
    // -literal path, which is uncharacterized for its own reasons (and where `/Ox`
    // does not even start allocating at r11 — see that doc's asymmetry list).
    if needs_negation && !cmp.signed && cmp.k < 0 {
        return Err(out_of_class(
            "`==`/`!=` against an unsigned literal above 32767: the difference \
             spine's `addi a,-k` is only used when the literal's UNSIGNED value \
             fits the immediate, and c2 materializes the constant instead; the \
             carry spines' raw-SIMM16 rule does not apply here",
        ));
    }
    if needs_negation && !negatable {
        return Err(out_of_class(
            "`==`/`!=` against i16::MIN: the difference spine needs `addi a,-k`, and \
             -(-32768) does not fit the immediate; out of class",
        ));
    }

    match (cmp.rel, cmp.signed) {
        // `a == k` → difference, then "is it zero".
        (Rel::Eq, _) => {
            // /O1: the `cntlzw` is the difference's last use, so it lands in r11.
            let d = if o1 { 11 } else { 10 };
            t.extend_from_slice(&encode_addi(11, a, -k16));
            t.extend_from_slice(&encode_cntlzw(d, 11));
            t.extend_from_slice(&encode_rlwinm(RET_REG, d, 27, 31, 31));
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
            // /O1 names the don't-care `subfe` source r11 as well as its dest, so
            // unlike /Ox it reads a *defined* (if dead) register here.
            let (d, src) = if o1 { (11, 11) } else { (9, 10) };
            t.extend_from_slice(&encode_subfic(11, a, k16));
            t.extend_from_slice(&encode_subfe(d, src, src));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, d));
        }
        // signed `a > k`: the 5-instruction spine. p = a (the greater side),
        // q = k. The final clrlwi exists solely to kill the `2` case.
        (Rel::Gt, true) => {
            // /O1: the `subfc` dest stays fresh (r11 is still live for the `eqv`),
            // but the `eqv` is r11's last use and every temp from there on collapses
            // onto it.
            let (e, f, g) = if o1 { (11, 11, 11) } else { (9, 8, 7) };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_subfc(10, a, 11)); // r10 dead; CA is the point
            t.extend_from_slice(&encode_eqv(e, a, 11));
            t.extend_from_slice(&encode_srwi31(f, e));
            t.extend_from_slice(&encode_addze(g, f));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, g));
        }
        // signed `a < k`: the signed `>` spine with the two operand roles
        // swapped, and *only* that — the register numbers, the instruction count
        // and the order are all identical. Both differing words are the ones that
        // read `a` and `r11`: `subfc r10,r11,r3` (not `r3,r11`) and
        // `eqv r9,r11,r3` (not `r3,r11`). `eqv` is commutative, so the swap is
        // invisible in the *value* and visible only in the bytes.
        (Rel::Lt, true) => {
            // /O1: same collapse as signed `>`; only the two swapped operand
            // roles distinguish this spine from it.
            let (e, f, g) = if o1 { (11, 11, 11) } else { (9, 8, 7) };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_subfc(10, 11, a)); // r10 dead; CA is the point
            t.extend_from_slice(&encode_eqv(e, 11, a));
            t.extend_from_slice(&encode_srwi31(f, e));
            t.extend_from_slice(&encode_addze(g, f));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, g));
        }
        // unsigned `a < k`. Unlike unsigned `>`, the literal cannot ride in the
        // `subfic` immediate: the borrow wanted here is the one out of `a - k`,
        // and `subfic` only computes `SIMM - rA`. So `k` is materialized and the
        // spine is four instructions rather than three — which shifts every
        // later register down one (`subfe r8,r9,r9`, not `r9,r10,r10`).
        (Rel::Lt, false) => {
            // /O1: here the `subfc` IS r11's last use (no `eqv` follows), so its
            // dead dest collapses onto r11 — the opposite of the signed spines
            // above, and the clearest evidence that the rule is about consumption
            // rather than about the instruction's kind.
            let (c, d, src) = if o1 { (11, 11, 11) } else { (10, 8, 9) };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_subfc(c, 11, a)); // dead; CA is the point
            t.extend_from_slice(&encode_subfe(d, src, src)); // terms cancel
            t.extend_from_slice(&encode_clrlwi31(RET_REG, d));
        }
        // signed `a >= k`. Two sign terms plus the unsigned borrow, summed by one
        // `adde`: `srawi` broadcasts the sign of the *left* side of the `>=` as
        // 0/−1, `rlwinm ...,1,31,31` takes the sign of the *right* side as 0/1,
        // and `subfc` contributes CA = unsigned(left) >= unsigned(right).
        // The two shifts are emitted in **source** order (`a` before `k`), so
        // they take r10 and r9 in that order — which is why `<=` below, whose
        // left side is the literal, emits them the other way round.
        (Rel::Ge, true) => {
            // /O1: only the `subfc` moves — it is r11's last use, and the two sign
            // temps must both stay live for the `adde`.
            let d = if o1 { 11 } else { 8 };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_srawi(10, a, 31)); // sign(a) as 0/-1
            t.extend_from_slice(&encode_srwi31(9, 11)); // sign(k) as 0/1
            t.extend_from_slice(&encode_subfc(d, 11, a)); // dead; CA is the point
            t.extend_from_slice(&encode_adde(RET_REG, 9, 10));
        }
        // signed `a <= k` is `k >= a`, so the roles invert: the 0/1 shift now
        // applies to `a` and the 0/−1 one to `k`. Emission still follows source
        // order, so `rlwinm` (on `a`) comes first and takes r10.
        (Rel::Le, true) => {
            // /O1: as for `>=` — only the `subfc` dest collapses.
            let d = if o1 { 11 } else { 8 };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_srwi31(10, a)); // sign(a) as 0/1
            t.extend_from_slice(&encode_srawi(9, 11, 31)); // sign(k) as 0/-1
            t.extend_from_slice(&encode_subfc(d, a, 11)); // dead; CA is the point
            t.extend_from_slice(&encode_adde(RET_REG, 10, 9));
        }
        // unsigned `a >= k`: CA out of `a - k` *is* the answer, so all that is
        // left is to materialize it. `subfze rD,rA` computes `~rA + CA`, so
        // against a preloaded −1 it yields CA alone. Note `subfc` writes its
        // (dead) difference back over r11 rather than taking a fresh register.
        (Rel::Ge, false) => {
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_addi(10, 0, -1)); // li r10,-1
            t.extend_from_slice(&encode_subfc(11, 11, a)); // r11 reused; CA is the point
            t.extend_from_slice(&encode_subfze(RET_REG, 10));
        }
        // unsigned `a <= k` is the one shape where the literal *can* ride in the
        // `subfic` immediate — the borrow wanted is the one out of `k - a`. So no
        // `li r11,k`, three instructions, and the −1 is emitted **first** even
        // though it takes the lower register number.
        (Rel::Le, false) => {
            t.extend_from_slice(&encode_addi(10, 0, -1)); // li r10,-1
            t.extend_from_slice(&encode_subfic(11, a, k16)); // r11 dead; CA is the point
            t.extend_from_slice(&encode_subfze(RET_REG, 10));
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
    /// Materialize a bare `return <param>` whose parameter is not the first:
    /// `dest = src`, one `mr` (`or dest,src,src`). Only ever the last entry, so
    /// `dest` is r3.
    RegMove { src: u8 },
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

/// Which optimization mode's codegen to emit. Read from `.ex`'s per-function
/// optimization word (`c2_il::IlBundle::opt_words`), never guessed from argv.
///
/// The two differ in **exactly one rule**, established over all 108 three- and
/// four-operator integer chains and all 27 depth-2 trees: a chain intermediate
/// whose predecessor is already dead goes to a fresh descending register under
/// [`OptMode::Ox`] and to r11 under [`OptMode::O1`]. Never a different opcode,
/// never a different operand order — only a register field.
///
/// `/Ox` and `/O2` share a word *and* emit identical bytes (verified per function
/// across eight fixtures once the tail branch's displacement, which is section
/// layout rather than codegen, is masked). So one variant covers both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptMode {
    /// `/Ox` and `/O2` — optimize, favour speed. Every lowering here was
    /// originally established against this mode.
    Ox,
    /// `/O1` (and `#pragma optimize("s", on)`) — optimize, favour size. What the
    /// dc3 workload compiles with.
    O1,
}

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
    // Which shape this is — and whether it is a shape at all — is decided by
    // `c2_il::chain_form`, the SAME predicate the IL parser gates on. The four
    // distinct-formal / N1 / N2 / division rules used to be spelled out twice,
    // here and (partly) in the parser; that is how the depth rule ended up
    // enforced only here, with the census claiming bodies the port refused.
    if c2_il::chain_form(&func.ops, &func.params) != Some(c2_il::ChainForm::Depth2Tree) {
        return None;
    }
    let (l0, l1, op1, l2, l3, op2, root) = match func.ops.as_slice() {
        [IlOp::Load(a), IlOp::Load(b), o1, IlOp::Load(c), IlOp::Load(d), o2, r] => {
            (*a, *b, *o1, *c, *d, *o2, *r)
        }
        // `chain_form` already proved the shape; this is the destructuring.
        _ => return None,
    };
    let regs: Vec<u8> = [l0, l1, l2, l3]
        .iter()
        .map(|t| reg_of(*t))
        .collect::<Option<_>>()?;

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

pub fn select_text(func: &IlFunction, mode: OptMode) -> Result<Vec<u8>, BackendError> {
    // Out-of-class, not a pass failure. As `BackendError::Pass` this landed in the
    // harness's `port-error` bucket while every other refusal in this file landed in
    // `codegen-gap`, and `differential` coerced it to `NotImplemented` anyway — so
    // the two instruments classified the same function differently. The parser now
    // refuses this shape first (`straight_line_is_out_of_class`); this stays as the
    // backstop.
    if func.params.len() > ARG_REGS.len() {
        return Err(out_of_class(&format!(
            "more than {} register arguments ({}): the rest are stack-homed and need \
             a frame; out of class",
            ARG_REGS.len(),
            func.params.len()
        )));
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

    // Capacity only (the ops stream bounds both): no behavior change.
    let mut stack: Vec<Operand> = Vec::with_capacity(4);
    let mut plan: Vec<PlanOp> = Vec::with_capacity(func.ops.len() / 2 + 2);

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
            // An FP constant only ever appears in an FP expression, which
            // `float_leaf_text` owns; reaching the integer selector means the
            // classifier disagreed with the parser.
            IlOp::FpLit { .. } => {
                return Err(out_of_class(
                    "floating-point constant in an integer expression; out of class",
                ))
            }
            // Integer division is not modeled (`divw`/`divwu`, and a constant
            // divisor strength-reduces to a multiply-high). FP division reaches
            // `float_leaf_text` instead and never gets here.
            IlOp::Div => return Err(out_of_class("integer division; out of class")),
            // An indirect load only ever appears as the whole body of an
            // indirect-load leaf, which `indirect_load_text` owns. Reaching the
            // affine selector would mean lowering `*p` as if it were a register
            // operand — and c2 does not: the load lands in the scratch register
            // and the arithmetic reads it from there.
            IlOp::LoadInd { .. } | IlOp::LoadIndSized { .. } => {
                return Err(out_of_class(
                    "indirect load feeding arithmetic; out of class",
                ))
            }
            // A sub-object address only ever appears as the whole body of an
            // address leaf, which `addr_leaf_text` owns. An address that feeds an
            // integer expression would have to be converted to an integer first,
            // and no capture establishes that lowering.
            IlOp::AddrOf { .. } => {
                return Err(out_of_class(
                    "sub-object address feeding arithmetic; out of class",
                ))
            }
            // An indirect store only ever appears as the last op of a store
            // leaf, which `store_leaf_text` owns. A store is not a value at
            // all — reaching the affine selector would mean pushing one onto
            // the operand stack.
            IlOp::StoreInd { .. } | IlOp::StoreIndFp { .. } => {
                return Err(out_of_class(
                    "indirect store in an expression; out of class",
                ))
            }
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
                    // A bare `return <param>` whose value is not in r3 — the
                    // whole body is one register move. MEASURED across every
                    // argument slot and both scalar widths (`w18_reg_move.cpp`):
                    //
                    //   int f(int a,int b)         { return b; }  7c832378 mr r3,r4
                    //   int f(int a,int b,int c)   { return c; }  7ca32b78 mr r3,r5
                    //   int C::m(int x,int y) const{ return y; }  7ca32b78 mr r3,r5
                    //   S*  f(int a, S* s)         { return s; }  7c832378 mr r3,r4
                    //   int f(…8 params…)          { return h; }  7d435378 mr r3,r10
                    //
                    // and then `blr`. The move is the same instruction for an
                    // int, an unsigned, a short, a `long long` and a pointer —
                    // one 4-byte word in one GPR, no extension anywhere — which
                    // is what lets one arm serve all of them. `this` is already
                    // at index 0 of `func.params`, so a member function's first
                    // explicit formal is r4 without a second rule.
                    //
                    // The FP file has the same shape and is NOT this arm:
                    // `float f(float a,float b){return b;}` is `fmr f1,f2`, and
                    // `float_leaf_text` refuses it because the FP-argument index
                    // cannot be derived from the positional one (see there).
                    Base::Phys(other) => {
                        plan.push(PlanOp::RegMove { src: *other });
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
    // Every plan entry emits at most 8 bytes (wide immediates), plus the `blr`.
    let mut text: Vec<u8> = Vec::with_capacity(plan.len() * 8 + 4);
    let last = plan.len().saturating_sub(1);
    let mut next_scratch: u8 = SCRATCH_REG;
    let mut prev_reg: u8 = SCRATCH_REG;
    // The accumulator decision is made once for the WHOLE chain, not per operation.
    // If the chain contains any addition, every intermediate reuses r11 — including
    // the subtractions ahead of that addition. Only a chain with no addition at all
    // gives each intermediate its own descending register.
    //
    // Deciding per-operation instead was a mis-emit found by the generated 4-leaf
    // sweep (270 cases): `a + b - c - d` emits `subf r11,r5,r3 ; subf r11,r6,r11 ;
    // add r3,r11,r4` — r11 twice, even though both of those are subtractions —
    // against `a - b - c - d`, which really does descend
    // `subf r11 ; subf r10 ; subf r3`. The two rules coincide at one intermediate,
    // which is why every 3-leaf chain matched and only 4 leaves exposed it.
    //
    // **All of the above is the `/Ox` rule.** Under `/O1` (favour size) there is no
    // descending case at all: this plan is a serial chain, so every intermediate's
    // predecessor is dead by construction, and `/O1` reuses r11 for a dead
    // predecessor unconditionally. `a - b - c - d` is
    // `subf r11,r4,r3 ; subf r11,r5,r11 ; subf r3,r6,r11` — where `/Ox` descends
    // r11, r10, r3 — and the operator-dependence disappears with it. Enumerated
    // over all 108 three- and four-operator chains: only register fields differ,
    // never an opcode or an operand order.
    let chain_has_add = plan
        .iter()
        .any(|e| matches!(e, PlanOp::Bin { op: IlOp::Add, .. } | PlanOp::AddImm { .. }));
    for (i, entry) in plan.iter().enumerate() {
        let dest = if i == last {
            RET_REG
        } else if mode == OptMode::O1 || chain_has_add {
            SCRATCH_REG
        } else {
            match entry {
                // Unreachable while `chain_has_add` is false, but keeps the arm
                // exhaustive and the intent local.
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
                    IlOp::Div
                    | IlOp::Load(_)
                    | IlOp::Lit(_)
                    | IlOp::FpLit { .. }
                    | IlOp::LoadInd { .. }
                    | IlOp::LoadIndSized { .. }
                    | IlOp::AddrOf { .. }
                    | IlOp::StoreInd { .. }
                    | IlOp::StoreIndFp { .. } => {
                        unreachable!("not a modeled integer binary op")
                    }
                }
            }
            PlanOp::AddImm { src, k } => emit_add_imm(&mut text, dest, resolve(src), k),
            PlanOp::LoadImm { k } => emit_load_imm(&mut text, dest, k)?,
            PlanOp::RegMove { src } => text.extend_from_slice(&encode_mr(dest, src)),
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
        (
            IlOp::Load(_)
            | IlOp::Lit(_)
            | IlOp::FpLit { .. }
            | IlOp::LoadInd { .. }
            | IlOp::LoadIndSized { .. }
            | IlOp::AddrOf { .. }
            | IlOp::StoreInd { .. }
            | IlOp::StoreIndFp { .. },
            _,
            _,
        ) => {
            unreachable!("not a binary op")
        }
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
    fn encode_lwz_matches_reference_words() {
        // Transcribed from the reference obj of fixtures/cpp/il_expr_deref.cpp and
        // il_expr_member.cpp — not derived from the encoding rule.
        assert_eq!(encode_lwz(3, 3, 0), [0x80, 0x63, 0x00, 0x00]); // *p          , p in r3
        assert_eq!(encode_lwz(3, 4, 0), [0x80, 0x64, 0x00, 0x00]); // *p          , p in r4
        assert_eq!(encode_lwz(3, 5, 0), [0x80, 0x65, 0x00, 0x00]); // *p          , p in r5
        assert_eq!(encode_lwz(3, 3, 4), [0x80, 0x63, 0x00, 0x04]); // s->b
        assert_eq!(encode_lwz(3, 3, 16), [0x80, 0x63, 0x00, 0x10]); // s->d
        assert_eq!(encode_lwz(3, 3, 12), [0x80, 0x63, 0x00, 0x0C]); // p[3]
        assert_eq!(encode_lwz(3, 3, -4), [0x80, 0x63, 0xFF, 0xFC]); // p[-1]
        assert_eq!(encode_lwz(3, 3, 32000), [0x80, 0x63, 0x7D, 0x00]); // p[8000]
        assert_eq!(encode_lwz(3, 4, 8), [0x80, 0x64, 0x00, 0x08]); // int f(int a,S* s){return s->c;}
    }

    #[test]
    fn store_leaf_text_is_one_store_and_a_blr() {
        // Every expected word transcribed from the reference obj of
        // `fixtures/cpp/w25_store_leaf.cpp` and `work/lf/probes/p1.cpp`, not
        // derived from the encoding rule.
        let mut f = IlFunction {
            mangled_name: "?s_b@@YAXPAUS@@H@Z".into(),
            source_path: None,
            params: vec![0xF509, 0xF609],
            ops: vec![
                IlOp::Load(0xF509),
                IlOp::Load(0xF609),
                IlOp::StoreInd { off: 4, width: 4 },
            ],
            tail_call: None,
            framed_call: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
            arg_sources: None,
        };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,4(r3) ; blr"
        );
        // A ZERO displacement is NOT free here — the store still happens. This is
        // the exact opposite of `addr_leaf_text`, whose zero case emits nothing,
        // and the two shapes share a designator.
        f.ops[2] = IlOp::StoreInd { off: 0, width: 4 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,0(r3) ; blr"
        );
        // The width picks the opcode, and nothing else does.
        f.ops[2] = IlOp::StoreInd { off: 12, width: 1 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..4],
            [0x98, 0x83, 0x00, 0x0C],
            "stb r4,12(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 14, width: 2 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..4],
            [0xB0, 0x83, 0x00, 0x0E],
            "sth r4,14(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 32, width: 8 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..4],
            [0xF8, 0x83, 0x00, 0x20],
            "std r4,32(r3)"
        );
        // `std` is DS-form: an offset that is not a multiple of 4 cannot be
        // encoded at all, so it refuses rather than dropping the low two bits.
        f.ops[2] = IlOp::StoreInd { off: 30, width: 8 };
        assert!(store_leaf_text(&f).unwrap().is_err());
        // BOTH register fields move: `void f(int x, S* s, int v){ s->b = v; }` is
        // `90a40004` — value r5, base r4 — and a lowering that hardcoded either
        // would pass every two-parameter case.
        f.params = vec![0x1111, 0xF509, 0xF609];
        f.ops[2] = IlOp::StoreInd { off: 4, width: 4 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![0x90, 0xA4, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "stw r5,4(r4) ; blr"
        );
        // A literal value goes through the SCRATCH register, never r3: measured
        // `39600007 91630000` for `void f(S* s){ s->a = 7; }`.
        f.params = vec![0xF509];
        f.ops = vec![
            IlOp::Load(0xF509),
            IlOp::Lit(7),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x07, 0x91, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20
            ],
            "li r11,7 ; stw r11,0(r3) ; blr"
        );
        // …and a wide literal is the `lis`+`ori` pair through the same register.
        f.ops[1] = IlOp::Lit(70000);
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..8],
            [0x3D, 0x60, 0x00, 0x01, 0x61, 0x6B, 0x11, 0x70],
            "lis r11,1 ; ori r11,r11,4464"
        );
        // Not a store leaf at all: the ordinary selector keeps its behaviour.
        f.ops = vec![IlOp::Load(0xF509), IlOp::LoadInd { off: 4 }];
        assert!(store_leaf_text(&f).is_none());
    }

    #[test]
    fn addr_leaf_text_is_one_addi_and_a_blr() {
        // Every expected word transcribed from the reference obj of
        // `fixtures/cpp/w16_addr_leaf.cpp` and `work/bma/probes/p{1,2,3}.cpp`,
        // not derived from the encoding rule.
        let mut f = IlFunction {
            mangled_name: "?a_off4@@YAPAHPAUS@@@Z".into(),
            source_path: None,
            params: vec![0xEE09],
            ops: vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 4 }],
            tail_call: None,
            framed_call: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
            arg_sources: None,
        };
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x38, 0x63, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "addi r3,r3,4 ; blr"
        );
        // A ZERO offset emits the `blr` alone — the address is already in r3.
        // A lowering that emitted `addi r3,r3,0` would be one word too long, and
        // it is the case a nonzero-only test cannot see.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 0 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x4E, 0x80, 0x00, 0x20],
            "a zero offset emits nothing at all"
        );
        // The base is the `addi`'s rA, not a hardcoded r3.
        f.params = vec![0x1111, 0xEE09];
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 4 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x38, 0x64, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "addi r3,r4,4 ; blr"
        );
        // …and at zero offset from that same non-first base c2 emits `mr r3,r4`
        // — measured, `int* f(int k, S* s){ return &s->a; }` is `7c832378`, the
        // same word as the pointer identity beside it. The one case a
        // zero-offset-from-r3 test cannot see is precisely this one: a bare
        // `blr` here would silently return `k` instead of the address.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 0 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x7C, 0x83, 0x23, 0x78, 0x4E, 0x80, 0x00, 0x20],
            "mr r3,r4 ; blr"
        );
        // An offset past the signed 16-bit immediate is `addis` + `addi`.
        f.params = vec![0xEE09];
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 32768 }];
        assert!(addr_leaf_text(&f).unwrap().is_err(), "32768 does not fit an addi");
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 32764 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x38, 0x63, 0x7F, 0xFC, 0x4E, 0x80, 0x00, 0x20],
            "32764 still fits"
        );
        // Anything that is not EXACTLY `[Load, AddrOf]` is not this shape.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 4 }, IlOp::Lit(1), IlOp::Add];
        assert!(addr_leaf_text(&f).is_none());
    }

    #[test]
    fn indirect_load_text_is_one_lwz_and_a_blr() {
        let mut f = IlFunction {
            mangled_name: "?ld_p@@YAHPAH@Z".into(),
            source_path: None,
            params: vec![0xEE09],
            ops: vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }],
            tail_call: None,
            framed_call: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
            arg_sources: None,
        };
        assert_eq!(
            indirect_load_text(&f).unwrap().unwrap(),
            vec![0x80, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20]
        );
        // The base's register comes from its position in `params`, which is where a
        // member function's `this` sits at index 0.
        f.params = vec![0x1234, 0xEE09];
        f.ops = vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 4 }];
        assert_eq!(
            indirect_load_text(&f).unwrap().unwrap(),
            vec![0x80, 0x64, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20]
        );
        // Anything that is not EXACTLY `[Load, LoadInd]` is not this shape: c2 does
        // not lower a load that feeds arithmetic as a destination-register load.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }, IlOp::Lit(1), IlOp::Add];
        assert!(indirect_load_text(&f).is_none());
        // …and the affine selector must refuse it rather than pick a register.
        assert!(select_text(&f, OptMode::Ox).is_err());
    }

    #[test]
    fn narrow_load_encoders_match_reference_words() {
        // Every word transcribed from a reference obj of
        // `fixtures/cpp/w12_narrow_getters.cpp` (and the probe TUs behind
        // `docs/IL_LOAD_TYPES.md` §3) — not derived from the encoding rule.
        //
        // `char f(char* p){return *p;}`            88630000  lbz r3,0(r3)
        assert_eq!(encode_lbz(3, 3, 0), [0x88, 0x63, 0x00, 0x00]);
        // `int f(char* p){return *p;}`             89630000  lbz r11,0(r3)
        assert_eq!(encode_lbz(11, 3, 0), [0x89, 0x63, 0x00, 0x00]);
        // `int f(int a,char* p){return *p;}`       89640000  lbz r11,0(r4)
        assert_eq!(encode_lbz(11, 4, 0), [0x89, 0x64, 0x00, 0x00]);
        // `s->c` at 4 / `s->u` at 8 / `p[3]`       88630004 / 88630008 / 88630003
        assert_eq!(encode_lbz(3, 3, 4), [0x88, 0x63, 0x00, 0x04]);
        assert_eq!(encode_lbz(3, 3, 8), [0x88, 0x63, 0x00, 0x08]);
        assert_eq!(encode_lbz(3, 3, 3), [0x88, 0x63, 0x00, 0x03]);
        assert_eq!(encode_lbz(11, 3, 4), [0x89, 0x63, 0x00, 0x04]);
        // `short f(short* p){return *p;}`          a0630000  lhz r3,0(r3)
        assert_eq!(encode_lhz(3, 3, 0), [0xA0, 0x63, 0x00, 0x00]);
        // `s->h` at 6 / `p[2]` at 4 / `t_uh` at 6  a0630006 / a0630004
        assert_eq!(encode_lhz(3, 3, 6), [0xA0, 0x63, 0x00, 0x06]);
        assert_eq!(encode_lhz(3, 3, 4), [0xA0, 0x63, 0x00, 0x04]);
        // `int f(short* p){return *p;}` under /Ox  a1630000  lhz r11,0(r3)
        assert_eq!(encode_lhz(11, 3, 0), [0xA1, 0x63, 0x00, 0x00]);
        // `long long f(long long* p){return *p;}`  e8630000  ld r3,0(r3)
        assert_eq!(encode_ld(3, 3, 0), [0xE8, 0x63, 0x00, 0x00]);
        // `s->q` at 16 / `t_q` at 8 / `p[2]` at 16 e8630010 / e8630008
        assert_eq!(encode_ld(3, 3, 16), [0xE8, 0x63, 0x00, 0x10]);
        assert_eq!(encode_ld(3, 3, 8), [0xE8, 0x63, 0x00, 0x08]);
        // DS-form: the low two bits are the form's, never the displacement's. A
        // caller must gate `off % 4`; if one ever did not, the word it would get is
        // the truncated one, not a rounded-up address.
        assert_eq!(encode_ld(3, 3, -8), [0xE8, 0x63, 0xFF, 0xF8]);
        assert_eq!(encode_ld(3, 3, 3), [0xE8, 0x63, 0x00, 0x00]);
        // `extsb r3,r11` / `extsh r3,r11` — rS in bits 21..25, rA in 16..20, so the
        // operand order in the mnemonic is the reverse of the field order.
        assert_eq!(encode_extsb(3, 11), [0x7D, 0x63, 0x07, 0x74]);
        assert_eq!(encode_extsh(3, 11), [0x7D, 0x63, 0x07, 0x34]);
        // `extsb r11,r11` (`*p + 1`, the refused arithmetic form) and `extsb r3,r3`
        // (`int f(char a)`, the refused widen-param rung) — both captured, both
        // distinct words, so the register fields are pinned in each direction.
        assert_eq!(encode_extsb(11, 11), [0x7D, 0x6B, 0x07, 0x74]);
        assert_eq!(encode_extsb(3, 3), [0x7C, 0x63, 0x07, 0x74]);
    }

    #[test]
    fn narrow_indirect_load_text_matches_the_captured_bodies() {
        let f = |ops: Vec<IlOp>, params: Vec<u32>| IlFunction {
            mangled_name: "?g@@YADPAD@Z".into(),
            source_path: None,
            params,
            ops,
            tail_call: None,
            framed_call: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
            arg_sources: None,
        };
        let blr = [0x4E, 0x80, 0x00, 0x20];
        let body = |ops: Vec<IlOp>, params: Vec<u32>| {
            indirect_load_text(&f(ops, params)).unwrap().unwrap()
        };
        let sized = |width, sext, off| {
            vec![IlOp::Load(0xEE09), IlOp::LoadIndSized { off, width, sext }]
        };
        // `char g_c_c(char* p){return *p;}`  ->  lbz r3,0(r3) ; blr
        assert_eq!(
            body(sized(1, false, 0), vec![0xEE09]),
            [&[0x88, 0x63, 0x00, 0x00][..], &blr].concat()
        );
        // `int g_i_c(char* p){return *p;}`   ->  lbz r11,0(r3) ; extsb r3,r11 ; blr
        // The load targets the SCRATCH register and the extension produces r3 —
        // the r11-then-r3 rule. `lbz r3 ; extsb r3,r3` is the plausible wrong emit.
        assert_eq!(
            body(sized(1, true, 0), vec![0xEE09]),
            [&[0x88 + 1, 0x63, 0x00, 0x00][..], &[0x7D, 0x63, 0x07, 0x74], &blr].concat()
        );
        // `int g_i_c2(int a,char* p){return *p;}` -> base r4, destination still r11
        assert_eq!(
            body(sized(1, true, 0), vec![0x1234, 0xEE09]),
            [&[0x89, 0x64, 0x00, 0x00][..], &[0x7D, 0x63, 0x07, 0x74], &blr].concat()
        );
        // `short g_s_s(short* p){return *p;}` ->  lhz r3,0(r3) — never `lha`
        assert_eq!(
            body(sized(2, false, 0), vec![0xEE09]),
            [&[0xA0, 0x63, 0x00, 0x00][..], &blr].concat()
        );
        // `short m_h(S* s){return s->h;}`    ->  lhz r3,6(r3)
        assert_eq!(
            body(sized(2, false, 6), vec![0xEE09]),
            [&[0xA0, 0x63, 0x00, 0x06][..], &blr].concat()
        );
        // `long long m_q(S* s){return s->q;}` -> ld r3,16(r3)
        assert_eq!(
            body(sized(8, false, 16), vec![0xEE09]),
            [&[0xE8, 0x63, 0x00, 0x10][..], &blr].concat()
        );
        // The 4-byte load keeps its own variant and its own bytes.
        assert_eq!(
            body(vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 4 }], vec![0xEE09]),
            [&[0x80, 0x63, 0x00, 0x04][..], &blr].concat()
        );
        // An 8-byte load whose offset is not a multiple of 4 cannot be a DS-form
        // displacement: c2 emits `li r11,3 ; ldx r3,r3,r11` instead (measured on a
        // `#pragma pack(1)` member, `fixtures/cpp/w12_narrow_neg.cpp`). The parser
        // refuses it; this is the second lock.
        assert!(indirect_load_text(&f(sized(8, false, 3), vec![0xEE09]))
            .unwrap()
            .is_err());
        // Sign extension is only ever modeled at width 1 — a signed halfword
        // widening is mode-dependent and refused upstream.
        assert!(indirect_load_text(&f(sized(2, true, 0), vec![0xEE09]))
            .unwrap()
            .is_err());
        assert!(indirect_load_text(&f(sized(8, true, 0), vec![0xEE09]))
            .unwrap()
            .is_err());
        // A narrow load feeding arithmetic is not this shape at all (c2 extends in
        // place — `extsb r11,r11` — and the leaf extends across registers).
        assert!(indirect_load_text(&f(
            vec![IlOp::Load(0xEE09), IlOp::LoadIndSized { off: 0, width: 1, sext: true }, IlOp::Lit(1), IlOp::Add],
            vec![0xEE09]
        ))
        .is_none());
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
        let plain = FrameLayout::default();
        // `int f(int a){ return g(a) + 1; }` — the verified 0x24-byte body. `a` is
        // already in r3, so the argument setup is empty.
        let b = framed_call_text(&[], 1, 0, plain).unwrap();
        assert_eq!(
            b.text,
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
        assert_eq!((b.bl_offset, b.prolog_len), (0x0C, 0x0C));
        // `+ 2` differs only in the addi immediate.
        assert_eq!(framed_call_text(&[], 2, 0, plain).unwrap().text[19], 0x02);
        // Placed at 0x08 in a packed `.text` (a leaf ahead of it), the `bl` is at
        // 0x14 and its displacement follows: `4BFFFFED`, not `4BFFFFF5`. Every
        // other byte of the body is unchanged.
        let at8 = framed_call_text(&[], 1, 0x08, plain).unwrap();
        assert_eq!(&at8.text[12..16], &[0x4B, 0xFF, 0xFF, 0xED]);
        assert_eq!(&at8.text[..12], &b.text[..12]);
        assert_eq!(&at8.text[16..], &b.text[16..]);
        assert_eq!(at8.bl_offset, 0x14);
    }

    /// The argument-setup word, against the reference obj it was missing from.
    ///
    /// `int f(int a,int b){ return g(b) + 1; }` at `/Ox /GS- /c`, `.text` = 40
    /// bytes (10 words), `.pdata` `40000a03` (FuncLen 10, PrologLen 3), REL24 at
    /// 0x10. The port emitted the 9-word body with no `or` and every downstream
    /// field to match — a live wrong-bytes emit on mainline, and the reason
    /// [`Selected::Framed`] carries a setup at all.
    #[test]
    fn framed_call_moves_a_non_first_formal_into_r3() {
        let setup = encode_mr(RET_REG, 4);
        let b = framed_call_text(&setup, 1, 0, FrameLayout::default()).unwrap();
        assert_eq!(b.text.len(), 0x28);
        assert_eq!(&b.text[12..16], &[0x7C, 0x83, 0x23, 0x78]); // or r3,r4,r4
        assert_eq!(&b.text[16..20], &[0x4B, 0xFF, 0xFF, 0xF1]); // bl, disp −0x10
        assert_eq!((b.bl_offset, b.prolog_len), (0x10, 0x0C));
        // …and from r5, the three-formal case (`or r3,r5,r5` = 7ca32b78).
        let b5 = framed_call_text(&encode_mr(RET_REG, 5), 1, 0, FrameLayout::default()).unwrap();
        assert_eq!(&b5.text[12..16], &[0x7C, 0xA3, 0x2B, 0x78]);
    }

    /// **The frame-size formula, against every captured witness.**
    ///
    /// `size = align16(80 + locals + 8 + 8 × saved)`. Rows are
    /// `(locals, gprs, fprs) -> frame`, each read off a reference obj's `stwu`
    /// displacement (`docs/CODEGEN_PPC_MVP.md` §"The frame model" names the probe
    /// source for each). The saved-register column is what the roadmap had
    /// recorded as "96 B for one by-value temporary, 112 B for two" — the driver
    /// is the callee-saved register count, and a by-value temporary moves the
    /// *locals* column instead.
    #[test]
    fn frame_size_fits_every_captured_witness() {
        let rows: &[(u32, u8, u8, u32)] = &[
            // saved GPRs 0..7, no locals: g(a)+1 … g(a)+b+c+d+e+f+g+h
            (0, 0, 0, 96),
            (0, 1, 0, 96),
            (0, 2, 0, 112),
            (0, 3, 0, 112),
            (0, 4, 0, 128),
            (0, 5, 0, 128),
            (0, 6, 0, 144),
            (0, 7, 0, 144),
            // saved FPRs 1..5: float g(a)*b … the FPR file uses the same slots
            (0, 0, 1, 96),
            (0, 0, 2, 112),
            (0, 0, 3, 112),
            (0, 0, 4, 128),
            (0, 0, 5, 128),
            // mixed: GPRs above FPRs in one shared descending slot array. The
            // 8-byte locals are the int→double conversion spill at 80(r1).
            (8, 2, 1, 128),
            (8, 3, 2, 144),
            (8, 4, 3, 160),
            (8, 0, 1, 112),
            // locals only (`char buf[n]` / `int buf[n]` passed to the callee)
            (1, 0, 0, 96),
            (5, 0, 0, 96),
            (9, 0, 0, 112),
            (64, 0, 0, 160),
            (3600, 0, 0, 3696),
            (4080, 0, 0, 4176),
            (4096, 0, 0, 4192),
            (8096, 0, 0, 8192),
            (8097, 0, 0, 8192),
            (16384, 0, 0, 16480),
            (12000, 0, 0, 12096),
            (16000, 0, 0, 16096),
            (16296, 0, 0, 16384),
            (16312, 0, 0, 16400),
            (17000, 0, 0, 17088),
            (20000, 0, 0, 20096),
            (20376, 0, 0, 20464),
            (20392, 0, 0, 20480),
            (24000, 0, 0, 24096),
            (32000, 0, 0, 32096),
            (32664, 0, 0, 32752),
            (32680, 0, 0, 32768),
            (32696, 0, 0, 32784),
            (40000, 0, 0, 40096),
            (200000, 0, 0, 200096),
            (4008, 0, 0, 4096),
            (4009, 0, 0, 4112),
            // locals AND saved registers together (`char buf[30000]` + 2/3 live)
            (30000, 2, 0, 30112),
            (30000, 3, 0, 30112),
        ];
        for &(locals, saved_gprs, saved_fprs, want) in rows {
            let l = FrameLayout { locals, out_slots: 0, saved_gprs, saved_fprs };
            assert_eq!(l.size(), want, "frame for {l:?}");
        }
        // The `out_slots` term, from the independent 480-case refutation sweep
        // (`docs/CODEGEN_FRAMED_CALLS.md` §1.2). None of this rung's own probes
        // could see it — they all pass eight arguments or fewer, where the two
        // forms of the rule coincide.
        let wide: &[(u32, u8, u8, u8, u32)] = &[
            // `int g();` with `int b[20]`: 80 bytes of locals and NO outgoing
            // arguments still reserves the 8-slot parameter area. A "frame >= 96"
            // model predicts 112 and is refuted by 176.
            (80, 0, 0, 0, 176),
            // Two calls of different arity, either order: nOutSlots = 12,
            // nSaved = 2 -> align16(16 + 96 + 16 + 8) = 144.
            (0, 12, 2, 0, 144),
            // 9 outgoing slots: the parameter area ends at SP+88 and the locals
            // still start at SP+96, so the frame steps at 4L + 96 + 8 crossing 16.
            (4, 9, 0, 0, 112),
            (32, 9, 0, 0, 144),
        ];
        for &(locals, out_slots, saved_gprs, saved_fprs, want) in wide {
            let l = FrameLayout { locals, out_slots, saved_gprs, saved_fprs };
            assert_eq!(l.size(), want, "frame for {l:?}");
        }
        assert_eq!(FrameLayout { locals: 0, out_slots: 9, ..Default::default() }.locals_base(), 96);
        assert_eq!(FrameLayout::default().locals_base(), 80);
    }

    /// The measured thresholds. Each boundary is a *pair* of captures, because a
    /// threshold read off one side is a guess.
    #[test]
    fn frame_helper_and_probe_thresholds_are_where_the_captures_put_them() {
        let g = |n| FrameLayout { saved_gprs: n, ..Default::default() };
        let f = |n| FrameLayout { saved_fprs: n, ..Default::default() };
        // GPRs: 2 open-coded `std`s, 3 is `__savegprlr_29`.
        assert!(!g(2).needs_gpr_helper());
        assert!(g(3).needs_gpr_helper());
        // FPRs: 3 open-coded `stfd`s, 4 is `__savefpr_28` — a DIFFERENT threshold.
        assert!(!f(3).needs_fpr_helper());
        assert!(f(4).needs_fpr_helper());
        // Stack probing: F = 20464 is four inline `ld`s, F = 20480 = 5 pages is
        // `_RtlCheckStack12`.
        let l = |locals| FrameLayout { locals, ..Default::default() };
        assert_eq!(l(20376).size(), 20464);
        assert!(!l(20376).needs_stack_check());
        assert_eq!(l(20376).probe_pages(), 4);
        assert_eq!(l(20392).size(), 20480);
        assert!(l(20392).needs_stack_check());
        // A frame that lands exactly on a page boundary crosses one boundary
        // fewer than a frame one word past it: F = 4096 probes nothing.
        assert_eq!(l(4008).probe_pages(), 0);
        assert_eq!(l(4009).probe_pages(), 1);
        assert_eq!(l(0).probe_pages(), 0);
        // Every helper shape refuses by name rather than emitting a prologue with
        // an unrelocated call in it.
        assert_eq!(g(3).out_of_class_ctx(), Some("frame-savegprlr-helper"));
        assert_eq!(f(4).out_of_class_ctx(), Some("frame-savefpr-helper"));
        assert_eq!(l(20392).out_of_class_ctx(), Some("frame-rtlcheckstack12"));
        assert!(g(3).prologue().is_err() && f(4).epilogue().is_err());
        // `stwux r1,r1,r12` is the allocation `_RtlCheckStack12` pairs with; the
        // word is captured, the shape is refused. Pinned so the constant cannot
        // rot while it is unreachable.
        assert_eq!(FRAME_STWUX.to_be_bytes(), [0x7C, 0x21, 0x61, 0x6E]);
    }

    /// The prologue and epilogue of every layout the emitter will build, word for
    /// word against the reference objs.
    #[test]
    fn frame_prologue_and_epilogue_match_the_reference_words() {
        let w = |v: &[u8]| -> Vec<u32> {
            v.chunks(4).map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]])).collect()
        };
        // `int f(int a,int b){ return g(a) + b; }` — one saved GPR, frame 96.
        let one = FrameLayout { saved_gprs: 1, ..Default::default() };
        assert_eq!(one.size(), 96);
        assert_eq!(
            w(&one.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xFBE1FFF0, 0x9421FFA0]
        );
        assert_eq!(
            w(&one.epilogue().unwrap()),
            vec![0x38210060, 0x8181FFF8, 0x7D8803A6, 0xEBE1FFF0, 0x4E800020]
        );
        // Two saved GPRs, frame 112: saved ascending in slot address, restored the
        // same way, and the restores come AFTER the `mtlr`.
        let two = FrameLayout { saved_gprs: 2, ..Default::default() };
        assert_eq!(
            w(&two.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xFBC1FFE8, 0xFBE1FFF0, 0x9421FF90]
        );
        assert_eq!(
            w(&two.epilogue().unwrap()),
            vec![0x38210070, 0x8181FFF8, 0x7D8803A6, 0xEBC1FFE8, 0xEBE1FFF0, 0x4E800020]
        );
        // `float f(float a,float b,float c){ return g(a)*b*c; }` — two FPRs.
        let f2 = FrameLayout { saved_fprs: 2, ..Default::default() };
        assert_eq!(
            w(&f2.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xDBC1FFE8, 0xDBE1FFF0, 0x9421FF90]
        );
        assert_eq!(
            w(&f2.epilogue().unwrap()),
            vec![0x38210070, 0x8181FFF8, 0x7D8803A6, 0xCBC1FFE8, 0xCBE1FFF0, 0x4E800020]
        );
        // Two GPRs and one FPR: the GPRs take the two slots under LR and the FPR
        // the one below them, but the PROLOGUE stores GPRs first (descending in
        // address after the run) while the EPILOGUE restores in ascending address
        // — so the two lists are not mirror images. Reference: `float f(int a,int
        // b,float x,float y){ return g(x)*y + (float)(a+b); }`, frame 128.
        let mix = FrameLayout { locals: 8, out_slots: 0, saved_gprs: 2, saved_fprs: 1 };
        assert_eq!(mix.size(), 128);
        assert_eq!(
            w(&mix.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xFBC1FFE8, 0xFBE1FFF0, 0xDBE1FFE0, 0x9421FF80]
        );
        assert_eq!(
            w(&mix.epilogue().unwrap()),
            vec![0x38210080, 0x8181FFF8, 0x7D8803A6, 0xCBE1FFE0, 0xEBC1FFE8, 0xEBE1FFF0, 0x4E800020]
        );
        // Locals with page probes: `int f(int a){ char buf[4009]; … }`, frame
        // 4112, one probe. And `int buf[4096]`, frame 16480, four probes.
        let p1 = FrameLayout { locals: 4009, ..Default::default() };
        assert_eq!(
            w(&p1.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xE981F000, 0x9421EFF0]
        );
        let p4 = FrameLayout { locals: 16384, ..Default::default() };
        assert_eq!(
            w(&p4.prologue().unwrap()),
            vec![0x7D8802A6, 0x9181FFF8, 0xE981F000, 0xE981E000, 0xE981D000, 0xE981C000, 0x9421BFA0]
        );
        // The `.pdata` PrologLen is the prologue's word count, which is now a
        // function of the layout rather than the hardcoded 3.
        assert_eq!(FrameLayout::default().prologue().unwrap().len() / 4, 3);
        assert_eq!(one.prologue().unwrap().len() / 4, 4);
        assert_eq!(two.prologue().unwrap().len() / 4, 5);
        assert_eq!(p4.prologue().unwrap().len() / 4, 7);
    }

    #[test]
    fn int_tail_call_passthrough_is_bare_branch() {
        // `return g(a)` — a is already in r3, so no arg setup: a bare `b g` at
        // offset 0 (`48000000`), reloc site 0 — byte-identical to the void
        // tail call. Verified against the live obj (.text=48000000, REL24 @0x0).
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309)]);
        let (text, reloc) = int_tail_call_text(&f, 0, OptMode::Ox).unwrap();
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
        let (text, reloc) = int_tail_call_text(&f, 0, OptMode::Ox).unwrap();
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
            arg_sources: None,
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
            select_text(&func, OptMode::Ox).unwrap(),
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
            select_text(&f, OptMode::Ox).unwrap(),
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
            select_text(&f, OptMode::Ox).unwrap(),
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
            select_text(&f, OptMode::Ox).unwrap(),
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
        let (text, consts) = float_leaf_text(&f, false).unwrap();
        assert_eq!(
            text,
            vec![
                0xEC, 0x01, 0x00, 0xB2, // fmuls f0,f1,f2
                0xEC, 0x20, 0x00, 0xF2, // fmuls f1,f0,f3
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        assert!(consts.is_empty(), "no literals in this body");
    }

    // ---- W13b pooled floating-point constants -------------------------------

    /// IEEE binary64 bits for a value, as the IL carries an FP literal.
    fn f64bits(v: f64) -> u64 {
        v.to_bits()
    }

    #[test]
    fn fp_constant_loads_through_a_relocated_addis_lfs_pair() {
        // `float k_add(float a){ return a + 1.0f; }` — the live capture is
        // `3d600000 c00b0000 ec21002a 4e800020`:
        //   addis r11,r0,0    <- REFHI(__real@3f800000) + PAIR
        //   lfs   f0,0(r11)   <- REFLO(__real@3f800000) + PAIR
        //   fadds f1,f1,f0
        // Both immediates are 0; the linker patches them.
        let f = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(1.0), double: false },
                IlOp::Add,
            ],
        );
        let (text, consts) = float_leaf_text(&f, false).unwrap();
        assert_eq!(
            text,
            vec![
                0x3D, 0x60, 0x00, 0x00, // addis r11,r0,0
                0xC0, 0x0B, 0x00, 0x00, // lfs   f0,0(r11)
                0xEC, 0x21, 0x00, 0x2A, // fadds f1,f1,f0
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        assert_eq!(
            consts,
            vec![FpConstRef { bits: f64bits(1.0), double: false, hi_off: 0 }]
        );
    }

    #[test]
    fn fp_constant_claims_its_register_before_any_interior_temporary() {
        // `ke` in w13b_fpool: c2 folds `a*2.0f*b*3.0f` to `(a*b)*6.0f` and emits
        //   fmuls f13,f1,f2 ; addis r11,r0,0 ; lfs f0,0(r11) ; fmuls f1,f13,f0
        // The interior temp is f13, NOT f0 — so the constant took pool slot 0
        // even though the multiply is *emitted* first. Allocating temporaries in
        // emission order instead would put the multiply in f0 and match every
        // single-op body, which is exactly why this case is pinned.
        let f = fpfunc(
            vec![0x09E3, 0x09E4],
            vec![
                IlOp::Load(0x09E3),
                IlOp::Load(0x09E4),
                IlOp::Mul,
                IlOp::FpLit { bits: f64bits(6.0), double: false },
                IlOp::Mul,
            ],
        );
        let (text, _) = float_leaf_text(&f, false).unwrap();
        assert_eq!(
            text,
            vec![
                0xED, 0xA1, 0x00, 0xB2, // fmuls f13,f1,f2
                0x3D, 0x60, 0x00, 0x00, // addis r11,r0,0
                0xC0, 0x0B, 0x00, 0x00, // lfs   f0,0(r11)
                0xEC, 0x2D, 0x00, 0x32, // fmuls f1,f13,f0
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn double_constant_uses_lfd_and_the_double_primary_opcode() {
        // `double kd(double a){ return a + 1.0; }` →
        //   addis r11,r0,0 ; lfd f0,0(r11) ; fadd f1,f1,f0
        // `lfd` is primary 50 (not 48) and `fadd` primary 63 (not 59).
        let f = {
            let mut g = fpfunc(
                vec![0x09E3],
                vec![
                    IlOp::Load(0x09E3),
                    IlOp::FpLit { bits: f64bits(1.0), double: true },
                    IlOp::Add,
                ],
            );
            g.float_leaf = Some(true);
            g
        };
        let (text, consts) = float_leaf_text(&f, true).unwrap();
        assert_eq!(
            text,
            vec![
                0x3D, 0x60, 0x00, 0x00, // addis r11,r0,0
                0xC8, 0x0B, 0x00, 0x00, // lfd   f0,0(r11)
                0xFC, 0x21, 0x00, 0x2A, // fadd  f1,f1,f0
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        assert!(consts[0].double);
    }

    #[test]
    fn fp_constant_pool_refuses_what_it_has_not_characterized() {
        // Two constants: c2 hoists both `addis` into a prologue group and
        // schedules the loads at first use, so the REFLO site is no longer
        // `hi_off + 4`. Refuse.
        let two = fpfunc(
            vec![0x09E3, 0x09E4],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(1.0), double: false },
                IlOp::Add,
                IlOp::Load(0x09E4),
                IlOp::FpLit { bits: f64bits(2.0), double: false },
                IlOp::Add,
                IlOp::Sub,
            ],
        );
        assert!(float_leaf_text(&two, false).is_err());

        // A constant divisor strength-reduces to a reciprocal multiply, so a
        // surviving Div against a literal is not something the model expects.
        let div = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(3.0), double: false },
                IlOp::Div,
            ],
        );
        assert!(float_leaf_text(&div, false).is_err());

        // A `float` literal whose binary64 pattern does not narrow exactly
        // would pool four bytes that are not the value c2 pooled.
        let inexact = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(0.1), double: false },
                IlOp::Add,
            ],
        );
        assert!(float_leaf_text(&inexact, false).is_err());

        // A width mismatch between the literal and the expression implies a
        // conversion the model does not emit.
        let mixed = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(1.0), double: true },
                IlOp::Add,
            ],
        );
        assert!(float_leaf_text(&mixed, false).is_err());
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
        compare_leaf_text(&c2_il::CompareLeaf { param: 0xE309, rel, signed, k }, OptMode::Ox).unwrap()
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

    // ---- multi-argument tail-call permutations ------------------------------

    #[test]
    fn argument_permutations_match_the_reference() {
        // Captured from `int g3(int,int,int)` call sites; `sources[i]` is the
        // parameter index that argument slot i wants.
        // Passthrough: the parameters are already placed, so no moves at all.
        assert!(permute_args_text(&[0, 1]).unwrap().is_empty());
        assert!(permute_args_text(&[0, 1, 2]).unwrap().is_empty());

        // g3(b,a,c) — swap r3/r4, r5 untouched.
        assert_eq!(
            permute_args_text(&[1, 0, 2]).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(a,c,b) — swap r4/r5. The temp still takes the source of the cycle's
        // LOWEST destination (r4's), not r3's, which is not in the cycle at all.
        assert_eq!(
            permute_args_text(&[0, 2, 1]).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7D, 0x64, 0x5B, 0x78, // mr r4,r11
            ]
        );
        // g3(c,b,a) — swap r3/r5, r4 untouched.
        assert_eq!(
            permute_args_text(&[2, 1, 0]).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x65, 0x1B, 0x78, // mr r5,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // The two 3-cycles run in OPPOSITE directions, so their middle moves come
        // out in opposite orders. Fixing either order as "the rule" mis-emits the
        // other, which is why both are pinned.
        // g3(b,c,a): r3<-r4, r4<-r5, r5<-r3.
        assert_eq!(
            permute_args_text(&[1, 2, 0]).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0xA4, 0x2B, 0x78, // mr r4,r5
                0x7C, 0x65, 0x1B, 0x78, // mr r5,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(c,a,b): r3<-r5, r4<-r3, r5<-r4.
        assert_eq!(
            permute_args_text(&[2, 0, 1]).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
    }

    #[test]
    fn argument_permutations_refuse_the_uncharacterized_shapes() {
        // Two disjoint cycles (`g4(d,c,b,a)`): c2 hoists both saves into r11 and
        // r10 and then picks one of several clobber-free orders. One capture does
        // not determine which.
        assert!(permute_args_text(&[3, 2, 1, 0]).is_err());
        // A repeated argument (`g3(b,a,b)`) emits a dead `mr r11,r4` that no
        // live-value-driven solver would produce.
        assert!(permute_args_text(&[1, 0, 1]).is_err());
        // More arguments than register slots.
        assert!(permute_args_text(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).is_err());
    }

    #[test]
    fn compare_lt_ge_le_against_a_nonzero_literal_match_the_reference() {
        use c2_il::Rel;
        // All six captured from `int f(int a){ return a <rel> 5; }` (and the
        // `unsigned` overloads) against the live toolchain.

        // signed `a < 5` — the signed `>` spine with the two operands that read
        // `a`/`r11` swapped, and nothing else changed. `eqv` is commutative, so
        // the swap is invisible in the value and visible only here.
        assert_eq!(
            cmp(Rel::Lt, true, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc r10,r11,r3
                0x7D, 0x69, 0x1A, 0x38, // eqv r9,r11,r3
                0x55, 0x28, 0x0F, 0xFE, // srwi r8,r9,31
                0x7C, 0xE8, 0x01, 0x94, // addze r7,r8
                0x54, 0xE3, 0x07, 0xFE, // clrlwi r3,r7,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `a < 5` — four words, not the three of unsigned `>`: the
        // literal cannot ride in a `subfic` immediate here, and materializing it
        // shifts the dead `subfe` down to r8/r9 (from r9/r10).
        assert_eq!(
            cmp(Rel::Lt, false, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc r10,r11,r3
                0x7D, 0x09, 0x49, 0x10, // subfe r8,r9,r9
                0x55, 0x03, 0x07, 0xFE, // clrlwi r3,r8,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // signed `a >= 5` — `srawi` (0/-1) on the left operand, `rlwinm …,1,31,31`
        // (0/1) on the right, plus CA, summed by one `adde`.
        assert_eq!(
            cmp(Rel::Ge, true, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x7C, 0x6A, 0xFE, 0x70, // srawi r10,r3,31
                0x55, 0x69, 0x0F, 0xFE, // srwi r9,r11,31
                0x7D, 0x0B, 0x18, 0x10, // subfc r8,r11,r3
                0x7C, 0x69, 0x51, 0x14, // adde r3,r9,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // signed `a <= 5` is `5 >= a`, so the two shifts swap which operand they
        // apply to — and, because emission follows source order, also swap
        // positions. Reusing the `>=` order here would be wrong bytes.
        assert_eq!(
            cmp(Rel::Le, true, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x54, 0x6A, 0x0F, 0xFE, // srwi r10,r3,31
                0x7D, 0x69, 0xFE, 0x70, // srawi r9,r11,31
                0x7D, 0x03, 0x58, 0x10, // subfc r8,r3,r11
                0x7C, 0x6A, 0x49, 0x14, // adde r3,r10,r9
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `a >= 5` — CA out of `a - 5` *is* the answer; `subfze` against
        // a preloaded -1 materializes it. `subfc` writes its dead difference back
        // over r11 instead of taking a fresh register.
        assert_eq!(
            cmp(Rel::Ge, false, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x39, 0x40, 0xFF, 0xFF, // li r10,-1
                0x7D, 0x6B, 0x18, 0x10, // subfc r11,r11,r3
                0x7C, 0x6A, 0x01, 0x90, // subfze r3,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `a <= 5` — the only shape whose literal rides in the `subfic`
        // immediate, so three words; and `li r10,-1` is emitted BEFORE the
        // `subfic` even though it takes the lower register number.
        assert_eq!(
            cmp(Rel::Le, false, 5),
            vec![
                0x39, 0x40, 0xFF, 0xFF, // li r10,-1
                0x21, 0x63, 0x00, 0x05, // subfic r11,r3,5
                0x7C, 0x6A, 0x01, 0x90, // subfze r3,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn compare_uncharacterized_relations_fail_closed() {
        use c2_il::Rel;
        // A wide literal needs lis+ori and the extra temp slot it consumes.
        assert!(matches!(
            compare_leaf_text(&c2_il::CompareLeaf {
                param: 0xE309,
                rel: Rel::Gt,
                signed: false,
                k: 70000,
            }, OptMode::Ox),
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
            arg_sources: None,
            params,
            ops,
        }
    }

    #[test]
    fn select_text_add_immediate() {
        // `a + 5` → addi r3,r3,5 ; blr
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(5), IlOp::Add]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
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
            select_text(&f, OptMode::Ox).unwrap(),
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
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x63, 0x00, 0x02, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_sub_immediate_folds_to_addi_neg() {
        // `a - 5` → addi r3,r3,-5 ; blr
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(5), IlOp::Sub]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x63, 0xFF, 0xFB, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_bare_constant_return_is_li() {
        // `return 42;` → addi r3,r0,42 (li) ; blr
        let f = func_with(vec![], vec![IlOp::Lit(42)]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x60, 0x00, 0x2A, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_bare_non_first_parameter_is_one_mr() {
        // `return b;` is `mr r3,r4 ; blr` (`or r3,r4,r4`, opcode 31 / XO 444).
        // Measured across the whole argument file — every word here is read off
        // a reference obj, see `fixtures/cpp/w18_reg_move.cpp`.
        let p = vec![0xE309, 0xE409, 0xE509, 0xE609];
        let sel = |tok: u32| {
            select_text(&func_with(p.clone(), vec![IlOp::Load(tok)]), OptMode::Ox).unwrap()
        };
        // The first parameter is already in r3 and emits nothing at all — the
        // control that keeps this arm from firing on every identity.
        assert_eq!(sel(0xE309), vec![0x4E, 0x80, 0x00, 0x20]);
        assert_eq!(sel(0xE409), vec![0x7C, 0x83, 0x23, 0x78, 0x4E, 0x80, 0x00, 0x20]);
        assert_eq!(sel(0xE509), vec![0x7C, 0xA3, 0x2B, 0x78, 0x4E, 0x80, 0x00, 0x20]);
        assert_eq!(sel(0xE609), vec![0x7C, 0xC3, 0x33, 0x78, 0x4E, 0x80, 0x00, 0x20]);
        // The eighth argument register, r10 — the far end of the file.
        let eight: Vec<u32> = (0..8).map(|i| 0xE309 + i * 0x100).collect();
        assert_eq!(
            select_text(
                &func_with(eight.clone(), vec![IlOp::Load(eight[7])]),
                OptMode::Ox
            )
            .unwrap(),
            vec![0x7D, 0x43, 0x53, 0x78, 0x4E, 0x80, 0x00, 0x20],
            "mr r3,r10 ; blr"
        );
        // The mode does not reach this arm: there is no intermediate to allocate.
        assert_eq!(
            select_text(&func_with(p.clone(), vec![IlOp::Load(0xE409)]), OptMode::O1).unwrap(),
            sel(0xE409)
        );
        // A token that is not a parameter still fails closed — the move needs a
        // source register and there is none.
        assert!(select_text(&func_with(p, vec![IlOp::Load(0x9999)]), OptMode::Ox).is_err());
    }

    #[test]
    fn select_text_rejects_immediate_multiply() {
        // `a * 3` strength-reduces (out of class) — must reject, not mis-emit.
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(3), IlOp::Mul]);
        assert!(matches!(select_text(&f, OptMode::Ox), Err(BackendError::NotImplemented(_))));
    }

    #[test]
    fn select_text_wide_add_immediate_uses_addis_addi() {
        // `a + 70000` → addis r3,r3,1 ; addi r3,r3,4464 ; blr.
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(70000), IlOp::Add]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
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
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x3C, 0x60, 0x00, 0x01, // addis r3,r0,1
                0x60, 0x63, 0x11, 0x70, // ori r3,r3,4464
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    /// The `/O1` allocator, against `/Ox`, on the shape that separates them:
    /// a four-leaf chain with no addition, where `/Ox` gives every intermediate
    /// its own descending register and `/O1` reuses r11 because each
    /// intermediate's predecessor is dead.
    ///
    /// Transcribed from captures of `int f(int a,int b,int c,int d){return a*b*c*d;}`
    /// at `/Ox /GS- /c` and `/O1 /GS- /c` (`docs/OPT_MODE.md` §3.1).
    #[test]
    fn o1_reuses_r11_where_ox_descends() {
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
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x4B, 0x29, 0xD6, // mullw r10,r11,r5   <- descends
                0x7C, 0x6A, 0x31, 0xD6, // mullw r3,r10,r6
                0x4E, 0x80, 0x00, 0x20, // blr
            ],
            "/Ox takes a fresh descending register for a dead intermediate"
        );
        assert_eq!(
            select_text(&f, OptMode::O1).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x6B, 0x29, 0xD6, // mullw r11,r11,r5   <- reuses r11
                0x7C, 0x6B, 0x31, 0xD6, // mullw r3,r11,r6
                0x4E, 0x80, 0x00, 0x20, // blr
            ],
            "/O1 reuses r11 once the predecessor is dead"
        );
    }

    /// A chain that *does* contain an addition already collapses to r11 under
    /// `/Ox`, so the two modes agree on it — the guard against "fixing" `/O1` by
    /// changing what `/Ox` emits.
    #[test]
    fn a_chain_with_an_addition_is_mode_independent() {
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Sub,
                IlOp::Load(0xE609),
                IlOp::Sub,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            select_text(&f, OptMode::O1).unwrap()
        );
    }

    /// The `/O1` comparison spines: same opcodes, operand order and immediates as
    /// `/Ox`, only the temporaries reallocated. Both sides transcribed from the
    /// captures in `docs/CODEGEN_W6_O1.md` (`int f(int a){return a < 5;}`).
    #[test]
    fn a_comparison_leaf_reallocates_temps_under_o1() {
        let cmp = c2_il::CompareLeaf {
            param: 0xE309,
            rel: c2_il::Rel::Lt,
            signed: true,
            k: 5,
        };
        // /Ox descends r10, r9, r8, r7 for the four temps after `li r11,k`.
        assert_eq!(
            compare_leaf_text(&cmp, OptMode::Ox).unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x05, // li     r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc  r10,r11,r3   (dead; CA is the point)
                0x7D, 0x69, 0x1A, 0x38, // eqv    r9,r11,r3
                0x55, 0x28, 0x0F, 0xFE, // rlwinm r8,r9,1,31,31
                0x7C, 0xE8, 0x01, 0x94, // addze  r7,r8
                0x54, 0xE3, 0x07, 0xFE, // rlwinm r3,r7,0,31,31
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
        // /O1 keeps the dead `subfc` fresh — r11 is still live for the `eqv` — and
        // collapses every temp from the `eqv` on, since that is r11's last use.
        assert_eq!(
            compare_leaf_text(&cmp, OptMode::O1).unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x05, // li     r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc  r10,r11,r3
                0x7D, 0x6B, 0x1A, 0x38, // eqv    r11,r11,r3
                0x55, 0x6B, 0x0F, 0xFE, // rlwinm r11,r11,1,31,31
                0x7D, 0x6B, 0x01, 0x94, // addze  r11,r11
                0x55, 0x63, 0x07, 0xFE, // rlwinm r3,r11,0,31,31
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    /// The unsigned `==`/`!=` immediate predicate, which is **not** the carry
    /// spines'. `a == 4294967295u` (stored as `k = -1`) must refuse: c2
    /// materializes the constant and subtracts, where the port used to emit
    /// `addi r11,r3,1` and come out 4 bytes short — in both modes. Meanwhile the
    /// *signed* `a == -1` and the unsigned *carry* spine at the same literal are
    /// both fine and must stay accepted.
    #[test]
    fn unsigned_eq_above_simm16_refuses_but_its_neighbours_do_not() {
        let mk = |rel, signed, k| c2_il::CompareLeaf { param: 0xE309, rel, signed, k };
        for mode in [OptMode::Ox, OptMode::O1] {
            assert!(compare_leaf_text(&mk(c2_il::Rel::Eq, false, -1), mode).is_err());
            assert!(compare_leaf_text(&mk(c2_il::Rel::Ne, false, -5), mode).is_err());
            // signed `== -1` is the ordinary difference spine.
            assert!(compare_leaf_text(&mk(c2_il::Rel::Eq, true, -1), mode).is_ok());
            // unsigned `>` rides the literal in the `subfic` immediate.
            assert!(compare_leaf_text(&mk(c2_il::Rel::Gt, false, -5), mode).is_ok());
            // and small unsigned literals still take the difference spine.
            assert!(compare_leaf_text(&mk(c2_il::Rel::Eq, false, 32767), mode).is_ok());
        }
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
            select_text(&tree4(IlOp::Add, IlOp::Add, IlOp::Mul), OptMode::Ox).unwrap(),
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
            select_text(&tree4(IlOp::Mul, IlOp::Mul, IlOp::Sub), OptMode::Ox).unwrap(),
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
            select_text(&tree4(IlOp::Mul, IlOp::Mul, IlOp::Add), OptMode::Ox).unwrap(),
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
                    select_text(&tree4(op1, op2, IlOp::Mul), OptMode::Ox),
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
                        select_text(&tree4(op1, op2, root), OptMode::Ox),
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
            arg_sources: None,
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
            select_text(&func, OptMode::Ox).unwrap(),
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
            arg_sources: None,
            params: vec![0xE309, 0xE409, 0xE509],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Add,
            ],
        };
        let text = select_text(&func, OptMode::Ox).unwrap();
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

// ---- The per-function selector: ONE dispatch, two emitters -----------------

/// What [`select_function`] produced for one function.
///
/// The variants differ only in what the *caller* still has to do — append a
/// branch, pool a constant, take a different obj shape. Everything that decides
/// **whether a function is in class at all** happens inside
/// [`select_function`], which is the point: before it existed, the packed
/// emitter and the COMDAT emitter each had their own copy of the dispatch, and
/// a diagnostic that wanted to ask "would the port accept this function?" had to
/// grow a third (`docs/GAPS.md` §6, "one fact, one locator").
pub enum Selected {
    /// A complete body. No relocation, no pooled constants.
    Plain(Vec<u8>),
    /// A tail call. The bytes are everything *before* the `b <callee>`; the
    /// caller appends the branch at `text_offset + len` and registers the REL24
    /// there, because the branch encodes its own `.text` offset.
    Tail(Vec<u8>),
    /// A floating-point leaf plus one [`FpConstRef`] per constant reference
    /// site, at offsets relative to the start of this text.
    Float {
        text: Vec<u8>,
        consts: Vec<FpConstRef>,
    },
    /// A framed non-leaf call. It owns its whole obj shape (`.pdata` plus the
    /// compiler label symbols), so the selector hands back only the argument
    /// setup — the bytes between the prologue and the `bl` — and the caller,
    /// which knows the function's `.text` offset, finishes the body. Empty
    /// whenever the call's argument is already the formal in r3, one
    /// `or r3,rN,rN` otherwise.
    Framed { setup: Vec<u8> },
}

/// **The port's per-function instruction selection**, in one place.
///
/// The dispatch order is load-bearing and is the union of the two orders the
/// packed and COMDAT emitters used to carry separately:
///
/// 1. `framed_call` — its own obj shape;
/// 2. `tail_call` — checked **ahead of** the leaf recognizers, so a tail call
///    can never lose its branch to a leaf pattern that happens to match its
///    argument-setup op stream;
/// 3. `empty_body` — a bare `blr`;
/// 4. the FP leaf — its op vocabulary (`Load`/`Lit`/`FpLit` + `+ - * /`) is
///    disjoint from the indirect-load and address leaves' (`LoadInd`/`AddrOf`),
///    so its position relative to them is free; it keeps the packed emitter's;
/// 5. the indirect-load leaf, then the address leaf — exact two-op streams;
/// 6. the comparison leaf — its own branchless spine;
/// 7. otherwise the ordinary arithmetic selector, which refuses whatever it
///    cannot lower.
///
/// `mode` is the per-function optimization mode read from `.ex`; the caller has
/// already refused a TU that mixes modes or carries one this port was not
/// verified against.
pub fn select_function(func: &IlFunction, mode: OptMode) -> Result<Selected, BackendError> {
    if func.framed_call.is_some() {
        // The argument setup, through the same selector the integer tail call
        // uses: `[Load(first formal)]` selects to a bare `blr` (an empty setup,
        // the value is already in r3) and `[Load(other formal)]` to
        // `mr r3,rN ; blr`. Dropping the `blr` leaves exactly the words that go
        // between the prologue and the `bl`.
        let mut setup = select_text(func, mode)?;
        let blr = encode_blr();
        debug_assert!(setup.ends_with(&blr), "select_text always terminates in blr");
        setup.truncate(setup.len() - blr.len());
        return Ok(Selected::Framed { setup });
    }
    if func.tail_call.is_some() {
        // Multi-argument: a register permutation, then the branch.
        if let Some(sources) = &func.arg_sources {
            return Ok(Selected::Tail(permute_args_text(sources)?));
        }
        // A VOID tail call (`void f(){ g(); }`, and the generated empty
        // destructor): no argument to compute, so the setup is empty.
        if func.ops.is_empty() {
            return Ok(Selected::Tail(Vec::new()));
        }
        // An integer tail call: the argument computed into r3. `int_tail_call_text`
        // appends the branch itself, so the setup is its text minus the last word.
        let (mut text, _) = int_tail_call_text(func, 0, mode)?;
        text.truncate(text.len() - 4);
        return Ok(Selected::Tail(text));
    }
    if func.empty_body {
        return Ok(Selected::Plain(encode_blr().to_vec()));
    }
    if let Some(double) = func.float_leaf {
        let (text, consts) = float_leaf_text(func, double)?;
        return Ok(Selected::Float { text, consts });
    }
    if let Some(t) = indirect_load_text(func) {
        return Ok(Selected::Plain(t?));
    }
    if let Some(t) = addr_leaf_text(func) {
        return Ok(Selected::Plain(t?));
    }
    if let Some(t) = store_leaf_text(func) {
        return Ok(Selected::Plain(t?));
    }
    if let Some(cmp) = &func.compare {
        return Ok(Selected::Plain(compare_leaf_text(cmp, mode)?));
    }
    Ok(Selected::Plain(select_text(func, mode)?))
}

/// **Diagnostic: would the port accept this one function?** Runs
/// [`select_function`] — the same dispatch the emitters run, not a copy of it —
/// plus the two gates that only `/Gy` raises.
///
/// This exists to size the **census/gate disagreement** (roadmap #44): the IL
/// parser is where acceptance is supposed to live, so that
/// [`c2_il::IlBundle::function_census`] and `PortC2` cannot disagree about what
/// is in class. Where a refusal has leaked into codegen instead, the census
/// over-claims, and a numerator with an unmeasured error term is not a
/// benchmark. `c2rs gap` runs this over every function the census calls in class
/// and reports the disagreement in the same block as the census.
///
/// Diagnostic only — nothing in the emitter consults it.
pub fn function_gate(
    func: &IlFunction,
    mode: OptMode,
    fn_level_linking: bool,
) -> Result<(), BackendError> {
    match select_function(func, mode)? {
        // A framed non-leaf call under `/Gy` used to refuse here, because its
        // `.pdata` was not modeled per COMDAT. It is now (W-UNW-1): each framed
        // function gets its own `.pdata` COMDAT tied to its `.text` by
        // SELECT_ASSOCIATIVE, so this arm is gone. Leaving it would have made
        // the diagnostic report a refusal for every framed function the emitter
        // actually emits — the disagreement counter wrong in the *under*-claiming
        // direction, which no test would have caught.
        Selected::Float { consts, .. } if fn_level_linking && !consts.is_empty() => {
            Err(out_of_class(
                "pooled floating-point constant under function-level linking (/Gy)",
            ))
        }
        _ => Ok(()),
    }
}

/// Map a function's `.ex` **optimization-settings word** to the mode this port
/// emits under, or refuse.
///
/// One locator: [`crate::PortC2::build`] applies it per function, and the
/// census/gate cross-check applies it to the word [`c2_il::FnCensus::opt_word`]
/// read off the same segment. A diagnostic that guessed `/O1` because the
/// workload compiles `/O1` would silently disagree with the emitter about every
/// `#pragma optimize` function in the corpus.
///
/// One bit of the word is NOT a mode: `0x0100` says the function is a
/// constructor or a destructor ([`c2_il::OPT_WORD_SPECIAL_MEMBER`], measured one
/// flag and one function kind at a time). It is masked off before the whole-word
/// compare, so a destructor's word reads as the mode it actually is — otherwise
/// every constructor and destructor in the corpus is a `codegen-gap` however
/// ordinary its body, which is what kept `A::~A() {}` (a bare `blr`, decoded as
/// `EmptyBody`) out of the emitter. Every other bit is still required to match a
/// word this port was verified against.
pub fn opt_mode_of_word(word: Option<u32>) -> Result<OptMode, BackendError> {
    match c2_il::opt_word_mode(word) {
        Some(c2_il::OptWordMode::Ox) => Ok(OptMode::Ox),
        Some(c2_il::OptWordMode::O1) => Ok(OptMode::O1),
        None => Err(out_of_class(&format!(
            // Reported as the RAW word, not the masked one: the census key has to
            // name what is actually in the file.
            "opt-mode {} : only {:08x} (/Ox, /O2) and {:08x} (/O1) are \
             implemented{}. See docs/OPT_MODE.md.",
            match word {
                Some(v) => format!("{v:08x}"),
                None => "unreadable".to_string(),
            },
            c2_il::OPT_WORD_OX,
            c2_il::OPT_WORD_O1,
            match word.map(|v| v & !c2_il::OPT_WORD_SPECIAL_MEMBER) {
                Some(0x0080_0005) => " — that is /Od",
                Some(0x0080_0004) => " — that is #pragma optimize(\"\", off)",
                _ => "",
            },
        ))),
    }
}

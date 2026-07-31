//! PPC word encoders — one fact per function, no dependencies.
//!
//! Every `encode_*` here turns operands into the four big-endian bytes of one
//! instruction, and nothing else: no `IlFunction`, no gate, no allocation
//! policy. That is why they live together in one alphabetizable file rather
//! than beside the lowerings that call them.
//!
//! The file also exists to make one specific defect impossible. Two branches
//! once landed two `encode_std`s 2,000 lines apart in the old single-file
//! `codegen.rs` and git flagged nothing (`docs/ARCHITECTURE_SEAMS.md` §1,
//! class 4). In one file a duplicate encoder is a compile error, in the same
//! file, immediately.

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
pub(crate) fn encode_ldr(rd: u8, ra: u8, ds: i16) -> [u8; 4] {
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

/// `mr rA, rS` — the `or rA, rS, rS` idiom c2 uses for a register-to-register
/// move (opcode 31, XO 444).
pub fn encode_mr(ra: u8, rs: u8) -> [u8; 4] {
    xo31(rs, ra, rs, 444)
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
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
    fn encode_addi_matches_reference_words() {
        assert_eq!(encode_addi(3, 3, 5), [0x38, 0x63, 0x00, 0x05]); // a+5
        assert_eq!(encode_addi(3, 3, -5), [0x38, 0x63, 0xFF, 0xFB]); // a-5
        assert_eq!(encode_addi(3, 0, 42), [0x38, 0x60, 0x00, 0x2A]); // li r3,42
    }

}

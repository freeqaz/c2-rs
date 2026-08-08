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

/// Encode an **X-form logical / shift** `op rA, rS, rB` — the register-register
/// bitwise and shift family, `lane w-build`.
///
/// **The field order is NOT the one [`encode_add`] uses, and that is the whole
/// reason this is a separate encoder rather than a parameter on that one.** The
/// D-form arithmetic instructions put the *destination* in the RT field at bits
/// 6–10; every instruction below puts the destination in the **RA** field at
/// bits 11–15 and its *source* in RS at 6–10. Encoding `and` through
/// `encode_add`'s layout produces a valid `and` with the destination and the
/// left operand exchanged — bytes that assemble, disassemble and fuzz-match, and
/// compute the wrong thing whenever the two differ.
///
/// Every `xo` below is read off a **transcribed capture**, never inferred:
/// `work/w-build/probe/bits.cod` and `bits2.cod`, at the workload's own
/// `/GR /O1 /Oi /EHsc`. The sixteen captured words are reproduced verbatim by
/// this encoder in [`the_logical_xforms_reproduce_their_captured_words`].
///
/// ```text
///   and  r3,r3,r4    7c632038      xo  28     and  r3,r11,r5   7d632838
///   or   r3,r3,r4    7c632378      xo 444     or   r3,r11,r10  7d635378
///   xor  r3,r3,r4    7c632278      xo 316     xor  r3,r11,r5   7d632a78
///   slw  r3,r3,r4    7c632030      xo  24     slw  r3,r11,r5   7d632830
///   srw  r3,r3,r4    7c632430      xo 536
///   sraw r3,r3,r4    7c632630      xo 792     sraw r11,r3,r4   7c6b2630
/// ```
///
/// `ra` is the DESTINATION, `rs` the left operand, `rb` the right one. The
/// shifts are **non-commutative** in exactly the way [`encode_subf`] warns about
/// — `rs` is the value shifted and `rb` the amount — so the three arguments are
/// named for their roles rather than for their field letters.
fn encode_logical_x(xo: u32, ra_dest: u8, rs_lhs: u8, rb_rhs: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((rs_lhs as u32 & 0x1F) << 21)
        | ((ra_dest as u32 & 0x1F) << 16)
        | ((rb_rhs as u32 & 0x1F) << 11)
        | (xo << 1);
    word.to_be_bytes()
}

/// `and rA, rS, rB` — XO 28. Commutative; captured `7c632038`.
pub fn encode_and(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(28, dest, lhs, rhs)
}

/// `or rA, rS, rB` — XO 444. Commutative; captured `7c632378`.
pub fn encode_or(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(444, dest, lhs, rhs)
}

/// `xor rA, rS, rB` — XO 316. Commutative; captured `7c632278`.
pub fn encode_xor(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(316, dest, lhs, rhs)
}

/// `slw rA, rS, rB` — XO 24. **Non-commutative**: `lhs` is shifted by `rhs`.
///
/// One instruction for both signednesses, and that is measured rather than
/// assumed: `int f(int a,int b){return a<<b;}` and the all-`unsigned` spelling
/// both emit `7c632030`, as does the mixed `int f(int a,unsigned b)`.
pub fn encode_slw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(24, dest, lhs, rhs)
}

/// `srw rA, rS, rB` — XO 536, the **logical** right shift. Captured `7c632430`.
pub fn encode_srw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(536, dest, lhs, rhs)
}

/// `sraw rA, rS, rB` — XO 792, the **arithmetic** right shift. Captured
/// `7c632630`.
///
/// **`sraw` and `srw` differ by one bit of the operand TYPE and by nothing in
/// the IL opcode**, which is the trap this family carries and the reason
/// `parse_expr` refuses a mixed-signedness right shift outright. Probed:
/// `int f(int a, unsigned b){return a>>b;}` is `sraw` and
/// `unsigned f(unsigned a, int b){return a>>b;}` is `srw` — **only the LEFT
/// operand decides**, and both spellings carry the identical IL byte `0A`.
pub fn encode_sraw(dest: u8, lhs: u8, rhs: u8) -> [u8; 4] {
    encode_logical_x(792, dest, lhs, rhs)
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

/// `bclr BO,BI` — a **conditional return**: branch to the link register when the
/// CR bit says so, opcode 19 XO 16, `LK = 0`.
///
/// This is w-rotate's **P2** in one word (`docs/rungs/2026-08-05-w-rotate.md`
/// §3, 46 of 46): a rotation guard branches to the block the loop falls out to,
/// and it **folds to `bclr` exactly when that block is a bare `blr`** — so the
/// guard carries no displacement at all and cannot go stale when the body's
/// length changes. It is the reason a variable-length loop body needs no
/// forward fixup for its entry test.
///
/// Captured: `4d820020` = `bclr 12,2` (branch-if-cr0.EQ to LR), every
/// `TWO`-regime cell of `work/w-varloop/probe.py`. [`encode_blr`] is this word
/// at `BO = `[`BO_ALWAYS`]`, BI = 0`, and the two agree by construction — there
/// is a test.
pub fn encode_bclr(bo: u8, bi: u8) -> [u8; 4] {
    let word: u32 =
        (19 << 26) | ((bo as u32 & 0x1F) << 21) | ((bi as u32 & 0x1F) << 16) | (16 << 1);
    word.to_be_bytes()
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

/// `extsb. rA, rS` — the **record form** of the byte sign-extension, opcode 31
/// XO 954 with `Rc = 1`. It writes **cr0** as a side effect, which is how `c2`
/// closes a **signed**-element sentinel walk: the character the next iteration
/// tests is widened and tested in one instruction, with no `cmplwi` at the
/// bottom of the body at all.
///
/// Captured: `7d6b0775` = `extsb. r11,r11` and `7d2b0775` = `extsb. r11,r9`
/// (`work/w-varloop/probe.py`, every `const char*` cell).
///
/// The signed sibling of [`encode_mr_record`], and a separate function rather
/// than an `rc: bool` on [`encode_extsb`] for the same reason that one is: the
/// two differ in whether a branch may read cr0 after them, and board #188 is
/// what this project already paid for confusing that.
pub fn encode_extsb_record(ra: u8, rs: u8) -> [u8; 4] {
    let mut w = u32::from_be_bytes(xo31(rs, ra, 0, 954));
    w |= 1;
    w.to_be_bytes()
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

/// `rlwimi rA, rS, SH, MB, ME` — rotate left word immediate then mask
/// **INSERT**: primary opcode 20, Rc=0. Unlike [`encode_rlwinm`] this reads
/// `rA` as well as writing it — the bits outside `MB..ME` survive, which is the
/// whole point and the reason W43 can fold a shift and an OR into one word.
pub fn encode_rlwimi(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
    let word: u32 = (20 << 26)
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

/// `rlwinm. rA, rS, SH, MB, ME` — the **record form** of [`encode_rlwinm`]:
/// primary opcode 21 with `Rc = 1`, so the masked result sets `cr0` and **no
/// compare instruction is issued at all**.
///
/// That last clause is the whole reason this encoder exists as its own name
/// rather than as an `rc: bool` parameter on [`encode_rlwinm`]. A caller that
/// reaches for the record form is making a *control-flow* decision — the branch
/// below it reads `cr0` — where a caller of the non-record form is computing a
/// value. Two names keep the two decisions apart at the call site.
///
/// **Pinned to real `c2` output, not derived from a manual.**
/// `clrlwi. r10,r10,31` at offset `0x48` of `_free_osfhnd` is `554a07ff`
/// (`work/w-osfinfo/ref/osfinfo/dis.txt`, the workload's own
/// `/O1 /Oi /EHsc /GR`), and `codegen::osf_handle_guard` asserts that word in
/// place against the whole 152-byte function.
pub fn encode_rlwinm_record(ra: u8, rs: u8, sh: u8, mb: u8, me: u8) -> [u8; 4] {
    let mut w = encode_rlwinm(ra, rs, sh, mb, me);
    w[3] |= 1;
    w
}

/// `clrlwi. rA, rS, N` — keep the low `32 − N` bits **and set cr0**. The
/// `rlwinm. rA,rS,0,N,31` form.
///
/// `N = 31` is the `& 1` test `_free_osfhnd` uses on its `osfile` byte; the
/// parameter is open because the reader derives it from the mask literal the IL
/// carries (`mask + 1` a power of two ⇒ `N = 32 − log2(mask + 1)`), and a class
/// that hardcoded 31 would have a field it could not vary.
pub fn encode_clrlwi_record(ra: u8, rs: u8, n: u8) -> [u8; 4] {
    encode_rlwinm_record(ra, rs, 0, n, 31)
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

/// `mr. rA, rS` — the **record form** of the `or` move, opcode 31 XO 444 with
/// `Rc = 1`. It writes **cr0** as a side effect and is how `c2` closes a
/// sentinel loop: the value the next iteration needs is copied and tested in one
/// instruction, so no `cmplwi` is issued at the bottom of the body at all.
///
/// Captured: `7d4b5379` = `mr. r11,r10` (`?HashString@@YAHPBDH@Z` + 0x20).
///
/// Deliberately its own function rather than a `rc: bool` on [`encode_mr`]: the
/// two differ in *which condition register the branch after them reads* — cr0
/// here against [`CR_COMPARE`]'s cr6 — and that is board #188's defect, which
/// this port has already paid for once.
pub fn encode_mr_record(ra: u8, rs: u8) -> [u8; 4] {
    let mut w = u32::from_be_bytes(xo31(rs, ra, rs, 444));
    w |= 1;
    w.to_be_bytes()
}

/// `mulli rD, rA, SIMM` — primary opcode 7. The whole of `a * k` for the
/// literals `codegen::ptr_walk_loop` admits; see
/// `c2_il::func::body::shapes::ptr_walk_loop::is_mulli_literal` for the 38-cell
/// grid that says which those are and what `c2` emits instead for the rest.
///
/// Captured: `1d0a007f` = `mulli r8,r10,127`.
pub fn encode_mulli(rd: u8, ra: u8, simm: i16) -> [u8; 4] {
    let word: u32 =
        (7 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (simm as u16 as u32);
    word.to_be_bytes()
}

/// `lbzu rD, d(rA)` — load byte and zero with **update**, primary opcode 35.
/// `rA` is written back with the effective address, so the pointer induction is
/// folded into the addressing mode and the loop body carries no separate
/// increment.
///
/// Captured: `8d490001` = `lbzu r10,1(r9)`.
pub fn encode_lbzu(rd: u8, ra: u8, d: i16) -> [u8; 4] {
    let word: u32 =
        (35 << 26) | ((rd as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (d as u16 as u32);
    word.to_be_bytes()
}

/// `divw rD, rA, rB` — signed word divide, opcode 31 XO 491.
/// Captured: `7ce823d6` = `divw r7,r8,r4`.
pub fn encode_divw(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    xo31(rd, ra, rb, 491)
}

/// `divwu rD, rA, rB` — **unsigned** word divide, opcode 31 XO 459.
/// Captured: `7c632396` = `divwu r3,r3,r4` (`work/w-divmod/twigrid.py`, row
/// `u-div-var`, byte-identical at `/O1` and `/Ox`).
pub fn encode_divwu(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    xo31(rd, ra, rb, 459)
}

/// `twi TO, rA, SIMM` — **trap word immediate**, primary opcode 3. The
/// architectural `TO` bits, MSB first, are
/// `[a<b signed, a>b signed, a=b, a<b unsigned, a>b unsigned]`.
///
/// `c2` emits exactly two of them for a signed integer `/` or `%` by a
/// **non-constant** divisor, and they are the two guards the C++ standard makes
/// undefined. Both were read off the encoding rather than paraphrased:
///
/// * **`twi 6, rD, 0`** — `TO = 0b00110` = *equal* ∪ *unsigned less-than*.
///   `rD <u 0` is unsatisfiable, so the instruction traps exactly when the
///   **divisor is zero**. Captured `0cc40000` = `twi 6,r4,0`.
/// * **`twi 5, rX, -1`** — `TO = 0b00101` = *equal* ∪ *unsigned greater-than*.
///   `rX >u 0xFFFFFFFF` is unsatisfiable, so it traps exactly when `rX == -1`,
///   and `rX` is `andc(divisor, rotlwi(dividend,1) - 1)`:
///   `rotlwi(n,1) - 1` is `0` **iff** `n == INT_MIN` (`0x80000000` rotates to
///   `1`), and `andc(d, 0)` is `d`, so `rX == -1` **iff**
///   `dividend == INT_MIN && divisor == -1`. That is the `INT_MIN / -1`
///   overflow guard, and the three-instruction predicate ahead of it is its
///   whole computation. Captured `0ca6ffff` = `twi 5,r6,-1`.
///
/// A **non-zero constant** divisor emits neither — `c2` decides both guards
/// statically (`work/w-hash/divgrid.py`, rows `s-mod-k7`/`s-div-k7`;
/// `work/w-divmod/twigrid.py` re-runs it over **24** literal cells covering both
/// signs, both signednesses, `INT_MIN`, `INT_MAX`, the `simm16` cliff, and the
/// same values reached through a `const` local, a namespace-scope `const` and an
/// enumerator) — and an **unsigned** divide emits only the first (`u-div-var`,
/// `u-mod-var`), because the overflow case cannot arise.
///
/// **There is a THIRD `TO`, and it is not a guard.** A divisor that is a
/// compile-time **zero** emits no division at all and a bare
///
/// * **`twi 7, r0, 0`** — `TO = 0b00111` = *equal* ∪ *unsigned less-than* ∪
///   *unsigned greater-than*, which is a tautology over the unsigned order, so
///   the instruction traps **unconditionally**. Captured `0ce00000`, and the
///   operand register is **`r0`** — not the dividend, not the divisor, because
///   the trap does not read anything.
///
/// Seven cells produce it and they are all the same value by different routes
/// (`a%0`, `a/0`, `a%0u`, `a/0u`, a `const int k=0`, a namespace-scope `const`,
/// an enumerator). `TO = 7` is *not* emitted for any other divisor, and the
/// grid observed no fourth value across 161 cells. None of this is shipped —
/// `div_mod_leaf` refuses every constant divisor — but the `TO` axis is
/// recorded here so a later rung does not rediscover it as an anomaly.
pub fn encode_twi(to: u8, ra: u8, simm: i16) -> [u8; 4] {
    let word: u32 =
        (3 << 26) | ((to as u32 & 0x1F) << 21) | ((ra as u32 & 0x1F) << 16) | (simm as u16 as u32);
    word.to_be_bytes()
}

// ---- W8: the conditional-branch family ------------------------------------
//
// `docs/CFG_SHAPE.md` §3.1 tabulates four forms; three of them are here and the
// fourth (the external `b`/`bl`) is [`super::calls::encode_tail_branch`]'s,
// because it encodes a section offset rather than a displacement and takes a
// relocation. **They are the same opcode.** An emitter that treats every `b`
// alike corrupts one of the two (§3.3, board #191), which is why the two
// encoders are deliberately not merged.

/// The condition-register field an explicit compare feeding a branch writes.
///
/// **cr6, and it is REUSED rather than allocated** — `?b_ifn` writes cr6 three
/// times in one body, each branch consuming its own before the next is issued
/// (`docs/CFG_SHAPE.md` §3.2). It is a named constant and not a literal `6`
/// because the *other* producer is different: a record-form instruction such as
/// `addic.` writes **cr0**, and c2 branches on cr0 there without an intervening
/// compare. A lowering that hard-codes `BI = 4*6 + bit` emits `409a…` where the
/// obj has `4082…` for every decrement-and-test loop — board #188, and the
/// reason this constant exists to be *passed in* the day a record-form producer
/// is admitted.
pub const CR_COMPARE: u8 = 6;

/// `BO` for "branch if the CR bit is SET".
pub const BO_TRUE: u8 = 12;
/// `BO` for "branch if the CR bit is CLEAR".
pub const BO_FALSE: u8 = 4;
/// `BO` for "branch always" — what makes `bclr` a plain `blr`.
pub const BO_ALWAYS: u8 = 20;

/// The bit within a CR field, by relation: LT=0, GT=1, EQ=2, SO=3.
pub const CR_BIT_LT: u8 = 0;
pub const CR_BIT_GT: u8 = 1;
pub const CR_BIT_EQ: u8 = 2;

/// `BI` = `4*crf + bit`.
pub fn cr_bi(crf: u8, bit: u8) -> u8 {
    4 * (crf & 7) + (bit & 3)
}

/// **The architectural reach of a `bc`**: `BD` is a signed 14-bit field scaled
/// by 4, so ±32764 bytes.
///
/// Measured, not assumed. `docs/CFG_SHAPE.md` §3.3.1 swept the displacement and
/// found c2 emitting a direct `bne` at **+32628** and the two-instruction
/// expansion — invert the condition, branch over an unconditional `b` — at
/// **+34148**. The switch is at the limit with **no slack**: c2 uses the full
/// field before expanding.
pub const BC_MAX_DISP: i32 = 32764;

/// Encode `bc BO,BI,<target>` — primary opcode 16, `AA=0`, `LK=0`.
///
/// `disp` is **self-relative**: `target_offset − branch_offset`, not relative
/// to the section start (`docs/CFG_SHAPE.md` §3.3). It carries **no
/// relocation**; `pa.cpp`'s seven code sections all report `nrel = 0` despite
/// six of them containing a branch.
///
/// Returns `None` past [`BC_MAX_DISP`], where the expansion is required. The
/// caller must not truncate: a truncated `BD` is a legal-looking branch to the
/// wrong place, which is the fuzzy-invisible failure class
/// `docs/CODEGEN_PPC_MVP.md` warns about.
pub fn encode_bc(bo: u8, bi: u8, disp: i32) -> Option<[u8; 4]> {
    if disp % 4 != 0 || !(-BC_MAX_DISP - 4..=BC_MAX_DISP).contains(&disp) {
        return None;
    }
    let word: u32 = 0x4000_0000
        | ((bo as u32 & 0x1F) << 21)
        | ((bi as u32 & 0x1F) << 16)
        | (disp as u32 & 0xFFFC);
    Some(word.to_be_bytes())
}

/// The largest displacement an unconditional `b` reaches: `LI` is a signed
/// 24-bit field scaled by 4.
pub const B_MAX_DISP: i32 = 0x01FF_FFFC;

/// Encode an **intra-section** unconditional branch `b` — primary opcode 18,
/// `AA=0`, `LK=0` — carrying its **true self-relative displacement** and taking
/// **no relocation**.
///
/// **This is board #191, and it is the same opcode as [`encode_tail_branch`].**
/// The two are different encodings of one instruction and the discriminator is
/// *where the target lives*, not what the branch is:
///
/// ```text
///   48000008   intra-section: LI is the real displacement, nrel = 0
///   4bffffec   external:      LI is −(own .text offset), plus a REL24
/// ```
///
/// A fixup pass that treats every `b` alike corrupts one of the two
/// (`docs/CFG_SHAPE.md` §3.3), which is why they are two functions here rather
/// than one with a flag.
///
/// **It has been written once before and deleted.** W10 built it for the `else`
/// arm's join branch, found that arm's block layout to be mode-dependent on a
/// threshold that is a c2 cost model, and removed the encoder rather than ship a
/// code path the oracle had never graded (w-frame row **F-c**). It comes back
/// with W11's guarded early return, whose `b` targets the **epilogue** — a block
/// that exists in both modes and whose length is a constant of the frame class,
/// so there is no threshold to fit.
///
/// `disp` is `target_offset − branch_offset`. Returns `None` for a misaligned or
/// out-of-range displacement rather than truncating: a truncated `LI` is a
/// legal-looking branch to the wrong place.
pub fn encode_b_intra(disp: i32) -> Option<[u8; 4]> {
    if disp % 4 != 0 || !(-B_MAX_DISP - 4..=B_MAX_DISP).contains(&disp) {
        return None;
    }
    let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC);
    Some(word.to_be_bytes())
}

/// Encode `cmpwi crf,rA,SIMM` — the **signed** immediate compare, opcode 11.
pub fn encode_cmpwi(crf: u8, ra: u8, simm: i16) -> [u8; 4] {
    let word: u32 = (11 << 26)
        | ((crf as u32 & 7) << 23)
        | ((ra as u32 & 0x1F) << 16)
        | (simm as u16 as u32);
    word.to_be_bytes()
}

/// Encode `cmplwi crf,rA,UIMM` — the **unsigned** immediate compare, opcode 10.
///
/// Which of the two a body gets comes from the shared operand TYPE triple at the
/// comparison and from nothing else: the relational opcodes are sign-agnostic,
/// and a pointer null-check is therefore an *unsigned* compare
/// (`docs/CFG_SHAPE.md` §3.2 — `?MemFree` and both `Pool.cpp` functions emit
/// `cmplwi`).
pub fn encode_cmplwi(crf: u8, ra: u8, uimm: u16) -> [u8; 4] {
    let word: u32 = (10 << 26)
        | ((crf as u32 & 7) << 23)
        | ((ra as u32 & 0x1F) << 16)
        | (uimm as u32);
    word.to_be_bytes()
}

/// Encode `cmpw crf,rA,rB` — the **signed register-register** word compare,
/// X-form: primary opcode 31, extended 0, `L = 0`.
///
/// [`encode_cmpwi`] is its immediate sibling and existed first because every
/// comparison the port had lowered until now put a literal on one side. The
/// register-register form is what a loop test against a *loaded* value needs,
/// and board **#1105** names its absence as the first of `Primes.cpp`'s
/// refusals.
///
/// **Pinned to real `c2` output, not derived from a manual.** `cmpw cr6,r10,r3`
/// at offset `0x14` of `?NextHashPrime@@YAHH@Z` is `7f0a1800`
/// (`work/w-loop/Primes_b.obj`, `/O1 /Oi /EHsc`, the workload's own flags), and
/// `codegen::frontier_bytes` (`cfg(test)`) asserts that word in place against the whole
/// 64-byte function.
///
/// **This encoder has no accept-path caller and that is deliberate.** Nothing in
/// [`super::select`] reaches it; the port still returns `NotImplemented` on
/// every body that would need it. It is an ISA transcription in a file of ISA
/// transcriptions, graded by a byte c2 really emitted — which is the distinction
/// board **#278** drew when it *deleted* `bss_deferred_layout`: that item's tests
/// asserted a **layout rule** that had been superseded, where these assert a
/// **fixed instruction encoding** that cannot be.
pub fn encode_cmpw(crf: u8, ra: u8, rb: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((crf as u32 & 7) << 23)
        | ((ra as u32 & 0x1F) << 16)
        | ((rb as u32 & 0x1F) << 11);
    word.to_be_bytes()
}

/// Encode `cmplw crf,rA,rB` — the **unsigned** register-register compare:
/// X-form, primary opcode 31, extended **32**, where [`encode_cmpw`]'s signed
/// form is extended 0.
///
/// **This encoder did not exist, and two published rung tables disagreed about
/// whether it did.** `w-extdata` §2 priced `osfinfo`'s missing encoders at two;
/// `w-undname` §5 corrected that to one on the ground that "`encode_cmplw`
/// already exists". It does not — what exists is [`encode_cmpw`] (extended 0,
/// signed) and [`encode_cmplwi`] (primary opcode 10, immediate), and neither
/// produces this word. The original count of two was right. Recorded here
/// rather than only in a rung because the next lane to read that table will
/// read this file too.
///
/// **Pinned to real `c2` output, not derived from a manual.**
/// `cmplw cr6,r3,r11` at offset `0x1c` of `_free_osfhnd` is `7f035840`
/// (`work/w-osfinfo/ref/osfinfo/dis.txt`, the workload's own
/// `/O1 /Oi /EHsc /GR`), and `codegen::osf_handle_guard` asserts that word in
/// place against the whole 152-byte function.
///
/// The signed/unsigned split is **not** cosmetic here: `_free_osfhnd` tests
/// `fh >= 0` with the signed immediate form two words earlier and
/// `fh < _nhandle` with this one, in the same body, on the same operand. A
/// class that used one form for both emits the right program with one wrong
/// word and every branch still resolving — `docs/GAPS.md` §6.
pub fn encode_cmplw(crf: u8, ra: u8, rb: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((crf as u32 & 7) << 23)
        | ((ra as u32 & 0x1F) << 16)
        | ((rb as u32 & 0x1F) << 11)
        | (32 << 1);
    word.to_be_bytes()
}

/// Encode `lwzx rD,rA,rB` — load word, **indexed**: X-form, primary opcode 31,
/// extended 23.
///
/// The scaled-index addressing mode `base[i]`: c2 emits `slwi rT,rI,2` (an
/// [`encode_rlwinm`] the port already has) and then this. It is the second and
/// last instruction in `Primes.cpp`'s 64 bytes with no encoder — see
/// `codegen::frontier_bytes` (`cfg(test)`) for the count that statement comes from.
///
/// **Pinned to real `c2` output**: `lwzx r10,r10,r9` at `0x24` is `7d4a482e`
/// and `lwzx r3,r11,r9` at `0x38` is `7c6b482e`, both from
/// `work/w-loop/Primes_b.obj`. Two distinct cells, so the `rD` and `rA` fields
/// are separated by the pins rather than only by the formula.
///
/// Same accept-path caveat as [`encode_cmpw`], for the same reason.
pub fn encode_lwzx(rd: u8, ra: u8, rb: u8) -> [u8; 4] {
    let word: u32 = (31 << 26)
        | ((rd as u32 & 0x1F) << 21)
        | ((ra as u32 & 0x1F) << 16)
        | ((rb as u32 & 0x1F) << 11)
        | (23 << 1);
    word.to_be_bytes()
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

    /// `bclr` and `extsb.` against the words real `c2` emits for a signed
    /// sentinel walk (`work/w-varloop/probe.py`, every TWO-regime cell).
    ///
    /// **`blr` is `bclr` at `BO_ALWAYS`, `BI = 0`** — asserted rather than
    /// asserted-in-prose, because [`encode_blr`] is a hard-coded constant and
    /// [`encode_bclr`] is computed, and two spellings of one instruction that
    /// nothing compares are two chances to be wrong about it.
    #[test]
    fn encode_bclr_and_extsb_record_match_reference_words() {
        assert_eq!(encode_bclr(BO_TRUE, cr_bi(0, CR_BIT_EQ)), [0x4D, 0x82, 0x00, 0x20]);
        assert_eq!(encode_bclr(BO_ALWAYS, 0), encode_blr());
        // `extsb. r11,r11` (the entry test) and `extsb. r11,r9` (the record
        // form) — the two spellings the loop emits, and the Rc bit is the whole
        // difference from `encode_extsb`.
        assert_eq!(encode_extsb_record(11, 11), [0x7D, 0x6B, 0x07, 0x75]);
        assert_eq!(encode_extsb_record(11, 9), [0x7D, 0x2B, 0x07, 0x75]);
        assert_eq!(u32::from_be_bytes(encode_extsb(11, 11)) | 1,
                   u32::from_be_bytes(encode_extsb_record(11, 11)));
        // The record form writes cr0 and the plain form does not: a branch may
        // read the CR after one and not the other (board #188).
        assert_eq!(u32::from_be_bytes(encode_extsb(11, 11)) & 1, 0);
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


    #[test]
    fn w8_branch_words_match_the_reference_obj() {
        // `?MemFree@NUISPEECH@@YAXPAX0K@Z`, docs/CFG_SHAPE.md §4.1/§3.1's worked
        // example: BO=4 (branch-if-clear), BI=4*6+2=26 (cr6's EQ bit), BD=+16.
        assert_eq!(cr_bi(CR_COMPARE, CR_BIT_EQ), 26);
        assert_eq!(encode_bc(BO_FALSE, 26, 16), Some([0x40, 0x9A, 0x00, 0x10]));
        // `?MemAlloc`, same body one word shorter.
        assert_eq!(encode_bc(BO_FALSE, 26, 12), Some([0x40, 0x9A, 0x00, 0x0C]));
        // §3.4's `?b_ifelse`/`?d_early` rows, the other sense.
        assert_eq!(encode_bc(BO_TRUE, 26, 8), Some([0x41, 0x9A, 0x00, 0x08]));
        assert_eq!(encode_bc(BO_TRUE, 26, 12), Some([0x41, 0x9A, 0x00, 0x0C]));
        // §3.7a's `?c_forcall` back edge: BO=12, BI=24 (LT), BD=-20.
        assert_eq!(encode_bc(BO_TRUE, 24, -20), Some([0x41, 0x98, 0xFF, 0xEC]));
    }

    #[test]
    fn a_branch_past_the_field_refuses_rather_than_truncating() {
        // §3.3.1 bracketed the switch between +32628 (direct) and +34148
        // (expanded), i.e. at the architectural limit with no slack. A
        // truncated `BD` is a legal-looking branch to the wrong place, so the
        // encoder returns None and the caller refuses.
        assert!(encode_bc(BO_FALSE, 26, 32628).is_some());
        assert!(encode_bc(BO_FALSE, 26, BC_MAX_DISP).is_some());
        assert!(encode_bc(BO_FALSE, 26, BC_MAX_DISP + 4).is_none());
        assert!(encode_bc(BO_FALSE, 26, 34148).is_none());
        // Not word-aligned: not a branch target at all.
        assert!(encode_bc(BO_FALSE, 26, 6).is_none());
    }

    #[test]
    fn w8_compare_words_match_the_reference_obj() {
        // §3.2's witness rows.
        assert_eq!(encode_cmplwi(CR_COMPARE, 3, 0), [0x2B, 0x03, 0x00, 0x00]); // ?MemFree
        assert_eq!(encode_cmplwi(CR_COMPARE, 11, 0), [0x2B, 0x0B, 0x00, 0x00]); // ?mmioGetInfo
        assert_eq!(encode_cmpwi(CR_COMPARE, 3, 0), [0x2F, 0x03, 0x00, 0x00]); // ?b_ifn
        assert_eq!(encode_cmpwi(CR_COMPARE, 3, 7), [0x2F, 0x03, 0x00, 0x07]); // ?d_switch
        assert_eq!(encode_cmpwi(CR_COMPARE, 31, 0), [0x2F, 0x1F, 0x00, 0x00]); // ?d_cont
    }

    /// **W-OSFINFO — `cmplw` against the byte real `c2` emitted**, plus the
    /// separation from the three compare encoders that already existed.
    ///
    /// The separation is the point rather than the value: two published rung
    /// tables disagreed about whether this encoder existed, and the reason the
    /// wrong one was believable is that `cmpw`, `cmplwi` and `cmplw` are one
    /// letter apart in the name and one field apart in the word.
    #[test]
    fn w_osfinfo_cmplw_matches_the_reference_obj_and_is_none_of_its_neighbours() {
        // `_free_osfhnd` +0x1c, `work/w-osfinfo/ref/osfinfo/dis.txt`.
        assert_eq!(encode_cmplw(CR_COMPARE, 3, 11), [0x7F, 0x03, 0x58, 0x40]);
        // …and the signed register form two words of the ISA away, which this
        // body does NOT use here — extended 0 against extended 32.
        assert_eq!(encode_cmpw(CR_COMPARE, 3, 11), [0x7F, 0x03, 0x58, 0x00]);
        assert_ne!(encode_cmplw(CR_COMPARE, 3, 11), encode_cmpw(CR_COMPARE, 3, 11));
        // …and the immediate form, which is a different primary opcode.
        assert_ne!(encode_cmplw(CR_COMPARE, 3, 11), encode_cmplwi(CR_COMPARE, 3, 11));
        // The `rB` field is separated from `rA` by a second pin.
        assert_eq!(encode_cmplw(CR_COMPARE, 11, 3), [0x7F, 0x0B, 0x18, 0x40]);
    }

    /// **W-OSFINFO — the record form of `rlwinm` against the byte real `c2`
    /// emitted**, and the one-bit separation from the non-record form.
    ///
    /// A dropped `Rc` bit is the failure this pins: the masked value would be
    /// identical, `cr0` would hold whatever the last instruction left there, and
    /// the `bt 2` below it would branch on a stale bit. The program is wrong and
    /// the obj still links.
    #[test]
    fn w_osfinfo_clrlwi_record_matches_the_reference_obj() {
        // `_free_osfhnd` +0x48, `work/w-osfinfo/ref/osfinfo/dis.txt`.
        assert_eq!(encode_clrlwi_record(10, 10, 31), [0x55, 0x4A, 0x07, 0xFF]);
        // The non-record form of the same mask differs in exactly the Rc bit.
        assert_eq!(encode_clrlwi31(10, 10), [0x55, 0x4A, 0x07, 0xFE]);
        assert_eq!(encode_clrlwi_record(10, 10, 31)[3] & 1, 1);
        assert_eq!(encode_clrlwi31(10, 10)[3] & 1, 0);
        // The mask width is a parameter, not a constant: `& 3` is `clrlwi. ,30`.
        assert_eq!(encode_clrlwi_record(10, 10, 30), encode_rlwinm_record(10, 10, 0, 30, 31));
        // …and the *non*-record `rlwinm` this body also emits, so the two
        // spellings of the same instruction family are pinned side by side:
        // `clrlwi r11,r3,27` (+0x34) and `slwi r9,r11,2` (+0x2c).
        assert_eq!(encode_rlwinm(11, 3, 0, 27, 31), [0x54, 0x6B, 0x06, 0xFE]);
        assert_eq!(encode_rlwinm(9, 11, 2, 0, 29), [0x55, 0x69, 0x10, 0x3A]);
    }

    /// Every word below is TRANSCRIBED from `work/w-build/probe/bits.cod` /
    /// `bits2.cod` — c2's own `/FAsc` listing at the workload's flags — and is
    /// checked against the encoder rather than against the table that produced
    /// it. That is the discipline `expr_opcode_name`'s header states: a value
    /// derived from the thing it validates checks nothing.
    ///
    /// The sixteen rows deliberately include every one that distinguishes the
    /// **RA-destination** X-form layout from [`encode_add`]'s RT-destination
    /// one — `and r11,r3,r4` and `and r3,r11,r5` differ in exactly the two
    /// fields that would swap.
    #[test]
    fn the_logical_xforms_reproduce_their_captured_words() {
        let w = |b: [u8; 4]| u32::from_be_bytes(b);
        // and — `a & b`, and the three-address chain `a & b & c`.
        assert_eq!(w(encode_and(3, 3, 4)), 0x7c63_2038);
        assert_eq!(w(encode_and(11, 3, 4)), 0x7c6b_2038);
        assert_eq!(w(encode_and(3, 11, 5)), 0x7d63_2838);
        assert_eq!(w(encode_and(10, 5, 6)), 0x7caa_3038);
        assert_eq!(w(encode_and(3, 11, 10)), 0x7d63_5038);
        // or
        assert_eq!(w(encode_or(3, 3, 4)), 0x7c63_2378);
        assert_eq!(w(encode_or(11, 3, 4)), 0x7c6b_2378);
        assert_eq!(w(encode_or(3, 11, 5)), 0x7d63_2b78);
        assert_eq!(w(encode_or(3, 11, 10)), 0x7d63_5378);
        // xor
        assert_eq!(w(encode_xor(3, 3, 4)), 0x7c63_2278);
        assert_eq!(w(encode_xor(3, 11, 5)), 0x7d63_2a78);
        // slw / srw / sraw
        assert_eq!(w(encode_slw(3, 3, 4)), 0x7c63_2030);
        assert_eq!(w(encode_slw(3, 11, 5)), 0x7d63_2830);
        assert_eq!(w(encode_srw(3, 3, 4)), 0x7c63_2430);
        assert_eq!(w(encode_sraw(3, 3, 4)), 0x7c63_2630);
        assert_eq!(w(encode_sraw(11, 3, 4)), 0x7c6b_2630);
    }

    /// The layout hazard, stated as a test rather than as a comment.
    ///
    /// `and r11, r3, r4` and `and r3, r11, r4` are DIFFERENT instructions, and
    /// an encoder that used [`encode_add`]'s RT-destination field order would
    /// produce the second when asked for the first. The bytes are valid either
    /// way, so nothing downstream — not `fuzzy%`, not a disassembler — would
    /// flag it; only a byte compare against c2 would, and only on a body where
    /// the destination and the left operand happen to differ.
    #[test]
    fn the_logical_destination_field_is_ra_and_not_rt() {
        assert_ne!(encode_and(11, 3, 4), encode_and(3, 11, 4));
        // What the WRONG layout would have produced for `and r11,r3,r4`, spelled
        // out: RT=11 at bits 6-10, RA=3 at 11-15 — which is `and r3,r11,r4`.
        let wrong = (31u32 << 26) | (11 << 21) | (3 << 16) | (4 << 11) | (28 << 1);
        assert_eq!(wrong.to_be_bytes(), encode_and(3, 11, 4));
        assert_ne!(wrong.to_be_bytes(), encode_and(11, 3, 4));
        // …and the captured byte says which one c2 emits for `a & b & c`'s first
        // instruction: `7c6b2038`, this encoder's `encode_and(11, 3, 4)`.
        assert_eq!(u32::from_be_bytes(encode_and(11, 3, 4)), 0x7c6b_2038);
    }

    /// `sraw` and `srw` are one IL opcode apart from each other by NOTHING —
    /// the distinction lives in the operand type — so the two encoders must not
    /// be interchangeable and the test says so with the captured pair.
    #[test]
    fn the_two_right_shifts_are_different_instructions() {
        assert_ne!(encode_sraw(3, 3, 4), encode_srw(3, 3, 4));
        assert_eq!(u32::from_be_bytes(encode_sraw(3, 3, 4)), 0x7c63_2630); // int
        assert_eq!(u32::from_be_bytes(encode_srw(3, 3, 4)), 0x7c63_2430); // unsigned
    }
}

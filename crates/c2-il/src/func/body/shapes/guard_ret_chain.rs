//! **W-IFN — a framed guard chain whose arms are `return K` and whose spine is
//! a block copy.**
//!
//! ```c
//!   R f(A a, B b, C c) {
//!       if (a == 0) return K0;          // both guards on a POINTER formal
//!       if (b == 0) return K1;
//!       memcpy(<one of them>, <the other>, N);
//!       // sub-shape S only:
//!       M *m = (M *)a;
//!       if (m->hi < m->lo) m->hi = m->lo;
//!       return 0;
//!   }
//! ```
//!
//! This is `src/xdk/nuispeech/mmio.cpp`'s `mmioGetInfo` (sub-shape **G**) and
//! `mmioSetInfo` (sub-shape **S**) — two of the three blocked bodies of the
//! frontier's **top byte-fraction row**, `ABC--`, where function bytes are the
//! whole remaining distance. The third, `mmioClose`, is **not** this class; see
//! §"what this refuses" and `work/w-ifn/MMIO_PRICE.md`.
//!
//! ## Why a TRANSCRIPTION and not a general `cflow-if-2`/`cflow-if-n` lowering
//!
//! The same argument [`super::osf_handle_guard`], [`super::alloc_init_or_fail`],
//! [`super::guard_chain_shared_tail`] and [`super::if_call_join`] make, and it is
//! `docs/ARCHITECTURE_SEAMS.md` §7's. What ships here is **twenty-one and
//! twenty-seven words of two named function classes, `/O1` only**,
//! `NotImplemented` outside. **Accepting these shapes is not a claim about
//! `cflow-if-2` or `cflow-if-n` as classes**, and
//! `c2_harness::gap::factors::PORT_CFG_CLASSES` is not widened for them.
//!
//! ## The facts the READER has to pin
//!
//! Read off the real obj at the workload's own flags
//! (`work/w-ifn/ref/mmio.dump.txt`) and decoded token by token in
//! `work/w-ifn/probe/mmio_ex.txt`, both committed **before** this file was
//! written. The emitted words are in `c2_core::codegen::guard_ret_chain`'s
//! module doc; what belongs here is:
//!
//! 1. **Both guard operands must be POINTERS.** c2 emits `cmplwi` — unsigned —
//!    for a pointer and `cmpwi` for an `int`, four bytes apart in one field
//!    (`work/w-ifn/probe/blkorder.cpp` cell `b1` is `2f030000` where this class
//!    is `2b030000`). The type is the only thing in the stream that says which,
//!    so it is required rather than decoded and discarded.
//! 2. **The copy's destination is the LAST argument in the stream and its source
//!    the second-to-last**, because c2 emits an argument list in reverse. Both
//!    witnesses confirm it in opposite directions: `mmioGetInfo` copies formal 0
//!    into formal 1 and `mmioSetInfo` copies formal 1 into formal 0, and the two
//!    streams differ exactly by swapping those two positions.
//! 3. **Which formal is the destination decides the whole register plan**, so it
//!    is not a field of one shape but the discriminator between two. `dst == 1`
//!    is a SWAP through r11 with no saved GPR; `dst == 0` is a park in r31 with
//!    one. There are two witnesses and they are two plans — `w-blockir` board
//!    #2306 registered one rule for its walker and one for its park and the
//!    answer was three constants both times, so this class does not posit a
//!    rule it has two points for.
//! 4. **The copy length must be at or above the measured expansion step.** 25
//!    cells at `/O1 /Oi` (`work/w-ifn/probe/mcpy.cpp`): `n <= 5` expands to
//!    loads and stores, every `n >= 6` is a `bl memcpy`. Below the step the port
//!    would emit a call c2 does not, which links.
//! 5. **The clamp compares two LOADED values and stores through the same base**,
//!    and its two member offsets must be DISTINCT — equal offsets are a compare
//!    of a value with itself, which c2 folds, and this class has not been graded
//!    on the folded form.
//! 6. **Every label distinct.** Two aliasing labels are one block, and every
//!    displacement after the alias would be right for a program this is not.
//!
//! ## The fence
//!
//! * **`/O1` only, asked FIRST, in the PARSER.** Board **#1638**, which has
//!   fired twice. A mode clause that lived only in the emitter would make the
//!   census count this body in class while `PortC2` refused it;
//!   `census_gate.rs` is the cross-check. The clause itself is this family's —
//!   a block reached from several places tail-duplicates above `/O1` on a
//!   threshold W10 bracketed and did not fit (board row X-b), and the
//!   materialised common epilogue is exactly that shape.
//! * **Exactly THREE formals and no `this`.** `parse_params` prepends the `this`
//!   token when the pre-body region binds one and REFUSES when the binding is
//!   undetermined, so "no `this`" is an established fact and not a count. Both
//!   witnesses take three; a different arity moves the argument registers and
//!   nothing here has been graded on it.
//! * **Exactly TWO guards.** `work/w-ifn/probe/blkorder.cpp` cell `b3` shows
//!   what a third looks like — one more four-word block, in source order — and
//!   this class has not been graded on one, so it refuses.
//! * **Both guards test `== 0`** and return a literal that fits `simm16`, since
//!   the arm is a `li r3,<K>`.
//! * **The two guards must test DIFFERENT formals, in index order.** Two tests
//!   of one formal is one compare in c2's hands.
//!
//! ## What this refuses, and why `mmioClose` is not in it
//!
//! `mmioClose` is the third blocked body of the same TU and it is a different
//! problem, not a longer one. It needs three mechanisms this class does not
//! have and this lane did not build: an indirect call through a loaded member
//! (`lwz`/`mtctr`/`bctrl`), an **elided** call — `mmioSetBuffer(hmmio,0,0,0)` is
//! in the IL and the obj carries no branch for it — and a park of a formal into
//! **r5, a volatile, across a `bl`**, which is only correct because c2 knows the
//! same-TU callee does not clobber it (`work/w-ifn/probe/park.cpp` p1/p2/p4:
//! replacing that callee with an external one moves the park to r30 and grows
//! the frame by 16 bytes). All three are named in `work/w-ifn/MMIO_PRICE.md`.

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_opt_stmt_marker, is_ptr4_kind, read_token_var, read_type,
    read_varint,
};
use crate::func::{GuardRetChain, GuardRetGuard, GuardRetSpine};

/// The `40` intrinsic selector for `memcpy`. Capture-verified: the selector
/// table in [`crate::func::body::expr_opcode_name`]'s doc lists **172** for
/// `memcpy`, and both witnesses carry `33 <int> 80 ac 00 00 00` — the varint
/// escape form of 172 — immediately before their `40`.
/// PROV[O] 172, from the intrinsic-selector table in `expr_opcode_name`'s doc, and both witnesses carry `33 <int> 80 ac 00 00 00` — the varint escape form of 172 — immediately before their `40`.
pub(crate) const MEMCPY_SELECTOR: i32 = 172;

/// The lowest copy length c2 lowers to a CALL at `/O1 /Oi`. Measured, 25 cells:
/// `work/w-ifn/probe/mcpy.cpp`. `c2_core::codegen::guard_ret_chain` re-asserts
/// the same window, and that module's test is what stops the two drifting.
/// PROV[O] the lowest copy length c2 lowers to a CALL at `/O1 /Oi`, measured over 25 cells (`work/w-ifn/probe/mcpy.cpp`) — a window with cells on both sides of the boundary, so no other value is consistent. `c2_core::codegen::guard_ret_chain` re-asserts it and a test stops the two drifting.
pub(crate) const MEMCPY_CALL_STEP: i32 = 6;

/// The lexical depth the body sits at when `parse_segment_shape` dispatches:
/// `eat_scopes` has already taken the body's own `53` and the first `if`'s.
/// PROV[O] the lexical depth the body sits at when `parse_segment_shape` dispatches, read off captures: `eat_scopes` has already taken the body's own `53` and the first `if`'s.
pub(crate) const GUARD_ENTRY_DEPTH: u8 = 3;

/// `29 <tok>` — a label definition.
pub(crate) fn eat_label(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x29) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `<op> <tok>` for a transfer opcode. Returns the target label.
pub(crate) fn eat_transfer(seg: &[u8], p: &mut usize, op: u8, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, op) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume any TYPE and return its two discriminating bytes.
pub(crate) fn eat_any_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) => {
            *p += w;
            Ok((tag, kind))
        }
        None => Err(blk(seg, *p, what)),
    }
}

/// `B9 <tok> <TYPE>` — a value read. Returns the token and whether the type is a
/// width-4 pointer, because for this class that decides the compare form.
pub(crate) fn eat_load(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, bool), Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    let (tag, kind) = eat_any_type(seg, p, what)?;
    Ok((tok, is_ptr4_kind(tag, kind)))
}

/// `33 <TYPE> <varint>` — a literal of any type. Returns the value.
pub(crate) fn eat_lit_any(seg: &[u8], p: &mut usize, what: &'static str) -> Result<i32, Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    eat_any_type(seg, p, what)?;
    read_varint(seg, p).ok_or(blk(seg, *p, what))
}

/// Consume `54 <k>`, requiring the exact depth `k`.
///
/// The depths are pinned rather than merely decoded, for
/// [`super::alloc_init_or_fail`]'s reason: they are the only place the *bracing*
/// of the source shows up in this stream, and a differently braced body is a
/// different block plan.
pub(crate) fn eat_close(seg: &[u8], p: &mut usize, k: u8, what: &'static str) -> Result<(), Block> {
    eat_opt_stmt_marker(seg, p);
    if !eat(seg, p, &[0x54, k]) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// One guard: `if (<formal> == 0) return <K>;` with its braces.
///
/// Entered at [`GUARD_ENTRY_DEPTH`] and left one scope shallower — which is the
/// asymmetry the stream actually has: the two inner closes come *before* the
/// arm's label and the outer one *after* it.
///
/// ```text
///   B9 <tok> <PTR>  33 <PTR> 0  1F  38 <Lskip>
///   53 53  [line]  33 <INT> <K>  41 <INT>  3A <Lepi>
///   [line] 54 05  [line] 54 04  29 <Lskip>  54 03
/// ```
pub(crate) struct Guard {
    pub(crate) tok: u32,
    pub(crate) ret: i32,
    pub(crate) skip: u32,
    pub(crate) epi: u32,
}

pub(crate) fn eat_guard(seg: &[u8], p: &mut usize) -> Result<Guard, Block> {
    let (tok, is_ptr) = eat_load(seg, p, "gret-guard-operand")?;
    if !is_ptr {
        // An `int` guard is `cmpwi`, not `cmplwi` — one word different, and it
        // links. `work/w-ifn/probe/blkorder.cpp` cell `b1` is the witness.
        return Err(blk(seg, *p, "gret-guard-operand-not-a-pointer"));
    }
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, "gret-guard-literal"));
    }
    let (tag, kind) = eat_any_type(seg, p, "gret-guard-literal-type")?;
    if !is_ptr4_kind(tag, kind) {
        return Err(blk(seg, *p, "gret-guard-literal-not-a-pointer"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "gret-guard-literal-value"))?;
    if k != 0 {
        return Err(blk(seg, *p, "gret-guard-not-against-null"));
    }
    if !eat_byte(seg, p, 0x1F) {
        return Err(blk(seg, *p, "gret-guard-not-cmp-eq"));
    }
    let skip = eat_transfer(seg, p, 0x38, "gret-guard-branch")?;

    if !eat_byte(seg, p, 0x53) || !eat_byte(seg, p, 0x53) {
        return Err(blk(seg, *p, "gret-arm-scopes"));
    }
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, "gret-arm-literal"));
    }
    if !eat_int_like(seg, p) {
        return Err(blk(seg, *p, "gret-arm-literal-type"));
    }
    let ret = read_varint(seg, p).ok_or(blk(seg, *p, "gret-arm-literal-value"))?;
    if !(-0x8000..=0x7FFF).contains(&ret) {
        return Err(blk(seg, *p, "gret-arm-literal-wider-than-simm16"));
    }
    if !eat_byte(seg, p, 0x41) || !eat_int_like(seg, p) {
        return Err(blk(seg, *p, "gret-arm-result-type"));
    }
    let epi = eat_transfer(seg, p, 0x3A, "gret-arm-jump")?;

    eat_close(seg, p, GUARD_ENTRY_DEPTH + 2, "gret-arm-close-inner")?;
    eat_close(seg, p, GUARD_ENTRY_DEPTH + 1, "gret-arm-close-outer")?;
    eat_opt_stmt_marker(seg, p);
    let skip2 = eat_label(seg, p, "gret-arm-label")?;
    if skip2 != skip {
        return Err(blk(seg, *p, "gret-arm-label-is-not-the-branch-target"));
    }
    eat_close(seg, p, GUARD_ENTRY_DEPTH, "gret-guard-close")?;
    Ok(Guard {
        tok,
        ret,
        skip,
        epi,
    })
}

/// One `memcpy(dst, src, n)` statement, as the `40` intrinsic.
///
/// ```text
///   33 <INT> 172  40 <TYPE>
///   33 <T> <align> 55 <T>          (x2 — the alignment hints, read and checked)
///   33 <T> <n>     55 <T>
///   B9 <src> <T> [2C <T> 0] 55 <T>
///   B9 <dst> <T> [2C <T> 0] 55 <T>
///   4C 4B
/// ```
struct Copy {
    dst: u32,
    src: u32,
    len: i32,
}

fn eat_copy(seg: &[u8], p: &mut usize) -> Result<Copy, Block> {
    let sel = eat_lit_any(seg, p, "gret-copy-selector")?;
    if sel != MEMCPY_SELECTOR {
        return Err(blk(seg, *p, "gret-intrinsic-is-not-memcpy"));
    }
    if !eat_byte(seg, p, 0x40) {
        return Err(blk(seg, *p, "gret-copy-not-an-intrinsic-call"));
    }
    eat_any_type(seg, p, "gret-copy-result-type")?;

    // The two alignment hints. Read and bounded rather than skipped: an
    // unmodelled value here would be a fact about the copy this class has not
    // graded, and at this length the lowering is a call regardless — which is
    // exactly the kind of "it cannot matter" that board #1148 is about.
    for what in ["gret-copy-align-0", "gret-copy-align-1"] {
        let a = eat_lit_any(seg, p, what)?;
        if !(1..=16).contains(&a) {
            return Err(blk(seg, *p, "gret-copy-alignment-out-of-range"));
        }
        eat_arg_sep(seg, p, what)?;
    }

    let len = eat_lit_any(seg, p, "gret-copy-length")?;
    eat_arg_sep(seg, p, "gret-copy-length")?;
    if !(MEMCPY_CALL_STEP..=0x7FFF).contains(&len) {
        // Below the measured step c2 expands the copy inline.
        return Err(blk(seg, *p, "gret-copy-length-outside-the-call-window"));
    }

    let src = eat_copy_operand(seg, p, "gret-copy-src")?;
    let dst = eat_copy_operand(seg, p, "gret-copy-dst")?;
    if !eat_byte(seg, p, 0x4C) {
        return Err(blk(seg, *p, "gret-copy-end"));
    }
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, "gret-copy-stmt-end"));
    }
    Ok(Copy { dst, src, len })
}

/// `55 <TYPE>` — the argument separator.
pub(crate) fn eat_arg_sep(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x55) {
        return Err(blk(seg, *p, what));
    }
    eat_any_type(seg, p, what)?;
    Ok(())
}

/// `B9 <tok> <PTR> [2C <PTR> 0] 55 <PTR>` — one pointer argument of the copy,
/// with the optional reinterpreting conversion one witness carries and the
/// other does not.
fn eat_copy_operand(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    let (tok, is_ptr) = eat_load(seg, p, what)?;
    if !is_ptr {
        return Err(blk(seg, *p, "gret-copy-operand-not-a-pointer"));
    }
    if eat_byte(seg, p, 0x2C) {
        let (tag, kind) = eat_any_type(seg, p, what)?;
        if !is_ptr4_kind(tag, kind) {
            return Err(blk(seg, *p, "gret-copy-convert-not-to-a-pointer"));
        }
        let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
        if k != 0 {
            return Err(blk(seg, *p, "gret-copy-convert-has-an-offset"));
        }
    }
    eat_arg_sep(seg, p, what)?;
    Ok(tok)
}

/// `B9 <tok> <PTR> 33 <INT> <off> 27 <PTR>` — a member designator, and then
/// optionally the `30 <TYPE>` that turns it into a value read.
fn eat_member(
    seg: &[u8],
    p: &mut usize,
    base: u32,
    deref: bool,
    what: &'static str,
) -> Result<i32, Block> {
    let (tok, is_ptr) = eat_load(seg, p, what)?;
    if !is_ptr || tok != base {
        return Err(blk(seg, *p, "gret-clamp-member-base"));
    }
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    if !eat_int_like(seg, p) {
        return Err(blk(seg, *p, what));
    }
    let off = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if !(0..=0x7FFF).contains(&off) {
        return Err(blk(seg, *p, "gret-clamp-member-offset-out-of-range"));
    }
    if !eat_byte(seg, p, 0x27) {
        return Err(blk(seg, *p, "gret-clamp-member-not-an-offset-add"));
    }
    eat_any_type(seg, p, what)?;
    if deref {
        if !eat_byte(seg, p, 0x30) {
            return Err(blk(seg, *p, "gret-clamp-member-not-a-load"));
        }
        eat_any_type(seg, p, what)?;
    }
    Ok(off)
}

/// **The production.** Returns `Err` on the first byte that is not its grammar,
/// working on its own cursor, so a body that declines still reports its dispatch
/// arm's blocker and no census key moves.
pub(crate) fn try_parse_guard_ret_chain(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not in the emitter.**
    // Board **#1638**, which has fired twice. Asked FIRST, before any body byte
    // is read, so the refusal cannot depend on how far the walk got.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "gret-not-o1"));
    }
    let params = parse_params(seg, lo)?;
    let formals = parse_formals(seg, lo)?;
    if params.len() != 3 || formals.len() != 3 || params[0] != formals[0] {
        return Err(blk(seg, start, "gret-not-three-formals-free-fn"));
    }

    let mut p = start;

    // ---- the two guards, in source order ------------------------------------
    let g0 = eat_guard(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "gret-second-guard-scope"));
    }
    let g1 = eat_guard(seg, &mut p)?;
    if g0.epi != g1.epi {
        return Err(blk(seg, p, "gret-guards-branch-to-different-epilogues"));
    }
    if g0.skip == g1.skip || g0.skip == g0.epi || g1.skip == g1.epi {
        return Err(blk(seg, p, "gret-labels-alias"));
    }
    let ix = |t: u32| params.iter().position(|&x| x == t);
    let (Some(i0), Some(i1)) = (ix(g0.tok), ix(g1.tok)) else {
        return Err(blk(seg, p, "gret-guard-operand-is-not-a-formal"));
    };
    if i0 != 0 || i1 != 1 {
        // Both witnesses test formal 0 then formal 1, and which formal a guard
        // tests decides which register its `cmplwi` reads once the park has run.
        return Err(blk(seg, p, "gret-guards-are-not-formals-0-then-1"));
    }

    // ---- the copy ------------------------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let copy = eat_copy(seg, &mut p)?;
    let (Some(dst), Some(src)) = (ix(copy.dst), ix(copy.src)) else {
        return Err(blk(seg, p, "gret-copy-operand-is-not-a-formal"));
    };
    if dst == src {
        return Err(blk(seg, p, "gret-copy-is-self-to-self"));
    }

    // ---- sub-shape G ends here; sub-shape S has the clamp -------------------
    let spine = if dst == 1 && src == 0 {
        GuardRetSpine::Copy {
            dst,
            src,
            len: copy.len,
        }
    } else if dst == 0 && src == 1 {
        let (lo_off, hi_off) = eat_clamp(seg, &mut p, params[0])?;
        GuardRetSpine::CopyClamp {
            dst,
            src,
            len: copy.len,
            lo: lo_off,
            hi: hi_off,
        }
    } else {
        return Err(blk(seg, p, "gret-copy-formals-are-not-a-graded-pair"));
    };

    // ---- `return 0;` and the plumbing ---------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "gret-tail-literal"));
    }
    let zero = read_varint(seg, &mut p).ok_or(blk(seg, p, "gret-tail-literal-value"))?;
    if zero != 0 {
        return Err(blk(seg, p, "gret-tail-is-not-return-zero"));
    }
    super::super::expr::eat_return_plumbing(seg, &mut p, true, GUARD_ENTRY_DEPTH as usize - 1)?;

    Ok(BodyShape::GuardRetChain(GuardRetChain {
        params,
        guards: vec![
            GuardRetGuard {
                formal: i0,
                ret: g0.ret,
            },
            GuardRetGuard {
                formal: i1,
                ret: g1.ret,
            },
        ],
        spine,
    }))
}

/// Sub-shape **S**'s tail: bind the destination to a local, then
/// `if (m->hi < m->lo) m->hi = m->lo;`.
///
/// Returns `(lo, hi)` — the offsets of the value read and of the member stored
/// into. They are returned in that order because that is the order the EMITTER
/// wants them (`lwz r11,<lo>` then `lwz r10,<hi>`), which is the REVERSE of the
/// order the IL states them in.
fn eat_clamp(seg: &[u8], p: &mut usize, dst_tok: u32) -> Result<(i32, i32), Block> {
    // `M *m = (M *)dst;`
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, "gret-clamp-bind-designator"));
    }
    let (m, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "gret-clamp-bind-token"))?;
    *p += w;
    if m == dst_tok {
        return Err(blk(seg, *p, "gret-clamp-bind-is-the-formal"));
    }
    let (t, is_ptr) = eat_load(seg, p, "gret-clamp-bind-source")?;
    if !is_ptr || t != dst_tok {
        return Err(blk(seg, *p, "gret-clamp-bind-is-not-the-copy-destination"));
    }
    if !eat_byte(seg, p, 0x2C) {
        return Err(blk(seg, *p, "gret-clamp-bind-convert"));
    }
    eat_any_type(seg, p, "gret-clamp-bind-convert-type")?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "gret-clamp-bind-convert-value"))?;
    if k != 0 {
        return Err(blk(seg, *p, "gret-clamp-bind-convert-has-an-offset"));
    }
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, "gret-clamp-bind-store"));
    }
    eat_any_type(seg, p, "gret-clamp-bind-store-type")?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, "gret-clamp-bind-stmt-end"));
    }

    // `if (m->hi < m->lo) m->hi = m->lo;`
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x53) {
        return Err(blk(seg, *p, "gret-clamp-scope"));
    }
    let hi = eat_member(seg, p, m, true, "gret-clamp-test-hi")?;
    let lo = eat_member(seg, p, m, true, "gret-clamp-test-lo")?;
    if hi == lo {
        return Err(blk(seg, *p, "gret-clamp-compares-a-member-with-itself"));
    }
    if !eat_byte(seg, p, 0x22) {
        return Err(blk(seg, *p, "gret-clamp-not-cmp-lt"));
    }
    let skip = eat_transfer(seg, p, 0x38, "gret-clamp-branch")?;
    if !eat_byte(seg, p, 0x53) || !eat_byte(seg, p, 0x53) {
        return Err(blk(seg, *p, "gret-clamp-arm-scopes"));
    }
    eat_opt_stmt_marker(seg, p);
    let store_hi = eat_member(seg, p, m, false, "gret-clamp-store-target")?;
    let store_lo = eat_member(seg, p, m, true, "gret-clamp-store-value")?;
    if store_hi != hi || store_lo != lo {
        return Err(blk(seg, *p, "gret-clamp-store-is-not-the-tested-pair"));
    }
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, "gret-clamp-store"));
    }
    eat_any_type(seg, p, "gret-clamp-store-type")?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, "gret-clamp-store-stmt-end"));
    }
    eat_close(seg, p, GUARD_ENTRY_DEPTH + 2, "gret-clamp-close-inner")?;
    eat_close(seg, p, GUARD_ENTRY_DEPTH + 1, "gret-clamp-close-outer")?;
    eat_opt_stmt_marker(seg, p);
    let skip2 = eat_label(seg, p, "gret-clamp-label")?;
    if skip2 != skip {
        return Err(blk(seg, *p, "gret-clamp-label-is-not-the-branch-target"));
    }
    eat_close(seg, p, GUARD_ENTRY_DEPTH, "gret-clamp-close")?;
    Ok((lo, hi))
}

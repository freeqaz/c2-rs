//! **W-JSON — the recognizer for the UTF-16 → UTF-8 copy loop.**
//!
//! `src/xdk/xjson/jsonwriter.cpp`'s only emitted function,
//! `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z`. The seventy-six words are in
//! `c2_core::codegen::json_utf8_copy`; this file is the accept/refuse boundary.
//!
//! ## The source shape
//!
//! ```cpp
//! long JsonWriter::GetBuffer(unsigned short *pBuffer, unsigned long *pSize) {
//!     unsigned long hr = 0;
//!     if (!pSize || (!pBuffer && *pSize != 0)) {
//!         hr = K_ARG_ERR;                       /* 0x80070057 */
//!     } else {
//!         unsigned long outputSize = 0, index = 0;
//!         if (mBufferSize > 0) {                /* OFF_SIZE */
//!             int offset = 0;
//!             do {
//!                 index++;
//!                 unsigned short wc =
//!                     *(unsigned short *)((char *)mBuffer + offset);   /* OFF_BUFFER */
//!                 offset += 2;
//!                 if (wc <= 0x7F) { … 1 byte … }
//!                 else {
//!                     unsigned long maxSize = *pSize;
//!                     if (wc <= 0x7FF) { … 2 bytes … } else { … 3 bytes … }
//!                 }
//!             } while (index < mBufferSize);
//!         }
//!         if (outputSize >= *pSize) hr = K_SIZE_ERR;   /* 0x803F0005 */
//!         *pSize = outputSize + 1;
//!     }
//!     return hr;
//! }
//! ```
//!
//! ## Why this recognizer is a TEMPLATE and not a hand-written grammar
//!
//! Every predecessor in this series (`osf_handle_guard`, `xlrc_create_guard`, …)
//! walks its body with a hand-written production per statement. That scales with
//! the body, and this body is **1,272 IL bytes over roughly sixty statements** —
//! three times the largest one written that way. Written the same way it would
//! be ~2,000 lines whose only content is "this byte, then that byte".
//!
//! So the walk is a **unifying template match** instead. [`PAT`] is the
//! reference function's own token stream with four kinds of hole in it:
//!
//! | | |
//! |---|---|
//! | [`P::B`] | an exact byte — every opcode, every operator, every scope depth |
//! | [`P::T`] | a token varint, **unified**: slot `n` must be the same token at every occurrence, and the twenty-five slots must be pairwise DISTINCT |
//! | [`P::Y`] | a TYPE, **unified as bytes** and additionally pinned to the reference's `(tag, kind)` — the two structural fields. Only the trailing per-TU type-table **id** is free |
//! | [`P::K`] / [`P::F`] | a literal varint: `K` pinned to the reference value, `F` captured into one of the four free fields |
//!
//! This is strictly *more* restrictive than a hand-written grammar, not less:
//! nothing is "skipped for now", the whole stream from the first statement to
//! the `4D` is consumed, and a single unexpected byte anywhere refuses the body.
//! It is also the honest shape of the claim — this class is a **transcription**
//! of one function, and the reader admits exactly the programs the emitter's
//! seventy-six words are right for.
//!
//! ## What is a field, and what is pinned (#1767)
//!
//! *A value is a free field when varying it changes only an immediate field of
//! the same instruction, and is PINNED when varying it would change which
//! instruction c2 emits.* Applied at its strict end here:
//!
//! | value | | why |
//! |---|---|---|
//! | `off_buffer`, `off_size` | **2 free fields** | each lands in an `lwz` displacement and nothing else. `off_size` is read at two sites and the two must be the same field |
//! | `k_arg_err`, `k_size_err` | **2 free fields** | each lands in its own `lis`+`ori` pair |
//! | **both status constants' halves — pinned non-zero** | refused otherwise | a zero half is a one-word materialization and a shorter body, exactly as W-XLR §4.4 records |
//! | **the eight UTF-8 constants** — `0x7F`, the `clrlwi` width, `0x7FF`, `0xC0`, `0xE0`, `0x80`, and the three `rlwimi` rotate/mask triples | **pinned** | they are one program, not eight immediates. `0xC0 | ((wc >> 6) & 0x1F)` becomes a *different instruction sequence* the moment the shift and the mask stop agreeing, and the emitter has one `rlwimi` per site with no chooser. Board **#1706** |
//! | **the three output-size increments 1/2/3** | pinned | they are the encoded lengths of the three arms |
//! | **the element step 2** | pinned | it is `sizeof(unsigned short)` and it appears in five different words — an `addi`, three pointer bumps and a `sthu` displacement |
//! | **the formal count and their order** | pinned at `this` + 2 | they decide which registers r3/r4/r5 the guards read, and r4 is *written* by the three-byte arm |
//!
//! **Zero words are chosen by a scheduler or a register allocator.**

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{eat_opt_stmt_marker, read_token_var, read_type, read_varint};
use crate::func::JsonUtf8Copy;

/// One element of the transcribed token pattern. See the module doc.
#[derive(Clone, Copy)]
enum P {
    /// An exact byte.
    B(u8),
    /// Zero or more `4F 01 <varint>` source-line markers. The line numbers
    /// themselves are a property of the source FILE, not of the program, so they
    /// are the one thing in the stream that must not be pinned.
    Line,
    /// A token varint, unified into slot `n`.
    T(u8),
    /// A TYPE, unified into slot `n` and pinned to that slot's `(tag, kind)`.
    Y(u8),
    /// A literal varint pinned to this value.
    K(i32),
    /// A literal varint captured into free field `n`, and unified with itself.
    F(u8),
}
use P::{Line, B, F, K, T, Y};

/// The number of distinct IL tokens the body names: three formals (`this`
/// included), six locals and sixteen labels.
const NTOK: usize = 25;
/// The number of distinct TYPEs it names.
const NTY: usize = 12;
/// The number of free fields.
const NFLD: usize = 4;

/// The `(tag, kind)` of each TYPE slot, read off the reference IL. The third
/// byte of a TYPE is a per-TU table **id** and is deliberately not here; these
/// two bytes are the structure — width, class and **signedness**.
///
/// The signedness entries are live wrong-emit fences, not decoration. The
/// relational opcodes are sign-agnostic (`docs/CODEGEN_W6_COMPARE.md` §1.1), so
/// `unsigned long outputSize` and `long outputSize` emit the **same `22` byte**
/// and differ only here — and c2 emits `cmpw` for the signed one where this
/// class's emitter has an unconditional `cmplw`, in four places. Board **#1788**
/// a second time: read the TYPE, not the byte.
const TY_TAG_KIND: [(u8, u8); NTY] = [
    (0x86, 0x42), // 0  unsigned long — hr, outputSize, index, maxSize, *pSize
    (0x86, 0x43), // 1  unsigned long *
    (0x86, 0x43), // 2  unsigned short *
    (0xA6, 0x43), // 3  JsonWriter * (the `this` designator's type)
    (0x86, 0x41), // 4  int — every UTF-8 constant's type
    (0xA6, 0x43), // 5  the element-count member
    (0xA6, 0x43), // 6  the buffer member
    (0x86, 0x43), // 7  the buffer member's loaded value
    (0x86, 0x43), // 8  char * — the (char *) cast the subscript goes through
    (0x86, 0x41), // 9  long — the return type and the pointer-arith scale
    (0x84, 0x22), // 10 unsigned short — the loaded code unit
    (0x82, 0x12), // 11 unsigned char — the narrowing each store goes through
];

/// Which slots are the formals, in `parse_params` order.
const SLOT_THIS: usize = 8;
const SLOT_BUFFER: usize = 3;
const SLOT_SIZE: usize = 1;

/// True for a wide constant this class materializes with the pinned `lis`+`ori`
/// pair: **both halves non-zero**. With a zero high half c2 emits one `li`, with
/// a zero low half one `lis` — either is a shorter body and a different block
/// plan, and the class has no witness of either.
fn is_two_word_constant(k: i32) -> bool {
    let u = k as u32;
    (u >> 16) != 0 && (u & 0xFFFF) != 0
}

/// The nine BLOCKS of the body, as pattern-element indices. Every structural
/// refusal inside the template reports the block it happened in rather than one
/// key for the whole file — board **#1704**'s defect, which is that a census
/// naming a single fall-through key cannot tell ten `_neg` cells apart. The
/// boundaries are the pattern's own label definitions (`29 <tok>`), so they are
/// the body's real blocks and not a hand-chosen split.
const REGIONS: [(usize, &str); 9] = [
    (0, "json-init-and-arg-guard"),
    (31, "json-arg-error-arm"),
    (51, "json-loop-preheader"),
    (103, "json-loop-head-and-one-byte-arm"),
    (255, "json-two-byte-arm"),
    (410, "json-three-byte-arm"),
    (602, "json-loop-back-edge"),
    (628, "json-size-check"),
    (683, "json-return"),
];

/// Which block pattern element `ix` is in.
fn region(ix: usize) -> &'static str {
    let mut r = REGIONS[0].1;
    for (at, name) in REGIONS {
        if ix >= at {
            r = name;
        }
    }
    r
}

/// Walk [`PAT`] over the stream, unifying tokens, types and fields.
fn match_pattern<'a>(
    seg: &'a [u8],
    p: &mut usize,
    toks: &mut [Option<u32>; NTOK],
    tys: &mut [Option<&'a [u8]>; NTY],
    flds: &mut [Option<i32>; NFLD],
) -> Result<(), Block> {
    for (ix, el) in PAT.iter().enumerate() {
        match *el {
            B(x) => {
                if seg.get(*p) != Some(&x) {
                    return Err(blk(seg, *p, region(ix)));
                }
                *p += 1;
            }
            Line => eat_opt_stmt_marker(seg, p),
            T(n) => {
                let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, region(ix)))?;
                let slot = &mut toks[n as usize];
                match slot {
                    Some(prev) if *prev != tok => return Err(blk(seg, *p, region(ix))),
                    Some(_) => {}
                    None => *slot = Some(tok),
                }
                *p += w;
            }
            Y(n) => {
                let (tag, kind, _, w) =
                    read_type(seg, *p).ok_or(blk(seg, *p, "json-type-undecodable"))?;
                if (tag, kind) != TY_TAG_KIND[n as usize] {
                    return Err(blk(seg, *p, "json-type-tag-kind"));
                }
                let bytes = &seg[*p..*p + w];
                let slot = &mut tys[n as usize];
                match slot {
                    Some(prev) if *prev != bytes => {
                        return Err(blk(seg, *p, "json-type-mismatch"))
                    }
                    Some(_) => {}
                    None => *slot = Some(bytes),
                }
                *p += w;
            }
            K(v) => {
                let at = *p;
                let k = read_varint(seg, p).ok_or(blk(seg, at, region(ix)))?;
                if k != v {
                    return Err(blk(seg, at, region(ix)));
                }
            }
            F(n) => {
                let at = *p;
                let k = read_varint(seg, p).ok_or(blk(seg, at, region(ix)))?;
                let slot = &mut flds[n as usize];
                match slot {
                    Some(prev) if *prev != k => return Err(blk(seg, at, "json-field-disagrees")),
                    Some(_) => {}
                    None => *slot = Some(k),
                }
            }
        }
    }
    Ok(())
}

/// **The recognizer.** `start` is the body's first statement byte — the `26` of
/// `hr = 0` — and `lo` is the `4C 4F 11` marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` on the first byte that is not its grammar, so a
/// body that declines still reports `try_parse_assign_body_detail`'s blocker
/// (`expr-brfalse`, which is what `jsonwriter.cpp` read at this lane's base) and
/// no census key moves.
pub(crate) fn try_parse_json_utf8_copy(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not in the emitter.**
    // Board **#1638**, which has fired twice. Asked FIRST, before any body byte
    // is read, so the refusal cannot depend on how far the walk got.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "json-not-o1"));
    }
    // `this` + two formals. `parse_params` prepends the `this` token when the
    // pre-body region binds one and REFUSES when the binding is undetermined, so
    // "there is a `this`" is an established fact and not a count.
    let params = parse_params(seg, lo)?;
    let formals = parse_formals(seg, lo)?;
    if params.len() != 3 || formals.len() != 2 || params[1..] != formals[..] {
        return Err(blk(seg, start, "json-not-a-two-formal-member-fn"));
    }

    let mut p = start;
    let mut toks: [Option<u32>; NTOK] = [None; NTOK];
    let mut tys: [Option<&[u8]>; NTY] = [None; NTY];
    let mut flds: [Option<i32>; NFLD] = [None; NFLD];
    match_pattern(seg, &mut p, &mut toks, &mut tys, &mut flds)?;

    let toks = {
        let mut t = [0u32; NTOK];
        for (i, s) in toks.iter().enumerate() {
            t[i] = s.ok_or(blk(seg, start, "json-token-slot-unbound"))?;
        }
        t
    };
    // **The twenty-five token slots must be pairwise DISTINCT.** Two of them
    // being the same token is a source in which two variables — or a variable
    // and a label — are one object, which is a different program and a different
    // block plan. Nothing in the template alone forbids it.
    for i in 0..NTOK {
        for j in (i + 1)..NTOK {
            if toks[i] == toks[j] {
                return Err(blk(seg, start, "json-two-token-slots-are-one-token"));
            }
        }
    }
    // **The TYPE slots are deliberately NOT required pairwise distinct**, and
    // that is a measurement rather than a relaxation. The first version of this
    // recognizer required it — the twelve slots ARE twelve distinct byte strings
    // in the workload's own TU — and the positive fixture, which is the same
    // program in a TU whose type table happens to intern `unsigned short *` once
    // instead of twice, **refused**. Two slots sharing bytes is a property of the
    // TU's type table, not of the program.
    //
    // Nothing is lost, because every separation that decides an instruction is
    // already carried by [`TY_TAG_KIND`], which is checked per occurrence above:
    // slot 0 is pinned `(0x86, 0x42)` and slots 4 and 9 `(0x86, 0x41)`, so the
    // unsigned-long slot **cannot** collapse into either signed one and the four
    // `cmplw`s cannot become `cmpw`s. The collapses the template still admits
    // are within one `(tag, kind)` group — `int` with `long`, `unsigned short *`
    // with `char *` — and each of those pairs is the same register, the same
    // width and the same word, with the pointer-arithmetic scale pinned to 1 by
    // the template itself.
    // The formals must be the tokens the body reads through r3/r4/r5, in that
    // order. Checked by NAME against `parse_params`, not by position in the
    // template, because the template alone would accept a body that swapped the
    // two pointers and read each through the other's register.
    if params[0] != toks[SLOT_THIS]
        || params[1] != toks[SLOT_BUFFER]
        || params[2] != toks[SLOT_SIZE]
    {
        return Err(blk(seg, start, "json-formals-are-not-the-tokens-the-body-reads"));
    }

    let f = |n: usize| flds[n].ok_or(blk(seg, start, "json-field-unbound"));
    let k_arg_err = f(0)?;
    let off_size = f(1)?;
    let off_buffer = f(2)?;
    let k_size_err = f(3)?;

    if !is_two_word_constant(k_arg_err) || !is_two_word_constant(k_size_err) {
        return Err(blk(seg, start, "json-status-constant-has-a-zero-half"));
    }
    // Each offset lands in an `lwz` displacement, so it must fit one, and it
    // must be a multiple of four because both members are four-byte scalars a
    // `lwz` reads — an unaligned displacement is a program c2 lowers differently
    // and this class has no witness of.
    for off in [off_buffer, off_size] {
        if i16::try_from(off).is_err() || off < 0 || off % 4 != 0 {
            return Err(blk(seg, start, "json-member-offset-out-of-class"));
        }
    }
    // The two members are DIFFERENT members. Equal offsets would make the buffer
    // pointer and the element count one field, which is not a program this class
    // has words for.
    if off_buffer == off_size {
        return Err(blk(seg, start, "json-both-members-at-one-offset"));
    }

    Ok(BodyShape::JsonUtf8Copy(JsonUtf8Copy {
        params: params.clone(),
        off_buffer,
        off_size,
        k_arg_err,
        k_size_err,
    }))
}

/// The reference function's own token stream, from the first statement byte to
/// the closing `4D`. Generated from `work/w-json/probe/ref.obj`'s IL capture and
/// checked against it by a test in this file.
const PAT: &[P] = &[
    B(0x26), T(0), B(0x33), Y(0), K(0), B(0x32), Y(0), B(0x4B), Line, B(0x53),
    B(0xB9), T(1), Y(1), B(0x38), T(2), B(0xB9), T(3), Y(2), B(0x39), T(4),
    B(0xB9), T(1), Y(1), B(0x30), Y(0), B(0x33), Y(0), K(0), B(0x20), B(0x38), T(4),
    B(0x29), T(2), B(0x53), B(0x53), Line, B(0x26), T(0), B(0x33), Y(0), F(0), B(0x32), Y(0),
    B(0x4B), Line, B(0x54), B(0x05), B(0x54), B(0x04), B(0x3A), T(5), B(0x29), T(4), B(0x53),
    B(0x53), Line, B(0x26), T(6), B(0x33), Y(0), K(0), B(0x32), Y(0), B(0x4B), Line,
    B(0x26), T(7), B(0x33), Y(0), K(0), B(0x32), Y(0), B(0x4B), Line, B(0x53),
    B(0xB9), T(8), Y(3), B(0x33), Y(4), F(1), B(0x27), Y(5), B(0x30), Y(0),
    B(0x33), Y(0), K(0), B(0x24), B(0x38), T(9), B(0x53), B(0x53), Line, B(0x26), T(10),
    B(0x33), Y(4), F(2), B(0x32), Y(4), B(0x4B), Line, B(0x29), T(11), B(0x53), B(0x53), Line,
    B(0x26), T(7), B(0x33), Y(0), K(1), B(0x35), Y(0), B(0x4B), Line, B(0x26), T(12),
    B(0xB9), T(8), Y(3), B(0x33), Y(4), K(0), B(0x27), Y(6), B(0x30), Y(7),
    B(0x2C), Y(8), B(0x00), B(0xB9), T(10), Y(4), B(0x33), Y(9), K(1), B(0x04), B(0x02),
    B(0x2C), Y(2), B(0x00), B(0x30), Y(10), B(0x32), Y(10), B(0x4B), Line, B(0x26), T(10),
    B(0x33), Y(4), K(2), B(0x0F), Y(4), B(0x4B), Line, B(0x53), B(0xB9), T(12), Y(10),
    B(0x2C), Y(4), B(0x00), B(0x33), Y(4), K(127), B(0x21), B(0x38), T(13), B(0x53), B(0x53),
    Line, B(0x26), T(6), B(0x33), Y(0), K(1), B(0x35), Y(0), B(0x4B), Line, B(0x53),
    B(0xB9), T(6), Y(0), B(0xB9), T(1), Y(1), B(0x30), Y(0), B(0x22), B(0x38), T(14), B(0x53),
    B(0x53), Line, B(0xB9), T(3), Y(2), B(0xB9), T(12), Y(10), B(0x2C), Y(4), B(0x00),
    B(0x33), Y(4), K(127), B(0x0B), B(0x2C), Y(11), B(0x00), B(0x2C), Y(10), B(0x00),
    B(0x32), Y(10), B(0x4B), Line, B(0x26), T(3), B(0x33), Y(9), K(2), B(0x35), Y(2), B(0x4B),
    Line, B(0xB9), T(3), Y(2), B(0x33), Y(10), K(0), B(0x32), Y(10), B(0x4B), Line,
    B(0x54), B(0x10), Line, B(0x54), B(0x0F), B(0x29), T(14), B(0x54), B(0x0E),
    B(0x54), B(0x0D), B(0x54), B(0x0C), B(0x3A), T(15), B(0x29), T(13), B(0x53), B(0x53), Line,
    B(0x26), T(16), B(0xB9), T(1), Y(1), B(0x30), Y(0), B(0x32), Y(0), B(0x4B), Line, B(0x53),
    B(0xB9), T(12), Y(10), B(0x2C), Y(4), B(0x00), B(0x33), Y(4), K(2047), B(0x21),
    B(0x38), T(17), B(0x53), B(0x53), Line, B(0x26), T(6), B(0x33), Y(4), K(2), B(0x0F), Y(0),
    B(0x4B), Line, B(0x53), B(0xB9), T(6), Y(0), B(0xB9), T(16), Y(0), B(0x22), B(0x38), T(18),
    B(0x53), B(0x53), Line, B(0xB9), T(3), Y(2), B(0x33), Y(4), K(192), B(0xB9), T(12), Y(10),
    B(0x2C), Y(4), B(0x00), B(0x33), Y(4), K(6), B(0x0A), B(0x33), Y(4), K(31), B(0x0B),
    B(0x0C), B(0x2C), Y(11), B(0x00), B(0x2C), Y(10), B(0x00), B(0x32), Y(10), B(0x4B), Line,
    B(0x26), T(3), B(0x33), Y(9), K(2), B(0x35), Y(2), B(0x4B), Line, B(0xB9), T(3), Y(2),
    B(0x33), Y(4), K(128), B(0xB9), T(12), Y(10), B(0x2C), Y(4), B(0x00), B(0x33), Y(4), K(63),
    B(0x0B), B(0x0C), B(0x2C), Y(11), B(0x00), B(0x2C), Y(10), B(0x00), B(0x32), Y(10),
    B(0x4B), Line, B(0x26), T(3), B(0x33), Y(9), K(2), B(0x35), Y(2), B(0x4B), Line,
    B(0xB9), T(3), Y(2), B(0x33), Y(10), K(0), B(0x32), Y(10), B(0x4B), Line, B(0x54), B(0x13),
    Line, B(0x54), B(0x12), B(0x29), T(18), B(0x54), B(0x11), B(0x54), B(0x10),
    B(0x54), B(0x0F), B(0x3A), T(19), B(0x29), T(17), B(0x53), B(0x53), Line, B(0x26), T(6),
    B(0x33), Y(4), K(3), B(0x0F), Y(0), B(0x4B), Line, B(0x53), B(0xB9), T(6), Y(0),
    B(0xB9), T(16), Y(0), B(0x22), B(0x38), T(20), B(0x53), B(0x53), Line, B(0xB9), T(3), Y(2),
    B(0x33), Y(4), K(224), B(0xB9), T(12), Y(10), B(0x2C), Y(4), B(0x00), B(0x33), Y(4), K(12),
    B(0x0A), B(0x33), Y(4), K(15), B(0x0B), B(0x0C), B(0x2C), Y(11), B(0x00),
    B(0x2C), Y(10), B(0x00), B(0x32), Y(10), B(0x4B), Line, B(0x26), T(3), B(0x33), Y(9), K(2),
    B(0x35), Y(2), B(0x4B), Line, B(0xB9), T(3), Y(2), B(0x33), Y(9), K(2), B(0x02),
    B(0x33), Y(4), K(128), B(0xB9), T(12), Y(10), B(0x2C), Y(4), B(0x00), B(0x33), Y(4), K(6),
    B(0x0A), B(0x33), Y(4), K(63), B(0x0B), B(0x0C), B(0x2C), Y(11), B(0x00),
    B(0x2C), Y(10), B(0x00), B(0x32), Y(10), B(0x4B), Line, B(0x26), T(3), B(0x33), Y(9), K(2),
    B(0x35), Y(2), B(0x4B), Line, B(0xB9), T(3), Y(2), B(0x33), Y(9), K(2), B(0x02),
    B(0xB9), T(12), Y(10), B(0x2C), Y(4), B(0x00), B(0x33), Y(4), K(63), B(0x0B),
    B(0x33), Y(4), K(128), B(0x0C), B(0x2C), Y(11), B(0x00), B(0x2C), Y(10), B(0x00),
    B(0x32), Y(10), B(0x4B), Line, B(0x26), T(3), B(0x33), Y(9), K(2), B(0x35), Y(2), B(0x4B),
    Line, B(0xB9), T(3), Y(2), B(0x33), Y(10), K(0), B(0x32), Y(10), B(0x4B), Line,
    B(0x54), B(0x13), Line, B(0x54), B(0x12), B(0x29), T(20), B(0x54), B(0x11),
    B(0x54), B(0x10), B(0x54), B(0x0F), B(0x29), T(19), B(0x54), B(0x0E), Line,
    B(0x54), B(0x0D), B(0x54), B(0x0C), B(0x29), T(15), B(0x54), B(0x0B), Line,
    B(0x54), B(0x0A), B(0x54), B(0x09), B(0x29), T(21), B(0xB9), T(7), Y(0),
    B(0xB9), T(8), Y(3), B(0x33), Y(4), F(1), B(0x27), Y(5), B(0x30), Y(0), B(0x22),
    B(0x39), T(11), B(0x29), T(22), Line, B(0x54), B(0x08), Line, B(0x54), B(0x07),
    B(0x29), T(9), B(0x54), B(0x06), B(0x53), B(0xB9), T(6), Y(0), B(0xB9), T(1), Y(1),
    B(0x30), Y(0), B(0x23), B(0x38), T(23), B(0x53), B(0x53), Line, B(0x26), T(0),
    B(0x33), Y(0), F(3), B(0x32), Y(0), B(0x4B), Line, B(0x54), B(0x08), Line,
    B(0x54), B(0x07), B(0x29), T(23), B(0x54), B(0x06), B(0xB9), T(1), Y(1),
    B(0xB9), T(6), Y(0), B(0x33), Y(4), K(1), B(0x02), B(0x32), Y(0), B(0x4B), Line,
    B(0x54), B(0x05), B(0x54), B(0x04), B(0x29), T(5), B(0x54), B(0x03), Line,
    B(0xB9), T(0), Y(0), B(0x2C), Y(9), B(0x00), B(0x41), Y(9), B(0x3A), T(24), Line,
    B(0x54), B(0x02), B(0x29), T(24), B(0x4F), B(0x12), B(0x47), B(0x54), B(0x01),
    B(0x54), B(0x00), B(0x4F), B(0x02), B(0x20), B(0x00), Line, B(0x4D),
];


#[cfg(test)]
mod tests {
    use super::*;

    /// The template is the reference function's whole stream: it must consume
    /// every byte from the first statement to the `4D` and contain exactly the
    /// four field holes the emitter reads. Counted here so a hand edit to [`PAT`]
    /// that drops or duplicates one is caught by a test rather than by an obj.
    #[test]
    fn the_template_has_the_shape_the_emitter_expects() {
        assert_eq!(PAT.len(), 716, "pattern elements");
        let n = |f: fn(&P) -> bool| PAT.iter().filter(|e| f(e)).count();
        assert_eq!(n(|e| matches!(e, F(0))), 1, "k_arg_err");
        assert_eq!(n(|e| matches!(e, F(1))), 2, "off_size — read at TWO sites");
        assert_eq!(n(|e| matches!(e, F(2))), 1, "off_buffer");
        assert_eq!(n(|e| matches!(e, F(3))), 1, "k_size_err");
        assert!(matches!(PAT[0], B(0x26)), "the body opens on the `hr` designator");
        assert!(matches!(PAT[PAT.len() - 1], B(0x4D)), "and ends on the body terminator");
        // Every token and type slot is reached, or the unification is vacuous
        // for it and the post-match binding would panic on an unbound slot.
        for s in 0..NTOK as u8 {
            assert!(PAT.iter().any(|e| matches!(e, T(n) if *n == s)), "token slot {s}");
        }
        for s in 0..NTY as u8 {
            assert!(PAT.iter().any(|e| matches!(e, Y(n) if *n == s)), "type slot {s}");
        }
    }

    /// **The clause `wjson_utf8_copy_neg.cpp`'s `n7` does not reach.** That cell
    /// stops earlier, at the body's trailing `4F 02` directive
    /// (`work/w-json/decline_probe.md`), so the zero-half refusal has no fixture
    /// witness and is graded here instead — which is the honest place for it,
    /// not a reworded comment in the `_neg` file.
    #[test]
    fn a_status_constant_with_a_zero_half_is_refused() {
        assert!(is_two_word_constant(0x8007_0057u32 as i32));
        assert!(is_two_word_constant(0x803F_0005u32 as i32));
        assert!(!is_two_word_constant(0x8007_0000u32 as i32), "a `lis` alone");
        assert!(!is_two_word_constant(0x0000_0057), "an `li` alone");
        assert!(!is_two_word_constant(0));
    }

    /// The block map is what makes ten `_neg` cells report ten keys instead of
    /// one (#1704). Its boundaries are the pattern's own label definitions, so a
    /// drifted index would silently merge two blocks' diagnostics.
    #[test]
    fn every_block_boundary_is_a_label_definition() {
        for (at, name) in REGIONS.into_iter().skip(1) {
            assert!(matches!(PAT[at], B(0x29)), "{name} does not open ON a `29`");
        }
        assert_eq!(region(0), "json-init-and-arg-guard");
        assert_eq!(region(PAT.len() - 1), "json-return");
    }
}

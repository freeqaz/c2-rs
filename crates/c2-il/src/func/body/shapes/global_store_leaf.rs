//! **W-WORDWRAP — the file-scope-global store leaf.** `void f(T x) { g = x; }`,
//! three words, and the smallest unconverted body on the whole frontier
//! (board **#2625**).
//!
//! ```cpp
//!   unsigned int g_uOption;
//!   void WordWrap_SetOption(unsigned int option) { g_uOption = option; }
//! ```
//!
//! ```text
//!   ?WordWrap_SetOption@@YAXI@Z   .text COMDAT, 0x0c B, nrel 4
//!     0000  3d600000  lis 11,0        REFHI  ?g_uOption@@3IA  + PAIR
//!     0004  906b0000  stw 3,0(11)     REFLO  ?g_uOption@@3IA  + PAIR
//!     0008  4e800020  blr
//! ```
//!
//! # The published key names none of the refusal
//!
//! `#2625` and `w-nc` **#2387** both file this body under `expr-jump`, and
//! `w-nc` hand-checked that *"there is no jump"* — twelve bytes of PowerPC with
//! no control flow whatsoever. The census window this lane took confirms it a
//! third time: the parse consumes the whole store and stops on the **exit-label
//! goto** that every accepted leaf also carries.
//!
//! ```text
//!   … 32 86 42 75 4b >3a< f9 09 54 02 29 f9 09 4f 12 47 54 01 54 00
//! ```
//!
//! `#1416`'s fall-through, on a body small enough that there is nothing to
//! argue about.
//!
//! # GRID G and GRID T — every clause below is a reading off a compiled cell
//!
//! `work/w-wordwrap/probe/gstore.cpp` (17 cells) and
//! `work/w-wordwrap/probe/gtype.cpp` (16 cells), real `c2.dll` under wibo.
//!
//! ```text
//!   G_u        the target, verbatim                         12 B  (accepted)
//!   G_i        `int` object and `int` formal — IDENTICAL bytes
//!   G_static   internal linkage — IDENTICAL bytes, STATIC symbol
//!   G_ext      a global this TU does NOT define — IDENTICAL bytes.  Refused
//!              here, because the symbol it relocates against is an undefined
//!              external and `data_syms` is the field that spells one; this
//!              class resolves a DEFINED object and must not claim to define
//!              a name the `.gl` says is imported.
//!   G_vol      `volatile` — identical bytes, and the IL carries a `2C`
//!              conversion the no-conversion clause below refuses anyway
//!   G_second   the value is the SECOND formal → `stw 4,0(11)`.  Refused: the
//!              class is fenced at ONE formal (see `try_parse_global_store_leaf`)
//!   G_lit      `g = 7u` → **lis r10** · `li r11,7` · `stw 11,0(10)`, 16 B.
//!              **The address scratch moves to r10 the moment the body needs a
//!              second register**, which is why the value must be a bare formal
//!   G_widen    `g_u = (unsigned char)x` → lis **r10** · clrlwi r11 · stw, 16 B
//!   G_narrow   `g_us = (unsigned short)x` → lis **r10** · sth 3,0(10), 12 B —
//!              the SAME LENGTH as an accepted cell and TWO WRONG WORDS.  This
//!              is the cell that makes the no-conversion clause load-bearing
//!              rather than tidy: a length check could not tell it apart.
//!   G_two      two statements → two `lis`, r11 and r10, 20 B
//!   G_arr      `g_arr[i] = x` → slwi · addi · stwx, 20 B
//!   G_arr2     `g_arr[2] = x` → lis · addi · stw 3,8(11), 16 B
//!   G_load     `return g_u` → the same `lis` with an `lwz`
//! ```
//!
//! **GRID T is the store-opcode table**, and it is enumerated rather than
//! computed from a width nibble:
//!
//! ```text
//!   82 11 …  signed char        stb        82 12 …  unsigned char, bool  stb
//!   84 21 …  short              sth        84 22 …  unsigned short, wchar_t  sth
//!   86 41 …  int, long, enum    stw        86 42 …  unsigned, unsigned long  stw
//!   86 43 …  a pointer          stw        88 81 …  long long            std
//!   88 82 …  unsigned long long std
//!   86 45 …  float              stfs f1    88 85 …  double               stfd f1
//! ```
//!
//! The last row is **refused by name**: a float formal arrives in the FP file,
//! `params` is the GPR mapping, and `IlFunction::fp_arg_sources` is the field
//! that carries the other one. Admitting it would emit `stw r3` for `stfs f1` —
//! a complete, plausible, wrong body, which is board **#232**'s shape.
//!
//! # `/Od` is the only mode that differs, and the gate is in the PARSER (#1638)
//!
//! `/O1`, `/O1 /Oi`, `/O2`, `/Ox` and `/Ox /Gy` all emit the identical three
//! words. `/Od` emits **five** (`stw 3,20(1)` · `lwz 11,20(1)` · `lis 10` ·
//! `stw 11,0(10)` · `blr`), and `opt_word_mode` already returns `None` there —
//! so the gate below is `is_some()` rather than `== O1`, and that is a
//! measurement and not a widening: `w-xtea3`'s classes are `/O1`-only because
//! their `/Ox` bytes DIFFER, and these do not.

use super::super::expr::{eat_return_plumbing, parse_formals};
use super::super::{blk, BodyShape, Block};
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode};
use crate::func::readers::{eat_byte, eat_opt_stmt_marker, read_token_var, read_type};

/// The width of the store, in bytes — the one thing the emitter reads off the
/// type. Enumerated from GRID T rather than derived from the tag's width
/// nibble, which `readers::read_type`'s own doc records as unreliable
/// (`0x86`, `0xA6`, `0x96`, `0xC6` all occur and mean different things).
///
/// `(tag, kind)` only. The third component is a type **id** (`74` int, `12`
/// long, `81 20` an enum, `83 08` a pointer) and every id sharing a `(tag,
/// kind)` pair emits the same word — `int`, `long` and an `enum` are all
/// `86 41 …` and all `stw`.
const STORE_WIDTHS: &[(u8, u8, u8)] = &[
    (0x82, 0x11, 1), // signed char
    (0x82, 0x12, 1), // unsigned char, and `bool` (id `30`)
    (0x84, 0x21, 2), // short
    (0x84, 0x22, 2), // unsigned short, and `wchar_t` (id `71`)
    (0x86, 0x41, 4), // int, long, an enum
    (0x86, 0x42, 4), // unsigned int, unsigned long
    (0x86, 0x43, 4), // a data pointer
    (0x88, 0x81, 8), // long long
    (0x88, 0x82, 8), // unsigned long long
];

/// The store width for an admitted `(tag, kind)`, or `None`.
///
/// **`(0x86, 0x45)` and `(0x88, 0x85)` are deliberately absent** — `float` and
/// `double`, which GRID T shows emit `stfs f1` / `stfd f1` out of the FP
/// register file. See the module header.
fn store_width(tag: u8, kind: u8) -> Option<u8> {
    STORE_WIDTHS.iter().find(|&&(t, k, _)| t == tag && k == kind).map(|&(_, _, w)| w)
}

/// **The recognizer.** `start` is the first byte after the body's own `53`;
/// `lo` is the `4C 4F 11` body marker; `depth` is the lexical depth the
/// dispatcher reached, which the return plumbing needs.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` without side effects, so a body that declines
/// still reports its dispatch arm's blocker and no census key moves.
pub(crate) fn try_parse_global_store_leaf(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** (board #1638). `/Od` is the
    // only mode whose bytes differ and `opt_word_mode` is already `None` there;
    // see the module header for the five modes that agree.
    if opt_word_mode(opt_word_at(seg)).is_none() {
        return Err(blk(seg, start, "gstore-mode-not-modelled"));
    }
    // **EXACTLY ONE argument register, and it is a formal rather than a
    // `this`.** `parse_params` counts `this` and `parse_formals` does not, so
    // requiring both to be 1 is what excludes a non-static member function —
    // whose value would arrive in r4 — without assuming `parse_params`' own
    // accounting (`params.rs` records the bug where that assumption cost four
    // wrong registers).
    //
    // Cell `G_second` is the measurement behind the count: a second formal
    // moves the stored register to r4, and this class has no field for it.
    let params = parse_params(seg, lo)?;
    let formals = parse_formals(seg, lo)?;
    if params.len() != 1 || formals.len() != 1 || params[0] != formals[0] {
        return Err(blk(seg, start, "gstore-not-exactly-one-formal-in-r3"));
    }

    let mut p = start;
    eat_opt_stmt_marker(seg, &mut p);
    // `26 <tok>` — the destination. Nothing here proves it is a DATA object:
    // `26` is also how a bound `.sy` automatic is designated
    // (`leaf_store::parse_ref_bind_stmt`). The proof is
    // `Bindings::resolve_bss_def` in `bundle::shape_to_function`, which is the
    // only thing entitled to look at a `.gl` record — the same seam
    // `BodyShape::StaticScanLoop` resolves its array through.
    if !eat_byte(seg, &mut p, 0x26) {
        return Err(blk(seg, p, "gstore-destination"));
    }
    let (dest_tok, w) = read_token_var(seg, p).ok_or(blk(seg, p, "gstore-destination-token"))?;
    p += w;
    // `B9 <tok> <TYPE>` — the value, and it must be the formal ITSELF.
    if !eat_byte(seg, &mut p, 0xB9) {
        return Err(blk(seg, p, "gstore-value-is-not-a-bare-load"));
    }
    let (val_tok, w) = read_token_var(seg, p).ok_or(blk(seg, p, "gstore-value-token"))?;
    p += w;
    if val_tok != params[0] {
        return Err(blk(seg, p, "gstore-value-is-not-the-formal"));
    }
    let (tag, kind, _, tw) = read_type(seg, p).ok_or(blk(seg, p, "gstore-value-type"))?;
    let ty = seg.get(p..p + tw).ok_or(blk(seg, p, "gstore-value-type"))?.to_vec();
    p += tw;
    let width = store_width(tag, kind)
        .ok_or_else(|| blk(seg, p, "gstore-value-type-is-not-a-stored-gpr-scalar"))?;
    // **NO CONVERSION.** A `2C` here is cell `G_narrow` — twelve bytes, the same
    // length as an accepted cell, and TWO different words (`lis r10` and
    // `sth 3,0(10)`). Refused before the `32` rather than after it, so the
    // refusal names the conversion and not the store.
    if seg.get(p) == Some(&0x2C) {
        return Err(blk(seg, p, "gstore-value-carries-a-conversion"));
    }
    // `32 <TYPE>` — the indirect store, whose TYPE must RESTATE the value's,
    // byte for byte. Every cell of GRID T does; a body where they differ is one
    // where a conversion has been folded into the store and nothing here has
    // graded its register plan.
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "gstore-not-an-indirect-store"));
    }
    if seg.get(p..p + tw) != Some(&ty[..]) {
        return Err(blk(seg, p, "gstore-store-type-does-not-restate-the-value-type"));
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "gstore-stmt-end"));
    }
    // **EXACTLY ONE statement.** Cell `G_two` is two, and it is not two copies
    // of this body: both `lis`es are hoisted above both stores and the second
    // one takes r10. `eat_return_plumbing` is what enforces the count — a
    // second statement leaves a `26` where the `3A` has to be.
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, depth)?;

    Ok(BodyShape::GlobalStoreLeaf { params, dest_tok, width })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GRID T's own rows, as the table the emitter reads.
    #[test]
    fn the_store_width_table_is_grid_t() {
        assert_eq!(store_width(0x82, 0x11), Some(1));
        assert_eq!(store_width(0x82, 0x12), Some(1));
        assert_eq!(store_width(0x84, 0x21), Some(2));
        assert_eq!(store_width(0x84, 0x22), Some(2));
        assert_eq!(store_width(0x86, 0x41), Some(4));
        assert_eq!(store_width(0x86, 0x42), Some(4));
        assert_eq!(store_width(0x86, 0x43), Some(4));
        assert_eq!(store_width(0x88, 0x81), Some(8));
        assert_eq!(store_width(0x88, 0x82), Some(8));
    }

    /// **The two FP rows are refused**, and the test names them rather than
    /// asserting a generic `None`: GRID T compiled both and both emit out of
    /// the FP register file.
    #[test]
    fn float_and_double_are_refused_by_name() {
        assert_eq!(store_width(0x86, 0x45), None, "float — c2 emits `stfs f1`");
        assert_eq!(store_width(0x88, 0x85), None, "double — c2 emits `stfd f1`");
    }

    /// `volatile unsigned int` is `96 42 80 20` (cell `G_vol`) — a tag this
    /// table has no row for, so the type gate refuses it even before the
    /// no-conversion clause does.
    #[test]
    fn the_volatile_tag_is_not_in_the_table() {
        assert_eq!(store_width(0x96, 0x42), None);
    }
}

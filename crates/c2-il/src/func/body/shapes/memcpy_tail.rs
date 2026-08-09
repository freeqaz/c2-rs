//! **W-XTEA2 — the body that is NOTHING BUT a `memcpy` into the receiver, and
//! which c2 lowers as a TAIL BRANCH.** `?SetKey@XTEABlockEncrypter@@QAAXPBE@Z`,
//! twelve bytes, one of the four blocked bodies of
//! `src/system/utl/EncryptXTEA.cpp`.
//!
//! ```cpp
//!   void XTEABlockEncrypter::SetKey(const unsigned char *uc) { memcpy(mKey, uc, 0x10); }
//! ```
//!
//! ```text
//!   ?SetKey@XTEABlockEncrypter@@QAAXPBE@Z    .text COMDAT, 0x0c B, nrel 1
//!     0000  38630010  addi r3,r3,16      the destination member, IN PLACE in r3
//!     0004  38a00010  li   r5,16         the length
//!     0008  4bfffff8  b    memcpy        REL24 — a TAIL branch, no frame, no `bl`
//! ```
//!
//! **The source costs nothing.** `uc` is formal 1, so it is already in r4, which
//! is `memcpy`'s second argument register — an in-place argument elision, and the
//! reason this body is three words rather than four.
//!
//! # The four cells that fix every word, compiled by this lane
//!
//! `work/w-xtea2/probe/mcpytail.cpp`, real `c2.dll` under wibo at `/O1 /Oi`:
//!
//! ```text
//!   off16   memcpy(k, uc, 0x10)   addi r3,r3,16 · li r5,16 · b memcpy     12 B
//!   len8    memcpy(k, uc, 0x8)    addi r3,r3,16 · li r5,8  · b memcpy     12 B
//!   off0    memcpy(n, uc, 0x10)                   li r5,16 · b memcpy      8 B
//!   freefn  memcpy(d, s, 0x10)                    li r5,16 · b memcpy      8 B
//!   rev     memcpy(uc, k, 0x10)   mr r11,r3 · mr r3,r4 · addi r4,r11,16
//!                                 · li r5,16 · b memcpy                   20 B
//! ```
//!
//! Three readings, and the last is why the class is narrow:
//!
//! 1. **The length is the `li`'s immediate and nothing else moves** (`len8`).
//! 2. **A zero offset emits NO `addi`** (`off0`, `freefn`) — so the `addi` is
//!    conditional on the offset and is not a "materialise the destination" word.
//! 3. **Reversing the direction is a DIFFERENT REGISTER PLAN** (`rev`): the
//!    destination is now formal 1 and the member is the source, so c2 parks the
//!    receiver in r11 and emits five words. This class admits only the direction
//!    in which both operands are already in the registers `memcpy` wants, and
//!    `rev` is its `_neg` cell.
//!
//! # What this shares with `guard_ret_chain`, and what it does not
//!
//! The `40`-intrinsic grammar is [`super::guard_ret_chain`]'s (`w-ifn`), and the
//! selector constant, the argument separator and the call window are read from
//! there rather than restated — `docs/GAPS.md` §6's "one fact, one locator".
//! What is **not** shared is where the minted `memcpy` symbol goes:
//!
//! * `w-ifn`'s user is FRAMED, and `CEILING.md` §11's NC-1 item 7 records its
//!   symbol landing **after that function's `$T` label**, on
//!   `coff::Function::helper_externals`.
//! * this user is a **LEAF and has no `$T` at all**, and the reference obj puts
//!   `memcpy` in the **callee region** — symbol index 17, immediately after
//!   `?SetKey`'s own function symbol at 16 and before the next section symbol at
//!   18 (`work/w-xtea2/ref/xtea.dump`). The probe reproduces it at index 14, and
//!   its three later users mint **no second symbol**.
//!
//! So the placement is a fact about the *user*, not about the name, and the two
//! placements coexist in one crate. `c2_core::comdat` is where this one is
//! stated, beside `w-ifn`'s.
//!
//! # The label channel is already right, and it was measured before the code
//!
//! `LABEL_COUNTER.md` §7.6's in-the-middle grid, `work/w-xtea2/labgrid.py`, cell
//! `x-setkey` at the workload's own `/O1`: **stride 2**, which is the ordinary
//! leaf's 1 plus the TU's one `memcpy` slot. `coff::plan_labels` already charges
//! exactly that pair — `mints_memcpy` once per TU, then 1 for the leaf — so this
//! class needs no `label_slots` arm and takes none. (Cell `x-setkey` reads 2 at
//! `/Ox` as well; the class is still `/O1`-only, on the mode gate below.)

use super::super::expr::{parse_formals, eat_return_plumbing};
use super::super::{blk, BodyShape, Block};
use super::designator::eat_offset_adds;
use super::guard_ret_chain::{
    eat_arg_sep, eat_lit_any, eat_load, MEMCPY_CALL_STEP, MEMCPY_SELECTOR,
};
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{eat_byte, eat_opt_stmt_marker, is_ptr4_kind, read_type, read_varint};

/// One copy operand as this class spells it: a pointer load, an **optional**
/// member designator, and any number of reinterpreting conversions.
///
/// `guard_ret_chain::eat_copy_operand` admits the load and **one** conversion
/// with no designator, which is the shape its own two witnesses carry. This one
/// is wider in exactly two ways, both witnessed above: the destination here is
/// `this + offsetof(mKey)` (a `33 <int> k 27 <PTR>` designator) and it carries
/// **two** conversions (`const void*` then `void*`). It is a separate function
/// rather than a widening of that one, because widening it would move bodies
/// inside a shipped, byte-graded class — board #232's direction.
struct Operand {
    tok: u32,
    off: i32,
}

fn eat_operand(seg: &[u8], p: &mut usize, what: &'static str) -> Result<Operand, Block> {
    let (tok, is_ptr) = eat_load(seg, p, what)?;
    if !is_ptr {
        return Err(blk(seg, *p, "mcpytail-operand-not-a-pointer"));
    }
    // The member designator, when there is one — through
    // [`super::designator::eat_offset_adds`], which is the shared locator for
    // the `27`/`28` run and is what every other consumer of this byte uses.
    // Rolling a second one here is board **#1334**'s finding in the making: five
    // live productions read `27` and their type predicates already disagree on
    // 16 of the 20 pairs any of them admits.
    //
    // `off0` and `freefn` are why it is OPTIONAL and why the emitter keys on the
    // VALUE: a zero offset still carries the designator in the destination
    // position (`33 <int> 0 27 <PTR>`) and carries none in the source position,
    // and both emit the same nothing.
    let off = eat_offset_adds(seg, p)
        .ok_or(blk(seg, *p, "mcpytail-designator"))?
        .0;
    // `addi` takes a signed 16-bit immediate, and a designator that did not fit
    // would be a `lis`/`ori` pair this class has no witness for.
    if !(0..=0x7FFF).contains(&off) {
        return Err(blk(seg, *p, "mcpytail-designator-offset-out-of-range"));
    }
    // The conversions. Zero (`freefn`'s destination), one (`w-ifn`'s witnesses)
    // or two (`?SetKey`'s destination: `const void*` then `void*`). Each is
    // required to be a pointer and to carry no offset of its own, which is
    // `guard_ret_chain::eat_copy_operand`'s clause and is kept.
    let mut convs = 0;
    while eat_byte(seg, p, 0x2C) {
        convs += 1;
        if convs > 2 {
            return Err(blk(seg, *p, "mcpytail-operand-conversion-chain-too-long"));
        }
        let (tag, kind, _, tw) =
            read_type(seg, *p).ok_or(blk(seg, *p, "mcpytail-operand-convert-type"))?;
        if !is_ptr4_kind(tag, kind) {
            return Err(blk(seg, *p, "mcpytail-operand-convert-not-to-a-pointer"));
        }
        *p += tw;
        let k = read_varint(seg, p).ok_or(blk(seg, *p, "mcpytail-operand-convert-value"))?;
        if k != 0 {
            return Err(blk(seg, *p, "mcpytail-operand-convert-has-an-offset"));
        }
    }
    eat_arg_sep(seg, p, what)?;
    Ok(Operand { tok, off })
}

/// **The recognizer.** `start` is the first byte after the body's own `53`; `lo`
/// is the `4C 4F 11` body marker; `depth` is the lexical depth the dispatcher
/// reached, which the return plumbing needs.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` without side effects, so a body that declines
/// still reports its dispatch arm's blocker and no census key moves.
pub(crate) fn try_parse_memcpy_tail(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** (board #1638). Every word
    // above is a `/O1 /Oi` reading, and `/Oi`'s expansion threshold is a
    // different constant at `/Ox` — a body admitted there could be one c2
    // expanded inline instead of calling. Asked before any body byte is read.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "mcpytail-not-o1"));
    }
    let params = parse_params(seg, lo)?;
    // Exactly two argument registers: the destination's base and the source.
    // `?SetKey` reaches this as `this` plus one formal and `freefn` as two
    // formals, and both are the same two registers — which is why the gate is on
    // `params` (r3, r4, …) and not on `parse_formals`.
    if params.len() != 2 {
        return Err(blk(seg, start, "mcpytail-not-exactly-two-argument-registers"));
    }
    // …and no formal beyond them, so `parse_params`' `this` accounting is
    // checked rather than assumed (`params.rs`'s own bug: a formal mapped to the
    // register `this` occupies).
    let formals = parse_formals(seg, lo)?;
    if formals.len() + 1 != params.len() && formals.len() != params.len() {
        return Err(blk(seg, start, "mcpytail-formals-do-not-account-for-params"));
    }

    let mut p = start;
    // **The source-line marker, and it is not optional in practice.** A body
    // written on ONE line carries none between the `LO` and its first token and
    // a body written over several carries `4F 01 <line>` — the members here have
    // no marker and `wx2_free` has two. `expr.rs`'s own note says a probe
    // written one function per line hides this entirely, and the first draft of
    // this recognizer was exactly that probe.
    eat_opt_stmt_marker(seg, &mut p);
    let sel = eat_lit_any(seg, &mut p, "mcpytail-selector")?;
    if sel != MEMCPY_SELECTOR {
        return Err(blk(seg, p, "mcpytail-intrinsic-is-not-memcpy"));
    }
    if !eat_byte(seg, &mut p, 0x40) {
        return Err(blk(seg, p, "mcpytail-not-an-intrinsic-call"));
    }
    let (_, _, _, tw) = read_type(seg, p).ok_or(blk(seg, p, "mcpytail-result-type"))?;
    p += tw;

    // The two alignment hints, read and bounded rather than skipped — for
    // `guard_ret_chain`'s reason: an unmodelled value here is a fact about the
    // copy this class has not graded. The cells carry `(1,8)`, `(1,1)` and
    // `(8,1)`, so the pair varies and only its range is claimed.
    for what in ["mcpytail-align-0", "mcpytail-align-1"] {
        let a = eat_lit_any(seg, &mut p, what)?;
        if !(1..=16).contains(&a) {
            return Err(blk(seg, p, "mcpytail-alignment-out-of-range"));
        }
        eat_arg_sep(seg, &mut p, what)?;
    }

    let len = eat_lit_any(seg, &mut p, "mcpytail-length")?;
    eat_arg_sep(seg, &mut p, "mcpytail-length")?;
    if !(MEMCPY_CALL_STEP..=0x7FFF).contains(&len) {
        // Below the measured step (`w-ifn`'s 25 cells) c2 expands the copy
        // inline and there is no branch at all; above `simm16` the `li` is a
        // pair this class has no witness for.
        return Err(blk(seg, p, "mcpytail-length-outside-the-call-window"));
    }

    // **The IL lists the SOURCE first and the DESTINATION second** — the order
    // `guard_ret_chain::eat_copy` already reads, re-confirmed on all five cells
    // of this lane's probe (in `rev`, the reversed one, the member is the FIRST
    // operand and the formal the second).
    let src = eat_operand(seg, &mut p, "mcpytail-src")?;
    let dst = eat_operand(seg, &mut p, "mcpytail-dst")?;
    if !eat_byte(seg, &mut p, 0x4C) {
        return Err(blk(seg, p, "mcpytail-call-end"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "mcpytail-stmt-end"));
    }

    // **THE REGISTER PLAN, and the whole reason this class is a class.** c2's
    // three words are right only when the destination's base is already in r3
    // and the source is already in r4. `rev` is the same source construct with
    // the two exchanged and it is FIVE words through r11 — so this is a refusal
    // that names its construct, not a limit hidden in a parse failure.
    if dst.tok != params[0] {
        return Err(blk(seg, p, "mcpytail-destination-base-is-not-the-first-argument"));
    }
    if src.tok != params[1] {
        return Err(blk(seg, p, "mcpytail-source-is-not-the-second-argument"));
    }
    // A source with a designator would be `addi r4,r4,k`, which no cell here
    // witnesses in the accepted direction.
    if src.off != 0 {
        return Err(blk(seg, p, "mcpytail-source-carries-a-member-offset"));
    }

    // The void tail. The call's result is discarded (the `4B` above), so there
    // is no result annotation and `has_result_type` is false. The marker before
    // it is the closing brace's own source line — `wx2_free` carries one here
    // too and the one-line members do not.
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, depth)?;

    Ok(BodyShape::MemcpyTail { params, dst_off: dst.off, len })
}

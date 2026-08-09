//! **W-BLOCKIR — the float array-walk counted loop**, the shape
//! `src/system/synth_xbox/IPP_basicmath_xbox.cpp` is made of, all four bodies.
//!
//! ```c
//!   void f(unsigned n, const float *a, float *b) {
//!       if (n == 0)
//!           return;
//!       for (unsigned i = 0; i < n; i++)
//!           b[i] += a[i];              // or  b[i] *= s;  or  c[i] = a[i] * b[i];
//!   }
//! ```
//!
//! # It is a TRANSCRIPTION, drawn around four workload functions
//!
//! [`super::ptr_walk_loop`]'s standard, [`super::static_scan_loop`]'s and
//! [`super::json_utf8_copy`]'s: named sub-shapes, `/O1` only, every cell inside
//! them measured against real `c2.dll` before this file was written
//! (`work/w-blockir/PROBES.md`, 28 cells over two rounds). It is **not** the
//! block IR `docs/CFG_SHAPE.md` §6 specifies — there is no fixup list here, no
//! liveness across a block boundary, no scheduler and no register allocator, and
//! the back edge's displacement is computed directly in the emitter rather than
//! through `c2_core::codegen::labels`, which is the escape hatch every shipped
//! loop class has used and which leaves that map's invariant 4 untouched.
//!
//! The distinction matters for what this file has to do: because the emitter has
//! **no free field** for a word that is not in one of the three sequences, a body
//! that differs anywhere would need a *different word*, so anything admitted by
//! accident is a wrong-bytes emit and not a wider class. Every clause below
//! refuses rather than generalises.
//!
//! # The three sub-shapes, and why they are three
//!
//! ```text
//!   A Compound  b[i] OP= a[i]        mr r11,b · GUARD · mtctr · sub r10,a,b
//!                                    lfsx f0,r10,r11 · lfs f13,0(r11)
//!                                    fOPs f0,f0,f13 · stfs f0,0(r11)
//!                                    addi r11,r11,4 · bdnz .-20 · blr
//!   B Scalar    b[i] OP= s           GUARD · addi r11,b,-4 · mtctr
//!                                    lfs f0,4(r11) · fOPs f0,f0,f1
//!                                    stfsu f0,4(r11) · bdnz .-12 · blr
//!   C Binary    c[i] = a[i] OP b[i]  GUARD · mr r11,b · mtctr
//!                                    sub r10,a,b · sub r9,c,b
//!                                    lfsx f0,r10,r11 · lfs f13,0(r11)
//!                                    fOPs f0,f0,f13 · stfsx f0,r9,r11
//!                                    addi r11,r11,4 · bdnz .-20 · blr
//! ```
//!
//! `GUARD` is `cmplwi cr6,r3,0 · bclr 12,26` in all three — `wb-loop`'s pass 1,
//! the rotated pre-test realised as a conditional return because the loop is the
//! function's tail — and `mtctr`/`bdnz` is its pass 2.
//!
//! **The park's position is a per-shape constant and the rule behind it is
//! NAMED but not claimed.** A `mr` park floats above the guard in A and not in
//! C; what fits all seven measured cells is the number of preheader `sub`s, and
//! `probe/walk.cpp`'s `c5`, `c6` and `d5` refute the register-position rule this
//! lane registered in advance (`work/w-blockir/PREREG.md` §5.2, scored in the
//! rung). Six / four / four witnesses respectively.
//!
//! **The walker is three rules, not one.** `WB_LOOP_FINDINGS.md` §4.3 states the
//! base-difference reduction and then says of the walker *"In all five measured
//! cells the walker is the array whose access is emitted last, which is
//! circular. `#1767`'s rule against a two-point fit applies; not claimed."* This
//! lane's grid is not circular — it varies declaration order, source order,
//! formal count and array count independently — but it still yields three
//! per-shape answers rather than one rule, and they ship as such:
//!
//! * **A** — the walker is the compound assign's own destination;
//! * **B** — there is only one array;
//! * **C** — the walker is the **later-declared** of the two right-hand arrays,
//!   which is why [`RIGHT_HAND_INDICES_MUST_INCREASE`] exists: `c1`
//!   (`c[i] = b[i] * a[i]`) is byte-identical to `Mul` and walks `b` either way,
//!   so a reader keyed on IL order would pick the wrong one. Refusing the
//!   descending spelling costs a body c2 does convert and keeps the accepted set
//!   equal to the graded set.
//!
//! And it does **not** extend past two right-hand arrays: `c4`
//! (`d[i] = a[i]+b[i]+c[i]`) walks the *second* of three and c2 restructures the
//! add tree to get there. That single cell is why clause 12 admits exactly one
//! right-hand operator.
//!
//! # What is OUTSIDE the class, each with a compiled cell rather than a caution
//!
//! | refused | what c2 does instead | cell |
//! |---|---|---|
//! | `-=`, `/=` | swaps the two loads — the walker's `lfs` comes first | `c7`, `c8` |
//! | a **signed** counter or bound | `cmpwi cr6,r3,0` + `bclr 4,25` | `c9` |
//! | `double` arrays | `lfdx`/`lfd`/`fadd`/`stfd`, `addi r11,r11,8` | `c11` |
//! | `int` arrays | `lwzx`/`lwz`/`add`/`stw` — the skeleton generalises | `c14` |
//! | a step other than +1 | preheader trip-count arithmetic (`addi -1`/`srwi`/`addi +1`) | `e3` |
//! | the counter used for anything but the subscript | a second live value and an interleaved schedule | `e1` |
//! | the loop not the function tail | the body continues past `bdnz` | `e2` |
//! | bound ≠ the guard's subject | **two** guards | `e4` |
//! | two statements in the loop body | a second store inside the loop, `bdnz .-24` | `e5` |
//! | `b[i] = s;` (a scalar splat) | a `_blkmov` tail call — not a loop at all | `c13` |
//! | `/Ox` | unrolls 4× behind `cmpwi cr6,r3,4` with a remainder loop, 688 B | `probe/ipp_ox` |
//! | no `if (n == 0) return;` | **byte-identical output** — and a different token stream, which this reader does not admit | `c10` |
//!
//! That last row is worth reading twice. The guard is *redundant in the obj* —
//! c2's `for` rotation needs the zero-trip test anyway and fuses the two — and
//! **load-bearing in the IL**, because the token stream with it and the token
//! stream without it are different. This reader consumes the one it graded.
//!
//! # The mode gate is asked HERE, before any body byte
//!
//! Board #1638: a gate that lives only in the emitter is a fact the census
//! cannot ask, so the census counts a function in class that `PortC2` refuses.
//! `/O1` only, and `/Ox` refuses on its first question.

use crate::func::body::expr::{eat_return_plumbing, parse_formals, BODY_SCOPE_DEPTH};
use crate::func::body::{blk, Block, BodyShape};
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat, eat_byte, is_fp_type, is_int4_type, is_ptr4_kind, read_token_var, read_type, read_varint,
};
use crate::func::{FloatWalkLoop, FloatWalkOp, FloatWalkShape};

/// The lexical depth the dispatcher has reached when this production is called.
///
/// `parse_segment_shape` eats the body's own `53` (depth 2) and then
/// `eat_scopes` greedily eats the **`if` statement's** `53`, so the cursor
/// arrives at the guard's `B9` with the `if` scope open at depth 3. Requiring it
/// exactly is what makes the two literal scope-closes below (`54 04`, `54 03`)
/// a statement about this stream rather than a coincidence.
const ENTRY_DEPTH: usize = BODY_SCOPE_DEPTH + 1;

/// The element scale the class is pinned to: `float` is 4 bytes, and the `addi
/// r11,r11,4` / `4(r11)` displacements are literals in the emitter. `c11`
/// (`double`) is the cell on the other side.
const FLOAT_SCALE: i32 = 4;

/// **Clause 8, named so the module header can point at it.** For
/// [`FloatWalkShape::Binary`] the two right-hand arrays must appear in the IL in
/// **increasing formal order**, because that — and not IL order — is what
/// selects the walker. `probe/walk.cpp`'s `c1` is the descending spelling; it is
/// byte-identical to `Mul` and this reader refuses it.
const RIGHT_HAND_INDICES_MUST_INCREASE: bool = true;

/// Consume `B9 <tok>` and return the token, leaving the cursor on the TYPE.
fn eat_load_tok(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume one **unsigned** 4-byte integer TYPE, returning its id.
///
/// Signedness is required rather than accepted: `int` and `unsigned` differ by
/// exactly one TYPE byte and decide `cmpwi`/`bclr 4,25` against
/// `cmplwi`/`bclr 12,26` — board #1788's fact, in a class that has no field for
/// the other pair (`c9` is the compiled cell).
fn eat_uint_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) if is_int4_type(tag, kind) && (kind & 0x0F) == 0x2 => {
            *p += w;
            Ok(id)
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume one width-4 **pointer** TYPE, returning its id. The id is TU-allocated
/// for a derived type (`const float *` is ≥ 0x1000 and differs per TU), so it is
/// returned for equality comparison only and never whitelisted.
fn eat_ptr_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) if is_ptr4_kind(tag, kind) => {
            *p += w;
            Ok(id)
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume one **single-precision** floating TYPE, in any cv-qualification.
/// `double` refuses here (cell `c11`), because every load, store, arithmetic and
/// step word would differ.
fn eat_float_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_fp_type(tag, kind) == Some(false) => {
            *p += w;
            Ok(())
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume a label operand after its opcode byte has been matched.
fn eat_label(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume the subscript designator `B9 <base> <ptrTYPE> · B9 <idx> <uintTYPE> ·
/// 33 <longTYPE> 4 · 04 · 28 00 00`, returning the base token.
///
/// The idiom is `static_scan_loop`'s, pinned there from a two-line reproducer:
/// `04` is `*` and `28 00 00` is the subscript operator whose two trailing bytes
/// are unread and therefore required **literally**. It produces a *designator*,
/// not a value — which is why the left-hand side of an assignment carries no
/// `30` and the right-hand operands do.
fn eat_subscript(
    seg: &[u8],
    p: &mut usize,
    idx_tok: u32,
    what: &'static str,
) -> Result<u32, Block> {
    let base = eat_load_tok(seg, p, what)?;
    eat_ptr_type(seg, p, what)?;
    if eat_load_tok(seg, p, what)? != idx_tok {
        return Err(blk(seg, *p, what));
    }
    eat_uint_type(seg, p, what)?;
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    // The scale literal's own TYPE is `long` — `IL_EXPR_LAYER.md` §4: subscript
    // offsets are typed `long` where member offsets are `int`. Read through
    // `read_type` rather than matched as three bytes, and only its width is
    // asserted; the id is not whitelisted.
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_int4_type(tag, kind) => *p += w,
        _ => return Err(blk(seg, *p, what)),
    }
    if read_varint(seg, p) != Some(FLOAT_SCALE) {
        return Err(blk(seg, *p, what));
    }
    if !eat(seg, p, &[0x04, 0x28, 0x00, 0x00]) {
        return Err(blk(seg, *p, what));
    }
    Ok(base)
}

/// **The recognizer.** `start` is the cursor the dispatcher left on the guard's
/// `B9`; `lo` is the `4C 4F 11` body marker; `depth` is the lexical depth the
/// dispatcher counted.
///
/// Non-committal on the same terms as every sibling in this arm: its own cursor,
/// `Err` on the first byte outside its grammar, so a body that declines still
/// reports the arm's own blocker (`expr-cmp-eq`) and no census key moves.
///
/// It is placed **LAST** in the `B9`/`33` arm's ladder. That is this lane's
/// FENCE ORDER decision and it is deliberate rather than free: going last makes
/// *"no body any production above accepts today can move"* true **by
/// construction**, which is what `w-bdnz` did in the other arm and what four of
/// the last six lanes got wrong when they argued disjointness instead.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_parse_float_walk_loop(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
    locals: &[u32],
    uint_locals: &[u32],
) -> Result<BodyShape, Block> {
    // **The mode gate, before any body byte** (boards #1638 / #139). `/Ox`
    // unrolls this loop four times behind its own pre-test and emits 688 bytes
    // where `/O1` emits 48; approximating that would be a wrong obj, so it
    // refuses here where the CENSUS can see the refusal too.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "fwalk-opt-mode"));
    }
    // Clause 1 — the guard's `if` scope is open and nothing else is. Both scope
    // closes below are literal, and this is what licenses them.
    if depth != ENTRY_DEPTH {
        return Err(blk(seg, start, "fwalk-entry-depth"));
    }

    let params = parse_formals(seg, lo)?;
    // Clause 2 — three or four formals: the bound, then two or three arrays, or
    // two arrays where the last is an FPR scalar. Fewer cannot be any sub-shape
    // and more is a register map this lane graded no cell of.
    if params.len() < 3 || params.len() > 4 {
        return Err(blk(seg, start, "fwalk-formals-arity"));
    }
    let bound_tok = params[0];
    // Clause 3 — the formals are distinct. An aliased list would make the
    // walker index ambiguous.
    for i in 0..params.len() {
        for j in i + 1..params.len() {
            if params[i] == params[j] {
                return Err(blk(seg, start, "fwalk-formals-alias"));
            }
        }
    }
    let idx_of = |tok: u32| params.iter().position(|&t| t == tok);

    let mut p = start;

    // ---- the GUARD: `if (n == 0) return;` ---------------------------------
    //
    // Clause 4 — the guard's subject is formal 0 and is unsigned. `e4` is the
    // cell where it is a different formal from the bound: c2 emits **two**
    // guards.
    if eat_load_tok(seg, &mut p, "fwalk-guard-load")? != bound_tok {
        return Err(blk(seg, p, "fwalk-guard-not-formal0"));
    }
    let uint_id = eat_uint_type(seg, &mut p, "fwalk-guard-type")?;
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "fwalk-guard-lit"));
    }
    if eat_uint_type(seg, &mut p, "fwalk-guard-lit-type")? != uint_id {
        return Err(blk(seg, p, "fwalk-guard-lit-type"));
    }
    if read_varint(seg, &mut p) != Some(0) {
        return Err(blk(seg, p, "fwalk-guard-lit-not-zero"));
    }
    // `1F` is `==`. The relational opcode is sign-agnostic; the operand TYPEs
    // above are what make this the unsigned compare, and they were required.
    if !eat_byte(seg, &mut p, 0x1F) {
        return Err(blk(seg, p, "fwalk-guard-not-eq"));
    }
    if !eat_byte(seg, &mut p, 0x38) {
        return Err(blk(seg, p, "fwalk-guard-brfalse"));
    }
    let l_skip = eat_label(seg, &mut p, "fwalk-guard-label")?;
    // The then-clause: a bare `return;`, nothing else. `53 · [4F 01 n] · 3A Lret
    // · [4F 01 n] · 54 04 · 29 Lskip · 54 03`.
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "fwalk-then-scope"));
    }
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "fwalk-then-return"));
    }
    let l_ret = eat_label(seg, &mut p, "fwalk-then-return-label")?;
    eat_marks(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, (ENTRY_DEPTH + 1) as u8]) {
        return Err(blk(seg, p, "fwalk-then-scope-close"));
    }
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "fwalk-skip-label"));
    }
    if eat_label(seg, &mut p, "fwalk-skip-label-tok")? != l_skip {
        return Err(blk(seg, p, "fwalk-skip-label-mismatch"));
    }
    eat_marks(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, ENTRY_DEPTH as u8]) {
        return Err(blk(seg, p, "fwalk-if-scope-close"));
    }

    // ---- the `for` scope, then `i = 0` -------------------------------------
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "fwalk-for-scope"));
    }
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x26) {
        return Err(blk(seg, p, "fwalk-ctr-designator"));
    }
    let (ctr_tok, w) = read_token_var(seg, p).ok_or(blk(seg, p, "fwalk-ctr-designator"))?;
    p += w;
    // Clause 5 — **the counter is an automatic local, POSITIVELY, and in the
    // list its own signedness belongs to.** `.sy`'s `uint_locals` is w-bdnz's
    // fourth positive list and this class needs the same one; board #764 and
    // #1984 are the two lanes where `.sy` — not codegen — was the last blocker.
    // Both layers must agree: the `.ex` TYPE below says `unsigned`, and a token
    // that is in `int_locals` instead is a body whose two readers disagree.
    if !uint_locals.contains(&ctr_tok) || locals.contains(&ctr_tok) {
        return Err(blk(seg, p, "fwalk-ctr-not-a-uint-local"));
    }
    if idx_of(ctr_tok).is_some() {
        return Err(blk(seg, p, "fwalk-ctr-is-a-formal"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "fwalk-ctr-init-lit"));
    }
    if eat_uint_type(seg, &mut p, "fwalk-ctr-init-type")? != uint_id {
        return Err(blk(seg, p, "fwalk-ctr-init-type"));
    }
    if read_varint(seg, &mut p) != Some(0) {
        return Err(blk(seg, p, "fwalk-ctr-start-not-zero"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "fwalk-ctr-init-store"));
    }
    if eat_uint_type(seg, &mut p, "fwalk-ctr-init-storetype")? != uint_id {
        return Err(blk(seg, p, "fwalk-ctr-init-storetype"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "fwalk-ctr-init-end"));
    }

    // ---- the ROTATION: `3A Ltest · 29 Lincr · i++ · 29 Ltest` --------------
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "fwalk-entry-jump"));
    }
    let l_test = eat_label(seg, &mut p, "fwalk-entry-jump-tok")?;
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "fwalk-incr-label"));
    }
    let l_incr = eat_label(seg, &mut p, "fwalk-incr-label-tok")?;
    if l_incr == l_test {
        return Err(blk(seg, p, "fwalk-labels-alias"));
    }
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x26) {
        return Err(blk(seg, p, "fwalk-incr-designator"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "fwalk-incr-designator"))?;
    p += w;
    if t != ctr_tok {
        return Err(blk(seg, p, "fwalk-incr-not-the-counter"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "fwalk-incr-lit"));
    }
    if eat_uint_type(seg, &mut p, "fwalk-incr-lit-type")? != uint_id {
        return Err(blk(seg, p, "fwalk-incr-lit-type"));
    }
    // Clause 6 — the step is literally +1 (`e3` is the `i += 2` cell, where c2
    // emits three words of preheader trip-count arithmetic).
    if read_varint(seg, &mut p) != Some(1) {
        return Err(blk(seg, p, "fwalk-step-not-1"));
    }
    // `35` is **postfix `i++`**, and it is not `0F`. The front end does not
    // normalise the two: `++i` folds to `+= 1` = `0F` and `i++` keeps `35`
    // (`docs/IL_STMT_GRAMMAR.md` §5). Both compile to the same `addi`, and this
    // reader admits the spelling it graded — `counted_accum_loop` requires the
    // other one for the same reason, from the other side.
    if !eat_byte(seg, &mut p, 0x35) {
        return Err(blk(seg, p, "fwalk-incr-not-postfix"));
    }
    if eat_uint_type(seg, &mut p, "fwalk-incr-type")? != uint_id {
        return Err(blk(seg, p, "fwalk-incr-type"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "fwalk-incr-end"));
    }

    // ---- the TEST: `29 Ltest · i < n · 38 Lexit` ---------------------------
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "fwalk-test-label"));
    }
    if eat_label(seg, &mut p, "fwalk-test-label-tok")? != l_test {
        return Err(blk(seg, p, "fwalk-test-label-mismatch"));
    }
    if eat_load_tok(seg, &mut p, "fwalk-test-ctr-load")? != ctr_tok {
        return Err(blk(seg, p, "fwalk-test-not-the-counter"));
    }
    if eat_uint_type(seg, &mut p, "fwalk-test-ctr-type")? != uint_id {
        return Err(blk(seg, p, "fwalk-test-ctr-type"));
    }
    // Clause 7 — the bound is a bare load of formal 0 and nothing else. An
    // expression here puts a temporary in the preheader and c2 emits no `bdnz`
    // at all (`wb-loop` §7.4's `a10`).
    if eat_load_tok(seg, &mut p, "fwalk-test-bound-load")? != bound_tok {
        return Err(blk(seg, p, "fwalk-bound-not-formal0"));
    }
    if eat_uint_type(seg, &mut p, "fwalk-test-bound-type")? != uint_id {
        return Err(blk(seg, p, "fwalk-test-bound-type"));
    }
    if !eat_byte(seg, &mut p, 0x22) {
        return Err(blk(seg, p, "fwalk-test-not-lt"));
    }
    if !eat_byte(seg, &mut p, 0x38) {
        return Err(blk(seg, p, "fwalk-test-brfalse"));
    }
    let l_exit = eat_label(seg, &mut p, "fwalk-exit-tok")?;
    if l_exit == l_test || l_exit == l_incr || l_exit == l_skip || l_exit == l_ret {
        return Err(blk(seg, p, "fwalk-labels-alias"));
    }

    // ---- the loop BODY: ONE statement, in a braced scope --------------------
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "fwalk-body-scope"));
    }
    eat_marks(seg, &mut p);
    // The destination designator, always a subscript on a pointer formal.
    let dst_tok = eat_subscript(seg, &mut p, ctr_tok, "fwalk-body-dst")?;
    let dst = idx_of(dst_tok).ok_or(blk(seg, p, "fwalk-body-dst-not-a-formal"))?;
    if dst == 0 {
        return Err(blk(seg, p, "fwalk-body-dst-is-the-bound"));
    }

    // The right-hand side is one of exactly three streams, and which one is
    // decided by the first byte after the destination designator.
    let (shape, op, walker, others) = if seg.get(p) == Some(&0xB9) {
        // Either a second SUBSCRIPT (shapes A and C) or a bare float LOAD
        // (shape B, the FPR scalar). Told apart by the TYPE after the token:
        // a pointer opens a subscript, a float opens the scalar.
        let save = p;
        let a_tok = eat_load_tok(seg, &mut p, "fwalk-body-rhs")?;
        let is_ptr = matches!(read_type(seg, p), Some((tag, kind, _, _)) if is_ptr4_kind(tag, kind));
        if !is_ptr {
            // ---- shape B: `dst[i] OP= s` --------------------------------
            eat_float_type(seg, &mut p, "fwalk-scalar-type")?;
            let s = idx_of(a_tok).ok_or(blk(seg, p, "fwalk-scalar-not-a-formal"))?;
            // Clause 9 — the FPR formal is **LAST**. Every other index in this
            // struct is mapped to a GPR positionally, and a float formal ahead
            // of an array would shift that map. It is also what pins the scalar
            // to `f1`: it is the first float parameter, and float parameters
            // occupy `f1…f13` in float-parameter order.
            if s != params.len() - 1 {
                return Err(blk(seg, p, "fwalk-scalar-not-last-formal"));
            }
            if params.len() != 3 {
                return Err(blk(seg, p, "fwalk-scalar-arity"));
            }
            let op = eat_compound_op(seg, &mut p)?;
            eat_float_type(seg, &mut p, "fwalk-scalar-op-type")?;
            (FloatWalkShape::Scalar, op, dst, Vec::new())
        } else {
            // Rewind and read the whole subscript through the one reader.
            p = save;
            let a_tok = eat_subscript(seg, &mut p, ctr_tok, "fwalk-body-rhs1")?;
            let a = idx_of(a_tok).ok_or(blk(seg, p, "fwalk-body-rhs1-not-a-formal"))?;
            if a == 0 {
                return Err(blk(seg, p, "fwalk-body-rhs1-is-the-bound"));
            }
            if !eat_byte(seg, &mut p, 0x30) {
                return Err(blk(seg, p, "fwalk-body-rhs1-deref"));
            }
            eat_float_type(seg, &mut p, "fwalk-body-rhs1-type")?;
            if seg.get(p) == Some(&0xB9) {
                // ---- shape C: `dst[i] = a[i] OP b[i]` -------------------
                if params.len() != 4 {
                    return Err(blk(seg, p, "fwalk-binary-arity"));
                }
                let b_tok = eat_subscript(seg, &mut p, ctr_tok, "fwalk-body-rhs2")?;
                let b = idx_of(b_tok).ok_or(blk(seg, p, "fwalk-body-rhs2-not-a-formal"))?;
                if b == 0 || b == a || b == dst || a == dst {
                    return Err(blk(seg, p, "fwalk-binary-operand-alias"));
                }
                if RIGHT_HAND_INDICES_MUST_INCREASE && b < a {
                    return Err(blk(seg, p, "fwalk-binary-operands-descending"));
                }
                if !eat_byte(seg, &mut p, 0x30) {
                    return Err(blk(seg, p, "fwalk-body-rhs2-deref"));
                }
                eat_float_type(seg, &mut p, "fwalk-body-rhs2-type")?;
                let op = eat_binary_op(seg, &mut p)?;
                // The plain binary's result is re-annotated before the store:
                // `2C <floatTYPE> 00`. Required literally — `2C` is the
                // conversion opcode and its trailing `00` is unread.
                if !eat_byte(seg, &mut p, 0x2C) {
                    return Err(blk(seg, p, "fwalk-binary-convert"));
                }
                eat_float_type(seg, &mut p, "fwalk-binary-convert-type")?;
                if !eat_byte(seg, &mut p, 0x00) {
                    return Err(blk(seg, p, "fwalk-binary-convert-tail"));
                }
                if !eat_byte(seg, &mut p, 0x32) {
                    return Err(blk(seg, p, "fwalk-binary-store"));
                }
                eat_float_type(seg, &mut p, "fwalk-binary-store-type")?;
                (FloatWalkShape::Binary, op, b, vec![a, dst])
            } else {
                // ---- shape A: `dst[i] OP= a[i]` -------------------------
                if params.len() != 3 {
                    return Err(blk(seg, p, "fwalk-compound-arity"));
                }
                if a == dst {
                    return Err(blk(seg, p, "fwalk-compound-operand-alias"));
                }
                let op = eat_compound_op(seg, &mut p)?;
                eat_float_type(seg, &mut p, "fwalk-compound-op-type")?;
                (FloatWalkShape::Compound, op, dst, vec![a])
            }
        }
    } else {
        return Err(blk(seg, p, "fwalk-body-rhs"));
    };
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "fwalk-body-end"));
    }

    // ---- the BACK EDGE and the EXIT ---------------------------------------
    //
    // Clause 10 — the loop is the function's TAIL, required by consuming the
    // `for` scope's close and then the plumbing immediately. `e2` is the cell
    // where a statement follows the loop: c2 continues past the `bdnz` and the
    // body is 60 bytes rather than 48.
    eat_marks(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, (ENTRY_DEPTH + 1) as u8]) {
        return Err(blk(seg, p, "fwalk-body-scope-close"));
    }
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "fwalk-back-edge"));
    }
    if eat_label(seg, &mut p, "fwalk-back-edge-tok")? != l_incr {
        return Err(blk(seg, p, "fwalk-back-edge-target"));
    }
    eat_marks(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "fwalk-exit-label"));
    }
    if eat_label(seg, &mut p, "fwalk-exit-label-tok")? != l_exit {
        return Err(blk(seg, p, "fwalk-exit-label-mismatch"));
    }
    eat_marks(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, ENTRY_DEPTH as u8]) {
        return Err(blk(seg, p, "fwalk-for-scope-close"));
    }
    // The shared void tail: `3A <lbl> · 54 02 · 29 <lbl> · 4F 12 · 47 54 01
    // 54 00`, and the fail-closed terminal — anything trailing rejects. The
    // return label must be the one the guard's `return;` jumped to, which is
    // what makes the two exits one block rather than two.
    let ret_at = p;
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH)?;
    if !ret_label_is(seg, ret_at, l_ret) {
        return Err(blk(seg, ret_at, "fwalk-return-label-mismatch"));
    }

    Ok(BodyShape::FloatWalkLoop(FloatWalkLoop {
        params,
        shape,
        op,
        walker,
        others,
    }))
}

/// Re-read the `3A <tok>` the plumbing skipped and compare its label with the
/// guard's. `eat_return_plumbing` deliberately does not check the token — it is
/// shared with every other shape — so this class asks the question itself rather
/// than widening a shared reader for one caller.
fn ret_label_is(seg: &[u8], at: usize, want: u32) -> bool {
    let mut q = at;
    if !eat_byte(seg, &mut q, 0x3A) {
        return false;
    }
    matches!(read_token_var(seg, q), Some((t, _)) if t == want)
}

/// Consume a run of `4F 01 <line>` markers.
fn eat_marks(seg: &[u8], p: &mut usize) {
    crate::func::readers::eat_opt_stmt_marker(seg, p);
}

/// The **compound-assign** opcode, `0F` (`+=`) or `11` (`*=`).
///
/// `10` (`-=`) and `12` (`/=`) are absent because they swap the two loads rather
/// than substituting one field — cells `c7` and `c8`, both compiled and read.
/// Every other opcode is outside the class outright.
fn eat_compound_op(seg: &[u8], p: &mut usize) -> Result<FloatWalkOp, Block> {
    match seg.get(*p) {
        Some(0x0F) => {
            *p += 1;
            Ok(FloatWalkOp::Add)
        }
        Some(0x11) => {
            *p += 1;
            Ok(FloatWalkOp::Mul)
        }
        _ => Err(blk(seg, *p, "fwalk-compound-op")),
    }
}

/// The **plain binary** opcode, `02` (`+`) or `04` (`*`). The two operator tables
/// are different and are not an offset apart, which is why this is a second
/// function and not a translation of the one above.
fn eat_binary_op(seg: &[u8], p: &mut usize) -> Result<FloatWalkOp, Block> {
    match seg.get(*p) {
        Some(0x02) => {
            *p += 1;
            Ok(FloatWalkOp::Add)
        }
        Some(0x04) => {
            *p += 1;
            Ok(FloatWalkOp::Mul)
        }
        _ => Err(blk(seg, *p, "fwalk-binary-op")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two operator maps are the sets real `c2` was compiled on, and the
    /// absentees are absent for measured reasons rather than caution.
    #[test]
    fn the_operator_maps_are_the_measured_sets() {
        let mut p = 0;
        assert_eq!(eat_compound_op(&[0x0F], &mut p).unwrap(), FloatWalkOp::Add);
        let mut p = 0;
        assert_eq!(eat_compound_op(&[0x11], &mut p).unwrap(), FloatWalkOp::Mul);
        // `-=` and `/=` SWAP THE TWO LOADS (cells `c7`, `c8`) — a different word
        // order, not a different field.
        for b in [0x10u8, 0x12] {
            let mut p = 0;
            assert!(eat_compound_op(&[b], &mut p).is_err());
        }
        // …and nothing else at all, including the plain-binary codes, which are
        // a DIFFERENT table and must not leak across.
        for b in [0x00u8, 0x02, 0x04, 0x13, 0x15, 0x35, 0xFF] {
            let mut p = 0;
            assert!(eat_compound_op(&[b], &mut p).is_err(), "IL {b:#04x}");
        }
        let mut p = 0;
        assert_eq!(eat_binary_op(&[0x02], &mut p).unwrap(), FloatWalkOp::Add);
        let mut p = 0;
        assert_eq!(eat_binary_op(&[0x04], &mut p).unwrap(), FloatWalkOp::Mul);
        for b in [0x03u8, 0x05, 0x0F, 0x11, 0xFF] {
            let mut p = 0;
            assert!(eat_binary_op(&[b], &mut p).is_err(), "IL {b:#04x}");
        }
    }

    /// The entry depth is the dispatcher's, not a guess: the body's own `53`
    /// makes 2 and `eat_scopes` greedily takes the `if`'s, making 3.
    #[test]
    fn the_entry_depth_is_one_below_the_then_clause() {
        assert_eq!(ENTRY_DEPTH, 3);
        assert_eq!(BODY_SCOPE_DEPTH, 2);
        assert_eq!(FLOAT_SCALE, 4);
        assert!(RIGHT_HAND_INDICES_MUST_INCREASE);
    }

    /// **The mode gate is the FIRST question, before any body byte** — board
    /// #1638. A stream at any optimization word but `/O1` refuses on its own
    /// key, and it refuses even when every later clause would have passed,
    /// which is why the assertion is on the KEY and not on `is_err()`.
    #[test]
    fn the_mode_gate_refuses_before_the_first_body_byte() {
        // A segment with no readable optimization word at all: `opt_word_at`
        // returns `None`, `opt_word_mode` returns `None`, and the production
        // must decline rather than default to `/O1`.
        let seg = [0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x00];
        let err = try_parse_float_walk_loop(&seg, 4, 0, ENTRY_DEPTH, &[], &[]).unwrap_err();
        assert_eq!(err.ctx, "fwalk-opt-mode");
    }

    /// The entry-depth clause is what licenses the two literal scope closes, so
    /// a cursor arriving at any other depth refuses **there** and not later on a
    /// `54 <k>` that happens not to match.
    #[test]
    fn a_cursor_at_the_wrong_depth_refuses_on_its_own_clause() {
        // `4F 1F` + the `/O1` optimization word, so the mode gate passes.
        let mut seg: Vec<u8> = vec![0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00];
        seg.extend_from_slice(&[0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x00]);
        for d in [BODY_SCOPE_DEPTH, ENTRY_DEPTH + 1, ENTRY_DEPTH + 2] {
            let err = try_parse_float_walk_loop(&seg, 11, 7, d, &[], &[]).unwrap_err();
            assert_eq!(err.ctx, "fwalk-entry-depth", "depth {d}");
        }
    }
}

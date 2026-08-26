//! **The static-array scan loop** — `?NextHashPrime@@YAHH@Z`'s shape, and the
//! first body class in this parser whose function **defines** the data it reads.
//!
//! ```c
//!   int P(int i) {
//!       static int a[N] = { …, 0 };
//!       for (int j = 0; a[j] != 0; j++) {
//!           if (a[j] >= i)
//!               return a[j];
//!       }
//!       return i;
//!   }
//! ```
//!
//! # It is a TRANSCRIPTION, drawn around one workload function
//!
//! `ptr_walk_loop`'s standard and `if_call_join`'s: one named class, `/O1` only,
//! every cell inside it measured. The emitted body
//! (`c2_core::codegen::static_scan_loop`) has **zero free immediate fields** —
//! sixteen words, every register and every displacement fixed — so this
//! recognizer is not choosing between lowerings. It is deciding whether a token
//! stream is *this* program, and every clause below refuses rather than
//! generalizes.
//!
//! That asymmetry is why the fence is where the work is. `ptr_walk_loop` can
//! afford a literal to vary because its emitter has a field for one; here a body
//! that differs anywhere would need a **different word**, so anything this file
//! admits by accident is a wrong-bytes emit and not a wider class.
//!
//! # The subscript idiom, pinned before it was relied on
//!
//! ```text
//!   26 <base> · b9 <index> <TYPE> · 33 <86 41 12> <scale> · 04 · 28 00 00 · 30 <TYPE>
//! ```
//!
//! Lane `w-cfg2` captured this from a **two-line** reproducer
//! (`int g[8]; int f(int i){ return g[i]; }`) and it comes back byte-for-byte
//! identical to all three occurrences in `Primes.cpp`, `28 00 00` included
//! (`work/w-cfg2/PRIMES_BODY.md` §3). Two opcodes it pins on the way:
//! **`0x04` is `*`** and `0x28` is the subscript's own operator with a `varU` of
//! 0. So the production is graded on a cell that has no loop in it at all.
//!
//! # What is OUTSIDE the class, and each has a reason rather than caution
//!
//! * **any element type but `int`** — the scale is required to be `4` and the
//!   element TYPE to be int-like. A `short` array is `slwi …,1` and an `lhz`;
//!   the emitter has no field for either;
//! * **any relation but `>=`** (`23`) in the guard, and any test but `!= 0`
//!   (`20` against the literal `0`) at the loop head — both decide a `bf` bit
//!   number, and both bits are constants in the emitter;
//! * **more than one formal**, or a non-`int` one. Everything about the register
//!   plan is measured at arity 1 (`r3` is the formal, `r9`/`r10`/`r11` are free);
//! * **an array that is not a function-local `static`** — the object must be
//!   COMDAT (`gl::DATA_ATTR_COMDAT`) and **initialized**. A namespace-scope
//!   `static` is a non-COMDAT `.data` placed *before* `.text`
//!   (lane `w-cfg2`'s GRID A cell `a4`, board #1682), which is a different
//!   section order; an uninitialized one is a `.bss` COMDAT (cell `a3`), which
//!   this lane graded no cell of;
//! * **an object whose `.in` initializer does not decode to exactly its `.gl`
//!   size** — refused rather than zero-filled, which is `IlBundle::data_tu`'s
//!   clause 7 in this class.
//!
//! # The copied-clause trap, named because it is the one that costs a day
//!
//! Board **#1636**: `ptr_walk_loop`'s accumulator clause is
//! `locals.contains(&acc) && ptr_locals.contains(&acc)`, correct there because
//! its two variables are one `int` and one pointer, and **vacuously false** in a
//! class whose variable is only one of the two. The induction variable here
//! (`j`) is a plain `int` local, so `locals` is the right list and `ptr_locals`
//! is the wrong one — and this file never mentions `ptr_locals` at all.

use crate::func::body::expr::{eat_return_plumbing, parse_formals};
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::body::{blk, Block, BodyShape};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_opt_stmt_marker, read_token_var, read_type, read_varint,
};
use crate::func::StaticScanLoop;

/// The scope depth the body opens at, mirrored from `expr::BODY_SCOPE_DEPTH`.
/// PROV[N] derived — mirrored from `expr::BODY_SCOPE_DEPTH`.
const BODY_SCOPE_DEPTH: usize = 2;

/// The element scale the emitter's `slwi rA,rS,2` encodes. Pinned as a named
/// constant because it is the one number in the token stream that could change
/// without changing the stream's *shape*, and the shift field is a literal in
/// `c2_core::codegen::static_scan_loop`.
/// PROV[S] `sizeof(int)` is 4 on this ABI. The `slwi rA,rS,2` word that encodes it is c2's choice of instruction; the scale is the language's. Named because it is the one number in the token stream that could change without changing the stream's shape.
const INT_SCALE: i32 = 4;

/// Consume `26 <tok>` and return the token.
fn eat_designator(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume `B9 <tok> <int TYPE>` and return the token.
fn eat_int_load(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    if !eat_int_like(seg, p) {
        return Err(blk(seg, *p, what));
    }
    Ok(tok)
}

/// **The SUBSCRIPT IDIOM**, whole: `a[j]` as an r-value, seven tokens.
///
/// ```text
///   26 <array> · b9 <index> <int TYPE> · 33 <TYPE> <scale> · 04 · 28 00 00 ·
///   30 <int TYPE>
/// ```
///
/// Both tokens are checked against the ones the caller already bound, so the
/// three occurrences in one body cannot silently name three different objects.
/// The scale is required to be [`INT_SCALE`] and the loaded type int-like: those
/// two facts together are what make `slwi …,2` and `lwzx` the right words, and
/// admitting either without the other is the one-field-two-facts defect
/// `docs/GAPS.md` §6 keeps recording.
fn eat_subscript(
    seg: &[u8],
    p: &mut usize,
    array_tok: u32,
    index_tok: u32,
    what: &'static str,
) -> Result<(), Block> {
    if eat_designator(seg, p, what)? != array_tok {
        return Err(blk(seg, *p, what));
    }
    if eat_int_load(seg, p, what)? != index_tok {
        return Err(blk(seg, *p, what));
    }
    // `33 <TYPE> <scale>` — the element size. Its TYPE is the pointer-arithmetic
    // type (`86 41 12` in the workload's own capture), read for its width and
    // not classified, exactly as `ptr_walk_loop` reads its stride literal.
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    match read_type(seg, *p) {
        Some((_, _, _, w)) => *p += w,
        None => return Err(blk(seg, *p, what)),
    }
    if read_varint(seg, p) != Some(INT_SCALE) {
        return Err(blk(seg, *p, what));
    }
    // `04` MULTIPLY, then the subscript operator itself. `28 00 00` is a `varU`
    // of 0 and is required literally: it is the only form any capture of this
    // idiom has ever produced.
    if !eat(seg, p, &[0x04, 0x28, 0x00, 0x00]) {
        return Err(blk(seg, *p, what));
    }
    // `30 <int TYPE>` — the indirection that makes it an r-value. Required to be
    // int-like: this is the fact that says the emitted load is `lwzx` and not
    // `lhzx` or `lbzx`.
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, what));
    }
    if !eat_int_like(seg, p) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// Consume `54 <n>` — one scope close.
fn eat_close(seg: &[u8], p: &mut usize, n: u8, what: &'static str) -> Result<(), Block> {
    eat_opt_stmt_marker(seg, p);
    if !eat(seg, p, &[0x54, n]) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// **The recognizer.** `start` is the first byte after the body's own `53` and
/// any leading scope/line markers; `lo` is the `4C 4F 11` body marker.
///
/// Non-committal in the sense every sibling production here is: its own cursor,
/// `Err` on the first byte that is not its grammar, no side effect on decline.
pub(crate) fn try_parse_static_scan_loop(
    seg: &[u8],
    start: usize,
    lo: usize,
    locals: &[u32],
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not only in the emitter.**
    //
    // The sixteen words are a `/O1` body and nothing else: this lane graded no
    // cell at any other mode, and the workload compiles `/O1`. The emitter
    // re-asserts it (`c2_core::codegen::static_scan_loop`) because
    // `select_function` is what `function_gate` runs — but a gate that lived
    // ONLY there makes the **census** count these bodies in class while
    // `PortC2` refuses them, an error term on the published coverage numerator.
    //
    // **That is not a hypothetical here.** This class shipped with the clause in
    // the emitter alone and `crates/c2-harness/tests/census_gate.rs` failed on it
    // in exactly the words it was written to fail in — three fixture functions
    // counted in class at `/Ox` and refused by the port. Board **#1638**,
    // w-cfgclass §5.3, second instance, and `docs/GAPS.md` §6's remedy applied:
    // *move the gate into the IL parser*.
    //
    // Asked FIRST, before any body byte is read, so the refusal cannot depend on
    // how far the walk got.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "scan-not-o1"));
    }
    let params = parse_formals(seg, lo)?;
    // **Exactly one `int` formal.** It lands in `r3` and stays there — the
    // FALLOUT block's `return i` emits *no instruction at all*, which is only
    // true at this arity. A second formal re-plans every register in the body.
    if params.len() != 1 {
        return Err(blk(seg, start, "scan-formals-not-1"));
    }
    let formal = params[0];

    let mut p = start;

    // ---- statement 1: `j = 0` ---------------------------------------------
    let idx_tok = eat_designator(seg, &mut p, "scan-index-designator")?;
    // An automatic local `.sy` knows about — the same membership test
    // `assign.rs` uses, and board #1636's list. `j` is a plain `int`, so this is
    // `locals` and never `ptr_locals`.
    if !locals.contains(&idx_tok) {
        return Err(blk(seg, p, "scan-index-not-a-local"));
    }
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "scan-index-init-lit"));
    }
    // **Zero, literally.** `li r11,0` is the emitted word and the emitter has no
    // field for another value.
    if read_varint(seg, &mut p) != Some(0) {
        return Err(blk(seg, p, "scan-index-init-not-zero"));
    }
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "scan-index-init-store"));
    }

    // ---- the ROTATION: `3A Ltest` · `29 Lincr` · `j += 1` · `29 Ltest` -----
    //
    // The IL jumps **over** the increment into the test, so the increment block
    // physically precedes the test block and the source's single test site is
    // emitted twice. Identical in shape to `ptr_walk_loop`'s, which is the
    // reading `work/w-cfg2/PRIMES_BODY.md` §4 records.
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "scan-entry-jump"));
    }
    let (l_test, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-entry-jump-tok"))?;
    p += w;
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "scan-incr-label"));
    }
    let (l_incr, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-incr-label-tok"))?;
    p += w;
    if l_incr == l_test {
        return Err(blk(seg, p, "scan-labels-alias"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_designator(seg, &mut p, "scan-incr-designator")? != idx_tok {
        return Err(blk(seg, p, "scan-incr-not-the-index"));
    }
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "scan-incr-lit"));
    }
    // Stride 1 — `addi r11,r11,1`. Any other stride is a different immediate in
    // a word the emitter spells as a constant.
    if read_varint(seg, &mut p) != Some(1) {
        return Err(blk(seg, p, "scan-incr-stride-not-1"));
    }
    if !eat_byte(seg, &mut p, 0x35) || !eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "scan-incr-op"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "scan-test-label"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-test-label-tok"))?;
    p += w;
    if t != l_test {
        return Err(blk(seg, p, "scan-test-label-mismatch"));
    }

    // ---- the TEST: `a[j] != 0` · `38 Lexit` --------------------------------
    //
    // The array token is bound HERE, at its first occurrence, and every later
    // occurrence is checked against it.
    let array_tok = eat_designator(seg, &mut p, "scan-array-designator")?;
    // **The array must NOT be an automatic local.** It is a `static`, i.e. a
    // memory object with a symbol, and a stack array would make the whole
    // relocation half of this class meaningless.
    if locals.contains(&array_tok) || array_tok == idx_tok || array_tok == formal {
        return Err(blk(seg, p, "scan-array-is-a-local"));
    }
    // Rewind to the designator and consume the whole idiom through the shared
    // helper, so the three occurrences cannot be spelled three ways.
    p = start_of_designator(seg, p, array_tok)?;
    eat_subscript(seg, &mut p, array_tok, idx_tok, "scan-test-subscript")?;
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "scan-test-lit"));
    }
    if read_varint(seg, &mut p) != Some(0) {
        return Err(blk(seg, p, "scan-test-not-vs-zero"));
    }
    // `20` — the NE relation, required literally. Its siblings decide a
    // different `bf` bit and the emitter spells `26` (CR6.EQ) as a constant.
    if !eat_byte(seg, &mut p, 0x20) {
        return Err(blk(seg, p, "scan-test-not-ne"));
    }
    if !eat_byte(seg, &mut p, 0x38) {
        return Err(blk(seg, p, "scan-test-brfalse"));
    }
    let (l_exit, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-exit-tok"))?;
    p += w;
    if l_exit == l_test || l_exit == l_incr {
        return Err(blk(seg, p, "scan-labels-alias"));
    }

    // ---- the GUARD: `{ if (a[j] >= i) …` -----------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "scan-for-body-scope"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "scan-if-scope"));
    }
    eat_subscript(seg, &mut p, array_tok, idx_tok, "scan-guard-subscript")?;
    if eat_int_load(seg, &mut p, "scan-guard-formal")? != formal {
        return Err(blk(seg, p, "scan-guard-not-the-formal"));
    }
    // `23` — GE, and the operand order matters: `a[j] >= i`, not `i <= a[j]`.
    // The emitted `cmpw cr6,r10,r3` reads the array element first and the branch
    // takes CR6.LT false; a swap is a different word.
    if !eat_byte(seg, &mut p, 0x23) {
        return Err(blk(seg, p, "scan-guard-not-ge"));
    }
    if !eat_byte(seg, &mut p, 0x38) {
        return Err(blk(seg, p, "scan-guard-brfalse"));
    }
    let (l_cont, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-cont-tok"))?;
    p += w;
    if l_cont == l_test || l_cont == l_incr || l_cont == l_exit {
        return Err(blk(seg, p, "scan-labels-alias"));
    }

    // ---- the VALUE return: `return a[j];` ----------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "scan-value-scope"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_subscript(seg, &mut p, array_tok, idx_tok, "scan-value-subscript")?;
    // `41 <int>` RETURN VALUE, then `3A <Lret>` — the early return out of the
    // loop body. Its target is the function's shared return label, which
    // `eat_return_plumbing` names at the end.
    if !eat_byte(seg, &mut p, 0x41) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "scan-value-return"));
    }
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "scan-value-jump"));
    }
    let (l_ret, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-value-jump-tok"))?;
    p += w;

    // ---- close out to the back edge ----------------------------------------
    eat_close(seg, &mut p, (BODY_SCOPE_DEPTH + 4) as u8, "scan-close-6")?;
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "scan-cont-label"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-cont-label-tok"))?;
    p += w;
    if t != l_cont {
        return Err(blk(seg, p, "scan-cont-label-mismatch"));
    }
    eat_close(seg, &mut p, (BODY_SCOPE_DEPTH + 3) as u8, "scan-close-5")?;
    eat_close(seg, &mut p, (BODY_SCOPE_DEPTH + 2) as u8, "scan-close-4")?;
    // **THE BACK EDGE.** `c2_core::codegen::labels` invariant 4 refuses a
    // backward *reference through the label map*; this class never asks it, and
    // computes the displacement from the block layout directly, exactly as
    // `ptr_walk_loop` and `ptr_walk_chain_loop` do.
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "scan-back-edge"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-back-edge-tok"))?;
    p += w;
    if t != l_incr {
        return Err(blk(seg, p, "scan-back-edge-target"));
    }

    // ---- the EXIT: `29 Lexit` · `return i` ---------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "scan-exit-label"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "scan-exit-label-tok"))?;
    p += w;
    if t != l_exit {
        return Err(blk(seg, p, "scan-exit-label-mismatch"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_int_load(seg, &mut p, "scan-return-load")? != formal {
        return Err(blk(seg, p, "scan-return-not-the-formal"));
    }
    // The shared tail closes the remaining scopes and names the return label —
    // the same `3A <l_ret>` the value arm jumped to, which is checked below.
    let before_tail = p;
    eat_return_plumbing(seg, &mut p, true, BODY_SCOPE_DEPTH + 1)?;
    // **The two returns must share one label.** The value arm's `3A <l_ret>` and
    // the fall-out's are the same block in the obj — one `blr` each, at `0x30`
    // and `0x3c` — and a body whose arms returned to different labels would be a
    // different block plan. Read back out of the tail rather than trusted.
    let (t, _) = tail_return_label(seg, before_tail).ok_or(blk(seg, p, "scan-return-label"))?;
    if t != l_ret {
        return Err(blk(seg, p, "scan-return-labels-differ"));
    }

    Ok(BodyShape::StaticScanLoop(StaticScanLoop {
        params,
        array_tok,
        scale: INT_SCALE,
    }))
}

/// Rewind `p` — which sits just past a `26 <tok>` — back to that designator's
/// own `26`.
///
/// The array's designator is read once to *bind* the token and once to consume
/// the idiom, and the second read goes through [`eat_subscript`] so all three
/// occurrences share one text. Computed from the token's own encoded width
/// rather than by remembering a cursor, so the two cannot disagree.
fn start_of_designator(seg: &[u8], p: usize, tok: u32) -> Result<usize, Block> {
    for w in [2usize, 4] {
        if p < w + 1 {
            continue;
        }
        let q = p - w - 1;
        if seg.get(q) == Some(&0x26) {
            if let Some((t, got)) = read_token_var(seg, q + 1) {
                if t == tok && got == w {
                    return Ok(q);
                }
            }
        }
    }
    Err(blk(seg, p, "scan-array-designator-rewind"))
}

/// The label token of the `3A <tok>` that opens the shared return tail.
fn tail_return_label(seg: &[u8], mut p: usize) -> Option<(u32, usize)> {
    // `41 <TYPE>` then `3A <tok>` — [`eat_return_plumbing`]'s own head, read for
    // its token only.
    if seg.get(p) == Some(&0x41) {
        p += 1;
        let (_, _, _, w) = read_type(seg, p)?;
        p += w;
    }
    if seg.get(p) != Some(&0x3A) {
        return None;
    }
    read_token_var(seg, p + 1)
}

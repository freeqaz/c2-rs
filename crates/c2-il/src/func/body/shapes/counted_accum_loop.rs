//! **W-BDNZ — the counted-`for` accumulate loop**, and the first class in this
//! parser that ships a *generator-derived* lowering rather than a transcription
//! of one workload function.
//!
//! ```c
//!   int f(int n, int k) {
//!       int s = INIT;
//!       for (C i = 0; i < n; ++i)     // C = int or unsigned
//!           s OP= k;                  // OP in { -=, *=, &=, |=, ^=, <<=, >>= }
//!       return s;
//!   }
//! ```
//!
//! # What it is: two of `wb-loop`'s THREE composable passes
//!
//! `docs/whitebox/WB_LOOP_FINDINGS.md` read c2's counted-loop lowering as three
//! **independent** passes, each with its own `-QX` disable switch and each
//! leaving the other two's output byte-identical (§7.7, obj-confirmed by two
//! counterfactual runs):
//!
//! ```text
//!   1  the rotated pre-test GUARD          lur.c              -NoLUR
//!   2  the mtctr/bdnz CONVERSION           p2\ppc\lower.c     -QXnobdnz
//!   3  the lwzu/stwu UPDATE FORM           p2\misc.c          -QXnopreinc
//! ```
//!
//! This class is passes **1 and 2 and not 3**, which is exactly the increment
//! §9 item 1 says a port can ship and be byte-correct on: *"shipping only rule 1
//! + rule 2 reproduces c2's obj exactly for every loop where the update form
//! does not apply"*. Pass 3 is **declined by name**, and the decline is a
//! measurement rather than caution — `wb-loop` §4.4/§7.5 put four rivals on a
//! frozen 10-cell grid and **elected none**: RU0′ and RU2 both score 8/10 with
//! *disjoint* failures, RU0′-b is retracted, and RU-H is filed explicitly
//! unfrozen. **This class contains no memory reference at all**, which is the
//! largest boundary that is provably outside that undecided question: a body
//! with no load and no store cannot be a cell of it.
//!
//! # The emitted form, read off real `c2`
//!
//! ```text
//!     mr     r11, r3
//!     li     r3, INIT
//!     cmp{w,lw}i cr6, r11, 0        <- pass 1: the rotated pre-test
//!     bclr   {4,25 | 12,26}         <- realised as a CONDITIONAL RETURN
//!     mtctr  r11                    <- pass 2: the trip count
//!     <OP>   r3, r3, r4
//!     bdnz   .-4                    <- pass 2: the latch
//!     blr
//! ```
//!
//! `wb-loop` §3 predicted the guard's *form* — "the START expression against the
//! BOUND, with the loop's own signedness, in `cr6`; a conditional RETURN when
//! the loop is the function tail" — and every cell here confirms it. Note what
//! that rules out: it is **not** a `trip_count > 0` test, and the four branch
//! conditions in `work/w-bdnz/probe/L4.obj` (`bclr 4,25`, `bclr 12,26`,
//! `bclr 12,24`, and `bf 25` when the loop is not the tail) all fall out of the
//! one rule. This class ships two of the four and refuses the others by name.
//!
//! # The boundary: `wb-loop`'s eight clauses, and six more this PORT needs
//!
//! Each row names the cell that exercises it. Every cell is in
//! `fixtures/cpp/wbdnz_ctr.cpp` or `fixtures/cpp/wbdnz_ctr_neg.cpp` and was
//! compiled by real `c2.dll` under wibo before this file was written.
//!
//! ```text
//!  wb-loop §5 clause                     the NEGATIVE cell, and what c2 does
//!  1 single back edge                    n_cont   c2 CONVERTS (bdnz + an inner
//!                                                 cmpwi/bt); the port refuses
//!  2 single exit                         n_break  c2 emits addic./bf 2, no bdnz
//!  3 32-bit integer local counter        n_i64    c2 emits cmpd/bt 24, no bdnz
//!  4 constant step, and here REQUIRED    n_step2  c2 CONVERTS with addi -1 /
//!    to be +1                                     srwi 1 / addi +1 -- the
//!                                                 trip-count arithmetic, which
//!                                                 wb-loop §9 item 4 leaves
//!                                                 UNREAD; the port refuses
//!                                        n_stepv  c2 emits no bdnz
//!  5 loop-invariant SYMBOL bound         n_bexpr  srawi/addze/addic., no bdnz
//!  6 counter used ONLY by the compare    n_ctru   c2 CONVERTS and keeps a
//!                                                 second addi r11,r11,1
//!  7 no call / computed branch /         n_call   framed, bl, no bdnz
//!    CTR-taking inner loop               n_nest   inner takes CTR, OUTER gets
//!                                                 addic./bf 2
//!  8 the body is ONE basic block         covered by n_cont and n_break
//!
//!  and this port's own, each forced by a measured RE-PLAN:
//!  9 the bound is formal SLOT 0          n_swap   accumulator cannot coalesce
//!                                                 into r3: it takes r11, the
//!                                                 guard becomes a FORWARD bf 25
//!                                                 and a closing mr r3,r11
//!                                                 appears
//! 10 the loop is the function TAIL       n_after  same re-plan, from the other
//!                                                 direction
//! 11 the operand is a FORMAL             n_litop  mulli/xori -- an immediate
//!                                                 form per opcode, and `andi.`
//!                                                 writes CR0
//! 12 OP is neither += nor /=             n_addop  += DELETES THE LOOP (mullw,
//!                                                 guard and all)
//!                                        n_divop  rotlwi/divw/twi/twi
//! 13 INIT fits simm16                    n_initover  lis/ori -- AND c2 puts the
//!                                                 guard compare BETWEEN them
//! 14 exactly two int-like formals        n_three  c2's text is byte-identical;
//!                                                 the port refuses anyway
//! ```
//!
//! # The signedness fence is the sharpest thing in this file (board #1788)
//!
//! `int i` and `unsigned i` differ in the IL by **exactly one TYPE byte** —
//! `86 41 74` against `86 42 75` — and the relational opcode (`22`) and the
//! branch (`38`) are byte-identical. `readers::eat_int_like` accepts **both**,
//! by design and for good reasons stated at its own definition. A recognizer
//! built on it here would emit `cmpwi`/`bclr 4,25` into an obj that has
//! `cmplwi`/`bclr 12,26`: four wrong bytes, in a body that is otherwise perfect.
//!
//! So every int-like TYPE in the counter's chain is read through
//! [`read_type`] and its **signedness nibble** is required to agree across the
//! declaration, the `+= 1`, and both operands of the compare — and the answer is
//! carried to the emitter in [`CountedAccumLoop::counter_unsigned`] rather than
//! re-derived there. The accumulator's own signedness is checked separately and
//! for a different reason: `>>=` is `sraw` on a signed accumulator and `srw` on
//! an unsigned one, so an unsigned accumulator is refused outright (there is no
//! field for the second word and no cell that grades it).
//!
//! # Two modes, and both are graded
//!
//! Unlike [`super::ptr_walk_loop`] and [`super::static_scan_loop`], which refuse
//! `/Ox` because their lanes graded no `/Ox` cell, this class accepts `/O1`
//! **and** `/Ox` — measured: `work/w-bdnz/probe/L5ox.obj` shows `/Ox` emitting
//! the identical eight words for both the signed and the unsigned cell (packed
//! into one `.text` instead of two COMDATs, which is section layout, not
//! codegen). Every cell of the fixture is graded at both, and the mode gate is
//! asked **here, before any body byte**, because board #1638's defect is a gate
//! that lives only in the emitter.
//!
//! # The `/Ox` arm was graded on ONE AXIS, and it is graded on the CROSS now
//!
//! *Lane `w-counted`, 2026-08-15.* "Every cell of the fixture is graded at
//! both" was true and it was not the same claim as "the accepted set is graded
//! at both". The class's free axes are **accumulate opcode × counter
//! signedness**; the fixture graded all seven opcodes on a *signed* counter and
//! exactly **one** unsigned cell, so **six of the fourteen crossed cells had
//! never been compiled at any mode** — which is what `w-slots`'
//! found-and-not-taken #5 meant by *"whether its `/Ox` acceptance is even
//! correct appears UNGRADED"*, and it was a fair reading of the record.
//!
//! **It is graded now and it is correct.** 20 of 20 in-class cells `match` at
//! `/O1`, `/Ox`, `/Ox /Gy`, `/Ox /EHsc /GR`, `/O2` and the workload's own
//! `/O1 /Oi /EHsc /GR` — **120 gradings against real `c2.dll`, `mismatch` 0** —
//! with a `+=` cell *outside* the class refusing at every one
//! (`work/w-counted/cross_grid.txt`). The nine missing cells are now rows of
//! `fixtures/cpp/wbdnz_ctr.cpp`, so they ride all 18 gate lanes rather than one
//! lane's scratch, and `work/w-counted/codegen_mutants.sh` shows the grid can go
//! red: making the guard ignore [`CountedAccumLoop::counter_unsigned`] reddens
//! **exactly** the ten unsigned cells at both modes and leaves the ten signed
//! ones `match`.
//!
//! **`/O2` is a third lane family on this arm and it was never named.**
//! `/O2`'s optimization word is `OPT_WORD_OX` byte for byte
//! (`docs/OPT_MODE.md` §3), so `Some(OptWordMode::Ox)` above admits **eight** of
//! the 18 gate lanes, not six. Everything the `/Ox` half of this gate does, it
//! does at `/O2` as well — including the label charge, which is 3 at both
//! (`crates/c2-il/src/func/mod.rs`'s `label_slots` arm for this class).
//!
//! **Narrowing this gate to `/O1` is a −8 fixture-verdict move**, measured over
//! all 18 lanes and 381 fixtures (`work/w-counted/narrow_probe.sh`), and −2 even
//! when paired with the `/O1` label charge it would unlock. `differential.rs`'s
//! `differential_wbdnz_ctr_ox_accepted` pins the `/Ox` pole so a later lane
//! narrowing this line reddens a test instead of quietly withdrawing byte-exact
//! output.

use crate::func::body::expr::{eat_return_plumbing, parse_formals};
use crate::func::body::{blk, Block, BodyShape};
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat, eat_byte, eat_opt_stmt_marker, is_int4_type, read_token_var, read_type, read_varint,
};
use crate::func::{CountedAccumLoop, CountedAccumOp};

/// The scope depth the body opens at, mirrored from `expr::BODY_SCOPE_DEPTH`.
/// PROV[N] derived — mirrored from `expr::BODY_SCOPE_DEPTH`, which carries the provenance. DISCLOSURE lists values derived from another marked constant under [N].
const BODY_SCOPE_DEPTH: usize = 2;

/// The IL compound-assignment opcode of an accepted accumulate, or `None`.
///
/// Positive by construction, and the two absentees are the point:
/// **`0x0F` (`+=`) is not here because c2 deletes the loop for it** — the
/// accumulation strength-reduces to a single `mullw`, guard and all
/// (`work/w-bdnz/probe/L3.obj`, cell `op_add`) — and `0x12` (`/=`) is not here
/// because it is a different spine with two `twi` traps in the body (cell
/// `op_div`). Neither is caution: both were compiled and read.
fn accum_op(b: u8) -> Option<CountedAccumOp> {
    match b {
        0x10 => Some(CountedAccumOp::Sub),
        0x11 => Some(CountedAccumOp::Mul),
        0x15 => Some(CountedAccumOp::Shl),
        0x16 => Some(CountedAccumOp::Sar),
        0x17 => Some(CountedAccumOp::And),
        0x18 => Some(CountedAccumOp::Xor),
        0x19 => Some(CountedAccumOp::Or),
        _ => None,
    }
}

/// One int-like TYPE, returned as `(id, unsigned)` — **the signedness is
/// returned rather than discarded**, which is the whole difference between this
/// and [`crate::func::readers::eat_int_like`] and the whole of board #1788 in
/// this class.
fn eat_int_type_signed(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, bool), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) if is_int4_type(tag, kind) => {
            *p += w;
            Ok((id, (kind & 0x0F) == 0x2))
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume `26 <tok>` and return the token.
fn eat_designator(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume `B9 <tok> <int TYPE>` and return `(token, unsigned)`.
fn eat_int_load(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, bool), Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    let (_, uns) = eat_int_type_signed(seg, p, what)?;
    Ok((tok, uns))
}

/// Consume a label operand `<op> <tok>` where `op` has already been matched.
fn eat_label(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// **The recognizer.** `start` is the first byte after the body's own `53` and
/// any leading scope/line markers; `lo` is the `4C 4F 11` body marker.
///
/// Non-committal in the sense every sibling production in this arm is: it works
/// on its own cursor and every failure returns `Err`, so a body that declines
/// still reports the blocker its own dispatch arm found and no census key moves.
///
/// It is placed **LAST** in the ladder's `0x26` arm — see the call site. That is
/// this lane's FENCE ORDER decision and it is deliberate: the four existing loop
/// classes each argue disjointness at their second statement, and this class's
/// second statement is one `53` away from `xlrc_create_guard`'s stated
/// separator. Going last makes "no body an earlier production accepts today can
/// move" true **by construction** rather than by an argument this lane has not
/// proved.
pub(crate) fn try_parse_counted_accum_loop(
    seg: &[u8],
    start: usize,
    lo: usize,
    locals: &[u32],
    uint_locals: &[u32],
) -> Result<BodyShape, Block> {
    // **The mode gate, before any body byte** (board #1638 / #139). A gate that
    // lives only in the emitter is a fact the CENSUS cannot ask, so the census
    // counts a function in class that `PortC2` refuses. `/O1` and `/Ox` are both
    // accepted and both graded; anything else — `/Od` above all — refuses here.
    match opt_word_mode(opt_word_at(seg)) {
        Some(OptWordMode::O1) | Some(OptWordMode::Ox) => {}
        _ => return Err(blk(seg, start, "ctr-loop-opt-mode")),
    }

    let params = parse_formals(seg, lo)?;
    // **Exactly two formals, bound first** (clauses 9 and 14). `swapf` and
    // `three` in `work/w-bdnz/probe/L4.obj` are the cells; `swapf` re-plans the
    // whole body and `three` does not, and both are refused, because the graded
    // set and the accepted set must be the same set.
    if params.len() != 2 {
        return Err(blk(seg, start, "ctr-loop-formals-not-2"));
    }
    let (bound_tok, operand_tok) = (params[0], params[1]);
    if bound_tok == operand_tok {
        return Err(blk(seg, start, "ctr-loop-formals-alias"));
    }

    let mut p = start;

    // ---- statement 1: `s = INIT` ------------------------------------------
    let acc_tok = eat_designator(seg, &mut p, "ctr-loop-acc-designator")?;
    if !locals.contains(&acc_tok) {
        // The same membership test `assign.rs` uses and for the same reason: a
        // file-scope `static int` is a memory object and folding its store away
        // would drop a real store.
        return Err(blk(seg, p, "ctr-loop-acc-not-a-local"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "ctr-loop-acc-init-lit"));
    }
    let (acc_type_id, acc_unsigned) = eat_int_type_signed(seg, &mut p, "ctr-loop-acc-init-type")?;
    // **An UNSIGNED accumulator refuses** (decline D14). `>>=` is `sraw` on a
    // signed accumulator and `srw` on an unsigned one — a different word, keyed
    // on a type `CountedAccumLoop` does not carry and no cell of this lane
    // graded. Refused for every opcode, not only the shift, so the accepted set
    // stays the graded set.
    if acc_unsigned {
        return Err(blk(seg, p, "ctr-loop-acc-unsigned"));
    }
    let acc_init = read_varint(seg, &mut p).ok_or(blk(seg, p, "ctr-loop-acc-init-varint"))?;
    // `li r3,INIT` is one instruction only inside `simm16`. Outside it c2 emits
    // `lis`/`ori` **and interleaves the guard compare between the two words**
    // (cell `init_over`), so this is a different block and not a wider field.
    if !(-0x8000..=0x7FFF).contains(&acc_init) {
        return Err(blk(seg, p, "ctr-loop-acc-init-wide"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "ctr-loop-acc-init-store"));
    }
    if eat_int_type_signed(seg, &mut p, "ctr-loop-acc-init-storetype")?.0 != acc_type_id {
        return Err(blk(seg, p, "ctr-loop-acc-init-storetype"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "ctr-loop-acc-init-end"));
    }

    // ---- the `for` scope opens, then `i = 0` -------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ctr-loop-for-scope"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    let ctr_tok = eat_designator(seg, &mut p, "ctr-loop-ctr-designator")?;
    // **The counter is an automatic local, POSITIVELY, and in the list its own
    // signedness belongs to.** `.sy`'s `int_locals` holds plain `int` and
    // `uint_locals` plain `unsigned` — two lists rather than one, because they
    // are the same storage and a different `cmp` (board #1788 one layer down).
    // Which list the token is in is checked against the `.ex` TYPE byte below,
    // so the two layers have to AGREE and neither is trusted alone.
    if ctr_tok == acc_tok || !(locals.contains(&ctr_tok) || uint_locals.contains(&ctr_tok)) {
        return Err(blk(seg, p, "ctr-loop-ctr-not-a-local"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "ctr-loop-ctr-init-lit"));
    }
    // **The counter's signedness is captured HERE and required to agree at every
    // later position** — board #1788. See the module header.
    let (ctr_type_id, ctr_unsigned) = eat_int_type_signed(seg, &mut p, "ctr-loop-ctr-init-type")?;
    // **THE TWO LAYERS MUST AGREE, and neither is trusted alone** — this is the
    // whole of board #1788 in one clause. `.ex`'s TYPE byte says which `cmp` the
    // guard takes; `.sy`'s list membership says the token is a foldable
    // automatic of that same type. A recognizer that read only `.ex` would trust
    // a signedness `.sy` contradicts; one that read only `.sy` would emit
    // `cmpwi` for a body whose IL says `86 42 75`. Requiring both is free — the
    // two readers have never disagreed on a cell of this class — and it is what
    // makes the clause survive a later widening of either layer.
    let in_int = locals.contains(&ctr_tok);
    let in_uint = uint_locals.contains(&ctr_tok);
    if ctr_unsigned != (in_uint && !in_int) {
        return Err(blk(seg, p, "ctr-loop-ctr-signedness-disagree"));
    }
    // The start must be **0** (decline D13's `start3` half): a non-zero start
    // makes the guard `cmpwi cr6,r11,START` *and* adds an `addi r11,r11,-START`
    // ahead of the `mtctr` — two more words, both fields this class does not
    // carry.
    if read_varint(seg, &mut p) != Some(0) {
        return Err(blk(seg, p, "ctr-loop-ctr-start-not-zero"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "ctr-loop-ctr-init-store"));
    }
    if eat_int_type_signed(seg, &mut p, "ctr-loop-ctr-init-storetype")? != (ctr_type_id, ctr_unsigned)
    {
        return Err(blk(seg, p, "ctr-loop-ctr-init-storetype"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "ctr-loop-ctr-init-end"));
    }

    // ---- the ROTATION: `3A Ltest` · `29 Lincr` · `i += 1` · `29 Ltest` -----
    //
    // The front end emits the `for` unrotated — jump over the increment into the
    // test — so the increment block physically precedes the test block. c2's own
    // rotation (`lur.c`, `wb-loop` §3) then turns the entry into the guard. This
    // is the same stream shape `ptr_walk_loop` reads, with an integer counter in
    // place of a walking pointer.
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "ctr-loop-entry-jump"));
    }
    let l_test = eat_label(seg, &mut p, "ctr-loop-entry-jump-tok")?;
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "ctr-loop-incr-label"));
    }
    let l_incr = eat_label(seg, &mut p, "ctr-loop-incr-label-tok")?;
    if l_incr == l_test {
        return Err(blk(seg, p, "ctr-loop-labels-alias"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_designator(seg, &mut p, "ctr-loop-incr-designator")? != ctr_tok {
        return Err(blk(seg, p, "ctr-loop-incr-not-the-counter"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "ctr-loop-incr-lit"));
    }
    if eat_int_type_signed(seg, &mut p, "ctr-loop-incr-lit-type")? != (ctr_type_id, ctr_unsigned) {
        return Err(blk(seg, p, "ctr-loop-incr-lit-type"));
    }
    // **STEP == +1, required literally** (clause 4). `wb-loop` §9 item 4: for a
    // non-unit step c2 emits a preheader trip-count computation (`addi -1`, then
    // `srwi` for a power of two or `divwu` for 3, then `addi +1`) whose selector
    // is **unread**. Cell `n_step2` is the `i += 2` witness — it converts, and
    // this class still refuses it, which is the honest direction.
    if read_varint(seg, &mut p) != Some(1) {
        return Err(blk(seg, p, "ctr-loop-step-not-1"));
    }
    // `0F` — add-assign. Anything else is a different induction variable.
    if !eat_byte(seg, &mut p, 0x0F) {
        return Err(blk(seg, p, "ctr-loop-incr-op"));
    }
    if eat_int_type_signed(seg, &mut p, "ctr-loop-incr-type")? != (ctr_type_id, ctr_unsigned) {
        return Err(blk(seg, p, "ctr-loop-incr-type"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "ctr-loop-incr-end"));
    }

    // ---- the TEST: `29 Ltest` · `i < n` · `38 Lexit` -----------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "ctr-loop-test-label"));
    }
    if eat_label(seg, &mut p, "ctr-loop-test-label-tok")? != l_test {
        return Err(blk(seg, p, "ctr-loop-test-label-mismatch"));
    }
    let (t, uns) = eat_int_load(seg, &mut p, "ctr-loop-test-ctr-load")?;
    if t != ctr_tok || uns != ctr_unsigned {
        return Err(blk(seg, p, "ctr-loop-test-not-the-counter"));
    }
    // **The BOUND is a loop-invariant SYMBOL and nothing else** (clause 5): a
    // bare `B9 <formal 0> <TYPE>` with no operator behind it. `wb-loop` §7.4's
    // `a10` is the measurement — `i < n/2+3` puts a temporary here and c2 emits
    // no `bdnz` at all — and R4's "kind == 7, displacement 0" is the same fact
    // stated in c2's own structures. This clause is what makes the bound a
    // symbol rather than an expression, and it needs no number out of `c2.dll`.
    let (t, uns) = eat_int_load(seg, &mut p, "ctr-loop-test-bound-load")?;
    if t != bound_tok {
        return Err(blk(seg, p, "ctr-loop-bound-not-formal0"));
    }
    if uns != ctr_unsigned {
        return Err(blk(seg, p, "ctr-loop-bound-signedness"));
    }
    // `22` — the LT relation. `23` (`<=`), `20` (`!=`) and the descending forms
    // each decide a different `bclr` bit pair and, for `<=`, an extra
    // `addi r11,r11,1` before the `mtctr` (cells `le`, `ne`, `down` in
    // `work/w-bdnz/probe/L4.obj`). Two of the four guard conditions ship here;
    // the byte is required literally so the other two cannot arrive.
    if !eat_byte(seg, &mut p, 0x22) {
        return Err(blk(seg, p, "ctr-loop-test-not-lt"));
    }
    if !eat_byte(seg, &mut p, 0x38) {
        return Err(blk(seg, p, "ctr-loop-test-brfalse"));
    }
    let l_exit = eat_label(seg, &mut p, "ctr-loop-exit-tok")?;
    if l_exit == l_test || l_exit == l_incr {
        return Err(blk(seg, p, "ctr-loop-labels-alias"));
    }

    // ---- the BODY: one compound assignment, optionally braced --------------
    eat_opt_stmt_marker(seg, &mut p);
    // The braced spelling `{ s OP= k; }` opens a scope here and closes it with
    // `54 04` before the back edge. Both spellings emit **identical text** —
    // graded, cells P1 and P10 — so both are accepted and the flag decides only
    // which scope-close to require.
    let braced = eat_byte(seg, &mut p, 0x53);
    if braced {
        eat_opt_stmt_marker(seg, &mut p);
    }
    if eat_designator(seg, &mut p, "ctr-loop-body-designator")? != acc_tok {
        return Err(blk(seg, p, "ctr-loop-body-not-the-accumulator"));
    }
    // **The operand is formal 1, loaded, and nothing else** (clause 11). A
    // literal operand is an immediate form per opcode — `mulli`, `xori`, and
    // `andi.` which writes CR0 — none of which this emitter has a field for.
    let (t, uns) = eat_int_load(seg, &mut p, "ctr-loop-body-operand-load")?;
    if t != operand_tok {
        return Err(blk(seg, p, "ctr-loop-operand-not-formal1"));
    }
    if uns {
        return Err(blk(seg, p, "ctr-loop-operand-unsigned"));
    }
    let op = match seg.get(p).copied().and_then(accum_op) {
        Some(op) => {
            p += 1;
            op
        }
        None => return Err(blk(seg, p, "ctr-loop-body-op")),
    };
    if eat_int_type_signed(seg, &mut p, "ctr-loop-body-optype")? != (acc_type_id, false) {
        return Err(blk(seg, p, "ctr-loop-body-optype"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "ctr-loop-body-end"));
    }

    // ---- the BACK EDGE: [`54 04`] · `3A Lincr` -----------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if braced && !eat(seg, &mut p, &[0x54, (BODY_SCOPE_DEPTH + 2) as u8]) {
        return Err(blk(seg, p, "ctr-loop-body-scope-close"));
    }
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "ctr-loop-back-edge"));
    }
    if eat_label(seg, &mut p, "ctr-loop-back-edge-tok")? != l_incr {
        return Err(blk(seg, p, "ctr-loop-back-edge-target"));
    }

    // ---- the EXIT: `29 Lexit` · `return s` ---------------------------------
    //
    // Requiring the return **immediately** is clause 10: `n_after`'s
    // `return s + 7;` puts an expression here, and c2 re-plans — the accumulator
    // stays in r11 and the guard becomes a forward `bf 25`. The whole eight-word
    // body depends on the loop being the function's tail.
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "ctr-loop-exit-label"));
    }
    if eat_label(seg, &mut p, "ctr-loop-exit-label-tok")? != l_exit {
        return Err(blk(seg, p, "ctr-loop-exit-label-mismatch"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    let (t, uns) = eat_int_load(seg, &mut p, "ctr-loop-return-load")?;
    if t != acc_tok || uns {
        return Err(blk(seg, p, "ctr-loop-return-not-the-accumulator"));
    }
    // The shared tail: `41 <int> · 3A <lbl> · 54 03 · 54 02 · 29 <lbl> · 4F 12 …`,
    // and the fail-closed terminal — anything trailing rejects.
    eat_return_plumbing(seg, &mut p, true, BODY_SCOPE_DEPTH + 1)?;

    Ok(BodyShape::CountedAccumLoop(CountedAccumLoop {
        params,
        acc_init,
        op,
        counter_unsigned: ctr_unsigned,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The opcode map is the set real `c2` was compiled on, and the two
    /// absentees are absent for measured reasons rather than caution.
    #[test]
    fn the_accumulate_opcode_map_is_the_measured_set() {
        use CountedAccumOp::*;
        for (b, want) in [
            (0x10u8, Sub),
            (0x11, Mul),
            (0x15, Shl),
            (0x16, Sar),
            (0x17, And),
            (0x18, Xor),
            (0x19, Or),
        ] {
            assert_eq!(accum_op(b), Some(want), "IL {b:#04x} is a graded accumulate");
        }
        // `+=` DELETES the loop — c2 strength-reduces the whole accumulation to
        // one `mullw`, guard and all (`work/w-bdnz/probe/L3.obj`, `op_add`).
        assert_eq!(accum_op(0x0F), None);
        // `/=` is a different spine: `rotlwi`/`divw`/`addi`/`twi`/`andc`/`twi`.
        assert_eq!(accum_op(0x12), None);
        // and nothing else is admitted at all.
        for b in [0x00u8, 0x01, 0x0E, 0x13, 0x14, 0x1A, 0x20, 0x22, 0x26, 0xFF] {
            assert_eq!(accum_op(b), None, "IL {b:#04x} is outside the class");
        }
    }

    /// **The `label_slots` refusal, pinned where it can be seen.** It is what
    /// makes `fixtures/cpp/wbdnz_ctr_then_framed_neg.cpp` a whole-TU refusal,
    /// and the must-fail mutation for it is recorded at
    /// `IlFunction::label_slots` itself. Asserted at BOTH values of
    /// `fn_level_linking`, because the charge this lane measured is
    /// mode-dependent and the method has no mode parameter — so neither
    /// spelling of the question may accidentally return a number.
    #[test]
    fn the_counted_loop_charges_an_unrepresentable_number_of_label_slots() {
        let f = crate::func::IlFunction {            body: crate::func::BodyShape::CountedAccumLoop(CountedAccumLoop {
                params: vec![0xE3, 0xE4],
                acc_init: 0,
                op: CountedAccumOp::Sub,
                counter_unsigned: false,
            }),
            ..crate::func::IlFunction::base("?p_sub@@YAHHH@Z", &None)
        };
        assert_eq!(f.label_slots(false), None);
        assert_eq!(f.label_slots(true), None);
        // The separating control: the same builder without the field is an
        // ordinary leaf and charges its lead + 1, so the `None` above is this
        // shape's answer and not the builder's.
        let plain = crate::func::IlFunction::base("?g@@YAHH@Z", &None);
        assert_eq!(plain.label_slots(false), Some(plain.label_lead() + 1));
    }

    /// **Board #1788, as an assertion rather than a comment.** The signed and
    /// unsigned int TYPEs differ in one byte; `eat_int_type_signed` must
    /// separate them where `eat_int_like` deliberately does not.
    #[test]
    fn the_counter_type_reader_separates_int_from_unsigned() {
        let int_ty = [0x86u8, 0x41, 0x74];
        let uint_ty = [0x86u8, 0x42, 0x75];
        let mut p = 0;
        assert_eq!(
            eat_int_type_signed(&int_ty, &mut p, "t").map(|(_, u)| u),
            Ok(false)
        );
        assert_eq!(p, 3);
        let mut p = 0;
        assert_eq!(
            eat_int_type_signed(&uint_ty, &mut p, "t").map(|(_, u)| u),
            Ok(true)
        );
        assert_eq!(p, 3);
        // The control that says the two are otherwise identical: they differ in
        // exactly one byte position out of three, which is why a whitelist that
        // matched on the tag alone would merge them.
        assert_eq!(int_ty[0], uint_ty[0]);
        assert_eq!(
            int_ty.iter().zip(uint_ty.iter()).filter(|(a, b)| a != b).count(),
            2
        );
        // `long`/`unsigned long` are the same fact one spelling over, and c2
        // emits byte-identical code for them — so they must classify the same
        // way and not refuse.
        let mut p = 0;
        assert_eq!(
            eat_int_type_signed(&[0x86u8, 0x41, 0x12], &mut p, "t").map(|(_, u)| u),
            Ok(false)
        );
        let mut p = 0;
        assert_eq!(
            eat_int_type_signed(&[0x86u8, 0x42, 0x22], &mut p, "t").map(|(_, u)| u),
            Ok(true)
        );
        // A `long long` is NOT int-like and refuses — clause 3's reader half.
        let mut p = 0;
        assert!(eat_int_type_signed(&[0x88u8, 0x81, 0x74], &mut p, "t").is_err());
    }
}

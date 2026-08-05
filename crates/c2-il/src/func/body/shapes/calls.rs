//! **The unified call shape** — the ONE copy.
//!
//! `docs/GAPS.md` §6 instance #9: the direct and the bound (`call through a
//! local`) forms each carried their own argument validation, and the two
//! drifted. They were unified into `tail_call_shape`, and this module exists so
//! that the future statement-call forms import it rather than growing a third.
//!
//! Also the call *sequence* (Class A many-calls, Class B values-live-across-
//! calls) and `plan_saved_gprs`. That half is the serial spine's, paired with
//! `c2-core/src/codegen/calls.rs` — `docs/ARCHITECTURE_SEAMS.md` §7.

use crate::func::body::chain::{
    additive_chain_canonical, has_repeated_leaf, leaves_ascending,
    straight_line_out_of_class_ctx,
};
use crate::func::body::expr::{
    eat_return_plumbing, intrinsic_selector, parse_expr, BODY_SCOPE_DEPTH,
};
use crate::func::body::{
    blk, Block, BodyShape, SeqCall, SeqEarlyReturnShape, SeqGuardShape, SeqTail, SlotArg,
};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_int_like_or_ptr4, eat_opt_stmt_marker, read_token_var,
    read_type, read_varint, TYPE_KIND_REAL_CLASS,
};
use crate::func::{IlOp, LINK_FIRST_SLOT};

use super::params::parse_params;

/// Every LOAD in a call-argument operand stream must name a **formal**.
///
/// The multi-argument path established this positively from the start
/// (`call-arg-nonformal`); the three single-argument paths did not, so
/// `int gi; int g(int); int u1() { return g(gi); }` — a global as the argument —
/// **parsed as an in-class integer tail call**. Codegen then refused it, so no wrong
/// bytes were ever emitted, but the census counted it as in class while the gate did
/// not, which breaks the invariant this repo is built on: acceptance lives in the IL
/// parser precisely so the census and the gate cannot disagree about what is
/// accepted. A census that over-reports is a broken instrument, and the widening
/// order is chosen from it.
///
/// Found by an independent characterization agent probing the bucket, not by any
/// fixture — the corpus had no call whose argument was a global.
pub(crate) fn arg_loads_are_formals(arg_ops: &[IlOp], params: &[u32]) -> bool {
    arg_ops.iter().all(|o| match o {
        IlOp::Load(t) => params.contains(t),
        _ => true,
    })
}

/// The non-trivial cycles of the argument permutation `sources`, as
/// `(count, longest)`. `sources[i]` is the formal index argument slot `i` wants,
/// so a fixed point is a value already in place.
///
/// `sources` must already have been proved to index inside itself
/// ([`tail_call_shape`]'s `call-arg-outer-formal` gate); this walk indexes `seen`
/// with an entry, so an out-of-range one **panics** rather than refusing. It did:
/// see that gate's comment.
fn permutation_cycles(sources: &[usize]) -> (usize, usize) {
    let n = sources.len();
    let mut seen = vec![false; n];
    let mut cycles = 0usize;
    let mut longest = 0usize;
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        let mut at = start;
        let mut len = 0usize;
        while !seen[at] {
            seen[at] = true;
            len += 1;
            at = sources[at];
        }
        cycles += 1;
        longest = longest.max(len);
    }
    (cycles, longest)
}

/// The longest argument-permutation cycle `c2_core::codegen::permute_args_text`
/// has been **verified** to lower, measured over complete grids rather than
/// sampled: all 24 permutations of a four-argument call and all 84 single cycles
/// of length 2–5 inside a five-argument one.
///
/// ```text
///   cycle length 2    0 mismatch / 10 cases
///   cycle length 3    0 mismatch / 20
///   cycle length 4   10 mismatch / 30
///   cycle length 5   16 mismatch / 24
/// ```
///
/// Past three, c2 does not use the minimal single-temp walk the port emits. It
/// hoists a **second** save into r10 and writes the destinations in a different
/// order — `int f(int a,int b,int c,int d){ return a4(c,d,b,a); }` is
///
/// ```text
///   7cab2b78  mr r11,r5      7cca3378  mr r10,r6
///   7c661b78  mr r6,r3       7c852378  mr r5,r4
///   7d445378  mr r4,r10      7d635b78  mr r3,r11      six moves, two temps
/// ```
///
/// against the port's five-move single-temp walk — a **live wrong-bytes emit on
/// mainline** (`Port=Mismatch @ 8`), independent of any framed shape. Twenty of
/// the thirty four-cycles happen to agree with the minimal walk and ten do not,
/// so "it worked on the fixtures" was luck of the sample: `il_call_perm.cpp` and
/// `il_call_multi.cpp` between them hold no cycle longer than three.
///
/// The order c2 actually picks past three is **not characterized** — the six
/// four-cycles split four/two on a property the grid describes but does not
/// explain — so the boundary is drawn at the measured edge rather than fitted.
pub(crate) const MAX_VERIFIED_PERM_CYCLE: usize = 3;

/// The `li rD,k` immediate — `addi rD,0,k`'s signed 16-bit field. A literal
/// argument outside it is `lis`+`ori`, measured one line apart in
/// `work/WLA/probe/p1.cpp`: `g3(a,b,32767)` is `li 5,32767` and
/// `g3(a,b,70000)` is `lis 5,1 ; ori 5,5,4464`.
pub(crate) const LI_IMM_MIN: i32 = -0x8000;
pub(crate) const LI_IMM_MAX: i32 = 0x7FFF;

/// **WLA — the literal call argument**: `g3(a, b, 7)`, `p->gk(j, 7)`.
///
/// The whole lowering is **one `li r<slot>,k` per literal slot** and no move at
/// all, because every other slot's formal is already in the argument register it
/// is being passed in. Read off the reference obj
/// (`work/WLA/probe/p1.cpp`, `/O1 /GS- /c`), and the member form is the free
/// form — `this` is just the formal in slot 0:
///
/// ```text
///   void f(int a)       { g2(a, 5); }        38800005 li 4,5    · b ?g2
///   void f(int a,int b) { g3(a, b, 7); }     38a00007 li 5,7    · b ?g3
///   int  f(O* p,int j)  { return p->gk(j,7); }38a00007 li 5,7   · b ?gk
///   void f(int a,int b,int c){ g4(a,b,c,9); }38c00009 li 6,9    · b ?g4
///   void f(int a)       { g3(a, 5, 6); }     li 5,6 · li 4,5    · b ?g3
/// ```
///
/// **Three refusals, each measured on the same probe rather than assumed.**
///
/// 1. **The formals must already be in place.** A literal beside a formal that
///    has to *move* is a different lowering: `g3(a,7,b)` is `mr r5,r4 ; li r4,7`
///    and `g3(7,a,b)` is `mr r5,r4 ; mr r4,r3 ; li r3,7` — the moves come first,
///    highest destination first, and the emission has to interleave with them.
///    That much a descending walk would get right; what no capture covers is the
///    same list over a real **cycle** (`g3(b,a,7)`), where the r11 break temp and
///    the `li` both want a slot in the order. So the gate is the positive one:
///    every non-literal slot must be `Formal(slot)`. `call-arg-lit-permuted`,
///    **733 functions** on the 878-TU workload, is what that costs.
/// 2. **The literal must fit `li`'s immediate** ([`LI_IMM_MIN`]) — the caller's
///    check, so a wide one never reaches here.
/// 3. **Every slot must be a register slot.** Past `r10` an argument is
///    stack-homed and its setup is a store, not a `li`.
///
/// Emission order is `c2_core::codegen::permute_args_parts`'s, where the bytes
/// are; this function decides only *what* is in class.
/// The formal indices of a [`BodyShape::MultiArgTailCall`]'s slot list, for the
/// two callers that build a **framed** [`SeqCall`] out of it — the statement-call
/// sequence and a chain's innermost call.
///
/// A literal is refused there rather than carried, and the refusal is a
/// **positive statement about what has been captured**: a framed call's
/// marshalling interleaves with the callee-saved copies (`plan_saved_gprs`'s
/// hoist/trail rule) and with the previous `bl`'s result save, and every witness
/// of that interleaving is a `mr`. The tail-call form has no such neighbours,
/// which is why WLA takes it and leaves this one.
///
/// It exists as a locator rather than as two `match` arms because both callers
/// implement the same rule, and this file's header is about what happens when
/// they each implement it privately.
pub(crate) fn seq_call_arg_sources(
    seg: &[u8],
    off: usize,
    slots: Vec<SlotArg>,
) -> Result<Vec<usize>, Block> {
    let mut sources = Vec::with_capacity(slots.len());
    for a in slots {
        match a {
            SlotArg::Formal(ix) => sources.push(ix),
            SlotArg::Lit(_) => {
                return Err(Block::refuse(seg, off, "callseq-multiarg-lit"))
            }
            // WR1: a data symbol's address inside a **framed** sequence call.
            // The `lis`/`addi` pair would have to be scheduled against the
            // callee-saved copies of a frame, and every capture behind
            // [`sym_addr_tail_call`] is a leaf tail call. Refused by name.
            SlotArg::SymAddr(_) => {
                return Err(Block::refuse(seg, off, "callseq-multiarg-sym"))
            }
        }
    }
    Ok(sources)
}

/// **WR1 — the tail call one of whose argument slots is a named data symbol's
/// address**: `void f(S* s){ s->so(&gI); }`, `void f(){ gso(&gI); }`.
///
/// Every word below is read off a reference obj at the fixture profile
/// (`work/wr1/probes/p2.cpp`, `/Ox /GS- /c`), and the class is drawn at the edge
/// of what those captures cover, not at the edge of what looks plausible.
///
/// **What IS admitted — one symbol, every other slot already in place:**
///
/// ```text
///   void a1(S* s)              { s->m1("aa"); }        lis r11 · addi r4,r11,0 · b
///   void a3(S* s,int k)        { s->m3(k, "cc"); }     lis r11 · addi r5,r11,0 · b
///   void a5()                  { g1("ee"); }           lis r11 · addi r3,r11,0 · b
///   void a8(int j,int k)       { g4(j, k, "hh"); }     lis r11 · addi r5,r11,0 · b
///   void a9(a..g)              { g8(a..g, "ii"); }     lis r11 · addi r10,r11,0 · b
///   void c1()                  { g2("jj", 7); }        lis r11 · li r4,7 · addi r3,r11,0 · b
/// ```
///
/// so the `lis` is **hoisted to the top of the function** and the address `addi`
/// takes its own place in the ordinary descending-destination walk beside the
/// literals — which is `c2_core::codegen::permute_args_parts`' existing rule, met
/// again rather than discovered (`docs/IL_CALL_IN_EXPR.md` §26.7 found it a third
/// time).
///
/// **Three refusals, each with a capture behind it and not one of them a guess:**
///
/// 1. **Two or more symbols** — `docs/IL_CALL_IN_EXPR.md` §17.3 (a)/(b), and it
///    is the reason 18,933 functions are a *phase* and not a rung: c2 emits one
///    `lis`/`addi` pair per function and derives the second address by
///    `.rdata` pool-offset difference (`addi r4,r3,-4`), which needs a whole-TU
///    pool layout before instruction selection, and *which* symbol anchors is a
///    hypothesis fitted to 14 witnesses with no mechanism behind it.
/// 2. **A formal that has to MOVE.** The probe has this cell and it is *not* the
///    reason for the refusal — `a2`/`c2`/`c3` each emit one `mr` and follow the
///    same hoist/trail rule — but the cell **beside** it breaks: `a4`
///    (`s->m4("dd", j, k)`, two formals shifting) emits `mr r11,r4 ; lis r10,0 ;
///    mr r6,r5 ; addi r4,r10,0 ; mr r5,r11`, i.e. c2 pre-saves into r11 and the
///    `lis` **moves to r10**, where the obvious descending walk needs no save at
///    all (§17.3 (d)). One moved formal and two are two different schedules and
///    the boundary between them is one probe wide, so the gate is the positive
///    one: every non-symbol, non-literal slot is the formal already sitting in
///    its own argument register. `call-arg-sym-permuted` is what that costs, and
///    it is a **measured ceiling** for a follow-on rung rather than an unknown.
/// 3. **A slot past `r10`.** Argument nine onwards is stack-homed and its setup
///    is a store.
fn sym_addr_tail_call(
    seg: &[u8],
    off: usize,
    params: Vec<u32>,
    slots: Vec<SlotArg>,
    callee_tok: u32,
    syms: usize,
) -> Result<BodyShape, Block> {
    let refuse = |ctx: &'static str| Block::refuse(seg, off, ctx);
    // **Two symbols, admitted for the `??__E`/`??__F` thunk only (W-R1).**
    //
    // The refusal above it stands on `docs/IL_CALL_IN_EXPR.md` §17.3 (a)/(b),
    // whose stated mechanism is "c2 materializes only the first through a
    // relocation pair and derives the rest by `.rdata` pool-offset difference".
    // That mechanism does **not** hold for the tail-call form at the workload's
    // own flags. MEASURED here, one TU per row, `/nologo /c /GR /O1 /Oi /EHsc`,
    // every word read off c2's own listing (`c2rs listing`):
    //
    // ```text
    //   void f(){ g("aa","bb"); }              lis r11,bb · lis r10,aa
    //                                          addi r4,r11,bb · addi r3,r10,aa · b
    //   void f(){ g(&gA,&gB); }                lis r11,gB · lis r10,gA
    //                                          addi r4,r11,gB · addi r3,r10,gA · b
    //   void f(){ g("aa",&gA); }               lis r11,gA · lis r10,aa · … · b
    //   void f(){ g("aa","bb",0); }            … addi r4 · addi r3 · li r5,0 · b
    //   void f(){ g(&gA,7,"cc"); }             lis r11,cc · lis r10,gA
    //                                          addi r5,r11,cc · addi r3,r10,gA
    //                                          li r4,7 · b
    //   static L sL("abc",0);  (??__EsL)       lis r11,`string' · lis r10,sL
    //                                          addi r4,r11 · addi r3,r10
    //                                          li r5,0 · b ??0L@@QAA@PBDH@Z
    // ```
    //
    // Six captures, two `.rdata` strings / two `.bss` externs / one of each /
    // with a literal on either side: **one independent `lis`/`addi` pair per
    // symbol, no pool difference anywhere**, and the two thunks the lane is about
    // are the same schedule as the rest.
    //
    // **So why is this still fenced to the bare `LO`?** Because opening it
    // generally is a different rung: §17.3's population is 18,933 functions the
    // pre-registration for this lane recorded as unchanged, the emission order it
    // implies is *not* the one this port already ships (the last row above puts
    // `li r4,7` AFTER both `addi`s, where `permute_args_parts`' descending walk
    // and WR1's own one-symbol capture `g2("jj",7)` put it before), and grading
    // that is an emit-side lane with its own captures. The refusal that remains
    // is therefore a **scope decline with a measured ceiling**, not an unknown.
    //
    // Nothing here can mis-emit either way: `c2_core::codegen::calls`'
    // `sym_slots_text` carries its own independent `count != 1` backstop, so a
    // body admitted here reaches the port and comes back `NotImplemented` — a
    // `codegen-gap`, which is the honest bucket for "decoded, not yet emittable".
    let two_sym_thunk = syms == 2 && crate::func::body_start_is_bare(seg);
    if syms > 1 && !two_sym_thunk {
        return Err(refuse("call-arg-multi-sym"));
    }
    if slots.len() > MAX_REGISTER_FORMALS {
        return Err(refuse("call-arg-sym-overflow"));
    }
    let in_place = slots.iter().enumerate().all(|(slot, a)| match a {
        SlotArg::SymAddr(_) => true,
        SlotArg::Lit(k) => (LI_IMM_MIN..=LI_IMM_MAX).contains(k),
        SlotArg::Formal(ix) => *ix == slot,
    });
    if !in_place {
        return Err(refuse("call-arg-sym-permuted"));
    }
    Ok(BodyShape::MultiArgTailCall { params, arg_sources: slots, callee_tok })
}

fn lit_arg_tail_call(
    seg: &[u8],
    off: usize,
    params: Vec<u32>,
    slots: Vec<SlotArg>,
    callee_tok: u32,
) -> Result<BodyShape, Block> {
    let refuse = |ctx: &'static str| Block::refuse(seg, off, ctx);
    if slots.len() > MAX_REGISTER_FORMALS {
        return Err(refuse("call-arg-lit-over-eight-slots"));
    }
    // Stated positively: every non-literal slot names the formal that is ALREADY
    // in that argument register.
    let in_place = slots
        .iter()
        .enumerate()
        .all(|(slot, a)| matches!(a, SlotArg::Lit(_)) || *a == SlotArg::Formal(slot));
    // **WLB — the one moved formal, at exactly two slots.** `g2(b, 7)`: slot 0
    // wants a formal that is not in r3, slot 1 is the literal. 699 of the 733
    // `call-arg-lit-permuted` functions are this, it is the ONLY list shape two
    // slots can take once a formal is out of place, and both of its cells are
    // captured (`work/WLA/probe/p2.cpp`, `/O1 /GS- /c`):
    //
    // ```text
    //   void f(int a,int b)      { g2(b, 7); }  mr r3,r4 · li r4,7   <- HOISTED
    //   void f(int a,int b,int c){ g2(c, 7); }  li r4,7 · mr r3,r5   <- descending
    // ```
    //
    // The deciding variable is a single boolean — does the `li`'s destination
    // register hold the value the move needs — and both of its values are
    // witnessed, which is what makes two slots a complete cell rather than a
    // sample. **Three slots is not**, and the same probe says why: `g3(c,b,7)`
    // and `g3(b,c,7)` follow the same hoist, and `g3(c,a,7)` — one formal moving
    // up while another moves down — breaks with `mr r11,r5` and puts the `li`
    // *inside* the walk. So the bound is the measured edge and not a fit; the
    // 34 remaining functions are `call-arg-lit-permuted` still.
    let one_moved_at_two = slots.len() == 2
        && matches!(slots[1], SlotArg::Lit(_))
        && matches!(slots[0], SlotArg::Formal(ix) if ix >= 1 && ix < MAX_REGISTER_FORMALS);
    if !in_place && !one_moved_at_two {
        return Err(refuse("call-arg-lit-permuted"));
    }
    Ok(BodyShape::MultiArgTailCall { params, arg_sources: slots, callee_tok })
}

/// **One locator for "are these call arguments a tail call this port can emit?"**
/// — the validation and the shape construction for `return g(…)` in every
/// position it appears: the direct form, the bound-to-a-local form
/// (`int z = g(…); return z;`), and the single statement call that is a whole
/// body (`void f(int a){ g(a); }`, which c2 lowers to a bare `b g`).
///
/// It exists because those paths carried **two copies** of the checks and the
/// copies had drifted apart in both directions — each copy was missing a gate the
/// other had, and each omission was live:
///
/// * **A wrong-bytes emit.** `int f(int a,int b){ int z = g(b + a); return z; }`
///   emitted `add r3,r4,r3` against the reference's `add r3,r3,r4`: c2
///   canonicalizes the leaves of a commutative argument expression, so `g(a+b)`
///   and `g(b+a)` are the **same** obj. The direct form `return g(b + a);`
///   refuses on [`leaves_ascending`] and always has; the bound-to-a-local copy
///   never asked. `Port=Match` for `a+b`, `Port=Mismatch @ 537` for `b+a`, from
///   two lines of C++ that differ by one transposition.
/// * **A panic.** `int f(int a,int b,int c){ int z = g2(a, c); return z; }` took
///   `c2rs census` down with `index out of bounds: the len is 2 but the index is
///   2` — [`permutation_cycles`] indexed its `seen` array with a *formal* index
///   past the argument count. The direct form got the `call-arg-outer-formal`
///   gate when that was found (`docs/GAPS.md` §6); this copy did not, and the CLI
///   must degrade cleanly, never panic.
///
/// Same family as every other entry in `docs/GAPS.md` §6: one fact, two
/// implementations, and the corpus only ever exercised the fixed one.
///
/// `args` is the argument list in **stream order** (reverse source order, so slot
/// `i` is `args[len-1-i]`); `params` is the formals list with a member function's
/// `this` at index 0; `off` is the segment offset a refusal reports.
pub(crate) fn tail_call_shape(
    seg: &[u8],
    args: Vec<Vec<IlOp>>,
    params: Vec<u32>,
    callee_tok: u32,
    off: usize,
) -> Result<BodyShape, Block> {
    let refuse = |ctx: &'static str| Block::refuse(seg, off, ctx);
    // No arguments at all: the bare `b <callee>`.
    if args.is_empty() {
        return Ok(BodyShape::VoidTailCall { callee_tok });
    }
    // WR1: a lone data-symbol address takes the SLOT path rather than the operand
    // path, because it is not a computation `select_text` can produce — it is two
    // instructions and a relocation quad. Everything else at one argument keeps
    // the operand stream, which can carry `g(a + 1)` and the slot list cannot.
    if args.len() > 1 || matches!(args[0].as_slice(), [IlOp::SymAddr(_)]) {
        // Two or more arguments: a **permutation of the formals**, or the formals
        // already in place beside one or more **literals** (WLA). Anything else —
        // an operand stream that has to be computed into an argument register —
        // would need its own register and interacts with the permutation temp in
        // ways no capture covers.
        let mut slots: Vec<SlotArg> = Vec::with_capacity(args.len());
        let mut lits = 0usize;
        let mut syms = 0usize;
        for slot in 0..args.len() {
            let ops = &args[args.len() - 1 - slot];
            match ops.as_slice() {
                [IlOp::Load(t)] => match params.iter().position(|q| q == t) {
                    Some(ix) => slots.push(SlotArg::Formal(ix)),
                    // An argument that is not one of this function's formals (a
                    // local, a global, a nested call result).
                    None => return Err(refuse("call-arg-nonformal")),
                },
                // **WLA — `g3(a, b, 7)` is `li r5,7` and nothing else.** The
                // signed-16-bit bound is `li`'s own: 70000 is `lis`+`ori`,
                // measured beside 32767, which is not.
                [IlOp::Lit(k)] => {
                    if !(LI_IMM_MIN..=LI_IMM_MAX).contains(k) {
                        return Err(refuse("call-arg-lit-wide"));
                    }
                    lits += 1;
                    slots.push(SlotArg::Lit(*k));
                }
                // **WR1 — a named data symbol's address.**
                [IlOp::SymAddr(tok)] => {
                    syms += 1;
                    slots.push(SlotArg::SymAddr(*tok));
                }
                _ => return Err(refuse("call-arg-computed")),
            }
        }
        // Asked BEFORE the literal path, because the symbol's `lis` is hoisted
        // ahead of the whole argument setup and a list carrying both is a
        // different schedule from either alone (`sym_addr_tail_call` admits the
        // literals it has captured beside a symbol; `lit_arg_tail_call` has never
        // seen one).
        if syms > 0 {
            return sym_addr_tail_call(seg, off, params, slots, callee_tok, syms);
        }
        if lits > 0 {
            return lit_arg_tail_call(seg, off, params, slots, callee_tok);
        }
        let mut arg_sources: Vec<usize> = Vec::with_capacity(slots.len());
        for a in &slots {
            match a {
                SlotArg::Formal(ix) => arg_sources.push(*ix),
                // Unreachable: `lits == 0` is exactly "no `SlotArg::Lit` was
                // pushed", stated positively rather than as an `unreachable!`,
                // because a panic in the CLI is the failure mode this file's
                // header records.
                SlotArg::Lit(_) => return Err(refuse("call-arg-lit-classified-twice")),
                // Unreachable for the same reason: `syms == 0` is exactly "no
                // `SlotArg::SymAddr` was pushed", and the symbol path returned
                // above.
                SlotArg::SymAddr(_) => return Err(refuse("call-arg-sym-classified-twice")),
            }
        }
        // **An argument that is a formal beyond the argument count.** `arg_sources`
        // indexes the *formals* list while everything below treats it as a
        // permutation of the *argument* slots, and the two lists are only the same
        // length when the call passes every formal. `int f(int a,int b,int c){
        // return g(a,c); }` gives sources `[0, 2]` over two slots: not a
        // permutation but a move out of a register the call does not otherwise
        // touch, which `permute_args_text` has no case for — and it indexed
        // [`permutation_cycles`]'s `seen` array out of bounds, i.e. **panicked**.
        if arg_sources.iter().any(|&ix| ix >= arg_sources.len()) {
            return Err(refuse("call-arg-outer-formal"));
        }
        // The two permutation shapes codegen cannot lower are rejected HERE rather
        // than there, so the census and the emission gate cannot disagree about
        // what is in class (the same reason the FP contraction and constant gates
        // live in this file). Both are captured in `fixtures/cpp/il_call_multi.cpp`
        // and explained at `c2_core::codegen::permute_args_text`.
        //
        // A value passed twice: c2 emits a dead `mr` through the temp, which no
        // live-value-driven solver produces.
        for (i, s) in arg_sources.iter().enumerate() {
            if arg_sources[..i].contains(s) {
                return Err(refuse("call-arg-duplicated"));
            }
        }
        let (cycles, longest) = permutation_cycles(&arg_sources);
        if cycles > 1 {
            return Err(refuse("call-arg-multicycle"));
        }
        // Past a three-element cycle c2 stops using the minimal single-temp walk
        // and hoists a second save into r10 — a live wrong-bytes emit, measured
        // over the complete 4- and 5-argument grids ([`MAX_VERIFIED_PERM_CYCLE`]).
        if longest > MAX_VERIFIED_PERM_CYCLE {
            return Err(refuse("call-arg-long-cycle"));
        }
        return Ok(BodyShape::MultiArgTailCall {
            params,
            arg_sources: slots,
            callee_tok,
        });
    }
    let arg_ops = args.into_iter().next().expect("exactly one argument");
    // The single call argument is an ordinary operand stream, so it is subject to
    // the same rewriter: `g(a + a)` is not `add` + branch.
    if has_repeated_leaf(&arg_ops) {
        return Err(refuse("call-arg-repeated-leaf"));
    }
    // And to the same reassociation: `g(b + a)` is not the source order either —
    // c2 canonicalizes the leaves and emits `add r3,r3,r4` for both orders. The
    // gate is vacuous for a single leaf (one leaf cannot be out of order), which is
    // why it asks the load count first.
    let n_loads = arg_ops.iter().filter(|o| matches!(o, IlOp::Load(_))).count();
    if n_loads > 1 && !leaves_ascending(&arg_ops, &params) {
        return Err(refuse("call-arg-noncanonical-order"));
    }
    if !additive_chain_canonical(&arg_ops) {
        return Err(refuse("call-arg-noncanonical-order"));
    }
    if !arg_loads_are_formals(&arg_ops, &params) {
        return Err(refuse("call-arg-nonformal"));
    }
    // The argument is computed into r3 by `c2_core::codegen::select_text`, the
    // same selector a straight-line leaf's body goes through, so it is subject to
    // **exactly the same** out-of-class rules — and those lived only in codegen for
    // this position. Measured: `int f(int a){ return g(a * 5); }` censuses 1/1 and
    // the port returns `NotImplemented` (a constant multiply strength-reduces to
    // shifts and adds), on mainline, in both directions of every fixture lane. A
    // census that over-claims is a broken instrument and the widening order is
    // chosen from it, so the predicate is asked here instead of there.
    //
    // Zero functions on the 878-TU workload, which is why the scan's disagreement
    // counter never saw it: it took a generated probe of the class's neighbours.
    if let Some(ctx) = straight_line_out_of_class_ctx(&arg_ops, &params) {
        return Err(Block::refuse(seg, off, ctx));
    }
    Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok })
}

/// **WCL — the argument list of a call whose slot 0 is already filled.**
///
/// [`tail_call_shape`]'s sibling for a **chain link**: the outer call of
/// `p->a()->b(k)`, whose receiver is the previous call's result and is therefore
/// already in r3. It is a separate locator rather than a parameter of that one
/// because almost nothing it does carries over — the two disagree on every rule
/// they both have an opinion about, and each disagreement is a capture:
///
/// * **the slot base.** Slot 0 is the receiver, so the explicit arguments start
///   at slot **1** and the first one goes to **r4**;
/// * **there is no permutation.** Every argument's value is in a callee-saved
///   GPR (that is what being live across the previous `bl` means) or is a
///   literal, and the two register files are disjoint, so no move can clobber
///   another's source and the cycle machinery has nothing to decompose. In
///   particular a value passed **twice** is two ordinary moves, not the dead
///   `mr r11` `tail_call_shape` refuses under `call-arg-duplicated`:
///   `p->Next()->gia2(j, j)` is `mr r4,r31 ; mr r5,r31`, captured;
/// * **the emission order is the opposite one**, ascending rather than
///   descending — see `c2_core::codegen::calls`, where the bytes are.
///
/// Sharing the *name* of a rule those three disagree with is how one fact grows
/// two implementations (`docs/GAPS.md` §6 #9); sharing the *code* here would be
/// how one implementation grows two facts.
///
/// `args` is the link's argument region in **stream order** (reverse source
/// order), exactly as [`eat_call_args`] hands it over, so slot `1 + i` is
/// `args[len-1-i]`. Read off the IL of `int f(O* p,int j,int k) { return
/// p->Next()->gia2(j, k); }`, whose link region is `B9 <k> … 55 · B9 <j> … 55 ·
/// 4C` — the same reversal every other argument position in this file uses, and
/// the one a private copy would have had to restate.
pub(crate) fn link_arg_slots(
    seg: &[u8],
    args: Vec<Vec<IlOp>>,
    params: &[u32],
    off: usize,
) -> Result<Vec<SlotArg>, Block> {
    let refuse = |ctx: &'static str| Block::refuse(seg, off, ctx);
    // Slot 0 is the receiver, so `n` explicit arguments occupy slots `1..=n` and
    // the last of them must still be a register. Past that a parameter is
    // stack-homed and its setup is a store, not a move — the same boundary
    // `callseq-over-eight-formals` draws for the formals list, drawn here for the
    // argument list because these two lengths are independent.
    if args.len() + LINK_FIRST_SLOT > MAX_REGISTER_FORMALS {
        return Err(refuse("mcall-chain-link-arg-overflow"));
    }
    let mut slots = Vec::with_capacity(args.len());
    for slot in 0..args.len() {
        slots.push(match args[args.len() - 1 - slot].as_slice() {
            [IlOp::Load(t)] => match params.iter().position(|&q| q == *t) {
                Some(ix) => SlotArg::Formal(ix),
                // A local, a global, or another call's result: not something the
                // save plan can have put in a callee-saved register.
                None => return Err(refuse("mcall-chain-link-arg-nonformal")),
            },
            // `li r<slot>,k` — the signed-16-bit immediate, the same bound the
            // call-sequence tail literal carries. A wider one is `lis`+`ori`.
            [IlOp::Lit(k)] => {
                if !(-0x8000..=0x7FFF).contains(k) {
                    return Err(refuse("mcall-chain-link-arg-lit-wide"));
                }
                SlotArg::Lit(*k)
            }
            // Anything computed. The operand stream would be rebased onto the
            // callee-saved register — `p->Next()->gia(k + 1)` is `addi r4,r31,1`,
            // captured — which is a second lowering of the leaf selector rather
            // than a use of it, and is the same refusal `plan_saved_gprs` makes
            // for a later statement call under `callseq-saved-computed-arg`.
            _ => return Err(refuse("mcall-chain-link-arg-computed")),
        });
    }
    Ok(slots)
}

/// Consume one **call header** — `26 <callee-tok> BD <ret TYPE> <conv> <varint
/// fn-type-id>` — and return the callee token.
///
/// Split out of [`parse_call_shape`] byte for byte so the statement-call sequence
/// ([`parse_call_sequence`]) reads the second and later calls through the same
/// decoder rather than a copy of it. Every refusal key is unchanged.
///
/// `pub(crate)` for the third importer, [`super::leaf_fp_tail`]: the FP tail call
/// has its own **argument** grammar (the integer operand vocabulary cannot spell
/// an FP value) but the identical call *head*, and a second copy of the head
/// decode is exactly the drift `docs/GAPS.md` §6 instance #9 records. In
/// particular the `call-conv` gate is here and nowhere else — a varargs callee
/// must place an FP argument in a GPR pair as well as in the FP file, and a
/// recognizer that re-read the head without that byte would emit half of it.
pub(crate) fn eat_call_head(seg: &[u8], p: &mut usize) -> Result<(u32, CallRet), Block> {
    let callee_tok = eat_callee_push(seg, p)?;
    let ret = eat_call_token(seg, p)?;
    Ok((callee_tok, ret))
}

/// What a CALL token's result TYPE is, to the one resolution any consumer needs:
/// whether it is in the **real** class.
///
/// A call whose result is a `float`/`double` makes the whole translation unit
/// carry the undefined external `_fltused` — **even when the result is discarded
/// and no FP register is touched at all**. `float gf(); void f(){ gf(); }` is a
/// bare `b ?gf@@YAMXZ` and its obj has one more symbol than the port emitted:
/// `Port=Mismatch @ offset 12`, the COFF header's `NumberOfSymbols`. That was a
/// **live wrong-bytes emit on mainline** (`docs/GAPS.md` §6 instance #14), found
/// by W36's generated sweep on the axis "the callee's return type, crossed with
/// discarded and returned", which no fixture had ever varied — every call in the
/// corpus returned `void`, `int` or a pointer.
///
/// It is instance #11's field one producer further out: `touches_floating_point`
/// enumerates the shapes whose *own body* does FP work, and a body that merely
/// **calls** an FP-returning function does none and still needs the hook. The
/// honest resolution here is a refusal, not a guess: `_fltused`'s placement is
/// measured as "after the first FP-touching function's symbol group", and whether
/// this new kind of FP-touching function participates in that ordering — and in
/// the per-TU label-counter surcharge it also drives (`docs/LABEL_COUNTER.md`
/// §1.1) — has not been captured. `call-ret-fp` is that refusal's census key, so
/// what it costs is a number rather than an argument.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CallRet {
    /// A `float`/`double`/other class-5 result.
    Real,
    /// Anything else — `void`, an integer, a pointer, an aggregate.
    Other,
}

impl CallRet {
    /// Refuse a **discarded** real result. Called at every site where the call's
    /// value is thrown away (`4C 4B`), which is exactly where nothing downstream
    /// can notice the FP-ness — the value-consuming sites are gated by their own
    /// `41` result annotation, and the FP tail call marks the function itself.
    pub(crate) fn discarded(self, seg: &[u8], off: usize) -> Result<(), Block> {
        match self {
            CallRet::Real => Err(Block::refuse(seg, off, "call-ret-fp")),
            CallRet::Other => Ok(()),
        }
    }
}

/// The `26 <callee-tok>` half of [`eat_call_head`].
///
/// Split out — byte for byte, every refusal key unchanged — because the **member**
/// call puts its receiver *between* the two halves:
/// `26 <method> · B9 <recv> <TYPE> 99 <TYPE> 00 · BD …`. W36's
/// [`super::mcall_tail`] therefore needs the halves separately, and a second copy
/// of either is the drift `docs/GAPS.md` §6 instance #9 records — in particular
/// the `call-conv` gate below, which lives in `eat_call_token` and nowhere else.
pub(crate) fn eat_callee_push(seg: &[u8], p: &mut usize) -> Result<u32, Block> {
    // 26 <tok> function/result ref.
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, "call-ref"));
    }
    // The `26 <tok>` symbol push NAMES THE CALLEE. The CALL token that follows
    // carries only a function-*type* id, so this token is the only thing that
    // distinguishes one callee from another; it is resolved through the `.gl`
    // symbol index (see `gl_symbol_index`).
    let (callee_tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "call-ref-tok"))?;
    *p += w;
    Ok(callee_tok)
}

/// The `BD <ret TYPE> <conv> <varint fn-type-id>` half of [`eat_call_head`].
pub(crate) fn eat_call_token(seg: &[u8], p: &mut usize) -> Result<CallRet, Block> {
    // The CALL token: `BD <TYPE ret> <flags> <varint fn-type-id>`. Nothing in it
    // is fixed but the `BD` — it is 8 to 13 bytes and self-delimiting field by
    // field, so it is decoded rather than matched.
    //
    // This replaces a hardcoded 6-byte "callee anchor" `00 80 01 10 00 00`,
    // which was never an anchor: it is `flags = 0` followed by the varint
    // `0x1001`, and `0x1001` is merely the first function type a single-function
    // fixture TU happens to create. True of every MVP fixture and of almost
    // nothing else — which is precisely what the `call-anchor-*` census buckets
    // were measuring.
    if !eat_byte(seg, p, 0xBD) {
        // `26 <sym>` followed by an INTRINSIC CALL rather than a `BD`. This is the
        // other half of the `0x40` production's footprint and it was the whole of
        // the `call-token-0x33` census bucket (7.4 % of blocked functions): a
        // member call whose `this` is an adjusted base pointer opens
        // `26 <method> 33 86 41 74 <2113> 40 …`, and an intrinsic result stored to
        // a symbol opens `26 <dest> 33 86 41 74 <id> 40 …`. Reported with the
        // selector so the two footprints can be summed; still `Err`, so the gate
        // is unchanged.
        if let Some(id) = intrinsic_selector(seg, *p) {
            return Err(Block {
                ctx: "call-intrinsic",
                byte: Some(0x40),
                off: *p,
                seg_len: seg.len(),
                aux: id as u64,
            });
        }
        return Err(blk(seg, *p, "call-token"));
    }
    let (_, ret_kind, _, ret_w) = read_type(seg, *p).ok_or(blk(seg, *p, "call-ret-type"))?;
    *p += ret_w;
    let ret = if ret_kind & 0x0F == TYPE_KIND_REAL_CLASS { CallRet::Real } else { CallRet::Other };
    // Calling convention: 0x00 = cdecl/stdcall, 0x04 = fastcall, 0x40 = varargs.
    // Only cdecl is in class — the others need argument-passing the port does
    // not implement, and accepting them would mis-emit rather than refuse.
    match seg.get(*p) {
        Some(0x00) => *p += 1,
        _ => return Err(blk(seg, *p, "call-conv")),
    }
    // The function-type id. NOT the callee: three different callees sharing one
    // signature produce byte-identical CALL tokens. The callee is bound from the
    // `26 <tok>` symbol push instead, so this field is decoded only to find the
    // token's end, then discarded.
    read_varint(seg, p).ok_or(blk(seg, *p, "call-fn-type-id"))?;
    Ok(ret)
}

/// Consume a call's **argument region** — `( expr 55 <TYPE> )* 4C` — and return
/// one operand stream per argument, in stream order.
///
/// Split out of [`parse_call_shape`] byte for byte, for the same reason
/// [`eat_call_head`] is. Every refusal key is unchanged.
/// **WR1 — a NAMED DATA SYMBOL's address standing as a whole call argument.**
///
/// ```text
///   26 <tok>              55 <TYPE>     f(&gI)     — the address, as it is
///   26 <tok> 2C <TYPE> b  55 <TYPE>     f(gArr)    — an array-to-pointer decay
/// ```
///
/// Returns the token, cursor left on the `55`. `None` — cursor untouched — for
/// anything else, so [`eat_call_args`] falls through to `parse_expr` and every
/// pre-existing refusal key is unchanged.
///
/// **Three things it deliberately does not consume, each a different lowering:**
///
/// * a **byte-offset run** (`33 <k> 27|28 …`). The addend is *not* folded into
///   the relocation — MEASURED (`work/wr1/probes/p2.cpp`): `g1(&gT.b)` is
///   `lis r11 ; addi r11,r11,0 ; addi r3,r11,4`, a **third** instruction and a
///   second `addi` whose base is the scratch, not the destination.
/// * a **`30` load** (`f(gI)` reading an `int` global). That is `lwz`, not
///   `addi`, and it is a different production with different bytes.
/// * a `26 <tok>` followed by `BD`, which is the **callee push** of the call
///   itself and never an argument — the same test `eat_data_designator` uses to
///   tell the two apart (`docs/IL_CALL_IN_EXPR.md` §17.5). It cannot arrive here
///   in practice (the callee is consumed before the argument region opens) and is
///   checked anyway, because reading a callee as a data address would relocate
///   `.text` against a function symbol as if it were data.
fn eat_sym_addr_arg(seg: &[u8], p: &mut usize) -> Option<u32> {
    eat_sym_addr_value(seg, p, 0x55)
}

/// **W-ARMS scratch sink — board #143's counterfactual, and NOTHING else.**
///
/// OFF unless `C2RS_SINK_OFF_ADD_ARG` is set, and therefore inert on every gate
/// lane, every fixture and every default scan. It exists to price
/// `expr-call-in-expr-recv-load-then-off-add-more` (1,038 emitted / 851 clean /
/// 267 names) the way §9.13 priced #127: one sink at the row's own refusal site,
/// one warm scan, Δ `emit-in-class` against a base measured on the same binary
/// with the sink disabled.
///
/// The construct is a **byte-offset add in a call argument** — `p->m(&t->s.k)`,
/// which c2 lowers as `addi rN,rBase,k` inside the argument permutation
/// (probe `work/warms/probe_offadd.cpp`, listing captured with `c2rs listing`):
///
/// ```text
///   ?a1@@YAXPAUS@@PAUT@@@Z:   38840008  addi r4,r4,8
///                             48000000  b    ?one@S@@QAAXPAH@Z
///   ?a3@@YAXPAUS@@PAUT@@H@Z:  7c8b2378  mr   r11,r4
///                             7ca42b78  mr   r4,r5
///                             38ab0008  addi r5,r11,8
///                             48000000  b    ?three@S@@QAAXHPAH@Z
/// ```
///
/// Two modes, because the two answer different questions and §9.13's E4 is the
/// record of registering only the one that cannot fail:
///
/// * `=honest` pushes `[Load, Lit, Add]`, which `tail_call_shape`'s slot path
///   refuses **by name**. It cannot mis-emit, and what it measures is *which
///   gate is next*.
/// * `=ceiling` pushes `[Load]` alone, dropping the offset. The census then
///   calls the body in class and the port emits `mr` where c2 emits `addi` —
///   **wrong bytes, deliberately**, which is exactly the over-claim
///   `census/gate disagreement` is structurally blind to. Its number is the
///   ceiling if the codegen existed; its *mismatches* are the demonstration.
///
/// Never a rung. The rung needs a new `SlotArg` variant and its ordering rule in
/// `crates/c2-core`, which this lane may not touch.
fn off_add_arg_sink(seg: &[u8], p: &mut usize) -> Option<Vec<IlOp>> {
    use crate::func::readers::{eat_operand_type, ValueClass};
    #[derive(Clone, Copy, PartialEq)]
    enum Mode {
        Off,
        Honest,
        Ceiling,
        Zero,
    }
    static MODE: std::sync::OnceLock<Mode> = std::sync::OnceLock::new();
    let mode = *MODE.get_or_init(|| match std::env::var("C2RS_SINK_OFF_ADD_ARG").as_deref() {
        Ok("honest") => Mode::Honest,
        Ok("ceiling") => Mode::Ceiling,
        Ok("zero") => Mode::Zero,
        _ => Mode::Off,
    });
    if mode == Mode::Off {
        return None;
    }
    // `B9 <tok> <TYPE ptr4> · ( 33 <int-like TYPE> <k> · 27 <TYPE ptr4> )+`,
    // cursor left on the `55` the caller consumes — the same contract
    // `eat_sym_addr_arg` has, and restored untouched on any refusal.
    //
    // **A RUN, and the arity is the finding.** `&t->s.k` is TWO off-adds, one
    // per designator step (`work/warms/il-p1/*.ex`: `33 …00 · 27 … · 33 …08 ·
    // 27 …`). The completeness walker's `Admit` set holds construct *classes*,
    // so granting `off-add` once and needing it twice takes the `adm.holds(blk)`
    // arm and renders `-more` with no third construct — which reads as "a
    // further construct is hiding behind this row" when what is actually behind
    // it is the SAME construct at a higher arity. A one-step recognizer measures
    // the row at a small fraction of itself; this one varies the count.
    let save = *p;
    let mut q = *p;
    let bail = |p: &mut usize| {
        *p = save;
        None::<Vec<IlOp>>
    };
    if !eat_byte(seg, &mut q, 0xB9) {
        return bail(p);
    }
    let Some((tok, w)) = read_token_var(seg, q) else { return bail(p) };
    q += w;
    if eat_operand_type(seg, &mut q) != Some(ValueClass::Ptr4) {
        return bail(p);
    }
    let mut sum: i32 = 0;
    let mut steps = 0usize;
    // Eight is past any designator chain the workload spells; a longer run is
    // refused rather than assumed to keep repeating.
    while steps < 8 {
        let mut r = q;
        if !eat_byte(seg, &mut r, 0x33) || eat_operand_type(seg, &mut r).is_none() {
            break;
        }
        let Some(k) = read_varint(seg, &mut r) else { break };
        match seg.get(r) {
            Some(&0x27) => {
                r += 1;
                if eat_operand_type(seg, &mut r) != Some(ValueClass::Ptr4) {
                    break;
                }
            }
            Some(&0x28) => r += 3,
            _ => break,
        }
        sum = match sum.checked_add(k) {
            Some(v) => v,
            None => break,
        };
        steps += 1;
        q = r;
    }
    if steps == 0 || seg.get(q) != Some(&0x55) {
        return bail(p);
    }
    // **The zero arm needs no codegen at all, and that is a byte fact.** A
    // designator chain summing to 0 addresses the base itself — `&q->m` where
    // `m` is at offset 0 is `q` — and c2 emits nothing for it: `?a2@@YAXPAUS@@0@Z`
    // and `?a6@@YAXPAUT@@PAH@Z` in `work/warms/probe_offadd.cod` are a bare
    // `b <callee>`, against `addi r4,r4,8` at offset 8. Same structure as #127's
    // 434-of-472 at adjust offset 0.
    if mode == Mode::Zero && sum != 0 {
        return bail(p);
    }
    *p = q;
    Some(match mode {
        Mode::Ceiling | Mode::Zero => vec![IlOp::Load(tok)],
        _ => vec![IlOp::Load(tok), IlOp::Lit(sum), IlOp::Add],
    })
}

/// [`eat_sym_addr_arg`], with the token that must terminate the address named by
/// the caller: `55` when it stands as a call **argument**, `99` when it stands as
/// a member call's **receiver** (`gObj.m(a)` — W-ADJUST,
/// [`super::mcall_tail::try_parse_member_tail_call`]).
///
/// One locator for one fact. The address is the same two instructions and the
/// same relocation quad in both positions, and every refusal above — the offset
/// run, the `30` load, the second convert, the callee push — is a fact about the
/// *address*, not about what consumes it. A private copy at the receiver position
/// would re-decide all four, which is `docs/GAPS.md` §6 instance #9 exactly.
///
/// Leaves the cursor **on** the terminator, so the caller consumes it with its
/// own reader (`55 <TYPE>` for an argument, [`super::mcall_tail::eat_this_bind`]
/// for a receiver), and restores it untouched on any refusal.
pub(crate) fn eat_sym_addr_value(seg: &[u8], p: &mut usize, terminator: u8) -> Option<u32> {
    let save = *p;
    if seg.get(*p) != Some(&0x26) {
        return None;
    }
    let mut q = *p + 1;
    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    // The callee push. Never an argument.
    if seg.get(q) == Some(&0xBD) {
        return None;
    }
    // At most ONE array-to-pointer / cv-strip convert, which emits nothing.
    if seg.get(q) == Some(&0x2C) {
        let mut r = q + 1;
        let (_, _, _, tw) = read_type(seg, r)?;
        r += tw;
        // The convert's trailing byte, whatever it is; a truncated one is not a
        // convert.
        seg.get(r)?;
        q = r + 1;
    }
    // The address must end HERE. An offset run, a load, or a second convert
    // means a construct with more instructions in it than the two this models.
    if seg.get(q) != Some(&terminator) {
        *p = save;
        return None;
    }
    *p = q;
    Some(tok)
}

pub(crate) fn eat_call_args(seg: &[u8], p: &mut usize) -> Result<Vec<Vec<IlOp>>, Block> {
    let mut args: Vec<Vec<IlOp>> = Vec::new();
    loop {
        if eat_byte(seg, p, 0x4C) {
            break;
        }
        // WR1: a named data symbol's address is a whole argument on its own and
        // is not an operand stream `parse_expr` can produce — the value is a
        // relocation, not a computation. Tried first; it consumes nothing unless
        // it matches the two captured spellings end to end.
        let ops = match off_add_arg_sink(seg, p) {
            Some(v) => v,
            None => match eat_sym_addr_arg(seg, p) {
                Some(tok) => vec![IlOp::SymAddr(tok)],
                None => parse_expr(seg, p, 0x55)?,
            },
        };
        // `55 <TYPE>` carries the **formal's declared type**, and it is widened in
        // step with the operand positions: a call whose argument is a pointer
        // spells it here as well as at the `B9` (`… B9 p 86 43 f4 08 · 55 86 43
        // f4 08 · 4C`, captured from `int h1(int*); int f(int* p){return h1(p);}`),
        // so admitting one without the other admits no real call site at all —
        // measured: widening only `parse_expr` moved 1,013,468 functions between
        // census keys and gained exactly **0**. The argument is in a register
        // either way; this position is an annotation, not a lowering choice.
        if !eat_byte(seg, p, 0x55) || eat_int_like_or_ptr4(seg, p).is_none() {
            // an argument whose terminator or formal type we do not model
            return Err(blk(seg, *p, "call-end"));
        }
        args.push(ops);
        if args.len() > 8 {
            // Past the eighth the arguments are stack-homed, which needs a frame.
            return Err(Block::refuse(seg, *p, "call-args-overflow"));
        }
    }
    Ok(args)
}

/// Consume a call's **post-op region** — `33 <int-like TYPE> k · (02 | 03)` — and
/// return the value the emitted `addi r3,r3,<imm>` adds. `+ k` gives `k` and
/// `- k` gives `-k`, because **the two are the same instruction**: MEASURED, one
/// probe per row at `/O1 /GS- /c` (`work/w41/probe/p1.cpp`, `p5.cpp`), and the
/// bodies differ in exactly the immediate field:
///
/// ```text
///   int f(int a){ return gf() + 20; }        … bl ?gf ; 38630014  addi r3,r3,20
///   int f(int a){ return gf() - 20; }        … bl ?gf ; 3863ffec  addi r3,r3,-20
///   int f(S* p){ return p->g() - 20; }       … bl ?g  ; 3863ffec  addi r3,r3,-20
///   int f(S* p){ return p->g() - 40000; }    … 3c63ffff addis ; 386363c0 addi   REFUSED
/// ```
///
/// **`03` was refused here and the refusal was not a measurement.** The comment
/// this replaces said "SUB/MUL (`03`/`04`) fail one of these eats" and grouped the
/// two, but they are not one fact: `- k` is `addi` with a negative immediate and
/// costs nothing, while `* k` strength-reduces to a shift/add sequence and is
/// genuinely out of class. Splitting them is worth **3,559** functions on the
/// 878-TU workload, every one of them a *member* call
/// (`expr-call-in-expr-recv-load-whole`, W41), and **0** free-function ones —
/// the row that pays for it is not the row this locator was written for, which
/// is why nobody had asked.
///
/// Shared rather than copied: [`super::mcall_tail::try_parse_member_tail_call`]
/// needs the identical region, and `GAPS.md` §6 instance #9 is the drift that
/// results when a second consumer re-reads a call region for itself.
pub(crate) fn eat_call_postop(seg: &[u8], p: &mut usize) -> Result<i32, Block> {
    // EXACTLY one literal `33 <TYPE> k` immediately followed by the operator. A
    // second call (`g(a)+g(1)` → `26 …`) or a second literal (`g(a)+1+2` → a
    // second `33 …`) fails one of these `eat`s.
    //
    // W30: the literal's TYPE goes through [`eat_int_like`], not an exact
    // `86 41 74` compare — see the call-tail literal note on
    // [`parse_call_sequence`]. `k` is a value and the emit is `addi r3,r3,k`
    // whatever width-4 integer spelling names it.
    if !eat_byte(seg, p, 0x33) || !eat_int_like(seg, p) {
        return Err(blk(seg, *p, "call-postop"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "call-postop-varint"))?;
    let k = match seg.get(*p) {
        Some(0x02) => k,
        // `- k` is `+ (-k)` in the same instruction. Negated with a checked
        // operation rather than a bare `-`: the varint is an `i64` and the range
        // test below is the only thing that bounds it, so the negation must not be
        // the thing that overflows.
        Some(0x03) => match k.checked_neg() {
            Some(n) => n,
            None => return Err(Block::refuse(seg, *p, "call-postop-wide")),
        },
        // MUL and everything else: strength-reduced or non-commutative, and
        // neither is one `addi`.
        _ => return Err(blk(seg, *p, "call-postop-op")),
    };
    *p += 1;
    // `k` must fit a single signed-16-bit `addi` immediate (the 0x24 frame).
    // Past it c2 emits `addis` + `addi`, which is a second instruction and a
    // different body length — measured on `± 40000` in `work/w41/probe/p5.cpp`.
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(Block::refuse(seg, *p, "call-postop-wide"));
    }
    Ok(k as i32)
}

/// The most formals a body this port emits may declare: past the eighth a
/// parameter is stack-homed and reading it is `lwz rD,<slot>(r1)`, not a register
/// move, which [`crate`]'s consumer `c2_core::codegen::select_text` refuses. Kept
/// in the parser so the census and the gate cannot disagree about it (the
/// under-claiming direction of `docs/GAPS.md` §6).
pub(crate) const MAX_REGISTER_FORMALS: usize = 8;

/// Parse a call shape (already positioned at the `26 <tok>` function ref): the
/// bare terminal void call, an integer tail call `return g(<arg>)` (passthrough
/// or arg-setup, plus the `g(a)+0` identity fold), the framed
/// `return g(a) + k` (k ≠ 0), or — the moment a call's result is *discarded* and
/// the body carries on — the Class A statement-call sequence
/// ([`parse_call_sequence`]). See [`parse_segment`] for the grammar; fail-closed
/// at every step. `lo` locates the formals for the arg-setup.
pub(crate) fn parse_call_shape(
    seg: &[u8],
    p: &mut usize,
    lo: usize,
    bound_to: Option<u32>,
) -> Result<BodyShape, Block> {
    let (callee_tok, ret) = eat_call_head(seg, p)?;

    // VOID terminal tail call: the `4C 4B` void call-end immediately follows the
    // CALL token (no argument setup, no consumed value), then only return
    // plumbing (no result type).
    //
    // `g(); g();` and `g(); return a+1;` used to fail right here — a second `26`
    // call or a `B9` statement stands where the return plumbing must. The first of
    // those is now the Class A sequence below; the return-plumbing attempt is
    // therefore made on a **copy** of the cursor, so a body that really is the
    // single terminal call still takes this arm and still emits the bare `b g`.
    if eat(seg, p, &[0x4C, 0x4B]) {
        // The result is thrown away — including, if it is a `float`/`double`, the
        // `_fltused` the discarded FP result still obliges the TU to declare.
        ret.discarded(seg, *p)?;
        let mut q = *p;
        if eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
            *p = q;
            return Ok(BodyShape::VoidTailCall { callee_tok });
        }
        if bound_to.is_none() {
            return parse_call_sequence(seg, p, lo, callee_tok, Vec::new());
        }
        // Preserve the original refusal for the bound-to-a-local production,
        // which has no statement-sequence form.
        eat_return_plumbing(seg, p, false, BODY_SCOPE_DEPTH)?;
        unreachable!("the plumbing parse just failed on the same cursor");
    }

    // INT call. The argument region is a **repetition**, not a single argument:
    //
    //     args := ( expr `55` <TYPE> )*  `4C`
    //
    // Each argument is a modeled sub-expression — a passthrough `B9 a INT`
    // (→ `[Load]`) or an arg-setup like `a + 1` (→ `[Load, Lit, Add]`) — followed
    // by `55 <TYPE>` carrying the *formal's* declared type, and the whole list is
    // terminated by `4C`. Arguments appear in **reverse source order**, rightmost
    // first (anchored on `parse_formals`, which reverses the `2D` stream so
    // `params[0]` is its last token; `fixtures/cpp/il_call_args2.cpp` holds the
    // `g2(a,b)` / `g2(b,a)` pair that separates the two readings).
    //
    // This used to accept exactly one argument, so every real call site blocked at
    // the second `B9` — the largest single census bucket.
    let mut args = eat_call_args(seg, p)?;
    // A call whose result is **discarded** (`4B` where the value would be
    // consumed): either the whole body — `void f(int a){ g(a); }`, which c2 tail-
    // calls exactly like the zero-argument form above — or the first statement of
    // a Class A sequence.
    if seg.get(*p) == Some(&0x4B) && bound_to.is_none() {
        ret.discarded(seg, *p)?;
        *p += 1;
        let mut q = *p;
        if eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
            *p = q;
            let params = parse_params(seg, lo)?;
            return tail_call_shape(seg, args, params, callee_tok, *p);
        }
        return parse_call_sequence(seg, p, lo, callee_tok, args);
    }
    if args.is_empty() {
        // A zero-argument int call (`return g();`). The value-consuming shapes
        // below all assume an argument region, so refuse rather than guess.
        return Err(Block::refuse(seg, *p, "call-args-none"));
    }
    // A call whose result is bound to a local that is then returned immediately —
    // `int z = g(a); return z;` — is byte-identical to `return g(a);`. c2
    // register-allocates the local and coalesces the copy, so both are a bare
    // `b <callee>`; captured on the one-, two- and three-argument forms.
    //
    // This is the `expr-call-in-expr` census bucket, and after the gate migration it
    // is the largest single blocker at 12.3% of blocked functions. It needs no new
    // codegen at all — only the IL model — so it routes to the existing tail-call
    // productions rather than growing a shape of its own.
    //
    // The local never becomes a memory object here, which is why this does not
    // reopen the store question `il_stmt_static.cpp` closed: the value is returned,
    // never written anywhere, and the shape below admits nothing between the store
    // and the return.
    if let Some(dst) = bound_to {
        //  32 <TYPE> 4B          store the call result into `dst`, discard the value
        //  [4F 01 <line>]*       a line change between the two statements
        //  B9 <dst> <TYPE> 41    load it straight back and return it
        if !eat_byte(seg, p, 0x32) || !eat_int_like(seg, p) {
            return Err(blk(seg, *p, "call-bound-store"));
        }
        if !eat_byte(seg, p, 0x4B) {
            return Err(blk(seg, *p, "call-bound-stmt-end"));
        }
        eat_opt_stmt_marker(seg, p);
        if !eat_byte(seg, p, 0xB9) {
            return Err(blk(seg, *p, "call-bound-reload"));
        }
        let (back, w) =
            read_token_var(seg, *p).ok_or(blk(seg, *p, "call-bound-reload-tok"))?;
        *p += w;
        // Anything other than reading back the very token just written is a
        // different program.
        if back != dst {
            return Err(Block::refuse(seg, *p, "call-bound-other-token"));
        }
        if !eat_int_like(seg, p) {
            return Err(blk(seg, *p, "call-bound-reload-type"));
        }
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        let params = parse_params(seg, lo)?;
        // The SAME validator the direct `return g(…)` form uses. This branch used
        // to carry its own copy, which was missing two of its gates — one wrong
        // byte and one panic; see [`tail_call_shape`].
        return tail_call_shape(seg, args, params, callee_tok, *p);
    }
    if args.len() > 1 {
        // Two or more arguments: only the pure-permutation shape is modeled, and
        // only as a tail call — validated through the one locator
        // ([`tail_call_shape`]) the bound-to-a-local form and the statement-call
        // form also use.
        let params = parse_params(seg, lo)?;
        let shape = tail_call_shape(seg, args, params, callee_tok, *p)?;
        // Only a terminal tail call: a post-op would consume the result and need
        // the framed path, which does not model multi-argument setup.
        if seg.get(*p) != Some(&0x41) {
            // `blk`, not a bare `byte: None`. The refusal IS about a byte — "the
            // token after a multi-argument call's `4C` is not the `41` result
            // annotation" — and discarding it rendered the key as
            // `call-multiarg-postop:eof`, which is what `Block::feature` prints
            // when there is no byte at all. 13,425 functions, the largest bucket in
            // the call family, filed under a name that says "end of segment" about
            // a position that is nowhere near one, with their composition
            // unsampled because the one distinguishing byte had been thrown away.
            return Err(blk(seg, *p, "call-multiarg-postop"));
        }
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        return Ok(shape);
    }
    let arg_ops = args.pop().expect("exactly one argument");
    // The single call argument is an ordinary operand stream, so it is subject to
    // the same rewriter: `g(a + a)` is not `add` + branch.
    if has_repeated_leaf(&arg_ops) {
        return Err(Block::refuse(seg, *p, "call-arg-repeated-leaf"));
    }
    // And to the same reassociation: `g(b + a)` is not the source order either.
    //
    // "The framed-call class carries no formals" is what this comment used to say,
    // and it was FALSE. It came from `MVP_FRAMED`, a pinned segment truncated at the
    // `LO` marker: a real `int f(int a) { return g(a) + 1; }` segment carries
    // `46 2D E5 09` like every other. The fixture omitted the region and the comment
    // inferred a property of the compiler from the omission — see `docs/GAPS.md` §6,
    // a truncated fixture cannot witness the region it omits. The pinned segments now
    // carry their real `53 53 26 <fn> 46 2D <formal>` prologue.
    //
    // The ordering gate is still skipped for a single operand, because it is vacuous
    // there — one leaf cannot be out of order — not because there are no formals.
    let n_loads = arg_ops.iter().filter(|o| matches!(o, IlOp::Load(_))).count();
    if n_loads > 1 {
        let formals = parse_params(seg, lo)?;
        if !leaves_ascending(&arg_ops, &formals) {
            return Err(Block::refuse(seg, *p, "call-arg-noncanonical-order"));
        }
    }
    if !additive_chain_canonical(&arg_ops) {
        return Err(Block::refuse(seg, *p, "call-arg-noncanonical-order"));
    }

    // Post-op region. EITHER the return plumbing begins directly at its `41`
    // result-type marker (no post-op → an integer tail call `return g(<arg>)`),
    // OR exactly one literal `33 <int> k` + ADD (`return g(a) + k`, framed).
    if seg.get(*p) == Some(&0x41) {
        // No post-op → integer tail call: compute the argument into r3, then
        // `b <callee>` (5-section leaf). The int analog of the void tail call;
        // `g(a)` is a bare `b g`, `g(a+1)` prepends `addi r3,r3,1`.
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        let params = parse_params(seg, lo)?;
        return tail_call_shape(seg, vec![arg_ops], params, callee_tok, *p);
    }
    let k = eat_call_postop(seg, p)?;
    eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;

    // W4b2-vi identity fold: a net post-op of 0 is NOT a framed call. `g(a)+0`
    // == `g(a)`, and the optimizer folds it to the bare `b g` (verified: the
    // `g(a)+0` obj is byte-identical to `g(a)`'s). Route it to the integer
    // tail-call production so it takes the 5-section leaf path — never the
    // 6-section framed obj (which would mis-emit a frame the reference elides).
    if k == 0 {
        let params = parse_params(seg, lo)?;
        return tail_call_shape(seg, vec![arg_ops], params, callee_tok, *p);
    }
    // A genuine `+ k` (k ≠ 0) is a framed non-leaf call — but the 6-section
    // framed path models only a **bare passthrough argument** (`g(a) + k`), not
    // arg-setup. `g(a+1) + 1` (a computed argument AND a framed post-op) is out
    // of class → reject (fail closed), never a mis-emitted framed obj.
    // The framed path takes a bare passthrough LOAD, which must still be a formal:
    // `int gi; g(gi) + 1` is a global read, not an argument already in r3.
    if matches!(arg_ops.as_slice(), [IlOp::Load(_)]) {
        let params = parse_params(seg, lo)?;
        if !arg_loads_are_formals(&arg_ops, &params) {
            return Err(Block::refuse(seg, *p, "call-arg-nonformal"));
        }
        // Past the eighth formal the value is stack-homed and its argument setup
        // is `lwz r3,<slot>(r1)`, not a register move — measured:
        // `int f(int a,…,int i){ return g(i) + 1; }` is `lwz r3,180(r1)`, and the
        // constant-body emitter used to emit *nothing* there.
        //
        // The refusal is the whole formals LIST, not just an argument past the
        // eighth, because that is the predicate `select_text` — which computes
        // this setup — actually raises. Refusing on the argument's index alone
        // would put the two out of step and re-open the census/gate disagreement
        // in the under-claiming direction (`docs/GAPS.md` §6). It is more
        // conservative than the ABI requires: `int f(int a,…,int i){ return g(a)
        // + 1; }` has its argument in r3 and would emit the plain body. Sized on
        // the 878-TU workload: **zero** functions, numerator unchanged either
        // way.
        if params.len() > MAX_REGISTER_FORMALS {
            return Err(Block::refuse(seg, *p, "framed-arg-over-eight-formals"));
        }
        // The formals list is carried, not dropped: the argument is *a* formal
        // but not necessarily the one already in r3, and c2 emits `or r3,rN,rN`
        // when it is not. Dropping the list here is how that word went missing
        // — see `c2_core::codegen::framed_call_text`.
        return Ok(BodyShape::FramedCall { add_k: k, callee_tok, params, arg_ops });
    }
    Err(Block::refuse(seg, *p, "framed-computed-arg"))
}

/// Parse the **Class A statement-call sequence** (`docs/GAPS.md` #35 step 2,
/// rung 1), positioned just past the first call's discarding `4B`.
///
/// ```text
///   seq  := stmt_call+ tail
///   stmt_call := <call head> <args> `4B`
///   tail := <void return plumbing>                          void body
///          | <call head> <args> [`33` <int> k `02`] <plumbing(result)>
///                                                           the last call's value
///          | `33` <int> k <plumbing(result)>                 `return <literal>;`
/// ```
///
/// Everything here is measured against real objs; the shapes and their bytes are
/// on [`BodyShape::CallSeq`]. Three facts this production turns on, each pinned by
/// a capture rather than assumed:
///
/// * **A single statement call with nothing after it is a TAIL call**
///   (`void f(int a){ g(a); }` → a bare `b ?g`, 5 sections, no frame), so the
///   caller tries the return plumbing before entering here and this function is
///   only ever reached with more body to parse. Emitting a frame for it would be
///   a mis-emit, not a gap.
/// * **The last call of a framed body is NOT tail-called.** `int f(){ g1();
///   return g2(); }` ends `bl ?g2 ; addi r1,r1,96 ; … ; blr`. The transform is off
///   once the function is framed.
/// * **Class A means no formal is read after the first call.** The first call's
///   arguments are evaluated before its `bl`, so a formal used only there dies
///   with it; a formal read by any later statement has to survive a call and c2
///   puts it in `r31` with a `std`/`ld` pair — Class B, a later rung, refused here
///   by name.
fn parse_call_sequence(
    seg: &[u8],
    p: &mut usize,
    lo: usize,
    first_callee: u32,
    first_args: Vec<Vec<IlOp>>,
) -> Result<BodyShape, Block> {
    parse_call_sequence_from(seg, p, lo, vec![(first_callee, first_args)], None, Vec::new())
}

/// The call-sequence loop, entered with a **prefix** of calls already read and
/// an optional guard over them.
///
/// **W10** added the second caller: [`super::guarded_seq::try_parse_guarded_seq`]
/// reads `if (x rel k) g(); [else h();]` itself and then hands the rest of the
/// body to *this* loop rather than to a copy of it. The tail forms, the
/// [`MAX_SEQ_CALLS`] bound, [`plan_saved_gprs`] and the
/// one-call-and-a-void-tail tail-call escape are therefore shared, which is the
/// `docs/GAPS.md` §6 #9 discipline: a guarded sequence cannot drift from the
/// sequence it guards, because there is one loop and not two.
pub(crate) fn parse_call_sequence_from(
    seg: &[u8],
    p: &mut usize,
    lo: usize,
    prefix: Vec<(u32, Vec<Vec<IlOp>>)>,
    guard: Option<SeqGuardShape>,
    // **W11** — `early` is the guarded early returns read ahead of this
    // sequence, in source order; empty for every other caller. Third caller of
    // this loop, and like W10's it hands the rest of the body here rather than
    // to a copy: the tail forms, the `MAX_SEQ_CALLS` bound, `plan_saved_gprs`
    // and the one-call-and-a-void-tail tail-call escape are all shared. The
    // last of those is load-bearing rather than tidy — three void early returns
    // over ONE trailing call is not a framed body at all
    // (`work/w-conv/p/probe3.cpp::w3`: three `bclr` folds and a tail `b`, 32 B,
    // no `.pdata`), and only this loop knows it.
    early: Vec<SeqEarlyReturnShape>,
) -> Result<BodyShape, Block> {
    let params = parse_params(seg, lo)?;
    // Past the eighth formal a parameter is stack-homed and `select_text` — which
    // computes every one of these calls' argument setups — refuses. Raised here so
    // the census cannot claim a body the gate declines (`docs/GAPS.md` §6, the
    // under-claiming direction).
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Block::refuse(seg, *p, "callseq-over-eight-formals"));
    }
    let mut raw: Vec<(u32, Vec<Vec<IlOp>>)> = prefix;
    let tail;
    loop {
        eat_opt_stmt_marker(seg, p);
        // (1) The body ends here: void return plumbing.
        {
            let mut q = *p;
            if eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
                *p = q;
                tail = SeqTail::Void;
                break;
            }
        }
        // (1b) …the same, written with an explicit `return;`. c2 records the
        // fallthrough as a SECOND `3A <label>` branch *to the same label* the
        // return plumbing then uses, and emits nothing for it: the two objs are
        // **byte-identical** (1090 B each, compared whole with the source path
        // held fixed and the timestamp zeroed).
        //
        // Requiring the two labels to MATCH is the whole gate. A real early
        // return branches somewhere else, and admitting that would drop a control
        // transfer on the floor — the difference between a no-op and a mis-emit is
        // exactly this token compare.
        if seg.get(*p) == Some(&0x3A) {
            if let Some((first, w)) = read_token_var(seg, *p + 1) {
                let mut q = *p + 1 + w;
                let same = seg.get(q) == Some(&0x3A)
                    && read_token_var(seg, q + 1).is_some_and(|(t, _)| t == first);
                if same && eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
                    *p = q;
                    tail = SeqTail::Void;
                    break;
                }
            }
        }
        // (2) `return <literal>;` — one `li r3,k` after the last `bl`. A literal is
        // the ONLY expression tail this rung admits: any operand read after a call
        // is a value live across it, which is Class B.
        //
        // **W30 — the literal's TYPE is read by spelling, not as an exact triple.**
        // This position required `86 41 74` (`int`) exactly, so `unsigned`, `long`,
        // `unsigned long`, an `enum`, a `const int` and a `volatile int` all
        // refused, although the emitted word is `li r3,k` in every one of them:
        // the type names the *value class*, and only the value reaches the
        // encoder. [`eat_int_like`] is the locator `2C`, `41`, `30` and W22's
        // operand positions already agree through, so this is one rule gaining a
        // call site rather than a second rule. Measured by counterfactual over the
        // 878-TU workload: **+7,771 functions**, the entire `callseq-tail-lit`
        // bucket and all of it one cause. The dominant workload spelling is
        // `86 41 08` — a width-4 signed type whose id no probe reproduced; it is
        // admitted on [`is_int4_type`]'s nibbles, which is what the four other
        // positions admit it on.
        //
        // The boundary is still real: [`eat_int_like`] requires the tag to say
        // 4-byte alignment **and** the kind to say 4-byte size, so `bool`, `char`,
        // `short`, `wchar_t`, `__int64`, `float`, `double` and pointers keep
        // refusing (`fixtures/cpp/w30_callseq_tail_intlike_neg.cpp`), and the
        // signed-16-bit `li` immediate check below is unchanged.
        //
        // [`is_int4_type`]: crate::func::readers
        if seg.get(*p) == Some(&0x33) {
            let mut q = *p;
            let k = (eat_byte(seg, &mut q, 0x33) && eat_int_like(seg, &mut q))
                .then(|| read_varint(seg, &mut q))
                .flatten()
                .ok_or(Block::refuse(seg, *p, "callseq-tail-lit"))?;
            eat_return_plumbing(seg, &mut q, true, BODY_SCOPE_DEPTH)
                .map_err(|_| Block::refuse(seg, *p, "callseq-tail-lit"))?;
            // `li rD,k` carries a signed-16-bit immediate; a wider one is
            // `lis`+`ori` and is not modeled here.
            if !(-0x8000..=0x7FFF).contains(&k) {
                return Err(Block::refuse(seg, *p, "callseq-tail-lit-wide"));
            }
            *p = q;
            tail = SeqTail::Lit(k);
            break;
        }
        // (3) Another call. Either a statement (`4B`, result discarded) or the
        // value the body returns.
        let (tok, ret) = eat_call_head(seg, p)?;
        let args = eat_call_args(seg, p)?;
        if eat_byte(seg, p, 0x4B) {
            ret.discarded(seg, *p)?;
            raw.push((tok, args));
            if raw.len() > MAX_SEQ_CALLS {
                return Err(Block::refuse(seg, *p, "callseq-too-long"));
            }
            continue;
        }
        // The value call. `41` = the result is returned as is; `33 <TYPE> k 02` =
        // returned plus a literal — the same post-op the single framed call
        // carries, and the same `addi r3,r3,k`. The literal's TYPE goes through
        // the same [`eat_int_like`] the tail literal above does: three positions
        // reading one rule, widened together on purpose. Leaving one of them on a
        // narrower gate is the shape of `docs/GAPS.md` §6 #9 — one rule, two
        // implementations, and the corpus only ever exercised the correct one.
        // Worth 0 functions on the workload today and 6 probe TUs in
        // `fixtures/cpp/w30_callseq_tail_intlike.cpp`.
        let add_k = if seg.get(*p) == Some(&0x41) {
            0
        } else {
            if !eat_byte(seg, p, 0x33) || !eat_int_like(seg, p) {
                return Err(blk(seg, *p, "callseq-postop"));
            }
            let k = read_varint(seg, p).ok_or(blk(seg, *p, "callseq-postop-varint"))?;
            if !eat_byte(seg, p, 0x02) {
                // non-ADD post-op → non-commutative / strength-reduced
                return Err(blk(seg, *p, "callseq-postop-op"));
            }
            if !(-0x8000..=0x7FFF).contains(&k) {
                return Err(Block::refuse(seg, *p, "callseq-postop-wide"));
            }
            k
        };
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        raw.push((tok, args));
        tail = SeqTail::CallValue { add_k };
        break;
    }

    // **A single call whose result is discarded and with nothing after it is a
    // TAIL call, not a framed body.** This used to be a `debug_assert` on the
    // grounds that the caller had already checked it — and the caller had not.
    //
    // Its plumbing probe runs at `BODY_SCOPE_DEPTH` and takes the tail-call arm
    // only if that probe succeeds; an **explicit `return;`** after the call does
    // not parse there, because c2 records the fallthrough as a *second*
    // `3A <label>` to the same label and only arm (1b) of the loop above knows
    // that. So `void h1(){ g1(); return; }` fell through to this production,
    // came back out with one call and a void tail, and emitted the 36-byte
    // framed body — where c2 emits the bare 4-byte `b ?g1@@YAXXZ`.
    //
    // MEASURED: `Port=Mismatch @ offset 2` against the reference obj
    // (`work/WCO/probe/ret.cpp`, `/Ox /GS- /c`; `.text` 12 B for the two
    // functions, `48000000` and `4BFFFFF8`), and it was **live on mainline** —
    // a wrong-bytes emit, not a gap. It was reachable from ordinary source and
    // from the 878-TU workload, where the `debug_assert` fires on
    // `src/system/hamobj/DancerSequence.cpp`; the release scan never saw it
    // because a `debug_assert` compiles out and 98.5 % of the workload's TUs
    // are `vocab-gap` and never byte-compared. `docs/GAPS.md` §6: an assertion
    // is not a gate, and the false-*green* direction is the hazard.
    //
    // Routed to the same [`tail_call_shape`] the caller would have used, rather
    // than refused: the body IS the tail call, and its arguments have been read
    // by the same locator.
    if raw.len() == 1 && matches!(tail, SeqTail::Void) {
        if !early.is_empty() {
            // **W11 — the same trap one production over.** `void w3(int a,int
            // b,int c){ if(a) return; if(b) return; if(c) return; v0(); }` is
            // NOT framed: c2 folds every guard to a `bclr` and tail-calls
            // `v0()` — 32 B, no `.pdata`, measured at `/O1` and `/Ox`. Falling
            // through to `tail_call_shape` below would emit a framed body and
            // silently drop three branches, which is a wrong-bytes obj that
            // still links. Named, not left to the escape.
            return Err(Block::refuse(seg, *p, "callseq-early-return-no-trailing-call"));
        }
        if guard.is_some() {
            // **W10 — a guard with nothing after it is NOT a framed body.**
            // `void f(int a){ if(a) g(); }` is fold band 2 plus a tail call
            // (`work/w-cross/p/probe2.cpp::e0`: `cmpwi cr6,r3,0 ; bnelr cr6 ;
            // b ?v0 ; blr`, 16 B, **no `.pdata`**). Emitting the 44-byte framed
            // body there would be a wrong-bytes obj that still links, so the
            // refusal is named rather than left to the escape below — which
            // would otherwise hand the guarded call to `tail_call_shape` and
            // silently drop the branch.
            return Err(Block::refuse(seg, *p, "callseq-guard-no-trailing-call"));
        }
        let (callee_tok, args) = raw.pop().expect("length checked");
        return tail_call_shape(seg, args, params, callee_tok, *p);
    }
    // **W10 — the guarded sequence is Class A only.** `probe3 P2`/`S0`/`S1` put
    // a formal in r31 beside a branch and the compare then reads r31 in one and
    // r3 in the others, depending on whether the entry block also clobbers r3.
    // That composes with the entry-block hoisting rule `guarded_seq`'s module
    // doc refuses, so it is refused here too — at the ONE place that knows the
    // saved set, after `plan_saved_gprs` has run.
    

    // Validate and normalize every call's arguments through the ONE locator every
    // other call shape uses, so the marshalling has a single implementation.
    let mut calls: Vec<SeqCall> = Vec::with_capacity(raw.len());
    for (i, (callee_tok, args)) in raw.into_iter().enumerate() {
        let (arg_ops, arg_sources) =
            match tail_call_shape(seg, args, params.clone(), callee_tok, *p)? {
                BodyShape::VoidTailCall { .. } => (Vec::new(), None),
                BodyShape::IntTailCall { arg_ops, .. } => (arg_ops, None),
                BodyShape::MultiArgTailCall { arg_sources, .. } => {
                    (Vec::new(), Some(seq_call_arg_sources(seg, *p, arg_sources)?))
                }
                // `tail_call_shape` returns exactly those three.
                _ => return Err(Block::refuse(seg, *p, "callseq-arg-shape")),
            };
        let _ = i;
        // Never a chain link: this production's calls are STATEMENTS, each
        // with its own complete argument list starting at slot 0.
        calls.push(SeqCall { callee_tok, arg_ops, arg_sources, link_args: None });
    }
    // Class A saves nothing; Class B saves one or two GPRs. Which formals, and in
    // which register, is [`plan_saved_gprs`].
    let saved = plan_saved_gprs(seg, &params, &calls, 0, *p)?;
    if guard.is_some() && (!saved.is_empty() || matches!(tail, SeqTail::Cmp { .. })) {
        return Err(Block::refuse(seg, *p, "callseq-guard-callee-saved"));
    }
    // **W11 — the same Class A restriction, at the same one place that knows the
    // saved set.** A guarded early return whose body also parks a formal in r31
    // has the compare reading r31 in some cells and r3 in others depending on
    // whether the entry block clobbers r3 (W10's `probe3 P2`/`S0`/`S1`), and
    // this class admits no entry-block move at all.
    if !early.is_empty() && (!saved.is_empty() || guard.is_some()) {
        return Err(Block::refuse(seg, *p, "callseq-early-return-callee-saved"));
    }
    Ok(BodyShape::CallSeq { params, calls, tail, saved, guard, early })
}

/// The largest number of callee-saved GPRs c2 open-codes with `std`/`ld`. At
/// **3** the prologue collapses to `bl __savegprlr_29` and the epilogue becomes a
/// tail branch into `__restgprlr_29` with no `blr` at all — a second REL24 site
/// per function, two extra `/Gy` label slots, and its own symbol-table position
/// (`docs/CODEGEN_FRAMED_CALLS.md` §2.3, §4.3, §4.4). Captured here as `u3.cpp`'s
/// neighbour `void f(int a,int b,int c,int d){ v1(a); v2(b); v3(c); v1(d); }`,
/// which is 60 B and helper-based. Refused, not guessed.
const MAX_INLINE_SAVED_GPRS: usize = 2;

/// **Which formals become callee-saved, and in what order** — the half of
/// `docs/CODEGEN_FRAMED_CALLS.md` §6 that "refused to yield a rule", closed here
/// for the call-sequence body by a refutation ladder of 12 captures.
///
/// Returns the parameter indices that take `r31`, `r30`, … in that order; empty
/// is Class A.
///
/// **The rule.** A formal read by any call *after the first* has to survive a
/// `bl`, so it is copied into a callee-saved register; the callee-saved file is
/// allocated **descending from r31 in PARAMETER order**.
///
/// ```text
///   void f(int a,int b,int c){ v1(a); v2(b); v3(c); }   72 B, F=112
///     std r30,-24(r1) ; std r31,-16(r1) ; stwu r1,-112(r1)
///     mr r31,r4 ; mr r30,r5 ; bl ?v1 ; mr r3,r31 ; bl ?v2 ; mr r3,r30 ; bl ?v3
/// ```
///
/// **Parameter order, refuted against first-use order.** The two coincide in
/// every probe `docs/CODEGEN_FRAMED_CALLS.md` §3.1 quotes, so the separating
/// capture is `void f(int a,int b,int c){ v1(a); v2(c); v3(b); }` — `c` is used
/// first. Its prologue and its two `mr` saves are **byte-identical** to the row
/// above (`mr r31,r4` = b, `mr r30,r5` = c); only the two `mr r3,rN` uses swap.
/// So the allocator walks the parameter list, not the use list.
///
/// **A formal used at the first call too is still saved** — `void f(int a){
/// v1(a); v2(a); }` emits `mr r31,r3` *before* a `bl` whose argument is already
/// in r3, so the predicate is "read by any call after the first", not "not read
/// by the first".
///
/// **Three live formals leave the class.** [`MAX_INLINE_SAVED_GPRS`].
///
/// **What is deliberately refused, with the capture that would settle it.**
/// Where the save moves go when the first call *also* needs argument marshalling
/// is measured and is not one rule: a save whose source register the marshalling
/// **overwrites** is hoisted in front of the whole marshalling, and one whose
/// source it leaves alone is emitted after it. Both halves in one capture,
/// `void f(int a,int b,int c,int d){ g2(a,d); v1(b); v2(c); }`:
///
/// ```text
///   mr r31,r4      b — r4 is about to be overwritten, so this is HOISTED
///   mr r4,r6       the marshalling (slot 1 <- d)
///   mr r30,r5      c — r5 is untouched, so this TRAILS
///   bl ?g2
/// ```
///
/// A "save as late as possible" reading predicts `mr r4,r6` first there, and is
/// **refuted** by `void f(int a,int b,int c,int d,int e){ g3(a,d,e); v1(b); }`,
/// where `mr r31,r4` precedes *both* marshalling moves although only the second
/// touches r4 — the hoist goes to the front, not to just before the writer.
/// Computing "the registers the first call's marshalling writes" needs a second
/// implementation of what the emitter does, and that is the shape of
/// `docs/GAPS.md` §6 #9, so this rung refuses a first call that needs any
/// marshalling at all while anything is saved. Cost on the 878-TU workload:
/// **0 functions** (measured by counterfactual).
///
/// `extra_saved` is the number of callee-saved GPRs the **tail** needs on top of
/// the formals — 0 for every Class A/B statement sequence, 1 for a tail that
/// keeps an earlier call's *result* live across a later `bl`
/// ([`super::mcall_cmp`]). It exists so the [`MAX_INLINE_SAVED_GPRS`] gate is
/// applied to the TOTAL: a body needing three registers is the `__savegprlr_29`
/// helper class whatever the third one holds, and a gate that counted only the
/// formals would let it through with a Class B prologue and a Class C body.
pub(crate) fn plan_saved_gprs(
    seg: &[u8],
    params: &[u32],
    calls: &[SeqCall],
    extra_saved: usize,
    p: usize,
) -> Result<Vec<usize>, Block> {
    let index_of = |t: u32| params.iter().position(|&q| q == t);
    let mut live = vec![false; params.len()];
    for c in calls.iter().skip(1) {
        if let Some(src) = &c.arg_sources {
            for &s in src {
                // `tail_call_shape` has already refused a source outside the
                // formals list (`call-arg-outer-formal`, GAPS §6 #5).
                if let Some(slot) = live.get_mut(s) {
                    *slot = true;
                }
            }
        }
        for o in &c.arg_ops {
            if let IlOp::Load(t) = o {
                if let Some(i) = index_of(*t) {
                    live[i] = true;
                }
            }
        }
        // **WCL — a chain link's arguments.** The third argument form, and it
        // has to be asked here for the same reason the other two are: a formal a
        // link reads is live across the previous `bl` and therefore has to be
        // saved. It is exactly this that makes `p->a()->b(k)` Class B while
        // `p->a()->b()` is Class A. A `Lit` costs no register — which is why the
        // literal cell stays Class A — and a link argument is never anything
        // else, because `link_arg_slots` refused it before this ran.
        for a in c.link_args.iter().flatten() {
            if let SlotArg::Formal(i) = a {
                if let Some(slot) = live.get_mut(*i) {
                    *slot = true;
                }
            }
        }
    }
    let saved: Vec<usize> = (0..params.len()).filter(|&i| live[i]).collect();
    if saved.is_empty() && extra_saved == 0 {
        return Ok(saved); // Class A — nothing survives a call.
    }
    if saved.len() + extra_saved > MAX_INLINE_SAVED_GPRS {
        return Err(Block::refuse(seg, p, "callseq-three-plus-saved"));
    }

    // The first call may marshal its own arguments beside the saves — the
    // interleaving is measured (see the doc comment) — but only where the
    // emitter can say exactly which registers that marshalling **writes**, since
    // that is what decides hoisted from trailing. A permutation's write set falls
    // out of the same cycle decomposition that produces its bytes, and a single
    // passthrough or literal argument writes r3 or nothing. A **computed**
    // argument does not qualify: under `/Ox` a chain intermediate goes to a fresh
    // *descending* register, which is the very file the saves live in, so the
    // write set is not `{r3}` and the interleaving is not the measured one.
    //
    // **A non-identity PERMUTATION at the first call is a different lowering and
    // was a live mis-emit until it was probed.** When a permuted argument's value
    // is also one of the callee-saved ones, c2 does not break the cycle with r11
    // at all — it uses the **callee-saved register itself** as the temp, because
    // the save has to happen anyway. Three witnesses, none of which contains r11:
    //
    // ```text
    //   void f(int a,int b){ g2(b,a); v1(a); v2(b); }        a->r31, b->r30
    //     mr r30,r4 ; mr r31,r3 ; mr r4,r3 ; mr r3,r30 ; bl ?g2
    //   void f(int a,int b,int c){ g2(b,a); v1(a); v2(c); }  a->r31, c->r30
    //     mr r31,r3 ; mr r3,r4 ; mr r4,r31 ; mr r30,r5 ; bl ?g2
    //   void f(int a,int b,int c){ g3(a,c,b); v1(a); v2(b); } a->r31, b->r30
    //     mr r30,r4 ; mr r4,r5 ; mr r5,r30 ; mr r31,r3 ; bl ?g3
    // ```
    //
    // Against the hoist/trail model above — which predicts the r11 walk unchanged
    // with the saves moved around it — that is **11 of 17 probes wrong**, found by
    // gridding the shape before shipping it. Which saved register serves as the
    // temp when several are saved is not determined by three captures, so this is
    // the measured edge and not a fit.
    let first = &calls[0];
    let unmodelled_first = match (&first.arg_sources, first.arg_ops.as_slice()) {
        (Some(src), _) => src.iter().enumerate().any(|(i, &s)| i != s),
        (None, []) => false,
        (None, [IlOp::Load(_)]) | (None, [IlOp::Lit(_)]) => false,
        (None, _) => true,
    };
    if unmodelled_first {
        return Err(Block::refuse(seg, p, "callseq-saved-with-first-call-setup"));
    }

    // Every later call's arguments must come **straight out of** a saved
    // register or be a literal. A computed one is `addi r3,r31,1` — the operand
    // stream rebased onto the callee-saved register, which is a second lowering
    // of `select_text` rather than a use of it. Captured
    // (`void f(int a,int b){ v1(a); v2(b + 1); }` -> `addi r3,r31,1`) and
    // refused until it goes through one locator.
    for c in calls.iter().skip(1) {
        // A **chain link** carries its arguments in `link_args` and nowhere else,
        // and `link_arg_slots` has already reduced each of them to a formal or a
        // literal — the identical predicate, asked at the point the bytes are
        // read. Its `arg_ops` is empty, so it takes the `(None, [])` arm below;
        // spelled out because "it happens to be empty" is not a reason.
        let ok = match (&c.arg_sources, c.arg_ops.as_slice()) {
            _ if c.link_args.is_some() => c.arg_sources.is_none() && c.arg_ops.is_empty(),
            (Some(_), _) => true,
            (None, []) | (None, [IlOp::Lit(_)]) => true,
            (None, [IlOp::Load(t)]) => index_of(*t).is_some(),
            (None, _) => false,
        };
        if !ok {
            return Err(Block::refuse(seg, p, "callseq-saved-computed-arg"));
        }
    }
    Ok(saved)
}

/// A bound on the statement calls one body may carry, so a corrupt stream cannot
/// make the parser build an unbounded list. Far above anything measured (the
/// widest probe is four) and far below anything a real body reaches before some
/// other production refuses it.
const MAX_SEQ_CALLS: usize = 64;

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the globs keep that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::testutil::*;
    #[allow(unused_imports)]
    use crate::func::body::{parse_segment, parse_segment_detail};
    #[allow(unused_imports)]
    use crate::func::bundle::LO_MARKER;
    #[allow(unused_imports)]
    use crate::func::readers::find_subslice;
    #[allow(unused_imports)]
    use crate::func::sy::{Formals, SyView};
    #[allow(unused_imports)]
    use crate::func::test_fixtures::*;
    /// A call argument that is not a formal must refuse — and it must refuse in the
    /// PARSER, so the census and the gate agree about it.
    ///
    /// `int gi; int g(int); int u1() { return g(gi); }` parsed as an in-class integer
    /// tail call: the multi-argument path checked its arguments against the formals
    /// list from the start, and the three single-argument paths never did. Codegen
    /// refused it downstream, so no wrong bytes were emitted — but the census counted
    /// it in class while the gate did not, and the widening order is chosen from the
    /// census. Found by a characterization agent probing the bucket; no fixture had a
    /// call whose argument was a global.
    #[test]
    fn a_call_argument_that_is_not_a_formal_refuses_in_the_parser() {
        // `INT_TAILRET` is `return g(a);` — rebind the argument LOAD to a token that
        // is not in the `2D` formals list, changing nothing else.
        let mut global_arg = INT_TAILRET.to_vec();
        let lo = find_subslice(&global_arg, &LO_MARKER).unwrap();
        let at = global_arg[lo..]
            .windows(2)
            .position(|w| w == [0xB9, 0xE5])
            .expect("the argument LOAD")
            + lo
            + 1;
        assert_eq!(parse_segment(&free_fn(INT_TAILRET), NO_LOCALS).is_some(), true, "control");
        global_arg[at] = 0xF0; // a token no `2D` entry names
        let b = parse_segment_detail(&free_fn(&global_arg), NO_LOCALS).unwrap_err();
        assert_eq!(b.ctx, "call-arg-nonformal");
    }

    /// A two-argument tail call that passes formals 0 and 2 of three must **refuse**,
    /// and above all must not take the process down.
    ///
    /// The permutation analysis sizes its `seen[]` by the argument count and indexes
    /// it with a *formal* index, so `int f(int a,int b,int c){ return g(a,c); }`
    /// panicked with `index out of bounds: the len is 2 but the index is 2` — on
    /// mainline, from `c2rs census`, on two lines of ordinary C++. The 878-TU
    /// workload never reached it because those bodies block earlier on their operand
    /// types, which is exactly why nothing caught it: a scan that is green is green
    /// only on the IL it saw.
    #[test]
    fn a_call_argument_from_a_formal_beyond_the_argument_count_refuses_and_does_not_panic() {
        let b = parse_segment_detail(ARG2_OUTER_FORMAL, NO_LOCALS).unwrap_err();
        assert_eq!(b.ctx, "call-arg-outer-formal");
        // `:mid`, and it has to be. The refusal is raised inside the call's
        // argument region, so the return plumbing after it is still unparsed and
        // a second unmodeled construct could be sitting in it. The key says so:
        // `:eof` is reserved for a refusal the parse reached the segment end
        // before raising. Positively: this block has segment left after it.
        assert!(b.off < b.seg_len, "the refusal is inside the segment, with bytes left after it");
        assert_eq!(b.feature(), "call-arg-outer-formal:mid");
        assert_eq!(parse_segment(ARG2_OUTER_FORMAL, NO_LOCALS), None);
    }

    /// The control for the refusal above: the same shape passing formals 0 and 1 —
    /// a real permutation of the argument slots — stays in class. The guard must
    /// cost nothing that was already accepted.
    #[test]
    fn a_two_argument_tail_call_over_the_leading_formals_is_still_in_class() {
        let mut inner = ARG2_OUTER_FORMAL.to_vec();
        // The `2D` formals list is in REVERSE source order and `parse_formals`
        // un-reverses it, so `E6` is `a` (index 0), `E7` is `b` and `E8` is `c`
        // (index 2) — and the argument stream is reverse source order too, so
        // `g(a,c)` pushes `c` then `a`. Rebinding the FIRST push from `c` to `b`
        // turns it into `g(a,b)`: sources `[0, 1]`, a permutation of the two
        // argument slots.
        let at = inner
            .windows(3)
            .position(|w| w == [0xB9, 0xE8, 0x09])
            .expect("the first argument push");
        inner[at + 1] = 0xE7;
        assert!(
            matches!(
                parse_segment(&inner, NO_LOCALS),
                Some(BodyShape::MultiArgTailCall { .. })
            ),
            "formals 0 and 1 are a permutation and must stay accepted"
        );
    }

    /// The **call-bound-to-a-local** form of both refusals above, which carried
    /// its own copy of the argument validation and was missing a gate at each of
    /// the two points. One locator now ([`tail_call_shape`]); this test is the
    /// pair that separates "the production refuses" from "the leaf order
    /// refuses".
    ///
    /// * `int z = g(b + a); return z;` was a **wrong-bytes emit** — c2
    ///   canonicalizes a commutative argument's leaves, so it emits the same
    ///   `add r3,r3,r4 ; b ?g` as `g(a + b)` and the port emitted `add r3,r4,r3`
    ///   (`c2rs diff`: `Port=Mismatch @ 537`).
    /// * `int z = g2(a, c); return z;` **panicked** `c2rs census`.
    ///
    /// The canonical-order control must stay in class, so the fix costs nothing
    /// that was already accepted.
    #[test]
    fn a_call_bound_to_a_local_gets_the_same_argument_gates_as_the_direct_form() {
        // The destination `z` is an automatic `int` local, which is what makes the
        // production reachable at all (`.sy` membership, not absence from `.gl`).
        let zc: [u32; 1] = [0xE909];
        let zo: [u32; 1] = [0xEB09];
        let view = |l: &'static [u32]| SyView {
            locals: l,
            ptr_locals: &[],
            formals: Formals::AllOneRegisterByConstruction,
        };
        let zc: &'static [u32] = Box::leak(Box::new(zc));
        let zo: &'static [u32] = Box::leak(Box::new(zo));
        // The wrong-bytes half: non-canonical leaves refuse …
        let b = parse_segment_detail(BOUND_ARG_NONCANON, view(zc)).unwrap_err();
        assert_eq!(b.ctx, "call-arg-noncanonical-order");
        // … and the canonical control is still an in-class integer tail call.
        assert!(
            matches!(
                parse_segment(BOUND_ARG_CANON, view(zc)),
                Some(BodyShape::IntTailCall { .. })
            ),
            "`int z = g(a + b); return z;` is byte-exact and must stay in class"
        );
        // The panic half: a formal past the argument count refuses, in the
        // parser, without indexing anything out of bounds.
        let b = parse_segment_detail(BOUND_ARG2_OUTER_FORMAL, view(zo)).unwrap_err();
        assert_eq!(b.ctx, "call-arg-outer-formal");
        assert_eq!(parse_segment(BOUND_ARG2_OUTER_FORMAL, view(zo)), None);
    }

    /// **Class A many-calls**, positive and negative, on segments transcribed from
    /// live captures. The three facts the production turns on are each one
    /// assertion here, because each is a shape c2 lowers *differently* from its
    /// neighbour:
    ///
    /// * a lone statement call is a TAIL call, not a framed body;
    /// * two statement calls are a framed body whose last call is `bl`, not `b`;
    /// * one statement call plus anything after it is already framed.
    #[test]
    fn class_a_many_calls_decode_and_the_lone_statement_call_stays_a_tail_call() {
        // Two statement calls: framed, Class A, nothing saved.
        let Some(BodyShape::CallSeq { calls, tail, params, saved, guard: None, .. }) =
            parse_segment(SEQ_TWO_VOID, NO_LOCALS)
        else {
            panic!("`g1(a); g2();` is the Class A many-call shape");
        };
        assert_eq!(params, vec![0xE609]);
        assert!(saved.is_empty(), "Class A saves nothing — the formal dies at the first call");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xE609)]);
        assert!(calls[1].arg_ops.is_empty(), "the second call takes no argument");
        assert_eq!(tail, SeqTail::Void);

        // One statement call and a literal return — framed on ONE call.
        let Some(BodyShape::CallSeq { calls, tail, .. }) =
            parse_segment(SEQ_ONE_THEN_LIT, NO_LOCALS)
        else {
            panic!("`g1(a); return 5;` is framed");
        };
        assert_eq!(calls.len(), 1);
        assert_eq!(tail, SeqTail::Lit(5));

        // The last call's value, bare and with the `+k` post-op.
        assert!(matches!(
            parse_segment(SEQ_CALL_VALUE, NO_LOCALS),
            Some(BodyShape::CallSeq { tail: SeqTail::CallValue { add_k: 0 }, .. })
        ));
        assert!(matches!(
            parse_segment(SEQ_CALL_VALUE_PLUSK, NO_LOCALS),
            Some(BodyShape::CallSeq { tail: SeqTail::CallValue { add_k: 1 }, .. })
        ));

        // A lone statement call is a TAIL call. Emitting the Class A frame for it
        // would be a wrong-bytes emit, not a gap: c2 gives it a bare `b ?g1` and
        // no `.pdata` at all.
        assert!(
            matches!(
                parse_segment(SEQ_LONE_STMT_CALL, NO_LOCALS),
                Some(BodyShape::IntTailCall { .. })
            ),
            "a lone statement call is `b ?g1`, a 5-section leaf"
        );

        // The Class A / Class B boundary: a formal read after the first call has
        // to survive a `bl`, so it is copied into `r31` and the body changes
        // class. `SEQ_LIVE_ACROSS` is `void f(int a,int b){ g1(a); g2(b); }`,
        // whose `2D` formals list is written b-then-a; `plan_saved_gprs` reads
        // parameter INDICES out of that list, so the save is index 1.
        let Some(BodyShape::CallSeq { saved, params, .. }) =
            parse_segment(SEQ_LIVE_ACROSS, NO_LOCALS)
        else {
            panic!("`g1(a); g2(b);` is the Class B many-call shape");
        };
        assert_eq!(params.len(), 2);
        assert_eq!(saved, vec![1], "b takes r31; a dies at the first call");
    }

    /// **Class B's liveness rule**, stated as a table over the axis the captures
    /// separate: which formals become callee-saved, and in what order.
    ///
    /// The register assignment is `r31, r30, …` **in parameter order**, and the
    /// separating capture for that — against the first-use order every probe in
    /// `docs/CODEGEN_FRAMED_CALLS.md` §3.1 happens to agree with — is the
    /// `use_order_is_not_the_rule` row: `v1(a); v2(c); v3(b)` allocates b→r31 and
    /// c→r30 exactly like `v1(a); v2(b); v3(c)`, and the two objs' prologues and
    /// save moves are byte-identical.
    #[test]
    fn class_b_saves_the_formals_that_survive_a_call_in_parameter_order() {
        let params = vec![0xA0u32, 0xA1, 0xA2, 0xA3];
        let call = |args: &[u32]| SeqCall {
            callee_tok: 1,
            arg_ops: args.iter().map(|t| IlOp::Load(*t)).collect(),
            arg_sources: None,
            link_args: None,
        };
        let nullary = || SeqCall {
            callee_tok: 1,
            arg_ops: Vec::new(),
            arg_sources: None,
            link_args: None,
        };
        // **WCL** — a chain LINK reading `params[i]`, which is the third way a
        // formal can be live across a `bl` and the third one this planner has to
        // see. A `Lit` link argument costs no register at all.
        let link = |slots: &[SlotArg]| SeqCall {
            callee_tok: 1,
            arg_ops: Vec::new(),
            arg_sources: None,
            link_args: Some(slots.to_vec()),
        };
        // The planner reads the decoded call list, not the byte stream — the
        // segment reaches it only so a refusal it raises can record the frame its
        // offset indexes. A one-byte stand-in is enough here and keeps the block
        // well-formed (offset 0 of a segment that has a byte 0).
        let plan = |calls: &[SeqCall]| plan_saved_gprs(&[0], &params, calls, 0, 0);

        // Nothing read after the first call: Class A, nothing saved.
        assert_eq!(plan(&[call(&[0xA0]), nullary()]).unwrap(), Vec::<usize>::new());
        // One formal live: it takes r31.
        assert_eq!(plan(&[call(&[0xA0]), call(&[0xA1])]).unwrap(), vec![1]);
        // Two: r31 then r30, ascending parameter index.
        assert_eq!(plan(&[call(&[0xA0]), call(&[0xA1]), call(&[0xA2])]).unwrap(), vec![1, 2]);
        // …and USE order does not enter it — this is the refutation row.
        assert_eq!(
            plan(&[call(&[0xA0]), call(&[0xA2]), call(&[0xA1])]).unwrap(),
            vec![1, 2],
            "use order is not the rule: c is used first and still takes r30"
        );
        // A formal read by the FIRST call too is still saved: `v1(a); v2(a);`
        // emits `mr r31,r3` before a `bl` whose argument is already in r3.
        assert_eq!(plan(&[call(&[0xA0]), call(&[0xA0])]).unwrap(), vec![0]);
        // One value, many later reads, one register.
        assert_eq!(
            plan(&[call(&[0xA0]), call(&[0xA1]), call(&[0xA1]), call(&[0xA1])]).unwrap(),
            vec![1]
        );

        // Three live formals is the `__savegprlr_29` helper class — refuse.
        let three = [call(&[0xA0]), call(&[0xA1]), call(&[0xA2]), call(&[0xA3])];
        assert_eq!(plan(&three).unwrap_err().ctx, "callseq-three-plus-saved");

        // The first call may marshal a SINGLE argument beside the saves — the
        // save is hoisted in front of it when the marshalling would overwrite its
        // source and trails it otherwise, both halves captured.
        let setup0 = [call(&[0xA1]), call(&[0xA2])]; // `v1(b)` is `mr r3,r4`
        assert_eq!(plan(&setup0).unwrap(), vec![2]);
        // …and the IDENTITY permutation is not marshalling at all.
        let id0 = [
            SeqCall {
                callee_tok: 1,
                arg_ops: Vec::new(),
                arg_sources: Some(vec![0, 1]),
                link_args: None,
            },
            call(&[0xA2]),
        ];
        assert_eq!(plan(&id0).unwrap(), vec![2]);
        // A NON-identity permutation at the first call is a different lowering:
        // c2 breaks the cycle through the callee-saved register instead of r11
        // and emits no r11 at all. The hoist/trail model is wrong on 11 of 17
        // probes there, so it is refused at the measured edge.
        let perm0 = [
            SeqCall {
                callee_tok: 1,
                arg_ops: Vec::new(),
                arg_sources: Some(vec![1, 0]),
                link_args: None,
            },
            call(&[0xA2]),
        ];
        assert_eq!(
            plan(&perm0).unwrap_err().ctx,
            "callseq-saved-with-first-call-setup"
        );
        // A computed first-call argument is refused under the same key: its write
        // set reaches the callee-saved file under `/Ox`.
        let comp0 = [
            SeqCall {
                callee_tok: 1,
                arg_ops: vec![IlOp::Load(0xA0), IlOp::Lit(1), IlOp::Add],
                arg_sources: None,
                link_args: None,
            },
            call(&[0xA2]),
        ];
        assert_eq!(
            plan(&comp0).unwrap_err().ctx,
            "callseq-saved-with-first-call-setup"
        );

        // A COMPUTED argument at a later call is `addi r3,r31,1` — the operand
        // stream rebased onto the saved register, a second lowering of
        // `select_text` rather than a use of it. Refuse.
        let comp1 = [
            call(&[0xA0]),
            SeqCall {
                callee_tok: 1,
                arg_ops: vec![IlOp::Load(0xA1), IlOp::Lit(1), IlOp::Add],
                arg_sources: None,
                link_args: None,
            },
        ];
        assert_eq!(plan(&comp1).unwrap_err().ctx, "callseq-saved-computed-arg");
        // A LITERAL argument at a later call is the same `li r3,k` as anywhere
        // else and needs no saved register of its own.
        let lit1 = [
            call(&[0xA0]),
            SeqCall {
                callee_tok: 1,
                arg_ops: vec![IlOp::Lit(5)],
                arg_sources: None,
                link_args: None,
            },
            call(&[0xA1]),
        ];
        assert_eq!(plan(&lit1).unwrap(), vec![1]);
    }

    /// W30: the call-tail literal's TYPE is read **by spelling**, not as the exact
    /// `86 41 74` triple — the whole `callseq-tail-lit` bucket (7,771 functions on
    /// the 878-TU workload) was one cause, and the emitted word is `li r3,k` for
    /// every width-4 integer spelling because only the value reaches the encoder.
    ///
    /// Written as a mutation of `SEQ_ONE_THEN_LIT` (`g1(a); return 5;`) so the
    /// only thing that varies between rows is the three-or-more bytes naming the
    /// literal's type — which is exactly the axis the old exact-triple gate was
    /// wrong about, and the axis a hand-written positive fixture would have had
    /// only one point on.
    #[test]
    fn a_call_tail_literal_takes_any_width_four_integer_spelling() {
        // `SEQ_ONE_THEN_LIT` carries `33 86 41 74 05` for the tail `return 5;`.
        let at = find_subslice(SEQ_ONE_THEN_LIT, &[0x33, 0x86, 0x41, 0x74, 0x05])
            .expect("the tail literal");
        let respell = |ty: &[u8]| {
            let mut s = SEQ_ONE_THEN_LIT[..at + 1].to_vec();
            s.extend_from_slice(ty);
            s.push(0x05);
            // The `41` result annotation names the same type.
            let rest = &SEQ_ONE_THEN_LIT[at + 5..];
            s.push(rest[0]);
            s.extend_from_slice(ty);
            s.extend_from_slice(&rest[4..]);
            s
        };

        // Every width-4 integer: the four bare triples, plus the id-carrying forms
        // an exact whitelist cannot see (an enum, a `const int`, a `volatile int`).
        for (ty, label) in [
            (&[0x86, 0x41, 0x74][..], "int (the control)"),
            (&[0x86, 0x42, 0x75][..], "unsigned"),
            (&[0x86, 0x41, 0x12][..], "long"),
            (&[0x86, 0x42, 0x22][..], "unsigned long"),
            (&[0x86, 0x41, 0x83, 0x20][..], "an enum, per-TU id 0x1003"),
            (&[0x86, 0x41, 0x08][..], "the workload's dominant spelling"),
            (&[0xA6, 0x41, 0x82, 0x20][..], "const int"),
            (&[0x96, 0x41, 0x82, 0x20][..], "volatile int"),
        ] {
            assert!(
                matches!(
                    parse_segment(&respell(ty), NO_LOCALS),
                    Some(BodyShape::CallSeq { tail: SeqTail::Lit(5), .. })
                ),
                "{label} ({ty:02X?}) must decode to the same `li r3,5` tail"
            );
        }

        // The boundary stays where `eat_int_like` draws it: the tag must say
        // 4-byte alignment AND the kind 4-byte size. Narrower, wider, FP and
        // pointer types keep refusing, by name, in the parser.
        for (ty, label) in [
            (&[0x82, 0x12, 0x30][..], "bool"),
            (&[0x82, 0x11, 0x70][..], "char"),
            (&[0x84, 0x21, 0x11][..], "short"),
            (&[0x84, 0x22, 0x71][..], "wchar_t"),
            (&[0x88, 0x85, 0x41][..], "double"),
            (&[0x86, 0x45, 0x40][..], "float"),
            (&[0x86, 0x43, 0x83, 0x08][..], "void*"),
        ] {
            let s = respell(ty);
            assert_eq!(parse_segment(&s, NO_LOCALS), None, "{label} must refuse");
            assert_eq!(
                parse_segment_detail(&s, NO_LOCALS).unwrap_err().ctx,
                "callseq-tail-lit",
                "{label} must refuse by name, in the parser"
            );
        }
    }

}

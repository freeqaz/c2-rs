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
use crate::func::body::{blk, Block, BodyShape, SeqCall, SeqTail};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_int_like_or_ptr4, eat_opt_stmt_marker, read_token_var,
    read_type, read_varint,
};
use crate::func::IlOp;

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
fn arg_loads_are_formals(arg_ops: &[IlOp], params: &[u32]) -> bool {
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
fn tail_call_shape(
    args: Vec<Vec<IlOp>>,
    params: Vec<u32>,
    callee_tok: u32,
    off: usize,
) -> Result<BodyShape, Block> {
    let refuse = |ctx: &'static str| Block { ctx, byte: None, off, aux: 0 };
    // No arguments at all: the bare `b <callee>`.
    if args.is_empty() {
        return Ok(BodyShape::VoidTailCall { callee_tok });
    }
    if args.len() > 1 {
        // Two or more arguments: only the pure-permutation shape is modeled. Every
        // argument must be a bare parameter LOAD — a computed argument would need
        // its own register and interacts with the permutation temp in ways no
        // capture covers yet.
        let mut arg_sources = Vec::with_capacity(args.len());
        for slot in 0..args.len() {
            let ops = &args[args.len() - 1 - slot];
            let tok = match ops.as_slice() {
                [IlOp::Load(t)] => *t,
                _ => return Err(refuse("call-arg-computed")),
            };
            match params.iter().position(|&t| t == tok) {
                Some(ix) => arg_sources.push(ix),
                // An argument that is not one of this function's formals (a local,
                // a global, a nested call result).
                None => return Err(refuse("call-arg-nonformal")),
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
        return Ok(BodyShape::MultiArgTailCall { params, arg_sources, callee_tok });
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
        return Err(Block { ctx, byte: None, off, aux: 0 });
    }
    Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok })
}

/// Consume one **call header** — `26 <callee-tok> BD <ret TYPE> <conv> <varint
/// fn-type-id>` — and return the callee token.
///
/// Split out of [`parse_call_shape`] byte for byte so the statement-call sequence
/// ([`parse_call_sequence`]) reads the second and later calls through the same
/// decoder rather than a copy of it. Every refusal key is unchanged.
fn eat_call_head(seg: &[u8], p: &mut usize) -> Result<u32, Block> {
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
                aux: id as u64,
            });
        }
        return Err(blk(seg, *p, "call-token"));
    }
    let (_, _, _, ret_w) = read_type(seg, *p).ok_or(blk(seg, *p, "call-ret-type"))?;
    *p += ret_w;
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
    Ok(callee_tok)
}

/// Consume a call's **argument region** — `( expr 55 <TYPE> )* 4C` — and return
/// one operand stream per argument, in stream order.
///
/// Split out of [`parse_call_shape`] byte for byte, for the same reason
/// [`eat_call_head`] is. Every refusal key is unchanged.
fn eat_call_args(seg: &[u8], p: &mut usize) -> Result<Vec<Vec<IlOp>>, Block> {
    let mut args: Vec<Vec<IlOp>> = Vec::new();
    loop {
        if eat_byte(seg, p, 0x4C) {
            break;
        }
        let ops = parse_expr(seg, p, 0x55)?;
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
            return Err(Block { ctx: "call-args-overflow", byte: None, off: *p, aux: 0 });
        }
    }
    Ok(args)
}

/// The most formals a body this port emits may declare: past the eighth a
/// parameter is stack-homed and reading it is `lwz rD,<slot>(r1)`, not a register
/// move, which [`crate`]'s consumer `c2_core::codegen::select_text` refuses. Kept
/// in the parser so the census and the gate cannot disagree about it (the
/// under-claiming direction of `docs/GAPS.md` §6).
const MAX_REGISTER_FORMALS: usize = 8;

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
    let callee_tok = eat_call_head(seg, p)?;

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
        *p += 1;
        let mut q = *p;
        if eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
            *p = q;
            let params = parse_params(seg, lo)?;
            return tail_call_shape(args, params, callee_tok, *p);
        }
        return parse_call_sequence(seg, p, lo, callee_tok, args);
    }
    if args.is_empty() {
        // A zero-argument int call (`return g();`). The value-consuming shapes
        // below all assume an argument region, so refuse rather than guess.
        return Err(Block { ctx: "call-args-none", byte: None, off: *p, aux: 0 });
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
            return Err(Block { ctx: "call-bound-other-token", byte: None, off: *p, aux: 0 });
        }
        if !eat_int_like(seg, p) {
            return Err(blk(seg, *p, "call-bound-reload-type"));
        }
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        let params = parse_params(seg, lo)?;
        // The SAME validator the direct `return g(…)` form uses. This branch used
        // to carry its own copy, which was missing two of its gates — one wrong
        // byte and one panic; see [`tail_call_shape`].
        return tail_call_shape(args, params, callee_tok, *p);
    }
    if args.len() > 1 {
        // Two or more arguments: only the pure-permutation shape is modeled, and
        // only as a tail call — validated through the one locator
        // ([`tail_call_shape`]) the bound-to-a-local form and the statement-call
        // form also use.
        let params = parse_params(seg, lo)?;
        let shape = tail_call_shape(args, params, callee_tok, *p)?;
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
        return Err(Block { ctx: "call-arg-repeated-leaf", byte: None, off: *p, aux: 0 });
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
            return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
        }
    }
    if !additive_chain_canonical(&arg_ops) {
        return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
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
        return tail_call_shape(vec![arg_ops], params, callee_tok, *p);
    }
    // Post-op `+ k`: EXACTLY one literal `33 <TYPE> k` immediately followed by
    // ADD. A second call (`g(a)+g(1)` → `26 …`), a second literal (`g(a)+1+2` →
    // a second `33 …`), or SUB/MUL (`03`/`04`) all fail one of these `eat`s.
    //
    // W30: the literal's TYPE goes through [`eat_int_like`], not an exact
    // `86 41 74` compare — see the call-tail literal note on
    // [`parse_call_sequence`]. `k` is a value and the emit is `addi r3,r3,k`
    // whatever width-4 integer spelling names it.
    if !eat_byte(seg, p, 0x33) || !eat_int_like(seg, p) {
        return Err(blk(seg, *p, "call-postop"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "call-postop-varint"))?;
    if !eat_byte(seg, p, 0x02) {
        // non-ADD post-op → non-commutative / strength-reduced
        return Err(blk(seg, *p, "call-postop-op"));
    }
    // `k` must fit a single signed-16-bit `addi` immediate (the 0x24 frame).
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(Block { ctx: "call-postop-wide", byte: None, off: *p, aux: 0 });
    }
    eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;

    // W4b2-vi identity fold: a net post-op of 0 is NOT a framed call. `g(a)+0`
    // == `g(a)`, and the optimizer folds it to the bare `b g` (verified: the
    // `g(a)+0` obj is byte-identical to `g(a)`'s). Route it to the integer
    // tail-call production so it takes the 5-section leaf path — never the
    // 6-section framed obj (which would mis-emit a frame the reference elides).
    if k == 0 {
        let params = parse_params(seg, lo)?;
        return tail_call_shape(vec![arg_ops], params, callee_tok, *p);
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
            return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 });
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
            return Err(Block { ctx: "framed-arg-over-eight-formals", byte: None, off: *p, aux: 0 });
        }
        // The formals list is carried, not dropped: the argument is *a* formal
        // but not necessarily the one already in r3, and c2 emits `or r3,rN,rN`
        // when it is not. Dropping the list here is how that word went missing
        // — see `c2_core::codegen::framed_call_text`.
        return Ok(BodyShape::FramedCall { add_k: k, callee_tok, params, arg_ops });
    }
    Err(Block { ctx: "framed-computed-arg", byte: None, off: *p, aux: 0 })
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
    let params = parse_params(seg, lo)?;
    // Past the eighth formal a parameter is stack-homed and `select_text` — which
    // computes every one of these calls' argument setups — refuses. Raised here so
    // the census cannot claim a body the gate declines (`docs/GAPS.md` §6, the
    // under-claiming direction).
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Block { ctx: "callseq-over-eight-formals", byte: None, off: *p, aux: 0 });
    }
    let mut raw: Vec<(u32, Vec<Vec<IlOp>>)> = vec![(first_callee, first_args)];
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
                .ok_or(Block { ctx: "callseq-tail-lit", byte: None, off: *p, aux: 0 })?;
            eat_return_plumbing(seg, &mut q, true, BODY_SCOPE_DEPTH)
                .map_err(|_| Block { ctx: "callseq-tail-lit", byte: None, off: *p, aux: 0 })?;
            // `li rD,k` carries a signed-16-bit immediate; a wider one is
            // `lis`+`ori` and is not modeled here.
            if !(-0x8000..=0x7FFF).contains(&k) {
                return Err(Block { ctx: "callseq-tail-lit-wide", byte: None, off: *p, aux: 0 });
            }
            *p = q;
            tail = SeqTail::Lit(k);
            break;
        }
        // (3) Another call. Either a statement (`4B`, result discarded) or the
        // value the body returns.
        let tok = eat_call_head(seg, p)?;
        let args = eat_call_args(seg, p)?;
        if eat_byte(seg, p, 0x4B) {
            raw.push((tok, args));
            if raw.len() > MAX_SEQ_CALLS {
                return Err(Block { ctx: "callseq-too-long", byte: None, off: *p, aux: 0 });
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
                return Err(Block { ctx: "callseq-postop-wide", byte: None, off: *p, aux: 0 });
            }
            k
        };
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        raw.push((tok, args));
        tail = SeqTail::CallValue { add_k };
        break;
    }

    // A single call whose result is discarded and with nothing after it is a
    // TAIL call, not a framed body — but the caller already checked that before
    // entering, so reaching it here would mean the grammar drifted.
    debug_assert!(
        raw.len() > 1 || !matches!(tail, SeqTail::Void),
        "a lone statement call with a void tail is the tail-call shape"
    );

    // Validate and normalize every call's arguments through the ONE locator every
    // other call shape uses, so the marshalling has a single implementation.
    let mut calls: Vec<SeqCall> = Vec::with_capacity(raw.len());
    for (i, (callee_tok, args)) in raw.into_iter().enumerate() {
        let (arg_ops, arg_sources) =
            match tail_call_shape(args, params.clone(), callee_tok, *p)? {
                BodyShape::VoidTailCall { .. } => (Vec::new(), None),
                BodyShape::IntTailCall { arg_ops, .. } => (arg_ops, None),
                BodyShape::MultiArgTailCall { arg_sources, .. } => (Vec::new(), Some(arg_sources)),
                // `tail_call_shape` returns exactly those three.
                _ => return Err(Block { ctx: "callseq-arg-shape", byte: None, off: *p, aux: 0 }),
            };
        let _ = i;
        calls.push(SeqCall { callee_tok, arg_ops, arg_sources });
    }
    // Class A saves nothing; Class B saves one or two GPRs. Which formals, and in
    // which register, is [`plan_saved_gprs`].
    let saved = plan_saved_gprs(&params, &calls, *p)?;
    Ok(BodyShape::CallSeq { params, calls, tail, saved })
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
fn plan_saved_gprs(params: &[u32], calls: &[SeqCall], p: usize) -> Result<Vec<usize>, Block> {
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
    }
    let saved: Vec<usize> = (0..params.len()).filter(|&i| live[i]).collect();
    if saved.is_empty() {
        return Ok(saved); // Class A — nothing survives a call.
    }
    if saved.len() > MAX_INLINE_SAVED_GPRS {
        return Err(Block { ctx: "callseq-three-plus-saved", byte: None, off: p, aux: 0 });
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
        return Err(Block { ctx: "callseq-saved-with-first-call-setup", byte: None, off: p, aux: 0 });
    }

    // Every later call's arguments must come **straight out of** a saved
    // register or be a literal. A computed one is `addi r3,r31,1` — the operand
    // stream rebased onto the callee-saved register, which is a second lowering
    // of `select_text` rather than a use of it. Captured
    // (`void f(int a,int b){ v1(a); v2(b + 1); }` -> `addi r3,r31,1`) and
    // refused until it goes through one locator.
    for c in calls.iter().skip(1) {
        let ok = match (&c.arg_sources, c.arg_ops.as_slice()) {
            (Some(_), _) => true,
            (None, []) | (None, [IlOp::Lit(_)]) => true,
            (None, [IlOp::Load(t)]) => index_of(*t).is_some(),
            (None, _) => false,
        };
        if !ok {
            return Err(Block { ctx: "callseq-saved-computed-arg", byte: None, off: p, aux: 0 });
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
        assert_eq!(b.feature(), "call-arg-outer-formal:eof");
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
        let Some(BodyShape::CallSeq { calls, tail, params, saved }) =
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
        };
        let nullary = || SeqCall { callee_tok: 1, arg_ops: Vec::new(), arg_sources: None };
        let plan = |calls: &[SeqCall]| plan_saved_gprs(&params, calls, 0);

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
            SeqCall { callee_tok: 1, arg_ops: Vec::new(), arg_sources: Some(vec![0, 1]) },
            call(&[0xA2]),
        ];
        assert_eq!(plan(&id0).unwrap(), vec![2]);
        // A NON-identity permutation at the first call is a different lowering:
        // c2 breaks the cycle through the callee-saved register instead of r11
        // and emits no r11 at all. The hoist/trail model is wrong on 11 of 17
        // probes there, so it is refused at the measured edge.
        let perm0 = [
            SeqCall { callee_tok: 1, arg_ops: Vec::new(), arg_sources: Some(vec![1, 0]) },
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
            },
        ];
        assert_eq!(plan(&comp1).unwrap_err().ctx, "callseq-saved-computed-arg");
        // A LITERAL argument at a later call is the same `li r3,k` as anywhere
        // else and needs no saved register of its own.
        let lit1 = [
            call(&[0xA0]),
            SeqCall { callee_tok: 1, arg_ops: vec![IlOp::Lit(5)], arg_sources: None },
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

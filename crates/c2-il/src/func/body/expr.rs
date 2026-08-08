use super::{blk, blk_type, Block};
use crate::func::readers::{
    eat, eat_byte, eat_int_like_or_ptr4, eat_operand_type, eat_opt_stmt_marker, eat_reinterpret_type,
    eat_value_type, is_int4_type, read_token_var, read_type, read_varint, ValueClass, INT_TYPE,
};
use crate::func::IlOp;

/// Recognize an **intrinsic-call selector** at `p`: the two-token unit
///
/// ```text
///   33 86 41 74 <varint id>   40
/// ```
///
/// and return the decoded id, or `None` if the bytes are not that shape.
/// Diagnostic only — every caller turns a hit straight into a [`Block`].
///
/// `0x40` is a **second CALL token**, the intrinsic call, occupying exactly the
/// slot `BD` occupies in an ordinary call (`docs/IL_CALL_GRAMMAR.md` §2). Its
/// callee identity is not in the token at all: it is the *preceding* `int`
/// literal, and the token itself is only `40 <TYPE result>` — no
/// calling-convention byte, no function-type id, and (unlike `2C`) **no trailing
/// field**. Two controlled nullary witnesses pin that:
///
/// ```text
///   void n_break()    { __debugbreak(); }
///     33 86 41 74 80 1f 02 00 00  40 82 07 03  4C 4B
///   void *n_retaddr() { return _ReturnAddress(); }
///     33 86 41 74 80 e5 00 00 00  40 86 43 83 08  4C  41 86 43 83 08 …
/// ```
///
/// With zero arguments the `4C` apply sits immediately after the result type, so
/// a `40 <TYPE> <varint>` reading would swallow it and leave the argument list
/// unterminated. See `docs/IL_INTRINSIC_CALL.md` §1.
///
/// Requiring the selector's type to be **exactly** `86 41 74` is deliberate: the
/// residual `expr-intrinsic-call` bucket then measures how often `0x40` is *not*
/// preceded by a plain `int` literal, which is the one structural claim this
/// decode rests on.
pub(crate) fn intrinsic_selector(seg: &[u8], p: usize) -> Option<i32> {
    if seg.get(p)? != &0x33 || seg.get(p + 1..p + 4)? != INT_TYPE {
        return None;
    }
    let mut q = p + 4;
    let id = read_varint(seg, &mut q)?;
    if seg.get(q)? != &0x40 {
        return None;
    }
    Some(id)
}

/// The census name for an intrinsic selector id, or `0xNN` when the id has not
/// been pinned.
///
/// Every name here is pinned by a **controlled fixture** whose `.gl` gave the
/// enclosing function's mangled name and whose reference obj gave the emitted
/// instructions — `fixtures/cpp/il_intrinsic_call.cpp`,
/// `il_intrinsic_nullary.cpp`, `il_intrinsic_bits.cpp` and
/// `il_intrinsic_layout.cpp`, tabulated in `docs/IL_INTRINSIC_CALL.md` §3–§4.
/// Ids observed in the real workload but *not* named there stay hex, for the
/// reason the relational-opcode table gives above: a hex bucket is a result, a
/// wrong name is a lie that survives into the roadmap. The two unnamed ids that
/// actually occur (`0xDE`/`0xDF`, 1758 sites each) are characterized in §5 —
/// trigger and literal pinned, division of labour still UNKNOWN.
///
/// The id space is a c1xx-internal table and is **not enumerable from the IL**;
/// these are the 20 ids that occur across `Dir.cpp`, `App.cpp` and `Game.cpp`
/// plus the ones the fixtures reach.
pub(crate) fn intrinsic_name(id: i32) -> String {
    let named = match id {
        // --- CRT string / memory family (ids 164..173) ---
        164 => "strcpy",
        165 => "strcmp",
        166 => "strcat",
        167 => "strlen",
        170 => "memcmp",
        172 => "memcpy",
        173 => "memset",
        // --- arithmetic / bit helpers ---
        15 => "abs",   // also `labs` — one id serves the whole name family
        17 => "fabs",
        159 => "_rotl",
        160 => "_rotr",
        226 => "_InterlockedIncrement",
        229 => "_ReturnAddress",
        236 => "__emul",
        237 => "__emulu",
        318 => "_InterlockedExchangeAdd",
        543 => "__debugbreak",
        813 => "_rotl64",
        814 => "_rotr64",
        815 => "_abs64",
        839 => "_byteswap_ushort",
        840 => "_byteswap_ulong",
        841 => "_byteswap_uint64",
        850 => "_CountLeadingZeros",
        921 => "_CountLeadingZeros64",
        1935 => "__frsqrte",
        1937 => "__fsel",
        1948 => "__mftb",
        1973 => "sqrt",
        // --- C++ runtime ---
        337 => "throw",
        // --- the class-layout family (2113..2119), the bulk of the bucket ---
        2113 => "this-adjust",       // base adjust for a member call's `this`, UNguarded
        2114 => "base-upcast",       // derived → base, null-guarded
        2115 => "base-downcast",     // base → derived, null-guarded, offset negated
        2116 => "vbase-upcast",      // through a virtual base's vbtable
        2117 => "base-member-addr",  // &member inherited from a non-virtual base
        2118 => "vbase-member-addr", // &member of a virtual base
        2119 => "dynamic-cast",
        _ => return format!("0x{:X}", id as u32),
    };
    named.to_string()
}

/// The lexical depth of a function body: `.sy` numbers the formals scope 1 and the
/// body 2, and the `.ex` scope opcodes agree.
pub(crate) const BODY_SCOPE_DEPTH: usize = 2;
/// Deeper than any real function; a stream claiming more is not one.
const MAX_SCOPE_DEPTH: usize = 64;

/// Consume any run of line markers and **lexical scope** opens and closes at a
/// statement boundary, maintaining `depth`.
///
/// `53` opens a scope; `54 <k>` closes the one at depth `k` — the operand is the
/// depth of the scope being closed, not a count of anything. Two witnesses pin
/// that reading and rule out "scopes still open": `{ … { … return y; } }` closes
/// `54 03 54 02` in its return plumbing, and `{ {…} {… return y;} }` closes its
/// first block with `54 03` and then *reopens* at 3, which a count would have
/// numbered differently.
///
/// Scopes are purely lexical for straight-line code — c2 register-allocates across
/// them and emits nothing at a brace — so this is decode only, and the shapes it
/// feeds are unchanged. It is what admits `{ int x = a + 1; { return x + 2; } }`,
/// which previously refused as `body-0x53`, the largest single blocking feature on
/// the real workload.
///
/// A close is taken only when it names the *current* depth, and never below the
/// body's own: the trailing `54 02` belongs to the return plumbing, and eating it
/// here would leave the plumbing to fail on a body that is in fact well-formed.
pub(crate) fn eat_scopes(seg: &[u8], p: &mut usize, depth: &mut usize) -> Result<(), Block> {
    loop {
        eat_opt_stmt_marker(seg, p);
        match seg.get(*p) {
            Some(&0x53) => {
                if *depth >= MAX_SCOPE_DEPTH {
                    return Err(blk(seg, *p, "scope-too-deep"));
                }
                *p += 1;
                *depth += 1;
            }
            Some(&0x54) => {
                let k = match seg.get(*p + 1) {
                    Some(&k) => k as usize,
                    None => return Err(blk(seg, *p, "scope-close-depth")),
                };
                if k != *depth || *depth <= BODY_SCOPE_DEPTH {
                    return Ok(());
                }
                *p += 2;
                *depth -= 1;
            }
            _ => return Ok(()),
        }
    }
}

/// Consume the shared statement/function-tail plumbing that follows the body
/// expression of *every* accepted shape, and require the parse to reach the end
/// of the segment (the fail-closed terminal — anything trailing rejects). With
/// `has_result_type`, a `41 <int-type>` result annotation is expected first
/// (present for an int return, absent for a void call). Layout (verified):
/// `[41 <int-like>]?` result-type · `3A <label>` branch · `[4F 01 <line>]*` ·
/// `54 <d> … 54 02` scope closes · `29 <tok>` return · `4F 12` ·
/// `47 54 01 54 00` GT-terminate · then
/// EITHER the segment end (a non-last function, split before the next `4F 1F`) OR
/// the module end `4F 02 20 00 · 4F 01 <line> · 4D` and trailing zero-fill (the
/// last function).
///
/// `3A <tok>` was previously labelled "assign", as if it stored the body
/// expression into a return temporary. It does not: it is an **unconditional
/// branch** and its operand is a label. `void f() { return; }` captures as
/// `53 3a <lbl> 3a <lbl> 54 02 29 <lbl> …` — two of them back to back with no
/// expression anywhere, so there is nothing for a store to store. The same opcode
/// carries `break`, `continue`, `goto` and the if/else join jump. Nothing here
/// depends on the distinction, since this function only skips the token, but the
/// old name would mislead anyone extending it. See `docs/IL_STMT_GRAMMAR.md`.
/// `depth` is the lexical nesting the body reached, so the run of scope closes can
/// be required exactly rather than accepted as "some `54`s". A body with no braces
/// is at depth 2 and closes `54 02`; `{ … { … return … } }` is at 3 and closes
/// `54 03 54 02`. Requiring the run to descend from the depth the *statement*
/// parse counted is what makes an unbalanced parse refuse instead of being read as
/// a shorter body — and the function tail's own `47 54 01 54 00` is the same
/// scheme two levels further out.
pub(crate) fn eat_return_plumbing(
    seg: &[u8],
    p: &mut usize,
    has_result_type: bool,
    depth: usize,
) -> Result<(), Block> {
    eat_return_head(seg, p, has_result_type, depth)?;
    eat_fn_tail(seg, p)
}

/// [`eat_return_plumbing`] up to and including `29 <label>`, without the function
/// tail.
///
/// Split out — byte for byte, no behaviour change — because one shape has a
/// **value expression between the RETURN and the tail**: a constructor's
/// `return this`. See [`super::shapes::eat_ctor_this_epilogue`]. Keeping the head
/// and the tail as named pieces means the constructor arm composes the same two
/// halves every other shape uses rather than restating either.
pub(crate) fn eat_return_head(
    seg: &[u8],
    p: &mut usize,
    has_result_type: bool,
    depth: usize,
) -> Result<(), Block> {
    if has_result_type {
        let save = *p;
        if !(eat_byte(seg, p, 0x41) && eat_int_like_or_ptr4(seg, p).is_some()) {
            *p = save;
            return Err(blk(seg, *p, "result-type"));
        }
    }
    // ASSIGN: 3A <tok>
    if !eat_byte(seg, p, 0x3A) {
        return Err(blk(seg, *p, "assign"));
    }
    let (_, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "assign-tok"))?;
    *p += w;
    // RETURN: the scope closes, innermost first, then `29 <tok>`. Each close is
    // preceded by its own `4F 01 <line>` — the source line of the `}` it closes —
    // so the marker is consumed inside the run, not once before it. A probe written
    // one function per line has no intervening markers at all and hides this
    // entirely; `il_stmt_scope.cpp` puts its braces on their own lines, which is
    // what a real translation unit looks like.
    for d in (BODY_SCOPE_DEPTH..=depth).rev() {
        eat_opt_stmt_marker(seg, p);
        if !eat(seg, p, &[0x54, d as u8]) {
            return Err(blk(seg, *p, "return-scope-close"));
        }
    }
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x29) {
        return Err(blk(seg, *p, "return"));
    }
    let (_, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "return-tok"))?;
    *p += w;
    Ok(())
}

/// The **function tail** every accepted body ends on, and the fail-closed
/// terminal: `4F 12 · 47 54 01 54 00` then EITHER the segment end (a non-last
/// function, split before the next `4F 1F`) OR the module end
/// `4F 02 20 00 · 4F 01 <line> · 4D` and its trailing zero-fill.
///
/// Split out of [`eat_return_plumbing`] — byte for byte, no behaviour change —
/// because one shape does not reach it straight from `29 <tok>`: the generated
/// empty destructor wedges its own opaque sub-object trailer in between (see
/// [`super::shapes::try_parse_empty_dtor_delegation`]). Sharing the tail keeps
/// "the parse must reach the end of the segment" in one place rather than in two
/// that could drift.
pub(crate) fn eat_fn_tail(seg: &[u8], p: &mut usize) -> Result<(), Block> {
    // Function-tail: 4F 12 · 47 54 01 54 00
    if !eat(seg, p, &[0x4F, 0x12]) || !eat(seg, p, &[0x47, 0x54, 0x01, 0x54, 0x00]) {
        return Err(blk(seg, *p, "fn-tail"));
    }
    // A non-last function's segment ends exactly here (the split cuts before the
    // next `4F 1F`). Otherwise the last function carries the module end.
    if *p == seg.len() {
        return Ok(());
    }
    if !eat(seg, p, &[0x4F, 0x02, 0x20, 0x00]) || !eat(seg, p, &[0x4F, 0x01]) {
        return Err(blk(seg, *p, "module-end"));
    }
    // The module-end marker's payload is the same varint-encoded source line as
    // every other `4F 01`, so it is four bytes longer past line 127.
    read_varint(seg, p).ok_or(blk(seg, *p, "module-end-line"))?;
    // …and then an OPTIONAL **empty module-level scope** — `53` opens scope 0
    // and `54 00` closes it, the same open/close vocabulary the return plumbing
    // above uses, with nothing between them.
    //
    // MEASURED, one TU per row, `/nologo /c /GR /O1 /Oi /EHsc`, captured through
    // `c2rs census --keep-il` (bytes are the last 12 of `.ex` with the zero-fill
    // stripped). The **discriminator is which function is LAST in the module**,
    // and the `ctl_both`/`ctl_both_rev` pair is the separating control — same two
    // functions, source order swapped:
    //
    // ```text
    //   TU                                       fns  module trailer
    //   add3 alone                                 1  4F 01 02 . . . 4D
    //   static int gi = 3; + add3                  1  4F 01 03 . . . 4D
    //   add3 + add2                                2  4F 01 03 . . . 4D
    //   static L sL(…);  then add3     (E, add3)   2  4F 01 04 . . . 4D
    //   add3  then static L sL(…);     (add3, E)   2  4F 01 04 53 54 00 4D
    //   struct L{L();~L();}; static L sL; (E, F)   2  4F 01 03 . . . 4D
    //   struct L{~L();};   static L sL;   (…, F)   2  4F 01 03 . . . 4D
    //   two static objects             (E, E)      2  4F 01 04 53 54 00 4D
    //   obj, add2, obj              (E, add2, E)   3  4F 01 05 53 54 00 4D
    // ```
    //
    // So it is present exactly when the last function is a `??__E` dynamic
    // initializer — the `??__F` atexit thunk, whose body is the same void member
    // call with an object receiver, does **not** carry it, which is why this is
    // not "a thunk was compiled" and not "the last body is a void tail call".
    // All twelve `w-r1b` `??__E` probes and both license TUs carry it; so does
    // `fixtures/cpp/il_dyninit_static.cpp`.
    //
    // **What is NOT claimed.** Why c1xx emits the empty scope is unread, and the
    // reader deliberately does not predicate on the name: `eat_fn_tail` has the
    // `.ex` bytes and no `.gl`, and a locator that needed the mangled name here
    // would be a second binding seam. Accepting it as an *optional, fully
    // anchored* three bytes between the `4F 01 <line>` and the `4D` is
    // fail-closed either way — a `53 54 00` anywhere else in the trailer still
    // refuses, and a body that reached this point with the plain trailer is
    // byte-identically unaffected.
    eat(seg, p, &[0x53, 0x54, 0x00]);
    if !eat_byte(seg, p, 0x4D) {
        return Err(blk(seg, *p, "module-end"));
    }
    // Trailing zero-fill to the end of `.ex`.
    while seg.get(*p) == Some(&0) {
        *p += 1;
    }
    if *p == seg.len() {
        Ok(())
    } else {
        Err(blk(seg, *p, "trailing"))
    }
}

/// Consume a postfix LOAD/LIT/ADD/SUB/MUL operand sub-stream, stopping (without
/// consuming) at the `stop` byte that begins the following production. Two call
/// sites, same integer-expression class: the straight-line leaf body stops at
/// the `41` result-type marker (the return plumbing); the call-argument region
/// stops at the `55` call-end marker. Fail-closed: any byte that is not a
/// modeled operand/opcode (a comparison `24`, shift `09`, bitwise `0B`, ternary
/// `43 42`, …) rejects the whole function. Requires at least one op.
///
/// `stop` is only ever tested at a token boundary, so it cannot collide with an
/// int-type byte (`86 41 74` — the `41`/`74` are consumed inside the LOAD/LIT
/// arm) or a literal varint (consumed inside the `33` arm).
///
/// ## The operand type is `int`-like **or a 4-byte pointer value**
///
/// The LOAD and LIT positions take [`eat_int_like_or_ptr4`], which is the widest
/// gate that changes nothing about what is emitted: both classes are one 4-byte
/// word in one register, the pointer classes are exactly the ones the
/// already-byte-graded pointer-identity and pointer-getter leaves lower with no
/// instruction at all, and the type here is an *annotation on a value*, not a
/// selector for a load width (that is [`super::shapes`]'s `30`, which is gated
/// separately and untouched). See `docs/IL_CALL_IN_EXPR.md` §21.
///
/// ## …and pointer operands are barred from arithmetic
///
/// `p + 1` on an `int *` is `addi r3,r3,4`, and a chain that added 1 would be
/// wrong bytes rather than a gap. MEASURED (§21.1): c1xx **pre-scales** — the
/// same body is `B9 p <int*> · 33 <long> 04 · 02`, literal 4 — so the modeled
/// chain would in fact emit the right instruction. That measurement is exactly
/// why the guard is here and not deleted:
///
/// * it is a *second* claim (that the front end scales at every arity, pointee
///   width and cv-spelling this parser can reach) on top of this rung's claim
///   (that a pointer value in a register is an int value in a register), and it
///   would need its own byte grading over its own sweep axis to ship;
/// * it costs **0** of the measured gain (§21.4) — not one gained body does
///   arithmetic on a pointer;
/// * the one wild shape that must not be admitted, the pointer difference
///   `p - q`, is `03` then `33 <int> 02` then `0A` — an arithmetic *shift* the
///   operand vocabulary already refuses — so with the guard the class fails
///   closed twice rather than once.
///
/// The guard is on the whole sub-expression, not on the adjacent token: the
/// pointer may be loaded before or after the operator, and one `Vec<IlOp>` is one
/// value, so a single check when the stream ends covers every interleaving.
///
/// ## …and a `2C` conversion is consumed where it costs no instruction
///
/// `2C <TYPE> 00` converts the value on top of the operand stack. It is admitted
/// **only when the target is that value's own [`ValueClass`]** — an int4→int4
/// conversion (a cv-strip, `int`↔`unsigned`, `long`) or a ptr4→ptr4 one (a
/// cv-strip, `T*`→`void*`), each of which is a register-to-register identity that
/// c2 emits nothing for. So the arm pushes no [`IlOp`] and no lowering changes.
/// This is not a new rule: it is the one
/// [`super::shapes::finish_indirect_load`] and
/// [`super::shapes::try_parse_ptr_identity_leaf`] have been byte-graded on since
/// the getter rungs, reached through the same [`eat_value_type`] locator. What is
/// new is the *position* — an operand of a general expression rather than a
/// leaf's single value, which is where the workload's population lives
/// (`docs/IL_CALL_IN_EXPR.md` §24).
///
/// Tracking the value's class as "the class of the last operand" is exact here
/// rather than approximate, and the argument is one line: every accepted
/// conversion preserves the class, every accepted operator over a *pointer* is
/// refused outright by the guard above, and arithmetic over int4 values yields an
/// int4 value — so an accepted sub-expression has exactly ONE class throughout.
/// Where the two could differ (`(void *)(s + 1)`, whose last operand is the
/// literal) the guard refuses the body anyway; only the census key changes.
/// `C2RS_SINK_OFF_ADD_ARG=expr` — W-ARMS's board #143 counterfactual over the
/// WHOLE of [`parse_expr`] rather than the call-argument position alone. OFF and
/// free on every gate lane and every default scan.
pub(crate) fn off_add_sink_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("C2RS_SINK_OFF_ADD_ARG").as_deref() == Ok("expr"))
}

/// `C2RS_SINK_REL=expr` — **w-cmp's board #420 counterfactual**, and the only
/// thing it exists to answer: *is `expr-cmp-eq` a FALL-THROUGH KEY?*
///
/// `expr-cmp-eq` is the head of `work/w-dclass/rerank.py`'s greedy re-ranking of
/// the FRONTIER — the one key that converts **3** TUs where every other converts
/// 0, 1 or 2. But a blocker key is the census label on the **first** refusal in a
/// body, and board **#150** has six confirmations that "the key stops being
/// reported" and "the function becomes emittable" are different events. The
/// sixth is `w-dclass` §6.1: unblocking `expr-op-0x27`, the **#1** key at 23,090
/// blocked emitted functions, was worth **six functions and zero TUs** — 23,084
/// of them were *renamed*, not converted.
///
/// This sink runs that same two-scan counterfactual on the relational family, and
/// it is **measurement-only by construction**:
///
/// * it consumes the opcode so the walk proceeds and the census reports whatever
///   the **next** unmodeled byte is — the successor key, which is the number the
///   fall-through question is about;
/// * it pushes **no** [`IlOp`]. A relational is not an `Add`, and a sink that
///   lowered it as one would be a wrong emit rather than a measurement;
/// * a walk that reaches the end **with a relational in it refuses anyway**,
///   under `expr-rel-sink-poison`. **Decoding is not accepting** — the same rule
///   the `26` and intrinsic arms carry. So the sink cannot move one obj byte
///   even when it is ON, and the poison count is itself the answer to "how many
///   bodies would this admit if a lowering existed".
///
/// OFF and free on every gate lane and every default scan.
pub(crate) fn rel_sink_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("C2RS_SINK_REL").as_deref() == Ok("expr"))
}

/// How much of the intra-body control-flow vocabulary [`branch_sink`] consumes.
///
/// Two levels, because `expr-brfalse` is raised in a place that makes one level
/// unable to answer the question the *rung* asks — see [`branch_sink`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BranchSink {
    /// The default. Nothing is consumed and every arm below is dead.
    Off,
    /// `C2RS_SINK_BRANCH=expr` — the two conditional branches, `38`/`39`, and
    /// nothing else. Answers *"is `expr-brfalse` a fall-through key in the
    /// literal sense — does the census simply report the next byte?"*
    Expr,
    /// `C2RS_SINK_BRANCH=cflow` — additionally the label `29`, the unconditional
    /// jump `3A` and the statement end `4B`: the whole intra-body control-flow
    /// token set. Answers *"if the port had a general conditional-CFG body class,
    /// would these TUs convert?"*, which [`BranchSink::Expr`] structurally
    /// cannot.
    Cflow,
    /// `C2RS_SINK_BRANCH=stmt` — additionally the scope brackets `53` and
    /// `54 <depth>`. Added **after** B1 and B2 both measured 0, because both
    /// substituted overwhelmingly to `expr-op-0x53`, and `0x53` is the statement
    /// layer's **scope-open bracket** (`shapes::control_flow::step`) — the `{`
    /// of the `then`-arm. A measurement that stops at a delimiter has not
    /// reached a construct, and reporting "the successor is `0x53`" as the
    /// answer would be reporting punctuation as work.
    Stmt,
}

/// `C2RS_SINK_BRANCH` — **w-brfalse's board #440 counterfactual**, and the one
/// thing it exists to answer: *is `expr-brfalse` a FALL-THROUGH KEY?*
///
/// `expr-brfalse` is the head of the FRONTIER ladder **after** w-cmp's
/// `C2RS_SINK_REL` correction — the one key that converts **5** TUs
/// (`IPP_basicmath_xbox`, `osfinfo`, `undname`, `mmio`, `jsonwriter`) where
/// every other converts 0, 1 or 2. But board **#150** now has seven
/// confirmations that *"the key stops being reported"* and *"the function
/// becomes emittable"* are different events, and w-cmp's own R8 removed **mass**
/// as a screen for the phenomenon: `expr-cmp-eq` was the **#12** key at 2,208
/// blocked emitted functions and was a fall-through key all the same.
///
/// **Where this key is raised is the reason there are two levels.**
/// `expr-brfalse` is `Block { ctx: "expr", byte: 0x38 }` — the fall-through arm
/// of [`parse_expr`]. On the workload's bodies the production that reaches it is
/// `super::parse_body`'s `parse_expr_classed(seg, &mut p, 0x41)`: the
/// **return-value expression** of the straight-line leaf class, tried only after
/// every non-committal shape recognizer above it has declined. So the key does
/// not say *"this body needs a branch instruction"*. It says **"the dispatcher
/// fell through to the straight-line class and the body turned out to have
/// control flow in it"** — which is structurally the position `expr-op-0x27`
/// occupies, and 0x27 is worth six emitted functions and zero TUs.
///
/// Consuming `38`/`39` alone therefore answers what the key's *name* asks and
/// **cannot** answer what the *rung* asks. [`BranchSink::Cflow`] exists so that
/// the second question gets its own arm instead of being read off the first.
///
/// Like [`rel_sink_enabled`], this is **measurement-only by construction**:
///
/// * it consumes the token so the walk proceeds and the census reports whatever
///   the **next** unmodeled byte is — the successor key, which is the number the
///   fall-through question is about;
/// * it pushes **no** [`IlOp`]. A branch is not a value, and a sink that lowered
///   one as anything would be a wrong emit rather than a measurement;
/// * a walk that reaches the end having consumed one **refuses anyway**, under
///   `expr-branch-sink-poison`. **Decoding is not accepting.** So the sink
///   cannot move one obj byte even when it is ON, and the poison count is itself
///   the answer to *"how many emitted functions was this family the LAST thing
///   in the way of"*.
///
/// OFF and free on every gate lane and every default scan.
pub(crate) fn branch_sink() -> BranchSink {
    static ON: std::sync::OnceLock<BranchSink> = std::sync::OnceLock::new();
    *ON.get_or_init(|| match std::env::var("C2RS_SINK_BRANCH").as_deref() {
        Ok("expr") => BranchSink::Expr,
        Ok("cflow") => BranchSink::Cflow,
        Ok("stmt") => BranchSink::Stmt,
        _ => BranchSink::Off,
    })
}

// -----------------------------------------------------------------------------
// The CHAIN SINK — lane `w-depth`, board **#660**.
// -----------------------------------------------------------------------------

/// How many bytes a chain-sink step consumes for one opcode, and **where that
/// width was pinned**.
///
/// Every variant here is a form some *existing* production in this tree already
/// consumes, or one `docs/IL_EXPR_LAYER.md` §0 states from a capture. Nothing is
/// inferred from an opcode's numeric neighbours: an earlier revision of
/// [`expr_opcode_name`](super::expr_opcode_name) guessed the relational opcodes
/// that way and got three of six wrong, and a wrong *width* is worse than a wrong
/// *name* — it desynchronises the stream and manufactures a fictitious successor
/// key, which is the one way this instrument could lie.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SkipForm {
    /// The opcode and nothing else.
    Bare,
    /// `<op> <TYPE>`.
    Type,
    /// `<op> <token>`.
    Tok,
    /// `<op> <TYPE> <varint>`.
    TypeVarint,
    /// `<op> <token> <TYPE>`.
    TokType,
    /// `<op> <TYPE> <token>`.
    TypeTok,
    /// `<op> <one raw byte>`.
    Byte1,
    /// `<op> <two raw bytes>`.
    Byte2,
    /// `<op> <varint> <token>`.
    VarintTok,
    /// `<op> <TYPE> <one raw byte> <varint>` — the CALL token's shape, and the
    /// **only** form in this table with a raw byte between two decoded fields.
    /// It exists because `TypeVarint` is one byte short of it and reusing that
    /// is exactly the desync this table is built to make impossible: the flags
    /// byte is `00` at 3,544,297 of 3,544,589 workload sites, so a
    /// `TypeVarint` reading consumes the `00` as the whole id and lands on the
    /// `80` of the real id's escape — a byte that opens no operand token
    /// anywhere in the grammar. See [`chain_skip_form`]'s `0xBD` row.
    TypeByteVarint,
    /// `43 <sub-opcode>` with a sub-opcode-dependent payload.
    Escape43,
    /// `4F 01 <varint>` — the line marker, and **only** that. `4F 12` is the
    /// function tail and must never be eaten here.
    Line4F,
    /// `66 <arity> <arity LEB ids>` — the **class-pair descriptor** of the
    /// 2113–2119 intrinsic family (lane `w-mass`, board **#1530**).
    ///
    /// The width is **not restated here**: the step calls
    /// [`mcall::eat_class_descriptor`], the one decoder for this token, exactly
    /// as `shapes/control_flow.rs`'s `0x66` arm does. That is the whole design.
    /// `mcall`'s own doc records two earlier readings of this token — a fixed
    /// two bytes, and a `read_token_var` token — which agree on `66 02 92 20
    /// 93 20` and both desync on the wide witnesses (`fb 8a 01`, `d3 80 02`)
    /// that `src/App.cpp` and `src/lazer/game/Game.cpp` carry. A second copy of
    /// that width in this table is the one way this instrument could lie, and
    /// the table's own header says a wrong *width* is worse than a wrong *name*.
    ClassDescr,
}

/// The **pinned** skip form of an operand-stream opcode, or `None` when this tree
/// does not know the width.
///
/// `None` is a result, not a gap to paper over: [`chain_sink_step`] turns it into
/// `expr-chain-noform-0xNN`, which reads as *"the chain cannot be measured past
/// here"* and is the honest terminal for a depth walk.
///
/// Provenance, one row per pin:
///
/// | opcode(s) | form | pinned by |
/// |---|---|---|
/// | `02` `03` `04` | Bare | [`parse_expr_classed`]'s own arms |
/// | `09` `0A` `0B` `0C` `0D` | Bare | `mcall::BARE_BINARY_OPS` |
/// | `1F`..`24` | Bare | `mcall::BARE_BINARY_OPS`, and [`rel_sink_enabled`]'s arm |
/// | `0F` | Type | the `+=` control witness in `mcall`'s own doc comment: `26 1a 0a · 33 86 41 74 03 · 0f · 86 41 74 · 4b` |
/// | `1A` | Bare | `expr_opcode_name` names it `!`; `mcall` excludes it from `BARE_BINARY_OPS` for its **arity**, not its payload |
/// | `26` | Tok | `IL_EXPR_LAYER.md` §0 — `designator := 26 <tok>` |
/// | `27` | Type | [`parse_expr_classed`]'s own `0x27` arm |
/// | `28` | Byte2 | `IL_EXPR_LAYER.md` §0 — `28 00 00` |
/// | `29` `38` `39` `3A` | Tok | [`branch_sink`]'s arms |
/// | `2C` | TypeVarint | [`parse_expr_classed`]'s own `0x2C` arm; §0 |
/// | `30` `32` | Type | `IL_EXPR_LAYER.md` §0 |
/// | `33` | TypeVarint | [`parse_expr_classed`]'s own LITERAL arm |
/// | `40` | Type | `IL_INTRINSIC_CALL.md` §1 — `40 <TYPE result>`, no trailing field |
/// | `43` | Escape43 | `IL_EXPR_LAYER.md` §8 — `43 42 <2 bytes>`, `43 37` carries nothing |
/// | `44` | Bare | `IL_EXPR_LAYER.md` §7 — payload-free at both captured sites |
/// | `4B` | Bare | [`branch_sink`]'s `Cflow` arm |
/// | `4C` | Bare | the CALL-END; **four** readers in this tree consume it as one byte, and it is confirmed on the ARGUMENT-BEARING population — see the `0x4C` row below |
/// | `4F` | Line4F | [`branch_sink`]'s `Stmt` arm |
/// | `53` | Bare, `54` | Byte1 | [`eat_scopes`] |
/// | `55` | Type | the `icall` line of `parse_segment`'s grammar — `55 INT` |
/// | `5C` | TypeVarint | `EH_RECORDS.md` §7.1's measured `5C <TYPE> <varint>`, and `control_flow.rs`'s `operand()` arm that consumes it today — see the `0x5C` row below |
/// | `66` | ClassDescr | `mcall::eat_class_descriptor`, the one decoder, called rather than restated — see the `0x66` row below |
/// | `67` | VarintTok | `IL_DECODE_REACH.md` §2 — `67 <varint vtable-byte-offset> <token>` |
/// | `9B` | TypeTok | `IL_EXPR_LAYER.md` §7 — the trailing field is a whole `read_token_var` |
/// | `B9` | TokType | [`parse_expr_classed`]'s own LOAD arm |
/// | `BD` | TypeByteVarint | `IL_CALL_GRAMMAR.md` §2.1/§2.2, and **three** readers in this crate that already consume it — see the `0xBD` row below |
///
/// Deliberately **absent**, so their absence is a decision: `1B`/`1C` (`||`/`&&`
/// — `mcall` records that no capture shows the byte at all), `64`, `66`, `3B`
/// `3C` `3D` (the switch family, whose table payload is not a fixed width),
/// **`5D`/`5E`** (the EH COUNT trailers — `EH_RECORDS.md` §7.1 gives both as
/// `<varint n> <varint state>` and no [`SkipForm`] variant can spell
/// `<varint> <varint>`, which is `0xBD`'s expressiveness problem and not a
/// missing measurement), and every byte no document in this tree names.
pub(crate) fn chain_skip_form(b: u8) -> Option<SkipForm> {
    use SkipForm::*;
    Some(match b {
        0x02 | 0x03 | 0x04 => Bare,
        // DIVIDE and MODULO (`lane w-divsplit`, board **#819**). The width is
        // read off the stream, not assumed from the neighbours: at all **4,674**
        // dc3 sites the operand token decodes to end exactly at the opcode and
        // the byte after it opens a new token — `32 <TYPE>`, a store, at 4,646
        // and `33 <TYPE> <payload>`, a literal, at 26 (`work/w-divsplit/shape.py`
        // and its `TOKEN IMMEDIATELY AFTER` table). A payload byte would have to
        // sit between those two and there is none.
        //
        // This is the SINK's width table — poisoned, environment-gated, off on
        // every gate lane and every default scan, and it pushes no [`IlOp`]. It
        // is how the successor question is asked (board **#622**: closing a
        // blocker may only move the label), and it is not an acceptance.
        0x05 | 0x06 => Bare,
        0x09 | 0x0A | 0x0B | 0x0C | 0x0D => Bare,
        0x0F => Type,
        0x1A => Bare,
        0x1F..=0x24 => Bare,
        0x26 => Tok,
        0x27 => Type,
        0x28 => Byte2,
        0x29 | 0x38 | 0x39 | 0x3A => Tok,
        0x2C => TypeVarint,
        0x30 | 0x32 => Type,
        0x33 => TypeVarint,
        // A COMPOUND-ASSIGN, `35 <TYPE>` — the same shape as `0F`, pinned by
        // `w-depth` from a capture of `src/system/math/Primes.cpp` at the
        // workload's own flags (`c2rs census … --keep-il`). The loop increment
        // of `for (int i2 = 0; primes[i2] != 0; i2++)` is
        //
        // ```text
        //   … 3A <ec09>  29 <ed09>  26 <i2>  33 86 41 74 01  >35< 86 41 74  4B …
        // ```
        //
        // — designator, literal `1`, the opcode, a 3-byte int TYPE, statement
        // end. So the WIDTH is `<TYPE>`, read straight off the stream.
        //
        // **It is deliberately not NAMED.** `0F` is `+=` on `mcall`'s own `x +=
        // 3` control and this occupies the identical slot with the identical
        // payload, so `35` is somewhere in the increment/compound-assign family
        // — but *which* member (post-increment discarding its value, a distinct
        // `+= 1`, something else) is not decided by one witness, and
        // `expr_opcode_name`'s header states the rule: a hex bucket is a result,
        // a wrong name is a lie that survives into the roadmap. The instrument
        // needs the width and not the name.
        0x35 => Type,
        0x40 => Type,
        // The RESULT ANNOTATION, `41 <int-like>` — `eat_return_plumbing`'s own
        // first field. It is `parse_expr`'s usual `stop` byte, so naming it is
        // how a chain walk is taken past the first `return`; see the loop head.
        0x41 => Type,
        0x43 => Escape43,
        0x44 => Bare,
        0x4B => Bare,
        // The CALL-END, `4C` — `BD`'s own closing bracket (`lane w-4c`, board
        // **#1383**). `4B` was in this table and `4C` was not, for exactly
        // `0xBD`'s reason one opcode further along: **four** readers in this
        // tree consume it as one byte today — `control_flow.rs`'s `operand()`
        // arm (`s.p += 1`, plus the EH bookkeeping that deliberately lives
        // here), [`mcall::eat_call_args_region`](super::mcall) (the *accepting*
        // parser's argument loop, i.e. the argument-bearing path itself),
        // `codec.rs`'s `ExToken::Lo`, and `codec.rs`'s **`IntCallEnd` =
        // `55 <INT TYPE> 4C`**, which *emits* an argument-bearing call end with
        // nothing after the `4C`.
        //
        // **Board #1318 had this measurement and refused to ship it, and that
        // refusal was right.** `w-bd` scored `4C` payload-free at 26,701 of
        // 26,701 sites — and every one of those is a **zero-argument** call,
        // the byte a `BD` token ends on. The `4C` that closes a call *with*
        // arguments is 2.46 M of the 3.5 M `BD` tokens and was not in that grid
        // at all. A green control is a statement about the population it ran
        // over, and a width pinned on the unrepresentative half is the failure
        // this table exists to prevent.
        //
        // So the evidence below is the OTHER population.
        //
        // Witnessed a first way by a capture taken at this master
        // (`work/w-4c/probe/ce_args.cpp`): calls with 0, 1, 2 and 3 arguments
        // whose callees differ only in arity, so the argument region's LENGTH
        // is the only field that moves —
        //
        // ```text
        //   0 args   BD 86 41 74 00 80 01 10 00 00                          >4C< 41
        //   1 arg    BD … B9 <x> 86 41 74 · 55 86 41 74                     >4C< 41
        //   2 args   BD … (B9 <x> 86 41 74 · 55 86 41 74) ×2                >4C< 41
        //   3 args   BD … (B9 <x> 86 41 74 · 55 86 41 74) ×3                >4C< 41
        // ```
        //
        // — graded `ReferenceReplay=ByteExact`, and the cdecl/`int` half
        // (`ce_args_min.cpp`, three functions, **all** argument-bearing)
        // **`Port=Match`**: the accepting parser walked those argument regions,
        // read the `4C` as one byte, and the obj is byte-exact against real
        // `c2.dll` under wibo. `c2rs census` on the same probe puts its own `>`
        // marker on the byte after a one-argument call's `4C`
        // (`… 55 86 41 74 4C >26<`), from a reader `lane w-4c` did not touch.
        //
        // Confirmed a second way on the workload (`work/w-4c/argwalk.py`), over
        // **1,978,436** argument-bearing sites in 876 dc3 TUs — the population
        // #1318 excluded, at **74×** the size of the grid it declined on. The
        // site is anchored by walking the argument region from a pinned `BD`
        // with `control_flow.rs`'s `operand()` widths and stopping AT the first
        // `4C`, **never stepping over one**, so the closing `4C`'s position is
        // fixed by the *other* tokens' widths and finding it does not
        // presuppose its own answer. 99.56 % of the non-zero-argument
        // population walks. The rivals:
        //
        // ```text
        //   P   payload-free        457 desyncs   (and see below — really 0)
        //   B1  4C <one raw byte>   1,460,194     73.8 %
        //   T   4C <TYPE>             214,003     and no room at all at 87.7 %
        //   K   4C <token>          1,371,969     69.3 %
        // ```
        //
        // A second, walk-free anchor (`55 <TYPE> 4C` at the emitter's own
        // `eat_int_like_or_ptr4` gate) sees 3,647,883 sites and lands strictly
        // inside a walked call's bracket **zero** times, so the two anchors
        // never disagree about a position.
        //
        // **The 457 are not desyncs of this reading, and that is measured
        // rather than argued** (`work/w-4c/unwit.py`). Every one lands on
        // `0x59` (446) or `0x08` (11) — bytes outside the judging vocabulary
        // because `operand()` refuses them, `08` being one of the six it lists
        // as deliberately unwitnessed. The control never mentions `4C`: the
        // walk *breaks* at `4C`, so a `59` it reaches was preceded by some
        // other token. `0x59` is reached at a token start **6,031** times and
        // `0x08` **3,819** times, and **not one of either follows a `4C`**.
        // They are opcodes in their own right, so the byte after these `4C`s is
        // an opcode position and the reading is intact.
        //
        // **What this does NOT do**: `4C` is a closing bracket, not an operand,
        // and this table is width-only — a chain walk that steps past a `4C` is
        // not thereby matching brackets. The sink pushes no [`IlOp`] and
        // poisons any walk that used it, so nothing here can move an obj byte.
        0x4C => Bare,
        0x4F => Line4F,
        0x53 => Bare,
        0x54 => Byte1,
        0x55 => Type,
        // The EH LIVE-STATE marker, `5C <TYPE> <varint state>` (`lane w-5c`,
        // board **#1423**). Emitted at the end of a statement in which an object
        // with a destructor became live — `docs/EH_RECORDS.md` §7.1, which
        // measured the width in 2026-07-31 and named the row it was under
        // (`cf-expr-0x5C`, *"309,804 bodies, the largest single row on the
        // control-flow axis"*).
        //
        // **This is `0xBD`'s diagnosis one family along, with the enum's half of
        // it already solved.** `5C` was never unwitnessed:
        // `control_flow.rs`'s `operand()` reads exactly this width **today**
        // (`cf-eh-live-type` then `cf-eh-live-state`, then `eh.live_stmts += 1`),
        // and four `ctor_dtor.rs` recognizers eat `5C <TYPE> <state>` inside
        // shapes the differential grades byte-exact. What was missing was the
        // ROW — `TypeVarint` could already spell it, so unlike `0xBD` this was a
        // plain omission and not an expressiveness one. A width table that
        // cannot spell a width it already knows refuses for the wrong reason;
        // one that simply forgot to write it down refuses for no reason at all.
        //
        // **It is NOT a bracket, and that distinction is why the estimate for
        // this rung is different from `4C`'s.** `4C` closes every call — one
        // floor under every call site in a body at once, which is how `w-4c`
        // came in at +109 % of its own prereg. `5C` is a *statement-terminal
        // trailer over a narrow population*: measured over the workload, the
        // bodies that carry one carry a **median of 1** and a mean of **1.245**
        // (292,839 of 335,772 carry exactly one), against `4C`'s 3.54 M sites.
        //
        // Witnessed a first way by a capture taken at THIS master
        // (`work/w-5c/probe/eh5c.cpp`, read out by `probe/read_5c.py`), where
        // the object's TYPE is the only field that moves across the rows:
        //
        // ```text
        //   void one_local()  { MemA s; }          5C a6 43 81 20 01  >4B<
        //   void two_locals() { MemA s; MemB t; }  5C a6 43 81 20 01  >4B<
        //                                          5C a6 43 8a 20 01  >4B<
        //   int userfn(int a){ MemA s; g(a); … }   5C 86 41 74 01     >4B<   (3-byte TYPE)
        //   struct HasMem { MemA m; int k; }       5C 86 46 80 20 01  >4B<
        // ```
        //
        // — graded `ReferenceReplay=ByteExact` against real `c2.dll` under wibo,
        // and `work/w-5c/probe/eh5c_min.cpp` (three generated destructors, **all
        // three bodies carrying a `5C`**, census 3/3 in class) **`Port=Match`**:
        // the port emitted an obj byte-exact from IL containing this token.
        //
        // Confirmed a second way on the workload (`work/w-5c/scwalk.py`), over
        // **335,716** anchored sites in 876 dc3 TUs. **Anchor A is non-circular
        // by construction**: the walk starts at the tree's own `LO_MARKER`
        // (`4C 4F 11 53`), uses `control_flow.rs`'s widths — a *different* table
        // from this one (board #1320) with the whole `5C`/`5D`/`5E` family
        // REMOVED — and **stops AT the first `5C`, never over one**, so the
        // site's position is fixed by the other tokens' widths. The rivals:
        //
        // ```text
        //   TV  5C <TYPE> <varint>        0 desyncs   0.000 %   <- this reading
        //   P   payload-free        335,716         100.000 %
        //   T   5C <TYPE>           210,570          62.723 %
        //   V   5C <varint>         130,991          39.018 %
        //   TT  5C <TYPE> <token>    59,181          17.628 %
        // ```
        //
        // `w-divsplit`'s decisive question — *is there anywhere for a payload to
        // be?* — answers **100.00 %** here, the exact opposite of `4C`'s 12.27 %:
        // the byte after a `5C` has bit 7 set at every one of the 335,716 sites,
        // so it opens a TYPE and the payload-free reading has nowhere to hide.
        // And a fixed width is dead on its face: the TYPE is 3 B at 197,660
        // sites, 4 B at 64,437, 5 B at 65,173, 6 B at 7,197 and 7 B at 1,249.
        //
        // A second, walk-free anchor (`55 <TYPE> 4C 5C` — `w-4c`'s own
        // argument-closing call-end, at the gate the emitter applies there) sees
        // **37,742** sites at **0** TV desyncs, and lands inside a token anchor A
        // stepped **zero** times, so the two anchors never disagree about a
        // position.
        //
        // **The STRUCTURE, which is not the same as the result.**
        // `control_flow.rs`'s own comment says the `5C` *"is the last token of
        // its statement (it stands immediately before the `4B`)"*. Under this
        // reading it is, at **275,112 of 335,716 (81.95 %)**. The other 18.05 %
        // land on `9B` (50,692), `55` (9,007), `99`, `26`, `30` — the
        // OPERAND-position spelling `docs/EH_RECORDS.md` §7.2 records beside the
        // statement one (*"both spellings occur in one probe"*), not a desync.
        //
        // **The one field the anchored population could not decide, and the
        // population that decides it.** `TypeVarint` and a hypothetical
        // `TypeByte1` agree at every state below `0x80`, and the anchored walk
        // reaches **0** sites where the state escapes — `0xBD`'s §2.2 situation,
        // where the corpus excluded neither reading. Here it does not have to
        // be: the escaped sites exist, they are just inside bodies whose walk
        // abandons at an unpinned opcode first. An over-inclusive raw scan
        // (every `5C` byte with a readable TYPE after it — the bias is stated
        // and the BASE RATE is printed beside the result) finds **9,744** sites
        // whose state byte is `80`:
        //
        // ```text
        //   varint (`80 <LE32>`)   lands on 4B  9,645 / 9,744  98.98 %
        //   one raw byte           lands on 4B      0 / 9,744   0.00 %
        //   base rate over all 544,783 raw `5C <TYPE>` positions   60.66 %
        // ```
        //
        // So the varint reading is 38 points above the base rate on that class
        // and the byte reading is 61 below it. It is decided, and it agrees with
        // the ACCEPTING side: `operand()` reads a varint there too, and a sink
        // that read a field differently from the reader beside it would report a
        // successor that reader can never reach.
        //
        // **What this does NOT do**: `5D` and `5E`, the count trailers, stay
        // unpinned. `EH_RECORDS.md` §7.1 gives both as `<varint n> <varint>` from
        // the same probes and **`SkipForm` has no variant that can spell
        // `<varint> <varint>`** — which is `0xBD`'s expressiveness problem, and
        // an enum change is not this row. See
        // [`the_unpinned_opcodes_refuse_rather_than_guess_a_width`].
        0x5C => TypeVarint,
        // The CLASS-PAIR DESCRIPTOR, `66 <arity> <arity LEB ids>` (lane
        // `w-mass`, board **#1530**). It is the token that follows the 2113–2119
        // class-layout intrinsic's `40 <TYPE result>`, and the reason the
        // `intrinsic` sink stops one token short of that production: with `66`
        // absent from this table, sinking the intrinsic renamed **17,693
        // emitted** functions (291,002 bodies) to `expr-class-descriptor` and
        // recovered nothing — board **#1465**'s rename, at the largest scale the
        // project has measured it.
        //
        // **Not a new width.** `mcall::eat_class_descriptor` is the one decoder
        // and this arm calls it; `shapes/control_flow.rs`'s `0x66` arm already
        // calls the same function, and `mcall`'s doc comment for it records the
        // two rival readings (fixed 2 bytes; a `read_token_var` token) that both
        // desync on the wide witnesses `fb 8a 01` / `e0 91 01` / `d3 80 02` in
        // `src/App.cpp` and `src/lazer/game/Game.cpp`, and the `55` argument
        // terminator that pins LEB against both. This is `0x5C`'s situation
        // exactly: a width the tree already knew and this table could not spell.
        0x66 => ClassDescr,
        0x67 => VarintTok,
        // The BIND, `99 <TYPE> <varint>`. `IL_EXPR_LAYER.md` §7 pins the field
        // by CONTRAST — "its trailing field is a whole `read_token_var`, not the
        // varint the adjacent `99` uses" — and `mcall`'s transcribed capture
        // shows it: `99 · 86 43 9C 20 · 00 · BD …`.
        0x99 => TypeVarint,
        0x9B => TypeTok,
        0xB9 => TokType,
        // The CALL, `BD <TYPE ret> <flags:1 raw byte> <varint fn-type-id>`
        // (`lane w-bd`, board **#1314**). `w-front3` (board **#1289**) measured
        // that `expr-chain-noform-0xBD` is the terminal of **9** of the
        // seventeen FRONTIER ladders — the instrument running out, not the TU.
        //
        // **`0xBD` was never unwitnessed, and that is the finding.** It is the
        // most heavily witnessed opcode in this crate: `IL_CALL_GRAMMAR.md`
        // §2.1 states the grammar and §2.2 pins the flags byte from a
        // three-convention probe, and **three** readers here already consume
        // exactly this width —
        // [`mcall::eat_call_and_args`](super::mcall) (the *accepting* parser,
        // graded byte-exact by the differential), `mcall`'s call-form walk, and
        // `control_flow.rs`'s `cf-call-fn-type-id` arm. What was missing was a
        // `SkipForm` able to *express* it: every existing variant is one byte
        // short or one field long, so `BD` fell through to `None` and the
        // instrument reported a floor as a price. A width table that cannot
        // spell a width it already knows refuses for the wrong reason.
        //
        // Witnessed a first way by a capture taken at THIS master
        // (`work/w-bd/probe/bd_cc.cpp`): four externals differing only in
        // calling convention, byte-identical `int` return type, and the flags
        // byte is the only field that moves —
        //
        // ```text
        //   __cdecl      26 <cd> BD 86 41 74 >00< 80 01 10 00 00
        //   __stdcall    26 <sc> BD 86 41 74 >00< 80 02 10 00 00
        //   __fastcall   26 <fc> BD 86 41 74 >04< 80 07 10 00 00
        //   varargs      26 <va> BD 86 41 74 >40< 80 06 10 00 00
        // ```
        //
        // — graded `ReferenceReplay=ByteExact` against real `c2.dll` under
        // wibo, and its cdecl half `Port=Match`, so the port reads a token of
        // this width and emits an obj byte-exact from it.
        //
        // Confirmed a second way on the workload rather than on a probe, the
        // way `05`/`06` were: over **3,544,589** anchored `BD` sites in 876 dc3
        // TUs (`work/w-bd/bdwalk.py`) this reading lands on a byte that opens an
        // operand token at **3,544,566**, and the rivals do not — dropping the
        // flags byte lands on the `80` of the id's own escape at **3,544,480**,
        // and stopping after the TYPE lands on the flags byte itself at
        // **3,544,319**. The 23 residual sites are `BD` bytes inside a
        // neighbouring TYPE's LEB payload or inside a 4-byte token, screened
        // mechanically and printed in full.
        //
        // **One thing this does NOT settle**, stated because the corpus cannot:
        // whether the flags field is a raw byte or a `read_varint`. Every value
        // it takes here is `00` (3,544,297), `40` (171) or `04`, all below
        // `0x80`, where the two readings agree — so they agree at
        // **3,544,501 of 3,544,589** sites and the corpus excludes neither.
        // `IL_CALL_GRAMMAR.md` §2.2 already carries that as UNKNOWN. The tie is
        // broken by matching the ACCEPTING parser, which reads a raw byte: a
        // sink that read a field differently from the acceptor would report a
        // successor key the acceptor can never reach, which is the one way this
        // instrument could lie.
        0xBD => TypeByteVarint,
        _ => return None,
    })
}

/// The chain sink's configuration, parsed once from `C2RS_SINK_CHAIN`.
///
/// The variable is a comma-separated list of **sink tokens**:
///
/// * `op:NN` — consume opcode `0xNN` in [`parse_expr_classed`] through
///   [`chain_skip_form`];
/// * `type` — the operand-TYPE gate: a LOAD or LITERAL whose TYPE is neither
///   int-like nor a 4-byte pointer is skipped rather than refused
///   (`expr-load-type-*` / `expr-lit-type-*`);
/// * `convert` — the `2C` target-type gate (`expr-convert-target`);
/// * `intrinsic` — the two-token intrinsic-call unit `33 <int> <id> 40 <TYPE>`.
///
/// Anything else in the list is a **hard error at first use**, reported as
/// `expr-chain-badtoken`, because a typo that silently disabled a sink step
/// would show up as a *shallower* chain — a number that flatters the instrument.
pub(crate) struct ChainSink {
    ops: [bool; 256],
    ty: bool,
    convert: bool,
    intrinsic: bool,
    any: bool,
    bad: bool,
}

impl Default for ChainSink {
    fn default() -> ChainSink {
        ChainSink {
            ops: [false; 256],
            ty: false,
            convert: false,
            intrinsic: false,
            any: false,
            bad: false,
        }
    }
}

impl ChainSink {
    fn parse(spec: &str) -> ChainSink {
        let mut c = ChainSink::default();
        for tok in spec.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            c.any = true;
            if let Some(hex) = tok.strip_prefix("op:") {
                match u8::from_str_radix(hex, 16) {
                    Ok(b) => c.ops[b as usize] = true,
                    Err(_) => c.bad = true,
                }
            } else {
                match tok {
                    "type" => c.ty = true,
                    "convert" => c.convert = true,
                    "intrinsic" => c.intrinsic = true,
                    _ => c.bad = true,
                }
            }
        }
        c
    }
}

/// `C2RS_SINK_CHAIN` — **lane `w-depth`'s board #660 instrument**, and the one
/// thing it exists to answer: *how DEEP is a body's parse chain?*
///
/// A blocker key is the census label on the **FIRST** refusal in a body. Board
/// **#622** measured that closing `xboxheap`'s `expr-op-0x27` moves the label to
/// `expr-op-0x32` and converts nothing, and boards **#420**/**#440** measured
/// whole families being *substituted* rather than removed. What none of them
/// could measure is the **length of the chain** — and a selector over the
/// frontier needs the length, not the head.
///
/// This sink generalises [`rel_sink_enabled`] and [`branch_sink`] from two fixed
/// opcode families to a **data-driven set**, so the chain can be walked one step
/// at a time without a new arm per step. It keeps every property those two have,
/// and the properties are the whole design:
///
/// * it consumes the token so the walk proceeds and the census reports whatever
///   the **next** unmodeled byte is — the successor key, which is what depth is
///   made of;
/// * it pushes **no** [`IlOp`], ever. Not one arm. A sink that lowered anything
///   would be a wrong emit rather than a measurement;
/// * a walk that reaches the end having used one **refuses anyway**, under
///   `expr-chain-sink-poison`. **Decoding is not accepting.** So the sink cannot
///   move one obj byte even when it is ON;
/// * an opcode whose width this tree has not pinned refuses under
///   `expr-chain-noform-0xNN` instead of guessing. See [`chain_skip_form`].
///
/// **`C2RS_SINK_OFF_ADD_ARG` is NOT in this family and must not be used as a
/// chain step.** Its `0x27` arm pushes `IlOp::Add` and has no poison — it is a
/// real widening behind an environment variable, which is why board **#403**
/// records `cargo test --workspace --release` going to *16 targets / 754 passed
/// / 2 failed* under it. Board **#622**'s `0x27 → 0x32` successor was measured
/// through it and is therefore a successor under a parser that also *accepts*
/// differently; board **#661** re-derives it here under the poison.
///
/// OFF and free on every gate lane and every default scan.
pub(crate) fn chain_sink() -> &'static ChainSink {
    static ON: std::sync::OnceLock<ChainSink> = std::sync::OnceLock::new();
    ON.get_or_init(|| match std::env::var("C2RS_SINK_CHAIN") {
        Ok(spec) => ChainSink::parse(&spec),
        Err(_) => ChainSink::default(),
    })
}

/// One chain-sink step at `p`, or `None` when the byte there is not sunk.
///
/// `Ok(q)` is the position after the token; `Err(key)` is an honest refusal —
/// either the width is unpinned, or the payload ran off the end of the segment.
fn chain_sink_step(seg: &[u8], p: usize) -> Option<Result<usize, &'static str>> {
    chain_step_with(chain_sink(), seg, p)
}

/// [`chain_sink_step`] against an explicit configuration.
///
/// Split out for the tests and for one reason only: [`chain_sink`] is a
/// `OnceLock` over a process-wide environment variable, so a test that wanted to
/// exercise a *particular* sink set could otherwise only do it by being the only
/// test in its process. That is the shape that makes a safety property go
/// unchecked.
fn chain_step_with(
    c: &ChainSink,
    seg: &[u8],
    p: usize,
) -> Option<Result<usize, &'static str>> {
    if !c.any {
        return None;
    }
    if c.bad {
        return Some(Err("expr-chain-badtoken"));
    }
    let b = *seg.get(p)?;
    if !c.ops[b as usize] {
        return None;
    }
    let Some(form) = chain_skip_form(b) else {
        return Some(Err("expr-chain-noform"));
    };
    let mut q = p + 1;
    let ty = |q: &mut usize| match read_type(seg, *q) {
        Some((_, _, _, w)) => {
            *q += w;
            true
        }
        None => false,
    };
    let tok = |q: &mut usize| match read_token_var(seg, *q) {
        Some((_, w)) => {
            *q += w;
            true
        }
        None => false,
    };
    let ok = match form {
        SkipForm::Bare => true,
        SkipForm::Type => ty(&mut q),
        SkipForm::Tok => tok(&mut q),
        SkipForm::TypeVarint => ty(&mut q) && read_varint(seg, &mut q).is_some(),
        SkipForm::TokType => tok(&mut q) && ty(&mut q),
        SkipForm::TypeTok => ty(&mut q) && tok(&mut q),
        SkipForm::Byte1 => {
            q += 1;
            q <= seg.len()
        }
        SkipForm::Byte2 => {
            q += 2;
            q <= seg.len()
        }
        SkipForm::VarintTok => read_varint(seg, &mut q).is_some() && tok(&mut q),
        // `BD <TYPE> <flags:1 raw byte> <varint>`. The middle field is stepped
        // over, never decoded: its own encoding is UNKNOWN (`IL_CALL_GRAMMAR.md`
        // §2.2) and a *step* only needs the width, which both candidate
        // readings agree on at every value the corpus contains. `q += 1` is
        // bounds-checked because the varint read that follows cannot be trusted
        // to fail on a cursor already past the end.
        SkipForm::TypeByteVarint => {
            ty(&mut q) && {
                q += 1;
                q <= seg.len() && read_varint(seg, &mut q).is_some()
            }
        }
        // `43 42 <2 bytes>` is the conditional expression and `43 37` carries
        // nothing; every other sub-opcode is unpinned and says so.
        SkipForm::Escape43 => match seg.get(q) {
            Some(&0x42) => {
                q += 3;
                q <= seg.len()
            }
            Some(&0x37) => {
                q += 1;
                true
            }
            _ => return Some(Err("expr-chain-noform")),
        },
        // The line marker only. `4F 12` is the function tail and eating it here
        // would walk the sink straight through the end of the body.
        SkipForm::Line4F => {
            if seg.get(q) != Some(&0x01) {
                return Some(Err("expr-chain-noform"));
            }
            q += 1;
            read_varint(seg, &mut q).is_some()
        }
        // `66 <arity> <arity LEB ids>` — delegated to the one decoder, which
        // starts at the opcode itself, so this rewinds `q` to `p` rather than
        // re-implementing the `66` check.
        SkipForm::ClassDescr => {
            let mut r = p;
            match super::mcall::eat_class_descriptor(seg, &mut r) {
                Some(_) => {
                    q = r;
                    true
                }
                None => false,
            }
        }
    };
    Some(if ok { Ok(q) } else { Err("expr-chain-short") })
}

/// Skip a whole TYPE at `p` for the `type` / `convert` chain-sink tokens.
///
/// A TYPE is self-delimiting ([`read_type`]), so this needs no width table and
/// no classification — which is the point: the operand-type gate refuses on the
/// type's *class*, not on its *length*, and the chain step is exactly "stop
/// asking what class it is".
fn chain_skip_type(seg: &[u8], p: &mut usize) -> Result<(), Block> {
    match read_type(seg, *p) {
        Some((_, _, _, w)) => {
            *p += w;
            Ok(())
        }
        None => Err(blk(seg, *p, "expr-chain-short")),
    }
}

/// The bytes that belong to the **statement and control-flow** layers rather
/// than to an expression, read by exactly one fence — the member-call value
/// model's, in [`parse_expr_classed`]'s generic fall-through.
///
/// **Not a new vocabulary.** Every byte here is already enumerated somewhere in
/// this crate as *not an expression token*, and the list is the union of those
/// enumerations rather than a judgement:
///
/// ```text
///   32                 store            super::mcall::Stop::Store
///   41 4B              statement end    super::mcall::Stop::StmtEnd
///   55                 argument end     super::mcall::Stop::ArgEnd
///   38 39 29 3A        branch/label/jump   `BranchSink::Cflow`'s own doc calls
///                                          these "the intra-body control-flow
///                                          vocabulary"; `parse_expr` consumes
///                                          them ONLY under a sink
///   4F 53 54           line marker, scope brackets   the `BranchSink::Stmt`
///                                          arms, and board #441 ("punctuation,
///                                          not a construct")
///   4D                 end of stream    `super::mcall`'s `Stop::Eof` neighbour
///   5C                 EH live state    wb-eh §5.3 — c2 DERIVES a pass from it
/// ```
///
/// One list rather than a predicate per byte: the set answers one question —
/// *did the walk run off the end of the expression?* — and a second copy of it
/// would be the one-fact-two-locators defect `docs/GAPS.md` §6 records.
fn is_statement_layer(b: u8) -> bool {
    matches!(
        b,
        0x29 | 0x32 | 0x38 | 0x39 | 0x3A | 0x41 | 0x4B | 0x4D | 0x4F | 0x53 | 0x54 | 0x55 | 0x5C
    )
}

pub(crate) fn parse_expr(seg: &[u8], p: &mut usize, stop: u8) -> Result<Vec<IlOp>, Block> {
    parse_expr_classed(seg, p, stop).map(|(ops, _)| ops)
}

/// [`parse_expr`] with the sub-expression's [`ValueClass`] preserved.
///
/// The class is `None` for every position that has always been accepted (a
/// 4-byte integer or pointer value, which the `41` result annotation and the
/// `55` call-end type already gate through `eat_int_like_or_ptr4`), and
/// `Some(ValueClass::Int1u)` for the one class this entry point adds — a `bool`
/// or `unsigned char` value. The caller **must** act on that: the result
/// annotation has to restate the class, because an `int`-annotated `bool` value
/// is the `rlwinm` mask (`GAPS.md` §6's "two facts sharing one field" again, with
/// the two facts being "this value is one register wide" and "this value is one
/// *byte* wide"). Every caller that does not know how to do that keeps calling
/// [`parse_expr`], which discards the class — and refuses the body one token
/// later at the annotation, honestly.
/// Record the **signedness** of a width-4 integer TYPE at `p`, if that is what
/// is there. Reads nothing else and consumes nothing (`lane w-build`).
///
/// `kind`'s low nibble is the class — `1` signed, `2` unsigned — and
/// [`is_int4_type`] already admits exactly that pair at width 4. This does not
/// re-derive the width or the tag: it asks the same predicate the parser gates
/// on and then reads one nibble, so a type this parser would refuse can never
/// set either flag.
///
/// A TYPE that is not a width-4 integer sets **neither** flag, which is the
/// conservative direction: a right shift over a value whose signedness was never
/// established refuses under `expr-shr-sign-unknown` rather than defaulting into
/// `sraw`. Defaulting is how `expr_opcode_name` once got three of six
/// relationals wrong, and a wrong *instruction* is worse than a wrong name.
/// The census ctx of a DIVIDE/MODULO refusal that carries the **operand TYPE**
/// it was reached with (`lane w-divsplit`, board **#816**).
///
/// Its own ctx rather than a flag on `"expr"`, because [`Block::feature`] renders
/// a nonzero [`Block::aux`] as an operand-type key for *every* ctx, and the
/// generic `"expr"` fall-through produces `expr-brfalse`, `expr-op-0x30`,
/// `expr-op-0x41` and a dozen more that must keep their published spellings.
/// One ctx, one rekey, one row of the board to pay for it.
pub(crate) const EXPR_TYPED_OP: &str = "expr-typed-op";

/// The two operand-stream opcodes this ctx covers: `05` divide and `06` modulo
/// (`docs/BOARD.md` #782 — `div_mod_leaf` is the shape whose census bucket these
/// two used to feed).
pub(crate) const DIV_MOD_OPS: [u8; 2] = [0x05, 0x06];

/// Record the `<tag> <kind>` of the TYPE at `p`, if that is what is there.
///
/// The **anti-#644 primitive**. Board #783 proposed splitting the division
/// population by reading "the operand TYPE triple immediately preceding the
/// `05`", and lane `w-divsplit` measured what that costs: a fixed-offset reader
/// at `mark - 3` is wrong or blind on **4,674 of 4,674** sites, because the
/// operand that ends at the opcode is a LITERAL — `33 <TYPE> <payload>` — whose
/// type ends two to six bytes earlier, not three.
///
/// So the type is recorded **where the parser reads it**, at the cursor that
/// already proved a TYPE starts there, and never re-derived from a stride. Reads
/// nothing else and consumes nothing, exactly like [`note_int4_signedness`]
/// beside it.
fn note_operand_type(seg: &[u8], p: usize, last: &mut Option<(u8, u8)>) {
    if let Some((tag, kind, _, _)) = read_type(seg, p) {
        *last = Some((tag, kind));
    }
}

fn note_int4_signedness(seg: &[u8], p: usize, signed: &mut bool, unsigned: &mut bool) {
    if let Some((tag, kind, _, _)) = read_type(seg, p) {
        if is_int4_type(tag, kind) {
            match kind & 0x0F {
                0x1 => *signed = true,
                0x2 => *unsigned = true,
                _ => {}
            }
        }
    }
}

pub(crate) fn parse_expr_classed(
    seg: &[u8],
    p: &mut usize,
    stop: u8,
) -> Result<(Vec<IlOp>, Option<ValueClass>), Block> {
    // Big enough for every fixture body; a longer stream grows normally.
    let mut ops = Vec::with_capacity(16);
    // Set by a LOAD or LIT whose TYPE was a 4-byte pointer rather than an
    // int-like one. Checked once, below, against the arithmetic in `ops`.
    let mut saw_ptr = false;
    // Set by a LOAD or LIT whose TYPE was the one-byte unsigned class, and by one
    // that was not. Checked once, below: the two may not mix, and the class may
    // not enter arithmetic — every capture of `bool` arithmetic converts first, so
    // a chain that did not is a shape with no witness behind it.
    let mut saw_int1u = false;
    let mut saw_wide = false;
    // The **SIGNEDNESS** of the width-4 integer operands, read separately from
    // [`ValueClass`] and only for the right shift (`lane w-build`).
    //
    // `ValueClass::Int4` collapses `int` and `unsigned` on purpose — every other
    // modeled operator emits the identical instruction over both, and
    // `is_int4_type` admits the pair so that a `2C` between them is the no-op it
    // is. `>>` is the one operator where the collapse is wrong: `86 41` emits
    // `sraw` and `86 42` emits `srw`, from the same IL byte `0A`. Rather than
    // shard `ValueClass` — five call sites, three byte-graded shapes — the two
    // flags are recorded here beside `saw_int1u`/`saw_wide`, which is the shape
    // this function already uses for a fact that only one operator reads.
    let mut saw_int4_signed = false;
    let mut saw_int4_unsigned = false;
    // The `<tag> <kind>` of the **most recently read operand TYPE**, for
    // [`EXPR_TYPED_OP`]. Updated at each of the three sites that read an operand
    // type — LOAD, LITERAL, and an admitted `2C` target — and read only by the
    // divide/modulo refusal below.
    //
    // "Most recently read" is the honest name for it and is not the same claim
    // as "the divisor's type": they coincide whenever the divisor is a leaf, and
    // `work/w-divsplit/split.py` measures that they are a leaf at **4,674 of
    // 4,674** sites on the dc3 workload. Where they would not coincide the
    // recorded type is still an operand type of the same expression, which is
    // what the int/float question is about.
    let mut last_type: Option<(u8, u8)> = None;
    // Set by `02`/`03`/`04` — arithmetic whose pointer form c2 SCALES by the
    // pointee width, which is what the pointer guard below exists to refuse.
    // The `27` byte-offset add is not that and does not set it.
    let mut scaled_arith = false;
    // Set by the [`rel_sink_enabled`] arm and by nothing else. A walk that
    // reaches the end with this set refuses under `expr-rel-sink-poison`, so the
    // sink can never move an obj byte.
    let mut saw_rel_sink = false;
    // Set by the [`branch_sink`] arms and by nothing else. A walk that reaches
    // the end with this set refuses under `expr-branch-sink-poison`, so the sink
    // can never move an obj byte.
    let mut saw_branch_sink = false;
    // Set by the [`chain_sink`] arms and by nothing else. A walk that reaches the
    // end with this set refuses under `expr-chain-sink-poison`, so the sink can
    // never move an obj byte. See [`chain_sink`] for the rest of the discipline.
    let mut saw_chain_sink = false;
    // **The operand stack's CLASSES** (`lane w-convert`, board **#701**), which
    // used to be a single `class: Option<ValueClass>` documented as "the class of
    // the last operand" with a note that the two "could differ
    // (`(void *)(s + 1)`, whose last operand is the literal)" and that the guard
    // refused the body anyway so only the census key changed.
    //
    // It is a real stack now for one reason: the guard is no longer a
    // whole-expression flag. `IlOp` is postfix and each binary op pops two and
    // pushes one (`crates/c2-core/.../straightline.rs`), so simulating the
    // CLASSES over the same discipline answers exactly the question
    // `expr-ptr-arith` is about — *was an arithmetic operator applied to a value
    // that was a pointer AT THAT MOMENT* — instead of the coarser *did a pointer
    // appear anywhere in this expression*. `(int)p + b` is `add r3,r3,r4`, an
    // integer addition over a value that used to be a pointer, and the coarse
    // flag refused it.
    //
    // `cstack_ok` is the model's own honesty: any token whose stack effect this
    // does not model clears it and the end-of-walk guard falls back to the
    // whole-expression flag. A model that guessed would be worse than the flag it
    // replaced.
    let mut cstack: Vec<ValueClass> = Vec::with_capacity(8);
    let mut cstack_ok = true;
    // **The member-call value model's poison** (`lane w-value`, board #1940):
    // the offset of the FIRST `26` whose whole production the walk consumed, or
    // `None` if it consumed none. Set by the `0x26` arm and read by exactly one
    // guard, the first of the end-of-walk guards below.
    //
    // Anchored at the first `26` and not at the last, and not at the byte the
    // walk ended on, for `super::mcall`'s own reason: the histogram files a
    // function by the *construct*, not by where the parse stopped. The block it
    // raises is the one this arm used to return on sight of that byte, so a body
    // whose only unmodeled construct is the call keeps its published key.
    let mut call_at: Option<usize> = None;
    // Set by the fold below when an operator's own operands included a pointer —
    // the exact form of `saw_ptr && <arith>` and `saw_ptr && <bitwise>`.
    let mut ptr_arith_exact = false;
    let mut ptr_bitwise_exact = false;
    // Fold one binary operator over the class stack. Underflow means the stream
    // is not the postfix shape this models, which is a fact about the stream and
    // not about the operator, so it clears the model rather than refusing here.
    macro_rules! fold_binary {
        ($flag:ident) => {{
            match (cstack.pop(), cstack.pop()) {
                (Some(r), Some(l)) => {
                    let ptr = r == ValueClass::Ptr4 || l == ValueClass::Ptr4;
                    if ptr {
                        $flag = true;
                    }
                    // Pointer arithmetic yields a pointer; integer arithmetic an
                    // integer. The `Ptr4` case is already refused by the time
                    // this matters, and it is pushed faithfully anyway so that a
                    // later reader of the top is never handed a fiction.
                    cstack.push(if ptr { ValueClass::Ptr4 } else { ValueClass::Int4 });
                }
                _ => cstack_ok = false,
            }
        }};
    }
    loop {
        let b = *seg.get(*p).ok_or(blk(seg, *p, "expr"))?;
        // **The chain sink is consulted BEFORE the stop byte** (`w-depth`, board
        // **#663**), and that ordering is the whole difference between measuring
        // one expression and measuring a body.
        //
        // The falsification probe `work/w-depth/probe/p5_mixed.cpp` is what
        // found it. Its declared operator inventory is the union of four
        // single-operator probes, `{1F, 0B, 0A, 38}`; with the stop checked
        // first the chain reported `{1F, 38, 0B}` and stopped — **one operator
        // short, and short in a direction that FLATTERS the instrument**,
        // because the `a >> 1` lives in the *second* return statement, past the
        // `41` result annotation the walk halts on. A depth that silently omits
        // everything after the first `return` is a lower bound advertised as a
        // count.
        //
        // Naming the stop byte in the sink set (`op:41`) now walks through it,
        // and the chain runs to the function tail `4F 12`, which
        // [`chain_skip_form`]'s `Line4F` refuses by construction. So the whole
        // body is in scope and the terminal is a byte the sink can never eat.
        if chain_sink().ops[b as usize] {
            if let Some(step) = chain_sink_step(seg, *p) {
                match step {
                    Ok(q) => {
                        *p = q;
                        saw_chain_sink = true;
                        // The sink skips a token by WIDTH, not by meaning, so
                        // the class stack cannot have followed it. The poison
                        // below refuses the body anyway; clearing the model
                        // keeps `cstack_ok` a literal claim — *every token was
                        // followed* — rather than one that holds only because
                        // something else refuses first.
                        cstack_ok = false;
                        continue;
                    }
                    Err(key) => return Err(blk(seg, *p, key)),
                }
            }
        }
        if b == stop {
            break;
        }
        // An INTRINSIC CALL. Recognized as the two-token unit
        // `33 86 41 74 <id>` + `40` so the census can report *which* intrinsic
        // (`expr-intrinsic-memcpy`, `expr-intrinsic-base-upcast`, …) instead of
        // one 9 %-of-the-workload `expr-intrinsic-call` bucket. **Decoding is not
        // accepting**: this returns `Err` exactly as the old fall-through did, so
        // the gate is byte-for-byte unchanged — only the census key moves. See
        // `docs/IL_INTRINSIC_CALL.md` for why none of the family can be lowered
        // yet (the emission depends on the *literal argument values*, not on the
        // id: id 2114 with offset `00` is nothing at all, with offset `04` it is
        // a null-guarded `addi` plus a control-flow split).
        if let Some(id) = intrinsic_selector(seg, *p) {
            // **w-depth chain sink — `C2RS_SINK_CHAIN=intrinsic` and nothing
            // else.** The two-token unit is consumed whole (`33 <int> <id>` is
            // already located by `intrinsic_selector`, so only the `40 <TYPE>`
            // is left to skip). No [`IlOp`]; poisons.
            if chain_sink().intrinsic {
                let mut q = *p + 4;
                if read_varint(seg, &mut q).is_none() || seg.get(q) != Some(&0x40) {
                    return Err(Block::refuse(seg, *p, "expr-chain-short"));
                }
                q += 1;
                match read_type(seg, q) {
                    Some((_, _, _, w)) => *p = q + w,
                    None => return Err(Block::refuse(seg, *p, "expr-chain-short")),
                }
                saw_chain_sink = true;
                continue;
            }
            return Err(Block {
                ctx: "expr-intrinsic",
                byte: Some(0x40),
                off: *p,
                seg_len: seg.len(),
                aux: id as u64,
            });
        }
        // **w-depth chain sink — `C2RS_SINK_CHAIN` and nothing else.** Placed
        // *before* the match so that a sink token can also close an opcode the
        // arms below already reach (`26`, whose arm always refuses, and `2C`,
        // whose arm refuses on an unmodeled target). It is inert unless the
        // variable names the byte. No [`IlOp`]; poisons. See [`chain_sink`].
        if let Some(step) = chain_sink_step(seg, *p) {
            match step {
                Ok(q) => {
                    *p = q;
                    saw_chain_sink = true;
                    continue;
                }
                // Reported at the byte, so the key carries it:
                // `expr-chain-noform-0x64` names the opcode whose width this
                // tree has not pinned.
                Err(key) => return Err(blk(seg, *p, key)),
            }
        }
        match b {
            0xB9 => {
                // LOAD <token> <int-type>
                let start = *p;
                *p += 1;
                let (tok, w) =
                    read_token_var(seg, *p).ok_or(blk(seg, *p, "expr-load-tok"))?;
                *p += w;
                // Read BEFORE the type is consumed; see `note_int4_signedness`.
                note_int4_signedness(seg, *p, &mut saw_int4_signed, &mut saw_int4_unsigned);
                note_operand_type(seg, *p, &mut last_type);
                match eat_operand_type(seg, p) {
                    Some(c) => {
                        saw_ptr |= c == ValueClass::Ptr4;
                        saw_int1u |= c == ValueClass::Int1u;
                        saw_wide |= c != ValueClass::Int1u;
                        cstack.push(c);
                    }
                    // neither int-like nor a 4-byte pointer → out of class.
                    // Report at the LOAD so the census bucket reads as a
                    // typed-operand gap, not a stray byte.
                    None if chain_sink().ty => {
                        // **w-depth chain sink — `C2RS_SINK_CHAIN=type`.** The
                        // TYPE is skipped whole rather than classified, and the
                        // stack class is recorded as `Int4` — an arbitrary
                        // choice with no consequence, because the walk is
                        // poisoned and the only reader of `class` is the `2C`
                        // arm, which under this configuration is measuring a
                        // successor and not admitting anything.
                        chain_skip_type(seg, p)?;
                        cstack.push(ValueClass::Int4);
                        cstack_ok = false;
                        saw_chain_sink = true;
                    }
                    None => return Err(blk_type(seg, *p, start, "expr-load-type")),
                }
                ops.push(IlOp::Load(tok));
            }
            0x33 => {
                // LITERAL: 33 <int-type> <varint>
                let start = *p;
                *p += 1;
                note_int4_signedness(seg, *p, &mut saw_int4_signed, &mut saw_int4_unsigned);
                note_operand_type(seg, *p, &mut last_type);
                match eat_operand_type(seg, p) {
                    Some(c) => {
                        saw_ptr |= c == ValueClass::Ptr4;
                        saw_int1u |= c == ValueClass::Int1u;
                        saw_wide |= c != ValueClass::Int1u;
                        cstack.push(c);
                    }
                    // **w-depth chain sink — `C2RS_SINK_CHAIN=type`.** Same rule
                    // one operand over; see the LOAD arm.
                    None if chain_sink().ty => {
                        chain_skip_type(seg, p)?;
                        cstack.push(ValueClass::Int4);
                        cstack_ok = false;
                        saw_chain_sink = true;
                    }
                    None => return Err(blk_type(seg, *p, start, "expr-lit-type")),
                }
                ops.push(IlOp::Lit(
                    read_varint(seg, p).ok_or(blk(seg, *p, "expr-lit-varint"))?,
                ));
            }
            0x02 => {
                *p += 1;
                scaled_arith = true;
                fold_binary!(ptr_arith_exact);
                ops.push(IlOp::Add);
            }
            0x03 => {
                *p += 1;
                scaled_arith = true;
                fold_binary!(ptr_arith_exact);
                ops.push(IlOp::Sub);
            }
            0x04 => {
                *p += 1;
                scaled_arith = true;
                fold_binary!(ptr_arith_exact);
                ops.push(IlOp::Mul);
            }
            // **The BITWISE and SHIFT binary operators** — `lane w-build`.
            //
            // Bare one-byte tokens, exactly as `mcall`'s `BARE_BINARY_OPS`
            // records them from a capture: no TYPE, no varint, no trailing
            // field. They pop two and push one, like `02`/`03`/`04`, and unlike
            // those they do **not** set `scaled_arith` — a pointer never enters
            // one of these at all (the guard below refuses it outright), so
            // there is no scaling question to answer.
            //
            // What reaches codegen from here is register-register only.
            // `select_text` refuses every non-register operand form under
            // `out_of_class`, and [`IlOp::And`] carries the five probed cells
            // that say why: the immediate axis of `&` alone has three
            // instruction families, one of them record-form (`andi.`), one of
            // them a two-instruction materialization into **r12**.
            0x0B => {
                *p += 1;
                fold_binary!(ptr_bitwise_exact);
                ops.push(IlOp::And);
            }
            0x0C => {
                *p += 1;
                fold_binary!(ptr_bitwise_exact);
                ops.push(IlOp::Or);
            }
            0x0D => {
                *p += 1;
                fold_binary!(ptr_bitwise_exact);
                ops.push(IlOp::Xor);
            }
            // `<<` is ONE instruction over both signednesses — `slw`, probed
            // three ways (`int<<int`, `unsigned<<unsigned`, `int<<unsigned`) —
            // so it needs no signedness decision and takes none.
            0x09 => {
                *p += 1;
                fold_binary!(ptr_bitwise_exact);
                ops.push(IlOp::Shl);
            }
            // `>>` is TWO instructions from ONE IL byte, and the byte does not
            // say which: `sraw` over a signed left operand, `srw` over an
            // unsigned one. The decision is made here, from the operand TYPEs
            // seen so far — which in a serial chain is exactly the left operand
            // and its own history — and it is made only when the history is
            // UNAMBIGUOUS.
            //
            // Both refusals are their own census key, so what the conservatism
            // costs is a number and not an argument:
            //
            // * `expr-shr-mixed-sign` — both signednesses are live in this
            //   expression. c2 decides on the LEFT operand alone (probed:
            //   `int >> unsigned` is `sraw`, `unsigned >> int` is `srw`), but
            //   this parser tracks a per-expression fact rather than a
            //   per-operand one, so it declines rather than guessing which
            //   operand the flag came from;
            // * `expr-shr-sign-unknown` — no width-4 integer TYPE was seen at
            //   all (a pointer or `bool` left operand, or a shift this parser
            //   reached without an operand). There is no witness for either
            //   lowering.
            //
            // The `2C` arm feeds the same two flags, and that is the arm that
            // makes this safe rather than merely careful: `is_int4_type` admits
            // `int` and `unsigned` alike, so `(unsigned)a >> b` is a conversion
            // this parser ALREADY accepts as a no-op while it changes the
            // instruction from `sraw` to `srw`. Recording the target's
            // signedness turns that body into a `expr-shr-mixed-sign` refusal
            // instead of a wrong emit.
            0x0A => {
                let op = match (saw_int4_signed, saw_int4_unsigned) {
                    (true, false) => IlOp::ShrS,
                    (false, true) => IlOp::ShrU,
                    (true, true) => {
                        return Err(Block::refuse(seg, *p, "expr-shr-mixed-sign"))
                    }
                    (false, false) => {
                        return Err(Block::refuse(seg, *p, "expr-shr-sign-unknown"))
                    }
                };
                *p += 1;
                fold_binary!(ptr_bitwise_exact);
                ops.push(op);
            }
            // **W-ARMS scratch sink — `C2RS_SINK_OFF_ADD_ARG=expr` and nothing
            // else.** `27 <TYPE>` is the BYTE-offset add: `&t->s.k` is
            // `B9 t · 33 <int> 0 · 27 · 33 <int> 8 · 27`, one step per designator,
            // and c2 lowers the whole run as a single `addi rD,rBase,<sum>`.
            //
            // It is a different construct from `02` and that is the whole reason
            // it has its own opcode: `p + 1` on an `int*` is `02` and emits
            // `addi r3,r3,4` — SCALED by the pointee — which is why the
            // pointer-arithmetic guard below refuses `02` over a pointer. `27`
            // carries the scaling already, so `[Load, Lit(k), Add]` is the
            // correct lowering and the guard must not see it. `scaled_arith`
            // separates the two facts the old single `saw_ptr && any-arith` test
            // conflated.
            //
            // MEASURED, not shipped: widening `parse_expr` here also obliges
            // `mcall::eat_int_operands`'s `Vocab::CallArg` to widen in lockstep,
            // or §9.14.6's correspondence guard goes red — a measure narrower
            // than its emitter manufactures phantom rungs. See
            // `docs/rungs/_draft-roadmap-9.17.md`.
            // **w-cmp scratch sink — `C2RS_SINK_REL=expr` and nothing else.**
            // The six relational opcodes, consumed so the walk can proceed to the
            // next unmodeled byte. See [`rel_sink_enabled`] for why this pushes no
            // op and why a walk that reaches the end still refuses.
            0x1F..=0x24 if rel_sink_enabled() => {
                *p += 1;
                // A relational pops two and pushes one, and this arm models
                // none of that. See the chain sink above for why the model is
                // cleared even though the poison already refuses.
                cstack_ok = false;
                saw_rel_sink = true;
            }
            // **w-brfalse scratch sink — `C2RS_SINK_BRANCH` and nothing else.**
            // The conditional branches `38 <tok>` / `39 <tok>`, consumed so the
            // walk can proceed to the next unmodeled byte. See [`branch_sink`]
            // for why this pushes no op, why a walk that reaches the end still
            // refuses, and why the wider [`BranchSink::Cflow`] level exists.
            0x38 | 0x39 if branch_sink() != BranchSink::Off => {
                *p += 1;
                let (_, w) =
                    read_token_var(seg, *p).ok_or(blk(seg, *p, "expr-branch-sink-tok"))?;
                *p += w;
                cstack_ok = false;
                saw_branch_sink = true;
            }
            // …and at [`BranchSink::Cflow`] the rest of the intra-body
            // control-flow vocabulary: the label `29 <tok>`, the unconditional
            // jump `3A <tok>` and the statement end `4B`. Same poison, same
            // absence of any [`IlOp`].
            0x29 | 0x3A if matches!(branch_sink(), BranchSink::Cflow | BranchSink::Stmt) => {
                *p += 1;
                let (_, w) =
                    read_token_var(seg, *p).ok_or(blk(seg, *p, "expr-branch-sink-tok"))?;
                *p += w;
                cstack_ok = false;
                saw_branch_sink = true;
            }
            0x4B if matches!(branch_sink(), BranchSink::Cflow | BranchSink::Stmt) => {
                *p += 1;
                cstack_ok = false;
                saw_branch_sink = true;
            }
            // The LINE MARKER, `4F 01 <varint>`, skipped exactly as
            // `shapes::control_flow::Scan::line_markers` skips it. **This arm
            // exists to remove an ambiguity, not to widen anything**: `0x4F` is
            // both the line-marker prefix and the function-tail prefix
            // (`4F 12`), so without it a residual `expr-op-0x4F` cannot be told
            // apart from a body that merely crossed a source line. It carries no
            // semantics and pushes no [`IlOp`] — but it still poisons, because a
            // level that consumed a token without poisoning would be a widening.
            0x4F if branch_sink() == BranchSink::Stmt && seg.get(*p + 1) == Some(&0x01) => {
                let mut q = *p + 2;
                if read_varint(seg, &mut q).is_none() {
                    return Err(blk(seg, *p, "expr-branch-sink-line"));
                }
                *p = q;
                saw_branch_sink = true;
            }
            // …and at [`BranchSink::Stmt`] the scope brackets themselves.
            // `54` carries the depth remaining after the pop as one byte
            // (`shapes::control_flow::step`'s falsification check); this sink
            // does not check it, because a sink that refused on a depth
            // mismatch would be reporting an integrity failure as a census key.
            0x53 if branch_sink() == BranchSink::Stmt => {
                *p += 1;
                saw_branch_sink = true;
            }
            0x54 if branch_sink() == BranchSink::Stmt => {
                if seg.get(*p + 1).is_none() {
                    return Err(blk(seg, *p, "expr-branch-sink-scope"));
                }
                *p += 2;
                saw_branch_sink = true;
            }
            0x27 if off_add_sink_enabled() => {
                *p += 1;
                match read_type(seg, *p) {
                    Some((_, _, _, w)) => *p += w,
                    None => return Err(blk(seg, *p, "expr-off-add-type")),
                }
                // The byte-offset add's second operand is IMPLICIT — it is the
                // TYPE, not a stack value — so the postfix discipline the class
                // stack models does not hold across this token. Clearing the
                // model is what keeps `C2RS_SINK_OFF_ADD_ARG` a counterfactual
                // rather than a second, ungraded acceptance rule (#403).
                cstack_ok = false;
                ops.push(IlOp::Add);
            }
            // A `26` SYMBOL PUSH — the single largest blocking feature on the real
            // workload (286,240 functions, 12.9 %). It used to fall through to the
            // generic `expr` refusal and be reported as one bucket named
            // `expr-call-in-expr`, which described 0.2 % of its own contents. **The
            // refusal is unchanged**; only the census key is, and the walk names the
            // construct the `26` opened rather than the byte the parse stopped on.
            // See `super::mcall` and `docs/IL_CALL_IN_EXPR.md` §14.
            // A `2C` CONVERSION applied to the value on top of the operand stack:
            // `2C <TYPE target> <varint 0>`. Admitted **only when the target is the
            // class the value already has**, which is the one case measured to emit
            // no instruction at all — so this arm pushes no [`IlOp`], changes no
            // lowering, and needs no codegen (`docs/IL_CALL_IN_EXPR.md` §24).
            //
            // The rule is not new here. It is the same rule
            // [`super::shapes::finish_indirect_load`] and
            // [`super::shapes::try_parse_ptr_identity_leaf`] have carried and had
            // byte-graded since the getter rungs — an int4→int4 conversion
            // (cv-strip, `int`↔`unsigned`, `long`) and a ptr4→ptr4 one
            // (cv-strip, `T*`→`void*`) are each free — reached through the same
            // [`eat_value_type`] locator rather than a second copy. What is new is
            // the *position*: a general expression operand rather than a leaf's
            // single value.
            //
            // **CORRECTED — `lane w-convert`, board #700.** This comment used to
            // read *"Cross-class refuses, and that is a conservatism with a
            // measured price"*, and it named the missing work exactly: a
            // reinterpret between the two width-4 classes had "never been graded
            // across the widths, cv-spellings and argument positions this parser
            // reaches". It has been now — 31 cells at two profiles, plus the
            // whole 3x3 of [`ValueClass`] pairs — so the **width-4 reinterpret is
            // admitted** through [`eat_reinterpret_type`], and the arm below has
            // three accepting/refusing branches rather than one.
            //
            // The refusal that remains is **narrower and better named**: it is
            // now an unmodeled *target* (a narrowing, a widening, a float) or the
            // one-byte-unsigned class on either side — four of the nine class
            // pairs, each of which costs a real instruction. See the table on
            // [`eat_reinterpret_type`].
            //
            // The trailing varint is required to be literally `0`. It is `00` at
            // every aligned site any capture has shown, and a field that never
            // varied is indistinguishable from a constant (`GAPS.md` §6), so it is
            // required rather than skipped and its own key counts the exceptions.
            0x2C => {
                let start = *p;
                let mut probe = *p + 1;
                // A conversion with nothing to convert is not a conversion, so
                // this refuses rather than guessing a class.
                //
                // **CORRECTED — lane `w-5c2`, board #1462, closing #1469, on
                // `w-one`'s and `w-ladders`' measurement.** This comment used to
                // read *"This cannot be reached by a well-formed stream"*, and
                // that claim is **refuted at 4,973 first-blocker witnesses across
                // 829 of 878 workload TUs**, plus 371 `emit_blockers` entries on
                // 197 of them — 5,344 across the three maps the gap screen sums
                // (board **#1354**, `docs/rungs/2026-08-08-w-one.md` §3.1).
                // `src/Main.cpp`'s hatched frontier ladder rests on this key, and
                // `w-one` measured that lifting it takes the TU from
                // `net=2 EXIT(no-lift)` to `net=3`.
                //
                // **Why the claim was wrong, and it is not a corpus accident:**
                // `cstack` is a *partial* model by construction. Every arm whose
                // stack effect `parse_expr` does not follow advances the cursor
                // without pushing, and every sink skip sets `cstack_ok = false`
                // for exactly that reason — so a `2C` after any of them sees an
                // empty stack in a stream that is perfectly well formed. Either
                // the old comment was wrong or 94.4 % of this workload's IL is
                // malformed, and the differential says which.
                //
                // The **refusal itself is correct and stays**; only the claim
                // about reachability was false. That distinction is the point:
                // board **#1413** is the same week's instance of the pattern
                // going the other way — `SeqGuardEmit`'s doc asserted a shape
                // *"is refused in the IL parser"*, it was not, and `w-clear`
                // found the emit wrong on **30 of 54 cells**. **A comment
                // asserting unreachability or a refusal is not evidence of
                // either**; a witness count or a test is.
                let Some(cls) = cstack.last().copied() else {
                    return Err(blk(seg, start, "expr-convert-no-value"));
                };
                if eat_value_type(seg, &mut probe, cls) {
                    // Class-preserving: the class on the stack is unchanged.
                } else if let Some(got) = eat_reinterpret_type(seg, &mut probe, cls) {
                    // **THE WIDTH-4 REINTERPRET** (`lane w-convert`, board
                    // **#700**), and the rung the comment above used to name as
                    // open. `int f(S *p){ return (int)p; }` and
                    // `S *f(int a){ return (S *)a; }` are each a bare `blr`, and
                    // [`eat_reinterpret_type`] carries the 3x3 that says which
                    // of the nine class pairs are and are not — four of them
                    // cost an instruction and it admits none of those.
                    //
                    // **This arm is additive-ACCEPT, not additive-refusal**, and
                    // that is said rather than claimed away: it is reached only
                    // where `eat_value_type` refused, so no body master accepted
                    // parses differently — but a future wrong answer here would
                    // be a wrong emit, not a new refusal. What holds it is the
                    // corpus (`scripts/sweep.d/77-reinterpret-2c.py`,
                    // `fixtures/cpp/wcv_reinterpret.cpp`), not the shape
                    // of the guard.
                    if let Some(top) = cstack.last_mut() {
                        *top = got;
                    }
                    // **A converted pointer indicts the value exactly as a
                    // loaded one does.** `(S *)a + 1` is `addi r3,r3,8` and
                    // `(S *)a + k` is `slwi r11,r4,3 ; add` — c2 SCALES pointer
                    // arithmetic — so a chain that added 1 unscaled would be a
                    // wrong emit rather than a gap. The end-of-walk
                    // `expr-ptr-arith` guard is what refuses it, and it only
                    // fires on `saw_ptr`. This line is the whole of that
                    // guarantee; the sweep fragment is the corpus that can
                    // express its absence.
                    if got == ValueClass::Ptr4 {
                        saw_ptr = true;
                    }
                } else if chain_sink().convert {
                    // **w-depth chain sink — `C2RS_SINK_CHAIN=convert`.** The
                    // target TYPE is skipped whole; the class is left where it
                    // was, because a conversion the sink did not model tells us
                    // nothing about the class it produced. Poisons.
                    chain_skip_type(seg, &mut probe)?;
                    saw_chain_sink = true;
                } else {
                    // An unmodeled target type — `char`, `short`, `long long`, a
                    // float, or the one-byte-unsigned class on either side of the
                    // conversion. Reported at the target TYPE so the key names it
                    // (`<tag><kind>`, never the per-TU id).
                    return Err(blk_type(seg, *p + 1, start, "expr-convert-target"));
                }
                if !eat_byte(seg, &mut probe, 0x00) {
                    return Err(blk(seg, probe, "expr-convert-tail"));
                }
                // **The conversion's TARGET signedness counts too** (`lane
                // w-build`), and this line is the one that makes the right
                // shift safe rather than merely careful. `is_int4_type` admits
                // `86 41` and `86 42` alike, so an `int`->`unsigned` `2C` is a
                // conversion this arm accepts as the no-op it is — while it
                // changes `>>` from `sraw` to `srw`. Feeding the target into
                // the same two flags turns `(unsigned)a >> b` into a
                // `expr-shr-mixed-sign` refusal. Read at `*p + 1`, the target
                // TYPE, which one of the two accepting arms above has already
                // proved is a TYPE. Since board #700 that TYPE may be a
                // *pointer* (the reinterpret), and this reader sets neither flag
                // for one — the conservative direction, and the same one it
                // takes for every other non-int4 type it is handed.
                note_int4_signedness(seg, *p + 1, &mut saw_int4_signed, &mut saw_int4_unsigned);
                note_operand_type(seg, *p + 1, &mut last_type);
                *p = probe;
            }
            // **THE MEMBER-CALL VALUE MODEL** (`lane w-value`, board **#1940**).
            //
            // This arm used to be exactly one line — `return
            // Err(mcall::classify(seg, *p))` — so the expression walk stopped at
            // the *first* `26` in a body and every construct behind it was
            // invisible to the census. Board **#1534** measures that family at
            // **449,274 bodies / 36,751 emitted** and records that it "has still
            // never had a whole-production counterfactual"; its prescription is a
            // bracket walk that consumes `26 … 4C` entire, which is
            // [`super::mcall::eat_call_value`].
            //
            // **The refusal is unchanged and cannot become an acceptance.** The
            // production pushes no [`IlOp`] — `IlOp` has no call variant and this
            // lane did not add one — and `call_at` below poisons the end of the
            // walk with the **same** [`super::mcall::classify`] block this line
            // used to return, anchored at the same byte. So the arm replaces one
            // `Err` with another `Err` and nothing else; see `eat_call_value`'s
            // acceptance theorem.
            //
            // What moves is *which* refusal a body reports. A body whose only
            // unmodeled construct is the call reports the identical key it
            // reports today; a body with something deeper behind the call now
            // reports **that**, which is the whole measurement.
            0x26 => {
                let at = *p;
                match super::mcall::eat_call_value(seg, p) {
                Some(v) => {
                    match v {
                        // The call's return TYPE is a class this parser models,
                        // so the value it leaves is modeled too and a following
                        // `2C` has something to convert.
                        super::mcall::CallValue::Value(c) => {
                            saw_ptr |= c == ValueClass::Ptr4;
                            saw_int1u |= c == ValueClass::Int1u;
                            saw_wide |= c != ValueClass::Int1u;
                            cstack.push(c);
                        }
                        // `void` leaves nothing, and the stack really is
                        // unchanged — so `cstack_ok` STAYS TRUE here. That is
                        // the one place this model can claim to have followed a
                        // token exactly, and it is why `CallValue` separates
                        // `Void` from `Opaque` instead of folding both into "no
                        // push".
                        super::mcall::CallValue::Void => {}
                        // A float, a narrow scalar, an aggregate, a `long long`.
                        // A value **is** left — the call returns one — and only
                        // its class is unknown, so the DEPTH is modeled and the
                        // class is not: a placeholder is pushed and `cstack_ok`
                        // is cleared, exactly as the `C2RS_SINK_CHAIN=type` arms
                        // two hundred lines up do and for the same stated
                        // reason ("an arbitrary choice with no consequence,
                        // because the walk is poisoned").
                        //
                        // **Pushing nothing here was measured and was wrong.**
                        // The first 878-TU scan of this model did that, and
                        // `expr-convert-no-value-0x2C` — the key board #1462 is
                        // written about — went **4,973 → 5,790 bodies**: a `2C`
                        // converting an opaque call's result found an empty
                        // stack and reported "a conversion with nothing to
                        // convert" about a stream that has something to convert.
                        // A model that under-reports its own stack depth
                        // manufactures witnesses for a key that means something
                        // else.
                        super::mcall::CallValue::Opaque => {
                            cstack.push(ValueClass::Int4);
                            cstack_ok = false;
                        }
                    }
                    if call_at.is_none() {
                        call_at = Some(at);
                    }
                }
                // Not a call production this walker can tokenize — a bare
                // data-symbol address push, or a token whose width it has not
                // pinned. Byte-for-byte the refusal this arm has always raised.
                None => return Err(super::mcall::classify(seg, at)),
                }
            }
            // **The DIVIDE / MODULO refusal, carrying its operand type**
            // (`lane w-divsplit`, board **#816**). Identical to the
            // fall-through below in every way that matters — same offset, same
            // blocking byte, `Complete::NoSignal` either way, and it refuses
            // exactly as before — except that the census key names the operand
            // TYPE the walk reached the opcode with. Board **#783** asked
            // whether `expr-op-0x05` is integer or floating-point division and
            // could not be answered from the key; this is the resolution.
            b if DIV_MOD_OPS.contains(&b) => {
                return Err(Block {
                    ctx: EXPR_TYPED_OP,
                    byte: Some(b),
                    off: *p,
                    seg_len: seg.len(),
                    // Bit 0 is a PRESENCE flag, not padding: `aux == 0` has to
                    // stay readable as "no operand type was recorded", and a
                    // `(tag, kind)` of `(0, 0)` is not a thing `read_type` can
                    // return, but relying on that would be relying on an
                    // absence.
                    aux: match last_type {
                        Some((tag, kind)) => 1 | ((tag as u64) << 16) | ((kind as u64) << 8),
                        None => 0,
                    },
                });
            }
            // **THE STATEMENT-LAYER FENCE** (`lane w-value`, board **#1942**).
            //
            // A walk that consumed a member-call production and then ran into a
            // byte of the *statement* layer has not found a second construct —
            // it has walked off the end of the expression. `parse_expr` is an
            // expression parser called with one `stop` byte; the statement list
            // is `mod.rs`'s and `assign.rs`'s job, and neither `4B` nor `5C` nor
            // a scope bracket is a thing an expression contains.
            //
            // **MEASURED before it was written, which is why it exists.** The
            // first build of the value model let those bytes take the head, and
            // the 878-TU scan moved **9,034 emitted functions onto a brand-new
            // `expr-op-0x4B`** and **2,844 onto `expr-op-0x5C`** — 86 % of the
            // whole redistribution, from keys that *name the construct*
            // (`expr-call-in-expr-recv-object-then-call-recv-object-more`) onto
            // two that name **punctuation**. That is board #441's finding
            // (`expr-op-0x53` is the scope-open bracket, punctuation not a
            // construct) and #1535's, arriving a third time; a re-key that makes
            // the histogram less informative is not a measurement.
            //
            // **The control-flow bytes are in the fence for a second reason,
            // and it is the sharper one.** `mcall` already names what follows a
            // call in its own suffix — `…-then-branch-brfalse`,
            // `…-then-plumbing-0x3A` — so letting a `38` take the head would
            // trade a key that states TWO facts for one that states one. The
            // module's `a_branch_target_is_a_label_defined_later_in_the_segment`
            // test is what said so, by going red.
            //
            // So these thirteen bytes yield to the call, and every genuine
            // expression construct behind it — a relational, a `30` deref, a
            // `27` off-add, a `9B`, an unmodeled operand TYPE, a `2C` target —
            // still takes the head. See [`is_statement_layer`] for where each
            // byte's membership comes from; not one of them is a new reading.
            _ if call_at.is_some() && is_statement_layer(b) => {
                return Err(super::mcall::classify(seg, call_at.unwrap()))
            }
            _ => return Err(blk(seg, *p, "expr")),
        }
    }
    // **THE MEMBER-CALL VALUE MODEL'S POISON, AND IT IS FIRST ON PURPOSE**
    // (`lane w-value`, board **#1941**).
    //
    // A body whose expression walked to the end THROUGH a `26`-rooted call
    // production is refused here, under the **same** [`super::mcall::classify`]
    // block the `0x26` arm used to return on sight of the byte. Nothing below
    // lowers a call — [`IlOp`] has no call variant — so this is a refusal and
    // not a gap in the guards.
    //
    // **The position is a decision, registered before the code was written**
    // (`work/w-value/PREREG.md` §1), and w-park's finding that FENCE ORDER is
    // where the last unnamed refusal hides is what made it worth registering.
    // Today the `26` refuses *before* any guard below can be reached, so first
    // is the only position that leaves every one of those keys measuring the
    // population it measures now.
    //
    // **It caught one immediately, and that is the value of pre-arming it.**
    // The very next guard is `ops.is_empty()` → `expr-empty-0xNN`, and the
    // value model pushes **no [`IlOp`]** — so a body that is one member-call
    // statement and nothing else arrives here with `ops` empty and, one line
    // lower, would have been re-filed from `expr-call-in-expr-*` to
    // `expr-empty-0x4B`. That is a key boards #660, #1319, #1455, #1465 and
    // #1538 have published counts against, moved by a construct that has
    // nothing to do with a sink.
    if let Some(at) = call_at {
        return Err(super::mcall::classify(seg, at));
    }
    // **THIS ARM SHADOWS ALL THREE SINK POISONS BELOW, AND THE POISON COUNTS
    // HAVE ALWAYS BEEN READ AS IF IT DID NOT** (lane `w-mass`, board **#1538**).
    //
    // The sinks push no [`IlOp`] — that is the property that makes them
    // measurement-only — so a walk that consumed an expression *entirely*
    // through sunk tokens arrives here with `ops` empty and reports
    // `expr-empty-0xNN` instead of the poison. Both are the same event, *the
    // sink set was the last thing in the way*, and only one of them is in the
    // key every published reading counts.
    //
    // MEASURED, not argued: `C2RS_SINK_CHAIN=intrinsic,op:66` over the 878-TU
    // workload puts **341 emitted functions (2,569 bodies)** under
    // `expr-empty-0x55` and **5,021 (40,210)** under the poison, and the 341 are
    // exactly the class-layout half's whole recovery — so a poison-only reading
    // of that arm reports **0** for a population whose real answer is 341.
    //
    // **Not reordered here.** Moving the check below the poisons is inert with
    // every sink off (all three flags are false, so this arm is reached on the
    // identical population) and would be the better instrument — but it
    // *redefines* a key that boards #660, #1319, #1455 and #1465 have published
    // counts against, and this tree's rule is that a denominator gets published
    // before it gets folded in. The sum is published in
    // `rungs/2026-08-08-w-mass.md` §3.3 and the reorder is filed there as its
    // own rung. **Read the poison and this key together, or neither.**
    if ops.is_empty() {
        return Err(blk(seg, *p, "expr-empty"));
    }
    // The sink's poison. A body whose expression walked to the end THROUGH a
    // relational opcode is refused here rather than accepted, because nothing
    // below lowers one. The count under this key is the sink's real answer:
    // bodies the relational was the LAST thing standing in the way of.
    if saw_rel_sink {
        return Err(Block::refuse(seg, *p, "expr-rel-sink-poison"));
    }
    // **The chain sink's poison** (`w-depth`, board #660), and it is the reason
    // the instrument can be pointed at an arbitrary opcode set without ever
    // becoming a widening. A body whose expression walked to the end THROUGH a
    // chain-sunk token refuses here rather than being accepted, because nothing
    // below lowers any of them. The count under this key is the instrument's
    // terminal: **the body's expression stream is now fully consumed, so the
    // sink set enabled at that moment IS the body's chain.**
    if saw_chain_sink {
        return Err(Block::refuse(seg, *p, "expr-chain-sink-poison"));
    }
    // The branch sink's poison, and the same rule one construct over. A body
    // whose expression walked to the end THROUGH a conditional branch (or, at
    // `Cflow`, through any intra-body control-flow token) is refused here rather
    // than accepted, because nothing below lowers one — there is no conditional
    // body class at all. The count under this key is the sink's real answer:
    // emitted functions the control-flow family was the LAST thing standing in
    // the way of.
    if saw_branch_sink {
        return Err(Block::refuse(seg, *p, "expr-branch-sink-poison"));
    }
    // The pointer-arithmetic guard. A pointer operand anywhere in this value plus
    // any modeled arithmetic anywhere in it refuses the whole function — see the
    // header. `expr-ptr-arith` is its own census key so the cost of the guard is
    // a number rather than an argument.
    //
    // **Precise since board #701 where the class stack held.** The flag
    // `saw_ptr` asks *did a pointer appear anywhere in this expression*; the
    // guard's actual subject is *was an arithmetic operator applied to a value
    // that was a pointer at that moment*. Those differ exactly where a `2C`
    // moved the value out of the pointer class first, and that is the whole of
    // the workload's `expr-convert-target` population: 5,711 functions, every
    // one of them `(int)p <op> …`, which c2 emits as a plain `add`/`subf`/
    // `mullw` with no scaling at all (measured — `work/w-convert/probe/m2.cpp`).
    //
    // The coarse flag stays as the fallback for any stream the stack model could
    // not follow, so the refusal can only ever get *narrower* on streams the
    // model understands and is bit-for-bit the old one everywhere else.
    let ptr_arith = if off_add_sink_enabled() {
        // Only the SCALED forms indict a pointer value; see the `0x27` arm.
        saw_ptr && scaled_arith
    } else if cstack_ok {
        ptr_arith_exact
    } else {
        saw_ptr
            && ops
                .iter()
                .any(|o| matches!(o, IlOp::Add | IlOp::Sub | IlOp::Mul))
    };
    if ptr_arith {
        return Err(Block::refuse(seg, *p, "expr-ptr-arith"));
    }
    // **The pointer guard's bitwise half** (`lane w-build`), and a SEPARATE
    // census key rather than a widening of `expr-ptr-arith`.
    //
    // Two reasons it is separate. The fact is different — `+` over a pointer is
    // *scaled* by the pointee width and is refused for that; `&` over a pointer
    // is refused because no capture establishes it at all, which is a different
    // kind of ignorance and should count separately. And merging it would move
    // functions into an existing bucket that four rungs have compared across
    // trees, which is the one failure a census instrument cannot survive
    // (`docs/GAPS.md` §6).
    // Precise on the same terms as `expr-ptr-arith` above (#701), and with the
    // same fallback: a pointer that was converted away before `&`/`|`/`^`/`<<`/
    // `>>` reached it is an integer operand, and a stream the class stack could
    // not follow is judged by the whole-expression flag exactly as before.
    let ptr_bitwise = if cstack_ok {
        ptr_bitwise_exact
    } else {
        saw_ptr && ops.iter().any(|o| o.is_bitwise_or_shift())
    };
    if ptr_bitwise {
        return Err(Block::refuse(seg, *p, "expr-ptr-bitwise"));
    }
    // **A right shift whose signedness was settled and then contradicted.**
    //
    // The `0A` arm decides from the flags as they stand *at the operator*,
    // which in a serial chain is the left operand's whole history and is the
    // right answer. This is the belt to that braces: an operand appearing
    // AFTER the shift that carries the other signedness means the expression as
    // a whole mixes the two, and rather than reason about whether the later
    // operand could have reached the shift's left-hand side, the body refuses.
    //
    // It is conservative in a direction with a witness: `(a >> b) | c` with `c`
    // unsigned is a body c2 lowers as `sraw` then `or`, which this parser would
    // have emitted correctly. That coverage is what the key counts.
    if ops.iter().any(|o| matches!(o, IlOp::ShrS | IlOp::ShrU))
        && saw_int4_signed
        && saw_int4_unsigned
    {
        return Err(Block::refuse(seg, *p, "expr-shr-sign-late"));
    }
    // The one-byte-unsigned guard, and it is the pointer guard's twin: the class
    // is free to be *moved* and not to be *computed on*. `b1 + b2` in C++ converts
    // both operands to `int` first, so an accepted chain over raw `bool` operands
    // has no witness at all; and a chain mixing the class with a width-4 one is a
    // conversion the IL would have spelled with a `2C`. Both refuse under their own
    // census keys, so what the guard costs is a number rather than an argument.
    if saw_int1u {
        // The bitwise/shift six are their own key here for the same reason the
        // pointer guard splits: `b1 + b2` promotes to `int` and has no witness,
        // and `b1 & b2` is a *different* absence of witness. `lane w-build`.
        if ops.iter().any(|o| o.is_bitwise_or_shift()) {
            return Err(Block::refuse(seg, *p, "expr-int1u-bitwise"));
        }
        if ops
            .iter()
            .any(|o| matches!(o, IlOp::Add | IlOp::Sub | IlOp::Mul))
        {
            return Err(Block::refuse(seg, *p, "expr-int1u-arith"));
        }
        if saw_wide {
            return Err(Block::refuse(seg, *p, "expr-int1u-mixed"));
        }
    }
    Ok((ops, saw_int1u.then_some(ValueClass::Int1u)))
}

/// Parse the formal-parameter list of a straight-line leaf: after the `46` ('F')
/// marker (before the `LO` marker), a run of `2D <token>` entries emitted in
/// *reverse* of declaration order. An empty list is legitimate (a zero-param
/// `int konst(){return 42;}` still emits `46` immediately before `LO`).
///
/// The marker is located by requiring the region it opens to **end exactly on the
/// `LO` marker** — `46 (2D <tok>)*` and then `lo` — not by taking the first `0x46`
/// byte in the segment. That distinction is load-bearing, and taking the first
/// byte was a live bug:
///
/// * a function on **source line 70** carries the line marker `4F 01 46`, whose
///   payload byte *is* `0x46`. `fixtures/cpp/il_expr_deref.cpp` caught it — one of
///   sixteen otherwise-identical bodies (`ld_ixneg`, at line 70) silently got an
///   **empty** formals list, while its neighbours two lines away parsed fine;
/// * the per-function `4F 33 …` header region before the body is a run of opaque
///   bytes that varies with the function and freely contains `0x46`.
///
/// An empty formals list is not fail-closed: `leaves_ascending` skips tokens that
/// are not formals, so a body whose formals vanished bypasses the reassociation
/// ordering gate entirely. Getting the anchor right is therefore a safety fix, not
/// only a coverage one.
///
/// The earliest candidate that lands exactly on `lo` is taken. No candidate
/// *before* the true marker can span past it unless it lands on `lo`, because the
/// true marker's own `0x46` is neither `0x2D` nor a token continuation there.
pub(crate) fn parse_formals(seg: &[u8], lo: usize) -> Result<Vec<u32>, Block> {
    let f = formals_marker(seg, lo)
        .ok_or(Block::refuse(seg, lo, "formals-marker"))?;
    let mut rev = Vec::new();
    let mut p = f + 1;
    while p < lo && seg.get(p) == Some(&0x2D) {
        p += 1;
        let (tok, w) = read_token_var(seg, p)
            .ok_or(Block::refuse(seg, p, "formals-tok"))?;
        p += w;
        rev.push(tok);
    }
    rev.reverse();
    Ok(rev)
}

/// The offset of the `46` formals marker — **the one anchor**, so that everything
/// reading the pre-body region agrees about where it is.
///
/// This used to be inlined in [`parse_formals`] while [`super::shapes`] located the
/// same marker with a plain "first `0x46` byte" search. That disagreement was a
/// live wrong-bytes emit, not a tidiness problem: a member function on source line
/// 70 carries the line marker `4F 01 46`, the `this` lookup anchored on *that*
/// `0x46`, found no `this` group ending there, and reported "no `this`" — which the
/// caller could not distinguish from a genuine non-member. Every explicit formal
/// then sat one register too low and `int C::gp(int* q) const { return *q; }`
/// emitted `lwz r3,0(r3)` for `lwz r3,0(r4)`. `fixtures/cpp/il_this_line70.cpp`
/// pins it.
pub(crate) fn formals_marker(seg: &[u8], lo: usize) -> Option<usize> {
    for f in 0..lo {
        if seg[f] != 0x46 {
            continue;
        }
        let mut p = f + 1;
        let mut ok = true;
        while p < lo && seg.get(p) == Some(&0x2D) {
            p += 1;
            match read_token_var(seg, p) {
                Some((_, w)) => p += w,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok && p == lo {
            return Some(f);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail, BodyShape, Complete};
    use crate::func::bundle::LO_MARKER;
    use crate::func::readers::{find_subslice, read_type};
    use crate::func::test_fixtures::*;

    // ---- the CHAIN SINK (`w-depth`, board #660) -----------------------------

    /// **The one property that makes the instrument free**: it is OFF unless
    /// `C2RS_SINK_CHAIN` names something, and every gate lane runs with the
    /// variable unset. A default that drifted ON would be a parser that skips
    /// opcodes on the gate — and `#403` is the precedent for exactly that class
    /// of accident, where turning a sink on took the tree to 16 targets.
    #[test]
    fn the_chain_sink_is_off_unless_the_environment_names_it() {
        assert!(std::env::var("C2RS_SINK_CHAIN").is_err(), "the test process must not set it");
        let c = chain_sink();
        assert!(!c.any, "chain sink must default OFF");
        assert!(!c.ty && !c.convert && !c.intrinsic && !c.bad);
        assert!(c.ops.iter().all(|on| !on), "no opcode may default to sunk");
        // …and with it off, `chain_sink_step` never claims a byte, at any byte.
        for b in 0u8..=255 {
            assert!(chain_sink_step(&[b, 0, 0, 0], 0).is_none(), "0x{b:02X} claimed while OFF");
        }
    }

    /// The **absences in [`chain_skip_form`] are decisions, not gaps**, and this
    /// pins them so a later widening has to delete an assertion rather than a
    /// blank line. `1B`/`1C` are `||`/`&&`, whose bytes `mcall` records no
    /// capture has ever shown; `3B`/`3C`/`3D` are the switch family, whose table
    /// payload is not a fixed width; `64`/`66` are named nowhere in this tree.
    ///
    /// **`0xBD` has left this list too** (`lane w-bd`, board **#1314**), and
    /// deleting it is the assertion this test's own header demands. It was
    /// never unwitnessed: `IL_CALL_GRAMMAR.md` §2.1 states the grammar, and
    /// `mcall::eat_call_and_args`, `mcall`'s call-form walk and
    /// `control_flow.rs`'s `cf-call-fn-type-id` arm all consume exactly
    /// `<TYPE> <1 raw byte> <varint>` today. What was missing was a `SkipForm`
    /// able to spell it. Re-witnessed by a capture at this master and confirmed
    /// over 3,544,589 workload sites — see `chain_skip_form`'s `0xBD` row.
    ///
    /// `w-depth` measured `0x00` and `0x05` as live chain terminals on the
    /// frontier — two more bytes the instrument refuses rather than guesses
    /// (rung §6). `0x35` was a third until a capture pinned its WIDTH; it is in
    /// the table and still has no name.
    ///
    /// **`0x05` and `0x06` have left this list** (`lane w-divsplit`, board
    /// **#819**), and the assertion below is the one that had to be deleted. The
    /// width is pinned by `lane w-divmod`'s four captured leaf bodies
    /// (`B9 <tok> <T> B9 <tok> <T> >05< 41 <T> 3A …`, graded 185/185 against
    /// real `c2.dll`) and re-confirmed on the workload at **4,674 of 4,674**
    /// sites, where the byte after the opcode opens a new token. The NAME was
    /// never in question — this is `div_mod_leaf`'s own `IL_DIV`/`IL_MOD`.
    ///
    /// **`0x59` and `0x08` have JOINED it**, and they arrived as *evidence*
    /// rather than as an omission (`lane w-4c`, board **#1387**). They are the
    /// only two bytes the argument-bearing `4C` walk ever lands on that no arm
    /// accepts, and `work/w-4c/unwit.py` shows both occur at token-start
    /// positions with a non-`4C` predecessor — `0x59` **6,031** times, `0x08`
    /// **3,819**, and neither ever after a `4C`. So they are opcodes this tree
    /// has not pinned, not payload; `08` is additionally one of the six bytes
    /// `control_flow.rs` refuses on purpose, and adding either here on the
    /// strength of a landing count would be the guess this table exists to
    /// prevent. `0x07`, `0x14` and `0x25` are witnessed at token starts by the
    /// same walk (2, 1 and 1 times) — a witness that they OCCUR, and no
    /// evidence at all about their widths.
    ///
    /// **`0x5D` and `0x5E` have JOINED it, and their reason is a THIRD kind**
    /// (`lane w-5c`, board **#1425**). They are not unwitnessed like `0x00`, and
    /// they are not unmeasured like `0x14`: `docs/EH_RECORDS.md` §7.1 gives both
    /// as `<varint n> <varint state>` from the same probe session that pinned
    /// `0x5C`, and `control_flow.rs`'s `operand()` reads them at that width
    /// today. **`SkipForm` has no variant that can spell `<varint> <varint>`** —
    /// which is exactly the problem `0xBD` had before `TypeByteVarint` existed,
    /// so opening them is an ENUM change and not a table row. Recorded here
    /// rather than added, because the difference between *"nobody measured it"*
    /// and *"the type cannot say it"* is the difference `chain_skip_form`'s
    /// `None` is otherwise unable to express (board #1314's finding, restated).
    #[test]
    fn the_unpinned_opcodes_refuse_rather_than_guess_a_width() {
        // `0x66` LEFT this list on 2026-08-08 (lane `w-mass`, board #1530) and
        // it is the third kind again, resolved the other way: the width was
        // never unmeasured — `mcall::eat_class_descriptor` has read it since the
        // D2 rung and `control_flow.rs` calls that same function — it was
        // unspellable, and `SkipForm::ClassDescr` delegates rather than restates
        // it. `0x64` stays: it is `mcall`'s by-value-return materialize, which
        // no reader in this tree consumes at a stated width.
        for b in [0x00, 0x07, 0x08, 0x14, 0x1B, 0x1C, 0x3B, 0x3C, 0x3D, 0x59, 0x64] {
            assert_eq!(chain_skip_form(b), None, "0x{b:02X} must have no pinned form");
        }
        // The EH COUNT trailers. Kept in their own loop with their own message,
        // so a lane that adds a `VarintVarint` variant deletes a line that names
        // the reason instead of one that reads like an oversight.
        for b in [0x5D, 0x5E] {
            assert_eq!(
                chain_skip_form(b),
                None,
                "0x{b:02X} is `<varint> <varint>`, which no SkipForm variant can spell"
            );
        }
        // …and the two that moved are `Bare`, not merely "not None": a width
        // guess in the other direction is the desync this table exists to
        // prevent.
        for b in DIV_MOD_OPS {
            assert_eq!(chain_skip_form(b), Some(SkipForm::Bare), "0x{b:02X} is payload-free");
        }
    }

    /// The pinned widths, checked against **transcribed capture bytes** rather
    /// than against the table that produced them.
    #[test]
    fn the_pinned_skip_forms_consume_exactly_their_capture() {
        // `99 <TYPE> <varint>` — `mcall`'s transcribed member-call capture,
        // `… 99 86 43 9C 20 00 BD …`. `IL_EXPR_LAYER.md` §7 pins the varint by
        // contrast with `9B`'s token.
        let bind = [0x99, 0x86, 0x43, 0x9C, 0x20, 0x00, 0xBD];
        assert_eq!(skip_one(&bind, 0x99), Some(Ok(6)), "99 ends on the BD");
        // `B9 <token> <TYPE>` — the LOAD, two-byte token form.
        let load = [0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74, 0x41];
        assert_eq!(skip_one(&load, 0xB9), Some(Ok(6)));
        // `27 <TYPE>` — the byte-offset add, from `IL_EXPR_LAYER.md` §2's table.
        let offadd = [0x27, 0x86, 0x43, 0xF4, 0x08, 0x30];
        assert_eq!(skip_one(&offadd, 0x27), Some(Ok(5)));
        // `35 <TYPE>` — the loop increment of `Primes.cpp`'s `for` at the
        // workload's own flags: `26 <i2> · 33 86 41 74 01 · 35 86 41 74 · 4B`.
        assert_eq!(skip_one(&[0x35, 0x86, 0x41, 0x74, 0x4B], 0x35), Some(Ok(4)));
        // `4F 01 <varint>` is the LINE MARKER and `4F 12` is the FUNCTION TAIL.
        // The second must refuse: eating it walks the instrument out of the body,
        // which is what makes the tail a terminal the sink can never consume.
        assert_eq!(skip_one(&[0x4F, 0x01, 0x46, 0x00], 0x4F), Some(Ok(3)));
        assert_eq!(skip_one(&[0x4F, 0x12, 0x47, 0x54], 0x4F), Some(Err("expr-chain-noform")));
        // A payload that runs off the end is `expr-chain-short`, not a silent
        // clamp — a clamp would report a fictitious successor.
        assert_eq!(skip_one(&[0x27], 0x27), Some(Err("expr-chain-short")));
    }

    /// **`BD <TYPE> <flags:1 raw byte> <varint>`, from the CAPTURE and not from
    /// the table that produced it** (`lane w-bd`, board **#1314**).
    ///
    /// These four rows are transcribed off `work/w-bd/probe/bd_cc.cpp`'s `.ex`,
    /// captured at this master by `c2rs capture` and graded
    /// `ReferenceReplay=ByteExact` against real `c2.dll` under wibo. The
    /// externals differ ONLY in calling convention and their return TYPE is
    /// byte-identical, so the flags byte is the only field that can move — which
    /// is what makes them a witness for the width rather than for one call.
    #[test]
    fn the_call_token_consumes_exactly_its_capture() {
        // int cd(int)            __cdecl     — and __stdcall, which is the same
        let cdecl = [0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0xB9];
        assert_eq!(skip_one(&cdecl, 0xBD), Some(Ok(10)), "cdecl ends on the B9");
        // int __fastcall fc(int) — the ONE byte that moves
        let fast = [0xBD, 0x86, 0x41, 0x74, 0x04, 0x80, 0x07, 0x10, 0x00, 0x00, 0xB9];
        assert_eq!(skip_one(&fast, 0xBD), Some(Ok(10)));
        // int va(const char*, ...)  varargs
        let vararg = [0xBD, 0x86, 0x41, 0x74, 0x40, 0x80, 0x06, 0x10, 0x00, 0x00, 0xB9];
        assert_eq!(skip_one(&vararg, 0xBD), Some(Ok(10)));
        // void v0()  — a 3-byte return TYPE and a zero-argument region, so the
        // token is followed straight by the `4C` that closes it.
        let void0 = [0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x09, 0x10, 0x00, 0x00, 0x4C];
        assert_eq!(skip_one(&void0, 0xBD), Some(Ok(10)));
        // void* p0() — a 4-byte return TYPE. The SAME reading has to absorb it,
        // which is what a fixed-width read cannot do: `IL_CALL_GRAMMAR.md` §1.3
        // measures 3/4/5-byte return types at 1,735,526 / 1,391,017 / 417,958
        // workload sites, so a fixed 3-byte skip mis-parses one call in two.
        let ptr = [0xBD, 0x86, 0x43, 0x83, 0x08, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x4C];
        assert_eq!(skip_one(&ptr, 0xBD), Some(Ok(11)));
        // …and a 5-byte return TYPE, the ~12 % tail. Transcribed from
        // `IL_CALL_GRAMMAR.md` §2.1's own widest observed token.
        let wide = [
            0xBD, 0x86, 0x43, 0x9B, 0xB9, 0x02, 0x00, 0x80, 0x9F, 0x9C, 0x00, 0x00, 0x4C,
        ];
        assert_eq!(skip_one(&wide, 0xBD), Some(Ok(12)));
        // A payload that runs off the end refuses rather than clamping.
        assert_eq!(skip_one(&[0xBD, 0x86, 0x41, 0x74], 0xBD), Some(Err("expr-chain-short")));
        assert_eq!(skip_one(&[0xBD, 0x86, 0x41, 0x74, 0x00], 0xBD), Some(Err("expr-chain-short")));
    }

    /// **The two rival readings, and why the corpus excludes them** — the
    /// control that makes the row above a measurement instead of an assertion.
    ///
    /// A control that cannot go red is not a control, so this asks the positive
    /// question: *would anything catch the width being wrong in the most likely
    /// way?* The most likely way is dropping the flags byte, and the answer is
    /// that the walk then lands on the `0x80` of the fn-type id's own escape —
    /// a byte no arm of `chain_skip_form` or of `control_flow.rs`'s `operand()`
    /// accepts. `work/w-bd/bdwalk.py` scores that over the workload at
    /// **3,544,480 of 3,544,589** sites; this pins the mechanism.
    #[test]
    fn dropping_the_call_flags_byte_lands_on_a_byte_no_arm_accepts() {
        let cdecl = [0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0xB9];
        // The pinned reading lands on `B9`, which IS an operand opcode…
        assert_eq!(skip_one(&cdecl, 0xBD), Some(Ok(10)));
        assert_eq!(cdecl[10], 0xB9);
        assert!(chain_skip_form(cdecl[10]).is_some(), "the landing byte opens a token");
        // …and the flags-byte-dropped reading, `TypeVarint`, lands on index 5.
        // Spelled out rather than computed so the byte is visible: TYPE is 3
        // bytes, the `00` is consumed as a whole short varint, and the cursor
        // stops on the escape marker.
        assert_eq!(cdecl[5], 0x80);
        assert_eq!(chain_skip_form(0x80), None, "0x80 opens nothing — the desync is visible");
    }

    /// **C3, the null step.** `0xBD` was `lane w-bd`'s only entry and `0x4C` is
    /// `lane w-4c`'s: every other byte's form is asserted against the table as
    /// it stood, so a widening that leaked into a neighbour has to delete a line
    /// here.
    #[test]
    fn pinning_the_call_moved_no_other_opcode() {
        for b in 0u8..=255 {
            if b == 0xBD {
                assert_eq!(chain_skip_form(b), Some(SkipForm::TypeByteVarint));
                continue;
            }
            let expect = match b {
                0x02..=0x06 | 0x09..=0x0D | 0x1A | 0x1F..=0x24 | 0x44 | 0x4B | 0x4C | 0x53 => {
                    Some(SkipForm::Bare)
                }
                0x0F | 0x27 | 0x30 | 0x32 | 0x35 | 0x40 | 0x41 | 0x55 => Some(SkipForm::Type),
                0x26 | 0x29 | 0x38..=0x3A => Some(SkipForm::Tok),
                0x28 => Some(SkipForm::Byte2),
                0x2C | 0x33 | 0x5C | 0x99 => Some(SkipForm::TypeVarint),
                0x43 => Some(SkipForm::Escape43),
                0x4F => Some(SkipForm::Line4F),
                0x54 => Some(SkipForm::Byte1),
                0x66 => Some(SkipForm::ClassDescr),
                0x67 => Some(SkipForm::VarintTok),
                0x9B => Some(SkipForm::TypeTok),
                0xB9 => Some(SkipForm::TokType),
                _ => None,
            };
            assert_eq!(chain_skip_form(b), expect, "0x{b:02X} moved");
        }
    }

    /// **The CALL-END, on the ARGUMENT-BEARING population** — the token streams
    /// are transcribed from `work/w-4c/probe/ce_args.cpp`'s fresh capture at
    /// this master, not from the table that reads them.
    ///
    /// Board **#1318** had `4C` payload-free at 26,701 of 26,701 sites and
    /// declined to ship it, because every one of those is a **zero-argument**
    /// call. The rows below are calls with one, two and three arguments — the
    /// 2.46 M of 3.5 M `BD` tokens that grid contained none of — and the
    /// argument region's LENGTH is the only thing that moves across them.
    #[test]
    fn the_call_end_closes_an_argument_region_and_carries_nothing() {
        const INT: [u8; 3] = [0x86, 0x41, 0x74];
        // The whole `26 <g> BD <TYPE> 00 <id>  (B9 <x> INT · 55 INT)*  4C  41`
        // shape, one argument. Stepping the `4C` must land on the `41` result
        // annotation with no byte in between.
        let one = [
            0xBD, INT[0], INT[1], INT[2], 0x00, 0x80, 0x03, 0x10, 0x00, 0x00, // the CALL token
            0xB9, 0xEF, 0x09, INT[0], INT[1], INT[2], // the argument
            0x55, INT[0], INT[1], INT[2], // its `55 <TYPE>` terminator
            0x4C, 0x41, // the CALL-END, then the result annotation
        ];
        assert_eq!(skip_one(&one, 0xBD), Some(Ok(10)), "the CALL token is 10 bytes");
        assert_eq!(one[20], 0x4C);
        assert_eq!(skip_one(&one[20..], 0x4C), Some(Ok(1)), "the CALL-END is ONE byte");
        assert_eq!(one[21], 0x41, "and the very next byte is the next opcode");
        assert!(chain_skip_form(one[21]).is_some(), "which this table can step");

        // TWO arguments, from the same capture: the token is byte-identical
        // except for the fn-type id, and the region is twice as long. A width
        // that depended on the argument count would have to differ here.
        let two = [
            0xBD, INT[0], INT[1], INT[2], 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, //
            0xB9, 0xF3, 0x09, INT[0], INT[1], INT[2], 0x55, INT[0], INT[1], INT[2], //
            0xB9, 0xF2, 0x09, INT[0], INT[1], INT[2], 0x55, INT[0], INT[1], INT[2], //
            0x4C, 0x41,
        ];
        assert_eq!(two[30], 0x4C);
        assert_eq!(skip_one(&two[30..], 0x4C), Some(Ok(1)));
        assert_eq!(two[31], 0x41);

        // A NESTED call's `4C`, which the workload walk excludes by
        // construction and this capture witnesses: the inner call's CALL-END is
        // followed immediately by the OUTER call's own `55 <TYPE>` argument
        // terminator (`int cnest(int x){ return g1(g1(x)); }`).
        let nested = [0x4C, 0x55, INT[0], INT[1], INT[2], 0x4C];
        assert_eq!(skip_one(&nested, 0x4C), Some(Ok(1)));
        assert_eq!(skip_one(&nested[1..], 0x55), Some(Ok(4)), "then the outer `55 <TYPE>`");

        // The zero-argument spelling still reads the same way — board #1318's
        // population is not contradicted, it is subsumed.
        let zero = [0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x09, 0x10, 0x00, 0x00, 0x4C, 0x41];
        assert_eq!(skip_one(&zero, 0xBD), Some(Ok(10)));
        assert_eq!(skip_one(&zero[10..], 0x4C), Some(Ok(1)));
    }

    /// **The rival readings, and the control that decides the residue.**
    ///
    /// `work/w-4c/argwalk.py` refutes `4C <byte>` at 1,460,194 of 1,978,436
    /// argument-bearing sites, `4C <TYPE>` at 214,003 (with no room for a TYPE
    /// at all at 87.7 %), and `4C <token>` at 1,371,969. This pins the
    /// MECHANISM for the one that matters, and it pins the residue's own
    /// explanation, which is the part a landing count cannot give.
    #[test]
    fn a_call_end_payload_would_eat_the_next_opcode_and_0x59_is_not_one() {
        const INT: [u8; 3] = [0x86, 0x41, 0x74];
        // `… 55 INT 4C 41 INT …` — one raw byte of payload swallows the `41`
        // and stops on `0x86`, a TYPE tag, which opens no operand token.
        let s = [0x4C, 0x41, INT[0], INT[1], INT[2]];
        assert_eq!(chain_skip_form(s[1]), Some(SkipForm::Type), "the real successor");
        assert_eq!(s[2], 0x86);
        assert_eq!(chain_skip_form(0x86), None, "a B1 reading lands on a TYPE tag");

        // THE RESIDUE. All 457 P desyncs on the workload land on `0x59` or
        // `0x08`, and the question that decides them never mentions `4C`: do
        // those bytes occur at token starts with some OTHER token in front?
        // `work/w-4c/unwit.py` says yes, 6,031 and 3,819 times, and never after
        // a `4C`. Transcribed here is one such site — a float subtract, then
        // `59`, with a `03` and not a `4C` before it.
        let float = [0x86, 0x45, 0x40];
        let notafter4c = [
            0xB9, 0xAF, 0x49, float[0], float[1], float[2], // load a float
            0x03, // subtract
            0x59, // …and the byte in question, with no `4C` anywhere
        ];
        assert_eq!(chain_skip_form(notafter4c[6]), Some(SkipForm::Bare));
        assert_eq!(skip_one(&notafter4c, 0xB9), Some(Ok(6)), "the load is 6 bytes");
        assert_ne!(notafter4c[6], 0x4C, "the predecessor is an operator, not a CALL-END");
        assert_eq!(chain_skip_form(0x59), None, "and `59` stays unpinned — it is not evidence about `4C`");
    }

    /// **The EH LIVE-STATE marker, from the CAPTURE and not from the table that
    /// reads it** (`lane w-5c`, board **#1423**).
    ///
    /// Transcribed off `work/w-5c/probe/eh5c.cpp`'s `.ex`, captured at this
    /// master by `c2rs capture` at the workload's own flags and graded
    /// `ReferenceReplay=ByteExact` against real `c2.dll` under wibo. The rows
    /// differ only in the destroyed object's TYPE, so the TYPE's LENGTH is the
    /// only field that moves — which is what makes them a witness for the
    /// *width* rather than for one destructor.
    #[test]
    fn the_eh_live_state_marker_consumes_exactly_its_capture() {
        // `void one_local() { MemA s; }` — a 4-byte object TYPE, state `01`, and
        // the `4B` statement end immediately after it.
        let one = [0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01, 0x4B];
        assert_eq!(skip_one(&one, 0x5C), Some(Ok(6)), "the marker ends on the 4B");
        assert_eq!(one[6], 0x4B, "and `4B` is the statement end");
        assert!(chain_skip_form(one[6]).is_some(), "which this table can step");

        // `int userfn(int a){ MemA s; g(a); return a+1; }` — the SAME token with
        // a 3-byte TYPE, from `EH_RECORDS.md` §7.1's own witness that `5C` is not
        // a ctor/dtor token. A fixed width cannot absorb both rows; the workload
        // carries 3-, 4-, 5-, 6- and 7-byte TYPEs here (197,660 / 64,437 /
        // 65,173 / 7,197 / 1,249 sites).
        let three = [0x5C, 0x86, 0x41, 0x74, 0x01, 0x4B];
        assert_eq!(skip_one(&three, 0x5C), Some(Ok(5)));

        // The OPERAND-position spelling, which `EH_RECORDS.md` §7.2 records
        // beside the statement one and which is 18.05 % of the workload's sites:
        // the marker stands before a `9B` bind rather than before a `4B`.
        let operand = [0x5C, 0xA6, 0x43, 0x8A, 0x20, 0x03, 0x9B, 0x86, 0x41, 0x74];
        assert_eq!(skip_one(&operand, 0x5C), Some(Ok(6)));
        assert_eq!(chain_skip_form(operand[6]), Some(SkipForm::TypeTok));

        // **THE ESCAPED STATE.** `EH_RECORDS.md` §7.1 published
        // `5C 86 41 74 80 01 01 00 00` in 2026-07-31 and this master's workload
        // reproduces it byte for byte at **9,645 sites in 812 TUs** — every
        // escaped-state site in the corpus is this exact sequence. It is the one
        // shape that separates `TypeVarint` from a one-raw-byte reading, and a
        // fixed-byte read stops on the `01` inside the LE32.
        let esc = [0x5C, 0x86, 0x41, 0x74, 0x80, 0x01, 0x01, 0x00, 0x00, 0x4B];
        assert_eq!(skip_one(&esc, 0x5C), Some(Ok(9)), "the escape is 5 payload bytes");
        assert_eq!(esc[9], 0x4B);

        // A payload that runs off the end refuses rather than clamping — a clamp
        // would report a fictitious successor.
        assert_eq!(skip_one(&[0x5C, 0x86, 0x41, 0x74], 0x5C), Some(Err("expr-chain-short")));
        assert_eq!(skip_one(&[0x5C], 0x5C), Some(Err("expr-chain-short")));
    }

    /// **The CLASS-PAIR DESCRIPTOR, `66 <arity> <arity LEB ids>`** (lane
    /// `w-mass`, board **#1530**), and the two rival readings `mcall`'s own doc
    /// records as having desynced on it.
    ///
    /// The narrow witnesses (`66 02 92 20 93 20`) are the ones a fixed-2-byte
    /// reading and a `read_token_var` reading both survive; the *wide* ones from
    /// `src/App.cpp` and `src/lazer/game/Game.cpp` are what separate them, and
    /// this row exists to keep them separated in this table too.
    #[test]
    fn the_class_pair_descriptor_consumes_exactly_its_capture() {
        // The narrow witness `mcall` transcribes, followed by the `55` argument
        // terminator that pins the width from the other side.
        let narrow = [0x66, 0x02, 0x92, 0x20, 0x93, 0x20, 0x55, 0x86, 0x41, 0x74];
        assert_eq!(skip_one(&narrow, 0x66), Some(Ok(6)), "arity byte + two LEB ids");
        assert_eq!(narrow[6], 0x55, "and the step lands on the argument terminator");

        // **The WIDE ids.** `fb 8a 01` is three LEB bytes, so a fixed-2-byte
        // reading stops inside the second id and a `read_token_var` reading
        // takes `fb 8a 01 …` as four bytes and oversteps by one. Only LEB lands
        // on the `55`.
        let wide = [0x66, 0x02, 0xFB, 0x8A, 0x01, 0xD3, 0x80, 0x02, 0x55, 0x86, 0x41, 0x74];
        assert_eq!(skip_one(&wide, 0x66), Some(Ok(8)));
        assert_eq!(wide[8], 0x55);

        // The ARITY is read, not assumed to be `02` — `IL_CALL_IN_EXPR.md` §4.3.
        let three = [0x66, 0x03, 0x92, 0x20, 0x93, 0x20, 0x94, 0x20, 0x55];
        assert_eq!(skip_one(&three, 0x66), Some(Ok(8)));
        let zero = [0x66, 0x00, 0x55, 0x86, 0x41, 0x74];
        assert_eq!(skip_one(&zero, 0x66), Some(Ok(2)));

        // A payload that runs off the end refuses rather than clamping. A clamp
        // would manufacture a fictitious successor key, which is the one way
        // this instrument could lie.
        assert_eq!(skip_one(&[0x66, 0x02, 0x92], 0x66), Some(Err("expr-chain-short")));
        assert_eq!(skip_one(&[0x66], 0x66), Some(Err("expr-chain-short")));

        // **The width is the DECODER's, not a copy of it.** If these two ever
        // disagree, the sink is stepping a token nothing else in the tree reads
        // the same way, and the successor keys it reports are fiction.
        for w in [narrow.as_slice(), wide.as_slice(), three.as_slice(), zero.as_slice()] {
            let mut p = 0usize;
            assert!(super::super::mcall::eat_class_descriptor(w, &mut p).is_some());
            assert_eq!(skip_one(w, 0x66), Some(Ok(p)), "the sink step IS the decoder");
        }
    }

    /// **The rival readings, and the population that decides the one the
    /// anchored walk could not** — the control that makes the row above a
    /// measurement instead of an assertion.
    ///
    /// `work/w-5c/scwalk.py` refutes payload-free at **335,716 of 335,716**
    /// anchored sites, `5C <TYPE>` at 210,570, `5C <varint>` at 130,991 and
    /// `5C <TYPE> <token>` at 59,181. The reading this pins is the last one
    /// standing at **0**.
    ///
    /// The interesting rival is the one those numbers cannot touch: `TypeVarint`
    /// and a hypothetical `TypeByte1` agree at every state below `0x80`, and the
    /// anchored walk reaches **zero** escaped sites. That is `0xBD`'s §2.2
    /// situation — and unlike `0xBD` it is decided, on the 9,744 raw-scan sites
    /// whose state byte IS `0x80`: the varint reading lands on the statement end
    /// at 98.98 % against a 60.66 % base rate, and the one-byte reading at
    /// **0.00 %**. This pins the mechanism.
    #[test]
    fn an_eh_state_read_as_one_byte_stops_inside_the_escape() {
        let esc = [0x5C, 0x86, 0x41, 0x74, 0x80, 0x01, 0x01, 0x00, 0x00, 0x4B];
        // The pinned reading lands on the statement end…
        assert_eq!(skip_one(&esc, 0x5C), Some(Ok(9)));
        assert_eq!(esc[9], 0x4B);
        // …and the one-raw-byte reading stops on index 5, spelled out rather than
        // computed so the byte is visible: TYPE is 3 bytes, the `80` is eaten as
        // the whole field, and the cursor lands inside the LE32's own payload.
        assert_eq!(esc[5], 0x01);
        assert_eq!(chain_skip_form(esc[5]), None, "0x01 opens nothing — the desync is visible");

        // And payload-free — the rival that took `4C` — is dead on its face here,
        // for the reason `w-divsplit` asks about: there is ALWAYS somewhere for
        // the payload to be. The byte after a `5C` has bit 7 set at 335,716 of
        // 335,716 anchored workload sites, so it opens a TYPE, and `4C`'s own
        // 87.7 %-no-room argument runs the other way.
        for s in [&one_local()[..], &esc[..]] {
            assert_ne!(s[1] & 0x80, 0, "the byte after a 5C is a TYPE tag");
            assert_eq!(chain_skip_form(s[1]), None, "…and opens no operand token");
        }
    }

    fn one_local() -> [u8; 7] {
        [0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01, 0x4B]
    }

    /// Run one [`chain_sink_step`] with `op` sunk, without touching the process
    /// environment (the config is a `OnceLock`, so a test cannot set it).
    fn skip_one(seg: &[u8], op: u8) -> Option<Result<usize, &'static str>> {
        let mut c = ChainSink::default();
        c.any = true;
        c.ops[op as usize] = true;
        chain_step_with(&c, seg, 0)
    }

    #[test]
    fn parse_formals_anchors_on_the_marker_that_reaches_lo() {
        // A function on source line 70 emits the line marker `4F 01 46`, whose
        // payload byte is `0x46`. Taking the first `0x46` in the segment finds
        // *that* and silently yields an empty formals list — which is not
        // fail-closed, because `leaves_ascending` skips non-formal tokens.
        let mut seg = vec![0x4F, 0x01, 0x46]; // line 70
        seg.extend_from_slice(IND_DEREF);
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        assert_eq!(parse_formals(&seg, lo), Ok(vec![0xEE09]));
        // And the whole body still parses, base register included.
        assert_eq!(
            parse_segment(&free_fn(&seg), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0xEE09],
                ops: vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }],
            })
        );
    }

    // ---- intrinsic-call (`0x40`) decode -------------------------------------
    //
    // Every byte array below is transcribed verbatim from a live-toolchain `.ex`
    // capture of a tracked fixture (`c2rs census <fixture> --keep-il <dir>`), not
    // hand-assembled — the whole point of the production is that its field widths
    // were guessed wrong twice before a capture settled them.

    /// `double t_fabs(double a){ return fabs(a); }`
    /// (`fixtures/cpp/il_intrinsic_call.cpp`, `?t_fabs@@YANN@Z`). Selector 17.
    const INTR_FABS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x11, 0x40, 0x88, 0x85, 0x41, 0xB9, 0x17,
        0x0A, 0x88, 0x85, 0x41, 0x55, 0x88, 0x85, 0x41, 0x4C, 0x41, 0x88, 0x85, 0x41, 0x3A, 0x19,
        0x0A, 0x54, 0x02, 0x29, 0x19, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `void n_break(){ __debugbreak(); }`
    /// (`fixtures/cpp/il_intrinsic_nullary.cpp`, `?n_break@@YAXXZ`). Selector 543,
    /// **zero arguments** — the witness that `40 <TYPE>` carries no trailing field.
    const INTR_NULLARY: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x80, 0x1F, 0x02, 0x00, 0x00, 0x40, 0x82,
        0x07, 0x03, 0x4C, 0x4B, 0x3A, 0xFF, 0x09, 0x54, 0x02, 0x29, 0xFF, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00,
    ];
    /// `A2 *l_up2(M *m){ return m; }`
    /// (`fixtures/cpp/il_intrinsic_layout.cpp`). Selector 2114, offset literal `08`.
    const INTR_UPCAST: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x80, 0x42, 0x08, 0x00, 0x00, 0x40, 0x86,
        0x43, 0xB1, 0x20, 0x66, 0x02, 0x92, 0x20, 0x93, 0x20, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86,
        0x41, 0x74, 0x08, 0x55, 0x86, 0x41, 0x74, 0xB9, 0x41, 0x0A, 0x86, 0x43, 0xB0, 0x20, 0x55,
        0x86, 0x43, 0xB0, 0x20, 0x4C, 0x41, 0x86, 0x43, 0xB1, 0x20, 0x3A, 0x43, 0x0A, 0x54, 0x02,
        0x29, 0x43, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `void l_this2(M *m){ m->mb(); }`
    /// (`fixtures/cpp/il_intrinsic_layout.cpp`). Selector 2113, offset literal `08`
    /// — byte-for-byte the same descriptor and offset as [`INTR_UPCAST`], reached
    /// through the `26 <sym>` path instead.
    const INTR_THIS_ADJUST: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF2, 0x09, 0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00,
        0x00, 0x40, 0xA6, 0x43, 0x96, 0x20, 0x66, 0x02, 0x92, 0x20, 0x93, 0x20, 0x55, 0x86, 0x41,
        0x74, 0x33, 0x86, 0x41, 0x74, 0x08, 0x55, 0x86, 0x41, 0x74, 0xB9, 0x48, 0x0A, 0x86, 0x43,
        0xB0, 0x20, 0x55, 0x86, 0x43, 0xB0, 0x20, 0x4C, 0x99, 0x86, 0x43, 0x97, 0x20, 0x00, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x17, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0x4A, 0x0A, 0x54,
        0x02, 0x29, 0x4A, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn intrinsic_call_census_reports_the_selector_not_the_opcode() {
        // The whole `0x40` production is one census bucket only because the
        // selector was never decoded. Every site must name the intrinsic.
        //
        // `INTR_THIS_ADJUST` reports `expr-` rather than `call-` since the body
        // dispatch keys on whether a `BD` follows the first `26 <tok>` immediately.
        // Here it does not — the `BD` is fifty bytes later, behind argument-shaped
        // material — so the body goes to the assignment parser and the intrinsic is
        // named from the expression it sits in. That is the claim I can support from
        // these bytes; asserting the enclosing construct is a call would be
        // asserting more. The selector is named either way, so the histogram is
        // unaffected in aggregate, and `intrinsic_call_decode_does_not_accept`
        // pins that both routings still refuse.
        for (seg, want) in [
            (INTR_FABS, "expr-intrinsic-fabs"),
            (INTR_NULLARY, "expr-intrinsic-__debugbreak"),
            (INTR_UPCAST, "expr-intrinsic-base-upcast"),
            (INTR_THIS_ADJUST, "expr-intrinsic-this-adjust"),
        ] {
            let seg = free_fn(seg);
            let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
            assert_eq!(b.feature(), want);
            // The block is reported at the selector literal, whose `40` follows.
            assert_eq!(seg[b.off], 0x33, "{want}");
        }
    }

    #[test]
    fn intrinsic_call_decode_does_not_accept() {
        // Decoding is not accepting. Every one of these still fails closed, so
        // the census and the emission gate cannot disagree — the same invariant
        // `census_agrees_with_the_gate_on_every_pinned_segment` checks globally.
        for seg in [INTR_FABS, INTR_NULLARY, INTR_UPCAST, INTR_THIS_ADJUST] {
            assert!(parse_segment(&free_fn(seg), NO_LOCALS).is_none());
        }
    }

    #[test]
    fn intrinsic_call_token_has_no_trailing_field() {
        // `40 <TYPE>` and nothing else: in the nullary capture the `4C` apply sits
        // immediately after the `void` result type, so a `40 <TYPE> <varint>`
        // reading (the shape `2C`/`99`/`9B`/`5C` have, and the one an earlier
        // session assumed) would swallow the terminator.
        let p = 4; // the selector literal, right after `4C 4F 11 53`
        assert_eq!(intrinsic_selector(INTR_NULLARY, p), Some(543));
        let tok = p + 9; // `33 86 41 74` + the 5-byte escaped varint
        assert_eq!(INTR_NULLARY[tok], 0x40);
        let (_, _, _, w) = read_type(INTR_NULLARY, tok + 1).unwrap();
        assert_eq!(&INTR_NULLARY[tok + 1..tok + 1 + w], &[0x82, 0x07, 0x03]); // void
        assert_eq!(INTR_NULLARY[tok + 1 + w], 0x4C); // the apply, with no field between
    }

    #[test]
    fn same_descriptor_and_offset_different_selector_is_a_different_emission() {
        // 2113 and 2114 carry an identical `66 02 92 20 93 20` class-pair
        // descriptor and an identical offset literal `08`, and c2 emits
        // `addi r3,r3,8` for one and a null-guarded five-instruction form for the
        // other (see `fixtures/cpp/il_intrinsic_layout.cpp`). So the census must
        // separate them, and a lowering keyed on the offset alone would be wrong.
        let up = parse_segment_detail(&free_fn(INTR_UPCAST), NO_LOCALS).unwrap_err();
        let this = parse_segment_detail(&free_fn(INTR_THIS_ADJUST), NO_LOCALS).unwrap_err();
        assert_ne!(up.feature(), this.feature());
        assert_eq!(up.aux, 2114);
        assert_eq!(this.aux, 2113);
        // Both offset literals really are the same byte.
        assert_eq!(INTR_UPCAST[32], 0x08);
        assert_eq!(INTR_THIS_ADJUST[35], 0x08);
    }

    #[test]
    fn selector_must_be_exactly_int_typed_or_the_decode_declines() {
        // The one structural claim the decode rests on is that `0x40` is always
        // preceded by an `int`-typed literal. Retype the `t_fabs` selector to
        // `unsigned` (`86 42 75`) and the decode must decline rather than report a
        // selector it cannot vouch for — falling back to the honest
        // `expr-intrinsic-call` residue, which is what measures the claim over the
        // real workload (measured: 0 of 213,411 sites land in the residue).
        let mut seg = INTR_FABS.to_vec();
        seg[5] = 0x86;
        seg[6] = 0x42;
        seg[7] = 0x75;
        assert_eq!(intrinsic_selector(&seg, 4), None);
        let b = parse_segment_detail(&free_fn(&seg), NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-intrinsic-call");
    }

    // ---- the `2C` conversion in an expression operand position (D12) --------
    //
    // Every array below is a whole function segment transcribed verbatim from a
    // live capture of `fixtures/cpp/w20_convert.cpp` (`c2rs census … --keep-il`),
    // not hand-assembled: the point of the production is *where* the `2C` sits
    // relative to the operands, and only a capture settles that.

    /// `unsigned c_u_of_i(int a) { return (unsigned)a; }` — one operand, the
    /// conversion between it and the `41` result. `int` → `unsigned`, both width 4.
    const CONV_ONE: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x1E, 0x53, 0x53, 0x26, 0x01, 0x0A,
        0x46, 0x2D, 0x00, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x00, 0x0A, 0x86, 0x41, 0x74, 0x2C,
        0x86, 0x42, 0x75, 0x00, 0x41, 0x86, 0x42, 0x75, 0x3A, 0x02, 0x0A, 0x54, 0x02, 0x29, 0x02,
        0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `unsigned ch_trail(int a, int b) { return a + (unsigned)b; }` — the
    /// conversion sits **between the two operands**, before the `02` ADD.
    const CONV_INTERIOR: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x2C, 0x53, 0x53, 0x26, 0x25, 0x0A,
        0x46, 0x2D, 0x24, 0x0A, 0x2D, 0x23, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x23, 0x0A, 0x86,
        0x41, 0x74, 0xB9, 0x24, 0x0A, 0x86, 0x41, 0x74, 0x2C, 0x86, 0x42, 0x75, 0x00, 0x02, 0x41,
        0x86, 0x42, 0x75, 0x3A, 0x26, 0x0A, 0x54, 0x02, 0x29, 0x26, 0x0A, 0x4F, 0x12, 0x47, 0x54,
        0x01, 0x54, 0x00,
    ];

    /// `unsigned ch_whole(int a, int b) { return (unsigned)(a + b); }` — the same
    /// two operands and the same ADD, with the conversion **after** the operator.
    /// Byte-for-byte the same operand stream as [`CONV_INTERIOR`] with the `2C`
    /// unit moved four bytes later, and it must lower identically.
    const CONV_TRAILING: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x2E, 0x53, 0x53, 0x26, 0x30, 0x0A,
        0x46, 0x2D, 0x2F, 0x0A, 0x2D, 0x2E, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x2E, 0x0A, 0x86,
        0x41, 0x74, 0xB9, 0x2F, 0x0A, 0x86, 0x41, 0x74, 0x02, 0x2C, 0x86, 0x42, 0x75, 0x00, 0x41,
        0x86, 0x42, 0x75, 0x3A, 0x31, 0x0A, 0x54, 0x02, 0x29, 0x31, 0x0A, 0x4F, 0x12, 0x47, 0x54,
        0x01, 0x54, 0x00,
    ];

    /// `int p_void(S *s) { return gv(s); }` — the POINTER half, and the position
    /// that carries it on the real workload: a `T*` → `void*` conversion inside a
    /// call-argument region, which is `parse_expr`'s other caller.
    const CONV_PTR_ARG: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x4D, 0x53, 0x53, 0x26, 0xA0, 0x0A,
        0x46, 0x2D, 0x9F, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xFC, 0x09, 0xBD, 0x86, 0x41, 0x74,
        0x00, 0x80, 0x0D, 0x10, 0x00, 0x00, 0xB9, 0x9F, 0x0A, 0x86, 0x43, 0x89, 0x20, 0x2C, 0x86,
        0x43, 0x83, 0x08, 0x00, 0x55, 0x86, 0x43, 0x83, 0x08, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0xA1, 0x0A, 0x54, 0x02, 0x29, 0xA1, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// Replace the first `2C` unit's TYPE in `seg` with `ty`, keeping the trailing
    /// `00`. Used to move the target across the class boundary without touching
    /// anything else about the segment.
    fn retarget(seg: &[u8], ty: &[u8]) -> Vec<u8> {
        let at = seg
            .windows(4)
            .position(|w| w[0] == 0x2C && w[1] == 0x86 && (w[2] == 0x42 || w[2] == 0x43))
            .expect("a 2C unit");
        let old = if seg[at + 2] == 0x43 { 4 } else { 3 };
        let mut v = seg[..at + 1].to_vec();
        v.extend_from_slice(ty);
        v.extend_from_slice(&seg[at + 1 + old..]);
        v
    }

    #[test]
    fn a_class_preserving_convert_is_free_at_every_position() {
        // The conversion emits nothing, so it must push no `IlOp` — and a body
        // that differs from an accepted one only by where its `2C` sits must
        // produce the IDENTICAL operand stream. That is the whole claim: c2
        // compiles `(unsigned)a + b`, `a + (unsigned)b` and `(unsigned)(a + b)` to
        // the same `add r3,r3,r4 ; blr`.
        assert_eq!(
            parse_segment(CONV_ONE, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x000A],
                ops: vec![IlOp::Load(0x000A)],
            })
        );
        let interior = parse_segment(CONV_INTERIOR, NO_LOCALS);
        let trailing = parse_segment(CONV_TRAILING, NO_LOCALS);
        assert_eq!(
            interior,
            Some(BodyShape::StraightLine {
                params: vec![0x230A, 0x240A],
                ops: vec![IlOp::Load(0x230A), IlOp::Load(0x240A), IlOp::Add],
            })
        );
        // Same shape, same operator, same operand order — only the token ids
        // differ, because they are two different functions in one TU.
        let Some(BodyShape::StraightLine { ops, .. }) = trailing else {
            panic!("the trailing form must parse as the same shape");
        };
        assert_eq!(ops, vec![IlOp::Load(0x2E0A), IlOp::Load(0x2F0A), IlOp::Add]);
    }

    #[test]
    fn a_pointer_convert_in_a_call_argument_is_free() {
        // The half of the workload population that is `calls-1`: the argument of a
        // tail call, cv-stripped or widened to `void*` on the way in.
        assert!(matches!(
            parse_segment(CONV_PTR_ARG, NO_LOCALS),
            Some(BodyShape::IntTailCall { .. })
        ));
    }

    #[test]
    fn the_width4_reinterpret_is_free_in_both_directions() {
        // **CORRECTED, board #700.** This test used to assert that
        // `int f(S* p){ return (int)p; }` and `S* f(int a){ return (S*)a; }`
        // REFUSE under `expr-convert-target-8643`, with a comment saying in as
        // many words that both are a bare `blr` and that the refusal was "a
        // conservatism, not a correction". `lane w-convert` graded the 3x3 and
        // the conservatism is spent: the width-4 reinterpret is admitted, and it
        // must produce the IDENTICAL operand stream to the same body without the
        // conversion, because c2 emits nothing for it.
        //
        // `86 43 83 08` is `void *` — the int4 → ptr4 direction, on the body
        // whose value is an `int` LOAD.
        let to_ptr = retarget(CONV_ONE, &[0x86, 0x43, 0x83, 0x08]);
        assert_eq!(
            parse_segment(&to_ptr, NO_LOCALS),
            parse_segment(CONV_ONE, NO_LOCALS),
            "a reinterpret must lower exactly as the class-preserving convert does"
        );
        assert!(parse_segment_detail(&to_ptr, NO_LOCALS).is_ok());
        // `86 41 74` is `int` — the ptr4 → int4 direction, on the body whose
        // value is an `S *` LOAD inside a call-argument region.
        let to_int = retarget(CONV_PTR_ARG, &[0x86, 0x41, 0x74]);
        assert!(matches!(
            parse_segment(&to_int, NO_LOCALS),
            Some(BodyShape::IntTailCall { .. })
        ));
    }

    #[test]
    fn a_reinterpret_to_a_pointer_indicts_the_value_for_the_pointer_guard() {
        // **The wrong-emit this rung could have shipped, as a test.** `(S *)a + 1`
        // is `addi r3,r3,8` and `(S *)a + k` is `slwi r11,r4,3 ; add` — c2 SCALES
        // pointer arithmetic — so a chain that accepted the reinterpret and then
        // added unscaled would emit wrong bytes rather than decline. The accepting
        // arm sets `saw_ptr`, and this is the assertion that it does: the interior
        // form is an ADD chain, and retargeting its conversion to `void *` must
        // turn an accepted body into an `expr-ptr-arith` refusal.
        assert!(parse_segment(CONV_INTERIOR, NO_LOCALS).is_some());
        let seg = retarget(CONV_INTERIOR, &[0x86, 0x43, 0x83, 0x08]);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-ptr-arith:mid");
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
    }

    #[test]
    fn a_convert_to_a_narrower_type_refuses() {
        // `(char)a` is `extsb r3,r3` and `(long long)a` is `extsw r3,r3` — real
        // instructions the modeled chain cannot produce. `82 11 70` is `char`.
        for (ty, want) in [
            (&[0x82u8, 0x11, 0x70][..], "expr-convert-target-8211"),
            (&[0x84, 0x21, 0x11][..], "expr-convert-target-8421"),
            (&[0x88, 0x81, 0x13][..], "expr-convert-target-8881"),
            (&[0x86, 0x45, 0x76][..], "expr-convert-target-8645"),
        ] {
            let seg = retarget(CONV_ONE, ty);
            let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
            assert_eq!(b.feature(), want);
        }
    }

    #[test]
    fn the_convert_s_trailing_field_is_required_to_be_zero() {
        // It is `00` at every aligned site any capture has produced, and a field
        // that never varied is indistinguishable from a constant (`GAPS.md` §6),
        // so it is required literally and its exceptions get their own key rather
        // than being skipped over.
        let at = CONV_ONE
            .windows(5)
            .position(|w| w[0] == 0x2C && w[1] == 0x86 && w[2] == 0x42 && w[4] == 0x00)
            .expect("the 2C unit");
        let mut seg = CONV_ONE.to_vec();
        seg[at + 4] = 0x01;
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-convert-tail-0x01");
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
    }

    #[test]
    fn unpinned_selector_ids_stay_hex() {
        // A hex bucket is a result; a wrong name is a lie that survives into the
        // roadmap. 222/223 occur 1758 times each on the real workload and their
        // trigger is pinned (`fixtures/cpp/il_intrinsic_byval.cpp`) while their
        // individual semantics are not, so they must not be named.
        assert_eq!(intrinsic_name(222), "0xDE");
        assert_eq!(intrinsic_name(223), "0xDF");
        assert_eq!(intrinsic_name(2120), "0x848");
        assert_eq!(intrinsic_name(17), "fabs");
    }

    // ---- lane w-divsplit: the DIVIDE/MODULO key carries its operand type ----

    /// `33 <TYPE> <payload>` — a literal operand, the form that ends at the
    /// division opcode at **4,674 of 4,674** dc3 sites.
    fn lit(ty: [u8; 3], v: u8) -> Vec<u8> {
        let mut b = vec![0x33];
        b.extend_from_slice(&ty);
        b.push(v);
        b
    }

    fn key_of(seg: &[u8]) -> String {
        let mut p = 0;
        parse_expr_classed(seg, &mut p, 0x41).unwrap_err().feature()
    }

    #[test]
    fn the_div_mod_key_names_the_operand_type() {
        // `(a) / 2` with int operands: the key that used to read `expr-op-0x05`
        // now says which 4,670 of the workload it is.
        let mut seg = lit([0x86, 0x41, 0x74], 10);
        seg.extend_from_slice(&lit([0x86, 0x41, 0x74], 2));
        seg.push(0x05);
        assert_eq!(key_of(&seg), "expr-op-0x05-8641");

        // Modulo takes the same treatment and keeps its own byte in the name —
        // #782's two buckets must stay two buckets.
        let mut seg = lit([0x86, 0x41, 0x74], 10);
        seg.extend_from_slice(&lit([0x86, 0x41, 0x74], 2));
        seg.push(0x06);
        assert_eq!(key_of(&seg), "expr-op-0x06-8641");

        // The UNSIGNED operand type is a different key, and it is reachable:
        // `86 42` is what `is_int4_type` admits beside `86 41`.
        let mut seg = lit([0x86, 0x42, 0x75], 10);
        seg.extend_from_slice(&lit([0x86, 0x42, 0x75], 2));
        seg.push(0x05);
        assert_eq!(key_of(&seg), "expr-op-0x05-8642");
    }

    #[test]
    fn a_division_with_no_operand_type_read_says_so() {
        // The presence flag earns its bit. A bare opcode records no type and the
        // key says `notype` rather than defaulting into the integer bucket —
        // "absence read as success" is `docs/STATUS.md` trap 5.
        assert_eq!(key_of(&[0x05]), "expr-op-0x05-notype");
    }

    /// **The FLOAT case is UNREACHABLE through this parser, and that is the
    /// answer to board #783** — not a gap in the instrument.
    ///
    /// A census key is a body's FIRST blocker. Every operand-producing arm
    /// admits a type only through `eat_operand_type` (`Int4`/`Ptr4`/`Int1u`) or
    /// an admitted `2C`, so a `float` or `double` operand refuses at the LOAD,
    /// one token *before* the opcode — under a different key. This test is the
    /// positive check for that claim rather than an argument for it.
    #[test]
    fn a_float_operand_blocks_at_the_load_not_at_the_division() {
        // `86 45 83` — tag 86, kind 45 (low nibble 5 = REAL), a 4-byte float.
        let mut seg = lit([0x86, 0x45, 0x83], 10);
        seg.extend_from_slice(&lit([0x86, 0x45, 0x83], 2));
        seg.push(0x05);
        let k = key_of(&seg);
        assert_eq!(k, "expr-lit-type-8645", "a float division blocks at its operand");
        assert!(!k.starts_with("expr-op-0x05"), "and so never reaches the 0x05 key");

        // Same one token over, through the LOAD arm. `09 0A` is the two-byte
        // token form — `read_token_var` takes four bytes when the second has its
        // high bit set, which is #644's own shape and would silently slide the
        // TYPE this test is about.
        let mut seg = vec![0xB9, 0x09, 0x0A];
        seg.extend_from_slice(&[0x86, 0x45, 0x83]);
        seg.push(0x05);
        assert_eq!(key_of(&seg), "expr-load-type-8645");
    }

    #[test]
    fn the_div_mod_key_is_an_exact_refinement_of_the_published_one() {
        // A REFINEMENT: every key this ctx can produce starts with the exact
        // string the board published (`expr-op-0x05` / `expr-op-0x06`), so the
        // 4,670 can only split and no row can move sideways into another
        // bucket. Checked over the whole product the producer can emit —
        // both opcodes x {no type, every tag x kind byte} — rather than over the
        // handful a fixture happens to reach.
        for b in DIV_MOD_OPS {
            let old = format!("expr-op-0x{b:02X}");
            let mut seen = std::collections::BTreeSet::new();
            for aux in std::iter::once(0u64).chain(
                (0u64..256).flat_map(|tag| (0u64..256).map(move |kind| 1 | tag << 16 | kind << 8)),
            ) {
                let blk = Block {
                    ctx: EXPR_TYPED_OP,
                    byte: Some(b),
                    off: 0,
                    seg_len: 0,
                    aux,
                };
                let new = blk.feature();
                assert!(new.starts_with(&old), "{new} is not a refinement of {old}");
                // …and the refinement is INJECTIVE on the recorded type: two
                // different types may never share a bucket, or the split would
                // be reporting a merge.
                assert!(seen.insert(new), "two aux values share one key");
                // The completeness axis is untouched — this refusal carries a
                // blocking byte exactly as the fall-through one did.
                assert_eq!(blk.completeness(), Complete::NoSignal);
            }
        }
    }
}

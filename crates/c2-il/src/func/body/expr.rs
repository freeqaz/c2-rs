use super::{blk, blk_type, Block};
use crate::func::readers::{
    eat, eat_byte, eat_int_like_or_ptr4, eat_operand_type, eat_opt_stmt_marker, eat_value_type,
    read_token_var, read_type, read_varint, ValueClass, INT_TYPE,
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
    // Set by `02`/`03`/`04` — arithmetic whose pointer form c2 SCALES by the
    // pointee width, which is what the pointer guard below exists to refuse.
    // The `27` byte-offset add is not that and does not set it.
    let mut scaled_arith = false;
    // The [`ValueClass`] of the value on top of the operand stack, or `None`
    // before the first operand. Only the `2C` arm reads it, and only to require
    // that a conversion stays inside the class it started in — which is what makes
    // the top of the stack knowable from the last operand alone. Every accepted
    // conversion is class-preserving and every accepted operator is int-only (the
    // pointer-arithmetic guard below bars the mixed case), so the whole
    // sub-expression has ONE class and this variable is that class, not a
    // shortcut for it.
    let mut class: Option<ValueClass> = None;
    loop {
        let b = *seg.get(*p).ok_or(blk(seg, *p, "expr"))?;
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
            return Err(Block {
                ctx: "expr-intrinsic",
                byte: Some(0x40),
                off: *p,
                seg_len: seg.len(),
                aux: id as u64,
            });
        }
        match b {
            0xB9 => {
                // LOAD <token> <int-type>
                let start = *p;
                *p += 1;
                let (tok, w) =
                    read_token_var(seg, *p).ok_or(blk(seg, *p, "expr-load-tok"))?;
                *p += w;
                match eat_operand_type(seg, p) {
                    Some(c) => {
                        saw_ptr |= c == ValueClass::Ptr4;
                        saw_int1u |= c == ValueClass::Int1u;
                        saw_wide |= c != ValueClass::Int1u;
                        class = Some(c);
                    }
                    // neither int-like nor a 4-byte pointer → out of class.
                    // Report at the LOAD so the census bucket reads as a
                    // typed-operand gap, not a stray byte.
                    None => return Err(blk_type(seg, *p, start, "expr-load-type")),
                }
                ops.push(IlOp::Load(tok));
            }
            0x33 => {
                // LITERAL: 33 <int-type> <varint>
                let start = *p;
                *p += 1;
                match eat_operand_type(seg, p) {
                    Some(c) => {
                        saw_ptr |= c == ValueClass::Ptr4;
                        saw_int1u |= c == ValueClass::Int1u;
                        saw_wide |= c != ValueClass::Int1u;
                        class = Some(c);
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
                ops.push(IlOp::Add);
            }
            0x03 => {
                *p += 1;
                scaled_arith = true;
                ops.push(IlOp::Sub);
            }
            0x04 => {
                *p += 1;
                scaled_arith = true;
                ops.push(IlOp::Mul);
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
            0x27 if off_add_sink_enabled() => {
                *p += 1;
                match read_type(seg, *p) {
                    Some((_, _, _, w)) => *p += w,
                    None => return Err(blk(seg, *p, "expr-off-add-type")),
                }
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
            // **Cross-class refuses**, and that is a conservatism with a measured
            // price rather than an oversight: `int f(S* p){ return (int)p; }` and
            // `S* f(int a){ return (S*)a; }` are both a bare `blr` on this target
            // (probed), so admitting them is a rung — but a reinterpret between the
            // two classes has never been graded across the widths, cv-spellings and
            // argument positions this parser reaches, and `expr-convert-target`
            // makes what it costs a number instead of an argument.
            //
            // The trailing varint is required to be literally `0`. It is `00` at
            // every aligned site any capture has shown, and a field that never
            // varied is indistinguishable from a constant (`GAPS.md` §6), so it is
            // required rather than skipped and its own key counts the exceptions.
            0x2C => {
                let start = *p;
                let mut probe = *p + 1;
                // A conversion with nothing to convert is not a conversion. This
                // cannot be reached by a well-formed stream and refuses rather than
                // guessing a class.
                let Some(cls) = class else {
                    return Err(blk(seg, start, "expr-convert-no-value"));
                };
                if !eat_value_type(seg, &mut probe, cls) {
                    // Either an unmodeled target type (`char`, `long long`, a
                    // float) or a cross-class reinterpret. Reported at the target
                    // TYPE so the key names it (`<tag><kind>`, never the per-TU id).
                    return Err(blk_type(seg, *p + 1, start, "expr-convert-target"));
                }
                if !eat_byte(seg, &mut probe, 0x00) {
                    return Err(blk(seg, probe, "expr-convert-tail"));
                }
                *p = probe;
            }
            0x26 => return Err(super::mcall::classify(seg, *p)),
            _ => return Err(blk(seg, *p, "expr")),
        }
    }
    if ops.is_empty() {
        return Err(blk(seg, *p, "expr-empty"));
    }
    // The pointer-arithmetic guard. A pointer operand anywhere in this value plus
    // any modeled arithmetic anywhere in it refuses the whole function — see the
    // header. `expr-ptr-arith` is its own census key so the cost of the guard is
    // a number rather than an argument.
    let ptr_arith = if off_add_sink_enabled() {
        // Only the SCALED forms indict a pointer value; see the `0x27` arm.
        saw_ptr && scaled_arith
    } else {
        saw_ptr
            && ops
                .iter()
                .any(|o| matches!(o, IlOp::Add | IlOp::Sub | IlOp::Mul))
    };
    if ptr_arith {
        return Err(Block::refuse(seg, *p, "expr-ptr-arith"));
    }
    // The one-byte-unsigned guard, and it is the pointer guard's twin: the class
    // is free to be *moved* and not to be *computed on*. `b1 + b2` in C++ converts
    // both operands to `int` first, so an accepted chain over raw `bool` operands
    // has no witness at all; and a chain mixing the class with a width-4 one is a
    // conversion the IL would have spelled with a `2C`. Both refuse under their own
    // census keys, so what the guard costs is a number rather than an argument.
    if saw_int1u {
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
    use crate::func::body::{parse_segment, parse_segment_detail, BodyShape};
    use crate::func::bundle::LO_MARKER;
    use crate::func::readers::{find_subslice, read_type};
    use crate::func::test_fixtures::*;

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
    fn a_convert_out_of_the_value_s_class_refuses_and_names_its_target() {
        // `int f(S* p){ return (int)p; }` and `S* f(int a){ return (S*)a; }` are
        // both a bare `blr` on this target — the refusal is a conservatism, not a
        // correction — so the key has to say what was refused rather than hide it
        // in a generic bucket. `86 43 83 08` is `void *`.
        let seg = retarget(CONV_ONE, &[0x86, 0x43, 0x83, 0x08]);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-convert-target-8643");
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
}

//! The assignment statement body — `s->m = v;` and its neighbours as a
//! *statement* rather than an expression, before the leaf recognizers see it.

use crate::func::body::chain::{
    canonical_chain_for_codegen, has_repeated_leaf, ChainReject,
    straight_line_out_of_class_ctx, substitute, MAX_SUBST_OPS,
};
use crate::func::body::expr::{eat_return_plumbing, eat_scopes, parse_expr, parse_formals};
use crate::func::body::mcall;
use crate::func::body::{blk, blk_type, Block, BodyShape};
use crate::func::readers::{
    eat_byte, eat_int_like, eat_int_like_or_ptr4, read_token_var, read_type,
};
use crate::func::IlOp;

use super::calls::parse_call_shape;
use super::params::parse_params;

/// Try to parse a body that is a **list of assignment statements** followed by a
/// returned expression:
///
/// ```text
///   body := ( `4F 01 <line>`* assign )* `4F 01 <line>`* expr(→41) <return int>
///   assign := `26 <dst>` expr(→32) `32 <TYPE>` `4B`
/// ```
///
/// `26 <dst>` pushes the destination, `32 <TYPE>` stores it (and yields the value,
/// which `4B` then discards). The `<TYPE>` is the destination's own — a conversion
/// is always a separate visible `2C`, so an int-like type here means no conversion.
///
/// **These bodies need no stores at all.** c2 register-allocates locals and
/// coalesces the copies, so the whole class collapses to the expression that
/// actually reaches the `return`. Captured:
///
/// ```text
///   int x; x = a; return x;              -> blr            (x is already r3)
///   int x = a; int y = x; return y;      -> blr
///   a = a + 1; return a;                 -> addi r3,r3,1
///   int x = 0; x = a + 1; return x;      -> addi r3,r3,1    (the x = 0 is dead)
///   a = 7; return a;                     -> li r3,7
/// ```
///
/// So this resolves the statement list by substitution and hands codegen the
/// resulting straight-line expression, which is exactly what the reference emits.
///
/// The destination must be a **formal** (positively, from the `2D` list) or an
/// automatic `int` **local** (positively, from `.sy` — see [`crate::func::sy`]).
/// Both sites that bind one ask, and the refusal is raised LAST, at the `26`
/// that pushed it — see [`dst_not_formal`], which carries what deferring it
/// measured (WAE, `docs/rungs/2026-07-31-assign-eof.md`).
///
/// An earlier version asked whether `.gl` named the destination and refused if so.
/// That looked sound and was not: a file-scope `static int sv` appears there as
/// `$sv`, whose leading `$` `gl_symbol_index` does not accept as an identifier, so
/// the token looked local and the store was silently dropped. Absence from a symbol
/// table proves nothing — it only says the table did not happen to name it.
///
/// `.sy` replaces that absence test with a membership one. Globals, `extern`
/// declarations and file-scope statics appear in `.sy` not at all, and a
/// function-scope `static` appears as a differently-tagged record, so a token
/// found in the locals section is a register-resident value and folding its store
/// into the expression that reads it is what c2 itself does — measured: for
/// `int f(int a){int x=a+1;int y=x+2;return y;}` and `int f(int a){return
/// (a+1)+2;}` the reference objs differ only in the source-filename bytes.
pub(crate) fn try_parse_assign_body_detail(
    seg: &[u8],
    start: usize,
    lo: usize,
    locals: &[u32],
    mut depth: usize,
) -> Result<BodyShape, Block> {
    let mut p = start;
    // The `26` push of the FIRST statement whose destination this class will not
    // fold, held until the rest of the body has had its say. See `dst_not_formal`
    // below for why it is deferred rather than refused on the spot.
    let mut deferred: Option<usize> = None;
    let mut env: Vec<(u32, Vec<IlOp>)> = Vec::new();
    // Read once for the per-destination check. A body with no formals marker is not
    // rejected here — the destination check below refuses it anyway, and deferring
    // lets the right-hand side report its own reason first, which is what makes the
    // census name the innermost unmodeled construct rather than this outer gate.
    let formals = parse_formals(seg, lo).unwrap_or_default();
    loop {
        // Brace scopes open and close *between* statements, so they are consumed at
        // the boundary rather than being a statement of their own.
        eat_scopes(seg, &mut p, &mut depth)?;
        if *seg.get(p).ok_or(blk(seg, p, "assign-stmt"))? != 0x26 {
            break;
        }
        let mut probe = p + 1;
        let (dst, w) =
            read_token_var(seg, probe).ok_or(blk(seg, probe, "assign-dst-tok"))?;
        probe += w;
        // `BD` here means this `26` was a callee push, not a destination. The caller
        // dispatched on the FIRST one, so reaching this means the right-hand side is
        // itself a call: `int z = g(a); …`. When the very next thing is a return of
        // that same local the whole body is a tail call, which `parse_call_shape`
        // already handles given the bound token — so hand it over rather than refuse.
        //
        // Only valid as the FIRST statement: with `env` non-empty, earlier
        // assignments have already been folded away and would be lost.
        // The right-hand side is a call when it opens with its own `26 <callee>`
        // followed by the `BD` CALL opcode — two tokens along from the destination,
        // not one.
        let rhs_is_call = *seg.get(probe).ok_or(blk(seg, probe, "assign-op"))? == 0x26
            && match read_token_var(seg, probe + 1) {
                Some((_, cw)) => seg.get(probe + 1 + cw) == Some(&0xBD),
                None => false,
            };
        if rhs_is_call {
            if env.is_empty() {
                let mut q = probe;
                // The destination test applies HERE TOO, and used to not: this
                // route hands `dst` to the call shape as a bound token without
                // ever asking what `dst` is, so `extern int gv; int f(int a){ gv =
                // g(a); return gv; }` censused in class as an `int-tail-call`
                // with the store to `gv` folded away — the exact defect `.sy` was
                // built to stop, at the one site that did not consult it. It was
                // never a live mis-emit, but only because `IlBundle::functions`
                // refuses any TU whose `.gl` carries an unclaimed data symbol
                // (`fixtures/cpp/il_gl_data_symbol.cpp`), and a function-level
                // class must not be sound only by a translation-unit accounting
                // rule about something else.
                //
                // AFTER the call parse, not before, for the reason the main path
                // defers as well — a check placed before it would relabel every
                // body this production already refuses for its own reasons.
                // MEASURED: 14,454 workload functions reach this branch with a
                // destination the class will not fold, and **0** of them parse;
                // refusing them up front would invent a 14,454-row census bucket
                // naming none of its own contents.
                let shape = parse_call_shape(seg, &mut q, lo, Some(dst))?;
                if !formals.contains(&dst) && !locals.contains(&dst) {
                    return Err(dst_not_formal(seg, p));
                }
                return Ok(shape);
            }
            return Err(blk(seg, probe, "assign-rhs-call"));
        }
        // `start_of_stmt` is the byte the statement opened on — the `26` this loop
        // just consumed as a destination. It may not be one: a member call in a
        // *value* position opens on its outermost method push, which is the same
        // two bytes. Only the right-hand side's own refusal can tell, so the
        // statement head is carried to [`mcall::reanchor_chain`], which puts it back
        // when (and only when) the walk and the bind count agree that the token was
        // a method. `docs/IL_CALL_IN_EXPR.md` §16.4 measured this as a 4.4×
        // undercount of chains; §18.3 is the fix.
        let start_of_stmt = p;
        p = probe;
        let rhs = match parse_expr(seg, &mut p, 0x32) {
            Ok(r) => r,
            Err(b) => return Err(mcall::reanchor_chain(seg, start_of_stmt, probe, b)),
        };
        // A store to any memory object (a global, or a file-scope `static`) is a
        // real write with a relocation, and treating it as a register copy silently
        // drops it. Testing *absence* from `.gl` failed exactly there: a
        // `static int sv` is in `.gl` as `$sv`, whose leading `$`
        // `gl_symbol_index` does not accept as an identifier, so the token looked
        // local and `static int sv; int f(int a){ sv = a; return a; }` mis-emitted.
        // Found by probing the de-conflated census, not by a fixture.
        //
        // A local is admitted only on `.sy`'s positive evidence, and only when
        // `.sy` said plain `int`, never address-taken, and bound its block 1:1 to
        // the `.ex` segments. Everything `.sy` cannot vouch for — a global, a
        // file-scope or function-scope static, a qualified or non-`int` local, a
        // local whose address escapes — leaves this list empty and refuses, as
        // before. **Recorded, not raised** — see [`dst_not_formal`].
        if !formals.contains(&dst) && !locals.contains(&dst) && deferred.is_none() {
            deferred = Some(start_of_stmt);
        }
        // The store opcode and the store's TYPE are **two facts and two keys**.
        //
        // They used to be one `||`, and the refusal that came out of it named the
        // *tag* and nothing else: `assign-store-type-0x86` was 8,222 workload
        // functions rendered through [`blk`], which packs no `aux`, so
        // [`crate::func::body::Block::feature`] fell through to its bare
        // `<ctx>-0x<byte>` arm. `0x86` is the **slot width** (4 bytes) and carries
        // no type class at all, so one bucket held every 4-byte non-int-like store
        // there is — pointer, code pointer, `float`, a `pack(4)` `long long` — and
        // named none of them. The sibling `expr-load-type-*` keys have rendered
        // `<tag><kind>` since `docs/GAPS.md` §6, and this site simply never got the
        // same treatment.
        //
        // Split so the type refusal can use [`blk_type`], which packs the whole
        // triple and renders `<tag><kind>` — `assign-store-type-8643` is a pointer
        // store and `assign-store-type-8645` a `float` one, decodable straight out
        // of `docs/IL_TYPE_TAGS.md` §2. A **split**, never a merge: the missing-`32`
        // case gets its own `assign-store-op` key rather than being folded in, since
        // a body with no store opcode at all is a different fact from one whose
        // store type is unmodeled, and merging buckets is the one failure a census
        // instrument cannot survive.
        if !eat_byte(seg, &mut p, 0x32) {
            return Err(blk(seg, p, "assign-store-op"));
        }
        if !store_type_gate(seg, &mut p) {
            return Err(blk_type(seg, p, p, "assign-store-type"));
        }
        // `4B` ends an expression statement and discards the yielded value. A
        // body that *uses* it (`x = y = a`) does not have one here and refuses.
        if !eat_byte(seg, &mut p, 0x4B) {
            return Err(blk(seg, p, "assign-stmt-end"));
        }
        let rhs = substitute(&rhs, &env)
            .ok_or(Block::refuse(seg, p, "assign-subst-overflow"))?;
        // Re-assigning shadows the previous definition, which is how a dead store
        // disappears: only the last definition can reach the return.
        env.retain(|(t, _)| *t != dst);
        env.push((dst, rhs));
        if env.len() > MAX_SUBST_OPS {
            return Err(Block::refuse(seg, p, "assign-too-many-locals"));
        }
    }
    eat_scopes(seg, &mut p, &mut depth)?;
    let ret = parse_expr(seg, &mut p, 0x41)?;
    let ret = substitute(&ret, &env)
        .ok_or(Block::refuse(seg, p, "assign-subst-overflow"))?;
    eat_return_plumbing(seg, &mut p, true, depth)?;
    let params = parse_params(seg, lo)?;
    // After substitution every remaining LOAD must be a parameter. Anything else
    // is a read of something this class cannot account for — an uninitialized
    // local, a global, or a token from a construct not modeled here.
    if !ret.iter().all(|o| match o {
        IlOp::Load(t) => params.contains(t),
        _ => true,
    }) {
        return Err(Block::refuse(seg, p, "assign-ret-nonformal"));
    }
    // Substitution is a *source* of repeated leaves even when the written source
    // has none: `int x = a; x = x + x;` substitutes to `a + a`, which c2 emits as
    // `slwi r3,r3,1`. This gate is what keeps that from being wrong bytes.
    if has_repeated_leaf(&ret) {
        return Err(Block::refuse(seg, p, "assign-repeated-leaf"));
    }
    // The **same** gate the straight-line path applies, at the second site that
    // produces a `StraightLine`. It was missing here, and the census therefore
    // counted bodies `select_text` refuses: `int f(int a){ int x = a; return
    // x * 3; }` substitutes to `a * 3`, censused in class, and the port returned
    // `NotImplemented` — the exact census/gate disagreement
    // `straight_line_out_of_class_ctx` was extracted from codegen to prevent,
    // reintroduced by a second producer that did not consult it. One fact, one
    // locator: the predicate is shared, not copied.
    if let Some(ctx) = straight_line_out_of_class_ctx(&ret, &params) {
        return Err(Block::refuse(seg, p, ctx));
    }
    // …and the **same canonicalization**, which this producer used to skip.
    //
    // Substitution reorders (`int x = b; return x + a;` resolves to `b + a`) and it
    // also *creates* pending-immediate chains the written source does not show
    // (`int x = a+1; return x+b;` resolves to `[a, 1, Add, b, Add]`). Only the
    // pre-canonicalizer fallback checks ran here, so a stream the straight-line
    // producer canonicalizes into `add r11,r3,r4 ; addi r3,r11,1` reached the
    // selector in source order and was refused — the census counted it and the
    // port declined it. One fact, one locator: the decision is shared, not copied.
    let ret = match canonical_chain_for_codegen(&ret, &params) {
        Ok(c) => c,
        // Both pre-canonicalizer refusals keep this producer's published key.
        Err(ChainReject::Order) | Err(ChainReject::Additive) => {
            return Err(Block::refuse(seg, p, "assign-noncanonical-order"))
        }
        Err(ChainReject::Affine) => return Err(Block::refuse(seg, p, "assign-affine-pending-imm")),
        // `lane w-build`, and a SEPARATE key per producer exactly as the three
        // above are: which producer resolved the chain is a fact about where
        // the widening would have to go.
        Err(ChainReject::Alloc) => return Err(Block::refuse(seg, p, "assign-alloc-undetermined")),
    };
    // …and LAST, the destination. Everything above reports first, so this key names
    // a function only when the destination really is the one thing left.
    if let Some(off) = deferred {
        return Err(dst_not_formal(seg, off));
    }
    Ok(BodyShape::StraightLine { params, ops: ret })
}

/// `C2RS_SINK_STORE_TYPE` — **lane `w-667`'s board #667 counterfactual**, and
/// the only thing that reads it.
///
/// # What it is for
///
/// Board **#667**'s key `assign-store-type-8643` is minted at exactly one site,
/// the `32 <TYPE>` gate in [`try_parse_assign_body_detail`], and there was **no
/// instrument for it** — `grep C2RS_ crates/c2-il` finds four sinks and all four
/// are in the *expression* layer. So the "one env var and two scans"
/// counterfactual every recent lane runs could not be run on this row at all
/// until this existed. It is the row's instrument and nothing else.
///
/// | value | the gate becomes | prices |
/// |---|---|---|
/// | unset (default) | [`eat_int_like`] | the shipped parser |
/// | `ptr4` | [`eat_int_like_or_ptr4`] | `-8643` + `-8644`, board #407's counterfactual 1 |
/// | `any` | any well-formed TYPE | the whole `assign-store-type` family, #407's counterfactual 2 |
///
/// # It is NOT a proposal, and the reason is specific
///
/// Board **#661** is the standing lesson that a sink can quietly *accept*
/// differently as well as measure: `C2RS_SINK_OFF_ADD_ARG`'s `0x27` arm ends
/// `ops.push(IlOp::Add)` with no poison arm, and `cargo test --workspace` is red
/// under it. This one has the same hazard in a different place. The value whose
/// store is folded away here is a **pointer**, and the destination gate that
/// decides whether folding is legal ([`dst_not_formal`]) admits an automatic
/// local only on `.sy`'s positive evidence that it is a plain, unqualified,
/// never-address-taken `int` — so a pointer destination that got past this gate
/// would be refused by that one, *if the body reaches it*. Nothing here
/// guarantees the body does. Every number this sink produces is quoted beside
/// its own `mismatch` count for that reason, and nothing promotes it to a
/// default.
///
/// OFF and free on every gate lane, every mode lane, the sweep, the mode cross
/// and every default scan: the `OnceLock` resolves to `Int` when the variable is
/// absent, which is the shipped `eat_int_like` call this replaced.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StoreTypeSink {
    /// `eat_int_like` — the shipped gate.
    Int,
    /// `eat_int_like_or_ptr4` — the sibling locator the *operand* gate one
    /// production upstream (`readers::eat_operand_type`) already uses.
    Ptr4,
    /// Any well-formed TYPE.
    Any,
}

fn store_type_sink() -> StoreTypeSink {
    // PROV[N] not load-bearing — a `OnceLock` measurement sink, the same contract as `expr.rs`'s five.
    static ON: std::sync::OnceLock<StoreTypeSink> = std::sync::OnceLock::new();
    *ON.get_or_init(|| match std::env::var("C2RS_SINK_STORE_TYPE").as_deref() {
        Ok("ptr4") => StoreTypeSink::Ptr4,
        Ok("any") => StoreTypeSink::Any,
        // An unrecognized spelling is the shipped gate, never a silent widening:
        // a typo'd sink that quietly measured something else is the failure mode
        // `docs/STATUS.md` trap 5 is about.
        _ => StoreTypeSink::Int,
    })
}

/// The store TYPE gate. Identical to the `eat_int_like` call it replaced unless
/// `C2RS_SINK_STORE_TYPE` names one of the two counterfactual arms.
fn store_type_gate(seg: &[u8], p: &mut usize) -> bool {
    store_type_gate_with(store_type_sink(), seg, p)
}

/// The gate with the arm passed in rather than read from the environment.
///
/// Split out **so the arms can be graded against each other**. The sink resolves
/// through a process-global `OnceLock`, so a test cannot exercise two arms in one
/// process — and the property that matters most about a counterfactual instrument
/// is a relation *between* arms, not a fact about any one of them:
/// `Int ⊆ Ptr4 ⊆ Any`. An arm that refused something the default accepts would
/// make the row's measured worth an underestimate in a direction nobody would
/// look, since the only reported number is the recovery and a narrower arm can
/// only lower it. `the_sink_arms_are_nested` is that check.
fn store_type_gate_with(sink: StoreTypeSink, seg: &[u8], p: &mut usize) -> bool {
    match sink {
        StoreTypeSink::Int => eat_int_like(seg, p),
        StoreTypeSink::Ptr4 => eat_int_like_or_ptr4(seg, p).is_some(),
        StoreTypeSink::Any => match read_type(seg, *p) {
            Some((_, _, _, w)) => {
                *p += w;
                true
            }
            None => false,
        },
    }
}

/// The destination refusal, raised at the `26` push it is about — **after** the
/// whole body has had its say (WAE).
///
/// # Why it is deferred, and what the immediate version measured
///
/// The gate itself is right and does not move: a destination this class cannot
/// prove register-resident must not have its store folded away. What moved is
/// *when it speaks*. Refused on the spot, it fired on the FIRST assignment
/// statement — before the remaining statements, the returned expression, the
/// return plumbing and the four post-substitution gates had been parsed at all —
/// so it named every body it stopped, whatever that body's real problem was.
///
/// MEASURED on the 878-TU dc3 workload (2,462,571 functions), by deferring exactly
/// as this function now does and reading the redistribution:
///
/// ```text
///   assign-dst-not-formal      13,887  ->  0        census 691,744 unchanged, +0
///     8,221  assign-store-type-0x86     the very next line: a 4-byte non-integer
///     1,906  assign-store-type-0x82     a 1-byte store
///     1,364  expr-jump                  a goto / break / loop exit
///       830  expr-op-0x27
///       789  expr-call-in-expr-recv-load-then-call-op-0x64
///       468  expr-call-in-expr-recv-load-then-call-data-addr-and-deref-load-more
///       232  expr-call-in-expr-recv-intrinsic-this-adjust-then-intrinsic-call
///        77  … 20 further keys
/// ```
///
/// **Not one function in the workload has the destination as its only blocker.**
/// Lifting the gate entirely converts 0; lifting it *and* the store-type check
/// below it converts 0 as well (the 8,221 then land on `expr-jump` / `expr-op-0x60`
/// / `expr-op-0x10`). The row was the 20th-largest on the board and its realizable
/// worth is exactly zero — `docs/rungs/2026-07-31-assign-eof.md` is the write-up.
///
/// This is the same discipline the formals-marker check at the head of
/// [`try_parse_assign_body_detail`] already follows, and for the same stated
/// reason: an outer gate that refuses early makes the census name *it* instead of
/// the innermost unmodeled construct, and the census's histogram IS the widening
/// order.
///
/// # The byte, and the `:eof` that was not one
///
/// The block carries `byte: Some(0x26)` at the destination push. The previous
/// version passed `byte: None` at a mid-segment offset, and
/// [`crate::func::body::Block::feature`] renders any `byte: None` block as
/// `<ctx>:eof` — so the key read `assign-dst-not-formal:eof` on a refusal that had
/// not reached the end of anything. That suffix is load-bearing when it is true
/// (it means the parse ran out of segment, so nothing can be hiding behind the
/// refusal) and this key claimed it falsely: 4,466 of the 13,887 rows were
/// `cflow-loop` bodies. A rung was ranked and scheduled on the strength of it.
/// `0x26` is the opcode the gate is actually about, it is one key rather than a
/// shard per right-hand side, and `hex[hex_mark]` now points at the push.
fn dst_not_formal(seg: &[u8], off: usize) -> Block {
    Block { ctx: "assign-dst-not-formal", byte: Some(0x26), off, seg_len: seg.len(), aux: 0 }
}


#[cfg(test)]
mod tests {
    use crate::func::body::parse_segment_detail;
    use crate::func::test_fixtures::{free_fn, NO_LOCALS};

    /// One assignment-statement body, parameterised on the operand LOAD's TYPE
    /// and on the STORE's TYPE **separately**.
    ///
    /// Separately because they are two positions with two gates, and the first
    /// draft of these tests conflated them — it varied one type and wrote it into
    /// both slots, which cannot distinguish "the store type is unmodeled" from
    /// "the operand type is". It asserted the store key for a `float` and got
    /// `expr-load-type-8645`; see the second test, which is what that refutation
    /// turned into.
    ///
    /// The skeleton is transcribed from a live capture rather than invented: lane
    /// w-dclass/C compiled `T f(T q, int k) { T x; x = q; return k; }` at the dc3
    /// workload's own `/O1 /Oi /EHsc /GR` profile and read the segment back
    /// through `c2rs census`. Everything but the two TYPE runs is byte-identical
    /// across every case below.
    fn assign_body(load_ty: &[u8], store_ty: &[u8]) -> Vec<u8> {
        let mut v = vec![
            0x46, 0x2D, 0xF4, 0x09, 0x2D, 0xF5, 0x09, // formals q, k
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0x26, 0xF8, 0x09, // push dst x
            0xB9, 0xF4, 0x09, // LOAD q …
        ];
        v.extend_from_slice(load_ty); //   … at the operand type
        v.push(0x32); // STORE …
        v.extend_from_slice(store_ty); //   … at the destination's type
        v.extend_from_slice(&[
            0x4B, // discard the yielded value
            0xB9, 0xF5, 0x09, 0x86, 0x41, 0x74, // LOAD k (int)
            0x41, 0x86, 0x41, 0x74, // result-type int
            0x3A, 0xF7, 0x09, 0x54, 0x02, 0x29, 0xF7, 0x09, // return
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // separator
            0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D, // module end
        ]);
        v
    }

    /// The census key this body refuses with. Every case here refuses — the
    /// destination is a bare token no `.sy` vouches for — so the interesting
    /// question is always *which* key, i.e. which gate spoke first.
    fn key(load_ty: &[u8], store_ty: &[u8]) -> String {
        parse_segment_detail(&free_fn(&assign_body(load_ty, store_ty)), NO_LOCALS)
            .expect_err("this body stores into a non-formal and must refuse")
            .feature()
    }

    /// A 4-byte integer operand, so the LOAD gate is satisfied and the STORE type
    /// is the only thing under test.
    const INT: &[u8] = &[0x86, 0x41, 0x74];

    /// **The counterfactual sink must be OFF in every test process**, exactly as
    /// `expr.rs`'s `C2RS_SINK_CHAIN` tripwire requires of its own.
    ///
    /// Without this, a shell that exported `C2RS_SINK_STORE_TYPE` for a scan and
    /// then ran `cargo test` in the same session would grade a **different
    /// parser** and every assertion in this module would be about a widening
    /// nobody shipped. The keys below are the ones the sink deletes, so the
    /// failure would be loud — but it would be loud in the wrong file, and
    /// `docs/STATUS.md` trap 5 is that absence reads as success unless something
    /// forbids it.
    #[test]
    fn the_store_type_sink_is_off_in_the_test_process() {
        assert!(
            std::env::var("C2RS_SINK_STORE_TYPE").is_err(),
            "the test process must not set C2RS_SINK_STORE_TYPE"
        );
        assert_eq!(super::store_type_sink(), super::StoreTypeSink::Int);
    }

    /// **The sink's default arm is the byte the shipped gate was, not merely a
    /// gate that agrees on the workload.**
    ///
    /// `store_type_gate` replaced a direct `eat_int_like` call, so the thing to
    /// pin is that the *default* dispatch is that same call — over both the
    /// accept side and the refuse side, at the exact type triples the census key
    /// is built from. A regression here would silently re-price board #667's
    /// entire row and no gate would see it: the sink emits nothing.
    #[test]
    fn the_default_arm_of_the_store_type_gate_is_the_shipped_eat_int_like() {
        use crate::func::readers::eat_int_like;
        for ty in [
            &[0x86u8, 0x41, 0x74][..],     // int — accepted
            &[0x86, 0x42, 0x75][..],       // unsigned — accepted
            &[0xA6, 0x41, 0x84, 0x20][..], // const int, per-TU id — accepted
            &[0x86, 0x43, 0xF4, 0x08][..], // int * — REFUSED, and this is #667's key
            &[0x86, 0x44, 0x88, 0x20][..], // a code pointer — REFUSED
            &[0x82, 0x12, 0x30][..],       // bool — REFUSED
            &[0x88, 0x85, 0x41][..],       // double — REFUSED
        ] {
            let (mut a, mut b) = (0usize, 0usize);
            let want = eat_int_like(ty, &mut a);
            assert_eq!(super::store_type_gate(ty, &mut b), want, "{ty:02X?}");
            assert_eq!(a, b, "cursor disagrees on {ty:02X?}");
        }
    }

    /// **The three sink arms are NESTED: `Int ⊆ Ptr4 ⊆ Any`, on both the verdict
    /// and the cursor.**
    ///
    /// This is the property that makes the counterfactual's headline number
    /// readable, and it is not obvious from the code: `eat_int_like` opens with
    /// an exact-triple whitelist (`INT_LIKE_TYPES`) *before* it consults
    /// `read_type`, so a triple on that list which `read_type` could not frame
    /// would be accepted by the **default** arm and refused by the **widest**
    /// one. The recovery this lane reports is the only number the arms produce,
    /// and a narrower wide-arm can only push it down — so the failure would read
    /// as a cleaner decline, which is the direction nobody re-checks.
    ///
    /// Graded over both sides of the boundary, plus the two malformed inputs a
    /// gate at a segment end really meets. **The first four cases are
    /// `INT_LIKE_TYPES` entire** — `INT_TYPE`, `UINT_TYPE`, `LONG_TYPE`,
    /// `ULONG_TYPE`, all four of them — so the whitelist hazard above is covered
    /// exhaustively rather than by sampling.
    #[test]
    fn the_sink_arms_are_nested() {
        use super::StoreTypeSink::{Any, Int, Ptr4};
        let cases: &[&[u8]] = &[
            &[0x86, 0x41, 0x74],           // int
            &[0x86, 0x42, 0x75],           // unsigned
            &[0x86, 0x41, 0x12],           // long
            &[0x86, 0x42, 0x22],           // unsigned long
            &[0xA6, 0x41, 0x84, 0x20],     // const int, per-TU id
            &[0x86, 0x43, 0xF4, 0x08],     // int *   — #667's key
            &[0x86, 0x44, 0x88, 0x20],     // code pointer
            &[0x96, 0x43, 0xF4, 0x08],     // int * volatile
            &[0x82, 0x12, 0x30],           // bool
            &[0x84, 0x21, 0x11],           // short
            &[0x88, 0x85, 0x41],           // double
            &[0x88, 0x81, 0x13],           // long long
            &[],                           // ran off the end
            &[0x00],                       // not a TYPE at all
        ];
        for ty in cases {
            let (mut a, mut b, mut c) = (0usize, 0usize, 0usize);
            let (i, p4, an) = (
                super::store_type_gate_with(Int, ty, &mut a),
                super::store_type_gate_with(Ptr4, ty, &mut b),
                super::store_type_gate_with(Any, ty, &mut c),
            );
            assert!(!i || p4, "Int accepts and Ptr4 refuses: {ty:02X?}");
            assert!(!p4 || an, "Ptr4 accepts and Any refuses: {ty:02X?}");
            // A wider arm must also agree on where the TYPE ends, or the two
            // scans are parsing from different offsets and their histograms are
            // not comparable.
            if i {
                assert_eq!(a, b, "Int/Ptr4 cursor disagree: {ty:02X?}");
            }
            if p4 {
                assert_eq!(b, c, "Ptr4/Any cursor disagree: {ty:02X?}");
            }
        }
    }

    /// **The store-type census key carries the type's KIND, not only its slot
    /// width.**
    ///
    /// It did not, and that is the whole content of this test. The refusal was
    /// raised through `blk`, which packs no `aux`, so `Block::feature` fell
    /// through to its bare `<ctx>-0x<byte>` arm and printed the *tag* — `0x86`
    /// is "this slot is 4 bytes wide" and says nothing whatever about the type.
    /// One bucket of 8,222 workload functions therefore held every 4-byte
    /// non-int-like store there is and named none of them.
    ///
    /// The kinds below are what that bucket actually contained, established by
    /// capture and not by reading the numbers off their neighbours: `86 43` a
    /// data pointer (6,820 of the 8,222) and `86 44` a code pointer (1,402),
    /// which sum to the whole bucket exactly with no third kind and no remainder.
    /// `docs/IL_TYPE_TAGS.md` §2 is the decode.
    #[test]
    fn store_type_key_names_the_kind_and_not_just_the_slot_width() {
        // The two the workload has, from `int *x; x = q;` and `FnPtr x; x = q;`.
        assert_eq!(key(INT, &[0x86, 0x43, 0xF4, 0x08]), "assign-store-type-8643");
        assert_eq!(key(INT, &[0x86, 0x44, 0x88, 0x20]), "assign-store-type-8644");
        // The 1-byte bucket, likewise: `82 12` is the bool/`unsigned char` class,
        // and it too used to print only its tag (`assign-store-type-0x82`).
        assert_eq!(key(INT, &[0x82, 0x12, 0x30]), "assign-store-type-8212");
        // Kinds the workload census reports as ZERO. They are spelled out HERE
        // because a zero count is only evidence when the key it counts can be
        // produced at all — absence read as success is this project's
        // most-repeated defect. Each is a distinct key, so the workload's zeros
        // are facts about the workload rather than about this renderer.
        assert_eq!(key(INT, &[0x86, 0x45, 0x40]), "assign-store-type-8645"); // float
        assert_eq!(key(INT, &[0x88, 0x85, 0x41]), "assign-store-type-8885"); // double
        assert_eq!(key(INT, &[0x82, 0x11, 0x10]), "assign-store-type-8211"); // signed char
        assert_eq!(key(INT, &[0x84, 0x21, 0x11]), "assign-store-type-8421"); // short
        assert_eq!(key(INT, &[0x84, 0x22, 0x21]), "assign-store-type-8422"); // ushort
        assert_eq!(key(INT, &[0x88, 0x81, 0x13]), "assign-store-type-8881"); // long long
        assert_eq!(key(INT, &[0x96, 0x43, 0xF4, 0x08]), "assign-store-type-9643"); // int* volatile
    }

    /// **Why those zeros are zeros — and it is not that the types do not occur.**
    ///
    /// This is the finding the first draft of these tests was wrong about, kept
    /// as a test rather than as a sentence because it bounds what any widening of
    /// this seam could ever be worth. In a real `T x; x = q;` the operand and the
    /// destination carry the *same* type, and the operand LOAD is parsed FIRST.
    /// `eat_operand_type` admits exactly three classes — 4-byte integer, 4-byte
    /// pointer, and the 1-byte unsigned pair — so every other type is refused one
    /// token before the store gate is reached and lands in an `expr-load-type-*`
    /// bucket instead.
    ///
    /// The consequence is a **closed vocabulary**: `assign-store-type` can only
    /// ever name a type that clears the operand gate and fails `eat_int_like`,
    /// which is the pointer classes and the 1-byte unsigned one — precisely the
    /// three keys the 878-TU workload produces. That census is therefore not a
    /// sample with a tail to discover; it is the whole set.
    #[test]
    fn everything_outside_the_operand_vocabulary_refuses_at_the_load_instead() {
        for (ty, want) in [
            (&[0x86u8, 0x45, 0x40][..], "expr-load-type-8645"), // float
            (&[0x88, 0x85, 0x41][..], "expr-load-type-8885"),   // double
            (&[0x82, 0x11, 0x10][..], "expr-load-type-8211"),   // signed char
            (&[0x84, 0x21, 0x11][..], "expr-load-type-8421"),   // short
            (&[0x84, 0x22, 0x21][..], "expr-load-type-8422"),   // unsigned short
            (&[0x88, 0x81, 0x13][..], "expr-load-type-8881"),   // long long
            // `volatile int` clears neither gate, and for a reason that is a
            // captured instruction rather than a width: a volatile operand is a
            // memory object, so c2 homes it in the frame and reads it back. See
            // `readers::is_volatile_tag`.
            (&[0x96, 0x41, 0x86, 0x20][..], "expr-load-type-9641"),
        ] {
            // Same type in both slots — which is what a real `T x; x = q;` emits.
            assert_eq!(key(ty, ty), want, "{ty:02X?}");
        }
    }

    /// The **accept** side of the same boundary, so the split is graded in both
    /// directions rather than only where it refuses.
    ///
    /// Every int-like spelling `eat_int_like` admits gets past the store TYPE and
    /// is then stopped by the *destination* gate, at a different byte with a
    /// different key. That is the positive statement this test is for:
    /// `assign-store-type` names the TYPE and only the type, so a reader cannot
    /// mistake its 10,128 rows for a destination problem — nor the reverse.
    #[test]
    fn int_like_store_types_pass_the_type_gate_and_stop_at_the_destination() {
        for ty in [
            &[0x86u8, 0x41, 0x74][..],  // int
            &[0x86, 0x42, 0x75][..],    // unsigned
            &[0x86, 0x41, 0x12][..],    // long
            &[0x86, 0x42, 0x22][..],    // unsigned long
            &[0xA6, 0x41, 0x84, 0x20][..], // const int (a per-TU id, not a fixed triple)
        ] {
            assert_eq!(key(INT, ty), "assign-dst-not-formal-0x26", "{ty:02X?}");
        }
    }
}

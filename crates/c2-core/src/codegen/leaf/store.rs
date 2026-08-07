//! The store leaf: `s->m = v;` as one `stb`/`sth`/`stw`/`std` at a folded
//! displacement. The third consumer of the sub-object designator
//! (`c2-il/src/func/body/shapes/designator.rs`); see `docs/IL_STORE_LEAF.md`.

use c2_il::{IlFunction, IlOp, FP_SCRATCH};
use crate::BackendError;
use crate::codegen::encode::{
    encode_blr,
    encode_lbz,
    encode_ld,
    encode_lfs,
    encode_lhz,
    encode_lwz,
    encode_stb,
    encode_std,
    encode_stfs,
    encode_sth,
    encode_stw,
};
use crate::codegen::alloc;
use crate::codegen::order;
use crate::codegen::schedule;
use crate::codegen::select::{ARG_REGS, OptMode, SCRATCH_REG, fits_i16, out_of_class};
use crate::codegen::straightline::emit_load_imm;

// `encode_std` has TWO independent witnesses and exactly one definition, in
// [`crate::codegen::encode`] with every other word encoder: the frame model
// captured it as the callee-saved GPR prologue store (`fbe1fff0` =
// `std r31,-16(r1)`), and this rung captured it as a `long long` member at
// offset 32 (`f8830020`, in `store_leaf_text`'s table below). One function with
// two independent captures beats two functions with one each — and after the
// §2.1 split a second copy would be a compile error in `encode.rs` rather than
// a duplicate 2,000 lines away, which is what happened to `encode_std` once
// already (`docs/ARCHITECTURE_SEAMS.md` §1, class 4).

/// Lower a **store leaf** — `void f(S* s, int v){ s->m = v; }` /
/// `void D::set(int v){ Base::m = v; }` / `void f(S* s){ s->m = 7; }` — to one
/// store instruction + `blr`, or to `li` + store + `blr` when the value is a
/// literal.
///
/// Recognized by an **exact** three-op stream `[Load(base), Load(value) | Lit(k),
/// StoreInd { off, width }]`, which `c2_il::try_parse_store_leaf` is the only
/// producer of. Returns `None` for anything else so the ordinary selector keeps
/// its behaviour unchanged, and the pattern is deliberately not a prefix match:
/// a store whose value is *computed* puts the computation in the scratch
/// register first (`s->m = a + b` is `add r11,r3,r4 ; stw r11,0(r3)`), which is
/// a different shape with no capture behind it here.
///
/// The measured lowering (`work/lf/probes/p1.cpp`, `p3.cpp`, every word read off
/// the reference obj at `/Ox /GS- /c`):
///
/// ```text
///   width 1  stb    s->c = v   (char, off 12)      9883000c
///   width 2  sth    s->s = v   (short, off 14)     b083000e
///   width 4  stw    s->a = v   (int, off 0)        90830000
///   width 8  std    s->q = v   (long long, off 32) f8830020   DS-form
///   literal         s->a = 7                       39600007 91630000   li r11,7 ; stw r11
///   literal         s->f = true  (bool)            39600001 99630000   li r11,1 ; stb r11
///   two regs        f(int x,S* s,int v){s->b=v;}   90a40004            stw r5,4(r4)
/// ```
///
/// **The literal goes through the scratch register r11, never r3.** That is the
/// same r11 rule [`indirect_load_text`] follows for a load feeding an extension,
/// and it is read off the capture rather than assumed — a `void` function's r3
/// holds nothing the ABI cares about, so `li r3,7` would have been just as
/// plausible and is not what c2 emits.
///
/// `func.params` maps both tokens to their incoming argument registers by
/// declaration order, with a member function's `this` already at index 0.
/// One statement of a **value-simple GPR store run**: a store whose value is
/// either a formal already live in a register or a literal this run has to
/// materialise. The two cases are exactly [`schedule::Stmt`]'s `producer:
/// None` / `Some(id)`, which is why this is the unit the models speak about.
struct SimpleStore {
    /// The base **SYMBOL** — what [`schedule::Stmt::base`] keys may-alias on,
    /// and what the overlap gate keys dead-store elimination on. For a bound
    /// reference (board **#1199**) this is the BOUND LOCAL's own token, never
    /// the formal it hangs off: that is the whole of board #1128, and it and
    /// `base_reg` are two derivations of one [`IlOp::BoundAddr`], so they
    /// cannot disagree.
    base_tok: u32,
    base_reg: u8,
    /// The store's effective displacement. **The one site the binding is
    /// discharged**: for a bound base this is `bind.off + <the store's own
    /// offset>`, summed here and nowhere else, so the offset the IL carries
    /// cannot be added twice.
    off: i32,
    width: u8,
    /// `Some(k)` — a literal, produced by an `li`/`lis`+`ori` this run emits;
    /// `None` — a formal, already live in `src`.
    lit: Option<i32>,
    /// The register the value comes out of. For a literal this is filled in
    /// from [`alloc::allocate`] after the whole run is parsed.
    src: u8,
    /// **Board #1199's backstop.** The stored VALUE is a bound reference — an
    /// interior address, which materialises one `addi` and is therefore a
    /// *register-derived producer*. `c2_il`'s `bind_run_ops` refuses this in the
    /// reader, under two keys so the mixed-kind half (boards #836/#868, and
    /// `xboxheap.cpp`'s own blocker) stays separately sizeable. Restated here
    /// because a parser that widened past its witness must come out as a gap and
    /// not as bytes.
    value_bound: bool,
    /// **THE CARRIER'S LVALUE HALF** — [`alloc::ProducerRoots`], board #1231.
    /// The root of the designator this store is written through: `base_tok`
    /// again, plus the one bit saying whether that root is a bind head.
    ///
    /// `base_tok` alone has been in this struct since #1199 and it is *not* the
    /// fact. The fact is a RELATION between this root and the value's, and until
    /// there was somewhere to put the value's root the relation could not be
    /// written down at all. Nine allocation keys died of that.
    lvalue_root: alloc::Root,
    /// **THE CARRIER'S VALUE HALF.** `None` for a literal, which has no
    /// designator and therefore no root — never a fabricated one.
    ///
    /// **This is the token the `IlOp::BoundAddr { .. }` arm below used to
    /// discard.** Both roots have been live at this one site since #1199; the
    /// value's was dropped by a `..` pattern, because [`alloc::Producer`] had no
    /// field that could receive it.
    value_root: Option<alloc::Root>,
}

/// The offsets a root carries here are **`None`, and that is board #908**.
///
/// `IlOp::BoundAddr`'s `off` is the SUM of the offset-add literals
/// (`c2_il`'s `eat_offset_adds`), and `IlOp::Load` carries no chain at all.
/// `c2_il::…::designator::eat_offset_adds_list` returns the list, but the seam
/// that would carry it this far is `IlOp::BoundAddr` itself — matched and
/// constructed in this file and in `super::super::store_run_call`, the call-tail
/// emitter path. So the list stops at the reader, and the gap is **one named
/// field** rather than an unmeasurable absence.
///
/// A one-element list holding the sum would be a lie in exactly the shape #908
/// warns about, so it is not written.
/// `is_bind` and `base` are passed separately and NOT inferred from each other.
/// They coincide at this seam — every bind here comes from an `IlOp::BoundAddr`
/// and every non-bind from an `IlOp::Load` — and that coincidence is a property
/// of this emitter, not of the IL: GRID P's `P9` (`PTRBIND`) is a root that
/// takes the bonus while `work/w-prod/bindbit.out` cannot show it is a `26`
/// bind head. Encoding `is_bind == base.is_some()` would make that cell
/// unrepresentable before anyone has decided what it is.
fn root(tok: u32, is_bind: bool, base: Option<u32>) -> alloc::Root {
    alloc::Root { tok, is_bind, base, offsets: None }
}

/// [`alloc::ProducerRoots`] for the producer whose statements are those of `run`
/// carrying literal `id`, or `None` when the pair cannot be stated.
///
/// **Board #1231.** The lvalue half is the root of the designator *this
/// producer's own stores* are written through — not the run's base, because a
/// run may store through several. So the pair is refused, rather than guessed,
/// when this producer's stores do **not** agree on one root:
///
/// * the value has no designator (a literal) — there is no value root;
/// * this producer's stores go through more than one root token, or disagree on
///   the bind bit — then *"the designator its own stores are written through"*
///   names no single thing, and `w-self2b`'s predicate is not defined.
///
/// A refusal here costs nothing: [`alloc::allocate`] does not read this field.
/// A guess would cost the next lane its grid.
fn producer_roots(run: &[SimpleStore], id: u32) -> Option<alloc::ProducerRoots> {
    let mut mine = run.iter().filter(|s| s.lit.map(|k| k as u32) == Some(id));
    let first = mine.next()?;
    let value = first.value_root.clone()?;
    let lvalue = first.lvalue_root.clone();
    for s in mine {
        if s.value_root.as_ref() != Some(&value) || s.lvalue_root != lvalue {
            return None;
        }
    }
    Some(alloc::ProducerRoots { value, lvalue })
}

/// Parse the whole `ops` stream as value-simple GPR groups, or `None`.
///
/// **Whole stream, not a prefix.** A residue that is not such a group — a
/// load-valued store, an FP store, anything else — makes this decline entirely
/// so the walk in [`store_leaf_text`] keeps its behaviour. `GAPS.md` §6: the
/// empty prefix matches everything, and the empty case is never the one the
/// rung is about, so the emptiness check is the caller's and precedes this.
fn parse_simple_gpr_run(
    ops: &[IlOp],
    reg_of: &dyn Fn(u32) -> Option<u8>,
) -> Option<Vec<SimpleStore>> {
    let mut out: Vec<SimpleStore> = Vec::new();
    let mut walk = ops;
    while let [b, v, IlOp::StoreInd { off, width }, tail @ ..] = walk {
        // **Board #1199 — the base position, and the ONE place the binding is
        // discharged.** A bound reference contributes its own token as the base
        // SYMBOL and the formal's register plus the bound object's offset as the
        // ADDRESS; both come out of the same `BoundAddr`, which is what makes
        // collapsing the two source spellings unspellable rather than merely
        // unreached. An unbound `Load` is `(its token, its register, 0)`, which
        // is exactly what this loop always computed.
        let (base_tok, base_reg, base_off, base_bind_of) = match b {
            IlOp::Load(t) => (*t, reg_of(*t)?, 0i32, None),
            IlOp::BoundAddr { tok, base, off } => (*tok, reg_of(*base)?, *off, Some(*base)),
            // Not a store group at all — leave `walk` non-empty so the caller's
            // whole-stream check declines, exactly as the narrower pattern this
            // loop used to open with did.
            _ => break,
        };
        // **The value's ROOT is kept here** — board #1231. The third arm used to
        // read `IlOp::BoundAddr { .. }` and throw the token away; both roots
        // have been live at this one site since #1199, and the relation between
        // them is the fact nine allocation keys could not state.
        let (lit, src, value_bound, value_root) = match v {
            IlOp::Load(t) => (None, reg_of(*t)?, false, Some(root(*t, false, None))),
            // A literal has no designator, so it has no root. `None`, never a
            // fabricated one.
            IlOp::Lit(k) => (Some(*k), SCRATCH_REG, false, None),
            IlOp::BoundAddr { tok, base, .. } => {
                (None, SCRATCH_REG, true, Some(root(*tok, true, Some(*base))))
            }
            _ => break,
        };
        out.push(SimpleStore {
            base_tok,
            base_reg,
            off: base_off.checked_add(*off)?,
            width: *width,
            lit,
            src,
            value_bound,
            lvalue_root: root(base_tok, base_bind_of.is_some(), base_bind_of),
            value_root,
        });
        walk = tail;
    }
    if walk.is_empty() && !out.is_empty() {
        Some(out)
    } else {
        None
    }
}

/// **The SCHEDULED store run** — the one place under `crates/` where
/// [`order::schedule`] decides emitted order, and [`alloc::allocate`] decides
/// the register.
///
/// Six lanes modelled this floor and every one shipped as a guard that
/// *refused*: `leaf_store` emitted source order and asked
/// [`order::producers_lead`] only for permission to decline. This function is
/// the wiring. `None` means "not this shape, walk it the old way"; `Some(Err)`
/// is an honest refusal.
///
/// # The rule, and where each half comes from
///
/// * **Which store goes where** — [`order::store_order`], `docs/ORDER.md`:
///   rank the distinct producers by *(use count descending, first-use
///   ascending)*, let `u = min(2, #unproduced)`, and a store whose producer has
///   rank `j` may not occupy position `< u + j`. 561/561 holdout.
/// * **Which producer is emitted first** — [`order::producer_order`], #582.
/// * **Where the producers sit among the stores** — [`order::layout_slots`],
///   #602: producer `i` immediately before store slot `min(i, u)`, with `u` the
///   *leading run* of unproduced stores in the FINAL order (#584, not
///   [`order::head_slots`]). 24,891/24,891 holdout, gated on `nsw ≤ 2`.
/// * **Which register each producer takes** — [`alloc::allocate`],
///   `docs/ALLOC.md`: use count descending, constants tying by **reverse**
///   source order, handed `r11`, `r10`, `r9` … descending. 250/250 holdout.
///
/// [`order::schedule`] composes the first three; this function composes that
/// with the fourth.
///
/// # This is additive-REFUSAL, and the distinction is not blurred
///
/// Every reading here is a **refusal** when a model answers `None`: the run is
/// outside the region the model is exact on, so it is declined rather than
/// answered at 98.6 %. Board **#621** measured a rival layout clause that
/// answers the *whole* population at 99.44 % fit / 97.30 % holdout and
/// deliberately did not ship it — 99 % is a rule with a residual, and an
/// emitter fed a 99 % layout emits wrong bytes on the other 1 %.
///
/// **This function cannot accept anything the parser did not already hand it.**
/// A widening of what *reaches* here is the parser's, is additive-ACCEPT, and
/// is stated as such where it lives (`c2_il`'s `try_parse_store_run`).
///
/// # `/O1` and `/Ox`
///
/// Takes no `OptMode`, and that is measured rather than assumed. Every grid
/// behind the four models was compiled at the WORKLOAD's `/O1 /Oi /EHsc`; the
/// fixture gate runs `/Ox`. `work/w-wire/mode_probe.py` compares the two modes'
/// emitted permutations to each other over 18 cases spanning both killer-cell
/// families, the interleaved layouts and the pool boundary: **18 of 18 the
/// same**. Board **#641**.
fn scheduled_gpr_run_text(func: &IlFunction) -> Option<Result<Vec<u8>, BackendError>> {
    match scheduled_gpr_run(&func.params, &func.ops)? {
        Err(e) => Some(Err(e)),
        Ok(run) => {
            let mut text: Vec<u8> = run.slots.into_iter().flat_map(|(_, w)| w).collect();
            text.extend_from_slice(&encode_blr());
            Some(Ok(text))
        }
    }
}

/// A scheduled store run, **slot by slot** rather than as one flat byte string.
///
/// [`scheduled_gpr_run_text`] concatenates this and appends the leaf's `blr`.
/// Board **#844**'s composition seam needs the same run as the *middle* of a
/// framed body with one word spliced into it, and a splice needs to know where
/// the seams are — so the emission is produced once, in slot form, and the two
/// consumers differ only in what they do with it. A second scheduler for the
/// same run is `GAPS.md` §6's "one fact, two locators", and this file's own
/// history (`store_leaf_text` walking source order beside `order::schedule`) is
/// what that costs.
pub(crate) struct ScheduledRun {
    /// One entry per emitted slot in emission order: `(this slot is a STORE,
    /// its words)`. Producers are `false`.
    pub(crate) slots: Vec<(bool, Vec<u8>)>,
    /// Distinct producers — equal literals CSE to one `li`, so equal `k` is one
    /// producer, which is the identity [`alloc::allocate`] is handed.
    pub(crate) nprod: usize,
    /// Stores that materialise **nothing** — a formal already live in a
    /// register. `w-seam`/#867's `u`.
    pub(crate) nsw: usize,
}

/// The scheduled store run, or `None` for a stream that is not one.
///
/// Split out of [`scheduled_gpr_run_text`] as a pure code move so that the leaf
/// and the #844 composition ask **one** scheduler. Every model consulted here —
/// [`order::schedule`], [`alloc::allocate`] and the three refusals above them —
/// is the leaf's, unchanged and unrelaxed: this function cannot accept anything
/// `store_leaf_text` did not already accept, and a composition that wanted more
/// would have to widen the models rather than route around them.
pub(crate) fn scheduled_gpr_run(
    params: &[u32],
    ops: &[IlOp],
) -> Option<Result<ScheduledRun, BackendError>> {
    let reg_of = |tok: u32| -> Option<u8> {
        params
            .iter()
            .position(|&t| t == tok)
            .filter(|&i| i < ARG_REGS.len())
            .map(|i| ARG_REGS[i])
    };
    let mut run = parse_simple_gpr_run(ops, &reg_of)?;

    // **Board #1199's backstop, and it is the frontier's last refusal.** A bound
    // reference in the stored-VALUE position is an interior address: one
    // `addi rD,rBase,off`, a **register-derived** producer. Beside a literal that
    // is the mixed-kind run `alloc::allocate` refuses wholesale (board #836:
    // clause 1 alone wrong on 29 of 81, this refusal wrong on 0), whose narrow
    // lift is refuted (#868: `addi`-interior 12/12, `slwi` 0/12) and whose
    // clause 1 is refuted on this very mix (#1134's `j1_lit2`).
    // `src/xdk/nuispeech/xboxheap.cpp` is exactly this shape.
    //
    // `c2_il`'s `bind_run_ops` refuses it in the READER, under two keys so the
    // mixed half stays separately sizeable — that is where acceptance lives, and
    // `census_gate.rs` is the invariant. This is the second lock: a parser that
    // widened past its witness comes out as a gap, not as bytes.
    if run.iter().any(|s| s.value_bound) {
        return Some(Err(out_of_class(
            "a store run whose value is a bound reference: an interior address is \
             a register-derived producer, and beside a literal that is the \
             mixed-kind run codegen::alloc refuses (boards #836/#868/#1134)",
        )));
    }

    // Displacement and width are checked BEFORE any model is consulted, so an
    // unencodable store refuses on its own terms rather than through a `None`
    // that would read as "outside the schedule's domain".
    for s in &run {
        if i16::try_from(s.off).is_err() {
            return Some(Err(out_of_class(
                "store offset exceeds a 16-bit displacement",
            )));
        }
        match s.width {
            1 | 2 | 4 => {}
            // `std` is DS-form: an offset that is not a multiple of 4 cannot be
            // encoded at all.
            8 if s.off % 4 == 0 => {}
            8 => {
                return Some(Err(out_of_class(
                    "8-byte store whose offset is not a multiple of 4 (std is DS-form)",
                )))
            }
            _ => return Some(Err(out_of_class("store of an unmodeled width"))),
        }
    }

    // **No two stores may write overlapping bytes of the same base object.**
    // c2 eliminates the dead one — `{ s->a=u; s->a=w; }` is a *single*
    // `stw r5,0(r3)` — so emitting both is wrong bytes. Keyed on the base
    // TOKEN, because two different tokens may alias at run time and c2 keeps
    // both stores. Checked here rather than after emission: the schedule may
    // reorder the pair, and a check that ran over the emitted order would be
    // asking a different question than the parser's.
    for (i, a) in run.iter().enumerate() {
        for b in &run[i + 1..] {
            if a.base_tok == b.base_tok
                && a.off < b.off + i32::from(b.width)
                && b.off < a.off + i32::from(a.width)
            {
                return Some(Err(out_of_class(
                    "two stores overlapping the same base object",
                )));
            }
        }
    }

    let stmts: Vec<schedule::Stmt> = run
        .iter()
        .map(|s| schedule::Stmt {
            // Equal constants are CSE'd to one `li`, so equal `k` is one id.
            // `as u32` is a bijection on `i32`, so distinct literals stay
            // distinct.
            producer: s.lit.map(|k| k as u32),
            base: s.base_tok,
        })
        .collect();

    let Some(slots) = order::schedule(&stmts) else {
        return Some(Err(out_of_class(
            "store run outside the schedule's domain (codegen::order)",
        )));
    };

    // The register per producer. A run with NO producer needs none —
    // `alloc::allocate` refuses an empty list by contract, and asking it would
    // turn an all-formal run into a refusal.
    let mut producers: Vec<alloc::Producer> = Vec::new();
    for (i, s) in stmts.iter().enumerate() {
        if let Some(id) = s.producer {
            match producers.iter_mut().find(|p| p.id == id) {
                Some(p) => p.uses += 1,
                None => producers.push(alloc::Producer {
                    id,
                    // Every producer that reaches here is a literal, so
                    // `li`/`lis`+`ori` — it reads no register.
                    kind: alloc::ProducerKind::Constant,
                    uses: 1,
                    first: i,
                    // **THE CARRIER**, board #1231 — and today it is `None` at
                    // every producer this emitter builds, which is a fact about
                    // the emitter and not about the carrier.
                    //
                    // `producer_roots` pairs the value's root with the root of
                    // the designator THIS producer's own stores go through. A
                    // literal has no value root, and every producer that reaches
                    // here is a literal (`s.producer` is `s.lit`), so the pair
                    // is refused. Board **#840** / `w-mixed` §5 is the reason
                    // that is not a defect: no register-derived producer reaches
                    // `alloc::allocate` from today's emitter at all, because a
                    // bind-valued store is refused upstream as `value_bound`.
                    //
                    // The decode itself is exercised on that refused shape by
                    // `the_carrier_decodes_both_roots_of_a_bind_valued_store`,
                    // so the seam is measured rather than assumed inert.
                    roots: producer_roots(&run, id),
                }),
            }
        }
    }
    // **A MULTI-WORD literal may not sit beside another producer.** This is a
    // constructed counterexample that FIRED — `docs/rungs/_2026-08-05-w-wire-prereg.md`
    // §4 registered it expecting a non-boundary, and real `c2` refuted that in
    // two independent ways at once (`work/w-wire/boundary_probe.py`, both
    // modes):
    //
    // ```text
    //   { a=100000; b=1; }        lis r11 ; li r10 ; ori r11 ; stw r10,4 ; stw r11,0
    //   { a=100000; b=200000; }   lis r11 ; lis r10 ; ori r11 ; ori r10 ; stw r11,0 ; stw r10,4
    // ```
    //
    // 1. the `lis`/`ori` pair is **SPLIT** — c2 interleaves the halves of two
    //    wide loads, so a producer is not one contiguous instruction and
    //    `layout_slots`, which indexes producers, cannot place it;
    // 2. the first cell's **STORE ORDER is `[1, 0]`**, where `store_order` says
    //    source order. ORDER is fitted on single-word `li` values only, and a
    //    two-word producer is outside the population it was measured on.
    //
    // A run whose ONLY producer is wide is unaffected and stays in class —
    // `{ a=100000; b=100000; }` is `lis ; ori ; stw ; stw`, one live range with
    // nothing to interleave with, and it is a cell the parser already admits.
    // `fits_i16` is `emit_load_imm`'s own predicate, shared rather than
    // restated: the gate has to mean "more than one word", and that is the one
    // place which decides it.
    if producers.len() > 1
        && run.iter().any(|s| matches!(s.lit, Some(k) if !fits_i16(k)))
    {
        return Some(Err(out_of_class(
            "multi-word literal beside another producer (its halves interleave)",
        )));
    }
    if !producers.is_empty() {
        // The pool starts above the live-in formals: `params[0]` is r3, so the
        // first free register is r(3 + len).
        let pool_floor = 3u8.saturating_add(params.len().min(9) as u8);
        let Some(assign) = alloc::allocate(&producers, pool_floor) else {
            return Some(Err(out_of_class(
                "store run outside the allocator's domain (codegen::alloc)",
            )));
        };
        for s in run.iter_mut() {
            if let Some(k) = s.lit {
                s.src = assign
                    .iter()
                    .find(|&&(id, _)| id == k as u32)
                    .map(|&(_, r)| r)
                    // `allocate` returns one pair per distinct producer and the
                    // producers were built from these same statements, so this
                    // is unreachable; refusing beats an index panic.
                    .unwrap_or(0);
                if s.src == 0 {
                    return Some(Err(out_of_class(
                        "store run whose producer took no register (codegen::alloc)",
                    )));
                }
            }
        }
    }

    let mut out: Vec<(bool, Vec<u8>)> = Vec::with_capacity(slots.len());
    for slot in &slots {
        let mut text = Vec::with_capacity(8);
        match *slot {
            schedule::Slot::Producer(id) => {
                // The statement this producer materialises for — any of them,
                // they share the value and (by `allocate`) the register.
                let Some(s) = run.iter().find(|s| s.lit.map(|k| k as u32) == Some(id)) else {
                    return Some(Err(out_of_class(
                        "schedule named a producer no statement consumes",
                    )));
                };
                if let Err(e) = emit_load_imm(&mut text, s.src, s.lit.unwrap_or(0)) {
                    return Some(Err(e));
                }
                out.push((false, text));
            }
            schedule::Slot::Store(k) => {
                let Some(s) = run.get(k) else {
                    return Some(Err(out_of_class("schedule named a store out of range")));
                };
                // Checked above, so these cannot fail; `else` refuses rather
                // than truncating a displacement.
                let Ok(d) = i16::try_from(s.off) else {
                    return Some(Err(out_of_class(
                        "store offset exceeds a 16-bit displacement",
                    )));
                };
                match s.width {
                    1 => text.extend_from_slice(&encode_stb(s.src, s.base_reg, d)),
                    2 => text.extend_from_slice(&encode_sth(s.src, s.base_reg, d)),
                    4 => text.extend_from_slice(&encode_stw(s.src, s.base_reg, d)),
                    8 => text.extend_from_slice(&encode_std(s.src, s.base_reg, d)),
                    _ => return Some(Err(out_of_class("store of an unmodeled width"))),
                }
                out.push((true, text));
            }
        }
    }
    Some(Ok(ScheduledRun {
        slots: out,
        nprod: producers.len(),
        // A store materialises nothing exactly when its value is a formal —
        // `SimpleStore::lit` is `None` — which is `schedule::Stmt`'s own
        // `producer: None`, so the two counts cannot drift apart.
        nsw: run.iter().filter(|s| s.lit.is_none()).count(),
    }))
}

pub fn store_leaf_text(
    func: &IlFunction,
    mode: OptMode,
) -> Option<Result<Vec<u8>, BackendError>> {
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .filter(|&i| i < ARG_REGS.len())
            .map(|i| ARG_REGS[i])
    };
    // The **run** is the general case and the single store is its length-1
    // instance, so there is one walk rather than two — `GAPS.md` §6's "one fact,
    // one locator" in the emitter, matching the parser's own single
    // `parse_store_stmt`. `c2_il::try_parse_store_leaf` and
    // `try_parse_store_run` are the only producers of `StoreInd`/`StoreIndFp`,
    // and both emit exact groups, so a residue that does not match a group is a
    // stream this function must refuse rather than ignore.
    // **At least one group.** The single-store version pattern-matched the whole
    // `ops` slice EXACTLY; a loop matches a *prefix*, and the empty prefix matches
    // everything — so an `ops`-less shape whose data lives in another field
    // (`IlFunction::compare`) walked straight past the loop and came out as a bare
    // `blr`. That is a live wrong-bytes emit created by this rung and caught by the
    // one fixture that puts a compare leaf in the same TU as a store
    // (`w29_fp_contract.cpp`, `Port=Mismatch @ 8`); `w25_store_leaf.cpp` has no
    // compare and was green over it. `GAPS.md` §6: turning an exact match into a
    // prefix match adds the empty case, and the empty case is never the one the
    // rung is about.
    if func.ops.is_empty() {
        return None;
    }
    // **The SCHEDULED path.** When the whole `ops` stream is value-simple GPR
    // groups — every group a `[Load(base), Lit(k) | Load(formal), StoreInd]` —
    // the emitted sequence is `codegen::order`'s and `codegen::alloc`'s, not
    // source order. See [`scheduled_gpr_run_text`]. Anything else (a
    // load-valued group, an FP group, a residue) falls through to the walk
    // below unchanged.
    if let Some(t) = scheduled_gpr_run_text(func) {
        return Some(t);
    }
    let mut rest = func.ops.as_slice();
    let mut text = Vec::with_capacity(16);
    let mut written: Vec<(u32, i32, u8)> = Vec::new();
    // How many store GROUPS have been emitted. **Not `written.len()`**: a
    // load-valued group deliberately records nothing there (see below), so using
    // the overlap list as the "did we match anything" flag would send a run of
    // load-valued stores with a trailing residue back to the ordinary selector
    // as if it were not a store at all — the `GAPS.md` §6 empty-prefix shape
    // that this file has already been bitten by once.
    let mut groups = 0usize;
    // How many LOAD-valued statements have been emitted in each register file.
    // The two files are allocated independently — MEASURED, `work/wsl/probe/p7.cpp`
    // `MX`: `{ d->f0=s->f0; d->d0=s->d0; d->f1=s->f1; d->d1=s->d1; }` is
    // `lfs f0 ; lfd f13 ; lfs f12 ; lfd f11`, one descending FP sequence over both
    // widths, while `fx3` runs an independent `r11 ; r10` beside its `f0`.
    let (mut gpr_loads, mut fp_loads) = (0usize, 0usize);
    // **The scratch register a loaded value lands in, and the ONE place the
    // `/O1` / `/Ox` split is stated for this shape.** MEASURED over
    // `work/wsl/probe/p6.cpp` — runs of 1..8 crossed with 2..6 pointer
    // parameters, both modes, plus pure-`float` and pure-`double` runs of 1..5 —
    // and `p7.cpp` for the boundary:
    //
    // ```text
    //   /O1   every statement    r11          f0
    //   /Ox   statement i        r(11 - i)    f0, then f(14 - j) for j >= 1
    // ```
    //
    // which is the same allocator `docs/OPT_MODE.md` §3.1 already records for
    // arithmetic chains: `/O1` reuses r11 because each intermediate's
    // predecessor is dead, `/Ox` gives every value its own descending register.
    // The parameter count does not enter it — `g0_5` through `g4_5` are
    // byte-identical — until the sequence reaches a register a parameter holds,
    // and **that is where this refuses** (see the bound below).
    let gpr_scratch = |i: usize| -> u8 {
        match mode {
            OptMode::O1 => SCRATCH_REG,
            OptMode::Ox => SCRATCH_REG.saturating_sub(i as u8),
        }
    };
    let fp_scratch = |j: usize| -> u8 {
        match mode {
            OptMode::O1 => FP_SCRATCH,
            OptMode::Ox if j == 0 => FP_SCRATCH,
            OptMode::Ox => 14u8.saturating_sub(j as u8),
        }
    };
    while !rest.is_empty() {
        match rest {
            // The **floating-point** store, `void f(S* s, float v){ s->f = v; }` —
            // one `stfs`/`stfd`. Two ops rather than three: the value's register is
            // already resolved, because the FP argument file is numbered over the FP
            // parameters alone and only the IL layer has the `.sy` view that says which
            // parameters those are ([`c2_il::IlOp::StoreIndFp`]). The base is the ordinary
            // GPR argument, and its index *is* its register number even with FP formals in
            // the list — an FP parameter fills no GPR but still consumes its slot, so the
            // two effects cancel exactly (`docs/ABI_EDGES.md` §2, and the capture
            // `void s_arg2(int x, S* s, float v){ s->f = v; }` → `stfs f1,4(r4)`).
            [IlOp::Load(b), IlOp::StoreIndFp { off, double, src }, tail @ ..] => {
                let d = match i16::try_from(*off) {
                    Ok(d) => d,
                    Err(_) => {
                        return Some(Err(out_of_class(
                            "FP store offset exceeds a 16-bit displacement",
                        )))
                    }
                };
                let Some(base) = reg_of(*b) else {
                    return Some(Err(out_of_class(
                        "FP store whose base is not a register argument",
                    )));
                };
                text.extend_from_slice(&encode_stfs(*double, *src, base, d));
                written.push((*b, *off, if *double { 8 } else { 4 }));
                groups += 1;
                rest = tail;
            }
            // **A store whose VALUE is an indirect load** — `d->a = s->b;`, the
            // body of every hand-written copy constructor and copy assignment.
            // Two instructions through the scratch register and no frame; the
            // widths pick both opcodes independently, and the parser has already
            // required the two TYPEs to be byte-identical, so they agree here by
            // construction. MEASURED (`work/wsl/probe/p1.cpp`, `p2.cpp`):
            //
            // ```text
            //   d->a = s->qb   81640004 91630000   lwz r11,4(r4) ; stw r11,0(r3)
            //   d->c = s->c    89640000 99630000   lbz ; stb
            //   d->h = s->h    a1640002 b1630002   lhz ; sth
            //   d->q = s->q    e9640008 f9630008   ld  ; std      (both DS-form)
            //   d->f = s->f    c0040010 d0030010   lfs f0 ; stfs f0
            //   d->g = s->g    c8040018 d8030018   lfd f0 ; stfd f0
            // ```
            //
            // Matched **ahead** of the three-op formal-valued group and of the
            // two-op FP one: a four-op group opens with the same `Load(base)` and
            // only its third op separates it, so an earlier shorter arm would
            // shadow it. The four-op arms are unambiguous against each other
            // because `LoadInd`/`LoadIndSized` and `LoadIndFp` are disjoint.
            [IlOp::Load(b), IlOp::Load(sb), l @ (IlOp::LoadInd { .. } | IlOp::LoadIndSized { .. } | IlOp::LoadIndFp { .. }), st @ (IlOp::StoreInd { .. } | IlOp::StoreIndFp { .. }), tail @ ..] =>
            {
                let (Some(base), Some(sbase)) = (reg_of(*b), reg_of(*sb)) else {
                    return Some(Err(out_of_class(
                        "load-valued store whose base is not a register argument",
                    )));
                };
                // The LOAD half.
                let (soff, lwidth, lfp) = match l {
                    IlOp::LoadInd { off } => (*off, 4u8, false),
                    IlOp::LoadIndSized { off, width, sext } => {
                        if *sext {
                            // A widening `2C` costs an `extsb` between the two
                            // instructions; the parser refuses it, and refusing it
                            // twice is the census/gate invariant.
                            return Some(Err(out_of_class(
                                "load-valued store whose value is sign-extended",
                            )));
                        }
                        (*off, *width, false)
                    }
                    IlOp::LoadIndFp { off, double } => (*off, if *double { 8 } else { 4 }, true),
                    _ => return None,
                };
                let Ok(sd) = i16::try_from(soff) else {
                    return Some(Err(out_of_class(
                        "load-valued store whose load offset exceeds a 16-bit displacement",
                    )));
                };
                // The scratch register for THIS statement, from the file's own
                // running count. **Refused where the descending sequence would
                // reach a register a parameter holds** — that is where c2 stops
                // being a plain descent and starts skipping live registers and
                // wrapping back to r11 (MEASURED, `work/wsl/probe/p7.cpp`: `L9`
                // is `r11 … r5` then **r11** again, and `P8` — two dead `int`
                // parameters ahead of the two pointers — is `r11 … r7, r4, r3,
                // r11`). Reconstructing that needs a liveness model; the gate is
                // drawn where the evidence is a straight descent, and the parser
                // states the same bound so census and gate cannot disagree.
                let (lreg, sreg) = if lfp {
                    let r = fp_scratch(fp_loads);
                    fp_loads += 1;
                    (r, r)
                } else {
                    let r = gpr_scratch(gpr_loads);
                    if r <= 2 + func.params.len().min(8) as u8 {
                        return Some(Err(out_of_class(
                            "load-valued store run longer than the free scratch descent",
                        )));
                    }
                    gpr_loads += 1;
                    (r, r)
                };
                if lfp {
                    text.extend_from_slice(&encode_lfs(lwidth == 8, lreg, sbase, sd));
                } else {
                    match lwidth {
                        1 => text.extend_from_slice(&encode_lbz(lreg, sbase, sd)),
                        2 => text.extend_from_slice(&encode_lhz(lreg, sbase, sd)),
                        4 => text.extend_from_slice(&encode_lwz(lreg, sbase, sd)),
                        // `ld` is DS-form, exactly as `std` is.
                        8 if sd % 4 == 0 => {
                            text.extend_from_slice(&encode_ld(lreg, sbase, sd))
                        }
                        8 => {
                            return Some(Err(out_of_class(
                                "8-byte load whose offset is not a multiple of 4 (ld is DS-form)",
                            )))
                        }
                        _ => return Some(Err(out_of_class("load of an unmodeled width"))),
                    }
                }
                // The STORE half, out of the same scratch register.
                let (off, width, sfp) = match st {
                    IlOp::StoreInd { off, width } => (*off, *width, false),
                    IlOp::StoreIndFp { off, double, src } => {
                        if *src != FP_SCRATCH {
                            return Some(Err(out_of_class(
                                "load-valued FP store out of an argument register",
                            )));
                        }
                        (*off, if *double { 8 } else { 4 }, true)
                    }
                    _ => return None,
                };
                if sfp != lfp || width != lwidth {
                    return Some(Err(out_of_class(
                        "load-valued store whose two halves disagree on width or register file",
                    )));
                }
                let Ok(d) = i16::try_from(off) else {
                    return Some(Err(out_of_class(
                        "store offset exceeds a 16-bit displacement",
                    )));
                };
                if sfp {
                    text.extend_from_slice(&encode_stfs(width == 8, sreg, base, d));
                } else {
                    match width {
                        1 => text.extend_from_slice(&encode_stb(sreg, base, d)),
                        2 => text.extend_from_slice(&encode_sth(sreg, base, d)),
                        4 => text.extend_from_slice(&encode_stw(sreg, base, d)),
                        8 if d % 4 == 0 => {
                            text.extend_from_slice(&encode_std(sreg, base, d))
                        }
                        8 => {
                            return Some(Err(out_of_class(
                                "8-byte store whose offset is not a multiple of 4 (std is DS-form)",
                            )))
                        }
                        _ => return Some(Err(out_of_class("store of an unmodeled width"))),
                    }
                }
                // **Not recorded in `written`.** The dead-store elimination the
                // overlap check below models does not happen when a load sits
                // between the two stores — MEASURED,
                // `{ d->a = s->a; d->a = s->b; }` emits BOTH stores, because `s`
                // may alias `d` and the first one is observable. Feeding these
                // groups to that check would refuse a shape c2 emits in full; the
                // gate that keeps a *loaded* run safe is the parser's aliasing
                // rule (no object both loaded from and stored to), which is a
                // different fact and lives where it can see the tokens.
                groups += 1;
                rest = tail;
            }
            [IlOp::Load(b), v @ (IlOp::Load(_) | IlOp::Lit(_)), IlOp::StoreInd { off, width }, tail @ ..] =>
            {
                let d = match i16::try_from(*off) {
                    Ok(d) => d,
                    // The parser gates this; if it ever changed, refuse rather than truncate.
                    Err(_) => {
                        return Some(Err(out_of_class(
                            "store offset exceeds a 16-bit displacement",
                        )))
                    }
                };
                let Some(base) = reg_of(*b) else {
                    return Some(Err(out_of_class(
                        "store whose base is not a register argument",
                    )));
                };
                let src = match v {
                    IlOp::Load(t) => match reg_of(*t) {
                        Some(r) => r,
                        None => {
                            return Some(Err(out_of_class(
                                "store whose value is not a register argument",
                            )))
                        }
                    },
                    // **Every literal-valued group now goes through
                    // [`scheduled_gpr_run_text`]**, which owns the whole
                    // materialisation-and-placement question. Reaching this arm
                    // with a `Lit` therefore means the stream is NOT all
                    // value-simple GPR groups — it mixes a literal with a
                    // load-valued or FP group — and c2 hoists the load, sinks
                    // its store past the next statement and gives the literal
                    // its own second scratch register there (MEASURED,
                    // `work/wsl/probe/p1.cpp`: `{ d->a=s->a; d->b=2; }` is
                    // `lwz r11 ; li r10,2 ; stw r10 ; stw r11`). The parser
                    // refuses that mix and so does this.
                    //
                    // The predecessor arm read `rest.len() == 3 &&
                    // text.is_empty()` — "a literal is only lowered for a run
                    // of ONE" — and that condition is **exactly** equivalent to
                    // this refusal now: a one-group all-literal stream is a
                    // value-simple GPR run, so the scheduled path claims it
                    // before the walk ever starts.
                    IlOp::Lit(_) => {
                        return Some(Err(out_of_class(
                            "literal value in a store run the schedule did not claim",
                        )))
                    }
                    _ => return None,
                };
                match width {
                    1 => text.extend_from_slice(&encode_stb(src, base, d)),
                    2 => text.extend_from_slice(&encode_sth(src, base, d)),
                    4 => text.extend_from_slice(&encode_stw(src, base, d)),
                    8 if d % 4 == 0 => text.extend_from_slice(&encode_std(src, base, d)),
                    8 => {
                        return Some(Err(out_of_class(
                            "8-byte store whose offset is not a multiple of 4 (std is DS-form)",
                        )))
                    }
                    _ => return Some(Err(out_of_class("store of an unmodeled width"))),
                }
                written.push((*b, *off, *width));
                groups += 1;
                rest = tail;
            }
            // Not a store stream at all: leave the ordinary selector's behaviour
            // unchanged. Past the first group it IS one, and a residue that does
            // not parse as a group refuses instead of being dropped.
            _ if groups == 0 => return None,
            _ => return Some(Err(out_of_class("store run with an unmodeled residue"))),
        }
    }
    // **No two stores may write overlapping bytes of the same base**, the
    // emitter's copy of the parser's dead-store gate: `{ s->a=u; s->a=w; }` is a
    // *single* `stw r5,0(r3)` in the reference, so emitting both is wrong bytes.
    for (i, a) in written.iter().enumerate() {
        for b in &written[i + 1..] {
            if a.0 == b.0
                && a.1 < b.1 + i32::from(b.2)
                && b.1 < a.1 + i32::from(a.2)
            {
                return Some(Err(out_of_class(
                    "two stores overlapping the same base object",
                )));
            }
        }
    }
    text.extend_from_slice(&encode_blr());
    Some(Ok(text))
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    #[test]
    fn store_leaf_text_is_one_store_and_a_blr() {
        // Every expected word transcribed from the reference obj of
        // `fixtures/cpp/w25_store_leaf.cpp` and `work/lf/probes/p1.cpp`, not
        // derived from the encoding rule.
        let mut f = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?s_b@@YAXPAUS@@H@Z".into(),
            source_path: None,
            params: vec![0xF509, 0xF609],
            ops: vec![
                IlOp::Load(0xF509),
                IlOp::Load(0xF609),
                IlOp::StoreInd { off: 4, width: 4 },
            ],
            tail_call: None,
            framed_call: None,
            call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
            data_sym: None,
        };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,4(r3) ; blr"
        );
        // A ZERO displacement is NOT free here — the store still happens. This is
        // the exact opposite of `addr_leaf_text`, whose zero case emits nothing,
        // and the two shapes share a designator.
        f.ops[2] = IlOp::StoreInd { off: 0, width: 4 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,0(r3) ; blr"
        );
        // The width picks the opcode, and nothing else does.
        f.ops[2] = IlOp::StoreInd { off: 12, width: 1 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..4],
            [0x98, 0x83, 0x00, 0x0C],
            "stb r4,12(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 14, width: 2 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..4],
            [0xB0, 0x83, 0x00, 0x0E],
            "sth r4,14(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 32, width: 8 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..4],
            [0xF8, 0x83, 0x00, 0x20],
            "std r4,32(r3)"
        );
        // `std` is DS-form: an offset that is not a multiple of 4 cannot be
        // encoded at all, so it refuses rather than dropping the low two bits.
        f.ops[2] = IlOp::StoreInd { off: 30, width: 8 };
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_err());
        // BOTH register fields move: `void f(int x, S* s, int v){ s->b = v; }` is
        // `90a40004` — value r5, base r4 — and a lowering that hardcoded either
        // would pass every two-parameter case.
        f.params = vec![0x1111, 0xF509, 0xF609];
        f.ops[2] = IlOp::StoreInd { off: 4, width: 4 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![0x90, 0xA4, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "stw r5,4(r4) ; blr"
        );
        // A literal value goes through the SCRATCH register, never r3: measured
        // `39600007 91630000` for `void f(S* s){ s->a = 7; }`.
        f.params = vec![0xF509];
        f.ops = vec![
            IlOp::Load(0xF509),
            IlOp::Lit(7),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x07, 0x91, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20
            ],
            "li r11,7 ; stw r11,0(r3) ; blr"
        );
        // …and a wide literal is the `lis`+`ori` pair through the same register.
        f.ops[1] = IlOp::Lit(70000);
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0x3D, 0x60, 0x00, 0x01, 0x61, 0x6B, 0x11, 0x70],
            "lis r11,1 ; ori r11,r11,4464"
        );
        // Not a store leaf at all: the ordinary selector keeps its behaviour.
        f.ops = vec![IlOp::Load(0xF509), IlOp::LoadInd { off: 4 }];
        assert!(store_leaf_text(&f, OptMode::Ox).is_none());
    }


    /// W37: the RUN — one store per group, in source order, plus the emitter's
    /// own restatement of the two gates the parser draws. Every expected word is
    /// transcribed from the reference obj of `work/w37/probe/p1.cpp`.
    #[test]
    fn store_run_text_is_one_store_per_statement_in_source_order() {
        let mut f = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?s2@@YAXPAUS@@HH@Z".into(),
            source_path: None,
            params: vec![0x0101, 0x0201, 0x0301],
            ops: vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::StoreInd { off: 0, width: 4 },
                IlOp::Load(0x0101),
                IlOp::Load(0x0301),
                IlOp::StoreInd { off: 4, width: 4 },
            ],
            tail_call: None,
            framed_call: None,
            call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
            fp_arg_sources: None,
            arg_sources: None,
            data_sym: None,
        };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x90, 0x83, 0x00, 0x00, // stw r4,0(r3)
                0x90, 0xA3, 0x00, 0x04, // stw r5,4(r3)
                0x4E, 0x80, 0x00, 0x20, // blr
            ],
            "?s2@@YAXPAUS@@HH@Z"
        );
        // SOURCE order, not offset order — the one thing an ascending run cannot
        // distinguish, and the reason `?s2r@@YAXPAUS@@HH@Z` is in the probe.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Load(0x0301),
            IlOp::StoreInd { off: 4, width: 4 },
            IlOp::Load(0x0101),
            IlOp::Load(0x0201),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x90, 0xA3, 0x00, 0x04, // stw r5,4(r3)
                0x90, 0x83, 0x00, 0x00, // stw r4,0(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "?s2r@@YAXPAUS@@HH@Z"
        );
        // The widths are per statement, and the FP group is two ops rather than
        // three (`?s2f@@YAXPAUS@@MN@Z`, the other register file).
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::StoreIndFp { off: 16, double: false, src: 1 },
            IlOp::Load(0x0101),
            IlOp::StoreIndFp { off: 24, double: true, src: 2 },
        ];
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0xD0, 0x23, 0x00, 0x10, // stfs f1,16(r3)
                0xD8, 0x43, 0x00, 0x18, // stfd f2,24(r3)
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // **A literal inside a run is no longer a refusal — it is SCHEDULED**,
        // and this cell is the one place in the file where that change is
        // visible as bytes. `{ s->a = u; s->b = 2; }` is `.0` to ORDER, which
        // answers `P0 S0 S1`: the `li` leads, then the two stores in source
        // order. MEASURED as case `M2` of `work/w-wire/mode_probe.py`,
        // byte-identical at `/O1` and `/Ox`:
        //
        // ```text
        //   Pli:r11 S0@r4 S4@r11
        // ```
        //
        // **This is an additive-ACCEPT in the emitter and it is said plainly
        // rather than blurred into the guards' additive-refusal property.** The
        // predecessor refused here as a second lock on the parser's own gate,
        // because it did not know the answer; it does now, and the answer is
        // graded against real `c2`. The parser is still the live lock — nothing
        // reaches this arm until `try_parse_store_run` widens — so this cell
        // moves no byte on the workload by itself.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Load(0x0201),
            IlOp::StoreInd { off: 0, width: 4 },
            IlOp::Load(0x0101),
            IlOp::Lit(2),
            IlOp::StoreInd { off: 4, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x02, // li r11,2
                0x90, 0x83, 0x00, 0x00, // stw r4,0(r3)
                0x91, 0x63, 0x00, 0x04, // stw r11,4(r3)
                0x4E, 0x80, 0x00, 0x20, // blr
            ],
            "a literal beside a formal is scheduled, not refused"
        );
        // **The overlap gate is NOT relaxed.** It is the other half of the
        // sentence above: two stores overlapping one base object are wrong
        // bytes, not a schedule, because c2 eliminates the dead one.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Load(0x0201),
            IlOp::StoreInd { off: 0, width: 4 },
            IlOp::Load(0x0101),
            IlOp::Load(0x0301),
            IlOp::StoreInd { off: 2, width: 4 },
        ];
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_err(), "overlapping stores");
        // …and a run of ONE with a literal is unaffected: that is the store
        // leaf's own captured `li r11,k ; stw r11`.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Lit(7),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![0x39, 0x60, 0x00, 0x07, 0x91, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20],
            "li r11,7 ; stw r11,0(r3) ; blr"
        );
        // **An `ops`-less function must return `None`, not a bare `blr`.** The
        // single-store version matched the whole slice exactly; a loop matches a
        // PREFIX, and the empty prefix matches everything. That turned every
        // comparison leaf in a store-bearing TU into four bytes of `blr` —
        // `w29_fp_contract.cpp`, `Port=Mismatch @ 8` — and `w25_store_leaf.cpp`,
        // which has no compare, was green over it.
        f.ops = Vec::new();
        assert!(store_leaf_text(&f, OptMode::Ox).is_none(), "an ops-less shape is not a store");
    }

    /// WSL: a store whose VALUE is an indirect load — the two-instruction pair,
    /// the widths that pick both opcodes, and the `/O1` / `/Ox` scratch split.
    /// Every expected word is transcribed from the reference obj of
    /// `work/wsl/probe/p1.cpp`, `p2.cpp` and `p6.cpp`, not derived from the
    /// encoding rule.
    #[test]
    fn load_valued_store_is_a_scratch_pair_and_the_mode_picks_the_register() {
        let mut f = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?c1@@YAXPAUS@@PAUQ@@@Z".into(),
            source_path: None,
            params: vec![0x0101, 0x0201],
            ops: vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadInd { off: 4 },
                IlOp::StoreInd { off: 0, width: 4 },
            ],
            tail_call: None,
            framed_call: None,
            call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
            fp_arg_sources: None,
            arg_sources: None,
            data_sym: None,
        };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x81, 0x64, 0x00, 0x04, // lwz r11,4(r4)
                0x91, 0x63, 0x00, 0x00, // stw r11,0(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "?c1@@YAXPAUS@@PAUQ@@@Z"
        );
        // The narrow and wide widths pick BOTH opcodes, and the two halves are
        // independent fields even though the parser requires them equal.
        f.ops[2] = IlOp::LoadIndSized { off: 0, width: 1, sext: false };
        f.ops[3] = IlOp::StoreInd { off: 0, width: 1 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0x89, 0x64, 0x00, 0x00, 0x99, 0x63, 0x00, 0x00],
            "lbz r11,0(r4) ; stb r11,0(r3)"
        );
        f.ops[2] = IlOp::LoadIndSized { off: 2, width: 2, sext: false };
        f.ops[3] = IlOp::StoreInd { off: 2, width: 2 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xA1, 0x64, 0x00, 0x02, 0xB1, 0x63, 0x00, 0x02],
            "lhz r11,2(r4) ; sth r11,2(r3)"
        );
        f.ops[2] = IlOp::LoadIndSized { off: 8, width: 8, sext: false };
        f.ops[3] = IlOp::StoreInd { off: 8, width: 8 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xE9, 0x64, 0x00, 0x08, 0xF9, 0x63, 0x00, 0x08],
            "ld r11,8(r4) ; std r11,8(r3) — both DS-form"
        );
        // The FLOATING-POINT pair lands in f0, never f1: `f1` is the first FP
        // ARGUMENT, and a loaded value is not an argument.
        f.ops[2] = IlOp::LoadIndFp { off: 16, double: false };
        f.ops[3] = IlOp::StoreIndFp { off: 16, double: false, src: FP_SCRATCH };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xC0, 0x04, 0x00, 0x10, 0xD0, 0x03, 0x00, 0x10],
            "lfs f0,16(r4) ; stfs f0,16(r3)"
        );
        f.ops[2] = IlOp::LoadIndFp { off: 24, double: true };
        f.ops[3] = IlOp::StoreIndFp { off: 24, double: true, src: FP_SCRATCH };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xC8, 0x04, 0x00, 0x18, 0xD8, 0x03, 0x00, 0x18],
            "lfd f0,24(r4) ; stfd f0,24(r3)"
        );
        // A run of THREE. `/Ox` gives each statement its own DESCENDING register
        // and `/O1` reuses r11 — the same allocator split `docs/OPT_MODE.md`
        // §3.1 records for arithmetic chains, and the one thing about this shape
        // that is not mode-independent. Transcribed from `?c3@@YAXPAUS@@0@Z` at
        // `/Ox /Gy` and at `/O1 /Gy`.
        let group = |off: i32| {
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadInd { off },
                IlOp::StoreInd { off, width: 4 },
            ]
        };
        f.ops = [group(0), group(4), group(8)].concat();
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x81, 0x64, 0x00, 0x00, 0x91, 0x63, 0x00, 0x00, // lwz r11 ; stw r11
                0x81, 0x44, 0x00, 0x04, 0x91, 0x43, 0x00, 0x04, // lwz r10 ; stw r10
                0x81, 0x24, 0x00, 0x08, 0x91, 0x23, 0x00, 0x08, // lwz r9  ; stw r9
                0x4E, 0x80, 0x00, 0x20,
            ],
            "/Ox descends r11, r10, r9"
        );
        assert_eq!(
            store_leaf_text(&f, OptMode::O1).unwrap().unwrap(),
            vec![
                0x81, 0x64, 0x00, 0x00, 0x91, 0x63, 0x00, 0x00,
                0x81, 0x64, 0x00, 0x04, 0x91, 0x63, 0x00, 0x04,
                0x81, 0x64, 0x00, 0x08, 0x91, 0x63, 0x00, 0x08,
                0x4E, 0x80, 0x00, 0x20,
            ],
            "/O1 reuses r11"
        );
        // The two register FILES are counted independently: an FP statement
        // between two GPR ones must not advance the GPR descent, and vice versa
        // (MEASURED, `?fx3@@YAXPAUW@@0@Z` is `r11 ; f0 ; r10`).
        f.ops = [
            group(0),
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadIndFp { off: 16, double: false },
                IlOp::StoreIndFp { off: 16, double: false, src: FP_SCRATCH },
            ],
            group(32),
        ]
        .concat();
        let t = store_leaf_text(&f, OptMode::Ox).unwrap().unwrap();
        assert_eq!(&t[0..4], [0x81, 0x64, 0x00, 0x00], "lwz r11");
        assert_eq!(&t[8..12], [0xC0, 0x04, 0x00, 0x10], "lfs f0");
        assert_eq!(&t[16..20], [0x81, 0x44, 0x00, 0x20], "lwz r10, not r9");
        // …and the FP descent's own second element is f13, not f1 or f12.
        f.ops = [
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadIndFp { off: 0, double: false },
                IlOp::StoreIndFp { off: 0, double: false, src: FP_SCRATCH },
            ],
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadIndFp { off: 4, double: false },
                IlOp::StoreIndFp { off: 4, double: false, src: FP_SCRATCH },
            ],
        ]
        .concat();
        assert_eq!(
            &store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[8..12],
            [0xC1, 0xA4, 0x00, 0x04],
            "lfs f13,4(r4)"
        );
        // **The descent refuses where it would reach a parameter's register.**
        // The parser draws the same bound; restating it here is the census/gate
        // invariant, and past it c2 skips live registers and wraps back to r11
        // rather than continuing down.
        f.params = vec![0x0101, 0x0201];
        f.ops = (0..8).flat_map(|i| group(i * 4)).collect();
        assert!(
            store_leaf_text(&f, OptMode::Ox).unwrap().is_err(),
            "eight statements with two parameters reaches r4"
        );
        // …and it is the PARAMETER COUNT that moves the bound, not the length.
        f.params = vec![0x0101, 0x0201, 0x0301, 0x0401];
        f.ops = (0..6).flat_map(|i| group(i * 4)).collect();
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_err(), "six with four");
        f.ops = (0..5).flat_map(|i| group(i * 4)).collect();
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_ok(), "five with four");
        // `/O1` has no bound at all, because every statement reuses r11.
        f.params = vec![0x0101, 0x0201];
        f.ops = (0..8).flat_map(|i| group(i * 4)).collect();
        assert!(store_leaf_text(&f, OptMode::O1).unwrap().is_ok());
    }

    /// **The rewrite cannot turn a live accept into a refusal**, proved by
    /// ENUMERATION rather than by the argument beside it.
    ///
    /// `scheduled_gpr_run_text` replaced a source-order emitter with one that
    /// asks [`order::schedule`] and [`alloc::allocate`], both of which answer
    /// `None` outside their domain — and a `None` here is an `out_of_class`
    /// refusal. If either could refuse a run the parser *admits*, the rewrite
    /// would delete accepted bytes.
    ///
    /// The class `c2_il::try_parse_store_run` admits today is exactly two
    /// families: an **all-formal** run, and an all-**same**-literal run — each
    /// through any number of base symbols. Both are walked here, to length 7
    /// over up to 3 symbols, with the count printed. `w-frame2`'s own reduction
    /// test is the model for this: state the claim as code, not beside it.
    #[test]
    fn the_schedule_never_refuses_a_run_the_parser_admits_today() {
        let mut checked = 0usize;
        for len in 1..=7usize {
            for mask in 0..3u32.pow(len as u32) {
                let bases: Vec<u32> =
                    (0..len).map(|i| (mask / 3u32.pow(i as u32)) % 3).collect();
                for produced in [false, true] {
                    let stmts: Vec<schedule::Stmt> = bases
                        .iter()
                        .map(|&b| schedule::Stmt {
                            // one literal shared by every store, or none at all
                            producer: if produced { Some(7) } else { None },
                            base: b,
                        })
                        .collect();
                    let sched = order::schedule(&stmts);
                    assert!(
                        sched.is_some(),
                        "the schedule refuses an admitted run: {stmts:?}"
                    );
                    // …and it is SOURCE order with the producer leading, which
                    // is what the predecessor emitted. Byte-equality of the
                    // rewrite on this whole class reduces to this.
                    let want: Vec<schedule::Slot> = produced
                        .then(|| schedule::Slot::Producer(7))
                        .into_iter()
                        .chain((0..len).map(schedule::Slot::Store))
                        .collect();
                    assert_eq!(sched.unwrap(), want, "not source order: {stmts:?}");
                    if produced {
                        // and the register is r11, for every parameter count
                        // that leaves a pool at all.
                        for nparams in 1..=8u8 {
                            let ps = [alloc::Producer {
                                id: 7,
                                kind: alloc::ProducerKind::Constant,
                                uses: len,
                                first: 0,
                                roots: None,
                            }];
                            assert_eq!(
                                alloc::allocate(&ps, 3 + nparams),
                                Some(vec![(7, SCRATCH_REG)]),
                                "one shared literal is r11 at {nparams} formals"
                            );
                        }
                    }
                    checked += 1;
                }
            }
        }
        assert!(checked >= 4000, "only {checked} runs enumerated");
    }

    /// **The constructed counterexamples**, built to fail. Each is one step
    /// outside the region a model is exact on, and each must come back a
    /// refusal rather than an answer. `docs/rungs/_2026-08-05-w-wire-prereg.md`
    /// §4 registered all five before the code was written.
    #[test]
    fn the_widening_refuses_one_step_outside_every_models_domain() {
        let base = |ops: Vec<IlOp>, params: Vec<u32>| IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?x@@YAXPAUS@@@Z".into(),
            source_path: None,
            params,
            ops,
            tail_call: None,
            framed_call: None,
            call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
            fp_arg_sources: None,
            arg_sources: None,
            data_sym: None,
        };
        let lit_group = |b: u32, off: i32, k: i32| {
            vec![
                IlOp::Load(b),
                IlOp::Lit(k),
                IlOp::StoreInd { off, width: 4 },
            ]
        };

        // 1. FOUR distinct literals. Past `MAX_MODELLED_PRODUCERS` c2 begins
        //    REUSING a freed register in preference to a fresh one, and the two
        //    probed four-producer runs with identical structure disagree — board
        //    #541. `alloc` refuses; so must this.
        let f = base(
            (0..4).flat_map(|i| lit_group(0x0101, i * 4, i + 1)).collect(),
            vec![0x0101],
        );
        assert!(
            store_leaf_text(&f, OptMode::O1).unwrap().is_err(),
            "four distinct literals are board #541, not a schedule"
        );
        // …and THREE are inside it, which is what makes the line above a
        // boundary rather than a blanket refusal.
        let f = base(
            (0..3).flat_map(|i| lit_group(0x0101, i * 4, i + 1)).collect(),
            vec![0x0101],
        );
        assert!(store_leaf_text(&f, OptMode::O1).unwrap().is_ok(), "three fit");

        // 3. THE POOL BOUNDARY. Eight formals hold r4..r11, so `pool_floor` is
        //    11 and one register is free where two are wanted. c2 descends into
        //    registers freed by already-emitted stores — including r3 itself —
        //    and that regime is unmodelled.
        let f = base(
            (0..2).flat_map(|i| lit_group(0x0101, i * 4, i + 1)).collect(),
            (0..8).map(|i| 0x0101 + i * 0x100).collect(),
        );
        assert!(
            store_leaf_text(&f, OptMode::O1).unwrap().is_err(),
            "two producers do not fit a one-register pool"
        );

        // 4. A WIDE literal beside a narrow one. **THIS COUNTEREXAMPLE FIRED,
        //    and it is the most valuable line in this test.** The prereg
        //    registered it predicting a NON-boundary — "`lis`+`ori` is two words
        //    for one producer, and `layout_slots` indexes producers, not words,
        //    so this is in domain and must be answered with the pair kept
        //    whole". Real `c2` refuted that twice over
        //    (`work/w-wire/boundary_probe.py`, identical at `/O1` and `/Ox`):
        //
        //    ```text
        //      c2:   lis r11 ; li r10 ; ori r11 ; stw r10,4(r3) ; stw r11,0(r3)
        //      port: lis r11 ; ori r11 ; li r10 ; stw r11,0(r3) ; stw r10,4(r3)
        //    ```
        //
        //    The `lis`/`ori` pair is SPLIT, and the store order is `[1,0]` where
        //    `store_order` says source order. Had the widening shipped without
        //    this probe it would have been a live wrong emit — board #232's
        //    exact shape, caught by a constructed counterexample instead of by
        //    a scan 255 commits later.
        let mut ops = lit_group(0x0101, 0, 100000);
        ops.extend(lit_group(0x0101, 4, 1));
        let f = base(ops, vec![0x0101]);
        assert!(
            store_leaf_text(&f, OptMode::O1).unwrap().is_err(),
            "a multi-word literal beside another producer interleaves its halves"
        );
        // …and a run whose ONLY producer is wide is NOT refused: one live range,
        // nothing to interleave with. `{ a=100000; b=100000; }` is
        // `lis ; ori ; stw ; stw` — the parser already admits this cell, so a
        // gate that caught it would delete accepted bytes.
        let mut ops = lit_group(0x0101, 0, 100000);
        ops.extend(lit_group(0x0101, 4, 100000));
        let f = base(ops, vec![0x0101]);
        assert_eq!(
            store_leaf_text(&f, OptMode::O1).unwrap().unwrap().len(),
            4 * 5,
            "lis + ori + two stores + blr"
        );

        // 5. `x_split`'s mask — `nsw = 3`, the cell board #602's domain gate is
        //    drawn for. Two symbols and two producers: inside
        //    `MAX_MULTISYM_PRODUCERS`, outside `MAX_SYMBOL_CROSSINGS`. The
        //    emitter must refuse it even though `store_order` answers.
        let split: Vec<schedule::Stmt> = [
            (None, 0u32),
            (None, 0),
            (Some(0u32), 1),
            (None, 0),
            (Some(1), 1),
            (Some(1), 1),
        ]
        .iter()
        .map(|&(producer, base)| schedule::Stmt { producer, base })
        .collect();
        assert!(
            order::store_order(&split).is_some(),
            "precondition: the STORE order is in domain, only the LAYOUT is not"
        );
        assert_eq!(
            order::schedule(&split),
            None,
            "nsw = 3 is outside the layout's exact region"
        );
    }

    /// **Board #844 / #866 — the run text a FRAMED body would need is exactly
    /// this run text, and the `blr` is why no framed body can have it.**
    ///
    /// Lane `w-seam` compiled every configuration below in four body kinds at
    /// the workload's own `/O1 /Oi /EHsc /GR` and compared the *run text* — the
    /// disassembly with the frame words, the `bl` and the callee-saved copies
    /// stripped — against the leaf's, character for character
    /// (`work/w-seam/gridt.out`, `work/w-seam/gridt2.out`):
    ///
    /// ```text
    ///   L    void f(S*,int,int){ <run> }                  the control
    ///   P2   void f(S*,int,int){ <run> gx(); gy(); }      a FRAME (9 frame words)
    ///   R    S*  f(S*,int,int){ <run> gx(); return s; }   `this` live across it
    ///
    ///   GRID T   60 selected / 60 reached / 60 GRADED / 0 out-of-regime
    ///            P2 12/12 IDENT   R 12/12 IDENT
    ///   GRID T2  36 selected / 36 reached / 36 GRADED / 0 out-of-regime
    ///            34 IDENT, 2 DIFFER — both `D11-argcall`
    /// ```
    ///
    /// So [`order::schedule`] and [`alloc::allocate`], fitted entirely on leaf
    /// bodies, **transfer unchanged into a framed body when the run precedes
    /// the call**, and the `mr r31,r3` a live-across-the-call object needs is
    /// *additive* — it is inserted into the run without moving one other word.
    ///
    /// **The boundary is the call's ARGUMENT, and it is measured.** When the
    /// trailing call takes one (`gx(u)`), the run does not transfer at all:
    /// c2 parks the object in a **volatile** `r10`, the store base changes
    /// mid-run, and the constants re-rank to `r11`/`r9` where the leaf takes
    /// `r11`/`r10`. Any framed seam has to gate on that.
    ///
    /// This test pins the leaf side of three of those cells against the
    /// reference bytes, and pins the **structural** fact that makes them
    /// unreachable from a framed body: [`scheduled_gpr_run_text`] appends
    /// [`encode_blr`] unconditionally, so its text is a whole body and nothing
    /// can bracket it with a frame. A lane that composes the two has to change
    /// that line and will land here.
    #[test]
    fn the_scheduled_run_text_is_a_whole_body_and_ends_in_blr() {
        let mk = |ops: Vec<IlOp>, params: Vec<u32>| {
            let mut f = func_with(params, ops);
            f.mangled_name = "?w_seam@@YAXPAUS@@HH@Z".into();
            f
        };
        let lit_group = |b: u32, off: i32, k: i32| {
            vec![IlOp::Load(b), IlOp::Lit(k), IlOp::StoreInd { off, width: 4 }]
        };
        // Three formals — `void f(S* s, int u, int v)` — so the pool floor is
        // r6 and three producers still fit r11/r10/r9.
        let p3 = vec![0x0101u32, 0x0201, 0x0301];

        // `C5-const-3x1`: { s->f0=7; s->f8=9; s->fc=11; }
        let mut ops = lit_group(0x0101, 0, 7);
        ops.extend(lit_group(0x0101, 32, 9));
        ops.extend(lit_group(0x0101, 48, 11));
        assert_eq!(
            store_leaf_text(&mk(ops, p3.clone()), OptMode::O1).unwrap().unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x07, // li  r11,7
                0x39, 0x40, 0x00, 0x09, // li  r10,9
                0x39, 0x20, 0x00, 0x0B, // li  r9,11
                0x91, 0x63, 0x00, 0x00, // stw r11,0(r3)
                0x91, 0x43, 0x00, 0x20, // stw r10,32(r3)
                0x91, 0x23, 0x00, 0x30, // stw r9,48(r3)
                0x4E, 0x80, 0x00, 0x20, // blr   <- the leaf-only word
            ],
            "C5-const-3x1: the framed P2/R cells emit the first six words of \
             this between a stwu and a bl"
        );

        // `C11-const-inter`: { s->f0=7; s->f8=9; s->f1=7; s->f9=9; } — the
        // REVERSE-source-order tie (clause 4). Both constants are at 2 uses and
        // the LATER one takes r11.
        let mut ops = lit_group(0x0101, 0, 7);
        ops.extend(lit_group(0x0101, 32, 9));
        ops.extend(lit_group(0x0101, 4, 7));
        ops.extend(lit_group(0x0101, 36, 9));
        assert_eq!(
            store_leaf_text(&mk(ops, p3.clone()), OptMode::O1).unwrap().unwrap(),
            vec![
                0x39, 0x40, 0x00, 0x07, // li  r10,7
                0x39, 0x60, 0x00, 0x09, // li  r11,9   <- the LATER constant
                0x91, 0x43, 0x00, 0x00, // stw r10,0(r3)
                0x91, 0x63, 0x00, 0x20, // stw r11,32(r3)
                0x91, 0x43, 0x00, 0x04, // stw r10,4(r3)
                0x91, 0x63, 0x00, 0x24, // stw r11,36(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "C11-const-inter"
        );

        // `D9-run7`: seven stores, two producers at 4 and 3 uses.
        let mut ops: Vec<IlOp> = Vec::new();
        for i in 0..4 {
            ops.extend(lit_group(0x0101, i * 4, 7));
        }
        for i in 4..7 {
            ops.extend(lit_group(0x0101, i * 4, 9));
        }
        let t = store_leaf_text(&mk(ops, p3), OptMode::O1).unwrap().unwrap();
        assert_eq!(t.len(), 4 * 10, "two `li`s, seven stores and the blr");
        assert_eq!(
            &t[..8],
            [0x39, 0x60, 0x00, 0x07, 0x39, 0x40, 0x00, 0x09],
            "li r11,7 ; li r10,9 — 4 uses outranks 3"
        );
        // **The structural fact this rung is about.** Every accepted run's text
        // ENDS in `blr` — it is a whole body, so there is no seam that can put
        // it in the middle of a framed one (board #844).
        assert_eq!(
            &t[t.len() - 4..],
            &encode_blr()[..],
            "the run text is leaf-only by construction"
        );
    }

    /// **BOARD #1199 — the bind carrier, in EMITTED BYTES, and the pair that
    /// proves it did not collapse.**
    ///
    /// Every expected word below is transcribed from real `c2.dll`'s own obj at
    /// the WORKLOAD's `/GR /O1 /Oi /EHsc` (board #1112) — `work/w-carrier/grid/`,
    /// one directory per cell, manifest frozen before the first `cl.exe` — and
    /// every one of those cells is `Port=Match` on the whole-obj differential,
    /// which is the sole judge. This test is the second lock, so a future edit
    /// cannot move a word and stay green without a toolchain.
    ///
    /// The load-bearing pair is `k_base1` against `k_base1_c`: the same body with
    /// and without `BE& l = h->mListHead;`. Real `c2` emits **different** bodies
    /// —
    ///
    /// ```text
    ///   k_base1    li 11,2 ; stw 11,16(3) ; stw 4,8(3)   BIND: two symbols, the
    ///                                                    pin holds source order
    ///   k_base1_c  li 11,2 ; stw 4,8(3) ; stw 11,16(3)   DIRECT: one symbol, the
    ///                                                    produced store leaves 0
    /// ```
    ///
    /// — and the port emits both, byte for byte. A carrier that discharged the
    /// binding into the store's displacement would emit the second where the
    /// first belongs, which is board **#1128**/#232.
    /// **BOARD #1231 — THE CARRIER, decoded at the emitter's own seam.**
    ///
    /// The two roots of `alloc::ProducerRoots` have both been live in
    /// [`parse_simple_gpr_run`] since board #1199, and the value's was thrown
    /// away by an `IlOp::BoundAddr { .. }` pattern because [`alloc::Producer`]
    /// had no field to receive it. Nine allocation keys died stating a
    /// **relation** in a structure that holds only per-producer facts; this test
    /// is that relation, read off the op stream.
    ///
    /// The shapes are GRID Z's six families (`work/w-self2b/roots.out`), each
    /// reduced to its deciding store. c2's own answer is in the `prod` column of
    /// that table and it is asserted here through
    /// `store_root_is_distinct_bind`, which is the predicate the table
    /// supports.
    ///
    /// **Every one of these runs is REFUSED by the emitter** — a bind-valued
    /// store is `value_bound`, and board #840 means no register-derived
    /// producer reaches `alloc::allocate` at all. So this decodes a shape the
    /// port declines to emit, on purpose: the carrier's job is to make the fact
    /// measurable, not to move a byte. `alloc::allocate_ignores_the_roots_
    /// carrier` pins the other half of that.
    #[test]
    fn the_carrier_decodes_both_roots_of_a_bind_valued_store() {
        // formals r3, r4; `k` and `m` are bound locals, `j` a second bind.
        let (h, p) = (0x0101u32, 0x0201u32);
        let (k, m) = (0x130Au32, 0x140Au32);
        let reg_of = |t: u32| match t {
            x if x == h => Some(3u8),
            x if x == p => Some(4u8),
            _ => None,
        };
        let kb = IlOp::BoundAddr { tok: k, base: h, off: 48 };
        let mb = IlOp::BoundAddr { tok: m, base: h, off: 48 };

        // (cell, class, the store group, c2's answer from GRID Z)
        let cases: [(&str, &str, Vec<IlOp>, bool); 5] = [
            // Z1 SELF-1B  — value and stores both path-spelled off the formal.
            ("Z1", "SELF-1B", vec![IlOp::Load(h), IlOp::Load(h), IlOp::StoreInd { off: 48, width: 4 }], false),
            // Z2 LOAD     — the bind's own name, stored through the same bind.
            ("Z2", "LOAD", vec![kb.clone(), kb.clone(), IlOp::StoreInd { off: 0, width: 4 }], false),
            // Z3 SELF-2B  — path-spelled value, stores through the bind.
            ("Z3", "SELF-2B", vec![kb.clone(), IlOp::Load(h), IlOp::StoreInd { off: 0, width: 4 }], true),
            // Z5 MIRROR   — the mirror image of Z3. c2 answers DIFFERENTLY.
            ("Z5", "MIRROR", vec![IlOp::Load(h), kb.clone(), IlOp::StoreInd { off: 48, width: 4 }], false),
            // Z6 TWOBIND  — one bind's name, stored through a SECOND bind to
            //               the same object.
            ("Z6", "TWOBIND", vec![mb.clone(), kb.clone(), IlOp::StoreInd { off: 0, width: 4 }], true),
        ];

        for (cell, klass, ops, is_prod) in cases {
            let run = parse_simple_gpr_run(&ops, &reg_of)
                .unwrap_or_else(|| panic!("{cell} ({klass}) must parse as a store group"));
            assert_eq!(run.len(), 1);
            let s = &run[0];
            let value = s
                .value_root
                .clone()
                .unwrap_or_else(|| panic!("{cell}: the VALUE's root must not be discarded"));
            let r = alloc::ProducerRoots { value, lvalue: s.lvalue_root.clone() };
            assert_eq!(
                r.store_root_is_distinct_bind(),
                is_prod,
                "{cell} ({klass}) — the #1231 predicate, decoded from the op stream"
            );
            // #908: the seam still carries `off` as a SUM, so the list half is
            // honestly absent rather than faked from it.
            assert_eq!(r.value_offsets_prefix_lvalue(), None, "{cell}: #908 gap");
        }

        // **Z3 against Z5 is the asymmetry, at this seam.** Same two roots, the
        // two positions exchanged, and c2 answers differently — which is what
        // no per-producer field could ever have represented.
        let z3 = parse_simple_gpr_run(
            &[kb.clone(), IlOp::Load(h), IlOp::StoreInd { off: 0, width: 4 }],
            &reg_of,
        )
        .unwrap();
        let z5 = parse_simple_gpr_run(
            &[IlOp::Load(h), kb.clone(), IlOp::StoreInd { off: 48, width: 4 }],
            &reg_of,
        )
        .unwrap();
        assert_eq!(z3[0].lvalue_root.tok, z5[0].value_root.clone().unwrap().tok);
        assert_eq!(z5[0].lvalue_root.tok, z3[0].value_root.clone().unwrap().tok);

        // A LITERAL value has no designator, so it has no root — `None`, never
        // a fabricated one, and `producer_roots` refuses the pair.
        let lit = parse_simple_gpr_run(
            &[IlOp::Load(h), IlOp::Lit(7), IlOp::StoreInd { off: 0, width: 4 }],
            &reg_of,
        )
        .unwrap();
        assert!(lit[0].value_root.is_none());
        assert!(producer_roots(&lit, 7).is_none());
    }

    #[test]
    fn the_bind_carrier_emits_both_spellings_and_they_stay_apart() {
        let (h, p) = (0x0101u32, 0x0201u32);
        let l = 0xFB09u32;
        let bound = IlOp::BoundAddr { tok: l, base: h, off: 8 };
        let mk = |ops: Vec<IlOp>| func_with(vec![h, p], ops);
        let text = |ops: Vec<IlOp>| store_leaf_text(&mk(ops), OptMode::O1).unwrap().unwrap();

        // `k_base1`: h->mSize = 2; BE& l = h->mListHead; l.mNext = p;
        assert_eq!(
            text(vec![
                IlOp::Load(h),
                IlOp::Lit(2),
                IlOp::StoreInd { off: 16, width: 4 },
                bound,
                IlOp::Load(p),
                IlOp::StoreInd { off: 0, width: 4 },
            ]),
            vec![
                0x39, 0x60, 0x00, 0x02, // li  r11,2
                0x91, 0x63, 0x00, 0x10, // stw r11,16(r3)
                0x90, 0x83, 0x00, 0x08, // stw r4,8(r3)   <- base r3, displacement 8+0
                0x4E, 0x80, 0x00, 0x20,
            ],
            "k_base1"
        );
        // `k_base1_c`, the DIRECT twin. One symbol, so the produced store may not
        // hold store position 0 and it moves — a different body.
        assert_eq!(
            text(vec![
                IlOp::Load(h),
                IlOp::Lit(2),
                IlOp::StoreInd { off: 16, width: 4 },
                IlOp::Load(h),
                IlOp::Load(p),
                IlOp::StoreInd { off: 8, width: 4 },
            ]),
            vec![
                0x39, 0x60, 0x00, 0x02, // li  r11,2
                0x90, 0x83, 0x00, 0x08, // stw r4,8(r3)
                0x91, 0x63, 0x00, 0x10, // stw r11,16(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "k_base1_c — the two spellings MUST NOT collapse"
        );

        // `k_off24`: the same shape bound at +24. **The one site the sum is
        // formed** — `24 + 0` and `24 + 4` — and the only thing that moves.
        let far = IlOp::BoundAddr { tok: l, base: h, off: 24 };
        assert_eq!(
            text(vec![
                IlOp::Load(h),
                IlOp::Lit(2),
                IlOp::StoreInd { off: 16, width: 4 },
                far,
                IlOp::Load(p),
                IlOp::StoreInd { off: 0, width: 4 },
                far,
                IlOp::Load(p),
                IlOp::StoreInd { off: 4, width: 4 },
            ]),
            vec![
                0x39, 0x60, 0x00, 0x02, // li  r11,2
                0x91, 0x63, 0x00, 0x10, // stw r11,16(r3)
                0x90, 0x83, 0x00, 0x18, // stw r4,24(r3)
                0x90, 0x83, 0x00, 0x1C, // stw r4,28(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "k_off24"
        );

        // `k_gap3`: three stores on the OTHER symbol between the bind and its
        // first use — the axis four earlier grids held fixed. The producer's
        // store is hoisted past two unproduced ones and the bound store trails.
        assert_eq!(
            text(vec![
                IlOp::Load(h),
                IlOp::Lit(2),
                IlOp::StoreInd { off: 16, width: 4 },
                IlOp::Load(h),
                IlOp::Load(h),
                IlOp::StoreInd { off: 0, width: 4 },
                IlOp::Load(h),
                IlOp::Load(h),
                IlOp::StoreInd { off: 4, width: 4 },
                bound,
                IlOp::Load(p),
                IlOp::StoreInd { off: 0, width: 4 },
            ]),
            vec![
                0x39, 0x60, 0x00, 0x02, // li  r11,2
                0x90, 0x63, 0x00, 0x00, // stw r3,0(r3)
                0x90, 0x63, 0x00, 0x04, // stw r3,4(r3)
                0x91, 0x63, 0x00, 0x10, // stw r11,16(r3)
                0x90, 0x83, 0x00, 0x08, // stw r4,8(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "k_gap3"
        );
    }

    /// **The backstop for the frontier's last refusal**, and the counterexample
    /// beside it.
    ///
    /// A bound reference in the stored-VALUE position is an interior address —
    /// one `addi`, a **register-derived** producer. `c2_il`'s `bind_run_ops`
    /// refuses it in the reader under two keys (so the mixed half stays
    /// separately sizeable), and this is the second lock: a parser that widened
    /// past its witness comes out as a gap, not as bytes.
    ///
    /// `src/xdk/nuispeech/xboxheap.cpp` is that shape — an interior address at 2
    /// uses beside a literal at 1 — and it is refused **here** by
    /// [`alloc::allocate`]'s mixed-kind rule if it ever reached it, which
    /// `order::tests::xboxheap_allocation_is_still_refused_and_the_answer_it_owes_is_recorded`
    /// already pins. Boards #836 (wrong on 0 of 81), #868 (12 of 36 on the narrow
    /// lift) and #1134 (clause 1 refuted on this very mix).
    #[test]
    fn a_bound_reference_in_the_value_position_is_refused_by_name() {
        let (h, p) = (0x0101u32, 0x0201u32);
        let l = 0xFB09u32;
        let bound = IlOp::BoundAddr { tok: l, base: h, off: 8 };
        let f = func_with(
            vec![h, p],
            vec![
                bound,
                bound,
                IlOp::StoreInd { off: 0, width: 4 },
                IlOp::Load(h),
                IlOp::Lit(0),
                IlOp::StoreInd { off: 20, width: 4 },
            ],
        );
        let e = store_leaf_text(&f, OptMode::O1)
            .expect("it is a store stream")
            .expect_err("and it must be a REFUSAL, never bytes");
        assert!(
            format!("{e:?}").contains("bound reference"),
            "the refusal must name the construct: {e:?}"
        );
        // …and the run that has the address producer ALONE is refused too — its
        // direct twin's obj is byte-identical and the direct twin is refused, so
        // emitting one and not the other is a divergence with no grid behind it.
        let f = func_with(
            vec![h, p],
            vec![bound, bound, IlOp::StoreInd { off: 0, width: 4 }],
        );
        assert!(store_leaf_text(&f, OptMode::O1).unwrap().is_err());
    }
}

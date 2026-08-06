//! **Mechanism I, the replacement stratum** — the body c2 emits for a caller
//! whose one call it expanded, which is the callee's body and **not** the
//! caller's setup with the callee appended.
//!
//! # What this is, and what it is NOT
//!
//! `docs/INLINE_PREDICATE.md` §0 separates two reasons an IL call can be absent
//! from c2's output. [`crate::elide`] is **E** — the callee's body does nothing,
//! so the call is dropped — and this module is **I**, the inliner expanding it.
//! The two are kept apart there and they are kept apart here: nothing in this
//! file reads [`c2_il::IlFunction::empty_body`] as a licence, and
//! [`crate::elide::drops_tail_call`] is asked **first** so that one function
//! never gets two answers.
//!
//! The composition rule is `w-seq`'s measurement and not a guess
//! (`docs/rungs/2026-08-08-w-seq.md` §4.1, graded by real c2 on 2,470 workload
//! callers whose reference obj carries *both* COMDATs):
//!
//! | hypothesis | graded | exact |
//! |---|---:|---:|
//! | **SPLICE-0** — c2's body for the caller **is** c2's body for the callee | 2,470 | **1,967** |
//! | — `seq` | 816 | **816** |
//! | — `tail` | 1,531 | 1,151 |
//! | — `framed` | 123 | **0** |
//! | SPLICE-P — the port's setup ++ the callee's body | 2,470 | 578 — **578/578** with no setup, **0/953** with one |
//! | SPLICE-N — two or more callees, concatenated | 548 | **0** |
//!
//! **The argument setup does not survive, and neither does the frame.** That is
//! the whole difference between this rule and the concatenation the same lane
//! registered first and lost: of the 1,892 SPLICE-P failures, 1,890 diverge at
//! word 0, which is the port's own first setup word.
//!
//! # The predicate, exactly as it was registered
//!
//! [`splice_body`] is the whole rule; `work/w-splice/PREREG.md` §1 is the
//! registered form and `work/w-splice/cells/` is the frozen grid that grades
//! each clause's boundary against real c2.
//!
//! | clause | what it requires | the cell that grades it |
//! |---|---|---|
//! | **S1** | the selection is [`Selected::Tail`] or [`Selected::Seq`] | `t07` — a **framed** caller. SPLICE-0 is **0 of 123** there, every one a destination-register rename |
//! | **S2** | exactly **one** call site | `t08` — two calls. SPLICE-N is **0 of 548** |
//! | **S3** | the port emits **nothing around the call**: an empty argument setup, and for a `Seq` an identity tail | `t04` (register move), `t05` (arithmetic), `t06` (pointer offset). Every one of w-seq's 503 SPLICE-0 failures is a **field** of the callee's body rewritten — a source rename (286), a destination rename (123), a displacement fold (~92) — and a non-identity setup is the thing that rewrites it |
//! | **S4** | the callee is not the caller | `t12`. `INLINE_PREDICATE.md` §4 grades `recurse` **336/336** declined by c2 as well |
//! | **S5** | the callee is defined in **this bundle**, unambiguously | `t14`, the control: an undefined callee keeps its REL24 |
//! | **S6** | the port composes a complete body for the callee, with **no frame**, and that body is **not itself spliced** | `t09` — an unlowerable callee. The one-level restriction is [`splice_body`]'s `allow_splice: false` recursion, and `t11` is the measurement of whether c2 closes the chain |
//! | **S7** | that body is at most [`INLINE_UNBOUNDED_BYTES`] bytes, and the callee is not varargs | `t13`. See below — this is the inline decision, taken on the safe side of its own boundary |
//! | **S8** | the caller materializes no data symbol of its own | the caller's whole body is discarded and an `F`-side data reference would go with it |
//! | **S9** | mechanism **E** does not fire for the caller | `elide.rs` keeps its answer; one function, one rule |
//!
//! **Every clause is readable from the IL bundle or from the port's own
//! emitter. None of them reads the reference obj.** That is what makes this a
//! predicate rather than the enumeration `w-seq` §6 priced at 726.
//!
//! # S7 IS THE INLINE DECISION, and it is taken where the decision is categorical
//!
//! `INLINE_PREDICATE.md` §2 is a *cost model* — graded 0.9716 on a 9,993-callee
//! frozen hold-out, with a 2.84 % residual its own §7 leaves **NOT MODELLED**.
//! A shipped emitter may not consult a 97 % rule; 3 % of a guess is a wrong
//! emit. What it may consult is the **region where that rule is categorical**:
//!
//! ```text
//! index(G) = s - 4*(nparams-1) - 8*[inline] - 48*[leaf]        <=  s
//!
//! EXTERNAL: N_max = UNBOUNDED  if index <= 64          (§6.17.4)
//! STATIC:   i = index/4;  N_max = UNBOUNDED if i <= 16 (§6.18.9)   i.e. index <= 64
//! either:   N_max = 0          if varargs(G)           (§6.18.5)
//! ```
//!
//! Both linkage classes turn unbounded at the **same** `index <= 64`, and every
//! correction term is subtractive, so `s <= 64` implies `index <= 64` implies
//! `N_max` unbounded — **independently of linkage, of `inline`, of `nparams`, of
//! the `leaf` bit that §5 calls the model's one unreadable input, and of the
//! site count**. Varargs is the single categorical exception and it is checked.
//!
//! The site-side exceptions of §2 are excluded structurally rather than
//! modelled: an **indirect** site names no callee and the IL parser refuses both
//! of its productions (`elide.rs`'s standing hazard note); a **conditional**
//! site is a `cond_pair` or a `Seq` with a `guard`, and S1/S3 refuse both — and
//! in any case that exception moves a ceiling from 260 to 164 bytes, both far
//! above this bound.
//!
//! # What it emits, and the relocation that makes it different from every other rule here
//!
//! The caller's `/Gy` COMDAT becomes the callee's — text, **relocations** and
//! data references, at the same in-section offsets. The caller acquires **no**
//! REL24 against the callee; it acquires the callee's own.
//!
//! > `t10` is that stated as a compiled cell. `void ext(); void g(){ext();}
//! > void f(){g();}` — the port emits `b ?g` for `?f` and c2 emits `b ?ext`, and
//! > **both are the word `48000000`**. FUNCTION BYTE MATCH compares a `.text`
//! > COMDAT's raw data, which does not contain its relocations, so it calls that
//! > pair `exact` today: board **#882**, 4,664 credited functions, and `w-seq`
//! > §4.3's `s12` is the same four lines. This rule *fixes* that cell rather than
//! > adding to it, and `work/w-splice/relocheck.py` verifies the relocation set
//! > of every function it moves, per symbol, against the reference obj.
//!
//! # Where it may NOT fire — [`crate::PortC2::build`]
//!
//! `IlBundle::functions()` refuses any TU that defines one of its own callees,
//! which is a strict superset of S5, so no bundle that can splice ever reaches
//! the whole-obj emitter. That refusal is **not** narrowed here. Independently,
//! `build` refuses a bundle in which this predicate fires, on **both** paths and
//! with the reason named: a spliced `Seq` loses its frame, and with it its
//! `.pdata` record and its `$M`/`$M`/`$T` label slots, while
//! `PortC2::frame_label_counter` is computed from the IL before any body exists.
//! `elide.rs` refuses the packed path for the narrower version of the same
//! reason; this one reaches the label counter, so it refuses both.

use c2_il::{IlFunction, SeqTail};

use crate::codegen::{OptMode, Selected, opt_mode_of_word};
use crate::comdat::{ComdatBody, ComdatDecline};
use crate::elide::{Reduction, TuEmptyCallees, drops_tail_call};

/// **S7's bound.** `INLINE_PREDICATE.md` §2's `N_max` is UNBOUNDED at
/// `index <= 64` in *both* linkage classes, and `index <= s`, so a callee whose
/// emitted body is at most this many bytes is inlined at every site whatever its
/// linkage, its parameter count, its `inline` keyword or the model's unreadable
/// `leaf` bit.
///
/// It is a **byte** count because `s` is: §6.5 reads it off the callee's own
/// emitted `.text`.
pub const INLINE_UNBOUNDED_BYTES: usize = 64;

/// **The bundle-level facts a `/Gy` body needs that are not properties of one
/// function** — mechanism E's callee set, and the callee bodies mechanism I
/// splices.
///
/// It carries [`TuEmptyCallees`] rather than replacing it, and [`Deref`]s to it,
/// so every existing caller of `drops_tail_call(f, tu)` and `tu.len()` reads the
/// same E context it always did. **E is still `elide.rs`'s and only its**; this
/// type is the carrier, not a second implementation.
///
/// # It is name-keyed, and *which* name is the whole problem
///
/// The same #918 that `elide.rs` documents: `IlFunction::mangled_name` is paired
/// **positionally** over `.ex` segments and disagrees with the per-record
/// `FnCensus::emit_name` on **74,955** rows of the dc3 workload. A name-keyed
/// fact read through the wrong binding is attached to the wrong function — for
/// E that turned 14 byte-exact bodies wrong, and for this rule it would splice
/// some *other* function's body into the caller. So the name is supplied by the
/// caller through [`TuContext::of_named`], exactly as E's is, and the callee
/// resolution rate under the two bindings on this very population is **3,702 vs
/// 546 of 3,928** (w-seq §8).
#[derive(Debug, Default, Clone)]
pub struct TuContext<'a> {
    empty: TuEmptyCallees,
    /// `(name, definition, the `.ex` optimization word)`, sorted by name.
    /// A name with more than one row is **refused** rather than resolved to the
    /// first — see [`TuContext::definition`].
    ///
    /// The definition is `None` for a name this bundle **defines and whose IL
    /// the parser refused**. Those rows are kept, and keeping them is not
    /// bookkeeping: [`TuContext::mentions`] is what tells a chain that *ended*
    /// from one the port could not *follow*, and a refused row that vanished
    /// from this vector would read as an external. See the type's docs.
    rows: Vec<(&'a str, Option<&'a IlFunction>, Option<u32>)>,
}

impl<'a> TuContext<'a> {
    /// The context for a caller with no bundle: nothing is elided and nothing is
    /// spliced.
    ///
    /// Required rather than defaulted, for [`crate::comdat`]'s reason: the two
    /// bundle-level mechanisms are the facts a per-function composition cannot
    /// derive, and a caller that *forgot* one would silently emit a call c2 does
    /// not emit. Saying "no bundle" out loud is the point.
    pub fn none() -> Self {
        Self::default()
    }

    /// Build both bundle-level contexts from one pass over `(name, definition,
    /// opt_word)` triples the caller has already bound.
    ///
    /// `opt_word` is the callee's own `.ex` optimization word. `None` means the
    /// caller does not track it per function — [`crate::PortC2::build`], where
    /// the TU has already been refused unless every function shares one mode —
    /// and then the caller's mode is used. A callee whose mode differs from the
    /// caller's is refused (`mode` decides a register field inside a chain, so
    /// splicing across a mode boundary would emit the wrong allocation).
    pub fn of_named(
        named: impl IntoIterator<Item = (&'a str, &'a IlFunction, Option<u32>)>,
    ) -> Self {
        Self::of_rows(
            named
                .into_iter()
                .map(|(n, f, w)| (n, Some(Reduction::Parsed(f)), w)),
        )
    }

    /// **The general constructor** — one row per name this TU defines, carrying
    /// what each of the two mechanisms can make of it.
    ///
    /// # Three questions, three row sets, and only one of them is every row
    ///
    /// | asked by | which rows |
    /// |---|---|
    /// | mechanism **E**'s closure ([`TuEmptyCallees::of_rows`]) | the rows with a [`Reduction`] — parsed, or refused **with a readable `no_effect_callee`** (board **#980**). Every other refused row contributes nothing, which is that board's conservative direction and is preserved here exactly |
    /// | [`TuContext::definition`], i.e. the splice's S5/S6 | **parsed rows only**. A `NoEffectCall` row is a body the parser refused, so the port has no bytes for it and S6 cannot compose a chain end out of it |
    /// | [`TuContext::mentions`] | **every row**, parsed or not, qualifying or not |
    ///
    /// **The third is the one a careless merge loses.** `S6-chain-truncated`
    /// refuses a splice when the chain's last link still names a callee this TU
    /// carries; a refused row dropped from this vector would read as an
    /// *external*, the clause would stop firing, and the splice would run off
    /// the end of a chain it cannot see. That is not a missing count — it is
    /// the wrong-relocation defect (#1009's 72 witnesses) coming back.
    ///
    /// So `Option<Reduction>` is three-valued on purpose: `Some(Parsed)` is a
    /// body both mechanisms can use, `Some(NoEffectCall)` is one only E can use,
    /// and `None` is a name this TU defines that **neither** can use and that
    /// still has to be visible.
    pub fn of_rows(
        rows: impl IntoIterator<Item = (&'a str, Option<Reduction<'a>>, Option<u32>)>,
    ) -> Self {
        let mut rows: Vec<(&'a str, Option<Reduction<'a>>, Option<u32>)> =
            rows.into_iter().filter(|(n, _, _)| !n.is_empty()).collect();
        rows.sort_by_key(|(n, _, _)| *n);
        // Mechanism E sees exactly what board #980 gives it and nothing else.
        let empty = TuEmptyCallees::of_rows(
            rows.iter().filter_map(|(n, r, _)| Some((*n, (*r)?))),
        );
        let rows = rows
            .into_iter()
            .map(|(n, r, w)| {
                let def = match r {
                    Some(Reduction::Parsed(f)) => Some(f),
                    // Refused: E may still have an edge, the splice may not.
                    _ => None,
                };
                (n, def, w)
            })
            .collect();
        Self { empty, rows }
    }

    /// [`TuContext::of_named`] over `IlFunction::mangled_name` with no per
    /// function optimization word — the binding `IlBundle::functions()` hands
    /// the emitter, where the TU's mode is already known to be uniform.
    ///
    /// **Do not reach for this from a census-row caller**, for the reason
    /// [`TuEmptyCallees::of`] gives: the census pairs names positionally.
    pub fn of(funcs: impl IntoIterator<Item = &'a IlFunction>) -> Self {
        Self::of_named(funcs.into_iter().map(|f| (f.mangled_name.as_str(), f, None)))
    }

    /// Mechanism E's context — [`elide`](crate::elide)'s and unchanged.
    pub fn empty_callees(&self) -> &TuEmptyCallees {
        &self.empty
    }

    /// **S5.** The one definition of `name` in this bundle, or `None` when zero
    /// or **more than one** row carries it.
    ///
    /// Two rows are refused rather than resolved to the first: the two spellings
    /// would have to be told apart by something this type does not have, and
    /// splicing the wrong one is a silent wrong-bytes emit. This is the same
    /// answer `TuEmptyCallees` gives a name whose definitions disagree.
    pub fn definition(&self, name: &str) -> Option<(&'a IlFunction, Option<u32>)> {
        let i = self.rows.binary_search_by(|(n, _, _)| (*n).cmp(name)).ok()?;
        if i > 0 && self.rows[i - 1].0 == name {
            return None;
        }
        if i + 1 < self.rows.len() && self.rows[i + 1].0 == name {
            return None;
        }
        Some((self.rows[i].1?, self.rows[i].2))
    }

    /// How many definitions the context can splice from — printed by
    /// diagnostics rather than inferred, the same reason [`TuEmptyCallees::len`]
    /// exists.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// True when the bundle contributed no definition at all.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

impl<'a> std::ops::Deref for TuContext<'a> {
    type Target = TuEmptyCallees;

    fn deref(&self) -> &TuEmptyCallees {
        &self.empty
    }
}

/// **S1–S5, S8, S9** — the part of the predicate that is a property of the
/// caller's own selection, with the callee it names.
///
/// Separated from [`splice_body`] because it is the half that can be asserted
/// without lowering anything, so a test can walk the boundary cells without an
/// IL bundle for the callee — and because `PortC2::build`'s refusal needs the
/// same question asked without paying for the callee's body.
///
/// `None` is the refusal, and it is the answer for every shape this rule was not
/// graded on. Nothing here is an assertion about the parser's invariants: an
/// [`IlFunction`] is a parse result and this is a predicate over it.
pub fn splice_callee<'a>(
    f: &IlFunction,
    selected: &Selected,
    tu: &TuContext<'a>,
) -> Option<&'a str> {
    // **S9.** Mechanism E answers first and keeps its `blr`. Asked before
    // anything else so that a reader meets the precedence at the top, and so
    // that the two rules can never both claim one function.
    if drops_tail_call(f, tu.empty_callees()) {
        return None;
    }
    // **S8.** The caller's whole body is discarded; a data symbol of its own
    // would be discarded with it and no cell grades that.
    if f.data_sym.is_some() {
        return None;
    }
    // **S1 and S3.** `Framed` is 0 of 123 and `CondPair` is a conditional site
    // that was never graded; `Plain` and `Float` name no callee at all. What
    // survives is the two shapes whose whole emitted body is the call — which is
    // S3, and which is why the two clauses are one match.
    let callee: &str = match selected {
        // A tail call with a non-empty setup is SPLICE-P's `port_words > 1`
        // stratum: **0 of 953**, with 1,890 of them diverging at word 0.
        Selected::Tail(setup) if setup.is_empty() => f.tail_call.as_deref()?,
        Selected::Seq { setups, .. } => {
            let seq = f.call_seq.as_ref()?;
            // **S2.** One call site. SPLICE-N is 0 of 548.
            if seq.calls.len() != 1 || setups.len() != 1 {
                return None;
            }
            // **S3.** Nothing around the call: no argument setup, no guarded
            // block (which is also §2's *conditional site*), no early return.
            if !setups[0].is_empty() || seq.guard.is_some() || !seq.early.is_empty() {
                return None;
            }
            // **S3, the tail half.** The body may do nothing to r3 after the
            // call, because c2's inlined body is the callee's and carries no
            // epilogue of the caller's. Two tails qualify and both are the ABI
            // identity:
            //
            // * `CallValue { add_k: 0 }` — return what the call left in r3,
            //   with the `addi` elided at 0. Class A, so nothing is saved.
            // * `SavedFormal { param: 0 }` — the WEC shape, an empty
            //   constructor delegating to one base: `mr r31,r3 ; bl ?B ;
            //   mr r3,r31`. The formal handed back is the one that was already
            //   in r3 at entry, so with an empty setup the whole body is the
            //   call. This is the 634-function `seq` family and `t03` is its
            //   cell.
            //
            // Every other tail — a literal, a read-through, a comparison —
            // emits words of its own after the `bl`, and c2's inlined body would
            // have to carry them too. Not graded, so refused.
            match seq.tail {
                SeqTail::CallValue { add_k: 0 } if seq.saved.is_empty() => {}
                SeqTail::SavedFormal { param: 0 } if seq.saved.as_slice() == [0] => {}
                _ => return None,
            }
            seq.calls[0].callee.as_str()
        }
        _ => return None,
    };
    // **S4.** Self-recursion. c2 declines it too — `INLINE_PREDICATE.md` §4
    // grades the `recurse` family 336/336 — and a rule that took it would
    // splice a body into itself.
    if callee == f.mangled_name {
        return None;
    }
    // **S5.** Defined here, once. Returned as the context's own `&'a str` so the
    // spliced body's relocations outlive the caller's borrow.
    let (_, _) = tu.definition(callee)?;
    self_named(tu, callee)
}

/// The context's own copy of `name`, with the context's lifetime.
///
/// [`TuContext::definition`] hands back the definition; the *name* has to come
/// from the same place, because a spliced [`ComdatBody`] carries `&str`
/// relocation targets that outlive the caller's borrow of its own callee field.
fn self_named<'a>(tu: &TuContext<'a>, name: &str) -> Option<&'a str> {
    let i = tu.rows.binary_search_by(|(n, _, _)| (*n).cmp(name)).ok()?;
    Some(tu.rows[i].0)
}

/// **THE MECHANISM.** The caller's `/Gy` COMDAT body, when it is the callee's.
///
/// `Ok(None)` is the refusal — the predicate declined, and the caller emits its
/// ordinary body. `Err` is the callee's own decline propagated: it cannot happen
/// through S6, which treats an uncomposable callee as a refusal, and the arm
/// exists so that a future edit which stops doing that goes red rather than
/// silent.
///
/// The recursion into the callee is **one level**: `allow_splice` is `false`, so
/// `body(G)` is `G`'s own lowering and never `G`'s callee's. Whether c2 closes
/// the chain is `t11`'s measurement and `work/w-splice/` records what it said;
/// the port takes one level in this rung either way.
pub fn splice_body<'a>(
    f: &IlFunction,
    selected: &Selected,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> Result<Option<ComdatBody<'a>>, ComdatDecline> {
    let Some(callee) = splice_callee(f, selected, tu) else {
        return Ok(None);
    };
    // **S7, the varargs half.** `N_max = 0` categorically (§6.18.5). MSVC
    // terminates a varargs argument list with `Z` where an ordinary one ends
    // `@`, so the mangled name ends `ZZ` — read off the name because that is
    // where §2's table says it is readable, IL side and obj side alike.
    if callee.ends_with("ZZ") {
        return Ok(None);
    }
    let Some((g, opt_word)) = tu.definition(callee) else {
        return Ok(None);
    };
    // The callee's own mode. A callee under a different `#pragma optimize` than
    // its caller allocates a chain intermediate to a different register
    // (`OptMode`'s doc), so splicing across that boundary would emit the wrong
    // register field. `None` is "the caller does not track it per function", and
    // then the TU has already been refused unless every function shares a mode.
    let g_mode = match opt_word {
        Some(_) => match opt_mode_of_word(opt_word) {
            Ok(m) => m,
            Err(_) => return Ok(None),
        },
        None => mode,
    };
    if g_mode != mode {
        return Ok(None);
    }
    // **S6.** The port must have a body for the callee — `t09` is the cell where
    // it does not — and that body is built with the splice DISABLED, which is
    // the one-level restriction.
    let Ok(g_sel) = crate::codegen::select_function(g, g_mode) else {
        return Ok(None);
    };
    let Ok(body) = crate::comdat::body_of(g, g_sel, g_mode, tu, false) else {
        return Ok(None);
    };
    // **S6, the frame half.** A framed callee carries a prologue, an epilogue
    // and a `.pdata` record whose association is a property of the function it
    // belongs to. Splicing one into a caller is a cell nobody graded.
    if body.frame.is_some() {
        return Ok(None);
    }
    // **S7.** The inline decision, on the side of its own boundary where it is
    // categorical in both linkage classes.
    if body.text.is_empty() || body.text.len() > INLINE_UNBOUNDED_BYTES {
        return Ok(None);
    }
    Ok(Some(body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::select_function;
    use crate::codegen::testutil::func_with;
    use c2_il::{CallSeq, IlOp, SeqCall};

    /// `int g(int a) { return a + 1; }` — a leaf the port lowers, 8 bytes.
    fn leaf(name: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add]);
        f.mangled_name = name.into();
        f.data_sym = None;
        f
    }

    /// `int f(int a) { return g(a); }` — an empty setup and one branch word.
    fn tail(name: &str, callee: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], Vec::new());
        f.mangled_name = name.into();
        f.data_sym = None;
        f.tail_call = Some(callee.into());
        f
    }

    fn ctx<'a>(funcs: &'a [IlFunction]) -> TuContext<'a> {
        TuContext::of(funcs)
    }

    fn fires(funcs: &[IlFunction], i: usize) -> bool {
        let tu = ctx(funcs);
        let sel = select_function(&funcs[i], OptMode::O1).unwrap();
        splice_callee(&funcs[i], &sel, &tu).is_some()
    }

    fn spliced(funcs: &[IlFunction], i: usize) -> Option<Vec<u8>> {
        let tu = ctx(funcs);
        let sel = select_function(&funcs[i], OptMode::O1).unwrap();
        splice_body(&funcs[i], &sel, OptMode::O1, &tu)
            .unwrap()
            .map(|b| b.text)
    }

    /// `t01` — the positive cell. The caller's body **is** the callee's, and the
    /// caller acquires no relocation of its own.
    #[test]
    fn an_empty_setup_tail_call_takes_the_callees_body() {
        let funcs = vec![leaf("?g@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        let body = splice_body(&funcs[1], &sel, OptMode::O1, &tu)
            .unwrap()
            .expect("S1-S9 hold: this is t01");
        let g_sel = select_function(&funcs[0], OptMode::O1).unwrap();
        let g_body = crate::comdat::comdat_body_from_selected(&funcs[0], g_sel, OptMode::O1, &tu)
            .unwrap();
        assert_eq!(body.text, g_body.text, "the caller's body is the callee's");
        assert!(
            body.calls.is_empty(),
            "THE CALLER ACQUIRED A RELOCATION AGAINST ITS CALLEE: the whole \
             mechanism is that it does not — board #882, and a leaf callee has \
             no relocation of its own to inherit"
        );
        assert!(body.frame.is_none());
    }

    /// **S3** — `t04`/`t05`/`t06`. A setup means c2 rewrote a field of the
    /// callee's body (a register rename, a displacement fold), and w-seq graded
    /// the concatenation **0 of 953** there.
    #[test]
    fn a_tail_call_with_an_argument_setup_is_refused() {
        // `int f(int a) { return g(a + 1); }` — the setup is one `addi`.
        let mut caller = tail("?f@@YAHH@Z", "?g@@YAHH@Z");
        caller.ops = vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add];
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        assert!(
            !fires(&funcs, 1),
            "SPLICE-P IS 0 OF 953 WITH A SETUP: 1,890 of those failures diverge \
             at word 0, which is the port's own first setup word"
        );
    }

    /// **S5** — `t14`, the control. A callee this bundle does not define is the
    /// ordinary tail call, relocation and all.
    #[test]
    fn a_callee_this_bundle_does_not_define_is_refused() {
        let funcs = vec![tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 0));
        // …and a definition of a DIFFERENT name must not admit it either.
        let funcs = vec![leaf("?other@@YAHH@Z"), tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 1));
    }

    /// **S5, the ambiguity half.** Two definitions of one name admit neither:
    /// telling them apart needs something this context does not have.
    #[test]
    fn a_name_defined_twice_is_refused() {
        let mut g2 = leaf("?g@@YAHH@Z");
        g2.ops = vec![IlOp::Load(0xE309), IlOp::Lit(2), IlOp::Add];
        let funcs = vec![leaf("?g@@YAHH@Z"), g2, tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(!fires(&funcs, 2));
    }

    /// **S4** — `t12`. A body spliced into itself is not a fixpoint, it is a
    /// loop; c2 declines self-recursion too (`recurse`, 336/336).
    #[test]
    fn self_recursion_is_refused() {
        let funcs = vec![tail("?r@@YAHH@Z", "?r@@YAHH@Z")];
        assert!(!fires(&funcs, 0));
    }

    /// **S9** — mechanism E answers first. An empty-bodied callee is `elide`'s
    /// and this rule must not claim it, or one function has two rules.
    #[test]
    fn mechanism_e_keeps_its_own_callers() {
        let mut g = leaf("?g@@YAXXZ");
        g.ops = Vec::new();
        g.params = Vec::new();
        g.empty_body = true;
        let mut caller = tail("?f@@YAXXZ", "?g@@YAXXZ");
        caller.params = Vec::new();
        let funcs = vec![g, caller];
        let tu = ctx(&funcs);
        assert!(
            tu.reduces_to_nothing("?g@@YAXXZ"),
            "the E context still resolves through the composite"
        );
        assert!(
            !fires(&funcs, 1),
            "MECHANISM E AND MECHANISM I BOTH CLAIMED ONE FUNCTION: E is asked \
             first and keeps its blr"
        );
    }

    /// **S8** — a caller that materializes a data symbol. Its whole body is
    /// discarded and the data reference would go with it.
    #[test]
    fn a_caller_with_a_data_symbol_is_refused() {
        let mut caller = tail("?f@@YAHH@Z", "?g@@YAHH@Z");
        caller.data_sym = Some("?gv@@3HA".into());
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        assert!(!fires(&funcs, 1));
    }

    /// **S7, the varargs half.** `N_max = 0` categorically, and the mangled name
    /// is where §2's table says the bit is readable.
    #[test]
    fn a_varargs_callee_is_refused() {
        let mut g = leaf("?g@@YAHHZZ");
        g.mangled_name = "?g@@YAHHZZ".into();
        let funcs = vec![g, tail("?f@@YAHH@Z", "?g@@YAHHZZ")];
        // S1-S5 hold — the refusal is S7's and only S7's.
        assert!(fires(&funcs, 1), "the structural half of the predicate holds");
        assert!(
            spliced(&funcs, 1).is_none(),
            "A VARARGS CALLEE WAS SPLICED: N_max is 0 for it in both linkage \
             classes (INLINE_PREDICATE.md §2, §6.18.5)"
        );
    }

    /// **S6** — a callee the port cannot lower has no body to splice, so the
    /// caller keeps its branch. The structural half still holds, which is why
    /// the two halves are separate functions.
    #[test]
    fn an_unlowerable_callee_leaves_the_branch_alone() {
        let mut g = leaf("?g@@YAHH@Z");
        g.ops = vec![IlOp::Load(0xE309), IlOp::Load(0xE409), IlOp::Div];
        let funcs = vec![g, tail("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(fires(&funcs, 1));
        assert!(
            spliced(&funcs, 1).is_none(),
            "the port has no bytes for this callee; splicing would emit a body \
             it cannot produce"
        );
    }

    /// **BOARD #980's BOUNDARY — a `NoEffectCall` row is E's and NEVER the
    /// splice's.**
    ///
    /// Lane `w-inl0` feeds mechanism E edges from rows the IL parser **refused**:
    /// a refused body whose grammar still proves it emits nothing but a call to
    /// one callee contributes `Reduction::NoEffectCall`. E can close through
    /// such a node because closing needs only *"does this emit anything"*.
    ///
    /// The splice cannot, and the reason is S6 rather than a policy: its rule is
    /// *"the caller's body IS the callee's body"*, and there is no body — the
    /// parser refused it. So [`TuContext::definition`] returns `None` for those
    /// rows and the walk refuses with `S6-callee-parse-refused`.
    ///
    /// **But the row must still be VISIBLE.** `?g` is a name this TU defines,
    /// and `mentions` says so, which is what `S6-chain-truncated` reads to tell
    /// a chain that *ended* from one the port could not *follow*. A resolution
    /// that dropped refused rows from the context would make `?g` read as an
    /// external — and running a splice off the end of a chain the port cannot
    /// see is the wrong-relocation defect this lane already closed once.
    #[test]
    fn a_no_effect_call_row_feeds_e_and_never_the_splice() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let mut caller = tail("?f@@YAXXZ", "?g@@YAXXZ");
        caller.params = Vec::new();
        // `?g` is REFUSED by the parser — there is no `IlFunction` for it at
        // all — and carries only the edge board #980 reads out of its grammar.
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", Some(Reduction::NoEffectCall("?h@@YAXXZ")), None),
            ("?f@@YAXXZ", Some(Reduction::Parsed(&caller)), None),
        ]);

        // E closes through the refused node — that is #980's whole content.
        assert!(
            tu.reduces_to_nothing("?g@@YAXXZ"),
            "BOARD #980 REGRESSED: mechanism E must close through a refused row \
             that carries a NoEffectCall edge"
        );

        // The splice must not be able to reach it as a body...
        assert!(
            tu.definition("?g@@YAXXZ").is_none(),
            "A REFUSED BODY WAS OFFERED TO THE SPLICE: S6 needs a COMPOSED body \
             for the chain's end and the parser refused this one"
        );
        // ...and must still be CARRIED by the context. `mentions()` is what
        // reads this row, and it arrives with `S6-chain-truncated`; what this
        // commit can assert is that the row is in the table at all, which is
        // the property that clause will depend on.
        assert_eq!(
            tu.len(),
            3,
            "A REFUSED ROW VANISHED FROM THE CONTEXT: it would later read as an \
             external, S6-chain-truncated would stop firing, and the splice \
             would run off the end of a chain it cannot see"
        );

        // And the two mechanisms still do not both claim `?f`: E takes it.
        let sel = select_function(&caller, OptMode::O1).unwrap();
        assert!(
            crate::elide::drops_tail_call(&caller, tu.empty_callees()),
            "?f tail-calls a name that reduces to nothing, so E answers"
        );
        assert!(
            splice_callee(&caller, &sel, &tu).is_none(),
            "S9: mechanism E is asked first and keeps its blr"
        );
    }

    /// A refused row with **no** readable `no_effect_callee` feeds neither
    /// mechanism and is still visible. That is the third value of the
    /// constructor's `Option<Reduction>`, and it is the majority of the refused
    /// population.
    #[test]
    fn a_refused_row_with_no_edge_is_visible_and_nothing_else() {
        let mut h = leaf("?h@@YAXXZ");
        h.ops = Vec::new();
        h.params = Vec::new();
        h.empty_body = true;
        let tu = TuContext::of_rows(vec![
            ("?h@@YAXXZ", Some(Reduction::Parsed(&h)), None),
            ("?g@@YAXXZ", None, None),
        ]);
        assert_eq!(tu.len(), 2, "still a name this TU defines");
        assert!(tu.definition("?g@@YAXXZ").is_none());
        assert!(
            !tu.reduces_to_nothing("?g@@YAXXZ"),
            "#980 IS CONSERVATIVE: a refused row with nothing readable \
             contributes NO edge to the closure"
        );
    }

    /// **S2** — `t08`. Two call sites is SPLICE-N, **0 of 548**.
    #[test]
    fn a_two_call_body_is_refused() {
        let mut caller = func_with(Vec::new(), Vec::new());
        caller.mangled_name = "?f@@YAXXZ".into();
        caller.data_sym = None;
        caller.call_seq = Some(CallSeq {
            calls: vec![
                SeqCall {
                    callee: "?g@@YAXXZ".into(),
                    arg_ops: Vec::new(),
                    arg_sources: None,
                    link_args: None,
                },
                SeqCall {
                    callee: "?g@@YAXXZ".into(),
                    arg_ops: Vec::new(),
                    arg_sources: None,
                    link_args: None,
                },
            ],
            tail: SeqTail::Void,
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
        });
        let mut g = leaf("?g@@YAXXZ");
        g.params = Vec::new();
        g.ops = Vec::new();
        g.empty_body = false;
        g.compare = None;
        let funcs = vec![g, caller];
        let tu = ctx(&funcs);
        if let Ok(sel) = select_function(&funcs[1], OptMode::O1) {
            assert!(
                splice_callee(&funcs[1], &sel, &tu).is_none(),
                "SPLICE-N IS 0 OF 548: a two-call body is not the concatenation \
                 of its callees and it is not either of them"
            );
        }
    }

    /// **S3, the `Seq` tail half.** A `Seq` whose tail does work of its own
    /// after the `bl` is refused: c2's inlined body would have to carry those
    /// words too, and no cell graded that.
    #[test]
    fn a_seq_with_a_working_tail_is_refused() {
        let mut caller = func_with(vec![0xE309], Vec::new());
        caller.mangled_name = "?f@@YAHH@Z".into();
        caller.data_sym = None;
        caller.call_seq = Some(CallSeq {
            calls: vec![SeqCall {
                callee: "?g@@YAHH@Z".into(),
                arg_ops: Vec::new(),
                arg_sources: None,
                link_args: None,
            }],
            // `return g(a) + 5;` — one `addi` after the call.
            tail: SeqTail::CallValue { add_k: 5 },
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
        });
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        let tu = ctx(&funcs);
        if let Ok(sel) = select_function(&funcs[1], OptMode::O1) {
            assert!(splice_callee(&funcs[1], &sel, &tu).is_none());
        }
    }

    /// **THE ONE-LEVEL RESTRICTION.** `h` is lowerable, `g` splices `h`, and
    /// `f` splices **`g`'s own lowering** — a branch to `?g`… no: `g`'s own
    /// lowering is a branch to `?h`, so `f` gets that. What `f` must NOT get is
    /// `h`'s body, because nothing in this rung measured whether c2 closes the
    /// chain (`t11`).
    #[test]
    fn the_splice_takes_exactly_one_level() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g@@YAHH@Z"),
        ];
        let h = spliced(&funcs, 1).expect("?g splices ?h");
        let f = spliced(&funcs, 2).expect("?f splices ?g's own lowering");
        assert_eq!(h.len(), 8, "?h is `addi r3,r3,1 ; blr`");
        assert_eq!(
            f,
            vec![0x48, 0x00, 0x00, 0x00],
            "THE SPLICE CLOSED A CHAIN IT WAS NOT GRADED ON: ?f must get ?g's \
             OWN lowering — one branch word — and never ?h's body. Whether c2 \
             closes it is work/w-splice/'s t11 and is a separate rung"
        );
    }

    /// The spliced body inherits the **callee's** relocations, at the callee's
    /// own offsets, and never one against the callee. `t10` is the compiled
    /// cell; this is the same statement where a test can hold it.
    #[test]
    fn the_spliced_body_inherits_the_callees_relocations() {
        let funcs = vec![
            tail("?g@@YAXXZ", "?ext@@YAXXZ"),
            tail("?f@@YAXXZ", "?g@@YAXXZ"),
        ];
        let tu = ctx(&funcs);
        let sel = select_function(&funcs[1], OptMode::O1).unwrap();
        let body = splice_body(&funcs[1], &sel, OptMode::O1, &tu)
            .unwrap()
            .expect("?g is defined here, lowerable and unframed");
        assert_eq!(body.text, vec![0x48, 0x00, 0x00, 0x00]);
        let names: Vec<&str> = body.calls.iter().map(|c| c.callee).collect();
        assert_eq!(
            names,
            vec!["?ext@@YAXXZ"],
            "BOARD #882: both bodies are the word 48000000 and only the \
             relocation tells them apart. ?f must relocate against ?ext — the \
             callee's own target — and never against ?g"
        );
        assert_eq!(body.calls[0].reloc_offset, 0);
    }

    /// A name defined once resolves; the ambiguity guard must not reject the
    /// *first* or *last* row of the sorted table by walking off it.
    #[test]
    fn definition_lookup_handles_the_table_edges() {
        let funcs = vec![
            leaf("?a@@YAHH@Z"),
            leaf("?m@@YAHH@Z"),
            leaf("?z@@YAHH@Z"),
        ];
        let tu = ctx(&funcs);
        for n in ["?a@@YAHH@Z", "?m@@YAHH@Z", "?z@@YAHH@Z"] {
            assert!(tu.definition(n).is_some(), "{n} must resolve");
        }
        assert!(tu.definition("?nope@@YAHH@Z").is_none());
    }

    /// An unnamed definition cannot be a callee — the same answer `elide` gives.
    #[test]
    fn an_unnamed_definition_is_never_a_splice_source() {
        let mut g = leaf("");
        g.mangled_name = String::new();
        let funcs = [g];
        let tu = TuContext::of(&funcs);
        assert_eq!(tu.len(), 0);
        assert!(tu.definition("").is_none());
    }
}

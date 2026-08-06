//! **Mechanism E** — the call c2 emits no branch, no relocation and no external
//! symbol for, because its callee's body does nothing.
//!
//! # What this is, and what it is NOT
//!
//! `docs/INLINE_PREDICATE.md` separates two reasons an IL call can be absent
//! from c2's output, and **only one of them is a cost model**:
//!
//! | | mechanism | governed by `/Ob`? | cost model? |
//! |---|---|---|---|
//! | **E** | the callee's body is empty, so the call is dropped | **no** | **no — syntactic** |
//! | I | the inliner expanded the callee | yes | yes |
//!
//! This module is **E and nothing else**. It reads no size, no linkage, no
//! `inline` keyword and no call count; `INLINE-P`'s `index`/`N_max` do not
//! appear here and must not. A rung that wants mechanism I needs the callee's
//! own *emitted* size (`INLINE_PREDICATE.md` §6.2), which is a different — and
//! much more expensive — ordering constraint.
//!
//! # The predicate, exactly as it was measured
//!
//! [`drops_tail_call`] is the whole rule. It fires for a function the selector
//! classified [`crate::codegen::Selected::Tail`] when **all** of:
//!
//! 1. **same-TU** — some function in *this* IL bundle is named by the tail
//!    call's callee;
//! 2. **emptiness** — that function's IL body decodes as
//!    [`c2_il::IlFunction::empty_body`], and no other definition of the same
//!    name in this bundle disagrees;
//! 3. **no data reference** — the caller materializes no named data symbol.
//!
//! and the body it licenses is the single word `blr`: **the argument setup goes
//! too.** That is not an inference. `work/w-empty/`'s GRID-1 (32 cells, stamp
//! `fea9877e`) and GRID-2 (8 cells, stamp `187e899a`) compile every cell twice
//! against real `c2` under wibo — once at the workload's flags and once with
//! `/Ob0` appended, so that "no REL24" cannot confuse E with I — and in **29 of
//! the 30** cells graded `E` the caller's whole `.text` COMDAT is one
//! `4e800020`, whatever the setup would have been: a register permutation
//! (`f02_perm`, 4 port words), an arithmetic argument (`f03_expr_arg`, 2), a
//! literal (`g05_const_arg`, 2), an FP argument (`g02_float_arg`), three
//! formals (`g06_three_args`), a global's address (`g01_data_addr_arg`).
//!
//! **The thirtieth is `f05_side_effect_arg` and it is stated rather than
//! rounded away.** `int sink; void g(int a){} void f(){ g(sink++); }` is E — the
//! call vanishes — and the caller keeps **four** words, because `sink++` has a
//! side effect that survives its argument. So the shipped body is *"one `blr`"*
//! only for a setup that is a pure computation over formals and literals, which
//! every `Selected::Tail` setup is by construction. The port **refuses** `f05`'s
//! caller outright (`expr-call-in-expr-op-0x35`), so no shipped rule depends on
//! that being true; if a later widening admits a side-effecting argument setup
//! into `Selected::Tail`, this rule must be re-graded before it may fire there.
//!
//! # Why each condition is there, with the cell that put it there
//!
//! * **Condition 1 is not decoration.** `c22_extern_callee` — the same call to a
//!   `?g` this TU does *not* define — keeps its REL24 at both flag settings.
//!   Dropping the same-TU test turns that into a wrong emit.
//! * **Condition 2 is the one c2 and the port do not agree about, and the
//!   disagreement is deliberately in the safe direction.** c2 applies E after
//!   its own dead-code elimination: `void g(int a){ int x = a; }`,
//!   `void g(int a){ if(a){} }`, `void g(){ return; }` and an empty `for` loop
//!   are all E (`c03`, `c15`, `c14`, `c05`), and the IL parser refuses all four,
//!   so this predicate does not fire on them and the port keeps its branch. The
//!   port under-fires; it does not guess.
//! * **Condition 2 also refuses the discriminator.** `int g(int a){ return a; }`
//!   emits the identical single `blr` as `void g(){}` **and is mechanism I**
//!   (`c19_ret_param`: no REL24 at `/O1`, one at `/Ob0`). At the workload's own
//!   flags its caller is observationally identical to an E caller — one `blr`
//!   word — so a rule fitted to the *bytes* would take it and a rule keyed on
//!   the *body* does not. This one declines a match it could have had, on
//!   purpose: `docs/INLINE_PREDICATE.md` §7 leaves I's residual `NOT MODELLED`,
//!   and 2.8 % of a wrong guess is a wrong emit.
//! * **Condition 3 refuses a cell nobody graded.** `g01_data_addr_arg` is E in
//!   c2, and the port's IL parser refuses its caller outright, so no graded cell
//!   exercises an elided tail call that also materializes a data symbol. Rather
//!   than let the workload be the first case, the predicate declines it — and
//!   `data_refs_of` would in any event fail to locate a relocation half inside a
//!   one-word `blr`.
//!
//! # THE HAZARD, stated where the next lane will read it
//!
//! > **E is a property of the call SITE as well as of the callee, and today the
//! > site condition is enforced by the IL parser and not by this file.**
//!
//! `f09_fnptr` — `void g(){} void f(){ void(*p)()=g; p(); }` — has an
//! `empty-body` callee and c2 emits `b ?g` **with a REL24**: E does not fire
//! through a function pointer, exactly as `INLINE_PREDICATE.md` §2 says of an
//! indirect site from mechanism I's side. The port is safe only because
//! `expr-call-in-expr-data-addr-then-plain-call-whole` is a parse refusal, and
//! `f10_virtual_ptr` likewise (`body-0x67`). **If either production is ever
//! accepted and lowered to a `tail_call`, this predicate becomes a wrong emit**
//! — board #232's shape. `crates/c2-harness/tests/empty_elision.rs` pins both
//! refusals against the real toolchain so that widening one turns a test red in
//! the same commit.

use c2_il::IlFunction;

/// The **same-TU empty-bodied callees** of one IL bundle: condition 1 and
/// condition 2 of [`drops_tail_call`], resolved once per TU instead of once per
/// call.
///
/// A name whose bundle carries *two* definitions, one empty and one not, is
/// **not** admitted — the two spellings would have to be told apart by something
/// this type does not have, and refusing is the honest answer. (The gate's own
/// name binding makes that impossible today; representing it beats asserting it.)
#[derive(Debug, Default, Clone)]
pub struct TuEmptyCallees {
    /// Sorted, deduplicated mangled names, every one of which is defined in this
    /// bundle **and** decodes as an empty body.
    empty: Vec<String>,
}

impl TuEmptyCallees {
    /// The context for a caller with no bundle: nothing is ever elided.
    ///
    /// Every production caller has a bundle; this exists so a test can state the
    /// "no elision" baseline without building one, and so the elision can never
    /// be reached by *forgetting* to pass a context — the parameter is required.
    pub fn none() -> Self {
        Self::default()
    }

    /// Collect the bundle's empty-bodied definitions, from
    /// **`(name, is_empty)`** pairs the caller has already bound.
    ///
    /// # The name binding is the CALLER's, and that is not a detail
    ///
    /// An IL bundle carries more than one function-name binding and **they
    /// disagree**. `crates/c2-il/src/func/bind.rs` says so in its own module
    /// doc; this lane measured the size of it, because the elision is a
    /// name-keyed fact and a name-keyed fact read through the wrong binding is
    /// attached to the wrong function:
    ///
    /// > **74,955** census rows of the dc3 workload carry an
    /// > `IlFunction::mangled_name` (paired *positionally* over `.ex` segments)
    /// > that differs from their `FnCensus::emit_name` (the per-record
    /// > emitted-symbol binding). `c2rs gap` prints that as
    /// > `fnbyte-name-disagree` on every scan.
    ///
    /// Keyed on the positional name, this predicate fired **14** times on the
    /// workload — every one of them a previously byte-exact body turned wrong,
    /// and **zero** of family A's 1,886 reached. Keyed on the binding the caller
    /// already trusts for this same population, it is right. So the name is an
    /// argument here rather than a field read off `IlFunction`: each caller
    /// passes the binding it uses for everything else about that row, and the
    /// elision cannot disagree with the instrument that grades it.
    ///
    /// [`TuEmptyCallees::of`] is the convenience for a caller whose binding *is*
    /// `mangled_name` — `IlBundle::functions()`, where the gate's own in-TU
    /// callee refusal already compares resolved callee names against that same
    /// list.
    /// Sorted once and scanned in runs rather than searched per row: a workload
    /// TU carries thousands of census rows, and a quadratic membership test here
    /// would be paid on every TU of every scan.
    pub fn of_named<'a>(named: impl IntoIterator<Item = (&'a str, bool)>) -> Self {
        let mut rows: Vec<(&str, bool)> =
            named.into_iter().filter(|(n, _)| !n.is_empty()).collect();
        rows.sort_unstable();
        let mut empty: Vec<String> = Vec::new();
        let mut i = 0;
        while i < rows.len() {
            let name = rows[i].0;
            let mut j = i;
            let mut all_empty = true;
            while j < rows.len() && rows[j].0 == name {
                all_empty &= rows[j].1;
                j += 1;
            }
            // A name defined twice with bodies that disagree about emptiness
            // admits neither: telling the two spellings apart needs something
            // this type does not have, and refusing is the honest answer.
            if all_empty {
                empty.push(name.to_string());
            }
            i = j;
        }
        Self { empty }
    }

    /// [`TuEmptyCallees::of_named`] over `IlFunction::mangled_name` — the
    /// binding `IlBundle::functions()` hands the emitter, and the one its own
    /// in-TU-callee refusal already compares resolved callee names against.
    ///
    /// **Do not reach for this from a census-row caller.** The census pairs
    /// names positionally and disagrees with the emitted-symbol binding on
    /// 74,955 workload rows; `of_named` exists so that caller can pass the name
    /// it actually trusts.
    pub fn of<'a>(funcs: impl IntoIterator<Item = &'a IlFunction>) -> Self {
        Self::of_named(
            funcs
                .into_iter()
                .map(|f| (f.mangled_name.as_str(), f.empty_body)),
        )
    }

    /// Is `name` defined in this bundle with an empty body?
    pub fn is_empty_callee(&self, name: &str) -> bool {
        self.empty.binary_search_by(|p| p.as_str().cmp(name)).is_ok()
    }

    /// How many names the context admits — printed by diagnostics rather than
    /// inferred from a behaviour, so "the context was built and found nothing"
    /// and "the context was never built" are different observations.
    pub fn len(&self) -> usize {
        self.empty.len()
    }

    /// True when no name is admitted.
    pub fn is_empty(&self) -> bool {
        self.empty.is_empty()
    }
}

/// **The predicate.** Does c2 drop this function's tail call?
///
/// Consult it *only* for a function [`crate::codegen::select_function`]
/// classified [`crate::codegen::Selected::Tail`] — `framed_call`, `call_seq` and
/// `cond_pair` carry their callees in their own fields and are not modeled here
/// (`f06_two_calls` and `f08_mixed` grade E in c2 and are `Selected::Seq` in the
/// port; both keep today's behaviour and stay `fnbyte-differs`, which is the
/// honest answer and not a silent one).
///
/// The three conditions, in the order the module docs give them.
pub fn drops_tail_call(f: &IlFunction, tu: &TuEmptyCallees) -> bool {
    let Some(callee) = f.tail_call.as_deref() else {
        return false;
    };
    // Condition 3 — a caller that materializes a named data symbol is a cell no
    // grid graded. See the module docs.
    if f.data_sym.is_some() {
        return false;
    }
    // Conditions 1 and 2, both resolved in `TuEmptyCallees::of`.
    tu.is_empty_callee(callee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::testutil::func_with;

    fn named(name: &str) -> IlFunction {
        let mut f = func_with(Vec::new(), Vec::new());
        f.mangled_name = name.into();
        f.data_sym = None;
        f
    }

    fn empty(name: &str) -> IlFunction {
        let mut f = named(name);
        f.empty_body = true;
        f
    }

    fn tail_caller(name: &str, callee: &str) -> IlFunction {
        let mut f = named(name);
        f.tail_call = Some(callee.to_string());
        f
    }

    /// `c00_empty` — the cell the whole rule is built on.
    #[test]
    fn a_same_tu_empty_callee_is_elided() {
        let funcs = vec![empty("?g@@YAXXZ"), tail_caller("?f@@YAXXZ", "?g@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 1);
        assert!(drops_tail_call(&funcs[1], &tu));
    }

    /// **Condition 1.** `c22_extern_callee`: the same call, to a callee this TU
    /// does not define, keeps its REL24 in the real obj at both flag settings.
    #[test]
    fn a_callee_this_tu_does_not_define_is_not_elided() {
        let funcs = vec![tail_caller("?f@@YAXXZ", "?g@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 0, "no definition, so no name is admitted");
        assert!(
            !drops_tail_call(&funcs[0], &tu),
            "dropping the SAME-TU condition would elide a call to an external"
        );
        // …and a same-TU definition of a DIFFERENT name must not admit it either.
        let funcs = vec![empty("?other@@YAXXZ"), tail_caller("?f@@YAXXZ", "?g@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert!(!drops_tail_call(&funcs[1], &tu));
    }

    /// **Condition 2.** `c19_ret_param` / `c21_ret_plus1`: a callee whose body
    /// is not empty is mechanism I or an ordinary call, never E.
    #[test]
    fn a_same_tu_non_empty_callee_is_not_elided() {
        let mut g = named("?g@@YAXXZ");
        g.tail_call = Some("?ext@@YAXXZ".into());
        let funcs = vec![g, tail_caller("?f@@YAXXZ", "?g@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 0);
        assert!(
            !drops_tail_call(&funcs[1], &tu),
            "dropping the EMPTINESS condition would elide every same-TU call"
        );
    }

    /// **Condition 3.** A caller that materializes a named data symbol is
    /// declined: `g01_data_addr_arg` is the cell, and no grid graded a `Tail`
    /// that survives it.
    #[test]
    fn a_caller_with_a_data_symbol_is_not_elided() {
        let mut caller = tail_caller("?f@@YAXXZ", "?g@@YAXXZ");
        caller.data_sym = Some("?gv@@3HA".into());
        let funcs = vec![empty("?g@@YAXXZ"), caller];
        let tu = TuEmptyCallees::of(&funcs);
        assert!(tu.is_empty_callee("?g@@YAXXZ"));
        assert!(!drops_tail_call(&funcs[1], &tu));
    }

    /// A function with no tail call at all is never asked twice: the predicate
    /// answers `false` rather than looking at `framed_call` or `call_seq`.
    #[test]
    fn a_function_with_no_tail_call_is_never_elided() {
        let funcs = vec![empty("?g@@YAXXZ"), named("?f@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert!(!drops_tail_call(&funcs[1], &tu));
    }

    /// Two definitions of one name that disagree about emptiness admit neither.
    #[test]
    fn a_name_defined_twice_with_different_bodies_is_refused() {
        let mut g2 = named("?g@@YAXXZ");
        g2.tail_call = Some("?ext@@YAXXZ".into());
        let funcs = vec![empty("?g@@YAXXZ"), g2, tail_caller("?f@@YAXXZ", "?g@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 0);
        assert!(!drops_tail_call(&funcs[2], &tu));
    }

    /// An unnamed definition cannot be a callee: it contributes nothing rather
    /// than admitting the empty string.
    #[test]
    fn an_unnamed_definition_admits_nothing() {
        let mut g = func_with(Vec::new(), Vec::new());
        g.mangled_name = String::new();
        g.empty_body = true;
        g.data_sym = None;
        let tu = TuEmptyCallees::of(&[g]);
        assert_eq!(tu.len(), 0);
        assert!(!tu.is_empty_callee(""));
    }
}

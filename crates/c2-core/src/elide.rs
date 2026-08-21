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
//! appear here and must not.
//!
//! > **Mechanism I is shipped too, since 2026-08-08, and it lives in
//! > [`crate::splice`]** — where the callee's *emitted* size does appear, as the
//! > one clause that pays `INLINE_PREDICATE.md` §6.2's ordering constraint. The
//! > two modules are asked in a fixed order and cannot both claim a function:
//! > [`drops_tail_call`] is consulted **first**, and `splice.rs`'s S9 declines
//! > whatever it takes. Nothing below changed when I shipped —
//! > `fnbyte-elided` / `-elided-exact` read 1,516 / 1,516 at both ends — and
//! > that is the control, not the assumption.
//!
//! # E is performed by **c2**, not by the front end — and that is why the port can see it
//!
//! `INLINE_PREDICATE.md` calls E *"the front end dropping a call"*, on the
//! strength of its being unaffected by `/Ob0`. **The IL says otherwise, and the
//! IL is what this module reads.** For `void g(){} void f(){ g(); }` at the
//! workload's flags, `c2rs census` reports three functions and calls `?f`
//! **`void-tail-call`** — the call is *in* the `.ex` stream c2 consumes, which
//! is exactly why the port emits a branch for it and why family A exists at all.
//! Whatever `/Ob` does or does not govern, the elision happens **behind** the IL
//! seam, in the same place `docs/GAPS.md` already puts c2's constant folding and
//! its inlining. Had c1xx dropped the call, there would be nothing here to
//! model.
//!
//! # The predicate, exactly as it was measured
//!
//! [`drops_tail_call`] is the whole rule. It fires for a function the selector
//! classified [`crate::codegen::Selected::Tail`] when **all** of:
//!
//! 1. **same-TU** — some function in *this* IL bundle is named by the tail
//!    call's callee;
//! 2. **the callee reduces to nothing** — that function's IL body decodes as
//!    [`c2_il::IlFunction::empty_body`], **or** it is itself an elidable tail
//!    call to a callee that reduces to nothing (the FIXPOINT, below), and no
//!    other definition of the same name in this bundle disagrees;
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
//! # E IS A FIXPOINT — and this is the part `w-empty` measured on one cell
//!
//! ```cpp
//! void h() {}
//! void g() { h(); }     // source body NOT empty
//! void f() { g(); }     // c2 emits BOTH ?f and ?g as a bare blr
//! ```
//!
//! `INLINE_PREDICATE.md` §1's *"whose source body is empty"* is refuted by that
//! cell (board #920): the rule is *"the callee's body **reduces to nothing**"*,
//! closed under E itself. [`TuEmptyCallees`] is that closure — the **least**
//! fixpoint of
//!
//! * `empty_body(g)` ⟹ `g` reduces to nothing (the seed — and since board
//!   **#1053** a *refused* body whose grammar proves it emits nothing at all
//!   seeds too, [`Reduction::NoEffectNothing`]), and
//! * `g` is an **elidable step** — a tail call to `h`, with no data symbol and
//!   none of the other body kinds — and `h` reduces to nothing ⟹ so does `g`.
//!
//! **The step imposes the same conditions on a mid-chain node that
//! [`drops_tail_call`] imposes on the caller**, because it is the same question
//! asked one level down: *does c2 emit anything at all for this function*.
//!
//! ## What the grid says, and where it says the chain STOPS
//!
//! `work/w-fix/`'s GRID-3 (20 cells, stamp `5697f371…`), GRID-3b (8 cells,
//! `7dbdce71…`) and GRID-3c (6 cells, `a8bf0d57…`) compile **34 cells** twice
//! each against real `c2` under wibo — at the workload's flags and again with
//! `/Ob0` — and grade **94 call edges of 94**, per edge and never per cell.
//!
//! | | what | verdict |
//! |---|---|---|
//! | **fires** | an all-empty chain at depth **1, 2, 3, 4, 5, 6, 8** | every edge `E`, every caller one `4e800020` |
//! | **fires** | a mid-node with an FP argument · three formals · a permutation · an arithmetic argument at every link · `static` · `inline` · a member · a `virtual` member called qualified · defined below its use | `E`, caller a bare `blr` |
//! | **fires** | a destructor chain 2, 3 and 4 deep, and the constructor mirror — board #924's own `??1?$_Rb_tree_base@…` shape | `E` throughout |
//! | **STOPS** | a link whose body calls an **external** (`k5`/`k6`/`k7`, depth 1, 2, 3) | `I` at every edge at or above it: the caller keeps its REL24 at `/Ob0` |
//! | **STOPS** | a mid-node that keeps **bytes** — `g(sink++)` (`k15`), a store to a global (`k16`) | the mid-node's own call is `E` and **its caller's is `I`**: what propagates is *emits nothing*, not *elided its call* |
//! | **STOPS** | a `Seq` mid-node with one elidable and one surviving call (`m4`) | `I`. A `Seq` mid-node whose calls **all** elide (`k8`, `k9`) **is** `E` in c2 — and this rule declines it anyway, §"what it refuses" |
//! | **STOPS** | an indirect mid-site (`k19`) | the mid-node keeps its REL24 and its caller is `I` |
//! | **STOPS** | mechanism **I** mid-chain — `int m(int a){return a;}` (`k12`) | `I` at **both** edges, and **both callers are a bare `blr` at `/O1`** — observationally identical to E and separated only by `/Ob0`. This is `c19`'s trap one level up and the reason the step reads the *body*, not the bytes |
//! | **not E** | a two-node **cycle** (`k10`) and direct self-recursion (`k11`) | no member is a bare `blr`. `?a` and `?b` each emit one branch word; `?r` emits a self-branch that takes **no relocation at all** — so the relocation observable reads `E` on `r → r` while the body is plainly not nothing, which is why the grid prints the caller's whole text beside every verdict |
//!
//! ## Termination is structural, not a timeout
//!
//! The iteration admits a name only on a `false → true` transition, so a
//! productive round admits at least one of the bundle's finitely many names and
//! there can be at most that many productive rounds. **A cycle is never
//! *seeded*, so it is never admitted** — which is also what c2 does (`k10`,
//! `k11`), reached here by construction rather than by a special case.
//!
//! > **2026-08-08 — the seed set is no longer just `empty_body`, so that last
//! > sentence is RE-DERIVED rather than inherited.** Board **#1053** admits a
//! > *refused* body as a seed ([`Reduction::NoEffectNothing`]). Termination is
//! > unaffected — the argument above reads the step and not the seeds — and the
//! > cycle refusal survives for a reason that has to be stated: **a seed carries
//! > no link**, because the reader that mints one accepts only bodies with no
//! > call token in them, and cycle membership requires an outgoing link. The four
//! > steps are on [`Reduction`], with the edit that would collapse them named.
//!
//! A round **ceiling** backs that argument up: if the loop ever runs past
//! `names + 1` rounds the context reports [`TuEmptyCallees::overflowed`] and
//! **admits nothing at all**. It cannot fire as written — a test says so — and
//! it exists so that a future edit which breaks monotonicity *terminates and
//! goes red* instead of hanging a scan.
//!
//! # What this rule REFUSES, with the cell that graded it
//!
//! * **A `Seq` mid-node all of whose calls elide.** `k8` and `k9` grade it `E`
//!   in c2 — a two-call body over two empty callees *is* a bare `blr`. The step
//!   declines it because the port's own `Selected::Seq` emitter does not model
//!   E (`w-empty` §11.4), so admitting it would put "this function emits
//!   nothing" in the context while the emitter emits two calls for it. One rule,
//!   two answers, is how a refusal becomes a wrong emit later.
//! * **Everything c2 reaches through its own dead-code elimination.** `k13` —
//!   `int m(int a){return a;} void g1(int a){ m(a); } void f(int a){ g1(a); }` —
//!   is `E` at **both** edges in c2 and the port keeps both branches: `?m` is
//!   not `empty_body`, so nothing seeds. Board #922's population, one level up.
//! * **A mid-node that materializes a data symbol** (`k16`), for the reason
//!   condition 3 exists at all.
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

/// **What one definition contributes to the fixpoint** — and the reason there are
/// two variants is board **#980**.
///
/// E's step asks one question of a body: *does c2 emit anything for it, given
/// what its callee does*. Until 2026-08-08 the only body that could answer was
/// one the IL parser **accepted**, because the answer was read off
/// [`IlFunction`]. The 370 differs of board #980 are the case where the answer is
/// readable and the body is not accepted: its whole content is one discarded call
/// plus a temporary the body's own grammar proves nothing else reads
/// (`c2-il`'s `body::shapes::no_effect`, `FnCensus::no_effect_callee`).
///
/// So the input is a *reduction fact* rather than a parsed function, and the
/// variants are the ways a caller can have one. All of them feed the **same**
/// fixpoint, the same cycle refusal and the same round ceiling.
///
/// # 2026-08-08 — **A REFUSED BODY MAY NOW SEED, and the rule this doc used to
/// state is superseded** (board **#1053**)
///
/// Until today this paragraph read *"a refused body contributes a **link and
/// never a seed**"*, and lane `w-memset` stopped at that sentence with the price
/// of crossing it measured: **227** `fnbyte-differs` and **4,256** chain
/// extensions, all of them bottoming out at `??$__destroy_aux@…` — `p->~T()` on a
/// class with a trivial destructor, whose whole body is two discarded literals
/// with **no call in it at all**. `NoEffectCall(&str)` cannot express that, so no
/// chain through it could close however many productions above it were read.
///
/// [`Reduction::NoEffectNothing`] is that capability, and it is a **strictly
/// stronger claim** than a link: it asserts that a refused body emits nothing
/// *unconditionally*. It is admitted only for a body
/// `c2_il::…::no_effect::no_effect_nothing` reads with a **total** walk over a
/// **closed vocabulary containing no call token**, so the assertion is something
/// read out of the IL and never the absence of a successful parse. Seeding on
/// *"we failed to parse it"* would make every parser gap a silent elision, which
/// is board #232's direction — the one live wrong emit this project has shipped.
///
/// ## Termination and the cycle refusal, RE-DERIVED and not inherited
///
/// `w-fix` (#950) argued *"a cycle is never **seeded**, so it is never
/// admitted"*. That sentence was true because [`IlFunction::empty_body`] was the
/// only seed, so it does not survive a widening of the seed set on its own
/// authority. It is re-derived here, in four steps:
///
/// 1. **Termination is untouched.** The iteration admits a name only on a
///    `false → true` transition, so a productive round admits at least one of
///    finitely many names and there are at most that many productive rounds. That
///    argument reads the **step**, not the seed set, so no widening of the seeds
///    can affect it. [`TuEmptyCallees::overflowed`]'s ceiling stays exactly as
///    written and still cannot fire.
/// 2. **A seeded name has NO outgoing link — and the guard is in the READER, not
///    in this file.** `NoEffectNothing` contributes `link = None`, and it may do
///    so honestly because `no_effect_nothing`'s vocabulary excludes every call
///    token: a body it accepts *names no callee*. `Parsed` + `empty_body`
///    likewise has no `tail_call` to step to.
///
///    > **Measured, and it corrects the obvious reading of that sentence.**
///    > Mutation **M3a** (`work/w-seed/mutate.sh`) gives `NoEffectNothing` a link
///    > as well as a seed and was registered as a must-fail. **It came back
///    > GREEN.** The reason is one line of the iteration below —
///    > `if in_r[i] { continue; }` — so a name that is already admitted is never
///    > asked for its link and the `None` here is **inert as the loop is
///    > written**. It is documentation of what the reader guarantees, not the
///    > enforcement of it.
///    >
///    > The enforcement is **M2**, which opens the reader's vocabulary so a body
///    > with a call in it can seed: that goes RED on 7 tests. And **M3b**, which
///    > makes `NoEffectCall` seed as well as link and so erases the distinction
///    > outright, goes RED on 2 — including
///    > `a_cycle_of_refused_links_beside_a_seed_admits_only_the_chain`.
///    >
///    > So: do not weaken `no_effect_nothing`'s vocabulary on the grounds that
///    > "`elide.rs` sets the link to `None` anyway". That arm cannot save you.
///    > M3a is reported green rather than rewritten until it went red, because a
///    > mutation edited to fail proves nothing.
/// 3. **Admission propagates only BACKWARDS along links, from a seed.** A name is
///    admitted iff it seeds or its link names an admitted name; following links
///    out of a cycle stays inside the cycle forever. So a cycle is admitted only
///    if one of its own members seeds.
/// 4. **A cycle member cannot seed.** Membership requires an outgoing link, and
///    by (2) a seed has none.
///
/// ∴ no cycle member is admitted. **If (2) is ever weakened — a seeding reader
/// that also returns a callee — this argument collapses and the round ceiling
/// becomes the only thing between the fixpoint and a hang.** GRID-N's `n06` is
/// the compiled cell, `a_cycle_is_not_elided_and_terminates` and
/// `the_round_ceiling_cannot_fire` are unchanged and still pass, and
/// `a_seeding_refused_body_has_no_link` asserts (2) directly.
#[derive(Debug, Clone, Copy)]
pub enum Reduction<'a> {
    /// A body the parser accepted. Seeds when [`IlFunction::empty_body`]; steps
    /// when it is an elidable tail call ([`elidable_step`]).
    Parsed(&'a IlFunction),
    /// A body the parser **REFUSED**, whose grammar nonetheless proves that it
    /// emits nothing but a call to this callee. **A link: it never seeds.**
    ///
    /// The refusal is unchanged by this: the row is still `FnVerdict::Blocked`,
    /// still `fnbyte-refused`, and `IlBundle::functions` still refuses its whole
    /// TU. What it contributes is one edge of the graph.
    NoEffectCall(&'a str),
    /// A body the parser **REFUSED**, whose grammar proves that it emits nothing
    /// **at all** — no call, no data symbol, no bytes. **A seed, and it carries no
    /// link.**
    ///
    /// Board **#1053**. `c2_il::…::no_effect::no_effect_nothing` is the reader and
    /// its walk is total over a closed vocabulary with no call token in it, which
    /// is what makes "carries no link" a property of the body rather than a
    /// convention of this enum — and (2) of the re-derivation above is exactly
    /// that property.
    ///
    /// The refusal is unchanged, as for [`Self::NoEffectCall`]: still
    /// `FnVerdict::Blocked`, still `fnbyte-refused`, and `IlBundle::functions`
    /// still refuses the TU.
    NoEffectNothing,
}

/// The **same-TU callees that reduce to nothing** in one IL bundle: conditions 1
/// and 2 of [`drops_tail_call`], resolved once per TU instead of once per call —
/// and closed under E itself, which is the whole of board #924.
///
/// The type's name is `w-empty`'s and is deliberately kept, so the board rows and
/// rungs that cite it still resolve; the **set** is no longer "the empty-bodied
/// ones" but the least fixpoint the module docs give, and
/// [`TuEmptyCallees::reduces_to_nothing`] is named for what it answers.
///
/// A name whose bundle carries *two* definitions that disagree — one empty and
/// one not, or two elidable steps to different callees — is **not** admitted:
/// the two spellings would have to be told apart by something this type does not
/// have, and refusing is the honest answer. (The gate's own name binding makes
/// that impossible today; representing it beats asserting it.)
#[derive(Debug, Default, Clone)]
pub struct TuEmptyCallees {
    /// Sorted, deduplicated mangled names, every one of which is defined in this
    /// bundle **and** reduces to nothing.
    empty: Vec<String>,
    /// How many productive rounds the fixpoint took. `0` is the one-step-free
    /// case (nothing but the seed), and it is *printed* rather than inferred:
    /// "the closure ran and added nothing" and "the closure never ran" are
    /// different observations, which is the same reason [`Self::len`] exists.
    rounds: usize,
    /// The round ceiling fired: the iteration was not monotone, so this context
    /// **admits nothing**. Unreachable as written (`the_round_ceiling_cannot_fire`),
    /// and the reason a broken edit terminates red instead of hanging a scan.
    overflow: bool,
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
    pub fn of_named<'a>(named: impl IntoIterator<Item = (&'a str, &'a IlFunction)>) -> Self {
        Self::of_rows(named.into_iter().map(|(n, f)| (n, Reduction::Parsed(f))))
    }

    /// [`TuEmptyCallees::of_named`] over [`Reduction`]s — the general form, and
    /// the one a caller with **refused** rows needs (board #980).
    ///
    /// Identical for a `Parsed`-only input, byte for byte: `of_named` is a
    /// wrapper over this and every existing test goes through it unchanged.
    pub fn of_rows<'a>(rows: impl IntoIterator<Item = (&'a str, Reduction<'a>)>) -> Self {
        let mut rows: Vec<(&str, Reduction<'a>)> =
            rows.into_iter().filter(|(n, _)| !n.is_empty()).collect();
        rows.sort_by_key(|(n, _)| *n);

        // ---- one node per distinct name ------------------------------------
        //
        // `seed` is w-empty's shipped condition 2. `link` is the fixpoint's step
        // and carries the callee it steps to; a name whose definitions disagree
        // about either contributes neither, for the reason the type's doc gives.
        let mut names: Vec<&str> = Vec::new();
        let mut seed: Vec<bool> = Vec::new();
        let mut link: Vec<Option<&str>> = Vec::new();
        let mut i = 0;
        while i < rows.len() {
            let name = rows[i].0;
            let mut j = i;
            let mut all_empty = true;
            let mut step: Option<Option<&str>> = None;
            while j < rows.len() && rows[j].0 == name {
                // **(seeds, link)** — and the two REFUSED variants sit on opposite
                // sides of it. `NoEffectCall` is a link and only a link: its claim
                // is conditional on a callee, so a chain built entirely of them is
                // never admitted. `NoEffectNothing` is a seed and carries **no**
                // link, which is not a convention here but a property of the body
                // its reader accepts — step (2) of the cycle re-derivation on
                // [`Reduction`].
                //
                // **That `None` is INERT, and it is measured rather than assumed.**
                // The iteration below skips a name already in `in_r`, so a SEEDED
                // name is never asked for its link at all: mutation M3a puts one
                // here and nothing goes red. What actually keeps a cycle out of
                // the seed set is that `no_effect_nothing` refuses every body with
                // a call in it (M2 - 7 tests red) and that `NoEffectCall` does not
                // seed (M3b - 2 tests red, including the cycle one). Editing this
                // line is neither how the property is maintained nor how it would
                // be lost.
                let (seeds, here) = match rows[j].1 {
                    Reduction::Parsed(f) => (f.empty_body(), elidable_step(f)),
                    Reduction::NoEffectCall(callee) => (false, Some(callee)),
                    Reduction::NoEffectNothing => (true, None),
                };
                all_empty &= seeds;
                step = Some(match step {
                    None => here,
                    // Two definitions that step to different callees, or one
                    // that steps and one that does not, admit neither.
                    Some(prev) if prev == here => here,
                    Some(_) => None,
                });
                j += 1;
            }
            names.push(name);
            seed.push(all_empty);
            link.push(step.flatten());
            i = j;
        }

        // ---- the least fixpoint --------------------------------------------
        //
        // A name is admitted only on a `false -> true` transition, so a
        // productive round admits at least one of the finitely many names and
        // there can be at most that many productive rounds. A cycle is never
        // seeded and therefore never admitted — which is what c2 does too
        // (`k10_cycle2`, `k11_self`), reached by construction and not by a case.
        let mut in_r = seed.clone();
        let ceiling = names.len() + 1;
        let mut rounds = 0usize;
        let mut overflow = false;
        loop {
            let mut changed = false;
            for i in 0..names.len() {
                if in_r[i] {
                    continue;
                }
                let Some(callee) = link[i] else { continue };
                if let Ok(j) = names.binary_search_by(|n| (*n).cmp(callee)) {
                    if in_r[j] {
                        in_r[i] = true;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
            rounds += 1;
            // THE ROUND CEILING. Unreachable as written; it is here so that an
            // edit which breaks the monotonicity argument above terminates and
            // goes red rather than spinning a workload scan forever.
            if rounds > ceiling {
                overflow = true;
                break;
            }
        }
        if overflow {
            return Self {
                empty: Vec::new(),
                rounds,
                overflow,
            };
        }
        let empty = names
            .iter()
            .zip(in_r.iter())
            .filter(|(_, &r)| r)
            .map(|(n, _)| (*n).to_string())
            .collect();
        Self {
            empty,
            rounds,
            overflow,
        }
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
        Self::of_named(funcs.into_iter().map(|f| (f.mangled_name.as_str(), f)))
    }

    /// Is `name` defined in this bundle by a body c2 emits **nothing** for?
    ///
    /// Empty-bodied, or an elidable tail call to a name this same predicate
    /// accepts — the fixpoint, resolved when the context was built.
    pub fn reduces_to_nothing(&self, name: &str) -> bool {
        self.empty.binary_search_by(|p| p.as_str().cmp(name)).is_ok()
    }

    /// How many productive rounds the closure took. `0` means the seed was
    /// already closed — no chain in this TU is deeper than one step.
    pub fn rounds(&self) -> usize {
        self.rounds
    }

    /// The round ceiling fired and this context admits nothing. Always `false`
    /// as the iteration is written; see [`Self::overflow`]'s field doc.
    pub fn overflowed(&self) -> bool {
        self.overflow
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
    let Some(callee) = f.tail_call() else {
        return false;
    };
    // Condition 3 — a caller that materializes a named data symbol is a cell no
    // grid graded, and `k16_mid_stores_global` grades what happens when one is
    // in the middle of a chain: its own call is `E` and its CALLER's is `I`.
    if !f.data_syms.is_empty() {
        return false;
    }
    // Conditions 1 and 2, both resolved in `TuEmptyCallees::of`.
    tu.reduces_to_nothing(callee)
}

/// **The fixpoint's step**, asked of a candidate *mid-chain* node: is this body
/// one c2 emits nothing for **provided** its callee reduces to nothing?
///
/// `Some(callee)` when it is, and this is deliberately the same three conditions
/// [`drops_tail_call`] imposes on the caller, because it is the same question
/// asked one level down. The two extra tests are not extra conditions — they are
/// what makes `select_function` return [`crate::codegen::Selected::Tail`] for
/// this body rather than one of the shapes that owns its own branch layout, and
/// so what makes "its whole emitted body is the call and a pure setup" true:
///
/// * `framed_call` / `call_seq` / `cond_pair` are asked **before** `tail_call`
///   in the selector, so a body carrying one of them is not a tail call at all.
///   `m4_seq_mixed_mid` is the graded reason a `Seq` link must not be admitted —
///   c2 emits a surviving branch for it and its caller grades `I`.
/// * the remaining body kinds cannot coexist with `tail_call`, but they are not
///   *asserted* away here: an `IlFunction` is a parse result and this is a
///   predicate over it, not over the parser's invariants.
fn elidable_step(f: &IlFunction) -> Option<&str> {
    if !f.data_syms.is_empty() || f.framed_call().is_some() || f.call_seq().is_some()
        || f.cond_pair().is_some()
    {
        return None;
    }
    f.tail_call()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::testutil::func_with;

    fn named(name: &str) -> IlFunction {
        let mut f = func_with(Vec::new(), Vec::new());
        f.mangled_name = name.into();
        f.data_syms.clear();
        f
    }

    fn empty(name: &str) -> IlFunction {
        let mut f = named(name);
        f.body = c2_il::BodyShape::EmptyBody;
        f
    }

    fn tail_caller(name: &str, callee: &str) -> IlFunction {
        let mut f = named(name);
        f.body = c2_il::BodyShape::Tail(callee.to_string());
        f
    }

    /// `c00_empty` — the cell the whole rule is built on.
    ///
    /// The context admits **two** names, which is the fixpoint showing through
    /// at depth 1: `?g` seeds it and `?f` — a tail call to a name that reduces
    /// to nothing, with no data symbol — reduces to nothing itself. It was 1
    /// before board #924, and nothing about `?f`'s own emitted body changed.
    #[test]
    fn a_same_tu_empty_callee_is_elided() {
        let funcs = vec![empty("?g@@YAXXZ"), tail_caller("?f@@YAXXZ", "?g@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 2);
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
        g.body = c2_il::BodyShape::Tail("?ext@@YAXXZ".into());
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
        caller.data_syms = vec!["?gv@@3HA".into()];
        let funcs = vec![empty("?g@@YAXXZ"), caller];
        let tu = TuEmptyCallees::of(&funcs);
        assert!(tu.reduces_to_nothing("?g@@YAXXZ"));
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
        g2.body = c2_il::BodyShape::Tail("?ext@@YAXXZ".into());
        let funcs = vec![empty("?g@@YAXXZ"), g2, tail_caller("?f@@YAXXZ", "?g@@YAXXZ")];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 0);
        assert!(!drops_tail_call(&funcs[2], &tu));
    }

    // =====================================================================
    // THE FIXPOINT — board #924. Every test below names the GRID-3 cell that
    // graded the shape against real c2; `work/w-fix/grid3*.out` are the runs.
    // =====================================================================

    /// Build a chain `f -> g1 -> … -> g{n-1} -> h`, `?h` empty, `?f` last —
    /// the shape of `k2`…`k4` and `m1`…`m3`.
    fn chain(n: usize) -> Vec<IlFunction> {
        let mut out = vec![empty("?h@@YAXXZ")];
        for i in (1..n).rev() {
            let callee = if i == n - 1 {
                "?h@@YAXXZ".to_string()
            } else {
                format!("?g{}@@YAXXZ", i + 1)
            };
            out.push(tail_caller(&format!("?g{i}@@YAXXZ"), &callee));
        }
        let top = if n > 1 { "?g1@@YAXXZ" } else { "?h@@YAXXZ" };
        out.push(tail_caller("?f@@YAXXZ", top));
        out
    }

    /// `k2_chain_d2` — the cell `w-empty` had and did not ship
    /// (`g07_empty_calls_empty`): c2 emits **both** `?f` and `?g1` as a bare
    /// `blr`, and one step of the closure is what reaches `?f`.
    #[test]
    fn a_chain_two_deep_elides_at_both_links() {
        let funcs = chain(2);
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 3, "?h seeds, ?g1 and ?f close");
        assert!(drops_tail_call(&funcs[1], &tu), "?g1 -> ?h");
        assert!(
            drops_tail_call(&funcs[2], &tu),
            "?f -> ?g1: THE FIXPOINT. One-step E stops here and c2 does not \
             (work/w-fix/grid3.out, cell k2_chain_d2, both edges E)"
        );
    }

    /// `k3`, `k4`, `m1`, `m2`, `m3` — depth 3, 4, 5, 6 and 8, every edge graded
    /// `E` with the caller a bare `blr`. The rule iterates; the grid is why it
    /// is allowed to.
    #[test]
    fn a_chain_stays_elided_at_every_graded_depth() {
        for n in [3usize, 4, 5, 6, 8] {
            let funcs = chain(n);
            let tu = TuEmptyCallees::of(&funcs);
            assert_eq!(tu.len(), n + 1, "depth {n}: every name reduces to nothing");
            for (k, f) in funcs.iter().enumerate().skip(1) {
                assert!(
                    drops_tail_call(f, &tu),
                    "depth {n}: link {k} ({}) must elide — GRID-3/3b grade every \
                     edge of depths 1..6 and 8 as E",
                    f.mangled_name
                );
            }
        }
    }

    /// `k5`/`k6`/`k7` — a link whose body is **not** empty stops the chain, at
    /// each of depths 1, 2 and 3: at `/Ob0` every caller at or above it keeps
    /// its REL24. Here the broken link calls an external the TU does not
    /// define, so it neither seeds nor steps to anything in the bundle.
    #[test]
    fn a_non_empty_link_stops_the_chain_at_every_depth() {
        for stop in 1usize..=3 {
            // `funcs` is [?h, ?g2, ?g1, ?f] — deepest first. A "stop at depth d"
            // makes the function d links BELOW `?f` call out of the bundle, so
            // it is `k5` at d = 3, `k6` at d = 2 and `k7` at d = 1.
            let mut funcs = chain(3);
            let idx = funcs.len() - 1 - stop;
            funcs[idx].body = c2_il::BodyShape::Tail("?ext@@YAXXZ".into());
            let tu = TuEmptyCallees::of(&funcs);
            for (k, f) in funcs.iter().enumerate().skip(1) {
                // Everything strictly deeper than the break still elides;
                // the broken link and everything above it does not.
                let below_the_break = k < idx;
                assert_eq!(
                    drops_tail_call(f, &tu),
                    below_the_break,
                    "stop at link {stop}: {} must {}elide — the chain STOPS at a \
                     non-empty body (GRID-3 k5/k6/k7: verdict I at every edge at \
                     or above it)",
                    f.mangled_name,
                    if below_the_break { "" } else { "NOT " }
                );
            }
        }
    }

    /// **THE CYCLE** — `k10_cycle2`. `void a(){b();} void b(){a();}`: neither
    /// member is a bare `blr` in c2, and neither is admitted here. The closure
    /// **terminates**, which is the property this test exists for.
    #[test]
    fn a_cycle_is_not_elided_and_terminates() {
        let funcs = vec![
            tail_caller("?a@@YAXXZ", "?b@@YAXXZ"),
            tail_caller("?b@@YAXXZ", "?a@@YAXXZ"),
            tail_caller("?f@@YAXXZ", "?a@@YAXXZ"),
        ];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(
            tu.len(),
            0,
            "A CYCLE WAS TREATED AS REDUCING TO NOTHING: it is never SEEDED, so \
             the least fixpoint must never admit it — and c2 agrees, GRID-3 k10 \
             grades ?a -> ?b CALL and ?b -> ?a I, neither member a blr"
        );
        assert_eq!(tu.rounds(), 0, "no productive round: nothing to propagate");
        assert!(!tu.overflowed());
        for f in &funcs {
            assert!(!drops_tail_call(f, &tu));
        }
    }

    /// Direct self-recursion — `k11_self`. `?r` emits a self-branch that takes
    /// no relocation at all, so the *relocation* observable reads `E` on
    /// `r -> r` while the body is plainly not nothing. The rule must not be
    /// fitted to that: `?r` is not seeded, so it is not admitted.
    #[test]
    fn direct_self_recursion_is_not_elided() {
        let funcs = vec![
            tail_caller("?r@@YAXXZ", "?r@@YAXXZ"),
            tail_caller("?f@@YAXXZ", "?r@@YAXXZ"),
        ];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 0);
        assert!(!drops_tail_call(&funcs[0], &tu));
        assert!(
            !drops_tail_call(&funcs[1], &tu),
            "GRID-3 k11 grades ?f -> ?r CALL: the caller keeps its branch"
        );
    }

    /// A cycle **hanging off** a live chain: the chain closes and the cycle
    /// still does not, in one bundle and one iteration.
    #[test]
    fn a_cycle_beside_a_live_chain_admits_only_the_chain() {
        let mut funcs = chain(3);
        funcs.push(tail_caller("?a@@YAXXZ", "?b@@YAXXZ"));
        funcs.push(tail_caller("?b@@YAXXZ", "?a@@YAXXZ"));
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 4, "?h ?g2 ?g1 ?f, and neither ?a nor ?b");
        assert!(!tu.reduces_to_nothing("?a@@YAXXZ"));
        assert!(!tu.reduces_to_nothing("?b@@YAXXZ"));
        assert!(tu.reduces_to_nothing("?f@@YAXXZ"));
    }

    /// **Mechanism I mid-chain** — `k12_cross_i`. `int m(int a){return a;}` is
    /// `I`, not `E`, and at `/O1` **both** its callers are a bare `blr`, so a
    /// rule fitted to the bytes would take the whole chain. `?m` is not
    /// `empty_body`, nothing seeds, and the port keeps both branches.
    #[test]
    fn mechanism_i_mid_chain_does_not_propagate() {
        let funcs = vec![
            named("?m@@YAHH@Z"),
            tail_caller("?g1@@YAHH@Z", "?m@@YAHH@Z"),
            tail_caller("?f@@YAHH@Z", "?g1@@YAHH@Z"),
        ];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 0, "?m is not empty, so nothing seeds the chain");
        assert!(!drops_tail_call(&funcs[1], &tu));
        assert!(
            !drops_tail_call(&funcs[2], &tu),
            "THE FIXPOINT WAS APPLIED THROUGH A NON-EMPTY LINK: GRID-3 k12 grades \
             both edges mechanism I, and only /Ob0 separates them from E"
        );
    }

    /// **A mid-node that keeps BYTES does not propagate** — `k15` (`g(sink++)`)
    /// and `k16` (a store to a global). Its own call is `E` in c2 and **its
    /// caller's is `I`**: what propagates is *emits nothing*, not *elided its
    /// call*. Here the mid-node carries a data symbol, which is how the port
    /// sees `k16`.
    #[test]
    fn a_mid_node_that_materializes_data_does_not_propagate() {
        let mut g1 = tail_caller("?g1@@YAXH@Z", "?h@@YAXH@Z");
        g1.data_syms = vec!["?gv@@3HA".into()];
        let funcs = vec![
            empty("?h@@YAXH@Z"),
            g1,
            tail_caller("?f@@YAXH@Z", "?g1@@YAXH@Z"),
        ];
        let tu = TuEmptyCallees::of(&funcs);
        assert_eq!(tu.len(), 1, "only ?h");
        assert!(
            !drops_tail_call(&funcs[2], &tu),
            "GRID-3 k16 grades ?f -> ?g1 as I: the mid-node still emits its store"
        );
    }

    /// **A `Seq` mid-node is refused**, and that is a decision rather than a
    /// gap: `k8`/`k9` grade a two-call body over two empty callees as `E` in c2.
    /// The port's `Selected::Seq` emitter does not model E, so admitting it
    /// would put "emits nothing" in the context while the emitter emits two
    /// calls. `m4_seq_mixed_mid` is the cell where c2 agrees with the refusal.
    #[test]
    fn a_seq_mid_node_is_refused() {
        let mut g1 = tail_caller("?g1@@YAXXZ", "?h@@YAXXZ");
        g1.body = c2_il::BodyShape::Seq(c2_il::CallSeq {
            calls: Vec::new(),
            tail: c2_il::SeqTail::Void,
            saved: Vec::new(),
            guard: None,
            early: Vec::new(),
            store_run: None,
        });
        let funcs = vec![
            empty("?h@@YAXXZ"),
            g1,
            tail_caller("?f@@YAXXZ", "?g1@@YAXXZ"),
        ];
        let tu = TuEmptyCallees::of(&funcs);
        assert!(!tu.reduces_to_nothing("?g1@@YAXXZ"));
        assert!(!drops_tail_call(&funcs[2], &tu));
    }

    /// The closure is **order-independent** — `m6_defined_after`, where every
    /// definition sits below its use. The bundle order must not decide the
    /// answer, so the same chain is asked forwards and backwards.
    #[test]
    fn the_closure_does_not_depend_on_bundle_order() {
        let forward = chain(5);
        let mut backward = forward.clone();
        backward.reverse();
        let a = TuEmptyCallees::of(&forward);
        let b = TuEmptyCallees::of(&backward);
        assert_eq!(a.len(), 6);
        assert_eq!(a.len(), b.len());
        for f in &forward {
            assert_eq!(
                a.reduces_to_nothing(&f.mangled_name),
                b.reduces_to_nothing(&f.mangled_name)
            );
        }
    }

    /// **THE ROUND CEILING CANNOT FIRE** as the iteration is written, and that
    /// is asserted rather than argued: a cycle, a chain and a fan-in in one
    /// bundle, and the closure still reports `overflowed() == false`.
    ///
    /// The ceiling exists so that an edit which breaks monotonicity **terminates
    /// and goes red** instead of hanging a workload scan — `work/w-fix/mutate.sh`
    /// mutation 1 is that edit, and this assertion is what catches it.
    #[test]
    fn the_round_ceiling_cannot_fire() {
        let mut funcs = chain(8);
        funcs.push(tail_caller("?a@@YAXXZ", "?b@@YAXXZ"));
        funcs.push(tail_caller("?b@@YAXXZ", "?a@@YAXXZ"));
        funcs.push(tail_caller("?c1@@YAXXZ", "?h@@YAXXZ"));
        funcs.push(tail_caller("?c2@@YAXXZ", "?h@@YAXXZ"));
        let tu = TuEmptyCallees::of(&funcs);
        assert!(
            !tu.overflowed(),
            "THE RECURSION GUARD WAS REMOVED: the fixpoint is no longer monotone, \
             so it re-admitted a name every round and would not have terminated \
             without the ceiling. The context now admits NOTHING, which is the \
             safe answer and not a working one"
        );
        assert!(
            tu.rounds() <= funcs.len() + 1,
            "a productive round must admit at least one name"
        );
        assert!(tu.reduces_to_nothing("?f@@YAXXZ"));
        assert!(!tu.reduces_to_nothing("?a@@YAXXZ"));
    }

    /// The elision and the membership are **the same fact**: for a tail call
    /// with no data symbol, `drops_tail_call` is true exactly when the caller
    /// itself reduces to nothing. If those two could disagree, the emitter and
    /// the closure would be modelling different rules.
    #[test]
    fn eliding_a_call_and_reducing_to_nothing_are_the_same_fact() {
        let mut funcs = chain(4);
        funcs.push(tail_caller("?ext_caller@@YAXXZ", "?nowhere@@YAXXZ"));
        let tu = TuEmptyCallees::of(&funcs);
        for f in &funcs {
            if f.empty_body() {
                continue;
            }
            assert_eq!(
                drops_tail_call(f, &tu),
                tu.reduces_to_nothing(&f.mangled_name),
                "{} disagrees with itself",
                f.mangled_name
            );
        }
    }

    // =====================================================================
    // BOARD #1053 — A REFUSED BODY MAY SEED. Every test below is about the
    // property that makes the cycle re-derivation on `Reduction` hold, and
    // the two that were already here (`a_cycle_is_not_elided_and_terminates`,
    // `the_round_ceiling_cannot_fire`) are deliberately untouched.
    // =====================================================================

    /// The positive: a **refused** body that emits nothing at all seeds the
    /// closure, and one link above it closes. That is the whole rung, and the
    /// workload's 223 conversions are this picture with two more links in it.
    #[test]
    fn a_refused_body_with_no_call_seeds_the_fixpoint() {
        let caller = tail_caller("?f@@YAXXZ", "?leaf@@YAXXZ");
        let rows = vec![
            ("?leaf@@YAXXZ", Reduction::NoEffectNothing),
            ("?f@@YAXXZ", Reduction::Parsed(&caller)),
        ];
        let tu = TuEmptyCallees::of_rows(rows);
        assert_eq!(tu.len(), 2, "the leaf seeds and ?f closes over it");
        assert!(drops_tail_call(&caller, &tu));
    }

    /// **STEP (2) OF THE RE-DERIVATION, asserted directly.** A seeded refused body
    /// carries **no link**, and that is what keeps a cycle out of the seed set now
    /// that a refused body can seed at all.
    ///
    /// Stated as a behaviour rather than as a field read: a `NoEffectNothing` row
    /// whose *name* is also the name of a `NoEffectCall` row cannot be both, so the
    /// closure must refuse it — if it silently preferred one, the enum would have
    /// a link and a seed on the same name and the argument would be gone.
    #[test]
    fn a_seeding_refused_body_carries_no_link() {
        // One name, two disagreeing refused definitions: one seeds, one steps.
        let rows = vec![
            ("?g@@YAXXZ", Reduction::NoEffectNothing),
            ("?g@@YAXXZ", Reduction::NoEffectCall("?h@@YAXXZ")),
        ];
        let tu = TuEmptyCallees::of_rows(rows);
        assert_eq!(
            tu.len(),
            0,
            "A NAME THAT BOTH SEEDS AND STEPS WAS ADMITTED. The two facts are \
             mutually exclusive by construction in `c2-il` (one reader requires a \
             call token the other's vocabulary excludes); if they ever reach this \
             closure together, refusing is the only answer that keeps \
             `Reduction`'s step (2) true"
        );
    }

    /// **THE WORKLOAD'S SHAPE, in the abstract.** A chain of *refused* links —
    /// board #980's dead temporary and #1053's destroy loop — bottoming out on a
    /// *refused* seed, with a parsed tail call on top. Before this lane the whole
    /// chain admitted nothing, because nothing in it seeded.
    #[test]
    fn a_chain_of_refused_links_closes_on_a_refused_seed() {
        let top = tail_caller("?wrapper@@YAXXZ", "?dr@@YAXXZ");
        let rows = vec![
            ("?leaf@@YAXXZ", Reduction::NoEffectNothing),
            ("?aux@@YAXXZ", Reduction::NoEffectCall("?leaf@@YAXXZ")),
            ("?dr@@YAXXZ", Reduction::NoEffectCall("?aux@@YAXXZ")),
            ("?wrapper@@YAXXZ", Reduction::Parsed(&top)),
        ];
        let tu = TuEmptyCallees::of_rows(rows);
        assert_eq!(tu.len(), 4, "every level reduces to nothing");
        assert!(
            drops_tail_call(&top, &tu),
            "the chain did not close: the seed is not propagating through the \
             refused links, which is the only thing this lane added"
        );
        // **Not "three rounds, one per link".** A round is one pass over the sorted
        // name vector, so a chain whose names happen to sort deepest-first closes
        // in a single pass and one that sorts the other way takes one round per
        // link. This assertion was written as `>= 3`, failed, and is corrected
        // rather than deleted: the ROUND COUNT is bundle-order dependent and only
        // the RESULT is not — which is exactly what
        // `the_closure_does_not_depend_on_bundle_order` exists to say, and why
        // `rounds` is printed as an observation rather than read as a depth.
        assert!(tu.rounds() >= 1, "the closure ran and was productive");
        assert!(tu.rounds() <= 5, "and terminated well inside the ceiling");
        assert!(!tu.overflowed());
        // The same bundle backwards must give the same answer, since the count
        // above is the one thing that moves.
        let back = TuEmptyCallees::of_rows(vec![
            ("?wrapper@@YAXXZ", Reduction::Parsed(&top)),
            ("?dr@@YAXXZ", Reduction::NoEffectCall("?aux@@YAXXZ")),
            ("?aux@@YAXXZ", Reduction::NoEffectCall("?leaf@@YAXXZ")),
            ("?leaf@@YAXXZ", Reduction::NoEffectNothing),
        ]);
        assert_eq!(back.len(), 4);
    }

    /// **THE CYCLE, with the new seed in the same bundle.** `?a` and `?b` are
    /// refused links pointing at each other; a genuine seed sits beside them.
    ///
    /// The chain closes and the cycle does not, which is `Reduction`'s steps (3)
    /// and (4): admission propagates only backwards along links from a seed, and
    /// following links out of a cycle stays in the cycle forever.
    #[test]
    fn a_cycle_of_refused_links_beside_a_seed_admits_only_the_chain() {
        let rows = vec![
            ("?leaf@@YAXXZ", Reduction::NoEffectNothing),
            ("?up@@YAXXZ", Reduction::NoEffectCall("?leaf@@YAXXZ")),
            ("?a@@YAXXZ", Reduction::NoEffectCall("?b@@YAXXZ")),
            ("?b@@YAXXZ", Reduction::NoEffectCall("?a@@YAXXZ")),
        ];
        let tu = TuEmptyCallees::of_rows(rows);
        assert!(tu.reduces_to_nothing("?leaf@@YAXXZ"));
        assert!(tu.reduces_to_nothing("?up@@YAXXZ"));
        assert!(
            !tu.reduces_to_nothing("?a@@YAXXZ") && !tu.reduces_to_nothing("?b@@YAXXZ"),
            "A CYCLE WAS ADMITTED. Nothing in it seeds — a seed carries no link and \
             a cycle member always has one — so the least fixpoint must never reach \
             it. GRID-N's n06 is the compiled cell"
        );
        assert_eq!(tu.len(), 2);
        assert!(!tu.overflowed(), "the closure must still terminate structurally");
    }

    /// A seed and a **live** definition of the same name admit neither: the two
    /// spellings would have to be told apart by something this type does not have.
    /// The pre-existing `a_name_defined_twice_with_different_bodies_is_refused`
    /// says this for `Parsed`; a seed is a stronger claim, so it is re-asked.
    #[test]
    fn a_seed_that_disagrees_with_a_parsed_definition_is_refused() {
        let mut live = named("?g@@YAXXZ");
        live.body = c2_il::BodyShape::Tail("?ext@@YAXXZ".into());
        let caller = tail_caller("?f@@YAXXZ", "?g@@YAXXZ");
        let rows = vec![
            ("?g@@YAXXZ", Reduction::NoEffectNothing),
            ("?g@@YAXXZ", Reduction::Parsed(&live)),
            ("?f@@YAXXZ", Reduction::Parsed(&caller)),
        ];
        let tu = TuEmptyCallees::of_rows(rows);
        assert_eq!(tu.len(), 0);
        assert!(!drops_tail_call(&caller, &tu));
    }

    /// **Condition 3 still bites over a seed.** A caller that materializes a named
    /// data symbol is declined even when its callee genuinely reduces to nothing —
    /// `g01_data_addr_arg`, and GRID-N's `n08` is the compiled cell.
    #[test]
    fn a_caller_with_a_data_symbol_is_not_elided_over_a_seed() {
        let mut caller = tail_caller("?f@@YAXXZ", "?leaf@@YAXXZ");
        caller.data_syms = vec!["?gv@@3HA".into()];
        let rows = vec![
            ("?leaf@@YAXXZ", Reduction::NoEffectNothing),
            ("?f@@YAXXZ", Reduction::Parsed(&caller)),
        ];
        let tu = TuEmptyCallees::of_rows(rows);
        assert!(tu.reduces_to_nothing("?leaf@@YAXXZ"));
        assert!(!drops_tail_call(&caller, &tu));
    }

    /// **Condition 1 still bites over a seed.** A seed in this bundle does not make
    /// a call to some *other*, undefined name elidable — `c22_extern_callee`, and
    /// GRID-N's `n07`.
    #[test]
    fn a_seed_does_not_admit_a_call_to_an_external() {
        let caller = tail_caller("?f@@YAXXZ", "?nowhere@@YAXXZ");
        let rows = vec![
            ("?leaf@@YAXXZ", Reduction::NoEffectNothing),
            ("?f@@YAXXZ", Reduction::Parsed(&caller)),
        ];
        let tu = TuEmptyCallees::of_rows(rows);
        assert!(tu.reduces_to_nothing("?leaf@@YAXXZ"));
        assert!(!drops_tail_call(&caller, &tu));
    }

    /// An unnamed definition cannot be a callee: it contributes nothing rather
    /// than admitting the empty string.
    #[test]
    fn an_unnamed_definition_admits_nothing() {
        let mut g = func_with(Vec::new(), Vec::new());
        g.mangled_name = String::new();
        g.body = c2_il::BodyShape::EmptyBody;
        g.data_syms.clear();
        let tu = TuEmptyCallees::of(&[g]);
        assert_eq!(tu.len(), 0);
        assert!(!tu.reduces_to_nothing(""));
    }
}

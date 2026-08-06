# w-fix — PREREGISTRATION

Lane **w-fix**, worktree off master `dcc9214`. Board **#924** — *the fixpoint
form of mechanism E*.

**Committed before one cell of GRID-3 exists and before one line of
`crates/` is touched.** Everything below is registered, including the numbers
that can lose. `git log --diff-filter=A -- work/w-fix/PREREG.md` is the proof of
order.

---

## 0. What the lane is for, in one sentence

`w-empty` shipped the **one-step** mechanism E (a tail call to a same-TU callee
whose IL body decodes `empty-body` emits nothing) and measured, on **one cell**,
that c2 closes E under itself:

```cpp
void h() {}
void g() { h(); }     // source body NOT empty
void f() { g(); }     // c2 emits BOTH ?f and ?g as a bare blr
```

This lane grids that boundary against real c2 and ships the fixpoint **only
where the cells state it**.

## 1. The claims

Every cell of GRID-3 is compiled **twice** — at the workload's own flags and
again with `/Ob0` appended — because at `/O1` alone an absent REL24 cannot tell
**E** (the call was dropped) from **I** (the inliner expanded it). Verdicts are
read from obj bytes (#843), the callee resolved by symbol **name**, never by
position. Every cell carries the `?anchor` → `?ext_anchor` positive control and
a cell whose anchor loses its relocation is **refused, not scored**.

| # | claim | loses if |
|---|---|---|
| **P1** | **THE CLAIM I MOST EXPECT TO LOSE.** In a chain of empty-bodied tail calls `f → g1 → … → gk → h` with `h` empty, **every** caller in the chain is a bare `blr` with no REL24 at **both** flag settings, for k = 0, 1, 2, 3 (chain depth 1…4) | any depth at which some caller keeps a REL24 to its callee at `/Ob0`. A depth at which the fixpoint stops is a **hard bound on what may ship** |
| **P2** | A **non-empty** body at depth d stops the chain: at `/Ob0` every caller at or above d keeps its REL24, at each of d = 1, 2, 3 | any cell where a caller above a non-empty link is a bare `blr` at `/Ob0` |
| **P3** | **The cycle terminates and is not E.** `void a(){b();} void b(){a();}` and `void r(){r();}` — neither member is emitted as a bare `blr` with no relocation at both settings. `INLINE_PREDICATE.md` §4's `recurse` family says c2 refuses to *inline* recursion and says nothing about elision; this measures it | either member grades `E`. **If it does, the fixpoint must still refuse it** — an admitted cycle is a non-terminating computation and the rule ships without it either way |
| **P4** | **Mechanism I mid-chain does not propagate E.** `int m(int a){return a;} int g(int a){return m(a);} int f(int a){return g(a);}` — `?f` keeps a REL24 to `?g` at `/Ob0` | `?f` has no REL24 at `/Ob0`, i.e. I and E compose. Then the port must still refuse, because it cannot compute I |
| **P5** | **The shipped fixpoint is a strict subset of what the grid grades.** Every chain shape the grid does not grade — a `Seq` mid-node, a mid-node that materializes data, a side-effecting mid-node, an indirect site — is **refused** by the shipped rule | the port fires on a shape no cell graded |
| **P6** | **The workload delta.** `fnbyte-differs` falls by **[100, 300]** (point prediction **143**, board #924's `??1?$_Rb_tree_base@…` family), every moved function is `shape=tail`, and `fnbyte-exact` rises by **exactly** the same number | a fall outside the band, or a rise that is not equal to the fall |
| **P7** | **CONTROL — nothing regresses.** **0** functions ENTER `differs`, checked per `(TU, symbol)` from the scan's own witness keys and never by subtracting two totals; `fnbyte-elided == fnbyte-elided-exact` at the tip | any function enters `differs`, or the two elided counters disagree |
| **P8** | **CONTROL — the structure is untouched.** `fnbyte-partition-broken` 0 · `fnbyte-match-tu-differs` 0 · `fnbyte-census-disagree` 0 · `census/gate disagreement` 0 · `mismatch` 0 · `codegen-gap` 0 · TU match **10** · factors A/B/C/D/E and `B∧C`/`A∧B∧C`/FRONTIER unchanged, by `diff` of two sorted `gap-metric` files | any of them moves |
| **P9** | **Three must-fail mutations, distinct messages.** (1) the recursion guard removed — the fixpoint must still terminate or go RED, **never hang**; (2) the fixpoint applied through a **non-empty** link; (3) a **cycle** treated as reducing to nothing. Each must go RED in the cell tests, the unit tests **or** the workload partition, each with the dropped condition named in the failing assertion | any mutation stays green |
| **P10** | **The packed-path mirror stays in lockstep** (#919): whatever `comdat_body_from_selected` elides, the packed path **refuses**, and a test says so | the two paths can disagree about one rule |

## 2. What ships, and the exact form

If and only if P1–P5 hold as measured:

> **E-FIX.** Over one IL bundle, the set `R` of functions whose body **reduces to
> nothing** is the *least* fixpoint of
>
> * `empty_body(g)` ⟹ `g ∈ R` (the seed — w-empty's shipped rule);
> * `g` is a tail call to `h`, `h ∈ R`, and `g` satisfies every condition
>   `drops_tail_call` imposes on a caller ⟹ `g ∈ R`.
>
> computed **once per TU** by iteration to quiescence, and the caller predicate
> is unchanged: a `Selected::Tail` function whose callee is in `R` emits one
> `blr`.

Least-fixpoint iteration is the termination argument: a cycle is never *seeded*,
so it is never admitted, and the round count is bounded by the number of names.
Both are asserted by tests, not by this paragraph.

The **same three conditions per step**. The packed path keeps its
`NotImplemented` mirror (#919) and gains nothing.

## 3. THE DECLINE FLOOR — registered before any measurement

The lane ships **nothing** in `crates/` if any of:

1. `fnbyte-exact` at the tip is **below** its measured baseline, or any function
   enters `differs`;
2. any of P8's controls moves;
3. any mutation of P9 stays green;
4. **any chain shape the rule would fire on was not graded by GRID-3** — a
   refusal is never wrong (CLAUDE.md), and w-empty's condition 3 exists for
   exactly this reason;
5. the grid shows the fixpoint stops at some depth ≤ 4 **and** the shipped rule
   cannot express that stop.

A decline is a result. `w-rtti` declined a whole ladder step and the rung is
worth reading; this lane's brief says the same thing.

## 4. Method commitments

* GRID-3 is `sha256`-frozen and **committed before the first `cl.exe`**, cell
  list and per-cell edges included.
* The grid is graded **per edge**, not per cell: a chain cell has k edges and
  each is scored separately, so "the chain collapsed" and "the top of the chain
  collapsed" are different observations.
* No number in this document is derived from a run that has not happened. The
  baseline `fnbyte-*` figures are quoted from `rungs/2026-08-07-w-empty.md` §1
  (`exact 35,839 · differs 3,338 · elided 1,373 / 1,373`) and are **re-measured**
  at this lane's base before anything is compared to them; a disagreement is
  reported, not absorbed.
* Everything name-keyed on a census row uses **`FnCensus::emit_name`** (#918).
  `IlFunction::mangled_name` is positional and disagrees on 74,955 rows; keying
  the one-step rule on it turned 14 byte-exact bodies wrong.
* Gate before landing: `scripts/gate.sh --jobs 6` and
  `cargo test --workspace --release`, both aggregated to printed counts.
</content>

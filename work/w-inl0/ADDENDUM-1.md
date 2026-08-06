# w-inl0 ADDENDUM-1 — GRID-M, registered before its first `cl.exe`

`work/w-inl0/cells/` · `sha256` in `work/w-inl0/CELLS.sha256`, committed in the
same commit as this file and **before any cell has been compiled**.

Every cell is compiled by the real toolchain at the workload's own profile
(`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`) with `w-empty`'s ANCHOR
appended — `void ext_anchor(); void anchor(){ ext_anchor(); }` — and graded
through **`grade_one`**, the same function the 878-TU scan runs. **A cell whose
`?anchor@@YAXXZ` is missing or is not `Exact` grades nothing**, and the test says
so rather than reading an absent row as a pass (STATUS trap 5).

Cells **m01**, **m02** and **m06** are compiled a **second** time with `/Ob0`
appended. E is not governed by `/Ob` and mechanism I is
(`INLINE_PREDICATE.md`, `w-fix` #954), so a body that is a bare `blr` at `/O1`
and a call at `/Ob0` is **I** and must not be read as E.

---

## 1. What is already measured, so a prediction here is not a restatement

Recorded before this grid exists, from the runs in `work/w-inl0/tip*_scan.txt`:

* the reader is `crates/c2-il/src/func/body/shapes/no_effect.rs` and it is
  **decode-only** — `parse_segment` is unchanged;
* on the 878-TU workload it fires on **12,690** refused rows, the fixpoint
  admits **1,409**, and of the 363 admitted rows c2 emitted a COMDAT for,
  **363 are one `4e800020` and 0 are anything else**;
* `fnbyte-differs` **3,195 → 3,057**, `fnbyte-exact` **35,982 → 36,120**,
  and a per-symbol diff of the two `--fnbyte-diff-jsonl` files says **138
  closed, 0 opened**, all 138 `??$_Destroy_Range@…`.

GRID-M is the *cell* evidence for the rule those numbers came out of, and its
job is the boundary rather than the population.

## 2. The registered predictions — 8 cells, 19 claims

| cell | what it varies | prediction |
|---|---|---|
| **m01** | THE SHAPE | c2's whole COMDAT for `?dr@…` **and** for `?destroy_range@…` is one `4e800020` with **0 relocations**; the port's `destroy_range` grades `tail` / **`Exact`**; `dr` grades **`Refused`** (the parser is untouched). **At `/Ob0`: unchanged** — this is E, not I. |
| **m02** | the callee keeps bytes | c2's `?dr@…` is **not** a bare `blr`; the port must **not** elide `destroy_range` — its verdict is anything but an elided `Exact`, and `TuEmptyCallees` admits neither name. |
| **m03** | a real `memset` on a formal | c2's `?clear@@YAXPAHH@Z` contains a **REL24** to `memset`; the reader returns `None` for it (asserted through the census key, which stays `expr-intrinsic-memset` and never becomes a no-effect row). |
| **m04** | a SECOND statement | the reader returns `None`; the port must not elide `destroy_range`. |
| **m05** | the FIXPOINT, one link deeper | c2 emits one `4e800020` for `?destroy_range2@…`, `?destroy_range@…` **and** `?dr@…`; the port grades **both** wrappers `Exact`. A one-step rule fails this cell. |
| **m06** | the RESIDUE — the `__false_type` loop overload | c2 emits one `4e800020` for `?destroy_range@…`; the port does **NOT** convert it, and `?aux@…`'s census key is **`return-scope-close-cflow-label`**. This is the production the 228 unconverted members of board #980 are behind. **At `/Ob0` c2 keeps a call somewhere in the chain** — the loop is inlining, not E. |
| **m07** | CONTROL — the callee is EXTERNAL | c2 keeps a **REL24** in `?dr@…`; nothing is admitted; the port must not elide. |
| **m08** | THE CYCLE | `TuEmptyCallees` admits **neither** `?a_@…` nor `?b_@…`, the closure terminates (`overflowed()` is false), and neither of c2's bodies is a bare `blr`. |

**The claim I most expect to lose is m06's second half** — that `/Ob0` separates
it. The whole chain may be E at both settings, in which case the residue is not
"inlining" but "a body the parser cannot read", which is a *different* rung and
a better one. Either result is reportable; a silent omission is not.

**The claim whose loss would be fatal** is m01's `/Ob0` row. If `?destroy_range`
is a bare `blr` at `/O1` and a call at `/Ob0`, the 138 conversions are mechanism
**I** wearing E's clothes, `c19_ret_param`'s trap one level out, and the rule
must be withdrawn.

## 3. The mutations — registered with the test each must turn red

Each is applied to the tree, the run is taken, and **the diff is checked to be
non-empty on the file it names** before the result is read (#951: a mutation
that did not mutate is a green run that means nothing).

| # | the guard removed | must go RED |
|---|---|---|
| **M1** | the *same temporary* test in `eat_dead_temp_arg` (`if again != dest`) | `a_different_temporary_in_the_argument_is_refused` |
| **M2** | the *callee* condition — make `Reduction::NoEffectCall` a **seed** instead of a link | the workload scan's `fnbyte-noeffect-ref-other` must go **> 0**, and `fnbyte-differs` must **rise** somewhere. A rule that fires without asking about its callee is the #950 hazard realized |
| **M3** | the *totality* test — drop `eat_return_plumbing`'s terminal | `trailing_bytes_after_the_function_tail_are_refused` |

## 4. What this grid does NOT claim

* Nothing here says c2 inlines. SPLICE/INLINE-P are `w-seq`'s and remain
  `NOT MODELLED`.
* Nothing here accepts a body. Every cell's `dr` stays `Refused`, and the
  `fnbyte-refused` total on the workload is unchanged at **130,573**.
* The 228 unconverted members of board #980 are **priced, not closed**.

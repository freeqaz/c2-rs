# w-column — PREREGISTRATION

Committed **before** the first scan of this lane was run on this tree.
Lane `w-column`, worktree `wt-w-column` off master **`85e180d4`**.

The question, from board **#1464** / `docs/rungs/2026-08-08-w-ladders.md` §3:

> no field in `TuResult` says *"the reader accepted this function and the emitter
> could not lower it"* — that verdict does not exist to be read.

and the tension: **acceptance lives in the IL parser by design** (#139), so every
emitter refusal is supposed to have a parser counterpart, and `fn_gate_refusals`
must be 0. Can the state exist at all?

---

## 0. WHICH OF THE THREE I EXPECT

**Possibility 1 — it can exist and should — with a correction to the premise.**

I expect to find that the column **already exists** and has since board #322 /
lane `w-fnbyte`, in `crates/c2-harness/src/gap/fnbytes.rs`, and that #1464's
claim is true only of `TuResult`'s *named scalar fields* — it is false of
`TuResult::emit`, the untyped `BTreeMap` FBM writes into. Concretely I expect
`grade_one` to already split

* `Err(gate)` → `FnByte::Refused`, shape **`parse-refused`** — the READER refused;
* `Ok(func)` then `complete_comdat` → `Err((shape, decline))` → `FnByte::Refused`
  / `FnByte::Partial` — the reader accepted and the EMITTER declined,

and to record both under `fnbyte-shape|…` / `fnbyte-decline|…`.

**The invariant I expect to be able to state and to hold:** a codegen refusal may
exist at rest **iff it is not a function of the IL body alone**. If it is a
function of the body alone, #139 applies and it must move into the parser or the
census over-claims. If it depends on something the parser structurally cannot see
— the `/Gy` argv flag, the reference obj's data-relocation halves — then no
parser counterpart is *possible* and the state is legitimate and permanent.

So of the four `Decline` stages I predict:

| stage | prediction |
|---|---|
| `opt-mode` | **0 by construction** — census gate (b) mirrors it |
| `selector` | **0 by construction on the in-class population** — this IS `fn_gate_refusals`' invariant. Today's key conflates it with `parse-refused`, so the published `fnbyte-decline\|selector` is ~130 k and is **not** a codegen reading |
| `gy-shape` | may be nonzero; reads **0** on this workload (STATUS `partial 0`) |
| `data-ref` | may be nonzero; **this is the one stage with no possible parser counterpart** |

**Direction I expect to be wrong in: OPTIMISTIC.** Board #770 stands at ten
optimistic, one pessimistic, one hit. The optimistic failure available here is
believing the existing column is a *usable price* for the frontier when it is
mostly zero. I therefore register the pessimistic half explicitly below.

**What I expect NOT to be able to do:** produce a resting verdict that prices the
codegen distance of a body the reader refuses. That is unmeasurable by
construction — there is no `IlFunction` to hand to `select_function` — and no
instrument in this lane will claim otherwise.

---

## 1. WHAT I EXPECT THE COLUMN TO READ ON THE FOUR READER-CLEAR TUs

`undname` · `vswprnc` · `vsnprnc` · `mmio`. At **master** these are `vocab-gap`;
READER-CLEAR is a property of the poisoned/hatched ladder instrument, not of this
tree. So their emitted functions will be overwhelmingly `parse-refused`.

Registered predictions, per TU, on the master binary:

* **P1.1** — `fnbyte-decline|gy-shape` + `fnbyte-decline|data-ref` + any
  `fnbyte-shape|<s>|fnbyte-refused` with `<s> != parse-refused`: **0 on all four**.
* **P1.2** — `fnbyte-differs` + `fnbyte-reloc-differs` (reader accepted, emitter
  LOWERED, bytes WRONG — the strongest codegen reading that exists): **nonzero on
  at least one of the four**, and I expect `mmio` to be the one, because #1418
  prices it at 3 of 11 functions remaining and the other 8 must be graded
  somewhere.
* **P1.3** — across all **16** frontier TUs: **at least 1** TU has a nonzero
  `differs`+`reloc-differs`, and the **modal** reading is **0**. I expect between
  **1 and 6** of the 16 to be nonzero.
* **P1.4** — `fnbyte-exact` is **nonzero on at least 10 of the 16**. A frontier TU
  is A∧B∧C, so its `.text` is largely functions the port already gets right.

**The vacuity floor, registered before the run:** if `differs+reloc-differs` reads
**0 on all 16**, the column is real but says nothing about this frontier, and I
will publish that as the result rather than repair it into looking useful. That
is `w-5c2`'s rung — a number that is zero by construction is not a measurement —
aimed at this lane's own deliverable.

---

## 2. WHAT I EXPECT TO SHIP

1. **A repair, not a new verdict.** `Decline::Selector` is used for **both** the
   parse refusal and the selector refusal (`fnbytes.rs:640-655`), so
   `fnbyte-decline|selector` is ~130 k of reader refusals with the codegen
   reading buried inside it. Split them. Expected effect: a new
   `fnbyte-decline|parse` carrying essentially all of the old count, and
   `fnbyte-decline|selector` dropping to **0** — which must then be labelled in
   the code as zero-by-construction, or it is exactly the published-column-that-
   can-only-read-zero `w-5c2` found.
2. **A CODEGEN column on the FRONTIER block**, read off FBM, per TU, with the
   unmeasurable remainder named rather than omitted.
3. **Labelling** — every codegen price on the board that is a hand-count says so.

**Not shipped, registered as declined in advance:** a new `FnVerdict` variant, a
new resting census bucket, or anything that could let an emitter refusal exist
without a parser counterpart. #139's shape is an over-claim and an over-claim is
worse than the gap.

---

## 3. THE NUMBERS I EXPECT NOT TO MOVE

TU match **11**, mismatch **0**, codegen-gap **0**, vocab-gap **860**,
capture-fail **7**, census **711,486 / 2,463,443**, `fnbyte-exact` **36,213**,
`fnbyte-differs` **2,111**, `fnbyte-reloc-differs` **861**, `fnbyte-refused`
**130,575**, `fn_gate_refusals` **0**. Every `gap-metric` line except any this
lane adds.

If `fnbyte-refused` moves at all, this lane has changed the port's behaviour and
that is a failure of the lane, not a finding.

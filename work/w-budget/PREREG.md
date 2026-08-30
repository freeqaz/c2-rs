# `w-budget` — PREREG

> **Lane `w-budget`, wave 21 L1. Kind: CONSTRUCT RUNG.**
> Base `1d52f8902`. Brief `docs/WAVE21_BRIEF_2026-08-29.md` §2.
> Board rows reserved: **#3849**–**#3855**.
> **Frozen before any `crates/` edit.** Committed first, per the brief's §5.

---

## §0. What this lane is for, in one sentence

Wave 20 (`w-instrcount`) resolved `no-instr-count` to the `.gl` function
record's `SIZE` field, `WORD [[fn]+0x50]`, sole writer `0x10b9bf6c`, produced by
the **front end** and read verbatim by c2. The port already decodes the
producing field and throws the value away. **This lane threads it**, so that the
inline growth budget `B` becomes a number the port can name and
`S6-budget-divided` stops being a blanket refusal.

## §1. What will be built

**B1 — the field's VALUE, decoded.** `crates/c2-il` already walks past the `.gl`
`SIZE` field in `gl_function_attrs` (it steps over it to reach `ATTR`) and its
own doc says *"Nothing here uses the field's VALUE."* A sibling reader returns
it. The existing walk's whole-file `None` discipline is preserved unchanged,
including its refusal of the `0x81..=0xff` single-byte form.

**B2 — the count on `IlFunction`, three-valued.** A new field beside
`inlinable`, filled by the two bundle-level constructions from the `.gl`, with
**`None` meaning UNASKED and every consumer required to behave exactly as it did
before the field existed.** That is `inlinable`'s own rule and it is what makes
B4's byte-neutrality argument structural rather than measured.

**B3 — `B` on a production path.** `BudgetModel::seed` (C3, already `R-derived`,
`PROV[R] 0x10b62708`/`0x10b6270a`/`0x10b62715`) is today called by nothing but
the surface renderer, because `splice.rs:425` says *"The port has no honest
`caller_instrs` to pass."* With B2 it has one: the **caller's** count, which is
what c2 seeds from at `0x10b626f5`/`0x10b626f7`/`0x10b62703`.

**B4 — `NestedBudget` gains a value, so `n ≥ 2` computes instead of refusing.**
Today `NestedBudget` is `Parent | Divided { k }` and `port_enter_site` returns
`Err("S6-budget-divided")` for the second. With `B` known the divided case is
`B / k`, a number. The refusal survives **only** for the case that is still
honestly unevaluable: no count was readable (`None`).

**B5 — C2 and C16 adopted.** C2 is the seed of c2's running growth total
`DAT_10c3f5cc` (`0x10b62703`); C16 is `35000 < DAT_10c3f5cc ⇒ decline`
(`0x10b60a63`). Both are `absent`/`no-instr-count` today and both are what the
read unblocks *in the strong sense* (`WB_INSTRCOUNT_FINDINGS` §7).

**B6 — C17, conditional and measured, not assumed.** C17 (`0x10b60a73`) is
`budget < instrs && instrs > 0x28 ⇒ decline`. Both operands become derivable, but
it is a **new refusal on a production path** and a refusal that fires changes an
emit. **It is adopted only if it is measured not to fire**, and if it fires the
negative result is published and the adoption is dropped. Registered here so
that dropping it is a recorded outcome and not a quiet omission.

## §2. The registered decision-surface check — `#3723`, and a byte delta does not discharge it

`splice.budget` is already a registered surface (`c2_core::surface::SURFACES`),
so the mechanism exists; what this lane owes is that the **new parameter** is
inside it and that its domain runs **past what any fixture reaches**.

**Registered before building:**

* The seed's input becomes a swept axis of `splice.budget`'s domain, at values
  **no corpus TU can reach**, explicitly including the `0x81..0xff`
  sign-extension band `65,409..65,535` that `WB_INSTRCOUNT_FINDINGS` §4 names as
  c2's real behaviour on that encoding.
* The `n ≥ 2` rows of the existing domain **must move** — that is the whole
  adoption. A run in which `DOMAIN.txt` does not move is this lane failing to
  do its job, not this lane being safe.

**THE REFUSAL-DOMAIN CONTROL, registered with its expected colour:**

> **RDC-1.** Perturb the threaded count to a value that *must* change the
> verdict — a caller count in the `0x81..0xff` band, which seeds
> `DAT_10c3f5cc` above `35000` and makes **C16 decline the caller's first
> site**. Required: `DOMAIN.txt` moves, with the row counts printed **before
> and after**, while the byte delta stays **0**.
>
> The control is watched **RED** (`#3336`) before any verdict is quoted from
> it: the E1 assertion
> (`surface::tests::the_decision_surface_domain_matches_the_committed_baseline`)
> must be seen to FAIL with the perturbation in place, and the failure message
> must name the moved rows. A control that cannot fail proves nothing.

**Falsifier for the whole lane:** if the domain does **not** move under RDC-1,
the parameter is not reaching the decision and the adoption is decoration —
report `FAILED`, do not ship it.

## §3. Fail axis (required, non-empty — `rungs/README.md` cost clause)

**A REFUSAL-BOUNDARY axis, and it can fail with every byte identical.** This
rung replaces one blanket refusal with a computed verdict, so the thing it can
get wrong is *which points refuse*. Observed as: the rendered domain of
`splice.budget`, line for line, at points the corpus cannot reach — including
the `n ≥ 2` region, which `S2` refuses upstream, and the `65,409..65,535` band,
which the `.gl` reader refuses upstream. Both are invisible to `gate.sh`.

Named second, because a construct rung on `PortC2::build`'s path owes it:
**cost** is not claimed as an axis here and no throughput number is offered —
`MEMORY`'s *"cost readings are inside build noise"* (three builds of one commit
differ by 0.93 %) makes any sub-1 % claim unresolved, and this change adds one
map lookup per bundle.

## §4. What would falsify each claim

| claim | falsifier |
|---|---|
| the threading is byte-neutral | any non-zero line in `scripts/gate_identity_diff.sh` over the 21 rows; any `mismatch` |
| the parameter reaches the decision | `DOMAIN.txt` unmoved under RDC-1 |
| C2/C16 are genuinely adopted | `check_table.py` check 4 (WITNESS) cannot find the token, or check 6 (CITES) disagrees with the frozen footprint |
| `n ≥ 2` is a computed verdict | the domain still renders `REFUSE S6-budget-divided` at every `n ≥ 2` point with a count in hand |
| C17 does not fire | it fires — then it is **not adopted**, and the firing is published |

## §5. Deliverable 2 — the multiply-blocked audit (`#3847`)

**Registered as an audit with a registered possible answer of "no change".**
All **12** `absent` rows are examined for a second blocker. The `blocker` column
holds one cell per row; C4 provably needs two. Registered constraints:

* **No `blocker` cell is edited on a row this lane did not read.** A blocker
  cell is a verdict.
* *"The model needs a second column, and here is what each row would carry"* is
  a complete result and is registered as an acceptable outcome.
* The audit is published as a table with, per row, the **named** second blocker
  or the reason there is none, each with a citation.

## §6. What this lane will NOT do

* **No `ported` numerator for the inliner** is invented (brief §3).
* **`128` is not adopted as the inline ceiling** (`#3732`).
* **C20 is untouched.** Its `fitted` pin is the chain's closure, which the
  port's fixpoint already has.
* **No `scripts/gate.sh` row is added** (`#3691`).
* **No clause is named by token in any `crates/` comment** — ids only. Spelling
  a screened token into a doc comment has reddened a row twice.
* **No emit is licensed by anything here.** The sole judge stays real `c2.dll`
  under wibo plus a byte-exact obj compare.

## §7. Seam note, declared up front as a brief correction

The brief's §4 seam table gives `w-budget` `crates/c2-core/src/splice.rs` and
`crates/c2-core/src/surface.rs` and no part of `crates/c2-il`. **Threading the
count structurally requires `crates/c2-il`**: the `.gl` `SIZE` field's value is
decodable only there (`func/gl.rs`), the count belongs on `IlFunction`
(`func/mod.rs`), and it is filled where `inlinable` is filled (`func/bundle.rs`,
`func/census.rs`). No peer lane this wave owns any of those files — `w-emitprice`
and `w-sizetest` are barred from `crates/**` entirely and `w-gatehash` from
`crates/c2-core/**` plus a new harness test — so the extension is collision-free.
It is taken, kept additive, and reported.

## §8. Evidence this lane owes

`scripts/gate.sh --jobs 16 --require-graded` (unqualified `GATE: PASS`);
`C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` with
**both** target and pass counts; `scripts/expr_sweep.sh`; the identity diff
showing the required-zero byte delta; RDC-1's `DOMAIN.txt` counts before and
after with the control watched RED; `python3 work/w-inlmetric/check_table.py`
green; `python3 work/w-inlmetric/gen_table.py --check` green.

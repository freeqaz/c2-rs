# w-carrier — PREREG for board #1199, THE BIND CARRIER

    Lane:   w-carrier, worktree branch `wt-w-carrier` off master `7a52aa2b`.
    Rung:   board #1199 — `IlFunction` cannot spell *"this token is `params[i]`
            plus 8"*. `w-bind` paid #839's READER half and named this row.
    Status: written and committed BEFORE the first `cl.exe` of this lane and
            BEFORE the first line of `crates/` is edited.

---

## 0. What this lane is, in one paragraph

`w-bind` (`7a52aa2b`) reads the reference bind and refuses `BodyShape::StoreRunBind`
**by name** in `shape_to_function`. What is missing is a **representation**, not a
model: the ordering half is already answered by `schedule.rs`'s shipped
`xboxheap_constructor_is_derived_not_fitted`. This lane owes a place to put the
fact *"local `L` denotes `params[i] + off`"* such that the state where that fact
is dropped, or discharged twice, or collapsed into the formal, **cannot be
spelled** — not merely "is not reached".

---

## 1. THE SHAPE, registered before it is written

The carrier is **not a new field**. It is a new `IlOp` variant:

```rust
IlOp::BoundAddr { tok: u32, base: u32, off: i32 }
```

*"the token `tok`, which denotes `base + off`"* — one op, standing in the op
stream exactly where `IlOp::Load(<bound local>)` stands today, in a store's base
position or in its value position or both. `shape_to_function`'s `StoreRunBind`
arm discharges `binds` into the op stream and then builds **the carriers that
already exist**: `IlFunction { params, ops }` for the plain tail, and #844's
`CallSeq { store_run: Some(StoreRunPrefix { ops, live_args }), … }` for the call
tail. `RefBind` never crosses into `crates/c2-core`.

**Why this is unspellable rather than merely unreached**, stated now so it can be
scored later:

1. **There is no second container to lose.** A `binds: Vec<RefBind>` beside the
   ops — the obvious shape — is settable on `IlFunction` while the ops live in
   `CallSeq::store_run`, so a consumer can hold the run and drop the bindings.
   That is #232's mechanism and #844's, one layer out. With the fact inside the
   op stream there is nothing beside the ops to drop.
2. **The symbol and the address come from ONE value.** A store's base symbol
   (what `schedule::Stmt::base` keys may-alias on) and its base register +
   displacement are two derivations of the same `BoundAddr`, so they cannot
   disagree. The collapse #1128 forbids — the bound store taking `this`'s
   symbol — would require the op stream to hold `Load(this)` where `BoundAddr`
   stands, and the reader cannot produce that: it never substitutes.
3. **A consumer that does not know the variant REFUSES.** Every existing op-stream
   walk is an exact slice pattern over `Load` / `Lit` / `StoreInd`; `BoundAddr`
   matches none of them, so an unwidened consumer falls to its own
   `out_of_class`, never to a shorter body.

**REGISTERED AS THE THING MOST LIKELY TO BE WRONG about the shape**: that a
variant of `IlOp` is the right home at all. The rival is a field on
`StoreRunPrefix`, which is where #844's carrier went. If `BoundAddr` forces a
match arm in three or more modules that have nothing to do with store runs, the
rival was right and this row loses.

---

## 2. THE PREDICTIONS

| # | registered |
|---|---|
| **P0** | **The registered loss.** A bind used ONLY in a store's *base* position materialises **nothing** — c2 folds `bind.off + store.off` into the store's displacement off the formal's own register and emits no `addi`. I expect this and I expect it to be the clause that loses: if c2 materialises the reference whatever it is used for, then every bind body has a `RegisterDerived` producer, the emittable sub-class is **empty**, and this lane ships a carrier with no accept surface at all. `b_use1`/`b_use2` decide it. |
| **P1** | **`src/xdk/nuispeech/xboxheap.cpp` does NOT convert. TU match 10 → 10.** Its run mixes an interior-address producer at 2 uses with a literal at 1, and `codegen::alloc::allocate` refuses a mixed-kind run wholesale (#836, wrong on 0 over 81 cells; #868 refuted the narrow lift at 12 of 36). |
| **P2** | **#868/#836 becomes MEASURABLE, and that is this lane's headline if P1 holds.** The target's first-refusal key moves off `store-run-bind-no-emitter-carrier:eof` onto a key that names the **mixed-kind allocation** refusal, and its count over the 878-TU workload is **1** — the residue `w-bind` measured. Registered as a key motion, not as a coverage motion. |
| **P3** | **The carrier's emitted population on the 878-TU workload is ZERO.** `w-bind` measured #839's whole residue at ONE body and P1 refuses it. Registered explicitly so that no representation change is reported as breadth. `fn_in_class` moves by **0**. |
| **P4** | **The two spellings stay apart in the PORT's own emitted bytes**, on every cell the port emits. `b_target_bind` / `b_target_direct` is the standing control (#1128, four words); any cell pair the port emits both halves of must differ where real `c2`'s differ and agree where real `c2`'s agree. |
| **P5** | **The zero-offset bind and the dead bind stay REFUSED.** `w-bind` declined both because their twins are TEXT IDENTICAL and one cell pair is not a population. If the carrier makes either look takeable this lane says so and leaves it. |
| **P6** | **The reader's accept class must equal the emitter's**, and where the carrier widens what reaches `order`/`alloc` past their exact region the gate goes in the READER with its own census key — `w-seam2` §6 had to move two gates for exactly this and `census_gate.rs` is what caught it. I expect to move **at least one** such gate and to have to restore `collect_store_run`'s clause 1 (one base symbol), whose stated justification — *"neither is reached from here, because `shape_to_function` refuses `BodyShape::StoreRunBind` by name"* — this lane's own change **falsifies**. |
| **P7** | **`codegen-gap` does not grow.** If the port refuses a body the census calls in class, the census is over-claiming; the answer is a reader gate, not a `codegen-gap`. Registered as an alarm on my own work, not as a metric to move (#1164 — the metric cannot register payment either way). |
| **P8** | The `88-store-run-call` port split on master is **83 Match / 1,493 NotImplemented** (#1205, and NOT the 44/1,532 two merge messages carry). I will tally it **at both ends with a base binary built in this tree**, and I expect it to move by **0**: no bind body exists in that fragment. |

---

## 3. THE DECLINE FLOORS, registered against the incumbent

Today's refusal is **right 100 % of the time on what it refuses**. Every floor
below is a way this lane can be worse than doing nothing, and any one of them
firing means the emitter half does not ship.

| # | floor |
|---|---|
| **D1** | A cell that grades `match` today stops matching. Controls: `b_ctrl_run`, `b_dead_ctrl`, and this lane's own leaf/direct controls. |
| **D2** | Any cell, anywhere — grid, fixture, sweep, cross — grades `Port=Mismatch`. The response is a **refusal keyed on a measured mechanism plus a fresh holdout on an axis the derivation held fixed**, never a narrowing around the failing cell (`w-seam2` F-1, which bound twice). |
| **D3** | Any scan alarm moves: `mismatch` off 0, `fnbyte-exact` below 36,212, `fnbyte-differs` above 2,111, `fnbyte-reloc-differs` above 861, `fnbyte-match-tu-differs` or `-reloc-differs` off 0. |
| **D4** | The port emits the **same bytes** for a bind cell and its direct twin where real `c2`'s differ — or different bytes where real `c2`'s are identical. That is #1128 collapsing, which is the whole reason the reader keeps the token. |
| **D5** | The census counts in class a body `PortC2` refuses. Measured with `census_gate.rs` and with a per-cell census/gap cross-check, **not** argued from the code. |
| **D6** | `fn_blockers` or `emit_blockers` **totals** move by more than the bodies this lane can name one by one. |
| **D7** | **A gate that refuses nothing** (#1175). Every gate this lane adds must be shown to FIRE on a named cell, by a count, not by reading the predicate. A gate keyed on the wrong predicate is indistinguishable from no gate. |

---

## 4. THE GRID, and what it must NOT hold constant

Frozen by `sha256` manifest **before the first `cl.exe`**, one directory per cell
(#1045), every cell graded by real `c2.dll` under wibo at the **workload's own**
`/GR /O1 /Oi /EHsc` (#1112), each accept cell **paired with a control** that is
the same body with the bind removed.

The question *"what does my grid hold constant"* has now been paid for four
times, most recently by a 63-cell grid and a 1,576-case corpus that were green
through two wrong emits because both held **the stored value against the call's
argument list** fixed. The axes this grid must vary, named before the cells
exist:

1. **the ROLE of the bound name** — base only, value only, both (the target's);
2. **the number of producers beside it** — 0, 1, 2 — and their **kind**, because
   that is the axis #836/#868 lives on;
3. **the bind's position** in the run — first, middle, last;
4. **the displacement** — the target's 8, and a second value, because #856 says
   this is a one-byte IL axis;
5. **the tail** — plain run and #1129's call, since `try_parse_store_run_bind`
   admits both;
6. **the number of binds** — one and two, off the same formal and off two;
7. **the base formal** — `this` and a second pointer formal.

And the thing this grid deliberately does **not** hold constant, because the
previous four grids did: **the number of stores between the bind and its first
use**, which is what `layout_slots`' symbol-crossing count is computed over.

Cells are crossed **against each other** in emitted bytes (`twins`), not only
graded by verdict label — that, and not the corpus, is what caught `w-seam2`'s
two wrong emits.

---

## 5. WHAT THIS LANE WILL NOT DO

* It will **not** fit an allocation rule to `xboxheap`. Six keys died that way and
  the seventh died on a frozen holdout; #1134 refutes clause 1 on this very mix.
* It will **not** lift #836/#868's mixed-kind refusal. If the carrier makes the
  lift look cheap, that is a finding and it goes in §Found-and-not-taken.
* It will **not** convert the dead bind or the zero-offset bind.
* It will **not** quote #866 as grounds for confidence — it is REFUTED in general
  (`w-seam2` §4), and it was quoted to two lanes before that was known.
* It will **not** assume the schedule is monotone in liveness (#1189).
* It will **not** report a representation change as breadth (P3).
* It will **not** tally only at its tip (#1205).

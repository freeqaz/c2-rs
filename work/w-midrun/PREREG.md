# w-midrun — PREREGISTRATION

Committed **before the lane's first probe obj and before the first line under
`crates/`**. Base master `503f8937` (the merge of `w-mrslot`).

The rung: **the EMITTER rung at the bottom of `src/xdk/nuispeech/xboxheap.cpp`'s
ladder** — `codegen::leaf::store::parse_simple_gpr_run` declining an **interior
address** in a store's VALUE position, which is what `c2rs gap` names as

```text
  codegen-gap  "a store run before a call whose values are not all a formal or
                a literal"
```

once the reader rung (peer lane `w-mixkind`'s mixed-kind clause) is lifted.

---

## 0. What the record already says, and the two corrections this lane carries in

Read before a line was written (`grep -ril` over `docs/`+`scripts/`, then
`docs/BOARD.md` by topic, oldest hit last):

* **`w-mrslot` §5.1 — board #1218's bottom rung is misidentified.**
  `leaf/store.rs`'s `value_bound` refusal has **no reachable input**:
  `bind_run_ops`' discharge loop rewrites the **BASE** position only. Confirmed
  by reading `crates/c2-il/src/func/body/shapes/leaf_store.rs:2246-2264` — one
  `out.push(IlOp::BoundAddr { .. })`, in the `b` arm. `alloc.rs`'s module doc
  still carries #1218's wording; it is peer lane `w-mixkind`'s file this week and
  is **not** edited by this lane unless that lane is provably out of it.
* **Board #879 / `w-fnbyte` §5.4 — `fnbyte-shape-framed-differs` is 123 and
  `framed-exact` is absent.** Registered here as a *hazard to measure*, not to
  assume away: **PRED-F** below states what this lane expects the relationship
  to be and how it will be checked.
* **`w-carrier` §9 item 4 — "the address-producer family … widening
  `parse_simple_gpr_run` to the four-op group would convert BOTH halves at once
  and is the cheaper door."** This lane takes exactly that door. The reason
  `w-carrier` gave for declining it — *"their objs are byte-identical to their
  direct twins, and the direct twins are refused (F2's four-op group)"* — is
  **partly stale on `503f8937`** and this lane opens on that (§1).

## 1. THE INCUMBENT, measured on `503f8937` before any change

`c2rs census --flags-file work/dc3-workload/flags.txt` over `w-carrier`'s own
committed GRID K cells:

| cell | spelling | census | first-refusal key / gate |
|---|---|---|---|
| `k_both1` | BIND, 1 use, leaf | 0/1 | `store-run-bind-address-producer:eof` |
| `k_both2` | BIND, 2 uses, leaf | 0/1 | `store-run-bind-address-producer:eof` |
| `k_both1_c` | **DIRECT**, 1 use, leaf | **1/1 in class as `store-leaf`** | **`census/gate DISAGREEMENT: 1` — `not implemented: sub-object address feeding arithmetic; out of class`** |
| `k_both2_c` | **DIRECT**, 2 uses, leaf | **1/1** | **same disagreement** |
| `k_val1` | BIND + literal | 0/1 | `store-run-bind-mixed-kind-alloc:eof` (peer lane's) |
| `k_target` | `xboxheap`'s ctor | 0/1 | `store-run-bind-mixed-kind-alloc:eof` (peer lane's) |

`c2rs diff`: `k_both1_c`, `k_both2_c`, `k_val1_c` are all
`ReferenceReplay=ByteExact · Port=NotImplemented`.

**So the incumbent is two different things and only one of them is clean:**

* **BIND half** — `bind_run_ops`' `STORE_RUN_BIND_ADDR_PRODUCER`. An honest
  reader refusal. **Wrong on 0.**
* **DIRECT half** — the census counts the body **in class** and `PortC2`
  refuses it. That is a **live `census/gate DISAGREEMENT`**, i.e. exactly the
  invariant `crates/c2-core/src/codegen/select.rs`'s `function_gate` exists to
  hold at 0. It is **not** a wrong emit (nothing is emitted) but it *is* an
  in-class claim with no byte behind it — `docs/STATUS.md` trap 2.

## 2. THE RULE THIS LANE PROPOSES — M1

> **An interior address in a store run's VALUE position is a PRODUCER**: one
> `addi rD, rBase, off`, register-derived, scheduled and allocated by the same
> `codegen::order` / `codegen::alloc` the literal producer already goes through.
>
> Two spellings, one producer:
>
> * **DIRECT** — the four-op group `[Load(b), Load(vb), AddrOf{voff},
>   StoreInd{off,width}]`; `rBase = reg_of(vb)`, the producer's offset is `voff`.
> * **BIND** — the three-op group whose value is `IlOp::BoundAddr{tok,base,off}`;
>   `rBase = reg_of(base)`, the producer's offset is `off`. Reaching this at all
>   needs `bind_run_ops`' `STORE_RUN_BIND_ADDR_PRODUCER` arm lifted **and** the
>   discharge loop extended to the VALUE position, which is what makes
>   `SimpleStore::value_bound` reachable for the first time (`w-mrslot` §5.1).
>
> **CSE identity**: two address values are ONE producer iff they agree on
> `(base token, offset)`. The id space is disjoint from the literal producers'.
>
> **DOMAIN — drawn deliberately inside every model's:**
> 1. **exactly one distinct producer in the run, and it is the address** — no
>    literal anywhere. The run is therefore **single-kind** by construction and
>    `alloc::allocate`'s mixed-kind refusal (#836/#868/#1134) is **never
>    consulted**. Peer lane `w-mixkind` owns that refusal and this lane does not
>    touch `alloc.rs` or `STORE_RUN_BIND_MIXED_KIND`.
> 2. **`off != 0`.** At `off == 0` c2 emits **nothing at all** (`IlOp::AddrOf`'s
>    own measured doc), so the value *is* the base register and there is no
>    producer. That is a different shape with no grid; it is REFUSED.
> 3. the displacement fits a signed 16-bit field (`addi`'s own).

## 3. PREDICTIONS

Each is registered with the cell class that can refute it.

* **PRED-1 (the rung).** On a run with exactly one interior-address producer at
  `off != 0` and `N` formal-valued stores, the port emits `addi r11, rBase, off`
  placed by `order::layout_slots` and the stores in `order::store_order`'s
  order, and the obj is **byte-exact** against real `c2.dll` at the workload's
  own flags — in **both** spellings, **both** as a leaf and as the #844
  composition. Refuted by any `Port=Mismatch` in GRID M's `dom` class.
* **PRED-2 (the spellings DIFFER at width, and both are predicted).** `w-carrier`
  §4.2 measured `k_both1`/`k_both2` byte-identical to their direct twins — at
  **zero** formal stores, where one symbol and two agree. Board **#1128**
  measured the opposite at `xboxheap`'s width. **This lane predicts the two
  spellings' objs DIVERGE as soon as the run carries ≥1 formal store through
  `this`, and that `order::schedule` predicts both from the base symbol alone.**
  A grid that came back "identical everywhere" would mean the grid is too narrow,
  not that #1128 is wrong.
* **PRED-3 (`off == 0`).** Real `c2` emits no `addi` and stores the base
  register itself. The shipped rule REFUSES it; the grid grades it anyway so the
  answer is on record for the next lane.
* **PRED-F (the framed hazard, board #879).** This lane's call-tail cells select
  as **`Selected::Seq`** (`func.call_seq` — `store_run_call` + `calls::call_seq_text`),
  **not** as `Selected::Framed` (`func.framed_call`). The 123
  `fnbyte-shape-framed-differs` are the `Framed` arm, a different selector arm,
  and `w-mrslot`'s GRID R already has **142 byte-exact cells** on the `Seq` path.
  **Checked, not assumed**: `fnbyte-shape-framed-differs` is reported at both
  ends, and every call-tail cell in GRID M is graded whole-obj by `c2rs gap`,
  which does not care which arm produced the bytes.
* **PRED-D (the census/gate disagreement).** Shipping M1 takes the DIRECT half's
  `census/gate DISAGREEMENT` from 1 to 0 per cell — by making the **gate agree
  with the census**, never by narrowing the census.

## 4. THE DIRECTION THIS LANE EXPECTS TO LOSE IN

**`codegen::order` — the PLACEMENT of the producer (`layout_slots`) or the STORE
ORDER (`store_order`) — not `codegen::alloc`.**

Both order models were fitted on runs whose producers are `li`. A `li` reads no
register; an `addi` reads `rBase`, so it has a real dependence and c2's scheduler
has a reason to move it that it never had for a constant. `leaf/store.rs`'s own
multi-word-literal refusal is the precedent that ORDER's population is narrower
than it looks — *"ORDER is fitted on single-word `li` values only"*.

The allocation is **not** where this lane expects to lose, and the reason is
structural rather than optimistic: at one producer `allocate` returns `POOL_TOP`
whatever the kind, so #836/#868/#1134 — every one of which is about the mix —
cannot decide a cell in this domain.

Secondary, registered so it cannot be claimed afterwards:

* **L2** — the `ROOT=OTHER` class (the address of one object stored into a
  different one) adds a base-symbol crossing and may fall outside
  `layout_slots`' `MAX_SYMBOL_CROSSINGS`.
* **L3** — **two distinct address producers** (still single-kind, so `allocate`
  answers) are declared **OUT of the shipped domain** and graded anyway. If they
  come back byte-exact, that is a finding for the next lane and **not** a licence
  to widen this one after the grade.

## 5. THE DECLINE FLOOR, NAMED AGAINST THE INCUMBENT

The incumbent (§1) is a **refusal, wrong on 0** on the BIND half and a refusal
wrong on 0 *plus* a live census/gate disagreement on the DIRECT half.

**Therefore the floor is not a percentage — it is zero.**

1. **M1 ships only if it is wrong on 0 of the graded cells in its declared
   domain.** One `Port=Mismatch` anywhere in `dom` and the rung is DECLINED and
   reverted, because a rule right on 95 % of a domain loses to a refusal that is
   right on 100 % of it. `mismatch` is the project's only correctness criterion.
2. **M1 ships only if it strictly converts.** Right-on-0-but-converts-0 is a
   no-op and is reported as a decline, not a win.
3. **A mismatch may be fenced only STRUCTURALLY** — by a clause stated over the
   construct (a kind, a spelling, an offset, a symbol count) that was in the grid
   *before* the grade. **Narrowing around the failing cells is forbidden**;
   `alloc.rs`'s standing prohibition (*"a successor may not narrow around
   `SELF-2B`"*) is the record of what that costs, and six allocation keys died
   that way.
4. **`census/gate disagreement` must not rise anywhere**, on the grid or on the
   878-TU scan. Widening the emitter while the census stands still is the one
   direction that lowers it; the reverse is forbidden.
5. **Every standing gate stays green**: `scripts/gate.sh` (all 12 lanes + the
   generated sweep + the mode cross), `c2rs bench`, `cargo test --workspace`,
   and the 878-TU scan's `mismatch 0`.

## 6. WHAT THIS LANE WILL NOT TOUCH

* `crates/c2-core/src/codegen/alloc.rs` — peer lane **`w-mixkind`**. Not one
  line, including its stale #1218 module-doc paragraph.
* the `STORE_RUN_BIND_MIXED_KIND` key and the `lits`-non-empty arm of
  `bind_run_ops` — same lane.
* `crates/c2-core/src/codegen/coff.rs` — the compiler-label counter. Both
  historical six-wrong-bytes defects came from there; this lane holds its rung
  rather than accept that collision.

**Shared surfaces named in advance, to be re-checked at merge as a question
distinct from "did git report a conflict":** `Producer`, `ProducerKind`,
`parse_simple_gpr_run`, `reg_of`, `order::lead_slots`, `order::head_slots`,
`save_slot`, `SimpleStore`, `alloc::Root`, `alloc::ProducerRoots`. Any change
to a shared type is a **WIDENING that serves every consumer** — never a
narrowing and never a shadowing predicate. `work/w-splice/peerkeys.py` is run at
both ends and any vanished key family is reported.

## 7. EVIDENCE THIS LANE OWES, AT BOTH ENDS

TU match · mismatch · codegen-gap · vocab-gap · capture-fail; the whole 139-line
`gap-metric` block `diff`ed; `fn_blockers`/`emit_blockers` row diff;
`fnbyte-shape-framed-differs`; sweeps `88-store-run-call` and
`89-store-run-live-arg` port splits; `cargo test --workspace --release` total
**and target count**, plus `git grep -c '#\[test\]'` base vs tip; GRID M's own
match/mismatch/gap split; and **the ladder** — what `xboxheap.cpp` names next,
measured on the real dc3 TU at the workload's own flags.

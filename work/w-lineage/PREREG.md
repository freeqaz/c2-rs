# w-lineage — PREREGISTRATION

Committed **before this lane's first probe obj and before its first line under
`crates/`**. Base: master `04727f37`, branch `wt-w-lineage`.

The brief splits this lane in two and requires the split to survive into the
commits:

1. **carry the full bind lineage** — the transitive chain, not one link;
2. **only then** state a rule over it, freeze a grid, and grade.

This file registers both, plus the thing neither of them is: a **reach claim**
that, if it holds, changes what part 2 can even be about.

---

## 0. What this lane may and may not ship (registered before any measurement)

* **F-1.** If the rule below is wrong on **≥ 1 in-domain cell** of the frozen
  grid, it is **DECLINED**. The floor is the INCUMBENT — `alloc::allocate`'s
  mixed-kind refusal, which is wrong on **0** cells of every grid ever thrown at
  it (81 `w-alloc2` · 62 GRID M · 72 GRID Z · 81 GRID P · 60 GRID X). A refusal
  is never wrong; a rule at 1 wrong is strictly worse than it, because
  `mismatch 0` is this project's only correctness criterion.
* **F-2.** The carrier (part 1) ships **whatever the grid says**, because it is
  additive and carries no verdict. `allocate` must be **untouched by part 1**
  and that is checked mechanically (`allocate_ignores_the_roots_carrier`), not
  asserted.
* **F-3.** If the rule ships, `alloc::allocate` may read the carrier **only**
  through a predicate that **refuses when the lineage is not COMPLETE** — walked
  to a provably non-bind base. A rule that reads a partial lineage as if it were
  whole is #232's direction: a later reader widening turns a clean refusal into
  a wrong emit.
* **F-4.** No clause of the rule may be narrowed around a failing cell of this
  or any prior grid. That is how eight of the eleven dead keys were written.

---

## 1. THE REACH CLAIM — `H-REACH`, and it is not a rule

> **Every `alloc::Root` with `is_bind == true` that today's reader can produce
> has `base` = a FORMAL. So its bind lineage is exactly ONE link deep, and
> `Root::base` is COMPLETE over the whole reachable population.**

**The mechanism, read off the source and to be MEASURED rather than asserted**
(`crates/c2-il/src/func/body/shapes/leaf_store.rs`):

* `parse_ref_bind_stmt` builds a `RefBind` only if the bind's value parses
  through `parse_addr_value`, which ends `params.iter().position(|&t| t ==
  vbase_tok)?` — **the bind's base must be a formal**;
* and it refuses `off == 0`.

A chained bind `T& c = a;` fails **both**: its base is another bind's token and
its displacement is 0.

**If `H-REACH` holds, then over the reachable population `H-CHAIN`, `H-DERIV`,
`H-STEP` and `H-2Z` are ONE PREDICATE**, and board #1266's shortfall — real,
decoded, and correctly stated — is **unreachable**: no consumer can be wrong
about a link that no input contains.

### 1.1 Predictions, registered before the first cell is compiled

| id | prediction | how it is falsified |
|---|---|---|
| **R1** | Every GRID X family carrying a **chained** bind (`M5` `CHAINBIND`, `M6` `DEEP-GP`, `M7`, `M9` `REVERSE`, and any other whose bind's base is a bind) is **out of the reader's class**, and `c2rs census` reports a key that is **NOT** `store-run-bind-mixed-kind-alloc` | any one of them reporting the mixed-kind key |
| **R2** | Every GRID X family whose binds all hang off **formals** reports **exactly** `store-run-bind-mixed-kind-alloc` — this lane's key is the only thing between them and the emitter | one of them reporting a different key, which would mean the reachable domain is smaller than this lane thinks and the grid is out of regime |
| **R3** | The **value** of an address producer inside a bind run can only ever be a **whole bound object** (`(int)&a`), never a path (`(int)&y->blk.q0`): `bind_run_ops` matches groups of exactly three ops and a path value is two ops. So **`SELF-2B` — the class that killed `H-MIX` — is also out of reach** | a `SELF-2B` cell reporting the mixed-kind key |

**R1 is the one I expect to lose on.** #1267 decoded the depth-3 chain out of
the `.ex` with `exdec.py`, not through this crate's reader, and there is a second
path into `RefBind` I have not traced to the bottom
(`parse_base_member_designator` inside `parse_addr_value`, which could in
principle admit a bind token as a base). If R1 fails, the lineage is reachable,
part 2 must answer the deep cells, and `H-LIN` below is graded against them.

---

## 2. PART 1 — THE CARRIER

Additive, and it is the model `w-prod` set: widen the shared type, disturb no
consumer, and **check** `allocate` is untouched rather than assert it.

```rust
pub struct Root {
    pub tok: u32,
    pub is_bind: bool,
    pub base: Option<u32>,
    pub offsets: Option<Vec<i32>>,
    /// NEW — the TRANSITIVE bind lineage, or `None` for "not carried".
    pub lineage: Option<Vec<u32>>,
}
```

`Some(v)` means: `v[0] == tok`, `v[i+1]` is the bind token `v[i]` is bound to,
and the walk **terminated at a provably non-bind base**. `None` is an honest
refusal and never a truncated list — #908's rule, applied to a second field.

**Today `Some(vec![tok])` is the only value any reader can produce**, and that
is `H-REACH` restated as an executable fact. It is not dead weight: it is the
**guard**. `IlOp::BoundAddr`'s base must resolve to a register for the run to be
emitted at all, so a reader widened to chained binds makes `reg_of` fail and the
run decline — the carrier cannot silently go stale in the wrong direction, and a
test pins that.

`ProducerRoots::lineage_related() -> Option<bool>` — `None` when either side is
uncarried; otherwise the two roots are the same token, or one's lineage contains
the other's token. **`bind_linked` is not changed and not shadowed** (#1266's
one-link method stays, with its doc).

---

## 3. PART 2 — THE RULE, ITS RIVALS, AND THE DECLINE FLOOR

```text
  H-LIN   the address producer takes POOL_TOP (r11)  iff  cu <= ru + 1 + d

            d = 1 when  the STORE designator's root is a BIND HEAD
                  AND   ru >= 2
                  AND   lineage_related(value, store) == Some(false)

          and `allocate` REFUSES the run outright when
          lineage_related(..) is None — an uncarried lineage is not a false one.

          DOMAIN: exactly two producers, one an interior address, one `li`.
          ru = the address producer's use count, cu = the literal's.
```

`H-LIN` is `H-DERIV` (#1265) **plus the completeness guard**, stated over the
transitive carrier instead of over a one-link approximation.

### 3.1 Rivals scored on the frozen grid

| rival | where it must differ | status on record |
|---|---|---|
| **the shipped refusal** | **THE DECLINE FLOOR** — it emits nothing, so it is wrong on 0 | incumbent, 356 cells, 0 wrong |
| `H-DERIV` (#1265) | **A DECLARED TWIN of `H-LIN` on any cell whose lineages are complete.** The generator must say so rather than report two results | 0 wrong of GRID X's 60, no standing |
| `H-CHAIN` (#1264) | `REVERSE` | refuted, 2 of 60 |
| `H-2Z` (#1243) | `CHAINBIND` | refuted, 3 of 81 |
| `H-2X` (#1227) | `MIRROR` | refuted, 12 of 72 |
| `cu <= ru+1` (#892) | `TWOBIND` at `cu = ru+2` | refuted, 12 of 62 |
| `cu <= ru+2` (#1221) | `SAME`/`MIRROR` at `cu = ru+2` | refuted, 28 of 60 |
| `always-prod` (`w-heap` §4.1.1) | everywhere above the frontier | refuted, 40 of 60 |
| `clause-1` | the whole tie regime | refuted, 20 of 60 |

**`H-LIN` and `H-DERIV` are DECLARED TWINS by name**, on `w-prod`'s #1246
precedent: a grid that cannot tell two rivals apart has to say so, and the
generator's separation assertion exempts exactly this pair and no other. If any
*other* pair of scored rivals turns out structurally indistinguishable on the
frozen grid, the generator **writes nothing**.

### 3.2 Classes the grid must contain

The brief names three and they are all **out-of-reach controls** under
`H-REACH`, which is why they are graded for R1 and not for the rule:

* depth-3+ lineages (`DEEP-GP`), `CHAINBIND`, and **both role orders** (`M5`/`M9`
  — the same declarations with the roles exchanged, `const` on both).

Inside the reachable domain the grid must cross, as STRUCTURAL axes:

* **the relation between the value's bind and the store's root**: `SAME`
  (the value's own bind — `xboxheap`'s own class), `MIRROR` (the formal's path),
  `TWOBIND-alias` (a second bind on the SAME sub-object), `TWOBIND-other`
  (a second bind at a different offset of the same formal), and **`XOBJ`** —
  a second bind off a **DIFFERENT FORMAL**, which is #1265's *"a bind chain
  crossing two different objects"* and which **no lane has compiled**;
* `(ru, cu)` at and around the frontier: `cu = ru+1`, `ru+2`, `ru+3`, at
  `ru` 1, 2, 3;
* **arity** — 2 formals against 3, which moves `pool_floor`;
* values vary inside every cell (displacements, literals, store counts).

### 3.3 The direction I expect to lose in

**`XOBJ`.** Every recorded `prod` answer comes from two binds off ONE object, and
every rule on record reads the two roots' *identity* while c2 may be reading
whether the two addresses can alias. `XOBJ` is the first cell where "a different
bind" and "a different object" come apart. **If `XOBJ` answers `const` at
`cu = ru+2`, `H-LIN` is wrong there and this lane declines**, and the surviving
statement is *"the bonus is a property of the OBJECT, not of the token"* — which
no carrier on record can state.

Secondary: **`TWOBIND-alias` vs `TWOBIND-other`.** GRID Z's `Z6` is the alias
form; if the two disagree, the relation is not a function of the bind graph at
all.

### 3.4 If the registered failure mode never fires

`w-midrun` wrote it plainly and it is registered here in advance: *a prereg whose
named failure mode never fires was not tested by its grid, only survived by it.*
If `XOBJ` and `TWOBIND-alias` both land where `H-LIN` predicts, this lane says so
in those words and does **not** report the grid as having tested the direction.

---

## 4. The ladder, and what "converting" would mean

`xboxheap.cpp` is at **one reader rung** (`STORE_RUN_BIND_MIXED_KIND`) **plus one
emitter rung** (`w-midrun` paid the emitter rung; `w-mixkind` §5 re-measured the
ladder at two rungs and found rung B reports `census/gate DISAGREEMENT: 1`).

Registered before measuring: **the ctor's own cell is `cu = 1, ru = 2` in the
`SAME` class**, where `d` cannot matter — `1 <= 2+1` already. So if this lane
converts `xboxheap`, **the conversion is not evidence for the disputed clause**,
and this lane must say so rather than bank it. The disputed clause lives only at
`cu = ru+2`.

Conversion requires, and this lane will report each separately:

1. the rule shipped into `allocate` at 0 wrong;
2. the reader key lifted, with `census/gate disagreement` back to **0**;
3. a **byte-exact obj** for `src/xdk/nuispeech/xboxheap.cpp` at the workload's
   own flags and cwd, against real `c2.dll` under wibo.

Any of 1–3 failing is reported as a decline with the number attached, not as a
partial conversion.

---

## 5. Board rows

`#1294`–`#1303` are this lane's. Unused numbers are left **unminted** and said
so.

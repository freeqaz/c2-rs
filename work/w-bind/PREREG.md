# w-bind — PREREG

    Lane:   w-bind, worktree branch `wt-w-bind`
    Base:   master `62af9b75`
    Rung:   board **#839** — THE REFERENCE BIND, the reader half
    Written: BEFORE the first `cl.exe` of this lane. Committed before the grid
             manifest, which is committed before the first cell is compiled.

Everything below is registered *before* any probe obj of this lane exists. The
scorecard in the rung reproduces this table verbatim and grades each row.

---

## 0. What is being consumed, not re-derived

`w-f23` §5.1 priced #839 to the byte and this lane does not re-measure it:

```text
  26 11 0a                          the destination — a LOCAL reference variable
  b9 0f 0a a6 43 81 20              `this`
  33 86 41 74 08 27 a6 43 98 20     + 8
  32 86 43 9b 20 4b                 stored, discarded
```

then lines 9–10 use that local's token as the store **BASE**:

```text
  b9 11 0a … 33 86 41 74 00 27 …  b9 11 0a …  32 …  4b
```

So `26 11 0a` **measures as a store whose DESTINATION is a `.sy` automatic**,
and the value stored is F2's sub-object address. `parse_store_stmt`
(`crates/c2-il/src/func/body/shapes/leaf_store.rs:405`) requires the base to be
a formal — `params.iter().position(|&t| t == base_tok)?` — so both later stores
refuse and the run never forms.

Two obligations, not one (w-f23 §5.1, board **#1160**):

1. a `26 <tok>` local admitted as a store **destination**;
2. that local admitted as a store **base**, carrying **its own base symbol**.

And two facts from the emitter side that this lane must not contradict:

* **#1128** — the two spellings emit **different bodies**: both producers swap
  and one store moves. A reader that collapses them emits the other body's
  words, which is board **#232**.
* **#865/#856** — the axis is *the number of distinct store-base values in the
  body*, and a `0x26` bind at displacement **0** does **not** make a second one.

---

## 1. The shape of the answer, registered before it is built

**The production will be a NEW `BodyShape` variant that `shape_to_function`
refuses BY NAME**, exactly as `w-f23`'s F3 did, and **`parse_store_stmt` will
keep refusing a local base on every path an emitting shape can reach.**

The reason is registered rather than discovered: a widened `parse_store_stmt`
would make `BodyShape::StoreRun` and `BodyShape::StoreLeaf` form over bodies
that today refuse, and those two shapes **do** reach `codegen`. That is #232's
mechanism verbatim — a refusal that becomes an emit. So the widening is put
somewhere that cannot reach an emitter at all, and the emitter's own
`reg_of(local) == None` is a *second* line, not the first.

**Registered consequence: `xboxheap.cpp` does NOT convert and TU match stays
10.** Two refusals remain on it and neither is this lane's or in this lane's
crate: **#868/#836** (`alloc`'s mixed-kind refusal, `crates/c2-core`) and — for
the bind spelling — the fact that no carrier exists for "a bind + a run + a
call" even after #844 gave the run+call one. Registering the target as
unreachable *before* probing is the only way the lane can be graded on what it
measured instead of on what it wanted.

---

## 2. The decline floor, registered against the incumbent

Today's refusal is **right 100 % of the time on what it refuses** — it emits
nothing, so it is never wrong. A reader that is *mostly* right is strictly
worse. The lane **declines and reverts** if any one of these fires:

| # | floor |
|---|---|
| **D1** | any frozen grid cell whose `c2rs gap` verdict is `match` at BEFORE is anything other than `match` at AFTER |
| **D2** | any frozen grid cell goes to `Port=Mismatch`, at either instrument, ever |
| **D3** | the 878-TU scan's `mismatch` is not 0, or `fnbyte-exact` shrinks below 36,212, or `differs` grows above 2,111, or `reloc-differs` grows above 861, or `match-tu-differs`/`match-tu-reloc-differs` leave 0 |
| **D4** | the BIND cell and the DIRECT cell receive the **same** reading (same census family and same first-refusal key) while their **reference objs differ in the .text bytes**. This is the acceptance criterion stated as a floor: the two spellings must stay distinguishable, and the evidence must be the emitted bytes, not the source |
| **D5** | `fn_blockers` total or `emit_blockers` total moves by more than the rows this lane names — i.e. a third family absorbs something |
| **D6** | the production admits a body whose bound local is **not positively a `.sy` automatic**. Absence from `.gl` proves nothing (`assign.rs`'s own history: a file-scope `static int sv` appears as `$sv` and looked local) |

D1/D2/D3 are hard reverts. D4 is a hard revert *of the reading*, not of the
lane: it would mean the production must carry the base symbol differently.

---

## 3. The predictions

| # | registered |
|---|---|
| **P0** | **THE REGISTERED LOSS — I expect to lose on DISPATCH ORDER, not on the grammar.** `26 <tok>` is already read at three sites — `assign.rs` (a destination push, `assign-dst-not-formal`), `no_effect.rs:573`, `cond_tail.rs:225` — and the *first* of those is a whole-body production over statement lists that this lane's bind statement is a prefix of. I predict the new production will be reached only after `assign.rs` has declined, that at least one grid cell will be swallowed by an existing production and print the wrong key, and that finding out which one costs the largest single block of this lane's time |
| **P1** | **`26 11 0a` is a store into a `.sy` automatic of kind `TYPE_KIND_DATA_PTR`** — i.e. the bound reference appears in `SyView::ptr_locals`, not in `locals` (the int automatics). If it appears in neither, the production has no positive membership test and the lane declines under D6 |
| **P2** | **The zero-offset bind is EXCLUDED, for F2's own reason at §3.2.** `#865`/`#856` measured that a bind at displacement 0 does not make a second store-base value, so admitting it with its own base symbol would be a *wrong reading* of the schedule even though nothing emits. I register the exclusion **before** the cell that tests it exists |
| **P3** | **`xboxheap.cpp` does not convert. TU match 10 → 10.** Named blockers: #868/#836 (`c2-core`, `alloc`'s mixed-kind refusal, owner: whoever lifts #868) and the absence of a bind-carrying carrier (`c2-core`, `IlFunction`/`CallSeq`, the successor to #844) |
| **P4** | **`IlBundle::functions()` does NOT widen — the count will be 0**, because the new shape is refused by name in `shape_to_function`. If it moves off 0 I will say so explicitly and prove the newly-accepted TUs still emit correctly, per the brief's alarm clause |
| **P5** | **`codegen-gap` on the 878-TU scan stays 0**, and that is board **#1164** and not a null result: the partition is per TU and every vocab-gap TU carries another undecodable body. The payment will be visible in per-function `fn_blockers`/`emit_blockers` and on single-function grid cells only |
| **P6** | **The residue is SMALL — I register ≤ 40 IL bodies over the 878-TU workload**, and I register that a count of **0** is a real possible outcome and would be a finding (F2's was 0). The number will be published whatever it is |
| **P7** | **The BIND and DIRECT cells' reference `.text` differ** — `w-heap` §4.2 says both producers swap and one store moves. I will show the two disassemblies side by side from **this lane's own** captures, not quoted from #1128, because a control I did not run is not a control |
| **P8** | **The bind used only ONCE, and the DEAD bind, are different cells and I expect them to read differently from the target.** A dead bind (`auto& l = m;` never used) is a statement c2 can delete entirely; a bind used once has one store-base value's worth of consequence. I predict the dead-bind cell's reference body is **identical** to the same body with the bind line removed |
| **P9** | **A bind to a member of a NON-`this` pointer formal reads the same as a bind to a member of `this`** — the base is a formal either way and the production must not care which. This is the axis my grid would otherwise hold constant, and it is varied on purpose (board #866's refutation, `w-seam2`) |

---

## 4. What this grid holds constant, asked before it is built

`w-seam2` found board #866 false in general because **both** its 63-cell grid and
its 1,576-case generated corpus held one structural axis constant. So, named up
front, what this grid **cannot** separate:

1. **The bound object is always a struct sub-object at a compile-time constant
   offset.** A bind through an array subscript with a variable index, or through
   a function call returning a reference, is *not* in the grid and the
   production will refuse both — but the grid cannot prove the refusal is for the
   right reason.
2. **The bind is always a `&`-reference or a pointer at the C++ level.** A
   `const&` cell is present; an rvalue reference is not.
3. **Every cell is a single-function TU**, because board #1164 says TU-level
   motion is only visible there. So the grid says nothing about a bind inside a
   TU that carries other bodies.
4. **The workload flags** `/GR /O1 /Oi /EHsc` throughout (board **#1112**);
   `/Ox` is a different population and this grid does not speak for it. The
   generated sweep runs at its own flags and is reported separately.
5. **All widths are 4.** No `stb`/`sth`/`std` through a bound reference.

Item 1 is the one I most expect a successor to find something behind.

---

## 5. Method commitments

* **Freeze before compile**: `work/w-bind/GRID.sha256` committed before the
  first `cl.exe` of the grid. A re-freeze, if a cell does not compile, is
  recorded in the rung the way `w-f23` §2.1 recorded its own.
* **One directory per cell** (board **#1045**).
* Every cell graded by **both** instruments — `c2rs census` and `c2rs gap` (the
  sole judge: real `c2.dll` under wibo, byte-exact obj compare with
  `TimeDateStamp` zeroed) — at the workload's own flags, with explicit
  `NO-VERDICT` / `NO-DIFFERENTIAL` lines so a blank cannot read as a clean run.
* **BEFORE is graded with master's own binary**, so motion is a measurement.
* **Cells are crossed against each other**, not only against the corpus.
  Board **#1174**: `88-store-run-call.py`'s 1,576 cases were at 0 mismatch
  through two wrong emits, and the cell that caught them was a hand-written
  cross-product. A green sweep is **not** sufficient evidence here and this lane
  does not treat it as such.
* `work/w-splice/peerkeys.py` at both ends, reported.
* Concurrent lane **w-gen2** owns `scripts/sweep.d/`; this lane does not touch
  it. This lane owns `crates/c2-il`'s `.ex` body reader and does not touch
  `crates/c2-core`.
* **Grep for an existing reader before adding one.** Already done for `0x26`:
  `assign.rs:85`, `no_effect.rs:573`, `cond_tail.rs:225`, and the `.gl` name
  separator in `glalias.rs:120`. The rung will state which of them the new
  production shares and which it deliberately does not.

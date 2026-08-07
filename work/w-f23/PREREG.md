# w-f23 — PREREG

Lane `w-f23`, worktree branch `wt-w-f23` off master **`ceca69b4`**.
Written and committed **before the first probe obj of this lane existed**. The
only bytes read before writing it are (a) already-published lane artifacts
(`docs/rungs/2026-08-08-w-heap.md`, `-w-gen.md`, `work/w-heap/ref/xboxheap/dis.txt`),
(b) `src/xdk/nuispeech/xboxheap.cpp`'s **source**, and (c) one `c2rs capture` of
that TU's `.ex`, which is a read of the workload and not a probe of a hypothesis.

## 0. The target and the flags

`src/xdk/nuispeech/xboxheap.cpp`, priced at **5** by `w-heap` §5. This lane is
briefed to pay **items 1 (F2)** and **2 (F3)**, both in `crates/c2-il`, and to
*assess* items 4 (`alloc`'s mixed-kind refusal, `c2-core`) and 5 (the reference
bind, reader-side).

Every measurement of the target and of any frozen grid cell is at the
**workload's own** `/GR /O1 /Oi /EHsc` (`work/dc3-workload/flags.txt`), never the
harness default `/Ox` — board **#1112**. The generated sweep's own profile is
`/Ox /GS- /c`, a **different population**; when a number comes from the sweep it
is labelled as such.

Two independent instruments per cell, as `w-heap`'s `run.sh` does it: `c2rs
census` (class verdict + first-refusal key) and `c2rs gap` (the whole-TU
differential against real `c2.dll` under wibo, the sole judge), one directory per
cell (board **#1045**).

## 1. What I read F2 and F3 to be — registered before building

Read off the target's own `.ex` (`work/w-f23/il/xboxheap/`), decoded by hand
against the source's eleven lines.

### F2 — a member's ADDRESS as a stored value

The value position of a store statement spelled as **a designator plus an
offset-add run with NO `30` load**, terminated by the ordinary `32 <PTR>` `4B`.
In the target it occurs **exactly once**, at source line 8 (`auto& listHead =
mListHead;`):

```text
  26 11 0a                       destination: the LOCAL reference variable
  b9 0f 0a a6 43 81 20           `this`
  33 86 41 74 08 27 a6 43 98 20  + 8      <- the address, and no `30`
  32 86 43 9b 20 4b              stored, discarded
```

**H-F2.** The reader change is confined to `parse_store_stmt`'s value position:
one more arm beside the `B9` formal, the `33` literal and `parse_load_value`'s
indirect load, producing `IlOp::AddrOf { off }` — the op `BodyShape::AddrLeaf`
already carries. **≤ 80 lines** in `leaf_store.rs`.

### F3 — a call after a store run

A statement-position member call following the run, at source line 11:

```text
  26 fc 09  b9 <this> …  2c … 99 … bd … 80 0f 10 00 00
  b9 <initSize> 86 42 75  55 86 42 75  4c  4b
```

**H-F3.** The accepting production is a **new `BodyShape`**, not a relaxed gate
on `try_parse_store_run` — `w-seam` §6.1 item 2's own wording and `w-heap`'s
prereg P2, scored HIT there. Its gate is `w-heap` §3.2's **syntactic** one: *the
call's argument setup is empty, or writes no register the run reads* — every
actual already occupying the slot it wants.

## 2. Predictions, each scorable

| # | registered |
|---|---|
| **P0** | **THE REGISTERED LOSS.** `w-heap` §3.2 states the F3 gate as syntactic and *"computable from the actual list alone"*. I predict that is **one level too coarse on the reader side**: the reader sees an IL argument *spelling*, not registers, and a member call's slot 0 is not an argument at all — it is a `2C`-converted `this` load inside a `26`/`99`/`BD` receiver group. So "the setup is empty" is a property of the **receiver form**, not of the argument list, and that is where this lane spends. I expect to be wrong about at least one clause of my own F3 gate |
| **P1** | F2 alone converts **nothing** and is not even reachable in the target's shipped spelling: the F2 statement's **destination** is `26 11 0a`, a **local** reference variable, and `parse_store_stmt` requires the base to be a formal. F2 is payable at a store through `this` (the *direct* spelling) and not at the bind |
| **P2** | **The reference bind is SEPARATE, not a corollary of F2/F3.** It is a third reader production — a `.sy` local in the destination *and* in two later statements' **base** position — plus `w-heap` §4.2's base-symbol obligation. I will price it and leave it |
| **P3** | `xboxheap.cpp` **does not convert**. TU match stays **10**. Three refusals remain after mine: #844's composition seam, #868/#836's mixed-kind allocation, #839's bind |
| **P4** | `IlBundle::functions()` **widens** — F2 admits store leaves and runs across the workload that refuse today. I will count the newly-accepted set per `(TU, emit_name)` and prove every one of them still emits `NotImplemented` or byte-exact. A count of 0 would mean the widening is unreachable and is itself a finding |
| **P5** | The emitter **already fails closed** on an `AddrOf` in a store value — `alloc::allocate`'s mixed-kind refusal (#836/#868) plus the store emitters' op-group match. If it does not, that is the lane's headline and the widening does not ship without an explicit additive refusal beside it |
| **P6** | `codegen-gap` **moves off 0** for the first time on the 878-TU scan, because paying a reader refusal is exactly the vocab-gap → codegen-gap transfer. `vocab-gap` 861 falls by the same amount or by less |
| **P7** | The 1,576 `88-store-run-call` cases stay at **0 mismatch** and some of the **1,532 `Port=NotImplemented`** become `Port=Match`. If any becomes `Port=Mismatch`, the lane **stops** — that is not a bug to fix later |

## 3. The decline floor, registered against the incumbent

Today's refusal is **right 100 % of the time on what it refuses**. A reader that
is mostly right is strictly worse. So the widening ships only if **all** of these
hold, and otherwise the lane declines and publishes the residue with a count:

* `mismatch` **0** on the 878-TU scan, on the 1,576 new sweep cases, and
  everywhere in `scripts/gate.sh`.
* `fnbyte-exact` does not shrink below **36,209**; `fnbyte-differs` does not grow
  above **2,111**; `fnbyte-reloc-differs` stays **861**;
  `fnbyte-match-tu-differs` and `-match-tu-reloc-differs` stay **0**;
  `fnbyte-partial` stays **0**.
* `sweep graded` does not fall below **18,190**; `sweep ungraded` does not rise
  above **96**; `cross ungraded` does not rise above **388**.
* Every newly-accepted `(TU, emit_name)` is enumerated and each is shown to emit
  `NotImplemented` or byte-exact. An un-enumerated widening is a decline.
* Every accepted shape is proven by a **frozen** cell — grid manifest committed
  before the first cell is compiled, on structural axes, reusing
  `88-store-run-call.py`'s axis vocabulary rather than inventing a second one.

## 4. What this lane will NOT do

* It will not fit an allocation rule to `xboxheap`. `w-heap` §4.1.1 refuted
  clause 1 on `j1_lit2`, and six keys are already on record as refuted for
  exactly this reason. The target's agreement with clause 1 is a coincidence of
  its use counts.
* It will not chase `expr-op-0x27`. `w-heap` records it as a **fall-through**
  here, not a construct, and #622/#662/#970 are the same dead end.
* It will not start at `x6` (board #1130) — the wrong regime.
* It will not touch `crates/c2-core/coff/`'s alignment promotion or the `.gl`
  alignment reader: concurrent lane **w-align16** owns those. This lane owns the
  `.ex` body reader path in `crates/c2-il`, and will `grep` that crate for an
  existing reader over the same fact before adding one.

# w-drop3 — PREREG

    Lane:      w-drop3
    Branch:    w-drop3, worktree off master `71e38a2`
    Target:    board #979 — the 140-body cluster `seq | ref-longer | del-only`,
               the ONLY cluster where the port emits *less* than c2
    Written:   2026-08-06, BEFORE any probe, any capture, any scan, any
               `cl.exe` invocation on this branch. Committed as the first
               commit of the lane.
    Rows:      #984–#989 reserved

Everything below is registered in advance. Sections marked **REGISTERED** are
scored in the rung whether they hit or miss; a loss is the deliverable, not an
embarrassment (w-seq §7's P1/P4 are the precedent).

---

## 0. What is known before the lane starts, and from where

Taken from `docs/DIFF_STRUCTURE.md` §3.2 (board **#979**, lane `w-bytes`) and
`docs/rungs/2026-08-08-w-seq.md` §5. Nothing here is measured by this lane yet:

* 140 bodies, 101 TUs, all one template `??$Obj@V…@@DataArray@@QBAPAV…@@H@Z`.
* Port 13 words, c2 20 words, c2 relocations **11**. The port's body is a
  **strict subsequence**: 8 equal words, 7 deletions at word index 8, then a
  5-word shared epilogue. `del-only` — the port substitutes nothing.
* The 7 missing words, byte-identical across all 140:
  `lis r11,0 / lis r10,0 / addi r5,r11,0 / addi r6,r10,0 / li r4,0 / li r7,0 / bl`.

## 1. The hypothesis — REGISTERED

### 1.1 H-CALLEE: the third call is `__RTDynamicCast`

**The missing call is the `dynamic_cast` runtime helper**, not an assert and not
a `__FILE__` string pair.

The reasoning is the argument register profile, read off the seven words before
any IL is opened. MSVC's helper has the signature

```c
void* __RTDynamicCast(void* pv, LONG VfDelta, void* SrcType, void* TargetType, BOOL isReference);
```

which lands `r3 = pv`, `r4 = VfDelta`, `r5 = SrcType`, `r6 = TargetType`,
`r7 = isReference`. The seven words set **r4 = 0**, **r7 = 0**, and **r5 / r6
from two relocated address halves**, and leave **r3 alone** — r3 is already the
result of the second `bl`. That is the helper's ABI slot for slot, including
which two of the five arguments are the relocated ones. `docs/DIFF_STRUCTURE.md`
§3.2 guesses "a `.rdata` string pair, by shape"; this prereg registers the
**RTTI type-descriptor pair** (`??_R0?AV<T>@@@8`) instead, and that disagreement
is the first thing the IL will settle.

Corroboration available before probing: `w-seq` §5's production table lists
**`expr-intrinsic-dynamic-cast` at exactly 140 differs**. Two independent
instruments landing on 140 is either the same population or a coincidence worth
naming. **REGISTERED as a prediction, not as a fact** — w-seq's 140 counts
differs naming a parse-refused *callee* under that key, which is not
definitionally the same set as w-bytes' 140-body cluster, and P4's precedent in
w-seq §7.1 is exactly two numbers agreeing that were not the same number.

**Falsifier.** The IL for one instance shows the third call's operands are a
string literal pair, an assert helper, a `__FILE__`/`__LINE__` pair, or any
callee that is not the dynamic-cast helper.

### 1.2 H-SKIP: the call is dropped by the seq loop terminating early

`crates/c2-il/src/func/body/shapes/calls.rs::parse_call_sequence_from` is a loop
that, on each iteration, tries in order:

1. `eat_return_plumbing(…, false, …)` → break with `SeqTail::Void`;
   1b. the `3A <label> 3A <same label>` fallthrough → break with `SeqTail::Void`;
2. `33 <int-like> k` → break with `SeqTail::Lit(k)`;
3. otherwise `eat_call_head` + `eat_call_args`, and push another call.

The port's emitted body is 5 prologue + 2 calls + 5 epilogue = **13 words with
no `li r3,k`**, so the port reached **`SeqTail::Void` after exactly two calls**.

**H-SKIP: arm (1) or (1b) succeeds at the byte position where the third call's
`dynamic_cast` expression begins**, so the loop breaks and the remaining element
of the body is never read. Nothing refuses; the residue of the segment is
discarded silently.

**This is THE bug if it holds**, and it is a class and not an instance: a
production that *probes* for a terminator and breaks on success will drop
whatever it failed to recognise, every time, with no counter and no refusal.
`docs/STATUS.md` trap 5 — absence reads as success unless something forbids it.

**Rivals, registered so the lane cannot claim H-SKIP by default:**

| | rival | what would show it |
|---|---|---|
| R1 | a **call-count cap** — `MAX_SEQ_CALLS` | the refusal key `callseq-too-long` appears; but a cap *refuses*, it does not skip, so this predicts a `refused` verdict and not a `differs` — it is already refuted by the cluster existing at all |
| R2 | **intrinsic misclassification** — the cast is read as an intrinsic that lowers to nothing, and the two `bl`s are the whole call list by construction | the third element parses into some `IlOp`/shape that the emitter then drops on the floor, rather than not being parsed at all |
| R3 | the value-call arm (3) **is** taken for the cast and `eat_return_plumbing` at the end consumes it | the port would emit a third `bl` — refuted by the bytes, registered for completeness |
| R4 | the third call is not in this function's segment at all (**#644**: a producer is not one contiguous field) — the port reads the whole segment correctly and c2 sources the cast from elsewhere | the segment ends after call 2 |

### 1.3 What the IL evidence must be

Confirmation of H-SKIP requires **all three**, and any one missing is a refutation:

1. The `.ex` segment for one instance contains a third callee/intrinsic element
   after the second call's `4B`.
2. `parse_call_sequence_from` breaks out of the loop with `*p` **strictly less
   than** the end of that element — i.e. bytes remain unread.
3. The arm it breaks on is (1) or (1b), and the bytes it consumed as "return
   plumbing" overlap the cast expression.

Item 2 is the one that makes "skip" a measurement rather than a story, and this
lane will print the byte offset and the residue length.

## 2. The predicted outcome — REGISTERED, as a fork with a prior

The brief's headline prediction is `140 → exact`, differs **3,195 → 3,055**,
exact **35,982 → 36,122**. This prereg **declines to register that as the sole
outcome** and registers a two-branch fork with a prior, because the cost of the
missing call is knowable in advance and it is not small.

| | landing | FBM prediction | prior |
|---|---|---|---:|
| **A — FIX** | the 7 words are emitted byte-exactly | differs 3,195 → **3,055**, exact 35,982 → **36,122**, refused **unchanged** | **0.30** |
| **B — HONEST REFUSAL** | the function refuses instead of emitting a wrong subset | differs 3,195 → **3,055**, exact **unchanged at 35,982**, refused 130,573 → **130,713** | **0.70** |

**Why B is favoured, stated before the probe.** Landing A must emit two RTTI
type-descriptor address halves. That needs (i) the `??_R0?AV…@@@8` type
descriptors as symbols, (ii) `REFHI`/`PAIR`/`REFLO`/`PAIR` relocation quads —
`3 REL24 + 2 × 4 = 11`, which is *exactly* the reference's 11 and is registered
here as an arithmetic prediction that the lane will check, (iii) an external
`__RTDynamicCast`, and (iv) whatever section the descriptors live in.
`docs/STATUS.md` records **`.rdata$r` declined twice** — `w-rdata` priced it at
seven independent refusals and `w-rtti` was briefed to ship it anyway and found
all seven still unpaid. A lane that guesses any of the four is a **wrong emit**,
which the correctness rule calls the worst state.

**Either landing is a win on the same axis**: `differs` falls by 140 and the
port stops emitting 140 bodies the judge calls wrong. Only A moves `exact`.

### 2.1 The falsifier for the whole lane

If `differs` does **not** fall by 140 — or falls by more, or `exact` shrinks by
any amount, or any FBM bucket moves that is not named in the row above — the
lane's model is wrong and the change does not land. `exact` shrinking by even 1
is an unconditional stop.

## 3. Controls — REGISTERED, each with its known answer

| # | control | known answer |
|---|---|---|
| C1 | scan `mismatch` | **0** at both ends |
| C2 | `fnbyte-match-tu-differs` | **0** at both ends |
| C3 | the FBM partition sums to 178,975 | holds at both ends; `fnbyte-partition-broken` **0** |
| C4 | TU match | **10 → 10** predicted (trap 8 / the `w-empty` precedent: 1,516 bodies closed moved TU match 10 → 10). Any move is a *finding*, not a target |
| C5 | census / gate disagreement | **0** at both ends |
| C6 | the 140 are named **by symbol**, not subtracted from totals | the before-set and the after-set are dumped and `comm`-ed, per `w-empty`'s rule that "zero moved the other way" is checked per symbol |
| C7 | `fnbyte-elided` / `-elided-exact` | **1,516 / 1,516**, unchanged — this lane must not perturb mechanism E |
| C8 | factors A/B/C/D/E, `B∧C`, `A∧B∧C`, FRONTIER | every digit unchanged |
| C9 | `cargo test --workspace --release` | baseline **978 passed / 30 targets** plus this lane's additions; a *shrunken* target count means an earlier target failed |
| C10 | `scripts/gate.sh --jobs 6` | **18/18 PASS, 0 mismatch** |

## 4. The structural requirement — REGISTERED as a deliverable in BOTH landings

Whatever the landing, **a silently skipped element must become structurally
impossible in this production.** A refusal must refuse the *function*; it must
never skip an *element*. Concretely, and registered so the lane cannot land
without it:

* the seq loop must not be able to exit with unconsumed body bytes;
* there is a **test** asserting that a segment with a trailing element the loop
  cannot read comes back `Err` and not `Ok` with a short call list;
* the test asserts a *refusal*, not a byte pattern, so it survives the eventual
  fix.

This is registered as **mandatory**, i.e. a lane that closes the 140 by
special-casing them and leaves the skip reachable has **failed** even if every
number in §2 hits.

## 5. What this lane may NOT do — REGISTERED as decline clauses

1. **Never key on the template name.** `??$Obj@V…@@DataArray@@` may appear in no
   accept path, no refusal key, and no test assertion. The fix is grounded in
   the IL grammar or it does not land.
2. **`IlBundle::functions()` must not widen in the same commit** (#878's loaded
   gun, restated by w-seq §5.2.4): every one of the 3,195 is already accepted by
   the per-function gate, so a TU-level widening that admits a TU carrying one
   ships a wrong obj.
3. **No scoring on `fnbyte-differs` falling alone.** w-seq §5.2.1 is explicit:
   a change graded on the net would ship wrong bodies. The per-symbol set
   (C6) is the grading instrument.
4. **No fixture is asserted to reproduce the shape until it is compiled.** If no
   hand fixture can reproduce the three-call seq (the parse may refuse it
   first), the lane asserts the *refusal* — `w-fix`'s precedent — and rests on
   the workload's 140 witnesses, and says so.

## 6. Prediction I most expect to lose

**That the 7 missing words are a `.rdata` string pair.** `DIFF_STRUCTURE.md`
§3.2 says so and this prereg contradicts it (§1.1). If the IL shows a string
pair, §1.1 is a loss and the published §3.2 stands.

Second most likely loss: **the 140 ≠ w-seq's 140.** Registered in §1.1 with
w-seq §7.1's precedent attached.

# Lane w-pair — findings. **Zero TUs converted.** The decline clause fired.

Prefixed `_`: this claims no completed rung. Base `wt-w-pair` from master
`3457c9f`. Pre-registration: [`_2026-08-04-w-pair-prereg.md`](_2026-08-04-w-pair-prereg.md).

**TU match is 8/878, exactly where it started.** Everything below is the
characterized boundary the prereg priced as the alternative deliverable.

---

## 0. Incumbents — measured at end of lane, not assumed

No crate under `crates/` was modified. Measured anyway, because a status is not
a count (`STATUS.md` trap 5):

| incumbent | recorded | measured here |
|---|---|---|
| `cargo test --workspace --release` | 677 passed, 0 failed, 25 targets | **677 / 0 / 25** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2,592 verdicts, 0 mismatch | **12/12 PASS, 2,592, 0** |
| `c2rs selftest` | 216 PASS, 0 FAIL | **216 / 0** |
| `c2rs gap` | match 8, mismatch 0, codegen-gap 0, vocab-gap 863, capture-fail 7 | **8 / 0 / 0 / 863 / 7** |
| factors A/B/C/D, FRONTIER | 28 / 338 / 114 / 8, 17 | **28 / 338 / 114 / 8, 17** |
| `cargo build` warnings | 0 | **0** |

P5 holds in every cell.

## 1. The frontier, re-derived rather than transcribed

`c2rs gap` reproduced in this worktree, then `c2rs census` run on **each** of
the 17 frontier TUs at the workload's flags. Exactly two are straight-line only:

| TU | fns | blocked | first blocker | cflow | EH |
|---|---:|---:|---|---|---|
| `src/Main.cpp` | 1 | 1 | `param-width-undetermined:mid` | straight | **`eh-state1`** |
| `src/xdk/nuispeech/xboxheap.cpp` | 1 | 1 | `expr-op-0x27` | straight | `eh-none` |
| `src/xdk/nuispeech/mmio.cpp` | 11 | **3** | `expr-cmp-eq` ×3 | if-2, if-n ×2 | none |
| `src/system/utl/EncryptXTEA.cpp` | 5 | 4 | memcpy, `0x27` ×2, `load-type-8882` | 2 straight, 2 loop | none |
| `src/system/math/Sort.cpp` | 1 | 1 | `assign-store-type-0x86` | loop | none |
| `src/xdk/LIBCMT/osfinfo.cpp` | 1 | 1 | `expr-cmp-ge` | if-n | none |
| `src/xdk/LIBCMT/undname.cpp` | 1 | 1 | `expr-cmp-ne` | if-n | none |
| `src/xdk/LIBCMT/vswprnc.cpp` | 1 | 1 | `expr-cmp-eq` | if-n | none |
| `src/xdk/xjson/jsonwriter.cpp` | 1 | 1 | `expr-brfalse` | loop | none |
| `src/xdk/xlrc/xlrcimpl.cpp` | 1 | 1 | `assign-rhs-call-0x26` | if-n | none |
| `src/system/negate_test.cpp` | 2 | 2 | `assign-store-type-0x86` ×2 | if-n ×2 | none |
| `src/system/synth_xbox/Biquad.cpp` | 2 | 2 | `expr-cmp-eq`, `…recv-load-then-plumbing-0x3A` | **`cf-expr-0x05`**, straight | none |
| `src/xdk/LIBCMT/vsnprnc.cpp` | 2 | 2 | `expr-cmp-eq`, `call-arg-lit-permuted:mid` | if-n, straight | none |
| `src/system/rndobj/wordwrap.cpp` | 3 | 3 | `expr-jump`, `expr-bit-and`, `expr-cmp-eq` | straight, if-n, **`cf-expr-0x05`** | none |
| `src/system/utl/Pool.cpp` | 3 | 3 | `0x27` ×2, `expr-brtrue` | **`cf-expr-0x05`**, if-1 ×2 | none |
| `src/system/synth_xbox/IPP_basicmath_xbox.cpp` | 4 | 4 | `expr-cmp-eq` ×4 | loop ×4 | none |
| `src/xdk/nuispeech/xboxmem.cpp` | 4 | 4 | `expr-cmp-ne`, `expr-cmp-eq` ×3 | straight, if-1 ×3 | none |

Two things worth carrying forward:

* **`mmio.cpp` is the closest TU on the frontier that is not straight-line: 8 of
  its 11 functions are already in class**, and all three blocked ones want
  `expr-cmp-eq` under `if-2`/`if-n`. It is the natural first target for the CFG
  step, and it is a better one than `Pool.cpp` (whose `if-1` bodies emit no
  branch and whose third function is `cf-expr-0x05`).
* **`cf-expr-0x05` converts zero TUs alone.** All three TUs carrying it
  (`Biquad.cpp`, `wordwrap.cpp`, `Pool.cpp`) also carry an `if-n`/`if-1`
  function, so the DIV-width refusal is never the last thing in the way. This is
  board #150's shape again: rank by TUs, not by rows.

## 2. `Main.cpp` — declined, and it is not a shape widening

Its emitted function is `main` (the census names the callee — `STATUS.md` trap
6). The `/FAsc` listing at the workload's flags shows what it actually needs:

* two `.pdata` COMDATs in one obj (`$T2596` for `main`, `$T2599` for the
  funclet), with the frame words `0c0001305H` and `040000a04H`;
* a `.rdata` group holding `__unwindtable$main`, `__ehfuncinfo$main`
  (`019930522H`, state table, `$T2592` IP-to-state map) and the `$M2590`/
  `$M2591` label pair that map it;
* the two-word EH header **inside `.text` before the prologue** — `DCD
  __CxxFrameHandler` / `DCD __ehfuncinfo$main`, i.e. the function's own bytes
  begin with two relocations;
* a separate `__unwind$2585` funclet with its own prologue, epilogue and frame;
* a stack-homed local object (`app$ = 80`), and three framed `bl`s.

That is the EH critical path, and it blocks by factor D over ~740 objs. It was
not attempted (prereg P6).

## 3. `xboxheap.cpp` — declined. The body is SCHEDULED.

The blocker chain, measured rather than predicted. Under
`C2RS_SINK_OFF_ADD_ARG=expr` the blocker moves `expr-op-0x27` →
**`expr-op-0x32`**, so the row was never one change away. Hand-decoding the
404-byte `.ex` segment gives eight statements, and the real `/FAsc` listing
gives the emission:

```text
   source (in order)                    emitted
   ----------------------------------   -------------------------------
   1  mSize      = size    → 0x10        0  li   r10,0
   2  mFreeHead  = this    → 0x00        1  stw  r5,10h(r3)      stmt 1
   3  mCount     = 0       → 0x14        2  addi r11,r3,8
   4  mUsedHead  = this    → 0x04        3  stw  r3,0(r3)        stmt 2
   5  auto& l = mListHead                4  stw  r10,14h(r3)     stmt 3
   6  l.mNext    = &l      → 0x08        5  mr   r31,r3
   7  l.mPrev    = &l      → 0x0C        6  stw  r3,4(r3)        stmt 4
   8  AllocatePageBlock(initSize)        7  stw  r11,8(r3)       stmt 6
                                         8  stw  r11,0Ch(r3)     stmt 7
                                         9  bl   ?AllocatePageBlock@…
                                        10  mr   r3,r31
```

Six productions are missing, five of them ordinary rungs and one not:

1. a `27` designator in a store statement (the designator layer already has it —
   the `expr-op-0x27` key is a *fall-through* from the generic expression parse
   after `try_parse_store_run` declined, not the store path's own refusal);
2. a store run **mixing** literal and formal values — `try_parse_store_run`
   refuses this today and its refusal is correct;
3. the `26` local-reference bind statement (`auto& l = mListHead`), whose whole
   emission is the `addi r11,r3,8` at slot 2;
4. a trailing framed call in a body that also stores — `StoreRun`'s tail admits
   only return plumbing or `return *this`, and `CallSeq` admits only calls;
5. the constructor's `return this` out of `r31` (`mr r31,r3` / `mr r3,r31`);
6. **the schedule.** The six stores come back in source order, but the three
   value-producing instructions are interleaved at slots 0, 2 and 5. A
   source-order emitter diverges at the **first instruction**.

## 4. The probe grid — six candidate placement rules, each refuted

`scripts/gt_store_sched.sh` (new, committed) reproduces every cell in one
compile at the workload's flags. `leaf_store.rs` already records four *register
allocation* rules fitted to this family and each refuted by another cell; this
grid adds the **placement** axis and the same thing happens.

The measure is the **gap**: slots between a value-producing instruction and the
first store that consumes it.

| cell | producer | gap | stores in source order? |
|---|---|---:|---|
| **C0 control** | none | — | **yes**, three stores, no setup — control passes |
| C1 `a=u; b=0` | `li r11,0` | 2 (forced) | yes |
| C2f `a=u; b=0; c=v` | `li` | 3 | **no** — 0, 8, 4 |
| D1 `a=0; b=u` | `li` | 2 (forced) | **no** — 4, 0 |
| D2 / D3 / D8 (literal first, 3/4/5 stores) | `li` | 3 | **no** |
| D7 `a=u; b=0; c=v; d=w` | `li` | 3 | **no** — 0, 8, 4, C |
| C7 (six formals then a literal) | `li` | 7 | yes |
| C8 (literal then six formals) | `li` | 3 | **no** |
| D6 `a=u+1; …` | `addi r11,r4,1` | 3 | **no** |
| E5 `a=1; b=2; c=u; d=v` | `li r11,1`, `li r10,2` | 4, 3 | **no** — 8, C, 0, 4 |
| C5 / D5 / E3 / E1 | `addi r11,r3,8` | **1** | yes |
| **E2** | `addi r11,r4,8` | **3** | **no** |
| **F1 / F2** (controlled swap) | `addi r11,r3,8` / `addi r11,r4,8` | **1** / **1** | yes |
| C3 / C9 (xboxheap minus the frame) | `li`+`addi` / `addi` | 4,4 / 5 | yes |

**P1 is REFUTED.** The stores of a mixed run are **not** emitted in source
order: seven cells reorder them. `leaf_store.rs` records the two-statement case;
D3 and D8 show it is not a swap-to-the-end either — the literal store lands at
slot 3 with further stores after it.

**P2 holds.** C7 hoists a producer seven slots above its consumer.

**P3 holds — and it holds against three more rules than it registered:**

| rule | reproduces | refuted by |
|---|---|---|
| R1 *(registered)* all producers first, first-use order | many | **C3** (a store sits between `li` and `addi`), **E5** (a store sits between the two `li`s) |
| R2 *(registered)* producer immediately before its consumer | C5 only | C1, C7, D2, D3 — hoists of 2 to 7 slots |
| R3 *(registered)* producers and stores alternate | — | **C0** (no producers), **C7** (six consecutive stores) |
| H3 *(found, not registered — declared)* consumer delayed to ≥3 slots, others source order | **14 of 23 cells** | **D5**, **E1** — gap 1 with two provably-disjoint stores available to pull ahead |
| H4 producer base register == consumer store base ⇒ no delay | most | **E1** — producer reads r3, consumer stores to r4, gap 1 |
| H5 producer reads **r3** ⇒ no delay | all but one | **F2** — the controlled swap. Producer reads r4 exactly as E2's does, and is **not** delayed |

**F1/F2 is the cell that ends the exercise.** It is the same statement structure
with the two pointer parameters exchanged, so the only thing that varies is
which architectural register each role occupies — and both emit gap 1. Every
surviving fit at that point had to be stated in terms of a specific register
number, which is the signature of fitting a machine scheduler rather than
recovering a lowering rule.

The honest claim is bounded: **this grid does not prove no rule exists.** It
proves that six successive candidate rules are each refuted by a cell of a
23-cell grid, that the last survivors are register-number-superstitious, and
that a fifth and sixth fitted rule now join the four `leaf_store.rs` already
records. `GAPS.md` §6 instance #10 — measure at the edge, do not fit the
scheduler — is why this is where the lane stops rather than where it starts
guessing.

## 5. What this means for the payoff metric

`xboxheap.cpp` is not "the single cheapest conversion in the project". It is
five ordinary rungs **plus an instruction scheduler**, and the scheduler gates
the other five: with all of items 1–5 built and the emission in source order,
the obj diverges at instruction 0. **Both** straight-line-only frontier TUs are
therefore behind a whole subsystem — EH for `Main.cpp`, scheduling for
`xboxheap.cpp` — and neither is a shape widening.

The implication for planning is the uncomfortable one: **the pre-Phase-7
frontier of 17 has no cheap member.** Fifteen sit behind the CFG step; the two
that do not sit behind something larger. A lane briefed to "take the cheapest
frontier TU" will keep finding this, because the frontier was ranked by *blocked
function count* and that quantity does not correlate with the work.

## 6. Proposed board rows (this lane does not edit `BOARD.md`)

* **`xboxheap.cpp` is scheduler-blocked, not widening-blocked.** DECLINED with
  the byte evidence in §3 and the grid in §4. Five refuted placement rules
  recorded so a sixth is not fitted. Refutation condition: any rule that
  reproduces all 23 cells of `scripts/gt_store_sched.sh`.
* **`Main.cpp` is EH-blocked.** DECLINED; §2 enumerates the two `.pdata`
  COMDATs, the `.rdata` unwind group, the in-`.text` EH header and the funclet.
* **The mixed-value store run reorders the STORES, not only the registers.**
  Extends the four allocation rules in `leaf_store.rs` with a placement axis;
  `try_parse_store_run`'s literal gate is confirmed necessary and is *not*
  merely conservative.
* **`mmio.cpp` is the frontier's best CFG target** (8/11 already in class, three
  `expr-cmp-eq` bodies under `if-2`/`if-n`), and `Pool.cpp` is its worst.
* **`cf-expr-0x05` converts 0 TUs alone** — all three carriers also hold an
  `if-1`/`if-n` function. Board #150's shape, third instance.

## 7. What was deliberately not done

* No CFG or block IR (lane w-cfg owns the spec; pre-empting it would be a second
  w-front).
* No codegen landed, and the gate was not widened by one byte. `PortC2` refuses
  exactly what it refused on `3457c9f`.
* `Pool.cpp` was not graded on.
* No seventh placement rule was fitted to the grid.

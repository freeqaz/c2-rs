# Prereg — lane w-cfgimpl: the CFG step, implemented and graded by the differential

    Tag:       W-CFGIMPL-PREREG
    Slug:      w-cfgimpl-prereg
    Date:      2026-08-04
    Lane:      w-cfgimpl (`wt-w-cfgimpl`, base master `4bc49a7`)
    Status:    FROZEN before the first line of `crates/` was edited and before
               any port output was compared against any reference obj.
    Spec:      `docs/CFG_SHAPE.md` (lane w-cfg, read-only). This lane is the
               first to run code against that document; §8.4 says it "should
               expect to find at least one thing wrong".

---

## 0. What was already known at freeze time, and what was not

Honesty about the instrument first, because a prediction made after the answer
is not a prediction.

**Already measured before this file was written** (recon, no `crates/` edit):

* the four `.text` COMDATs of `src/xdk/nuispeech/xboxmem.cpp` at the workload's
  own flags, disassembled — they reproduce `CFG_SHAPE.md` §4.1/§4.3 exactly,
  including `?MemFree`'s 36 bytes verbatim;
* the three blocked `.text` COMDATs of `src/xdk/nuispeech/mmio.cpp`, likewise;
* all four `.ex` function segments of `xboxmem.cpp`, decoded by hand;
* the incumbent `cargo test --workspace --release`: **677 passed, 0 failed, 25
  targets**, re-run on this branch at `4bc49a7`.

**Not measured, and what every prediction below is about:** any byte the *port*
emits. `PortC2` has never emitted a conditional branch of any kind. Nothing
below is scored against a reference obj that has already been compared.

Registered bias, in advance: *I want the answer to be — the two-arm conditional
tail call is a small paired widening, `?MemFree` and `?MemSize` come out
byte-exact on the first differential, and `xboxmem.cpp` converts as a TU.* The
third clause is the one I expect to be wrong, and §4 prices that.

---

## 1. The class this lane proposes to admit

Named **`cond-tail-pair`**, and stated as a *shape*, per `CFG_SHAPE.md` §6.2
item G — the port must be able to say "this `cflow-if-1` is fold band 3" and
refuse otherwise, rather than emitting a branch and being wrong on six of seven
leaf bodies (§3.5).

```
<ret> f(p0, …, pn) {
    if (p_i <rel> <lit>) { <tail call A>; }     // arm ends in a transfer
    <tail call B>;                              // that is NOT the epilogue
}
```

both arms being an existing accepted tail-call argument list (`SlotArg::Formal`
/ `SlotArg::Lit`), and **neither arm reaching the epilogue**. That last clause
is what puts the shape inside band 3 by construction rather than by a cost
model: `CFG_SHAPE.md` §3.5 band 2 is reached "when one successor **is** the
function's epilogue", and band 1 needs both arms to be constants. A body with
two distinct tail calls can be neither.

---

## 2. Predictions — the emission half

Each carries a **named rival**: a different reading that is not merely "the
prediction is false".

| # | prediction | rival if it fails |
|---|---|---|
| **A1** | The port emits `?MemFree@NUISPEECH@@YAXPAX0K@Z`'s 36 bytes **exactly** as `CFG_SHAPE.md` §4.1 transcribes them: `7c8b2378 2b030000 409a0010 7ca42b78 7d635b78 4bffffec 7d655b78 38800000 4bffffe0`, with two `REL24` at 0x14 and 0x20 and none on the `bc`. **This is the known-answer control**: the target string was published by another lane before this lane existed. | R-A1: the `bc` also takes a relocation, or the intra-function displacement is section-start-relative like the external `b` (`CFG_SHAPE.md` §3.3's two-encodings hazard is real and I get it backwards). |
| **A2** | The entry block's shuffles follow this rule, which reproduces all three of `MemAlloc`/`MemFree`/`MemSize`: **(a)** a formal both arms need at *different* destination registers is parked in **r11** in the entry block; **(b)** a formal both arms need at the *same* destination register has its move hoisted into the entry block; **(c)** everything else stays in its arm; **(d)** within any block, moves are emitted in **descending destination register** order (the incumbent `moves_descending` rule, unchanged). | R-A2: the entry block is not a hoist at all but a fixed "save every clobbered live-in to r11 descending" prologue, and (b) is a coincidence of `MemAlloc` having exactly one same-destination formal. |
| **A3** | The compare reads the value at its **post-hoist** location, not its home register — i.e. `?mmioGetInfo`'s `mr r11,r3 ; cmplwi cr6,r11,0` is the rule and `?MemFree`'s `cmplwi cr6,r3,0` is the same rule with an untouched r3. Entry moves are emitted **before** the compare. | R-A3: the compare is always emitted first, against the home register, and `mmioGetInfo` differs for a reason other than the hoist. |
| **A4** | `BI` is `4*6 + bit` here — cr6 — because the producer is an explicit `cmplwi`, and `CFG_SHAPE.md` §3.2's cr0 rows require a record-form producer this class cannot contain. The port will hard-code **nothing**: `BI` is computed from a condition-register field carried in the IR. | R-A4: even a plain compare feeding a forward branch sometimes lands in cr0, and cr6 is a property of the *fold band* rather than of the producer. |
| **A5** | Byte-exactness needs **no** displacement-range check to be exercised (every branch in class is under ±32764), but the check will be **written anyway** per §6.2 item D, and a unit test will drive it. | — (not a rival; a scope statement, scored as done/not-done) |

## 3. Predictions — the class, the count, and what does not move

| # | prediction | rival if it fails |
|---|---|---|
| **B1** | **`?MemFree` and `?MemSize` come out byte-exact.** They are the same 36-byte payload up to two relocation targets (§4.3). | R-B1: `?MemSize`'s `2c … 41` result-convert tail changes the arm's emission and only `?MemFree` lands. |
| **B2** | **`xboxmem.cpp` does NOT convert as a TU on the CFG step alone**, and `CFG_SHAPE.md` §4.3 is right that it cannot. Its other two functions each need a second, non-CFG widening: `?GetXAllocAttributes` needs the `!=0` → `addic`/`subfe` bool spine folded with `<<30 \| 0x249b0000` into `lis`+`rlwimi`, and `?MemAlloc` needs an **assignment to a local inside an arm** plus `(x>>27)&8` folded into one `rlwinm r4,r4,5,28,28`. | R-B2: one or both of those is already in the port's expression vocabulary and only the control flow was in the way, so the TU converts on the CFG step alone. |
| **B3** | **`mmio.cpp` does NOT convert**, and is a worse target than `xboxmem.cpp` despite being named the frontier's best CFG target. Its three blocked functions are all **framed** (`.pdata` COMDAT + a `$M` label pair each) and need, on top of the CFG: an inlined-intrinsic `memcpy` call, a member load/store at a byte offset, `cmplw` register-vs-register, a `b` to a materialized epilogue block, and — in `mmioClose` — an **indirect call through a loaded function pointer** (`mtctr`/`bcctrl`), plus two branches on **cr0** from a `cmplwi cr0` the port has never emitted. | R-B3: frames are already modeled (`framed_call`), so the marginal cost is only the branch plus `memcpy`, and `mmio.cpp` is the cheaper of the two after all. |
| **B4** | **TU match ends at 8 or higher; it never goes down.** `mismatch` stays **0** on every lane of `scripts/gate.sh` and on the 878-TU scan. | R-B4: the new class over-accepts a body outside band 3 and the scan records a live `mismatch` — the outcome `CFG_SHAPE.md` §3.5 exists to prevent. |
| **B5** | **`src/system/utl/Pool.cpp` is still refused**, function-for-function, after the widening. Its two `cflow-if-1` functions are band-2 `bclr` folds and its constructor is `cf-expr-0x05`. A widening that makes any of them "in class" has over-accepted. **Known-answer control #2.** | R-B5: the band-3 gate is expressible only as "two tail calls", which `Pool.cpp` does not have, so this control is vacuous rather than informative. |
| **B6** | The whole-workload **census does not move by more than a few hundred functions**, and the *per-function census is not this lane's metric*. `docs/BOARD.md` #150's trap: bucket size and conversion yield are different questions. | — (scope statement) |

## 4. Priced decline clause

Stated before the first edit, so that stopping is a decision and not a
retreat.

* **If the `cond-tail-pair` emission is not byte-exact on `?MemFree` after two
  rounds of correction**, this lane stops widening and ships the *measured
  divergence* — the port's bytes beside c2's, word for word, with the
  emitter decision from `CFG_SHAPE.md` §4.2 that each divergent word belongs to.
  That is a direct falsification of a published spec and is worth more than a
  third guess at it.
* **If `cond-tail-pair` lands byte-exact but neither `?GetXAllocAttributes` nor
  `?MemAlloc` is inside a day's reach**, this lane **converts zero TUs**, says
  so in the first line of its rung, and ships: the CFG production (block IR,
  label fixups, both branch encodings, the range check), the two byte-exact
  frontier functions, the fixtures, and a characterization of exactly what the
  remaining two functions need. Per the brief: *a declined conversion with a
  characterized boundary is a real deliverable*.
* **If any gate regresses** — a `FAILED` count above 0, a `mismatch` above 0,
  TU match below 8 — the widening is reverted in the same session and the
  revert is committed with its reasoning.
* **Loops and `switch` are out of scope and will not be started**, whatever the
  budget. `CFG_SHAPE.md` §3.7c's CTR loops are an instruction family absent from
  the port and from `docs/`; `switch` unblocks 0 frontier TUs.

## 5. Incumbents, frozen

| gate | incumbent |
|---|---|
| `cargo test --workspace --release` | **677 passed, 0 failed, 25 targets** (re-run on this branch) |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2,592 verdicts, 0 mismatch |
| `c2rs selftest` | 216 PASS, 0 FAIL |
| 878-TU scan | match **8**, mismatch **0**, codegen-gap **0**, vocab-gap **863**, capture-fail **7** |
| census | 706402 / 2463318 |
| emitted census | 38457 / 178972 |
| census/gate disagreement | 0 |
| FRONTIER | 17 |
| A/B/C/D | 28 / 338 / 114 / 8 |
| `cargo build` warnings | 0 |

**Read the FAILED count, never the passed count** — a failing target aborts the
run, so a low passed-count is not a regression of that size. **Perf geomean is
not a signal**: it wobbles 586–689× across runs of a byte-identical binary.

## 6. What a count here means

Every count in this lane's rung is evidence about the predicate that produced
it. The class predicate is stated in §1 and any change to it invalidates the
counts taken before the change; a rung that widens the predicate mid-run will
say so and re-take every number.

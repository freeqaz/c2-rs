# w-tu2 PREREG — `mmio.cpp` as the standing test of board #465

    Lane:    w-tu2 (`wt-w-tu2`), branched at master `3158f1e`
    Date:    2026-08-05
    Target:  `src/xdk/nuispeech/mmio.cpp` — w-tu1's pre-registered standing test
    Board:   taking #480-#489

**Committed before any mechanism exists and before the surcharge grid is run.**
The baseline reproduction (§1) and the obj-shape survey (§2) are orientation and
were done first, on purpose: w-tu1's actual selection criterion was *dump the
reference obj before you plan*, and my brief makes that a precondition of
planning rather than a result.

---

## 1. Baseline — reproduced, every digit

`c2rs gap` at the workload's own `/O1 /Oi /EHsc /GR`, from this worktree:

    match 9 · mismatch 0 · codegen-gap 0 · vocab-gap 862 · capture-fail 7
    A 28 (LO 27) · B 338 · C 169 · D 9 · E 2
    B∧C 151 · A∧B∧C 27 · A∧B∧C∧D 7 · A∧B∧C∧(D∨E) 9
    FRONTIER 18 · frontier-if-A 140 · emit-predicate-worth 124

**R1 — HIT.** Identical to the brief's stated baseline in every field.

---

## 2. The obj-shape survey, and the correction it forces on w-tu1's criterion

All 18 frontier reference objs compiled at the workload profile (18 of 18, a
counted result, not an absence). `work/w-tu2/survey.py`.

> ### **R2 (registered as a CORRECTION, not a prediction).** w-tu1's criterion is stated as *"all eight sections already in the writer's vocabulary; every alternative carries at least three of those"*. **The section-name axis cannot discriminate on the frontier at all**: factor C is a *membership condition* for the FRONTIER, so **all 18** frontier TUs are inside the writer's 10 names, `outside-writer` empty for every one. What actually separated `xboxmem` was not section *names* but obj-shape features *within* those names — `.pdata` COMDATs, `$M`/`$T` labels, frames.

Measured, `xboxmem` (converted) against `mmio`:

| feature | xboxmem | mmio |
|---|---:|---:|
| sections / `.text` COMDATs | 8 / 4 | 18 / 11 |
| `.pdata` COMDATs | **0** | **3** |
| `$M` labels · `$T` symbols | **0 · 0** | **6 · 3** |
| framed functions | **0** | **3** |
| indirect calls (`bcctrl`) | 0 | 1 |
| cr0-field branches | 0 | 2 |
| 64-bit callee-saved saves | 0 | 2 |

`mmio` carries **every feature `xboxmem` lacked**, and ranks **17th of 19** on
adverse shape, ahead of only `keygen_xbox`. This is verbatim the case w-tu1 §7
named as unreachable — *"a TU at 6 refusals with 0-of-1 emitted and a frame, a
`.pdata` record and a `$M` number is not"* — except that `mmio` has three frames,
three `.pdata` records and six `$M` numbers.

**But the shape table OVER-prices it, and I record that against myself before
measuring.** The port already emits `.pdata`, `$M`, `$T` and framed prologues
(`coff/writer.rs:86,150,400`; the W-UNW-1 shape). Frames and `.pdata` are **not**
new mechanisms. So the shape table is a *prior*, not the price — the same status
w-dclass's reprice table has.

---

## 3. My own re-price of `mmio` — registered at **17**, against the table's 5

Counted off the three blocked bodies' disassembly, over and above the framed
shape the port already has. Six are selection/allocation/scheduling facts
(§4.3's declared blind spot), consistent with w-tu1's 15 and w-dclass's 10.

| # | fact | class |
|---|---|---|
| 1 | `std r31,-16(r1)` / `ld r31,-16(r1)` callee-saved slot in the frame | HARD |
| 2 | the *decision* to hold a formal in r31 across a call | ALLOC |
| 3 | multi-block CFG inside a framed function (guard blocks) | HARD |
| 4 | a **materialized common epilogue** reached by `b` | HARD |
| 5 | early-return values (`li r3,5` / `li r3,11`) merging into it | HARD |
| 6 | `cmplwi cr6` guard + `bne cr6` | (incumbent) |
| 7 | `cmplwi cr0` + `bne cr0` — a **second CR-field regime**, and which one c2 picks | SELECT |
| 8 | member load at offset `lwz r11,28(r31)` | HARD |
| 9 | member store at offset `stw r11,32(r31)` | HARD |
| 10 | `cmplw` register-vs-register | HARD |
| 11 | `ble cr6,+8` skip-store, a third branch shape | HARD |
| 12 | **intra-TU** REL24 (`bl mmioFlush`) — target is a COMDAT in this obj | HARD |
| 13 | indirect call `lwz r11,8(r31)` / `mtctr` / `bcctrl` | HARD |
| 14 | 3-argument call setup (`li r5,0x48 ; mr r4,r11 ; mr r3,r31`) | SELECT |
| 15 | `.pdata` unwind word values for these frames (`40 00 15 03` etc.) | HARD |
| 16 | `$M` prologue-end offset varies with the save set (0xc vs 0x10) | SCHED |
| 17 | **the `$M` counter GAPS between functions** (+3 and +8) | **SCHED — and see §4** |

---

## 4. The registered prediction, and it is about fact 17

`mmio`'s six `$M` numbers are `3381,3382,3383` · `3386,3387,3388` ·
`3396,3397,3398`. The gaps are **+3** and **+8**, so **2 and 7 counter slots are
charged by something between them**. Any wrong slot is six wrong bytes in the
symbol table and the TU does not match.

Board **#286/#287** (lane `w-label`, `LABEL_COUNTER.md` §4.1) measured that this
control-flow surcharge is derivable from **neither the emitted obj** (`ho-ternary`
and `cf-ifelse` are the same emitted shape and charge +2 and +1) **nor the `.gl`
label seed** (`cf-if2`/`cf-ifelse` share a seed and differ by +1). The only
unexamined channel is a per-function `.ex` field nobody has found.

`xboxmem` had **zero** `$M` labels, so the counter never arose. `mmio` has six
and its three blocked bodies are exactly the *charging* shapes.

> ### **R3 — the registered prediction.** `mmio` is **blocked on #286/#287**, not on codegen breadth. I predict the surcharge grid in §5 finds **no rule that holds on cells it was not fitted to**, and therefore that `mmio` does **not** convert in this lane. **If a rule does hold out-of-sample, R3 is a MISS and I say so** — that would be a bigger result than the conversion.

> ### **R4 — what that says about #465.** #465 prices a frontier TU by *how much is already emitted*. `mmio` scores **8 of 11 = 73 %**, the **best on the frontier**. If R3 holds, #465's own metric ranks as its best candidate a TU blocked on a registered open problem — so **#465 is REFUTED as a selection rule**, by the very TU w-tu1 pre-registered to confirm it. The mechanism I predict: the 8 already-emitted functions are **8-byte `li r3,0 ; blr` stubs**, 64 bytes of 2,794 (2.3 %), while the 3 blocked ones are 316 bytes of `.text` plus all 3 `.pdata`, all 6 `$M` and all 3 `$T`. **A function COUNT is blind to that, exactly as #269's refusal count was blind to what was already emitted.**

**R5.** The port's own refusal on `mmio` stays honest — `NotImplemented`, never a
wrong emit. TU match ends at **9**; point estimate 9, and I register that a lane
whose registered outcome is "no movement" must still leave the tree green.

**R6.** Gate unmoved: `cargo test` 813 / 0 / **27 targets**; `gate.sh` 18/18 with
4,482 verdicts; sweep **96 ungraded HELD**; cross **388 ungraded HELD**.

**R7.** I will not ship a schedule rule fitted to `mmio`'s own cells. If the §5
grid produces a rule, it must hold on held-out cells or be refused by name.

---

## 5. The experiment

A constructed grid through **real c2** at the workload's profile, varying the
interior control-flow shape of a framed function and reading the `$M` numbers of
a **following** framed function out of the symbol table. That measures the
surcharge directly, on cells of my choosing, and is the only thing that can
settle R3 either way. Held-out cells are chosen before the fit is attempted.

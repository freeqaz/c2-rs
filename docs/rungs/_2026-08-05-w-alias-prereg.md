# w-alias — PRE-REGISTRATION, written before any Rust number existed

    Tag:       w-alias-prereg
    Slug:      w-alias-prereg
    Date:      2026-08-05
    Fixtures:  none — this lane ships a READER, not an emitter; no obj changes
    Census:    unchanged (no `crates/c2-core` change; `PortC2` consumes nothing here)
    Record:    docs/rungs/_2026-08-05-w-alias-findings.md

## What is being built

`docs/rungs/_2026-08-04-w-emitp-findings.md` §6, items 1, 2, 4 and 5, in
`crates/c2-il`: the `.gl` **tag-0x10 ALIAS** record decode, the `alias: Token →
Token` table, its corpus invariants, and the `DISCLOSURE.md` rows.

**Item 3 — "apply it once at the `in` `02`-node resolution site only" — is
registered here as NOT IMPLEMENTABLE in this lane**, and the reason is stated
before the work rather than after it: `PortC2` has no emit-set model, so that
site does not exist in `crates/`. What ships is the resolution *operator* the
site would call, plus the documented prohibition on applying it to the `.gl`
reference list.

**Seam.** `crates/c2-il` only. `wt-w-rdata` has three commits touching
`crates/c2-core/src/coff/function.rs` and `wt-w-reach` owns `crates/c2-harness`;
both are checked, not assumed.

## The registered predictions

**Verification method: two independent implementations must agree.** w-emitp's
Python (`work/w-emitp/alias.py`) is frozen; this lane's Rust is written from the
same disassembly transcript. Every number below is Python's, published at
`_2026-08-04-w-emitp-findings.md` §1c and §2.2, and the Rust must land on it.

### Decode — 850 TUs of the capture cache

| # | quantity | registered **point** | interval |
|---|---|---:|---|
| **R1** | tag-0x10 records | **96 220** | exact — no interval |
| **R2** | bound | **95 820** | exact |
| **R3** | of the bound, shape `??_E<X>` → `??_G<X>` | **95 818** | exact |
| **R4** | `head_fail` / `rt_fail` / unbound target / self / dup | **352 / 0 / 48 / 0 / 0** | exact |
| **R5** | TUs whose Rust and Python alias tables are equal **name for name** | **850 / 850** | exact |
| **R6** | SHIFT null `p−1` bound / `p+1` bound | **1 795 / 2 449** | exact |
| **R7** | SHIFT null pairs, either direction | **0** | exact |
| **R8** | `dom(alias) ∩ U` | **0** | [0, 0] |

**R1–R8 are registered as EXACT because an interval would make them
unfalsifiable.** Two implementations of one transcript either agree or one of
them is wrong; "close" is the failure mode, not a pass.

### Model — the Rust table substituted into w-emitp's frozen `scan.py`

`scan.py` is not edited. The substitution replaces `alias.scan` with a loader
for the Rust table and nothing else, so a difference in any row below is a
difference in the *decode*, not in the model.

| # | quantity | registered **point** | interval |
|---|---|---:|---|
| **M1** | `JFP_ALIAS` per-TU exact, of 850 | **308** | exact |
| **M2** | `JFP_ALIAS` micro-F1 | **0.94413** | exact to 5 dp |
| **M3** | `ALIAS_IN` per-TU exact / F1 | **472 / 0.99243** | exact |
| **M4** | `JFP` (the alias-free incumbent — the KA control) | **132 / 0.92655** | exact |
| **M5** | `ALIAS_REF` − `RGL`, F1 | **+0.00000** | exact |

**Per-TU exact is the metric that matters (board #250) and it is printed beside
micro-F1 in every table this lane writes.**

### The port — registered as MOVING NOTHING

| # | quantity | registered |
|---|---|---|
| **P1** | `c2rs gap` match / mismatch / vocab-gap / capture-fail / FRONTIER | **8 / 0 / 863 / 7 / 19**, unchanged |
| **P2** | A / B / C / D / E | **28 / 338 / 169 / 8 / 2**, unchanged |
| **P3** | `scripts/gate.sh` | **18/18 PASS, 4 410 verdicts**, unchanged |
| **P4** | `scripts/expr_sweep.sh` | **16 394 / 16 298 graded / 96 ungraded / 0 mismatch** — the 96 **held** |
| **P5** | mode cross | **75 829 of 76 217 / 0 mismatch** |
| **P6** | `cargo test --workspace --release` | **781 + this lane's new tests**, 0 failed |

**P1–P5 must not move, and if any of them does, that is the alarm and not the
result.** Nothing in `PortC2` reads this table; a moved obj number would mean the
reader is not additive, which is the exact failure board #232 records.

## What would make me decline

1. **Any of R1–R8 disagreeing with Python.** I report the disagreement as the
   lane's finding and **do not adjust the Rust until it matches** — fitting one
   implementation to the other destroys the only check this lane has.
2. **`dom(alias) ∩ U > 0` anywhere in the corpus.** §6 rule 4 ("never emit a
   name in `dom(alias)`") would then be capable of suppressing a name that has a
   body and must be emitted. I would ship the count and a refusal hook and
   **not** the unconditional rule.
3. **Any requirement to touch `crates/c2-core` or `crates/c2-harness`.** Both
   are live under other lanes; I stop and report what I could not do.
4. **Any widening of a path that emits.** This lane adds a reader. If a change
   would turn a refusal into an emit, it does not land — and I must be able to
   construct the case that would make it wrong *before* it lands.
5. **The one-shot Part-1 gate is NOT spent by this lane.** The question is put
   to the coordinator in the findings doc; it is not answered here.

## Declared bias

I expect **R1–R7 to reproduce exactly** and I am aware that is the comfortable
prediction. The place I expect to be wrong is **R5**: reproducing three
aggregate counts is much weaker than reproducing 850 tables name-for-name, and
two run-walkers agreeing in aggregate while disagreeing on individual TUs is a
real and unremarkable outcome. If R1–R4 hit and **R5 misses**, the aggregate
agreement is a coincidence of compensating errors and the lane's headline is
that, not the decode.

Second declared bias: I expect **P1–P5 unchanged** and I am aware that a lane
predicting its own change moves nothing has no way to be surprised by success.
The falsifiable half is that the *tests* and the *decode counts* must move.

# w-lo — PRE-REGISTRATION (written before the first measurement)

    Lane:      w-lo
    Board:     #158, the dynamic-initializer thunk — DECODE HALF ONLY
    Date:      2026-08-02
    Base:      master @ a091e37, branch `wt-w-lo`
    Seam:      crates/c2-il/src/codec.rs, crates/c2-il/src/func/bundle.rs
    Record:    ROADMAP §10.11, §10.12; fixtures/cpp/il_dyninit_static.cpp

## The claim under test

§10.12 measured, over nine functions in five captures, that `LO_MARKER`
(`4C 4F 11`) is not one atom: the grammar is **`4C`, then an OPTIONAL `4F 11`
record, then `53`**. Every `??__E`/`??__F` dynamic-initializer / atexit thunk
carries the bare `4C`; everything else measured (including the synthesized
`??_G` deleting destructor) carries `4C 4F 11`.

This lane re-tokenizes so a bare-`4C` body decodes. **The obj-emit half
(`.rdata` + `.bss` + `.CRT$XCU` beside `.text`) is lane w-objshape's and is not
attempted here.** Both halves are needed for the two license TUs, so **no TU
match is expected from this lane and none will be chased.**

## Bias, stated in writing

My prior is that this change is **census-NEGATIVE and TU-neutral**, and I am
registering that before measuring because the tempting report ("the census went
up, the decode landed") is the opposite of what I expect. The re-tokenization
adds function segments that were previously invisible to
`split_function_bodies_at`; those segments are `??__E`/`??__F` thunks whose
bodies are address-materialization + tail-call shapes the port does not accept.
So they land in the **denominator** and not in the numerator, and the census
**percentage falls**. A census rise would be the surprise, not the success.

The second bias worth naming: I expect to be tempted to widen the bare-`4C`
anchor beyond what the data supports, because a looser anchor decodes more.
`4C` is one byte and is overloaded (last byte of `IntCallEnd` `55 86 41 74 4C`,
first of `VoidCallEnd` `4C 4B`), so a loose anchor invents segments out of
payload bytes. The registered discipline: the new anchor must be **strictly
additive** — it may only fire in a `4F 1F` region that contains **no** `4C 4F 11`
at all, so every function that decodes today keeps byte-identical treatment.

## Registered predictions

Units named first, because the one prior-lane miss on this board was a
per-function estimate for a per-TU change.

| # | quantity | unit | incumbent (a091e37) | predicted |
|---|---|---|---|---|
| P1 | per-function census **numerator** | functions | 706,402 | **+0** (80 % conf: 0 … +200) |
| P2 | per-function census **denominator** | functions | 2,462,571 | **+300 … +6,000**, point estimate **+1,500** |
| P3 | per-function census **percentage** | pp | 28.69 % | **falls**, by 0.00 … 0.07 pp; point **−0.02 pp** |
| P4 | TU **match** | TUs of 878 | 6 | **6** (exactly — no change) |
| P5 | TU **mismatch** | TUs | 0 | **0** |
| P6 | TU **vocab-gap** | TUs | 865 | **falls by 0 … 12**, point **−2** |
| P7 | TU **codegen-gap** | TUs | 0 | **rises by 0 … 12**, point **+2** (the two license TUs) |
| P8 | `fn_total` on `TomCryptLicense.cpp` / `ZlibLicense.cpp` | segments | 0 / 0 | **1 / 1** |
| P9 | fixtures changing **verdict** (`c2rs perf`, 212) | fixtures | 100 Match / 112 NotImplemented | **0 change** — 100 / 112 |
| P10 | fixtures changing **selftest** result | fixtures | 212 PASS | **0 change** |
| P11 | `gate.sh` lanes | lanes | 12/12 PASS, 2,544 verdicts, 0 mismatch | **identical**, 0 mismatch |
| P12 | K1 round-trip (`il_roundtrip.rs`) | fixtures | byte-exact, all | **byte-exact, all** — no fixture excluded |
| P13 | workspace tests | targets / tests | 24 targets, 606 passed | **24 targets, ≥606 passed** (new unit tests only add) |

Notes on the shape of each:

* **P8 is the only prediction that is per-TU and directly caused.** It is the
  narrowest test of the byte claim: `split_function_bodies_at` returns 0
  segments for those two TUs today purely because it anchors on `4C 4F 11`.
* **P7 vs P5 is the load-bearing distinction.** If `IlBundle::functions()`
  starts returning `Some` for a TU, the port is consulted. It must answer
  `NotImplemented` (→ `codegen-gap`), because the obj carries `.rdata`/`.bss`/
  `.CRT$XCU` and the port emits a fixed four-section shell. If it instead
  *emits* and the compare fails, that is a **mismatch** — an alarm, and a
  decline.
* **P12** — the K1 test's `#158` branch currently asserts "a body with no
  `4C 4F 11` must decode to 0 tokens". If this lane succeeds that branch becomes
  false. It will be updated to a **new positive claim** — such a body decodes to
  ≥1 token *and* carries ≥1 function — never to a hole or a skip.

## Binding decline clauses (registered in advance, BINDING)

The change is **DECLINED and not merged**, whatever it does to any census or
ceiling, if any of these holds:

1. TU **match < 6**.
2. TU **mismatch > 0** anywhere.
3. Any `gate.sh` lane lost (a lane that FAILs, SKIPs, or returns NO-RESULT, or a
   fixture-verdict count below 2,544).
4. K1 round-trip not byte-exact on any of the 212 fixtures, or made green by
   excluding a fixture.
5. Any `cargo test --workspace --release` target lost (< 24 targets) or any
   test failing.

There is no interval in which losing one of these is a pass. **Declining on
measurement is a good outcome** and will be reported as the deliverable if the
anchor cannot be made strictly additive.

## What would falsify the byte claim

If a `4F 1F` region with no `4C 4F 11` turns out **not** to have a `4C` in the
position the grammar predicts (after `46 <formals>`, before the first `53`), or
if the bare-`4C` anchor fires on regions that are payload collisions rather than
functions, the §10.12 grammar reading is wrong as stated and the lane reports
that instead of a decode.

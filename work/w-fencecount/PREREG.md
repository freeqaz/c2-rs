# PREREG — w-fencecount (instrument rung: the fence-blocks-exact counter)

Frozen BEFORE the first scan, the first probe compile, and the first `crates/`
change. Base for every claim: branch point of `wt-w-fencecount` (`8fbe6ef5`,
master at dispatch). Published base numbers taken from `docs/STATUS.md`'s
generated block at tree `c8dce...`/`c8bce`-era (`c8ec`): **match 25 · mismatch 0
· codegen-gap 0 · vocab-gap 845 · capture-fail 8**; workspace tests **1,527
passed / 41 targets**; `git grep -c '#[test]'` summed = **1,816** (measured at
this base before anything ran).

## The deliverable being registered

A standing per-fence counter printed by the 878-TU scan (`c2rs gap`) beside the
class table, plus stable `gap-metric` keys, all NEW (no existing key renamed,
re-keyed, or moved):

* `fence-blocks-sole:<cause>` — graded TUs held out of `match` whose ENTIRE
  `IlBundle::decode_causes()` set is exactly `{cause}`.
* `fence-blocks-exact:<cause>` — of those, TUs where **every emitted body is
  FnByte-exact** (per-TU `fnbyte-exact == fnbyte-denominator`, denominator
  **> 0** — the vacuous d=0 case is excluded by construction, positively).
* `fence-blocks-exact-bodies:<cause>` — the byte-exact emitted bodies inside
  the `fence-blocks-exact` TUs.
* `fence-blocks-first:<cause>` — TUs where `<cause>` is the FIRST blocker of a
  multi-cause set. Printed with the standing caveat: the port stops at the
  first refusal, so a first-blocker key is NOT a distance.
* Controls: `fence-held-tus`, `fence-cause-firings` (arity: causes summed over
  held TUs — residue counts entities, arity counts their contents),
  `fence-residue-no-cause` (vocab-gap TU with EMPTY causes; known answer 0),
  `fence-accounting-broken` (totality: sole + first-of-multi == held; known
  answer 0), `fence-arity-broken` (first-cause/causes-list cross-field
  consistency; known answer 0), `fence-match-tus-checked` (positive count of
  match TUs verified to contribute zero).

Attribution source is the EXISTING re-ask seam (`IlBundle::decode_causes`,
lane w-vocab / w-vec) and the EXISTING per-TU FnByte reader
(`GapReport::fn_byte_by_tu`'s `emit` keys) — no new reader of either fact.
This lane changes ZERO behavior in `crates/c2-il` / `crates/c2-core`.

## Documentary findings already in hand at freeze (from reading, not running)

* **The brief's positive-control premise is STALE.** The brief mandates:
  "vsnprnc.cpp … is held out of `match` solely by the inline fence; if your
  counter does not fire on this TU, the instrument is wrong." Merge
  `b7e0a772` (w-fence2, 2026-08-09, an ancestor of this base) narrowed the
  inline fence and **converted vsnprnc.cpp to `match`** (TU match 19 → 20);
  five later merges took match to 25 with no sign of a vsnprnc regression.
  The two mandates in the brief ("the 25 matching TUs must show
  fence-blocks-exact 0" AND "the vsnprnc control must fire") are jointly
  satisfiable only if vsnprnc is not among the 25 — it is. The control is
  therefore realized as a FIXTURE that reconstructs the vsnprnc shape
  (below), and the workload reading is registered as a prediction (D1).
* **Latent diag/gate drift found by reading:** `decode_causes`' LOCAL_CALLEE
  clause (`crates/c2-il/src/func/diag.rs` ~490) re-asks the BROAD
  `callee_defined_here` while the gate (`bundle.rs` ~2302–2339) asks the
  w-fence2-NARROWED `callee_defined_here_unmodelled` with the plain-external
  /O1 exemption. On a decoding TU with an exempted local callee (vsnprnc),
  the documented invariant `causes.is_empty() == decodes` is violated —
  latent because the scan calls `decode_causes` only when `!decodes`.
  Registered consequence: the counter's `locally-defined-callee` key names
  the **intra-TU-call complex as diag re-asks it** (broad), not the narrowed
  gate clause precisely; the caveat is printed with the block, and the drift
  is filed as a finding (board row), NOT fixed here (shared surface).

## The positive control (fixture, toolchain-gated)

One new fixture, prefix `wfcnt`: a caller tail-calling a **`static`** callee
large enough that c2 keeps the call (F1's static /O1 inline ceiling is
`(300,308]`; the callee is built well above it), both bodies inside the port's
codegen class. Expected at the workload-style `/O1` profile:

* reference obj: caller is a small wrapper CARRYING the call (proved by
  `fnbyte-exact == denominator == 2`, which is impossible if c2 inlined);
* port: `IlBundle::functions` refuses the TU at the narrowed fence (static is
  not exempt); `decode_causes` = exactly `["locally-defined-callee"]`;
* so the scan over this fixture must read
  `fence-blocks-exact:locally-defined-callee ≥ 1` **with the fixture present
  by name in the graded set** — a check positive on content, not an absence.

The integration test asserts each of those as a separate assertion with a
distinct failure message, and the run count ("graded N > 0 TUs") is asserted
before any of them. Mutation discipline: each new guard is watched RED once
during development (mutate, observe the distinct message, revert) and the
mutations are recorded in the rung doc.

**Frozen rule: if the control fixture fails to isolate (extra causes co-fire),
the FIXTURE is adjusted until sole-cause holds; the instrument is not.** The
instrument's definition above is frozen at this commit.

## Predictions (probability form; units are TUs unless the key says bodies)

| # | prediction | p |
|---|---|--:|
| D1 | `vsnprnc.cpp` is `match` at base — the brief's control premise does not hold on the workload | 0.90 |
| D2 | `fence-blocks-exact:locally-defined-callee` = **0** at base on the 878-TU scan (w-fence2 already converted the only vsnprnc-shaped TU; w-fence2 §1 measured T1 ALL-EXACT-NO-MATCH = 1 and it is `vec.cpp`, held by `gl-stop-26-introduced`, not the fence) | 0.75 |
| D2a | …= 1 or more (some unnoticed TU has the shape) | 0.15 |
| D3 | `fence-blocks-sole:locally-defined-callee` at base = **0** | 0.55 |
| D3a | …in 1..=2 | 0.30 |
| D4 | `fence-blocks-first:locally-defined-callee` ≥ 1 at base (keygen_xbox and/or wordwrap bind and carry intra-TU edges among other causes) | 0.50 |
| D5 | the largest sole-cause key at base is a BINDING-family cause (`bind-record-count-ne-segments` or a `gl-stop-*`) | 0.65 |
| D6 | `fence-residue-no-cause` = 0 and `fence-accounting-broken` = 0 and `fence-arity-broken` = 0 at base AND tip | 0.90 |
| D7 | match delta 0; mismatch 0 at every level; census +0; fnbyte-exact delta 0; every pre-existing 878-TU scan key digit-for-digit identical base→tip | 0.90 |
| D8 | the control fixture isolates: sole cause `locally-defined-callee`, fnbyte 2/2, `fence-blocks-exact` fires on it by name | 0.65 |
| D8a | …c2 keeps the >308 B static callee's call at /O1 (the enabling half alone) | 0.75 |
| D9 | workspace tests: base 1,527 / 41 targets → tip in [1,531, 1,539] passed, targets **42** (one new integration target) | 0.60 |
| D10 | `git grep -c '#[test]'` delta in [+4, +10] (base 1,816) | 0.65 |
| D11 | the diag/gate drift (broad vs narrowed LOCAL_CALLEE) is demonstrated live on a captured decoding TU (invariant `causes.is_empty()==decodes` violated) | 0.70 |
| D12 | ≥ 1 unnamed refusal fires at a pre-armed place (armed: fixture-list regeneration after the last fixture edit; the control fixture needing >1 iteration to isolate; a gate first-run self-void interaction despite w-gate3048) | 0.60 |
| D13 | mismatch 0 everywhere: 878 TUs, all gate lanes, both fixture modes | 0.97 |

Registered direction on D2/D3: the coordinator's brief predicts the opposite
(vsnprnc fires on the workload); this PREREG registers that the workload count
is ZERO and the instrument's mandatory firing is carried by the fixture. If
D2 is wrong and a workload TU fires, that is a better outcome, not a failure.

## Unnamed-refusal budget

One budgeted. Pre-armed places listed in D12. Anything else is over budget and
is reported as such.

# w-value — PRE-REGISTRATION

**Frozen before the first change to `crates/` and before the first fixture this
lane authors.** Everything in §0 and §1 was taken from base-only instruments at
master **`285a94d7`**: the 878-TU scan (`work/w-value/scan_base.{out,jsonl}`),
the two fixture scans (`fixbase.jsonl` at `/O1`, `fixbase_ox.jsonl` at `/Ox`),
`cargo test --workspace --release`, and `c2rs census` on the two commissioned
TUs. No number below is inherited from a rung, a board row or a survey: five
consecutive survey prices have been wrong and the standing instruction is to
re-derive.

Lane: `w-value`, worktree branch `wt-w-value` off master **`285a94d7`**.
Commission: ROADMAP §10.26 item 2 (as amended by §10.26.2, which makes it the
sole next code seam) — **the member-call value model**, `26 … 2C … 99 … BD`.

---

## 0. The base, measured and not quoted

| | value at `285a94d7` |
|---|---:|
| TU match | **18** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 |
| vocab-gap · capture-fail | 853 · 7 |
| FRONTIER | **9** |
| factor A / B / C | 28 / 338 / 169 |
| `b-and-c` · `a-and-b-and-c` | 151 · 27 |
| function census | **711,494 / 2,463,443** (28.88 %) |
| emitted census | **39,193 / 178,977** (21.90 %) |
| `gap-metric` keys | **251** |
| workspace tests | **1,320 passed, 0 failed, 36 targets** |
| fixtures at `/O1` (305) | 154 match · 9 codegen-gap · 142 vocab-gap · **0 mismatch** |
| fixtures at `/Ox` (305) | 139 match · 17 codegen-gap · 149 vocab-gap · **0 mismatch** |

### 0.1 The family, counted at base rather than quoted (`work/w-value/fam.py`)

| family (first-blocker prefix) | bodies | **emitted** |
|---|---:|---:|
| `expr-call-in-expr*` | 449,274 | **36,751** |
| `expr-convert-no-value*` | 4,973 | 371 |
| `expr-op-0x99` | 0 | 0 |
| `expr-op-0xBD*` | 34 | 31 |
| every key, all TUs | 1,751,949 | 130,567 |

**36,751 reproduces board #1534's headline digit for digit**, and 4,973
reproduces #1462's. So the family is the one the commission names and the base
is the one the board rows were written against.

### 0.2 The two commissioned TUs, re-derived at base

`c2rs census … --flags-file work/dc3-workload/flags.txt --cwd <dc3>`:

* **`src/Main.cpp`** → `0/1 in class`, first blocker
  **`param-width-undetermined:mid`**, blocking window
  `… 2d 0a 0a >4c< 4f 11 53 4f 01 04 26 fb 09 26 0e 0a 2c a6 43 81 20 00 99 …`.
  **wb-eh #1864 reproduces exactly**: the row dies in the FORMALS HEADER, four
  tokens before the body opens, and never reaches a `26`.
* **`src/system/synth_xbox/Biquad.cpp`** → `0/2`, two bodies:
  `expr-cmp-eq` (the 838-byte `?SetCoefficients`) and
  **`expr-call-in-expr-recv-load-then-plumbing-0x3A`** (the 162-byte ctor),
  whose window is the spine itself:
  `4c 4f 11 53 >26< e8 09  b9 f7 09 a6 43 81 20  2c a6 43 81 20 00  99 86 43 84 20 00  bd 82 …`.

**The whole `param-*` family is 6,967 bodies / 682 emitted** — 54× smaller than
the `26` family — so `Main.cpp`'s head is a *small* seam that happens to sit in
front of a large one. Registered here because it decides P4 below.

### 0.3 The frontier, per TU, at base (`work/w-value/frontier_base.txt`)

25 blocked emitted bodies over the 9 frontier TUs. **Two of the 25 are in the
`26` family** (Biquad's ctor, and 2 of keygen's 18 — `…-data-addr-then-plain-call-and-op-more`
and `…-op-0x1F`). Everything else is `expr-cmp-eq` (10), `expr-jump` (9),
`expr-op-0x27` (4), and one each of eight further keys.

---

## 1. What ships, and the acceptance theorem it is built on

**The member-call VALUE model, in `parse_expr_classed`'s `0x26` arm.** Today
that arm is `return Err(mcall::classify(seg, *p))` — one byte, no walk. It
becomes: tokenize the whole `26 … BD … 4C` production with a bracket walk over
`mcall`'s existing width-complete readers, model its stack effect, push the
call's return value onto `cstack`, and **continue the walk**. A body that
reaches the end of the walk having consumed one is refused there, under
`mcall::classify` anchored at the *first* `26` — i.e. **the exact Block the
current arm produces**.

> ### THE ACCEPTANCE THEOREM, registered before the code is written
>
> The new arm is reached only on byte `0x26` inside `parse_expr`, and **every
> path that reaches byte `0x26` inside `parse_expr` returns `Err` today,
> unconditionally**. The arm can therefore only replace one `Err` with another
> `Err`: at the end of the walk the poison re-raises the *same* `Block`, and
> anywhere earlier some other arm's own refusal fires. **`parse_expr` cannot
> return `Ok` on any body it refuses today.** Acceptance is bit-identical,
> the census cannot over-claim (#139), and `mismatch` cannot move.
>
> This is the property the whole lane rests on, so it is registered as a claim
> to be *falsified by measurement*, not assumed: §0's per-TU set, the 305
> fixtures at both profiles, and the census counts are the three levels that
> can catch it being false.

**What the model does NOT do**, registered so it cannot be claimed later:

* It pushes **no `IlOp`**. `IlOp` has no call variant and this lane does not add
  one — the emitter is untouched, and nothing in `crates/c2-core` changes.
* It does not touch `IlBundle::functions()`, `PORT_CFG_CLASSES`, or any
  recognizer.
* It handles **only** a `26` run that reaches a `BD`-opened region and its
  closing `4C`. A bare data-symbol address push (`f("hello")`, ~18 % of the
  bucket per `IL_CALL_IN_EXPR.md` §2) is **not** a call, is left refused exactly
  as today, and its keys must not move.

**FENCE ORDER, decided in advance** (w-park's finding, streak 6/9): the call
poison goes **first** among the end-of-walk guards, ahead of the three sink
poisons and ahead of `expr-ptr-arith` / `expr-ptr-bitwise` / `expr-shr-sign-late`
/ the `int1u` three. Reason: today the `26` refuses *before* any of those can be
reached, so first is the ordering that leaves the published spellings of those
keys measuring the same population they measure now. Any other position would
move functions between two keys for a reason that is not the construct.

---

## 2. Predictions, in probability form

Scored in the rung in the registered direction. `p` is my credence before any
tip measurement exists.

| # | prediction | p |
|---|---|---:|
| **P1** | **Verdict-neutral at all three levels**: per-TU verdict set unchanged BY NAME over all 878 (0 only-in-base, 0 only-in-tip, 0 moved); 305 fixtures unchanged by name at `/O1` **and** `/Ox`; function census and emitted census **+0**; `mismatch` 0 everywhere | **0.97** |
| **P2** | The family's **emitted** head count falls by **≥ 10,000** of 36,751 — i.e. the whole-production counterfactual #1534 has never had shows that most of the family is shadowing something deeper | 0.70 |
| P2b | …and falls by **≥ 20,000** | 0.40 |
| **P3** | `expr-op-0x27` (already #1 at 22,373 emitted / 402,139 bodies) is the **largest single recipient** of the moved heads | 0.55 |
| **P4** | `src/Main.cpp`'s first blocker at tip is **unchanged** (`param-width-undetermined:mid`) — the value model pays wb-eh's R2 and does **not** move this TU's head, because its head is R1 and R1 is a `.sy` seam this lane does not touch | 0.92 |
| **P5** | `Biquad.cpp`'s ctor **moves off** `expr-call-in-expr-recv-load-then-plumbing-0x3A`, and its successor is `expr-jump` (opcode `3A`) | 0.60 (move: 0.85; successor named right: 0.70) |
| **P6** | `FRONTIER` stays **9** and the frontier TU set is unchanged by name | 0.93 |
| **P7** | **Conversions: 0.** No TU and no fixture moves `vocab-gap → match` | 0.90 |
| **P8** | The `0x2C` width A/B (§3) shows a **zero** decode-reach delta over all 878 TUs — no site in this workload carries a `2C` payload ≥ `0x80` — so the desync is unwitnessed here and the adoption is **DECLINED** | 0.75 |
| **P9** | **Test-count DELTA: +7** (#1749 — registered as a delta, not a total; base is 1,320) | 0.45 (±2: 0.80) |
| **P10** | **At least one refusal I have not named above turns up** in the tip scan's key diff — the budgeted unnamed refusal. Pre-armed on FENCE ORDER, per w-park | 0.75 |
| P11 | `gap-metric` key **count** stays 251 (no key vanishes, no key appears) — the keys' *values* are expected to move and that is the deliverable | 0.55 |

Board #770's streak is ~10 optimistic / 2 pessimistic / 1 hit across the
project. The direction of any miss is recorded in the rung.

---

## 3. The side-probe: `0x2C`'s width, and a rule for adopting it

`WB_READER_FINDINGS.md` §3.4 records that c2 reads **one raw byte** after
`2C`'s TYPE and the port reads a **varint**; §5.4 designs the check and calls it
"a query, not an experiment"; `WB_EH_FINDINGS.md` §4.3 records **P3.5 as a
MISS** — wb-eh found no site with a payload ≥ `0x80` across 7 decoded sites.

**The black-box discriminator, which needs no disassembly and no `cl.exe`:**
flip the port's own `2C` payload rule from varint to one raw byte in a scratch
build and re-run the 878-TU scan. The two rules are identical below `0x80` and
desynchronise above it, so **any** site with a payload ≥ `0x80` shows up as a
decode-reach delta — a moved first-blocker key — and no site does if the delta
is empty.

> **DECLINE CLAUSE D5, frozen:** the `0x2C` raw-byte width is adopted into
> `crates/` **only if** that A/B produces a nonzero delta, which would be a
> black-box witness. If the delta is zero the reading is *unwitnessed on this
> corpus*, `DISCLOSURE.md`'s grey-zone rule prefers the black-box derivation
> that does not exist, and the lane **declines the adoption and publishes the
> zero**. A width changed on a disassembly reading with no corpus witness is an
> adoption with nothing able to grade it.

---

## 4. Decline clauses, with sizes

Each fires on a measurement, not on a judgement.

* **D1 — verdict movement.** If the 878-TU per-TU verdict set differs by name in
  either direction, or the function/emitted census moves by any amount, the
  model is not neutral: **revert `crates/`, ship nothing, publish the diff.**
* **D2 — `mismatch`.** Any `mismatch > 0`, anywhere (workload, fixtures at
  either profile, `gate.sh`, `expr_sweep`, `mode_cross`): **revert.** This is
  the one direction the correctness rule forbids.
* **D3 — the counterfactual is too small to be one.** If the family's emitted
  head count moves by **< 1,000** of 36,751 (< 2.7 %), the walk is not
  consuming the production in practice and the lane **does not ship the arm**;
  it publishes the number as the price, per #1534's own R4-shaped rule.
* **D4 — fixture movement.** Any of the 305 fixtures moving in either direction
  at `/O1` or `/Ox`: **revert.**
* **D5 — the `0x2C` adoption**, above.
* **D6 — a hatch needle.** If `work/w-front3/hatch.py` needles a line this lane
  changed, follow w-park's precedent: **re-take the needle against the tree**
  and retire the row only if the clause is fully paid. A clause this lane does
  not pay stays live.

---

## 5. What this lane will NOT claim

* Not a conversion. P7 registers 0 and the campaign's own §10.26.1 says
  "first-scan reach stays ~0 (the reader still gates)".
* Not a widening of the accepted class. See the acceptance theorem.
* Not a price for `Main.cpp`. wb-eh §6 prices it at fifteen refusals; this lane
  pays **R2** and nothing else, and R1 (`param-width-undetermined`, the `.sy`
  binding) still stands in front of it. A rung that paid R2 and reported
  "`Main.cpp` is closer" without saying R1 is untouched would be the fifth
  consecutive wrong survey price.
* Not an adoption from `docs/whitebox/` unless D5's condition is met, in which
  case the `DISCLOSURE.md` row ships in the **same commit**.

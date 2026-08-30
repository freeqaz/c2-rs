# `WB_EMITPRICE` — the five `emit-change` clauses, priced two-sided

> **Characterization lane `w-emitprice`, 2026-08-29.** Wave 21 L2, brief
> [`../WAVE21_BRIEF_2026-08-29.md`](../WAVE21_BRIEF_2026-08-29.md) §2. Board
> **#3856**–**#3862**. Prereg [`../../work/w-emitprice/PREREG.md`](../../work/w-emitprice/PREREG.md),
> committed at `314f2696f` **before the image was opened**.
>
> **Predicted reach 0, delivered 0.** Zero `crates/` bytes, no
> [`DISCLOSURE.md`](DISCLOSURE.md) row, no `gate.sh` row (`#3691`).
> Image `sha256 c80981c0…a66258`, verified.
>
> **This lane may not edit `work/w-inlmetric/CLAUSES.tsv` or
> [`ref/P_INLINE.md`](ref/P_INLINE.md)** — `w-budget` owns both this wave
> (`#3814`). Every correction this page proposes to either is recorded **here,
> with its evidence**, exactly as `w-instrcount` did, and lands in a later wave.
>
> Evidence tiers: **`[R]`** read from the disassembly · **`[O]`** confirmed
> against a real obj or `.gl` · **`[I]`** an interpretive step.

---

## 0. The result in one table, and it is a PARTITION and not a ranking

The commission was to price five rows and rank them. **The ranking is an
artifact and the lane's own registered check (`PREREG` §5, A1) fires**: four of
the five price identically at **zero** in the units the goal is written in, so
what exists is a partition into three classes, not an order over five rows.
Publishing an order over a 4-way tie is manufacturing signal, and `#3505` is
now **seven for seven**.

| # | clause | what ADOPTING it buys | what the standing REFUSAL costs today | larger | class |
|---|---|---|---|---|---|
| **C7** | ceiling VALUE `DAT_10c46318` = 128 | **NEGATIVE.** 3 wrong emits at `/O1` taken raw, 24 through the naive 4 B/word unit; the only translation scoring 0/0 at `/O1` has a converter **fitted to the same bracket the incumbents were fitted to**, so the value adds no information the port lacks. §2 | **0 byte-exact functions on the workload** — `fenced_inlined_callee`'s own measurement, and the incumbent pair is wrong-emit-free on all 82 `/O1` cells. §2.3 | **REFUSAL, decisively** | **priced NEGATIVE** — the one row that is genuinely an emit change, and the change is the wrong direction |
| **C9** | favour-speed bit skips the size test | **0** in `Δmatch`/`Δfnbyte`; byte-neutral **by construction**, because bit 23 is **0 at `/O1`**. High in characterization. §3 | **0** — the arm is never taken at `/O1`, and `/Ox` TUs are refused at the `IlBundle::functions` gate regardless | tie at zero; C9 wins on characterization | **ADOPTABLE AND BYTE-NEUTRAL.** `emit-change` is the wrong blocker |
| **C10** | `__forceinline` bypass, `[sym+0x4c] & 0x2000` | **0** in `Δmatch`/`Δfnbyte`, for three independent reasons. High in characterization: a registered decision point with **no input**. §4 | **0** — the population is refused upstream, `[O]` on this lane's own cells | tie at zero; C10 wins on characterization | **BLOCKED ON ONE MISSING READER** |
| **C11** | legality refuse, `[sym+0x20] & {0x40,0x100,0x400,0x1000}` | **UNKNOWN, and the reason is the finding.** Refuse-side, so `Δmatch`/`Δfnbyte` ≤ 0 and warranty ≥ 0. §5 | **unmeasurable** — nothing in `crates/` decodes the field | unknown | **BLOCKED ON THE SAME MISSING READER**, one field earlier |
| **C12** | legality refuse, `[sym+0x4c] & {0x200,0x80000}` | **UNKNOWN**, bounded at **n = 6 observed full-width `ATTR` values, 0 carrying either bit**. §5 | **unmeasurable**, same reason | unknown | **BLOCKED ON THE SAME MISSING READER** |

> ### THE HEADLINE: three of the five are ONE missing link, not three reasons
>
> **C10, C11 and C12 are all blocked by the same absent thing — the port has no
> `.gl` symbol-record DECODER.** It has a fixed-pattern framing walk
> (`gl_function_attrs`) that reaches exactly **one byte**, and every one of
> these three clauses tests something that byte cannot hold:
>
> | clause | the bit | why the port cannot see it |
> |---|---|---|
> | C10 | `[sym+0x4c]` bit **13** | past the low byte; needs `ATTR`'s continuation decode |
> | C12 | `[sym+0x4c]` bits **9** and **19** | same, and bit 19 needs the **four-byte** form |
> | C11 | `[sym+0x20]`, four bits | a *different field*, read **before** the walk's framing anchor |
>
> That is exactly the shape `CLAUSES.tsv` already records for the other large
> group — *"`no-instr-count` … **That is one missing link, not six reasons**"* —
> and it was not visible because the `blocker` column holds one cell per row and
> `emit-change` is a statement about *scope*, not about *what is missing*.
>
> **`gl.rs` predicted this in its own words and nobody had cashed it**
> (`crates/c2-il/src/func/gl.rs:1520`):
>
> > *"A reader that ever needs `ATTR`'s **value** — not its bit 6 — must decode
> > the continuation; **nothing does today**, and this paragraph is here so that
> > the next one does not discover it the way this field's `SIZE` neighbour was
> > discovered."*
>
> Three clause rows are the next one. §4.2 proves the reader is **necessary and
> not merely convenient**: the low byte the port returns is **identical** for
> `inline` and `__forceinline`.

**Proposed `blocker` corrections, for whoever owns `CLAUSES.tsv` next** (this
lane may not make them). Each is a change of *diagnosis*, and each row's `read`
cell is affected too:

| row | `blocker` today | proposed | `read` today | proposed |
|---|---|---|---|---|
| C7 | `emit-change` | `emit-change` — **unchanged, and now PRICED, at NEGATIVE** | `R1` | `R1` |
| C9 | `emit-change` | **`none`** — adoptable, byte-neutral by construction (C15's precedent) | `R1` | `R1` |
| C10 | `emit-change` | **`no-gl-record-decoder`** | `R1` | **`R2`** — a NAMED link between c2's quantity and anything `crates/` can compute is missing, which is `R2`'s definition |
| C11 | `emit-change` | **`no-gl-record-decoder`** | `R1` | **`R2`** |
| C12 | `emit-change` | **`no-gl-record-decoder`** | `R1` | **`R2`** |

**One row of the five is genuinely an emit change.** The wave brief's framing —
*"they are `absent` but derivable; what stops them is that each is an emit
change needing a two-sided price"* — is right about C7 and wrong about the
other four, and the correction is what the pricing bought.

---

## 1. The method, and what "two-sided" was held to mean

`#1042` and NC-5/`#2691` both **flipped their answer** when the refusal's own
cost was counted rather than only the change's. So each row is priced in the
units `PREREG` §2 fixed in advance:

* **U1 `Δmatch`** — whole objs byte-exact, of 878.
* **U2 `Δfnbyte-exact`** — emitted functions byte-exact, of 162,205.
* **U3 warranty** — live wrong emits opened or closed. A **sign**, and it
  **dominates** U1/U2: `PROGRESS_METRIC.md` §5.2, *a wrong emit scores strictly
  below the refusal it replaced.*
* **U4 characterization** — goal (1): does the adoption turn a baked constant or
  a blind spot into a **named, settable decision point**?

Ruled inadmissible in advance and not used: the census (fail-open), the
`c2rs subsys` agreement number (`#3845` — writable by prose), clause-table row
movements (this lane's own bookkeeping), and throughput.

**Read before probe** (`../WHITEBOX_LEVERAGE_2026-08-21.md`). Four of the five
rows were priced with **no compilation at all**; the one probe this lane ran
(§4.2) was taken only after the read was priced and found *structurally unable*
to answer, and it is four cells.

---

## 2. C7 — the ceiling VALUE, and the two error kinds are NOT one currency `[O]`

`work/w-emitprice/c7_price.py`, output `c7_price.out`. **A RE-READ**:
`work/w-sizebracket/series.jsonl` is 168 unique cells already compiled and
already graded against real `c2.dll`. Nothing was recompiled — `w-lowerband`'s
own move on the same file, asking a different question.

### 2.1 The finding that is on no page: the consequence FLIPS between the port's two seams

The port applies a size rule at **two** places with **opposite hazard**:

| seam | rule | FALSE-INLINE (says inline, c2 kept) | FALSE-KEEP (says keep, c2 inlined) |
|---|---|---|---|
| `splice.rs` **S7** — the ACCEPT region; the port **performs** the expansion | `body ≤ INLINE_UNBOUNDED_BYTES` (64) | **WRONG EMIT** | lost reach |
| `comdat::fenced_inlined_callee` — a **REFUSAL**; the port declines the TU | `body ≤ INLINE_DECLINE_BYTES` (128) | lost reach | **WRONG EMIT** |

`comdat.rs` states this in words — *"the hazard is inverted, and this is the one
place the size question is safe to get wrong"* — and this is the first thing to
put a number on it. **The two columns may never be summed and never traded**
(§5.2), which is why a single "how often is the rule right" score cannot price
C7 and why the row sat unpriced.

### 2.2 The numbers, at the workload's own `/O1`, over 82 cells

```
rule                                                    n  FALSE-INLINE  FALSE-KEEP
splice::INLINE_UNBOUNDED_BYTES   [ACCEPT seam]         82        0            5
comdat::INLINE_DECLINE_LOOP_BYTES [FENCE seam]         82        0            4
comdat::INLINE_DECLINE_BYTES     [FENCE seam]          82        3            0
C7 ADOPTED: 128 raw as a byte count                    82        3            0
C7 ADOPTED: 128 instrs x 4 B/PPC word                  82       24            0
C7 ADOPTED via a converter FITTED to (108,116]         82        0            0
[c2 unit] .gl SIZE < 128  (w-lowerband §6.7.1)         82        8            8
```

Three readings, and the third is the price:

1. **The incumbent pair is jointly wrong-emit-free on every `/O1` cell.** S7 at
   64 has **0 FALSE-INLINE** (its wrong-emit direction); the fence at 128 has
   **0 FALSE-KEEP** (its wrong-emit direction). Their whole standing cost is
   `5 + 3` cells of lost reach on a probe family.
2. **Adopting C7's value costs wrong emits at the accept seam** — 3 at `/O1`
   raw, **24** through the naive one-instruction-is-one-PPC-word conversion.
3. **The only translation scoring 0/0 at `/O1` is one whose converter is fitted
   to the very bracket the incumbent constants were fitted to** — and it costs
   12 at `/Ox`. So **C7's VALUE carries no information the port does not
   already have.** Only its UNIT would, and the unit is `P_INLINE` §6.6.1's
   second missing link, which §6.7 confirmed still stands.

### 2.3 The refusal's own cost, which is the half that decides

Counted, not asserted, and from a **standing** measurement rather than this
lane's instrument: `comdat.rs`'s own module doc, on the 878-TU workload —

> *"the coarse parser-shaped form costs **1,074** byte-exact functions; this one
> costs **0**."* (`work/w-inlfence2/crossing.md`)

and the deliberately-not-refused population beside it: *"1,081 byte-exact
functions call [a callee the port cannot lower], and every one of them is a
callee of 65–308 emitted bytes, which is the class c2 **keeps the call to**."*

**So the standing refusal's measured cost on the workload is zero byte-exact
functions, and the adoption's cost is 3–24 wrong emits.** This is the one row
where counting the refusal's own cost was *available* and it did **not** flip
the answer — `#1042`'s pattern does not always repeat, and saying so is part of
the price.

**Verdict: DECLINE C7, and it is priced NEGATIVE rather than unpriced.**
Consistent with brief §3's standing prohibition on adopting 128 (`#3732`) and
now carrying a number for it.

### 2.4 Controls (`#3336`)

`c7_price.py --controls`, watched before any verdict above was quoted. C2 and C3
are the degenerate thresholds (0 and ∞) and must produce errors on exactly one
side each — they do. **C4 re-derives `w-lowerband` §6.7.1's published `/O1`
figures — 8 kept-below and 8 inlined-above — EXACTLY, through an independent
implementation**, so the two instruments are demonstrably reading the same
population.

---

## 3. C9 — the favour-speed bit is **0** at `/O1`, and the port already holds the word `[R]` `[O]`

`work/w-emitprice/c9_bit23.py`, output `c9_bit23.out`. **No compilation.**

### 3.1 The question that was left open, in the words that left it open

`P_INLINE` §6.7.3:

> *"a datum nobody had read: the favour-speed bit's IMAGE value is `1` … and
> non-zero means C8's size test is **skipped**. `FUN_10b82338` writes it from
> bit 23 of a per-function option word … **so the default being ON does NOT
> license "therefore `/O1` clears it", and this page does not claim it.**"*

C9's `CLAUSES.tsv` gloss says *"the workload is pinned to `/O1`, so the bit is
single-valued"* — and **which** value it is single-valued at was never
established. That is the whole of what made the row unpriceable.

### 3.2 It needs no further disassembly `[R]`

The read is already taken and gives the formula:

```
10b82338:  mov eax,DWORD PTR [ecx+0x1c]     ; the per-function option word
10b8233c:  xor edi,edi
10b82356:  xor esi,esi / 10b8235d: inc esi  ; esi = 1
10b8236b:  cmp DWORD PTR ds:0x10c3de20,0x2  ; -> the DAT_10c3dddc arm  (DEAD: §6.8.6 measures 0)
10b8237a:  cmp DWORD PTR ds:0x10c2eaac,edi  ; -> same arm              (DEAD: 0 in raw .data)
10b8238b:  mov ecx,eax
10b8238d:  shr ecx,0x17                     ; >> 23
10b82390:  and ecx,esi
10b82392:  mov DWORD PTR ds:0x10c2e310,ecx  ; DAT_10c2e310 = (optword >> 23) & 1
```

Both alternate arms are dead on this workload by *measurements another lane
already published* (§6.8.6), so `0x10b82392` is the live store.

**The same function is the option word's whole fan-out**, and naming it costs
nothing extra: bit 8 → `DAT_10c2e314` (`0x10b82360`), bit 31 → `DAT_10c2e318`
(`0x10b82372`), bit 23 → `DAT_10c2e310` **or** `DAT_10c3dddc`, bit 18 then bit
12 → `DAT_10c2e30c` (`0x10b823ae`, `0x10b823bd`), and `[[…]+0x80]+0x76` ←
the whole word (`0x10b82352`, which is §6.7.2's S3 source). `[R]`, not pursued.

### 3.3 The VALUE comes out of `crates/`, not out of the image `[O]`

The port already parses that word. `c2_il::OPT_WORD_O1` and `OPT_WORD_OX` are
the only two `opt_mode_of_word` admits:

| const | value | bit 23 | C8's size test | c2's behaviour |
|---|---:|---:|---|---|
| `OPT_WORD_O1` | `0x00200005` | **0** | **RUNS** | favour size |
| `OPT_WORD_OX` | `0x00a00005` | **1** | **SKIPPED** | favour speed |
| `OPT_WORD_O1_NO_FP_CONTRACT` | `0x00200001` | 0 | RUNS | favour size |
| `OPT_WORD_OX_NO_FP_CONTRACT` | `0x00a00001` | 1 | SKIPPED | favour speed |

`c9_bit23.py` reads the constants **out of the crate source at run time**, so
the published answer cannot go stale if they move (brief §5's *"bind every
published count to a recipe"*).

> **The image supplies WHICH BIT; the port supplies WHAT IS IN IT.** That is
> C13's two-sides-meeting shape, and C13 is the only `[R]`-derived row on the
> 24 with that property. §6.2 item 4 says so of C13; it is now true of a second
> row.

### 3.4 What it explains, and the corroboration that came free

`P_INLINE` §2.1c reports an anomaly and offers a hypothesis:

> *"`/Ox` is where the units swap … **`/Ox` is not separating** — 320 B inlined
> beside 196 B kept … Consistent with §2.1's favour-speed bit turning this very
> test off."*

It was consistent; **it is now derived.** A size test that does not run is
exactly what a non-separating size bracket looks like.

**And §2's own table corroborates from the other side, on cells chosen for a
different question**: every candidate threshold accumulates **12–33** errors on
the 86 `/Ox` cells and **0–5** on the 82 `/O1` ones. A size rule fails on `/Ox`
because at `/Ox` there is no size rule.

### 3.5 The price

* **BUY, U1/U2: 0.** At `/O1` bit 23 is 0, so the arm is never taken and a
  `favor_speed` parameter defaulting to the read value reproduces the port's
  current behaviour exactly — **byte-neutral by construction, not by
  measurement**, which is precisely the argument that let C15 convert
  (`splice.rs:361`, *"`#pragma inline_depth` appears in 0 of the 100 hold-out
  TUs, so nothing moves it, which is why adopting C15 changes no emitted byte
  by construction"*).
* **BUY, U4: high.** It is a real settable decision point with a **real input
  already in the port**, which no other row on this page can say.
* **REFUSAL COST: 0.** The clause cannot fire at `/O1`, and no `/Ox` TU reaches
  the size seam at all — `comdat.rs`'s own statement of its precondition:
  *"`IlBundle::functions` hands a TU on only when every locally-defined callee
  has PLAIN EXTERNAL linkage … **and every segment is at `/O1`** (the mode the
  bound is measured at)"* (`crates/c2-core/src/comdat.rs:272`–`277`). Quoted as
  that module's claim about its own gate, not re-derived here.
* **Verdict: C9 is ADOPTABLE and its blocker is `none`, not `emit-change`.**
  A byte-neutral lane may take it. This is the row the partition recommends
  first, and it is the cheapest of the five.

---

## 4. C10 — the parameter with no input, and the reader is NECESSARY

### 4.1 The port already carries `forceinline` as a swept parameter `[R]` source

`w-clausegen` found it and the brief asked this lane not to lose it. Confirmed
and sharpened — the parameter is real, it is registered in the decision surface,
and **`port_enter_site` hard-codes `false` at `splice.rs:603`**:

```rust
// C15, in c2's own order: … The port cannot see that bit, so it passes
// `false` and takes the test.
if model.declines_at_maxlevel(next, false) {
```

**Under the amended goal — expose decision points as named, settable parameters
— C10 already IS one, with no input wired to it.** That is the peculiar state,
and it changes both the price and the classification.

### 4.2 The probe, and the read was priced FIRST and found unable to answer `[O]`

`work/w-emitprice/attr_twins.py`, output `attr_twins.out`.

`0x10b60a28` (`and eax,0x2000` / `jne 0x10b60a3c`) says c2 tests bit 13 and
accepts. `0x10b9bf70`/`0x10b9bf78` say `[sym+0x4c]` is the `.gl` `ATTR` word.
**Neither says what SETS bit 13**, and no amount of further disassembly can — it
is a fact about what the *front end* writes. So the read was priced, found
structurally unable to answer, and a four-cell probe was taken.

Four cells, one source, one keyword position, at the **workload's own flags**.
Three are **byte-length-identical** (`__forceinline ` = 14 chars = `inline` + 8
spaces = 14 spaces), so their records frame at the same displacements and any
difference is the attribute rather than the framing.

| cell | source form | TYPE flags | `SIZE` | **`ATTR`** | width | **low byte** |
|---|---|---:|---:|---:|---:|---:|
| `fi_no` | plain | `0x00` | 123 | **`0x1068`** | 2 | `0x68` |
| `fi_yes` | `__forceinline` | `0x20` | 123 | **`0x38c8`** | 2 | **`0xc8`** |
| `fi_inl` | `inline` | `0x20` | 123 | **`0x18c8`** | 2 | **`0xc8`** |
| `fi_noi` | `__declspec(noinline)` | `0x00` | 123 | `0x1009028` | **4** | `0x28` |

> #### `fi_yes XOR fi_inl = 0x2000` — EXACTLY, and it is C10's bit.
>
> `__forceinline` against plain `inline` is a **single-bit** difference at
> `[sym+0x4c]` bit 13. **C10's clause is `[O]` now and no longer only `[R]`.**

> #### AND THE LOW BYTES ARE IDENTICAL AT `0xc8`, WHICH IS THE HALF THAT PRICES THE ROW.
>
> `gl_function_attrs` returns a `u8`. So the port's **entire** view of `ATTR`
> **provably cannot separate `inline` from `__forceinline`.** A low-byte proxy
> is not merely fitted — it is **impossible**. The missing reader is
> **necessary**, not convenient, and that is a stronger statement than "absent".

`docs/STATUS.md`'s standing `plan-glattr` line is the same fact from the
instrument side: the bit histogram has **eight bins**,
`0:2 1:0 2:961 3:22913 4:0 5:305 6:22910 7:22186`. Bits 9, 13 and 19 are not
among them, and never have been.

### 4.3 Controls, and one went RED for real `[O]`

The first version of `attr_twins.py` anchored its framing walk on *"the first
`0x80`"* and stopped **inside** the fixed run `00 00 80 0a 10 00 00 00 00 80`,
reading an identical `0x5480` for all four cells. **All four controls fired and
the script refused to print a verdict** — which is what `#3336` is for, and it
is recorded rather than tidied away. Re-anchored on the run's invariant tail.

The controls that then passed:

* **C1** — `fi_no`'s `ATTR` reproduces `P_INLINE` §2.1d's published
  plain-function **`0x1068`** on a cell that lane never compiled. An
  independent reproduction of a published value is the strongest control here.
* **C2** — `__declspec(noinline)` crosses `0x8000` and takes the **four-byte**
  form, exactly as §2.1d predicts. **So the four-byte form is live on real
  code**, which is what makes C12's bit 19 reachable at all.
* **C3** — C13's *adopted* bit 6 behaves: set on plain, **clear** under
  `__declspec(noinline)`, reproducing `w-mmioclose`'s 9-of-9 / 11-of-11.
* **C4** — the three length-matched cells share a `SIZE` of 123.

### 4.4 The price

* **BUY, U1/U2: 0, for three independent reasons.**
  1. `declines_at_maxlevel` is `!forceinline && max_level != INLINE_MAXLEVEL_UNBOUNDED && …`,
     and `BUDGET_C2.max_level` **is** `INLINE_MAXLEVEL_UNBOUNDED`, so the
     predicate is identically `false` whatever `forceinline` holds. Wiring a
     real input moves **no** verdict at the default model — byte-neutral **by
     construction**, C15's argument again.
  2. `BudgetModel::charge` and `::seed` have **no production caller at all**;
     they are reached only from `surface_rows()` and tests. `splice.rs`'s own
     `cargo test` at line 2179 enforces that `port_enter_site` acquires no
     consumer outside the module.
  3. **The population is refused upstream.** `IlBundle::functions` admits only
     callees whose `.gl` TYPE flags byte is exactly `0x00`, and this lane's own
     cells measure `__forceinline` at **`0x20`** — so every `__forceinline`
     callee is already out of the port's admitted set before the inline seam is
     reached. `[O]`.
* **BUY, U4: high** — it closes the gap between a registered decision point and
  a real input, which is the amended goal's own shape.
* **REFUSAL COST: 0**, by reason (3), and it is measured rather than argued.
* **Verdict: C10 is NOT an emit change.** Its blocker is the missing
  `ATTR`-continuation reader. **Reclassify.**

---

## 5. C11 and C12 — one predicate, not two, and the price is honestly UNKNOWN

### 5.1 The legality function, read end to end `[R]`

`FUN_10b5c06b`, 60 bytes, complete:

```
10b5c06b:  8b 41 20              mov    eax,DWORD PTR [ecx+0x20]
10b5c06e:  a9 00 04 00 00        test   eax,0x400        ; C11
10b5c073:  75 2f                 jne    0x10b5c0a4       ; -> REFUSE
10b5c075:  8b 49 4c              mov    ecx,DWORD PTR [ecx+0x4c]
10b5c078:  f7 c1 00 00 08 00     test   ecx,0x80000      ; C12
10b5c07e:  75 24                 jne    0x10b5c0a4
10b5c080:  a9 00 10 00 00        test   eax,0x1000       ; C11
10b5c085:  75 1d                 jne    0x10b5c0a4
10b5c087:  f7 c1 00 02 00 00     test   ecx,0x200        ; C12
10b5c08d:  75 15                 jne    0x10b5c0a4
10b5c08f:  a8 40                 test   al,0x40          ; C11  <- `al`, the LOW BYTE
10b5c091:  75 11                 jne    0x10b5c0a4
10b5c093:  a9 00 01 00 00        test   eax,0x100        ; C11
10b5c098:  75 0a                 jne    0x10b5c0a4
10b5c09a:  0f b6 c1              movzx  eax,cl           ; C13 — the LOW BYTE of +0x4c
10b5c09d:  c1 e8 06              shr    eax,0x6
10b5c0a0:  83 e0 01              and    eax,0x1
10b5c0a3:  c3                    ret                     ; = bit 6, the ACCEPT
10b5c0a4:  33 c0                 xor    eax,eax
10b5c0a6:  c3                    ret                     ; the single REFUSE sink
```

**Two things neither clause row records, and the first changes how they must be
priced:**

1. **The six refusal tests are INTERLEAVED and all six branch to ONE sink.**
   `+0x20`, `+0x4c`, `+0x20`, `+0x4c`, `+0x20`, `+0x20`. So the order is
   unobservable and the six are commutative in effect: **C11 and C12 are one
   predicate split across two `CLAUSES.tsv` rows by field, not two clauses.**
   Neither can be adopted, priced or exercised without the other, and any port
   counterpart is one function. This is a bookkeeping split, not a structure
   of c2.
2. **C13 — the one adopted row — is in this same function and reads the same
   dword as C12.** `movzx eax,cl` takes the **low byte** of `[sym+0x4c]`. So
   C13 converted and C12 did not for a reason that is neither difficulty nor
   attention: **field WIDTH.** C13's bit is bit 6; C12's are bits 9 and 19.

### 5.2 `[sym+0x20]` IS an IL field — P4 is a MISS `[R]`

The prereg predicted (P4) that C11 is not derivable because `[sym+0x20]` is a
back-end symbol-arena word. **Falsified, in exactly the way P4 named.** In the
`.gl`/symrec reader `FUN_10b9b8e9`:

```
10b9be5b:  call 0x10c1f9e9                  ; i32c
10b9be60:  mov  DWORD PTR [esi+0x40],eax
10b9be63:  call 0x10c1f91b                  ; <- the varint reader
10b9be68:  mov  DWORD PTR [esi+0x20],eax    ; <- [sym+0x20], VERBATIM FROM THE IL
10b9be6b:  test eax,0x200                   ; and bit 9 of it gates an extra field
...
10b9bf46:  cmp  DWORD PTR [ebp-0x78],0xe    ; the function-record arm
10b9bf6c:  mov  WORD PTR [esi+0x50],ax      ; C24's SIZE
10b9bf70:  call 0x10c1f91b                  ; the SAME reader
10b9bf75:  and  eax,0xfffffffb              ; <- bit 2 FORCE-CLEARED
10b9bf78:  mov  DWORD PTR [esi+0x4c],eax    ; ATTR
```

`0x10c1f91b` is the same reader on both sides, and the path between the two
stores is **straight-line**: `0x10b9befc`'s `je` and `0x10b9bf0a`'s `jle` both
fall *into* the kind-`0xe` test. So `[sym+0x20]` arrives from the IL, one
region earlier in the same record as `ATTR`.

> **A correction to any claim that `[sym+0x4c]` is `ATTR` verbatim:
> `and eax,0xfffffffb` at `0x10b9bf75` force-clears bit 2 before the store.**
> It touches none of C10's, C12's or C13's bits, so no price moves — but
> `FUNCS.tsv`'s own label for this function already says *"`varU` → `+0x4c`
> **WITHOUT** the `0x4` force-clear"* of a *different* arm, so there are two
> arms and they disagree about this bit. Recorded, not pursued. **C24's own
> clause is about `WORD [sym+0x50]` and is untouched** — that store *is*
> verbatim.

### 5.3 …and no enumerated pass sets any of C11's four bits `[R]`

`work/w-emitprice/f20.py`, output `f20.out`. `#3505` is six for six on *"no
writer exists"* being a claim about an instrument's index, so the write
**classes** are enumerated rather than one spelling grepped, and the classes
that cannot be decided are **printed rather than assumed empty**.

```
operands at +0x20, decoded                            : 2782
   ...inside a Ghidra function extent                 : 2719
   ...WRITE mnemonic AND memory operand is destination :  497
   ...in a function CORROBORATED on the symbol record  :    5
```

The five, with `w-instrcount`'s own corroboration filter (≥ 3 of
`+0x37 +0x4c +0x50 +0x52 +0x54 +0x58`, ≥ 1 of the two that are specific):

| addr | instruction | owner | what |
|---|---|---|---|
| `0x10b9be68` | `mov [esi+0x20],eax` | `FUN_10b9b8e9` | **the IL-verbatim initialiser** |
| `0x10b9bf96` | `or [esi+0x20],eax` | `FUN_10b9b8e9` | `eax = (WORD[[0x10c472e8]+0xcda] & 1) << 5` — **bit 5** |
| `0x10b9b284` | `or [esi+0x20],0x2000` | `FUN_10b9b161` | bit 13 — `DISCLOSURE W-ALIAS-2`'s alias-target Mark bit |
| `0x10b8290a` | `mov [esi+0x20],edi` | `FUN_10b8289c` | register-sourced, **OPAQUE** |
| `0x10b516f5` | `mov [esp+0x20],eax` | `FUN_10b50c43` | **a visible FALSE POSITIVE** — a stack slot; matching a displacement on any base cannot exclude it |

**No enumerated write sets `0x40`, `0x100`, `0x400` or `0x1000`.** The only
immediate is `0x2000`, which is none of the four.

**What this instrument cannot decide, said rather than left to be found:** four
OPAQUE register-sourced writes; 60 `lea …,[reg+0x20]` advanced-base sites; and
block copies — which, **unlike** the `+0x50` case `w-instrcount` closed, are
**not** empty at this displacement, since a `0x14`-dword heap copy spans it. So
the result is *"no enumerated pass sets them"* and **not** *"the IL value
survives"*.

**A negative result recorded rather than tidied**: the first version of this
script reported **2,006** writers, because a mnemonic-only filter counts
`mov eax,DWORD PTR [ecx+0x20]` as a write. Control **C1b** now watches the
destination test reject **1,509** such rows on every run.

### 5.4 The price, and it is honestly UNKNOWN

Both rows are **refuse-side** clauses. Adopting either into the port can only
*narrow* the accept set, so:

* **U1/U2 ≤ 0.** It converts nothing and can only withdraw.
* **U3 ≥ 0.** It can only close a hole — if the port's admitted set contains a
  function c2's legality check refuses, adopting the clause closes a live wrong
  emit, which is the highest-value outcome this repo recognises.
* **Both are the SAME unmeasured population**, and it is unmeasurable today for
  the reason §0 gives: nothing in `crates/` decodes either field wide enough.

**The only bound that exists is this lane's, and it is small and honest:** over
the **6** full-width `ATTR` values anyone has ever read — this lane's four cells
plus `P_INLINE` §2.1d's `0x1068` and `0x801028` — **0 carry `0x200` and 0 carry
`0x80000`.** For C11 there is **no** bound at all: not one `[sym+0x20]` value
has ever been read out of real IL by anything in this repo.

> **The honest price for C11 and C12 is: *unknown until a reader exists that
> exercises them*, and the reader is the same one C10 needs.** That is a real
> finding and is published in those words rather than as a manufactured number,
> under the brief's own licence. **Its actionable content is that the three
> rows should be bought together or not at all**, because one reader
> discharges all three, and buying it for C10 alone leaves two-thirds of its
> value on the table.

---

## 6. The artifact check, run and reported — including that it FIRED

`PREREG` §5 registered five ways this lane's ranking could be an artifact,
before the numbers existed. All five were run.

| | check | result |
|---|---|---|
| **A1** | modal price class ≥ 4 of 5 ⇒ declare a **PARTITION**, not a ranking | **FIRED.** C9, C10, C11 and C12 all price at **0** in U1/U2; only C7 differs, and it is negative. The deliverable is published as a partition into three classes and **no order is asserted over the four**. `#3505` is **seven for seven**. |
| **A2** | every row must carry ≥ 1 count from a **standing** instrument | **PASS.** C7 ← `comdat.rs`'s `1,074 / 0` workload measurement and `w-sizebracket`'s committed cells; C9 ← the crate's own `OPT_WORD_*` constants; C10/C11/C12 ← `docs/STATUS.md`'s `plan-glattr` eight-bin histogram. No row's price rests only on a script written here. |
| **A3** | is the order really "which rows the workload exercises"? | **PASS — `exercised` is not carrying it.** C10 is `exercised = yes` and sits in the *same* class as C11 and C12, which are `no`; C7 is `exercised = yes` and is the *only* row in its own class. Recomputing the partition with the `exercised` term dropped gives the identical three classes. |
| **A4** | nothing ranked by a prose-writable quantity (`#3845`) | **PASS.** No `c2rs subsys` number appears anywhere in this pricing. |
| **A5** | 5-of-5 confirmed predictions ⇒ re-examine, do not celebrate | **NOT FIRED — 3 of 5, with two real misses.** §7. |

---

## 7. The predictions, scored — two MISSES, and both were productive

| | prediction | outcome |
|---|---|---|
| **P1** | C7's price is NEGATIVE — adopting 128 costs wrong emits in both directions and buys 0 | **CONFIRMED**, and now with a number: 3 wrong emits raw, 24 naive, 0 buy. The "both directions" half is refined: the errors are one-directional per seam and their *consequence* is what flips. |
| **P2** | C9's price is UNKNOWN and the blocker is a READ | **MISS.** The read was already taken; what was missing was joining `0x10b8238d`'s formula to the port's own `OPT_WORD_*` constants. C9 is **settled and adoptable**, not unknown. Falsified on P2's own stated terms — *"locating a committed read that settles it"*. |
| **P3** | C10 is misclassified; wiring the reader is byte-neutral by construction | **CONFIRMED**, and by three independent mechanisms rather than the one predicted, the third (`[O]`, upstream refusal at TYPE flags `0x20`) measured by this lane. |
| **P4** | C11 is NOT derivable — `[sym+0x20]` is a pass-computed word | **MISS.** It is IL-borne, `0x10b9be68`, from the same reader as `ATTR`. The corrected finding is *sharper* than the prediction: C11 joins C10 and C12 behind one missing reader instead of being a dead end. |
| **P5** | C12 is warranty-shaped, not conversion-shaped | **CONFIRMED, with an amendment**: it is warranty-shaped *and* its population is unmeasurable, so the price is UNKNOWN rather than merely small — and C11/C12 turned out to be **one predicate**, which P5 did not anticipate. |

**Both misses came from predicting that a link was missing when it was
present** — P2's read and P4's IL provenance — and in both cases the correction
made the row *cheaper*, not dearer. That is the same direction `#3846` names:
a page's number read as the port's state.

---

## 8. Found and not taken

Ranked, with the frame axis applied. **This lane adopts none of them.**

1. **The `.gl` symbol-record decoder** — the one missing link behind C10, C11
   and C12. Its shape is now known on both ends: `ATTR`'s continuation
   (`0x10c1f91b`, two-or-four bytes, bit 15 the flag — `[O]` on this lane's
   four cells) and the `+0x40`/`+0x20` region that precedes `gl_function_attrs`'
   framing anchor. `gl.rs:1520` already names the first half as owed.
   **Adopting it is a reader, not an emit change**, and the three clause rows
   follow from it.
2. **`FUN_10b82338` is the option word's whole fan-out** (§3.2). Five globals
   from one per-function word, of which this repo has named exactly one. Two of
   them — `DAT_10c2e30c` (bit 18, then bit 12) and `DAT_10c2e314` (bit 8) —
   appear in no page here.
3. **`[sym+0x20]` bit 9 gates an extra record field** (`0x10b9be6b`). Any real
   `.gl` decoder must handle it, and no page here names it. **Not** C12's
   `0x200`, which is on `+0x4c` — same bit number, different field, and the
   collision is worth a sentence so nobody conflates them.
4. **`[sym+0x4c]` bit 12 gates a sub-record decode** (`0x10b9bf99`), and it is
   **set on all four** of this lane's cells — so it is live on ordinary code and
   any decoder that ignores it will desynchronise.
5. **`and eax,0xfffffffb` at `0x10b9bf75`** — `ATTR`'s bit 2 is force-cleared on
   this arm and, per `FUNCS.tsv`'s own label, *not* on another. Two arms
   disagreeing about a bit of a field three clauses test is worth one read.
6. **`fi_noi`'s `ATTR` is `0x1009028`, not §2.1d's `0x801028`.** Different
   function, so not a contradiction — but the four-byte payload's **high**
   half (`0x0100`) is nonzero here and was zero there, and nothing in this repo
   reads bits 16–31 of `ATTR` at all. C12's `0x80000` is bit 19, i.e. **in that
   half**.

---

## 9. Gate evidence

Predicted reach **0** and byte delta **0**, both stated up front and both held:
this lane wrote **zero `crates/` bytes**. See
[`../rungs/2026-08-29-w-emitprice.md`](../rungs/2026-08-29-w-emitprice.md) for
the counts.

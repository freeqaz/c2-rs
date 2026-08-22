# WB_LABELCHARGE — PREREG for read R3 (the label charge)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

**Lane:** `w-read-r3` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered.

**Subject.** Read **R3** of the funded read-plan
(`docs/whitebox/READ_PLAN_2026-08-21.md` §3 row R3, §4, §5.1; funded by the
owner 2026-08-22 — `docs/DECISIONS_2026-08-22.md` decision 1): **the label
charge**. Enumerate the call sites of the "take a number" allocator
`FUN_10b97dd0` and of the generic label constructor `FUN_10b9a455`, read the
name formatter `FUN_10b99dfe`'s switch, and state the charge on
`DAT_10c2edd0` as a **property of the code** rather than of a differencing
experiment.

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` —
**verified by this lane before any address was read** (`C2_MAP_METHOD.md` §0),
and re-verified against `~/ghidra-projects/bin/c2dll`, the flat export's
input, which matches. The export is dated 2026-08-04; its input digest
matching the pinned image is what licenses quoting its addresses
(`READ_PLAN` §5.4). In this worktree `compilers/` is a symlink to the main
checkout's, so the digest is the same bytes on both paths.

---

## 0. THE TRAP THIS LANE IS DEFINED AGAINST

`docs/LABEL_COUNTER.md:3-18`'s own banner: **four consecutive lanes measured
this subject wrong**, and every one of their numbers was a real reading of a
different quantity. The counterfactual form `[subject, control]` vs
`[leaf, control]` measures **`Δseed + Δcharge`**; the seed is a function of the
source text because `c1xx` and c2 share one id space.

* `w-bdnz`'s **+7** reproduces to the digit with a **true charge of +2**.
* Eight **unused declarations**, emitting not one instruction, move the
  counterfactual by **+16** while the true charge stays **1**.

Board **#3368** struck a roadmap ground for citing those measurements.

> **THE CONSEQUENCE FOR THIS LANE, REGISTERED HERE.** A stride obtained by
> perturbing source and diffing label numbers is **not** the charge, and this
> lane may not produce one. The charge is *"how many times does c2 execute the
> instruction at `0x10b97de5`"*, and the read is what settles it. **Every
> number this lane publishes as a charge must be traceable to an enumerated
> call site**, or to an in-TU stride taken in `LABEL_COUNTER.md` §7.6's
> subject-in-the-middle form with `base` measured in the same obj — never to a
> whole-TU displacement. Where this lane's finding disagrees with a number in
> `LABEL_COUNTER.md`, the findings doc says **which of the two is a
> counterfactual and which is a charge**, in those words.

---

## 1. Prior art this lane must NOT re-derive

Checked before writing this file (`docs/`, `scripts/`, `crates/` and
`docs/BOARD.md` all grepped for the six addresses). This subject has more
prior art than any other in the repo. Already held:

| already known | where |
|---|---|
| the counter is `DAT_10c2edd0`, one 32-bit TU-global; the **sole increment** is `inc DWORD PTR ds:0x10c2edd0` at **`0x10b97de5`**; the 28-byte allocator `FUN_10b97dd0` faults with internal error `0x37` if the counter is still 0 | `WB_LABEL_FINDINGS.md` §1.1 (2026-08-09, lane `wb-label`, board #2430–#2459) |
| `FUN_10b97dd0` has **31 direct call sites**; `FUN_10b9a455` (the generic label constructor) is one of them and is itself called **132 times from 86 functions** | ibid.; repeated at `READ_PLAN` §1, `coff/label.rs:1-26` |
| `FUN_10b9a455`'s body: `kind 3` object, `+0x31 = 0x20`, `puVar1[10] = FUN_10b97dd0()` (i.e. `sym[+0x28]`), then `*(int*)(puVar1+0x3f) = DAT_10c2e918++` | ibid. §1.3 |
| the IL reader's case 3 takes `sym[+0x28]` from the stream via `FUN_10c1f91b` and charges **nothing** | ibid. §1.3 |
| the two seed installs add **no constant**: IL directive `0x16` sets the counter from the stream; per-TU setup sets it to `max(IL value, current)` | ibid. §1.3 |
| the formatter `FUN_10b99dfe` (682 B) reads `sym[+0x30]`/`+0x31`/`+0x43` and never increments; `$L*` numbers come from **`sym[+0x3f]`**, filled from the second per-function counter `DAT_10c2e918`, reset to 1 in `FUN_10b7e113` | ibid. §1.2 |
| the downward end `DAT_10c2ed40` with the crossing check | ibid. §1.1 |
| `LABEL_SEED_GAP = 9` is **nine allocations**, not an offset (P8.4 refuted, P8.1 supported, **count open**); `FUN_10b9a4a7` is *one* of the kinds that does it | ibid. §1.4, §6 open #1 |

**What is therefore NEW and is this lane's numerator:** the *enumeration* — one
row per call site with its caller, its guard, and its charge — plus the
closure argument, plus whatever the enumeration explains or fails to explain
about the measured surcharge table.

**Inherited-not-re-read.** The eight facts above are inherited. They are
excluded from this lane's `sites-read` numerator unless independently
re-derived; where this lane re-derives one it says so and reports any
disagreement as a finding about the predecessor, not a footnote.

**Dispatch defect check, registered before the work.** The brief's figures
("31 sites of `FUN_10b97dd0`", "132 of `FUN_10b9a455`", "`0x10b97de5`") are a
**survey's**, not the coordinator's own verification, and every address list
this wave supplied to a lane has needed correction by the lane that used it.
They are treated as a **starting hypothesis**; P1.4/P1.5 below score them, and
any that is wrong is reported as a dispatch defect in the rung.

---

## 2. The grading rule

Registered **before** any call site, xref set or arm body was read. Tier
**PREREG** by [`PREREG.md`](PREREG.md)'s ladder: committed to git before the
answer existed anywhere in this lane. Each prediction below is scored
HIT / MISS / UNGRADED in `WB_LABELCHARGE_FINDINGS.md`; **misses are reported as
misses and are not smoothed**, and a prediction vague enough to be
unfalsifiable earns nothing and is marked UNGRADED rather than counted.

**Denominator: `sites-read/163`**, where 163 = 31 + 132 is the survey's
figure. If the true counts differ, the denominator is restated and the
survey's is reported as wrong. **Honest partial coverage beats a claimed
total**: a claim of 163/163 reached by skimming is worth less than 60/163
where each row names a guard.

---

## P1 — CLOSURE: is the mechanism closed by construction?

This is the load-bearing claim. `READ_PLAN` §3 row R3 asserts the rule is
*"closed by construction — one increment instruction"*. That is an assertion
about the code and it is falsifiable.

| # | prediction | grade if |
|---|---|---|
| **P1.1** | **Every write reference to `DAT_10c2edd0` in the image** is one of: (a) the increment at `0x10b97de5`, or (b) a **seed install** that assigns the counter a value from outside (the IL stream, or `max(IL, current)`). **No third kind of write exists** — no `add [mem], k` for `k > 1`, no decrement, no per-function reset. | HIT if the enumerated write set partitions into (a) and (b) with nothing left over; **MISS if any third write exists**, and that MISS is the more valuable finding |
| **P1.2** | `FUN_10b97dd0` has **exactly one** reachable path that reaches `0x10b97de5`, and it charges **exactly +1 per call that returns**. There is no early-return path that yields a number without incrementing, and no path that increments twice. | HIT / MISS on the read body |
| **P1.3** | **No call site of `FUN_10b97dd0` lies on a loop back edge** in its caller, so each static site charges 0 or 1 per invocation of its caller — never an unbounded amount. | HIT at 0 loop-resident sites; **any loop-resident site is a MISS and is a finding**: it means the per-caller charge is data-dependent and the site table alone cannot price a TU |
| **P1.4** | The **31** direct call sites of `FUN_10b97dd0` reproduces exactly. | HIT / MISS with the true count |
| **P1.5** | The **132** call sites / **86** distinct callers of `FUN_10b9a455` reproduces exactly. | HIT / MISS with the true counts |
| **P1.6** | There is **no indirect route** to `FUN_10b97dd0` — its address is never taken into a data table, a vtable, or a callback slot. (This is the hole that would make an enumeration of *direct* sites not a closure.) | HIT if the only references are `call` instructions; **MISS if the address appears as data**, which would void the word "closed" |

> **P1.6 is the prediction that can void the headline.** "Closed by
> construction" survives an indirect route only if the route is itself
> enumerable. Registered because the enumeration is worthless without it, and
> because `C2_MAP_METHOD.md` §7's standing lesson is that the path you read may
> not be the path the inputs take.

## P2 — THE ENUMERATION: what does each site charge, and under what guard?

| # | prediction | grade if |
|---|---|---|
| **P2.1** | **≥ 20 of the 31** allocator sites are read to the level of *(caller, guard condition, object kind being constructed)* — not merely listed with a caller name. | HIT at ≥ 20 rows carrying all three fields, MISS below |
| **P2.2** | The sites **partition by object kind**: each site is inside a constructor/initializer for one identifiable kind of c2 object (label, section, symbol, EH record, …), and **the kind is what predicts the charge**, not the calling construct. Falsifiable: a site whose charge depends on a *count* (a loop bound, an operand count) rather than on a kind. | HIT / MISS |
| **P2.3** | `FUN_10b9a455` is the **single busiest** of the 31 sites by dynamic frequency, and the remaining 30 together account for a **minority** of a typical TU's charge. Graded structurally (how many of the 30 are once-per-TU / once-per-section initializers rather than per-construct), since the read cannot count dynamic executions. | HIT / MISS, reported as a structural count with the limit named |
| **P2.4** | **At least one** of the 31 sites is guarded by a *first-time* flag — a "have I minted this yet" test — which is the mechanism `LABEL_COUNTER.md` §1.1's dedup rows (`_fltused` once per TU; a helper width or FP constant *an earlier function already introduced* costs **0**) require to exist. | HIT if such a guard is found and named with its flag's address; MISS if the dedup has no such mechanism, which would mean the dedup lives somewhere this read did not look |

## P3 — THE SURCHARGE TABLE, RE-DERIVED FROM THE SITES

`LABEL_COUNTER.md` §1.1 is seven measured rows. It was **fitted from objs**.
If the mechanism is closed, the site enumeration should *explain* it — and
where it cannot, that is the honest boundary of the read.

The seven rows: `_fltused` **+1** once per TU · `__savegprlr_N`/`__restgprlr_N`
**+2** per distinct N · `__savefpr_M`/`__restfpr_M` **+2** per distinct M · a
newly pooled FP constant **+2** per distinct `(bits,width)` · a signed
`>`/`<` over two call results **+2** · a callee external the IL names **0** at
any count · a helper/constant an earlier function introduced **0**.

| # | prediction | grade if |
|---|---|---|
| **P3.1** | **≥ 5 of the 7 rows** are explained by the enumerated sites — i.e. the row's integer equals a count of identified charging sites executed for that construct, with the guard naming why. | HIT at ≥ 5, MISS below |
| **P3.2** | The **two zero rows** are explained by *absence of a site on that path*, not by a subtraction: an IL-named callee external and a re-used helper reach no call to `FUN_10b97dd0` at all. | HIT / MISS |
| **P3.3** | The **`+2` for a signed `>`/`<` over two call results** (the one surcharge in the table that **mints no symbol at all** — `LABEL_COUNTER.md` §1.1's own note) is explained by two `FUN_10b9a455` calls on the materialisation path. **This is the hardest row** and it is registered as the one most likely to miss. | HIT / MISS |
| **P3.4** | **`LABEL_SEED_GAP = 9`**: **≥ 4 of the nine** allocations are named by call site with the object each constructs. Full enumeration of all nine is **not** predicted — `WB_LABEL_FINDINGS.md` §6 open #1 has stood since 2026-08-09 and a lane that predicts it will close is predicting a hope. | HIT at ≥ 4 named, MISS below, and the count reported either way |
| **P3.5** | **The `/Gy` `+3 per function`** is explained by three identified charging sites on the COMDAT path. Registered separately from P3.4 because it is a *different* fitted constant in the same file (`coff/label.rs`). | HIT / MISS |

## P4 — THE FORMATTER CHARGES NOTHING

| # | prediction | grade if |
|---|---|---|
| **P4.1** | `FUN_10b99dfe` and its entire call subtree contain **zero** calls to `FUN_10b97dd0`. **Naming a label never charges the counter.** | HIT at 0; MISS at ≥ 1, which would mean the charge depends on whether a label is ever *printed* |
| **P4.2** | The formatter's selection is a **pure function of `sym[+0x30]`, `sym[+0x31]`, `sym[+0x43]` and `sym[+0x4d]`** — no global state, no counter read other than `sym[+0x28]`/`sym[+0x3f]`. | HIT / MISS with any global it does read |
| **P4.3** | `$LC`/`$LL`/`$LN` are selected by **bits** of `sym[+0x43]` (`0x10` → `$LC`, `0x4` → `$LL`, neither → `$LN`) — `WB_LABEL_FINDINGS.md` §1.2 re-derived independently here rather than inherited. | HIT / MISS |

## P5 — THE CONTROL, SPECIFIED BEFORE IT IS RUN

`READ_PLAN` §5.3 and `ref/README.md:49`: **`[R]` says "the instructions were
read correctly", not "this is what c2 does."** The `.bss` bump rule was read
correctly and was wrong about c2. So this lane ends in a probe against real
c2 output, and the probe is fixed here so it cannot be tuned afterwards.

### 5.1 The shape requirement — what makes a control capable of failing

**The twelfth absence-read-as-success in this repo was a control on a body
that could not have moved.** A control on a function with **one** compiler
label, or with **none**, is structurally incapable of detecting a misread
charge. So the probe set is fixed here to require **all** of:

1. a TU whose subject function's true in-TU stride is **≥ 3** — i.e. it
   charges more than the framed base, so a charge error can show;
2. at least **two** functions after the subject in `.text` order, so a wrong
   charge propagates into *their* `$M` numbers and is visible as a
   displacement rather than being absorbed at the end of the TU
   (`LABEL_COUNTER.md` §7.6 step 5: *a wrong charge on the LAST function moves
   nothing*);
3. at least one **dedup** cell — the same helper width or pooled constant
   introduced twice — because P2.4's first-time guard is exactly what a naive
   site-count would get wrong (it would charge twice);
4. at least one cell where the **counterfactual and the true charge disagree
   by a known amount**, so the instrument itself is under test in the same
   run. `s_loc8` (8 unused locals: counterfactual **+8**, true charge **0**)
   is the registered cell.

### 5.2 The predictions

| # | prediction | grade if |
|---|---|---|
| **P5.1** | On a probe set meeting (1)–(4), the **absolute** `$M`/`$T` numbers of every function in the TU are predicted exactly from `seed + LABEL_SEED_GAP + 3·nfuncs + Σ(per-function consumption)`, with each consumption term traced to enumerated sites. **≥ 90 %** of predicted symbol numbers land exactly. | HIT at ≥ 90 %, MISS below, with the residuals named individually |
| **P5.2** | Cell (4) reproduces the banner: the counterfactual reading of `s_loc8` is **+8** and its true in-TU stride is **1**. If it does not, this lane's instrument is wrong and every number in this document is void. | HIT / MISS — **this is the instrument's own self-test, and a MISS voids the lane** |
| **P5.3** | Cell (3)'s dedup: the second introduction of the same helper width costs **0**, matching `LABEL_COUNTER.md` §1.1's last row, and the read names the flag that makes it 0. | HIT / MISS |
| **P5.4** | **The `LABEL_SEED_GAP = 9` invariance test.** The gap is a *fitted* constant in `crates/c2-core/src/coff/label.rs:9`, and whether it moves for a TU with different section needs is recorded as **unvaried** since 2026-08-09 (`WB_LABEL_FINDINGS.md` §6 open #1). Registered prediction: **the 9 MOVES** on at least one of {a defined initialized global, an uninitialized global, a string literal, `/GF`}. | HIT if any cell reads ≠ 9; **MISS if all cells read 9**, which is a positive result for the shipped constant and is reported as such |

> **P5.4 is registered against the port's own interest and can embarrass it.**
> If the gap moves, `LABEL_SEED_GAP = 9` is fitted to the fixture population
> and there is a **latent** wrong-emit fenced only by what the port currently
> refuses. That is a finding worth more than the read, and it is why the cell
> is in the probe set rather than left to a later lane.

### 5.3 What the control could NOT catch — stated in advance

* It cannot see a site that **no probe in the set executes**. The enumeration
  is static; the probe exercises a handful of paths. A site guarded by a
  condition none of these TUs meets is read `[R]` and stays `[R]`.
* It cannot distinguish **two sites that always fire together** (e.g. a
  constructor that always takes exactly two numbers vs. two constructors each
  taking one). Any such pair is reported as a pair, not resolved.
* It cannot settle **ORDER**. `READ_PLAN` §4's spec-shape for R3 says so in
  the spec's own voice, and this lane repeats it: **this read gives the
  *charge*, not the *order*.** A charge rule without an order rule still
  cannot place a label; the other half is **R8** (block emission order,
  5–10 d, `CEILING` §6.1 phase 1, the one UNSERVED phase, and the only read
  with no known address for its rule).
* It cannot see anything about **`/Ox`**'s loop charge, which
  `LABEL_COUNTER.md` §7.7 open #3 leaves at four magnitudes and no rule. This
  lane does not propose one.

## P6 — THE `/FAsc` LISTING SEAM IS ALREADY CLOSED FOR THIS PURPOSE

The brief offers the listing as a control route. **It is closed by
measurement**, and this lane registers that it will not re-open it:
`LABEL_COUNTER.md` §7.3 / `WB_LABEL_FINDINGS.md` §3.3 measured, at the byte
level, two bodies with **identical 24 `.text` bytes** and **different charges
(2 and 4)** printing **the same label name at the same index**. `$LN`/`$LL`/
`$LC` come from the *second* counter `DAT_10c2e918`, which counts label
*objects* including the free ones from the IL, so `stride ≥ max($LN)` fails on
the first row. `docs/rungs/README.md`'s separate warning about the seam
(canonical unrelocated words, symbols named where the obj carries
displacements) is a second reason.

| # | prediction | grade if |
|---|---|---|
| **P6.1** | The listing remains closed as a route to the **charge**, and the read **explains why** rather than merely restating the measurement: the formatter reads `sym[+0x3f]` (object ordinal) where the charge is `sym[+0x28]` (global id), and the two counters are incremented at different places. | HIT if the read names both increment sites and the field split; MISS otherwise |
| **P6.2** | The listing **is** a sound instrument for a *different* question this lane may answer for free: which labels c2 **invented** vs. which arrived pre-numbered in the IL, discriminable because a front-end id is *below* the TU's seed. (`$top$2561` below the TU's first c2 label 2613 — `WB_LABEL_FINDINGS.md` §1.3.) | HIT / MISS / UNGRADED if not attempted |

---

## What this lane will NOT claim

- **No `crates/` change of any kind.** Docs-only. No emit rule, no refusal
  predicate, no fixture, no byte. If a `crates/` comment is found stale by
  this read it is **reported in the rung, not edited** (R1's precedent:
  it found one in `alloc.rs` and left it).
- **No `DISCLOSURE.md` row is owed** unless something is adopted, and nothing
  will be. Explaining a black-box-fitted constant incurs no debt; *replacing*
  it with a disassembly-derived one does (`WB_LABEL_FINDINGS.md` §8's own
  tiering).
- **No order rule.** P5.3 above.
- **No new stride measured in the counterfactual form**, at all, for any
  purpose.
- **No re-pricing of R8 from this read.** A read produces a spec, not an
  implementation (`DECISIONS_2026-08-22.md` decision 1's own warning), and
  #1767's rule against extrapolating a 3-cell measurement applies here too.

## Registered outcome shape

`built` **only if** the enumeration lands with a non-trivial `sites-read/163`
numerator **and** the control of §5 is run in a provisioned environment and
either goes green or goes red with the red resolved or reported. If **P5.2**
(the instrument self-test) misses, the outcome word is **FAILED**, in those
words, whatever else the lane produced — a lane whose instrument is not the
one it claims has measured nothing.

If the closure question (P1) comes back **negative** — an indirect route, a
third write, a loop-resident site — that is **not** a FAILED lane. `READ_PLAN`
§3's "closed by construction" is a premise this read exists to check, and
refuting it is the more valuable outcome. The rung says so plainly either way.

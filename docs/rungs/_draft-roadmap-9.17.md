DRAFT for `docs/ROADMAP.md` §9.17 — written by lane `w-arms`, to be landed by the
coordinator. Nothing in §1–§9.16 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-arms-prereg.md`, committed at `2db819c` before the
first scan.

---

### 9.17 W-ARMS — the largest site on the emitted board is 8 % assignments, and the biggest blocker on it is worth 6 (2026-08-01)

Boards **#142** (the clean-not-whole receiver arms) and **#143**
(`…recv-load-then-off-add-more`). **No rung.** Both were measured and both
declined, and the two measurements are worth more than a rung of this size would
have been:

* **#142 is decomposed for the first time**, into 27 named receiver constructs
  in emitted-function units — the site was one undifferentiated bucket *by
  construction*, because all three member-call productions threw away the
  `Block` that says what stopped them.
* **#143's `-more` is an arity artefact.** Every one of its 1,038 emitted
  functions is discounted by a suffix that means "the same construct twice", not
  "a second construct".
* **`expr-op-0x27` — the #1 blocking feature on the emitted board, 22,759
  emitted and 407,016 bodies — converts 6 emitted functions** when its named
  token is granted.

---

#### 9.17.1 The site had no instrument, and that was a property of the source

§9.13 sized the three receiver-designator `prod` sites at 37,060 blocked
emitted, "the largest single site on the emitted board, larger than any census
key", and could say nothing about what is *in* them. That is not an oversight of
the analysis. `mcall_{tail,chain,cmp}` each call `eat_receiver_this`, which
returns an `Err(Block)` carrying the refusal context **and the byte**, and each
maps it to a flat `prod_tag("…-recv-not-a-plain-b9-load")` — discarding both. No
scan of any existing axis could have decomposed the site.

The tag is refined in place, keeping the published site name as a **prefix**
(`<old site>/<construct>`), so every figure keyed on the old string is recovered
by a prefix test. It costs nothing in the harness: `prod` is already a census
axis and already a row-dump column.

**It names the construct, never the position** — [`mcall::Fail::blocker`]'s own
rule, and the same vocabulary (`off-add`, `deref-load`, `store`, `plain-call`,
`virtual`, `temp-bind`, `convert`, `ternary`, `call-in-expr`), so the two axes
cross without a translation table. Three enumerated portable tests, no toolchain:

* `every_receiver_refusal_has_a_name` — 8 contexts × 256 bytes × EOF × 3 arms,
  the residue asserted **inside** the loop so a failure names its witness;
* `the_receiver_vocabulary_is_injective` — the designator and bind positions are
  disjoint, so no two rows of the decomposition can be summed into a double
  count;
* two **arity** axes the opcode alone cannot see —
  `the_intrinsic_receiver_arm_separates_by_selector` (2113…2119 must give seven
  distinct names, and the test's own encoder is checked against the captured
  `80 41 08 00 00`) and
  `a_literal_behind_an_offset_add_is_named_for_the_add_not_the_byte`.

**Read-only over the census, run rather than argued.** The 878-TU scan with the
refined tag reproduces the aggregate report **line for line** (the only
difference is the wall clock) and **161,262 of 161,262** dump rows are identical
outside the tag column. The tag-coverage residue stays **0** — no body enters a
production, declines, and reaches an untagged bail. `c2rs perf` geomean 540× at
base, 541×/547× at tip.

**And it lands on two published figures it was not fitted to.** §9.13 derived,
by hand and from a different join, that 3,062 emitted rows of
`expr-intrinsic-this-adjust` are clean, and that **135,926** of that row's
135,941 bodies decline at `eat_receiver_this`. The new axis reads **3,062** and
**135,923** — the second differing by 3 bodies across the two HEADs. A
decomposition agreeing to the unit with a number computed a different way is what
separates this from a relabelling.

#### 9.17.2 The first version of the axis repeated §9.14.7's disease, and the workload caught it

The table's first run filed **5,806 emitted functions** under
`then-op-0x33` — an honest hex bucket, so the totality test passed. The bytes say
otherwise:

```text
53 · 26 79 51 · b9 18 5b a6 43 d5 37 · 33 86 41 74 00 · 27 a6 43 d0 34 · 99 …
     the method   the base pointer      the literal      the OFFSET ADD    bind
```

`33 <int-like> <k>` behind a `27` is a **byte-offset add on the designator**
(`p->f.m()`); the literal only feeds it. Naming the byte the run stopped in front
of is precisely the defect §9.14.7 records for `op-0x55` and the one #139 exists
to cure — reproduced here, in the instrument written to avoid it, one commit
after reading that section. A two-token lookahead names it `then-off-add`, and
the arity test above is what keeps it named.

**After the repair, zero rows land in a hex bucket at all.** All 27 arms carry a
construct name. That is a stronger residue statement than the pre-registration
asked for, and it is a *measured* zero rather than a designed one.

#### 9.17.3 #142, decomposed — and the site is 8 % assignments

At `1f3e00e`, 878-TU dc3 workload, emitted-function units.

| the three receiver-designator sites | emitted | clean | clean ∧ complete |
|---|---:|---:|---:|
| `tail-recv-not-a-plain-b9-load` | 23,158 | 7,824 | 0 |
| `chain-recv-not-a-plain-b9-load` | 2,490 | 18 | 2 |
| `cmp-second-recv-not-a-plain-b9-load` | 6 | 0 | 0 |
| **total** | **25,654** | **7,842** | **2** |

**The denominator aged by 11,406, and not through the rung that took it.** §9.13
read 37,060 / 9,111 / 1,399; #128 converted 1,385. The other 11,406 emitted
functions left the *chain* arm because #128 **re-routed** them to other
productions — §9.13 published that re-route in the body column
(`chain-recv…` 94,948 → 30,183) and never restated it in the emitted one
(13,896 → **2,490**). Pre-registering against 37,060 minus a conversion count
was a MISS by 28 %. §9.14's P3 records a denominator ageing by *date*; this one
ages by a **neighbouring rung's re-routing**, which is invisible in the number
the board item carries.

The clean stock is intact: 7,842 against §9.13's 7,712 residue. **7,741 of it
(98.7 %) reads `complete-none`.**

| receiver construct | emitted | clean | names | `calls-0` | walker |
|---|---:|---:|---:|---:|---:|
| `no-b9-this-adjust` (intrinsic 2113) | 9,653 | **3,063** | 1,290 | 0 | 881 |
| `then-off-add` (`base + k`) | 5,803 | **2,856** | 1,270 | 759 | 252 |
| `b9-not-a-ptr4` | 2,278 | 174 | 34 | 519 | 12 |
| `no-b9-literal` | 1,125 | 107 | 89 | 145 | 206 |
| `then-store` | 1,100 | 25 | 17 | 31 | 0 |
| `then-operand-load` | 1,090 | 292 | **4** | 326 | 0 |
| `no-b9-plain-call` | 1,026 | 10 | 10 | 0 | 197 |
| `no-b9-base-member-addr` (2117) | 706 | 153 | 60 | 0 | 21 |
| `no-b9-base-downcast` (2115) | 598 | 221 | 104 | 0 | 6 |
| `then-dynamic-cast` (2119) | 543 | **542** | 115 | 0 | 0 |
| `no-b9-dynamic-cast` | 460 | 1 | 1 | 1 | 0 |
| `no-b9-convert` | 335 | 200 | 95 | 96 | 111 |
| …15 further named arms | 937 | 198 | — | 183 | 292 |
| **total** | **25,654** | **7,842** | — | **2,060** | **1,978** |

**Three arms are 82.4 % of the clean stock** and they are three different orders
of work: an intrinsic with no production at all (3,063), a designator offset add
(2,856), and a `dynamic_cast` receiver (542).

**8.0 % of the site — 2,060 emitted functions — has no receiver in it, and that
is a byte fact rather than a judgement.** `calls-0` is a body with **no CALL
token anywhere**, so it cannot contain a member call. The body dispatch offers
*every* statement-head `26` to the member-call productions, and an assignment
whose destination is a symbol opens on the same byte:

```text
26 d5 bd 04 00 · b9 5b 0a 86 43 83 20 · 32 86 43 83 20 · 4b     *dest = src;
26 29 0a       · 33 86 41 74 13        · 0f 86 41 74    · 4b     x <op>= 0x13;
```

`32` is `mcall`'s own `Stop::Store`. `then-store` (1,100), `then-operand-load`
(1,090) and `no-b9-literal` (1,125) are assignment statements the production was
offered and declined. Any ranking that reads 25,654 as "receiver work" is over by
at least 2,060 and by more than that on the arms whose `calls-0` share is
partial.

#### 9.17.4 The blocker names ARE trustworthy — and §9.14's repair is not why

The brief asked whether the repaired completeness walker made the names
trustworthy. It is answerable and the answer is a measurement, not an argument:
cross the census key against the **measured** receiver construct, per row, over
the 7,840 clean-not-complete rows. The noun map is deliberately generous to the
"trustworthy" hypothesis and every arm has one, so nothing is absorbed into an
undecidable bucket.

| | rows | |
|---|---:|---|
| the key **names the construct at the receiver position** | **7,421** | **94.7 %** |
| the key names something else | 419 | 5.3 % |
| undecidable | **0** | — |

**Registered 55 %, interval [25 %, 85 %]. Measured 94.7 % — a MISS above the
ceiling, and it corrects a published sentence.** §9.13 wrote that these rows'
census key "names the second reader's stop, not the first reader's refusal",
which is true of the *mechanism* and reads as an indictment of the *name*. The
name is right 19 times in 20, because the two readers stop on the same construct
one token apart: the production bails at the `33` literal, `parse_expr` walks it
and stops at the `27`, and the key says `expr-op-0x27`. Two independent readers
also agree to the unit on the biggest arm — 3,062 clean rows keyed
`expr-intrinsic-this-adjust` against §9.13's independently derived **3,062**.

**§9.14's repair is almost entirely out of reach of this site, and the control
that says so could have said the opposite.** The repair is inside `mcall`'s
completeness walker, and only that walker mints `expr-call-in-expr-*` keys. So
the question "how much of the site was in the repair's blast radius" is a
countable one:

| at the three sites | emitted rows | |
|---|---:|---|
| key minted by the **completeness walker** | 1,978 | 7.7 % — the population §9.14 *could* have moved |
| key minted by another reader | 23,676 | 92.3 % |
| keys still naming `type-ptr` | **0** | the repair's own success criterion, reproduced here |
| `complete-whole:grammar` over the whole site | **72** | of 25,654 |

and **7,741 of the 7,842 clean rows (98.7 %) read `complete-none`**, which is by
definition a refusal the walker never produced.

The pre-registration said the repair moved **0** of these, which was too strong:
1,978 were reachable. What is measured is the substance — 92.3 % of the site's
keys come from a reader §9.14 did not touch, and the site has 72 completeness
readings across 25,654 emitted functions.

So what #142 is missing is **not truth, it is a completeness bit**. Every arm's
name is right and no arm has a widening estimate attached, which is exactly why
§9.13 called the residue "genuinely unmeasured" — and the reason is structural:
`Block::completeness` returns `NoSignal` for any keyed byte refusal whose `ctx`
is not `CALL_IN_EXPR`, and these refusals are minted by the statement and
assignment layers.

The 419 disagreements are the real second-reader stops and they concentrate:
291 `then-operand-load` rows keyed `expr-convert-target-8642` / `-A641` /
`expr-ternary`, and 87 `no-b9-literal` rows keyed
`expr-call-in-expr-recv-intrinsic-this-adjust-then-intrinsic-call`.

#### 9.17.5 #143 — the `-more` is a COUNT, and the row is worth 6 here and 356 elsewhere

The row reproduces §9.14's figures exactly: **1,038 emitted, 851 clean, 267
distinct names**, and 1,008 of 1,038 bail at
`tail-argument-not-in-the-operand-vocabulary`. **All 1,038 read
`complete-more:grammar`; none reads `-whole`.**

The shape, from a probe whose census key is the row's own
(`work/warms/probe_offadd.cpp`): a **byte-offset add in a call argument**,
`p->one(&t->s.k)`. c2's listing gives the lowering directly:

```text
?a1@@YAXPAUS@@PAUT@@@Z    38840008  addi r4,r4,8
                          48000000  b    ?one@S@@QAAXPAH@Z
?a3@@YAXPAUS@@PAUT@@H@Z   7c8b2378  mr   r11,r4
                          7ca42b78  mr   r4,r5
                          38ab0008  addi r5,r11,8
                          48000000  b    ?three@S@@QAAXHPAH@Z
```

**The `-more` is an arity artefact.** `&t->s.k` is **two** `27` off-adds, one per
designator step — from the probe's own `.ex`:

```text
b9 01 0a 86 43 89 20      the base pointer
33 86 41 74 00 · 27 …     + 0      (t->s)
33 86 41 74 08 · 27 …     + 8      (.k)
55 86 43 f4 08 · 4c       the formal's type, apply
```

`Admit` holds construct **classes**, so the second off-add takes the
`adm.holds(blk)` arm: `need = NEED_MORE`, `broke_on` never set, and the key
renders `-then-off-add-more` **with no `-and-<kind>` third construct**. The
walker's own comment on that arm reads *"a construct that repeats means its
production did not consume the thing the classifier named — a bug, not a body"*.
Here it is a body, and the construct legitimately repeats. Every one of the 1,038
carries a discount that means "twice", not "and something else" — and a one-step
recognizer prices the row at a fraction of itself (the first version of this
lane's sink fired on 1 of the probe's 4 witnesses).

**Four sinks, one base, measured on the same binary with the sink disabled.**

| sink | grants | Δ bodies | Δ emitted | graded against the oracle |
|---|---|---:|---:|---|
| **off** (control) | — | 0 | **0** | reproduces every published number |
| `zero` | an off-add run summing to **0** | 0 | **0** | `Port=Match` (`pz.cpp`) |
| `honest` | the run as `[Load, Lit(sum), Add]` | +5 | **+5** | `Port=Match` (`ph.cpp`) |
| `expr` | `27` as an operator in **all** of `parse_expr` | +6 | **+6** | `Port=Match` (`ph.cpp`) |
| `ceiling` | the run with the **offset dropped** | +1,471 | **+356** | **`Port=Mismatch @ 8`** (`pn.cpp`) |

The `zero` arm is the #127 analogue — a designator chain summing to 0 addresses
the base itself and c2 emits nothing for it, so no codegen is needed. #127's
offset-0 arm was 92 % of its row. **Here it is 0 emitted functions**, though it
is not vacuous: it moves 703 bodies and 136 emitted rows off their keys and
converts none of them. No rate transfers between two arms of one family, and
none transfers between two *families* either.

**Exactly ONE independent refusal separates +5 from +356, and it is named.**
`expr-op-0x27` reads **22,456 in both** the `honest` and `ceiling` scans, so the
351-function difference is entirely `tail_call_shape`'s slot path, which has no
`SlotArg` for a computed address. Every member call is multi-argument by
construction (the receiver is slot 0), so every one of them takes that path.
Registered **≥ 3** independent refusals; measured **1**.

#### 9.17.6 The row above it: `expr-op-0x27` is worth 6

`expr-op-0x27` is the **#1 blocking feature on the emitted board** — 22,759
emitted, 407,016 bodies, 23.2 % of the blocked body column. Granting its named
token in `parse_expr` converts **6 emitted functions**. The row leaves the board
entirely and **201,618 bodies re-file under `expr-op-0x30`**, the indirect load,
which was below the cut before and is now the largest row on the body axis.

The byte-offset add is a **designator-chain prefix**. What stands behind it is
the rest of the member-access chain, and the chain is the work. That is the same
finding §9.14.5 recorded for `recv-load-whole` — a row that looks like the find
of the session and is a phase — reached from the other direction, and it is the
sharpest instance of §8.7's rule that a blocking-feature count is a *position in
a queue*, not a quantity of work.

The widening is principled and is *why* the number is trustworthy: `27` is the
**byte**-offset add and `02` is the scaled one (`p + 1` on an `int*` emits
`addi r3,r3,4`), which is why `parse_expr`'s pointer-arithmetic guard refuses
`02` over a pointer and why `27` may be exempt from it. The sink separates the
two facts the old single `saw_ptr && any-arith` test conflated. Shipping it would
also oblige `mcall::eat_int_operands`'s `Vocab::CallArg` to widen in lockstep, or
§9.14.6's correspondence guard goes red.

#### 9.17.7 DECLINE, and the control that would not have caught the over-claim

**#143 is declined in this lane**, under the rule registered before the
measurement. The realizable worth here is **6 emitted functions** — §8.7's
decline size, and smaller than the 8 the strongest lane of the week declined at.
The 356 needs a new `SlotArg` variant and its **ordering rule** inside the
permutation walk, in `crates/c2-core`, which this lane may not touch. §9.13.1's
ALARM is exactly the rule at issue: `?a3` is `mr r11,r4 ; mr r4,r5 ;
addi r5,r11,8` — **one** non-address move in the walk, which is the n ≤ 1 case
where address-last and address-second agree and where a wrong rule ships green.

**The workload differential could not have caught the over-claim, and this is the
thirteenth time that shape has come up.** The `ceiling` sink emits `mr` where c2
emits `addi`. The 878-TU scan under it still reads **6 match, 0 mismatch,
census/gate disagreement 0** — because none of the six byte-exact TUs carries the
shape. Only the dedicated probe failed, `Port=Mismatch @ offset 8`, and the
`zero` arm's `Port=Match` beside it is what says the probe can also pass.
Registering "0 mismatch on the workload" as this rung's control would have been
§9.13's E4 verbatim.

#### 9.17.8 Pre-registration score — 13 of 17, and three of the four misses are the findings

| | registered | measured | |
|---|---|---|---|
| A1 | site emitted 35,700, [32,000 , 38,500] | **25,654** | **MISS**, below the floor |
| A2 | clean 7,730, [6,300 , 9,200] | **7,842** | HIT |
| A3 | clean ∧ complete 60, [0 , 900] | **2** | HIT |
| A4 | the intrinsic family is the largest, 40 % [20 , 70] | largest, **50.8 %** | HIT |
| A5 | top three ≥ 80 % of clean | **82.4 %** | HIT |
| A6 | arms with ≥ 500 clean: 4, [2 , 8] | **3** | HIT |
| A7 | 0 rows in an unnamed bucket | **0** — after a repair | HIT (see below) |
| A8 | the axis is read-only, to the unit | report identical, 161,262/161,262 | HIT |
| C1 | key/construct agreement 55 %, [25 % , 85 %] | **94.7 %** | **MISS**, above the ceiling |
| C2 | ≥ 30 % name something else | **5.3 %** | **MISS** |
| C3 | §9.14's repair moved **0** of these | **1,978 (7.7 %) were reachable**; 0 still name `type-ptr` | HIT on the substance, the **0** was too strong |
| B1 | the row ages by ≤ ±15 % | 1,038 / 851 / 267, **exact** | HIT |
| B2 | Δ emitted 60, [0 , 400] | **0 / 5 / 6 / 356** | HIT on the interval |
| B3 | ≥ 3 independent further refusals | **1** | **MISS** |
| B4 | DECLINE | **DECLINE** | HIT |
| B5 | the disabled sink reproduces the base | every number | HIT |
| B6 | the differential is the control that can fire | it **fired** | HIT |

* **A1 is a new way for a denominator to age.** §9.14's P3 records one that aged
  by *date*; this one aged because the **neighbouring rung re-routed** 11,406
  emitted functions out of the site without converting them, and the board item
  carried only the conversion count. Subtracting a rung's realized gain from a
  site it touched is not a correction — it is a guess that the rung moved
  nothing else.
* **C1/C2 are one miss and it corrects §9.13.** The prediction inherited §9.13's
  sentence about second-reader stops and read it as "the names are wrong". The
  names are right 94.7 % of the time; the thing that is missing is the
  completeness bit. A published mechanism restated as a quality judgement is how
  a wrong prior gets inherited, and the fix is that C1 was registered as a
  *number* with an interval that could have contained either answer.
* **B3's miss is the section's largest finding.** It was registered off the
  `-more` suffix, i.e. off exactly the discount the measurement then showed to be
  an arity artefact. Reading a suffix as evidence about *what* is behind a row,
  when it is computed by a set that cannot count, is #110/#139's failure in a
  third costume.
* **A7 passed in the letter and failed in the spirit.** `op-0x33` is an honest
  hex bucket, so a totality test could not see that 5,806 functions were filed
  under the byte the run stopped in front of. Totality residue 0 is not a
  control (#144); the arity test is.
* **B2 registered one number for a quantity that has four.** All four realized
  values land inside the interval, which makes the hit weak evidence: the
  question "what does granting this construct cost" has a different answer for
  each of four gates and the registration did not say which gate it meant.

#### 9.17.9 Gate evidence

At `39ae1e2`, worktree branched from `1f3e00e` (verified: the harness's own
worktree branched from `origin/master`, **587 commits behind**, and was reset
before any work — the third lane this week to meet that), cache addressed by its
canonical main-repo path.

* `cargo test --workspace` — base `1f3e00e` **589 passed, 0 failed, 1 ignored**
  → tip **594 passed, 0 failed, 1 ignored**. Both measured, not inferred: the
  base was rebuilt from `git checkout 1f3e00e -- crates` and re-run.
  **`#[test]` grep over `crates/` 590 at base → 595 at tip.** Grep and runner
  reconcile at both ends once the one `#[ignore]`d test is added to the runner's
  passed count (590 = 589 + 1; 595 = 594 + 1), so no grep line here is prose or a
  doc comment. Five new portable tests, all enumerated over their domain.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes ran, 0 FAIL / 0 SKIP /
  0 NO-RESULT, **2,520 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `c2rs selftest` — **210 PASS, 0 FAIL, 0 skip**.
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* `scripts/cross_sweep.sh` — see §9.17.10.
* 878-TU workload scan — **6 match, 0 mismatch**, 865 vocab-gap, 7 capture-fail;
  bodies **706,402 / 2,462,571 (28.69 %)**; emitted **36,059 / 178,968
  (20.15 %)**; census/gate disagreement **0**. Identical to base on all of them,
  with the row dump armed and with it off.
* `c2rs perf` — geomean **540×** at base, **541× / 547×** at tip over the 100
  matched fixtures; 100 Match, 0 mismatch, 110 not-implemented. The two sinks add
  one `OnceLock` read each on a locator that runs per call argument, and the
  measurement is here because arguing it would not be.
* Probes — `pz.cpp` `Port=Match` under `zero`; `ph.cpp` `Port=Match` under
  `honest` and `expr`; `pn.cpp` `Port=NotImplemented` under `zero` and
  **`Port=Mismatch @ offset 8`** under `ceiling`.

**No fixture was added and no `fixtures/cpp/` entry changed**, because nothing
shipped. The probes live under `work/` and are named in this section so the
decline is reproducible; a fixture for a shape the port refuses would put a claim
in every gate lane that this lane did not earn.

#### 9.17.10 Board items

* **#147 — the off-add ARGUMENT slot, 356 emitted, in `crates/c2-core`.**
  Needs a `SlotArg` variant for `base + k` and its **position in the permutation
  walk**. Route to whoever owns codegen. The capture grid is §9.13.1's with one
  axis added, because §9.13.1's ALARM is the exact rule at issue: (walk length
  0…4) × (the address at slot 0 / a middle slot / last) × (offset 0 / small /
  past the `addi` immediate) × (free and member callers). The measured
  counterfactual, the probes and the four sinks are in `39ae1e2` behind
  `C2RS_SINK_OFF_ADD_ARG`; **do not re-derive the row's worth from its census
  size**, which overstates it by 3.6×.
* **#148 — `expr-op-0x27` is worth 6 emitted functions.** The #1 row on the
  emitted board. Behind it is `expr-op-0x30` and the rest of the member-access
  chain (201,618 bodies re-file there). The board should carry the number so
  nobody schedules 22,759.
* **#149 — the completeness walker cannot COUNT, and reads its own inability as
  a bug.** `Admit` holds construct classes, so a construct that legitimately
  repeats renders `-more` with no `-and-<kind>`, and the code comments that state
  as "a bug, not a body". **Every `-then-<x>-more` key with no `-and-` is a
  candidate for the same misreading**, and each one is a row somebody may discount
  on a suffix that means "twice". Same family as #110/#139: one measure, wrong
  about what it is measuring. An `Admit` that carried a multiplicity would name
  these `-whole2` instead.
* **#150 — the receiver-designator site is at least 8.0 % assignments.** 2,060 of
  25,654 emitted rows are `calls-0` — no CALL token in the body at all. The body
  dispatch offers every statement-head `26` to the member-call productions. Any
  sizing of #131/#142 off 25,654 is over by at least that, and the three arms
  concerned (`then-store`, `then-operand-load`, `no-b9-literal`) are named and
  countable now.
* **#151 — the three receiver arms worth ranking**, with no rate borrowed between
  them: `no-b9-this-adjust` 3,063 clean (this is #140's row, sized at **472**
  emitted end to end — the clean figure is 6.5× its measured worth),
  `then-off-add` 2,856 clean (the receiver-side twin of #143, and #143's
  argument-side arm converted 6), `then-dynamic-cast` 542 clean over 115 names.
  §9.13 measured two arms of one family converting **19×** apart and this lane
  measured a third at 6 against a clean ceiling of 851 — **a `clean` figure has
  now been wrong by 6.5×, 19× and 142× on three different rows of one site.**
* **#152 — `Block::completeness` returns `NoSignal` for every refusal minted
  outside `CALL_IN_EXPR`**, which is 98.7 % of #142's clean stock. That is not a
  defect — it is honest — but it means the largest site on the emitted board can
  never be ranked by completeness while its keys come from the statement and
  assignment layers. Either the walker reaches these positions or the board needs
  a second completeness producer for them.

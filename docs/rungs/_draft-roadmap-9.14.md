# DRAFT for `docs/ROADMAP.md` §9.14 — paste verbatim, then delete this file.

Kept out of `ROADMAP.md` on purpose: that file is the recorded add/add conflict
site for concurrent lanes (`docs/rungs/README.md`), the coordinator lands §9.14
serially, and lane `w-eh` is live in `docs/` at the same time. Everything below
is the section text.

---

### 9.14 W-RERANK — three corruptions of one ranking input, and two of them were one defect (2026-08-01)

Lane `w-rerank`, boards **#139**, **#110**, and §9.11's lost suffix. Instrument
work: **the census numerator does not move by one function**, and the emitted
board's *size* does not change either — 125,203 blocked emitted, 43,042 of them
clean, before and after. Only the attribution moves, and it moves 13,321 emitted
functions off keys that named a construct which was never a blocker.

#### 9.14.1 Pre-registration (written and committed before any measurement)

Committed at `f96c2d0`, the lane's first commit, before the base scan was run.

| | registered | refuted if | measured | |
|---|---|---|---|---|
| **P1** | emitted on `type-ptr` keys falls to ≤ 300 | > 1,500 remain | **13,521 → 200** | **HIT** |
| **P2** | the numerator is unchanged **to the unit** | any Δ ≠ 0 | 706,402 / 36,059, thrice | **HIT** |
| **P3** | #110 and #139 are one defect; ≥ 90 % of the `-whole{k≥2}` over-count goes | drop < 5,000 | **−3,761** | **MISS** |
| **P4** | 0–8,000 `-more` bodies become measurable | > 30,000 | wrong **direction** | **MISS** |
| **P5** | the completeness repair is total, residue named, agreement 100 % | any hole or disagreement | 2,462,571/2,462,571; 466,553 agree / **0** disagree | **HIT** |
| **P6** | the guard goes **red on the base measure** at the ptr class | it passes at base | **9 of 16,352 TYPEs**, ptr among them | **HIT** |
| **P7** | 2–6 rows move in the top 25; ≥ 1 dies; ≥ 1 appears | nothing moves | 4 die, 3 appear, 5 move rank | **HIT** |
| **P8** | the guard finds ≥ 1 **further** disagreement | it finds none | **four** further classes | **HIT** |
| **P9** | `gate.sh` PASS, 0 mismatch | any mismatch | 12/12, 2,520 verdicts, 0 | **HIT** |
| **P10** | a fixture in the refused shape is `Port=Match` | refusal or mismatch | 8/8 in class, byte-exact | **HIT** |

**8 of 10, and both misses are worth more than the hits they sit beside.**

* **P3 was registered against a stale denominator, which is the WR1 lesson in a
  new costume.** The "~27,600 `-whole{k}` over-count" comes from §6u
  (2026-07-31), *before* WR1 and W-ADJUST re-keyed the family. At this HEAD the
  entire `-whole{k≥2}` population is **15,773 bodies**, so a ≥ 20,000 drop was
  arithmetically impossible and a ≥ 90 % drop was never what #110 claimed. The
  number was copied out of a board item without re-measuring the thing it was a
  number about. §9.13's E1/E2 note says a body-column anchor is not a source of
  an emitted estimate *even when transparently discounted*; this is the same
  failure with the axis held fixed and the **date** moving.
* **P4 got the sign wrong.** `-more` did fall, by 27,595 bodies — inside the
  registered interval — but almost none of that became *measurable*. UNMEASURED
  rose by 16,927 in the same scan. Narrowing the measure to its emitter (the
  `55` annotation, the pointer-arithmetic rule) means the greedy chain now stops
  on constructs with no production, which is a **more honest** reading and the
  opposite of the one registered. A magnitude landing inside an interval for the
  wrong mechanism is not a hit.

#### 9.14.2 #139 and #110 are the same defect, and it was wrong in BOTH directions

The brief named three independent corruptions. Two of them are one line.

`mcall.rs`'s completeness walker read a call argument's operand TYPE through
`eat_int_like` — width-4 integers only — while the shipping path
(`eat_call_args` → `parse_expr` → `eat_operand_type`, which all three member-call
productions route through) has admitted 4-byte pointers there since W22. The
greedy walker charges the difference as a granted `Blocker::Type(Ptr)`, and that
one grant produces **both** published symptoms at once:

* the key prints `-then-type-ptr`, a second construct that was never a
  blocker — **#139**;
* the grant count is one too high, so `-whole{k}` over-counts — **#110**.

They were tracked as two board items and repaired by one locator.

**And the measure was not merely narrow.** Run against the reproduced base gate,
the enumerated guard reports **9 of 16,352 TYPEs** disagreeing, in two
directions:

| `(tag, kind)` | class | emitter | base measure | |
|---|---|---|---|---|
| `86 43`, `86 44`, `A6 43`, `A6 44` | pointer, plain and `const` | admits | **refuses** | #139 / #110 |
| `82 12` | one-byte-unsigned | admits | **refuses** | never reported |
| `96 41`, `96 42`, `B6 41`, `B6 42` | **`volatile`** int / unsigned | **refuses** | admits | never reported |

The `volatile` row is the dangerous one and it had no board item. A measure
*wider* than its emitter manufactures phantom **completeness** — a row reads
`-whole` and the shipping path refuses it outright — and §9.13's E4 is the
record that this direction is invisible to `census/gate disagreement`, because
nothing refuses and nothing mis-emits. `volatile` at an operand position is not
a nicety either: admitting it in the *emitter* was a live wrong-bytes emit
across five shapes (W32), which is why `eat_operand_type` gates it and
`eat_int_like_or_ptr4` does not.

Three further positions were out of correspondence and are now in it:

| position | emitter | measure, before |
|---|---|---|
| the `55` call-end annotation | `eat_int_like_or_ptr4` | `eat_type` — **any** TYPE |
| pointer arithmetic in an argument | refused (`p + 1` is `addi r3,r3,4`) | admitted |
| one-byte-unsigned arithmetic / mixing | refused | admitted |
| a class-preserving `2C` conversion | admitted, emits nothing | refused |

The `55` gate alone is **2,925 of 13,500** enumerated operand streams, all in
the over-claiming direction.

#### 9.14.3 The repair reconciles to the unit, and one row is an exact identity

Blocked bodies **1,756,169 → 1,756,169** and blocked emitted **125,203 →
125,203**, both exactly. 724 distinct keys become 673. Nothing entered the
census and nothing left it; rows were renamed.

The cleanest single control is a row §6u predicted by name and by number:

```
expr-call-in-expr-recv-load-then-type-ptr-whole   2,107 -> 0
expr-call-in-expr-recv-load-whole                 6,495 -> 8,602      (= 6,495 + 2,107)
```

The row whose key said "the receiver form **and** a pointer type" loses its
second construct entirely and its bodies land on the **form-alone** key, to the
unit. §6u wrote, before any of this was built, that the repair would "merge (2)
into `recv-load-whole` and create a conflated 8,602 bucket". It is 8,602.

The `-and-type-ptr` rows behave the same way — the phantom is stripped and what
was the *third* construct becomes the second:

```
…recv-load-then-type-ptr-and-off-add-more  22,570 -> 0
…recv-load-then-off-add-more                    0 -> 22,564
…recv-object-then-type-ptr-and-call-more   19,651 -> 0
…recv-object-then-call-recv-object-more         0 -> 18,912
```

#### 9.14.4 §9.11's lost suffix: completeness is a FIELD now, not a substring

WR1 moved 39,967 functions from keys carrying `-whole`/`-more` into keys
carrying `:eof`/`:mid`. Nothing was lost and every new name is truthful, but the
two encodings live in different halves of the rendered key, so a ranking table
built by grepping `-whole` under-counts that family — and a ranking *is* such a
table. §9.13 had to re-derive the join by hand to re-check a 1,399-row figure.

`Complete` is that fact's home: a closed seven-value vocabulary, computed from
the block's own state and **never from the rendered string** (grepping the key
was the defect; a better-informed grep is the same defect), carrying its
provenance so the two producers stay separable. It is a fifth census axis beside
`cflow`, `eh`, `dispatch` and `prod`, for the reason all four of those are
axes: an orthogonal fact goes beside the key rather than into its name.

**The oracle cannot grade a correspondence**, so it is graded the three ways one
can be:

| | check | result |
|---|---|---|
| agreement | against `feature()`'s own rendering, whole enumerated key space, and on the 878-TU workload | **466,553 agree, 0 disagree** |
| totality | every body gets a reading | **2,462,571 / 2,462,571**, and the residue is the *named, printed* row `complete-none` (1,243,453) |
| injectivity | seven readings, seven distinct `complete-`prefixed names | holds; no two can be summed into a double count |

The workload row that matters: **1,289,616 rows carry no suffix at all**. Those
are exactly what a `-whole` grep silently scores as "not whole".

Reconciled against §9.11's published figures to the unit:

| | §9.11 | measured here | |
|---|---:|---:|---|
| `call-arg-multi-sym:eof` | 18,931 | **18,932** | +1 |
| the family total | 39,967 | **39,968** | +1 |

and the `+1` is not slack — it is W-ADJUST's own recorded delta
(`docs/rungs/2026-08-01-w-adjust.md` line 166, `+1 call-arg-multi-sym:eof`).

The report now prints the join both producers answer:
**83,543 blocked bodies are grammar-complete**, of which the `-whole` grep can
see 57,533.

#### 9.14.5 The re-ranked emitted board

Totals identical on both scans: bound-emitted 161,262 = 36,059 in class +
125,203 blocked; clean 43,042 (34.38 %). `clean` = `cflow-straight*` ∧ `eh-none`
∧ `calls<2`.

**Rows that DIE** (all four were in the base top 25; all four are `type-ptr`):

| row | base emitted | tip | clean |
|---|---:|---:|---:|
| `…recv-object-then-type-ptr-and-call-more` | 5,663 (rank 5) | **0** | 0 |
| `…recv-load-then-type-ptr-and-op-more` | 1,598 (rank 16) | **0** | — |
| `…recv-object-then-type-ptr-and-deref-load-more` | 1,462 (rank 18) | **0** | — |
| `…recv-load-then-type-ptr-and-off-add-more` | 1,043 (rank 24) | **0** | — |

…and below the cut, `chained-then-type-ptr-and-op-more` (586),
`recv-object-then-type-ptr-and-plumbing-more` (449),
`recv-field-off0-then-call-nested-call-and-type-ptr-more` (419),
`recv-load-then-type-ptr-and-deref-load-more` (351),
`…-and-branch-more` (316), `chained-…-and-off-add-more` (231). **13,321 emitted
functions in total leave `type-ptr` keys; 200 remain, and those 200 are real** —
a `Blocker::Type(Ptr)` reached at a position with no emitter.

**Rows that APPEAR:**

| row | tip emitted | rank | clean | what it actually is |
|---|---:|---:|---:|---|
| `…recv-object-then-call-recv-object-more` | 5,610 | **NEW at 5** | **0** | 100 % `calls-2plus` — a frame phase, not a rung. 1,139 distinct names |
| `…recv-object-then-deref-load-more` | 1,465 | 316 → **18** | 1 | likewise phase-gated |
| `…recv-load-then-off-add-more` | 1,038 | **NEW at 24** | **851** | 1,008 of 1,038 bail at `tail-argument-not-in-the-operand-vocabulary` — §6n **category (1)**, a private limit inside a production that already ships. 267 distinct names |

**Rows that MOVE:** `recv-load-then-intrinsic-call` 11 → 8 (+805);
`recv-load-whole` **32 → 17** (+777); `…call-recv-load-and-deref-load-more`
13 → 11; `expr-op-0x9B` 17 → 16; `expr-load-type-8645` 8 → 9.

Ranked by **clean ceiling** instead, two rows are new in the top 25 and one
jumps ten places:

| | clean | was | emitted | row |
|---|---:|---:|---:|---|
| 6 | 1,485 | 712 (16) | 1,506 | `expr-call-in-expr-recv-load-whole` |
| 12 | **851** | — (NEW) | 1,038 | `…recv-load-then-off-add-more` |
| 23 | 459 | — (NEW) | 560 | `…recv-load-then-type-int1-more` |

**And the biggest riser is not a rung — the `prod` axis says so.**
`recv-load-whole` reads 1,485 clean of 1,506 (98.6 %) and looks like the find of
the session. It is not: 792 of it bails at
`tail-void-body-does-not-end-at-the-call` and 711 at
`framed-result-not-consumed-by-a-literal-post-op`. It is the **statement/frame**
population — §6u's category (6) and §27.4's "not an argument question" — and
`clean` cannot see that, because `calls-1` ∧ straight ∧ `eh-none` is true of a
body that simply does not end at its call. §8.7 already says `clean` is an
optimistic ceiling and not an estimate; this is the sharpest instance yet, and
it is the reason the production axis is printed beside the joint.

**The one genuinely new candidate the corrupted input was hiding** is therefore
`…recv-load-then-off-add-more`: 1,038 emitted, 851 clean, 267 distinct mangled
names, and 97 % of it inside one shipping recognizer's argument vocabulary. Its
old name was `…-then-type-ptr-and-off-add-more`, which said the work was "a
pointer type **and** a byte-offset add". The pointer half was never there.

#### 9.14.6 The generalized guard, mechanized

> **When a census key names a construct, the measure's acceptance vocabulary
> must match the emitter's.** A measure narrower than its emitter manufactures
> phantom rungs; a measure wider than its emitter manufactures phantom
> completeness.

This is mechanically checkable and is now checked, by two portable tests that
need no toolchain:

* `a_measure_and_its_emitter_admit_the_same_types` — every `(tag, kind)` in
  `0x80..=0xFF × 0x00..=0xFF` that `read_type` parses (**16,352** TYPEs);
* `a_measure_and_its_emitter_admit_the_same_operand_streams` — the full cross of
  two operand classes × five operator shapes (none, `+`, `-`, `*`, `2C`) × the
  `55` annotation type (**16,875** streams).

Three properties make them controls rather than restatements:

1. **Both sides are driven end to end through their own entry points** over the
   same bytes — `shapes::calls::eat_call_args` for the emitter, the completeness
   walker's own argument region for the measure. A test that asserted a property
   of a *shared helper* would pass no matter how far `parse_expr` drifted from
   it, which is precisely the drift that produced #139.
2. **The domain is enumerated, not sampled.** A witness list would have missed
   the class that was wrong, because that class had witnesses on the emitter
   side and none on the measure side.
3. **They have been observed red, four times**, each on a class not yet
   repaired: 9/16,352 against the reproduced base gate; 1/16,352 for
   one-byte-unsigned; 2,925/13,500 for the `55` annotation; 1,053/13,500 for the
   stream rules; and 333/16,875 under a deliberate mutation removing the `2C`
   arm.

**The guard found more than it was built for, twice, and both are recorded
because both were nearly published as reasoning instead.**

* The first version of the shared locator returned `Int4 | Ptr4`, argued from
  the two gates' definitions: the `55` annotation is `eat_int_like_or_ptr4`, so
  a one-byte-unsigned value "would be refused one token later". That confuses
  the *formal's* declared type with the *argument expression's* type — `f(int)`
  called with a `bool` annotates `55 86 41 74` over an `82 12` operand, and the
  emitter takes it. The enumerated guard found the single excluded pair on its
  first run. **A correspondence argued from definitions is a claim; a
  correspondence enumerated over the domain is a measurement.**
* The `2C` conversion was sized at **0** and documented as a deliberate, bounded
  omission — measured on the *base* tree. Both halves were wrong: the key is
  spelled `…-then-convert`, and repairing the operand TYPE let the walk reach
  past the pointer it used to stop at, so the row went **829 → 13,325 bodies and
  26 → 1,144 emitted in one scan**. **A residue sized before the repair that
  exposes it is not sized.**

**What the guard does not cover, stated with its reason.** It is scoped to the
call-argument region, where the correspondence is exact and the emitter is
`eat_call_args`. `Vocab::IntrinsicRecv` is deliberately excluded and kept at the
old int-only vocabulary: nothing in the intrinsic family is lowered at all, so
there is no emitter to correspond to, and widening it to match a production that
does not exist would be a claim. That is the same honesty gate `form_is_measured`
applies, and naming the position in an enum is what makes the exclusion visible
instead of accidental.

#### 9.14.7 The repair's own fix reintroduced the disease, and a fixture caught it

The pointer-arithmetic refusal filed as **`…-then-op-0x55`** — the byte the
operand run stopped *in front of*, not the construct — which is exactly what
#139 exists to cure. `Fail::note` gives ties to the first note and the same loop
records a `FailKind::Value` at that offset one line earlier, so the construct
name was silently discarded. Named now (`…-then-ptr-arith`,
`…-then-int1u-misuse`) via `note_forcing`, because a stream refusal is a
property of the whole run rather than of one byte.

It was caught by `fixtures/cpp/wrr_arg_vocab_neg.cpp`, which exists because the
repair's *premise* is gradable even though the repair is not. `mark_whole` is
diagnostic — its `Err` stays an `Err` — so no byte moves and
`census/gate disagreement` is structurally blind to the whole change. §9.13's E4
is the record of registering a control that cannot see the failure mode. The
control that works is the differential:

* `wrr_arg_vocab.cpp` — **8/8 in class, `Port=Match`**, byte-exact: a pointer
  argument, a cv-qualified pointer, a class-preserving `int*`→`void*` convert, a
  pointer beside an int in one operand run, member and free-function callers.
* `wrr_arg_vocab_neg.cpp` — **0/5 in class, `Port=NotImplemented`**: pointer
  arithmetic, a `double` argument, a `long long` argument, a cross-class
  reinterpret. Without the negative half the positive cannot tell a correct
  vocabulary from one that admits everything — and "admits everything" is the
  direction that now propagates into the census.

Both are in `fixtures/cpp/`, so both are in **every** gate lane. That is the
distinction §9.13's brief draws: `differential.rs` grades a fixed list of three
fixtures and adding a fixture does not put it in that lane; `scripts/gate.sh`
walks the corpus, and the verdict count went **2,496 → 2,520**, which is
exactly 2 fixtures × 12 lanes.

#### 9.14.8 An environment hazard that cost this lane an hour, and reads as an ALARM

The first base scan reported **6 mismatch, 0 match** on untouched `master`, on
the six TUs §9.13 published as matching — with the census reproducing §9.13
exactly to the function. It is not a regression. **The capture cache is not
portable across worktrees by PATH LENGTH.**

The reference obj embeds its own output path, and the cache captures into the
entry's directory. Reaching the shared cache through a worktree symlink
(`…/.claude/worktrees/<name>/work/capture-cache`, 90 chars) serves objs that
were captured under the main repo's path (48 chars), so the port's obj is 42
bytes longer and the compare diverges at COFF offset 8 — `PointerToSymbolTable`
— by one section header's worth of shift. Addressing the same bytes by their
literal path restores 6/6 match.

Two things follow, and the second is the one worth keeping:

1. `scripts/configure_existing_worktree.sh` links `compilers/` and copies
   `work/dc3-workload/` and does **not** touch the capture cache. That is
   correct, and the reason should be in the script: a lane that "helpfully"
   symlinks it gets six phantom mismatches.
2. **`--validate-cache N` cannot see this.** It re-captures *in place*, in the
   long-path directory, then compares and self-heals — so it reports
   "6 re-captured and agreed, 0 POISONED" and returns the fresh obj, turning a
   loud failure into a silent pass. An instrument that repairs the condition it
   is supposed to detect reports the absence of the thing it just removed.

Use `C2RS_GAP_CACHE=<main-repo>/work/capture-cache` verbatim from a worktree.

#### 9.14.9 Gate evidence

At `15ed8aa`, worktree configured against the shared toolchain, cache addressed
by its canonical path.

* `cargo test --workspace` — base `99ed418` **584 passed, 0 failed, 1 ignored**
  → tip **589 passed, 0 failed, 1 ignored**. Both measured, not inferred: the
  base was rebuilt from `git checkout 99ed418 -- crates` and re-run.
  **`#[test]` grep over `crates/` 585 at base → 590 at tip.** Grep and runner
  agree at both ends once the one `#[ignore]`d test is added to the runner's
  passed count (585 = 584 + 1; 590 = 589 + 1), so no grep line is prose or a
  doc-comment here — the whole-tree grep, which *is* polluted by `docs/`, reads
  594 at base and is not the number quoted.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes ran, 0 FAIL / 0 SKIP
  / 0 NO-RESULT, **2,520 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `c2rs selftest` — **210 PASS, 0 FAIL, 0 skip** (208 at §9.13 plus this lane's
  two fixtures).
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* 878-TU workload scan — **6 match, 0 mismatch**, 865 vocab-gap, 7 capture-fail;
  bodies **706,402 / 2,462,571 (28.69 %)**; emitted **36,059 / 178,968
  (20.15 %)**; census/gate disagreement **0**. Identical to base on all of them.
* Fixtures — `wrr_arg_vocab.cpp` 8/8 in class and `Port=Match`;
  `wrr_arg_vocab_neg.cpp` 0/5 and `Port=NotImplemented`.

#### 9.14.10 Board items

* **#139 — CLOSED.** With **#110**, which was the same defect.
* **#143 — `…recv-load-then-off-add-more`, 1,038 emitted / 851 clean.** The one
  new candidate this re-rank exposed. 1,008 of 1,038 bail at
  `tail-argument-not-in-the-operand-vocabulary` — category (1), a private limit
  in a shipping recognizer — across 267 distinct mangled names. Size it off its
  own counterfactual: §9.13 measured two arms of one family converting **19×
  apart**, so no rate transfers.
* **#144 — the `volatile` operand class was admitted by the measure and refused
  by the emitter, and had no board item.** Repaired here. The general form is
  the guard in §9.14.6; the specific worry is that `eat_operand_type`'s
  `volatile` gate has **one** call site by design (W32), and any second reader
  of an operand position is a candidate for the same divergence.
* **#145 — `scripts/configure_existing_worktree.sh` should say why it does not
  link the capture cache**, and `--validate-cache` should report a path-length
  mismatch instead of self-healing it (§9.14.8).
* **#146 — extend the correspondence guard beyond the call-argument region.**
  The pattern generalizes to every place a census measure shadows a shipping
  production; the argument region is simply the one #139 was about. Each new
  pair costs one enumerated test and, on this evidence, finds something.

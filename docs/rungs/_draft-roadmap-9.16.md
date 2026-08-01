# 9.16 W-TU — #122 never moved the metric, and the metric was measuring the wrong population (2026-08-01)

Lane `w-tu`, board **#122**. Measurement and instruments; **no codegen change,
no TU converted, and none was convertible.** Base and tip `1f3e00e` +
this lane's two commits.

Headline: **TU match 6 → 6.** Three findings, in descending order of how much
they change what to do next:

1. **#122's "6 → up to 15" was the item's own ceiling restated as an outcome.**
   The string has never existed in this repository.
2. **The leading indicator counts the wrong population, and a third constraint
   binds harder than either.** The port emits one `.text` COMDAT per `.ex`
   function segment and has **no emit-set model**, so **only 25 of 871 graded
   TUs can ever be byte-exact** at any codegen quality. Six already are. TU
   match is ceilinged at **25/878** until Phase 7 exists, against a terminal
   target of 871.
3. **There is no one-away lever, and the ten are not ten different things —
   they are one thing.** **17 of the 19** reachable near-match TUs block on
   control flow. Exactly **one** (`xboxheap.cpp`) is free of both control flow
   and EH, and it is **three** independent refusals away, not the two the board
   records.

---

## 9.16.1 #122 — the number never moved, and the "15" is arithmetic, not measurement

**Verdict: the projection branch.** The nine were never converted; nothing
regressed; the TUs did not convert. The completion record carried the board
item's *ceiling* into the past tense.

The evidence, each piece of which could have come out the other way:

* **Master's own merge commit says so in its subject line.** `6b07500`,
  *"Merge: WLR — nine TUs one function from byte-exact, and none of them was a
  rung"*, opens its body with **`TU match 6 -> 6`** and continues: *"The
  pre-registered estimate, committed before any code and stated in TUs, was 0
  conversions of 9. Actual 0."* The lane that owned the item reported the miss
  correctly and in the right unit. The board did not read it.
* **The string `15/878` has never existed in this repository.** `git log --all`
  with `-S"15/878"`, `-S"15 of 878"`, `-G"TU match 1[0-9]"` and
  `-G"match +(7|8|9|1[0-5]) +[0-9]" -- docs` all return **zero commits**, over
  the whole DAG — every branch, merged or not, so a lane that moved the number
  and never merged would still have shown. A grep of all 15,849 lines of commit
  message across all refs for a TU match other than 6 returns nothing.
* **Every recorded statement of the metric says 6.** Ten distinct sentences
  across `ROADMAP.md`, `GAPS.md` and the rung docs; the values are
  `6`, `6/878`, `6 → 6`, *"6 before and 6 after"*, *"flat at 6/878"*.
* **The scan reproduces 6 at `1f3e00e` today**, warm cache, 871 hits.

**Where "15" comes from.** It is `6 + 9`: the current match count plus the size
of the bucket the item was scoped to. `GAPS.md` §8.7 closes with *"Nine of them
are one blocked emitted function away from a whole byte-exact TU, which is the
cheapest thing on the board that moves the payoff metric."* That sentence is the
item. "Up to 15" is its ceiling, and a ceiling is what you write **before** the
work, not after. (It is *not* the other 15 in that section — §8.7's published
`≤0` bucket is also 15, but that is a distance bucket including 14 TUs with no
functions, and nobody would phrase it "→ up to 15/878".)

**Why this is the most dangerous artifact on the board, stated plainly.** §9.9.2
and §9.13 record controls that passed while measuring the wrong thing. This is
worse than that class, because there was no measurement at all: a projection was
promoted to a result by the act of closing the item. The specific mechanism is
that **the board's payoff field and its outcome field were the same field**, so
"what this would buy" and "what this bought" are indistinguishable once the
status flips. Anything that records an estimate and a status in one place has
this defect.

The remedy is the one this project already uses everywhere else and did not
apply to its own board: **pre-register the estimate in its own artifact, score it
separately, and never let the estimate be the record of the outcome.** The lane
did exactly that (`GAPS.md` §9.1, estimate 0 of 9, actual 0, scored exact) and
the board overwrote it with its own guess.

## 9.16.2 The leading indicator counts `.ex` bodies; the goal is written in emitted functions

`gap.rs::near_match_tus` measures distance as `fn_total - fn_in_class` — blocked
**IL bodies**. A byte-exact TU is a claim about its **`.text` COMDATs**. §8.1
already established these are wildly different populations (2,462,571 against
178,968); nothing had crossed them per TU. Both distances now print on every
scan:

| bucket | blocked **bodies** (published) | blocked **emitted** (new) |
|---|---:|---:|
| ≤ 0 | 1 | **2** |
| ≤ 1 | 10 | **19** |
| ≤ 10 | 25 | **82** |
| ≤ 100 | 32 | **399** |
| ≤ 1000 | 210 | **857** |

The two disagree by 12× at ≤100 and they **rank differently**, which is the part
that matters for steering: `src/system/math/Rand2.cpp` is 8 blocked bodies but
**2** blocked emitted functions; `src/system/net/JsonMemory.cpp` is 7 and **3**;
`src/system/math/vec.cpp` is **565** blocked bodies and **zero** blocked emitted.
Ranking by bodies puts real work and bookkeeping in the same bucket.

**And a correction to the published band that costs one TU.** `≤1: 10` is
**cumulative**, and its first member is `src/system/utl/Spew.cpp` at distance
**0**, which already matches. The bucket holds **nine** one-away TUs and one
already-converted one. Every brief that has said "ten TUs are one function from
byte-exact" has been counting a TU that is zero functions from it.

## 9.16.3 The emit-set ceiling — 25 of 871, and it is the binding constraint

Neither distance is distance-to-match, because a third condition binds before
either:

> `PortC2::build` takes `il.functions()` — **one entry per `.ex` function
> segment** — and under `/Gy` pushes exactly one `.text` COMDAT per entry
> (`crates/c2-core/src/lib.rs:192` and the `fn_level_linking` loop).
> **There is no emit-set model anywhere in the port.**

So when a TU's `.ex` segment count differs from its reference obj's `.text`
COMDAT-leader count, the port writes the wrong number of sections and the obj
diverges however good the codegen is. `emit-emitted` is that leader count and
`fn_total` is that segment count, so the predicate is a comparison of two
numbers every scan already had:

| | TUs |
|---|---:|
| `.ex` segments **==** obj `.text` COMDATs — reachable in principle | **25** |
| `.ex` segments **>** COMDATs — port would emit **spurious** COMDATs | **842** |
| `.ex` segments **<** COMDATs — port would **miss** COMDATs | **4** |

**TU match cannot exceed 25/878 before Phase 7**, and six of the 25 are the
current matches. The terminal target is 871. So §8.3's Phase 7 is not the last
phase in the plan — it gates **846 of the 871**, and no amount of Phase 1–6
widening touches them.

`src/system/math/vec.cpp` is the clean demonstration and it is live: **zero**
blocked emitted functions, both emitted functions in class, and it is
`vocab-gap` — 802 `.ex` bodies against 2 emitted COMDATs.
`src/system/synth_xbox/MeterEffect.cpp` fails in the other direction: 10 bodies
against **13** COMDATs, so three of c2's emitted functions have no IL body at all
and no widening can produce them.

### The control, because a ceiling asserted is not a ceiling measured

The reading is that `fn_total` counts `.ex` segments and `emit-emitted` counts
`.text` COMDAT leaders. If that is wrong the ceiling is void. The invariant that
can go red: **no `match` TU may violate it** — a byte-exact obj cannot carry a
different number of `.text` COMDATs than the port wrote.

* On the workload: **0 violations**, printed on every scan beside the ceiling.
* The base rate makes it a real test rather than a tautology: agreement holds for
  25 of 871 = **2.9 %**, so six matching TUs all agreeing by accident is ~10⁻⁹.
* The unit test does not stop at "it is zero". Per **#145** — a validator that
  cannot see the defect it exists for is worse than none — it mutates a
  **matching** TU into a violation (5 segments, 2 COMDATs) and requires the count
  to go to 1, plus asserts the mutation did not change the `match` count, so the
  control tests the emit-set reading and not the class filter.

## 9.16.4 The near-match band, per TU, by the byte

All 25 in the ≤10-by-bodies band, censused one capture each at the workload's own
`/O1 /Oi /EHsc`. `reach` = the emit-set condition of §9.16.3.

| dist | emitd | reach | TU | the blocked function(s) | key | cflow / EH | what must actually fall |
|---:|---:|:--:|---|---|---|---|---|
| 0 | 0 | ✅ | `system/utl/Spew.cpp` | — | — | straight | **matches** |
| 1 | 1 | ✅ | `Main.cpp` | `?Run@App@@QAAXXZ` 222 B | `param-width-undetermined:mid` | straight / **eh-state1** | **Phase 5** — the whole EH record |
| 1 | 1 | ✅ | `system/math/Primes.cpp` | (unnamed) 294 B | `expr-jump` | **loop** | **Phase 6** |
| 1 | 1 | ✅ | `system/math/Sort.cpp` | `?HashString@@YAHPBDH@Z` 261 B | `assign-store-type-0x86` | **loop** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/LIBCMT/osfinfo.cpp` | (unnamed) 445 B | `expr-cmp-ge` | **if-n** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/LIBCMT/undname.cpp` | (unnamed) 532 B | `expr-cmp-ne` | **if-n** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/LIBCMT/vswprnc.cpp` | (unnamed) 508 B | `expr-cmp-eq` | **if-n** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/nuispeech/xboxheap.cpp` | `CXboxHeap::CXboxHeap` 404 B | `expr-op-0x27` | straight / none | **3 refusals**, one of them Phase 4 — §9.16.5 |
| 1 | 1 | ✅ | `xdk/xjson/jsonwriter.cpp` | `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` 1349 B | `expr-brfalse` | **loop** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/xlrc/xlrcimpl.cpp` | `?CreateClient@CXLrcImpl@@…` 519 B | `assign-rhs-call-0x26` | **if-n** | **Phase 6** |
| 2 | 1 | ❌ | `ChecksumData_xbox.cpp` | 192 B + the **data object** `?gFileChecksums@@3PAUFileChecksum@@A` 152 B | `expr-op-0x27`, `data-sym-not-extern:eof` | straight | **Phase 7** (3 bodies / 1 COMDAT) |
| 2 | 2 | ✅ | `system/negate_test.cpp` | 2 × (unnamed) 388/396 B | `assign-store-type-0x86` ×2 | **if-n** ×2 | **Phase 6** |
| 2 | 2 | ✅ | `system/synth_xbox/Biquad.cpp` | 838 B, 162 B | `expr-cmp-eq`, `…recv-load-then-plumbing-0x3A` | **cf-expr-0x05 / eh-unknown**, straight | **Phase 6** + a member-call production |
| 2 | 2 | ✅ | `xdk/LIBCMT/vsnprnc.cpp` | 536 B, 181 B | `expr-cmp-eq`, `call-arg-lit-permuted:mid` | **if-n**, straight | **Phase 6** + arg permutation |
| 3 | 3 | ✅ | `system/rndobj/wordwrap.cpp` | 97 / 502 / 2661 B | `expr-jump`, `expr-bit-and`, `expr-cmp-eq` | straight, **if-n**, **cf-expr-0x05** | **Phase 6** |
| 3 | 3 | ✅ | `system/utl/Pool.cpp` | 431 / 234 / 230 B | `expr-op-0x27` ×2, `expr-brtrue` | **cf-expr-0x05**, **if-1** ×2 | **Phase 6** |
| 3 | 1 | ❌ | `xdk/nuiapi/nuidetroit.cpp` | 155 / 187 / 874 B | `expr-ptr-arith:mid`, `param-multi-reg:mid`, a member-call chain | straight ×2, **cf-expr-0x08** | **Phase 7** (3 bodies / 1 COMDAT) |
| 3 | 3 | ✅ | `xdk/nuispeech/mmio.cpp` | 286 / 419 / 512 B (8 of 11 already in class) | `expr-cmp-eq` ×3 | **if-2**, **if-n** ×2 | **Phase 6** |
| 4 | 4 | ✅ | `system/synth_xbox/IPP_basicmath_xbox.cpp` | `?Add_InPlace@IPP@@…`, `?MulConstant_InPlace@…`, `?Mul_InPlace@…`, `?Mul@IPP@@…` | `expr-cmp-eq` ×4 | **loop** ×4 | **Phase 6** |
| 4 | 4 | ✅ | `system/utl/EncryptXTEA.cpp` | 191 / 244 / 492 / 478 B | `expr-intrinsic-memcpy`, `expr-op-0x27` ×2, `expr-load-type-8882` | straight ×2, **loop** ×2 | **Phase 6** (2 of 4) |
| 4 | 4 | ✅ | `xdk/nuispeech/xboxmem.cpp` | `?GetXAllocAttributes@…`, `?MemAlloc@…`, `?MemFree@…`, `?MemSize@…` | `expr-cmp-ne`, `expr-cmp-eq` ×3 | straight, **if-1** ×3 | **Phase 6** (3 of 4) |
| 7 | 3 | ❌ | `system/net/JsonMemory.cpp` | 7 bodies, **all `cflow-straight`, all `eh-none`** | `expr-op-0x27`, `call-ref-cflow-jump` ×3, `call-arg-multi-sym:mid`, `call-bound-store-0x86` ×2 | straight ×7 | **Phase 7** (11 bodies / 3 COMDATs) |
| 8 | 2 | ❌ | `system/math/Rand2.cpp` | 8 bodies | `expr-op-0x27` ×3, `call-ref-cflow-jump` ×4, … | straight ×6, **cf-expr** ×2 | **Phase 7** (13 bodies / 2 COMDATs) |
| 8 | 6 | ❌ | `system/oggvorbis/VorbisMem.cpp` | 8 bodies, one **eh-state1** | `call-ref-cflow-jump` ×3, `expr-op-0x27`, … | straight | **Phase 7** (12 bodies / 7 COMDATs) |
| 8 | 12 | ❌ | `system/synth_xbox/MeterEffect.cpp` | 8 bodies | `expr-intrinsic-this-adjust` ×2, `expr-op-0x27`, … | **loop** ×2, **if-1** ×3, **if-2** | **Phase 7** (10 bodies / **13** COMDATs) |
| 18 | — | ✅ | `keygen_xbox.cpp` (20th reachable, outside ≤10) | 18 of 20 | `expr-jump` ×8, `assign-store-type-0x86` ×2, … | **loop** ×11, **if** ×2 | **Phase 6** |

**Read the `reach` column first.** Six of the 25 in the published band —
`ChecksumData_xbox`, `nuidetroit`, `JsonMemory`, `Rand2`, `VorbisMem`,
`MeterEffect` — can never be byte-exact by widening. They are in the near-match
band because they have few blocked *bodies*, and they are unreachable because
their body count is not their COMDAT count. The sting is `JsonMemory.cpp`: it is
the **only** TU in the 4–10 band whose every blocked body is `cflow-straight` and
`eh-none` — the one clean widening target in the band — and the emit set puts it
out of reach anyway.

## 9.16.5 The key names, taken to the byte — and `expr-op-0x27` is the second reader's stop

The brief's warning held: **the key name names the blocker in 0 of the 9.**

* For **seven**, the body is `cflow-if-n` or `cflow-loop`. `c2-il::func::census`'s
  `every_in_class_row_is_a_single_basic_block` asserts every in-class row is
  `cflow-straight`, and it holds over all readable in-class bodies on the
  workload. The named `cmp`/`jump`/`brfalse` is a real construct in a real body
  and removing it converts nothing — §9.3's refutation, re-confirmed at this HEAD.
* For **`Main.cpp`**, `param-width-undetermined:mid` is a distraction; the body is
  the only one of the nine with `maxState ≥ 1`.
* For **`xboxheap.cpp`**, the key does not merely name the wrong construct — **its
  byte pointer is provably not the cause**, and that was shown rather than argued.

### The `xboxheap` ladder, rebuilt at this HEAD

§9.4's probe ladder, reconstructed from the real source
(`mSize = size; mFreeHead = this; mCount = 0; mUsedHead = this; auto& listHead =
mListHead; …; AllocatePageBlock(initSize);`) with each refusal isolated:

```
  L1  mSize=size; mFreeHead=this; mUsedHead=this;              1/1 in class  store-run
  L2  …the same run plus `mCount = 0;`                         0/1  expr-op-0x27
  L3  mSize=size; BLOCK& lh=mListHead; lh.mNext=&lh; …         0/1  expr-op-0x27
  L4  mSize=size; AllocatePageBlock(initSize);                 0/1  expr-op-0x27
```

Three structurally unrelated constructs, **one key**. Taken to the byte, with the
`.ex` segments dumped (`census --keep-il`) and compared:

```
  blocking byte reported for L2, L3, L4:   segment offset 96, all three
  segment bytes 88..104, ALL FOUR probes:  43 81 20 33 86 41 74 00 >27< a6 43 f5 08 b9 fd 09
  first byte at which each differs from L1 (the IN-CLASS control):
      L2 vs L1   offset 159   — AFTER the reported blocking byte
      L3 vs L1   offset  54   — BEFORE it
      L4 vs L1   offset 112   — AFTER it
```

**`L1` is in class and contains the identical byte at the identical offset behind
an identical 96-byte prefix.** So byte 96 is admissible, and the bracket the
census prints as *"the byte that blocked the parse"* is pointing at a byte that
demonstrably does not block. What `expr-op-0x27` records here is **where the
second reader stopped after the first reader declined** — §9.13's `this-adjust`
pathology exactly, now on the **largest row on the board**: `expr-op-0x27` is
407,016 bodies (23.2 %) and 22,759 emitted (18.2 % of blocked emitted), and the
roadmap has measured it three times at 0.14–2.5 % completion without a mechanism
for why. This is the mechanism. It is not a construct; it is a fall-through.

**Consequence: every "go to the byte" investigation that started from an
`expr-op-0x27` window has been reading the wrong bytes**, and the row's ranking
in both censuses is a ranking of a residue, not of a rung. Fixing the census to
report the *first reader's* refusal reason for these bodies is the highest-value
instrument job on the board and it is not this lane's seam.

### And `xboxheap` is still THREE refusals away, not two

`GAPS.md` §9.4 lists three and marks refusal (1) — *"a literal value in a store
run of more than one (`mCount = 0;` among seven formal/`this` stores)"* —
**"Taken — see §9.5"**. It was not taken. WLR admits a run in which **every**
statement stores the *same* literal; `xboxheap`'s run stores formals **and** one
literal, and WLR's own doc refuses that case explicitly (*"The mixed
literal/formal run is refused for the same reason and a second one"*). The two
statements contradict each other inside one document set.

Measured, with a control that could have failed:

```
  A  h->mSize=9;  h->mCount=9;  h->mX=9;          ok   store-run      (WLR's own shape)
  B  h->mSize=size; h->mCount=0; h->mX=x;         GAP  expr-op-0x27   (xboxheap's shape)
  C  h->mSize=size; h->mCount=0;                  GAP  expr-op-0x27   (the minimum of B)
  D  h->mSize=size; h->mCount=n;                  ok   store-run      (CONTROL: no literal)
```

**D is the control and it passed**: strip the literal and the same two stores are
in class, so the literal is what refuses and the reading is not an artifact of
the formals. Had D been refused, the whole attribution would have been wrong.

So `xboxheap` needs (1) the **mixed** literal/formal store run — which WLR
measured and declined on evidence, because at length 2 c2 returns the stores in
the *opposite* order to the source and the literal's position across lengths 2–6
is a two-queue schedule with a ready-time; (2) the interior sub-object reference
bind; (3) a framed member call on `this` with an argument, which is §8.3 **Phase
4**, whose governing rules are recorded as fitted hypotheses with no mechanism.
**The one TU in the band that is neither control flow nor EH still contains a
Phase-4 item.**

## 9.16.6 Is there a one-away lever? No — and the ten are not ten different things

The brief's fourth question offered two answers and the measurement gives a
third, which is worse than either.

* **No single rung converts even one TU**, let alone two. The 25 reachable TUs
  are 6 already matching and **19 not**. Of those 19: **17 block on control
  flow** (`cflow-if-*`, `cflow-loop` or an undecoded `cf-expr-*` in at least one
  blocked body), **1** on the whole EH record (`Main.cpp`), and **1** on three
  refusals including a Phase-4 item (`xboxheap.cpp`). That is the complete
  partition — every reachable TU is in exactly one of the three.
* So it is **not** "ten TUs each needing a different thing". It is **one thing,
  needed by seventeen of them**: Phase 6. The distance metric was not misleading us
  about diversity — it was hiding a *concentration*, which is the opposite error
  and points the other way.
* §8.3 currently has Phase 6 **demand-gated and last but one**, on the
  counterfactual that has said "718 functions, five scans running". That
  counterfactual is measured in **functions**. Measured in **TUs at the near
  edge**, control flow is the single largest item on the board: it is the sole
  blocker of 17 of the 19 TUs that can be reached at all. **Both readings are
  correct and they rank Phase 6 at opposite ends**, because one counts body mass
  and the other counts payoff. §8.2 says the payoff metric is TU match.
* The cheapest honest re-plan is therefore: **Phase 7 (emit-set) and Phase 6
  (control flow) are the whole remaining program for TU match, in that order** —
  Phase 7 because it gates 846 of 871 and nothing else touches them, Phase 6
  because it gates 17 of the 19 that Phase 7 does not. Phases 1–4 as currently
  ranked convert **zero** TUs at the near edge; they are census work.

## 9.16.7 Pre-registration, scored

Registered in `docs/rungs/_2026-08-01-w-tu-prereg.md`, committed at `3db930a`
before any per-TU measurement. Declared bias: pessimistic and *borrowed* (§9 was
read first), with E5/E6/E8 flagged as the ones that could go wrong. They are
exactly the ones that did.

| # | claim | est | interval | actual | score |
|---|---|---|---|---|---|
| E1 | #122 is the **projection** branch of the four | projection | one of four | projection | **HIT** |
| E2 | commits recording a TU match ≠ 6, all branches | 0 | [0, 2] | **0** | **HIT** |
| E3 | of the 9, converted by removing only the named first blocker | 0 | [0, 1] | **0** | **HIT** |
| E4 | most of the 25 any **single** change converts | 1 | [1, 3] | **0** | **MISS** — below the floor; not even one |
| E5 | of the 7 at distance 4–10, all-straight and `maxState == 0` | 2 | [0, 5] | **1** (`JsonMemory`) | **HIT** (inside, 1 off the point) |
| E6 | of the 9, key names that do **not** name the real blocker | 2 | [0, 5] | **9 of 9** | **MISS**, badly — above the ceiling |
| E7 | TUs converted by this lane | 0 | [0, 1] | **0** | **HIT** |
| E8 | `xboxheap` refusals remaining after WLR | 2 | [2, 3] | **3** | point **MISS**, inside the interval |

**5 of 8 on the point, and the three misses carry the lane's value.**

* **E6 is the important miss.** I registered 2 of 9 key names as misleading and
  the answer is **9 of 9** — I under-predicted a failure mode I had been
  explicitly warned about three times in the brief, because I was implicitly
  treating "the key names a real construct in the body" as "the key names the
  blocker". For the seven control-flow TUs both are true of the construct and
  false of the blocker, and I had already written down the reason (the
  single-basic-block invariant) before making the estimate. The prediction was
  refuted by evidence I already possessed.
* **E4 low is the useful direction.** I allowed that *something* might convert
  one TU. Nothing does. That is what makes §9.16.6 a re-plan rather than a
  ranking tweak.
* **E8** was the borrowed-prior failure: I took `GAPS.md` §9.4's "Taken" at its
  word for one number instead of re-running the probe, which is §9.14's
  *"a board item's quantity ages"* applied to a refusal count rather than a
  denominator.

## 9.16.8 The absence-read-as-success instance this lane produced, and caught

A `cargo test --workspace --release` run launched while an incremental rebuild
was in flight reported **`ok` for every target, 0 failed** — and produced **20**
result lines summing to **422 passed**, against the true **24** lines and **591**.
**169 tests did not run and the run reported success.** Nothing in the output
said "fewer targets than usual"; every line that existed was green.

It was caught only because the base-vs-tip comparison the brief mandates put 589
next to 422 and the difference was not +2. **A tip-only reading would have gone
into this document as the tip total.** Thirteenth recorded instance, and the
first where the *test runner itself* was the instrument that read absence as
success. The mitigation is the one already in use for gate lanes — compare a
count, never a status — and it should extend to test totals: **record the number
of test targets, not just the number of tests**, because a lost target is
invisible in the sum.

## 9.16.9 Gate evidence

| lane | base `1f3e00e` | tip |
|---|---|---|
| `cargo test --workspace --release` | **589 passed, 0 failed, 1 ignored, 24 targets** | **591 passed, 0 failed, 1 ignored, 24 targets** |
| `#[test]` grep over `crates/` | **590** | **592** (+2, both new) |
| `scripts/gate.sh --jobs 4` | — | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, **2,520 fixture-verdicts** |
| `c2rs selftest` | — | **210 PASS, 0 FAIL** |
| 878-TU workload scan | match **6**, mismatch 0, codegen-gap 0, vocab-gap 865, capture-fail 7 | **identical** |
| census | 706,402 / 2,462,571 (28.69 %) | **identical** |
| emitted census | 36,059 / 178,968 (20.15 %) | **identical** |
| census/gate disagreement | 0 | **0** |
| distance (bodies) | ≤0: 1, ≤1: 10, ≤10: 25, ≤100: 32, ≤1000: 210 | **identical** |
| distance (**emitted**) | not measured | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: 399, ≤1000: 857 |
| emit-set ceiling | not measured | **25 of 871**, violations among matching TUs **0** |

`cross_sweep` not run: **no codegen was touched.** The diff is `gap.rs`
(three read-only report methods and two tests), the report block in `main.rs`,
and three scripts. `PortC2`, `codegen` and every recognizer are untouched, so
there is no lowering whose cross product could have moved.

**Caveat on the environment, stated because it is printed on every scan and
should not be silently inherited:** this box's `wibo` is `1.0.1-7`, older than
the known-good `1.0.1-23`. The scan warns that this makes the *replay* column a
fake divergence alarm while *census and mismatch counts stay byte-identical*.
This lane reports no replay number and used `--replay-every 0`, so nothing above
depends on it — but the next lane to quote a replay figure from this machine
must upgrade first.

## 9.16.10 Found and not taken, ranked

1. **The `expr-op-0x27` attribution** (§9.16.5) — the board's #1 row by both
   censuses is a second-reader fall-through, not a construct. Making the census
   report the *first* reader's refusal for these bodies would re-rank the top of
   the widening order. Largest instrument job on the board; not this lane's seam.
2. **Phase 7, sized for the first time**: 842 TUs where the port would emit
   spurious COMDATs, 4 where it would miss them. The 4 are the cheaper end and
   include two license TUs (`TomCryptLicense`, `ZlibLicense`) with **zero** `.ex`
   bodies and **one** emitted COMDAT each — the smallest possible instance of the
   emit-set problem, and a much better first probe than anything with 802 bodies.
3. **The census names the callee, not the function, for any body containing a
   call** (`GAPS.md` §9.6, unfixed). Confirmed again here: probe `L4`, whose only
   function is a constructor, is reported as
   `?AllocatePageBlock@L4@@QAAPAXI@Z`. Every function name in the near-match
   table for a call-bearing body is wrong.
4. **`GAPS.md` §9.4's "Taken — see §9.5" on refusal (1) is incorrect** and should
   be corrected in place when §9 is next edited, together with the refusal count
   for `xboxheap` (3, not 2).
5. **The `.ex` carries data objects.** `ChecksumData_xbox.cpp`'s third census row
   is `?gFileChecksums@@3PAUFileChecksum@@A`, 152 B, keyed
   `data-sym-not-extern:eof` — a *data* symbol occupying a function segment. That
   is worth a line in the emit-set model, because it means `fn_total` is not
   purely a function count and the ceiling of §9.16.3 may be slightly
   conservative in TUs of this shape.

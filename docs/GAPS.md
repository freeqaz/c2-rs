# GAPS — the measured distance from here to real-TU coverage

Status: living worklist (written 2026-07-29, revised four times the same day —
for the P2b function-level census and the variable-token-width finding; then
for R1, the W5 chain mis-emit fix, W6 compare leaves and the CALL grammar; then
at end of day for R2/R3, W13a float leaves and the cast/intrinsic-call
characterization that refuted the `expr-cast` bucket name; then for W5 depth-2
trees and W13b pooled FP constants, which between them moved the census by **one
function** and produced this document's first *falsified bucket attribution*
(§4). All numbers re-measured with `c2rs gap` / `c2rs census` at HEAD (cebfb88) —
nothing below is quoted from memory). Companion to
[`ROADMAP.md`](ROADMAP.md): the roadmap says *what order*; this doc says *what
is blocking, how much of the real corpus each blocker holds hostage, what each
rung unlocks, and the exact commands that decide whether a rung is done*.

The goal restated: `c2rs gap` over the real dc3-decomp workload (878 TUs,
real `/O1 /Oi /EHsc` flags) reports a nonzero — then growing, then dominant —
**match** bucket, with zero mismatches, at port speed. **As of 2026-07-29 that
bucket is nonzero for the first time: 6 of 878** (R1/R3, §4). The word that
matters in the goal is now "growing".

---

## 1. State of the world (the regression baseline)

What is proven today, stated precisely enough that a regression is visible.
Any run that degrades a number in this table is a regression, not noise.

> **As-of marker — every number in this section and in §2/§2b was measured at
> commit `cebfb88` (W13b).** `main` has since advanced by ~40 commits of
> concurrent-session work (the statement layer, chain canonicalization,
> multi-arg tail calls, the expression/intrinsic decode, indirect-load leaves,
> `/O1` support). **Re-measured 2026-07-30 at HEAD `2724ca5`** (independent
> review; commands per §6): fixture gate **32 match / 0 mismatch / 59 refuse**
> over 91 fixtures at `/Ox`, and **28 match / 0 mismatch** at each of `/O1`,
> `/O2`, `/Ox /Gy` (`scripts/mode_lane.sh`, 90 fixtures at run time — the
> corpus was growing under the measurement); workload **6 / 0 / 0 / 865 / 7**
> of 878 TUs, census **109,501 / 2,462,571 (4.45%)**; generated sweep 2,589
> cases, 0 mismatch. The cebfb88 numbers below stand as the historical
> baseline; quote the HEAD ones.

| Claim | Number (2026-07-29) | Command that re-proves it |
|---|---|---|
| Standalone-c2 replay is byte-exact **including the COFF timestamp** on the whole capturable real workload | 871/871, 0 diverged — **re-proven at full strength 2026-07-29** on the post-token-fix code (43.4 s at `--jobs 16`), matching the 2026-07-20 full pass | `c2rs gap … --replay-every 1` |
| Standalone-c1 (front-end) replay is byte-exact | 25/25 fixtures | `c2rs replay-c1` |
| The port is byte-exact on its accepted class, fail-closed outside it | **21/41 fixtures Match**, rest NotImplemented, **0 mismatch** — and 0 mismatch across all 878 real TUs | `c2rs diff`, `c2rs perf`, `c2rs gap` |
| **Real-corpus TU coverage** (the tripwire metric, §5) | **match 6 / 878 (0.7%)** — nonzero since 2026-07-29 (R1); unmoved by W5 trees and W13b | `c2rs gap …` |
| **Real-corpus coverage, per function** (the headline numerator, P2b) | **79,719 / 2,462,571 functions in class (3.24%)** (cebfb88) | `c2rs gap …` (FUNCTION CENSUS block), `c2rs census <cpp>` |
| Port speed where it works | geomean **1081× per obj** over the **17** fixtures matching at 4afcaa7 (2.1–5.0 µs vs ~4.0 ms); ~897k objs/s at 32 threads vs ~3.1k for real c2. The matching set has since grown to **21** (W13a, W5 depth-2 trees, W13b ×2) and the geomean has **not** been re-measured over it — quote the 1081× with its 17 attached, or re-run. W13b is the first class whose objs carry a second section and four relocations, so it is also the first that could plausibly move the per-obj figure | `c2rs perf`, `c2rs perf-scale` |
| Test suite | green with toolchain present — **202 tests, 0 failed** at cebfb88 | `cargo test --workspace --release` |
| IL codec round-trip | `encode(parse(b)) == b` on the full fixture spread, fail-closed | `il_roundtrip.rs` (in the suite) |

> **On that speed figure**: an earlier revision published ~1524×, measured over
> 13 matching fixtures. The set was 17 when 1081× was measured — it had gained
> the empty TU (1841×), the empty-function TU (1122×), the W6 compare leaves
> and the `*`/`-` chains (852×) — so the two geomeans are **not comparable**,
> and the drop is a change of population, not a regression. Any per-fixture
> number that got slower would be; none did. The population has since moved
> again (W13a took it to 18, W5 depth-2 trees to 19, W13b to 21), which is
> exactly why the rule is: **quote this metric with its fixture count attached,
> and re-measure before re-ranking it.**

The replay-soundness row is the foundation: the *reference* side of every
differential is real c2 on real code, so every other number in this doc is
measured against truth, not against an approximation of it.

> ### The mismatch bucket first went nonzero on 2026-07-29
>
> **This was the first thing the differential ever caught — and no longer the
> only one: the full tally (~40 wrong-bytes emits found and closed by the
> sweep, the mode lanes and the `/O1` re-target within the following day) is
> in §6's "mismatch is an alarm" rule.**
>
> `w5_chain.cpp` reported **`Port=Mismatch`**: an obj that *differs* from
> c2's, not a refusal. Not a real-workload TU (the workload bucket stayed 0)
> and not a shipped defect — but a genuine silent wrong-bytes emit inside the
> class the port already claimed, and therefore the highest-priority item on
> the board the moment it appeared (§6, "mismatch is an alarm, not a gap").
>
> **Root cause.** The port used a single scratch (`r11`) for every
> intermediate of a chain. c2 only does that for **additive** chains:
>
> ```
> a+b+c+d  ->  add   r11,r3,r4 ; add   r11,r11,r5 ; add   r3,r11,r6
> a*b*c*d  ->  mullw r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
> a-b-c-d  ->  subf  r11,r4,r3 ; subf  r10,r5,r11 ; subf  r3,r6,r10
> ```
>
> An additive chain collapses into one running accumulator (the additive
> term-collection pass); a `*`/`-` chain gives every intermediate its own
> register, descending from `r11`.
>
> **Why the corpus could not catch it.** *The two rules coincide at exactly
> one intermediate.* Every MVP fixture up to `a-b-c` is a 2-op chain, where
> "reuse r11" and "descend from r11" produce identical bytes. The corpus had
> no 3-op `*`/`-` chain at all, so it did not *contain a discriminator* —
> a green run over it was consistent with both rules, and byte-exactness on
> it was never evidence for the one the port implemented.
>
> **Fixed** (40749e7): plan operands stay symbolic until emission, so
> `Base::Prev` resolves to the previous entry's real destination; allocation
> **refuses below `r9`**, because the deepest characterized chain is
> `a*b*c*d*e` and outside that class c2's allocator recycles dead registers
> and schedules, so numbering order is not emission order
> (`docs/CODEGEN_W5_SCRATCH.md` §2, §6).
>
> **The lesson, which is a coverage lesson, not a codegen one.** Corpus
> breadth is load-bearing in a specific way this doc should keep saying out
> loud: a fixture set earns its keep by *separating candidate rules*, not by
> being green. It was a characterization fixture written to probe the
> *neighbouring* class that produced the discriminator — which is exactly the
> negative-fixture discipline working, one class earlier than intended.

## 2. Where every real TU dies today (the funnel)

`c2rs gap`, 878 dc3 TUs, real flags, 37.3 s at `--jobs 16` (re-run at cebfb88;
every bucket unchanged from the end-of-day scan — W5 trees and W13b moved no
TU):

| Bucket | TUs | % | Meaning |
|---|---|---|---|
| **match** | **6** | **0.7** | byte-exact vs real c2 — **R1 + R2/R3, the first nonzero match bucket** |
| mismatch | 0 | 0.0 | port emitted wrong bytes (correctness bug — must stay 0) |
| codegen-gap | 0 | 0.0 | IL decoded, `PortC2` refused |
| **vocab-gap** | **865** | **98.5** | `c2_il` cannot decode the bundle's functions |
| capture-fail | 7 | 0.8 | reference pipeline itself can't compile the TU here |

**Five of the 6 matches are empty TUs** (R1, §4): a TU that defines no
functions still gets a full five-file bundle and a real 720-byte COFF obj from
c2, and that obj needs no instruction selection at all. Seven such TUs exist in
the workload; **two are deliberately refused** because they carry a stray
`4F 1F` after the module end, which defeats the *positive* `is_empty_module`
test (no `LO` marker AND no `4F 1F`). Emitting an empty obj for a TU that
really has code is exactly the mis-emit the fail-closed rule forbids, so 2/878
is the right price for keeping the test positive.

**The sixth is `system/utl/Spew.cpp`, and it is the first match with a `.text`
section at all** — the first real TU where *every* function was in class at
once. It took two rungs: R2 (empty function bodies) put its functions in class,
and R3 (the `/Gy` COMDAT `.text`-per-function shape, which R2 promptly exposed
as a live mismatch) made the obj shell right. Structurally it is a different
kind of match from the other five: those are vacuous, this one is the funnel
actually starting to work.

Scale of what sits behind the vocab-gap wall, measured from the scan JSONL
and the P2b census:

- **2,462,571 functions** across the 871 capturable TUs, of which **79,719
  (3.24%) are in class today**. Ten TUs have **0** functions (fully
  preprocessed-away bodies); 40 TUs have ≤10; 79 have ≤100; 359 have ≤500
  (`.gl`-name-derived per-TU distribution, retained for its *shape*; see the
  denominator warning below before quoting its absolute numbers).
- **664.5 MB of `.ex`** bytes total; roughly **94.5% of bundle bytes are
  opaque** to the codec (typed coverage ~5.5%, `IL_BUNDLE_MVP.md` §K2a).
- Decode is **all-or-nothing per TU** (`functions()` returns `None` if *any*
  function segment is outside the modeled grammar — or if the module has zero
  segments). A TU-level `match` therefore requires essentially *every*
  function class in that TU to be both decodable and codegen-complete. Two
  consequences that shape everything below:
  1. The TU-grained scan **cannot rank** the W5–W14 ladder — 865 × "il
     function decode failed" is one undifferentiated bucket. This is what the
     P2b function census (GAP-0, now landed) exists to fix.
  2. The headline metric is **functions in-class** (79,719 / 2,462,571) and
     will stay so for a long time. R1 moved the TU bucket precisely because
     empty TUs are the one population where "every function in the TU is in
     class" is vacuously true. `Spew.cpp` (R2+R3) is the first TU that needed
     a whole real TU's worth of classes at once, and it is the shape every
     further TU-level movement will take.

> ### The denominator is 2,462,571 — never 902,730
>
> An earlier revision of this document put the corpus at **902,730 functions**
> by counting `.gl` mangled names. **That is not a function count** and must
> not be re-derived: `mangled_names` accepts only `?…@@…` forms, and `.gl`
> also lists externals, so it both under- and over-counts relative to bodies.
> Measured on one real TU (`system/world/Dir.cpp`, 1.5 MB `.ex`):
>
> | Instrument | Count |
> |---|---:|
> | `.gl` mangled names | 2,153 |
> | `4F 1F` fn-start markers | 5,340 |
> | **`LO` body markers (`4C 4F 11`)** | **5,239** |
> | function tails (`4F 12 47 54 01 54 00`) | 5,243 |
>
> The last two agree to 0.08%; the two-byte `4F 1F` scan is ~2% high because
> that pair also occurs inside token and varint payloads. The census therefore
> anchors on the `LO` body marker (`func::split_function_bodies`, which starts
> each segment at the `4F 1F` immediately preceding its `LO` so the formals
> region stays in-segment, and never reuses a start — a collision blocks the
> later body honestly at `formals-marker` rather than silently merging two
> functions).

### 2b. Where every real *function* dies (the P2b census)

The instrument: `c2rs census <cpp> [--flags-file F --cwd D] [--keep-il DIR]`
for one TU, and the `FUNCTION CENSUS` + `blocking features` block the
`c2rs gap` report now prints scan-wide. Each function segment goes through the
*same* positive parser as the port and keeps its **first** blocking
`(production, byte, offset)` — plus, since 56d5800, a **window of the bytes
around that offset**, printed once per feature with the offending byte
bracketed:

```
 1 x expr-cmp-gt
     ... b9 ed 09 86 41 74 b9 ee 09 86 41 74 >24< b9 ed 09 ...
```

That hexdump is what converts a bucket into a decoded production; every
grammar correction below came out of one.

> **Current numerator, 2026-07-30: 473,611 / 2,462,571 (19.23 %)**, with a
> **measured census/gate disagreement of 0** — see `ROADMAP.md` §6c (the repair
> that took it *down* by 9,230), §6d (W22, +15,924), §6f (D14, the `.gl` record
> form the symbol index could not see, +9,027) and §6i (W25 + W26, the store leaf
> and the one-byte-unsigned value class, +45,956 between them), plus the sizing
> box in §6 below. Everything in the rest of this section is historical; quote the
> number above.

**79,719 / 2,462,571 functions in class (3.24%)** (cebfb88, 37.3 s at
`--jobs 16`); **re-measured at HEAD `2724ca5` on 2026-07-30: 109,501 (4.45%)**,
with the histogram head re-attributed by the expression-layer decode — the
current top eight (`expr-call-in-expr` 11.7%, `body-0x53` 7.2%, the intrinsic
2117/2113 pair 12.1%, float/double/`void*` 9.4%, `body-0x29` 1.6%) is in
`ROADMAP.md` §G5, which supersedes the table below.
Progression across the day on the identical instrument — the first
two steps decode fixes, the third a very small new class with an outsized count,
then three rungs of real codegen worth a rounding error between them:

| | in class | % |
|---|---:|---:|
| start of day | 4,154 | 0.17 |
| + variable token width (GAP-1) | 7,114 | 0.29 |
| + CALL-token decode (GAP-1) | 7,954 | 0.32 |
| + empty function bodies (`w10_empty_fn.cpp`, a44c8f3) | 78,028 | 3.17 |
| + W13a float/double leaves (9c7ba7d) | 79,041 | 3.21 |
| + signed varint short form (66f408d) | 79,718 | 3.24 |
| + W5 depth-2 trees (9b7df37) **and** W13b one-constant bodies (cebfb88) | **79,719** | **3.24** |

**The last row is the one to read carefully: two rungs of codegen, +1 function.**
W5 depth-2 trees moved it by 0 and W13b by 1 (`expr-load-type-864540` went
81,478 → 81,477). That is the "fixture pass without a real-TU improvement"
condition §6 names, and by the letter of §4 neither rung is *done* — the honest
statement is that both are byte-exact on classes the real corpus barely contains.
Recorded here rather than softened, because this document's whole function is to
keep the census the public claim.

Top 8, percentages of the **2,382,852** *blocked* functions, with what each
bucket is now **known** to be (`docs/IL_CALL_GRAMMAR.md`,
`docs/IL_CAST_CONVERT.md`):

| Functions | % | Feature | What the bytes are |
|---:|---:|---|---|
| 363,684 | 15.3 | `call-token-0xB9` | **member / indirect calls** — callee is an *expression*, not `26 <tok>` |
| 167,205 | 7.0 | `expr-intrinsic-call` | the `0x40` token — a **SECOND call token**, not a cast |
| 144,276 | 6.1 | `call-token-0x33` | the **same** intrinsic-call production, result assigned |
| 119,800 | 5.0 | `expr-call-in-expr` | a call nested inside an expression |
| 81,477 | 3.4 | `expr-load-type-864540` | **float** operand — the row W13b moved, by one |
| 80,284 | 3.4 | `call-token-0x26` | `26 dest 26 callee BD …` — assign a call result |
| 75,081 | 3.2 | `expr-load-type-888541` | **double** operand (3.1 % last scan; a rounding boundary, not a corpus move) |
| 70,078 | 2.9 | `body-0x53` | first statement is an `if`/compound |

`expr-load-type-864383` (**void\***) falls just below this cut at **47,640
(2.0%)** — quotable now because it is from this scan, not the superseded one —
then `expr-load-type-864275` (37,060 / 1.6%), `call-end-0x26` (36,640 / 1.5%),
`fn-tail-0xB9` (29,552 / 1.2%), `body-0x9B` (28,487 / 1.2%). Behind the top eight
is a long tail of **1,050 more distinct features** (1,217 at the mid-day
measurement; the count falls as retirements accumulate).

Two rows left this table today, for opposite reasons — the distinction matters
more than either number:

- **`body-0x3A` (107,253 / 4.4%) is gone because it was PORTED.** It was the
  empty function body; R2 accepted it. This is the only legitimate way a bucket
  leaves the histogram.
- **The whole `call-anchor-*` family is gone because it was never there.** See
  the box below.

Every percentage in the table also moved because the blocked denominator shrank
from ~2.45 M to 2,383,530 and now to 2,382,852. Do not diff raw percentages
across scans without that.

> **The `expr-load-type-XXXXXX` / `expr-lit-type-XXXXXX` names above no longer
> exist.** Since 2026-07-30 the key is `<tag><kind>` and carries no type id, so
> `expr-load-type-864540` reads `expr-load-type-8645`, `-888541` reads `-8885`,
> and the pointer rows that used to be hundreds of separate shards are two keys.
> §6's sharding bullet has the reason and the exact-partition check; the corrected
> ranking is `docs/IL_CALL_IN_EXPR.md` §20. Any table in this document naming a
> six-hex-digit type key predates that and is quoting a shard, not a construct.

> **A census bucket may be the instrument, and a census NAME may be a guess.**
> Two distinct failure modes, and this doc has now been wrong in both.
>
> **(a) The bucket was a parser defect — twice.** `call-anchor-0x00` (235,886),
> `-0x08` (43,269) and `-0x20` (24,600) — **12.4% of blocked functions** — all
> went to **0** when the CALL token started being decoded instead of matched
> against a hardcoded 6-byte "anchor" that was never an anchor (GAP-1). The
> first instance was the `call-token-0x01…0x05` / `expr-load-type-0N00A6`
> families, which were token-width misalignment. Misalignment and a stale model
> both look exactly like new vocabulary, and both produce buckets big enough to
> reorder a roadmap.
>
> **(b) The name was guessed and was wrong — three times.**
> 1. **The relationals.** Inferred from numeric order; three of six labels
>    wrong, `==` unnamed (45421f6, `CODEGEN_W6_COMPARE.md` §1.1).
> 2. **`call-anchor-*`.** A name for a structure that did not exist — which is
>    why it appears in both lists.
> 3. **`expr-cast`.** `0x40` was named `cast` from a single witness. It is the
>    intrinsic-call token; the real cast is `2C <TYPE> <varint>` (GAP-1,
>    `docs/IL_CAST_CONVERT.md`). The cost was not one mislabelled row: the
>    guess split one 13% production across two rows six apart, ranked one of
>    them as the cheapest large characterization job, and scored the other as
>    unrelated W11 demand.
>
> **The rule: name a bucket only from a capture that pins it — otherwise leave
> it hex.** And before scheduling work against any bucket, dump the bytes at its
> recorded offset (`c2rs census` now prints them) and confirm the parse arrived
> there **aligned**.

Reading the bucket names (`func::Block::feature`) — `<production>-0xNN` means
the parse was inside that grammar production and could not consume byte `NN`:

- `call-token-*` — the byte where the `BD` CALL token was expected after the
  callee expression.
- `body-*` — the byte opening the function body, where only a call ref (`26`),
  LOAD (`B9`) or literal (`33`) is modeled.
- `expr-*` — inside the operand stream. `expr-*-type-NNNNNN` reports the whole
  inline operand type, because the triple *is* the feature (int vs unsigned vs
  float vs pointer); a bare byte would bucket them all together.
- Named `expr-*` buckets (`expr-intrinsic-call`, `expr-convert`,
  `expr-call-in-expr`, the relationals) now carry **capture-verified** names
  only — see the box above for the three occasions on which they did not.

> **The measured relational opcodes**, since earlier revisions of this doc
> published the wrong ones (45421f6, `docs/CODEGEN_W6_COMPARE.md` §1.1):
> `0x1F` `==`, `0x20` `!=`, `0x21` `<=`, `0x22` `<`, `0x23` `>=`, `0x24` `>`.
> The guessed table had `0x20`→`cmp-eq`, `0x21`→`cmp-ne`, `0x23`→`cmp-le`,
> `0x25`→`cmp-ge` and **no name for `==`**. Any earlier revision quoting an
> `expr-cmp-*` bucket by name is quoting the wrong relation.

**What this implies for the ladder — INFERENCE, not measurement.** The
histogram is measured; the attribution of buckets to rungs is not. With the
CALL grammar *and* the `0x40` production characterized, the top splits into two
very different things:

1. **Out-of-class by construction — must keep failing closed, indefinitely.**
   `call-token-0xB9` (**15.3%**, the largest bucket) is *not* a missing
   opcode. `BD` is a postfix operator over whatever the operand stream pushed,
   and here the callee is an **expression**: `b9 <tok> <TYPE>` for an indirect
   call, `26 <method> <obj-expr> 99 …` for a member call. An indirect call has
   **no relocatable callee name anywhere in the bundle**; a member call needs
   a `this` argument and possibly vtable dispatch. `IL_CALL_GRAMMAR.md` §6.2
   lists both as shapes a widened parser may *not* accept. Widening decode
   here without W11/W12 codegen converts a refusal into a mis-emit — the exact
   trade the §1 box shows the cost of. Do not read this 15.3% as a to-do item;
   read it as a permanent ceiling on the census until generalized calls and
   member addressing genuinely exist.
2. **Schedulable, in rough cost order.** The head is now the **intrinsic-call
   family at ~13%** — `expr-intrinsic-call` (7.0%) and `call-token-0x33`
   (6.1%) are one production, the second differing only in that the result is
   assigned. It is the largest schedulable thing on the board, and it is not
   cheap: *decoding* `40 <TYPE>` with its `(<expr> 55 <TYPE>)* 4C` argument
   loop and the `66 02 <tok> <tok>` class-pair descriptor is small and buys
   census accuracy immediately, but *accepting* it needs an allow-list of
   intrinsic ids pinned by controlled fixture **with their argument literals
   constrained too** — c2's expansion turns on the literal values, not just the
   id (`IL_CAST_CONVERT.md` §1.4: one offset byte apart separates zero
   instructions from a null-guarded four-instruction sequence). Then the
   remaining call-shaped rows — `expr-call-in-expr` 5.0% + `call-token-0x26`
   3.4% = **8.4%**, which is W11. Then non-`int` operand types — float 3.4% +
   double 3.2% = **6.6%**, with `void*` (2.0%) behind them: W12 demand plus the
   FP shapes W13b refuses, and knowing how to *skip* a `double` is still not
   knowing how to lower it. **W13b is now the measured caution on this row**: it
   lowers the float/double *leaf with one constant* byte-exactly and took **one**
   function out of 81,478, so these two rows are overwhelmingly not made of
   leaves. Then `body-0x53` (2.9%), a leading `if` — W8.

`body-0x3A` (4.4%) used to head this list and is no longer on it: R2 ported it.
W6 (comparisons) and W7 (shifts) remain absent from the top eight; W6's leaf
class landed anyway, on the strength of a staged fixture rather than measured
demand, and it moved the census by less than either decode fix did. The
fixture-driven series now runs to **four**, with a clear trend:

| rung | driven by | census |
|---|---|---:|
| R2 empty function bodies | measured bucket (`body-0x3A`, 4.4%) | **+70,074** |
| W6 compare leaves | staged fixture | small |
| W13a float/double leaves | staged fixture | +1,013 |
| W5 depth-2 trees | staged fixture | **0** |
| W13b one-constant bodies | staged fixture | **+1** |

One rung chosen from the histogram outweighs the four chosen from the fixture
pile by roughly four orders of magnitude. That is the argument for demand-driven
ordering, and it is no longer an argument from principle.

The standing caveat survives unchanged: **a blocking feature is the *first*
thing that stopped the parse, not the only thing missing.** Clearing the top
bucket moves those functions to their *next* blocker, not necessarily into
class.

## 3. Gap taxonomy

Every distinct blocker between here and real-TU coverage. Ordering within
this section is by dependency, not payoff; the ranked worklist is §4.

### GAP-0 — Measurement grain: function-level census (P2b) — **CLOSED 2026-07-29**

- **What it was**: the scan bucketed TUs only; it could not say *which* IL
  feature blocks *how many* real functions. The decoder failed closed at the
  first unknown byte without reporting which production/byte it died in.
- **Closed by**: commits 63b1ad1 (`c2-il`: `Block` / `FnVerdict` / `FnCensus`
  / `IlBundle::function_census`, keyed on the first blocking
  `(production, byte, offset)`) and ec401a5 (`c2rs census` subcommand + the
  scan-wide census and histogram in the `gap` report).
- **Measured result**: **79,719 / 2,462,571 functions in class (3.24%)** — §2b.
  The denominator is anchored on the `LO` body marker, **not** `.gl` mangled
  names (see the boxed warning in §2); the previously published ~902,730 was a
  `.gl` name count and is wrong.
- **Extended by** 56d5800: the census keeps the bytes around each blocking
  site and prints one bracketed hexdump per feature. That is the feature that
  actually cracked the grammar — the CALL token, `body-0x3A` and `body-0x53`
  were all read straight off it.
- **Held**: unknown opcodes census as honest hex buckets (`expr-op-0xNN`,
  `body-0xNN`, …), never guessed names — the census *is* the measurement of
  the unknown vocabulary. Where names *were* guessed (the relationals, and
  `expr-cast`) they were measured and corrected, §2b. The census is diagnostic
  only: acceptance is unchanged and the emitter never consults it.
- **Residual**: the histogram ranks *first blockers*, which is not the same as
  ranking *rungs* — see the caveat at the end of §2b. The `26`/`B9`
  characterization follow-on (R0b) and the `0x40`/`2C` follow-on (P2d,
  `IL_CAST_CONVERT.md`) have both landed, and together they are what makes
  §2b's attribution possible at all.

### GAP-1 — IL decode vocabulary (ROADMAP G2): the 98.5% wall

- **What**: `c2_il::func::parse_segment` accepts a handful of body shapes (int
  add/sub/mul chains, depth-2 int trees, void/int tail calls, one framed-call
  form, and since 2026-07-29 the comparison leaf `<load> <lit> <rel> 2C`, the
  empty module, the empty function body and the float/double leaf — with **at
  most one FP literal** — `IL_BUNDLE_MVP.md` plus
  `CODEGEN_W6_COMPARE.md` and `CODEGEN_W13_FLOAT.md` have the grammar).
  **The FP literal is decoded as of cebfb88**:
  `33 <lit-TYPE> <8 bytes binary64 LE> <width:u16 LE>`, where `lit-TYPE`
  (`86 4a 40` float / `88 8a 41` double) is **not** the operand type tag
  (`86 45 40` / `88 85 41`) — the `kind` byte differs by 5, a distinction the
  old single-`INT_TYPE` model could not see because for `int` the two coincide.
  Note also that the *acceptance* gates for FP constants live in this parser
  (`try_parse_float_leaf`) rather than in codegen, deliberately, so the census
  and the emitter cannot disagree about what is in class.
  Everything else in `.ex` is undecoded: comparisons outside the leaf shape,
  shifts (`09`/`0A`), bitwise (`0B`/`0C`/`0D`), logical `!`/`||`/`&&`
  (`1A`/`1B`/`1C`), ternary (`43 42`), branch/label tokens (`38`/`39`,
  `54 03/04/05`), the **intrinsic call** (`40`) with its `66` class-pair
  descriptor, the **convert** (`2C`) outside the comparison-leaf position,
  memory (`30`/`32`/`27`), member bind (`99`/`9B`), switch (`3B–3D`), float ops
  beyond the W13a leaf and the typed `Box::Volume` vocabulary.
  Also undecoded: the `.ex` header/index region
  (`0x00–0x0A54`, the single largest opaque chunk), the FnHeader interior,
  most of `.gl`, and **all of `.sy` / `.in` / `.db`**.
- **Special case — CLOSED 2026-07-29 by R1**: real TUs with *zero* functions
  need no new opcode at all, only an "empty module" acceptance plus empty-TU
  obj emission. `is_empty_module` recognizes them **positively** — `.ex` must
  carry neither a `LO` body marker (`4C 4F 11`) nor a function-start marker
  (`4F 1F`) — deliberately *not* "the split returned no segments", which would
  also fire on a bundle we merely failed to split. `coff::emit_empty_obj`
  emits the 720-byte four-section shell (`.drectve`, `.debug$S`, two
  `.XBLD$W`, 11 symbols, no relocations), verified against the live toolchain.
  Result: **match 0 → 5 / 878**. Two of the seven candidates are refused — a
  stray `4F 1F` sits after their module end — which is the fail-closed test
  doing its job at a cost of 2 TUs.
- **Variable token width (measured 2026-07-29, commit 40f767d)**: IL tokens
  are **2 *or* 4 bytes, per token — not a per-file constant**. In one capture
  of `system/world/Dir.cpp` the `4F 02` module marker appears both as
  `4f 02 e3 09` and as `4f 02 a4 96 03 00`. The discriminator is **bit 7 of
  the token's second byte**: clear → 2 bytes, set → two more follow. Verified
  by applying the rule at every `B9` LOAD site: 21,443 sites land on a valid
  3-byte operand type. `func::read_token_var` implements this; the old
  whole-file `detect_token_width` is wrong for real TUs and the function
  parser no longer consults it.
  - **This fabricated census buckets.** A 2-byte read of a 4-byte token leaves
    the parse standing on the token's own tail bytes, which look like unknown
    opcodes. The `call-token-0x01…0x05` and `expr-load-type-0N00A6` families
    were misalignment, not vocabulary; both vanished, and the in-class
    numerator moved 4,154 → 7,114 on the identical instrument. Treat any
    "new opcode" found by a fixed-width reader as suspect until re-checked.
  - **Outstanding**: `crates/c2-il/src/codec.rs` still reads a fixed 2-byte
    token (`tok16`) and carries the same latent defect on real TUs. It is
    round-trip gated, so it fails **closed** (falls back to an opaque span)
    rather than mis-decoding — no correctness exposure — but it caps typed
    coverage on real bundles. Port it to the `read_token_var` rule, gate
    unchanged, before pointing the codec at the real workload.
- **Three coexisting variable-width encodings, all now pinned** (2026-07-29,
  8131ba2 characterization + 2870fc1 implementation;
  **`docs/IL_CALL_GRAMMAR.md`** is the full byte evidence). Conflating them is
  what produced the bogus census buckets:
  1. **Operand token** — `read_token_var`, 2 or 4 bytes, bit 7 of byte 1.
     **Independently confirmed** by a controlled 32,000-symbol fixture that
     forces genuine wide tokens: `v31999` loads as `b9 e2 86 01 00`, decoding
     to exactly `0x09E3 + 31999`, with the fixed `41` / `54 02 29` markers
     landing where the 4-byte read predicts.
  2. **Statement/literal varint** — `read_varint`, 1 or 5 bytes. **Corrected
     2026-07-29** by controlled fixture (`IL_CAST_CONVERT.md` §3.2): the short
     form is a **signed** byte, not an unsigned one (`return -5;` is
     `33 86 41 74 fb`); `-128` is **forced** into the escape form because
     `0x80` is the escape marker and cannot also be a payload; and the escape
     carries **8** payload bytes for tag-`0x88` types (`long long`), not 4, so
     the width depends on the operand type. See the outstanding item below.
  3. **Type** — `<tag> <kind> <LEB128 id>`, **3, 4 or 5 bytes** (`read_type`,
     new). Across 8,628 real call sites the return type splits 4,157 / 3,123 /
     1,358 between the three widths, so even a "3 or 4" rule mis-parses one
     call in six.
  - **Outstanding — `read_varint` blocks every negative literal.** It rejects
    `0x81..0xFF` outright, which is fail-closed and therefore safe (no
    mis-decode is possible), but it means every small negative constant blocks
    its function: a **self-inflicted share of the census**, a decode defect
    rather than a corpus feature. Fixing it requires threading the operand type
    through, for the 4-vs-8-byte escape.
- **The CALL token is decoded, not anchor-matched** (2870fc1):
  `CALL := BD <TYPE ret> <flags> <varint fn-type-id>`, 8–13 bytes,
  self-delimiting field by field. The old `CALL_CALLEE_ANCHOR` was
  `flags = 0` + `varint(0x1001)` — `0x1001` being merely the first function
  type a single-function fixture TU happens to create, true of every MVP
  fixture and of essentially nothing else. Consequences worth keeping:
  - the calling convention is restricted to `0x00` (cdecl); `0x04` fastcall
    and `0x40` varargs **refuse**, because accepting them would need argument
    passing the port does not have;
  - the fn-type id is decoded only to find the token's end and then
    **discarded** — it is *not* the callee (three different callees sharing a
    signature emit byte-identical CALL tokens), so callee identity has to come
    from `.gl` by token;
  - **`26 <tok>` is a symbol/lvalue push, not a call prefix** — it is equally
    the destination of an assignment, and two in a row is the ordinary
    "assign a call result" statement;
  - census effect **7,114 → 7,954**, and the re-attribution exposed what the
    mis-parse had been hiding: float and double operand types are now visible
    at 3.4% and 3.1%.
  - **The `call-anchor-*` family (~12.4% of blocked functions) went to 0** —
    see the boxed note in §2b. It was the instrument, not the corpus.
- **There is a SECOND call token, `0x40`, and the census used to call it
  `expr-cast`** (characterized 2026-07-29, 9c7ba7d;
  **`docs/IL_CAST_CONVERT.md`** is the byte evidence). It is the *intrinsic*
  call, and it occupies exactly the slot `BD` occupies:

  ```
  INTRINSIC-CALL := 33 <int-TYPE> <selector>   a bare int literal
                    40 <TYPE result>            no flags byte, no fn-type id
                    ( <expr> 55 <TYPE> )*
                    4C
  ```

  The decisive measurement: across `Dir.cpp`, `App.cpp` and `Game.cpp`, `0x40`
  follows a bare `int` constant at **6,838 of 6,839** aligned sites (the one
  exception is a parse-misalignment artifact). A cast would predominantly follow
  LOADs and sub-expressions; this follows a constant essentially always. That
  constant is the intrinsic selector — pinned by controlled fixture at 15 `abs`,
  17 `fabs`, 159/160 `_rotl`/`_rotr`, 164 `strcpy`, 165 `strcmp`, 167 `strlen`,
  170 `memcmp`, 172 `memcpy`, 173 `memset`, 1973 `sqrt`, plus a dominant
  **2113–2119** class-layout / base-offset-adjustment family. Three
  consequences:
  - `call-token-0x33` (6.1%) is the **same production with an assigned
    result** — so the family's real footprint is **~13%**, not 7%, and it is the
    largest schedulable bucket in the histogram (§2b, §4);
  - **`0x66` is not a call.** `IL_CALL_GRAMMAR.md` §7 ranks it the **#1
    unidentified blocker** (1,148 Dir.cpp bodies); it is the class-layout
    family's class-pair descriptor, `66 02 <tok classA> <tok classB>`. Read that
    doc's ranked-unknowns table with this correction applied;
  - **the real cast opcode is `2C <TYPE> <varint>`** (bucket `expr-convert`),
    and it is the hazard of the area: the *same* `2c 86 41 74 00` is
    simultaneously nothing, an `extsb`, an `extsh`, a `clrlwi` and a
    3-instruction `fctiwz` sequence, discriminated entirely by the **source**
    type, which the operand stack does not carry. That is why `2C` is accepted
    **only** directly over a comparison result today (W6), and why a blanket
    "casts are free" rule would silently drop sign-extensions
    (`IL_CAST_CONVERT.md` §2.2, §4.2).
- **Frequency**: **865/878 TUs (98.5%)**, i.e. 96.76% of the 2,462,571 real
  functions (§2b) — almost nothing reaches codegen. ~94.5% of bundle
  bytes opaque.
- **Unlocks**: decode alone moves TUs from `vocab-gap` to `codegen-gap` —
  which is the census becoming *exact* (the port's own NotImplemented reasons
  become the histogram) — and is a hard prerequisite for every match.
- **Depends on**: GAP-0 (closed) for ordering *within* this gap; the
  histogram in §2b is that ordering.
- **Difficulty**: the main body of work, but incremental by construction —
  the codec's typed-islands-over-opaque-spans model means each new token
  class lands round-trip-gated without destabilizing the rest. Landmines:
  never weaken the round-trip gate to land a class; token width is
  **per-token**, read it structurally (bit 7 of byte 1) and never from a
  per-file heuristic — misalignment does not look like an error, it looks
  like new vocabulary; `.sy` becomes load-bearing around W12–W14 when types
  stop being inferable from `.ex` alone.

### GAP-2 — Codegen classes (ROADMAP G1): the W-ladder proper

- **What**: `PortC2` lowers the MVP class plus, since 2026-07-29, `*`/`-`
  chains past two operations (W5 chains), **depth-2 expression trees** (W5
  trees), empty function bodies (R2), the
  `/Gy` COMDAT obj shape (R3), the branchless compare→bool leaf (W6),
  float/double leaves over parameters (W13a) and **one pooled FP constant per
  body** (W13b). The missing classes, with
  mechanisms per class, are the W5–W14 table in `ROADMAP.md` §G1: W5 expression
  trees past **depth 2**, W6's `<`/`<=`/`>=` against a non-zero literal,
  W7 shifts/bitwise,
  W8 control flow, W9 div/mod, W10 general frames+locals, W11 generalized
  calls, W12 memory/struct access, **W13b beyond one constant** (which needs
  c2's constant evaluator *and* its scheduler), W14 data
  sections/globals — plus the intrinsic-call family (no longer long tail: it is
  ~13% of blocked functions, §2b) and a census-driven long tail proper (switch
  tables, 64-bit carry chains, virtual calls).
- **Seven classes landed 2026-07-29**, every one characterized first:
  - **R2 — empty function bodies.** `w10_empty_fn.cpp` exact; no expression to
    select, and the largest single census jump so far: **7,954 → 78,028**.
  - **R3 — the `/Gy` COMDAT `.text`-per-function shape**, forced by R2 turning
    a latent flag dependency into a live mismatch. Not a missing *class* but a
    missing emitter **input**: `/O1` and `/O2` imply `/Gy`, the bundle never
    records it, so the same IL legitimately yields two different objs. Every
    fixture uses `/Ox`, which does not — so matching every fixture never
    licensed emitting for a real-workload TU. `system/utl/Spew.cpp` became the
    first function-bearing real TU to match. Standing implication: any claim of
    the form "the port handles class X" must say **under which flags**.
  - **W13a — float/double leaves over parameters** (`CODEGEN_W13_FLOAT.md`) —
    `mvp_fmul3.cpp` is `Port=Match`. The FP register model **shares nothing**
    with the integer one, and each difference is a place a grafted integer path
    would emit *wrong bytes* rather than run out of range: the pool is
    `[f0, f13, …, f1]` with `f0` allocatable and *first* and the result `f1`
    *last* (no analogue of `select_text`'s "refuse below `r9`"); an FP `+`
    chain does **not** collapse into one accumulator; `fsubs fD,fA,fB` is
    `fA − fB`, the **opposite** of `encode_subf`'s reversal, so reusing the
    integer convention silently negates every FP subtraction; and `fmuls` takes
    the multiplier in the **C** field. One encoder covers both precisions
    (primary opcode 59 / 63, identical XO and register fields). Gated hard
    against the shapes that mis-emit rather than overflow: FP **literals**
    (W13b — a constant costs an `.rdata` COMDAT + a REFHI/REFLO relocation pair
    + a GPR), `2C` converts, float/double mixing, any `*` under a `+`/`-`
    (contraction to `fmadds`/`fmsubs` is **mandatory**), and repeated leaves
    (`a+a` becomes `a*2.0f`, a constant again). Obj-shell effect for this
    class: exactly one extra symbol, the undefined external `_fltused`; the
    *general* trigger rule for that symbol is still open (§7 of that doc).
  - **W13b — one pooled FP constant per body** (`CODEGEN_W13_FLOAT.md` §5) —
    `w13b_fconst.cpp` and `w13b_fdedup.cpp` are `Port=Match`. Each distinct value
    gets its own `.rdata` COMDAT (4 B / `0x40301040` float, 8 B / `0x40401040`
    double, `Selection = 2`, aux checksum 0, big-endian contents), appended after
    `.text` in first-reference order, loaded through `addis`+`lfs`/`lfd` with a
    REFHI+PAIR / REFLO+PAIR quad against an EXTERNAL
    `__real@<lowercase big-endian ieee hex>`. The pool is keyed on
    **(bit pattern, width)** TU-wide, and the two symbols land immediately after
    the symbol of the function that *first* references the constant.
    **Three findings that generalize past this class**, each one a rule whose
    wrong alternative matched the entire prior corpus:
    1. **A section's relocations follow *that section's own* raw data**, not all
       sections'. Emitter-wide, latent since the first relocation ever written,
       and unobservable while `.text` was last. Put a `.rdata` behind `.text` and
       c2's four REFHI/REFLO records sit *between* them.
    2. **A constant claims its FP register before any interior temporary does**,
       in IL order — so the allocator cannot walk the emitted instruction list.
       Only `ke` (`a*2.0f*b*3.0f`, a body with a constant *and* a temp) separates
       this from "allocate in emission order"; every single-operator body matches
       both.
    3. **c2, not c1xx, is the floating-point constant evaluator.** The IL carries
       every source literal, and c2 folds (`a+0.0f`/`a*1.0f`/`a-0.0f` → bare
       `blr`, nothing pooled — but `a*0.0f` is **not** folded, so the gate is per
       `(operator, value)`, not per value), strength-reduces (`a/3.0f/7.0f` → one
       `fmuls` by 1/21, inexact and therefore a genuine numeric transform) and
       reassociates (`a*2.0f*b*3.0f` → `(a*b)*6.0f`). So "how many literals does
       the source have" is not the gate; "how many survive c2" is, and only a
       capture answers it.
    Gated at one constant because with two the schedule also changes (every
    `addis` hoists to a prologue group, each `lfs` goes at first use, FP registers
    recycle) — characterized by exactly two captures, `p1` and `p5`, which is not
    enough to implement from. The gates live in the **IL parser**, not codegen, so
    the census and the emitter agree by construction.
  - **W5 chains**, **W5 depth-2 trees** and **W6 compare leaves**, all below.
  - **W5 chains** (`CODEGEN_W5_SCRATCH.md`) — c2 allocates temporaries from a
    cursor descending `r11 → r10 → r9 → …`, skipping live registers and
    wrapping; the port now follows it for `*`/`-` chains and **refuses below
    `r9`**, since the deepest characterized chain is `a*b*c*d*e` and beyond it
    c2 recycles dead registers and schedules. This is the change that fixed
    the mis-emit (§1).
  - **W5 depth-2 trees** (9b7df37) — `w5_tree2.cpp` is `Port=Match` on all four
    shapes: left child into one scratch, right into another, root into `r3`. One
    wrinkle gates the depth and is **characterized but not explained**: with a
    `+` root the two children's registers are **swapped** relative to every other
    root operator, reproducibly and order-independently (`(a*b)+(c*d)` and
    `(c*d)+(a*b)` are byte-identical, so c2 canonicalizes the commutative root by
    parameter order and then hands the first term `r10`). Accepted at exactly this
    depth, refused above it. Depth-3 trees and the eleven negatives of
    `w5_tree_neg.cpp` (product flattening, additive term-reordering, spilling
    into `r31`, the unexplained `n_imm_sum2` register order) still fail closed.
  - **W6 compare leaves** (`CODEGEN_W6_COMPARE.md`) —
    `il_bool_materialization.cpp` is `Port=Match`, 6/6 in class. c2 lowers
    these **branchlessly**: no `cmpw`/`cmplw` at all, but carry-bit and
    bit-extraction idioms (`addic`/`subfe`, `cntlzw`+`rlwinm`,
    `neg`+`andc`+`srwi`, `eqv`+`addze`). The **`k == 0` folds are mandatory
    and dispatched first** — c2 folds a zero literal to a shorter sequence or
    to a constant (`li r3,0` / `li r3,1`), and two of the six fixture
    functions land there, so a general-spine-only implementation would
    mis-emit them. `<`, `<=`, `>=` against a non-zero literal stay **out of
    class**: the spine's instruction order for a literal lhs is unresolved and
    guessing it is exactly the silent wrong-bytes failure mode.
- **Frequency**: still 0 TUs in `codegen-gap` (decode fails first), so the
  per-class codegen demand is not directly measured. What *is* measured is
  the decode-side proxy in §2b, and its shape changed again today: the largest
  bucket (15.3%) is member/indirect calls, which are **not schedulable
  codegen work** but a permanent fail-closed class until W11/W12; the
  schedulable head is now the intrinsic-call family (~13%), then the remaining
  call-shaped rows (8.4%, W11), then non-`int` operand types (6.6% float +
  double — W12 plus the FP shapes W13b refuses), then a leading `if` (2.9%, W8).
  Treat that as inference (see §2b), not as a codegen measurement.
  Staged fixture evidence exists for the W5 depth-3 tree, W7, W8 and
  cast/intrinsic classes (`w5_tree3.cpp`, `add3.cpp`'s
  `select_max`/`shift_mask`, `il_call_return.cpp`, `w13_*.cpp`, `w13b_*.cpp`,
  `il_convert_scalar.cpp`, `il_intrinsic_call.cpp`) — fixtures sample the
  grammar we guessed matters; the census measures the grammar that does. There
  are now **four** worked examples of the difference (the table in §2b): W6,
  W13a (+1,013), W5 depth-2 trees (0) and W13b (+1), against R2's
  histogram-chosen +70,074.
- **Unlocks**: this is the gap whose closure moves *functions in-class*, and
  eventually TUs into `match`.
- **Depends on**: GAP-1 per class (decode first), GAP-0 for order; W10/W11
  additionally on the W-UNW-1 label-counter model; W13b/W14 on new COFF
  section/reloc emission.
- **Difficulty + landmines** (all from this repo's own probes):
  - **Register *allocation* is a byte-visible part of the output, not an
    implementation detail** — this is the landmine that actually detonated
    (§1). c2's rule differs *between operator families* (`+` collapses into an
    accumulator, `*`/`-` do not), and any two candidate rules that agree on
    the shapes in the corpus are indistinguishable by a green run. Registers
    that hold values which are never read still occupy byte-visible numbers
    (`CODEGEN_W6_COMPARE.md` §6: `subfe r9,r10,r10` reads an undefined `r10`,
    and dead `subfc` destinations still consume a slot). Allocate them as real
    temps; never "optimize" one away.
  - **c2 canonicalizes and reassociates arithmetic chains; the port evaluated them
    in source order.** Two rewrites, ~20 mis-emits, all in the straight-line class
    that had been called byte-exact since the MVP:

    A commutative chain is canonicalized **by register**. All five permutations of
    `a + b + c` emit the identical `add r11,r3,r4 ; add r3,r11,r5`, and `b + a`
    emits the same `add r3,r3,r4` as `a + b`. And a mixed `+`/`-` chain is
    reassociated *even when its operands already are in register order* — c2 treats
    it as a sum of signed terms and applies the negatives first, from the lowest
    positive term:

    ```text
      a + b - c  ->  subf r11,r5,r3 ; add r3,r11,r4    = (a - c) + b
      b - c + a  ->  subf r11,r5,r3 ; add r3,r11,r4      identical bytes
      a - c - b  ->  subf r11,r4,r3 ; subf r3,r5,r11   = (a - b) - c
    ```

    So the second rewrite is invisible to any gate built on the first, and the gate
    cannot simply refuse mixed chains either: `a - b + c` and `a - b - c` already
    satisfy c2's order and are byte-exact (`il_reassoc_ok.cpp` holds them as the
    separating cases). Subtracting a *literal* folds into the `addi` immediate and
    emits nothing, so it never counts.

    Both are **refusals, not canonicalizations**, on purpose. The rule looks like
    "start at the lowest positive term, apply the negatives ascending, then add the
    remaining positives", but that is inferred from ten captures, and implementing it
    wrong puts the mis-emit straight back. A canonicalizer is worth real coverage —
    it would admit every permutation rather than one in six — but it needs its own
    capture matrix first, and it is the clearest single codegen win left.
  - **A hand-picked corpus is systematically blind, and enumeration is not.** Both
    of the above, and the repeated-leaf mis-emit below, were found by
    `scripts/expr_sweep.sh` — 2,352 generated integer expressions, each compiled and
    byte-compared — not by any fixture. They had survived months because every
    hand-written positive happened to use distinct operands in ascending order:
    whoever writes the fixtures writes the shapes they are already thinking about, so
    the corpus is biased toward exactly the cases the implementation already handles.
    The sweep found ~20 wrong-bytes bugs on its first run and reports 0 now. **Run it
    after any change to expression selection**; a green fixture sweep does not
    substitute for it. This is the strongest available answer to "a green run is
    sound only on the IL it was tested against".
  - **An operand used twice licenses c2's algebraic rewriter — and this one was
    live for months.** `return a + a;` emitted `add r3,r3,r3`; c2 emits
    `rlwinm r3,r3,1,0,30` (`slwi r3,r3,1`), byte-identical to what it produces for
    `a * 2`. So a repeated leaf makes the operand stream stop describing the
    instructions, and the straight-line integer class — the oldest and most-used
    one — mis-emitted every such body from the day it was written.

    Two things make this the most instructive landmine in this document. First,
    **the rule was already known**: `try_parse_float_leaf` has had exactly this
    gate from the start ("a repeated leaf can trigger algebraic rewriting"). It was
    simply never applied on the integer side, so the corpus proved the FP path safe
    and said nothing about the integer one. Second, **no fixture could have caught
    it**: every integer positive was `a + b + c` or `a - b`, all distinct operands.
    There was no separating case, so a fully green run over 60 fixtures was
    consistent with the bug. Cf. `fixtures/README.md` — "a green corpus is only as
    strong as its ability to *separate* the candidate rules".

    The rewrite is also not one rule: `a - a` is a constant zero and `a * a` has no
    shift form, so a gate built around the `+` case alone would still be guessing.
    All are refused, and the gate runs on the *resolved* stream because
    substitution creates repetition the source does not have — nothing in
    `int x = a; x = x + x;` repeats an operand, but it resolves to `a + a`
    (`fixtures/cpp/il_repeated_leaf.cpp`).
  - **Non-commutative hazard list** (`CODEGEN_PPC_MVP.md`): `subf` computes
    rB−rA (operands *reversed*); shifts have fixed order and signedness
    picks `sraw` vs `srw`; `cmpw`/`cmplw` direction is not swappable; W6 adds
    `subfc`/`subfe`/`subfic`/`srawi`, whose operand roles are pinned across
    four independent captures. A swap is a silent corruption differential
    testing exists to catch — every such encoder stays exact-pattern until
    probed.
  - **Folds change the instruction *shape*, not just an immediate.** The
    `k == 0` comparison folds (`CODEGEN_W6_COMPARE.md` §4.6) and the `g(a)+0`
    identity fold are dispatched before the general path for a reason: the
    general spine emits different, longer, wrong bytes for them. The IL is
    **unfolded** — the folding happens inside c2 — so it is always the port's
    job, never something the front end hands over pre-simplified.
  - **A fold is keyed on the `(operator, value)` PAIR, not on the value** — and
    W13b is the case that proves the distinction is load-bearing rather than
    pedantic (`CODEGEN_W13_FLOAT.md` §5.9). `a + 0.0f`, `a * 1.0f` and `a - 0.0f`
    all compile to a bare `blr` with nothing pooled, but `a * 0.0f` does **not**
    fold: it loads `__real@00000000` and multiplies, because signed zero and NaN
    make the fold unsafe. A gate written as "refuse the value 0.0" refuses a body
    c2 really does lower; a fold written as "anything times zero is zero" emits a
    wrong `blr`. W13b briefly did the former for all four. Only a fixture holding
    **both halves** (`w13b_ffold.cpp`) separates the two candidate rules.
  - **c2, not c1xx, evaluates floating-point constants** — the IL hands the
    backend every literal the source wrote, and c2 folds, strength-reduces
    (`a/3.0f/7.0f` → one `fmuls` by 1/21, *inexact*, so a real numeric transform)
    and reassociates (`a*2.0f*b*3.0f` → `(a*b)*6.0f`) them. So the count of
    literals in the IL is not the count of constants in the obj, and any gate
    phrased over the IL's literals is guessing at c2's arithmetic. This is what
    caps W13b at one constant per body, and it is the general shape of the hazard
    for every future constant-bearing class.
  - **W-UNW-1 — CLOSED 2026-07-30.** `.pdata` label counters (`$M2545/…`) were
    a fixed seed hardcoded for a single-function TU. The seed is now **read**:
    it is the u32 at `.gl` offset 7 plus 9, and the counter advances 1 per leaf,
    4 per framed function (5 under `/Gy`, which also charges 3 per function up
    front). `docs/OBJ_GY_SHAPES.md` §3.5/§3.6; both emitters implement it and
    `fixtures/cpp/wunw_*.cpp` grade it byte-exact in all four mode lanes. Two
    classes have a *different* stride (comparison leaves 3, FP leaves 2 plus 2
    per pooled constant) and are refused when a framed function shares the TU.
  - `.pdata` carries a real reflected-CRC-32 checksum; new sections mean new
    CONST/DERIVED byte classification work per `OBJ_FORMAT_MVP.md`.
  - **A second register model is a second set of rules, not a parameter.**
    W13a's FP allocator inverts almost everything the integer one asserts —
    pool order, whether `+` collapses to an accumulator, the operand order of
    the subtract, which field carries a multiplier
    (`CODEGEN_W13_FLOAT.md` §2, §3, §6). Each of those, grafted from the
    integer path, produces *wrong bytes* rather than a refusal. When a new type
    class arrives, re-derive its allocator from captures; do not parameterize
    the existing one. W13b then broke the *independence* of the two models: a
    pooled constant takes an address GPR off the integer cursor **and** an FPR off
    the FP cursor, so they are no longer separable allocators.
  - **Allocation order is not emission order.** W13b's second correction: a
    constant claims its FP register *before* any interior temporary does, in IL
    order (`CODEGEN_W13_FLOAT.md` §5.8). An allocator that walks the emitted
    instruction list matches every body with no interior temp — which is every
    single-operator body, i.e. most of the corpus of small fixtures — and is
    wrong the first time a body has both. The general form: **an allocator must be
    driven by the IL order, and a fixture that cannot distinguish IL order from
    emission order is not evidence about either.**
  - **The obj's LAYOUT rules can be latent for as long as the section list is
    short.** A section's relocation records belong immediately after *that
    section's own* raw data. The emitter had "after all sections' raw data",
    which is the same offset whenever the relocated section is last — true of
    every obj this port emitted until W13b put a `.rdata` behind `.text`
    (`CODEGEN_W13_FLOAT.md` §5.7). Nothing in the corpus was evidence against the
    wrong rule. When adding a section, re-derive the *offsets* of everything that
    follows it, not just the section's own bytes.
  - **Flag regime — this one has now detonated.** Every codegen byte fact was
    characterized under `/Ox /GS-`; the real workload compiles `/O1 /Oi /EHsc`,
    and `/O1` implies `/Gy`, which changes the *obj shape* (COMDAT `.text` per
    function) without changing the IL. R2 turned that into a live mismatch on
    `Spew.cpp` and R3 fixed it by making function-level linking an explicit
    emitter input. The general lesson stands and is now evidenced: **the bundle
    does not record everything the obj depends on**, so a fixture pass under
    one flag set never licenses emission under another. Remaining divergences
    (inlining/EH scaffolding for otherwise in-class bodies) are still unprobed
    — expect fresh CONST/DERIVED passes per real-flag class.

### GAP-3 — Workload manifest: the 7 capture-fails

- **What**: 3× C1083, 2× C1189, 2× C2084 — all `synth_xbox`/`soundtouch`
  files the real 360 build excludes (x86-only `#error` guards) or builds
  with per-target flags. A harness/manifest refinement, not a port gap.
- **Frequency**: 7/878 TUs (0.8%).
- **Unlocks**: an honest denominator (878 → 871 measurable, or per-TU flag
  overrides in `gen_dc3_workload.sh`).
- **Depends on**: nothing. **Difficulty**: trivial; do it whenever the noise
  annoys.

### GAP-4 — Architecture: shape-matcher → real lowering pipeline (ROADMAP G4)

- **What**: codegen is a positive shape-matcher with an intentionally empty
  `passes/` tree. W8 (first CFG) and W10 (frames) force a block/instruction
  IR; COLOR register-order modeling becomes real at W5/W10.
- **Frequency**: not corpus-measurable — it is a scaling blocker for GAP-2,
  not a corpus bucket.
- **Depends on / unlocks**: restructure *at* W8, not before (per ROADMAP §G4
  — keep widening the matcher until the CFG step forces the IR, keeping
  every differential gate green through the restructure).
- **Difficulty**: the one genuinely architectural step on the ladder; the
  risk is not the IR but keeping 0-mismatch through the rewrite — land it as
  a refactor gated by the full fixture + gap-scan suite, no widening in the
  same change.

### GAP-5 — Front-end port (ROADMAP G3): not on the critical path to `match`

- **What**: `c1-core` (source→bundle) so that composition (`P3 compose`,
  source→obj fully in-process) needs no Microsoft binary. Replay proof
  (P-F0.1) landed; characterization (P-F0.2) and the crate (P-F1) have not.
- **Frequency**: blocks **0** of the gap-scan match bucket — the scan
  captures IL with the real front end. It gates only the composition
  milestone and the >2.4× downstream-speedup regime (§5).
- **Depends on**: backend class definitions (it widens in lockstep).
- **Difficulty**: medium, with one named risk: `.db` line-record semantics
  (smallest, least understood file); any preprocessor use leaves the class —
  the recognizer must fail closed.

## 4. The ladder — ordered worklist with per-rung acceptance

Rungs below the W-numbering are the instruments; W-rungs are the port. Every
rung ends with the same three gates (spelled out in §6): **fixture gate**
(byte-exact positives, NotImplemented negatives, suite green, perf
re-confirmed), **census gate** (the function-level in-class numerator rises
by the rung's measured population), **scan gate** (`c2rs gap` re-run, JSONL
diffed against the previous baseline — buckets move only in the good
direction). A rung whose fixture gate passes but whose census/scan gate
doesn't move is **not done** — it modeled a shape the real corpus doesn't
contain, which is a finding, not progress.

| Rung | Work | Passes when (measurable) |
|---|---|---|
| ~~**R0 = P2b**~~ **DONE 2026-07-29** | Function-level census: record (production, byte, offset) at each decode rejection; aggregate per-feature histogram over the 871-TU workload | **Passed**: `c2rs gap` and `c2rs census` print a per-feature histogram whose in-class + blocked counts sum to the **2,462,571**-function denominator (`LO`-marker anchored — **not** the ~902,730 `.gl` name count an earlier revision of this row demanded; see §2). Measured: 7,114 in class (0.29%) at the time, 1,237 blocking features, §2b |
| ~~**R0b**~~ **DONE 2026-07-29** | Characterize the `26 <tok>` / `B9` grammar behind `call-token-0xB9` (14.8%) and `call-anchor-0x00` (9.6%) | **Passed**: `docs/IL_CALL_GRAMMAR.md` decodes all three variable-width encodings, the CALL token, callee resolution via `.gl` and the statement grammar, with every claim marked `[CF]` (controlled fixture) or `[DIR]` (real TU); `c2-il` now decodes the CALL token (2870fc1). Outcome: `call-token-0xB9` = member/indirect calls (**out of class by construction**, not a rung); `call-anchor-*` was the port's own bug and is **gone**; census 7,114 → **7,954** |
| ~~**R1**~~ **DONE 2026-07-29** | Empty-module acceptance + empty-TU obj emission | **Passed**: gap-scan **match 0 → 5/878** — the first nonzero match bucket, and the §5 downstream tripwire **trips**. (The row originally demanded ≥10; the real count of zero-function TUs is 7, of which 2 are refused on purpose — see §2. A rung target set from an estimate is not a gate; the gate is "nonzero, with 0 mismatch".) |
| ~~**R2**~~ **DONE 2026-07-29** | Empty **function** bodies — the `body-0x3A` bucket | **Passed**: `w10_empty_fn.cpp` exact; census **7,954 → 78,028 (0.32% → 3.17%)**, the largest single jump so far from the smallest class. The `IL_CALL_GRAMMAR.md` §4.2 trailing-expression variant still rejects, because the whole-body parse must still reach the segment end |
| ~~**R3**~~ **DONE 2026-07-29** | COMDAT `.text` per function under `/Gy` — forced by R2, which turned it into a live mismatch | **Passed**: `system/utl/Spew.cpp` is the **first function-bearing real TU to match**; gap-scan **match 5 → 6/878**, mismatch back to 0. See the `/Gy` box in `ROADMAP.md` §G5 — this was a missing emitter *input*, not a missing class |
| ~~**R0c = P2d**~~ **DONE 2026-07-29** | Characterize `expr-cast` (`0x40`, then 6.8%) — the largest bucket believed schedulable | **Passed, by refutation**: `docs/IL_CAST_CONVERT.md`, every claim marked `[CF]` or `[DC3]`. `0x40` is **not a cast**; it is a second CALL token (the intrinsic call), `call-token-0x33` is the same production with an assigned result, the real cast is `2C <TYPE> <varint>`, and `0x66` is the class-layout descriptor rather than `IL_CALL_GRAMMAR.md` §7's #1 unknown call opcode. A rung can pass by proving its premise wrong; that is a result, not a failure |
| **R4** — *head of the current ranking* | **Decode** the intrinsic-call production (`40 <TYPE>`, the `(<expr> 55 <TYPE>)* 4C` argument loop, the `66 02 <tok> <tok>` descriptor) — ~13% of blocked functions with `call-token-0x33` | Census: the two buckets collapse into one and report the intrinsic **id**, not one opaque byte; bodies where an intrinsic call is not the blocker reach their real blocker, so other buckets *grow* — that is the pass signal, not a coverage jump. **Acceptance is explicitly out of scope**: `parse_segment` still returns `NotImplemented`. Admitting any id needs the id **and** its argument literals pinned by controlled fixture (`IL_CAST_CONVERT.md` §1.4, §4.1) |
| **R5** | `read_varint`'s signed short form (GAP-1) — a decode defect, not a class | Small negative literals stop blocking; census rises by whatever they were worth. Needs the operand type threaded through for the 4-vs-8-byte escape |
| ~~**W5 chains**~~ **DONE 2026-07-29** | `*`/`-` chains past 2 ops: descending cursor `r11→r10→r9…`, refused below `r9` | **Passed**: `w5_chain.cpp` `Mismatch → Match`; this is the mis-emit fix of §1 |
| ~~**W5 trees, depth 2**~~ **DONE 2026-07-29** | Depth-2 multi-scratch trees: left child into one scratch, right into another, root into `r3` | **Passed on the fixture gate, FAILED on the census gate**: `w5_tree2.cpp` is `Port=Match` on all four shapes, all 11 `w5_tree_neg.cpp` functions still `NotImplemented` — and the census moved by **0**. By this table's own rule ("a rung whose fixture gate passes but whose census gate doesn't move is **not done** — it modeled a shape the real corpus doesn't contain") that is a finding, not progress, and it is recorded as one. The depth is capped by the unexplained `+`-root register swap (GAP-2) |
| **W5 trees, depth 3+** | Deeper trees (the liveness-gated, wrapping cursor + level-order emission of `CODEGEN_W5_SCRATCH.md` §7) | `w5_tree3.cpp` exact; all 11 functions of `w5_tree_neg.cpp` still `NotImplemented`; the `+`-root swap explained rather than re-observed; census multi-scratch buckets move — and on the depth-2 evidence, **expect them not to**, so measure before building |
| ~~**W6 leaves**~~ **DONE 2026-07-29** | Compare→bool materialization, branchless, `k == 0` folds first | **Passed**: `il_bool_materialization.cpp` `Port=Match`, 6/6 in class. Census movement was small — the honest reading is that this rung was fixture-driven, not demand-driven |
| **W6 rest** | `<`, `<=`, `>=` against a non-zero literal; wide literals | The §4.5 spine's instruction order for a literal lhs is **pinned by capture** (it is currently UNRESOLVED, `CODEGEN_W6_COMPARE.md`), then exact |
| **W7** | Shifts + bitwise + strength reduction | `shift_mask` exact; hazard-listed encoders land exact-pattern, opt-in; census `09`/`0B` buckets move |
| **W8** | Control flow (first CFG; GAP-4 restructure lands here) | `select_max` + conditional shapes exact; restructure merged with **0 mismatch** on fixtures *and* the full 878-TU scan; census branch-token bucket moves |
| **W9** | Div/mod (incl. const-divisor multiply-high) | census div bucket moves |
| **W10** | General frames + locals + per-function `.pdata` counters (W-UNW-1) | multi-function TUs with frames decode+emit; first TUs where *every* function is in-class flip to `match` — target population: the 40 TUs with ≤10 functions |
| **W11** | Calls generalized (args r3–r10, stack spill, multiple calls) | census call buckets move; match bucket starts climbing the ≤100-fn TU population (79 TUs) |
| **W12** | Memory / struct access (`.sy` becomes load-bearing) | census memory buckets (`30`/`32`) move |
| ~~**W13a**~~ **DONE 2026-07-29** | Float/double leaves over parameters — a **separate** FP register model, not a parameterization of the integer one | **Passed**: `mvp_fmul3.cpp` `Port=Match`; `w13_fabi/fops/fscratch/fneg.cpp` all replay `ByteExact` and all keep returning `NotImplemented`, which is what pins the boundary. Census 78,028 → **79,041** (+1,013) — small, and fixture-driven rather than demand-driven, like W6 |
| ~~**W13b**~~ **DONE 2026-07-29 at ONE constant per body** | Float **constants** — `.rdata` COMDAT per distinct value keyed on (bits, width), `addis`+`lfs`/`lfd`, REFHI/REFLO+PAIR quads, `__real@…` symbols placed after the first referencing function | **Passed on the fixture gate, ~0 on the census gate**: `w13b_fconst.cpp` and `w13b_fdedup.cpp` `Port=Match`; `w13b_fpool.cpp` (2+ literals) and `w13b_ffold.cpp` (the identity folds) keep refusing; census 79,718 → **79,719** (**+1**). The §2.6 cursor/constant interaction that this row named as a blocker turned out to be an *ordering* fact for one constant — the constant allocates before the interior temporaries — and is closed at that width; beyond one constant it is genuinely open, so the rung landed with the ceiling rather than through it. Two side-effects outweigh the coverage: the emitter-wide relocation-layout fix, and the finding that **c2 is the FP constant evaluator** |
| **W13b, 2+ constants** | Two or more *surviving* constants: the prologue `addis` group, `lfs` at first use, FP-register recycling — plus c2's constant evaluator (folds, reciprocal strength reduction, reassociation) | A capture set large enough to **separate candidate scheduling rules**, not just exhibit them — `p1`, `p5`, `n_k_two` and `p_const2::k4`/`k5` already give four different arrangements from four probes (`CODEGEN_W13_FLOAT.md` §5.6, §7.5). Do not implement from those. Given the +1 above, measure the demand before building |
| **W14** | Data sections / globals (`ADDR32` relocs) | census global buckets move; match bucket now tracks whole subsystems |
| **P-F0.2→P-F2, P3** | Front-end track + compose (parallel, off the match-bucket critical path) | Grade 1 `PortC1 == captured` per file; Grade 2 `PortC2(PortC1(src)) == pipeline obj`; `compose` timed |

**R0, R0b and R0c have all run, and the W-numbering above is not the running
order.** The measured head of the histogram (§2b) now ranks the schedulable
work as ~~R2 (empty function bodies, 4.4%)~~ *[done]* → ~~`expr-cast`
characterization~~ *[done, and it refuted the name]* → **R4** (the
intrinsic-call family, `expr-intrinsic-call` 7.0% + `call-token-0x33` 6.1% =
**~13%** of blocked functions, one production — the largest schedulable thing
on the board) → **W11** (remaining call shapes, 8.4%) → **W12** (non-`int`
operand types: float 3.4% + double 3.2% = 6.6%, plus `void*` 2.0%) → **W8**
(`body-0x53`, 2.9%), with W6/W7 absent from the top eight entirely, and R5
(the `read_varint` fix) cheap enough to slot in anywhere.

**Three cautions on that ranking — and the third is now earned by measurement
rather than argued.** First, **~13% is a decode rank, not an
acceptance estimate**: R4 buys census accuracy cheaply, but *accepting* the
production needs an allow-list of intrinsic ids whose argument literals are
also pinned, because c2's expansion turns on the literal values — one offset
byte apart is the difference between zero instructions and a null-guarded
four-instruction sequence. Second, the largest bucket of all,
`call-token-0xB9` at **15.3%**, is **not a rung**: member and indirect calls
have no relocatable callee name / need a `this` and vtable model, and
`IL_CALL_GRAMMAR.md` §6.2 requires them to keep failing closed. A ladder that
"fixes" them before W11/W12 exist would be trading a refusal for a mis-emit.
The attribution is inference; the acceptance-gate structure is what is fixed,
not the order.

Third: **the float/double rows are no longer W13b demand, and this is the first
time an attribution in this table has been falsified against the corpus rather
than reasoned about.** Earlier revisions read `expr-load-type-864540` +
`expr-load-type-888541` (6.6%, **156,558** functions) as "W13b/W12". W13b has now
landed and moved that pair by **one function**. So whatever those functions are,
they are overwhelmingly *not* FP leaves with at most one constant. Attribute the
rows to **W12** (memory/struct access — the `Box::Volume`-shaped float member
load lives there) until a census with a decoded FP grammar says otherwise, and
treat the episode as the cheapest possible demonstration of the standing caveat:
a bucket names the *first* thing that stopped the parse, and clearing the shape
you assumed it was does not clear the bucket.

Note also that the census numerator is the per-rung yardstick: R1 tripped the
match tripwire on empty TUs, but every remaining rung is judged by how many of
the **2,382,852** blocked functions it moves — and by which bucket they move
*to*, since a function's first blocker is rarely its only one.

Session discipline: one rung (or one census-bucket slice of a rung) per
session; re-run the scan at session end; keep every JSONL
(`work/dc3-workload/scan-YYYYMMDD.jsonl`, gitignored) so coverage is a
monotone, diffable series; update the baseline table in `ROADMAP.md` §G5 and
the populations here when they move.

## 5. The payoff contract — what downstream integration exists and when this
work starts paying

The consuming project (decomp-synth, the guided-search decompilation engine
this port was built to accelerate) assessed c2-rs for its frontier scoring
loop on 2026-07-29. Condensed verdict, so the payoff line is visible from
inside this repo:

- **At the time of the assessment: NO-GO for every frontier-scoring use.** The
  port covered 0 TUs of the real corpus and 0.29% of its functions, and every
  speed path through it is bounded by the c1xx front-end cost the scoring loop
  must still pay per *source* candidate (compiles are ~245 ms on PCH units,
  c1xx ≈ 45 ms of it; even a 100%-coverage backend caps the funnel at ≲2.4×
  without the front-end port). **That input has since changed**: the match
  bucket is 6/878 and the census is **3.24%** (§2, §2b). The verdict itself is
  the consumer's to re-issue — nothing in this repo may declare it re-opened —
  but the mechanical precondition it named has been met.
- **The only doctrinally-legal integration shape** (recorded so the target
  is stable): a **reject-only, fail-closed pre-filter**. The consumer treats
  the port's three-way verdict as: `NotImplemented` → full real compile (no
  saving, no risk); port-emitted **match** → full real compile *anyway*
  (every accepted result is still witnessed by the real toolchain — the port
  never mints a solve); only a port-emitted **mismatch** may skip a real
  compile, and skips are continuously audited by real-compiling 1-in-N of
  them. The port is never the judge; it is a fast way to spend less time
  proving negatives.
- **The byte-identity bar the consumer holds this repo to**: replay
  raw-identical **including the timestamp**, re-proven per-TU before any
  IL-derived result is used; the port byte-exact (timestamp-zeroed) per
  accepted class, with the mismatch bucket at a hard 0.
- **The tripwire that reopens downstream use — TRIPPED 2026-07-29.** The
  named signal was the gap-scan **match bucket going nonzero on real TUs**;
  R1, then R2+R3, took it to **6/878**, with the mismatch bucket back at a hard
  0 across all 878. That is the exact, mechanical condition the downstream
  assessment set for re-opening, and it now holds. It pays more with every TU
  the match bucket gains, so the honest framing is: the re-assessment is
  *unblocked*, and 5 empty TUs plus one small real one are the floor of what it
  will find, not a result.
- **Already-live secondary payoff**: c2-only replay as a scorer backend for
  any IL-space search lane is a GO-when-relevant (~days of wiring) — the
  871/871 replay-soundness proof is the asset, and it is done. Keeping it
  green (the `--replay-every` lane) protects banked value.

## 6. How to verify honestly

The commands that constitute "this rung is done". Run from the repo root;
toolchain via `scripts/fetch_compilers.sh`, wibo on `PATH` or sibling
`../wibo`; the workload inputs via `scripts/gen_dc3_workload.sh` (needs a
sibling `../dc3-decomp` checkout).

```sh
# 1. Fixture parity + fail-closed boundary + suite + perf (the fixture gate)
cargo test --workspace --release
cargo run --release -p c2-harness --bin c2rs -- diff
cargo run --release -p c2-harness --bin c2rs -- perf

# 1a. THE MODE-LANE GATE — every lane in scripts/lanes.txt, one result each.
#     This is ONE command and it is the whole lane set; do not hand-type a list
#     of modes. `c2rs diff` above hardcodes `/Ox /GS- /c`, so on its own it has
#     never compiled /EH, /Oi or /O1 at all. 12 lanes, ~6 s cold at --jobs 4,
#     ~1 s warm. It prints `N/N lanes ... M fixture-verdicts`; quote both, and
#     treat a run reporting 0 graded as a failure, not a pass. `--selftest`
#     needs no toolchain and proves the gate fails when it should.
scripts/gate.sh --jobs 4

# 2. The real-workload gap scan (the census + scan gates). Prints the TU
#    buckets, the FUNCTION CENSUS numerator, and the top-20 blocking-feature
#    histogram — and, since 2026-07-30, a PROVENANCE header (both trees' git
#    HEADs, the resolved toolchain paths, the wibo version) before any of it.
#    Captures are cached content-addressed, so this is ~47 s cold and ~1 s warm.
cargo run --release -p c2-harness --bin c2rs -- gap \
  --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --jsonl work/dc3-workload/scan-$(date +%Y%m%d).jsonl \
  --replay-every 25 --jobs 16

# 2a. …and the cache is never trusted without a sampling check. This re-captures
#     every 50th cache hit through the real toolchain and byte-compares it;
#     a disagreement is named per entry and exits non-zero. Run it whenever a
#     scan is about to be quoted, and use --no-cache to bypass entirely.
cargo run --release -p c2-harness --bin c2rs -- gap \
  --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --jobs 16 --validate-cache 50

# 2b. Single-TU census while developing a widening step: run it before and
#     after and watch named functions move from a blocking feature to a shape.
#     Prints a bracketed hexdump of the bytes at one blocking site per
#     feature — read it before believing any bucket. --keep-il drops the
#     captured bundle in a (gitignored) scratch dir for grammar work.
cargo run --release -p c2-harness --bin c2rs -- census system/world/Dir.cpp \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp \
  --keep-il work/il-scratch

# 3. Replay soundness at full strength (periodically, and before trusting
#    any IL-derived artifact): every TU, byte-exact including timestamp
cargo run --release -p c2-harness --bin c2rs -- gap \
  --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --replay-every 1 --jobs 16

# 4. The generated sweep (one axis at a time) and the CROSS-PRODUCT lane (every
#    accepted shape family beside every other, both orders). NOTE: cross_sweep
#    still carries its OWN four modes (packed, /Gy, /O1, /O2) rather than the
#    registry, so it compiles no /EH — a named, still-open instance of the
#    un-enumerated-lane defect (§7, docs/CROSS_PRODUCT.md).
#    The second is what #12 below says a merge needs and nobody was doing by
#    hand: it asks the port for its own family list, discovers a representative
#    of each by grading the sweep corpus, and fails by name on a family no
#    fragment can supply. ~70 s cold. See docs/CROSS_PRODUCT.md.
scripts/expr_sweep.sh
scripts/cross_sweep.sh
```

The rules that keep the numbers honest:

- **Byte-exact means byte-exact.** Replay: raw-identical including the COFF
  timestamp. Port: identical with only the 4-byte timestamp zeroed. No
  fuzzy thresholds anywhere; real c2 under wibo is the sole judge.
- **A histogram can be sharded, and a sharded histogram lies about its own
  head.** The `expr-load-type-XXXXXX` bucket name truncates the LEB type id to one
  byte, and derived-type ids are allocated *per translation unit* from 0x1000 — so
  the same six `std::exception` inlines census as `A6438B` in one TU and `A6438A` in
  another, and one class of blocker was split across hundreds of names. Regrouped by
  family, `A643xx` (const-pointer / `this` loads) is **750,421** blocked functions
  and `8643xx` is **294,810**: ~44% of all blocked functions, against the 304,813 of
  `expr-call-in-expr` that the histogram had been calling the head for weeks. No
  single shard ever looked large enough to lead. The lesson is not "regroup this
  bucket" — it is that **a bucket key derived from data the compiler allocates per
  input is not a stable key**, and any histogram built on one needs its key checked
  before its ranking is believed. Ask of every bucket name: could two occurrences of
  the same construct land in different buckets?

  > **It recurred, and it recurred because the regrouping above was done by hand
  > and the key itself was left broken.** The paragraph was written, the family was
  > summed once for one analysis, and every ranking taken afterwards went on reading
  > the sharded histogram. It hid a second rung's worth of movement: **82.9 % of the
  > address-leaf rung's +40,621 (−33,688) came out of `expr-load-type-*` shards that
  > no ranked list could attribute**, and that rung was therefore ranked entirely
  > from the 17.1 % that happened to have a named key. In one sample the plain
  > designator was refusing 928 functions against the intrinsic's 184 — a 5.0×
  > ratio, invisible.
  >
  > **Fixed 2026-07-30**: [`Block::feature`] renders `<tag> <kind>` and not the id.
  > 1,257,718 functions moved out of 848 shard names into 29 real ones, an exact
  > coarsening verified per TU *and* per frame class with zero residual, census
  > unchanged at 320,641/2,462,571. The corrected head is
  > `expr-load-type-A643` at **666,907** and `expr-load-type-8643` at **316,800** —
  > together **983,707, 45.9 % of blocked** — where the top *visible* row had been
  > `expr-intrinsic-this-adjust` at 141,800. (The 750,421/294,810 figures above were
  > taken at an earlier census; the population has since moved, not the finding.)
  > The general rule, restated so the next instrument gets it for free: **the
  > partition a census reports must be a function of the construct, never of a value
  > the compiler allocated per input.** `mcall`'s `aux` layout states it as an
  > invariant of its bit packing, which is why that family never sharded.
- **A counterfactual answers "is this bucket one shape?" in one second; sampling
  guesses at it.** The step-2 handoff ranked `callseq-tail-lit` (7,771) first and
  warned it was "one bucket holding several shapes, exactly the thing §6 warns
  about" — so the prescribed next step was to sample the bytes at its blocking
  sites and group them by production. Deleting the gate and re-scanning gave
  **+7,771 exactly**: one bucket, one cause, and the answer arrived before any
  sampling. The complement matters as much: the same instrument sized the rung
  that had been *ranked above it* at **+2**, and a third run with both gates
  lifted came back exactly additive, which is the only way to know the smaller
  rung was not being masked by the larger one. Since a warm scan is ~1 s, the
  rule is: **before sampling a bucket, lift its gate and re-scan — and lift it
  together with every gate that fires earlier on the same bodies.** Sampling is
  for buckets a counterfactual says are heterogeneous, not for deciding whether
  they are.
- **A first-blocker histogram attributes a construct to wherever the parse
  stopped, not to what the construct is.** Sampling 21,319 blocking sites showed
  `expr-call-in-expr` is ~80% *member calls* — and that the same member-call
  production also fills the `expr-load-type-xx43xx` pointer-load rows, which sum to
  1,127,384 functions, 47.9% of everything blocked. So **neither bucket's size
  measures the production**: the same construct is filed under a call bucket or a
  load bucket depending only on which operand the parser reached first. This
  compounds the sharding above rather than repeating it — sharding is an unstable
  *key*, this is an unstable *attribution*. Two consequences worth keeping: a
  bucket is not a work item until you have sampled the bytes at its blocking sites
  and grouped them by production; and clearing one production can shrink several
  unrelated-looking buckets at once, so predicted movement should be stated
  per-production, never per-bucket.
- **Fourteen live wrong-bytes emits and two live panics, none of them found by the
  fixture corpus.** Every one came from review, adversarial probing or a
  *generated* sweep axis, and most have the same shape: *two facts that happen to
  share one field until some construct pulls them apart.* Two of them break that
  pattern in a way worth naming separately — see #9–#10: *one rule, two
  implementations, and the corpus only ever exercised the correct one*. #11
  returns to the original shape, in the obj shell rather than in an instruction;
  **#12 is #11's own field one consumer later** — the sharpest instance of #2 the
  project has produced — and **#14 is that same field one PRODUCER out**, which
  makes `_fltused` the single most productive four bytes in this list.
    1. The `this` token was located by a bare first-`0x46` search. That byte is also
       the payload of the line marker for **source line 70**, so a member function
       there lost its `this` and every formal dropped a register.
    2. `parse_this_token` was then fixed — in the one shape where the bug had been
       found. Straight-line bodies, tail calls, comparisons and float leaves went on
       mapping formals from r3, so `int S8::m(int x) const { return x + 1; }`
       emitted `addi r3,r3,1` for `addi r3,r4,1`. "One fact, one locator" had been
       obeyed to the letter: **a locator nobody consults is not shared.**
    3. A TYPE's tag carries **alignment** and its kind carries **size**. Equal for
       every naturally-aligned type, so reading the tag was indistinguishable from
       reading the size until `#pragma pack(4)` put an 8-byte `long long` behind a
       4-byte tag, and one `lwz` landed at the wrong offset.
    4. A formal's **index** in the formals list stood in for its **argument-register
       number**. The same number for every scalar parameter, so the two were
       indistinguishable until a by-value aggregate wider than 8 bytes took more than
       one GPR: `int gb(Big v, H* h) { return h->mi; }` emitted `lwz r3,0(r4)` where
       c2 emits `lwz r3,0(r6)` (`il_param_aggr_neg.cpp`).
    5. **The fifth instance is not a wrong-bytes emit but a panic**, found
       2026-07-30 while probing the de-sharded census. A formal's **index in the
       formals list** stood in for its **argument-slot index** in the multi-argument
       tail call's permutation analysis — equal whenever the call passes every
       formal, and the corpus had no call that skipped one. `int g(int,int); int
       f(int a,int b,int c){ return g(a,c); }` gives sources `[0, 2]` over two slots
       and indexed a `seen[]` of length 2 with 2: `c2rs census` **panicked**, on
       mainline, on two lines of ordinary C++, against a hard constraint that says
       the CLI degrades cleanly and never panics. The 878-TU scan was green through
       it because those bodies block earlier on their operand types — **a green scan
       is green only on the IL it saw**, and this one had never seen a call that
       skipped a formal. Now refused as `call-arg-outer-formal`; pinned by
       `ARG2_OUTER_FORMAL` with the formals-0-and-1 permutation as its control.
    6. **A formal's index in the formals list stood in for its FP-argument
       register number.** `float_leaf_text` maps parameter `n` to `f(n+1)`, and
       the floating-point file is numbered over the **FP parameters alone**, so
       every parameter list that was all-`float` or all-`double` made the two
       numbers equal — and every FP fixture in the corpus was one of those.
       `float mixfp(int a, float b, float c) { return b*c; }` emitted
       `fmuls f1,f2,f3` where c2 emits `fmuls f1,f1,f2`, on mainline, with all
       four mode lanes and the 3,743-case sweep green (found 2026-07-30 while
       probing the register-move rung). The remarkable part is that
       `w13_fabi.cpp` **states the rule in a comment and carries the failing case
       as `fp_skip`** — it hid because that TU holds an out-of-class function and
       the port is all-or-nothing per TU, so those bytes were never emitted.
       **A characterization fixture that documents a rule the emitter does not
       implement is not a test of it**, and a whole-TU gate can hide a per-
       function mis-emit indefinitely. Put the case in a TU of its own.
    7. **The seventh is the second instance's shape again, in the other register
       file.** A bare `return <parameter>` that is not the first is one register
       move; the integer class has gated exactly that since it was written
       (`straight_line_out_of_class_ctx`), and the FP class never got the gate,
       so `float f(float a, float b){ return b; }` emitted **nothing** where c2
       emits `fmr f1,f2`. Found in the same probe as #6 and fixed by the same
       rule (every formal must be an FP operand of the body). "A locator nobody
       consults is not shared" now has two instances, four years of code apart in
       spirit and two functions apart in fact.
    8. **The eighth is the fourth instance's shape in the framed call's argument.**
       `framed_call_text` emitted one byte-constant 0x24-byte body; the parser
       required the call's argument to be *a* formal and then **dropped the
       formals list**, so the emitter assumed the formal already in r3. c2 emits
       `or r3,rN,rN` first whenever it is not, and the `.pdata` `FuncLen`, both
       `$M` label values and the REL24 site all followed it wrong.
       **37 of 47 probes around the accepted class mismatched** — every argument
       at a non-zero formal position, every member function (`this` takes r3, so
       a one-parameter member's argument is in r4), and every free function with
       a leading `float`, `double`, `long long`, pointer or 8-byte aggregate
       parameter, each of which takes one GPR slot on this ABI. Past the eighth
       formal it is not a register move at all but `lwz r3,180(r1)`, which the
       old emitter also answered with nothing.
       It hid for a reason that is now boringly familiar and was *visible for
       free*: every framed fixture and **all 363 generated framed cases** are
       `int F(int a) { return g(a) + 1; }` — one parameter, necessarily in r3 —
       so the argument's index and its register were the same number everywhere
       the class had ever been graded. Four mode lanes, a 4,706-case sweep, an
       878-TU scan and a green `cargo test` were all green over it. Found by
       compiling the neighbours of a shape the rung was about to rewrite, which
       is the same method that found #6 and #7. The tell available *before*
       compiling anything: **`framed_call_text` took no parameter that could
       distinguish two formals**, so it could not have been emitting a
       formal-dependent word, and the class it served plainly had formals.
    9. **One rule, two implementations, and each copy was missing a gate the
       other had.** The direct `return g(…)` form and the bound-to-a-local form
       (`int z = g(…); return z;`) both validate a call's arguments, in two
       copies, and the copies had drifted apart in **both** directions:
       * the bound form never asked [`leaves_ascending`], so
         `int f(int a,int b){ int z = g(b + a); return z; }` emitted
         `add r3,r4,r3` against the reference's `add r3,r3,r4` — c2 canonicalizes
         a commutative argument's leaves, so `g(a+b)` and `g(b+a)` are the *same*
         obj. `c2rs diff` read `Port=Match` for `a+b` beside
         `Port=Mismatch @ 537` for `b+a`, **two lines of C++ that differ by one
         transposition**;
       * the bound form also never got instance #5's `call-arg-outer-formal`
         gate, so `int f(int a,int b,int c){ int z = g2(a, c); return z; }`
         **panicked** `c2rs census` with the identical `index out of bounds: the
         len is 2 but the index is 2`. **The same defect, in the same file, four
         months and one fix apart, because only one of the two copies was
         repaired.**
       Both closed by making the two copies one (`tail_call_shape`, which the new
       statement-call sequence also uses). Fixture `il_call_bound_neg.cpp`, with
       the canonical-order body as its control. Zero functions on the workload —
       which is exactly why nothing saw it.
    10. **A rule fitted to the shapes the corpus happened to contain.**
       `permute_args_text` lowers a multi-argument call's register permutation by
       decomposing it into cycles and walking each with one temp (r11). Measured
       over **complete** grids rather than sampled — all 24 permutations of a
       four-argument call, and all 84 single cycles of length 2–5 inside a
       five-argument one — that rule is right at cycle length 2 (0/10 wrong) and 3
       (0/20), and **wrong at 4 (10 of 30) and 5 (16 of 24)**. Past three, c2
       hoists a *second* save into r10 and reorders the writes:
       `int f(int a,int b,int c,int d){ return a4(c,d,b,a); }` is six moves and
       two temps against the port's five and one, so the obj came out four bytes
       short and diverged at offset 8. **Live on mainline**, in the plain
       multi-argument tail call, with nothing framed about it.
       It hid because twenty of the thirty four-cycles *do* agree with the minimal
       walk, and because the two fixtures that grade this class
       (`il_call_perm.cpp`, `il_call_multi.cpp`) between them contain no cycle
       longer than three — `rev4`, the fixture written to be the hard case, is two
       disjoint 2-cycles. Now gated at the measured edge (`call-arg-long-cycle`);
       what c2 does past three is *described* by the grid and not explained, so
       the boundary is drawn where the evidence stops rather than fitted to six
       data points. Cost on the workload: **0 functions.**
       The generalizable bit, and it is not the same as #1–#8: a rule can be
       *derived* from captures, *verified* against every fixture, and still be
       wrong on a region the corpus never entered — and the cheap way to find out
       is to enumerate the parameter's whole range (here: all cycle lengths) rather
       than to add one more hand-picked case.
    11. **The eleventh is the same shape in the OBJ SHELL rather than in an
       instruction**, found 2026-07-30 by the FP-store rung's own fixture on its
       first run. A translation unit that touches floating point carries an
       undefined external `_fltused`, and the port keyed that on
       `coff::Function::is_float`, set from `float_leaf.is_some()`. That field
       was answering two questions — *"this body does FP arithmetic, so its label
       stride is 2"* and *"this TU needs the CRT's float-support hook"* — and
       every function that had ever set it satisfied both, because the only FP
       class the port had was the W13 arithmetic leaf. An FP **store**
       (`void f(S* s, float v){ s->f = v; }`) satisfies only the second: it is a
       store leaf, stride 1, and it needs the marker. The port emitted **all
       fourteen** positive objs one symbol short — `Port=Mismatch @ offset 12`,
       the COFF header's `NumberOfSymbols`. Two things are worth keeping. First,
       the tell was in the same place as always and needed no compiling:
       **`is_float` had one producer and two consumers asking different
       questions.** Second, an all-FP-store TU cannot separate "the first
       FP-touching function" from "the first FP-arithmetic function" — only a
       *mixed* one can, so the placement rule was re-captured over four orderings
       (`fixtures/cpp/w28_fltused_order.cpp`) rather than assumed to survive the
       widening. See `docs/CODEGEN_FP_ARGS.md` §4.
    12. **The twelfth is the eleventh's field, one consumer later, and it was
       found only because two agents' work was merged.** Splitting `is_float`
       into `touches_floating_point` fixed the `_fltused` consumer and left the
       other one — `IlFunction::label_slots`, which still read `float_leaf` and
       so gave the FP **store** leaf a compiler-label stride of **1** where c2
       gives it **2**. A framed function's `$M`/`$T` numbers come from a counter
       every function in the TU consumes, so the framed function downstream got
       `$M2564/$M2563/$T2565` against the reference's `$M2565/$M2564/$T2566` —
       six wrong bytes in an obj that still links. The rule, MEASURED as the
       three-way capture that separates the two candidates:

       ```text
         void lead(S* s, int v)      { s->i = v; }     $M2558 $M2559 $T2560
         void lead(S* s, float v)    { s->f = v; }     $M2559 $M2560 $T2561
         float lead(float a, float b){ return a * b; } $M2559 $M2560 $T2561
       ```

       The stride goes with the **register file**, not with the body shape.
       This is instance #2 in its purest form — *fixed in the one shape where the
       bug had been found* — and the tell was, again, free: splitting a field
       means auditing **every** reader of it, and `grep` for `float_leaf` would
       have shown two.
       What is new here is *when* it became reachable. **Neither agent's corpus
       could contain the case**: the label counter has an observable effect only
       when a framed function follows, and until Class A many-calls (#35 step 2)
       landed there was no framed shape that could share an in-class TU with an
       FP store — the FP rung's fixtures have no framed function and the framed
       rung's have no floating point. It existed only in the *merge*, and it was
       found by deliberately compiling the cross product of the two rungs before
       trusting the merged tree. The practice that generalizes: **a merge of two
       independently-green branches is a new corpus, and the shapes only it
       contains have never been graded by anyone.** `scripts/expr_sweep.sh` now
       generates that cross product (six leaf kinds x three call bodies x three
       orderings) rather than relying on someone thinking of it again.
       **Generalized 2026-07-31** to every family rather than these two:
       `scripts/cross_sweep.sh` asks the port for its own list of accepted
       shape families (the `FnVerdict::InClass` labels), discovers a
       representative of each by grading the whole sweep corpus, and compiles
       every ordered pair of them — both orders, diagonal included — plus the
       arity axis #13 needs and every ordered triple over the families that
       carry a TU-level external, in packed / `/Gy` / `/O1` / `/O2`. 4,901
       configurations, 19,604 gradings, **0 mismatches**, and **86 of the 171
       unordered family pairs occurred in no matched TU of the fixture corpus
       or the whole sweep corpus** — nothing had ever graded them. Two things
       it found on its first run, neither of them a mis-emit and both of them
       holes: the three `call-sequence*` families — *the* class that made #12
       reachable — had **no single-family case anywhere** in 5,922 generated
       cases, since every TU that reached them carried a second function
       (`scripts/sweep.d/71-call-sequence.py` closes it); and 8 family pairs
       never emit in **any** configuration, at any arity or mode — every FP
       family beside every framed or call-sequence one, which is exactly #12's
       configuration and exactly #13's outstanding debt. The lane can prove the
       port does not mis-emit those only because it emits nothing at all, and
       it says so out loud rather than counting them green. `docs/CROSS_PRODUCT.md`.
    13. **Not a thirteenth emit — the TWELFTH's repair, wrong one row further
       out.** #12 was fixed by giving any FP-touching function a label stride of
       2. That fits the capture it was taken from — *one* FP leaf ahead of one
       framed function — and predicts **4** slots for two FP functions where c2
       gives 3, and **6** for three where c2 gives 4. It never emitted wrong
       bytes, because the TU-level gate refused the pair it would have applied
       to; it was **latent**, and it would have gone live the moment that gate
       came off, which is exactly what the next rung did. Re-measured seed-free
       (the *difference* between two framed functions' labels in one TU, so the
       `.gl` seed cancels), the rule is **one slot per function plus one for the
       TU if anything touches floating point** — the `_fltused` external's slot.
       `docs/ROADMAP.md` §6m. (That measurement stands; the *unification* built
       on it — "one slot per TU-level external", matching §4.4's **two** for the
       `__savegprlr`/`__restgprlr` pair — is **refuted**, and is itself a fourth
       instance of this section's shape: a rule fitted to the two points it was
       derived from, correct at both, wrong just outside. `LABEL_COUNTER.md`
       §2.1; the surcharge table that fits is §1.1.)
       Three things worth keeping. **A per-function method cannot hold a per-TU
       quantity**, and `IlFunction::label_slots` is one — at `n = 1` the two
       formulations are indistinguishable, which is why the wrong one looked
       right and why no single-FP-function probe could ever have separated them.
       **A fix for a mis-emit deserves the same enumeration the mis-emit got**:
       #12 was found by a cross product and repaired from a single row.
       And **a gate that hides a wrong rule is a debt, not a fix** — the refusal
       that kept this latent was itself recorded as a handoff, so the wrong rule
       and the thing that would expose it were scheduled together.
    14. **The fourteenth is #11's field one PRODUCER further out, and it had been
       live on mainline for as long as the void tail call has existed.**
       `float gf(); void f() { gf(); }` is a bare `b ?gf@@YAMXZ`. It touches no
       floating-point register at all — the result is discarded — and its obj
       still carries the undefined external `_fltused`. The port emitted one
       symbol too few: **`Port=Mismatch @ offset 12`**, the COFF header's
       `NumberOfSymbols`, exactly #11's failure at exactly #11's offset.
       #11 split `is_float` into `touches_floating_point` and #12 fixed its
       second *consumer*; this is its second **producer**. The predicate
       enumerates the shapes whose own *body* does FP work — the float leaf, the
       FP tail call, the FP store — and a body that merely **calls** an
       FP-returning function does none of them and still needs the hook. So the
       tell was in the same place as always and needed no compiling: a predicate
       named "touches floating point" that is defined as a list of *shapes*
       cannot be complete, because the property is about the whole translation
       unit and the list is about this port's grammar.
       Bounded by probe rather than guessed: `float`, `double` and `long double`
       results mis-emit; `float*` does not; an FP *argument* does not (the FP
       tail call marks the function itself); and merely declaring the callee
       without calling it does not. Refused under `call-ret-fp` at the one
       locator every call shape goes through — modeling it would mean claiming
       that `_fltused`'s measured placement rule and the per-TU label-counter
       surcharge extend to a new kind of FP-touching function, and neither has
       been captured. Cost: **0 functions** on the workload.
       **How it was found is the transferable part.** It came from a generated
       sweep axis W36 added — *the callee's return type, crossed with discarded
       and returned* — on that axis's first run. Nothing had ever varied it:
       every call in the fixture corpus, in all 10,194 pre-existing sweep cases,
       in four mode lanes and in the 878-TU scan returns `void`, `int` or a
       pointer. A green scan of 2.4 million functions is green only on the IL it
       saw, and the *return type of a callee* is a property no census bucket has
       ever been keyed on, so nothing could have reported its absence.
    15. **The fifteenth is `is_volatile_tag` at a THIRD position, and it had been
       live on mainline since the store leaf landed (W25).** A `volatile` stored
       VALUE is a memory object: `void f(Q* s, volatile int v){ s->a = v; }` is
       `stw r4,28(r1) ; lwz r11,28(r1) ; stw r11,0(r3)` — c2 homes the parameter
       in the frame and reloads it, so the body is not a leaf at all — where the
       store leaf emitted the bare `stw r4,0(r3)`. `Port=Mismatch @ 12`/`@ 8`, on
       two lines of C++, in class, with `census` reading 1/1. Instance #13 put
       the gate on the base LOAD; W35 then measured that the same bit at the
       `27`/`30` designator positions is **free** and wrote that down; and the
       VALUE position was never asked. So this is not "one fact, two locators" —
       it is one fact with **three** positions, two of them examined and one of
       them never named. The tell needed no compiling: a predicate whose comment
       enumerates the positions where it matters, applied at a strict subset of
       them. Found 2026-07-31 by W37's generated cv-qualification axis, which
       varies a qualifier that changes no operator and no shape. Cost of the
       repair: **0 functions** on the workload.
    16. **The sixteenth is the EMPTY PREFIX, and it is the only one on this list
       created by the rung that found it.** Generalizing `store_leaf_text` from
       an *exact* three-op pattern match to a **loop** over op groups added a
       case the exact match did not have: the empty `ops` slice, which every
       shape whose data lives in another field (`IlFunction::compare`) presents.
       Those walked past the loop with nothing matched and came out as a bare
       `blr` — every comparison leaf sharing a TU with a store, four bytes where
       c2 emits seven instructions. Caught within the hour by the one fixture
       that puts a compare leaf in the same TU as a store
       (`w29_fp_contract.cpp`, `Port=Mismatch @ 8`); `w25_store_leaf.cpp`, the
       store rung's own fixture, has no compare and was green over it. Two
       things generalize. **Turning an exact match into a prefix match adds the
       empty case, and the empty case is never the one the rung is about** —
       so a generalization of a pattern needs its degenerate input tested, not
       just its new one. And the fixture that caught it is a *cross-shape* TU,
       which is the same argument `scripts/cross_sweep.sh` makes at corpus scale:
       the shapes a merge or a generalization newly puts side by side have never
       been graded by anyone.
  What the corpus had in each case was the *safe half of the pair*: member functions
  with load bodies but not straight-line ones, straight-line bodies in free functions
  but not members, `long long` at natural alignment but never packed, for #4 not one
  parameter in the entire fixture corpus that was anything but a scalar, and for #5
  not one call that passed a strict subset of its caller's formals, and for #6/#7
  not one FP parameter list that was anything but uniform. A hand-written
  corpus is biased toward the shapes whoever wrote it was thinking about, and it is
  biased in a way that is invisible from inside it. Two practices follow, and both
  paid off the same day: **sweep the cross product, not one axis at a time**
  (`scripts/expr_sweep.sh` — the member-function-across-source-lines axis exists only
  because of #1), and **have someone adversarial read the anchors**, because #2 was
  found by a reviewer assigned to an unrelated change.
- **A fixture that states the rule and carries the failing case can still grade
  nothing.** `w13_fabi.cpp` documents in its own comment that the FP file is numbered
  over **floating-point parameters alone**, and it contains
  `float mixfp(int a, float b, float c){ return b*c; }` — which emitted
  `fmuls f1,f2,f3` for `fmuls f1,f1,f2`. It never fired, for a mundane reason: the
  port emits an obj only when EVERY function in a TU is in class, and that file has
  an out-of-class sibling, so the whole TU graded `NotImplemented` and the failing
  case inside it was never compared. A second emit hid the same way — `float f(float
  a, float b){ return b; }` emitted **nothing** where c2 emits `fmr f1,f2`. Both were
  live on mainline for as long as the fixture existed, and both are the session's
  recurring shape: a formal's *index* standing in for its *register number*, this
  time in the FP file rather than the GPR one.
  Two consequences. **Check `census` says `N/N` before believing a fixture proves
  anything** — a positive case sharing a file with a refused one is decoration. And
  when a fixture's comment states a rule, that rule needs a case in a file that
  actually grades, which usually means its own TU.
- **A grammar measure cannot see a codegen construct the grammar does not
  distinguish.** The whole-body-complete count has ranked three rungs correctly —
  each converting 1:1 with the bucket drop equalling the census gain exactly — and it
  is still blind in a way that cost a whole rung. `data-addr` looks like one shape to
  the grammar whether the call materializes one address or three; **87.4% of that row
  passes two addresses**, and c2 emits one `lis`/`addi` pair per *function*, deriving
  the rest as `addi rD, rAnchor, <difference of pool offsets>`. Instruction selection
  therefore depends on a whole-translation-unit pool layout, which no per-body
  grammar can express. Estimate 11,000, outcome **0**; the bias direction was called
  right and every named deduction was real, and none of them was what stopped it.
  This is a *different* blindness from the second-blocker gap: that one was "what
  else blocks this body", this one is "what does the emitter have to know that the
  parser never asks". Before committing to a row, check that its dominant sub-shape
  has a **local** lowering — and note the tell, which is available cheaply: several
  byte-identical source functions in one TU emitting different instruction
  sequences means the decision is not local.
- **When a measure has a population it cannot be wrong about, report that
  population every run.** The frame measure (`docs/IL_CALL_IN_EXPR.md` §18) counts
  CALL tokens per body outside the grammar, so it is a byte walk with no parse to
  fail closed. Its first version counted a `BD` whenever the following bytes were
  merely TYPE-shaped, and it was 98.0 % right against the reference objs — which
  looked fine. It was also reading **10,088 in-class LEAVES as two-call bodies**, and
  that was visible for free: a shape the whole-body parser *accepted* as a leaf
  cannot contain two calls, so the in-class functions are a standing control group of
  280,020 that the measure must place exactly. Requiring every field of the CALL
  token literally — conv `00`, the `80` escape form, id ≥ 0x1000, each a field that
  never varied over 15,095 wild sites — took the control group to **0** and the obj
  grade to 98.7 %. The general rule: a diagnostic that runs outside the parser has no
  fail-closed behaviour of its own, so **give it a population whose answer is already
  known and print the disagreement in the same report as the result.** An obj-graded
  sample of 705 said "good enough"; the control group said "wrong", and only one of
  them was cheap enough to run on every scan.
- **A count of things the compiler *reads* is not a count of things it *emits*, and
  the census is the former.** `src/lazer/meta_ham/HamUI.cpp` has **9,551 function
  bodies in its `.ex`** (9,551 `4C 4F 11` markers, 2.76 MB) and c2 emits **350
  functions**; corpus-wide, 2,462,571 IL bodies against **178,969 emitted — 7.3 %**.
  Both denominators are legitimate and they answer different questions: the port's
  gate is all-or-nothing over every segment, so the census's denominator is the right
  one for "will this TU come in class", and the emitted count is the right one for
  "how much of the output does this construct produce". Mixing them silently is the
  hazard — §18.5 applies a framed share measured on emitted functions to a population
  of IL bodies and says so at the point of use. The related latent question, still
  open: the port has no model of *which* bodies c2 emits. It fails closed today only
  because `.gl` binds fewer names than there are segments, which is a gate doing this
  work by accident.
- **A compiler-GENERATED body has no freedom, so its grammar bound is nearly
  tight.** Two rungs in a row over generated destructors came in *under* their point
  estimate (15.7% and 30% low) while staying inside their upper bound (93.5% of it,
  the second time). The reason is structural: an estimate is discounted from the
  bound for gates the grammar adds — literal offsets, exact trailers, argument
  counts — but a body the *compiler* wrote has no latitude in any of them, so those
  gates exclude almost nothing. For hand-written bodies the discount is real; for
  generated ones, quote the bound minus only deductions you have measured a
  population for. The corollary is that the whole-body-complete count is a genuinely
  predictive instrument here: on the member-destructor rung the two `-whole` buckets
  fell to 0 and 587 and their drop equalled the census gain exactly.
- **Estimate the fix, not the finding.** A rung's estimate is scoped to the call
  site it was measured at, and the same defect often sits at more than one. The `66`
  class-pair descriptor's refs were being stepped as a fixed two bytes in *two*
  places; the measured estimate (10,469, from the census key that counted
  whole-body-complete destructors) covered one of them, and the realized yield was
  **+25,395** — 9,637 from the estimated site and **15,758** from
  `try_parse_base_member_load`, which no estimate had covered. The prediction "fewer
  than 10,469" was correct for its scope and wrong by 2.4x overall. Before quoting a
  number, `grep` for every site that implements the same rule.
  This also revises a figure this document has used as a cautionary tale: decoding
  intrinsic 2117 "moved 32 of its 149,200 functions" partly because *this* bug was
  refusing what it admitted — not only because those bodies are non-leaves.
- **A residue that makes no sense is a measurement.** That descriptor bug was not
  found by a probe; it was found because a census split spread 17,757 functions over
  **197** distinct `op-0xNN` buckets. Flat over the byte range is the signature of
  reading a **payload as vocabulary**, and no amount of narrow probing would have
  shown it: `66 02 92 20 93 20` is consistent with fixed-2, with LEB128 *and* with
  `read_token_var`. Only the wide witnesses (`fb 8a 01`, `ff ff 01`, `d3 80 02`)
  separate them, and only a corpus large enough to contain wide type ids has any.
  When a histogram has a long flat tail of hex buckets, suspect the parser before
  the vocabulary.
- **A green differential is not evidence that a *binding* is right.** `.sy` used to
  bind its blocks to `.ex` segments only when the counts were equal, and on the
  workload they are close but not equal (9,629 against 9,602). Relaxing that to "take the
  first `n_segments`" measured **census +2,981 with 0 mismatch** — a clean green run
  by every gate this project has. It is also wrong: the per-formal token lookup then
  fails for **343,315 of 554,056** functions, because the surplus blocks are
  *interspersed* rather than a tail, so 62% of the bindings were attaching one
  function's data to another. The mismatch count stayed 0 for a mundane reason —
  those functions refuse for other reasons and never reach an emitter, so a wrong
  binding has nothing to be wrong *about* yet. It becomes a wrong-bytes emit the day
  their other blockers clear. The lesson generalizes past `.sy`: when a change makes
  a *correspondence* rather than a *decode*, the oracle cannot grade it, and the
  thing to measure is the correspondence's own invariants (here: does every `.ex`
  formal token appear in the block it was bound to?). A relaxation that improves the
  census and keeps the oracle green is exactly the shape a plausible-but-wrong
  binding takes.
  **How it was then closed, since the same invariant is what graded the fix** (see
  `func::sy::SyLocals`): the binding is now keyed on identity — a block's header
  token is its segment's *exit label* — and the acceptance evidence is four
  measurements over 871 TUs, none of which the oracle could have supplied.
  2,434,636 of 2,434,639 segments yield an exit label; each of those tokens names
  **exactly one** block (0 misses, 0 ambiguities); the bindings are strictly
  increasing in **every** file (0 order violations); and the formal-token invariant
  holds for 99.95% of the candidate pairs, against 38% under the positional
  relaxation. The 1,118 that fail it are refused, so 100% of the bindings actually
  made are ones the invariant confirmed. The census moved 211,012 → 228,298
  (+17,286, 0 functions lost, 0 TUs changing class) with mismatch 0 — but the
  mismatch-0 is *not* the evidence, and this bullet exists to keep that distinction.
  **A second binding was made on this rule and it is the worked example of it**
  (D14, `ROADMAP.md` §6e). `gl_symbol_index` anchored a `.gl` record on "the name
  is the run right after a NUL", and 9,028 generated destructors had a callee whose
  record uses a **second separator byte, `26`**, which that anchor cannot see. The
  fix decides which symbol a token names, so it was graded on four things the
  oracle could not have supplied, and the order they are listed in is the order of
  their strength. **(a) Framing identity**: two adjacent records of the same class
  differ in exactly one byte — `80 75 14 00 00 00 00 04 84 30 **00** ??YString@@…`
  against `80 85 14 00 00 00 00 04 c2 30 **26** ??_GString@@…` — so the token field
  is at the same offset in both, and a different field would have to occupy the
  same position in the same layout. **(b) A population whose answer the SOURCE
  fixes**: a generated empty destructor delegates to a sub-object's destructor, so
  all 35,946 in-class ones must resolve to a destructor mangling, and all 35,946
  do, with 0 exceptions — a misread field would name arbitrary symbols. **(c)
  Injectivity, three-valued**: a token two records disagree about is dropped rather
  than bound to the first. **(d) A counterfactual**: the identical binary with `26`
  removed gains 0, so the whole +9,028 is that byte and not the rest of the
  rewrite. What is NOT closed is stated too — no fixture grades a `26`-form binding
  through an obj, because eleven probes failed to reproduce the form in a
  controlled TU. The general form worth keeping: **when the oracle cannot grade a
  correspondence, look for a population whose answer the SOURCE LANGUAGE fixes**
  (here: what a generated destructor is allowed to call), because that is evidence
  the container and the compiler both have to agree with.
  One more thing fell out of grading the correspondence instead of the output: the
  "surplus" blocks were never surplus. `.sy` has exactly one block per `.ex`
  **function tail** in all 856 files that parse; it is
  `bundle::split_function_bodies` that finds 2,462,571 bodies where there are
  2,464,543 tails, so the census denominator is itself ~1,972 functions short and
  the "extra" blocks are the ones it misses. A count that disagrees does not tell
  you which side is wrong.
- **An instrument that silently drops cases reports a pass it did not earn.**
  A block added to `scripts/expr_sweep.sh` used `for n in range(...)` as a loop
  variable; `n` is the generator's own file counter, so the loop rewound it and
  the next 1,233 cases **overwrote already-written ones**. The sweep then ran
  green over 2,610 cases where it should have run 3,843 — a pass, on a third
  fewer inputs, with nothing in the output saying so. The only tell was the
  printed case count *falling* against the number the last session recorded, and
  that count exists only because a previous session printed it. **A generated
  corpus must report its own size on every run, and that size has to be compared
  against the last one**; "0 mismatches" over an unknown denominator is the same
  vacuous green as a sweep with the toolchain absent, which this script already
  guards against explicitly.
- **A measurement artifact read from the wrong tree.** The parallel-agent workflow
  gives every worktree a reflinked copy of `work/`, so the same *relative* path
  (`work/dc3-workload/scan-t1.jsonl`) exists in several trees holding different
  data, and which one a shell tool reads depends on the working directory it was
  last left in. That produced a published "this rung measured +0" for a rung that
  measures **+88,116** — a wrong conclusion with a plausible mechanism attached to
  it, which is worse than an obviously wrong number. Two guards, both cheap: quote
  **absolute** paths for measurement artifacts, and print a scan's row count and
  denominator before differencing it against another, because two scans agreeing on
  `fn_total` proves only that the corpus is the same, never that the binary was.
  **The denominator guard is now proven insufficient, not merely weak** (2026-07-30):
  the corpus moved mid-session and `fn_total` matched anyway, because a workload tree
  can change in ways that add and remove no IL body at all. A count is not an
  identity. Every scan now prints and records the **workload tree's git HEAD plus a
  dirty flag, the c2-rs HEAD, the resolved toolchain paths and the wibo version**
  (`c2_harness::provenance`, JSONL record 0, tagged `"record":"provenance"`), and
  every field degrades to `unknown` rather than failing when git is absent. Quote a
  number with its provenance line or do not quote it.
- **Name the loader, or it will name your results for you.** A stale sibling wibo
  (`1.0.1-7` against the known-good `1.0.1-23`) turns the gap scan's replay column
  from `36 checked / 0 diverged` into `36/30` — a **fake correctness alarm on the
  oracle seam** — while the census, the mismatch count and every blocking-feature row
  stay byte-identical. Nothing in the report named the binary, so the only visible
  change was the one number that looked like a real regression. `gap` and `selftest`
  now print the resolved wibo path and its `--version`, and warn loudly (never
  fatally — env-driven toolchains are by design) when it parses older than
  `c2_reference::WIBO_KNOWN_GOOD`. The version compare is numeric per component
  precisely because the failing pair sorts the wrong way as text (`"7" > "23"`).
- **A cache is an instrument that answers without doing the work, so the only
  interesting property is whether a wrong one is detectable.** `c2rs gap` now caches
  reference captures content-addressed — source **bytes**, the exact flag string, the
  compile cwd, the `cl.exe`/`c1xx.dll`/`c2.dll` bytes, the wibo version, and the
  workload tree's git identity (HEAD + a content digest of every tracked
  modification, which is what closes the *header* hazard a `.cpp` hash cannot see);
  never mtimes. Two properties are worth copying to the next cache in this repo:
  **(1) a collision degrades to a miss, not to a wrong answer** — the entry stores
  its full key material verbatim and a hit is served only when those bytes compare
  equal, so the 128-bit hash's odds are a curiosity rather than a load-bearing claim;
  **(2) it is never trusted without a bypass-and-compare** — `--validate-cache N`
  re-captures every Nth hit through the real toolchain and byte-compares, naming the
  field and offset that differed, and `--no-cache` bypasses entirely. Demonstrated:
  one flipped byte in a cached 2.5 MB `.ex` is served silently as a clean scan
  (exit 0, and in that instance the headline census did not move either), and is
  named exactly — `.ex differs at offset 1276107` — with exit 1 under the validator.
  **What the validator found on its own control case is the better argument for
  having one.** Two facts about "the same" capture, neither previously written down:
  `cl.exe` names the bundle `_CL_<hex>` from a **per-invocation nonce**, and the
  reference obj's COFF `TimeDateStamp` is **wall clock** — one cold scan's 878 objs
  carry 58 distinct values across its 5-minute window. Both are normalized away
  explicitly (the timestamp by the project's own criterion, and reported as its own
  verdict rather than folded into "identical"), and both would have been invisible
  without a check whose control group is a capture that is supposed to agree.
- **A failed search is not evidence of absence.** Three of this project's
  wrong-bytes emits are the same mistake: code asked "did I find X?" and read
  "no" as "there is no X". `.gl` did not name a destination, so the token was
  taken for a local — but a file-scope `static` is `$sv`, which the index does
  not accept as an identifier, and the store vanished. No `this` group was found
  ahead of the first `0x46`, so the function was taken for a non-member — but on
  source line 70 the first `0x46` is the line marker's payload, and every formal
  dropped a register (`il_this_line70.cpp`). The fix in both cases is the same
  shape: make *absence* something you can positively see, and give the answer a
  third value — **undetermined** — that refuses. A two-valued answer silently
  converts a decode failure into wrong bytes; a three-valued one converts it into
  a `NotImplemented`.
  **The fourth instance cost coverage rather than correctness, which is why it sat
  for so long.** `gl_symbol_index` looked for a record's name immediately after a
  NUL; 12,505 of the 33,059 `?`-mangled names in eight real TUs have a `26` there
  instead, so "no name found" was read as "no such symbol" and 9,028 generated
  destructors refused (D14, `ROADMAP.md` §6e). Two things generalize. A search whose
  failure is *fail-closed* still needs the same scrutiny as one whose failure is
  wrong bytes — it just presents as a stubborn census bucket rather than an alarm,
  and this project's own ranked worklist had been carrying it as "the largest named
  item that needs no new instruction lowering" for a rung. And an anchor should be
  chosen so that the thing it looks for is a **field of the record**, not a byte
  value the record happens to contain: the separator is a field with two measured
  values, and pinning it to one of them is the same class of mistake as pinning a
  token's width to two bytes.
- **One fact, one locator.** The `46` formals marker was located correctly in
  `parse_formals` (anchored to end on `LO`, after the line-70 bug) and
  incorrectly in `parse_this_token` (first matching byte) *at the same time*, for
  weeks. Fixing an anchor is not done until every reader of that anchor shares
  it; a second copy is where the bug goes to live. Deleting the unsafe helper
  (`find_byte`) was part of the fix, not tidying.
- **A truncated fixture cannot witness the region it omits.** The three pinned
  `.ex` segments started at the formals marker, so the pre-body region — where
  `this` is bound, and where the emit was wrong — appeared in no test. They read
  as verbatim captures and were verbatim *suffixes*. If a fixture is trimmed, the
  trimmed part is untested, and saying so in the fixture is what stops the next
  reader from assuming otherwise.
- **`mismatch` is an alarm, not a gap.** Any nonzero mismatch bucket — on
  fixtures or the workload — is a correctness bug that outranks all widening
  work. The port's value downstream depends on this bucket staying 0.
  **"Fired exactly once" stopped being true within a day** (an earlier
  revision of this bullet said it; corrected 2026-07-30): after `w5_chain.cpp`
  (§1, 40749e7) the alarm fired repeatedly as the instruments widened — ~20
  reassociation/repeated-leaf emits from the first generated sweep (e5edcb4,
  2b6cbe5), 10 more from bounding chain acceptance to the enumerated region
  (1001267), 3 from `.gl` symbol binding (a892c76), 3 from the first `/Gy`
  fixture lane (`mode_lane.sh`, 2a19090), 4 from the `/O1` comparison-spine
  matrix (abe0512). Every one was fixed or gated closed before widening
  continued, and the buckets measure 0 at HEAD (fixtures, four
  mode lanes, 878-TU scan, 4,011-case sweep — all 0 mismatch). The lesson
  moved: the alarm firing often, on instrument widening rather than on user
  reports, is the differential working — a long green streak under an
  unchanged instrument is what should raise suspicion.
  **And the streak has now been broken by a probe rather than by an
  instrument.** Instances 6 and 7 above were found by compiling five throwaway
  `.cpp` files while characterizing an unrelated rung's lowering — not by the
  sweep, not by a lane, not by the scan, all three of which were green over
  both bugs for as long as they existed. That is the clearest available reading
  of "coverage-bounded": three instruments agreeing costs nothing against a
  shape none of them contains, and the cheapest way to find one is to compile
  the neighbouring source you were about to assume something about.
- **A green corpus is only as strong as its discriminators.** The §1 mis-emit
  survived because two candidate allocation rules coincide on every shape the
  corpus contained. When a rule is inferred from captures, ask what fixture
  would *separate* it from the nearest plausible alternative — and add that
  fixture, positive or negative, before claiming the class.
- **A fixture pass without a real-TU improvement is not progress** — the
  census/scan gates exist precisely because fixtures sample the grammar we
  *guessed* matters; only the 878-TU scan measures the grammar that *does*.
- **Diff scans, don't overwrite them.** Coverage must be monotone
  scan-over-scan; the dated JSONLs are the longitudinal record (per-TU
  `class`/`reason` diffing catches a rung that fixes one bucket by breaking
  another).
- **Measure committed code.** Build from a clean tree before a scan you
  intend to record — a binary carrying uncommitted WIP produces numbers no
  future scan can be diffed against.
- **The denominator has one definition**: functions counted at the `LO` body
  marker (`4C 4F 11`), 2,462,571 over this workload. Never re-derive it from
  `.gl` mangled names (~902,730, wrong) or from raw `4F 1F` scans (~2% high).
  A coverage percentage is only comparable to a previous one if both used it.
- **A new census bucket may be a parser bug, not a feature — this has now
  happened twice.** Misaligned reads and stale models both look exactly like
  unknown vocabulary. The variable-token-width fix deleted two bucket families
  that were pure misalignment; the CALL-token decode deleted the entire
  `call-anchor-*` family, ~12.4% of blocked functions, which was measuring a
  hardcoded anchor that never existed (GAP-1, §2b). Before scheduling work
  against a bucket, read the hexdump `c2rs census` prints for it (or dump the
  bytes at the recorded offset with `--keep-il`) and confirm the parse arrived
  there **aligned**.
- **A bucket's size is not the number of functions fixing it unlocks.** The
  census records where a parse *stops*, so a bucket counts functions whose
  **first** blocker is that feature — not functions with nothing else in the way.
  Decoding intrinsic 2117 `base-member-addr`, a 6.3 % / 149,200-function bucket,
  moved exactly **32** functions in class, and the blocker histogram showed no
  other bucket growing: the rest of that 149,200 still stop at the same feature
  because the decode landed in the indirect-load *leaf* recognizer and their
  bodies are not leaves. A bucket is an upper bound on the win and often a very
  loose one. Before scheduling against a percentage, ask which *shape* the fix
  lands in and how many of the bucket's functions are that shape — or measure the
  delta on one TU first.
  **Measured to the bottom, twice, on the same row.** `expr-op-0x27` — the
  byte-offset add in a general expression position, 505,122 functions and 24 % of
  everything blocked — was written into `IL_CALL_IN_EXPR.md` §21.5 as "the ranked
  next rung". A counterfactual that admits it releases all 505,122 and leaves
  **685 whole bodies, 0.14 %**; the rest move one token deeper onto an indirect
  load in an expression position, a by-value bind and a store. The rung taken
  instead was **1/12th its size and 100 % whole-body complete**, and converted
  1:1. Two rungs running, the largest row has been the wrong answer, and both
  times the cheap counterfactual said so in one scan. **Rank by whole-body
  completeness, not by row size — and when the row has no `-whole` bit, spend the
  scan to get one before scheduling against it.**

  > **CORRECTED 2026-07-30 by W25 (`docs/IL_STORE_LEAF.md`), and the correction
  > is a limit on the instrument, not on the ranking rule.** That 685 is right
  > about what it measured and wrong as a statement about the row: the
  > counterfactual admitted the **token** `27` inside `parse_expr` and asked
  > whether the body then parsed to the end — so it could only ever count bodies
  > that finish as an *expression*. Half of `expr-op-0x27` is a **statement**,
  > `s->m = v;`, which fails one token later at the `32` store whatever the `27`
  > does, and no widening of `parse_expr` can reach it. A production-level rung
  > then took **22,095** functions out of the same row, 32x the number the token
  > counterfactual reported for it.
  >
  > The rule that follows: **a counterfactual measures what the surrounding
  > grammar can already finish.** "Admit this token" and "admit this production"
  > are different questions with different answers, and a near-zero completeness
  > figure means "nothing completes *through this token's own arm*", never "this
  > row is empty". Before writing a row off, check what its bodies **are** — one
  > pass of whole-segment dumps over a stride sample of TUs, which is what found
  > the store here.
  There is one family where completeness is free rather than counterfactual: a
  census key ending `:eof` is a refusal raised *after* the parse reached the end
  of the segment, so **every function under it is grammar-complete by
  construction**. `expr-out-of-class-bare-nonfirst-formal:eof` was 43,319 of
  those, and the estimate formed from it landed inside ±700 with the whole of its
  stated low bias attributable to one named second site.
  **There is a second such family, and it went unread for as long as the first
  one did.** A key of the form `fn-tail-0xNN` is raised by `eat_fn_tail`, which
  every accepted shape reaches *last* — so a body filed under one has already
  parsed under a real shape and is grammar-complete for the same reason. That is
  what `fn-tail-0xB9` was: 29,552 functions, listed as "the largest call-free row
  that is not part of the pointer-expression layer" and never decomposed, and it
  is a **constructor's `return this` sitting between the RETURN and the tail**,
  which costs no instruction at all. 28,717 of them converted 1:1 with a residue
  of 3 that both instruments named (`IL_CALL_IN_EXPR.md` §23). The generalizable
  part is not the shape: it is that **"where the parse stopped" has a small set
  of positions that imply completeness, and the census key spells them** — read
  the key's *position* before spending a counterfactual on its size.

  **`fn-tail-0x26`, 4,663 functions, is the same family and is MEASURED AT ZERO**
  (`docs/IL_CALL_IN_EXPR.md` §24.1, refuted at zero build cost — one query
  against a scan that already existed). All 4,663 are `calls-2plus`: 0 `calls-0`,
  0 `calls-1`. §18's frame axis settles it outright, because two calls always
  need a frame, so the **takeable population is 0** until the general-frame
  phase. Its twin `fn-tail-0xB9` split 28,720 `calls-0` / 832 `calls-1`, which is
  precisely why *that* one converted 28,717 of 29,552 at 1:1 and this one
  converts nothing. **Membership in the free-completeness family says a body
  finished parsing and says nothing whatever about whether it needs a frame —
  they are independent axes, and the frame axis is the cheap one.** Check it
  first, on every candidate, before any counterfactual: this is the third time a
  row has re-entered a ranking on its size alone after already having been
  measured at zero, and each time the re-entry happened because the row was
  carried forward by its *population* rather than by its verdict.

  > **A third measurement of the same row, 2026-07-31 (W34), and it found the
  > answer somewhere neither of the first two looked.** The production-level
  > counterfactual — admit the whole indirect-load *operand* production inside
  > `parse_expr`, not just the token — released `expr-op-0x27`'s 461,786 and
  > finished **6,816**, a **67.8×** row-to-counterfactual gap against the
  > control-flow lane's 67×. So the rule above holds a third time and the prior
  > is now stable enough to quote. What it did **not** predict is where those
  > 6,816 live. The estimate reasoned about which *new* sub-shape the surrounding
  > grammar could finish and guessed the call argument (`return g(s->m);`);
  > measured, that sub-shape is worth **at most 7 functions on the whole
  > workload**. **5,161 of the 6,816 were bodies the port already had a
  > recognizer for** — plain `return p->mid.in.b;`, refused by a private
  > "exactly one offset add" limit inside the indirect-load leaf that the address
  > and store leaves had never had. The rule that generalizes, and it is cheap:
  > **before sizing a big blocker row's sub-shapes, ask what the recognizer that
  > already covers the obvious one is refusing.** A first-blocker histogram
  > cannot distinguish "this construct is unimplemented" from "this construct is
  > implemented and one gate inside it says no" — both stop the parse at the same
  > byte — so the largest sub-population in a head row can be a shape the port
  > thinks it already has. The tell is available without compiling anything:
  > the row's blocking byte was `27`, and `27` appears in four *accepted* shapes.
- **A row can be a whole PRODUCTION filed under an opcode, and the tell is that
  the row has no `-whole` bit while its twin does.** `expr-op-0x99` was the
  largest single key on the board — 280,283 functions, 11.4 % of everything
  blocked, and 364,690 behind a cleared `expr-op-0x27`. It was not a missing
  token. It is this document's own `expr-call-in-expr-recv-*` family under a
  second name, reached by the one route that never calls `mcall::classify`: the
  body dispatch tells a call from an assignment by asking whether a `BD` follows
  the statement-head `26 <tok>`, and for a **member** call it does not — the
  receiver sits between the method push and the CALL token — so `p->m();` went to
  the assignment parser, which read the receiver as an ordinary LOAD and stopped
  on the `99` bind under `parse_expr`'s generic fall-through. `x = p->m();`, one
  byte different, kept its method push where `parse_expr` could see it and was
  filed as a member call all along.
  This is the unstable-*attribution* bullet above in its worst form so far, and
  the difference is worth stating: a **sharded key** splits one construct across
  names that still look like the construct, so summing them recovers it. This
  split one construct across a call bucket and an **opcode** bucket, and no
  amount of regrouping by name could have joined them. Two things follow.
  **The diagnostic was available for free and nobody read it**: the D2 family
  prints a whole-body-completeness bit and `expr-op-0x99` printed none, because
  the opcode fall-through raises a bare `Block`. *A row with no `-whole` bit
  sitting at the top of the ranking is not "unmeasured"; it is evidence that the
  row is not reaching the classifier that would measure it.* And **the repair was
  already in the tree, scoped to one case**: `reanchor_chain` had fixed exactly
  this mis-anchoring for *chained* receivers (a measured 4.4× undercount) and
  stopped there. Generalizing it — same walk, same three conditions, no second
  tokenizer — de-conflated the row 1:1 with the census unchanged, and the rung it
  then made visible was **+20,912** (W36, `docs/rungs/2026-07-31-member-call.md`).
  "Fixed in the one shape where the bug had been found" is instance #2's shape,
  and this is its census-instrument form.
- **Before borrowing a rate from a sibling bucket, check that the two agree on
  the axis that decides the rate.** W36's estimate was LOW by 2.99× and the whole
  error is one line of reasoning. It took the `-whole` rate of the sibling
  `expr-call-in-expr-*` family on `calls-1` bodies (3,849 / 59,346 = 6.5 %) and
  applied it to the row's own 114,059 `calls-1` functions. But the sibling family
  is the same production in a *value* position, and `recv-load*` there is 25,308
  functions with **two** `calls-1` in the whole of it — a member call in an
  assignment RHS is nearly always in a body that makes more than one call, while
  a member call that IS the statement is nearly always the only one. The
  statement-position rate is 30 %, not 6.5 %. The asymmetry was in the scan
  before the estimate was written and was read as context rather than as the
  reason the rate could not transfer. Two other anchors were written down at the
  same time: the 67× row-to-counterfactual prior (which the estimate explicitly
  discounted, correctly — the realized ratio is 13.4×) and plain source-language
  reasoning (28,500, the closest of the three, off by 1.36×). **When the anchors
  disagree by 7×, the one to trust is the one whose population you can show is
  the same population.**
- **The coverage-costing form has a MIRROR, and it is harder to see than the
  original.** §6 already records the "one rule, N implementations, and the oldest
  copy is narrower" shape — the `27`/`28` run walked by three leaves with one
  private single-add copy, 5,161 functions (W35). W37 found the same cost with the
  opposite structure: a **shared** recognizer that only one caller ever asks.
  `eat_ctor_this_epilogue` decodes a constructor's `return this` and has since
  W19; it had **exactly one consumer**, the empty-body arm; and the moment a
  second production asked it, it was worth **42,238 functions** — 81 % of that
  rung's whole counterfactual ceiling, in a rung whose estimate was about
  statement lists and never mentioned tails. Neither form is visible to any gate
  here, for the same reason: refusing more emits nothing, so no byte compare and
  no census/gate disagreement can see it. The question to ask of a shared helper
  is therefore not only "do its callers agree" but "**how many callers does it
  have, and is that number 1?**" — a `grep` with one hit is the same evidence as a
  `grep` with two that disagree.
- **"One fact, one locator" has a coverage-costing form, and it is easier to miss
  than the wrong-bytes form.** The `27`/`28` byte-offset-add run is walked by the
  address leaf, the store leaf and the load leaf. The first folds an arbitrary
  run of literal offsets and has since it was written — its own comment carries
  the capture, `&s->arr[2]` emits one `addi r3,r3,48` — and the store leaf
  inherited that walk when it was built on the shared designator. The load leaf
  kept a **private single-add copy** and refused everything past the first,
  costing **5,161 functions**. Nothing was ever wrong; a rule simply existed
  three times and one copy was older than the other two. Two things follow.
  The instances in the list above are all *wrong bytes*, so a reader could
  reasonably conclude the pattern is a correctness pattern — it is not, and this
  form presents as a stubborn census bucket that stays at the top of the ranking
  for weeks. And **the direction of the drift is asymmetric**: a copy that is
  narrower than its siblings is invisible to every gate this project has, because
  refusing more is never an alarm. When a rule is found to have N sites, the
  question is not only "do they agree" but "**does the oldest one still do what
  the newest one does**".
- **Estimate the fix, not the finding — and the way to do that is to size the
  second site as its own counterfactual BEFORE shipping the first.** §6 already
  records the `66` descriptor fix realizing 2.4× its estimate because a second
  site was found afterwards. W34 had the same structure — the offset-add run has
  two call sites, the plain designator and the intrinsic-2117 one — and measured
  the second (**+1,346**) as a separate scan before either shipped, so the rung's
  number was 6,507 and not 5,161-then-a-surprise. The two were then confirmed
  **exactly additive** by a third scan with both lifted, which is the only way to
  know the smaller site was not being masked by the larger. `grep` for every site
  is step one; a counterfactual per site is step two, and it costs one warm scan
  each.
- **A conservative gate can be sized, and until it is, its cost is a rumour.**
  The FP leaf was gated closed on 2026-07-30 by requiring every formal to be an
  FP operand of the body, which shut two live wrong-bytes emits and cost 1,005
  census functions; the note that shipped with it said the remaining
  over-refusal "is not measured". Measuring it took one scratch build — make the
  gate sink its refusals under their own census key instead of returning `None`
  — and the answer is that the over-refusal is **exactly those 1,005 and not one
  function more**, all `calls-0`, all whole-body complete, 1,004 of them a single
  FP LOAD. The pessimistic reading, that a whole-formals-list gate would spill
  into the 98,813 + 82,810 float/double operand rows, is **false**: those block
  on operand types, member loads and conversions well ahead of any question
  about register numbering. Two things follow. A gate raised *after* the
  whole-body parse succeeds is free to measure, because its refusals are already
  complete bodies. And an unsized over-refusal gets quoted as a range in the next
  ranking, which is how a 1,005-function rung ends up compared against a
  29,552-function one as though the comparison were close.
- **The frame axis refutes candidates for free, and completeness does not imply
  it.** `fn-tail-0x26` was carried as "the other member of the free-completeness
  family, unexamined" for a whole rung. The first thing measured about it settled
  it: **all 4,663 are `calls-2plus`**, which §18 already proves needs a frame, so
  its takeable population is 0 — no scratch build, no counterfactual, one query
  against a scan that already existed. Its sibling `fn-tail-0xB9` split 28,720
  `calls-0` / 832 `calls-1` — which is exactly why *that* one converted 28,717 of
  29,552 at 1:1 while this one converts nothing. So the family says a body has
  *finished parsing* and says nothing whatever about whether it needs a frame.
  **Two independent axes; check the cheap one first.** `expr-op-0x99`, now the #2
  row at 280,282, has **zero** `calls-0` functions and should be refuted the same
  way before anyone ranks it.

  > **And the re-entry is the recurring part, not the row.** `fn-tail-0x26` was
  > still being written up as "the same family and still unexamined" *after* this
  > bullet was written, in the same document, because the two live 130 lines
  > apart and the earlier one carried the row forward by its **population**
  > (4,663) rather than by its **verdict** (0). Corrected 2026-07-31. That is the
  > third row to re-enter a ranking on size alone having already been measured at
  > zero, so the operational rule is not "measure the frame axis" — that already
  > happened — it is: **when a row is refuted, go back and amend every place its
  > size is quoted.** A refutation recorded in only one of the places that rank
  > the row has not removed the row from the ranking.
- **A row that names a token near the leaves of the grammar is a gate, not a
  rung — three measurements now say so.** The pointer *type* released 983,707 and
  finished **1.4 %**; `expr-op-0x27` released 505,122 and finished **0.14 %**;
  `expr-convert` released 225,341 and finished **2.47 %**
  (`IL_CALL_IN_EXPR.md` §21, §22, §24). All three were the top or near-top row
  when scheduled. The generalizable form: **the size of a first-blocker row
  measures how early its token appears, not how much work is behind it.** Ask of
  every head row: is this token a *value annotation* the rest of the body then
  has to be understood anyway? If so, the rung is whatever is complete behind it,
  and that number needs a counterfactual before the row gets on a schedule.
- **A stated bias direction can carry a measured bound instead of a name, and
  should.** §13.1 requires the estimate to name its bias and its cause. D12 went
  one step further: its low bias was the familiar "a second parser site
  implements the same rule" hazard, and instead of leaving it as a hazard the
  other sites were grepped and *counted* — 4 functions on the whole workload — so
  the estimate shipped as "+5,562, low by at most 4". It landed at +5,562. An
  unbounded bias direction is an excuse; a bounded one is a prediction.
- **A rung's own REFUSALS are residue and must be named too.** Rungs are audited
  on "where did the un-gained population go", and the answer usually points at
  keys that already existed. D12 created 11,479 functions of *new* refusal keys,
  and 5,684 of them — larger than the rung's own +5,562 gain — turned out to be
  a single over-refusal: `eat_int_like`'s exact four-triple whitelist rejects a
  conversion whose target is a width-4 integer carrying a per-TU type id (an
  enum, a typedef, a `const int`), which `is_int4_type` would admit on the
  nibbles and which emits the same nothing. It was invisible until the refusal
  got its own key. **Give a new gate a key on the way in, not after someone asks
  what it cost.**
- **The census can over-claim, and the direction of a census/gate disagreement
  does not make it safe to leave unrecorded.** §22.5 found a producer that
  claimed functions in class that `PortC2` refused; the same disagreement existed
  in the other direction of the same seam — `int f(int a,int b,int c){
  return a + b*c; }` censused in class and the port returned `NotImplemented`,
  because a `*` after the first operator was gated in `codegen` and not in
  `parse_segment` (`IL_CALL_IN_EXPR.md` §24.7). Nothing wrong is emitted, which
  is exactly why it survived — but it means the headline numerator is an upper
  bound by an unmeasured amount, and a numerator with an unmeasured error term is
  not a benchmark. Every gate that decides in-class membership belongs behind one
  predicate that both producers call.

  > ### Sized, 2026-07-30: **9,230 functions, 2.24 % of the numerator — and none
  > ### of it was the shape §24.7 named**
  >
  > The instrument is `IlBundle::census_functions`, which pairs every census row
  > with the emitter's own function record, and `codegen::function_gate`, which
  > runs `PortC2`'s per-function selector over it. `c2rs gap` and `c2rs census`
  > print the disagreement in the same block as the numerator on **every run**,
  > because this is the §6 rule about giving a diagnostic a population whose
  > answer is already known: every in-class function, whose answer must be
  > "accepted".
  >
  > Three findings, in decreasing order of how much they should change what the
  > next person does.
  >
  > **1. The characterized case was 0 % of the real total.** §24.7 was written
  > from three probes of the straight-line class. On the 878-TU workload that
  > class contributed **zero** disagreements — 62,813 straight-line functions and
  > not one whose operand stack goes past depth 2. The whole 9,230 was two
  > *other* causes that nobody had looked for: **9,028** generated empty
  > destructors whose callee token has no `.gl` symbol, and **202** functions
  > carrying an optimization word the port does not emit under. A characterized
  > defect is a witness, not a measurement, and the ratio here is not a rounding
  > error — it is the entire quantity.
  >
  > **2. The fixture corpus held 14 more, of three further kinds** — the §24.7
  > depth rule (9, `w5_tree_neg.cpp`), the `==`/`!=`-against-a-large-unsigned rule
  > (4, `w6_unsigned_wide.cpp`) and FP scratch exhaustion (1, `w13_fscratch.cpp`).
  > Every one sat in a fixture that **grades nothing**, because its TU has an
  > out-of-class sibling and the port is all-or-nothing per TU. So the same
  > property that hides a mis-emit (§6's `w13_fabi.cpp` bullet) also hides a
  > census/gate disagreement, and for the same reason. A cross-check that runs
  > per *function* sees straight through it; a TU-level differential cannot.
  >
  > **3. Two of the four moved gates were already spelled out twice.** The
  > comparison leaf's wide-literal and `i16::MIN` rules were in the parser **and**
  > in `compare_leaf_text`, in sync by luck; the third rule of the same family —
  > a large unsigned under `==`/`!=` — was in codegen alone, and that is exactly
  > the one that leaked. Partial duplication is worse than none: it makes the
  > seam look already handled. The repair is one predicate
  > (`CompareLeaf::out_of_class_ctx`, `chain_form`) that the parser gates on and
  > codegen consults, with codegen's own check demoted to the second lock the way
  > `indirect_load_text` already documents.
  >
  > **The corrected numerator is 411,934 → 402,704 (16.73 % → 16.35 %)** — and
  > then 418,628 with the unrelated W22 widening (§6d), which is recorded
  > *separately* on purpose: a correctness repair that costs coverage must not be
  > netted against a widening that buys it, or the repair becomes invisible —
  > accounted for **1:1** by three new census keys —
  > `callee-unresolved-dtor-delegation:eof` 9,028, `opt-mode-00200001` 136,
  > `opt-mode-00200101` 66 — with **zero** movement in any pre-existing key, zero
  > TUs changing class and mismatch 0. Going down is the correct outcome and the
  > point of the exercise; the previous number counted functions the port refuses.
  >
  > Kept honest by `crates/c2-harness/tests/census_gate.rs`, which runs the
  > cross-check over the whole fixture corpus and asserts the disagreement equals
  > its **recorded** value (1, named and sized) rather than allow-listing it away,
  > so a gate landing in codegen instead of the parser fails a test.
- **A capture is not perfectly reproducible, and a first-blocker histogram will
  show it.** Two 878-TU scans of the *same* binary over the *same* corpus
  disagreed by **one function** in `src/system/hamobj/HamIKEffector.cpp`
  (`expr-op-0x99` 554 vs 555, `expr-intrinsic-base-upcast` 51 vs 50), with
  `fn_total` and the in-class numerator identical. Re-running the single-TU
  census with both the pre-change and post-change binaries gives 555/50 every
  time, so it is not a code change — it is run-to-run variance in the capture
  under a 16-way parallel scan, landing on a blocked-vs-blocked *attribution*
  rather than on acceptance. Two consequences: a one-function difference in a
  histogram row is below this instrument's noise floor and must not be reasoned
  from, and a rung whose claimed movement is single digits needs the same
  measurement repeated, not just re-read.
- **A guessed name is worse than a hex bucket — this has now happened three
  times.** (1) The relational opcode labels were inferred from numeric order
  and three of six were wrong. (2) `call-anchor-*` named a structure that did
  not exist. (3) `expr-cast` named `0x40` a cast on a single witness; it is the
  intrinsic-call token, and the mistake split one ~13% production across two
  separately-ranked rows and put the wrong one on the schedule (§2b,
  `docs/IL_CAST_CONVERT.md`). The rule: **name a bucket only from a capture
  that pins it — otherwise leave it hex.** A hex bucket is a result; a name is
  a claim, and claims get grouped, ranked and scheduled as if they were
  evidence.
- **The numerator and the denominator must be compiled in the same mode.**
  They were not. Every fixture and every sweep case is captured with the
  default `/Ox`; the 878-TU workload compiles `/O1`. Those emit different code
  for the same source — 7 of 9 sampled matching fixtures differ — so the
  byte-exactness claim was about `/Ox` while the coverage percentage was about
  `/O1`. `.ex` says so explicitly, in a per-function word after each `4F 1F`
  that the port did not read (`docs/OPT_MODE.md`). Worst of it: the whole-chain
  accumulator rule that cost 270 mis-emits to establish is `/Ox`-only — `/O1`
  allocates by ordinary liveness, so 47 of the 108 enumerated integer chains
  differ between the modes. Both modes are supported now, and the mode is read
  rather than assumed, but **when comparing any two numbers in this document,
  check they came from the same `/O` flag** — and note `/O1`/`/O2` imply `/Gy`
  while `/Ox` does not, so a `.text` size comparison across them is measuring
  two axes at once.
- **One corpus, one lane, is one sample of the argv space.** Every fixture
  compiled `/Ox` for the project's whole history, so the COMDAT emitter — which
  only `/Gy` reaches — had never been run on a fixture that calls, floats or
  frames. Adding `scripts/mode_lane.sh` and pointing it at the *same 88 fixtures*
  with different flags immediately found three wrong-bytes emits, two of them
  bugs that had already been found and fixed once in the sibling packed emitter.
  A second lane over an unchanged corpus was worth three bugs, which says the
  corpus was never the limiting factor — the single capture profile was.
- **Two unknowns in one equation absorb each other's error, and no table of
  totals can separate them.** The `$M`/`$T` compiler-label numbers are
  `seed + Σ(stride of each preceding function)`, and `OBJ_GY_SHAPES.md` §3.4
  had 21 witnesses of the total with neither term known. It fitted a model —
  correctly for every int class — and got the float stride wrong by one and the
  pooled-constant stride wrong by two, because `v_float_framed` (float-leaf,
  framed) and `v_leaf_framed` (leaf, framed) both showed 2550 and were read as
  agreeing when they are two TUs with two different seeds. Adding rows could
  never have fixed it. What fixed it was finding a witness that pins **one** of
  the two: the seed is in `.gl`, four bytes at a fixed offset, and with it every
  stride is a one-class-at-a-time subtraction. The general rule: when a measured
  quantity is a sum of two unknowns, stop collecting sums and go looking for
  either term — and until you have one, do not describe the fit as a model that
  "fits all N witnesses", because it fits them the way any two free parameters
  fit a line.
- **A constant gap over a biased sample is not a fit.** The same `.gl` field
  read as a **LEB128** gives 1256 for `mvp_framed`, and `B − 1256 = 1289` held
  across every fixture checked at first — because the counter sits in the
  2500–2700 range and every value there LEBs to two bytes with the continuation
  bit set. It is a fixed-width u32; the two readings agree on the whole of a
  corpus whose values happen to share one property and diverge the moment the
  low byte falls under 0x80. A wrong decode that is *linear* in the right one is
  the worst kind, because a constant offset looks exactly like a calibration
  constant. Ask what property the sample shares before believing a constant.
- **A constant that no input can move is not validated by any number of green
  runs — and the signature is in the function's SIGNATURE.** `framed_call_text`
  took `(add_k, base_off)`. Neither argument can distinguish two formals, so the
  function was structurally incapable of emitting a formal-dependent word, and
  the class it served (`return g(<formal>) + k`) obviously has formals. That is a
  cheaper tell than any capture: **before trusting a lowering, check that its
  inputs span the things its output is allowed to depend on.** The same reading
  applies to the frame size it hardcoded — `96` could not vary with anything,
  and the frame is `align16(80 + locals + 8 + 8×saved)`. Both were found in the
  same hour by asking that one question of one signature, after the differential,
  four mode lanes, a 4,706-case sweep and an 878-TU scan had been green over
  them for as long as the function existed.
- **A constant is only as validated as the inputs it was evaluated at.**
  `framed_call_text` wrote the `bl` word `4BFFFFF5` literally. That is correct —
  MSVC's `disp = −(own .text offset)` — for a framed function at `.text` 0, and
  a framed TU was gated to exactly one function, so 0 was the only input the
  constant ever saw. Removing the gate turned it into a wrong-bytes emit on the
  second line of the first multi-function fixture (`4BFFFFED` at 0x14). This is
  the "safe half of the pair" shape again with a new tell: **when a gate is
  removed, grep the code the gate was protecting for values that were constants
  only because the gate made them constants.** Two constants in this file had
  that property; the other (`FRAMED_PROLOG_LEN` vs `FRAMED_BL_OFFSET`, equal for
  this frame class and unequal for a 5-word prologue) was split pre-emptively for
  the same reason.
- **A gate written to buy time gets counted as coverage.** `functions()` refused
  a TU with a framed function and any sibling (`n_defined != 1`), while `c2rs
  census` graded the same TU 2/2 in class. That is §22.5's producer disagreement
  in the direction that inflates the numerator, and it sat there long enough to
  be quoted: two functions counted as in-class that the port would never emit.
  Nothing wrong was emitted, which is exactly why it survived. **A gate whose
  comment says "for now" is a claim about the numerator, and the census producer
  has to be told about it or the numerator is wrong by however much it refuses.**
- **The corpus is a live working tree, and `fn_total` agreeing does not prove it
  held still.** Two scans of the 878-TU workload taken 40 minutes apart differed
  in 6 TUs, and one of them moved a function between blocking buckets. Neither
  binary nor flags had changed in a way that could do that; **`dc3-decomp`
  itself had advanced** — a sibling agent committed to it mid-session, headers
  changed, and 6 `.ex` files came out tens of bytes different. `fn_total` was
  identical across the two scans (2,462,571) and `fn_in_class` was identical
  (411,934), so the guard this document already recommends — print the row count
  and denominator before differencing — passed while the corpus underneath had
  in fact changed. The fix is the same shape as "quote absolute paths": **record
  the workload tree's `git rev-parse HEAD` in the scan, and re-run the *baseline*
  binary against the current tree before claiming a delta.** Doing that here
  turned "6 TUs changed" into "0 TUs changed" and made the zero-movement claim
  real rather than approximate.
- **Two binaries with the same name, one of them stale, and the gate that
  notices is the one you are least likely to read.** `C2RS_WIBO` pointed at
  `../wibo/build/wibo` — the obvious path, and the one this session's very first
  orientation command found — turns the 878-TU scan's replay column from
  **36 checked / 0 diverged** into **36 / 30**, while leaving the census, the
  class buckets and the mismatch count byte-identical. The repo's own default
  resolves `../wibo/build/**release**/wibo` (1.0.1-23, current); `build/wibo` is
  a 1.0.1-7 build from four months earlier, and the two produce different objs
  for the same input — `capture_il_with` already carries a comment about wibo
  ≥ 1.0.1-23 reaping guest temporaries. Three things follow. **A 90 % replay
  divergence that leaves every other column identical is an environment
  report, not a compiler one** — check the toolchain before the code. **An env
  override silently opts out of the resolution logic that was written to be
  right**, so a scan quoting `C2RS_*` overrides has to say which binaries they
  resolved to. And the replay gate did its job precisely by being loud about a
  difference nothing else could see, which is the argument for keeping it on
  every scan rather than sampling it away.
- **A field the port skips is indistinguishable from a field that is always
  the same.** The optimization word above was stepped over silently for months;
  so was the source-line marker before it turned out to carry a varint payload,
  and `read_type` still mis-reads aggregates by treating a 5-byte field as 3
  (`IL_TYPE_TAGS.md` §1). Every fixed-width skip in the parser is a place where
  a real distinction can hide, and the cheap test is to diff the `.ex` of two
  sources that differ *only* in the property you suspect — four changed bytes
  is a much easier read than a grammar.
- **A row can be UNMEASURABLE rather than unmeasured, and from the outside the
  two look identical.** This document already records that "a row with no
  `-whole` bit sitting at the top of the ranking is not 'unmeasured'; it is
  evidence that the row is not reaching the classifier that would measure it"
  (`expr-op-0x99`, W36). W37 is the other half of that pair and it is worse:
  `expr-call-in-expr-recv-load-then-bit-and` — **102,382 functions, 5.5 % of
  everything blocked, the largest key on the board** — *was* reaching the
  classifier, and the classifier had no production for a bare binary operator, so
  `mark_whole`'s greedy chain stopped dead at the token and the pair was reported
  UNMEASURED **by construction, for every operator row, forever**. Both failures
  present as the same thing: a six-figure row at the top of a ranking with no
  completeness figure. The distinction matters because the repairs are opposite —
  one is a mis-anchored dispatch, the other is a missing arm in the measure — and
  neither is visible from the key. **Ask of a bare row whether the instrument
  could have printed a bit for it at all**, and if the answer is no, fix the
  instrument before ranking anything against the row. Granting the five bare
  one-byte operators (`09 0A 0B 0C 0D`, `BARE_BINARY_OPS`) de-conflated this one
  in a single warm scan, with the numerator unchanged and all 219 moved keys
  summing to exactly 0.
- **Two free axes said zero and the row was ranked anyway.** The same document
  says the frame axis refutes candidates for free and names `expr-op-0x99` as the
  row to refute that way. That row *became* the bit-and row, and the answer was
  sitting in the baseline scan's own cross-tables: **102,379 of 102,382 are
  `calls-2plus` and 102,370 are `cflow-if-1`** — `if (p->Flags() & k)`, which
  needs a frame *and* basic blocks before it needs an `and`. The takeable
  population is at most 4. Two additions to the rule as stated. **The
  control-flow axis refutes for free too**, it is printed in the same block as
  the frame axis, and either one alone settles this row. And **read the
  cross-tabulation before writing the estimate, not after**: doing so would have
  made W37's estimate "0, range 0–4" instead of "8,000, range 1,500–30,000" and
  the whole rung a fifteen-minute decline. `docs/rungs/2026-07-31-bit-and-declined.md`.
- **An estimate built from "what token comes next" beat four estimates built
  from a ratio.** Every recent estimate that missed was a *scaling* — a
  67× row-to-counterfactual prior, a sibling family's `-whole` rate — and each
  missed by 1.3× to 3.0×, one of them in the wrong direction and outside its own
  range. W37's was built instead by tabulating the four C++ spellings of `x & k`
  and asking, for each, which IL token follows the operator: a result annotation
  (`-whole`), a branch, a compare, a store. Three of the four rows were right and
  the fourth was right in kind, and the direction was called correctly for the
  first time in five rungs. The generalizable form: **a prediction about the next
  token is checkable against the grammar you already have; a prediction scaled
  from another row's rate is checkable against nothing.** Where both are
  available, the first is the estimate and the second is a sanity bound.
- **The whole `&` operator is worth zero, measured at both of its rows, and that
  is a fact about the workload rather than about the port.** 134,763 functions —
  7.2 % of everything blocked — stop at a `0B`. Admitting the token in
  `parse_expr` releases every one of them and moves the census numerator by
  **exactly 0**: 32,368 of the free-standing `expr-bit-and` row land on
  `expr-brtrue` and 102,374 of the member-call row on a `brfalse`/`brtrue` one
  token later. `&` on this corpus is a **condition**, never a value. The reusable
  observation is not about `&`: it is that a row's *operator* can be perfectly
  homogeneous and still be a control-flow row, because an operator names what is
  computed and says nothing about what consumes it. The `-then-` key's second
  half is what says that, which is the whole argument for the instrument fix
  above.
- **`:eof` was a rendering of one field, and 63 % of what it claimed was false.**
  `Block::feature` printed `<ctx>:eof` for **any** block with `byte: None`, and
  ~73 producers raise a byte-less refusal at a *mid-parse* cursor — a post-parse
  predicate over the decoded operand list, a parameter width `.sy` withheld, a
  name-level refusal. The suffix is not decoration: it is read as "the refusal
  was raised *after* the parse reached the segment end", which makes every
  function under the row grammar-complete by construction and its count directly
  a widening estimate. `assign-dst-not-formal:eof` was ranked at 13,887 on
  exactly that reading and measured **+0 twice**.

  The repair is one field. `Block` now carries `seg_len` beside `off` — an offset
  is meaningless without the frame it indexes, so the two travel together — and
  the renderer earns `:eof` from `off == seg_len` and prints `:mid` otherwise.
  Both routes to that offset are **exact, not approximate**: `blk` reads
  `seg.get(p)` at the live cursor, which is `None` only past the last byte, and
  the two post-parse gates use `Block::at_end`, which is sound because
  `eat_fn_tail` returns `Ok` *only* at `p == seg.len()` — so an accepted body's
  cursor **is** the segment end. `Block::refuse(seg, off, ctx)` derives `seg_len`
  from the segment rather than taking it, so no producer can record an offset
  against the wrong frame; adding the field made all 98 construction sites a
  compile error, which is how the enumeration was got rather than remembered.

  Measured over the 878-TU workload, census numerator **unchanged at 691,744 /
  2,462,571 (28.09 %)**, blocked total unchanged at 1,770,827, every row summing
  exactly. Of **26,935** functions under a `:eof` key, **9,848 were genuine and
  17,087 (63.4 %) were not**:

  | ctx | claimed `:eof` | genuine `:eof` | actually `:mid` |
  |---|---|---|---|
  | `param-width-undetermined` | 6,974 | 0 | 6,974 |
  | `call-arg-computed` | 5,544 | 5,537 | 7 |
  | `expr-out-of-class-bare-nonformal` | 4,127 | 4,127 | 0 |
  | `call-args-none` | 3,299 | 0 | 3,299 |
  | `this-undetermined` | 2,568 | 0 | 2,568 |
  | `param-multi-reg` | 1,851 | 0 | 1,851 |
  | `expr-ptr-arith` | 1,678 | 0 | 1,678 |
  | `call-arg-outer-formal` | 695 | 1 | 694 |
  | `expr-out-of-class-formals9` | 125 | 125 | 0 |
  | `module-end` | 48 | 48 | 0 |
  | `formals-marker` | 16 | 0 | 16 |
  | `call-arg-nonformal` | 8 | 8 | 0 |
  | `mcall-framed-args` | 1 | 1 | 0 |
  | `callee-unresolved-tail-call` | 1 | 1 | 0 |

  Three things generalize. **One `ctx` can legitimately be both**, and the two
  mixed rows are the proof the split is a real property and not a relabelling:
  `call-arg-computed` and `call-arg-outer-formal` both reach `tail_call_shape`
  from a *statement* call (`void f(int a,int b){ h(a+1,b); }`), whose return
  plumbing is already consumed, and from a *value* call (`return h(a+1,b);`),
  whose plumbing is not — the same predicate, the same key, opposite ends of the
  segment. Reproduced from hand-written source through the live toolchain before
  the table was believed, four keys at once (`work/eof/probe/p3.cpp`).
  **The complement had to be its own bucket**, not a merge into `<ctx>-0xNN`:
  merging is the one failure a census instrument cannot survive (the type-id
  shattering above), and here it would have hidden the split entirely.
  And **the six rows that were 100 % spurious are all pre-parse refusals** —
  widths withheld at the `LO` marker, a formals marker that never bound, a
  receiver whose class is undetermined — which is the shape of the error to
  expect next time: a gate that runs *before* the body is read cannot possibly be
  at the end of it, and was claiming to be.

  **The free corroboration, and it is the same tell that found the defect.** The
  D6 frame measure walks the whole segment counting CALL tokens and does not stop
  where the parse stops, so it is independent of the key. Stated positively: a
  row that is genuinely `:eof` has had its whole body consumed by the modeled
  grammar, so **every function under it must carry a call count that grammar can
  produce** — and it does. `expr-out-of-class-bare-nonformal:eof` reads
  `calls-0` on **4,127 of 4,127** (a straight-line arithmetic body admits no CALL
  token at all); `call-arg-computed:eof` reads `calls-1` on **5,537 of 5,537** (a
  single statement call); `expr-out-of-class-formals9:eof` 125/125 and
  `module-end:eof` 48/48 are `calls-0`. Across all eight `:eof` rows, 9,846 of
  9,848 agree, and the 2 that do not sit well inside `call_tokens`' own measured
  1.3 % error. The `:mid` rows are the opposite picture — every one of them is a
  *mixture*, and **2,883 of `param-width-undetermined:mid`'s 6,974 are
  `calls-2plus`**, multi-call bodies that cannot be at the end of a grammar with
  no production for them. This is the frame axis used to **refute**, never to
  rank; it is the identical move that caught `assign-dst-not-formal` through
  `cflow-loop`, and it cost one query against a scan that was already on disk.

  **Reading older records.** Six key names in this repo's history no longer
  exist: `param-width-undetermined:eof`, `call-args-none:eof`,
  `this-undetermined:eof`, `param-multi-reg:eof`, `expr-ptr-arith:eof` and
  `formals-marker:eof` are now the same rows spelled `:mid`, in full and with the
  counts unchanged — `docs/IL_STORE_LEAF.md` §, `docs/IL_CALL_IN_EXPR.md` §21 and
  its two tables quote them as they read at the time and are left alone. The two
  mixed keys split rather than moved. Nothing merged, so every recorded
  comparison remains valid; only the suffix that was never true was withdrawn.

---

## 7. The lane that existed and was not enumerated (2026-07-31)

*Appended as its own section rather than folded into §6's instrument-failure log,
because a peer lane is editing §6 concurrently. Same class of finding; it belongs
with those entries and should be read beside them.*

**A lane that exists but is not enumerated is a lane that does not run.**

`scripts/mode_lane.sh` has taken a mode plus arbitrary flags since it was written,
and it has always worked. Nothing enumerated the lanes. There was no registry, no
gate command, no test — so the set of lanes that ran on any given day was the set
whoever was at the keyboard remembered to type. The four recorded throughout these
docs are `/Ox`, `/O1`, `/O2` and `/Ox /Gy`, and **not one of them compiles `/EH`**,
on a workload whose every TU is compiled `/EHsc`. Two `/EHsc` lanes were added on
2026-07-31, both green, and they caught a live wrong-bytes emit every other lane
was blind to. Nothing whatsoever made them run again the next day.

That gap had already made the entire EH surface vacuous once: a defect exposing
35,964 already-in-class functions survived two rungs because every standing lane
compiled without `/EH`, so the row collapsed onto its own control. This is the
third time the same rule has been paid for — **a green run is sound only over the
configurations it was RUN at** — and the first two payments both produced a fix
that added a lane. Adding a lane does not close it. Only enumerating them does.

Closed by `scripts/lanes.txt` (the list, in one place, as data) plus
`scripts/gate.sh` (the one command that runs it), and made binding by
`crates/c2-harness/tests/lane_registry.rs`. Five things are worth carrying
forward past this instance:

- **A lane's absence must be visible as an absence.** Each lane now prints a
  `LANE-RESULT` line and the gate re-derives the verdict from its fields; a zero
  exit status is not accepted as evidence that a lane ran. The result table is
  rendered by walking the **registry**, never by walking whatever result files
  happen to exist, and its row count is compared against the registry length
  before any verdict is computed. A lane that dies, is killed or is skipped by the
  loop is `NO-RESULT`, which fails and is named. Demonstrated live: a lane patched
  to `exit 0` before grading is reported `NO-RESULT`, not `PASS`.
- **`mode_lane.sh` had the vacuity hole `sweep_mode.sh` was written to close.**
  Every check `sed`-ed a number out of the gap report, and a number that is not
  there parses as zero — so a lane in which nothing was graded passed: `mismatch`
  absent → 0 → exit 0, a green row in any table, and no denominator anywhere to
  contradict it. The SKIP pre-check does not cover it, because SKIP is the
  toolchain being **absent** and this is the toolchain being **present and
  refusing everything**. One rule, two implementations, and only one of them had
  the guard. It is now checked **positively — the lane must have GRADED
  something** — and never as an enumeration of the ways a run can come back
  empty, because the next empty run will be empty in a way nobody enumerated.
  Demonstrated live with a realistic cause (a forced include of a missing header:
  197/197 `C1034` capture-fail), which the gate reports as
  `vacuous — 0 of 197 graded`.
- **All-SKIP is not green, and partial-SKIP is a failure.** Toolchain absence
  exits 0 per the CLAUDE.md hard constraint, but prints `GATE: SKIPPED … NOTHING
  WAS GRADED` and says in the headline that the run establishes nothing. Absence
  of the toolchain skips *every* lane, so some lanes skipping while others run
  means a lane declined for a reason of its own — a fault, not a degradation.
- **A cross that manufactures aliases buys breadth on paper and none in fact.**
  The lane set is `6 code-shape configurations × the EH axis = 12`, and each
  configuration was kept only where it graded the 197 fixtures **differently**
  from one already in the list: `/Ox /Gy` differs from `/Ox` in 8 rows, `/O1 /Oi`
  in 6, `/O2` in 6, while `/O1 /Gy`, `/O2 /Gy` and `/Ox /Oi` differ in **0** —
  `/O1` and `/O2` already imply `/Gy` (`OPT_MODE.md` §3.3) and `/Ox` implies
  `/Oi`. Crossing both flags everywhere would have added four lanes grading
  nothing new.
  **But note carefully what that measurement does and does not license.**
  `/O1 /EHsc` also differs from `/O1` in **0** verdict rows today, and it is
  emphatically not redundant: the reference obj is a *different obj* (the `/EHsc`
  capture of `w27_fp_reg` is 4,662 bytes against 4,654), so the port is
  reproducing genuinely different output and merely arriving at the same verdict.
  **Verdict-identical is not redundant.** `/Gy` on `/O1` is dropped because the
  flag is already implied, not because its rows matched; the two look alike in a
  verdict table and are not alike, and a registry pruned on the table alone would
  have deleted the `/EHsc` lanes as duplicates.
- **A property this load-bearing cannot be asserted only by a script somebody
  runs by hand.** For its first day, the only thing checking that the shipped
  registry still carried an `/EH` lane was a case inside `gate.sh --selftest` —
  so a "tidy up the lane list" commit would have been caught by nothing `cargo
  test` runs, and the failure would have been silent in exactly the way this
  whole section is about. The assertion now lives in
  `crates/c2-harness/tests/lane_registry.rs` (portable, no toolchain): the
  registry parses to a **positive** lane count, the `/EHsc` axis is crossed over
  **every** base configuration, some lane actually *varies* `/Oi` (a `/Ox /Oi`
  lane would not — `/Ox` already implies it), `/O1` and `/O2` are separate lanes,
  and `/O1 /EHsc` is required by name so a future prune cannot delete it for
  grading zero new rows. Each of those was **observed failing** against a
  deliberately mutated registry before being believed; the count floor is checked
  first and masks the specific assertions on a deletion, so the mutations that
  demonstrate them substitute flags instead and keep the count at 12. The
  `gate.sh --selftest` case is kept as a strictly weaker subset for machines with
  no cargo, and is labelled as such so it is not read as a second definition.

**The gate reproduced the bug class on itself within an hour of existing**, which
is worth recording precisely because it shows how low in the stack this shape
sits. `parse_registry` filtered rows with `awk 'NF >= 2'`, so a row carrying a
slug and no flags was **silently dropped**: `--list` reported one lane fewer than
the file contained, and the gate would then have run, and faithfully reported on,
a list that was not the list in the file. Every honesty property above would have
held — over the wrong registry. The fix is the same positive shape as everywhere
else: count the non-comment rows *before* parsing and require the two counts to
agree, so a row that does not parse is named rather than absent. **A component
built to make absences visible will still have absences of its own; the rule has
to be applied to the instrument, not just through it.**

Two more findings fell out of building it, both of the same shape as the `/EH`
gap:

- **No lane had ever passed `/Oi`.** The dc3 workload compiles `/O1 /Oi /EHsc`;
  `/Oi` moves 6 of the 197 fixture verdicts and had never appeared on any lane.
  The workload's exact profile is now the lane `O1-Oi-EHsc`.
- **`/O2` does not exercise the port's `/O1` code path.** A deliberate break
  planted in the `OptMode::O1` arm of `straightline.rs` failed all four `/O1`
  lanes and **left `/O2` green**, so `/O2` is not standing in for `/O1` in any
  lane budget. That is a fact about the port's mode mapping, established by the
  gate demonstration rather than assumed from `OPT_MODE.md`'s "same layout,
  differing only in the mode".

**Has anyone seen it fail?** Yes, four ways, all reproduced:

| break | gate says |
|---|---|
| wrong scratch register in the `OptMode::O1` arm (scratch copy) | `GATE: FAIL`, names `O1`, `O1-EHsc`, `O1-Oi`, `O1-Oi-EHsc`; `/Ox`, `/Ox /Gy`, `/O2`, `/Od` stay `PASS`; raises the MISMATCH alarm |
| a lane patched to `exit 0` before grading | `GATE: FAIL`, names `Ox-Gy`, `Ox-Gy-EHsc` as `NO-RESULT` |
| a lane whose every capture fails (`/FIno_such_header.h`) | `GATE: FAIL`, names it `vacuous — 0 of 197 graded` |
| sibling `wibo` not resolvable from a scratch tree | `GATE: SKIPPED … NOTHING WAS GRADED`, exit 0, explicitly not green |

`gate.sh --selftest` keeps all of that as 15 automated cases. It drives the real
`collect`+`decide` path with fabricated lane logs — not a reimplementation, which
would only prove the copy agrees — needs no toolchain, and asserts among other
things that the shipped registry still carries an `/EH` lane, so deleting those
rows is noticed. It also asserts its own case count, because a truncated selftest
is precisely the failure it exists to catch.

**Cost of the full gate**, 32-core host, `C2RS_JOBS=8`, 2,364 fixture-verdicts:

| capture cache | `--jobs 1` | `--jobs 4` | `--jobs 12` |
|---|---|---|---|
| cold (2,364 real `cl.exe` captures — every lane's flag string is a new key) | — | **6 s** | — |
| warm | 4 s | 1 s | <1 s |

Quote the cold number: it is what the gate costs the first time a tree runs it,
and the warm ones only hold once the flag strings are in the cache. Either way
this is nowhere near impractical, which is exactly why there was never an excuse
for the list to be implicit. Re-measure before adding an axis.

## 8. The emitted-function census — the binding, and a pre-registered estimate

`docs/ROADMAP.md` §8.2 ranks this instrument first on the board, and §8.4 names
the hole it fills: the published census numerator (**697,251 / 2,462,571 =
28.31 %**) counts **IL bodies**, and c2 emits **178,097** functions from those
2,462,571 bodies — **7.23 %**. The overlap between the numerator and the emitted
set was bounded only at **[22, 173,149] of 178,097**, which is to say unmeasured.
A body c2 never emits has never been graded by a byte compare and never can be:
the differential grades whole objs, and those objs do not contain it.

### 8.1 The estimate, registered before the instrument was run

Written down **before** the workload read-out, per the estimate discipline in
`docs/ROADMAP.md` §8.5. What was known at the time of writing:

* one TU by hand (`src/App.cpp`): 25 of 158 emitted functions in class = **15.8 %**;
* a 371-TU prototype over the largest cached objs — 142,205 emitted, 131,041
  bound 1:1, **27,307 in class = 19.20 % of emitted**, residue 7.85 %.

The remaining ~500 workload TUs are the *small* ones, ~36k emitted functions.
Counting refusals rather than applying a discount:

1. Their emitted sets are the same population — header-inline instantiations,
   destructors, small accessors — so the prototype rate transfers unchanged.
   Predicts ~19 %.
2. Small TUs skew simpler, and a simpler emitted function is likelier in class.
   Pushes **up**.
3. Every residue row is excluded from the numerator by construction, so any
   measured figure is a **floor**, not a point.

**Registered estimate: 34,000 in-class ∩ emitted of 178,097 — 19.1 %; interval
[30,000, 40,000]; fail-closed floor = the measured bound-and-in-class count,
ceiling = that plus the whole unbound residue.**

The direction of error this instrument is *built* to have is downward: an
emitted symbol nothing binds is counted as residue, never as in class.

### 8.2 The route, and why the other one was not needed

Two routes were on the table: the `.gl`/`.sy` record binding plus obj symbol
names, or a **masked body hash** (the body bytes with the per-TU token fields
zeroed). **The record route was taken**, on evidence rather than preference:

* The obj half is not in doubt. Under `/Gy` (implied by the workload's `/O1`)
  c2 gives every emitted function its own COMDAT `.text` section, so *counting
  sections* is counting emitted functions and each section's leader symbol is
  that function's name. Two independent implementations agree on `src/App.cpp`:
  156 `.text` + 2 `.text$yd` COMDAT sections, 158 leaders.
* The IL half turned out to be **one over-fitted byte away from working**. The
  gate's framing predicate (`codec::gl_offset_framed`) pins `gl[o-5] == 0x10` —
  which is not a tag but the third byte of the *preceding* `80 <LE32>` field, so
  it demands that field's value lie in `0x1000..=0x10FF`. The fixtures all do.
  `src/App.cpp` does not (`0x19AB`, `0xA4F6`, …), which is why the gate's reader
  finds **34 records in a TU with 9,033 bodies and 158 emitted functions**.
  Requiring only the two high bytes to be zero finds **6,069**, of which 6,068
  land on a `4F 1F` function start.
* The masked hash was therefore never needed. It solves a *different* problem —
  recognizing the same inline body across TUs — and it could not have answered
  this one, because the question is which *symbol* a row is, and a body hash
  carries no symbol.

The known obstacle was real and is unrelated to either route:
`Bindings::positional` reports names for ~0 real functions, and it still does.
`FnCensus::emit_name` is a **third** binding beside the gate's and the census's,
per record and diagnostic-only.

Two rules make it work on real input where the gate's does not:

1. **Containment, not equality.** A record binds to the segment *containing* its
   body-start offset. 6,068 of App.cpp's 6,069 record offsets are `4F 1F`
   markers but only 5,908 are *census* segment starts, because the census anchors
   on the `LO` body marker; equality would drop the other 160 into a residue
   bucket that meant nothing.
2. **The 32-byte name bound, which is load-bearing.** Without it a record whose
   shape this reader cannot frame borrows its predecessor's name. Measured over
   371 workload TUs: **3,799 emitted symbols claimed by two rows each without the
   bound, 0 with it**, and +706 in-class.

### 8.3 The read-out

Workload scan, 878 TUs, `/O1 /Oi /EHsc`, shared capture cache:

```
FUNCTION CENSUS (P2b): 697251/2462571 functions in class (28.31%)   ← unchanged
census/gate disagreement: 0        mismatch: 0

EMITTED CENSUS (§8): 34083/178968 emitted functions in class (19.04%)
  bound 161262 | residue 17706: 6879 compiler-generated (no IL body),
                                10827 unexplained  (9.89% of the denominator)
  ceiling if every residue symbol were in class: 51789 (28.94%)
```

**The answer is 34,083 of 178,968 — 19.04 %, with the true value in
[19.04 %, 28.94 %].** The interval is the residue, stated as a floor and a
ceiling rather than as a point with an error bar: the binding fails closed, so an
emitted symbol it cannot claim is residue and never numerator. §8.1's published
bound was `[22, 173,149]`; it is now 3.4 % wide of the denominator instead of
97 %.

The denominator reads **178,968** against the audit's 178,097 (+0.49 %, and the
median TU is 141 emitted against the audit's 139) — corpus-HEAD drift, the same
0.5 % the audit itself recorded against §6's independent 7.3 %.

**The pre-registered estimate (§8.1) was 34,000; the measurement is 34,083 —
0.24 % out, inside the [30,000, 40,000] interval.** The rate transferred from
the 371-TU prototype (19.20 %) to the whole workload (19.04 %) essentially
unchanged, so refusal #2 ("small TUs skew simpler, pushes up") was **wrong** and
refusal #1 was right. Recording that: the sample was already representative and
the adjustment was not needed.

### 8.4 What the number says, and what it does not

**28.31 % of bodies is 19.04 % of emitted code.** The two numbers are close, and
that is itself the finding: the fear in §8.1 — that the numerator might be
almost entirely header-inline bodies c2 never compiles, covering "0.01 % of the
code c2 actually emits" — is **refuted**. It is not 97 % either. The accepted
class covers about a fifth of the compiler's real output.

It also explains the flat TU-match count without appeal to anything else. A TU
matches only when **every** function in it is emitted byte-exact; at 19 % per
emitted function, a TU with the median 141 of them has no realistic chance, and
the six that match are the ones with almost nothing in them.

**The widening order over emitted code is not the widening order over bodies.**
Same scan, two rankings of the top rows:

| row | all bodies | emitted only |
|---|---:|---:|
| `expr-op-0x27` | 412,797 (23.4 %) | 22,831 (18.0 %) |
| `body-cflow-label` | — | 14,947 (11.8 %) |
| `expr-intrinsic-this-adjust` | — | 8,790 (6.9 %) |
| `expr-intrinsic-base-member-addr` | 41,678 | 6,472 (5.1 %) |

`c2rs gap` now prints both. Rank rungs off the second when the goal is TU match.

### 8.5 How the binding is graded, since the oracle cannot grade it

A byte compare grades emitted bytes; it cannot say whether row *R* is symbol *S*
(§6, and the `.sy` positional relaxation that was census +2,981, mismatch 0, and
wrong on 62 % of its bindings). So the binding is held to invariants stated
positively, all of them printed on every scan:

| check | reads |
|---|---|
| **Injectivity** — a name two rows claim binds to neither; a row two records claim binds nothing | 233 name-conflicts, 33,552 records lost to row-conflicts, **0 emitted symbols claimed twice** |
| **Totality over records** — `records == bound + outside + nameless + row-conflicts + name-conflicts` | 1,515,160 records, **0 accounting breaks** |
| **Totality over symbols** — `emitted == bound + generated + unexplained` | 178,968 = 161,262 + 6,879 + 10,827, exact |
| **Ground truth** — on a byte-exact TU the oracle *has* graded the whole symbol table, so `in-class == emitted` there | **6 TUs, residue 0** |
| **Denominator self-check** — a COMDAT `.text` with no leader symbol refuses the whole obj | **0 refusals in 871** |

The totality identity earned its keep immediately: it read **607 breaks** on the
first workload run, because `dropped_row_conflict` was counting *rows* while the
identity is stated over *records*. A row conflict consumes two or more records
and cost one. Fixed, and pinned by
`a_three_record_collision_accounts_for_all_three_records`.

**Negative controls**, each holding the guard's quantity fixed while mutating one
thing, each with its own failure message:

* the name-distance bound — same two records, only the padding moves: at exactly
  32 both bind, at 33 the second binds **nothing** and above all does not borrow
  its predecessor's name;
* a record pointing before the first row — same two records, only the offset
  moves: counted as `outside`, never clamped onto row 0;
* a broken framing — same name and offset value, only the two separator bytes
  move: **no record at all**, rather than a record bound to whatever the next
  four bytes read as;
* a `match` TU whose symbols do not all bind — same one byte-exact TU, only the
  bound count moves: the ground-truth check reads 3, not 0;
* a truncated obj and an aux count past the symbol table — same one COMDAT
  `.text` with one leader: `None`, never a short emitted set;
* a COMDAT `.text` with no leader — same two sections: `None`, never a set of one.

### 8.6 The residue is a reader limit, not a population — and that is the next job

**10,827 unexplained, and 57.4 % of it is ordinary `?…` functions:**

| class | count | share |
|---|---:|---:|
| ordinary | 6,214 | 57.4 % |
| dtor | 3,372 | 31.1 % |
| ctor | 807 | 7.5 % |
| operator / template-operator / special-generated / undecorated | 434 | 4.0 % |

If the residue were "c2 synthesized it, there was never a body", it would sit in
the special-member classes. It does not. The binding is **losing ordinary
functions**, and the loss channel is named: **152,941 records are `nameless`** —
framed, but with no symbol run ending within 32 bytes. Recovering them is what
would narrow the interval, and it is a `.gl` record-shape question, not a census
one. The `dtor` and `ctor` rows are *not* laundered into the generated bucket
without evidence; an implicitly-declared destructor plausibly has no IL body, but
plausibly is not measured.

### 8.7 The near-match TUs, and 14 of the published 46 have no functions at all

`docs/ROADMAP.md` §8.2's leading indicator reconciles **exactly**, and the
reconciliation matters:

| distance | measured TUs | + TUs with no functions | published |
|---|---:|---:|---:|
| ≤ 0 | 1 | 14 | 15 |
| ≤ 1 | 10 | 14 | **24** |
| ≤ 100 | 32 | 14 | **46** |
| ≤ 1000 | 207 | 14 | 221 |

The 14 are 7 `capture-fail` (never measured), 5 `match` empty modules and 2
deliberately-refused empty ones. **So the actionable near-match set is 32, not
46** — and 24-within-1 is really 10.

| blocked | emitted | in class | src |
|---:|---:|---:|---|
| 0 | 2 | 2 | `src/system/utl/Spew.cpp` (match) |
| 1 | 1 | 0 | `src/Main.cpp` |
| 1 | 1 | 0 | `src/system/math/Primes.cpp` |
| 1 | 1 | 0 | `src/system/math/Sort.cpp` |
| 1 | 1 | 0 | `src/xdk/LIBCMT/osfinfo.cpp` |
| 1 | 1 | 0 | `src/xdk/LIBCMT/undname.cpp` |
| 1 | 1 | 0 | `src/xdk/LIBCMT/vswprnc.cpp` |
| 1 | 1 | 0 | `src/xdk/nuispeech/xboxheap.cpp` |
| 1 | 1 | 0 | `src/xdk/xjson/jsonwriter.cpp` |
| 1 | 1 | 0 | `src/xdk/xlrc/xlrcimpl.cpp` |
| 2 | 1 | 0 | `src/ChecksumData_xbox.cpp` |
| 2 | 2 | 0 | `src/system/negate_test.cpp` |
| 2 | 2 | 0 | `src/system/synth_xbox/Biquad.cpp` |
| 2 | 2 | 0 | `src/xdk/LIBCMT/vsnprnc.cpp` |
| 3 | 3 | 0 | `src/system/rndobj/wordwrap.cpp` |
| 3 | 3 | 0 | `src/system/utl/Pool.cpp` |
| 3 | 1 | 0 | `src/xdk/nuiapi/nuidetroit.cpp` |
| 3 | 11 | 8 | `src/xdk/nuispeech/mmio.cpp` |
| 4 | 4 | 0 | `src/system/synth_xbox/IPP_basicmath_xbox.cpp` |
| 4 | 4 | 0 | `src/xdk/nuispeech/xboxmem.cpp` |
| 5 | 5 | 0 | `src/system/utl/EncryptXTEA.cpp` |
| 7 | 3 | 0 | `src/system/net/JsonMemory.cpp` |
| 8 | 2 | 0 | `src/system/math/Rand2.cpp` |
| 8 | 7 | 1 | `src/system/oggvorbis/VorbisMem.cpp` |
| 8 | 13 | 1 | `src/system/synth_xbox/MeterEffect.cpp` |
| 12 | 14 | 1 | `src/system/synth_xbox/HeadsetXferEffect.cpp` |
| 14 | 6 | 0 | `src/system/os/CritSec.cpp` |
| 18 | 20 | 2 | `src/keygen_xbox.cpp` |
| 19 | 24 | 8 | `src/system/utl/TempoMap.cpp` |
| 23 | 14 | 0 | `src/xdk/LIBCMT/rtti.cpp` |
| 27 | 9 | 2 | `src/xdk/nuiapi/headtracker.cpp` |
| 91 | 45 | 9 | `src/system/synth/Pollable.cpp` |

**Read the `blocked` and `emitted` columns together.** In 22 of the 32, `blocked`
and `emitted` are within one or two of each other — every remaining blocked body
in those TUs is a body c2 emits, so the distance is real work, not bookkeeping.
Nine of them are one blocked *emitted* function away from a whole byte-exact TU,
which is the cheapest thing on the board that moves the payoff metric.

### 8.8 What this does not do, and what it costs

* **It changes nothing.** Census 697,251 / 2,462,571, disagreement 0, mismatch 0,
  gate 12/12 with 2,412 fixture-verdicts — all identical to the pre-change
  baseline. `EmitBinding` is read by the report and by nothing else; the emitter,
  `shape_to_function` and acceptance never see it.
* **The gate's own framing predicate is untouched.** Loosening
  `codec::gl_offset_framed` would move the accepted class, and this is
  instrumentation. That the instrument's framing is the better reading of the
  format is now recorded (`the_gates_framing_sees_one_record_where_the_instrument
  _sees_three`); acting on it is a separate, gated decision.
* **`emit-in-class` is still not oracle-graded per function.** It is graded per
  function *only* on the 6 byte-exact TUs. What it fixes is narrower and worth
  stating exactly: the numerator now has a denominator made of code that appears
  in an obj, so it is at least the kind of claim a byte compare *could* grade.
* **Cost**: one extra `.gl` pass per TU. Warm-cache scan 3.7 s for 878 TUs.

## 9. The nine one-away TUs, measured (W-ONEAWAY, 2026-08-01)

### 9.1 The estimate, registered before any conversion was attempted

Written after step 1 (running `c2rs census` on all nine) and **before** any code
was touched, per the ceiling rule.

> **Estimate: 0 of the 9 TUs converted to byte-exact. Interval [0, 1].**
> Unit is **TUs**, matching the unit of the change.
>
> **Bias: optimistic.** My prior on reading the brief was 2–3. The two axes that
> collapse it — control-flow class and EH class — are already printed by the
> census and cost nothing; I had simply not crossed the near-match list with
> them. The single live candidate is `src/xdk/nuispeech/xboxheap.cpp`, and its
> blocker sits on a row the roadmap has three times measured as a **reservoir,
> not a rung** (`expr-op-0x27`, 0.14–2.5 % completion). I expect the actual to be
> 0 and the failure mode to be that 404 B of straight-line body contains far more
> refusals than the one the first-blocker histogram shows.

### 9.2 The nine, by name, key, and the two axes that decide them

`c2rs census <tu> --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp`,
one capture each, at the workload's own `/O1 /Oi /EHsc`. Every TU below reports
`0/1 functions in class` — one body, and it is the one c2 emits.

| TU | function | blocker key | cflow | EH | body |
|---|---|---|---|---|---:|
| `src/Main.cpp` | `?Run@App@@QAAXXZ` | `param-width-undetermined:mid` | straight | **eh-state1** | 222 B |
| `src/system/math/Primes.cpp` | (unnamed) | `expr-jump` | **loop** | none | 294 B |
| `src/system/math/Sort.cpp` | `?HashString@@YAHPBDH@Z` | `assign-store-type-0x86` | **loop** | none | 261 B |
| `src/xdk/LIBCMT/osfinfo.cpp` | (unnamed) | `expr-cmp-ge` | **if-n** | none | 445 B |
| `src/xdk/LIBCMT/undname.cpp` | (unnamed) | `expr-cmp-ne` | **if-n** | none | 532 B |
| `src/xdk/LIBCMT/vswprnc.cpp` | (unnamed) | `expr-cmp-eq` | **if-n** | none | 508 B |
| `src/xdk/nuispeech/xboxheap.cpp` | `?AllocatePageBlock@CXboxHeap@NUISPEECH@@AAAPAU_BLOCK_ENTRY@12@I@Z` | `expr-op-0x27` | straight | none | 404 B |
| `src/xdk/xjson/jsonwriter.cpp` | `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` | `expr-brfalse` | **loop** | none | 1349 B |
| `src/xdk/xlrc/xlrcimpl.cpp` | `?CreateClient@CXLrcImpl@@YAPAVCXLrcClient@@PAI@Z` | `assign-rhs-call-0x26` | **if-n** | none | 519 B |

**7 of the 9 are not single basic blocks** — 4 `cflow-if-n`, 3 `cflow-loop`.
That is not a soft obstacle. `c2-il::func::census`'s
`every_in_class_row_is_a_single_basic_block` asserts that **every** in-class row
reads `cflow-straight`, and it holds on the workload over all 455,049 readable
in-class bodies. Converting any of the seven means Phase 6, the whole
control-flow phase, not a widening.

### 9.3 The comparison-spine ranking is an artifact — the fourth recorded case

`ROADMAP.md` §8.3 Phase 1 names the comparison spine as *"small by census but
**8 of the 17 TUs within 3 functions of matching block on a `cmp` row**, the
highest match-bucket leverage anywhere on the board."* The brief that opened this
lane said to **measure it, not assume it**. Measured, on all 17 (a census run per
TU, the nine of §9.2 plus the eight at distance 2–3):

**The count roughly reproduces and the conclusion does not.** Seven of the 17 TUs
carry a `cmp` first-blocker — `osfinfo`, `undname`, `vswprnc`, `Biquad`,
`vsnprnc`, `wordwrap`, `mmio` — nine blocked functions between them. And:

| function | key | cflow class |
|---|---|---|
| `osfinfo` [0] | `expr-cmp-ge` | `cflow-if-n` |
| `undname` [0] | `expr-cmp-ne` | `cflow-if-n` |
| `vswprnc` [0] | `expr-cmp-eq` | `cflow-if-n` |
| `vsnprnc` [0] | `expr-cmp-eq` | `cflow-if-n` |
| `mmio` [0] | `expr-cmp-eq` | `cflow-if-2` |
| `mmio` [1] | `expr-cmp-eq` | `cflow-if-n` |
| `mmio` [7] | `expr-cmp-eq` | `cflow-if-n` |
| `Biquad` [0] | `expr-cmp-eq` | `cf-expr-0x05` (`eh-unknown`) |
| `wordwrap` [2] | `expr-cmp-eq` | `cf-expr-0x05` (`eh-unknown`) |

**Nine of nine are inside a branch or an undecoded body. Zero are
`cflow-straight`.** The comparison spine (W6, `docs/CODEGEN_W6_COMPARE.md`)
lowers `return a <rel> k` **branchlessly**, to a boolean, in a single basic
block; not one of these nine sites is a boolean materialization. Every one is a
branch *condition*, and the body it sits in is refused by the control-flow
invariant regardless of what happens to the `cmp` — `c2-il::func::census`'s
`every_in_class_row_is_a_single_basic_block`, which holds on the workload over
all 455,049 readable in-class bodies.

**So widening the comparison spine converts 0 of the 17 TUs, and would have
converted 0 no matter how far it was widened.** The signal was real as a count
and empty as leverage, because the instrument that produced it — a first-blocker
histogram — cannot see the control-flow axis, which is precisely §6n's
*"a large blocking row is one of five things and a first-blocker histogram
distinguishes none of them"* applied to a **match-bucket** claim rather than a
census one. The cross that refutes it is free and was already in every scan.

### 9.4 What the nine actually need, and the one that is close

Crossing §9.2's table with the port's two hard axes:

| need | TUs | phase |
|---|---:|---|
| control flow (`cflow-if-n` / `cflow-loop`) | **7** | Phase 6 |
| the whole EH record (`eh-state1`) | 1 (`src/Main.cpp`) | Phase 5 |
| neither — reachable by widening | **1** (`xboxheap.cpp`) | — |

`src/Main.cpp`'s `App::Run` is `cflow-straight`, and it is still not reachable:
it is the only body in the nine with `maxState >= 1`, so at the workload's
`/EHsc` it mints a `__CxxFrameHandler` prefix, a second `.pdata` and an unwind
funclet (`docs/EH_RECORDS.md`). Its first blocker,
`param-width-undetermined:mid`, is a distraction.

**`src/xdk/nuispeech/xboxheap.cpp` is exactly three independent refusals away,
and they were counted by construction rather than estimated.** The TU is one
constructor; every line of it was decoded from the `.ex` and checked against the
header's member offsets. A probe ladder (`work/oneaway/p*.cpp`, `s*.cpp`) rebuilt
the body one statement at a time and censused each rung:

```text
  H::H(a,b){ mSize=size; }                                  1/1 in class  store-run
  H::H(a,b){ mSize=size; mCount=0; }                        0/1  expr-op-0x27
  H::H(a,b){ mSize=size; mFreeHead=this; mUsedHead=this; }  1/1 in class  store-run
  H::H(a,b){ …; B& listHead = mListHead; }                  0/1  expr-op-0x27
  H::H(a,b){ …; AllocatePageBlock(initSize); }              0/1  expr-call-in-expr-
                                                                 recv-load-then-plumbing-0x3A
```

so the three are **independent** — each refuses on its own, with the other two
absent — and they are:

1. **a literal value in a store run of more than one** (`mCount = 0;` among
   seven formal/`this` stores). Category (1), a private limit inside a
   recognizer that already exists (`leaf_store.rs`'s
   `if stmts.len() > 1 && any(value_is_lit)`). **Taken — see §9.5.**
2. **a local reference bound to an interior sub-object address**
   (`B& listHead = mListHead;`), which stores a *computed address* into a local
   and then uses that local as a base. Not taken.
3. **a framed member call on `this` with an argument, inside a statement list**
   (`AllocatePageBlock(initSize);`). Not taken — this is `ROADMAP.md` §8.3's
   Phase 4, whose governing rules are recorded there as fitted hypotheses with
   no mechanism.

Two of the three are real work at their stated size; none of the three is
category (3) or (4). **The nine are not "one function away" in any sense that
predicts cost** — the §8.7 column counts *functions*, and one function can be
three rungs or a phase.

The distance-2 TU immediately behind them, `src/ChecksumData_xbox.cpp`, is two
rungs away and one of them is `expr-call-in-expr-data-addr-1sym-then-plain-call-whole`
— the `-whole` family (`ROADMAP.md` §6u), not a leaf.

### 9.5 What was taken: WLR, the one-value literal store run

`docs/rungs/2026-08-01-lit-run.md`. Refusal (1) above, carved at the point where
c2's allocator stops mattering: a store run every statement of which stores the
**same** literal is one materialization hoisted to the top of the body plus the
stores in source order, at every length, width, base and mode probed. The
multi-value case stays refused, and the negative fixture carries the grid that
refutes all four allocation rules fitted to it.

* fixtures: `wlr_lit_run.cpp` census **21/21**, `c2rs diff` **`Port=Match`**;
  `wlr_lit_run_neg.cpp` census **0/11**, **`Port=NotImplemented`**.
* workload: census 697,251 → **703,047** (28.31 % → 28.55 %), **+5,796**;
  emitted census 34,083 → **34,169** (19.04 % → **19.09 %**), **+86**;
  mismatch **0**, census/gate disagreement **0**, TU match **6 → 6**.

**The +5,796 / +86 ratio is the useful output.** 1.5 % of the bodies this rung
admits are bodies c2 emits — the sharpest per-rung confirmation yet of
`ROADMAP.md` §8.1's finding, and a caution to anyone reading a body-census delta
as progress.

### 9.6 Two instrument defects found on the way, neither fixed here

* **The census names the wrong symbol when a body contains a call.**
  `xboxheap.cpp`'s single body is `CXboxHeap::CXboxHeap` — every line matches the
  source and the header's member offsets — and the census row reads
  `?AllocatePageBlock@CXboxHeap@NUISPEECH@@AAAPAU_BLOCK_ENTRY@12@I@Z`, the
  symbol the body *calls*. Reproduced from hand-written source: in the probe
  ladder only `p6`/`s6`/`s7` — exactly the probes containing a call — carry a
  name at all, and it is the callee's. So the §8.7 table's function names are
  unreliable for any row whose body makes a call, and `src/Main.cpp`'s
  `?Run@App@@QAAXXZ` should be read with that in mind. The blocker keys, the
  counts and both class axes are unaffected.
* **`scripts/gen_rung_index.sh` writes `INDEX.md` itself and prints a progress
  line.** `gen_rung_index.sh > docs/rungs/INDEX.md` therefore produces a file
  whose first line is `wrote /…/INDEX.md` and which is missing its own header —
  a shell idiom that is right for every other generator in this tree and wrong
  for this one. Caught by `rung_registry.rs`, which is the point of it.

### 9.7 The seventh live wrong-bytes emit, and it was not in the new code

Adding a sweep fragment for §9.5's rung (`scripts/sweep.d/84-lit-run.py`) turned
up **7 mismatches on its first run** — an ALARM, and the finding outranks the
rung.

**`emit_load_imm` emitted a redundant `ori dest,dest,0` for every wide literal
whose low 16 bits are zero.** c2 emits `lis` alone:

```text
  s->a = 65536;      3d600001            lis r11,1   — port emitted 3d600001 616b0000
  s->a = 131072;     3d600002            lis r11,2
  s->a = 65535;      3d600000 616bffff   the HIGH half is emitted even when zero,
                                         so the elision is one-sided
```

**It is not the new rung's defect, and that was proven rather than argued.** The
smallest failing case is `void f(S* s){ s->a = 65536; }` — a store run of
*one*, the path WLR explicitly does not touch. Stashing the two crate changes
and re-grading gives `Port=Mismatch @ offset 8` on the pre-WLR tree. The defect
is as old as `emit_load_imm`, and `emit_load_imm` is the single locator for
every materialized constant in the port, so it was reachable from the chain
layer, the call-argument layer and the store layer alike.

**Nothing that existed before caught it.** 91 fixtures, 12 mode lanes, 14,122
generated cases and 878 real TUs were green over it, and the reason is exactly
`ROADMAP.md` §6n's: no axis anywhere varied a *literal's low half*. That makes
this the **seventh** live wrong-bytes emit found by a generated sweep against
zero found by hand-written fixtures, and it has the same shape as the other six
— it changes no operator and no shape, so no review would see it.

The fix is one `if`. Pinned by `wlr_lit_run.cpp`'s `kz1`/`kz2`/`kz3`/`kn2` — the
run of **one** included beside the runs, because that is the case that reproduces
— and by the fragment's value axis, which crosses
`{0, ±1, 7, ±32767/32768, 65535, 65536, 100000, 2147483647}` with run lengths
1..7. The negative wide load (`s->a = -65536;`, a bare `lis r11,-1` in the
reference) stays **refused**: it could be served by the same branch, `-70000` is
unwitnessed, and a fail-closed refusal is not a bug.

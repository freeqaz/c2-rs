# GAPS — the measured distance from here to real-TU coverage

Status: living worklist (written 2026-07-29, revised three times the same day —
for the P2b function-level census and the variable-token-width finding; then
for R1, the W5 chain mis-emit fix, W6 compare leaves and the CALL grammar; then
at end of day for R2/R3, W13a float leaves and the cast/intrinsic-call
characterization that refuted the `expr-cast` bucket name. All numbers
re-measured with `c2rs gap` / `c2rs census` at HEAD — nothing below is quoted
from memory). Companion to
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

| Claim | Number (2026-07-29) | Command that re-proves it |
|---|---|---|
| Standalone-c2 replay is byte-exact **including the COFF timestamp** on the whole capturable real workload | 871/871, 0 diverged — **re-proven at full strength 2026-07-29** on the post-token-fix code (43.4 s at `--jobs 16`), matching the 2026-07-20 full pass | `c2rs gap … --replay-every 1` |
| Standalone-c1 (front-end) replay is byte-exact | 25/25 fixtures | `c2rs replay-c1` |
| The port is byte-exact on its accepted class, fail-closed outside it | **18/37 fixtures Match**, rest NotImplemented, **0 mismatch** — and 0 mismatch across all 878 real TUs | `c2rs diff`, `c2rs perf`, `c2rs gap` |
| **Real-corpus TU coverage** (the tripwire metric, §5) | **match 6 / 878 (0.7%)** — nonzero since 2026-07-29 (R1) | `c2rs gap …` |
| **Real-corpus coverage, per function** (the headline numerator, P2b) | **79,718 / 2,462,571 functions in class (3.24%)** | `c2rs gap …` (FUNCTION CENSUS block), `c2rs census <cpp>` |
| Port speed where it works | geomean **1081× per obj** over the **17** fixtures matching at 4afcaa7 (2.1–5.0 µs vs ~4.0 ms); ~897k objs/s at 32 threads vs ~3.1k for real c2. `mvp_fmul3` has since joined the matching set (18) and the geomean has **not** been re-measured over it — quote the 1081× with its 17 attached, or re-run | `c2rs perf`, `c2rs perf-scale` |
| Test suite | green with toolchain present | `cargo test --workspace --release` |
| IL codec round-trip | `encode(parse(b)) == b` on the full fixture spread, fail-closed | `il_roundtrip.rs` (in the suite) |

> **On that speed figure**: an earlier revision published ~1524×, measured over
> 13 matching fixtures. The set was 17 when 1081× was measured — it had gained
> the empty TU (1841×), the empty-function TU (1122×), the W6 compare leaves
> and the `*`/`-` chains (852×) — so the two geomeans are **not comparable**,
> and the drop is a change of population, not a regression. Any per-fixture
> number that got slower would be; none did. The population has since moved
> again (W13a took it to 18), which is exactly why the rule is: **quote this
> metric with its fixture count attached, and re-measure before re-ranking it.**

The replay-soundness row is the foundation: the *reference* side of every
differential is real c2 on real code, so every other number in this doc is
measured against truth, not against an approximation of it.

> ### The mismatch bucket was nonzero once, on 2026-07-29
>
> **It is the first thing the differential has ever actually caught.**
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

`c2rs gap`, 878 dc3 TUs, real flags, ~36 s at `--jobs 16` (2026-07-29, end of
day):

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

- **2,462,571 functions** across the 871 capturable TUs, of which **79,718
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
  2. The headline metric is **functions in-class** (79,718 / 2,462,571) and
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

**79,718 / 2,462,571 functions in class (3.24%).** Progression across the day
on the identical instrument — the first two steps decode fixes, the third a
very small new class with an outsized count, the fourth a much larger piece of
codegen worth a fraction of it:

| | in class | % |
|---|---:|---:|
| start of day | 4,154 | 0.17 |
| + variable token width (GAP-1) | 7,114 | 0.29 |
| + CALL-token decode (GAP-1) | 7,954 | 0.32 |
| + empty function bodies (`w10_empty_fn.cpp`, a44c8f3) | 78,028 | 3.17 |
| + W13a float/double leaves (9c7ba7d) | 79,041 | 3.21 |
| + signed varint short form (66f408d) | **79,718** | **3.24** |

Top 8, percentages of the 2,383,530 *blocked* functions, with what each bucket
is now **known** to be (`docs/IL_CALL_GRAMMAR.md`, `docs/IL_CAST_CONVERT.md`):

| Functions | % | Feature | What the bytes are |
|---:|---:|---|---|
| 363,684 | 15.3 | `call-token-0xB9` | **member / indirect calls** — callee is an *expression*, not `26 <tok>` |
| 167,205 | 7.0 | `expr-intrinsic-call` | the `0x40` token — a **SECOND call token**, not a cast |
| 144,276 | 6.1 | `call-token-0x33` | the **same** intrinsic-call production, result assigned |
| 119,800 | 5.0 | `expr-call-in-expr` | a call nested inside an expression |
| 81,478 | 3.4 | `expr-load-type-864540` | **float** operand |
| 80,284 | 3.4 | `call-token-0x26` | `26 dest 26 callee BD …` — assign a call result |
| 75,081 | 3.1 | `expr-load-type-888541` | **double** operand |
| 70,078 | 2.9 | `body-0x53` | first statement is an `if`/compound |

`expr-load-type-864383` (**void\***) falls just below this cut and its count is
deliberately not re-quoted from the superseded scan. Behind the top eight is a
long tail of further distinct features (1,217 more rows at the mid-day
measurement; the row count has moved with every retirement since and is
likewise not re-quoted).

Two rows left this table today, for opposite reasons — the distinction matters
more than either number:

- **`body-0x3A` (107,253 / 4.4%) is gone because it was PORTED.** It was the
  empty function body; R2 accepted it. This is the only legitimate way a bucket
  leaves the histogram.
- **The whole `call-anchor-*` family is gone because it was never there.** See
  the box below.

Every percentage in the table also moved because the blocked denominator shrank
from ~2.45 M to 2,383,530. Do not diff raw percentages across scans without
that.

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
   double 3.1% = **6.6%**, with `void*` behind them: W13b/W12 demand, and
   knowing how to *skip* a `double` is still not knowing how to lower it. Then
   `body-0x53` (2.9%), a leading `if` — W8.

`body-0x3A` (4.4%) used to head this list and is no longer on it: R2 ported it.
W6 (comparisons) and W7 (shifts) remain absent from the top eight; W6's leaf
class landed anyway, on the strength of a staged fixture rather than measured
demand, and it moved the census by less than either decode fix did. W13a is the
second example of the same thing — a fixture-driven rung, worth 1,013 functions
against the 70,074 that R2's measured bucket was worth. Both are still the
argument for demand-driven ordering.

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
- **Measured result**: **79,718 / 2,462,571 functions in class (3.24%)** — §2b.
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
  add/sub/mul chains, void/int tail calls, one framed-call form, and since
  2026-07-29 the comparison leaf `<load> <lit> <rel> 2C`, the empty module, the
  empty function body and the float/double leaf — `IL_BUNDLE_MVP.md` plus
  `CODEGEN_W6_COMPARE.md` and `CODEGEN_W13_FLOAT.md` have the grammar).
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
- **Frequency**: **865/878 TUs (98.5%)**, i.e. 96.79% of the 2,462,571 real
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
  chains past two operations (W5 chains), empty function bodies (R2), the
  `/Gy` COMDAT obj shape (R3), the branchless compare→bool leaf (W6) and
  float/double leaves over parameters (W13a). The missing classes, with
  mechanisms per class, are the W5–W14 table in `ROADMAP.md` §G1: W5 expression
  **trees**, W6's `<`/`<=`/`>=` against a non-zero literal, W7 shifts/bitwise,
  W8 control flow, W9 div/mod, W10 general frames+locals, W11 generalized
  calls, W12 memory/struct access, W13b float **constants**, W14 data
  sections/globals — plus the intrinsic-call family (no longer long tail: it is
  ~13% of blocked functions, §2b) and a census-driven long tail proper (switch
  tables, 64-bit carry chains, virtual calls).
- **Five classes landed 2026-07-29**, every one characterized first:
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
  - **W5 chains** and **W6 compare leaves**, both below.
  - **W5 chains** (`CODEGEN_W5_SCRATCH.md`) — c2 allocates temporaries from a
    cursor descending `r11 → r10 → r9 → …`, skipping live registers and
    wrapping; the port now follows it for `*`/`-` chains and **refuses below
    `r9`**, since the deepest characterized chain is `a*b*c*d*e` and beyond it
    c2 recycles dead registers and schedules. This is the change that fixed
    the mis-emit (§1). Trees still fail closed — the eleven negatives of
    `w5_tree_neg.cpp` (product flattening, additive term-reordering, spilling
    into `r31`, the unexplained `n_imm_sum2` register order) are the reason.
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
  double, W13b/W12), then a leading `if` (2.9%, W8). Treat that as inference
  (see §2b), not as a codegen measurement.
  Staged fixture evidence exists for the W5-tree, W7, W8, W13b and
  cast/intrinsic classes (`w5_tree2/3.cpp`, `add3.cpp`'s
  `select_max`/`shift_mask`, `il_call_return.cpp`, `w13_*.cpp`,
  `il_convert_scalar.cpp`, `il_intrinsic_call.cpp`) — fixtures sample the
  grammar we guessed matters; the census measures the grammar that does. W6 and
  W13a are the worked examples of the difference: both landed on fixture
  evidence, and W6 moved the census less than either decode fix did while W13a
  moved it 1,013 against R2's measured 70,074.
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
  - **W-UNW-1**: `.pdata` label counters (`$M2545/…`) are a fixed seed for
    the first function but shift as preceding functions consume slots —
    resolved for single-function TUs, must be modeled per-function before
    W10/W11 touch multi-function TUs (a real TU averages ~2,800 functions
    over the corrected denominator — §2).
  - `.pdata` carries a real reflected-CRC-32 checksum; new sections mean new
    CONST/DERIVED byte classification work per `OBJ_FORMAT_MVP.md`.
  - **A second register model is a second set of rules, not a parameter.**
    W13a's FP allocator inverts almost everything the integer one asserts —
    pool order, whether `+` collapses to an accumulator, the operand order of
    the subtract, which field carries a multiplier
    (`CODEGEN_W13_FLOAT.md` §2, §3, §6). Each of those, grafted from the
    integer path, produces *wrong bytes* rather than a refusal. When a new type
    class arrives, re-derive its allocator from captures; do not parameterize
    the existing one.
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
| **W5 trees** | Multi-scratch expression trees (the liveness-gated, wrapping cursor + level-order emission of `CODEGEN_W5_SCRATCH.md` §7) | `w5_tree2.cpp`/`w5_tree3.cpp` exact; all 11 functions of `w5_tree_neg.cpp` still `NotImplemented`; census multi-scratch buckets move |
| ~~**W6 leaves**~~ **DONE 2026-07-29** | Compare→bool materialization, branchless, `k == 0` folds first | **Passed**: `il_bool_materialization.cpp` `Port=Match`, 6/6 in class. Census movement was small — the honest reading is that this rung was fixture-driven, not demand-driven |
| **W6 rest** | `<`, `<=`, `>=` against a non-zero literal; wide literals | The §4.5 spine's instruction order for a literal lhs is **pinned by capture** (it is currently UNRESOLVED, `CODEGEN_W6_COMPARE.md`), then exact |
| **W7** | Shifts + bitwise + strength reduction | `shift_mask` exact; hazard-listed encoders land exact-pattern, opt-in; census `09`/`0B` buckets move |
| **W8** | Control flow (first CFG; GAP-4 restructure lands here) | `select_max` + conditional shapes exact; restructure merged with **0 mismatch** on fixtures *and* the full 878-TU scan; census branch-token bucket moves |
| **W9** | Div/mod (incl. const-divisor multiply-high) | census div bucket moves |
| **W10** | General frames + locals + per-function `.pdata` counters (W-UNW-1) | multi-function TUs with frames decode+emit; first TUs where *every* function is in-class flip to `match` — target population: the 40 TUs with ≤10 functions |
| **W11** | Calls generalized (args r3–r10, stack spill, multiple calls) | census call buckets move; match bucket starts climbing the ≤100-fn TU population (79 TUs) |
| **W12** | Memory / struct access (`.sy` becomes load-bearing) | census memory buckets (`30`/`32`) move |
| ~~**W13a**~~ **DONE 2026-07-29** | Float/double leaves over parameters — a **separate** FP register model, not a parameterization of the integer one | **Passed**: `mvp_fmul3.cpp` `Port=Match`; `w13_fabi/fops/fscratch/fneg.cpp` all replay `ByteExact` and all keep returning `NotImplemented`, which is what pins the boundary. Census 78,028 → **79,041** (+1,013) — small, and fixture-driven rather than demand-driven, like W6 |
| **W13b** | Float **constants** — `.rdata` COMDAT per distinct value, `addis`+`lfs`/`lfd`, REFHI/REFLO pairs, plus the constants c2 *synthesizes* (`a+a`→`a*2.0f`, `x/k`→reciprocal multiply) | float-constant fixtures exact; the `CODEGEN_W13_FLOAT.md` §2.6 cursor/constant interaction resolved by capture first (it is currently UNKNOWN and blocks this rung); census float buckets move |
| **W14** | Data sections / globals (`ADDR32` relocs) | census global buckets move; match bucket now tracks whole subsystems |
| **P-F0.2→P-F2, P3** | Front-end track + compose (parallel, off the match-bucket critical path) | Grade 1 `PortC1 == captured` per file; Grade 2 `PortC2(PortC1(src)) == pipeline obj`; `compose` timed |

**R0, R0b and R0c have all run, and the W-numbering above is not the running
order.** The measured head of the histogram (§2b) now ranks the schedulable
work as ~~R2 (empty function bodies, 4.4%)~~ *[done]* → ~~`expr-cast`
characterization~~ *[done, and it refuted the name]* → **R4** (the
intrinsic-call family, `expr-intrinsic-call` 7.0% + `call-token-0x33` 6.1% =
**~13%** of blocked functions, one production — the largest schedulable thing
on the board) → **W11** (remaining call shapes, 8.4%) → **W13b/W12** (non-`int`
operand types: float 3.4% + double 3.1% = 6.6%, plus `void*`) → **W8**
(`body-0x53`, 2.9%), with W6/W7 absent from the top eight entirely, and R5
(the `read_varint` fix) cheap enough to slot in anywhere.

Two cautions on that ranking. First, **~13% is a decode rank, not an
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

Note also that the census numerator is the per-rung yardstick: R1 tripped the
match tripwire on empty TUs, but every remaining rung is judged by how many of
the **2,383,530** blocked functions it moves — and by which bucket they move
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
  bucket is 6/878 and the census is **3.21%** (§2, §2b). The verdict itself is
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

# 2. The real-workload gap scan (the census + scan gates; ~36 s at -j16).
#    Prints the TU buckets, the FUNCTION CENSUS numerator, and the top-20
#    blocking-feature histogram.
cargo run --release -p c2-harness --bin c2rs -- gap \
  --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --jsonl work/dc3-workload/scan-$(date +%Y%m%d).jsonl \
  --replay-every 25 --jobs 16

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
```

The rules that keep the numbers honest:

- **Byte-exact means byte-exact.** Replay: raw-identical including the COFF
  timestamp. Port: identical with only the 4-byte timestamp zeroed. No
  fuzzy thresholds anywhere; real c2 under wibo is the sole judge.
- **`mismatch` is an alarm, not a gap.** Any nonzero mismatch bucket — on
  fixtures or the workload — is a correctness bug that outranks all widening
  work. The port's value downstream depends on this bucket staying 0. This
  has fired exactly once (§1, `w5_chain.cpp`, 2026-07-29) and was fixed the
  same session, before any widening continued.
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

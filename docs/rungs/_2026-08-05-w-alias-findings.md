# w-alias — the tag-0x10 ALIAS decode is IN `crates/`, and a second implementation lands on w-emitp's Python **to the digit**: 96 220 / 95 820 / 95 818, **850 of 850 tables equal name for name**, `JFP_ALIAS` **308 / 0.94413**

    Tag:       w-alias
    Slug:      w-alias-findings
    Date:      2026-08-05
    Fixtures:  none — this lane ships a READER; no obj byte changes and no
               fixture could grade it
    Census:    706555/2463393 (28.68%) → unchanged, +0 — no `crates/c2-core`
               change; `PortC2` consumes nothing here
    Record:    this file; prereg `docs/rungs/_2026-08-05-w-alias-prereg.md`

    Lane:      w-alias, worktree `wt-w-alias` off master `9378b00` (confirmed
               by `git rev-parse`, and master had moved a dozen times that day)
    Prereg:    committed at `58efc67` BEFORE the first Rust number existed.
               Scored in §7.
    Ships:     `crates/c2-il/src/func/glalias.rs` (+ tests), the corpus dump
               test, two `DISCLOSURE.md` rows, and the `README.md` consequence.
               **Nothing under `crates/c2-core` or `crates/c2-harness`** — both
               are live under other lanes and were checked, not assumed.

**One line.** w-emitp §6 is implemented in `crates/c2-il` and it reproduces
**every** number w-emitp measured in Python — not three aggregates but **850
per-TU tables, name for name, 0 disagreeing entries** — and the model those
tables feed comes back at `JFP_ALIAS` **per-TU exact 308 of 850** and **micro-F1
0.94413**, `ALIAS_IN` **472 / 0.99243**, `ALIAS_REF` equal to `RGL` to the digit.

> ### Two implementations of one disassembly transcript, written independently, agree on **96 220** tag-0x10 records, **95 820** bound, **95 818** `??_E<X>` → `??_G<X>`, **352 / 0 / 48 / 0 / 0** failures, and the null binding **1 795 / 2 449** with **zero** pairs. Then the Rust table is substituted into a **byte-identical** copy of w-emitp's `scan.py` and every model on the page reproduces exactly.

> ### ⚠ AND THE TU VALUE OF THIS CHANNEL IS **+0 TODAY**, CONDITIONAL ON A RUNG THAT DOES NOT EXIST. Lane `w-reach` measured the join this lane deliberately did not: `JFP_ALIAS`'s **TU reach is 122 — exactly `JFP`'s 122, gained 0 and lost 0 by name.** The +176 per-TU-exact gain is real and it **converts nothing yet**. §5.1 carries the mechanism, the coupling and the numbers, and **no reader should take a figure on this page as TU movement.**

**And the lane's own instrument caught the thing it was built to catch.** The
one-character widening that accepts every kind-4 tag instead of `0x10`
**raises** the bound count 95 820 → **137 379** — a *better*-looking number —
while `dom(alias)` acquires **29 291 members that have a body**. Under w-emitp
§6 rule 4 ("never emit a name in `dom(alias)`") that is 29 291 symbols
suppressed that must be emitted. The corpus test's assertion fires and four unit
tests go red. **That is what "widening a reader is how a refusal becomes a wrong
emit" looks like when you build the counterexample first.**

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-alias`, **rebased onto master `dbcb855`** (it was cut at `9378b00`; master moved through `w-rdata` and `w-reach` during the lane, and every gate figure in §6 is from the **rebased** tip) |
| c2.dll | `compilers/X360/16.00.11886.00/c2.dll`, image base `0x10b00000` |
| wibo | `../wibo/build/release/wibo` (via `C2RS_WIBO`) |
| toolchain | `C2RS_COMPILERS` and `C2RS_WIBO` set **explicitly** before every measurement — board **#299**; `compilers/` is gitignored and absent from a fresh worktree, and without this every instrument degrades to `SKIP:` and exits 0 |
| IL | the harness's capture cache, w-joint's `cacheindex.py` output unchanged (`work/w-db/cacheidx.tsv`, paths made absolute). **850 of 857** |
| truth `E` | w-emit's `truth/`, 174 417 names |
| truth `D` | regenerated here with w-joint's `truth_data.py` **unmodified**: `\|D_all\|` **685 848**, `\|D_data\|` **232 156**, `\|D_lead\|` **205 808**, arity **1 534 428 = 1 147 426 + 387 002**, TOT residue **0**, A1/A2/A3 **0/0/0**, AGREE **850/850** — **w-emitp's and w-joint's figures to the digit**, which is this lane's known-answer control before any alias number is quoted |
| scratch | `work/w-alias/` (gitignored); scripts force-added, no IL, obj or `_CL_*` committed |

**The 21-TU quarantine is INTACT and w-emitpred's one-shot Part-1 gate is
UNSPENT.** §9 puts the question to the coordinator and does not answer it.

---

## 1. What was implemented, against §6 item by item

| §6 | requirement | status |
|---|---|---|
| **1** | Accept tag `0x10` in the `.gl` reader — kind-4 header + one `varU` at the `+0x54` position, gated on **RT + BIND**, not on the next record's header | **DONE** — `crates/c2-il/src/func/glalias.rs` |
| **2** | Build `alias: Token → Token`, asserting the corpus invariants rather than assuming them | **DONE** — `GlAliasTable` carries both the name- and token-keyed relations; every invariant is a published **count**, and one of them is an `assert` |
| **3** | Apply it **once**, at the `in` `02`-node resolution site only | **NOT IMPLEMENTABLE HERE, and registered as such in the prereg before the work started.** `PortC2` has **no emit-set model**, so that site does not exist in `crates/`. What ships is the operator the site would call — `resolve_name` / `resolve_token`, documented as once-only and non-transitive — and the prohibition on the reference list, in the module docs, with the measurement behind it |
| **4** | **Never emit a name in `dom(alias)`** | **DELIBERATELY NOT HARD-CODED.** §3 is why. The reader publishes `dom_with_body`, which is the count that makes the rule safe, and the corpus test asserts it is 0 |
| **5** | `DISCLOSURE.md` rows for `0x10b9c01e/24/30` and `0x10b99621/35` | **DONE** — rows **W-ALIAS-1** (adoption) and **W-ALIAS-2** (route). They are the **first two rows in that ledger**, so `README.md`'s clean-room claim moved from blanket to per-finding **in the same branch**, which that file's own checklist step 4 requires |

**What was deliberately refused, and why each refusal is not laziness.**

* **The `.in` `02`-node enumeration was not ported.** Enumerating those nodes is
  a *model* input, not a reader; `crates/c2-il`'s `.in` readers exist to serve
  the `.data` writer and element tag `02` is **refused** there on purpose
  (`ininit.rs` — a pointer-valued initializer carries its address entirely in a
  relocation). Widening that refusal to reach an emit-set model would widen a
  path that **emits bytes**, for a consumer that does not exist. It is not done.
* **The alias is not applied to the `.gl` reference list.** `ALIAS_REF` is `RGL`
  to the digit — `|P|` included — and this lane reproduced that with its own
  table. It is stated in the module docs so the next lane does not re-try it.
* **No transitive closure.** An alias never targets an alias; a reader that
  chased the chain would model something c2 does not do. There is a unit test.
* **`crates/c2-core` and `crates/c2-harness` were not touched.** `wt-w-rdata`
  has three commits on `crates/c2-core/src/coff/function.rs`; `wt-w-reach` owns
  `crates/c2-harness`. Checked with `git worktree list` before the first edit.

---

## 2. THE DECODE — two implementations, one transcript, and they agree

`crates/c2-il/tests/gl_alias_corpus.rs` (Rust) against `work/w-emitp/alias.py`
(Python, frozen), 850 TUs, no toolchain needed.

| | w-emitp (Python) | **this lane (Rust)** | |
|---|---:|---:|---|
| tag-0x10 records | 96 220 | **96 220** | ✔ |
| bound | 95 820 | **95 820** | ✔ 0.99584 |
| shape `??_E<X>` → `??_G<X>` | 95 818 | **95 818** | ✔ 0.99998 |
| `head_fail` | 352 | **352** | ✔ |
| `rt_fail` | 0 | **0** | ✔ |
| target does not bind | 48 | **48** | ✔ |
| self-alias / duplicate | 0 / 0 | **0 / 0** | ✔ |
| SHIFT null `p−1` bound | 1 795 | **1 795** | ✔ 0.01873 |
| SHIFT null `p+1` bound | 2 449 | **2 449** | ✔ 0.02556 |
| SHIFT null pairs, either direction | 0 | **0** | ✔ |
| `dom(alias)` with a body | 0 | **0** | ✔ |

> ### And the aggregate agreement is the WEAK form. `work/w-alias/cmp.py` compares the **850 per-TU tables name for name**: **850 of 850 agree on every count AND on every pair**, with **0** disagreeing entries in either direction. That was the lane's declared bias — that two run-walkers could agree in aggregate while disagreeing per TU — and it did not happen.

**Why the shape line is not circular, restated because it is the load-bearing
one.** The gate is RT + BIND. RT compares two token readers' widths; BIND asks
the `.gl` symbol index. **Neither mentions `??_E` or `??_G`.** The shifted reads
pass the *same* gate — they bind 1 795 and 2 449 times — and produce **zero**
pairs. The count null is 40×; the shape null is infinite.

**One thing the RT gate is honestly worth is less than it looks, and it is said
here rather than left to be discovered.** `read_token_var` and `var_u` take
their width from the same bit of the same byte, so RT is in practice a **bounds
check** — which is exactly why `rt_fail` is 0 on 96 220 records. It is kept
because a future width rule that split the two readers must fail loudly rather
than disagree silently.

---

## 3. THE COUNTEREXAMPLE, CONSTRUCTED BEFORE ANYTHING SHIPPED

The standing rule is that a widening needs the case that would make it wrong,
built first. This lane ships a *reader*, so the question is sharper: **what would
make a consumer of this table wrong?** Two answers, both constructed as byte
streams in `crates/c2-il/src/func/glalias/tests.rs`.

### 3.1 `shift_null_binds_yet_pairs_nothing` — the BIND gate is not self-validating

A `.gl` stream is built in which the alias's anchor holds `12 34 00 56`, and a
decoy record is planted under token `0x3400` — the token a read **one byte late**
produces. The null then **binds**, cleanly, and pairs `??_EFilePath…` to
`?decoy@@YAXXZ`.

> ### A decode that offered "the field bound" as its evidence would be offering **nothing**. What separates the true position from the false one is the **shape**, and the shape is not in the gate. This is why the null is a shipped public function and not a sentence in a doc.

### 3.2 `an_alias_that_also_has_a_body_is_counted` — the case that makes RULE 4 a WRONG EMIT

§6 rule 4 says never emit a name in `dom(alias)`. That rule **suppresses a
symbol**. A name carrying *both* a tag-0x0E body record and a tag-0x10 alias
record would be suppressed **and must be emitted**. The corpus says this never
happens — `dom(alias)` has **0** bodied members over 96 220 records — but a
reader that *assumes* it has turned a measured fact into a silent premise.

So the rule is **not hard-coded**. `GlAliasStats::dom_with_body` is published,
the constructed stream proves the counter works, and the corpus test **asserts**
the count is zero rather than printing it. §4 shows that assertion firing.

**This is also the answer to "what did you try that failed to produce a
counterexample".** Nothing failed: both cases were constructible, both are in
the tests, and one of them fires on the real corpus the moment the reader is
widened by one character.

---

## 4. THE COUNTERFACTUAL — broken transiently, measured, reverted in the same run

`work/w-alias/counterfactual.sh`. Both breakers are **one token**, both had
their predictions written into the script **before it was run**, and the script
asserts `git status --porcelain crates/` is empty after each revert. It is.

### 4.1 B1 — read the target field one byte late

Registered: `bound` and `shape` must collapse; `JFP_ALIAS` must fall to *exactly*
`JFP` and `ALIAS_IN` to *exactly* `ORACLE`; **and `tag10`, `RGL`, `INIT`, `SKIP`,
`ORACLE`, `JFP` must not move at all.**

| | intact | **B1** |
|---|---:|---:|
| tag-0x10 records | 96 220 | **96 220** — the RECORD is still found; only the FIELD moved |
| bound | 95 820 | **2 449** |
| shape | 95 818 | **0** |
| unbound target | 48 | 93 419 |
| **`JFP_ALIAS` exact / F1** | **308 / 0.94413** | **132 / 0.92655** — `JFP` to the digit |
| **`ALIAS_IN` exact / F1** | **472 / 0.99243** | **151 / 0.97888** — `ORACLE` to the digit |
| `RGL` / `INIT` / `SKIP` / `ORACLE` / `JFP` / `ALIAS_REF` | — | **every digit unchanged** |

> ### The breaker reproduces the published null **to the case**: one byte of displacement turns the model back into the incumbent it was built on, and touches nothing else. A breaker that broke everything would prove nothing; this one leaves five incumbents byte-identical.

### 4.2 B2 — accept every kind-4 tag (`0x04`/`0x0E`/`0x10`), the widening that RAISES a count

| | intact | **B2** |
|---|---:|---:|
| tag-0x10 records | 96 220 | **1 899 000** |
| **bound** | 95 820 | **137 379** — the widening makes the headline count **43 % BIGGER** |
| shape | 95 818 | **95 818** — so all 41 561 added bindings are **junk** |
| **`dom_with_body`** | **0** | **29 291** |
| the corpus test's assertion | passes | **FIRES** |
| unit tests | 16 pass | **4 FAIL** |

> ### This is w-small's shape reproduced on this lane's own instrument: a one-character relaxation that **improves a count** and is a live defect. Under §6 rule 4 it would suppress **29 291 symbols that have bodies**. The count that catches it is the one §3.2 says must exist, and it catches it in the same second.

**Restored:** 16/16 unit tests pass, `git status --porcelain crates/` empty.

---

## 5. THE MODEL — the Rust table, through a byte-identical `scan.py`

`work/w-alias/scan_rust.py` is **byte-identical** to `work/w-emitp/scan.py` —
`cmp` says so and the check is in the reproduction recipe. `scan.py` resolves
`import alias` from its own directory, so dropping `work/w-alias/alias.py`
(which decodes nothing; it serves the Rust dump, keyed by FNV-1a of the `.gl`
bytes) beside it substitutes the **decode** and changes **nothing** in the model.

| variant | `\|P\|` | precision | recall | **micro-F1** | **EXACT / 850** |
|---|---:|---:|---:|---:|---:|
| `ORACLE` *(a **CEILING**, never a model)* | 167 213 | 0.99997 | 0.95867 | 0.97888 | **151** |
| **`ALIAS_IN`** *(ORACLE + the alias — still a ceiling)* | 171 805 | 0.99997 | 0.98500 | **0.99243** | **472** |
| `ALIAS_BOTH` | 171 805 | 0.99997 | 0.98500 | 0.99243 | **472** |
| **`JFP_ALIAS`** *(a **MODEL** — conditions on no truth)* | 156 479 | 0.99825 | 0.89558 | **0.94413** | **308** |
| `JFP` — w-db's model | 150 833 | 0.99899 | 0.86391 | 0.92655 | 132 |
| `RGL_ALIAS_IN` | 898 489 | 0.19127 | 0.98531 | 0.32035 | 35 |
| **`ALIAS_REF`** | **129 604** | **1.00000** | **0.74307** | **0.85260** | **132** |
| `RGL` — the incumbent | 129 604 | 1.00000 | 0.74307 | 0.85260 | 132 |
| `INIT` / `SKIP` | 613 532 / 400 998 | 0.27289 / 0.36420 | 0.95991 / 0.83732 | 0.42496 / 0.50761 | 34 / 34 |
| **`ALIAS_SHIFT1`** — the null | 167 213 | 0.99997 | 0.95867 | 0.97888 | **151** |

Movement, name for name: `ORACLE → ALIAS_IN` **gained 321, lost 0**;
`JFP → JFP_ALIAS` **gained 176, lost 0**; `ALIAS_REF − RGL` **+0.00000** on
every column, `|P|` included; `ALIAS_SHIFT1` is `ORACLE` to the digit.

**Per-TU exact is the metric that matters and it is printed beside micro-F1 in
every table on this page** (board #250, `STATUS.md` trap 8).

### 5.1 What this is NOT — and w-reach has now MEASURED it, so this is not a caveat, it is a result

**TU match is 8 at both ends of this lane.** `PortC2` has no emit-set model, so
nothing in the port reads this table and no obj byte can move — §6's gate
numbers confirm it.

**`+176` is not a TU number, and this lane never computed the join that would
turn it into one.** Lane `w-reach` did, and the answer is the one that matters:

| model | per-TU exact / 850 | **TU REACH** (`∩ B∧C`) |
|---|---:|---:|
| `JFP` — the incumbent | 132 | **122** |
| `ORACLE` — the old ceiling | 151 | **134** |
| **`JFP_ALIAS` — what this lane implements** | **308** | **122** |
| `ALIAS_IN` — the new ceiling | 472 | **134** |

> ### **`JFP_ALIAS` is worth +0 TU reach today — gained 0, lost 0, as a set difference by name, in both arms.** A +176 move in per-TU exact bought nothing. That is board #250's lesson one level further out: per-TU exact is the right metric for the *emit predicate*, and it is still not the payoff metric.

**The mechanism is a single fact at 100.00 %, and it is structural rather than
unlucky.** Of the TUs this channel gains, **176 of 176** (model) and **321 of
321** (ceiling) carry **`.rdata$r`**, and **0** of them are already inside
`B∧C`. It could hardly be otherwise: an alias record is `??_E<X>` → `??_G<X>`,
which needs a **vftable**; a vftable at `/GR` mints **RTTI**; RTTI lands in
**`.rdata$r`** — the one section name factor **C** is blocked on. Against a base
rate of 0.779, 321 of 321 is ~10⁻³⁵ by chance.

> ### **AND THE COUPLING IS THE RESULT.** `.rdata$r` alone is worth **+1**. The alias alone is worth **+0**. **Together they are +91.** Behind a `.rdata$r` writer, `B∧C` goes **151 → 315** and this channel becomes worth **+90 (model) / +158 (ceiling)** — measured by w-reach at four values of C per TU, not interpolated.

**So the honest statement of this lane's worth is: it is half of the only pair
in the project that currently converts anything, and the other half
(`.rdata$r`) was DECLINED by lane `w-rdata` at seven facts.** A future reader
who finds "+0 reach" without the coupling will conclude this work was worthless.
It is not; it is a zero-free-parameter model, confirmed 15/15 through the sole
judge, whose value is *joint* with a rung that does not exist yet. Cite
`docs/rungs/_2026-08-05-w-reach.md` and **board `#302`** for every reach figure
above — **none of them is this lane's to claim.**

**And one correction from w-reach that this page must not paper over:** on the
850-TU join, `B∧C` is **145, not 151**. The 21-TU difference between the 850-TU
emit corpus and the 871 graded TUs is **exactly w-emitpred's held-out
quarantine, set-equal by name**, and **6** of those 21 are inside `B∧C`. Every
reach figure in the table above is quoted against **145**. That is also the
cleanest statement of why this lane could not have computed the join itself:
the denominators differ, and the difference is the quarantine.

**A zero-codegen implementation converts exactly 1 TU** (`src/system/decomp_pch.cpp`),
the same one for all four models. If exactly one TU ever moves on this channel,
that is the expected and correct outcome.

### 5.2 The red control — and it comes back CORRECT

w-reach's signal for over-widening: `ALIAS_IN` must **not** be exact on the two
`??__E` whole-TU byte-exact matches, because the channel bounds one acceptance
path and not the match set. Measured here **independently, from this lane's own
`scan.jsonl` against the 8 matching TUs `c2rs gap` printed**:

| | exact on the 8 byte-exact matching TUs |
|---|---|
| `RGL` / `JFP` / `ORACLE` | **6 of 8** |
| **`ALIAS_IN`** | **6 of 8** |
| **`JFP_ALIAS`** | **6 of 8** |

The two missed are exactly `src/system/synth/tomcrypt/TomCryptLicense.cpp` and
`src/system/zlib/ZlibLicense.cpp` — the two `??__E` whole-TU dynamic-initializer
TUs, by name. **This implementation does not claim them, so it has not
over-widened**, and the control could have gone red: had the alias channel
pulled either TU's set into agreement, the signal would have fired here.

---

## 6. Gate

Toolchain set explicitly (**board #299**); `C2RS_COMPILERS` and `C2RS_WIBO` were
exported before every command in this section.

| lane | result | baseline |
|---|---|---|
| `cargo test --workspace --release` | **799 passed, 0 FAILED, 27 targets** | **+17 tests, +1 target**, all this lane's — 16 constructed unit cases plus the corpus dump, which is its own test binary |
| `scripts/gate.sh --jobs 6` | **18/18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, 4 410 fixture-verdicts** | 18/18, 4 410 — unchanged |
| `scripts/expr_sweep.sh` (a gate row) | see §6.1 | 16 394 / 16 298 graded / 96 ungraded / 0 mismatch |
| `scripts/mode_cross.sh` (a gate row) | see §6.1 | 75 829 of 76 217 / 0 mismatch |
| 878-TU workload scan | **match 8, mismatch 0, codegen-gap 0, vocab-gap 863, capture-fail 7** | every digit unchanged |
| factors A / B / C / D / E | **28 / 338 / 169 / 8 / 2** of 871 graded | unchanged |
| `B∧C` · `A∧B∧C` · FRONTIER · frontier-if-A | **151 · 27 · 19 · 141** | unchanged |
| census / `census/gate disagreement` | **706555/2463393 (28.68%)** · **0** | unchanged |

The `c2rs gap` line quoted verbatim, as the evidence the run was real rather
than a `SKIP`:

```text
GAP REPORT (878 TUs in 3.4s)
  match             8    0.9%
  mismatch          0    0.0%
  codegen-gap       0    0.0%
  vocab-gap       863   98.3%
  port-error        0    0.0%
  capture-fail      7    0.8%
  capture cache: 871 hit, 7 miss, 0 uncacheable  |  validator: 0 re-captured and agreed …
```

### 6.1 An environment finding other lanes need tonight

**The first `gate.sh` run came back `GATE: FAIL` with `expr-sweep NO-RESULT` and
`mode-cross NO-RESULT`, and it was not this branch.** `/tmp` is a `tmpfs` whose
**inodes are 100 % exhausted** — `df -h /tmp` reports **19 G free** while
`df -i /tmp` reports **1 048 576 / 1 048 576 used**, so `sweep_gen.py` dies with
`OSError: [Errno 28] No space left on device` on a filesystem that has space.
The sweep and the cross are the two gate rows that write tens of thousands of
tiny files, so they are the two that die, and the gate correctly refuses to call
a run that graded nothing a pass.

**The fix used here was `TMPDIR=` plus `--work` pointing off `/tmp`**, not
cleaning `/tmp` — several lanes are live and their scratch is in there. Any lane
seeing `NO-RESULT` on those two rows tonight should check `df -i` before
believing it broke something.

### 6.2 What the coverage instruments can and cannot see

**`scripts/expr_sweep.sh` cannot see `crates/c2-il` at any setting** — it drives
`c2-core`. **This lane's work lands entirely in `crates/c2-il`, so the sweep's
verdict says nothing about it**, and that is stated rather than left as an
implied green. The instruments that *do* grade this lane are: the 16 constructed
unit tests, the 850-TU corpus dump with its assertion, the name-for-name
comparison against an independent implementation, and the counterfactual.

---

## 7. Scoring the pre-registration — 13 hits, 0 misses, 1 correction

| # | registered **point** | **measured** | |
|---|---|---|---|
| **R1** | tag-0x10 records **96 220** | **96 220** | **HIT**, exact |
| **R2** | bound **95 820** | **95 820** | **HIT**, exact |
| **R3** | shape **95 818** | **95 818** | **HIT**, exact |
| **R4** | 352 / 0 / 48 / 0 / 0 | **352 / 0 / 48 / 0 / 0** | **HIT**, exact |
| **R5** | tables equal name-for-name on **850/850** | **850/850**, 0 differing pairs | **HIT** — and this was the declared bias |
| **R6** | null bound **1 795 / 2 449** | **1 795 / 2 449** | **HIT**, exact |
| **R7** | null pairs **0** | **0** | **HIT** |
| **R8** | `dom(alias) ∩ U` **0** | **0** | **HIT** |
| **M1** | `JFP_ALIAS` exact **308** | **308** | **HIT** |
| **M2** | `JFP_ALIAS` F1 **0.94413** | **0.94413** | **HIT** |
| **M3** | `ALIAS_IN` **472 / 0.99243** | **472 / 0.99243** | **HIT** |
| **M4** | `JFP` **132 / 0.92655** (the KA control) | **132 / 0.92655** | **HIT** |
| **M5** | `ALIAS_REF` − `RGL` F1 **+0.00000** | **+0.00000**, `\|P\|` identical | **HIT** |
| **P1–P5** | the port moves nothing | **nothing moved** | **HIT** |
| **P6** | tests **781 + this lane's**, 0 failed | **798 / 0 / 27 targets** | **HIT** |

**One correction, and it is against this lane's own instrument rather than
against a prediction.** The FNV-1a join key was first written in Rust with the
prime one hex digit too long (`0x1000000001b3` for `0x100000001b3`). It is a
perfectly good hash and a **useless join key**, and every lookup missed — 850
`KeyError`s, which is the loudest possible failure and exactly the right one.
The signature is worth recording: **the low 32 bits agreed and the high 32 did
not**, which is what a wrong multiplier looks like and what a wrong *input*
never looks like. Fixed at `a68a0ba`.

**Declared bias, scored.** I said the place I expected to be wrong was **R5** —
that aggregate agreement is much weaker than per-TU agreement and that
compensating errors are an ordinary outcome. R5 came in at 850/850 with zero
differing pairs, so the bias was declared and not realised. I also said a lane
predicting its own change moves nothing cannot be surprised by success; that
stands, and it is why §6.2 names what the coverage instruments cannot see rather
than resting on `18/18 PASS`.

---

## 8. What this lane did NOT measure — named, so absence never reads as success

1. **`|{TU : model exact} ∩ B∧C|`** — the join that turns per-TU exact into TU
   reach. **Lane `w-reach` owns it**, and it owns `crates/c2-harness`.
   Deliberately not computed and deliberately not extrapolated.
2. **The `.in` `02`-node application site**, because it does not exist in
   `crates/`. §1.
3. **Anything at all in `crates/c2-core`** — `wt-w-rdata` is live there.
4. **The 352 `head_fail` records** (0.00366). Counted by both implementations,
   characterised by neither.
5. **The 48 unbound targets.** Same.
6. **The 510 outside-`U` emitted names on 162 TUs**, w-emitp's §3.2 next
   channel. Untouched.
7. **The 798 `$`-class residual names**, now the largest class.
8. **`0x10b28ca3`** — the instruction that turns `+0x20 & 0x2000` into the COFF
   Mark bit. Named in the disclosure row, **not decoded**.
9. **`0x10b8ac60`**, the second reader of the alias bit. Read, modelled nowhere.
10. **Order.** A right set in the wrong order is still a mismatch.
11. **The 21 quarantined TUs.** Untouched — §9.
12. **Whether any of this holds off this workload's flags.** Every statement is
    at `/O1 /EHsc /GR`.

---

## 9. The one-shot Part-1 gate — the question, PUT and not answered

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**. Ten consecutive lanes have preserved it. w-emitp declined to
spend it and recommended holding **until this implementation exists**. It exists
now, so the recommendation's condition is met and the question is live.

**What I would test, if told to spend it:** `JFP_ALIAS` — per-TU exact and
micro-F1 on the 21 held-out TUs, against the in-sample **308 / 850 = 0.36235**
and **0.94413**, with `JFP` (**132 / 0.92655**) as the paired control on the same
21. One table, two models, both metrics, no tuning afterwards.

**The argument FOR spending it now.** w-emitp's stated condition was that the
model have an implementation to be wrong about, and it now does — one that a
second implementation, 850 name-for-name tables, 16 constructed cases and a
two-breaker counterfactual all agree on. If the held-out 21 disagreed, this is
the moment that would be cheapest to find out.

**The argument AGAINST, which I think is still stronger.** The alias channel has
**zero free parameters**: the field position is transcribed from `0x10b9c02b` /
`0x10b9c030`, the gate is RT + BIND (which know nothing about the data), and
`JFP_ALIAS` inherits w-db's four binary choices and adds none. A held-out
population can only re-measure a decode that 95 820 records, a 40× count null, an
infinite shape null and 15/15 draws through real `c2.dll` already pin. **And the
model still converts zero TUs**, because nothing in `PortC2` consumes it. The
gate is worth more spent on the first model that *changes an obj*.

**My recommendation: still do NOT spend it — spend it when a consumer exists in
`PortC2` and the emit set can be wrong in bytes.** I am not spending it, and
the decision is the coordinator's.

---

## 10. Proposed board rows — **numbers NOT minted**

Same discipline as w-emitp and the lanes before it: **no number minted, no `#N`
pinned in code, `BOARD.md` / `ROADMAP.md` / `rungs/INDEX.md` untouched by hand**.
`Y-` is w-emitp's; this lane uses **`Z-`**.

| proposed | item | claim | where |
|---|---|---|---|
| **Z-0** *(= `#302`, minted by w-reach — **not** re-minted here)* | **THE CHANNEL'S TU VALUE IS +0 TODAY AND IS GATED ON `.rdata$r`** — `JFP_ALIAS` reach **122**, identical to `JFP`'s 122, gained 0 lost 0. **176 of 176** gained TUs carry `.rdata$r` and **0** are already in `B∧C`; `.rdata$r` alone is +1, the alias alone is +0, **together +91** | **not this lane's measurement** — it is `w-reach`'s, cited so this page cannot be read as a TU claim. Recorded as a proposed row because a reader finding "+0" without the coupling will conclude the work was worthless | §5.1, `_2026-08-05-w-reach.md` |
| **Z-a** | **w-emitp's tag-0x10 ALIAS decode is IMPLEMENTED in `crates/c2-il`** and a second, independently written implementation lands on the first **to the digit**: 96 220 / 95 820 / 95 818, 352 / 0 / 48 / 0 / 0, nulls 1 795 / 2 449 with **zero** pairs — and, the strong form, **850 of 850 per-TU tables equal name for name, 0 differing entries** | this is the first result in the project verified by *two implementations of one transcript* rather than by one implementation and a null. The Rust table then reproduces every model number through a **byte-identical** copy of w-emitp's `scan.py` | §2, §5 |
| **Z-b** | **A ONE-CHARACTER WIDENING RAISES THE BOUND COUNT 95 820 → 137 379 AND IS A WRONG EMIT.** Accepting every kind-4 tag adds 41 561 junk bindings (the `??_E`→`??_G` shape does not move) and puts **29 291 BODIED names into `dom(alias)`**, which §6 rule 4 would suppress | **w-small's shape, on this lane's own instrument, caught by a counter built before the code shipped.** Board #232 survived 255 commits on exactly this pattern | §3.2, §4.2 |
| **Z-c** | **§6 RULE 4 IS SHIPPED AS A COUNT, NOT AS A RULE.** "Never emit a name in `dom(alias)`" is licensed by `dom(alias) ∩ U = 0`, which is a *measurement*; the reader publishes `dom_with_body` and asserts it, so a corpus that breaks the premise fails loudly instead of silently suppressing a symbol that has a body | a measured fact adopted as a silent premise is how a reader becomes wrong later. The constructed stream that breaks it is a unit test | §3.2 |
| **Z-d** | **THE BIND GATE IS NOT SELF-VALIDATING, and there is a constructed stream that proves it** — a target field read one byte late binds cleanly to a planted decoy. The corpus agrees at scale: the shifted reads bind 1 795 / 2 449 times and pair **zero** | "the field bound" is not evidence of position. This is why the null ships as a public function, and why the `??_E`→`??_G` shape is reported as a *result* and never as a gate | §2, §3.1 |
| **Z-e** | **THE FIRST TWO `DISCLOSURE.md` ROWS EXIST, and `README.md`'s clean-room claim is now PER-FINDING** — W-ALIAS-1 (adoption: the record's bit layout, `0x10b9c01e`/`24`/`30`) and W-ALIAS-2 (route: `0x10b99621`/`35`) | the ledger's own checklist requires the README edit not to lag the code, and it did not. The grey-zone alternative was tried and is insufficient: a black-box search for the field position binds either side of it | `docs/whitebox/DISCLOSURE.md` |
| **Z-e2** | **THE OVER-WIDENING CONTROL COMES BACK CORRECT**: `ALIAS_IN` and `JFP_ALIAS` are exact on **6 of the 8** byte-exact matching TUs, missing exactly the two `??__E` whole-TU TUs by name | the channel bounds one acceptance path, not the match set. Measured independently here from this lane's own scan against the 8 TUs `c2rs gap` printed; it could have gone red | §5.2 |
| **Z-f** | **`/tmp` INODE EXHAUSTION MAKES `gate.sh` REPORT `NO-RESULT` ON THE SWEEP AND THE CROSS WHILE `df -h` SHOWS 19 G FREE.** `df -i` reads 1 048 576 / 1 048 576. Both rows write tens of thousands of tiny files; the gate correctly refuses to call the run a pass | an environment failure that looks exactly like a lane breaking two instruments. `TMPDIR=` + `--work` off `/tmp` is the fix; cleaning `/tmp` under live lanes is not | §6.1 |

---

## 11. Reproducing every number here

```sh
export C2RS_COMPILERS=<repo>/compilers C2RS_WIBO=<wibo binary>   # board #299
export C2RS_LANEROOT=<main-repo>
WT=$PWD                                    # the worktree

# 0. the cache index — w-joint's, paths made absolute (no toolchain)
awk -v m=$C2RS_LANEROOT -F'\t' '{n=split($2,a,"/"); print $1"\t"m"/work/capture-cache/"a[n]"\t"$3}' \
    $C2RS_LANEROOT/work/w-db/cacheidx.tsv > work/w-alias/cacheidx.tsv

# 1. the extended truth — w-joint's script, UNMODIFIED (no toolchain)
python3 $C2RS_LANEROOT/work/w-joint/truth_data.py work/w-alias/cacheidx.tsv \
        work/w-alias/dtruth $C2RS_LANEROOT/work/w-emit/truth 6

# 2. THE RUST DECODE over 850 TUs (no toolchain)
C2RS_ALIAS_CACHEIDX=$WT/work/w-alias/cacheidx.tsv \
C2RS_ALIAS_OUT=$WT/work/w-alias/rust_alias.jsonl C2RS_ALIAS_JOBS=6 \
    cargo test -p c2-il --release --test gl_alias_corpus -- --nocapture

# 3. RUST vs PYTHON, table by table, name by name (no toolchain)
python3 work/w-alias/cmp.py work/w-alias/cacheidx.tsv work/w-alias/rust_alias.jsonl 6

# 4. the model — scan_rust.py is BYTE-IDENTICAL to w-emitp's scan.py; check it
cmp work/w-alias/scan_rust.py $C2RS_LANEROOT/work/w-emitp/scan.py
C2RS_ALIAS_JSONL=$WT/work/w-alias/rust_alias.jsonl \
    python3 work/w-alias/scan_rust.py work/w-alias/cacheidx.tsv work/w-alias/dtruth \
            $C2RS_LANEROOT/work/w-emit/truth work/w-alias/scan.jsonl 6
python3 $C2RS_LANEROOT/work/w-emitp/score.py work/w-alias/scan.jsonl

# 5. THE COUNTERFACTUAL — breaks, measures, reverts, asserts the tree is clean
work/w-alias/counterfactual.sh

# 6. the gate.  /tmp inodes are exhausted on this box — see §6.1
TMPDIR=$WT/work/w-alias/gate scripts/gate.sh --jobs 6 --work $WT/work/w-alias/gate/run
```

All Python here is stdlib-only and read-only against the corpus. `work/` is
gitignored; the scripts are force-added as records, and no IL, obj or `_CL_*`
artifact is committed.

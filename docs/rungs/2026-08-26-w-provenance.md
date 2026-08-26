# w-provenance — the derived-vs-fitted census: `crates/` had 5 provenance marks and no denominator; it now has 189 marks against 1,051

    Tag:       w-provenance
    Slug:      w-provenance
    Date:      2026-08-26
    Kind:      construct (instrument — decision 15's third lane)
    Outcome:   instrument
    Fixtures:  none — instrument lane: a greppable provenance convention for
               load-bearing constants in `crates/`, seeded from
               `DISCLOSURE.md`, plus the per-module counter and its control
    Census:    +0
    Record:    `docs/whitebox/DISCLOSURE.md` § "The in-code provenance markers"
               (the convention and the definitions); this file (the numbers and
               what they cost); board `#3629`–`#3634`
    Fail axis:  this lane can fail with every byte identical, and did not: the
               census is required to print a DENOMINATOR per module and a
               non-empty untagged residue. A run that printed only numerators,
               or that could not be made red by removing a marker, is a
               failure whatever the gate says (`#3336`).

## 0. What this lane was for

Decision 15 restructures the working scoreboard onto per-subsystem metrics.
This lane builds the one that tracks goal (1) — *understanding MSVC's
internals* — most directly: **how much of each module of the port is READ from
the binary versus FITTED to observations of its output.**

That ratio was not merely low. It was **not a number at all**: `DISCLOSURE.md`
is a register of *adoptions*, so a fitted constant leaves no trace in it by
construction, and nothing counted anything.

## 1. Prereg, and how it scored

`work/w-provenance/PREREG.md`, committed at `d84ba4f2a` **before the first
count**. Ten predictions.

| # | prediction | outcome | verdict |
|---|---|---|---|
| P1 | 17 rows in the adopted-findings table | **17** | hit (a transcription check, declared as such) |
| P2 | 11 of 17 name a `crates/` path | **13** | **miss, pessimistic by 2** |
| P3 | **2** rows whose cited `crates/` site is dead | **0** | **miss, pessimistic** — and the prereg registered the bias in the *other* direction ("I expect to be wrong toward more dead"). Wrong twice: wrong number, wrong direction |
| P4 | 120–250 `const`/`static` in `codegen/**` | **307** | **miss, low by 23 %** |
| P5 | 60–120 in `coff/**` | **56** | **miss, low — by 4 below my own floor** |
| P6 | 40–100 in the `c2-il` shapes/admission vocabulary | **84** in `func/body/shapes`, **89** in `func/body` | hit for `shapes`; the second file group was not in the range's scope as written, which is the range's fault |
| P7 | 8–15 `[R]` **after the seed pass** | **12** | **hit**, and precisely: the seed pass alone produced 9 (`c2-reference`) + 1 (`plan`) + 2 (`c2-il`) |
| P8 | 15–30 % of the scoped population tagged at tip | **18.0 %** (189 / 1,051) | hit |
| P9 | the largest tag class at tip is `[F]` | `[R]` = **100**, `[F]` = **4** | **miss, and badly** — see §5 |
| P10 | ≥ 1 row whose symbol lives but whose file moved | **0** | miss |

**Three hits, six misses, one half.** The prereg's own framing was that P3's
error would be optimistic; it was pessimistic. Recorded rather than smoothed:
this lane's model of the tree was worse than its model of the instrument.

## 2. The deliverables

### 2.1 The convention — `DISCLOSURE.md` § "The in-code provenance markers"

    PROV[X] <citation> — <what and why>
    PROV-BLOCK[X] <citation> — <…>     (covers every population member in its block)

`X` ∈ `R` read · `O` obj-confirmed · `F` fitted · `S` specified by an external
standard · `N` not load-bearing, with a reason. `R` and `O` are
`ref/README.md` §2's own letters, unchanged, so nobody learns two systems.

**Why the token is prefixed** is a measurement, not taste: a bare `[X]` grep at
`6c753ead0` counts `params[src]` as a marker (`#3629`) and three prose `[R]`
mentions as tags they are not.

**The `[O]`/`[F]` discriminator**, which is the only hard call and is worked in
the doc on two constants one file apart: *does the constant have an off-sample
failure mode the observations that produced it could not see?* `mangle.rs`'s
`LITERAL_TEXT_BYTE_LIMIT = 32` is `[O]` — its probe cells sit at 31 and 32,
one on each side, so no other value survives. `data.rs`'s
`MAX_OBJECTS_PER_SECTION = 2` is `[F]` — its own doc measures the residue at
**47 of 48**.

### 2.2 The seed pass — all 17 rows, cross-referenced

**13 of 17 rows name a `crates/` path, and 13 of 13 are LIVE.** Every cited
file exists; every cited symbol is still present. The four that name none are
`W-STAGETAP-2`/`-4`/`-5` (`c2host/stagetap.c` only — all seven symbols there
are live too) and `W-MID-4`, which by its own text adopts nothing.

**Zero dead citations.** The prereg predicted two. Seven of the 13 resolve to a
live *named constant* (the rest to module docs, a function, or a comment),
which cleared the prereg's decline floor of five, so the seed pass ran rather
than being reported as `FAILED as a seed`.

Two sites are **correctly unmarked** and are recorded so the absence does not
read as an oversight: `W-MEMCPY-1`'s site is one comment that states no
constant, and `W-STAGETAP-6`'s `FuncWalk::{sym, sym_kind}` are struct fields.
Nothing was adopted at either.

### 2.3 The definition, and the counter

> **A load-bearing constant is a named `const` or `static` item in non-test
> code in `crates/`, whose value — if changed — could change a byte the judge
> grades, a refusal the port issues, or a verdict an instrument publishes as
> evidence.**

The third clause keeps `c2-reference/tests/middle_interfaces.rs` in scope, and
that file holds four of the ledger's own adopted table addresses.

The counted **proxy** is deliberately wider — every `const`/`static` outside a
test region — and a proxy member that is genuinely not load-bearing is expected
to say `PROV[N]` with a reason. That is what makes the residue elsewhere a
statement rather than an artifact of a proxy that counts everything. 18 items
carry `[N]` at this tip.

### 2.4 The control, watched failing

`scripts/provenance_census.py --self-test` plants a fixture with hand-
enumerated counts and demands **three reds**: removing an item marker moves
`[R]` by one and untagged by one; removing a `PROV-BLOCK` marker moves both by
two; an uncited marker is reported as a defect.

**It was watched failing before it was trusted.** With `MARK_RE` broken so it
cannot see a `PROV[X]` token, the suite goes red on **9** checks and **exits
1**; clean and restored both **exit 0**. Transcript:
`work/w-provenance/control_red.txt`. The exit codes were checked directly
rather than through a `tee` pipeline — the first attempt at this evidence read
`EXIT=0` on the broken run because `$?` after a pipe is `tee`'s, which is the
same shape as the `--check` flag that could not fail.

## 3. The census

`scripts/provenance_census.py`, at this lane's tip. Rows in **path order**,
never sorted by count.

| module | pop | `[R]` | `[O]` | `[F]` | `[S]` | `[N]` | untagged | rule marks |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `crates/c2-core/src` | 4 | 0 | 0 | 0 | 0 | 0 | **4** | 0 |
| `crates/c2-core/src/codegen` | 307 | 88 | 2 | 0 | 9 | 3 | **205** | 0 |
| `crates/c2-core/src/codegen/leaf` | 3 | 0 | 0 | 0 | 0 | 0 | **3** | 0 |
| `crates/c2-core/src/coff` | 56 | 0 | 43 | 4 | 9 | 0 | **0** | 0 |
| `crates/c2-core/src/plan` | 1 | 1 | 0 | 0 | 0 | 0 | **0** | 0 |
| `crates/c2-harness/src` | 21 | 0 | 0 | 0 | 0 | 0 | **21** | 0 |
| `crates/c2-harness/src/cli` | 31 | 0 | 0 | 0 | 0 | 0 | **31** | 0 |
| `crates/c2-harness/src/gap` | 33 | 0 | 0 | 0 | 0 | 0 | **33** | 0 |
| `crates/c2-harness/src/search` | 2 | 0 | 0 | 0 | 0 | 0 | **2** | 0 |
| `crates/c2-harness/tests` | 123 | 0 | 0 | 0 | 0 | 0 | **123** | 0 |
| `crates/c2-harness/tests/cellgrade` | 4 | 0 | 0 | 0 | 0 | 0 | **4** | 0 |
| `crates/c2-il/src` | 19 | 0 | 0 | 0 | 0 | 0 | **19** | 2 |
| `crates/c2-il/src/func` | 205 | 2 | 2 | 0 | 0 | 0 | **201** | 4 |
| `crates/c2-il/src/func/body` | 89 | 0 | 0 | 0 | 0 | 0 | **89** | 0 |
| `crates/c2-il/src/func/body/shapes` | 84 | 0 | 0 | 0 | 0 | 0 | **84** | 0 |
| `crates/c2-obj/src` | 43 | 0 | 0 | 0 | 0 | 0 | **43** | 0 |
| `crates/c2-reference/src` | 7 | 1 | 1 | 0 | 0 | 5 | **0** | 0 |
| `crates/c2-reference/tests` | 19 | 8 | 1 | 0 | 0 | 10 | **0** | 0 |
| **TOTAL** | **1,051** | **100** | **49** | **4** | **18** | **18** | **862** | **6** |

Tagged **189 / 1,051 = 18.0 %**. Two modules are at **100 %** coverage
(`coff`, `c2-reference`) and are the calibration; the rest is the seed.

**The residue is named, not silent, and part of it is a fence.** The **214**
untagged items under `crates/c2-harness/**` were **not reachable by this lane**:
that crate is peer `w-submetric`'s this wave. The remaining **648** are open
work with a denominator attached to each row.

**The rule marks have no denominator and the tool says so in its own output.**
Six exist, all seeded from DISCLOSURE rows onto the functions those rows name
(`record_head`, `gl_function_attrs`, `read_line_record`, `encode_line_record`,
`data_tu`, `in_alias_report`). The set of load-bearing *rules* is not
mechanically enumerable; publishing a ratio there would be worse than the
silence it replaced.

## 4. What the census found on its first run

### 4.1 `codegen/mop.rs` — 88 read constants on the emit path, and no ledger row

`mod op` holds c2's own opcode indices into the mnemonic table `0x10b1b260`;
`OPCODES` holds base words from `0x10c3a578` with forms from `0x10c39b18`,
transcribed from `ref/ENCODE_OPCODES.txt`; `EncodeParams::C2` holds
`P_ENCODE.md` §5's field placements, read arm by arm 79/79. **`base_word` is
the port's only source of a primary opcode**, so this is the emit path.

**`DISCLOSURE.md` has no row whose `Adopted into` is any
`crates/c2-core/src/codegen/` file.** `W-MID-1`/`W-MID-2` name
`c2-reference/tests/middle_interfaces.rs` and nothing else. `mop.rs`'s own
module doc asserts *"`docs/whitebox/DISCLOSURE.md` carries the provenance
rows"*; at `6c753ead0` it does not. A second, smaller instance:
`EX_CLASS_TABLE = 0x10b25e48`, unregistered.

**Stated, not repaired.** Filing a row is an adoption decision — step 4 of the
ledger's own checklist moves `README.md`'s clean-room wording in the same
commit — and this lane's charter is to annotate provenance, never to change it.
Board `#3632`; `DISCLOSURE.md` § "Adoptions this ledger does not carry".

### 4.2 The COFF writer contains not one read constant

`coff/**` at 100 % coverage: **0 `[R]`**, 43 `[O]`, 4 `[F]`, 9 `[S]`.

**This is a description, not a deficiency**, and the doc says so: a COFF writer
*should* be `[O]`/`[S]` — the container format is published and c2's choices
within it are visible in its output. It licenses no lane. What it buys is that
the next reader of `label.rs` sees **`[F]` beside a constant named `READ`**:
`SeedGapModel::READ`, whose three coefficients `W-SEEDGAP-1` records as black
box and whose own doc already said *"a fit to a read grid, one level short of
the mechanism."* Its predecessor, the literal `LABEL_SEED_GAP = 9`, is the
fitted constant that died at `#3388`.

### 4.3 The brief's calibration pole was four days stale

The dispatch named `codegen/encode.rs` as *"a black-box re-derivation …
fitted-by-construction"* and asked whether it is `[F]` or `[O]`. **Neither.**
Lane `w-s1` retired that re-derivation on 2026-08-22 — the 85 hand-copied
opcodes moved into `mop.rs` — and `encode.rs`'s own module doc records it.
What is left is 11 items: **9 `[S]`** PowerPC-ISA values and **2 `[O]`**
(`CR_COMPARE`, board `#188`: *which* CR field is c2's choice, not the ISA's;
and `BC_MAX_DISP`, where `CFG_SHAPE.md` §3.3.1 measured c2 using the full field
with no slack). **Zero `[F]`.**

That is also the census's first before/after datapoint, and it predates the
census: read-before-probe turned 85 fitted-by-construction facts into 88 read
ones in one lane. The instrument exists so the next such move is a number
rather than a paragraph in a module doc.

## 5. The prediction that missed worst, and what it means

**P9 said `[F]` would be the largest class. `[R]` is, 100 to 4.** Two reasons,
and only one of them is flattering:

1. **88 of the 100 are one table** — `mop.rs`'s block, a single read transcribed
   in a single lane. Excluding it, `[R]` = **12** and P9's shape is nearly
   right. A census with 18 % coverage does not have a representative sample and
   this row is the proof; the doc's rule that the tracked signal is the
   **CHANGE**, never the level, exists for exactly this.
2. **`[F]` is under-counted because the untagged 862 are untagged.** The fitted
   population is precisely the one with no ledger to seed from, so it is the
   part the seed pass could not reach. **Reading `[F]` = 4 as "the port is
   barely fitted" would be the census measuring itself** — `#3505`'s family,
   one instrument over.

## 6. Gate evidence

Comment-only edits in `crates/`: **every added line begins `///`, `//!` or
`//`, and no line was deleted.** Verified mechanically per commit —
`git diff -U0 -- crates/ | grep '^+' | grep -v '^+++' | grep -vE '^\+\s*(///|//!|//)'`
is empty at every commit of this lane.

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 8` | **GATE: see §6.1 verdict line** |
| `scripts/gate_identity_diff.sh` base→tip | **see §6.1** |
| `scripts/provenance_census.py --self-test` | PASS (exit 0); **watched FAIL (exit 1, 9 red checks)** with `MARK_RE` broken |
| `scripts/board_audit.sh` | 0 cited-but-absent · 0 unresolved anchors · 0 duplicates |
| `cargo build -p c2-core -p c2-reference` | clean |

**The base table** is `work/coordinator/gatebase/base_d5c73a728.txt`, and it is
a valid base for this lane because `crates/`, `scripts/` and `fixtures/` are
**byte-identical** between `d5c73a728` and this lane's base `6c753ead0` —
verified with `git diff --stat`, not assumed; the gate's graded-tree hash
covers exactly those three directories and the two intervening commits are
docs-only.

### 6.1 Verdict

Run detached at the lane tip (`crates/` clean; the only untracked file was
this rung, under `docs/`), `scripts/gate.sh --jobs 8`, 210 s, waited on by PID
in a bounded loop. **Read from the verdict line, never the exit code.**

```
GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them graded a corpus,
  the sweep graded 19460 of 19556 generated cases and the cross graded
  90424 of 90812 case-lane cells, with 0 mismatches anywhere
  (96 sweep cases carried ungraded — the reference rejects the source),
  and 18/18 lanes ran again through a DEBUG-profile c2rs for
  7038 more fixture-verdicts at 0 panics
```

**The `HATCH-RED REFUSED` qualifier is PRE-EXISTING and is not this lane's.**
`base_d5c73a728.txt` carries the identical headline, word for word, with the
same `HATCH-STALE` reason (board `#1389`). Quoted rather than dropped, because
the run does not establish what a full run establishes and saying so is the
qualifier's job.

**Identity diff — required-zero, over the enumerated 21 rows:**

```
$ scripts/gate_identity_diff.sh work/w-provenance/gate_base.txt work/w-provenance/gate_tip.out
count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS
```

Every row identical: eighteen mode lanes at `391/391`, `expr-sweep`
`19556/19556` (19,460 graded), `mode-cross` `90812/90812` (90,424 graded),
`debug-lane` `18/18` (2,479 matched). `graded: 7038 fixture-verdicts`,
`0 mismatch` everywhere, base and tip.

`gate_identity_diff.sh --self-test` also re-run at this tip and passes —
enumeration 21, control silent, `#3515`'s signature found at exactly 14 lines
over 7 rows, truncation refused.

## 7. Found and not taken

1. **File the missing `mop.rs` DISCLOSURE row** (and `EX_CLASS_TABLE`'s).
   Blocked deliberately: an adoption decision, and it moves `README.md`.
   ~½ day for a lane that owns both files. **Ranked first.**
2. **Wire the census under `cargo test`** — `crates/*/tests/provenance.rs`
   shelling out to the script and asserting the per-module rows. Needs a lane
   that owns a `crates/` test file; this one was comment-only fenced.
   Closes `#1406` for this instrument properly rather than by precedent.
3. **Tag `crates/c2-il/src/func/**` (290 items) and `c2-obj/src` (43).**
   `c2-obj` should be almost entirely `[S]` (PE/COFF) and is a cheap second
   zero-`[R]` calibration. `c2-il`'s decode vocabulary is where `[F]` should
   actually live and is the most informative untagged block in the tree.
4. **`crates/c2-harness/**` (214 items)** — fenced to `w-submetric` this wave.
   Its `gap/` keys are instrument definitions and are the natural join with
   decision 15's per-subsystem tuple.
5. **A two-tree diff mode** (`provenance_census.py --since <sha>`) so the
   tracked signal — the CHANGE per module — is printed rather than
   reconstructed by hand. The doctrine is written; the tooling for it is not.
6. **`codegen/**`'s 205 untagged** are dominated by per-fixture register
   numbers (`R_A`, `F_ACC`, `BO_FALSE` …), each justified by a captured word.
   They are the population the brief's "fitted-by-construction" phrase actually
   describes, now that `encode.rs` turns out not to be. Expect them to land
   `[O]`, and expect the exercise to be tedious rather than hard.

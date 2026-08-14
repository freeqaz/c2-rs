# w-dbgassert — the assertion was wrong and the writer was right, and the gate could not have told you either way

    Tag:       w-dbgassert
    Slug:      dbgassert
    Date:      2026-08-14
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization: is the FALSE `coff/writer.rs` assertion wrong, or is the writer wrong?
    Census:    unchanged, +0
    Record:    this file; board #3074 (CLOSED here, moved to `## Done`); rows #3083-#3087, PREREG `docs/rungs/_2026-08-14-w-dbgassert-prereg.md`

## The answer, in one sentence

**The assertion was wrong and the writer was right** — `layout_sections` gives an
uninitialized section `PointerToRawData = 0` by design, measured against real c2,
so `ptrs[i]` is not a file offset there at all; and the oracle agrees, because
the very input that trips the assertion (`fixtures/cpp/wwbss_two.cpp` at
`/Ox /Gy`) is **`match / byte-exact` against real `c2.dll` under wibo**.

## 1. Reproduced first, positively

Not read — run. Three witnesses, three different numbers, all at
`crates/c2-core/src/coff/writer.rs:658`:

    # 1. the unit tests (this is the reproduction command; from the repo root,
    #    DEBUG profile, which is cargo's default and which nothing here runs)
    cargo test -p c2-core --lib

    ---- coff::tests::xlrc_helper_symbols::a_shared_bss_is_spliced_into_the_shell_between_the_watermarks ----
    panicked at crates/c2-core/src/coff/writer.rs:658:9: left: 496  right: 0
    ---- coff::tests::xlrc_helper_symbols::two_shared_bss_objects_bump_forwards_and_emit_their_symbols_backwards ----
    panicked at crates/c2-core/src/coff/writer.rs:658:9: left: 536  right: 0
    test result: FAILED. 499 passed; 2 failed

    # 2. a REAL FIXTURE through a DEBUG c2rs — the witness #3074 does not have,
    #    and the one that matters, because it is the path the oracle grades
    cargo build -p c2-harness --bin c2rs
    ./target/debug/c2rs gap --list work/w-dbgassert/probe/list.txt \
        --flags-file work/w-dbgassert/probe/flags.txt --jobs 1 --no-cache
        # flags.txt = `/Ox /GS- /c /Gy`;  list.txt = wwbss_two.cpp, wwrap_gstore.cpp

    panicked at crates/c2-core/src/coff/writer.rs:658:9: left: 604  right: 0
    panicked at crates/c2-harness/src/gap/scan.rs:1452:5: a scoped thread panicked

496 and 536 are the unit cells; **604 is the fixture**. Confirmed pre-existing at
base `da3ed0d3` — every one of these was produced before this lane changed a byte
of `crates/`.

**One correction to #3074's own text, and it is made IN THAT ROW as well as
here**, because a row that closes with the wrong cardinality is what a later lane
cites: #3074 says *"two `debug_assert_eq!`s at writer.rs:658"*, and `gate.sh`'s
own framing inherits it. **It is ONE SITE WITH THREE WITNESSES** — 496 and 536
are the two unit cells, 604 is the fixture. The distinction is not pedantry: a
sibling census that counts firing *tests* instead of firing *sites* gets the
population wrong, and §4's `0 of 75` is a count of sites.

## 2. Which side is right — asked of the ORACLE, not of the code

`CLAUDE.md`: the real `c2` under wibo plus a byte-exact obj compare is the sole
judge. The two fixtures that *are* this class — the test's own doc comment says
so: *"`fixtures/cpp/wwbss_two.cpp` is this cell against real c2"* — graded at
`/Ox /Gy`, which is the lane that reaches `emit_comdat_obj`:

| fixture | class | reason |
|---|---|---|
| `fixtures/cpp/wwbss_two.cpp` | **`match`** | **`byte-exact`** |
| `fixtures/cpp/wwrap_gstore.cpp` | **`match`** | **`byte-exact`** |

So the port's obj for the input the assertion calls invalid is byte-for-byte what
real c2 emits. **The assertion is wrong about a writer that is right.** No alarm:
`mismatch 0` here is not the vacuous kind, because these two rows are `match` and
not `codegen-gap` — the judge actually looked.

The mechanism, now that the oracle has settled the direction:
`coff/container.rs::layout_sections` pushes `ptrs[i] = 0` for a section with
`uninit_size.is_some()`, and its own doc comment says why — *"an uninitialized
section advances the cursor by nothing and gets `PointerToRawData = 0`, however
large its `Section::size`"*, which `docs/OBJ_DYNINIT_SHAPE.md` §1 records as
**measured on real c2 obj bytes**, and as the *refutation* of the natural guess.
`ptrs[i]` for a `.bss` is a COFF null, not a file offset, and comparing the file
cursor to it is a category error.

**The dating is the finding.** The assertion entered on **2026-07-29**
(`cebfb88d`, W13b — pooled FP constants), when no section in this emitter could
be uninitialized, so it was *true* as written. It became **false on 2026-08-10**,
when `w-wordwrap2` (board #3032) spliced the shared `.bss` into Rule S1′ slot `B`
of `emit_comdat_obj`. It then sat false for **four days and every gate run in
that window was green**, because no instrument can execute it.

And the repair was already in the tree, twice. The two emitters that met `.bss`
*first* — `coff/dyninit.rs:284` and `coff/data.rs:541` — each carry exactly

```rust
if s.uninit_size.is_none() { debug_assert_eq!(b.0.len(), ptrs[i]); }
debug_assert_eq!(s.file_len(), s.raw.len());
```

`writer.rs` (both emitters) and `coff/ehscope.rs` are the copies that did not get
it. All three now do. The second line is the one with teeth and the one the bare
cursor form could never make: **an uninitialized section must carry no raw
bytes**, or the write and the layout cursor disagree by exactly `raw.len()` and
every later section's offset is wrong. That is the failure the original assertion
was reaching for; it was checking the wrong operand for it.

## 3. Release vs debug — asked positively

Not "`debug_assert` compiles out, therefore nothing changes". Two independent
checks, both positive:

| check | result |
|---|---|
| same source, both profiles, same obj, `cmp` | `c2rs prefilter --source fixtures/cpp/wwbss_two.cpp --flags-file … --emit-obj … --obj-name 'Z:\p.obj'` from `target/debug/c2rs` and from `target/release/c2rs`: **1,112 bytes each, `cmp`-clean, byte-identical** |
| the debug binary against the real oracle | `./target/debug/c2rs gap` on the two fixtures: **`match 2 / 100.0%`, `mismatch 0`** — the debug build's bytes are c2's bytes |
| the debug binary across the whole fixture corpus, all 18 lanes | per-lane `match` counts **equal the release lane's, digit for digit** (`/Ox /Gy`: graded 381, match 150, mismatch 0, both profiles) |

**The release binary takes the same path with the check absent.** The assertion
is purely diagnostic; there is no behavioural divergence to find. This was the
cheap answer and it is registered as such — P2 at 0.97 — but it was *checked*,
because the expensive answer would have been a much bigger finding and reading
the code cannot tell them apart.

## 4. The siblings — measured by coverage, not counted by grep — board #3084

`grep` gives the population; it cannot say which are ever *evaluated*. Measured
with `-C instrument-coverage` over a debug `cargo test --workspace` (116
`.profraw`, 37 test binaries, merged and exported with `llvm-cov`; the analysis
is `work/w-dbgassert/cov/`):

| population | count |
|---|---|
| `debug_assert` sites under `crates/` (source lines, comments excluded) | **75** |
| …reachable by **any standing instrument** | **0 of 75** |
| …executed at least once by a debug-profile `cargo test --workspace` | **66** |
| …**never executed by any test, in any profile** | **9** |
| …measured **FALSE** | **1** (`writer.rs:658`), with 3 witnesses |

The 9 that nothing executes:

    crates/c2-core/src/coff/ehscope.rs:433, 557, 559, 562, 570, 649, 654, 655
    crates/c2-core/src/coff/writer.rs:862

Eight of nine are one emitter. `coff/ehscope.rs` sits at **45 % region coverage**
under the whole test suite against `writer.rs`'s 94 % — its writing half is
reached by no test binary at all. That is *not* the same as dead code: the EH
emitter is exercised by the six `/EHsc` gate lanes, which run the release binary,
where every one of these is compiled out. **Two of the nine are ones this lane
just added** (`ehscope.rs:557`, `559`) — said out loud rather than quietly
counted, because a guard nobody has ever executed is a guard nobody has seen
fire, which is the whole complaint of this rung.

So the honest statement about the other 74: **one is refuted, 65 have been
evaluated and held, and 9 are unfalsified — not verified.** Nothing has ever
evaluated them.

### 0 of 75, and why that number is exact rather than approximate

`scripts/gate.sh`, `scripts/expr_sweep.sh`, `scripts/mode_cross.sh`, `c2rs gap`'s
878-TU scan, `scripts/status.sh` and the workspace test row **all** run
`--release`. `debug_assert` is `if cfg!(debug_assertions) { … }`; in a release
profile it is not a weak check, it is **absent**. The entire verification
apparatus of this repository is structurally incapable of reporting a false
assertion, and this is `docs/GAPS.md`'s absence-read-as-success — ~15 recorded
instances, twice inside the merge gate itself — reaching the emitter's own
assertions.

## 5. The second defect, which only the debug lane could see — board #3085

Running the fixture corpus through a **debug** `c2rs` — something nothing in this
repo had ever done — did not stop at `writer.rs`. It also found, at
`crates/c2-harness/src/gap/fnbytes.rs:2346`:

    thread '<unnamed>' panicked at crates/c2-harness/src/gap/fnbytes.rs:2346:48:
    attempt to subtract with overflow

`let last = bs.len() - 1;` in the **SPLICE-N** arm. That arm is reached whenever
`callees.len() != 1`, and that includes **zero**, so `bs` can be empty and the
`usize` underflows. This is the same defect family one layer over: not a
`debug_assert` but the dev profile's `overflow-checks`, equally invisible to
every `--release` instrument, and it is in an **instrument** rather than the
emitter — a silently wrapped value there produces a wrong *number*, and numbers
are what this repo publishes.

The repair is `saturating_sub(1)` and it is **exactly count-preserving**: for
non-empty `bs` the two spellings are identical; for empty `bs` the release build
wraps to `usize::MAX`, the loop over an empty `bs` runs zero times either way,
`cat` stays empty and the row lands as `spliceN|…|differs|n0` — which is what
`last = 0` also produces. No published counter moves. Verified empirically as
well as by argument: §6's base-vs-tip report identity.

**Left deliberately unrepaired, and it is a real question**: SPLICE-N is being
*asked* of bodies that name no callee at all, where the hypothesis is **vacuous
rather than refuted**, and those bodies land in the `differs` population that
boards #968/#974/#975 read. Sizing that is not this lane's; it gets its own row.

## 6. The required-zero delta, both directions

The changes are (a) inside `debug_assert` bodies and (b) one provably
count-preserving expression. Nothing emitted moves, and that is measured rather
than asserted:

| check | result |
|---|---|
| release `c2rs gap` over the 381 fixtures at `/Ox /Gy`, **base `da3ed0d3`** vs **tip** | **1,861 report lines identical**, digit for digit, after normalizing only the scan's `--jobs 8` progress order and the provenance header (git hash / binary sha, which differ by construction) |
| debug-emitted vs release-emitted obj, `wwbss_two.cpp` | **byte-identical**, 1,112 bytes |
| the port's obj vs real c2's, both profiles | **`match / byte-exact`** |

The base half was produced by `git checkout da3ed0d3 -- crates/`, a release
rebuild, the same scan, and a restore — not by quoting a number from a previous
session.

## Estimate vs outcome

PREREG frozen and committed as `docs/rungs/_2026-08-14-w-dbgassert-prereg.md`
(`221f3f95`) **before the first build**, and the tree at that commit contains no
change to `crates/`.

| # | registered | outcome | verdict |
|---|---|---|---|
| P1 | **assertion wrong, P = 0.88** (writer wrong 0.10, neither 0.02) | assertion wrong, settled by the oracle | **HIT** — and a *weak* one by the brief's own rule: I registered the likely answer at high confidence and it was the answer. The load-bearing part is §3 and §5, which the prior did not predict |
| P1′ | the discriminator: 496/536 are undisturbed cursor values and the following section's `ptrs` is the same number | held; the fixture added a third value, 604 | **HIT** |
| P2 | release and debug produce **identical bytes**, P = 0.97 | identical, `cmp`-clean, 1,112 bytes, and `match` at both profiles | **HIT** |
| P3a | **70** `debug_assert` sites under `crates/`, 80 % interval 40–130 | **75** | **HIT** (in interval, +7 %) |
| P3b | **0 of them** reachable by any standing instrument, P = 0.95 | **0 of 75** | **HIT**, exact |
| P3c | **60 %** executed by a debug `cargo test --workspace`, interval 30–85 % | **88 %** (66 of 75) | **MISS** — I was wrong, and on the high side: the suite reaches far more of them than I gave it credit for. The 9 it misses are concentrated in one emitter, which the fraction alone would have hidden |
| P3d | **1** additional FALSE sibling, interval 0–4 | **0** additional false — but **9 unfalsified**, which is not the same as 0 | **HIT** on the number, and the number was the wrong question |
| P4a | fixtures `match` +0, `mismatch` 0, census +0, workload `match` +0 | all +0 / 0 | **HIT** |
| P4b | **required-zero byte delta** | 1,861-line base/tip report identity; obj `cmp`-clean | **HIT** |
| P4c | `cargo test --workspace --release` **1,548 → 1,550** (+2 tests), 42 targets | **1,548 / 42, +0 tests** | **MISS.** I registered a test I then did not write, and I was right not to: **no test I can add to the release suite could catch this defect class**, because the fault is that the suite runs in the wrong profile. A release test here would have been theatre |
| P4d | debug `cargo test --workspace` **RED at base with exactly 2 failures**, **GREEN at tip** | base: 42 targets, 1,546 passed, **2 failed**; tip: 42 targets, **1,548 passed, 0 failed** | **HIT**, both halves |
| P5 | `Outcome: instrument`, P = 0.80 | `instrument` | **HIT** |
| P6 | one debug lane, **under 3 min** added to a gate run, interval 1–15 min | **6 s cold, 0.65 s warm** for the unit row; **125 s** for the full 18-lane fixture sweep | **MISS, low** — the registered interval's floor was 1 min and the unit lane is a hundred times cheaper than that. I priced it as if the workspace had dependencies to build; it has **zero external crates**, so a from-scratch debug build of all five is 6 s |

Two misses (P3c, P6) and one registered-then-abandoned row (P4c). Saying it in
the words the brief asks for: **I was wrong about how much of the assertion
population the test suite reaches, and wrong by two orders of magnitude about
what a debug lane costs.**

### Both misses ran in the direction that argued AGAINST building the instrument

That is the useful direction to be wrong in, and it is worth stating as a
property of the pair rather than leaving it as two unrelated rows:

* **P3c** said the debug suite reaches only 60 % of the assertion population. If
  that had been true, a debug lane would have been a *partial* instrument
  covering three fifths of the sites. It reaches **88 %**, so the lane is a
  far better instrument than its own designer priced it as.
* **P6** said a debug lane costs 1–15 minutes. If that had been true, it would
  have been a real line item against a gate whose cheap lanes cost seconds. It
  costs **6 s cold / 0.65 s warm**.

**P6's miss is therefore the strongest single argument FOR taking the lane**, and
it should be read that way rather than as an embarrassment: I priced the debug
build as if the workspace had a dependency graph to compile. **It has none —
`CLAUDE.md`'s std-only, zero-external-crates rule means a from-scratch debug
build of all five crates is 6 seconds.** The constraint that exists to keep the
port honest turns out to make its own missing instrument free. That is an
unbudgeted dividend of a rule adopted for a different reason, and it is the
cheapest argument on the table: **the reason nobody built this was a cost that
does not exist.**

Had either estimate been right, the honest recommendation at the end of this
lane would have been weaker. Both were wrong the same way, and neither error was
in a direction that flattered the lane's own conclusion.

## The blindness — proposed, priced, and NOT imposed

The repair above fixes one assertion. It does not fix the reason nobody knew.
`scripts/debug_lane.sh` ships here as the instrument; **it is deliberately not
wired into `scripts/gate.sh`**, because the gate is shared, a peer lane is live,
and the brief's instruction was to propose and price rather than impose.

What it is: the 381 fixtures through a **debug** `c2rs` at every lane in
`scripts/lanes.txt`, printing one `DEBUG-LANE-RESULT` line per lane with the same
vacuity guard `mode_lane.sh` has (a lane that graded fewer than `total` is a
FAIL, not a pass). It is not a byte judge and does not replace one — `mismatch`
is still graded by `gate.sh` against real c2. What it adds is the two faults a
release build cannot express: a **false `debug_assert`**, and an **arithmetic
overflow**. It found one of each on its first run.

First full sweep, at tip:

    DEBUG-LANE-TOTAL lanes=18 ran=18 failed=0
    18 lanes x 381 fixtures, 0 panics, 0 mismatch, per-lane `match` equal to the
    release lane's digit for digit (O1 177 · O1-EHsc 178 · O1-Oi 179 ·
    O1-Oi-EHsc 180 · Ox 150 · Ox-EHsc 150 · Ox-Gy 150 · Ox-Gy-EHsc 150 ·
    O2 156 · O2-EHsc 156 · Od 18 · Od-EHsc 18 · O1-Oi-GR 179 ·
    O1-Oi-EHsc-GR 180 · Ox-GR 150 · Ox-EHsc-GR 150 · Od-GR 18 · Od-EHsc-GR 18)

**The price, measured on this box, warm capture cache:**

| candidate | cost | would it have caught #3074? | would it have caught §5? |
|---|---|---|---|
| **(b) debug unit row** — `cargo test --workspace --lib`, no toolchain | **6 s cold, 0.65 s warm**, 1,386 tests, 5 targets | **yes** — the two `xlrc_helper_symbols` tests fire | **no** — no unit test reaches `gap/fnbytes.rs`'s SPLICE-N arm |
| **(c) debug fixture sweep** — `scripts/debug_lane.sh`, 18 lanes | **125 s** warm; needs the toolchain, so it SKIPs cleanly without one | **yes**, with a third witness (604) the unit tests do not produce | **yes** — this is how §5 was found |
| (b) + (c) | **≈ 126 s** | both | both |

Against the gate's own measured cost (`lanes.txt`: 6 s cold for 2,364
fixture-verdicts at `--jobs 4`; the current gate run is minutes), **(b) is free
and (c) is ~2 minutes.**

**Disposition, 2026-08-14: the coordinator ACCEPTED the recommendation and
DECLINED the wiring.** `scripts/debug_lane.sh` ships exactly as written, standing
outside `scripts/gate.sh`. The stated reason is the right one and is recorded
here rather than paraphrased: **making a debug panic a merge blocker is the
user's decision, not the coordinator's and not this lane's.** The
counter-argument below is what that decision turns on, so it stays in the record
at full weight rather than being trimmed now that the answer went the other way.

The recommendation, for whoever owns the gate: **take (b) unconditionally** — it
is 0.65 s, it needs no toolchain, it runs in the portable lane where everything
else already runs, and it is the row that catches a false emitter assertion.
**Take (c) as a gate lane too**, on the evidence that it found a defect (b) could
not, and that its per-lane counts are a free cross-profile identity check on the
release lanes. The counter-argument, stated rather than omitted: (c) doubles the
number of `c2rs` builds a gate run needs (debug and release), and a debug binary
that panics turns a *diagnostic* into a *gate failure* — which is the point, but
it means a false assertion now blocks a merge instead of being invisible. That is
the trade, and it is the correct direction: this repo's rule is that a wrong
number is worse than a stopped run.

## Found and not taken

1. **`coff/ehscope.rs` is 45 % covered and 8 of its 9 assertions have never been
   evaluated.** The largest concentration of unfalsified invariants in the tree,
   in the emitter for the axis (`/EHsc`) that `lanes.txt`'s own header records as
   having once made *"the entire EH surface vacuous."* A lane that drives the EH
   emitter from a unit test would falsify or confirm eight invariants at once.
   Not taken here: it is a test-writing lane, not a characterization one, and
   `coff/` single-occupancy should be released.
2. **SPLICE-N is asked of zero-callee bodies** (§5). Vacuous rows in a
   `differs` population that three board rows read. Sizing it needs the 878-TU
   scan's emit map, not the fixture corpus.
3. **`overflow-checks` is a whole second class this lane only sampled.** The
   fixture corpus found one underflow. The 878-TU workload is 2.3× the input
   diversity and has never been run through a debug binary at all. Cost is the
   scan's own cost times the debug slowdown; unpriced here.
4. **The `debug_assert` vs `Err` question, already answered once and not
   propagated.** `rungs/2026-08-04-w-label.md` records five invariants written as
   *ordinary `Err`* rather than `debug_assert`, deliberately. That is the
   structural fix — an invariant that returns `Err` is graded by every instrument
   in the repo, in every profile. 75 sites is too many to convert on a whim, but
   the 9 unexecuted ones are exactly the population where a `debug_assert` buys
   nothing at all.

## A note on `Kind:`

`characterization`, and the fit is imperfect rather than silently forced. The
lane's *question* was a characterization question — which of two things is
right — and it was settled by grading real c2's obj bytes, which is kind 3's
criterion. It did land code, which kind 3 says it does not: three corrected
assertions, one `saturating_sub`, one script. What it did **not** do is kind 2's
defining act — re-express an already-byte-exact class through new machinery — so
`construct` would have been the worse label. The deliverable is the finding; the
code is its consequence.

## Gate evidence

**Re-graded on the REBASED tree.** The lane was rebased onto master `8a210c27`
(`w-ir-e` merged, rows #3078–#3082) after its first full gate. The rebase
conflicted **only** on `docs/BOARD.md` and `docs/rungs/INDEX.md` — **no code
conflict, and none in `gap/fnbytes.rs`** — and it changed no byte of this lane's
own `crates/` or `scripts/` content. But the *tree* is new: it now carries
`w-ir-e`'s `codegen/` work, which no run of mine had seen. The merged
configuration is one no prior run covered (`w-ir-cond`'s rule, `cf40b43b`), so
every row below is the **post-rebase** run. The pre-rebase run is recorded
underneath it, unchanged, because a discarded green run is still evidence about
the tree it graded.

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 4 --require-graded` (**rebased**) | **GATE: PASS (HATCH-RED REFUSED)**, exit 0. 18 lanes in the registry — **18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**; **6,858 fixture-verdicts**; sweep **19,556 of 19,556 reached, 19,460 GRADED, 0 mismatch**; cross **90,424 of 90,812 graded, 0 mismatch**. `graded tree` **`b865e54d6939` (728 files) — identical at both ends** |
| `cargo test --workspace --release --no-fail-fast` (**rebased**) | **1,567 passed, 0 failed, 42 targets**, exit 0 — identical to master `8a210c27`'s own **1,567 / 42** (`rungs/2026-08-14-ire.md` §306). **This lane's delta is +0 tests**, and §"Estimate vs outcome" P4c says why that is right rather than a shortfall |
| `crates/c2-harness/tests/rung_registry.rs` | **2 passed, 0 failed** (`rung_docs_claim_their_tag_slug_and_fixtures_exactly_once`, `rung_index_is_generated_and_current`) |
| `cargo test --workspace` (**DEBUG — the point of this lane**), base `8a210c27` | **RED**, and the defect is still live at the *new* base: the same two tests, the same `writer.rs:658`, the same `left: 496 / right: 0` and `left: 536 / right: 0`. `w-ir-e`'s merge did not touch it |
| `cargo test --workspace` (**DEBUG**), tip | **GREEN: 42 targets, 1,567 passed, 0 failed, 0 panics** |
| `scripts/debug_lane.sh` (**DEBUG**, 18 lanes × 381 fixtures, rebased) | **18/18 PASS, 0 panics, 0 mismatch, 70 s**; per-lane `match` **identical to the release gate's, all 18 lanes**, checked by diff rather than by eye |
| `scripts/board_audit.sh` (after minting #3083–#3087) | **all-zero**: 0 cited-but-not-on-board, 0 unresolved section anchors, 0 raw line-number anchors, 0 rows behind the prose, 0 duplicate row numbers (**1,696** board rows, 259 ROADMAP citations) |
| release `c2rs gap`, 381 fixtures `/Ox /Gy`, base vs tip | **1,861 lines identical** (taken pre-rebase, against base `da3ed0d3`; the diff under test is byte-identical after the rebase) |
| fixtures, oracle | `wwbss_two.cpp` **`match / byte-exact`**, `wwrap_gstore.cpp` **`match / byte-exact`** |

Every count in the rebased gate is **identical to master's own tip run of
record** (`rungs/2026-08-14-ire.md`): 18 PASS, 6,858 fixture-verdicts, sweep
19,556 / 19,460 / 0, cross 90,424 / 90,812 / 0. The only difference is the tree
hash and its file count — **728 against master's 727** — which is
`scripts/debug_lane.sh` and nothing else. `HATCH-RED REFUSED` is the standing
condition on this tree (board #1389), recorded verbatim in master's own base
*and* tip runs; it is not this lane's.

**Pre-rebase run, kept rather than dropped.** Base `da3ed0d3`, `graded tree`
**`b0ff574cdb34` (727 files) — identical at both ends**, GATE PASS (HATCH-RED
REFUSED), 18 PASS / 0 FAIL / 0 SKIP / 0 NO-RESULT, 6,858 fixture-verdicts, sweep
19,556 / 19,460 / 0, cross 90,424 / 90,812 / 0; release tests **1,548 / 42**
(master's count before `w-ir-e`), debug tests **RED 1,546 / 2 FAILED** at base and
**GREEN 1,548 / 0** at tip; `debug_lane.sh` **18/18 PASS, 0 panics, 125 s**.
Two independent trees, the same verdict.

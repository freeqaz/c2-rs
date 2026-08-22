# w-shiproadmap — the ordered rewrite: a proposal's own escape clause fired, and the successor is that clause written out

    Tag:       w-shiproadmap
    Slug:      w-shiproadmap
    Date:      2026-08-22
    Kind:      construct
    Outcome:   built
    Fixtures:  none — construct rung: the project's steering surface. It builds
               no crates/ machinery. What it builds is the property that a
               reader arriving at the roadmap reads the program the owner
               ranked, and finds the vendor-DLL service where the ranking puts
               it rather than in the headline.
    Census:    +0 — no crates/ edit of any kind, no emit rule, no refusal
               predicate, no fixture, no byte. Docs only.
    Record:    this file; authority is `docs/DECISIONS_2026-08-22.md` decision 2
               (owner) and `docs/GOAL_DECISION_2026-08-21.md` § "AMENDED"
    Board:     **#3380**–**#3383**, from the block the coordinator reserved for
               this lane. All four used; none released. No existing row edited.

> **`coff::Function` field this lane's work would eventually write: NONE.**
> Arch review finding 3's prophylactic. This lane is docs. It ports no pass,
> writes no field, and says so rather than leaving it open.

---

## 0. The order, and what it did and did not authorize

The owner, 2026-08-22, quoted verbatim in `docs/DECISIONS_2026-08-22.md`:

> *"#2 rewrite the doc and get it aligned with our earlier goal statement"*

**What it authorizes:** re-ranking `SHIPPING_ROADMAP_2026-08-19.md`'s product
recommendation against the goal statement, which is a product decision two
lanes correctly refused to make on their own.

**What it does not authorize, and §4 is about the one place this bit:** deciding
anything the goal statement itself leaves open. An alignment order is not a
licence to mint a milestone definition. Where the successor had to face a
judgement the owner has not made, it presents alternatives and picks none
(board **#3382**).

**And the goal ranking is quoted, never paraphrased.** `SHIPPING_ROADMAP_2026-08-22.md`
§1.1 reproduces both owner quotations in full — the 2026-08-21 statement and
its § "AMENDED" re-ranking — because paraphrase is how a ranking becomes
stronger or weaker than it was stated. The tree already has the receipt for
that failure mode: *"ranked equally"* was live for a matter of hours and was
still the operative wording in **nine** live surfaces a day later, including the
block coordinators copy into every brief (**#3370**).

## 1. The finding, and it is about the 08-19 page rather than about this lane

**A proposal that wrote its own falsification condition was falsified by it
three days later, and every stage of the mechanism worked.** Board **#3380**.

`SHIPPING_ROADMAP_2026-08-19.md` §1 closed with:

> *"If the product requirement forbids using the vendor DLL, this is not a
> near-term shipping project. It is a compiler-backend reconstruction program
> and should be staffed, budgeted and reviewed as one."*

Four steps, none of which required an argument:

1. **2026-08-21** — the owner states goal (2): parity, a 100 % open-source
   implementation. The conditional is now true.
2. **Same day** — `w-goaldocs` banners §1 and says so, and **does not act**:
   *"it does not decline track 1, which is a product decision and this page's
   own request for review."*
3. **2026-08-22** — `w-readdocs` sharpens it with the ranking and adds a
   *second*, independent disqualification, and again does not act.
4. **2026-08-22** — the owner acts.

**The cheapest supersession this repo has processed**, and the reason is that
nobody had to establish that the recommendation was wrong. The page had already
written down the condition under which it would be, and the condition became
checkable the moment the owner answered a question the page did not ask.

The lane-discipline half is worth as much: two lanes detected that a live
document's premise had flipped, bannered it, and stopped at the edge of their
authority — one day apart, independently. **The generalization for future
proposals: write the escape clause into the proposal.** A recommendation with a
stated falsification condition is a recommendation that can be retired by
observation instead of by argument.

## 2. What was carried forward, and why each item survived

The order named three carry-forwards and the lane honoured all three plus one
the order named negatively (*do not silently drop*).

| carried | from | why it survives the goal decision | where it is now |
|---|---|---|---|
| **The three meanings of 100 %** | 08-19 §4 | It is a *disambiguation*, not a recommendation. Goal (2) is stated in its terms: `870/870` on the pinned dc3 workload is **native dc3 parity**, not proof of all `c2.dll` behavior. `ARCHITECTURE_PROPOSAL` §7 item 2 independently asked for it as standing language | 08-22 §2, with §2.4 repeating the adopt-as-language recommendation |
| **Operating-model items 3–6** | 08-19 §6 | All four are about honest measurement of *reproduction*, which is orthogonal to which product ships. Two get **stronger** under the goal decision: item 3 (per-stage metrics) because a per-stage account is now a deliverable, item 5 (DEV/HELDOUT) because §3.3's model-or-fit question is exactly what a re-fit on the acceptance population would hide | 08-22 §6.1, each with the live caveat it has acquired since |
| **Every measurement** | 08-19 §3, §5, §8 | The verifier-throughput thesis is retired **symmetrically**: a measurement may no longer justify a lane, and may no longer forbid one. So the numbers stay and lose only their standing as arguments | 08-22 §3 (position), §7.2 (the service), §8 (cost) |
| **§6 item 1 — retire the board** | 08-19 §6 | Named in the brief as *"do not silently drop"*. It is a governance change with real costs on both sides and it is nobody's decision yet | 08-22 §6.4 as **OPEN**, two-sided, plus §9 item 4 |

**What was not carried, and the reason is one sentence each.** The two-track
strategy (the escape clause fired). Track 1 as headline (§7's two
disqualifications). The *"indicative duration"* lines on M0/M1/M2 — not because
they were wrong, but because `CEILING` §5 says every forward figure in this repo
is a **lower bound** with optimism dominating ~5:1, and a duration line on a
reconstruction program is the one thing this project has the most evidence it
cannot supply. M2's framing as a **gate** — it has since been built, graded and
re-priced, and under goal (1) its snapshots are the deliverable rather than the
thing standing in front of one.

## 3. The vendor service: two disqualifications, and a supporting case that had already inverted

Board **#3383**. The successor allows the service to appear **only** as an
explicitly subordinate option, and it separates three things the 08-19 page's
§5 had braided together.

**The two disqualifications, both structural:**

1. **It moves the parity scoreboard by zero.** Goal (2) is a 100 %
   *open-source* implementation scored `match` → 870/878. A service answering
   every request from the real pinned `c2.dll` adds nothing to that number and
   **depends on the binary parity is defined as replacing**. In §2's language it
   is 2.1, and 2.1 is not 2.2.
2. **It is the opaque binary**, so it emits none of the signals goal (2) is
   valued for supplying to goal (1) — *"actual code we can tweak to instrument +
   help produce signals about the compiler's state"*. It moves the parity
   scoreboard by zero **and** the instrument story by zero.

**Throughput disqualifies it in neither direction**, and this is the half most
likely to be misapplied: the retirement is symmetric, so 2.76× / 1.44× / 1.01×,
the `1/(1−p)` ≈ 1.03× hybrid curve and #3262's *"under 2 %"* are all carried as
measurements that neither justify nor forbid.

**And the finding that is not a re-ranking at all:** two measurements from
2026-08-21 inverted the service's *supporting* case and neither reached the page
a reader opens for the product argument.

- The claimed **shared-plumbing dependency runs the other way**.
  `w-stageoracle` declined residency in writing — *"one process per compile is
  precisely what makes snapshot determinism testable — no cross-compile state,
  no allocator reuse, no counter carry-over"* — so building M1's residency "for
  free" for the oracle would have been building the thing most likely to break
  the oracle's load-bearing property. `ARCHITECTURE_PROPOSAL` §7.1 struck the
  sharing claim in place; the 08-19 §5 that motivated it was never revisited.
- **Spawn was never the lever.** #3262 counted rather than inferred: `c2rs`
  startup is under **1 ms**, and the expensive spawn is `cl.exe` under wibo, one
  in six. A fork server buys **under 2 %** there.

The exactness evidence stands and is carried: **10,580 fork-vs-spawn
comparisons, 0 differing objects.** What the successor says about the service is
what `ARCHITECTURE_PROPOSAL` §7 already said and what this lane did not need to
change: ship it as a product decision on its own merits, do not let it reorder
the reconstruction program, do not let the reconstruction program block it —
and never report it as progress toward goal (2).

## 4. The decision the rewrite was NOT allowed to make

Board **#3382**, and it is the place where an alignment order could most easily
have become a product decision by accident.

**Retiring the headline product left the program with no definition of
"shippable" at all.** The 08-19 page never had to answer the question: its
headline *was* the shippable thing. With the service subordinate, "what is a
shippable milestone for a reconstruction program" is live, unanswered, and
load-bearing — it sets what a lane may call done.

Three candidates are posed at 08-22 §5.1, each with its cost:

- **(A) nothing ships until 870/870.** Matches goal (2)'s binary scoreboard
  exactly; leaves the program with no legible external output for a duration
  `CEILING` §5 forbids anyone to state.
- **(B) the characterization record is the artifact.** Most aligned with the
  **primary** goal, and it is already happening under M-CHAR; it is a
  documentation release and must not be allowed to read as a claim about the
  port.
- **(C) the port ships as an instrument** for the two named consumers at
  whatever coverage it has, with its refusal boundary as the advertised
  interface. Needs the decision surface to exist first (S1 on), and needs the
  permuter population measurement (**#3369**) — shipping earlier advertises a
  search space `DIFF_STRUCTURE.md` says may point at nothing.

**None is picked, and that is the deliverable of this section.** The owner
ranked two ends; the owner did not say what a milestone is. Writing one of these
in as decided would have been the same error the two bannering lanes declined to
commit (**#3380**), committed one step later in the process.

The same restraint governs the whole §9: the two rows the owner has decided are
recorded as decided; the five open rows are posed as proposals with
alternatives, in the order they bind.

## 5. Estimate vs outcome

Registered before the work, in the brief and this lane's own reading of it.

| # | registered | realized | verdict |
|---|---|---|---|
| E1 | the rewrite is a docs deliverable, `Census: +0`, zero `crates/` bytes | **exactly that** — `git diff --stat` over the lane touches `docs/` only, 5 files + 1 new + 1 new rung | **HIT** |
| E2 | ~3–5 live sites cite the 08-19 roadmap and need amending | **3** live sites (`docs/README.md`, `ARCHITECTURE_PROPOSAL` Basis, `ARCHITECTURE_PROPOSAL` §7) + 1 banner on the dated record itself; 4 dated records deliberately left | **HIT**, low end |
| E3 | the 08-19 page's measurements would need correcting against today's tree | **MISS, in the reassuring direction.** On the seven fields its §3.2 table shares with `STATUS.md`'s generated block — 878 TUs / 870 graded / 8 capture-fail / `match` 26 / `mismatch` 0 / `vocab-gap` 844 / `codegen-gap` 0 — every digit agrees, and the page says so itself. Nothing needed correcting; the numbers were right and only their *standing as arguments* changed | **MISS — pessimistic** |
| E4 | *not registered* — nothing predicted that the review's own "immediate release-hygiene defect" would still be live | **93 diff lines, `C` vs `en_US`**, three days after it was filed (**#3381**) | **unregistered finding** |

The E3 miss is worth naming because it is the *rare* direction (`CEILING` §5:
optimistic misses dominate ~5:1, and both recorded pessimistic misses were
"I thought this was structurally cheap" inverted). Here the lane budgeted for
re-measuring a superseded page and found a page whose measurements had not
moved. **The 08-19 review was wrong about the recommendation and right about
every number in it**, which is the distinction the successor's §3 is built to
preserve.

## 6. The axis on which this rung can fail with every byte identical

Required by the construct-rung **COST CLAUSE** (board #3336, as amended
2026-08-21: name an axis on which the rung *can* fail, and say what you measured
on it). This lane changes no byte of `crates/`, so a byte criterion is a
tautology here and would abstain rather than pass. Three axes, all checkable:

1. **CITATION RESOLUTION.** Every path and link the new page cites must resolve.
   **Measured**: 8 markdown link targets, **0 missing**; 30 backticked file
   paths, **all 30 resolve** (four resolve outside `docs/` — `scripts/board_audit.sh`,
   `docs/whitebox/DISCLOSURE.md` — or are correctly named as artifacts a funded
   read has not produced yet — `ref/P_ENCODE.md`, `ref/P_ILRECORD.md`). **This
   axis fired once and was repaired**: the page originally cited
   `PROGRESS_METRIC.md §5.2`, and there is no §5.2 — see §7.
2. **CARRY-FORWARD COMPLETENESS.** The order named four things to carry
   (three positively, one negatively). **Measured**: all four present and
   attributed — §2 above is the row-by-row check, and the negatively-named one
   (§6 item 1) is at 08-22 §6.4 as OPEN with both sides priced, not dropped.
3. **NON-EDIT OF THE DATED RECORD.** The 08-19 page must gain a banner and lose
   nothing. **Measured**: `git diff --numstat 0636051e9` reads **`21  0`** for
   that file — a pure insertion, **zero deleted lines**. Body, measurements,
   estimates, exit gates and both pre-existing annotation banners are
   byte-identical. (The other two amended files read `31 1` and `17 5`; both are
   live docs, amended in place with dated notes, which is the intended
   treatment.)

## 7. Found and not taken

Ranked, sized, and each with the reason it was not taken by this lane.

1. **`PROGRESS_METRIC.md §5.2` DOES NOT EXIST, and three sites cite it.**
   The rule *"a wrong emit scores strictly below the refusal it replaced"* lives
   in that page's **§0 bullet 4** and **§4.2** (the mismatch-zeroing guard);
   §5 is *"Measured values and the backfill"* and has no §5.2. The dangling
   citation appears in **`PROGRESS_METRIC.md`'s own §0 line 30**, in its
   **2026-08-22 banner (twice)**, and in **`FUNCTION_BYTE_MATCH.md:44`**. The
   new roadmap cites §0/§4.2 and says so inline. **Not taken** because it is a
   correction to two pages this lane was not sent to edit, with peers in flight;
   it is one `sed` and a re-read. **It is the class `board_audit.sh` cannot see**
   (#3367/#3368 — cross-doc `file.md:§N` staleness), and it is now the fourth
   motivating instance for the detector lane that is still unbuilt.
2. **The rung-index locale defect — one line.** `#3381`, §3.5 of the new page.
   `LC_ALL=C` in `scripts/gen_rung_index.sh` plus one regeneration. **Not
   taken** because a docs lane rewriting a gate-visible generated file, on a
   branch with three peers in flight who will each add a rung doc, is how a
   merge conflict becomes a red suite. It is `M0`'s first bullet and item 2 of
   the successor's §10.
3. **`DIFF_STRUCTURE.md` wants a rescan, not an edit.** Its numbers are from
   tree `0c8a185` (3,195 wrong bodies against today's 1,960 + 530) and its own
   banner marks §3.2 refuted. **Not taken** — it is a scan, not a docs edit, and
   it is the cheap prerequisite to the permuter population measurement. Carried
   as §10 item 3.
4. **The permuter population measurement — one day.** Whether hand-written
   decomp near-misses are inlining-dominated the way the port's own refused
   bodies are. **Not taken** — it is a measurement lane, not this one, and it is
   the coordinator's recommendation rather than an owner decision (#3369). It
   is §9 item 2 and §10 item 4.
5. **`docs/README.md`'s scope paragraph is stale in a second way** that this
   lane did not touch: it still opens by describing every doc's scope as *"the
   MVP function class — a single straight-line integer-arithmetic leaf
   function"*, which was true of the format docs and is not true of `STATUS.md`,
   `CEILING.md`, `GOAL_DECISION`, `READ_PLAN` or either roadmap. **Not taken**
   — `w-docmap` owns the structure of that file this same day, and two lanes
   editing one navigation page is exactly the collision the worktree protocol
   exists to avoid.

## 8. Gate evidence

Docs-only lane: no fixture, no census, no emit path touched. The gates run
because a rung doc and a `BOARD.md` edit are both inside what the portable lane
grades.

| lane | result |
|---|---|
| `scripts/gen_rung_index.sh` | regenerated (never hand-edited), exit **0**, `INDEX.md` **+1 line** — this rung's row. Locale caveat in §7 item 2: this lane's ambient locale is `en_US.UTF-8`, which is what the committed index encodes |
| `scripts/board_audit.sh` | exit **0**. 1,984 distinct board rows · 259 distinct ROADMAP citations · **CITED BUT NOT ON THE BOARD 0** · unresolved section anchors **0** · raw line-number anchors **0** · rows behind the prose **0** · **DUPLICATE ROW NUMBERS 0** (the check that would catch a collision with a peer lane's block) |
| `cargo test --workspace` | exit **0** — **1,770 passed, 0 failed, 1 ignored**, over **48 test targets** + 5 doc-test targets (53 `test result:` lines) |

**Exit codes captured directly**, never through a pipe: each command was run as
`cmd > log 2>&1; echo $?` and the code read off the echo. Both figures above are
the echoed value, not an inference from the output text.

**THE ENVIRONMENT IS ASSERTED, NOT THE EXIT CODE** (#3341, #3219, #3231).
`census_gate` finished in **141.82 s**, not `0.00 s` — so the toolchain was
present and the capture-based tests actually executed; `differential` ran its
**31** cases in 2.31 s including `differential_reference_byte_exact_port_not_implemented`,
which cannot pass without real `c2.dll` under wibo. This worktree is provisioned
by symlink (`compilers -> ../c2-rs/compilers`) plus the sibling `../wibo` build,
which is why a fresh-worktree skip did not occur. **The log contains 0
occurrences of `SKIP: toolchain absent`, and that number is quoted here only to
say it is NOT evidence** — libtest swallows stdout for a passing test and a
skipping test passes, so a provisioned run and an unprovisioned run both print
0 without `--nocapture`. The duration is the signal.

**Not run, and why.** `scripts/gate.sh --jobs 16 --require-graded`,
`scripts/expr_sweep.sh`, `c2rs bench` and an 878-TU scan were not run. This lane
edits no `crates/` byte, no fixture, no lane list and no gate input, so every
one of those would be a re-measurement of an unchanged tree at ~80 s plus a
~1,261 s cold `mode_cross` leg in a fresh worktree (#3266) — and the criterion
they grade cannot fail on a docs-only diff, which is abstention rather than
evidence (#3336's rule). §6 names three axes that **can** fail on this rung and
reports what was measured on each. The brief asked for `board_audit.sh` and
`cargo test --workspace`; both are above.

## 9. What the next reader should do first

Read `docs/SHIPPING_ROADMAP_2026-08-22.md` §9. Two rows there are the owner's
and are recorded as decided; five are open, and the first of them —
`ARCHITECTURE_PROPOSAL` §8 decision 0's branch choice — is the one the funded
reads (R1→R2→R3) exist to be read against. **The reads do not decide it and
this page does not recommend a branch.**

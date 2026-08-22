# w-readdocs — the goal RANKING and read-before-probe propagated, and the doc that carried the doctrine turned out to be wrong about the instrument

    Tag:       w-readdocs
    Slug:      w-readdocs
    Date:      2026-08-22
    Kind:      construct
    Outcome:   built
    Fixtures:  none — construct rung: the repo's own steering surface. It
               builds no crates/ machinery. What it builds is the property
               that a session orienting off any entry point reads the goal as
               the owner RANKED it, and reads read-before-probe as a dispatch
               precondition rather than a paragraph in one 2026-08-21 doc.
    Census:    +0 — no crates/ emit rule, no refusal predicate, no fixture, no
               byte. Every `crates/` edit is comment text and the diff is
               verified comment-only two ways (§5).
    Record:    this file; authority is `docs/GOAL_DECISION_2026-08-21.md`
               § "AMENDED" (owner) and `docs/WHITEBOX_LEVERAGE_2026-08-21.md`
    Board:     **#3369**, **#3370** minted from the pointer, which moves to
               **#3371**. No existing row is edited.

> **`coff::Function` field this lane's work would eventually write: NONE.**
> Arch review finding 3's prophylactic. This lane is docs plus comment text,
> ports no pass, writes no field, and says so rather than leaving it open.

> **⚠ THIS LANE'S ASSIGNMENT CHANGED MID-FLIGHT.** The brief's
> `PROGRESS_METRIC` / `FUNCTION_BYTE_MATCH` item was to propagate
> `WHITEBOX_LEVERAGE_2026-08-21.md` §5's *mismatch anatomy* framing. The
> coordinator then found §5 was **wrong**, retracted it, and replaced the item
> with *verify it independently and amend §5 in place*. §1 is that
> verification; it is the most consequential thing this rung contains and it is
> **not** what the lane was sent to do.

---

## 0. What is being propagated

Three commits landed and nothing downstream caught up.

**`088a13194` — the owner RANKED the goals.** Quoted in
`GOAL_DECISION_2026-08-21.md` § "AMENDED" because the second sentence changes
the port's value model, not just its priority:

> *"Goal #1 is definitely the biggest. #2 is also very valuable and helps #1 by
> giving us not just docs, but actual code we can tweak to instrument + help
> produce signals about the compiler's state. this is especially valuable for
> training AI models to reverse the compiler and give us a matching pretext.
> (and build a better permuter to 'brute force' fixing code that is close, but
> wrong because of opaque compiler internal state)"*

**Goal (1) — understanding, in service of decomp — is primary.** Goal (2) —
parity — remains a real end and is additionally **instrumental to (1)**: the
port is an executable, tweakable model of c2 that emits signals about compiler
state the opaque binary cannot be made to emit. Two named consumers now exist
and lanes may be priced against them. Design rule: general layers expose
decision points (allocation order, schedule ties, label counters) as **named
enumerable parameters whose default reproduces c2 byte-exactly**.

**`088a13194` also created `WHITEBOX_LEVERAGE_2026-08-21.md`** — read before
probe, also `ROADMAP_SLICING_2026-08-21.md` §6 rules 6 and 7 and a paragraph in
root `CLAUDE.md`.

**`a0d3bb58b` — `whitebox/READ_PLAN_2026-08-21.md`**, nine ranked reads and
board **#3367**/**#3368**.

**The words `"ranked equally"` were live for a matter of hours.** A day later
they were still the operative wording in nine live surfaces. That is
**#3370**, and §4 says why it is not merely a tidiness finding.

---

## 1. THE FINDING: `WHITEBOX_LEVERAGE` §5(c) PROPOSES BUILDING AN INSTRUMENT THAT SHIPPED SIXTEEN DAYS EARLIER, AND THE SHIPPED INSTRUMENT'S OUTPUT REFUTES THE TABLE §5(c) PROPOSED

Board **#3369**. The coordinator wrote §5, found the error the following day,
and asked this lane to verify it independently rather than take the retraction
on trust. All three points check out against the tree at this lane's base
(`a0d3bb58b`), and the verification found nothing the coordinator had wrong.

### 1.1 "The missing instrument is mismatch anatomy" — FALSE, it shipped 2026-08-06

| §5(c) proposes | the tree has |
|---|---|
| "a differ that decodes both sides of every `fnbyte-differs` function" | `crates/c2-harness/src/gap/fndiff.rs`, **1,369 lines** |
| "and classifies the diff by implicated stage" | `classify_pair(port, refw) -> PairClass` at `fndiff.rs:487`, over a `Kind` enum at `:82` |
| (not proposed, and present) | LCS word alignment with insert/delete runs **paired into substitutions**; `same_multiset` at `:738` — the pure-reordering bit, with its own unit test `a_pure_reordering_reads_as_same_multiset` at `:1327`; relocation-site awareness |
| "published beside FBM under FBM's separation rule" | `docs/DIFF_STRUCTURE.md`, whose own banner reads *"Nothing here reaches a numerator, appears in an accept/refuse path, or grades the port"* |
| priced at "**1–2 wk raw**" | **withdrawn** — refreshing the numbers is one scan |

**And it is wired, not shelved.** `gap/fnbytes.rs:2569` calls
`fndiff::signature(...)` on the `fnbyte-differs` path with no flag guard — the
comment there says so: *"Additive: `fndiff-` keys only, and it runs on the
`differs` path alone, so a scan with no differs pays nothing."*
`gap/render.rs:1295+` prints the `DIFF STRUCTURE` block on **every** `c2rs gap`
scan: cluster table, substituted-words-by-decoded-field-class,
first-divergence-by-word-index histogram, and the relocation-aware line.
`--fnbyte-diff-jsonl` (`cli/gap.rs:57`) plus `scripts/fndiff_report.py` are the
per-symbol opt-in **on top of** an unconditional census — deliberately, so the
counts are never conditional on somebody passing a flag (`gap/mod.rs:81`).

### 1.2 The measured output refutes §5(c)'s diff-class table

At tree `0c8a185` over the then-3,195 differing bodies and 5,189 substituted
words (`DIFF_STRUCTURE.md` §1–§2, lane `w-bytes`, boards #976–#983):

| §5(c)'s class | §5(c)'s implicated stage | **measured** |
|---|---|---|
| permutation | scheduling | **0 bodies. The class is EMPTY.** |
| field-only | allocation | **2 words of 5,189** |
| immediate-only | layout / label plan | **2 words** (12 more reg+disp) |
| length-changing | selection / expansion | the population — but by *inlining*, not selection breadth |
| — | — | **5,173 (99.7 %) differ in OPCODE**; **0 undecoded**; **94.3 % of bodies wrong at word 0** |

**One mechanism, not four stages**: c2 inlined a callee where the port emitted
a call. 100 % of the port's differing bodies make a call or tail branch and
78.9 % of c2's counterparts make none at all.

**The consequence for the permuter argument is the part worth keeping.** §5(c)
sells the table as *"the permuter's fitness gradient — a field-only diff says
search allocation, not search everything."* Measured, that gradient points at
**nothing**: 2 field-only words and 0 reorderings say *do not search allocation
and do not search scheduling*.

### 1.3 "The tables are already read and dumped" — FALSE, and it was never load-bearing

`docs/whitebox/scripts/dump_opcode_tables.py` contains exactly two VAs:
`MNEMONIC_TABLE_VA = 0x10B1B260` and `MACHINE_TABLE_VA = 0x10B202B0`. The
base-word `0x10c3a578` and encode-form `0x10c39b18` tables are **not** dumped —
which is exactly what `READ_PLAN` §1 already said in its own inventory
(*"does **not** yet read the base-word or encode-form tables"*) and what
**R2**'s job (a) is.

Independently: **mismatch anatomy never needed them.** `fndiff.rs` decodes PPC
directly under `CODEGEN_W6_COMPARE.md`'s rule — a word is decoded only when its
form's field partition **re-encodes it bit-exactly** (`Decoded::reencode`), and
anything else is returned `undecoded`. That is why its undecoded count is 0
without reading a single table out of `c2.dll`. So R2's *"also unlocks mismatch
anatomy"* was a **spurious second justification**; R2 stands on **I2**, which
is the row that was actually priced. Struck in both docs.

### 1.4 What IS open, and it is not what §5(c) said

* `DIFF_STRUCTURE.md`'s **numbers are stale**: 3,195 is at tree `0c8a185`; the
  tree now reads `fnbyte-differs` **1,960** + `fnbyte-reloc-differs` **530**.
  A re-take is one scan.
* The page's **own ⚠ banner** marks §3.2 and one row of §4 **REFUTED** by
  `w-drop3`'s relocation reading (#984–#989) — the 140-body cluster is
  mechanism misread as deletion, and the `exact` bucket's 861-function caveat
  has since been closed by construction at `w-relo`/#884.
* **Neither is a new instrument and neither is 1–2 wk.**

### 1.5 Reported, deliberately NOT resolved: what a permuter would actually need

Two things, and both are somebody else's call:

1. **The measured wrong-body population is not "close but wrong" — it is an
   INLINING decision.** If there is a real permuter lever in this tree it looks
   more like `crates/c2-core/src/splice.rs:57-60` — an inline cost model graded
   **0.9716** with a **2.84 % NOT-MODELLED** residual that **no emitter
   consults** — than like a tie-break search. Named; **not priced**, because
   pricing it is a lane and this is a docs rung.
2. **Do not conflate two populations.** The owner's permuter use case is
   *matching pretext for hand-written decomp source*. The port's
   `fnbyte-differs` set is *the port's own refusals and wrong emits*. Nothing
   in this tree has measured the two against each other, and §5's framing slid
   between them without noticing. `WHITEBOX_LEVERAGE` §4 states the consumer
   and does **not** claim the populations coincide; that is now said out loud
   in the amended §5 so the next reader cannot make the slide silently.

### 1.6 How it was amended

`WHITEBOX_LEVERAGE_2026-08-21.md` §5 gets a dated ⚠ banner naming the
coordinator as the author of the error, the wrong paragraph is **kept verbatim**
under a strike rather than deleted, and the refuted table gains a **MEASURED**
column instead of being removed — the measured column is the finding. Parts
(a) and (b) of §5 are untouched and correct; they are what §3 of this rung
propagates.

---

## 2. The sweep — every file amended, and why

**Live orientation docs (real edits).**

| file | reason |
|---|---|
| `README.md` | § "The goal" opened on *"Two ends, ranked equally"*. Now: goal 1 is the bigger one, goal 2 is an end **and** a means; names both consumers, the decision-surface design rule, and read-before-probe |
| `docs/README.md` | the `GOAL_DECISION` entry sends readers to its § "AMENDED" and warns that the doc's own line 18 is a strike-in-place. New entries for `WHITEBOX_LEVERAGE` (flagging its ⚠ §5) and `whitebox/READ_PLAN` |
| `docs/STATUS.md` | prose only — **the generated block is untouched**. The ranking, plus a read-before-probe paragraph: a number on this page that came out of a fit now has a read attached to it (`READ_PLAN` §2). Its `DIFF_STRUCTURE` row now says the instrument is shipped and unconditional and carries both reader traps |
| `docs/GOAL_DECISION_2026-08-21.md` | line 18's *"ranked equally"* struck **in place, naming its own superseding section inline** — the mitigation for #3370 |
| `docs/CEILING.md` | goal banner ranked; **§6.1's 17-lane item F table annotated as the BLACK-BOX price**, with R4 (F1, 3–5 d) and R7 (no new reading, 3–5 d) named. Every number kept |
| `docs/STEP5_PRICING_2026-08-21.md` | **the most consequential row of the sweep**: I2 ↔ **R2** (2–4 d), I1 ↔ **R5** (15–25 d), R5 gated on R2. Plus §3's 13/65 line and a reads row on §4's curve. Four explicit non-claims |
| `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md` | §8 decision 0 gets a block saying what the read-plan changes and **explicitly not deciding it** (§3.2). §6's F0 paragraph gets a cross-reference, not a correction — it already reasons the read-before-probe way |
| `docs/rungs/README.md` | the standing-facts block (copied verbatim into every brief) carried *"ranked equally"*; ranked, plus a new bullet making both rules dispatch preconditions. Characterization lane gets the read-plan as its target list; construct rung gets the decision-surface clause |
| `docs/PROGRESS_METRIC.md`, `docs/FUNCTION_BYTE_MATCH.md` | §5(a)/(b)'s answer to the owner's sliding-judge question — **the gate stays binary** — and FBM §0 named as the standing template for gradients. **The §5(c) table is deliberately NOT propagated.** Closed a real gap: FBM had **no link** to `DIFF_STRUCTURE`, the gradient that extends it |
| `docs/GAPS.md`, `docs/PRIOR_ART.md`, `docs/SHIPPING_ROADMAP_2026-08-19.md` | ranking added to the existing `w-goaldocs` banners. SHIPPING_ROADMAP's track 1 acquires a **second** disqualification: a vendor-backed service **is** the opaque binary, so it cannot emit the signals goal (2) is now valued for supplying to goal (1) |
| `docs/whitebox/READ_PLAN_2026-08-21.md` | R2's *"unlocks mismatch anatomy"* struck (§1.3) |

**Dated records (banner / strike only, no history rewritten).**
`docs/ARCH_REVIEW_2026-08-21.md` §7 and `docs/STRATEGY_REVIEW_2026-08-13.md`
§9 — each gets a nested pointer under its existing `w-goaldocs` annotation,
with a note that the ranking leaves that annotation's reasoning intact (both
turn on the throughput retirement, which did not move). The 209+ dated rung
records under `docs/rungs/2026-*.md` were **deliberately not swept**, including
`2026-08-21-goaldocs.md:35`, which says *"Two ends, ranked equally"* and was
correct when written.

---

## 3. Outside `docs/` — #3314's class, and it fired again

`w-goaldocs` predicted a `docs/` sweep and measured three sites outside it.
This lane grepped `crates/`, `scripts/`, `c2host/`, `fixtures/` and `tools/`
for the goal vocabulary at the start rather than the end. **Result: the three
sites `w-goaldocs` fixed were already correct** — `perf.rs`, `stream/ex.rs` and
`status.sh` carry the retirement and needed no ranking repair except one.

What the grep found instead was the *other* half of the doctrine: **read
before probe asks that every fitted constant in `crates/` carry a pointer to
the read that would replace it** (`WHITEBOX_LEVERAGE` §1; the index is
`READ_PLAN` §2), and **not one of them did**. Six files amended, comment text
only:

| file | what it now points at |
|---|---|
| `crates/c2-core/src/codegen/alloc.rs` | the 52,416-config negative result and clause 2's 7-of-56 holdout refutation → **R1** (0.5 d; decides whether the ten refuted keys have an explanation at all) and **R4** (3–5 d) |
| `crates/c2-core/src/codegen/schedule.rs` | the 13,104-config negative result → **R7** (3–5 d, *no new reading*). Names the scope difference honestly: this schedules a store run, c2's is a machine scheduler over tuple regions |
| `crates/c2-core/src/codegen/order.rs` | the 1,048,576-config search, 50 of 54 misses in one bucket → **R7** |
| `crates/c2-core/src/coff/label.rs` | `LABEL_SEED_GAP = 9` and the `/Gy` `+3` → **R3** (2–4 d), **closed by construction** because there is exactly one increment instruction. Carries both limits: R3 gives the charge not the order (**R8**), and `LABEL_COUNTER.md:3-18`'s banner about four lanes misreading strides (#3368) |
| `crates/c2-core/src/codegen/encode.rs` | this file is a black-box re-derivation of `0x10c3a578` / `0x10c39b18` → **R2**, which is also the read that specs **I2**. Bounded: relocations are not in R2's scope |
| `crates/c2-harness/src/perf.rs` | the ranking, **and the trap it invites**: one named consumer wants *volume*, and *"a consumer would benefit"* is a **justification** — the move the standing rule bars. Recorded so a future reader does not mistake the consumer for a reinstated thesis |

---

## 4. Why #3370 is not a tidiness row

`GOAL_DECISION_2026-08-21.md` was **correct the whole time**. It was amended in
place, which is the house rule and is right. The failure is that **an
amendment in place propagates to nothing**, and a struck clause at the top of a
document reads exactly like a live one to anyone who does not scroll — its own
line 18 still said *"ranked equally"* eighteen lines above the section
superseding it, and every downstream page had quoted the top.

The worst site was `docs/rungs/README.md`'s § *"Standing facts every lane brief
must carry"*, which a coordinator **copies verbatim into every brief**. A stale
clause there reproduces itself into every lane dispatched that day.

Two mitigations shipped rather than argued: the `GOAL_DECISION:18` strike now
**names its own superseding section inline**, and *copying "ranked equally"
into a brief* joins the dispatch-defect list that already names *ranking a lane
on throughput grounds*. The general form is **#3368**'s, one level up: *a
citation to a decision is a citation to a document, and the document's own
amendments are part of the citation.* `board_audit.sh` cannot see this class
either — third motivating instance for the cross-doc detector lane.

---

## 5. Gate evidence

### 5.1 `crates/` is comment-only — verified TWO ways, because one is not enough

The brief requires filtering the diff to non-comment lines and showing it is
empty. Done, and a second independent check because a line-oriented filter can
be fooled by a block comment:

```sh
# (1) every added/removed line in crates/ that is not a comment line
git diff a0d3bb58b..HEAD -- crates/ | grep -E '^[+-]' \
  | grep -vE '^(\+\+\+|---)' | grep -vE '^[+-][[:space:]]*(//|/\*|\*)'
#   -> NO OUTPUT (grep exit 1)

# (2) each file, comment lines stripped, hashed at both ends
for f in <the six>; do
  git show HEAD~n:$f | grep -vE '^[[:space:]]*(//|/\*|\*)' | md5sum
  grep        -vE '^[[:space:]]*(//|/\*|\*)' $f            | md5sum
done
#   -> IDENTICAL non-comment content, 6 of 6
```

Check (2) is the one that matters: it compares **content**, not line polarity,
and it is the form `CLAUDE.md`'s formatter rule asks for (*compare content, not
status*). Both agree. **No `crates/` behaviour changed in this lane.**

### 5.2 `scripts/board_audit.sh`

```
EXIT=0
```

**Its output changed in exactly one line, and that line is the two rows this
lane minted:**

```
< board rows            : 1977 distinct numbers
> board rows            : 1979 distinct numbers
```

Everything else is byte-identical to the base run: `CITED BUT NOT ON THE BOARD
0`, `UNRESOLVED SECTION ANCHORS 0`, `RAW LINE-NUMBER ANCHORS 0`, `ROWS BEHIND
THE PROSE 0`, `DUPLICATE ROW NUMBERS 0`, and the same nine suppressed lines
with the same counts. `#3369`/`#3370` are cited from `ROADMAP.md` nowhere, so
no citation check moves — which is expected for rows minted by a docs lane and
is stated rather than left as an absence.

### 5.3 `cargo test --workspace`

Run as the brief specifies. **Exit code captured directly into a file — never
through a pipe**, because `${PIPESTATUS[0]}` is bash and this box runs zsh:

```sh
cargo test --workspace > work/w-readdocs/suite.log 2>&1
echo "CARGO_EXIT=$?" > work/w-readdocs/suite.exit
```

See §5.4 for the counts and the exit code as measured.

### 5.4 Suite result

*(filled from `work/w-readdocs/suite.exit` and `suite.log`; see the merge
note.)*

**Read the skip count correctly** (#3341): a skipping test **passes**, and
libtest swallows stdout for a passing test, so *"0 occurrences of `SKIP:
toolchain absent`"* is **vacuous** — it is satisfied by the failure it claims
to detect. This lane does not assert on it. The environment, not the exit
code, is the thing to state.

---

## 6. What this rung does NOT claim

1. **That the sweep is exhaustive.** It is grep-driven over the ranking's
   vocabulary (`ranked equally`, `rank equally`, `two ends`, `perfect
   reproduction`, `GOAL_DECISION`, `thesis`) across `docs/`, `crates/`,
   `scripts/`, `c2host/`, `fixtures/` and `tools/`. **A doc that argues from
   the equal ranking without using the words is invisible to it** — the same
   limit `w-goaldocs` declared, and it has not been closed.
2. **That decision 0 is settled.** It is not, and §3.2's block in
   `ARCHITECTURE_PROPOSAL` is written to inform it and explicitly not to
   answer it. Whether R1→R2→R3 is a cheaper Phase 0, a prerequisite to Phase 0
   or an orthogonal third thing is a scheduling judgement **this lane does not
   make**.
3. **That the reads re-price I1, I2 or item F.** A read produces a **spec**;
   those rows are implementations. Applying `CEILING` §5's ~5:1 to a read is a
   **units error** and the §4 curve row says so in place of a number.
4. **That any read is a fact about c2.** `[R]` means the instructions were read
   correctly (`whitebox/ref/README.md:49`); the `.bss` bump rule was read
   correctly and was wrong about c2. Every read still ends in a confirmation
   probe, and the byte judge is untouched by all of it.
5. **That `DIFF_STRUCTURE.md` was re-measured here.** It was not. §1.2 quotes
   it at tree `0c8a185` **with that tree named**, and §1.4 states the two
   reasons a re-take is owed. Quoting it without the tree would repeat the
   error this rung is about.
6. **That `CLAUDE.md` was edited.** It was not. `088a13194` already put the
   ranking and read-before-probe into its § "The goal", and it is current.

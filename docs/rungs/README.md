# `docs/rungs/` — one file per rung

Rung write-ups land here, one file per rung, instead of growing a new
`§6`-letter section in `docs/ROADMAP.md`. The reason is in
`docs/ARCHITECTURE_SEAMS.md` §2.5/§3.4: on 2026-07-30 nine rungs landed in
parallel and *every* merge conflicted on `ROADMAP.md`, `GAPS.md` and
`expr_sweep.sh`, while the tag `W23` and the section letters §6e/§6f/§6g/§6i
were each claimed twice by concurrent agents. Section numbers and small integers
allocated concurrently collide silently; **filenames collide as add/add
conflicts git flags loudly**.

## The convention

* **Filename is the claim.** `YYYY-MM-DD-<slug>.md`. The slug is the rung's
  identity and matches its fixture prefix's meaning (`w25_store_leaf.cpp` →
  slug `store-leaf`). Two rungs claiming one slug is an add/add conflict.
* **`W`-numbers are assigned at merge**, by the one serial actor with the one
  sequence — never by a branch. A branch may leave `Tag:` as its slug until
  then. See §3.4.
* **Header block is machine-read.** `crates/c2-harness/tests/rung_registry.rs`
  (portable lane, no toolchain) parses the indented `Key: value` block and
  asserts: tags unique, declared slug equals the filename slug, every named
  fixture exists, every rung names at least one fixture, no fixture prefix
  claimed twice, and `INDEX.md` equal to what `scripts/gen_rung_index.sh`
  generates.
* **`INDEX.md` is generated**, never hand-edited: `scripts/gen_rung_index.sh`.
* Files whose name starts with `_`, plus `README.md` and `INDEX.md`, are not
  rung docs and are skipped by both the test and the index.

Start from `_TEMPLATE.md`.

## Lane kinds

Adopted 2026-08-13 from `docs/STRATEGY_REVIEW_2026-08-13.md` §4 lever 1: the
fixture-claim rung was the only unit of work the repo maintained, and that unit
cannot carry any of `CEILING.md` §6.1's seven phases — which is why five of
them never had a building rung. Three kinds are now first-class. The registry
test already admits all three: kinds 2 and 3 use the `Fixtures: none — <reason>`
+ `Census: … +0` path (`rung_registry.rs`, the instrument-rung exception).

1. **Fixture-claim rung** (the default, everything above): names fixtures,
   claims a prefix, moves the census. The unit for TU-shaped work.
2. **Construct rung** (precedent: board **#290**, item B of `CFG_SHAPE.md`
   §6.2): builds shared machinery — an IR type, a pass, a gate predicate — by
   re-expressing **already-byte-exact** classes through it. `Fixtures: none —
   construct rung: <what it builds>`; `Census: +0`; success criterion is a
   **required-zero byte delta**, graded by a line-for-line identity diff of
   per-lane gate counts before/after. A construct rung that changed any byte
   FAILED, whatever else it did.
3. **Characterization lane** (precedent: `2026-08-13-wb-live.md`): reads real
   c2's behavior — whitebox addresses plus obj-grid confirmation — and lands
   findings, not code. `Fixtures: none — characterization: <the question>`;
   `Census: +0`; prereg frozen before the first probe; every load-bearing
   claim cites an address or a grid cell; disassembly-derived adoption rules
   (`docs/whitebox/DISCLOSURE.md`) apply unchanged.

## Outcome, one word

Every rung doc's header carries an `Outcome:` line (new docs; the 209 landed
before 2026-08-13 are not backfilled). Exactly one of:

- `converted` — TU `match` moved.
- `declined` — a priced refusal; the decline and its price are the deliverable.
- `instrument` — built or corrected a measuring instrument.
- `built` — a construct rung or characterization lane that landed what it
  preregistered (zero-delta held / findings confirmed).
- `FAILED` — none of the above. Stated in that word. A lane that neither
  converts, declines, prices, nor builds is not "a compound finding" — before
  this field existed, a failed lane was indistinguishable in the record from a
  successful one at the level of artifacts produced (STRATEGY_REVIEW §2 H3).

The merge funnel checks the field is present and matches the headline before
authoring `work/merge-<lane>.txt`.

## Two rules a probe must satisfy — added 2026-08-17

Both were derived independently by two lanes in one wave (`w-fence163` board
**#3219**, `w-mutcensus` **#3231**), each catching the defect **in the
flattering direction** in its own instrument, before publishing an affected
result. They bind any lane that runs a campaign of measurements.

**1. Carry a control whose failing set is pinned by NAME, and re-run it in
every environment the campaign uses.** A control pinned by *count* passes in an
unprovisioned worktree the moment the count matches. The concrete failure: a
fresh `git worktree add` has no `compilers/` (gitignored by design, and it does
not follow a new worktree), so every capture-based test **skips**. The
red-maker then reports *"3 passed"* in 0.00 s, and **cargo swallows the SKIP
line for a passing test** — so a mutant that should be RED reads GREEN with a
clean suite, the right target count, and the right exit code. `w-mutcensus`'s
variant is worse still: its registered baseline was **byte-identical with and
without a toolchain** (1,648/0/42 either way, differential 84.17 s vs 0.00 s),
so the prereg's own probe definition *and* its `targets != 42` invalidation
rule were **both blind**. It surfaced as a **contradiction between two runs of
one mutant** — never by inspection. Assert the *executed-test count* and the
differential's *duration*, not the exit code. A colour taken in an unvalidated
environment is **void, not provisional**: discard it, re-run it, and keep the
invalid log.

**2. Derive the results table from the logs; never accumulate it.** This is
what let `w-mutcensus` reapply three classifier corrections retroactively
across a 159-run campaign. An accumulated table cannot be re-derived when the
classifier turns out to be wrong, and the classifier **was** wrong three times.

## Board numbers are allocated by the coordinator, not read from the pointer

Added 2026-08-17, on evidence from the same wave: **row-by-row verification is
the strongest check a lane can run and is still structurally insufficient.**
`w-fence163` and `w-gatewire` both minted `#3218`–`#3221`, each having verified
row-by-row against a master where neither had landed. `w-mutcensus` verified
`#3218`–`#3230` against `BOARD.md`, both peer branches, **and** every `#32NN`
citation in `ROADMAP.md`, concluded `#3222`, and was still wrong — the
verification was not at fault. Two lanes on separate branches are blind to each
other's mints by construction, and `board_audit.sh` counts duplicates only
*after* both merge.

So: **a lane with peers in flight leaves its rows UNNUMBERED and says so, or
takes a block the coordinator allocated explicitly.** The next-free pointer in
`BOARD.md` is a hint, not an authority — it is routinely already wrong at the
tip that prints it, because unmerged branches hold blocks it cannot see.

## Standing facts every lane brief must carry — updated 2026-08-18

These are the things lanes have most often been briefed *wrongly* about. A
coordinator dispatching a lane copies this block; a lane that finds one of
them stale reports it as a **dispatch defect**, not a preamble (`w-dagorder`'s
pattern — it was sent at a lever discharged the same day the review naming it
was written, and **detection was incidental**, so the defect's true rate is
unmeasured).

* **Run the gate as `scripts/gate.sh --jobs 16 --require-graded` — ~80 s.**
  `--jobs 4` is ~153 s and is not the safer choice; the box was never
  CPU-starved. A lane's **first** gate in a fresh worktree still pays a
  ~1,261 s cold `mode_cross` leg (`w-gateperf` §11.1, board **#3266**) — that
  is a property of the worktree, not of the change.
* **Run suites as `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release
  --no-fail-fast`.** Armed 2026-08-18. Without it, a fresh worktree with no
  `compilers/` skips every capture-based test **silently** — cargo swallows
  the `SKIP` line for a passing test — so a registered RED reads GREEN with a
  clean suite, the right target count and the right exit code (#3219, #3231).
  The two runs are otherwise **byte-identical**; only the durations differ.
* **Count `gap-metric` keys with `grep -cE '^ *gap-metric \S+ \S+$'` → 394.**
  Never `grep -c 'gap-metric'` → **396**: two prose lines merely *mention*
  keys. **Three consecutive lanes hit this**, and the third *explained* the
  +2 by attributing it to a peer's merge (#3269). Hence the general rule:
  **a lane that finds an unexpected delta owes a measurement before it owes a
  cause** — a wrong count invites re-counting, a wrong cause removes the
  reason to check.
* **`fnbyte-*` is not a pure function of the commit** (#3249). It reads
  (commit × capture-cache state × untracked workload). Re-read base and tip
  **back to back**; state the cache state and `dc3-decomp` head; treat any
  effect under ~10 bodies as **unattributable rather than reporting it**.
* **`fnbyte-*` denominators are 71.2 % bodies the shipped image never
  contains** (#3254) — `/Gy` COMDATs the linker discards. A `fnbyte` ratio is
  progress over *what c2 emits*, which is the port's job; it is never progress
  over *the game*.
* **The per-symbol `fnbyte-differs` set compare is void** for any lane that
  changes the admitted population (#3237). Use a name-stable per-TU shape
  multiset.
* **"Graded tree identical at both ends" applies only to revert-everything
  lanes** (#3215) — any lane landing a test moves it by construction. **Do not
  compare release-binary sha256 across worktrees** (#3224): `CARGO_MANIFEST_DIR`
  is compiled in, so that comparison is void by construction.
* **Board blocks are allocated by the coordinator** — see the section above.

## What is here, and what is not

The historical rungs live in `docs/ROADMAP.md` §6a–§6m and in the per-subject
`docs/*.md` write-ups, and they **stay there** — history does not conflict, only
growth does (§9.6: "freeze and fork, don't migrate"). The files here are the
rungs that already had a `§6`-letter section, restated as a claim of their tag
and fixtures with a pointer to the authoritative record; the remaining
prefixes in `fixtures/cpp` (`w5`, `w6`, `w10`, `w12`, `w13`, `w13b`, `w14`,
`w15`, `w16`, `w17`, `w18`, `w19`, `w20`, `w21`, `w23`, `wfr`) are not yet
claimed here. Backfilling them is `ARCHITECTURE_SEAMS.md` §6 step 4; until
then the registry test asserts uniqueness over what *is* claimed and prints
the unclaimed prefixes rather than pretending to full coverage.

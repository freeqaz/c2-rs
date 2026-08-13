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

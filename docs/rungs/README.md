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

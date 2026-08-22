# DOC_CONVENTIONS — how documents work in this repo

**What this is.** The conventions this tree *already follows*, written down.
Almost nothing here is new: it was recovered by reading the 70 top-level docs,
the 64 under `docs/whitebox/` (11 of them in `ref/`), the 389 under `docs/rungs/` and the scripts that
generate three of them. Where a rule is genuinely proposed rather than
observed, the paragraph says **PROPOSED** and nothing downstream depends on it.

**Why it is worth having.** Two of this project's recorded defects are
documentation-mechanics defects, not analysis defects: **#3370** (an amendment
landed in place propagated to nothing, and the superseded wording was still
operative in nine live surfaces a day later) and **#3367/#3368** (`file.md:NNN`
citations are load-bearing and `scripts/board_audit.sh` cannot see them break).
Both are conventions failing silently. A convention nobody wrote down cannot be
checked, and this repo's own doctrine is that an unenforced rule is the missing
piece rather than another paragraph.

Written 2026-08-22 by lane `w-docmap`, under the owner's structure order
(`DECISIONS_2026-08-22.md` decision 4). The layout model was
`../decomp-synth/docs`, adopted where it fits and *not* adopted where it
conflicts — see §8.

---

## 1. Freshness classes — the four kinds of document here

Every row of [`README.md`](README.md) carries one of these. They are not
severity levels; they say **how to read the page**.

| class | what it means | how you can tell |
|---|---|---|
| **live** | maintained; if it is wrong, fix it in place | no date in the filename, no `Status: … <date>` freeze |
| **generated** | produced by a script; **never hand-edit** | the file says so at the top, and a test or a script fails if the tree disagrees — see §4 |
| **dated record** | a measurement or decision made on a day, kept as written | a date in the filename (`*_2026-MM-DD.md`, `rungs/YYYY-MM-DD-*.md`) **or** a `Date:` / `Decided` / `Lane:` header block |
| **superseded** | a dated record something later overturned | a banner at the top naming what replaced it. The body is **not** rewritten |

**A dated record is not stale advice — it is evidence.** The failed attempt,
the estimate that missed 5:1, the rule refuted by the next cell: this repo's
most valuable pages are the ones that record a negative result, and rewriting
one to be "current" destroys the only thing it was for. `ROADMAP.md` is 8,000+
lines that are largely this, which is exactly why `docs/STATUS.md` exists as
the one-page live answer and why `README.md` routes a newcomer there first.

## 2. Strike-in-place amendment — the house rule, and its known failure mode

When a dated record is overturned, the tree's rule is **amend in place with a
banner, never rewrite and never delete**. `GOAL_DECISION_2026-08-21.md`
§ "AMENDED", `LABEL_COUNTER.md`'s ⚠ banner and `WHITEBOX_LEVERAGE_2026-08-21.md`
§5(c) are the models: the superseding text goes at the top, the original stays
untouched below it, and the reader is told which to take.

> **AND IT IS ALSO THE FAILURE MODE — board #3370.** *A struck clause at the
> top of a doc reads exactly like a live one to anyone who does not scroll.*
> "Ranked equally" was superseded by the owner within hours and was still the
> operative wording in **nine live surfaces** a day later, including
> `rungs/README.md`'s § "Standing facts every lane brief must carry" — the
> block a coordinator copies verbatim into every brief — and the opening
> paragraphs of the three documented entry points.

Two mitigations already shipped, and they are the convention now:

1. **A strike names its own superseding section inline**, at the struck line —
   not only in a banner 18 lines above. `GOAL_DECISION_2026-08-21.md:18` is the
   worked example.
2. **An amendment is not landed until its consumers are swept.** Grep for the
   struck wording across `docs/`, `crates/`, `scripts/` and `README.md` in the
   same lane. `#3314` and `#3370` are the same defect one day apart, and
   `#3314`'s three non-`docs/` sites were reached by no `docs/` grep.

## 3. Citations — the chain, and the one detector that exists

A citation in this tree is `docs/<NAME>`, a bare `<NAME>` resolved against the
citing directory, `<NAME>:NNN`, `<NAME> §6`, or `crates/…/bar.rs:57-60`. They
are **load-bearing in five places**: docs prose, `BOARD.md` rows (the `Where`
column), rung docs, `crates/` comments, and merge-commit messages.

(The placeholders in that sentence are spelled `<NAME>` rather than `FOO.md`
on purpose. A detector that reads its own documentation reads the examples as
citations — the first draft of this file added three findings to the audit, all
of them quoted defects. Any prose *about* a broken citation has to avoid
writing the broken path literally.)

* **Cite the document and the section, not a line, when a line is not the
  point.** `#3368`'s general form: *a citation to a decision is a citation to a
  document, and the document's own amendments are part of the citation.* A
  `:NNN` that still resolves after 400 lines are inserted above it is silently
  wrong, and no tool can catch that.
* **Never move or rename a document without measuring its inbound citations
  first.** `grep -rI -F '<basename>' .` is the measurement. This is why the
  2026-08-22 structure pass is a navigation layer that **moved nothing**: of
  70 top-level docs exactly two had zero inbound references.
* **[`scripts/doc_cite_audit.sh`](../scripts/doc_cite_audit.sh)** is the
  detector, built 2026-08-22 after the gap was named three times. It checks
  that every cited `.md` resolves and that every unambiguous `:NNN` is inside
  the file. It prints the number of citations it checked — a matcher that
  matched nothing and a clean tree look identical from the outside — and every
  suppression class is counted in the output rather than hidden.

  ```sh
  scripts/doc_cite_audit.sh --self-test   # POSITIVE CONTROL — watch it go red first
  scripts/doc_cite_audit.sh               # the whole docs/ tree, ~2 s
  ```

  **It is not wired into `scripts/gate.sh`, deliberately.** On arrival
  (2026-08-22, tree `cbb1bf976`) it reports **39 findings**, every one of them
  pre-existing and nearly all inside dated records that §1 says must not be
  edited. The four classes, none of them fixed by the lane that built the
  detector:

  | class | count | note |
  |---|---:|---|
  | eight rung docs place `DISCLOSURE` at the top level of `docs/` when it lives under `whitebox/` | 8 | all dated records |
  | `BOARD.md` cites three 2026-08-05 rung files that are not in the tree (`w-tu4`, `w-loop`, `w-fuzzy`) | 12 | plus one more in `CFG_SHAPE.md` |
  | `ROADMAP.md` cites a front-end bundle document that does not exist, twice, and one prereg that does not | 3 | live doc, real defect |
  | `:NNN` into a file that was later shortened, unambiguous name | 8 | `container.rs`, `gap.rs`, `main.rs` |

  The remainder are three `_draft-roadmap-*` files a prereg pointed at, two
  read-plan reference pages proposed but not yet written, and a rung doc
  pointing at its own non-`_` name. Gating a red detector teaches people to
  pass `--no-verify`. **PROPOSED:** wire it into the gate once this backlog is
  triaged, or add a `--baseline` mode so the gate reads "no *new* findings".
* **What it cannot do, stated so a green run is not over-read:** it cannot tell
  whether the cited line still says what the citer thinks. That is #3370's
  actual failure mode and it remains a human check.

## 4. The generated files — never hand-edit these

| file | generator | what enforces it |
|---|---|---|
| `STATUS.md`'s metric block | `scripts/status.sh --write` | the doc's own first line: *"This doc is a cache, not a source … if the block and the tree disagree, the tree is right"* |
| `rungs/INDEX.md` | `scripts/gen_rung_index.sh` | `crates/c2-harness/tests/rung_registry.rs` fails in the **portable** lane if the file on disk differs from what the script produces |
| `perf/perf_scale.png` | `scripts/plot_perf.py` from `perf/perf_scale.csv` | nothing — regenerate it when you change the data |
| `whitebox/c2_*.tsv`, `whitebox/labels/*.tsv` | `whitebox/scripts/build_*.py` | nothing — they are pinned to the image sha256 in `whitebox/C2_MAP_METHOD.md` §0 |

`BOARD.md` is the deliberate exception: it is **hand-maintained** because there
is no header block to generate it from, and its only protection is
`scripts/board_audit.sh`. Adding a board item means adding its row in the
**same commit** that mints the number.

## 5. Naming patterns

| pattern | where | means |
|---|---|---|
| `SCREAMING_SNAKE.md` | `docs/` top level | a subject document — a characterization, a spec, a metric definition |
| `SUBJECT_2026-MM-DD.md` | `docs/` top level | a **dated record**: a review, a decision, a pricing, a proposal made on that day |
| `WB_<SUBJECT>_PREREG.md` / `_PREREG_R2` / `_R3` | `docs/whitebox/` | predictions frozen **before** the lane looked. R1 before the first grep of the export; R2 before the first `cl.exe`; R3 (once) mid-grid |
| `WB_<SUBJECT>_FINDINGS.md` | `docs/whitebox/` | what was read, every claim carrying an absolute VA, prereg scored |
| `P_<SUBSYS>.md` | `docs/whitebox/ref/` | the address-indexed reference page for one subsystem (`P_DAG`, `P_REGALLOC`, `P_COFF`, …). **Cited bare from six directories** — do not move one |
| `YYYY-MM-DD-<slug>.md` | `docs/rungs/` | a rung. **The filename is the claim**; two rungs claiming one slug is an add/add conflict git flags loudly, which is the whole reason rungs are files and not `ROADMAP.md` sections |
| `_<anything>.md` | `docs/rungs/` | not a rung: preregs, findings, drafts. Skipped by both `gen_rung_index.sh` and the registry test |

## 6. The prereg/findings pairing

A characterization lane's first commit is its prereg, and the prereg is scored
in the findings document. This is not ceremony — `WB_READER_PREREG_R2.md`
exists because all four of round 1's predictions came back wrong, and that is
recoverable only because round 1 was frozen before the lane looked.

**Read a findings document with its prereg open.** Read alone it is a story;
read against the prereg it is evidence. `docs/whitebox/README.md` lists the
pairs.

## 7. Rung docs, board rows, and outcomes

Fully specified in [`rungs/README.md`](rungs/README.md); the parts that bind
every document, not just rungs:

* **One `Outcome:` word per lane** — `converted`, `declined`, `instrument`,
  `built`, or `FAILED`. A lane that produced none of its deliverable says
  **FAILED** in those words, not a compound headline.
* **Three lane kinds** — fixture-claim, construct, characterization — and phase
  work is dispatched as the latter two, never as a TU lane.
* **Board numbers are allocated by the coordinator**, taken from the pointer at
  the bottom of `BOARD.md`, and the row lands in the same commit as the number.

## 8. What this repo does NOT adopt from `../decomp-synth/docs`

That tree is the layout model the owner named, and three of its conventions
were examined and **declined** here, each for a stated reason:

* **Topical subdirectories.** decomp-synth moves docs into `architecture/`,
  `research/`, `plans/`. Here that would break citations in every one of the
  five places listed in §3, and `board_audit.sh` cannot see the breakage. The
  measured inbound-citation counts are the argument: `GAPS.md` 620,
  `BOARD.md` 568, `CFG_SHAPE.md` 357, `ALLOC.md` 349. **Routing is done with
  index pages instead** — `README.md` and `whitebox/README.md` — which cost
  nothing and break nothing.
* **A `**Hub:**` breadcrumb on line 1 of every doc.** It would mean touching
  all 523 markdown files under `docs/`, most of them dated records that §1 says stay as written.
  The index pages carry the same routing at the index end.
* **A navigability metric (`docgraph.py`, sinks / hop-distance / gloss %).**
  Not priced. **PROPOSED at most**, and only if somebody names what decision it
  would change — this repo's standing rule is that an instrument is justified
  by the decision it moves, and four consecutive lanes here were dispatched off
  rankings that turned out to be artifacts of their own instrument.

Adopted from it, and worth naming: **purpose-first routing** ("if you are
asking X, go to Y"), **a reason on every index row** rather than a restated
title, and **writing the conventions down as a file** at all.

## 9. What nobody writes

* **No absolute machine paths** in file contents. Use `C2RS_*` env vars or
  repo-relative defaults; toolchain location is env-driven by design.
* **No AI/agent trailers** on commits — human identity only.
* **No captured or generated IL, objs, or build artifacts** committed. Only the
  fixtures' `.cpp` is tracked.
* **No "neutrality" or "behavior-preserving" classifier presented as a gate.**
  The compiler is the sole judge. Measurement of neutrality is required; a
  classifier standing in for the byte judge is banned.
* **No bare metric quoted out of `ROADMAP.md`.** Read `STATUS.md` first — it
  carries which numbers are targets, which are drivers, and why `mismatch 0` is
  not evidence of correctness.
* **Do not scrub the project context**: `dc3`, the `e:\lazer_build_gmc1` build
  root and XDK id `16.00.11886.00` are intentional.

## Where to go next

| Go to | When |
|---|---|
| [`README.md`](README.md) | you want the routing layer itself — which document answers the question you actually have |
| [`rungs/README.md`](rungs/README.md) | you are writing a lane and need the rung header, the three lane kinds, and the standing facts every brief must carry |
| [`whitebox/README.md`](whitebox/README.md) | you are adding to the binary record and need the prereg/findings pairing in context |
| [`../scripts/doc_cite_audit.sh`](../scripts/doc_cite_audit.sh) | you just edited a doc and want to know whether you broke a citation — run `--self-test` first, then the audit |

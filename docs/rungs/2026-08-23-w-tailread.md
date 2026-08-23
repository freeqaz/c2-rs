# w-tailread — the attribute table was already recorded, the 767 was never a set, and the dispatch tail expands nothing

    Tag:       w-tailread
    Slug:      w-tailread
    Date:      2026-08-23
    Kind:      characterization
    Outcome:   built
    Fixtures:  none — characterization lane: R6's top-ranked follow-up
               (docs/rungs/2026-08-23-w-read-r6.md § "Found and not taken" 1–2)
    Census:    unchanged → unchanged (+0)
    Record:    docs/whitebox/ref/P_OPATTR.md (spec) ·
               docs/whitebox/WB_TAILCLASS_FINDINGS.md (grade) ·
               docs/whitebox/WB_TAILCLASS_PREREG.md (prereg)

## What it admits, and what it refuses

**Admits.** `0x10c3afd8` read completely — **664 entries**, extent *derived*
(`0x10b1b260 + 0x298*12 == 0x10b1d180` exactly), **byte-identical to the
mnemonic table's `flags` field on 664 of 664**, low three bits decoded as an
**operand-shape class** (1 move / 2 load / 3 store / 4 sign-extend / 0 other;
classes 5–7 and bit `0x80` unused), **38 consumers** located image-wide. The
dispatch tail read to its five callees and shown to **emit nothing**. The
**delete primitive `0x10bd5516`** named for the first time in this record.
Peephole arm 6 read *and* obj-checked. **`0x10b1d180` SETTLED.** R6's
registered gap closed by reading all ten fall-through bodies.

**Refuses.** The `mr r8,r8` idiom is **not explained** — 3,792 instances, one
register, branch-adjacent, and a plausible story is deliberately absent from
the record. `P_EXPAND.md` §3 is **not re-scored**. The 506 callback sites are
**counted, not classified**. Every `[R]` stays `[R]`; exactly one claim is
`[O]`, and it came back **negative**.

**The deliverable came back a different shape than the brief asked for, and
that is the finding.** The brief asked to *"convert '767 opcodes reach the
tail' into 'opcode X is / is not expanded'"*. **There was never a 767-element
set.** `0x2ff` is 767 and it is the walk's entire domain: set the bound to
`0x400` and the "measurement" reports 1024; `0x600` reports 1536. What replaces
it is stronger — **the tail's word delta is zero for every opcode reaching it**,
because it attaches an operand rather than emitting one.

## Estimate vs outcome

Predicted reach **0**, realised **0**. **Zero `crates/` bytes and zero
`fixtures/` bytes**, as fenced — `git diff --numstat f53877aa5..HEAD -- crates
fixtures` is empty. Docs-only: **9 files, +2,176 / −73**, over 16 commits.

Prereg scored **11 HIT / 2 MISS / 4 PARTIAL / 4 UNGRADED** over 21. The four
UNGRADED are marked `[POST]` in the prereg itself: I had already seen those
answers during the orientation pass, said so before the read, and they are **not
counted as hits**.

**R6's calibration note worked, and then cost me one.** R6 recorded that every
one of its misses predicted the mechanism *tidier* than it is; the prereg says I
biased toward messier. Five predictions were of that form and **four hit**. The
one MISS (P4.4) failed in the *opposite* direction: I predicted a messier origin
for R6's `twlti` (a second-order trap) when the truth was the tidy one — R6's
own stated hypothesis, applied exactly as stated. Biasing toward mess is a
correction, not a law.

## Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **PASS — 53 targets, 1,837 passed, 0 failed, 1 ignored, exit 0.** Under that flag exit 0 *is* the liveness assertion (`crates/c2-harness/tests/require_toolchain.rs` makes a toolchain-less run fail rather than skip). No fixed duration cut and never "0 SKIP lines", which is vacuous (#3341) |
| `scripts/gate.sh --jobs 4 --require-graded` | **PASS, exit 0.** 18 lanes in the registry — **18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**; **6,948 fixture-verdicts**; sweep **19,460 of 19,556** graded, **0 mismatch**; cross **90,424 of 90,812** cells graded, **0 mismatch**; debug lane 18/18, 6,948 verdicts, match 2,423, **0 mismatch, 0 PANIC**. Graded tree `ec61c412fb69`, 769 content-hashed files |
| `scripts/board_audit.sh` | **rows 1,996 → 2,000 (+4)** — exactly the four reserved. 0 unresolved section anchors, 0 raw line-number anchors, 0 rows behind the prose, **0 duplicate row numbers**, 0 cited-but-absent. Exit 0, which its own header calls advisory; the row-count diff is the evidence |
| 878-TU workload scan | **not run** — this lane changes no `crates/` byte, so the scan cannot move. Recorded as not-run rather than as a pass |
| fixtures, `c2rs census` | **n/a** — `Fixtures: none` |

**A green gate is a NON-REGRESSION statement for this lane and nothing more.**
The gate cannot see a single thing this lane did: it changes no `crates/` byte,
so no gate row can rise or fall on the strength of it. Stated because a
characterization lane quoting a green gate as if it graded the work is exactly
the misreading the rung format exists to prevent.

### The lane's own evidence, which is not a gate row

* `docs/whitebox/scripts/dump_tailclass.py` — `--table`, `--consumers`,
  `--tail`, `--minters`, `--extended`; sha256-fenced, disassembling the pinned
  PE directly.
* `docs/whitebox/scripts/probe_selfmove.py` — the corpus control, **120,000
  objs**, which **refuted** the licence for arm 6's read.

**Both were watched failing on deliberately broken input before being trusted**
(`CLAUDE.md`), and both fences found something:

* `dump_tailclass.py` refuses a truncated image **and** a single flipped byte
  *inside the table it reads* — `REFUSE: sha256 … is not the pinned image`,
  exit 1 both.
* `probe_selfmove.py` has three outcomes and all three were watched: absent
  cache → `SKIP`, exit 0; empty corpus → `VACUOUS`, exit 2, **not** a pass; a
  real obj with one word patched to `mr r4,r4` → `REFUTED`, exit 1. **That last
  test found a real ordering bug in the probe** — the vacuity check preceded the
  refutation check, so the one input that most clearly refutes the claim was
  being reported `VACUOUS`. Fixed, with the reason in a comment.
* Smoke-running every documented mode found a **second** bug: `--minters` still
  called a method this lane had renamed, and would have crashed.
* Checking *what actually matched* found a **third**: the consumer scan's hex
  regex admitted addresses **below** the table. Nothing in this image matched
  there, so the count of 38 was right by luck; it is now an explicit range test
  and right by construction.
* Widening that same scan to the extended table found a **fourth, and it
  refuted a claim this page had already published.** `P_OPATTR.md` §7.1 said
  *"one referencing function"*, from a base-literal grep. The truth is **13
  references in 3 functions** — and a naive range scan says 20, because **both
  tables live inside `.text`**, so `objdump -d` disassembles their bytes as code
  and invents branch operands landing in the table's own range. Seven phantoms,
  filtered by function membership.

A fence that has never been seen to refuse is not evidence that anything was
fenced — and **all four** of the things these checks caught were mine. Three of
the four were found by *running* something rather than reading it.

## The instrument defect, which is the transferable part

`dump_expansion.py`'s **"767 opcodes reach the tail"** is its walk's whole
domain. **My first repair for the minting question had the identical defect**: a
transitive *"can an instruction constructor be reached from the tail"* query
answers **True** — and True for every control too, because c2's call graph is
strongly connected through its arena and diagnostic machinery. A 22-hop witness
path through the error machinery is not evidence about codegen.

Two saturated predicates, one lane apart. It was **discarded, not shipped**, for
a BFS **minimum hop count** — 8 for all three tail bodies, 1 for three controls
that demonstrably mint — whose caveat prints in the tool's own output so the
number cannot be quoted without it. The argument `P_OPATTR.md` actually rests on
is the **direct reading of the five callees**, not the hop count.

**The rule: before quoting a count, change a parameter it should not depend on
and re-run.** 767 → 1024 → 1536 takes ten seconds and would have caught this in
R6.

## Found and not taken

Ranked, and the first is the one a follow-up should buy.

1. **What is `mr r8,r8`?** 3,792 instances in 1,206 of 120,000 objs, **all
   `r8`**, branch-adjacent, at `/Ox`, with no relocation covering them. Real,
   reproducible, unexplained — and **obj-visible**, so it needs no disassembly
   and no wibo. The cheapest open question this lane leaves.
2. **Re-score `P_EXPAND.md` §3 with a signed word delta.** Both primitives are
   now named (`0x10bd3824` mint / `0x10bd5516` delete); the instrument needs a
   delete oracle beside its mint oracle. At least four arms scored `0..0` in
   fact delete.
3. **The second byte table `0x10c3b270`** — same `0x298` extent, two consumers
   bound-checked at `0x295` with out-of-range default `0x64`. In no document,
   and this lane did not read it.
4. **Opcode `0x302`** — arm recovered at `0x10c0e479`, meaning not chased.
   Absent from `P_ILRECORD.md`'s minting arms, as is R6's `0x2f5`.
5. **`WB_EXPAND_FINDINGS.md:79` and board #3432 carry the same false
   "unrecorded" sentence** that `P_EXPAND.md` §1.2 did. The page is amended;
   those two were outside this lane's four reserved rows and are **not** fixed.
6. **Arm 14's handler `0x10c16d83` is unread**, and it is where the `mr`
   self-move exception lives. This lane read arm 6 and obj-cleared it (0 of
   32,569 `fmr`); the violator is the sibling it did not read.

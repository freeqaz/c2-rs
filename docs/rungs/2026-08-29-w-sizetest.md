# `w-sizetest` — the mask is a constant, the size test does not run, and the brackets bracket nothing

    Tag:       w-sizetest
    Slug:      w-sizetest
    Date:      2026-08-29
    Kind:      characterization
    Outcome:   built
    Fixtures:  none — characterization lane; predicted reach 0, delivered 0
    Census:    +0 (no `crates/` code written; no compiled file changed)
    Record:    docs/whitebox/WB_SIZETEST_FINDINGS.md
    Prereg:    work/w-sizetest/PREREG.md, committed 7dd4e264e BEFORE the image was opened
    Board:     #3870–#3876
    Image:     compilers/X360/16.00.11886.00/c2.dll, sha256 c80981c0…a66258 (verified in-lane)

**Predicted reach 0 and byte delta 0, stated up front and both held.** This
lane wrote zero bytes under `crates/`; the graded-tree hash printed by
`gate.sh` is identical to the base tree's, which is the check rather than the
claim (§ Gate evidence).

---

## What it admits, and what it refuses

**Admits:** four `[R]` facts about `FUN_10b5fb5f` and one exhaustive `[R]+[O]`
exclusion, each with its address and each regenerable by
`sh work/w-sizetest/regen.sh`.

**Refuses, by name:**

* **No ceiling value is named.** `#3732` refuted 128 with 8 counterexamples in
  each direction; naming 256, 261 or 267 in its place would be the same error
  with a fresh number. The deliverable is the **exclusion**, made exhaustive
  over the parameter's whole attainable range.
* **No `crates/` adoption and no `DISCLOSURE.md` row.** Nothing here is copied
  into the port, so the provenance convention has nothing to record. Adoption
  is a later wave's.
* **No universal negative without its query set.** Every census in the lane
  prints its blind spots (`globrefs.py` names the immediate-form class it
  cannot see; `cfg.py` names indirect branches, unwind edges and caller-saved
  registers).
* **`docs/whitebox/ref/P_INLINE.md` is not edited.** It is `w-budget`'s this
  wave (`#3814`). Three corrections owed to it are recorded on this lane's own
  page with addresses, exactly as `w-instrcount` did last wave.

## The commission, and the fact that it was wrong

`WAVE21_BRIEF` §2 L4 commissioned this lane to **name a caller-supplied mask
parameter, "named nowhere in this repo"**, from `#3830`. The prereg registered
the tension before the image was opened — `P_INLINE` §6.5 (`#3717`–`#3722`,
2026-08-27) already names `edi` as the constant `0x2000` — and framed the
falsifier as a **dominance** question rather than the proximity question both
prior pages had answered.

**Result: `#3830` is wrong in three separate clauses.**

| `#3830` / brief says | measured |
|---|---|
| `edi` is **caller-supplied** | `mov edi,0x2000` at `0x10b5fc31` **dominates** `0x10b5fc95`; three `edi` writes in the body, no other; **no call site writes `edi`** |
| `edi` is **one of five parameters** | the five are `ecx`, `edx` and three stack dwords (`ret 0xc`); the **count is right** and `edi` is not in the list |
| **named nowhere in this repo** | named on `P_INLINE` §6.5 under a heading, two days earlier |

So the mask is **`__forceinline`**, and the brief's primary deliverable is a
**correction** rather than a name. This is the fourth wave running in which a
lane's most valuable output was contradicting its own brief.

**The question does have a real answer, and this lane found it anyway:** there
IS a caller-supplied mask in `FUN_10b5fb5f` — `param_4`, tested `& 0xf00` at
`0x10b5fc01`, a flat refusal when the callee's `ATTR` carries bit 4. It is `0`
at two of the three call sites; at the third it is scan-carried IL data off
tuple kind `0x17` / opcode `0x312`. **It is covered by no clause row.**
`#3830` pointed at the wrong instruction, not at a phantom.

## Estimate vs outcome

| | predicted (prereg §4) | realized |
|---|---|---|
| method | a read: one 377-byte body, three call sites, two data globals | exactly that, plus a PE section read |
| the declined alternative | a linkage × size ladder, 8+ cells per arm | **not run** — it would have re-measured numbers the tree already has and still could not attribute them, because attribution needed the dominance fact, which is a read |
| new probe cells | 0 | **0** |
| `crates/` bytes | 0 | **0** |

**Read-before-probe held, and the direction of the bias is worth recording:**
the read did not merely answer the question cheaper — **the probe could not
have answered it at all.** No obj distinguishes "the mask is a constant" from
"the mask is a parameter whose callers all pass the same value"; that
distinction exists only in the instruction stream. `WHITEBOX_LEVERAGE` §1's
doctrine is usually argued on cost; this is a case where it is a matter of
*decidability*.

**Estimate bias on the brief's second item:** the brief priced the ceiling
brackets as *"a live puzzle"*. It is not a puzzle — it is a closed enumeration,
because `DAT_10c46318` turns out to have **one reader and two writers** in the
entire image. The puzzle framing came from treating `0x10 << k` as the value
set rather than as one arm of it.

## What the size test decides, and why the answer is *nothing*

Factored with `ebx = 0` (both predecessors of `0x10b5fc69` are `xor ebx,ebx`):

```
candidate  ⟺  ( DAT_10c2e310 != 0                    # favour-speed: SIZE TEST SKIPPED
                OR count < DAT_10c46318              # under the ceiling
                OR ATTR & 0x2000                     # __forceinline
                OR (DAT_10c2e2fc != 0 AND [sym+0x80] != 0
                    AND (ATTR & 0x2 OR DAT_10c2eaac != 0) AND NOT (ATTR & 0x10)) )
              AND ( DAT_10c2e2fc != 0 OR ATTR & 0x2080 )
```

`#3830`'s quoted block elides `0x10b5fc99`–`0x10b5fcb7` behind a `...`, and the
elided chain has **four routes to `return 0`** — which is where its
*"over the ceiling still passes"* comes from.

**`DAT_10c2e310`'s image value is `1`.** No page in this tree recorded that;
c2's default state is *size test off*. And bodies at `.gl SIZE` **183, 211,
253, 260** inline at `/O1` against a ceiling of `16 << 3 = 128`, with the
over-ceiling escape closed by their own **measured** `ATTR = 0x68`
(`WB_INSTRCOUNT` §2.4). So bit 23 of the per-function option word `[fn+0x1c]`
is set and **`0x10b5fc8a` never executes on any profile this project has
measured.**

One assumption left open and named rather than buried: `DAT_10c2eaac` has 14
writers and this lane read none of them. It does not touch the mask result.

## The brackets, excluded exhaustively

`work/w-sizetest/ceiling_range.py` → `docs/whitebox/grids/w-sizetest/ceiling_range.out`:

| ladder | counts | frozen | ceilings **in** the span | ceilings that **reproduce** it |
|---|---|---|---|---|
| GRID-I STATIC | 253, 260, 267, 274 | inl, inl, called, called | 256 only | **NONE** |
| GRID-I EXTERNAL | 85, 92, 99, 106 | inl, inl, called, called | **NONE** | **NONE at any `k`** |
| D family (12 cells) | 183, 365, 855 | inl, called, called | 256, 512 | 256 (`k = 4`) |

**Consistent with all three at once: NONE.** The external exclusion is
unconditional; the static ladder is refuted by one cell (the 260-count one) and
**the two static datasets disagree with each other** under the size-test
hypothesis, which is more informative than either alone.

`WB_INSTRCOUNT` §6's careful hedge — *"the 'if' in the first line is
load-bearing"* — should be **promoted to a decided negative**: the antecedent
is false, not unverified.

## A control this lane failed, and caught before publishing

Reading `FUN_10b6242a` linearly, this lane wrote down that
`0x10b624c0`–`0x10b624dc` was dead code, on the grounds that
`xor eax,eax; test eax,eax; je` is constant-false. **It is not.** `cfg.py`'s
block enumeration shows `0x10b624c2` has two in-edges, one carrying the POGO
bit; `0x10b624c0` is a block leader with four in-edges. The corrected reading
is the better fact — a per-callee favour-speed save/override/restore, live only
under POGO. Recorded because it is the **same class as the two errors this lane
corrects**: a linear read of a listing is not a reading of control flow, and
that is precisely why the lane built a dominator tool instead of reading
harder.

## Gate evidence

Byte delta **0** and reach **0**, stated before the run and confirmed by it:
the graded-tree hash below is the base tree's.

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **62 targets · 2,014 passed · 1 failed · 2 ignored** — the one failure is `rung_index_is_generated_and_current` (below) |
| `scripts/gate.sh --jobs 16 --require-graded` | **`GATE: PASS`** — 18/18 lanes ran and every one graded a corpus; the sweep graded **19,542 of 19,638** generated cases, the cross **91,900 of 92,288** case-lane cells, **0 mismatches anywhere**; 18/18 ran again through a DEBUG-profile `c2rs` for **7,056** more fixture-verdicts at **0 panics**. Verdict read from the `GATE:` LINE, never the exit code |
| **graded tree** | **`c1eb31f530bd`** (810 files: `crates fixtures scripts`, content-hashed) — **identical at both ends of the run**, which is the byte-delta-0 evidence rather than an assertion of it |
| `c2rs selftest` | all fixtures `PASS`, **0** `SKIP: toolchain absent` lines — the worktree was verified before any work, per `WAVE21_BRIEF` §5 |
| `scripts/board_audit.sh` | 0 duplicate row numbers · 0 unresolved section anchors · 0 cited-but-absent |
| `tracked_artifact_audit.sh` ABS_FWD regex over this lane's files | **0** absolute machine paths under `work/w-sizetest/` and `docs/whitebox/grids/w-sizetest/` |
| 878-TU workload scan | not run — no `crates/` change to grade, and a scan that regrades an unchanged port is not evidence |
| fixtures, `c2rs census` | not moved — `Fixtures: none` |

**The one red, and why it is expected rather than excused:**
`rung_index_is_generated_and_current` fails because `docs/rungs/INDEX.md` is
**generated at merge** (`WAVE21_BRIEF` §4: *"will be red at every lane tip and
that is expected, not yours to fix"*). Its three sibling tests in the same
target **pass**, including
`rung_docs_claim_their_tag_slug_and_fixtures_exactly_once` — so this rung's own
header block is validated by the same file that reports the red.

**On `#3835`, which `w-gatehash` is automating this wave:** this lane's tree
moved *during* the first gate run (docs-only commits), which is exactly the
condition `#3835` says produces an authoritative-looking transcript over two
different trees. It did not here, and the reason is checkable rather than
assumed — the graded tree is hashed over `crates fixtures scripts` only, and
`gate.sh` printed **the same hash at both ends**. That comparison was made by
hand; making it automatic is `#3863`–`#3869`'s job.

## Found and not taken

Ranked; full form with addresses in `WB_SIZETEST_FINDINGS.md` §8.

1. **What actually moves the two brackets.** The size test is excluded
   exhaustively and the reason is read. The remaining candidate with a linkage
   input is `0x10b60a81` (`test DWORD PTR [edi+0x37],0x400`) in
   `FUN_10b60930`, covered by no clause row — and this lane re-derived
   `P_INLINE` §6.5's negative independently: **`0x37` appears zero times in
   `FUN_10b5fb5f`'s 377 bytes**, so no linkage field is tested in the candidacy
   function. The brackets are a ready-made grade for any reading of it.
2. **`param_4` and IL opcode `0x312`** — a caller-supplied refusal with no
   clause row. Naming `0x312` is a table lookup.
3. **`[sym+0x80]`'s identity, because this tree names it two ways** —
   `P_INLINE` §1/§6.5 *"the POGO profile record"* vs `WB_INSTRCOUNT` §3.1
   *"the function body object"* holding the 32-bit recount at `+0x8e`. Three
   pages now depend on the ambiguity.
4. **Option word `[fn+0x1c]` bits 21 and 23 mapped to `cl` switches.** Would
   turn §4.4's derivation into a direct `[R]`, and would say what turns the
   size test **on** — the configuration under which every published size claim
   in this tree becomes true.
5. **`ATTR` bit 7** — prereg P4 predicted no other reader; **falsified**, seven
   `0x2080` sites plus a bare bit-7 test at `0x10b5c9ad` immediately before a
   separate `0x2000` test. Bit 7 is a distinct, coarser mark than
   `__forceinline`.
6. **`[sym+0x94] & 0x8000`** (`0x10b7e764`) clears **both** gate globals — a
   per-function "inlining off" stamp, sibling to `WB_INSTRCOUNT` §3.1's
   `& 0x100`.

## Owed to `P_INLINE.md` next wave (this lane may not edit it)

1. **§2.1b's mechanism is refuted a second way** — the predicate its inference
   is about does not execute, so `.gl SIZE` cannot be an *upper bound on the
   compared quantity* when nothing is compared. Its **headline** stands.
2. **§4 (`?supershuffle`) cites `0x10b5fe14`**, which that page's own
   2026-08-18 correction already struck as being past `FUN_10b5fb5f`'s end.
   The measurement is untouched; the attribution is wrong twice.
3. **§6.5's `edi` conclusion is right and its argument does not support it** —
   *"nothing between the two writes it"* is an interval claim; the dominance
   fact is now printed and should replace it.
4. **§5's `min(0x10 << k, 1000)` is not a `min`** — `16 << 6 = 1024 > 1000`,
   and the arms are selected by `k ≤ 6`, not by magnitude.

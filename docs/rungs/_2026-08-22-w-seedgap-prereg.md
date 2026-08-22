# PREREG — `w-seedgap`: replace the fitted `LABEL_SEED_GAP = 9` with the read formula

Lane `w-seedgap`, branch `wt-w-seedgap`, base `9b9530791`. Written and committed
**before** any change to `crates/`. Funded by `docs/DECISIONS_2026-08-22.md`
decision 5 (the high-leverage build rule); repairs board **#3388**.

Reservations: **#3402–#3405**. Expected outcome word: `built`.

---

## 1. The defect, as filed

`crates/c2-core/src/coff/label.rs:65` ships

```rust
pub const LABEL_SEED_GAP: u32 = 9;
```

as a compilation-independent constant. Read **R3** (`docs/whitebox/ref/P_LABEL.md`
§4, `docs/whitebox/WB_LABELCHARGE_FINDINGS.md` §5, `docs/LABEL_COUNTER.md` §8.1,
board #3388) measured it over 22 cells as

```text
gap = 7 + 2·[/Og] + 1·[/GF ∧ a string literal pooled in the data phase]
```

The defect is **latent, not live**: nothing the port emits today is wrong. What
is live is the **licence** — the constant reads as compilation-independent and
every caller inherits that reading.

## 2. The 22 measured cells — what this lane's arithmetic must reproduce

From `P_LABEL.md` §4.1 / `LABEL_COUNTER.md` §8.1. Two framed functions in every
cell; only data or flags ahead of them; seed read as `u32_le(.gl[7..11])`.

| mode | base | + file-scope `const char* g = "x";` |
|---|---:|---:|
| `/Od` | **7** | **7** |
| `/Os` | **7** | **7** |
| `/Ot` | **7** | **7** |
| `/Oy` | **7** | **7** |
| `/Ob2` | **7** | **7** |
| `/Og` | **9** | **9** |
| `/Ox` | **9** | **9** |
| `/Ox /Gy` | **9** | **9** |
| `/Ox /GF` | **9** | **10** |
| `/O1` | **9** | **10** |
| `/O2` | **9** | **10** |

Plus §4.2's eleven zero-movers, all **9** at `/Ox` and all **9** at `/O1`: an
initialized global, an uninitialized global, an externally-visible const, a
64-element array, a 4 KiB `.bss` array, three globals at once, `/Gy`, `/GS` on,
`/EHsc`, `/GR`, `/Oi`, and the `/Oi /EHsc /GR` cluster.

## 3. Registered predictions

**P1 — required-zero.** `scripts/gate.sh --jobs 4` at the tip is **identical**
to the base measured on this tree at `9b9530791`:

```text
lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
graded: 6948 fixture-verdicts across all lanes
sweep:  19460 of 19556 graded, 0 mismatch
cross:  90424 of 90812 graded, 0 mismatch
debug:  18/18 lanes, 6948 verdicts, match 2423, 0 mismatch, 0 PANIC
per-lane match: O1 182 · O1-EHsc 183 · O1-Oi 184 · O1-Oi-EHsc 185 · Ox 153 ·
  Ox-EHsc 153 · Ox-Gy 153 · Ox-Gy-EHsc 153 · O2 159 · O2-EHsc 159 · Od 21 ·
  Od-EHsc 21 · O1-Oi-GR 184 · O1-Oi-EHsc-GR 185 · Ox-GR 153 · Ox-EHsc-GR 153 ·
  Od-GR 21 · Od-EHsc-GR 21   (mismatch 0 on every row)
```

Any deviation is a FAILURE of this lane, not a finding about the gate.

**P2 — the model reproduces every cell in §2**, pinned by portable unit tests
that need no toolchain. If a cell disagrees with the formula **as filed**, the
filed formula is under test too and the disagreement is reported, not forced.

**P3 — `[/Og]` is derivable from the port's own inputs and evaluates `true` on
every TU the port admits today.** The per-function optimization word is in the
IL (`c2_il::IlBundle::opt_words`), `opt_word_mode` admits exactly `/Ox`
(`0x00a00005`) and `/O1` (`0x00200005`) plus their `fp_contract`-off spellings,
and **both imply `/Og`**; `/Od` (`0x00800005`) and `#pragma optimize("",off)`
(`0x00800004`) are refused upstream. So `LABEL_SEED_GAP`'s shipped `9` is the
formula evaluated at `(og = true, pooled = false)`, and the `9` is right **only
because of two upstream refusals** that this lane will name in the type.

**P4 — the third term's second conjunct is NOT soundly decidable from the
port's inputs, and the obvious detector is an over-approximation.**
`gl_string_comdat_names` (`crates/c2-il/src/func/gl.rs:2058`) reports every
`??_C@…` record in `.gl`, but `LABEL_COUNTER.md` §8.1 records that a literal
**in a function body costs 0** while one **in the data phase costs +1** — so a
non-empty `??_C@` set does not mean the charge is taken. Registered as a
prediction: if a sound decision procedure turns out to exist in the tree, this
prediction is refuted and I say so.

**P5 — `/GF` reaches neither the port nor the IL.** It is not a field of
`IlBundle`, not a bit of the optimization word, and not a member of
`PlanInputs` (`crates/c2-core/src/plan/mod.rs:96`, which carries only
`function_level_linking`). It is an argv fact, exactly as `/Gy` is. Note that
**`/O1` and `/O2` imply `/GF`**, so its first conjunct is ON in **8 of the 18
graded lanes** — the term is the workload's own profile, not an exotic one.

## 4. What this lane will ship

1. `LABEL_SEED_GAP`'s three coefficients (`7`, `2`, `1`) named as a **read
   model**, and the two compilation facts named as **settable inputs** — per
   `docs/GOAL_DECISION_2026-08-21.md` § AMENDED, decision points are named
   settable parameters, not baked constants.
2. `LABEL_SEED_GAP` **derived** from that model at a named, cited
   configuration rather than written as a literal `9`.
3. A settable entry point that accepts any gap, so a permuter can move it.
4. Portable unit tests pinning every cell in §2 and the `[/Og]` derivation.
5. Board rows #3402–#3405, a rung doc, a `DISCLOSURE.md` row for the adopted
   read constants.

## 5. What would make me DECLINE (in whole or in part)

- **Full argv plumbing of `/GF`** — declining this is the expected outcome and
  it is priced here in advance: it requires a new field on `PlanInputs` and on
  `PortC2`, threaded through `build` into both writers, and set at four harness
  call sites (`gap/scan.rs:97`, `prefilter.rs:292`, `listing.rs:537`,
  `cli/census.rs:115`). `gap/scan.rs` is adjacent to lane `w-s0`'s surface, so
  it carries a collision risk this lane is forbidden to take unilaterally.
  **It buys zero coverage today**, because the second conjunct is refused
  upstream on every path that reaches `plan_labels`, and because no lane in
  `scripts/lanes.txt` passes `/GF` explicitly. Two-sided: the cost of NOT
  plumbing it is that the third term stays an assumption; that assumption is
  discharged here by citation to the refusals and by a test, not by silence.
- **A detector for "pooled in the data phase"** — declined if P4 holds. Shipping
  an over-approximating detector would replace a fitted constant with a fitted
  *rule*, which is strictly worse: the constant is at least visibly wrong.
- **Any change that moves a single byte.** If the required-zero identity in P1
  does not hold, the change is reverted and the lane reports `FAILED` in that
  word.

## 6. Corrections to the dispatch brief, registered up front

- The brief cites `docs/whitebox/LABEL_COUNTER.md` and
  `docs/whitebox/scripts/gt_label_seedgap.py`. Neither exists; the real paths
  are **`docs/LABEL_COUNTER.md`** and **`scripts/gt_label_seedgap.py`**.
- The brief frames the third term's condition as possibly undecidable *and*
  frames `/Od` as the exposed lane. `/Od` is indeed the exposed lane for the
  `[/Og]` term, but the brief does not note that **`/O1`/`/O2` imply `/GF`**,
  which puts the third term's first conjunct ON in 8 of the 18 graded lanes.

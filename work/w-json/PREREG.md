# w-json — PRE-REGISTRATION

Lane `w-json`, worktree branch `wt-w-json` off master **`3622e06c`**.
Target: `src/xdk/xjson/jsonwriter.cpp` — one function,
`?GetBuffer@JsonWriter@@QAAJPAGPAK@Z`, 304 `.text` bytes / **76 words**.

**Committed BEFORE the first change to `crates/` and before the first cell this
lane authored.** Everything below was measured on the base binary
(`3622e06c`, `cargo build --release` in this worktree) or read out of the
reference obj / the captured IL; nothing below is a build of this lane's code.

---

## 0. The base, measured in this worktree

`work/w-json/scan_base.out` — the full 878-TU scan at the workload's own flags,
taken before any change to `crates/`.

| | base `3622e06c` |
|---|---|
| TU match | **17** |
| mismatch | 0 |
| codegen-gap | 0 |
| vocab-gap | 854 |
| port-error | 0 |
| capture-fail | 7 |
| **FRONTIER** | **10** |
| frontier-if-A | 132 |
| factor A / B / C | 28 / 338 / 169 |
| `A∧B∧C` | 27 |
| factor D | 17 |
| `A∧B∧C∧D` | 15 |
| function census | 711,493 / 2,463,443 |
| emitted census | 39,192 / 178,977 |
| `fnbyte-exact` | 36,220 |
| `fnbyte-differs` | 2,111 |
| `fnbyte-tus-full` | 13 |
| `gap-metric` keys | **246** (`work/w-json/keys_base.txt`) |

`jsonwriter.cpp` at this base: `vocab-gap`, `0/1 functions in class`, dispatch
`disp-assign`, fall-through census key **`expr-brfalse`**, control-flow class
**`cflow-loop`**, frame class **`calls-0`**, EH class `eh-none`.

---

## 1. §1.0 — THE REFUSAL CHAIN AT THIS BASE, against the survey's "reader 16 and CLEAR"

`w-xlr` §10 put this row second-by-worth and first-by-published-price, quoting
`w-front3`'s ladder: *"reader **16, and CLEAR** — the only frontier row whose
reader price is a completed chain rather than an instrument limit."*

**That number is a READER-LADDER depth, not a conversion price, and this lane
registers before its first probe that it is not the price.** The last two
surveys' single-number prices were both wrong in the same direction
(`w-osfinfo`'s "≥ 6" was nine; `w-xlr`'s "a single mechanism" was thirteen), and
in both cases the error was that the published number counted *one seam*.
Re-derived here, function by function — there is exactly one function — against
the reference obj (`work/w-json/probe/ref.obj`, `scripts/gt_dump.py`) and the
decoded IL (`work/w-json/il/`, 1,272 bytes from the `4C 4F 11` anchor to the
`4D`):

| # | crate | refusal | size, priced now |
|---|---|---|---|
| R1 | `c2-il` | the **recognizer for the body class** — ~1,272 IL bytes, ~60 statements, 14 minted labels, three `if`/`else` arms nested two deep inside a `do`/`while` | the dominant cost. `osf_handle_guard` is 689 lines for 381 IL bytes and `xlrc_create_guard` 705 for 446; linear in IL bytes this is **≈ 2,000 lines**, 3× the largest recognizer this project has shipped in one lane |
| R2 | `c2-il` | **ONE BACK EDGE** — `39 fd 09` (`brtrue`) at the bottom of the loop targets label `fd 09`, defined ~950 bytes earlier. Every conversion lane to date registered "0 back-references" as decline clause D2 and met it | one recorded label offset + one comparison for a forward cursor; **not** a block IR. Licensed explicitly in D2 below |
| R3 | `c2-il` | the `0x54 <n>` token — 31 occurrences, in runs, at scope ends | unknown production; must be walked or skipped |
| R4 | `c2-il` | `IlFunction::callees()` needs an arm; this body has **zero** callees (`calls-0`) and two helper externals the IL never names | one arm — **pre-empted before the first census run** (§2.1) |
| R5 | `c2-il` | `label_lead` — measured **4** (§3), where `LABEL_COUNTER.md` §1.1's surcharge table predicts **2** | one arm, and a **refutation** if it holds |
| R6 | `c2-core` | a **FRAMELESS `__savegprlr_28` frame** — `mflr r12` / `bl __savegprlr_28` and **no `stwu`**; epilogue is `b __restgprlr_28` alone, **no `addi r1,r1,F`**. `FrameLayout::prologue_gpr_helper` (shipped by `w-xlr`) emits three words including the `stwu`, and `size()` would compute 128 for `saved_gprs = 4` | a third `out_of_class_ctx_*` predicate + two emitter methods |
| R7 | `c2-core` | `encode_lhzx` — X-form, primary 31 / xo 279 | **does not exist** (checked in `encode.rs`, not assumed — `w-osfinfo` #1760 and `w-xlr` §7.2 are the rule in both directions) |
| R8 | `c2-core` | `encode_sthu` — primary 45 | **does not exist** |
| R9 | `c2-core` | the **76-word emitter**, 12 blocks, one backward displacement | `codegen/xlrc_create_guard.rs` is 483 lines for 38 words; **≈ 900 lines** |
| R10 | `c2-core` | the **register assignment** — r28–r31 callee-saved, r6/r7/r8 loop-carried, r30 a loop-invariant `li 1` **hoisted above the loop**, `r10 = r4+2` feeding `r11 = r10+2` where `r4+4` would do | pinned by transcription; refused when the shape varies |
| R11 | `c2-core/coff` | the helper pair's symbols after `$T` with **ZERO other callee externals** — `.text+aux · fn · $M(end) · $M(prologue) · .pdata+aux · $T · __restgprlr_28 · __savegprlr_28`. `w-xlr` shipped `helper_externals` for the *two-callee* shape; the zero-callee shape is a **new cell** in the layer `w-xlr` found has no published price | expected 0 new code, 1 new cell — the column `w-xlr` §2 says nobody has priced |
| R12 | `c2-core` | the **parser mode gate** (#1638) — the class is `/O1` only | one clause |
| R13 | `c2-core` | `Selected` arm + `PortC2::build` dispatch | one arm |
| R14 | — | the **fixtures** — a positive `.cpp` that is 1/1 in class and whose obj is `match`, and a `_neg` with one distinct clause per cell | `w-osfinfo` #1765 took three corrections on a 31-word body. **This lane has an advantage no predecessor had: the dc3 source of the target function is readable**, so the positive fixture is a transliteration rather than a reconstruction |

**The re-price is FOURTEEN named refusals**, against the survey's "reader 16 and
CLEAR". Ten of the fourteen are outside the reader ladder entirely, and the two
that dominate the lane (R1, R9) are each larger than any single artefact this
project has shipped in one lane.

**Priced NOT-refusals** — each checked in this tree rather than assumed, in both
directions (`w-xlr` §7.2's rule):

| | status |
|---|---|
| `encode_rlwimi` (5 sites) | **EXISTS** |
| `encode_sth`, `encode_cmplw`, `encode_cmplwi`, `encode_lwz`, `encode_addi`, `encode_addis`, `encode_ori`, `encode_mr`, `encode_bc`, `encode_b_intra`, `encode_rlwinm` (the `clrlwi 11,11,25`) | **ALL EXIST** |
| `.pdata` — the record is `0x00000000 / 0x40004C02` = bit30 · FuncLen 76 · PrologLen **2** | `coff::pdata_record` produces it **unchanged** from `prolog_len = 8`, `func_len = 304`. **PAID already** |
| the four COFF-symbol-layer refusals `w-xlr` found | R11 — expected already paid by `helper_externals`, but **the zero-callee cell has never been emitted**, so it is priced as a cell, not as zero |

### 1.1 The "four COFF-symbol-layer refusals" column, checked

`w-xlr` §2 records four of its thirteen in the COFF symbol layer *"which neither
published price has a column for"*. Checked against this TU's reference symbol
table before the first probe: **this TU's group is `w-xlr`'s group minus the two
callee externals.** So the entries exist, and the price is the emptiness of the
callee region — one cell, not four refusals.

---

## 2. The unnamed refusal — budgeted explicitly (streak 5 of 7)

Five conversion lanes in a row found a reader refusal no survey had priced;
`w-osfinfo` and `w-xlr` both budgeted for one and it did **not** fire, and both
say the reason is that they read the predecessor's rung first. That is what §2.1
does.

**One unnamed refusal is budgeted.** Predicted location, in order:

1. `crates/c2-core/src/coff` — the **zero-callee helper region** (R11). This is
   where `w-xlr`'s budget pointed and where its four unpriced ones were.
2. `crates/c2-il`'s **TU-level accounting for a framed LEAF** — this function has
   a `.pdata` record, a `$M`/`$M`/`$T` triple and **no call token at all**.
   Every framed function this port has emitted had a call.
3. `IlFunction::label_slots` — the stride for a framed leaf (§3).

### 2.1 Pre-empted before the first census run, from `w-xlr` §7.1 and `w-osfinfo` §4.3

* `IlFunction::callees()` — **an arm goes in before the first census run** (this
  is `w-undname` #1743's site, and both successors pre-empted it successfully).
* the **unclaimed-`.gl`-symbol** gate — checked; this TU's `.gl` names no callee.
* **#1704's mislabeled key** — the `_neg` cells will be read *per cell* with an
  applied-and-reverted committal-dispatch probe, never from the fall-through
  key. `w-cfgclass` §6.2's method, paying a **seventh** time.

---

## 3. §3 — THE LABEL LEAD IS **4**, AND `LABEL_COUNTER.md` §1.1 PREDICTS **2**

Measured off the reference obj and the captured `.gl`, before any build:

```text
  .gl label counter  (u32 at .gl[7..11])          2578
  plan_labels seed   = 2578 + LABEL_SEED_GAP(9)   2587
  the /Gy 3-per-function pre-pass (1 function)    2590
  reference obj:  $M2594 / $M2595 / $T2596
  ⇒ lead = 2594 − 2590 = 4, exactly
```

`LABEL_COUNTER.md` §1.1's surcharge table charges this function
**framed base 5 + 2 for the first-introduced `__savegprlr_28`/`__restgprlr_28`
pair**, i.e. `extra = 2`. **The obj forces 4.** No other surcharge in that table
applies: no FP anywhere (`_fltused` +1), no pooled FP constant (+2), no signed
`>`/`<` over two call results (+2). And the b-count rule `w-xlr` refuted would
predict 3 (`+0x78`, `+0x0b0`, `+0x11c`), which is also wrong.

**Registered prediction: the lead is 4**, and both counterfactuals will be
*built and scanned against real `c2.dll`*:

| lead | source of the number | registered verdict |
|---:|---|---|
| 0 | the null control | `mismatch` |
| **2** | **`LABEL_COUNTER.md` §1.1's own surcharge table** | `mismatch` |
| 3 | #1761's refuted b-count rule | not built — already refuted by `w-xlr` |
| **4** | **the reference obj** | **`match`** |

If that holds, it is a **second refutation in two lanes**, and this one is of
§1.1's surcharge table rather than of the rule §1.1 replaced. The honest
alternative reading — registered now so it cannot be retrofitted — is that the
extra +2 belongs to **the do-while**, this being the port's first framed body
with a back edge; §1.1's "an intra-section branch costs nothing at any count" was
measured on 29 probes that are all `if`/`else` shapes. **This lane will not
propose that as a rule on one witness** (`w-xlr` §3's lesson: a rule fitted to a
population that varies over {0,1} cannot be told from a constant). It records the
number, the counterfactuals, and the gap.

---

## 4. The conversion call

| outcome | P registered |
|---|---:|
| `jsonwriter.cpp` converts (match **17 → 18**, FRONTIER **10 → 9**) | **0.40** |
| nothing converts, priced decline at match 17 | **0.60** |

The streak is five for five at 0.60 / 0.55 / 0.55 / 0.55 / 0.50. **This is
registered BELOW every one of them**, and the reasons are specific rather than
modest:

* **Size.** 76 words against a shipped maximum of 38, and ~1,272 IL bytes against
  a shipped maximum of 446. R1 and R9 together are ~2,900 lines. No lane in this
  series has shipped half that.
* **The class is the port's first body with a back edge in a `$M`-labelled
  frame**, and §3 says the label counter already disagrees with the published
  table by exactly the amount an unmodeled loop surcharge would be. A wrong lead
  is three wrong symbol records in an obj that links — `docs/GAPS.md` §6.
* **The frame class is new** (R6): frameless *and* helper-saving. `FrameLayout`
  has no representation for "saved GPRs, zero frame".
* **The `.text` byte fraction is 0.0 %, so there is no partial credit.** Per
  `w-osfinfo` §8.1 and `w-xlr` §7.1 this term has now gone five lanes without
  biting and **is registered as RETIRED, not as a risk.** It is named here only
  to record that it was deliberately not counted.
* Against all of that: the **positive fixture is a transliteration of a source
  file this lane can read**, which is the single largest cost `w-osfinfo` #1765
  reports and which this lane very likely does not pay.

---

## 5. Itemized test delta (registered as a DELTA, #1749's mitigation)

Base measured in this worktree: **`work/w-json/tests_base.out`**.
Registered delta: **+18**, itemized, with a confidence per row — because the
last two misses were both **estimation**, not arithmetic, and an unmarked
itemization cannot say which row moved.

| row | registered | confidence | why that confidence |
|---|---:|---|---|
| `codegen::json_utf8_copy` — the 76 words, prologue, epilogue, the free fields | **8** | **LOW** | this is exactly the row that missed in both predecessors (+3 in `w-osfinfo`, −3 in `w-xlr`), and in opposite directions. A test that grades two things against one obj is one test (`w-xlr` §7.3.1) |
| `codegen::frame` — the frameless helper prologue/epilogue pair | **2** | **MEDIUM** | `w-xlr` registered 3 here and shipped 0, because the class test graded them. This lane registers 2 on the ground that the *predicate* (`out_of_class_ctx_gpr_helper_leaf`) needs its own test, which is not gradable against the class obj |
| `codegen::encode` — one per new encoder (`lhzx`, `sthu`) | **2** | **HIGH** | one test per new encoder is the shipped convention and both encoders are certain to be new (R7/R8, checked in the file) |
| `c2-il` — the shape (positive) + the fences | **4** | **LOW** | the fence count is unknown until the `_neg` cells are read per cell |
| `c2-il` — the `label_lead` arm | **1** | **MEDIUM** | |
| differential | **1** | **HIGH** | one per fixture pair, every lane |
| **total** | **+18** | | |

---

## 6. Decline clauses — thresholds AND sizes

Each clause names what makes it fire, and what the lane costs if it does.

* **D1 — the block plan.** Decline if **any** of the 76 words needs a chooser —
  a scheduler or register-allocator decision the emitter must make rather than
  transcribe. Expectation: **0 chosen words**, and the immediates below as free
  fields. *If it fires*: the whole lane, decline at match 17, and the finding is
  which word and which chooser.
* **D2 — the reader, with ONE licensed back-reference.** Every predecessor
  registered "0 back-references". This class has **exactly one** (R2), and it is
  licensed here in advance: a forward cursor records the offset of label `fd 09`
  when it is defined and compares it at the `39` at the bottom. Decline if the
  recognizer needs (a) a **block IR**, (b) a **value merge at a join** whose
  result depends on which arm arrived, or (c) a **second** back edge.
  *If it fires*: decline before R9 is written; ~1 day of R1 is sunk.
* **D3 — the frameless helper frame.** Decline if it cannot be made byte-neutral
  **by construction**: `out_of_class_ctx` and `out_of_class_ctx_gpr_helper` must
  be **textually unchanged**, every shipped emitter must still run the one it ran
  before, and the new pair must be reachable only from an emitter that asks for
  it by name. *If it fires*: decline; the finding is that `FrameLayout` needs a
  frame-size representation it does not have.
* **D4 — previously-emitted objs.** Decline if **any** obj that matched at the
  base differs at the tip. Expectation **0**, evidenced by the **per-TU verdict
  SET compared by NAME at both ends** over 878 TUs (0 only-in-base, 0
  only-in-tip, exactly one verdict moved and none the other way) — a count can
  hide one lost and one gained — plus the per-lane `match` counts from
  `scripts/gate.sh`.
* **D5 — the label lead.** Decline if the measured lead is not **4**. Both
  counterfactuals (0 and **2**, the latter being §1.1's own prediction) must be
  **built and scanned red**; a lead that "works" without its counterfactuals is a
  fit. *If it fires*: the obj is three symbol records wrong and the lane declines
  with the number as the deliverable.
* **D6 — the encoders.** `encode_lhzx` and `encode_sthu` must each be pinned to a
  byte real `c2` emitted in **this** obj — `7d6b322e` (`lhzx r11,r11,r6`) and
  `b5240002` (`sthu r9,2(r4)`) — not to a manual's bit layout.
* **D7 — a refusal becoming a wrong emit.** ANY `mismatch` anywhere, on either
  scan or any gate row, at any point. #232: refusal → wrong-emit is strictly
  worse than a gap. *If it fires*: revert to the refusal and decline.
* **D8 — the symbol layer.** Decline if the zero-callee helper shape (R11) cannot
  be expressed without moving an existing symbol index in any previously-emitted
  obj.
* **D9 — the fixture.** Decline if the positive fixture cannot be made **1/1 in
  class at `/O1`** *and* `match`. A fixture that is in class but whose obj is not
  `match` is the wrong-emit direction and is D7.
* **D10 — `ptr_walk_loop`'s unpaid #1638 clause.** Registered **NOT TAKEN**, as
  in both predecessors. Still open, still behind a matched TU.
* **D11 — the PACKED layout.** Registered **REFUSED BY NAME** in advance, on
  `w-xlr` #1787's ground: every witness of the helper-pair symbol placement is a
  `/Gy` obj with a `$T` to put them after, the class is `/O1` only, and `/O1`
  implies `/Gy`. Zero match cost.

---

## 7. What this lane registers it will NOT do

* It will not propose a mechanism for §3's extra +2 on one witness.
* It will not widen `out_of_class_ctx` or `out_of_class_ctx_gpr_helper`.
* It will not claim `cflow-loop` as a class. If the TU converts it converts by
  transcription, `/O1` only, and `PORT_CFG_CLASSES` is unchanged.
* It will not adopt anything from `docs/whitebox/` without a DISCLOSURE row in
  the same commit. Seven conversion-class lanes in a row have needed zero.
* It will read no disassembly of any other frontier TU; §10 of the rung will be a
  re-survey of published prices at this tip, not a re-pricing.

---

## 8. Metric predictions (conditional on the conversion)

| metric | predicted |
|---|---:|
| TU match | **18** |
| mismatch | 0 |
| codegen-gap | 0 |
| vocab-gap | 853 |
| capture-fail | 7 |
| **FRONTIER** | **9** |
| frontier-if-A | 131 |
| factor A / B / C | 28 / 338 / 169 (unchanged) |
| `A∧B∧C` | 27 (unchanged) |
| factor D | 18 |
| `A∧B∧C∧D` | 16 |
| `A∧B∧C∧(D∨E)` | 18 |
| function census | 711,494 (+1) |
| emitted census | 39,193 (+1) |
| `fnbyte-exact` | 36,221 |
| `fnbyte-differs` | 2,111 (unchanged) |
| `fnbyte-tus-full` | 14 |
| **peer keys** | **0 vanished, 1 appeared**, 31-ish changed and every one arithmetic on the one converted function |
| per-TU set | exactly one verdict moves, zero the other way, over 878 TUs at both ends |
| gate fixture-verdicts | 5,418 + 18 × 2 = **5,454** |
| **label lead** | **4**, with 0 and 2 both built red |
| **workspace tests** | **+18** (a DELTA; §5) |

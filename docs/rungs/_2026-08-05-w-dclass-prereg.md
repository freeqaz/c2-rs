# w-dclass — PREREGISTRATION

    Tag:       w-dclass
    Slug:      w-dclass-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a prereg, not a rung. It admits no shape, moves no
               accept/refuse boundary and emits no obj byte. The rung it
               pre-registers is `_2026-08-05-w-dclass.md`.
    Census:    706555/2463393 (28.68%) at registration — unchanged, +0.
    Record:    this file. Committed BEFORE the fleet was spawned and before any
               `crates/` edit on this branch.
    Lane:      w-dclass, worktree `wt-w-dclass` off master `9f9e6c0`.
    Ships:     nothing under `crates/`.

---

## 0. The brief, and the premise I am registering doubt about

I am briefed to **widen factor D** — the per-function codegen class — because
`w-joint2` measured `match = A ∧ B ∧ C ∧ (D∨E)` with `|D∨E| = 10`, so TU match
is capped at 10 until codegen widens. The immediate target named is the
**FRONTIER 19**, and both the brief and the coordinator's mid-task message rank
it by **blocked emitted function count**, highlighting the nine TUs blocked by
exactly one function each as "the highest-density target."

**I register, before measuring, that I expect this ranking to be refuted, and
that it has already been refuted twice on the record.**

* `rungs/2026-08-04-w-cfgimpl.md` §6 item 5: *"The frontier ranking is by
  blocked-function count and that is the wrong key — **third lane in a row to
  find this**. `xboxmem.cpp` was joint-last at 4 blocked and was the cheapest
  real target; `mmio.cpp` was advertised as the best CFG target at 3 blocked of
  11 and is one of the most expensive. A frontier ranked by **distinct unmodeled
  constructs** would have said so."*
* Same rung, §6 item 2: **all five single-blocked-function frontier TUs were
  disassembled** (`osfinfo`, `undname`, `vswprnc`, `xlrcimpl`, `negate_test`)
  and *"every one of them is **framed**, with data-symbol `REFHI`/`REFLO` pairs,
  cr0 record-form branches, stack locals, `srawi`/`mulli`/`lwzx`, or
  `__savegprlr_26`."*
* `rungs/2026-08-04-w-conv.md` priced the frontier at **≥6 independent refusals
  per TU**, minimum over the (then) seventeen, and board **#269**'s standing
  decline clause — *a frontier TU at ≥4 independent refusals is not a target* —
  **fires on every member**.

A "blocked function" is a unit of **counting**; a refusal is a unit of **work**.
The two are not proportional and three lanes have now said so. The ranking the
brief hands me is the same key, one metric out, and the brief itself warns that
this exact error "cost the section ladder a lane."

## 1. Predictions — registered before the fleet was spawned

Intervals are inclusive. A prediction with no interval is scored as a hit/miss
on the stated proposition.

| # | prediction | interval |
|---|---|---|
| **R1** | Baseline reproduces master's block exactly: match 8, mismatch 0, vocab-gap 863, capture-fail 7, A/B/C/D/E = 28/338/169/8/2, `A∧B∧C` 27, FRONTIER 19 | exact |
| **R2** | The FRONTIER's blocked functions, ranked by **distinct unmodeled construct** rather than by count, produce a **different order** from the coordinator's table — specifically, at least one TU in the "1 blocked function" group prices **strictly more expensive** than at least one TU with ≥2 blocked functions | — |
| **R3** | `w-conv`'s **≥6 independent refusals** floor **holds** on re-derivation, and the minimum over the nineteen is in | [4, 9] |
| **R4** | **TU match at the end of this lane** | [8, 11] |
| **R5** | The single cheapest FRONTIER TU by distinct-construct price is **NOT** one that the blocked-function ranking puts first. My point estimate for the cheapest is `src/xdk/nuispeech/xboxheap.cpp` (1 fn, `cflow-straight`, `eh-none`, `calls-1`, 404 B IL) — the only single-function FRONTIER TU that is both straight-line and EH-free | — |
| **R6** | `expr-op-0x27`, the #1 blocker over emitted code at 23,090 (17.6%), is **NOT** a flag flip. Promoting `C2RS_SINK_OFF_ADD_ARG=expr` alone does **not** put `xboxheap.cpp`'s single function in class | — |
| **R7** | The number of FRONTIER TUs converted by this lane | [0, 3] |
| **R8** | Closing `expr-cmp-eq` **completely** (all contexts) would convert, by itself, exactly these 3 TUs: `vswprnc.cpp`, `mmio.cpp`, `IPP_basicmath_xbox.cpp` — and no others, because every other TU carrying it carries a second distinct key | exact, by name |
| **R9** | At least one subagent's first widening will be **refuted by its own constructed counterexample** before it ships. The project's record is that constructed counterexamples find what green gates do not — five live families in three days | — |
| **R10** | The census (`706555/2463393`) moves by **less** than the TU-match-relevant work suggests; any census gain this lane reports is a **driver**, not the result | — |

## 2. Declared bias

I am a **build** lane briefed after eight measurement lanes, and the brief tells
me I am "the one whose mistakes ship wrong objects." That is a bias in two
opposite directions at once and I name both:

1. **Toward building something, anything**, to avoid being a ninth measurement
   lane. This is the bias that ships a fitted rule on one witness. The counter
   is board #269's decline clause, which is **not mine to soften** — it fires at
   4 and `w-conv` measured 6.
2. **Toward declining everything**, because the prior art above makes every
   FRONTIER member look expensive and a decline is always defensible. The
   counter is R7's lower bound: I have registered that 0 conversions is a
   possible honest outcome, so I cannot claim credit for predicting it if I
   never tried.

I also note that I am **re-deriving a ranking whose refutation I have already
read**, which makes R2 a prediction I am motivated to confirm. The mitigation is
that R2 is scored on a **disassembly**, not on my reading of it, and the
per-TU construct lists are published whether or not they support R2.

## 3. Decline clauses

* **D1** — If the re-derived distinct-construct price of a candidate TU is **≥ 4
  independent unmodeled constructs**, board #269's standing clause fires and I
  decline that TU as a target, regardless of its blocked-function count. I do
  not get to re-price the clause because my brief wants a conversion.
* **D2** — If a widening's **constructed counterexample** produces a
  `Port=Mismatch` that the widening's own accept predicate does not refuse, the
  widening is **reverted**, not narrowed in place, and the revert plus its
  reasoning is committed.
* **D3** — If closing the cheapest shape reveals a next blocker such that the TU
  is still ≥4 constructs away, I ship the **partial** widening only if it is
  independently correct and gated, and report **+0 TU** honestly rather than
  quoting the census.
* **D4** — I do **not** spend the one-shot Part-1 gate. If this lane produces a
  model worth spending it on, I ask through the report and stop.
* **D5** — No subagent mints a board number. I assign from **#400–#419**.

## 4. Known-answer controls

* **KA1** — the baseline scan must read `capture-fail 7` and `cache 871 hit / 7
  miss`. A bad `--cwd` gives `capture-fail 878 / match 0` and looks ordinary.
  **Checked: it reads 7.**
* **KA2** — `df -i /tmp` is read **before** any gate, so a red instrument cannot
  be misread as my code. **Checked at lane start: 195824 / 1048576 (19%).**
* **KA3** — every count this lane reports names the **population** it is over
  (`blocked functions` vs `blocked EMITTED functions` vs `graded TUs`). Two
  lanes have been burned by mixing joins that looked interchangeable.
* **KA4** — a positive check with a printed count, never an absence. If a
  widening "works because nothing broke," that is instance 17 of the project's
  most-repeated defect and it is not a result.

## 5. Board numbers reserved

**#400–#419.** Nothing outside that range is minted by this lane or its fleet.

# w-fencecount — **NO WORKLOAD TU IS HELD OUT OF `match` BY A SINGLE FENCE**: the two-sided pricing directive has nothing to price on the 878, the counter that says so is standing, and the control the brief mandated was already PAID

    Tag:       w-fencecount
    Slug:      fencecount
    Date:      2026-08-13
    Kind:      fixture-claim
    Outcome:   instrument
    Fixtures:  wfcnt_fence_holds_exact.cpp
    Census:    per-function 705,259/2,410,886 → 705,259/2,410,886 (29.25 % →
               29.25 %); emitted 39,181/162,049 → 39,181/162,049. **+0** — this
               lane changes no acceptance predicate and no emitter; the fixture
               is a control the counter reads, not a class the port admits.
    Record:    this file. PREREG `work/w-fencecount/PREREG.md`, frozen at
               `3106c988` **before the first scan, the first probe compile and
               the first `crates/` change**. The base-vs-tip key diff is
               `work/w-fencecount/keydiff.{py,txt}`; the drift finding is
               `work/w-fencecount/DRIFT.md`; the cell this lane measured and did
               **not** ship is `work/w-fencecount/KEPT_CALL_CANDIDATE.md`.
    Lane:      w-fencecount, worktree branch `wt-w-fencecount` off master
               **`8fbe6ef5`**. Master did not move while the lane ran, so the
               gate of record is taken at this tip and no rebase was needed
               (`fcb9afe9`, the amended `_TEMPLATE.md`, is an ancestor).
    Ships:     `GapReport::fence_blocks` + `FenceBlocks`/`FenceCauseRow` +
               `FENCE_CAUSES` (`crates/c2-harness/src/gap/`), a printed
               FENCE-BLOCKS-EXACT block, **93** new `gap-metric fence-*` keys,
               one fixture, one integration target (`tests/fence_count.rs`),
               **+5** `#[test]`. Board rows **#3062**–**#3066**.
    Adopts:    **nothing.** No `docs/DISCLOSURE.md` row, no whitebox constant,
               no bracket, no flag bit. §7.

---

## 1. The result

> ### **`sole` IS 0 AND `exact` IS 0 ACROSS ALL 23 CAUSES, AND THAT IS THE FINDING.** No TU in the 878-TU workload is held out of `match` by a single fence: **845 held TUs carry 1,716 cause firings**, so every one of them is blocked by two or more clauses at once. **CLAUDE.md's two-sided fence-pricing directive, as of today, has nothing on this workload to price.** A fence's own cost is the TUs it alone holds; there are none. Board **#3062**.

> ### **AND THE CONTROL THE BRIEF MADE MANDATORY WAS ALREADY PAID.** The commission required the counter to fire on `src/xdk/LIBCMT/vsnprnc.cpp` — *"if your counter does not fire on this TU, the instrument is wrong"* — and that TU is **`match` at base**: `w-fence2` narrowed the inline fence and converted it on 2026-08-09 (board #2470), five merges before this lane. The brief's two mandates (*"the 25 matching TUs must show fence-blocks-exact 0"* **and** *"the vsnprnc control must fire"*) are jointly unsatisfiable. **Registered as PREREG D1 at p = 0.90 before the first scan ran**, then confirmed by the scan. Board **#3063**.

> ### **SO THE CONTROL IS A FIXTURE, AND IT IS A REAL TWO-SIDED PRICE.** `fixtures/cpp/wfcnt_fence_holds_exact.cpp`: a `static` callee (linkage `03`, so `w-fence2`'s plain-external exemption does not reach it) and its caller. Sole cause `locally-defined-callee`; **`fnbyte-exact 2` of `fnbyte-denominator 2`** — every emitted body byte-exact against real `c2`, and no obj emitted. The counter reads `sole 1 · exact 1 · bodies 2 · first 0`. Board **#3064**.

> ### **`decode_causes` DOES NOT AGREE WITH THE GATE, AND ITS OWN DOCUMENTED INVARIANT IS FALSE TODAY.** On `fixtures/cpp/wfence2_kept_local_callee.cpp` the bundle **decodes** (`functions()` is `Some`, the TU is a whole-obj `match`) while `decode_causes` reports **two** firing causes — `unclaimed-gl-symbol` and `locally-defined-callee`. `diag.rs`'s module doc states the contract as *"the invariant `causes.is_empty() == decodes`"*. It does not hold. Latent, deliberately unrepaired. Board **#3065**.

> ### **NOTHING CAN BE OVER c2's STATIC INLINE CEILING AND INSIDE THE PORT'S CLASS AT THE SAME TIME.** Two fixture drafts tried to make c2 *keep* a big static call: a >308-byte multiply chain and an 84-term add chain. Both add `body-out-of-class`, because the port's largest lowered body anywhere is **152 bytes** (`guard_chain_shared_tail`, GRID-W's port side) and c2's static inline ceiling is `(300,308]` (F1). The two windows do not overlap. Board **#3066**.

| | base `8fbe6ef5` | tip | Δ |
|---|--:|--:|--:|
| **TU match** | 25 | **25** | **0** |
| **mismatch** · codegen-gap · port-error | 0 · 0 · 0 | **0 · 0 · 0** | **0** |
| vocab-gap · capture-fail | 845 · 8 | **845 · 8** | **0** |
| per-function census | 705,259 | **705,259** | **+0** |
| emitted census | 39,181 | **39,181** | **+0** |
| `fnbyte-exact` / `-denominator` | 35,734 / 162,049 | **35,734 / 162,049** | **0** |
| `progress-mass` · `frontier` | 0.21016 · 2 | **0.21016 · 2** | **0** |
| **pre-existing `gap-metric` keys** | **277** | **277 compared, 0 differ** | **0** |
| new keys | — | **93, all `fence-*`** | **+93** |
| per-TU verdicts, **by name** over 878 TUs | — | **0 changed, 0 only-in-base, 0 only-in-tip** | **0** |
| workspace tests / targets | 1,527 / 41 | **1,532 / 42** | **+5 / +1** |
| `#[test]` (`git grep -c`) | 1,539 | **1,544** | **+5** |
| fixtures | 380 | **381** | **+1** |

---

## 2. What it admits, and what it refuses

**It admits nothing.** No acceptance predicate, no emitter and no gate changes;
`git diff master..HEAD -- crates/c2-il crates/c2-core` is **empty**. The
deliverable is a measurement, which is why `Kind: fixture-claim` (the lane does
claim a fixture prefix, `wfcnt`) and `Outcome: instrument` are different fields.

### 2.1 The counter, defined

For each named decode-gate cause, over the **graded** TUs:

| key | reading |
|---|---|
| `fence-blocks-sole:<cause>` | TUs whose entire `decode_causes()` set is exactly `{cause}` — *this fence and nothing else the diagnostic models* |
| `fence-blocks-exact:<cause>` | of those, TUs where **every** emitted body is FnByte-exact: per-TU `fnbyte-exact == fnbyte-denominator`, **denominator > 0** |
| `fence-blocks-exact-bodies:<cause>` | the byte-exact bodies inside those TUs — the arity companion, in body units |
| `fence-blocks-first:<cause>` | TUs where the cause is the FIRST blocker of a multi-cause set |

**Three refusals, each stated where the number is read rather than only here:**

* **`first` is NOT a distance.** The port stops at its first refusal, so every
  blocked TU names exactly one blocker however many it has (the coordinator's
  standing rule; `w-mixed`'s ladder). Printed in the block's own header.
* **`sole` is bounded by the diagnostic's vocabulary.** A clause
  `decode_causes` does not re-ask cannot appear, so `sole` is an upper bound on
  *"this fence and nothing else"*, never an oracle-graded distance.
* **`exact` is not a conversion forecast.** A whole-obj verdict is a
  conjunction over the emit set, the sections and both tables; per-function
  exactness is a statement about bodies (`FUNCTION_BYTE_MATCH.md` §7). The
  fixture header says this in the file a future lane will read first.

### 2.2 It reads existing seams and builds no second reader

Attribution is the scan's own `gate_cause`/`gate_causes` (from
`IlBundle::decode_causes`, the one existing re-ask of the gate's predicates);
exactness is the same `fnbyte-exact`/`fnbyte-denominator` pair
`GapReport::fn_byte_by_tu` already reads. **No fence predicate is re-implemented
here** — duplicate readers of one fact are the merge failure textual conflict
detection structurally cannot see, and this instrument was briefed against
exactly that.

`FnByte::Exact` is **read and never narrowed**, per the standing rule from the
lane that nearly erased an 861-function finding by redefining it.

---

## 3. THE READING, AND WHY THE ZERO IS THE FINDING

```text
held 845 · cause-firings 1,716 · residue-no-cause 0 · accounting-broken 0
arity-broken 0 · class-disagree 0 · on-match-tu 0 · match-tus-checked 25
decodes-not-match 0
```

**Every `sole` cell and every `exact` cell is 0.** The only non-zero rows are
first-blockers:

| cause | `first-of-multi` |
|---|--:|
| `gl-stop-26-introduced` | **819** |
| `drectve-not-boilerplate` | 13 |
| `bind-record-count-ne-segments` | 11 |
| `body-out-of-class` | 2 |
| **`locally-defined-callee`** | **0** |

`1,716 / 845 = 2.03` causes per held TU. The inline fence — the fence the brief
named, and the one every recent conversion lane has hit last — appears on
**one** TU in the whole workload (`src/keygen_xbox.cpp`, three causes, first
blocker `body-out-of-class`).

> **The directive that commissioned this instrument cannot be exercised on this
> workload today.** *"Every new fence is priced two-sided, in the units the goal
> is written in, before it ships"* (CLAUDE.md, from #1042 and NC-5/#2691) needs a
> population of TUs a fence alone holds. That population is **empty**. This is
> not a null result about the instrument; it is a measured negative about the
> lever, and it says the next fence's price must be argued from a *constructed*
> cell — which is what the fixture is — rather than read off the workload.

**What the zero is not.** It is not evidence that fences are free: a fence that
is second-in-line on a TU blocked by four things costs nothing *today* and
costs a conversion the moment the other three are paid. `w-vsnprnc` is exactly
that history — the fence was invisible until the codegen was paid, and then it
was the whole remaining price. The counter is standing so that the next time a
TU's other blockers close, the fence's cost appears as a number on the same scan
instead of being discovered by a lane that has already spent its budget.

---

## 4. THE CONTROL, AND THE MANDATE IT REPLACES

### 4.1 Why the workload cannot be the control

`fence-blocks-exact` has one historical positive: `vsnprnc.cpp`, both functions
byte-exact with the TU still `vocab-gap` (`w-vsnprnc` §1). **`w-fence2` paid
it.** So the shape's only workload instance is gone, and a counter whose only
positive reading has been paid off is indistinguishable from one that is not
wired up — this repo's most-recorded defect (`STATUS.md` trap 5, ~15 instances
across 8 instruments).

### 4.2 The fixture, and the three drafts that failed first

`fixtures/cpp/wfcnt_fence_holds_exact.cpp` — a `static` callee and its caller.

| draft | what it tried | why it failed |
|---|---|---|
| **1** | `static` callee, 36-term multiply chain (far over F1's `(300,308]`, so c2 keeps the call) | `body-out-of-class` — `expr-op-0x0F`; the port does not lower a multiply chain. Two causes, not sole |
| **2** | `static` callee, 84-term alternating add chain | same key, same reason: the port's largest lowered body anywhere is **152 B** |
| **3** | `static` int-returning **recursive** callee (F5: c2 never inlines recursion, so size is irrelevant) | sole cause held, but `fnbyte-differs` on the callee — `S3-tail-setup`; the splice declines and the port's two words are not c2's one |
| **shipped** | `static` **small** callee; c2 **inlines** it and the port's mechanism-I splice reproduces the result | sole cause `locally-defined-callee`; `fnbyte-exact 2 / 2`; `fnbyte-spliced-exact 1` |

**The shipped cell is the INLINED mechanism, not the KEPT one**, and saying so
is the point: `vsnprnc`'s own shape was a *kept* call. §8 sizes the kept-call
cell this lane measured and did not ship.

### 4.3 What the integration test asserts, in premise order

`crates/c2-harness/tests/fence_count.rs` drives the **real** pipeline (capture
with real `c2.dll` under wibo → port → byte judge), with a distinct failure
message per assertion and every premise asserted **before** the reading it
supports, so no early guard can make a later assertion unreachable
(`ROADMAP.md` §9.18.8):

1. the run **graded exactly 1 TU and it is the fixture by name** — positive on
   content, never an enumeration of ways a run can be empty;
2. class `vocab-gap`;
3. causes **exactly** `[locally-defined-callee]`;
4. `fnbyte-denominator == 2` and `fnbyte-exact == denominator` — which is *also*
   the check that c2's behaviour has not moved under the cell: if c2 stopped
   inlining this callee, or the splice stopped reproducing it, the count breaks
   rather than the claim going unchecked;
5. `sole 1 / exact 1 / bodies 2 / first 0`, plus every known-answer-0 control;
6. the three published `gap-metric` keys carry the same numbers — a reading that
   exists only inside a struct is not the instrument.

`SKIP: toolchain absent` degrades cleanly; it never panics.

---

## 5. GRADED ON ITS OWN INVARIANTS — the oracle cannot grade a correspondence

The compiler judges obj bytes; it cannot say whether TU *T* is held by fence
*F*. So the instrument is graded the way this repo grades bindings: **totality
with a named printed residue, plus an arity check, plus agreement where the
answer is independently known.**

| invariant | key | reading |
|---|---|--:|
| **totality** — every held TU is attributed to exactly one row or counted broken | `fence-accounting-broken` | **0** (845 = 845 + 0) |
| **arity** — causes summed over held TUs; entities vs their contents | `fence-cause-firings` vs `fence-held-tus` | **1,716 / 845** |
| **residue** — `vocab-gap` with an EMPTY cause list | `fence-residue-no-cause` | **0** |
| **named residue outside the family** — decodes and still not `match` | `fence-decodes-not-match` | **0** |
| …of those, carrying a cause anyway | `fence-class-disagree` | **0** |
| **agreement, known answer** — a `match` TU carrying a cause | `fence-on-match-tu` | **0** |
| …stated positively over its population | `fence-match-tus-checked` | **25** |
| **cross-field arity** — first blocker present and a member of its own list | `fence-arity-broken` | **0** |

**Totality alone was not accepted as the grade** — the dropped-`DUP` lesson:
residue counts entities, arity counts their contents, and a bug that drops
contents without dropping entities leaves totality silent. `exact_tus` and
`exact_bodies` are the same discipline applied to the headline itself.

### 5.1 Every guard was watched RED once

A guard nobody has seen fire is untested. Five mutations, five distinct
messages, each reverted:

| # | mutation | the message that fired |
|---|---|---|
| 1 | drop the `d > 0` vacuity guard | *"only the all-exact TU may count as fence-blocks-exact: the bodiless TU is vacuous…"* |
| 2 | `[only]` → `[only, ..]` (count a multi-cause TU as sole) | *"the first blocker (what functions() stops on) takes the first-of-multi row"* |
| 3 | `first_is_member = true` (skip the cross-field check) | the `fence_controls…` cell, on the totality identity |
| 4 | start the key list empty (drop the closed-vocabulary zeros) | *"a cause that never fires still prints, at zero — absence must not read as success"* |
| 5 | control's expected `sole` → 2 | *"the inline fence must be counted as the SOLE blocker of one TU; got 1 — the whole per-cause map: {…}"* |

**Mutation 5 also proved the control is not silently skipping**: its message
printed real captured data, so the 0.07 s runtime is a fast capture and not an
absent toolchain.

---

## 6. NEUTRALITY — additive, proven as a map and by name

`work/w-fencecount/keydiff.py`, base binary against tip binary, same list, same
flags, same cwd:

```text
metric keys: 277 in base, 370 in tip
277 keys compared (every key BASE printed), 0 differ
93 new keys — families: fence-* (93); 7 nonzero, 86 zero
per-TU verdicts: only-in-base 0, only-in-tip 0, CHANGED 0
class counts: match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8, both ends
```

**A set comparison, not a count.** The seven non-zero new keys are
`fence-held-tus`, `fence-cause-firings`, `fence-match-tus-checked` and the four
`fence-blocks-first:*` rows; every `sole`/`exact` key is 0 at both ends.

---

## 7. Estimate vs outcome — the PREREG scored

| # | registered | p | outcome |
|---|---|--:|---|
| **D1** | `vsnprnc.cpp` is `match` at base — **the brief's control premise does not hold** | 0.90 | **HIT** |
| **D2** | `fence-blocks-exact:locally-defined-callee` = **0** on the workload | 0.75 | **HIT** |
| D2a | …≥ 1 | 0.15 | did not fire |
| **D3** | `fence-blocks-sole:locally-defined-callee` = 0 | 0.55 | **HIT** |
| **D4** | `fence-blocks-first:locally-defined-callee` ≥ 1 | 0.50 | **MISS — it is 0**; the one TU carrying the cause has `body-out-of-class` first |
| **D5** | the largest sole-cause key is a BINDING-family cause | 0.65 | **VOID** — there are no sole-cause keys at all; scored as a miss, §7.1 |
| **D6** | residue / accounting / arity controls all 0 at both ends | 0.90 | **HIT** |
| **D7** | match 0 · mismatch 0 · census +0 · fnbyte-exact 0 · every pre-existing key identical | 0.90 | **HIT**, 277 keys and 878 verdicts |
| **D8** | the control fixture isolates and fires by name | 0.65 | **HIT**, at the fourth draft |
| D8a | c2 keeps a >308 B static callee's call at `/O1` | 0.75 | **UNTESTED as shipped** — the route was abandoned at draft 2 (§4.2); a sub-lane's `noinline` cell reached the same reading by another mechanism (§8) |
| **D9** | tests in `[1,531, 1,539]`, targets **42** | 0.60 | **HIT** — 1,532 / 42 |
| **D10** | `#[test]` delta in `[+4, +10]` | 0.65 | **HIT** — +5 (1,539 → 1,544) |
| **D11** | the diag/gate drift is demonstrated live | 0.70 | **HIT, and harder than registered** — two causes, one unpredicted |
| **D12** | ≥ 1 unnamed refusal fires at a pre-armed place | 0.60 | **HIT** — §7.2 |
| **D13** | mismatch 0 everywhere | 0.97 | **HIT** |

**11 hits, 2 misses, 1 void, 1 untested.**

### 7.1 The lane's own bias, and the one prediction that mattered

Both misses are the same error in the same direction: **D4 and D5 both assume
some cause somewhere is a TU's whole story**, and none is. The PREREG predicted
the *inline fence's* row would be zero (D2, D3 — correct) and then predicted
non-zero rows elsewhere, which is the shape of assuming the instrument will
find *something*. It found a structural zero, and the structural zero is worth
more than any row would have been.

**D1 is the prediction that carried the lane.** The brief made a control
mandatory that had been paid off five merges earlier; registering it as stale
*in advance, at p = 0.90, with the evidence cited from `git log`* is what turned
an unsatisfiable mandate into a real fixture instead of into a weakened
assertion. The alternative — adjusting the counter until the workload produced
the reading the brief expected — is the failure this repo has recorded most
often.

### 7.2 The unnamed-refusal budget — ONE budgeted, ONE spent, at a pre-armed place

**Board #1380's shape fired.** Reverting mutation 1 with `git checkout --` ate
the then-uncommitted counter implementation, because the implementation and the
mutation were in the same file and only the mutation was supposed to go.
Pre-armed in PREREG D12 (*"commit first, then apply"*) and spent anyway: the
work was re-applied from context and **committed before any later mutation**, so
mutations 2–5 ran against a committed tree and cost nothing. The general form
for the next lane: **a mutation is an edit to a file you may not have
committed** — the revert command cannot tell your work from your mutation.

---

## 8. Found and not taken

1. **The KEPT-call mechanism has no cell** (sized: one fixture, one afternoon).
   The shipped control exercises the *inlined* path — c2 inlines, the port
   splices. `vsnprnc`'s own shape was a **kept** call, and the two reach the
   counter through different port machinery (`splice` vs the emitted `bl` plus
   `comdat::fenced_inlined_callee`). A sub-lane measured a working kept-call
   cell — `__declspec(noinline)` on a small static callee, which clears the
   `.gl` attribute bit `0x40` that `w-mmioclose` shipped, so c2 keeps the call —
   and it reaches the same `sole 1 / exact 1 / bodies 2` reading by the other
   mechanism. **Deliberately not shipped**: it would adopt a second linkage-byte
   fact into a control whose job is to be boring, and one control that fires is
   the requirement. Recorded with its measurements in
   `work/w-fencecount/KEPT_CALL_CANDIDATE.md` so the next lane starts from a
   cell that ran rather than from an idea.
2. **The `decode_causes` drift is unrepaired and is now written down** (§1,
   board #3065, `work/w-fencecount/DRIFT.md`). Repairing it means re-pointing a
   shared surface with callers in three test targets and one scan field — the
   shared-predicate erasure this repo has recorded three times — so this lane
   carries the caveat at the point of reading instead. **The second cause
   (`unclaimed-gl-symbol`) was not predicted and nobody has looked at it.**
3. **`gl-stop-26-introduced` is 819 of 845 first-blockers**, which is the same
   ranking every other instrument gives and is restated here only because it is
   now derivable from a key rather than from prose.
4. **The counter cannot see a fence the diagnostic does not model** (§2.1). The
   `w-mmio3` sibling clauses inside `IlBundle::functions` — the `close_call_chain`
   elision and the r5 park — have no cause string, so a TU they hold reads as
   held by whatever else fires. Sizing that residue needs a cause per clause,
   which is a `c2-il` change and not an instrument one.

---

## 9. Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release --no-fail-fast` | **1,532 passed, 0 failed, 42 targets** (base **1,527 / 41** — the target count ROSE by the new integration target, which is what says none was silently dropped) |
| `#[test]` delta, `git grep -c` at both revs | **+5** (1,539 → 1,544) |
| `cargo test -p c2-harness --release --test fence_count` | **1 passed** (the control; mutation-verified §5.1) |
| `cargo test -p c2-harness --release --test rung_registry` | §9.1 |
| `scripts/board_audit.sh` | §9.1 |
| `scripts/gate.sh --jobs 4` | §9.1 |
| 878-TU workload scan, base **and** tip | match **25** · mismatch **0** · codegen-gap **0** · vocab-gap **845** · capture-fail **8**, both ends; 277 pre-existing keys identical, 878 verdicts identical (§6) |

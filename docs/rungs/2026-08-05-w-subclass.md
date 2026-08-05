# w-subclass — the CFG screen learns to hold a PARTIAL claim

    Tag:       w-subclass
    Slug:      w-subclass
    Date:      2026-08-05
    Fixtures:  none — an instrument change with no emit to grade (§8)
    Census:    711,427 / 2,463,393 unchanged (28.88 %), +0 — this lane ships no codegen
    Record:    this file

**Lane:** `w-subclass` · **Board:** closes **#778**, opens **#806**–**#810**
**Base:** master `37858fd` · **Prereg:** `work/w-subclass/PREREG.md`, committed
at `05aa296` before any measurement on this tree.

**What this lane is:** an instrument change and nothing else. **No `crates/`
emit path was touched, no `Selected` variant was added, and `cflow-loop` did NOT
enter `PORT_CFG_CLASSES`.** `fnbyte-differs` is 0 before and after.

---

## 1. The result, in one box

| | |
|---|---|
| **The mechanism** | `PORT_CFG_CLASSES` is `&[CfgClass { class, sub: Whole \| Keys(&[…]) }]` instead of `&[&str]`. `Whole` is exactly what a bare string meant |
| **Narrower or equal — the algebra** | `admits` is `class == class && <sub>`; `Keys` only ever **conjoins**, so `admits(Keys) ⟹ admits(Whole)` for the same class, for every input |
| **Narrower or equal — the MEASUREMENT** | **`⊥ 0 ⊆ ENUMERATED 2 == SHIPPED 2 ⊆ ⊤ 14`, of 17 frontier TUs**, re-derived from the live 878-TU workload on every scan, compared as **sets by name**. 0 violations over 3 checks |
| **Identity at the unrestricted end** | The 878-TU scan's CFG screen is **byte-identical** before and after — all 17 rows, the legend and the control — and **all 53 pre-existing `gap-metric` values are unchanged**, with 9 keys added |
| **Is the census key a fine enough handle?** | **Yes, and by a wide margin: 240 distinct `cflow-loop\|<key>` keys on the workload**, one of which is **`ptr-walk-mod-loop`** — the census's own label for the single loop the port emits |
| **Must-fail M1** (matcher ignores its key) | **CAUGHT.** 6 unit tests fail; on the live workload `cfg-reach-bottom` 0 → **2**, `cfg-bounds-violations` 0 → **1**, and the screen prints `NESTING: **FAIL**` naming both TUs |
| **Must-fail M2** (exact → `starts_with`) | **CAUGHT**, and it **found a hole in this lane's own instrument** — see §5 |
| **Workspace tests** | **888 passed, 0 failed, 27 targets** (9 tests added) |

**#778 is closed as a MECHANISM only.** Whether anything goes into a restricted
entry is a loop lane's decision, on a loop lane's evidence. §6 is the recipe.

---

## 2. What #778 actually was

`w-rotate` §7 and `w-sched2` §8 both measured real, honest, **partial** coverage
of `cflow-loop` and both declined to record it. The claim `w-sched2` could
support was:

> *"`cflow-loop`, restricted to the sentinel walk at `/O1`, pointer formal at
> slot 0, chains of single-word producers with no hoisted literal"*

A flat `&[&str]` matched against the bare census class string can hold only
`"cflow-loop"`, which is the wholesale claim, which is false. Both lanes were
right to refuse, and both said the refusal does not scale.

**The screen licenses no emit** — confirmed before touching it, three ways, and
this is the property everything else rests on:

1. **Exactly two readers before the change** — `cfg_reach` (`factors.rs:595`)
   and `cfg_reach_control` (`factors.rs:630`), both `git show 05aa296`. Both are
   pure over `results`; neither reads an obj.
2. Their callers are `render.rs` (printing) and `metrics()` (printing).
   **`grep -rn 'PORT_CFG_CLASSES\|CfgClass' crates/c2-core/ crates/c2-il/`
   returns nothing** — the accept/refuse boundary (`codegen::select_function`,
   `codegen::function_gate`, `IlBundle::functions`) does not mention it, and
   nothing in `c2-harness` feeds it back into a `Backend`. The check is a
   printed empty result, not an impression.
3. The empirical check: **`fnbyte-differs` is 0 on the before and after scans,
   and every other emit-facing metric is byte-identical** (§3). A change that
   had reached an emit path could not leave all 53 unchanged.

So a wrong answer here mis-*reports*, and the failure mode to design against is
a predicate **more permissive** than the flat list — a lane reporting coverage
it does not have.

---

## 3. IDENTITY — the unrestricted end, measured at both ends

Registered as **R1**, the prediction most able to lose: if the rewrite moved the
verdict on even one TU, the mechanism would be *different* rather than
narrower-or-equal, and the change would be unsound.

Baseline at `05aa296` (pre-change), final at `828f133`, same workload, same
capture cache:

```
$ diff <baseline gap-metrics> <final gap-metrics>
22a23,31
>     gap-metric cfg-reach-bottom 0
>     gap-metric cfg-reach-enumerated 2
>     gap-metric cfg-reach-shipped 2
>     gap-metric cfg-reach-top 14
>     gap-metric cfg-bounds-violations 0
>     gap-metric cfg-subclass-entries 4
>     gap-metric cfg-subclass-restricted 0
>     gap-metric cfg-subclass-unwitnessed 0
>     gap-metric cfg-subclass-intruders 0
```

**Nine keys added; not one of the 53 existing values moved.** Among them
`match 10`, `mismatch 0`, `frontier 17`, `factor-c 169`, `b-and-c 151`,
`fnbyte-differs 0`.

And the screen itself, all 17 rows plus legend plus control, diffed as text:

```
$ diff b_screen.txt a_screen.txt
IDENTICAL: all 17 frontier rows + legend + control unchanged
```

**By name, not by count** — `SHIPPED reachable: [src/Main.cpp,
src/xdk/nuispeech/xboxheap.cpp]`, which is exactly the two rows that read
`REACHABLE` in the baseline. A count of 2 on both sides would have been
satisfied by swapping one TU for another; the names are what rule that out.

---

## 4. THE BRACKET — how "narrower or equal" is measured rather than argued

The algebra is a one-liner and this project does not accept algebra as evidence
about an instrument. `GapReport::cfg_reach_bounds` runs `cfg_reach_with` against
four lists over the same `results`, every scan:

| bound | list | live value | what it is for |
|---|---|---:|---|
| `⊥` | every shipped entry rewritten `Keys(&[])` | **0** | must be empty — the live exercise of the `Keys` path, and M1's detector |
| `ENUMERATED` | every entry rewritten as the exact key set this scan observed | **2** | must equal `SHIPPED` — `Keys` and `Whole` agreeing where they are built to |
| `SHIPPED` | `PORT_CFG_CLASSES` | **2** | today's answer, the only one anyone acts on |
| `⊤` | every class the frontier mentions, wholesale | **14** | a hypothetical the port has no claim to — the honest size of the refusal |

`0 ⊆ 2 == 2 ⊆ 14, of 17 frontier TUs (7 classes in ⊤, 17 (class,key) pairs
enumerated)`, **0 violations over 3 checks, taken as sets by name.**

**R3 registered `⊤ − SHIPPED ≥ 8` and it is 12** — registered because an inert
`⊤` would make the nesting vacuously true and demonstrate nothing. 12 of the 17
frontier TUs are held back by CFG class alone.

**`⊥` is not decoration.** Without it, `CfgSub::Keys` would be a code path no
run reaches, which this project rates worse than an absent one (`w-rotate` §7.2,
`w-frame` row F-c). `ENUMERATED` exercises it harder still — 17 `(class, key)`
pairs on the frontier, and the ledger shows the shipped classes carry
**455 / 36 / 192 / 2** distinct keys across the whole workload.

### 4.1 The ledger

One row per entry, so a partial claim is auditable against the workload meant to
justify it. Two ways a restriction goes quietly wrong, both counted:

* **A listed key no scan witnesses** — a claim doing nothing, trap 5 with the
  claim still on the page. Reported as `unwitnessed <n>`.
* **The matcher and the declaration disagreeing** — `admitted` recomputed by
  asking `admits` about every observed key, `declared` by literal membership.
  Reported as `intruders <n>`.

On this tree all four rows read `cross-check n/a (whole class — no declaration
to compare)`. **`n/a` and never `PASS`**: printing a pass for a check nobody
took is the absence-read-as-success this row exists to forbid.

---

## 5. THE MUTATIONS — and M2 found a hole in this lane's own instrument

### 5.1 M1 — `CfgSub::Keys(_) => true`, the matcher ignoring its key

The wrongly-permissive mutation in its purest form.

* **6 unit tests fail.**
* **On the real 878-TU workload:** `cfg-reach-bottom` **0 → 2**,
  `cfg-bounds-violations` **0 → 1**, and the screen prints

  > `NESTING: **FAIL** — BOTTOM is not empty (2 TUs: src/Main.cpp,
  > src/xdk/nuispeech/xboxheap.cpp) — a list admitting no census key reached
  > something, so the matcher is ignoring its key argument`

Note what M1 does *not* move: `cfg-reach-shipped` stays 2, `frontier` stays 17,
every other metric is unchanged. A gate watching only the headline figure would
have passed it. The `⊥` bound is the only thing that sees it.

### 5.2 M2 — exact → `starts_with`, and the finding

Census keys nest densely: `expr-cmp-eq` is a strict prefix of
`expr-cmp-eq-and-branch-more`, and **both are live `cflow-loop` keys on this
workload** (734 and 7 functions). A prefix restriction grows silently every time
the census mints a neighbour.

M2 failed the prefix-witness unit test, naming the intruder. **It did not fail
the ledger's intruder cross-check — the check that exists precisely to catch a
matcher admitting beyond its declaration.**

The reason is structural: **no shipped entry is restricted**, so on the live
workload the cross-check reports `n/a` on all four rows and grades nothing. An
ungraded code path by construction, in the instrument this lane built, found by
the calibration the brief required. Fixed at `828f133`:
`cfg_subclass_ledger_with(list)` mirrors `cfg_reach_with`, and a test builds a
restricted entry over three live `cflow-loop` keys where two extend the third.
Exact matching admits **1 of 3** and the cross-check is `Some([])`;
`starts_with` admits **3 of 3** and it names the two undeclared keys.

**M2 now fails two tests instead of one**, and the second is the cross-check.
This is the argument for running the mutation on a measuring device rather than
reasoning about it: the hole was in the grader, not the thing graded.

---

## 6. WHAT A LOOP LANE MUST NOW SUPPLY

Four things, in order. The mechanism supplies the fourth; the first three are
the lane's.

1. **A census key that distinguishes the sub-class.** The restriction is
   expressed over `(cflow class, census key)` — the only pair the screen has —
   so a restriction finer than any census key can distinguish is **not
   expressible on the screen side**, and the honest price is a census-side key
   mint in `c2-il` first. **Registered as R5, and the workload answers it
   decisively:** `cflow-loop` carries **240 distinct keys**, and one of them is
   **`ptr-walk-mod-loop`** — the census's own in-class label for
   `codegen::ptr_walk_loop`, the single loop the port emits. The handle exists.
2. **Evidence graded by real `c2` under wibo** that the port's emit is
   byte-exact over exactly the bodies those keys name, and no others.
3. **A witness that the restriction is not inert** — the ledger prints
   `unwitnessed` per entry, and a listed key with no cases on the workload is a
   claim no run can grade.
4. **The bracket**, free: `cfg_reach_with` and `cfg_subclass_ledger_with` let a
   lane price a candidate restriction against the real workload **before**
   proposing it — the move neither `w-rotate` nor `w-sched2` had available.

**This lane deliberately did none of 1–3 for `cflow-loop`.** The entry a loop
lane would write is `CfgClass { class: "cflow-loop", sub:
CfgSub::Keys(&["ptr-walk-mod-loop"]) }` — 1 of 240 keys — and whether the
evidence supports it is that lane's finding, not this one's. An instrument
change landed in the same rung as the measurement motivating it has nothing
independent to grade it against, which is `w-sched2` §8's own condition and it
is honoured here in the mirror.

---

## 7. Found and not taken — two pre-existing looseness measurements

Both are in the screen's `Unclassified` arm, both predate this lane, and
**neither is fixed here**: each would move the reachability figure and therefore
wants its own prereg and its own grading. Filed rather than patched.

### 7.1 `⊤` surfaced a MASKED shortfall on its first run (board **#807**)

`⊤` came back **14**, and 17 − 2 `Unclassified` rows predicted 15. The missing
one: `cfg_reach` returns `NeedsClass` **before** it checks the shortfall, so a
TU that is *both* blocked on a missing class *and* carrying unclassified bodies
reports only the first. Under `⊤` the class blocker vanishes and the shortfall
surfaces.

Measured over the 17 frontier TUs — **3 have a shortfall, not the 2 the screen
shows**:

| TU | blocked | classified | shortfall | today's verdict |
|---|---:|---:|---:|---|
| `src/system/rndobj/wordwrap.cpp` | 3 | 2 | 1 | `needs a CFG class the port lacks: cflow-if-n` |
| `src/system/synth_xbox/Biquad.cpp` | 2 | 1 | 1 | `1 blocked fn with NO CFG class` |
| `src/system/utl/Pool.cpp` | 3 | 2 | 1 | `1 blocked fn with NO CFG class` |

**The reading that matters:** a lane taking `wordwrap.cpp` as *"just needs
`cflow-if-n`"* would be wrong — teaching the port `cflow-if-n` leaves it
`Unclassified`. The screen is not lying (both verdicts mean "not reachable")
but the *actionable* one is incomplete.

### 7.2 `classified` counts IN-CLASS rows too (board **#808**)

`cfg_reach`'s doc says the classified count is compared against `fn_blockers`'
total, but the loop sums **every** crossed row, and `fn_cflow`'s cross is
written over every function — `FnVerdict::key` spells in-class labels and
blocker keys into one namespace (`scan.rs`). So a frontier TU with in-class
functions contributes rows to `classified` that are not in `fn_blockers`.

Positive check, **3 of 17 frontier TUs have `classified > blocked`**:

| TU | blocked | classified | in-class fns |
|---|---:|---:|---:|
| `src/keygen_xbox.cpp` | 18 | 20 | 2 |
| `src/system/utl/EncryptXTEA.cpp` | 4 | 5 | 1 |
| `src/xdk/nuispeech/mmio.cpp` | 3 | 11 | 8 |

The inflation can **mask** a §7.1 shortfall: `mmio.cpp` has 8 spare in-class
rows, so up to 8 unclassified blocked bodies there would be invisible. The fix
is to count only crossed rows whose key is in `fn_blockers`; it moves the figure
in the **narrowing** direction (more `Unclassified`, fewer `Reachable`), which
is the safe direction but is still a change of meaning and not this lane's.

---

## 8. What was deliberately not built

* **`cflow-loop` did not enter `PORT_CFG_CLASSES`**, restricted or otherwise
  (§6). The brief forbade it and the reasoning is `w-sched2` §8's.
* **No `Selected` variant, no `crates/c2-core` change, no fixture.** There is no
  emit to grade, so a fixture could not be turned red by breaking anything this
  lane wrote — `w-sched2` #792's standard, applied here.
* **`cfg_reach_control` stays CLASS-level** (`covers_class`, not `admits`). The
  cross-tab key on a *matching* TU is the census's **in-class label**, not a
  blocker key, so asking a blocker-key restriction about it is a category error
  that would fail the control on a converted TU for a reason unrelated to it.
  Written down rather than discovered by the first lane to restrict a class.
* **Neither §7 looseness was fixed.**
* **`scripts/status.sh` was not taught the new keys.** The nine `cfg-*` metrics
  are published in the `GAP-METRICS` block and readable by `p_metric`, but no
  registry row was added — that is a `STATUS.md` surface decision and the
  block's own reachability numbers are not yet consumed there either
  (`w-bc` §5.1 is still open). The keys exist so the figures *can* be collected
  without prose; nothing here claims they are.

---

## 9. Prereg, fully scored

| # | prediction | outcome |
|---|---|---|
| **R1** | IDENTITY — reachable set identical by name, every `gap-metric` unchanged | **HELD.** 53 values unchanged, 9 added; screen text byte-identical; `[Main.cpp, xboxheap.cpp]` both sides |
| **R2** | `reach(⊥) == 0` | **HELD.** 0 |
| **R3** | `reach(⊤) − reach(SHIPPED) ≥ 8` | **HELD.** 14 − 2 = **12** |
| **R4** | `ENUMERATED` differs from `SHIPPED` on 0 TUs | **HELD.** 0, and equal as sets |
| **R5** | ≥ 5 distinct census keys crossed with `cflow-loop` | **HELD, by 48×.** **240**, including `ptr-walk-mod-loop`. The registered failure branch — "the price is a census-side key mint" — does not fire |
| **R6** | M1 caught, `⊥` jumps, control prints FAIL with a count | **HELD.** 6 unit tests; `⊥` 0→2; violations 0→1; both TUs named |
| **R7** | M2 caught by the prefix witness, naming the key | **HELD — and it exposed §5.2.** The ledger cross-check did *not* catch it, because no shipped entry is restricted. Fixed in a separate commit; M2 now fails 2 tests |
| **R8** | `fnbyte-differs` 0; tests 0 failed; gate PASS; `status.sh --check` PASS; `board_audit.sh` clean | **HELD** — §10 |

---

## 10. Gate evidence

All taken at tree `9f5120a`, the tree being landed.

| check | result |
|---|---|
| `cargo test --workspace --release` | **888 passed, 0 failed, 27 targets** (9 tests added by this lane) |
| `scripts/gate.sh --jobs 6` | **GATE: PASS — 18/18 lanes, 0 FAIL, 0 SKIP, 0 NO-RESULT; 4,698 fixture-verdicts.** Sweep **16,710/16,710 reached, 16,614 graded, 0 mismatch**; mode cross **81,905 selected, 81,517 graded, 0 mismatch**. Log: `work/w-subclass/gate_final.txt` |
| `scripts/status.sh --check` | **PASS — 23 metrics registered, parsers pinned, absence renders NO-RESULT** |
| `scripts/board_audit.sh` | **CITED BUT NOT ON THE BOARD: 0**, unresolved anchors 0, raw line anchors 0, rows behind prose 0 |
| `gap-metric fnbyte-differs` | **0**, before and after |
| `gap-metric mismatch` | **0**, before and after |

### 10.1 The first gate run SKIPPED, and that is worth recording

A bare `scripts/gate.sh --jobs 6` from a worktree reported **18 SKIP, exit 0**.
The script says so itself — *"this exits 0 by design and is NOT a green gate.
This run establishes nothing about the port"* — which is the mitigation working,
and it is trap 5 exactly: a lane that read the exit code would have banked a
green gate over nothing graded. A worktree sits three directories below the repo
root, so `../wibo` and `<repo>/compilers` do not resolve; the documented
`C2RS_WIBO` / `C2RS_COMPILERS` / `C2RS_DC3` overrides fix it
(`work/w-subclass/env.sh`, machine-local and uncommitted). **The recorded run is
the second one**, and the number to check is `graded: 4698`, never the exit code.

The gate was run **twice more**: once at `828f133` (the mechanism) and once at
`9f5120a` (after an unused accessor was removed and the rung's reader count was
pinned to a measured `grep`). Both PASS with identical counts.

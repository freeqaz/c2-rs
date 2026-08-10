# w-seclayout — the section-layout model is NOT the successor: at the workload's own flags the port is already on the COMDAT writer, the `26`-stop is worth **+2 binds and 0 converts**, and the survey found a latent wrong emit on the 1:1 path instead

    Tag:       w-seclayout
    Slug:      w-seclayout
    Date:      2026-08-10
    Fixtures:  none, and the reason is measured rather than asserted. This lane
               is a SURVEY that ends in a priced decline: it ships two doc
               corrections in `crates/` and no behaviour. The one cell it
               considered — a `_neg` pinning "a `.gl` record set covering every
               segment is not c2's emit set" — was measured against the
               fixtures that already exist and would grade **nothing**:
               `fixtures/cpp/il_gl_sep26.cpp` already carries a
               3-records / 1-emitted over-emit at `/Ox`, asserts
               `NotImplemented`, and is graded in all 12 mode lanes. A cell
               that grades nothing is worse than none (#2698/#2699), so it was
               declined and the shape is pinned in `bind.rs`'s doc and board
               **#2903** instead.
    Census:    per-function unchanged, **+0**. TU match **23 → 23**, mismatch
               **0 → 0**, codegen-gap **0 → 0**, vocab-gap **848 → 848**,
               capture-fail **7 → 7**. `fnbyte-exact` **35,810 → 35,810**
               (**+0**). `gap-metric` keys: **0 added, 0 removed, 0 changed
               value**. Zero of 878 TU class verdicts moved, and **zero of
               878 rows differ on ANY `--jsonl` field**. This lane changes no
               executable statement — the only `crates/` edits are doc
               comments — and the neutrality is the *whole* metric table
               rather than a selected row.
    Record:    this file; PREREG `work/w-seclayout/PREREG.md`, committed at
               **`26595bdb`** — before the first `crates/` change of any kind,
               including the reverted counterfactual. Scored in §9.
    Lane:      w-seclayout, worktree branch `worktree-agent-a611741296081ab81`
               off master **`5127a20e`** (*"docs: ROADMAP §10.35 — session
               close"*), re-derived with `git merge-base HEAD master` =
               `5127a20ee2c4c93dfdc01166768a52aca1a3d0a7`. Workload stamp
               **dc3 `104e7df9c10acfe56ee3a87d75f0a9c85740df11`**, tracked tree
               CLEAN (one untracked dir, `work/`) — **unchanged from
               `w-selbind` and `w-frame783`, the third lane running at one
               stamp** (#2392). `work/dc3-workload/files.txt` sha256
               `4996839bf897…6853b6` (878 lines), `flags.txt` sha256
               `fa8ba48aa212…fbcb48`, **used as they stand and never
               regenerated** (#2700). Toolchain
               `compilers/X360/16.00.11886.00`, wibo `1.2.0-c2rs.1`. Base
               binary sha256 **`ac193eab2651…c902e3`**, copied to
               `work/w-seclayout/c2rs-base` before the first edit (#2409) —
               that binary produced the base scan and the counterfactual
               comparison in §5.4. It was **destroyed mid-lane** by a
               `git filter-branch` that stripped this lane's oversized scratch
               out of its own commits (captured IL, objs and binaries, which
               `CLAUDE.md` forbids committing and which a `git add -f` of the
               lane directory had swept in). It was **rebuilt at the same
               merge-base** as sha256 **`06ec9f9473da…165487`** — a different
               hash only because rustc embeds the build path, and this build is
               in a separate merge-base worktree — and that is the binary the
               final base/tip neutrality scan uses. The tip's is
               **`886231ceec3d…8c0f62`**. Every figure below is a scan at both
               ends, each end scanned by **its own binary**.
    Ships:     **no behaviour.** Two doc corrections in `crates/`:
               `gl.rs`'s `Name26Introduced` clause comment (its `/Ox` premise
               named as such, with the `/O1` measurement that replaces it) and
               `bind.rs`'s `Bindings::per_record` (the missing clause 4, with
               the one workload TU that falsifies its unstated premise).
               `docs/CEILING.md` §15. Board rows **#2900**–**#2907**;
               **#2908**–**#2939** left explicitly unminted.
               **+0** `#[test]` (1,497 → 1,497), **0** new cargo targets.
    Adopts:    **nothing.**

---

## 0. THE COMMISSION, AND THE ONE SENTENCE THAT REPLACES IT

> The brief: *"Price the section-layout model, the named successor to the
> frontier. … That stop is not a reader defect — it is COMDAT-style linkage
> against a packed single-`.text` writer, i.e. a section-layout model."*

**The premise is a `/Ox` fact and the 380 are a `/O1` population.**

> ### #232 was measured at `/Ox /GS- /c`. The dc3 workload compiles at **`/O1`**, `/O1` implies `/Gy`, and `PortC2::flags_imply_function_level_linking` therefore routes **every one of the 380** to `coff::emit_comdat_obj` — which already gives each function its own COMDAT `.text`. Read off seven of the 380 at the workload's own flags: **117 `.text` sections over 7 objs, 117 COMDAT, 0 packed, MIXED on 0 of 7.** The layout the successor was supposed to build is the layout the writer already produces. Board **#2900**.

So the lane declines the build and prices what is actually there. **A priced
decline was the commission's own stated success condition and this is one.**

---

## 1. RESULTS, IN THE UNIT THE COMMISSION ASKED FOR

| question | answer | instrument |
|---|---|---|
| how many mechanisms | **six**, and only one of them is the writer | §7 |
| how many of the 380 would **BIND** | **+2** | counterfactual scan, §5.4 |
| how many of the 380 would **CONVERT** | **0** | same scan, `match` 23 → 23 |
| `fnbyte-exact` delta | **0** (35,810 → 35,810) | both scans |
| is it a routing question | **partly, and the routed-to path is already correct** | §5.1 |
| what the fence would have to be | **factor A** — and nothing in the input supplies it | §7 |

---

## 2. `CEILING.md` §11.4, WORKED FIRST, OFF THIS LANE'S OWN CAPTURE

The pass is in `work/w-seclayout/PREREG.md` §1, frozen before the first
`crates/` change. Its two live items:

**Item 8 — `gate_cause`, and nothing else.** Over this lane's own
`target380.txt`, rebuilt from its own scan: `gate_cause` is
`gl-stop-26-introduced` on **379** and `drectve-not-boilerplate` on **1**;
`gate_causes` carries `gl-stop-26-introduced` on **380** and
`body-out-of-class` on **380**. #2864 reproduces digit for digit.

**Item 8b — and this is the one that decided the lane.** See §4.1.

---

## 3. #232 RE-DERIVED — the defect was the WRITER, not the relaxation and not a missing fence

Re-derived from the board row, `w-cross`'s and `w-order`'s rungs and the clause
itself, and frozen in the PREREG before any measurement could bias it.

| candidate | verdict |
|---|---|
| the relaxation itself (`d0d8a98`, W-ADOPT #151) | **no.** The widened *scanner* is still shipped today; `w-cross`'s fix widened nothing back |
| the absence of a fence | **no.** A fence existed, was reasoned about in `d0d8a98`'s own commit message — *"the one place the widening could have produced wrong bytes instead of a refusal"* — and shipped `fixtures/cpp/il_gl_sep26.cpp` asserting `NotImplemented` at base and tip |
| **a writer that emits the wrong layout** | **YES.** 6 sections against 7, both symbols packed into one `.text`, in the opposite order — `Port=Mismatch @ offset 2`, `NumberOfSections` |

**What the fence guarded was a run that *ends* at `26`; what broke is a run that
*begins* at one.** That name is NUL-terminated like any other and every
downstream field arithmetic on it is correct — nothing was wrong with the
*name*. What was wrong is the **obj shape it implies**, and the gate was widened
without asking the writer whether it could express that shape.

**The 255 commits are a separable process finding**: `scripts/gate.sh` did not
run `expr_sweep.sh`, so the merge gate structurally could not see the class.
Closed by `w-book3`. The mechanism and the survival time have different causes
and quoting one as the other is how this row gets mis-read.

**The generalization, which is the part that transfers**: *seeing a name* and
*being able to emit a body under it* are different claims. W-ADOPT's test
conflated them. **§6 is that same conflation, alive, on a different path.**

---

## 4. THE INSTRUMENT THIS LANE REFUSED TO QUOTE

### 4.1 `selective_bind` reads `records < segments` on 380 of 380 and it is an artifact of the selection

The obvious way to price *"would the 380 bind if the walk were repaired"* is the
`--jsonl` field `selective_bind`. Read naively it says **records < segments on
380 of 380**. **It is an artifact, and this lane's own selection is what makes
it one.** `records` comes from `gl::gl_bound_names`, which is
`gl_defined_names_framed(…).unwrap_or_default()` — so a TU whose walk **stops**
reads `records = 0`, and the 380 were selected *for stopping*. Measured:
`records == 0` on **380 of 380** (`work/w-seclayout/records0.py`).

Nine rankings this session were artifacts. This would have been the tenth, and
it was named in the PREREG **before** the answer was known. Board **#2905**.

### 4.2 What replaced it, and the cross-check that licenses it

`work/w-seclayout/glwalk26.py` — `gl_defined_names_framed` transcribed at *this*
tree (shipped framing `codec::gl_offset_framed_relaxed`, `NameFit::
InlineOrStringTable`, all six stops) with `Name26Introduced` **recorded and not
taken**. It is a transcription, so it was checked against the reader it
transcribes: the Rust `selective_bind` quad under the reverted counterfactual
agrees with it on **7 of 7** read TUs — 16/16, 13/13, 24/34, 19/23, 21/32,
73/119, 116/236 — before any number was quoted off it.

---

## 5. WHAT THE 380 ACTUALLY NEED — READ, NOT COUNTED

Seven TUs, captured and read by this lane at the workload's own flags:
`work/w-seclayout/{SECLAYOUT,JOIN,EMITJOIN}.txt`.

### 5.1 The section layout, and the routing answer

| TU | CF records | `.ex` segs | obj `.text` | all COMDAT? | named-not-emitted |
|---|---:|---:|---:|---|---:|
| `synth_xbox/MeterEffect.cpp` | 13 | 13 | **13** | yes | **0** |
| `synth_xbox/HeadsetXferEffect.cpp` | 16 | 16 | 14 | yes | **2** |
| `utl/TempoMap.cpp` | 24 | 34 | 22 | yes | 2 |
| `LIBCMT/rtti.cpp` | 19 | 23 | 14 | yes | 5 |
| `nuiapi/headtracker.cpp` | 21 | 32 | 9 | yes | 12 |
| `synth/Pollable.cpp` | 73 | 119 | 43 | yes | 30 |
| `utl/UrlEncode.cpp` | 116 | 236 | 2 | yes | **114** |

**`.text` is 100 % COMDAT on 7 of 7 — 117 sections, 0 packed, MIXED on 0 of 7.**
The section *kind* and the one-per-function *shape* are what `emit_comdat_obj`
already produces, and `PortC2::build` already routes these TUs to it on `/O1`.

**So the routing question is answered YES for the thing it was asked about, and
it changes nothing**, because the refusal is upstream of the writer entirely:
all 380 stop in the reader.

**And `EMITTED-but-NOT-NAMED` is 0 on 7 of 7** — clause 3 is discharged
everywhere, which is what "the emit set is entirely named" means. The whole
residue is in the other direction.

### 5.2 What the writer does have wrong, and it is one byte

`writer::emit_comdat_obj` hard-codes `COMDAT_SELECT_NODUPLICATES` (1) for every
`.text`. c2 emits `IMAGE_COMDAT_SELECT_ANY` (2) on **99 of the 117**.

**`26` is not the byte that predicts it** — **80** of the 117 emitted records
carry `SELECT_ANY` and are *not* `26`-introduced. The byte that does, **117 for
117**, is the record's own FLAGS at `name_nul + 5`, which
`gl::record_is_plain_external` **already reads** as `FLAGS_PLAIN`:

```text
   linkage=05  flags=0x00  ->  Selection = NODUPLICATES(1)      18 of 117
   linkage=05  flags=0x20  ->  Selection = ANY(2)               98 of 117
   linkage=05  flags=0x60  ->  Selection = ANY(2)                1 of 117
```

**The control, and it is why this is not shipped.** All **32** records the port
emits today — across the 23 byte-exactly matching workload TUs — read
`flags == 0x00`. So the `ANY` branch is unreachable on every obj the port has
ever produced. That is simultaneously the neutrality proof and the reason
shipping it buys nothing: a *denominator* purchase of exactly #2865's kind.
Board **#2901**.

### 5.3 The pincer, from the scan's own `--factors-tsv`

```text
   MeterEffect.cpp        A----     13 = 13 = 13, and 28 `.rdata$r` sections
   headtracker.cpp        -BC--     21 records,   9 emitted
   UrlEncode.cpp          -BC--    116 records,   2 emitted
   Pollable.cpp           -B---
   rtti.cpp               -B---
   HeadsetXferEffect.cpp  -----
   TempoMap.cpp           -----
```

**Not one of the seven has `A ∧ C`, and `D ∨ E` is 0 on all seven.** The one
that satisfies factor A fails C on the `.rdata$r` ladder head — seven refusals,
declined by `w-rdata` and re-declined by `w-rtti` — and the two that satisfy C
fail A by 12 and by 114 bodies. Board **#2902**.

### 5.4 The counterfactual: **binds +2, converts 0**

`GlBindStop::Name26Introduced` built out of the **binding policy only**
(`w-decouple`'s seam, so neither fence ground set moves and #2622/#2623's
−1 `fnbyte-exact` cannot be confounded into the result), the same 878 TUs, the
same committed list and flags, then **reverted**.

| key | base | 26-stop removed |
|---|---:|---:|
| `match` | 23 | **23** |
| `mismatch` | 0 | **0** |
| `codegen-gap` / `vocab-gap` / `capture-fail` | 0 / 848 / 7 | 0 / 848 / 7 |
| `fnbyte-exact` | 35,810 | **35,810** |
| `factor-a` / `a-and-b-and-c` | 28 / 27 | 28 / 27 |
| `selbind-one-to-one-tus` | 22 | **24** |
| `selbind-selective-tus` | 12 | 506 |
| `selbind-emit-subset-gate-tus` | 34 | 342 |
| `selbind-total-tus` | **0** | **0** |
| TU class verdicts moved | — | **0 of 878** |
| `gap-metric` keys changed value | — | **6** |

The +494 that become selective all die at clause 4 by construction. The first
cause of **819** TUs merely moves one clause along: 492 to
`bind-record-count-ne-segments`, 316 to `gl-stop-varargs-record`, 9 to
`gl-stop-name-too-far`, 2 to `body-out-of-class`. Board **#2904**.

### 5.5 The stem column, and this lane's own prediction refuted

#2243's `dstem` collapse over the `26`-introduced names of the read seven:
**9 → 9, 3 → 3, 2 → 2, 5 → 5, 4 → 4, 8 → 8, 37 → 37**. Not one instantiation
family. They are per-class compiler-generated members — `??_E`/`??_G` deleting
destructors, in-class members, `?what`, `??0exception` — replicated across
*classes*, which the stem test does not collapse and must not be read as
collapsing. **PREREG P7 predicted a ≥ 2× collapse at p 0.75 and is WRONG.**
Board **#2906**.

---

## 6. WHAT THE SURVEY FOUND ON THE WAY — a latent wrong emit of #232's exact kind

`Bindings::selective` states the over-emit obligation and refuses on it
unconditionally at clause 4 (#2820). **`Bindings::per_record` — the shipping 1:1
path — has no such clause.** Its soundness rests on an unstated premise: that a
record set covering *every* `.ex` segment **is** c2's emit set. `w-selbind`
refuted that premise for a *subset*; nobody re-asked it for the total case.

Measured over its own population — the **29** TUs `gl_body_start_coverage`
reports `n of n`, which `CEILING.md` §12 calls *"full coverage of this
acceptance path"* — **exactly one fails factor A**:

```text
src/system/synth_xbox/HeadsetXferEffect.cpp
    `.gl` body-start coverage    16 of 16     <- per_record would bind all 16
    obj `.text` COMDATs          14           <- c2 emitted 14
    absent from the obj ENTIRELY — not defined, not undefined externals:
        ??_ECXAPOParametersBase@ATG@@WCA@AAPAXI@Z
        ??_GCXAPOParametersBase@ATG@@UAAPAXI@Z
```

**`CEILING.md` §12's 29 is not a sound bound on this acceptance path; 28 is.**

Three live fences hold it — `Name26Introduced` in front, `unclaimed-gl-symbol`
(#1721) and `body-out-of-class` behind, all three read out of the reverted
counterfactual's own `gate_causes` — so this is **latent and not live**. That
is exactly the status #232 had before `d0d8a98`, and #232 also had a fence that
covered a neighbouring shape. Board **#2903**, doc'd at `bind.rs::per_record`.

**No new fixture, and the reason is measured.** `fixtures/cpp/il_gl_sep26.cpp`
already carries a 3-records / 1-emitted over-emit at `/Ox` (measured this lane,
`work/w-seclayout/probe.sh`), asserts `NotImplemented`, and is graded in all 12
mode lanes. A second cell on the same axis grades nothing (#2698/#2699).

---

## 7. THE PRICED DECLINE — six mechanisms, one of them the writer

| # | mechanism | state |
|---:|---|---|
| 1 | per-function COMDAT `.text` | **ALREADY SHIPPED** — `emit_comdat_obj`, routed on `/O1` |
| 2 | aux `Selection` from the `.gl` FLAGS byte | measured 117/117, **worth 0 today**, unreachable until 3 |
| 3 | **factor A** — `selective` clause 4, and `per_record`'s missing one | **no solution in the input**; 6 of 7 read TUs, 380 of 380 |
| 4 | factor C's `.rdata$r` | 7 refusals, declined twice (§2.4) |
| 5 | `.text$yd` / `.xdata$x` | the ladder's remaining two steps |
| 6 | `body-out-of-class` codegen | 380 of 380 |

**Does this convert anything without a codegen distance §10 already measured?
No — and the number is 380 of 380.** `body-out-of-class` is in the
`gate_causes` set of every one of the 380, and `D ∨ E` is 0 on all seven read
TUs. Even a perfect factor A and a perfect writer leave codegen owed on the
whole population.

**The one-sentence version.** §14.4 called `gl-stop-26-introduced` and
`body-out-of-class` *"the section-layout model under another name"*. The layout
is already right; the first clause is worth **+2 binds and 0 converts**; and
what is left is **factor A and codegen** — the same two things §13.2 and §10
already named, with one fewer place to look.

---

## 8. NEUTRALITY, FOUR LEVELS, WITH DIRECTIONS

This lane changes no executable statement — the only `crates/` edits are doc
comments — so the neutrality claim is the *whole* table rather than a selected
row. Both ends scanned by their own binary: base
`06ec9f9473da…165487` (rebuilt at the merge-base `5127a20e`, **KEPT** at
`work/w-seclayout/c2rs-base`), tip `886231ceec3d…8c0f62`.

| level | base | tip | direction |
|---|---|---|---|
| **1 — obj bytes** | mismatch **0** | mismatch **0** | unchanged; 19,556 `expr_sweep` cases and the `mode_cross` grid at the tip |
| **2 — TU verdicts** | 23 / 0 / 0 / 848 / 7 | 23 / 0 / 0 / 848 / 7 | **0 of 878 moved**, checked **per TU** and not by subtracting totals |
| **2b — gate first cause** | — | — | **0 of 878 moved** |
| **3 — per-function bytes** | `fnbyte-exact` 35,810 | 35,810 | **+0**; `fnbyte-differs` 1,898 → 1,898 |
| **4 — every `gap-metric` key** | 277 keys | 277 keys | **0 added, 0 removed, 0 changed value** |
| **5 — every FIELD of every ROW** | 878 rows | 878 rows | **0 rows differ on any field**, `detail` string included (`rowdiff.py`) |

Level 5 is there because "0 moved" can be an artifact of which columns were
compared. `rowdiff.py` compares every key of every `--jsonl` row, including the
rendered `detail` prose, and finds **0** differences over 878 TUs.

**The `fnbyte-exact` counterfactual, run before shipping as required**: the only
`crates/` edit this lane ever *compiled* is the reverted 26-stop
counterfactual, and it read **35,810 → 35,810**. A walk change cost one earlier
lane −1, so it was measured rather than assumed.

**One claim retracted inside the lane.** An intermediate commit message asserted
the tip binary was *byte-identical* to the base. It is not: the doc comments
shift line numbers, and `#[track_caller]`/panic-location strings carry them, so
the binaries differ. That observation came from a build the gate had made after
`hatch_red.py` reverted `crates/` (§10), i.e. from the base tree wearing the
tip's name. The neutrality above is measured on output, not on the binary.

---

## 9. PREREG SCORED

| # | claim | p | outcome |
|---:|---|---:|---|
| **P1** | ships no `crates/` behaviour, declines with a price | 0.85 | **RIGHT** |
| **P2** | `fnbyte-exact` delta exactly 0 | 0.93 | **RIGHT** |
| **P3** | of the 380, **0** would bind under the counterfactual | 0.80 | **WRONG — 2 bind** |
| **P4** | of the 380, **0** would convert | 0.97 | **RIGHT** |
| **P5** | the `/Gy` writer does not already cover it; not a routing question | 0.88 | **HALF-WRONG — see below** |
| **P6** | the Selection byte is `SELECT_ANY`, not `NODUPLICATES` | 0.70 | **RIGHT**, 99 of 117 |
| **P7** | the `26` names collapse ≥ 2× under `dstem` | 0.75 | **WRONG — 68 names, 68 stems** |

**P3 — and the antecedent is exactly where it failed.** The registered
antecedent was *"clause 3 or clause 4 fires on every one of them"*, with the
conjunct *"and no earlier clause fires"* correctly attached. What was **not**
checked is the other end: that no LATER contract applies. Two TUs do not reach
`Bindings::selective` at all — their records come out **1:1** with the segments,
so they take `Bindings::per_record`, which has no clause 4 to fire. **The
antecedent named the wrong function.** This is the standing lesson in its exact
form: an antecedent that only makes the registered clause true is not the
antecedent the claim needs.

**P5, and it is the deciding row, scored honestly as half-wrong.** Its
registered antecedent was *"c2's real obj for at least one read TU is MIXED"* —
and **MIXED occurs on 0 of 7 at the workload's flags**, so the antecedent is
**false**. The conclusion happens to survive (the writer does not cover it — the
Selection byte is wrong on 99 of 117), but it survives for a reason the
antecedent does not name. **Registering an antecedent taken from a `/Ox`
reproducer for a `/O1` population is the same error the commission itself
carried**, and it is only visible because the antecedent was written down. The
falsifier as written — *"every read TU's obj being uniformly COMDAT ⇒ it is a
routing question"* — fired, and the correct verdict is that the routing is
right and the *byte* is wrong, which is neither branch the falsifier offered.

**P4 was the registered unlosable row and its falsifiers were written down**
(*"a TU of the 380 reaching `class == match`"*), and it held: 0 of 878 class
verdicts moved under a counterfactual that removed the clause outright.

**Score: 4 right, 2 wrong, 1 half.** Both wrong rows are wrong in the direction
of *this lane having under-estimated how much the input can name*, which is the
same direction #2820 and #2860 were wrong in.

---

## 10. A PROCESS FINDING, PAID FOR IN THIS LANE — `scripts/gate.sh` DISCARDS UNCOMMITTED `crates/` WORK

#2668 says *"commit before any gate row or mutation script"*. This lane can now
name the mechanism, because it paid for it.

`gate.sh`'s **first** lane is `work/w-hatch/hatch_red.py`, whose arms restore the
tree with an **unconditional `git checkout -- crates/`** in a `finally`
(`hatch_red.py:192`; its own module doc says so at line 45). The lane's *verdict*
is a pure function of the log, so a `REFUSED HATCH-STALE` — which is what this
tree produces — is reported **after** the arms and their `finally` have already
run.

Observed here, both directions, same script and same refusal:

| `crates/` state when `gate.sh` started | what happened |
|---|---|
| two doc edits **uncommitted** | discarded. The gate then built the *base* tree and pinned it as the tip: `sha ac193eab2651`, which is the merge-base binary's own hash |
| the same two edits **committed** | preserved. The gate built and pinned `sha 886231ceec3d`, the tip's |

The second-order cost is the one worth recording: the run **looked green** and
was green — on a tree that no longer contained the change under test. The
byte-identical-binary claim retracted in §8 is exactly that artifact. Board
**#2907**.

---

## 11. WHAT THIS LANE DECLINED TO RE-PRICE

* the seven / six / nine mechanisms on `vec.cpp` and `decomp_pch.cpp`
  (`w-phase7b` §4–§5, #2827) — read, not re-derived;
* factor C's greedy ladder (§2.4) — declined twice already, and C is
  necessary-not-sufficient;
* the emit-order rule (#259, `w-order`) — the ordering half of the same model,
  untouched here.

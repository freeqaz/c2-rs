# `w-permeasure` — the permuter PRE-MEASUREMENT (2026-08-25)

    Tag:       w-permeasure
    Slug:      w-permeasure
    Date:      2026-08-25
    Kind:      characterization lane
    Base:      a8593651b
    Corpus:    ../dc3-decomp @ 15a64d92f1975868e55a1c670d312a8e464074c3, 0 dirty
    Rows:      #3534–#3538
    Fixtures:  none — characterization lane: reads real-c2 and real-decomp behaviour
    Census:    +0
    Reach:     0 — predicted 0, delivered 0. Zero `crates/` bytes.
    Gate:      scripts/gate.sh --jobs 4 --require-graded — see §7
    Outcome:   **instrument**
    Prereg:    docs/rungs/_2026-08-25-w-permeasure-prereg.md (committed first, ab195639d)
    Finding:   docs/PERMUTER_POPULATION.md

---

## 0. The outcome word, and why it is `instrument`

This lane was asked to decide **which permuter is worth building**, and it
decided. It converted no TU (`converted` is wrong), it did not decline — the
measurement was made and the recommendation is given (`declined` is wrong) —
and it produced no shared machinery for the port (`built` is wrong). It built a
**graded measuring instrument** and published what it measured. That is
`instrument`.

**Reach 0 was predicted and delivered.** No `crates/` byte moved.

---

## 1. The question, and the answer in one line

Is the failure population a permuter would actually face — hand-written decomp
near-misses — shaped like the port's own wrong-body population? Board **#3369**
is the conflation.

**No. They are opposite, and the difference is not marginal.** Full numbers,
both sides re-measured on this tree with one lens, are in
[`docs/PERMUTER_POPULATION.md`](../PERMUTER_POPULATION.md); the four rows that
decide it:

| | port `a8593651b` (1,968 bodies) | decomp `N99` (405 bodies) |
|---|---:|---:|
| substituted words differing in **opcode** | **99.87 %** (7,902 / 7,912) | **2.14 %** (54 / 2,520) |
| … differing in a **register** | 0.04 % (3) | **52.50 %** (1,323) |
| **pure reorderings** | **0** of 1,968 | **32** of 405 = 7.90 % |
| the port's mechanism (one side calls, the other does not) | the whole population | **0 of 405** |

---

## 2. Was the population reachable? YES, and the prereg's decline floor was not hit

The prereg's §4 floor 1 was `|N| < 100 ⇒ declined`. Measured over
`../dc3-decomp`'s 979 units carrying both a target and a base obj:

| | |
|---|---:|
| pairable `.text` COMDAT bodies (`P`) | **29,163** |
| byte- and target-identical | 20,475 |
| byte-identical, callee **name** differs (broken out, not folded) | 7,158 |
| **`N` — bytes differ: the near-miss population** | **1,530** |
| `N90` / `N99` (joint on the row, never a product of rates) | 1,098 / **405** |

`|N99| = 405 ≥ 50`, so floor 4 was not hit either and the recommendation is
owed rather than withheld.

---

## 3. Predictions — scored against §3 of the prereg, unedited

| # | prediction | p | outcome |
|---|---|---:|---|
| **P1** | `N90`'s opcode share of substituted words **< 90 %** | 0.75 | **HIT** — 12.11 %. Registered against `N₉₀`; `N99` is 2.14 % |
| **P2** | pure reorderings **> 1 %** of `N` | 0.60 | **HIT** — 3.53 % (54 / 1,530); 7.90 % in `N99` |
| **P3** | bodies wrong at word 0 **< 50 %** of `N` | 0.80 | **HIT** — 1.24 % (19 / 1,530) |
| **P4** | register-only substitutions **≥ 5 %** of `N90` words | 0.70 | **HIT** — 54.45 % |
| **P5** | `\|N\| ≥ 200` | 0.85 | **HIT** — 1,530 |
| **P6** | control arm 1 ≥ 99 % **on the first run** | **0.45** | **HIT** — 100.0000 %, 1,968 / 1,968 |
| **P7** | the recommendation **flips** off `splice.rs`'s inline model | 0.70 | **HIT** — §5 |
| **P8** | transfer-count disagreement **≥ 10 %** of `N` | **0.30** | **MISS** — **0.0 %**, and the miss is the sharpest single result in the lane |

**Seven hits and one miss, and the miss is the one that matters.** P8 was
registered at 0.30 as the hedge — *"inlining is a material minority here too"* —
and it is **0 of 1,530**, joint, measured on the row. There is no minority. The
mechanism that is the entirety of the port's population does not occur once in
the permuter's.

**P6 is a hit against my own stated reasoning and is reported as a warning, not
a win.** The prereg put it below even *because every re-derivation control in
this repo's history caught something on its first run*. Arm 1 did not — and
§4 is why that was the wrong thing to be reassured by.

---

## 4. THE BUDGETED SURPRISE FIRED THREE TIMES, AND NEVER WHERE THE CONTROL WAS LOOKING

Arm 1 was **100.0000 % green through all three defects**, because it grades the
**lens** and every defect was in the **input**. Full accounts in
`PERMUTER_POPULATION.md` §5.1; the shape of them:

1. **Whole-section bodies.** Taking a COMDAT `.text` section's raw data as the
   body — `c2-obj`'s rule, correct for the objs it was written for, wrong here,
   where a function sits at `Value = 8` behind a section definition and ahead of
   an `__unwind$` block. Published headline before the fix: **84.8 %
   `port-longer|sub+ins|branch-target` over 8,313 bodies**, an artifact of
   *section offset*.
2. **Link state read as compiler behaviour.** Board **#984 in the mirror**: the
   `/Gy` placeholder displacement is `-(offset of the branch word)`, so where
   #984 found byte equality **crediting** an unchecked relocated word, the same
   fact here **penalises** — the same call at a different offset is different
   bytes.
3. **Alignment padding read as inserted code.** 530 of 531 bodies in one cluster
   inserting **exactly two words**, which were `00000000 00000000` at the end of
   the section.

**What caught them.** (1) was caught by **control arm 3** — `decomp.db`'s own
100.0 verdict, the only arm this lane did not author — reading a complete match
as 12 words short. (2) and (3) were caught by **a cluster too clean to be a
compiler decision**: 84.8 % in one signature, and exactly 2 extra words in 530
of 531.

**The methodological result, and it generalises past this lane:** a control that
re-derives an instrument from that instrument's own serialized inputs cannot see
a defect in how those inputs were *obtained*. Arm 1 is a strong control and it
was strong about the wrong half. **`N` fell 8,313 → 2,062 → 1,530 and the
headline inverted twice**, with arm 1 at 100 % throughout.

---

## 5. The recommendation

**Do not build the inline-decision permuter.** `crates/c2-core/src/splice.rs`'s
S7 clause and `INLINE_PREDICATE.md`'s 0.9716 cost model with its 2.84 %
NOT-MODELLED residual are the right knob for the **port's** population — where
99.87 % of substituted words are opcode differences — and measurably the wrong
one for the permuter's, where in `N99` an opcode difference touches 10.6 % of
bodies and 2.14 % of words and the inlining signature is **0 of 405**.

**Build the operand-level search**, in this order, by `N99`'s substituted words
(2,520 words over 397 non-capped bodies of 405):

1. **register assignment** — 52.50 %
2. **stack-slot displacement** — 20.20 %
3. **immediate / literal choice** — 16.23 %
4. **instruction schedule** — a body-level class: **7.90 % of `N99` bodies are
   pure reorderings**, against **0 of 1,968** on the port side
5. branch target / block layout — 7.34 %

**Jointly, 89.4 % of `N99` bodies (355 of 397 non-capped) carry no
opcode-class substitution and agree on their callee set** — reachable in
principle by an operand-level search. In `N` that figure is **39.6 %**.

**This list is already in the repo and the measurement says it was right.**
`docs/rungs/README.md` § "Lane kinds" 2's decision-surface clause — adopted
2026-08-22 from the owner's goal re-ranking — requires general layers to expose
*"allocation order, scheduling tie-breaks, label counters"* as named, enumerable
parameters whose default reproduces c2 byte-exactly. Items 1, 2 and 4 are that
clause's own three. **The clause needs no amendment; it needs the layers.**

**What is NOT recommended, deliberately.** No permuter is priced here, no lane
is proposed, and 89.4 % is a statement about the shape of the target and **not**
a hit rate. `N99` is 405 bodies of 29,163 pairable, and it is selected for
near-ness by construction — so its 2.14 % opcode share must never be quoted as
"how MSVC differs from a naive decomp". `N`'s 23.63 % is the row for that.

---

## 6. `DIFF_STRUCTURE.md` — rescanned, not edited

The brief asked for a rescan rather than an edit. Done, and **the shape is
confirmed while every count moved**:

| | `0c8a185` (the page) | `a8593651b` (this lane) |
|---|---:|---:|
| bodies | 3,195 | **1,968** |
| substituted words | 5,189 | **7,912** |
| opcode share | 99.7 % | **99.87 %** |
| pure reorderings | 0 | **0** |
| first word already wrong | 94.3 % | **92.78 %** |
| LCS-capped / accounting breaks | 0 / 0 | 0 / 0 |

The population nearly halved, the substituted-word count rose by half again, and
the opcode share went **up**. **The page is not edited** — it is a dated record,
its own ⚠ banner already marks §3.2 and one row of §4 refuted, and #3369's rule
is that a doc kept verbatim under a dated banner beats a tidy page. This rung
and `PERMUTER_POPULATION.md` §2.2 are the rescan beside it.

---

## 7. Gate

`sh scripts/gate.sh --jobs 4 --require-graded`, run on this tree.

    18 lanes, 34s at --jobs 4
    expr sweep:  checked=19556  mismatches=0  graded=19460  ungraded=96
    mode cross:  checked=90812  mismatches=0  graded=90424  ungraded=388
    cache-bad=0 in both

**`mismatch 0` is not evidence this lane was correct** (`STATUS.md` trap 5) —
this lane changed no `crates/` byte, so the gate could not have moved. It is
quoted as the required-zero floor, not as a grade. The lane's grade is control
arms 1–3 and §4's account of what they missed.

---

## 8. Artifacts

| path | what |
|---|---|
| `docs/PERMUTER_POPULATION.md` | the finding |
| `docs/rungs/_2026-08-25-w-permeasure-prereg.md` | the prereg, committed before any measurement |
| `work/w-permeasure/permeasure.py` | the graded lens (3 control arms; refuses to print without arms 1–2) |
| `work/w-permeasure/port_fndiff.jsonl` | the shipped instrument's own rows, `a8593651b` (gitignored — raw bytes) |
| `work/w-permeasure/decomp_rows.jsonl` | one row per near-miss body (gitignored) |
| `work/w-permeasure/measure.txt` | the full report |
| `work/w-permeasure/gate.log` | §7 |

`permeasure.py` lives under `work/` and not `scripts/` because
`DECISIONS_2026-08-22.md` decision 12 gives `scripts/` to `w-hygiene` for this
wave and names this lane docs-only. **That is in tension with #1406** — an
instrument whose output is quoted as evidence should run under `gate.sh` — and
the tension is recorded rather than resolved: the fence is today's and explicit,
`gate.sh` cannot run this instrument anyway (it needs `../dc3-decomp` and a
scan's JSONL), and the precedent for a characterization lane's scorer living in
`work/` is `w-memfit`'s `score.py` (#2064) and `w-memcpy`'s `gridm2.py`.
**If a later lane promotes it to `scripts/`, arms 1 and 2 are already written as
a self-checking `control` subcommand that exits non-zero.**

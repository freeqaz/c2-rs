# PREREG — `w-json`, board **#3155**: the residue blocked by SIZE

    Lane:   w-json
    Kind:   construct rung (`docs/rungs/README.md` § "Lane kinds", precedent #290)
    Base:   master `5a25656a`
    Frozen: 2026-08-15, BEFORE the first `crates/` change

Probability form. Registered, then graded in the rung doc's "estimate vs
outcome" table. A row that is wrong is recorded as wrong; a prereg that gets
edited after the fact is not a prereg (`w-fencea` §8.1).

---

## 0. The deliverable, in one sentence

Complete **#3124**'s migration for `json_utf8_copy` — its **two `reach::direct`
call sites** become `BodyLayout` terminators — and grade it by a
**required-zero byte delta**. **Zero TUs converted, by design.** A conversion
means behaviour moved and the lane FAILED.

`pool_ctor_chain`'s one remaining site is **not this lane's**: its back edge is
`bdnz`, `Terminator` has no variant, and `CFG_SHAPE.md` §6.3 declines the
CTR-loop discovery that would justify one. It is `#3146`'s and is not re-priced
here. **Item F is not re-priced by anything in this lane** — peer
`w-itemf-price` holds it.

## 1. The denominator, named — `#3125`

`match` has three meanings in this repo and they move independently. Every
`match` row below names its population and its denominator:

| population | denominator | what it is |
|---|---:|---|
| **878-TU dc3 workload scan** (`c2rs gap`) | **878 TUs** | the goal metric |
| **fixture gate** (`scripts/gate.sh`) | **381 fixtures × 18 lanes** | the correctness gate |
| **`c2rs perf`'s `/Ox` profile** | the `Ox` gate lane | the perf-path fixture count |

And a fourth count that is *not* called `match` and is graded here anyway:
**`fnbyte-exact`**, over emitted function bodies.

## 2. Findings registered

| # | claim | p |
|---|---|---:|
| **H1** | `json_utf8_copy` is a **`#3142`** shape and **not** a `#3154` one: it publishes **three** positions off the same running `t.len()` — `prolog_len`, the `bl __savegprlr_28` `REL24` site, and the `b __restgprlr_28` `REL24` site — so its branches could not have moved alone, exactly as `w-layout`'s `P17` says and `ptr_walk_chain_loop` alone contradicts | 0.92 |
| **H2** | **`#3155`'s own budget figures do not match the emitter.** The row says *"10 sites, 4 labels"*; `w-fencea` §7 says *"ten branch sites, four block labels (`Lelse`/`Lloop`/`Lwide`/`Lnul`)"*. I count **14** branch words (13 `patch` calls + 1 inline back edge) and **10** block labels. The `2` in `#3144`/`#3124`'s residue table is a different and correct count — **`reach::direct` textual call sites** | 0.75 |
| **H3** | **`#3155`'s `label_slots` `Some(5)` is wrong.** `IlFunction::is_framed()` returns **true** for this class, so `label_slots(false)` is `Some(label_lead() + 4)` = **`Some(8)`**, not `Some(lead + 1)`. The admission is still valid — arm 1, lead 4 ≥ 1 — and the grading test reads the number out of `c2-il` rather than from the board, which is exactly why the wrong figure costs nothing | 0.85 |
| **H4** | the migration composes with **no new mechanism**: `BodyLayout::admitting_back_edges` for the back edge, `Terminator::TailCall` for the external epilogue branch, `FinishedBody::at`/`start_of`/`tail_sites` for the three published positions. One new `ChargedClass` variant, one line in `ALL`, two `match` arms in the two grading tests | 0.70 |
| **H5** | the body is **20 basic blocks** in `BlockOrder::IlStatement` | 0.60 |

**Sites moved: 2 of the 3 remaining.** No stretch is registered — 3 of 3 would
require the CTR terminator §6.3 declines, and registering a stretch I am
forbidden to take is not a stretch.

## 3. The required-zero rows — every one is `+0`

| # | quantity | registered |
|---|---|---|
| **P1** | 878-TU scan `match` | **25 → 25**, delta **0** |
| **P2** | 878-TU scan `mismatch` | **0** at both ends |
| **P3** | 878-TU scan, every other field: `codegen-gap` **0**, `vocab-gap` **845**, `port-error` **0**, `capture-fail` **8**, `frontier` | identical |
| **P4** | **fixture-gate `match`, all 18 lanes** | **+0 on every lane**, the table `diff`s empty. **The base values are READ AT THIS LANE'S BASE and not copied from a brief** — `w-fencea`'s `P3` was one low on six lanes because it inherited them (§8.1, the fifth and sixth handed-down number a lane had to check rather than use) |
| **P5** | `c2rs perf`'s `/Ox` profile | **+0** |
| **P6** | `fnbyte-exact` | **35,734 → 35,734** |
| **P7** | `gap-metric` keys | **370**, identical digit for digit. **Not 372** — that figure counts two prose lines mentioning the string; settled three ways this week (`w-fencea` §8.1, `w-labeltable`, and this lane's own extractor, anchored at line start) |
| **P8** | per-TU verdict lines | **878**, `diff` empty sorted |
| **P9** | census | **+0** — no fixture named, no prefix claimed |
| **P10** | `graded tree` identical at both ends of **each** run; base predicted **`e6d4bfb38066`**, **730** files | a run whose two ends disagree is **void** |
| **P11** | sweep **19,556 / 19,460 / 0**; cross **90,812 / 90,424 / 0** | both ends |
| **P12** | `mismatch` anywhere | **0**. A `mismatch` is an alarm, not a gap |

**Any byte that moves = the lane FAILED, named, and not rationalized.**

## 4. Tests

| # | registered |
|---|---|
| **P13** | base **1,619** passed / **42** targets (runner); `git grep -c '#[test]' -- 'crates/*'` base **1,629**. Tip **1,619 + N** with **N ≤ 24**, **targets 42**, 0 failed. **This is a CEILING with NO discount factor applied** — five of the six times a discount was applied on this project it was the error |
| **P14** | the grep delta equals the runner delta, and the grep runs a **constant +10** ahead at both ends — `#3076`'s **ninth** reproduction |

## 5. Mutants

**A construct rung that moves call sites with no red mutant is not graded.** The
bar is `w-item-d`'s 34-red off-by-one-word, `w-layout`'s eight across six
independent real-obj controls, `w-fencea`'s eight of nine with one registered
green in advance.

| # | registered |
|---|---|
| **P15** | **≥ 6 mutants**, **≥ 3 red on a REAL obj** graded by real `c2.dll` under wibo. The real-obj oracle is `src/xdk/xjson/jsonwriter.cpp`, one of the 878-TU scan's 25 `match` TUs, plus the tracked fixture `wjson_utf8_copy.cpp` at `/O1` |
| **P16** | **`M-G` is registered GREEN in advance**: permuting the **declare** order away from the emission order must move **no byte**. `declare` mints an identity and nothing else; if this reddens, `BodyLayout` has a hidden dependence on declaration order and the migration is not the re-expression it claims to be |
| **P17** | **`M-B`, an off-by-one-word back edge** (`Lcont`'s `bc` names the block after `Lloop`), reddens a real obj — not merely a unit test |
| **P18** | a **separating control** is green under each red mutant: a TU that is `match` at base and stays `match` under the mutation, so "red" is not "the harness fell over" |

## 6. Scope — `CFG_SHAPE.md` §6.3, all binding

**No code motion. No cost model. No loop rotation. No CTR-loop discovery. No
neutrality classifier. No instruction scheduling. The relaxation pass is NOT
built** — `w-item-d` and `w-fencea` both declined it and `w-layout`'s `LY-a`
records that it *"now has somewhere to stand and is still declined: no corpus
body is 32 KB"*.

**Ownership.** `crates/c2-core/src/codegen/` is this lane's. `crates/c2-il` is
peer `w-stmt5`'s and is **read, never written**. `crates/c2-core/src/coff/` is
**off-limits**, single-occupancy. `codegen::labels` remains the **single reader
of a pending intra-section branch site** — no second fixup list. No shared
predicate is narrowed, shadowed or redefined; `LabelMap` has **nine** clients
and a change to its invariants changes all nine.

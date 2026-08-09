# w-fence2 — PREREG

**FROZEN before the first `crates/` change, before the first probe cell and
before the first fixture line.** Committed as its own commit; nothing below is
edited afterwards. Scored in the rung.

    Lane:      w-fence2   (board rows #2470–#2499)
    Branch:    worktree-agent-a65d37d683c307542, off master `acb151ed` (the wb-label merge)
    Worktree:  .claude/worktrees/agent-a65d37d683c307542
    Scratch:   work/w-fence2/

---

## 0. WORKLOAD STAMP — re-derived, not quoted

The dc3 tree is **not pinned** (#2392: it moved 23 commits mid-lane once and
shifted `fnbyte-denominator` by 9.4 %). Everything below is from **this lane's
own** 878-TU scan, `work/w-fence2/scan_base.{out,jsonl}`, taken before any
change:

    c2-rs      acb151ed084e4693275677bb2d47ec50d400beb2  (clean)
    workload   d7a3c1aa9d5d57a1176790c0e15a723edd2e03a0  (clean)
    binary     07052c44c1b89213dbf1bd71f64dea6c
    cl.exe / c2.dll / c1xx.dll   X360 16.00.11886.00
    wibo       1.2.0-c2rs.1
    flags      /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I …   (workload's own)

| base metric | value |
|---|--:|
| **TU match** | **19** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 |
| vocab-gap · capture-fail | 852 · 7 |
| `fnbyte-exact` | **35,793** |
| `fnbyte-differs` | 1,898 |
| `fnbyte-refused` | 114,649 |
| `fnbyte-denominator` | 162,092 |
| `fnbyte-tus-full` (`e == d`) | 27 |
| per-function census | 712,280 / 2,463,470 (28.91 %) |
| emitted census | 39,226 / 162,092 (24.20 %) |
| factor-a · -b · -c · -d · -e | 28 · 338 · 169 · 20 · 2 |
| FRONTIER | 7 |
| `gap-metric` keys | (diffed as a map at the tip) |

**Note the deltas against every inherited digit**: `w-vsnprnc` reported
`fnbyte-exact` 35,791 and `fnbyte-denominator` 178,977 at its tip; both moved
before this lane started. **Nothing in this PREREG is quoted from another
lane's table.**

### 0.1 The T1 sweep, at this lane's base (deliverable 4, re-run)

`work/w-fence2/t1.py`, `w-nc`'s own detector
(`fnbyte-denominator > 0 ∧ exact == denominator ∧ class != match`):

    T1  ALL-EXACT-NO-MATCH : 2   src/system/math/vec.cpp        (exact 2/2, vocab-gap)
                                 src/xdk/LIBCMT/vsnprnc.cpp     (exact 2/2, vocab-gap)
    T1b ZERO-BYTE          : 8   src/system/decomp_pch.cpp + 7 capture-fail TUs

`w-nc` measured the byte-distance-zero population at **2** (`vec.cpp`,
`decomp_pch.cpp`) *before* `w-vsnprnc` made `vsnprnc.cpp`'s two functions
byte-exact. It is **3** now (2 T1 + 1 real T1b), and `vsnprnc.cpp` is the new
member — the one this lane is commissioned to convert.

---

## 1. THE HYPOTHESIS

`src/xdk/LIBCMT/vsnprnc.cpp` is `fnbyte-exact 2/2` and still `vocab-gap`.
`w-vsnprnc` §5 isolated the refusal four ways: **`IlBundle::functions`' wholesale
inline fence** — *any* callee this TU also defines refuses the whole TU —
because `vsprintf_s` tail-calls `_vsprintf_s_l`, which this TU defines
(`work/w-fence2/il-vsnprnc/*.gl`: both records carry linkage byte `05`,
defined-external; `.ex` splits `4F 1F` at 3066 and 3602).

> **H — the port already owns BOTH halves of the inline question at the
> composition seam, and the parser's wholesale refusal is the only thing
> stopping it being asked.** `c2_core::comdat::fenced_inlined_callee`
> (`w-inlfence2`, §10.29.1) refuses any composed body that relocates against a
> locally-defined callee the port can lower whose lowered body is `<= 64` bytes
> (`splice::INLINE_UNBOUNDED_BYTES`) — the categorical **accept** region, used as
> a refusal. `c2_core::splice` performs the expansion in the same region. What is
> missing is the **decline** region: a bound `T > 64` above which c2 is measured
> never to inline, so that the seam refuses everything **between** the two and
> the port may keep its call above `T`.

The narrowing is therefore two edits and one measured constant:

* **`c2-core`** — `fenced_inlined_callee` fires when the callee is lowerable and
  its lowered body is `<= T` (was: `<= 64`). Strictly MORE refusing at the seam.
  Direction: acceptance → refusal, which cannot produce a wrong obj.
* **`c2-il`** — `IlBundle::functions`' wholesale clause is narrowed so a
  locally-defined callee no longer refuses the TU **when the facts the seam needs
  are available**: the callee is bound by the same total `per_record` binding to
  one of this TU's own `.ex` segments, and both caller and callee segments carry
  an optimization word naming a mode the port emits (`OptWordMode::O1`/`Ox`).
  Direction: refusal → acceptance, and **only** where the seam above is live.
* **`T`** — `INLINE_DECLINE_BYTES`, to be **measured** (§2) on the decline side
  only. Not fitted to `vsnprnc`.

**The safety argument, stated in advance so it can be graded:** the emit path
stays total because (i) `Bindings::per_record` yields nothing unless the bound
records are 1:1 with the `.ex` segments, so a TU that binds has a complete
defined-name list; (ii) every locally-defined callee is by construction one of
this TU's own functions, and `PortC2::build` composes **every** function in the
TU, so a callee the port cannot lower fails the whole TU before an obj exists;
(iii) `fenced_inlined_callee` runs inside `comdat_body_from_selected`, which both
`build` and the FBM instrument call, so no body reaches an obj without being
asked.

---

## 2. THE MEASUREMENT — decline side only

`WB_INLINE_FINDINGS` §7 forbids the accept side and offers five **decline** rows.
This lane will re-derive the boundary on the workload rather than adopt a
bracket, with a scratch instrument on `gap/fnbytes.rs` (applied, measured,
**reverted**, committed as a `.patch` and never as a `crates/` change; board
**#1380** — commit first, then apply):

> **GRID-W.** For every (caller, locally-defined callee) call site the port
> composes a body for, cross the callee's own **reference** `.text` COMDAT size
> against whether the **reference caller's own relocation set names the callee**
> — i.e. whether c2 KEPT the call. `kept` is the decline side, directly, with no
> inference from the port's own correctness.

`T` is then chosen **strictly above** the largest callee size at which any
`inlined` (not-kept) site is observed, with a stated margin, and never fitted to
`_vsprintf_s_l`'s 152 bytes. If no such `T` below 152 exists, the TU is
**DECLINED** and the price is reported (§5, C1a).

---

## 3. THE DECLINE CLAUSES, each SIZED IN ADVANCE

| # | declined | size |
|---|---|---|
| **D1** | **the accept side of the inline predicate** — any rule that makes the port PERFORM an expansion it has not already got (`splice` keeps its own S1–S9 exactly as shipped) | `WB_INLINE_FINDINGS` §7: *"The accept side is not offered."* This lane adds **0** new expansion |
| **D2** | **the STATIC linkage class** — `WB_INLINE_FINDINGS` F1's `(300,308]` ceiling. Not taken as a second, higher `T` | sized at the tip: the number of workload call sites whose locally-defined callee's `.gl` linkage byte is `03`. Predicted **small**; measured and reported either way |
| **D3** | **the favour-speed ceilings** `(212,252]` / `(156,164]` (F1/F2 at `/O2`, `/Ox`) | `T` is registered as an **`/O1`-only** constant. Mode gate in the PARSER (#1638). The `/O2`, `/Ox`, `/Od` gate lanes are the executable form; sized at 6 lanes × 325 fixtures |
| **D4** | **`/Ob0` ⇒ nothing inlines** (F3, 34 cells) | converts 0: `/Ob0` alone yields opt word `00800005`, which `opt_word_mode` refuses, and `/O1 /Ob0` can only make the decline rule MORE right |
| **D5** | **varargs callee ⇒ never inlined** (F5) | already spent: `comdat.rs` reads it off the mangled name (`ends_with("ZZ")`) and `Bindings::is_varargs` refuses a defined variadic name. **0** new |
| **D6** | **direct recursion ⇒ never inlined** (F5) | already spent: `callee_is_one_c2_expands` compares by address. **0** new |
| **D7** | **the budget and the POGO cost model** | `WB_INLINE_FINDINGS` §4.1/§4.2 record them READ, NOT CONFIRMED; no row proposes them. **0** |
| **D8** | **closing the `fnbyte` residue the seam still cannot see** — the `port = none` callees (`w-inlfence2` §3.3: 1,081 byte-exact functions call one) | not this lane's: it needs a lowering for 81–308 B bodies. Sized at 1,081 by `w-inlfence2`, re-derived at this lane's base if the scratch reaches it |
| **D9** | **rewriting `w-inlfence`'s, `w-inlfence2`'s or `w-vsnprnc`'s rungs** | held. Their assertions are inverted in place with a dated comment where a measurement refutes them; the rungs are dated records |

---

## 4. PRE-ARMED REFUSALS — one unnamed refusal budgeted

1. **FENCE ORDER / CLAUSE REACHABILITY.** `w-inlfence` §10.2's finding is that *a
   new refusal key being non-zero says the clause runs, never that it should
   have*. Every `_neg` cell here must have its base verdict taken as a
   **counterfactual against a binary built at master**, and the cells must be
   ordered so the clause under test is the one that fires.
2. **`_neg` CELLS THAT MEASURE THE WRONG THING** (board #2085; `w-inlfence` §5
   hit this twice in one afternoon). Every negative cell must carry a
   probe-verified DISTINCT key, and one positive control per file.
3. **THE FIXTURE LIST.** Regenerate `ls fixtures/cpp/*.cpp` **after** the last
   fixture and `wc -l`-check it (`w-fltret` §9.2's third unnamed refusal was a
   313-entry list that omitted its own `_neg` file).
4. **BOARD #1380 — commit before any revert.** Every scratch application is
   preceded by a commit of all real work.
5. **`body.calls` COMPLETENESS.** The seam iterates the composed body's calls.
   If any call carrier (`cond_pair`'s two arms, `if_call_join`'s two callees, a
   `CallSeq`'s list) does not appear there, the seam is not total and the parser
   narrowing is unsound. **To be proven by a `#[test]`, not assumed.**
6. **PEER SESSIONS.** Master is advancing concurrently and one lane's board range
   was taken out from under it. Re-check `git log` before staging; rebase before
   reporting; audit commits not authored here.

---

## 5. PREDICTIONS — registered, with probabilities

No discount factor is applied (standing rule).

| # | prediction | p |
|---|---|--:|
| **C1** | **`src/xdk/LIBCMT/vsnprnc.cpp` CONVERTS: TU match 19 → 20** | **0.55** |
| **C1a** | the complementary branch — it is DECLINED because no `T < 152` clears the measured inlined population with margin | 0.45 |
| **C2** | **`fnbyte-exact` delta is exactly 0** | **0.55** |
| **C2b** | `fnbyte-exact` delta is in `[-5, +5]` | 0.85 |
| **C2c** | `fnbyte-exact` **does not fall** (delta ≥ 0) | 0.80 |
| **C3** | **0 of `w-fltret`'s 444 are re-admitted**, by name | **0.92** |
| **C4** | mismatch stays **0** at all three levels — 878 TUs, 325 fixtures × `/O1` **and** `/Ox`, 18 gate lanes, sweep and cross | 0.96 |
| **C5** | the T1 population at the tip is **1** (`vec.cpp`), i.e. exactly `vsnprnc.cpp` leaves it | 0.55 |
| **C6** | raising the seam's bound from 64 to `T` moves ≥ 1 function from `fnbyte-differs` to `fnbyte-refused` | 0.70 |
| **C7** | it moves **0** from `fnbyte-exact` to `fnbyte-refused` | 0.80 |
| **C8** | **`#[test]` DELTA is +7**, `±3` (targets 38 → 38 or 39) — **calibrated DOWN**: four consecutive lanes over-estimated this in the same direction (w-vsnprnc registered +14 and got +7) | 0.60 |
| **C9** | GRID-W finds ≥ 1 site where c2 INLINED a callee larger than 80 emitted bytes (i.e. `w-inlfence2`'s ~80 B separation is not the true ceiling) | 0.50 |
| **C10** | the largest callee c2 is measured to INLINE anywhere on the workload is **< 152 bytes** | 0.60 |
| **C11** | the Phase-7 factor model's false positive (`A∧B∧C∧(D∨E)` = 20 against a match set of 19) is **removed** by this lane, i.e. the joint and the match set agree at the tip | 0.50 |
| **C12** | the scan's ALARM half — matching TUs outside some factor — stays **0** | 0.97 |
| **C13** | ≥ 1 unnamed refusal fires at a **pre-armed** place in §4 | 0.65 |
| **C14** | `fn_gate_refusals` is 0 keys at both ends | 0.85 |
| **C15** | no `gap-metric` key vanishes | 0.85 |

**Registered direction: OPTIMISTIC on C1** (this is a lane commissioned to
convert one named TU, and a lane that expects to convert usually does not).
Board #770.

---

## 6. NEUTRALITY, at three levels — the plan, registered

1. **878 TUs BY NAME**, base vs tip, each scanned with **its own binary** (the
   base one from a `git checkout master -- crates fixtures` round trip with
   `git status` clean at both ends). Set comparison, not a count. **The
   DIRECTION of every moved verdict is stated**: a fence narrowing may move a
   verdict toward acceptance only where the bytes already match, and may move
   `vocab-gap → codegen-gap` (a TU that now decodes and is refused later) — which
   is reported, not netted out.
2. **Every `gap-metric` key diffed as a key→value MAP** (not by `diff`): keys
   vanished / appeared / changed, with every changed key accounted.
3. **All fixtures at `/O1` AND `/Ox`**, both binaries, the list regenerated
   **after** the last fixture and `wc -l`-checked, compared per TU by name.

Plus the full gate (18/18, 0 mismatch anywhere), `expr_sweep`, `mode_cross`,
`cargo test --workspace --release --no-fail-fast` (#2262), `board_audit.sh`,
`rung_registry`, and `c2rs selftest` green.

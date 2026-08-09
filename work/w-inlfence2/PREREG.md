# PREREG — lane `w-inlfence`: make the port REFUSE what c2 inlines

    Lane:      w-inlfence          <-- LANE-NAME CLAIM, board #2096's prevention
    Kind:      CORRECTNESS (a fence; a refusal, not a conversion)
    Date:      2026-08-09
    Branch:    wt-w-inlfence
    Base:      master 0faa855a (merge w-fltret2; 8dd1a577 w-callprice is an
               ancestor of it, checked with `git merge-base --is-ancestor`)
    Rows:      #2150-#2169  (verified free: the highest minted row on master is
               #2214, and 2142-2199 / 2120-2129 are unminted gaps; no `**2150**`
               .. `**2169**` row exists in docs/BOARD.md)
    Rung:      docs/rungs/2026-08-09-w-inlfence.md
    Scratch:   work/w-inlfence/          (never /tmp)

**IF A PEER SESSION IS READING THIS: THE NAME `w-inlfence`, ROWS #2150-#2169,
THE RUNG FILENAME AND `work/w-inlfence/` ARE CLAIMED BY THIS COMMIT.** Board
#2096 cost two sessions four namespaces on 2026-08-09 because nothing claimed a
name; this block is that row's stated prevention, discharged in the first
commit. `git log master --oneline | grep -i inlfence` was empty at freeze.

**FROZEN BEFORE THE FIRST BUILD, THE FIRST SCAN AND THE FIRST LINE OF
`crates/`.** Nothing below has been measured. §0 states exactly what was read,
so that nothing here can be mistaken for a blind prediction.

---

## 0. The honest boundary — what was already READ at freeze time

Read in full from the repo before this file was written (reading committed
source and docs is not measuring; every *number* this lane reports will be
re-derived):

* `CLAUDE.md`, `docs/STATUS.md` (including its generated block, collected at
  tree `154c8580`),
* `docs/rungs/2026-08-09-w-fltret2.md` in full,
* `docs/whitebox/WB_INLINE_FINDINGS.md` in full,
* `docs/BOARD.md` rows #2096 and the row-number census,
* **and, decisively, the port's own source**: `crates/c2-il/src/func/bundle.rs`
  around `IlBundle::functions`, `crates/c2-il/src/func/bind.rs`
  (`Bindings::names`), `crates/c2-il/src/func/mod.rs`
  (`IlFunction::callees`, `SeqTail`), `crates/c2-core/src/codegen/select.rs`
  (`select_function`), `crates/c2-core/src/splice.rs` (`TuContext`),
  `crates/c2-harness/src/gap/fnbytes.rs` (the FBM partition).

**So the following are KNOWN at freeze and are NOT predictions:**

1. `crates/c2-il/src/func/bundle.rs:1854-1870` **already contains a whole-TU
   same-TU-callee refusal**, in these words — *"A callee that is also DEFINED
   here is out of class: c2 may inline it, and the port cannot"* — implemented
   as `funcs.iter().any(|f| f.callees().any(|c| names.iter().any(|n| n == c)))
   → return None`, where `names` is `Bindings::names()`, the `.gl`-defined
   names bound to `.ex` segments.
2. `IlFunction::callees()` enumerates `tail_call`, `framed_call`,
   `call_seq.calls`, `cond_pair`, `if_call_join`, `guard_chain_shared_tail`,
   `alloc_init_or_fail`, `osf_handle_guard`, `xlrc_create_guard`. The float
   value tail `SeqTail::CallValueFp` carries **no callee of its own** — it
   returns the *last* `call_seq.calls` entry's result — so on a plain reading
   the w-fltret class's callee IS in `callees()`.
3. `select_function(func, mode)` — the **per-function** decision procedure that
   FBM (`gap/fnbytes.rs`) and the census run — takes **no TU context at all**.
4. `c2_core::splice::TuContext` **does** carry, per name the TU defines, the
   parsed `IlFunction` (or `None` when the parser refused it) and the callee's
   own `.ex` optimization word. `gap/fnbytes.rs` already builds one.
5. `WB_INLINE_FINDINGS.md` §7 asserts *"`IlBundle::functions()` refuses any TU
   where a callee is also defined"* — consistent with (1).

This creates the lane's central tension, and it is registered rather than
resolved: **fact (1) says the 444 should already be refused, and w-fltret2
§9 says all 444 are same-TU.** Both cannot be true of the same predicate. §2
registers which way it breaks.

**Not touched at freeze:** every number. No `cargo build`, no scan, no probe,
no `cl.exe`. The toolchain is not yet linked into this worktree (`compilers/`
absent, `wibo` not on `PATH`) — linking it is the first post-freeze act.

---

## 1. Direction

Registered **PESSIMISTIC about reach, OPTIMISTIC about the fence being
buildable.** Board #770's tally runs optimistic; the correction here is to
register that **this lane most likely converts nothing and removes something**,
and that the honest deliverable is a smaller wrong-emit surface, not a bigger
census.

**The asymmetry is stated in advance and every uncertain cell is resolved
toward refusal**: refusing too much costs *reach* (a number that is a driver);
refusing too little costs *correctness* (the only target). Where this lane
cannot tell whether c2 inlines, it refuses.

---

## 2. Predictions, in probability form

`p` is my credence **before** the first build.

### 2.1 The base, re-derived

| # | prediction | p |
|---|---|--:|
| **P1** | at base `0faa855a`: `fnbyte-exact` = **36,228** and `fnbyte-differs` = **2,555** | 0.85 |
| **P2** | at `05d743f7` (w-fltret's parent): `fnbyte-exact` = **36,228**, `fnbyte-differs` = **2,111** | 0.80 |
| **P3** | the newly-differing set, per `(TU, emit_name)`, is exactly **444** | 0.75 |
| **P4** | emitted census: **39,644** at base and **39,200** at `05d743f7` (STATUS's block figure) | 0.75 |
| **P5** | of the 444, **≥ 99 %** have a callee this same TU also defines | 0.85 |

### 2.2 The crux — what the port can SEE about a callee at accept time

| # | prediction | p |
|---|---|--:|
| **P6** | **callee DEFINEDNESS is visible at accept time**, and needs no new IL reading: it is `Bindings::names()` ∩ `IlFunction::callees()`, already computed | **0.92** |
| **P7** | callee **BODY SIZE** is *not* visible without lowering the callee, and the port can lower it only when the callee is itself in class — so a size-conditioned fence is available on a strict subset and must refuse the remainder | 0.75 |
| **P8** | **the whole-TU fence at `bundle.rs:1865` DOES fire on the 444's TUs**, so the 444 are a *census + FBM* defect and there is **no live wrong-obj liability** from them today | **0.55** |
| **P9** | (the complement) the whole-TU fence has a **hole** the 444 pass through — a callee not enumerated by `callees()`, or a name `Bindings::names()` does not carry | 0.45 |
| **P10** | whichever of P8/P9 holds, **the per-function seam (`select_function` / the census) has NO same-TU-callee fence**, so the census's emitted column over-claims | **0.90** |

P8 and P9 are exhaustive and mutually exclusive. **Registering P8 at only 0.55
is the point**: I have read the code that should refuse these and I still do
not know that it does, because two published lanes assert the opposite. The
first measurement this lane takes settles it, and I am registering that I
expect to be surprised roughly half the time.

### 2.3 The fence, and what it costs

| # | prediction | p |
|---|---|--:|
| **P11** | a COARSE fence — *refuse any function one of whose callees this TU defines*, applied at the per-function seam — drops `fnbyte-differs` by **≥ 400** | 0.65 |
| **P12** | …and by **≥ 444**, i.e. it reaches into the base 2,111 as well | 0.50 |
| **P13** | the coarse fence also drops **`fnbyte-exact`**, because some same-TU-defined callees are ones c2 did **not** inline (too large) and the port's `bl` is right today. `fnbyte-exact` falls by **≥ 1** | 0.60 |
| **P14** | …and the exact loss is **≤ 25** | 0.55 |
| **P15** | …and **≤ 5** | 0.30 |
| **P16** | of the base **2,111**, the fence reaches **≥ 200** | 0.50 |
| **P17** | …**≥ 500** | 0.30 |
| **P18** | …**< 100** | 0.25 |

P16/P17/P18 are a registered distribution, not a direction — w-fltret2 §9.2's
instruction (#2095) is that a conversion count is worthless unless it is
crossed with the oracle, and its mirror is that a *refusal* count is worthless
unless it is crossed with what the refusal costs. P13-P15 are that crossing.

### 2.4 Verdicts, and the moves that are INTENDED

**A refusal moving a verdict is the intended direction here**, so the expected
moves are named before they are measured. Anything not named below moving is a
finding, not a pass.

| # | prediction | p |
|---|---|--:|
| **P19** | **878 TUs by name**: the match set is the identical 18 names at base and tip. **Expected move: NONE.** | 0.85 |
| **P20** | `mismatch` is **0** at base and **0** at tip | 0.95 |
| **P21** | **all 251 gap-metric keys accounted**; the only ones that move are `fnbyte-differs` (**down**), `fnbyte-refused` (**up by the same amount**), the census emitted/in-class keys (**down**), and any new decline key this lane mints. `fnbyte-denominator` unchanged; `fnbyte-partition-broken` **0** | 0.65 |
| **P22** | **every fixture at `/O1` AND `/Ox`**: **no fixture moves `Match` → `not-implemented`.** This is a *discriminator for P8*: if the whole-TU fence already refuses these TUs, no fixture can move; a fixture that moves is evidence for P9 and must be investigated, not accepted | 0.70 |
| **P23** | `TU match` is **18 → 18** | 0.90 |

### 2.5 Method

| # | prediction | p |
|---|---|--:|
| **P24** | the fence is **≤ 40 lines** of `crates/`, in the parser (`c2-il`) rather than in `c2-core` — acceptance in the parser, #139 | 0.55 |
| **P25** | at least **one** must-fail mutation is available and fails as designed (deleting the fence restores the exact `fnbyte-differs` figure) | 0.85 |
| **P26** | **≥ 1 unnamed refusal** — some clause, gate or ordering this lane did not anticipate — costs it a measurement. Pre-armed at: FENCE ORDER (the fence must run before the accounting gate, or `unclaimed-gl-symbol` masks it), CLAUSE REACHABILITY (a `_neg` cell whose key is the same as its `_pos` twin's is confounded), and CENSUS/EMIT DISAGREE (`census disagree 0` is a live control and a parser-side fence that the codegen seam does not share will trip it) | 0.65 |
| **P27** | `#[test]` **DELTA** (not total) is **> 0** — this lane adds tests for the fence in both directions | 0.80 |

---

## 3. Decline clauses, with sizes

Each is a stated size at which this lane stops, so that "it did not work out"
cannot be written after the fact.

* **D1 — POPULATION.** If the re-derived newly-differing set at base is
  **< 300** (against 444), this lane declines to ship and reports the
  discrepancy instead. A fence priced on a population that does not reproduce
  is the ninth inherited-claim failure this week.
* **D2 — THE EXACT LOSS.** If the coarse fence costs **> 25** `fnbyte-exact`
  functions, the coarse fence is **not shipped**. This lane then prices the
  refined fence (WB_INLINE's decline side: a callee whose lowered body is
  **> 308 B** at `/O1`, or **> 80 B** and loop-bodied, is one c2 will not
  inline, so the port's own `bl` may stand) and ships that only if its exact
  loss is **≤ 5**. Above that, this lane ships nothing and reports both prices.
* **D3 — TU MATCH.** If `TU match` falls by **≥ 1**, revert. No amount of
  `fnbyte-differs` buys a TU.
* **D4 — MISMATCH.** If `mismatch` is nonzero anywhere — the 878-TU scan, the
  fixture gate, the sweep, the cross — revert immediately. This is absolute.
* **D5 — INVISIBILITY.** If definedness turns out **not** to be visible at the
  seam where the census decides, this lane does **not** guess. It states
  exactly which field is missing and prices the coarse alternative (refusing
  the whole value-tail-with-callee shape) in reach, and ships that or nothing.
* **D6 — GATE.** If `scripts/gate.sh` is not `GATE: PASS` with 0 mismatch at
  the shipping tip, nothing lands.
* **D7 — NO UNILATERAL REVERT.** If the arithmetic says the w-fltret class
  should be reverted, this lane **recommends** it with the numbers and leaves
  the decision to the coordinator. It does not revert a peer's landed work.
* **D8 — TOOLCHAIN.** If the toolchain cannot be linked into this worktree,
  this lane ships **no `crates/` change at all** and reports a code-reading
  result only. A fence that was never graded by real `c2` is not a fence.

---

## 4. What this lane will NOT do

* It will not ship an **accept**-side inline rule. WB_INLINE_FINDINGS §7 is
  explicit — *"The accept side is not offered"* — and `INLINE_PREDICATE.md`'s
  2.84 % residual is a wrong emit on the accept side. Every rule this lane
  ships makes the port emit **less**.
* It will not re-fit the cost model, quote the POGO tables, or touch
  `docs/whitebox/DISCLOSURE.md`.
* It will not widen `seq_call_arg_slots` or any reader.
* It will not hand-edit `docs/STATUS.md`'s generated block.

---

## 5. Scoring

Every prediction above is scored in the rung, HIT / MISS / HALF, with the
misses stated first. P8/P9 are scored as one exhaustive pair. P16/P17/P18 are
one registered distribution and at most one can hit.

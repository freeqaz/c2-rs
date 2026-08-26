# PREREG — `w-symbind`: is SYMBOL BINDING a third fused layer, and is it separable?

    Tag:       w-symbind
    Date:      2026-08-26
    Kind:      instrument rung
    Base:      f202268f6 (c2-rs master, clean — decision 14's own commit)
    Rows:      #3597–#3602 (reserved at dispatch; minted in the commit that uses them)
    Fixtures:  none — instrument rung: the `symbind-*` family in
               `crates/c2-harness/src/gap/symbind.rs`
    Census:    +0 — no acceptance predicate moves. `crates/c2-il` is READ, never
               written (`w-atend` owns it this wave). **This lane may not widen
               emission and may not convert one body of the population it measures.**
    Reach:     +0 emitted functions, +0 TUs — required, and it is not this lane's grade
    Funded by: `docs/DECISIONS_2026-08-22.md` decision 14, row `w-symbind`
    Status:    REGISTERED BEFORE ANY `symbind-*` NUMBER EXISTED

**The failure axis this instrument can fail on with every byte identical**
(`docs/rungs/README.md`, board **#3336**): **the fused-cell count.** An
instrument whose "symbol-binding layer" is empty, or whose separation cell is
the admission set wearing a new key name, has abstained rather than passed.
§5 states the number that must print and the threshold below which this lane
reports **FAILED**.

---

## 0. The question, stated so it can come back "no"

`w-decodereach` (wave 11, board **#3582**) measured, off the `decode_bodies()`
seam `w-unfuse` built:

    decode-reach-grammar          711,729
    decode-reach-admitted         707,728
    grammar-not-admitted            4,001   (admitted-not-grammar: 0)

decomposed in the same walk as

    callee-unresolved-tail-call:eof   2,282
    data-sym-unresolved:eof           1,665
    data-sym-not-extern:eof              52
    callee-defined-in-tu:eof              1
    data-sym-strlit-fenced:eof            1

All `:eof`. All refused at `shape_to_function` (`census.rs:957`), **downstream
of `AdmissionPolicy`**, which `Decoded` cannot see.

**The question: is symbol binding a third fused layer, and is it separable the
way the grammar layer was?**

Four sub-questions, in the order decision 14 asks them:

1. **Characterize the 4,001** — per key, per TU, per symbol; overlap with the
   emitted census, with `fnbyte-exact`, and with the reach strengths.
2. **Is the refusal about the BODY, or about the PROGRAM?**
3. **Is the layer separable the way the grammar layer was?** — and if so, what
   is the seam and what does it cost. **This lane is not authorized to build
   it.**
4. **Ship the measure** — a key a later lane can watch MOVE. `w-decodereach`'s
   own hard-won rule: **the signal is the CHANGE in a population, never its
   distance from 0.**

---

## 1. WHAT IS BELIEVED NOW, before this file existed

Read out of the tree, not remembered. **Every one is expected to move** — the
`dc3-decomp` workload moved seven times inside one lane (#3428) and moved again
between `b814d1db2` and `c13cebbca` (#3583).

| quantity | believed | source, and its staleness |
|---|---:|---|
| IL bodies in the workload | **2,417,794** | `STATUS.md` generated block, tree `c13cebbca` |
| … admitted | **707,728** (29.27 %) | same |
| `decode-reach-grammar` | **711,729** | `#3582`, tree `5a013b8f4` |
| **`grammar-not-admitted`** | **4,001** | `#3582` — **the population this lane characterizes** |
| `fnbyte-exact` | **35,912** | `STATUS.md` block |
| S0 `fnbyte-blind-attempted` / `-exact` | 113,557 / **388** | `rungs/2026-08-22-s0-blind-reach.md` §1 |
| TU `match` | **25/878** | `STATUS.md` block (the page's prose row still says 8 — quote the block) |

---

## 2. THE INSTRUMENT, named before it is written

New file `crates/c2-harness/src/gap/symbind.rs`; keys namespaced **`symbind-*`**;
its own `TuResult` map (**two maps cannot collide** — `#3554`'s lesson, and the
`cflow-residue` sweep that doubled a published number is why); its own printed
block; **never in `scripts/gate.sh`**; **licenses no emit**; `NO-RESULT` over an
empty scan. `docs/FUNCTION_BYTE_MATCH.md` §0's five properties, verbatim, as the
three gradients before it adopted them.

### 2.1 The seam it reads, and why it is not a new parse

`c2_il::Relax { sym_names }` — `IlBundle::census_functions_relaxed`'s
**existing** relaxation, level 1 (`name-from-gl`), already computed once per TU
in `gap::scan` for S0 (`gap/blind.rs`). Its whole content
(`census.rs:830–845`) is: when `bind.resolve(tok)` or `bind.resolve_data(tok)`
fails, supply `$blind$callee` / `$blind$data` **and change nothing else**.

So the pairing this lane publishes is

    STRICT verdict   ×   RELAXED verdict,   row by row, positionally

and its **fused** cell is *"the strict census refused this body, and supplying a
NAME — nothing else — makes the same predicate admit it"*. That is the size of
the symbol-binding layer as the existing seam can reach it.

**This is not `w-decodereach`'s tautology (`#3582`) and not `#3565`'s.** The two
verdicts come from two passes over the same segmentation through code neither of
which is defined as the other, and `Relax::level(0)` is pinned equal to the
incumbent by `c2_il`'s own `strict_relax_is_the_incumbent_census` — which makes
**level 0 a free identity control**: at depth 0 every `symbind-fused` cell must
be **0**, and at depth 1 it must not be. Both halves get a full workload scan
(`#3564`'s pair discipline — *a pin showing only that a parameter is inert is
equally consistent with the parameter being wired to nothing*).

### 2.2 The axes

| key | what it counts |
|---|---|
| `symbind-observable` | census rows walked — **the denominator, printed first** |
| `symbind-in-class` | strict admitted |
| **`symbind-fused`** | strict **Blocked**, relaxed **InClass** — **THE HEADLINE** |
| `symbind-residue` | strict Blocked, relaxed Blocked |
| `symbind-fused\|<strict key>` | which refusal |
| `symbind-fused-shape\|<relaxed shape>` | **which GRAMMAR was underneath it** |
| `symbind-fused-cross\|<strict>\|<relaxed shape>` | the 2-D cross — the "one phenomenon or several" answer |
| `symbind-residue\|<strict>\|<relaxed>` | what the relax seam does NOT reach |
| `symbind-missing\|callee\|data\|both\|neither` | which side of the binding was blind, off `$blind$*` in the relaxed `IlFunction` |
| `symbind-missing-sites` | placeholder SITES, so "one symbol" vs "many" is a number |
| `symbind-fused-grammar` / `-notgrammar` | crossed with `Decoded::reached_shape` (called, never copied) |
| `symbind-fused-frame` / `-model` | crossed with `decode::reach_of_cflow` / `modeled_of_cflow` |
| `symbind-fused-emitted` / `-notemitted` / `-unnamed` | crossed with `FnCensus::emit_name` |
| `symbind-fused-mangling\|<class>` | `gap::classify::mangling_class` of the refused function |
| `symbind-tu-bucket\|<range>` | per-TU concentration, sum-safe buckets |
| `symbind-tus-any` / `symbind-tus-scanned` | TUs carrying ≥1, over TUs walked |
| `symbind-relax-level\|<name>` | **which relaxation produced these numbers** (`DECODER`'s discipline) |

### 2.3 The controls — every one must be watched FAILING

| key | known answer | how it is made to fail |
|---|---:|---|
| `symbind-monotonicity-broken` | 0 | strict InClass ∧ relaxed Blocked. **Not by construction** — it is a claim that the relaxation only widens. Executed mutation in a unit test. |
| `symbind-partition-broken` | 0 | `in-class + fused + residue != observable`. Executed: a bucket that stops being written. |
| `symbind-census-desync` | 0 | `relaxed.len() != census.len()`, or `rc.index != c.index` — fails closed for the TU (`#918`). |
| `symbind-placeholder-none` | 0 | a **fused** row whose relaxed body carries no `$blind$*` anywhere the public accessors can see. Nonzero is a FINDING, not an alarm, and it is printed as one. |
| `symbind-fused` at `C2RS_BLIND_LEVEL=0` | 0 | the identity arm — a whole second workload scan. |

**And the thing this instrument CANNOT see, printed with its denominator**
(`#3470`, `#1002`): the identity of the missing symbol. `FnCensus` publishes no
such field, `Decoded` deliberately exposes no `shape()`, and `crates/c2-il` is
`w-atend`'s fence. The relaxed `IlFunction`'s `callees()` / `data_syms` see the
callee and data-symbol sites and **nothing else**, so a placeholder reachable
only through a third field is invisible — which is exactly what
`symbind-placeholder-none` counts. Named as owed, not smuggled.

---

## 3. PREDICTIONS — falsifiable, with confidence, registered before the scan

**P1 — re-derivation.** `symbind-fused-grammar + symbind-residue-grammar` equals
`decode-reach-grammar-not-admitted` on the same scan, and that figure is
**4,001 ± 200** at this tree. **p = 0.80.** (The band is the workload's, not the
instrument's.)

**P2 — THE BRACKET: how much of the layer the existing seam separates.**
`symbind-fused ≥ 0.85 × grammar-not-admitted`, i.e. **≥ 3,400 of ~4,001**.
**p = 0.60.**
Reasoning, from the source: the three `data-sym-*` keys (1,718) are *raised by*
the resolver the relaxation replaces, so they relax by construction of the seam;
`callee-unresolved-tail-call` (2,282) is the **default arm** of `census.rs`'s
`match label` — `callee_unresolved_sites.rs` says so in those words, *"whatever
no earlier arm claimed"* — and `shape_to_function` has `?` routes that are **not
name lookups at all**. The band's whole uncertainty is how many of the 2,282 are
structural.

**P3 — and it will NOT be all of it.** `symbind-residue > 0`. **p = 0.55.**
The two fence keys (`callee-defined-in-tu`, `data-sym-strlit-fenced`) are asked
*after* `shape_to_function` succeeds, so relaxing the resolver **cannot** clear
them — it can only move a body into them. A registered MISS here (residue = 0)
would mean the whole layer is exactly the resolver, which is a cleaner answer
than the one predicted.

**P4 — `callee-unresolved-tail-call` is SEVERAL phenomena, not one.**
`symbind-fused-shape` carries **≥ 4 distinct relaxed shape labels**
(**p = 0.75**) and `symbind-fused-cross` **≥ 6 distinct cells** (**p = 0.70**).
Sharper form: **the `callee-unresolved-tail-call` row alone spans ≥ 3 relaxed
shapes** (**p = 0.65**) — because `sym_fail`, the probe that files the three
`data-sym-*` keys, is computed **only** for `BodyShape::MultiArgTailCall`, so a
body of any other shape whose *data* symbol failed is filed under a key naming a
CALLEE. If that is right, the key's name is wrong for part of its own
population, and the number says how much.

**P5 — spread, not concentrated.** ≥ **100** TUs carry at least one fused row
(**p = 0.60**), and **no single TU carries more than half** of them
(**p = 0.85**).

**P6 — which side of the binding.** `symbind-missing|data` (data-only) exceeds
`symbind-missing|callee` (callee-only). **p = 0.50 — a deliberate coin-flip**, so
the miss is as informative as the hit.

**P7 — the judge has already spoken on part of this population, and it does NOT
license an emit.** `fnbyte-blind-exact|<one of the five keys>` is **> 0** on this
scan — i.e. some symbol-binding-refused bodies already compose to c2's exact
BYTES behind a placeholder. **p = 0.70.** Read off `gap/blind.rs`'s existing key
(one fact, one locator — this lane recomputes nothing). **It is BYTES ONLY**:
the relocations would be against a name the seam invented, so this is a
measurement and never a reason to admit anything.

**P8 — the answer to "is it separable" is PARTIAL, and I register the shape
before measuring it.** A *bypass* exists (`Relax`) and a *layer* does not:
`shape_to_function` fuses **resolution** with **construction** in one pass, so
its `None` conflates "the symbol did not resolve" with "the model has no
carrier", and the census recovers the reason by asking the same question a
SECOND time (`sym_fail`, `bind_refusal_key`, both computed *before* the call
because it consumes the shape). Registered numerically: **≥ 15 distinct `None`
routes in `shape_to_function` mapped onto ≤ 9 published keys** (**p = 0.70**),
counted from source in §4 of the rung.

**P9 — the refusal is about the PROGRAM.** ≥ 90 % of `symbind-fused` is refused
for a property of what surrounds the body rather than of the body
(**p = 0.75**), operationalized as: supplying a name from outside the body flips
the verdict, and the body's own bytes were never consulted.

**P10 — the identity arm.** At `C2RS_BLIND_LEVEL=0`, `symbind-fused` reads
**exactly 0** and `symbind-residue` reads exactly `observable − in-class`.
**p = 0.90.**

**A registered MISS reported plainly beats a smoothed hit.** Wave 11 produced
several and every one was reported; #3562 is the model.

---

## 4. WHAT WOULD MAKE THIS LANE REPORT `FAILED`

Frozen here, before the numbers:

* **`symbind-fused` reads 0** on the level-1 scan. The instrument would then be
  measuring nothing, or measuring admission under a new key name — `#3336` at
  program scale, which is the failure the whole wave-11/12 line exists to
  prevent. **FAILED, in those words.**
* `symbind-observable` reads 0 → **NO-RESULT**, no key published, and no
  sentence with a ratio in it.
* Any control above reads nonzero and the lane cannot say why → **FAILED**.
* `scripts/gate.sh` prints anything other than `GATE: PASS` (read the **verdict
  line**, never the exit code — both failure shapes appeared in wave 11).

## 5. WHAT THIS LANE MAY NOT DO

* **May not widen emission.** Not one body of the 4,001 may be converted. Under
  `PROGRESS_METRIC.md` a wrong emit scores strictly below the refusal it
  replaced.
* **May not write `crates/c2-il`** (`w-atend`), **`scripts/**`** or
  `crates/c2-harness/src/perf.rs` (`w-perfstep`), or **`docs/whitebox/**`**
  (`w-opclass`). Anything this lane would have written to `docs/whitebox/` is
  reported in the rung instead.
* **May not build the seam §3's P8 describes.** That is a construct rung and a
  separate funding decision.
* **May not put a `symbind-*` key in `scripts/gate.sh`**, ever.

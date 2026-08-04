# Pre-registration — lane w-front: the 17-TU codegen FRONTIER

Committed **before the first line of implementation code** and before any
conversion attempt. Base: `wt-w-front` off master `5e278f0`.

## 0. Disclosure about this prereg's own timing

The brief orders "run the scan first and **read the FRONTIER by name**. Do not
work from any list in the docs." So this file is written **after** two
diagnostic reads:

1. one warm 878-TU `c2rs gap` scan on my own base, which printed the frontier;
2. `c2rs census` over each of the 17 frontier TUs, which printed the per-function
   blocker, control-flow class and EH class.

Both are *reads of the incumbent*, not outcomes of anything I built. No source
file has been edited. Saying so here is the point: a prereg that hides which of
its inputs were already measured is worth less than one that names them.

## 1. Bias, stated in writing

**I want to convert a TU.** The frontier is advertised as "the cheapest
conversions available anywhere in the project" and a lane that lands zero looks
like a lane that did nothing. The two concrete ways that pull goes wrong:

* **Reading my seam permissively.** My assigned seam is `crates/c2-core`'s
  codegen modules; `crates/c2-il` belongs to lane w-r2. §3 below records that
  every blocker on the frontier is raised *inside `crates/c2-il`*. The tempting
  move is to decide that `c2-il/src/func/body/shapes/` is "really" codegen and
  edit it anyway. I commit in advance: **no file under `crates/c2-il` is
  edited by this lane**, and if a conversion needs one it is reported as a
  cross-seam item and left.
* **Building the CFG anyway.** The brief defers the block/instruction IR
  explicitly. §3 shows 15 of 17 frontier TUs need it. The pull is to "just do a
  small one". I commit in advance: **no branch/label lowering is added to
  `codegen`**, and no restructuring of `select_function` into a general
  IL→lower→emit pipeline.

Weaker bias: I would rather report a number than a list.

## 2. The incumbent, re-measured on my own base

dc3-decomp bracketed `86357b5846c848a9d3cac46ace67bad099fbb598` **before and
after** the scan; the scan's own provenance record says
`workload_head 86357b58…`, `workload_dirty false`, `c2rs_head 5e278f01…`,
`c2rs_dirty false`.

| | |
|---|---|
| 878-TU scan | match **8**, mismatch **0**, codegen-gap **0**, vocab-gap **863**, capture-fail **7** |
| FRONTIER | **17** TUs |
| census | 706,402 / 2,463,318 (28.68%) |
| emitted census | 38,457 / 178,972 (21.49%) |
| census/gate disagreement | **0** |
| `cargo test --workspace --release` | *(to be taken; incumbent 665 passed / 24 targets)* |
| `scripts/gate.sh --jobs 6` | *(to be taken; incumbent 12/12, 2,592 verdicts, 0 mismatch)* |

## 3. What I measured before predicting

The frontier's 17 TUs carry **35 blocked functions** between them. Classified by
the decode-only control-flow axis:

| cflow class | blocked fns |
|---|---|
| `cflow-straight` / `cflow-straight+expr-modeled` | **8** |
| `cflow-if-1` | 5 |
| `cflow-if-2` | 1 |
| `cflow-if-n` | 10 |
| `cflow-loop` | 8 |
| `cf-expr-0x05` (body did not decode end to end) | 3 |

A TU converts only when **every** one of its blocked functions converts. Exactly
**two** frontier TUs have all their blocked functions in the straight-line rows:

* `src/xdk/nuispeech/xboxheap.cpp` — 1 fn, `expr-op-0x27`, `cflow-straight`,
  `eh-none`.
* `src/Main.cpp` — 1 fn, `param-width-undetermined:mid`, `cflow-straight`, but
  **`eh-state1`** (needs the whole EH record).

The other **15** each contain at least one `cflow-if-*` / `cflow-loop` /
`cf-expr-0x05` function. There is no ordering of leaf-shape widenings that
converts any of them.

Second measured fact, from the scan's own block: **census/gate disagreement is
0** — `c2_core::codegen::function_gate` accepts every function `c2-il` puts in
class. Every one of the 35 blockers is therefore a refusal raised by the *IL
shape recognizer* in `crates/c2-il/src/func/body/`, not by anything in
`crates/c2-core/src/codegen/`. My seam contains no live refusal on the frontier.

## 4. Predictions

Scored in the rung, right or wrong.

| # | prediction |
|---|---|
| **A1** | Final TU match = **8**. I predict **zero** conversions from inside this lane's seam. |
| **A2** | mismatch stays **0** everywhere — scan, gate, fixtures. Non-negotiable. |
| **A3** | Census delta = **0 exactly**, and the emitted-census delta = 0. Nothing this lane does changes acceptance. |
| **A4** | Fixture-verdict count stays **2,592** with no verdict changing; `cargo test --workspace --release` stays at 665 passed / 0 failed / 24 targets. |
| **B1** | **No frontier TU is one leaf-shape widening away except `xboxheap.cpp`.** Falsified if any other frontier TU turns out to have all blocked functions straight-line. |
| **B2** | `xboxheap.cpp` is one `expr-op-0x27` widening away — *and that widening is entirely inside `crates/c2-il`*, i.e. not takeable here. Falsified if the widening turns out to need a `c2-core/codegen` change I am allowed to make. |
| **B3** | `Main.cpp` is **not** one widening away even though it is straight-line: `eh-state1` means the EH record must be modeled too. Falsified if `param-width-undetermined:mid` alone would convert it. |
| **B4** | The single highest-leverage widening over the frontier's 35 blocked functions is `expr-cmp-eq` (**14** functions across 7 TUs) — but because 13 of those 14 sit behind `cflow-if-*`/`cflow-loop`/`cf-expr-0x05`, granting it alone converts **0** TUs. Registered because "the biggest bucket" and "the cheapest conversion" are different questions and this lane exists to separate them. |
| **C1** | The lane's deliverable is the per-TU/per-function blocker table plus the cross-seam finding, not a conversion. Registered so a zero is not retro-narrated as a shortfall — the brief names it a success and I am taking it at its word. |

## 5. Binding decline clauses

Any one ⇒ **DECLINED**, whatever else the lane achieved.

1. TU match **< 8** (the named incumbent).
2. **mismatch > 0 anywhere** — scan, gate, sweep, or a fixture. *A TU landing
   `mismatch` is the single worst outcome available to this lane; it is strictly
   worse than the honest `NotImplemented` standing today.*
3. A lost `gate.sh` lane, or a `SKIP`/`NO-RESULT` where the incumbent had `PASS`.
4. Any existing fixture verdict changing.
5. A K1 round-trip byte moving.
6. **Any file under `crates/c2-il`, `crates/c2-core/src/coff.rs`, or
   `crates/c2-harness` modified by this lane** — the seam clauses from §1, made
   binding.

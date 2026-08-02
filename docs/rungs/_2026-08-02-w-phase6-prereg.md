# w-phase6 — pre-registration

Written and committed **before any measurement of the registered quantities**.

Base **`a091e37`** (`docs: §10.14 — #159 step one attempted; the standalone
reader failed its cross-check`), verified with `git log -1` as the first command
of the session. Worktree `wt-w-phase6` off `master`, created with
`scripts/setup_worktree.sh` (toolchain verified non-`SKIP`: `w5_chain.cpp` →
4/4 in class).

**This lane writes no code.** Nothing under `crates/` is touched — `gap.rs`,
`codec.rs` and `bundle.rs` are owned by concurrent lanes. The deliverable is
`docs/PHASE6_RANKING.md` plus this prereg, scored.

## Lane premise

`ROADMAP.md` §10.2: the **emit-set ceiling is 25 of 871 graded TUs**
(`fn_total == emit["emit-emitted"]`), **6 already matched**, so every widening
rung in Phases 1–6 summed can move TU match by at most **19 TUs, ever**.
§10.4/§9.16.6 then partition those 19:

| what blocks it | TUs | source |
|---|---:|---|
| **control flow** | **17** | §9.16.6 |
| the whole EH record (`Main.cpp`) | 1 | §9.16.6 |
| three refusals incl. a Phase-4 item (`xboxheap.cpp`) | 1 | §9.16.6 |

and §10.5 ranks **Phase 6 second in the whole plan** on that 17.

## The incumbent, registered as the control

**The incumbent is §9.16.6's `17`, and the claim attached to it in §10.4 is
"it is not nineteen TUs each needing a different thing; it is one thing needed
by seventeen of them".** Every estimate below is scored against that, not
against a bare threshold. If Phase 6 converts 17 TUs, §10.5's ordering stands
unchanged and this lane's output is a confirmation.

**The predicate the 17 was built from is a *presence* test**, quoted verbatim
from §9.16.6: *"`cflow-if-*`, `cflow-loop` or an undecoded `cf-expr-*` in at
least one blocked body"*. A presence test over a TU's blocked functions cannot
distinguish *"control flow appears among the blockers"* from *"control flow is
the last blocker"*, and a TU converts only when **every** blocked function
converts. That distinction is the whole subject of this lane.

The discriminator already exists in the instrument and does not need building:
`CfResidue` (`crates/c2-il/src/func/body/shapes/control_flow.rs:126`) renders
`cflow-<shape>+expr-modeled` for a body whose operand stream is **inside the
port's graded vocabulary** — blocked on control flow *alone* — and bare
`cflow-<shape>` for one that also needs expression work. A bare `cf-…` key is an
undecoded body and claims nothing in either direction.

## Declared bias

**Deflationary, and strongly. Named here so a deflationary result is not read as
insight.**

This lane was briefed with four recorded artifact-rankings (the `cmp`-spine
match-bucket claim worth 0 conversions; #150's 22,759 → 6; #127's 8,790 → 472;
§10.13's "the wall does not decompose"), with "declining is a GOOD outcome", and
with the project's own statement that its best single lane output was a
re-ranking from 48,102 to 718. I therefore *expect* to find the 17 deflates, and
that expectation is not evidence. Three concrete guards:

1. **E1 predicts the incumbent reproduces exactly.** If §9.16.6's own predicate
   does not return 17 on the tip scan I must report that the incumbent moved,
   which is a different and less convenient finding than "the predicate is weak".
2. **E4 is registered at a number that can be wrong in the inflationary
   direction.** If fewer than 12 of the 19 need ≥2 items, my bias cost me.
3. **A second bias, opposite in sign, and it is the one I expect to cost me:**
   I have already noticed while reading `gap.rs:685` that `emit_set_reachable_tus`
   compares `fn_total` — an **`LO`-anchored** count (§10.11, §10.12) — against
   `emit-emitted`, while `PortC2::build` consumes `IlBundle::functions()`, which
   is **`4F 1F`-anchored**. That is an attractive "the ceiling itself is wrong"
   story and I want it to be true. E5 registers it as a *bounded* claim about the
   two license TUs only, because that is all §10.11 measured, and anything wider
   would be the same over-read §10.10 made.

## Registered estimates

| # | claim | point | interval | what would refute it |
|---|---|---:|---|---|
| **E1** | **The incumbent reproduces.** Re-running §9.16.6's *presence* predicate — control flow (`cflow-if-*` / `cflow-loop` / `cflow-multi-exit` / `cflow-switch`, or an undecoded `cf-*`) in ≥1 blocked body — over the 19 unmatched emit-set-ceiling TUs on the tip scan | **17** | [15, 19] | anything outside ⇒ the incumbent moved between §9.16.6 and the tip; report that first |
| **E2** | **Control flow is SUFFICIENT** on this many of the 19 — i.e. **every** blocked function in the TU is a decoded non-`straight` `cflow-<shape>+expr-modeled` body, so a Phase-6 rung with no expression work converts the whole TU | **1** | [0, 3] | ≥ 8 ⇒ §10.4's "one thing needed by seventeen" survives the joint measure and the incumbent stands |
| **E3** | **The top-ranked single control-flow construct, by TUs it converts ALONE** (all other blocked functions in the TU already in class) | **0 TUs** | [0, 2] | ≥ 3 ⇒ a single construct is a real rung and should be scheduled |
| **E3b** | *Conditional on E3 > 0*, which construct ranks first | `if-1`/`if-2` (`CfShape::Forward`) | — | `loop` or `switch` first ⇒ the cheap-shape assumption in `control_flow.rs:95` is wrong about this population |
| **E4** | **How many of the 19 need ≥2 distinct items**, counting each `CfShape` as one item and "expression vocabulary" and "an undecoded body" as one item each (§10.13's decomposition test, applied to the near edge) | **17 of 19** | [12, 19] | ≤ 11 ⇒ the near edge decomposes where the wall did not, and my bias cost me |
| **E5** | **The ceiling predicate is splitter-dependent.** `TomCryptLicense.cpp` and `ZlibLicense.cpp` are recorded by §10.10/§10.11 as `fn_total = 0` vs 1 emitted COMDAT — i.e. **outside** the 25 — while `IlBundle::functions()` sees 1 segment. Both are absent from the 25 on the tip scan, and the 25 is therefore an `LO`-anchored count of a `4F 1F`-anchored property | **YES, both absent** | — | either license TU appearing *in* the 25 ⇒ `fn_total` is not what §10.11 says it is |
| **E6** | **The 6 matched TUs are a subset of the 25** (the ceiling's own control, `emit_set_violations() == 0`) | **0 violations** | — | any nonzero ⇒ the ceiling is void and E1–E4 are measuring nothing |

## Method, fixed in advance

1. One warm scan: `c2rs gap --list work/dc3-workload/files.txt --flags-file
   work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 --jsonl <abs>`.
2. The 25 = `{ r : r.class != capture-fail ∧ r.fn_total == r.emit["emit-emitted"] }`,
   read straight off the JSONL. The 19 = those with `class != match`.
3. **The joint, per TU, never a marginal product.** For each of the 19, a
   per-function table from `c2rs census <cpp> --flags-file … --cwd …` — one row
   per `.ex` segment carrying `(verdict key, cflow key, name)`. A TU's blocker
   set is the **set union over its blocked rows**, and the conversion question is
   asked of that set, not of any row.
4. **Every ranked row's control-flow class is checked**, per the brief's third
   trap. A blocked function is credited to a construct only if its own `cflow`
   key names that construct; a `cflow-straight` row is never credited to Phase 6
   no matter what its blocker key says. This is the exact check the `cmp`-spine
   claim skipped.
5. **`cf-*` rows are counted as their own item and never as a control-flow
   construct.** An undecoded body's shape is unknown; crediting it to Phase 6
   would be a product of two ignorances (`gap.rs:143`'s own rule).
6. Before ranking any key, `grep -rn` it across **all** of `docs/`, oldest hit
   read last.
7. Any Python here is analysis over the harness's own JSONL/stdout. It
   re-derives **no** rule the harness owns; the one place it comes close (the
   set-union in step 3) is cross-checked against `c2rs census`'s own printed
   `in class` count on a named TU before any number is believed (§10.14).

## What this lane will NOT claim

* Not that Phase 6 is worth building or not — only what it converts at the near
  edge, in TUs.
* Not a number for anything outside the 25. The ceiling is the premise.
* Not that "in class" implies "byte-exact". Every count below is a **necessary**
  condition; §8.1's precedent (census 4.45 % → 28.69 %, TU match 6 → 6) is the
  standing reason that gap is not rhetorical.

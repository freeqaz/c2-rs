# w-selbind — PREREG

Frozen **before the first `crates/` change**, the first probe cell and the first
fixture line. Board rows **#2820–#2859**. Merge-base **`34b3b4a9`**
(`docs: CEILING §12 …`), branch `worktree-agent-a661339a13ee781bf`.

Base figures re-derived at THIS merge-base and not from a predecessor's rung
(#2793 / w-phase7b §9.2 — that exact transcription split-missed two lanes):
`docs/STATUS.md`'s generated block, collected at tree `9b2cd3c5`, whose
`crates/`, `fixtures/` and `scripts/` are **byte-identical** to `34b3b4a9`
(`git diff --stat` empty), reads **1,486 passed, 0 failed, 41 targets**, match
**23**, mismatch **0**, vocab-gap **848**, capture-fail **7**, `factor-a` 28,
`B∧C` 151, `A∧B∧C` 27, FRONTIER 4, `emit-predicate-worth` **124**,
`frontier-if-a` **126**, `fnbyte-exact` **35,810**.

Workload stamp: dc3 **`104e7df9c10acfe56ee3a87d75f0a9c85740df11`**, tracked tree
CLEAN (one untracked dir, `work/`). **dc3 has now moved five times in six
lanes** — `d7a3c1aa` → … → `a8cb9ca6` (w-phase7b) → `104e7df9`. The committed
workload list and flags are used **as committed and never regenerated** (#2700).
Toolchain `compilers/X360/16.00.11886.00`, wibo `1.2.0-c2rs.1`. Base binary
`work/w-selbind/c2rs-base`, md5 **`cf1bcf937c6ab4f807eb442ac60bde70`**, copied
out of `target/release/` before the first edit and **KEPT** (#2409: a
`git checkout master -- crates/` round trip is not a counterfactual).

---

## 0. FINDINGS ALREADY IN HAND — registered as findings, not predictions

`CEILING.md` §11.4 was worked first, off this lane's own capture, before any
`crates/` change. Three of its answers are recorded here because they are
measurements and must not be scored as forecasts.

**F1 — the commission's antecedent is FALSE under the shipping reader, and this
is board #2784 read one instrument too loosely.** #2784 says *"`vec.cpp`'s emit
set is nameable from its own `.gl` — both bodies, at exact `.ex` split points"*,
and this lane's brief carries it forward as *"both constructors are present in
`.gl` … nothing about its bytes is missing"*. Measured at this tree
(`work/w-selbind/emitnamed.py`, `wideframe.py`):

```text
codec::gl_offset_framed (SHIPPING)  36 framed records over 811 .ex segments
    ??0Vector3@@QAA@MMM@Z    NO RECORD
    ??0Vector4@@QAA@MMMM@Z   NO RECORD
```

Both offsets are *spelled* — `80 6a 82 01 00` sits at `.gl` +27521 — but the
frame test fails on `gl[o-5] == 0x10`: the record's PREV field is **`0x189a`**,
outside `[0x1000, 0x10FF]`. `IlBundle::gl_body_start_coverage` reported
"present 373 of 811" because **its own doc says `present` is deliberately an
over-count** (any `80 <LE32>`, framed or not). *Spelled* is not *named*: a
binding needs the `(offset, name)` pair, and under the shipping frame neither
emitted body has one. **So a selective binding, alone, binds NEITHER of the two
bodies `vec.cpp` emits** — it would bind 32 of the 811 discarded STL bodies and
omit both constructors, which is a wrong obj, not a gap.

**F2 — the nameability #2784 asserts requires #2783's UNSHIPPED frame
relaxation, and the two rows were never joined.** Re-derived here, not quoted:
relaxing `gl[o-5]` to `PREV < 0x10000` takes `vec.cpp` from **36 → 369** framed
records, **0** offsets that are not `.ex` split points at either width, **0**
records sharing an offset — and **both constructors then bind**, `??0Vector3` at
98,922 and `??0Vector4` at 105,430, each to its own segment. So the frame
relaxation w-phase7b measured and declined (#2783, "converts nothing") is a
**precondition** of the mechanism #2784 licenses. It is an eighth mechanism on
`vec.cpp`, in front of D-V1.

**F3 — `gate_cause` reads `gl-stop-26-introduced`, and the walk stops at record
9 of 36.** `gate_causes` = `[gl-stop-26-introduced, body-out-of-class]`,
`gl_body_starts` = `[373, 811]`, `fn_names` 150, `fn_total` 811, class
`vocab-gap`, `fnbyte-exact` **2 of 2** — item 8 answered through the one field
§11.4 says answers it, and through none of the three that have been wrong.
Item 9 (NC-5, the port's own fences) is **still not reached**: no body is looked
at, so `comdat::fenced_inlined_callee`, `elide`'s E and `splice`'s S7 are unasked
on this TU and remain part of its price.

---

## 1. THE CONTRACT THIS LANE SETS OUT TO BUILD

> **THE SELECTIVE BINDING CONTRACT.** A selective binding binds a *subset* of
> the `.ex` segments — those whose split point is the framed body-start offset
> of a `.gl` record. A segment it does not bind yields no `IlFunction`, hence no
> symbol and no `.text` bytes, so the binding is sound **only if c2 emits no
> body for any unbound segment**. The port cannot observe c2's emit set, so it
> discharges the obligation from the input, in the refusing direction:
>
> 1. every record's framed body-start offset must **be** an `.ex` split point;
> 2. no two records may name the same segment;
> 3. **every unclaimed mangled `.gl` run must be accounted for by something that
>    proves the reference obj carries that symbol WITHOUT a body of ours** — and
>    a resolved *call callee* is **not** such an account on this path, because
>    under the 1:1 binding a locally-defined callee is a bound segment the inline
>    fence can see, and under a selective binding it is exactly what a callee
>    account would hide.
>
> Clause 3 is the totality condition. Where the records are 1:1 with the
> segments it reduces to the incumbent gate; where they are not it is **strictly
> stronger** than the quantifier it replaces. That is why this is not the
> widening `w-vec` §8 ruled out (*"binding fewer names than segments is the one
> change that lets a wrong obj out of the gate on 851 TUs"*): it does not bind
> fewer names than segments — it binds fewer **segments** than names, and
> refuses unless the names are exhausted.

**Why clause 3 cannot be "zero unclaimed mangled runs", measured before it was
written.** That clause would refuse TUs the incumbent path already binds:
`mmio.cpp` (11 records / 11 segments) has **1** unclaimed mangled run and
`wordwrap.cpp` (3 / 3) has **2**. The incumbent accounting is therefore kept and
narrowed, not replaced. `EncryptXTEA.cpp`, a match, has **0** unclaimed mangled
runs and **4** unclaimed non-mangled ones — which is why the clause is over the
mangled population and not over every run (#1721).

## 2. WHAT SHIPS

`Bindings::selective` enforcing the above; `IlBundle::functions()` routed
through it with the 1:1 case reaching the incumbent code unchanged; a `diag`
cause name so `gate_cause` reports the new refusal; `IlBundle`'s pure reader for
the join; a `--jsonl` field and a rendered `c2rs gap` block. **No fixture** — no
class is admitted, and a `_neg` cell that grades nothing is worse than none
(#2698/#2699). **No emitter arm, no parser clause.**

---

## 3. REGISTERED PREDICTIONS

| # | registered | p |
|---|---|--:|
| **C1** | `src/system/math/vec.cpp` converts — match **23 → 24** | **0.02** |
| **C1b** | the decline branch: every mechanism named and counted at this tree, with **≥ 1 not among w-phase7b's seven** | **0.90** |
| **C2** | `fnbyte-exact` delta **exactly 0** (35,810 → 35,810). Expected +0 and said so: the two bodies are already exact, so any win here is a BINDING win like `w-fence2`'s, never a byte one | **0.92** |
| **C2b** | delta in `[−2, +2]` | 0.96 |
| **C2c** | `fnbyte-exact` does not FALL (registered unlosable — this lane must not repeat #2622's −1) | 0.94 |
| **C3** | **THE DECIDING ROW.** The selective contract, built total, **REFUSES `vec.cpp`**, and the refusal is clause 3 — the NAME accounting — **not** the 1:1 quantifier the commission names. **Its antecedent, which is the half that must be checked**: under the widest frame with **0** false positives, `vec.cpp` leaves **≥ 100** mangled `.gl` runs unclaimed. **What would VOID it**: the unclaimed set turning out to be fully accountable (all of it extern data, emitted data defs, or generated forms with no `.ex` body), which would make clause 3 pass and put the refusal back on the quantifier | **0.85** |
| **C4** | the selective contract's bindable population over the 871 graded TUs is **equal to** the 1:1 contract's — **+0 TUs**, so selectivity is not the lever either | **0.75** |
| **C5** | the open join of w-phase7b §10 item 3 (*is `emitted ⊆ claimed` on all 871 TUs?*), answered here: the TUs where **every** emitted symbol carries a framed record number **≤ 40** under the shipping frame | **0.70** |
| **C6** | `emit-predicate-worth` stays **124** and `frontier-if-a` stays **126** | 0.92 |
| **C7** | **mismatch 0** at every level — 878 TUs, 369 fixtures × 2 modes × 2 binaries, 18 gate lanes, the sweep, the cross (registered unlosable) | 0.96 |
| **C8** | 878-TU neutrality BY NAME (#2667, full `src` path — a basename compare drops 37 rows): **0** class verdicts changed, 0 toward acceptance and 0 away, and **0** per-TU byte triples changed | 0.85 |
| **C9** | `#[test]` delta **+6**, `± 4` the whole claim; integration-test **targets 41 → 41** (this lane adds no test *file*). Base re-derived from `STATUS.md`'s generated block at THIS merge-base | 0.60 |
| **C10** | ≥ 1 unnamed refusal fires at a pre-armed place. **Pre-armed**: (a) a `diag` cause the new path can reach that `decode_causes`' `causes.is_empty() == decodes` invariant then breaks on; (b) the census's `Bindings::positional` and the gate disagreeing about class once the gate's name list can be shorter than `segs`; (c) `Bindings::names()` being consumed as *"one name per segment, in order"* somewhere the selective list is not; (d) `label_counter` / `plan_labels` charged over a short function list; (e) `#2793`'s shape — a mechanical multi-site edit that `cargo build --release` reports green | **0.50** |
| **C11** | the `gap-metric` key map **GROWS** (this lane adds `selbind-*` keys) and **no pre-existing key changes value** | 0.80 |
| **C12** | `hatch-red` REFUSES, and the refusal REPRODUCES at the merge-base in a tree with none of this lane's `crates/` present (#1406) | 0.85 |
| **C13** | the new bound on THIS acceptance path: **29 → 29** (w-phase7b's 29, unmoved). #2791's caveat carried: 29 bounds one path, not the project | 0.80 |
| **C14** | T1 still fires on `vec.cpp` at the tip (`fnbyte-exact == denominator == 2`, class ≠ match) | 0.90 |

**Downstream rows declared conditional.** C4, C5 and C13 are all conditional on
C3 not being VOID: if clause 3 turns out satisfiable on `vec.cpp` the contract's
denominator is a different measurement and those three are re-derived, not
scored.

**The row that could actually go wrong** is **C8/C2c**, and the mechanism is
named: `functions()` currently consumes `Bindings::names()` as a list that is
1:1 with `segs`, in order — `names.iter().take(n_defined).zip(&segs)`, the
`defined` set the inline fence is built from, and the `accounted` list. A
selective `names` breaks all three at once, in the *licensing* direction for the
fence (a shorter `defined` set is a WEAKER fence, #2623 with its sign flipped).
The mitigation registered in advance: the 1:1 path keeps the incumbent code and
is not routed through the new one, so neutrality on the 23 matches is by
construction rather than by measurement — and the measurement is run anyway.

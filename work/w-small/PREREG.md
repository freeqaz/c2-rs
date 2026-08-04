# w-small — prereg

    Lane:   w-small (`wt-w-small`), branched at master `c303ad0`
    Date:   2026-08-04
    Seam:   crates/c2-core (except coff/reloc.rs), crates/c2-il, docs/rungs/
    Scope:  two leads found and priced by other lanes and deliberately not taken
            by them. Independent; separate commits so the gate can attribute.

Committed **before the first `crates/` edit and before either measurement**.

---

## 0. Baseline claimed by the brief, to be re-measured on this tip

| metric | brief's incumbent |
|---|---|
| `cargo test --workspace --release` | 763 passed / 0 FAILED / 25 targets |
| `scripts/gate.sh --jobs 8` | 12/12 PASS, 2,940 verdicts |
| sweep row | 16,164 reached / 16,068 graded / **96 ungraded** / 0 mismatch |
| cross row | 79,887 / 79,499 graded / 0 mismatch |
| `c2rs gap` (878 TU) | match **8**, mismatch **0**, FRONTIER **19** |

The brief's sweep/cross figures are LARGER than `docs/STATUS.md`'s and than
w-label's rung (14,817 / 63,723). Both are quoted here; whichever this tip
actually produces is the incumbent, and the difference is reported rather than
reconciled away.

---

## 1. Lead 1 — `Bindings::per_record`'s `!=` may relax to `>`

### 1.1 The site, read before predicting

`crates/c2-il/src/func/bind.rs:446`:

```rust
let (bound, unclaimed) = gl_defined_names(gl);
if bound.len() != segs.len()
    || bound.iter().zip(starts).any(|(&(off,_), &s)| off as usize != s)
{ return None; }
```

`segs` are the `.ex` `4F 1F` function segments; `bound` are `.gl`'s framed
defined-name records, in offset order. The `zip` is length-limited by the
shorter side, so it only ever checks the first `min(bound,segs)` entries.

### 1.2 What `>` admits that `!=` refused — stated before running anything

`bound.len() < segs.len()`, with the bound records' offsets a **prefix** match
on the split points. (A missing *leading* record still refuses, because then
`bound[0].off == starts[1] != starts[0]`.)

The downstream consequence is **not** the one w-shapes hypothesised. At
`crates/c2-il/src/func/bundle.rs:1176` the emit loop is

```rust
for (i, (name, seg)) in names.iter().take(n_defined).zip(&segs).enumerate()
```

— bounded by `names.len()`. So a short `names` does not misname a body; it makes
the port **silently drop the trailing `segs.len() - bound.len()` segments** and
emit an obj with that many fewer `.text` COMDATs and symbols. That is right
exactly when c2 also dropped them (the unreferenced-internal-linkage case
w-shapes §3.3 exhibits) and is a wrong emit otherwise.

### 1.3 The gap I expect to find, named ahead of the probe

`gl_defined_names` (`crates/c2-il/src/func/gl.rs:258`) has **five** paths that
return `(Vec::new(), Vec::new())` — i.e. `bound.len() == 0` — for a TU that
plainly defines functions:

1. a framed record whose nearest preceding symbol run is > 32 bytes away, or absent
2. a record name that is not `looks_mangled` — **`extern "C"` lands here**
3. a run that ends at `26` rather than NUL
4. `linkage_needs_a_directive` — `__declspec(dllexport)`
5. a **`26`-introduced** defined name — board #232's COMDAT-linkage case

Under `!=`, all five refuse the TU (`0 != segs.len()`). Under `>`, `0 > n` is
false, so all five **accept with zero names**, and the loop emits **zero
functions**. `PortC2::build` (`crates/c2-core/src/lib.rs:383`) then takes the
`funcs.is_empty()` arm and, if `shell_only_tu()` says yes, emits the bare
four-section shell — for a translation unit that has code. That is board #276's
failure shape, reintroduced.

### 1.4 Prediction — **P1**

> **P1a.** I will construct at least **one** TU that is `Port=NotImplemented`
> on master and `Port=Mismatch` under the one-character relaxation, at the
> sweep's own profile `/Ox /GS- /c`. First candidate:
> `extern "C" int f(int a){return a+1;}` (path 2 above).
>
> **P1b.** The mismatch will be a *section/symbol-count* divergence — an obj
> missing a `.text` COMDAT or missing the whole `.text` — not a name swap. So
> it mismatches early in the file (offset 2 or 8), not in the string table.
>
> **P1c.** w-shapes' `0 mismatches over 16,164` will REPRODUCE on this tip.
> The generated corpus does not write `extern "C"`, `dllexport` or a
> `26`-introduced defined name in a TU that also has a plain function, so the
> sweep is silent about exactly the class that breaks. If P1a lands and P1c
> also lands, the finding is **"the sweep's silence here is trap 5, absence
> read as success"** and that is the deliverable.

**Decline clause D1.** *One* reproduced `Port=Mismatch` declines Lead 1
outright, regardless of the +8. A wrong emit is strictly worse than a refusal
(CLAUDE.md's one correctness rule) and no count of green cases buys it back.

**If P1a fails** — i.e. I cannot construct a mismatching case after exhausting
all five paths — then the brief's instruction applies: the +8 came from
somewhere else and I must find where before shipping. In that event I will
identify which of the 210 `65-linkage-comdat` cases flipped and read its `.gl`,
and I still decline unless the flip is explained by a rule, not by a count.

**Registered TU-match effect of Lead 1: 0.** The +8 was measured in a generated
`/Ox` fragment; the 878-TU workload is `/O1` and 863 of it refuses at decode
before binding is consulted. Interval [8, 8].

---

## 2. Lead 2 — `ho-and` / `ho-or`

### 2.1 The shapes

From `work/w-label/cflabels.py` HELDOUT, at `/O1 /GS- /c`:

```cpp
  ho-and  int P(int a,int b){ if (a && b) return 5; gp(a); return 0; }
  ho-or   int P(int a,int b){ if (a || b) return 5; gp(a); return 0; }
```

w-label measured both at `stride 5 / minted 5 / sur +0` — **the counter charges
nothing** — with two `bc`s naming one two-predecessor forward interior target.

### 2.2 The constraint I am under

w-label's AA-a: every body with a **backward** intra-section branch charges the
label counter ≥ +1 in 11 of 11 cells while `coff::plan_labels` charges 0, so
emitting one is six wrong bytes in the symbol table on top of a wrong block. If
either shape needs a backward reference **I refuse it and say so**. And AA-b/AA-c:
the surcharge is a function of neither the emitted obj nor the `.gl` seed, so a
shape needing a surcharge I cannot read is refused, never modelled.

### 2.3 Prediction — **P2**

> **P2a.** Both shapes are refused at **decode**, in `crates/c2-il`, not at
> emit. The port will report `Port=NotImplemented` and the census will name a
> `c2-il` blocker key rather than a codegen one.
>
> **P2b.** Disassembly will show both are **forward-only** in the emitted `.text`
> (w-label read this off `cftargets.py` already), so AA-a does not fire and the
> refusal is a `codegen`/`c2-il` judgement rather than a `coff/` one.
>
> **P2c.** The number of **independent refusals** between master's accepted class
> and either shape is **≥ 2**, and I predict **≥ 4** for at least one of them —
> in which case the standing decline clause (*a target at ≥ 4 independent
> refusals is not a target*) fires and I decline, exactly as w-conv's clause
> fired on all seventeen FRONTIER TUs.

**When the row's blocker is a class whose emitter already exists, the ceiling IS
the estimate and no discount applies.** So: if the refusals are ≤ 3 and every one
of them is a widening of an emitter that already exists, I ship and the estimate
is the ceiling. Otherwise I decline with the count.

**Registered TU-match estimate: 8**, interval **[8, 8]**, bias declared **high**.
Argument: A∧B∧C is 27 and the whole FRONTIER is priced at ≥ 6 independent
refusals each, so no single control-flow widening converts a workload TU. A
fixture-level `Port=Match` on a new shape is the realistic upside and is not TU
match.

---

## 3. The counterfactual standard I hold myself to

For anything I ship: a **specific** breaker, applied transiently, measured,
reverted in the same script, with `git status --porcelain crates/` proven empty
afterwards. w-shapes set the bar by reproducing w-sect's published 154 to the
case; a breaker that breaks everything proves nothing.

For anything I **decline**, the counterfactual runs the other way: I must show
the relaxation *does* produce bytes and that they are *wrong*, with the mismatch
offset quoted.

## 4. What the sweep cannot see, registered now so silence is not read as coverage

Lead 1's site is in **`crates/c2-il`**. w-label's rung §5 records that the
coverage instrument (`work/w-frame/sweep.py`) **cannot cover `crates/c2-il` at
any `C2RS_SWEEP_KEEP` value — it only reads `c2-core`.** So any coverage figure
I quote is about `c2-core` alone, and I will say so rather than let it read as
whole-lane coverage. (This is about the *coverage* instrument. The *grading*
sweep, `scripts/expr_sweep.sh`, runs the whole port end to end and does grade
`c2-il` behaviour — the two are different instruments with similar names and
conflating them would be a mistake in the safe-looking direction.)

# PHASE6_RANKING — the "17 of 19 block on control flow" is true and converts nothing

**W-PHASE6, 2026-08-02. Measurement only; no code was written.** Scan
`work/dc3-workload/scan-w-phase6.jsonl`, tree `0ff0728`, binary
`d47262d65e58fba5a6c9e9a44ea85f90`, workload `dc3-decomp@173eb73`,
wibo 1.0.1-23. Pre-registration:
[`rungs/_2026-08-02-w-phase6-prereg.md`](rungs/_2026-08-02-w-phase6-prereg.md),
committed before the first measurement.

---

## The one-line result

**§9.16.6's `17` reproduces exactly, on the same two exceptions, and it is a
*presence* count. Measured jointly per TU, Phase 6 — the entire control-flow
phase, every construct, at any width — converts `0` of the 19. So does the
entire expression layer without it (2, and even those two do not survive their
other axes). All 19 need at least two constructs at once. The near edge does not
decompose, exactly as §10.13 found the wall does not.**

This does **not** refute §10.5's ordering. It refutes the sentence §10.4 attaches
to it — *"it is not nineteen TUs each needing a different thing; it is **one
thing** needed by seventeen of them"*. There is no one thing. Phase 6 is a
**necessary** half of a two-half rung for 17 TUs, and is worth zero delivered
alone. Ranking it as the second phase is still defensible; scheduling it as a
phase that pays on completion is not.

| the claim | measured |
|---|---:|
| §9.16.6 / §10.4 — control flow blocks **17** of the 19 | **17** ✓ reproduced |
| …and therefore Phase 6 converts them | **0** ✗ |
| the whole expression + statement layer, no control flow | **2** |
| the two together, minus the 3 TUs with an undecoded body | **16** |
| of the 19, how many need ≥ 2 constructs | **19 of 19** |

---

## 1. The emit-set ceiling, enumerated — all 25, by name

`emit_set_reachable_tus()`, i.e. `fn_total == emit["emit-emitted"]`, over the 871
graded TUs. **`emit_set_violations() = 0`** on this scan (prereg **E6 ✓**) — the
control that makes the ceiling a measurement rather than an argument.

### The 6 already matched

| # | TU | `.ex` segs | in class | emitted COMDATs |
|---:|---|---:|---:|---:|
| 1 | `src/system/synth_xbox/GainEffect.cpp` | 0 | 0 | 0 |
| 2 | `src/system/synth_xbox/HeadsetPlaybackEffect.cpp` | 0 | 0 | 0 |
| 3 | `src/system/synth_xbox/PeakDetector.cpp` | 0 | 0 | 0 |
| 4 | `src/system/synth_xbox/soundtouch/source/SoundTouch/mmx_optimized.cpp` | 0 | 0 | 0 |
| 5 | `src/system/synth_xbox/soundtouch/source/SoundTouch/sse_optimized.cpp` | 0 | 0 | 0 |
| 6 | `src/system/utl/Spew.cpp` | 2 | 2 | 2 |

### The 19 unmatched — the entire remaining budget of the pre-Phase-7 plan

| # | TU | segs | in class | **blocked** |
|---:|---|---:|---:|---:|
| 7 | `src/Main.cpp` | 1 | 0 | 1 |
| 8 | `src/system/math/Primes.cpp` | 1 | 0 | 1 |
| 9 | `src/system/math/Sort.cpp` | 1 | 0 | 1 |
| 10 | `src/xdk/LIBCMT/osfinfo.cpp` | 1 | 0 | 1 |
| 11 | `src/xdk/LIBCMT/undname.cpp` | 1 | 0 | 1 |
| 12 | `src/xdk/LIBCMT/vswprnc.cpp` | 1 | 0 | 1 |
| 13 | `src/xdk/nuispeech/xboxheap.cpp` | 1 | 0 | 1 |
| 14 | `src/xdk/xjson/jsonwriter.cpp` | 1 | 0 | 1 |
| 15 | `src/xdk/xlrc/xlrcimpl.cpp` | 1 | 0 | 1 |
| 16 | `src/system/negate_test.cpp` | 2 | 0 | 2 |
| 17 | `src/system/synth_xbox/Biquad.cpp` | 2 | 0 | 2 |
| 18 | `src/xdk/LIBCMT/vsnprnc.cpp` | 2 | 0 | 2 |
| 19 | `src/system/rndobj/wordwrap.cpp` | 3 | 0 | 3 |
| 20 | `src/system/utl/Pool.cpp` | 3 | 0 | 3 |
| 21 | `src/xdk/nuispeech/mmio.cpp` | 11 | 8 | 3 |
| 22 | `src/system/synth_xbox/IPP_basicmath_xbox.cpp` | 4 | 0 | 4 |
| 23 | `src/system/utl/EncryptXTEA.cpp` | 5 | 1 | 4 |
| 24 | `src/xdk/nuispeech/xboxmem.cpp` | 4 | 0 | 4 |
| 25 | `src/keygen_xbox.cpp` | 20 | 2 | 18 |

**54 blocked bodies in total.** Because these TUs satisfy segments == COMDATs and
the port emits one `.text` COMDAT per segment, **every blocked body here is a
body c2 actually emits** — there is no never-emitted residue to discount, which
is not true of the workload at large (§8.1: c2 emits 7.23 % of IL bodies).

Five of the six matches define zero functions; `Spew.cpp` is the only match with
a body in it. That has not changed.

---

## 2. Method — and why it is a joint, not a product

Per TU, one `c2rs census <cpp> --flags-file work/dc3-workload/flags.txt --cwd
<dc3>` at the workload's own `/O1 /Oi /EHsc`, giving one row per `.ex` segment
carrying `(verdict key, cflow key, EH key, segment length, name)`. A TU's
requirement is the **set union over its blocked rows**; the conversion question
is asked of that set. No marginal is ever multiplied by another.

**Known-answer cross-check first (§10.14's rule).** The parsed per-function
tables were compared against the harness's own JSONL on three fields —
`fn_total`, `fn_in_class` and the full `fn_blockers` map — for all 19 TUs.
**All three agree on all 19.** No rule the harness owns is re-derived here; the
analysis is a set-union over the harness's own output.

### The two axes that decide it, and why the blocker key is not one of them

The brief's third trap: *crossing a row with the control-flow axis is a filter,
never a ranking*, and a first-blocker key cannot see the control-flow axis. So
the requirement per body is read off the two axes the instrument computes
independently of the accepting parser, and the blocker key is used only to
**name** things, never to decide them:

| axis | source | says |
|---|---|---|
| **shape** | `CfShape` (`control_flow.rs:77`) | `straight` ⇒ needs no block IR; anything else ⇒ needs Phase 6 |
| **residue** | `CfResidue` (`control_flow.rs:126`) | `+expr-modeled` ⇒ operand stream is inside the port's graded vocabulary; bare ⇒ needs expression work **as well** |

Third and fourth categories fall out and are kept separate rather than folded in:
a body that is `cflow-straight+expr-modeled` **and still blocked** needs neither
(3 bodies — the statement grammar), and an undecoded `cf-*` body has **unknown**
shape and is credited to nothing (3 bodies; crossing "we cannot read this body's
control flow" with a blocker is a product of two ignorances, `gap.rs:143`).

### The direction of every error here is deflationary-safe

`+expr-modeled` is an **optimistic** upper bound on "blocked on control flow
alone", and the project has the counterexample already: `rungs/2026-07-31-assign-eof.md`
records **438 bodies reading `+expr-modeled` of which not one parsed** when the
gate was lifted — *"a cross-tab against the cflow axis bounds a row from above…
it is not a proxy for the recognizer."* Since the count of CF-only-blocked
bodies below is **1**, an over-claiming instrument can only push it toward 0.
The result is therefore sound in the direction it is stated, which the
`Phase 6 + X = N` bundles in §5 are **not** — those are ceilings and are labelled
as such.

---

## 3. The joint, per TU — all 19

`CF:<shape>` = needs the block IR for that shape. `EXPR` = needs operand
vocabulary the port does not have. `STMT` = blocked with neither axis firing.
`UNDEC` = undecoded body, shape unknown. `EH` = `maxState >= 1`, needs the whole
EH record.

| TU | blocked | what each blocked body needs | distinct categories |
|---|---:|---|---:|
| `Main.cpp` | 1 | `EXPR` **+ `EH`** (`eh-state1`) | **2** |
| `Primes.cpp` | 1 | `CF:loop`+`EXPR` | 2 |
| `Sort.cpp` | 1 | `CF:loop`+`EXPR` | 2 |
| `osfinfo.cpp` | 1 | `CF:if-n`+`EXPR` | 2 |
| `undname.cpp` | 1 | `CF:if-n`+`EXPR` | 2 |
| `vswprnc.cpp` | 1 | `CF:if-n`+`EXPR` | 2 |
| `xboxheap.cpp` | 1 | `EXPR` — but **three independent refusals**, counted by construction (GAPS §9.4) | **3** |
| `jsonwriter.cpp` | 1 | `CF:loop`+`EXPR` | 2 |
| `xlrcimpl.cpp` | 1 | `CF:if-n`+`EXPR` | 2 |
| `negate_test.cpp` | 2 | `CF:if-n`+`EXPR` ×2 | 2 |
| `Biquad.cpp` | 2 | `UNDEC`; `EXPR` | 2 |
| `vsnprnc.cpp` | 2 | `CF:if-n`+`EXPR`; `STMT` | 3 |
| `wordwrap.cpp` | 3 | `STMT`; `CF:if-n`+`EXPR`; `UNDEC` | 4 |
| `Pool.cpp` | 3 | `UNDEC`; `CF:if-1`+`EXPR` ×2 | 3 |
| `mmio.cpp` | 3 | `CF:if-2`+`EXPR`; `CF:if-n`+`EXPR` ×2 | 3 |
| `IPP_basicmath_xbox.cpp` | 4 | `CF:loop`+`EXPR` ×4 | 2 |
| `EncryptXTEA.cpp` | 4 | `EXPR` ×2; `CF:loop`+`EXPR` ×2 | 2 |
| `xboxmem.cpp` | 4 | `EXPR`; `CF:if-1`+`EXPR` ×3 | 2 |
| `keygen_xbox.cpp` | 18 | `CF:loop`+`EXPR` ×11, `CF:if-n`+`EXPR`, `CF:if-1` (alone), `EXPR` ×4, `STMT` | **5** |

### The 54 blocked bodies, by what they need

| | bodies | share |
|---|---:|---:|
| **control flow ALONE** (non-`straight`, `+expr-modeled`) | **1** | 1.9 % |
| control flow **and** expression | **37** | 68.5 % |
| expression only (`cflow-straight`, residue Expression) | 10 | 18.5 % |
| neither axis, still blocked (the statement grammar) | 3 | 5.6 % |
| undecoded — shape unknown, credited to nothing | 3 | 5.6 % |

**One body in 54 at the near edge is blocked on control flow alone**, and it is
`keygen_xbox.cpp [2]`, inside the TU with the largest requirement set on the
board. Building the block IR and stopping converts that one body and no TU.

---

## 4. The ranking asked for — control-flow constructs by TUs converted

### 4a. Delivered alone, today

| construct | blocked bodies in the 19 | TUs it appears in | **TUs it converts** |
|---|---:|---:|---:|
| `CF:loop` | 20 | 6 | **0** |
| `CF:if-n` | 11 | 9 | **0** |
| `CF:if-1` | 6 | 3 | **0** |
| `CF:if-2` | 1 | 1 | **0** |
| `CF:switch` | 0 | 0 | **0** |
| `CF:multi-exit` | 0 | 0 | **0** |
| **all of Phase 6 together** | 38 | 17 | **0** |

Prereg **E3 ✓** (registered 0, interval [0, 2]).

### 4b. The counterfactual ranking — Phase 6's marginal, given the entire expression and statement layer already delivered

This is the only setting in which the constructs separate at all. It is an
**optimistic ceiling**: it grants every expression construct in the workload for
free, which is Phases 1–4 complete.

| construct | **+TUs** | which |
|---|---:|---|
| **`CF:if-n`** | **+6** | `negate_test`, `osfinfo`, `undname`, `vsnprnc`, `vswprnc`, `xlrcimpl` |
| **`CF:loop`** | **+5** | `Primes`, `Sort`, `IPP_basicmath_xbox`, `EncryptXTEA`, `jsonwriter` |
| `CF:if-1` | +1 | `xboxmem` |
| `CF:if-2` | **+0** | — (`mmio` needs `if-2` **and** `if-n`) |
| `CF:switch` / `CF:multi-exit` | +0 | not present at the near edge at all |
| **all four together** | **+14** | the individual marginals sum to **12**; `mmio` and `keygen_xbox` need several CF constructs at once |

### 4c. The inversion — this is why the unit matters

Ranking Phase 6's first rung by body mass and ranking it by TUs give **opposite
answers**, and the gap is 6×:

| construct | blocked bodies, **whole workload** | rank | TUs converted at the near edge | rank |
|---|---:|:--:|---:|:--:|
| `if-1` | **238,766** | **1** | **+1** | 3 |
| `loop` | 91,344 | 2 | +5 | 2 |
| `if-n` | 43,658 | 3 | **+6** | **1** |
| `if-2` | 29,187 | 4 | +0 | 4 |
| `switch` | 304 | 5 | +0 | 5 |

`if-1` — the diamond, 238,766 blocked bodies, the largest control-flow row in the
project — converts **one** TU. `if-n` is 5.5× smaller and converts **six**. A
Phase-6 plan sized off the census histogram would build the diamond first and buy
1 TU. This is the brief's *"a construct that appears in 14,988 blocked functions
but is the last blocker on 1 TU is worth 1 TU here"*, instantiated.

---

## 5. Why it does not decompose — the cmp row and the branch are one rung

The single largest expression family at the near edge is the comparison spine:
**17 of the 54 blocked bodies carry an `expr-cmp-*` first blocker**, spread over 8
TUs. GAPS §9.3 measured that row and closed it at **0 conversions at any width**,
on the ground that all nine sites it found were inside a branch. That reproduces
here and extends:

| `expr-cmp-*` site's control-flow class | bodies |
|---|---:|
| `cflow-if-n` | 6 |
| `cflow-if-1` | 3 |
| `cflow-loop` | 4 |
| `cflow-if-2` | 1 |
| `cf-expr-0x05` (undecoded) | 2 |
| **`cflow-straight`** | **1** |

**16 of 17 are inside a branch, a loop or an undecoded body.** The one
`cflow-straight` site is new since §9.3 (`xboxmem.cpp [0]`,
`?GetXAllocAttributes@NUISPEECH@@YAKH@Z`, 131 B) and is plausibly the boolean
materialization W6 lowers — recorded so §9.3's *"zero are `cflow-straight`"* is
not quoted as still exact. It converts nothing: `xboxmem`'s other three bodies
are `cflow-if-1`.

Run the pairing the other way and the same 8 TUs appear:

| bundle | alone | **with all of Phase 6** | Phase 6's marginal |
|---|---:|---:|---:|
| comparison spine (`expr-cmp-*`) | 0 | **6** | **+6** |
| `assign-store-type-0x86` | 0 | 2 | +2 |
| branch tests (`expr-brtrue`/`brfalse`) | 0 | 1 | +1 |
| `expr-op-0x27` (#150) | 1 | 1 | +0 |

**Each of the top two is worth 0 alone and 6 together.** §10.6 struck Phase 1's
comparison-spine rationale on the first half of that sentence; §10.4 promoted
Phase 6 on the second. Both readings are of the same joint, and neither half pays
on its own.

**And the entanglement is structural, not incidental.** `OPERATOR_GRANTS.md`
already granted the relational tokens and rescanned: the census numerator moved
**+0**, and the population landed on `expr-brfalse` **+19,409** and `expr-brtrue`
+2,955 — *"84.0 % of the free-standing relational population is one token from a
conditional branch."* The successor of the cmp row **is** a Phase-6 construct. So
the `6` in the table above is a ceiling that is *known* to be optimistic, and the
honest statement is stronger than the table: the comparison spine and the block
IR are not two rungs that happen to co-occur, they are one rung reported under
two keys.

---

## 6. How many of the 19 need ≥ 2 constructs

**All 19.** Registered at 17 (interval [12, 19]) — prereg **E4 ✓** at the top of
the interval; my own two axes give 17, and the last two come from axes those two
cannot see:

| | TUs | evidence |
|---|---:|---|
| ≥ 2 categories on the shape × residue axes alone | **17** | this scan |
| `Main.cpp` — `EXPR` **and** `eh-state1` (the whole EH record) | +1 | the `fn_eh` axis, this scan; GAPS §9.4 |
| `xboxheap.cpp` — three **independent** refusals in one 404 B body | +1 | GAPS §9.4's probe ladder, `work/oneaway/p*.cpp` — each refuses with the other two absent |
| **total** | **19 of 19** | |

The shape of that is worth more than the count. **The two TUs §9.16.6 singled out
as *not* control-flow — `Main.cpp` and `xboxheap.cpp` — are the only two the tip
scan's own axes report as needing exactly one thing, and both turn out on a third
axis to need two or three.** The 17 that "block on control flow" are, without
exception, the ones that need control flow **and** something else. The
concentration §10.4 read into the partition is real; what is concentrated is the
*non-decomposition*, not a lever.

This is §10.13's result at the other end of the board: 305 of 451 wall TUs need
≥2 items there; 19 of 19 near-edge TUs need ≥2 constructs here. Two independent
populations, same shape. **The board's model — a list of separable items with
individual prices — does not hold at either end.**

---

## 7. The ceiling's own predicate is splitter-dependent (prereg E5 ✓)

`emit_set_reachable_tus()` compares `fn_total` against `emit-emitted`.
**`fn_total` is `LO`-anchored** (`split_function_bodies_at`, `bundle.rs:39`,
keyed on `4C 4F 11`), while `PortC2::build` consumes `IlBundle::functions()`,
which is **`4F 1F`-anchored** (`split_functions_at`). §10.11 and §10.12 proved
those two disagree, and named the population: `??__E` / `??__F` dynamic-init
thunks carry a bare `4C`.

Confirmed on this scan, as registered:

| TU | `fn_total` (census, `LO`) | segments (gate, `4F 1F`) | emitted COMDATs | in the 25? |
|---|---:|---:|---:|:--:|
| `src/system/synth/tomcrypt/TomCryptLicense.cpp` | **0** | 1 (§10.11) | 1 | **no** |
| `src/system/zlib/ZlibLicense.cpp` | **0** | 1 (§10.11) | 1 | **no** |

Both are filed in §10.2's `segments < COMDATs` bucket by a count that is not the
one the port uses. On the port's own splitter they satisfy the ceiling predicate.

**What this licenses, and what it does not.** It does *not* license "the ceiling
is 27". The disagreement is not signed — a TU inside the 25 today whose gate
splitter finds *more* segments than the census would leave it — and the gate-side
count has never been computed across the workload. The bounded, supported
statement is: **the "25 of 871", and therefore §10.2's "19 TUs, ever", are an
`LO`-anchored count of a `4F 1F`-anchored property, and at least two TUs are
known to be on the wrong side of it.** Recomputing `emit_set_reachable_tus()` on
`IlBundle::functions().len()` is a one-line change in `gap.rs` (owned by another
lane) and is the cheapest open correction to the plan's headline bound.

This is §10.11's own lesson applied to the instrument that states the bound: *a
count is only evidence about the predicate that produced it.*

---

## 8. Pre-registration, scored

| # | claim | registered | interval | measured | |
|---|---|---:|---|---:|:--:|
| **E1** | §9.16.6's presence predicate reproduces | 17 | [15, 19] | **17**, on the same two exceptions (`Main.cpp`, `xboxheap.cpp`) | **HIT** |
| **E2** | TUs where control flow is **sufficient** | 1 | [0, 3] | **0** | **HIT** |
| **E3** | top single CF construct, TUs converted alone | 0 | [0, 2] | **0** for every construct | **HIT** |
| **E3b** | *(conditional on E3 > 0)* which ranks first | `if-1`/`if-2` | — | not triggered; on the counterfactual ranking of §4b it would have been **`if-n`**, so this would have MISSED | **n/a (would MISS)** |
| **E4** | of the 19, how many need ≥ 2 constructs | 17 of 19 | [12, 19] | **19 of 19** | **HIT**, at the interval's edge |
| **E5** | both license TUs absent from the 25; the 25 is splitter-dependent | YES | — | **YES**, `fn_total = 0` vs 1 emitted COMDAT on both | **HIT** |
| **E6** | `emit_set_violations() == 0` | 0 | — | **0**; ceiling 25 of 871, match 6 | **HIT** |

**5 clean hits, 1 hit at the edge, 1 conditional that would have missed.**

The declared bias was deflationary and it is confirmed in one direction and
refuted in the other. Confirmed: E2 and E3 came in at the bottom of their
intervals. **Refuted where it counted:** E1 registered that the incumbent would
reproduce and it reproduced *exactly*, on the same two named exceptions — the
`17` is not a soft number and §9.16.6's predicate is not sloppy; it is a correct
answer to a question whose unit is not TUs. And E4 came in **above** my point
estimate, which is the direction a deflationary bias does not produce: I
under-counted the requirement because I did not credit the EH axis on `Main.cpp`
or GAPS §9.4's constructed ladder on `xboxheap.cpp`.

E3b is recorded as a miss-that-did-not-fire rather than dropped. I registered
`if-1`/`if-2` because §8.6 and `control_flow.rs:95` frame the forward diamond as
the cheap first shape and it is by far the largest row. On TUs it is third. Had
E3 come out nonzero I would have ranked the wrong construct, for exactly the
reason this document is about.

---

## 9. What this does not claim

* **Not that Phase 6 should be de-ranked.** It is a necessary half of 17 of the
  19, and §10.5's ordering (Phase 7, then Phase 6) is untouched by anything here.
  What is refuted is the *sentence* — "one thing needed by seventeen of them" —
  and the scheduling model it implies, in which a phase pays on completion.
* **Not a TU-match forecast.** Every number above is a **necessary** condition.
  "In class" is not "byte-exact"; §8.1's precedent — per-function census 4.45 % →
  28.69 %, TU match 6 → 6 — is the standing reason that gap is not rhetorical.
* **Not sound past 19 TUs.** The population is the emit-set ceiling and nothing
  outside it, and the ceiling itself carries §7's caveat above.
* **The item sets in §3 are lower bounds.** A first-blocker key names one
  construct and cannot see what is behind it; `xboxheap.cpp` is the measured
  instance (one key, three independent refusals). Every "needs ≥ N" here is a
  floor and every "`Phase 6 + X` converts N" is a ceiling.
* **Names are labels, not identities.** Per GAPS §9.6 the census names the
  *callee* for any call-bearing body. Unaffected: the counts, the blocker keys,
  and both class axes — so the rankings stand.

## 10. Reproducing it

```sh
c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
         --cwd <dc3-decomp> --jobs 16 --jsonl <out>.jsonl
# the 25:  { r : r.class != "capture-fail" and r.fn_total == r.emit["emit-emitted"] }
# the 19:  those with r.class != "match"
c2rs census <tu> --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp>
# per blocked row: column 3 is the shape+residue key that decides §3.
```

The per-TU tables in §3 are the union over the `GAP` rows of that census output.
Cross-check before believing any re-derivation: the parsed rows must reproduce
the harness's own `fn_total`, `fn_in_class` and `fn_blockers` for the TU.

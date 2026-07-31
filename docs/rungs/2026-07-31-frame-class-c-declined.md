# W33 — Class C (`__savegprlr_N`) declined on measurement, and the frame ladder's ceiling

    Tag:       W33
    Slug:      frame-class-c-declined
    Date:      2026-07-31
    Fixtures:  mvp_call_seq_b_neg.cpp
    Census:    549,148 unchanged — the rung is DECLINED; its measured worth is 0
    Record:    scripts/gt_frame_class.py, docs/CODEGEN_FRAMED_CALLS.md §7

`docs/CODEGEN_FRAMED_CALLS.md` §7 ranks **Class C — a framed function with ≥3
saved GPRs, using the `__savegprlr_N`/`__restgprlr_N` pair** as rung 5, the next
one on the frame ladder. Everything it needs had just been measured: the helper
prologue and the tail-branch epilogue (§2.3, §2.3a), the 5 relocations, the
reverse-first-reference symbol position (§4.3, one LIFO), and the label stride
`+2` per **distinct helper width first introduced** (`LABEL_COUNTER.md` §1.1).

It was sized before it was built, and it converts **0 functions**. This document
is the measurement, the two instruments, and the reason the ladder's *order* is
not a schedule.

## What it admits, and what it refuses

Nothing changed. `plan_saved_gprs` still refuses at `MAX_INLINE_SAVED_GPRS = 2`
with `callseq-three-plus-saved`, `FrameLayout::out_of_class_ctx` still refuses
`frame-savegprlr-helper`, and `IlFunction::label_slots` still returns the flat
framed `5` — which is correct **only** while no helper-using function is
admitted, and which is why the three had to land together or not at all. They
did not land, so the §4.4 stride refutation stays latent exactly as
`CODEGEN_FRAMED_CALLS.md` §6b records it.

The refusal fixture already existed: `mvp_call_seq_b_neg.cpp`'s `three_live`
(`void f(int a,int b,int c,int d){ v1(a); v2(b); v3(c); v1(d); }`), **0/7 in
class**, blocking under `callseq-three-plus-saved:eof`.

## Estimate vs outcome

**Estimate, written down before either counterfactual was run** (verbatim in
`work/scan/ESTIMATE.txt`, which is scratch and untracked — it is reproduced in
full here): **1 function, range 0–2, biased HIGH.** Grounds, in the order
they were formed:

1. `callseq-three-plus-saved` is **0** in the first-blocker histogram over all
   2,462,571 functions. It is the *first* test inside `plan_saved_gprs`, so any
   body that reached `plan_saved_gprs` with 3+ live formals files under it.
   Zero arrivals, not "few".
2. **What that bucket has already been filtered by** — today's W31 rung came in
   1.71× low by estimating off a bucket without asking that question. Upstream
   of the gate sit the routing into the call-sequence shape, `eat_call_head` /
   `eat_call_args` per call and `tail_call_shape` per call; their refusals file
   under `callseq-*` / `call-*` keys, and the **entire `callseq-*` family on the
   878-TU workload is one function** (`callseq-postop-op-0x27`). The upstream
   filter is not hiding a Class C population — it is starving the whole lane.
3. Class B, the immediately preceding rung on the same production with a
   strictly *weaker* requirement, measured **2** by the same counterfactual,
   twice.

**Outcome: 0.** Bias direction correct; magnitude 1 function, which is the
stated noise floor.

### Counterfactual A — lift the gate

`MAX_INLINE_SAVED_GPRS = 2 → 8`, rebuild, rescan the 878 TUs:

```
  FUNCTION CENSUS (P2b): 549148/2462571 functions in class (22.30%)   <- identical
```

**+0.** This is also the **under-claiming** check the standing brief asks for and
that nothing else tests: the gate's over-refusal is not "small", it is exactly
zero functions.

### Counterfactual B — sink the whole production, bucketed by saved count

A is only sound if `callseq-three-plus-saved` is really where such bodies would
land. B removes that assumption: make `parse_call_seq` refuse at its `Ok(...)`
with a key naming `saved.len()`, so every body that reaches the end of the
call-sequence grammar is counted by its Class.

```
  FUNCTION CENSUS: 541366/2462571 (21.98%)      -7,782 against baseline
      7780  SCRATCH-callseq-sink-saved0:eof     Class A  (nothing saved)
         2  SCRATCH-callseq-sink-saved1:eof     Class B  (1 saved GPR)
         0  saved2 / saved3plus                 no bucket emitted at all
```

The whole framed multi-call lane is **7,782 functions**, and its saved-GPR
distribution is 7,780 / 2 / 0 / 0. Not one workload function reaches this
production needing even **two** saved GPRs, let alone three. `seq.saved` is the
only path to a nonzero `FrameLayout::saved_gprs` anywhere in the port (grep:
`crates/c2-core/src/lib.rs:312,425`), so B bounds every route into Class C, not
just the one A tested.

## The ceiling — a second instrument, because the counterfactual cannot see it

A counterfactual measures **what the surrounding grammar can already finish**
(`GAPS.md` §6). It is therefore silent on the question that decides whether a
declined rung is *dead* or merely *early*: a class whose bodies all block three
tokens earlier in the expression layer measures 0 whether the corpus contains
none of it or 25,000 of it.

`scripts/gt_frame_class.py` is the other instrument. It classifies every
**emitted** `/Gy` function in the reference objs the gap scan already caches, by
its prologue, exactly as §2.1–§2.5 defines the classes — no IL parser involved,
so nothing it reports is bounded by what the port can parse.

```
  corpus: 878 sources     objs 871     emitted /Gy functions 178,968

    103,246  57.69%   leaf / tail (no frame)
     30,497  17.04%   B  1-2 saved GPR, inline std
     24,836  13.88%   C  >=3 saved GPR, __savegprlr_N
      7,211   4.03%   A  nothing saved
      6,723   3.76%   D  1-3 saved FPR, inline stfd
      5,835   3.26%   E  >=4 saved FPR, __savefpr_M
        396   0.22%   D+ inline FPRs beside inline GPRs
        224   0.13%   F  both helper pairs
```

**Class C is the third-largest class in dc3 — 25,060 emitted functions (14.0 %)
counting Class F, and 3.4× the size of the Class A framed bodies the port already
emits.** It is not small; it is unreachable. And Class B, which the port *has*
built, has a ceiling of 30,497 and converts **2**.

That pair of numbers is the finding. **The frame is not the binding constraint on
the frame ladder.** §7's rung order is a correct ordering of difficulty and a
wrong schedule: rungs 5 and 6 both convert 0, and rung 4 converted 2 out of a
ceiling of 30,497, because ~99 % of multi-call bodies stop in the expression
layer long before any question about saved registers arises (7,782 of 802,655
`calls-2plus` functions reach the call-sequence production at all).

Two things fall out for free, both stated because they are re-run rather than
remembered — the script prints a `<== REFUTES` tag if either fails:

* **The measured thresholds hold over 75,722 real framed functions.** No inline
  save run reaches 3 GPRs or 4 FPRs. §2.3/§2.4 pinned each threshold with a pair
  of designed probes; this is the same two constants against the corpus.
* **`FRAME_MAX_SAVED_NO_SPILL = 17` is a real boundary that real code hits.**
  The saved-GPR histogram runs 3…18 and stops dead at 18 — `|r14..r31|`, the
  whole callee-saved file, exactly §1.3's spill point — with **111 functions**
  there (0.15 % of framed). The refusal is correctly placed and cheap.

### The trap this instrument has, named because it fired

`work/capture-cache` is shared by every tool in the repo. Running
`scripts/expr_sweep.sh` and `scripts/cross_sweep.sh` between two censuses took
the cache from 871 entries to **39,364**, and the unfiltered class shares moved
by five points (Class C 13.88 % → 9.41 %) purely from synthetic single-function
objs. `gt_frame_class.py --sources work/dc3-workload/files.txt` filters each
entry by the source its `meta.txt` names. **A census over a shared cache must
name its corpus or the number is not a number.**

The other denominator hazard, stated so the two figures are never divided: the
census denominator is **IL** functions across 878 TUs (2,462,571 — a header body
counted once per TU that includes it), this one is **emitted** `.text` COMDATs
(178,968). They differ by ~13.8× and are not a ratio.

## Gate evidence

Nothing functional changed, so every lane is the baseline re-proven.

| lane | result |
|---|---|
| `cargo test --workspace --release` | **438 pass, 0 fail** |
| `c2rs bench` | **163 pass, 0 fail, 0 error** |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **76 / 74 / 74 / 74 match, mismatch 0** |
| `scripts/expr_sweep.sh` | **7,673 checked, 0 mismatches** |
| `scripts/cross_sweep.sh` | **7,545 configurations × 4 lanes, 0 mismatches** |
| 878-TU workload scan | match 6, **mismatch 0**, census **549,148 / 2,462,571 = 22.30 %**, **disagreement 0** |
| fixtures, `c2rs census` | `mvp_call_seq_b.cpp` **18/18**, `mvp_call_seq_b_neg.cpp` **0/7** |

## Found and not taken

| item | size | what stops it |
|---|---:|---|
| **Class C itself** | 0 today / 25,060 emitted at the ceiling | the expression layer, not the frame. Build it when the counterfactual moves off 0 — the measurement to re-run is counterfactual B, one edit and a 28 s scan |
| Classes D/E/F (rung 6) | 0 today / 13,178 at the ceiling (6,723 + 396 D, 5,835 E, 224 F) | same production, same starvation. B's bucket table would show them as `saved*` rows and shows none |
| the `label_slots` stride correction, landed alone | 0 bytes today | it is the one of the three that is *safe* alone (inert until a helper-using function is admitted) and the one that is *unverifiable* alone — no fixture would exercise it, and `LABEL_COUNTER.md` §3 is a whole section on what happens to a model fitted to the classes that are in the capture set. Left latent, as §6b records it |
| the 18-saved-GPR spill regime | 111 emitted functions | `FRAME_MAX_SAVED_NO_SPILL`; the frame-size rule under-predicts there by an unmeasured amount (§1.3). Correctly refused, and now known to be 0.15 % of framed rather than "rare" |
| the 6 GPR / 1 FPR "inline runs past the threshold" | 0 — **a false positive of the first instrument** | varargs argument homing: `??$sprintf_s@…` opens `mflr ; stw r12,-8(r1) ; std r5,32(r1) … std r10,72(r1) ; stwu`, six `std`s off r1 at **positive** displacements. A displacement-blind filter publishes that as a refutation of the threshold of 3. The final script requires a negative displacement; the shipped version of a measurement is the one that survived its own false positives |

### The riskiest thing left unmeasured

**Whether the 25,060 Class C functions stay Class C once the expression layer
admits them.** Every number above about the ceiling is read off c2's *own*
output for the *whole* body; the port will reach those bodies through a grammar
that today accepts a strict subset of what they do. The counterfactual can only
be re-run when they arrive, and until then the ceiling is an upper bound on a
population whose *shape* at admission time is unknown — in particular
`LABEL_COUNTER.md` §4's inlining row (+5 slots per inlined call site, three data
points, one callee class, the first site's +3 unexplained) applies to precisely
these bodies, and real workload TUs are inlined into constantly. A framed
function downstream of an inlined body gets the wrong `$M` the moment the class
gate admits one, and that is a *second* stride fact Class C will have to land
together with, on top of the three this rung was scoped around.

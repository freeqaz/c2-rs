// W-UNW-1 (**negative**): a floating-point leaf that POOLS A CONSTANT, sharing a
// translation unit with a framed function. The constant-free half of this pair is
// byte-exact and lives in `wunw_float_beside_framed.cpp`; this is the part still
// refused, and why.
//
// This file used to hold the constant-free case, on the reading "an FP leaf
// consumes 2 counter slots against the 1 every emitted class consumes". **That
// reading is retracted.** The 2 is `1 + the TU's `_fltused` slot`;
// `coff::plan_labels` charges that slot once per TU, not once per FP function;
// and `leaf-float-led` — the same leaf with `_fltused` already charged to a
// function ahead of it — measures **1**. The counter was already right and the
// gate was refusing on a number the emitter no longer used.
//
// What a pooled constant adds is **+2 per newly pooled `(bits,width)`**
// (`docs/LABEL_COUNTER.md` §1.1). Two facts make that a refusal rather than an
// arithmetic exercise, and **either one alone is sufficient**:
//
//  1. *Newly* is a per-TU question. `const1-dup-led` measures the surcharge at
//     **0** for a constant an earlier function in the TU already pooled, so the
//     leaf below is stride 3 in this file and would be 1 if some function above
//     it had already pooled `2.5f`. `IlFunction::label_slots` is a per-function
//     method and cannot answer it — exactly the structural reason the `_fltused`
//     `+1` had to move to `plan_labels`. This is that same shape one remove
//     further out, and it is a real missing rule, not a private limit.
//
//  2. The obj layout is wrong, and this is MEASURED rather than unknown.
//     `c2_core::coff::emit_obj` places the pooled constants' `.rdata` COMDATs and
//     then `.pdata` last. The reference does not: it lists `.rdata` and `.pdata`
//     **interleaved, in `.text` order**, each at the position of the first
//     function that needs it. `float L1(float a){return a*2.5f;} void S1(){q1();
//     q2();} float L2(float a){return a*3.5f;}` is `.rdata .pdata .rdata`, a
//     shape `emit_obj` cannot produce at all. Across 240 captured TUs at
//     `/Ox /GS- /c` — every order of one or two constant-pooling FP leaves against
//     one or two framed functions — **six** distinct orders occur and this
//     emitter can express one.
//
//     Rule (1) was implemented as a probe and graded against that grid: 234 of the
//     240 graded (6 capture failures), **106 Match and 128 Mismatch, and every one
//     of the 234 is accounted for by the section order alone** — Match exactly
//     where the emitter's fixed order coincides with the reference's, Mismatch
//     exactly where it does not, 0 cases contradicting. So the label rule in (1)
//     is right and is not what blocks this; the section table is.
//
// Both functions census **in class** — that is the per-function verdict and it is
// correct — while the TU as a whole is `Port=NotImplemented`. The refusal lives at
// the translation unit, which is where the label counter lives.
int g(int);

// The FP half: a W13b leaf with ONE newly pooled constant. `leaf-float-c1-led`
// measures stride 3; `label_slots` answers `None` rather than 3, because 3 is
// right only while nothing above it has already pooled `2.5f`.
float scale(float a) { return a * 2.5f; }

// The framed half, which is what makes the counter observable.
int f(int a) { return g(a) + 1; }

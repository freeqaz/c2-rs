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
//  2. The obj layout is uncaptured. `c2_core::coff::emit_obj` places the pooled
//     constants' `.rdata` COMDATs and then `.pdata` last, with a
//     `debug_assert!(pool.is_empty(), …)` guarding the combination, because **no
//     captured TU has both** a constant pool and a framed function. Admitting (1)
//     alone would replace an honest refusal with a guessed section order.
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

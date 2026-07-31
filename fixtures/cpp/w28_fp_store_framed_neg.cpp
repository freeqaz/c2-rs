// **Negative** — the row that separates "+2 per pooled FP constant" from "+2 per
// *newly* pooled FP constant": TWO float leaves pooling the SAME constant, beside
// a framed function.
//
// This file used to hold an FP *arithmetic* leaf beside a framed function, and
// its own comment predicted the promotion — "the leaf below is 2 and could be
// admitted; what admitting it needs is for `IlFunction` to carry whether a float
// leaf pooled a constant". It now does, and the constant-free half is byte-exact
// in `wunw_float_beside_framed.cpp` and in `w28_fp_store_framed.cpp`.
//
// ## Why THIS shape, and not simply "a leaf with a constant"
//
// `wunw_float_neg.cpp` holds one leaf with one newly pooled constant, whose
// stride `leaf-float-c1-led` measures at **3**. Read alone, that row licenses the
// rule "a float leaf costs 1, plus 2 if it pools a constant" — a per-function
// rule, statable in `IlFunction::label_slots`, and **wrong**.
//
// This file is the row that refutes it. `const1-dup-led` measures the surcharge
// at **0** for a constant an earlier function in the TU already pooled
// (`docs/LABEL_COUNTER.md` §1.1), so `s1` below costs 3 and `s2` — pooling the
// identical `(bits, width)` — costs **1**. Two functions, textually identical
// bodies but for their names, different strides. No per-function method can
// return both numbers, which is the same structural fact that moved `_fltused`'s
// `+1` out of `label_slots` and into `coff::plan_labels` one rung ago.
//
// So the missing rule is a TU-level dedup of `(bits, width)` in `plan_labels`,
// and this pair is the smallest case that can grade it — at n = 1 the per-function
// rule and the per-TU rule are indistinguishable, which is exactly how the
// `_fltused` repair came out wrong the first time.
//
// A **second** and independent reason this stays refused: `c2_core::coff::emit_obj`
// does not know the `.rdata`/`.pdata` section order, because no captured TU has
// both a constant pool and a framed function. Its `debug_assert!(pool.is_empty(), …)`
// says so. Landing the counter rule alone would trade an honest refusal for a
// guessed section order.
//
// Every function here censuses **in class** — that is the per-function verdict and
// it is correct — while the TU as a whole is `Port=NotImplemented`. The refusal
// lives at the translation unit, which is where the label counter lives, so it
// adds nothing to `census_gate.rs`'s recorded residual.

void g1();
void g2();

// The first leaf pools `2.5f`: a NEW `(bits, width)`, surcharge +2, stride 3.
float s1(float a) { return a * 2.5f; }

// The second pools the SAME `2.5f`: already introduced, surcharge 0, stride 1.
// The two bodies differ only in name.
float s2(float a) { return a * 2.5f; }

// A stride-1 integer leaf between the constants and the frame, so an error of
// two in either stride above cannot be absorbed by an adjacent function.
int inc(int a) { return a + 1; }

// The framed half, which is the only thing that makes the counter observable.
void seq2() { g1(); g2(); }

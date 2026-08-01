// **A floating-point ARITHMETIC leaf sharing a translation unit with a framed
// function** — the pair `wunw_float_neg.cpp` used to hold as a negative, and the
// last of the eighteen (FP leaf, framed family) pairs `docs/CROSS_PRODUCT.md`
// counted as never emitted in ANY configuration.
//
// ## What was actually wrong, and it was one predicate
//
// `IlFunction::label_slots` answered `None` — "undetermined" — for *every* float
// leaf, on the reading "a float leaf is 2, or 4/6 with pooled constants", and
// the TU-level gate in `IlBundle::functions` refuses the whole TU when any
// non-framed function's stride disagrees with what `coff::plan_labels` advances.
// So one `None` refused every pair of (float or double leaf) x (any framed
// family): call-sequence and its six tails, the single framed call, and the
// empty constructor delegating to a base.
//
// The 2 was never the function's own stride. It is `1 + the TU's `_fltused`
// slot`, and that slot has been `plan_labels`'s — charged once per TU, not once
// per FP function — since the eleven-row table in `w28_fp_store_framed.cpp`.
// The counter was therefore already getting this class right, and the gate was
// refusing on a number the emitter no longer used.
//
// MEASURED seed-free and in-TU, with the in-TU anchor control holding on every
// row (`docs/LABEL_COUNTER.md` §1, `scripts/gt_label_stride.py`):
//
//   leaf-float          float leaf, first FP function in the TU     stride 2
//   leaf-float-led      float leaf, `_fltused` charged to a lead    stride 1
//   leaf-double-led     double leaf, `_fltused` charged to a lead   stride 1
//   leaf-float-c1-led   float leaf, ONE newly pooled constant       stride 3
//   leaf-float-c2-led   float leaf, TWO newly pooled constants      stride 5
//   const1-dup-led      reuses a constant an earlier function pooled   +0
//
// The first three rows are this file. The last three are why the pooled-constant
// leaf is still refused, and they are `wunw_float_neg.cpp` and
// `w28_fp_store_framed_neg.cpp`.
//
// ## What this file separates
//
// Both orders, both widths, and an integer leaf between the FP leaf and the
// framed function — because a counter error an adjacent function absorbs is
// invisible without a separator, and because `_fltused` is placed after the
// **first** FP-touching function's complete symbol group, which makes order a
// real axis rather than a cosmetic one.

int g(int);
void q1();
void q2();

// FP leaf FIRST, so `_fltused` lands ahead of both framed functions and the
// leaf's own slot is the one under test.
float fmul(float a, float b) { return a * b; }

// A stride-1 integer leaf as the separator: an error of one in the FP leaf's
// stride cannot be absorbed by this.
int inc(int a) { return a + 1; }

// The framed half — Class A many-calls, 4 packed / 5 under `/Gy`. It is the only
// thing that makes the counter observable at all.
void seq2() { q1(); q2(); }

// A double leaf AFTER the framed function: `_fltused` is already charged, so
// this row is `leaf-double-led` = 1 and it moves the labels of everything below.
double dmul(double a, double b) { return a * b; }

// A second framed function, so the stride of everything between the two is read
// as a DIFFERENCE and does not depend on the `.gl` seed.
int addk(int a) { return g(a) + 1; }

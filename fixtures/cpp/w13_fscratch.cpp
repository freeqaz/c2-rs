// W13a — the floating-point temporary allocator.
//
// The FP scratch order is NOT the integer rotating cursor of
// docs/CODEGEN_W5_SCRATCH.md. It is a descending rotating cursor over the pool
// [f0, f13, f12, ..., f2, f1] — f0 first, then down from f13, wrapping — that
// skips registers still holding a live value and forces the last value into
// f1. `fm13` is the discriminating case: 13 live parameters force the cursor to
// walk almost the whole pool, and it wraps twice.
//
// Also pinned here: an FP additive chain does NOT collapse to a single
// accumulator the way the integer `add` chain does — every intermediate is a
// distinct value with its own register.

float fm3(float a, float b, float c)                       { return a * b * c; }
float fm4(float a, float b, float c, float d)              { return a * b * c * d; }
float fm5(float a, float b, float c, float d, float e)     { return a * b * c * d * e; }
float fm6(float a, float b, float c, float d, float e,
          float f)                                         { return a * b * c * d * e * f; }

float fs4(float a, float b, float c, float d)              { return a - b - c - d; }
float fa4(float a, float b, float c, float d)              { return a + b + c + d; }

float ft2(float a, float b, float c, float d)              { return (a + b) * (c + d); }
double dt2(double a, double b, double c, double d)         { return (a + b) * (c + d); }

// All 13 FPRs live at entry: the cursor must skip live parameters and wrap.
float fm13(float a1, float a2, float a3, float a4, float a5, float a6,
           float a7, float a8, float a9, float a10, float a11, float a12,
           float a13) {
  return a1 * a2 * a3 * a4 * a5 * a6 * a7 * a8 * a9 * a10 * a11 * a12 * a13;
}

// Written in the reverse order — c2 flattens and re-linearizes the product, so
// this must compile to bytes identical to `fm13`.
float fm13r(float a1, float a2, float a3, float a4, float a5, float a6,
            float a7, float a8, float a9, float a10, float a11, float a12,
            float a13) {
  return a13 * a12 * a11 * a10 * a9 * a8 * a7 * a6 * a5 * a4 * a3 * a2 * a1;
}

// A single skip: the cursor steps over f13 because `m` is still live.
float fskip(float a, float b, float c, float d, float e, float f, float g,
            float h, float i, float j, float k, float l, float m) {
  return (a * b) * (c * d) * m;
}

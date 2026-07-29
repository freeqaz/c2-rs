// W13b **negative** — bodies whose IL carries more than one floating-point
// literal. Every function here must keep returning `NotImplemented`; the
// positive dedup/width/placement cases live in `w13b_fdedup.cpp`.
//
// The reason these refuse is not that c2 does something exotic with the *obj*
// shape — it is that **c2, not c1xx, folds floating-point constants**. Both
// bodies below reach the backend with two literals in the IL and leave it having
// pooled exactly one:
//
//   * `ke`   → c2 reassociates to `(a*b) * 6.0f`, pooling `__real@40c00000`;
//   * `kdiv` → c2 turns `a/3.0f/7.0f` into one `fmuls` by the reciprocal,
//     pooling `__real@3d430c31` (1/21) — a value that is not even exactly
//     representable, so this is a real numeric transform, not a rewrite.
//
// Modeling either means modeling c2's constant evaluator, so the port refuses
// any body with two or more FP literals. Two further captures (`p1` =
// `(a+1.0f)-(b+2.0f)`, `p5` = `a+1.0f-b-2.0f+c`) show what it would also have to
// model to emit two *surviving* constants: c2 hoists every `addis` into a
// prologue group in IL order (r11 then r10), schedules each `lfs` at its first
// use, and recycles the FP register once a constant dies (`p5` reloads into
// `f0`). So with two constants the REFLO site is no longer `hi_off + 4` — the
// assumption the one-constant path is built on. See
// `docs/CODEGEN_W13_FLOAT.md` §5.3.

float ke(float a, float b) { return a * 2.0f * b * 3.0f; }
float kdiv(float a) { return a / 3.0f / 7.0f; }

// W13b — the positive multi-constant witness: four functions, one pooled
// constant each. What it pins that the single-function `w13b_fconst.cpp` cannot:
//
//   * `ka` and `kc` hold the same value, and the obj has **one** `.rdata` COMDAT
//     for it — dedup is TU-wide, and both reference sites relocate against the
//     same `__real@3f800000`;
//   * `kd` is the same *number* at double width and gets its own 8-byte COMDAT
//     and its own `__real@3ff0000000000000`, so the pool is keyed on the bit
//     pattern **and** the width, not the value;
//   * each `.rdata` section symbol + `__real@…` external is emitted immediately
//     after the symbol of the function that *first* references it — not grouped
//     at the end — so `_fltused` lands after `ka`'s group, not after `ka`;
//   * a section's relocations sit right after **that section's** raw data, so the
//     four `.text` REFHI/REFLO records fall between `.text` and the first
//     `.rdata` rather than after every section (the bug this fixture caught).
//
// The bodies that make c2 *fold* constants, and the two-live-constant scheduling
// it switches to, live in `w13b_fpool.cpp` and must keep refusing.

float ka(float a) { return a + 1.0f; }
float kb(float a) { return a + 2.0f; }
float kc(float a) { return a + 1.0f; }
double kd(double a) { return a + 1.0; }

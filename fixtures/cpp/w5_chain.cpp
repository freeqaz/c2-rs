// W5 negative-by-surprise: LINEAR chains of three or more ops. These are inside
// the *current* operand-stack-depth-2 gate, but the current single-scratch (r11)
// model is only right for chains whose last op is `add`. c2 gives every
// intermediate of a `*`/`-` chain its own register (r11, r10, r9, ...), so
// `a*b*c*d` is `mullw r11 ; mullw r10 ; mullw r3`, not `mullw r11 ; mullw r11 ;
// mullw r3`. Add-chains keep one register because the additive canonicalization
// collapses them to a single accumulator value.
//
// These fixtures make that divergence a first-class, checkable fact rather than
// an uncovered corner. See docs/CODEGEN_W5_SCRATCH.md.
int c4_mul(int a, int b, int c, int d) { return a * b * c * d; }

int c4_sub(int a, int b, int c, int d) { return a - b - c - d; }

int c4_add(int a, int b, int c, int d) { return a + b + c + d; }

int c5_mul(int a, int b, int c, int d, int e) { return a * b * c * d * e; }

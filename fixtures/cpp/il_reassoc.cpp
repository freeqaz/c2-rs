// Arithmetic chains, in every operand order — all byte-exact via canonicalization.
//
// c2 does not evaluate a chain left to right. It canonicalizes by **register**:
//
//   * a multiplicative chain sorts its factors ascending;
//   * an additive chain is treated as a sum of signed terms and emitted as the
//     lowest-numbered positive register, then every negative register ascending,
//     then the remaining positives ascending, then the folded literal.
//
// So all five permutations of `a + b + c` emit the identical
// `add r11,r3,r4 ; add r3,r11,r5`, `b + a` emits the same `add r3,r3,r4` as
// `a + b`, and these three are byte-identical to each other in pairs:
//
//   a + b - c   ->  subf r11,r5,r3 ; add r3,r11,r4     = (a - c) + b
//   b - c + a   ->  subf r11,r5,r3 ; add r3,r11,r4       the same bytes
//   a - c - b   ->  subf r11,r4,r3 ; subf r3,r5,r11    = (a - b) - c
//   a + b - 1   ->  add r11,r3,r4 ; addi r3,r11,-1       literal folded, applied last
//
// The port used to emit **source** order, which is numerically right and byte-wrong
// for every chain not already written in canonical form — roughly five in six of the
// cases below. That was ~20 distinct mis-emits in the straight-line class, the one
// described as byte-exact since the MVP, and no hand-written fixture had found any
// of them: every positive in the corpus happened to use distinct operands in
// ascending order. They were found by `scripts/expr_sweep.sh`, which enumerates
// ~2,350 such expressions; run it after any change to expression selection.
//
// The pairs matter more than the individual cases. `a - b + c` (already canonical,
// always worked) sitting next to `a + b - c` (reassociated, used to mis-emit) is the
// whole content of the additive rule — the two differ only in which operator comes
// first. Likewise `a - b - c` against `a - c - b`. A gate or a rewrite that treats
// mixed chains as one category gets one of each pair wrong.
//
// `lit_only` and `add_then_lit_sub` pin that a literal term folds into the running
// `addi` immediate and is applied last, so it never participates in the ordering.

int swap_add(int a, int b) { return b + a; }
int swap_mul(int a, int b) { return b * a; }
int swap_sub(int a, int b) { return b - a; }

int perm3_abc(int a, int b, int c) { return a + b + c; }
int perm3_acb(int a, int b, int c) { return a + c + b; }
int perm3_bac(int a, int b, int c) { return b + a + c; }
int perm3_bca(int a, int b, int c) { return b + c + a; }
int perm3_cba(int a, int b, int c) { return c + b + a; }
int perm3_mul(int a, int b, int c) { return a * c * b; }

int mixed_add_then_sub(int a, int b, int c) { return a + b - c; }
int mixed_sub_then_add(int a, int b, int c) { return a - b + c; }
int mixed_sub_mid(int a, int b, int c) { return b - c + a; }
int mixed_sub_asc(int a, int b, int c) { return a - b - c; }
int mixed_sub_desc(int a, int b, int c) { return a - c - b; }

int add_then_lit_sub(int a, int b) { return a + b - 1; }
int lit_only(int a) { return a + 1 - 2; }

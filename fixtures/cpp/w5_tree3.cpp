// W5, depth-3 balanced trees — the multi-scratch case proper. A `-` root over
// two `*` nodes over four `+` nodes: nothing is absorbable, so c2 emits all six
// interior nodes as separate live values and the scratch descent is visible in
// full.
//
// `t3_four` (4 params, so r7..r11 are free) pins the pure descent
// r11, r10, r9, r8, r7, r6 -> r3. `t3_eight` (8 params, so only r11 is free at
// entry) pins the *liveness-driven* form of the same rule: the cursor skips
// argument registers that are still live and recycles them as they die, then
// wraps back to r11. See docs/CODEGEN_W5_SCRATCH.md.
int t3_four(int a, int b, int c, int d) {
    return ((a + b) * (c + d)) - ((a + c) * (b + d));
}

int t3_eight(int a, int b, int c, int d, int e, int f, int g, int h) {
    return ((a + b) * (c + d)) - ((e + f) * (g + h));
}

// Asymmetric: the small right subtree still takes the next cursor slot, and the
// only difference between the two operand orders is the final subf's operands
// (the whole prefix is byte-identical) — c2 does NOT re-rank the subtrees.
int t3_wide(int a, int b, int c, int d, int e, int f) {
    return ((a + b) * (c + d)) - (e * f);
}

int t3_wide_flip(int a, int b, int c, int d, int e, int f) {
    return (e * f) - ((a + b) * (c + d));
}

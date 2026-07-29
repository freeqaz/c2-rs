// W5 NEGATIVE neighbours — tree-shaped source that c2 does NOT lower as a tree.
// Every one of these must keep returning NotImplemented after the depth-2 gate
// is lifted; a naive "post-order, one register per node" selector would emit
// plausible-but-wrong bytes for each. See docs/CODEGEN_W5_SCRATCH.md.

// (1) Product flattening: a `*` node with a `*` child becomes ONE n-ary product,
//     re-linearized into a chain, so the tree shape disappears from the obj.
int n_mul_of_mul(int a, int b, int c, int d) { return (a * b) * (c * d); }
int n_mul_of_add(int a, int b, int c, int d) { return (a + b) * (c * d); }
int n_right_mul(int a, int b, int c, int d) { return a * (b * (c * d)); }

// (2) Additive canonicalization: an additive node with an additive child becomes
//     one n-ary sum, its terms REORDERED (subtracted terms first, then added
//     ones), accumulated in a single register. The leaf order in the obj no
//     longer matches the source.
int n_add_of_add(int a, int b, int c, int d) { return (a + b) + (c + d); }
int n_sub_of_add(int a, int b, int c, int d) { return (a + b) - (c + d); }
int n_right_sub(int a, int b, int c, int d) { return a - (b - (c - d)); }
int n_reorder(int a, int b, int c, int d) { return a + b - c - d; }

// (3) Immediates folded into a *sum node*: `(a+b)+1` is a three-term sum, not an
//     affine tweak of a tree node, and its materialization does not follow the
//     plain "one addi in place" rule.
int n_imm_sum(int a, int b, int c, int d) { return (a + b + 1) * (c + d); }
int n_imm_sum2(int a, int b, int c, int d) { return ((a + b) + 1) * ((c + d) + 2); }

// (4) Subtree re-ranking: when the two children need different numbers of
//     registers, c2 emits the heavier one first, so the register order does not
//     follow source order.
int n_rerank(int a, int b, int c, int d) { return (a + b) * (c + d + 1); }

// (5) Register pressure past the volatile set: a depth-5 tree over 4 params
//     forces c2 to save and restore a NON-volatile (r31) around the body, adding
//     a prologue/epilogue that the leaf codegen has no model for at all.
int n_spill(int a, int b, int c, int d) {
    return (((a + b) * (c + d)) - ((a + c) * (b + d))) *
           (((a + d) * (b + c)) - ((a - b) * (c - d)));
}

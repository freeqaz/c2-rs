// **W71 positives — the neighbours `expr-alloc-undetermined` must NOT take.**
// `lane w-build`. Read `w71_alloc_undetermined_neg.cpp` first; it carries the
// mis-emit this pair exists for.
//
// A guard that refuses the divergent chain is only worth having if it refuses
// *only* that. The rows below are every four-leaf shape adjacent to
// `((a + b) * c) * d` in the operator lattice, and every one of them was
// re-graded against real c2 after the guard went in. They are here so that a
// future widening of the guard — or a future *fix* that removes it — has to
// keep them.
//
// The measured allocations at `/Ox` (`work/w-build/probe/alloc-Ox.cod`):
//
//     a * b * c * d       mullw r11 ; mullw r10 ; mullw r3      descending
//     a - b - c - d       subf  r11 ; subf  r10 ; subf  r3      descending
//     (a * b) - c - d     mullw r11 ; subf  r10 ; subf  r3      descending
//     (a * b) + c + d     mullw r11 ; add   r11 ; add   r3      collapsed
//     a + b + c + d       add   r11 ; add   r11 ; add   r3      collapsed
//
// The last three are what makes the boundary legible: an `add` **consuming** an
// intermediate collapses it to r11, and an `add` **producing** one does not.
// Both of the tree's rules agree on all five; they part company only at
// `((a + b) * c) * d`, where the add produces and a `mullw` consumes.
//
// `a + b - c - d` and `a + b + c - d` reach codegen reassociated — c2
// canonicalizes an additive chain and `canonicalize_chain` reproduces it — so
// they are in this file at their SOURCE spelling on purpose: the guard runs on
// the canonicalized stream (`a - c - d + b`, whose trailing `add` has no
// successor), and a guard that ran on the source order would refuse them.
int p_mul4    (int a, int b, int c, int d) { return ((a * b) * c) * d; }
int p_sub4    (int a, int b, int c, int d) { return ((a - b) - c) - d; }
int p_add4    (int a, int b, int c, int d) { return ((a + b) + c) + d; }
int p_mul_sub (int a, int b, int c, int d) { return ((a * b) - c) - d; }
int p_mul_add (int a, int b, int c, int d) { return ((a * b) + c) + d; }
int p_sub_add (int a, int b, int c, int d) { return ((a - b) - c) + d; }
int p_add_sub (int a, int b, int c, int d) { return ((a + b) - c) - d; }
int p_add2_sub(int a, int b, int c, int d) { return ((a + b) + c) - d; }
// Three leaves — one intermediate, where every candidate rule agrees and the
// guard must therefore stand aside however the operators are arranged.
int p_add_mul (int a, int b, int c)        { return (a + b) * c; }
int p_mul_mul (int a, int b, int c)        { return (a * b) * c; }
int p_sub_mul (int a, int b, int c)        { return (a - b) * c; }
// Five leaves, no add before a non-add.
int p_mul5    (int a, int b, int c, int d, int e) { return (((a * b) * c) * d) * e; }

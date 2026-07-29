// W5 (multi-scratch expressions), depth-2 trees — the shapes c2 keeps as REAL
// trees (both operand values live at once, so a second scratch is mandatory).
// The IL is plain postfix: LOAD a, LOAD b, <op>, LOAD c, LOAD d, <op>, <root>.
//
// Verified reference `.text` (see docs/CODEGEN_W5_SCRATCH.md): the left node
// takes r11, the right node r10, the root r3 — EXCEPT for the `+` root, where
// the two swap (left=r10, right=r11). A tree stays a tree only when the root
// cannot absorb a child: a `*` root with no `*` child, or an additive root
// (`+`/`-`) whose children are BOTH `*`. Every other 4-leaf shape is
// canonicalized by c2 into a linear chain — those live in w5_tree_neg.cpp.
int t2_mul_add(int a, int b, int c, int d) { return (a + b) * (c + d); }

int t2_mul_sub(int a, int b, int c, int d) { return (a - b) * (c - d); }

int t2_sub_mul(int a, int b, int c, int d) { return (a * b) - (c * d); }

int t2_add_mul(int a, int b, int c, int d) { return (a * b) + (c * d); }

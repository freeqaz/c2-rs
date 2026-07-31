// WAE **positive** — the assignment-statement class, unchanged.
//
// This rung admits nothing new. It moves *when* the destination gate speaks
// (`crates/c2-il/src/func/body/shapes/assign.rs`, `dst_not_formal`): the refusal
// is recorded on the offending `26` push and raised only after the whole body
// has parsed, so the census names the innermost unmodeled construct instead of
// this outer gate. The accepted set must therefore be **bit-identical**, and
// this file is the control group that says so — every body below is one c2
// register-allocates and coalesces away, and every one of them was in class
// before the change.
//
// `int x; x = a; return x;` is a bare `blr`; the chains fold to the expression
// that actually reaches the `return`; a dead store disappears because only the
// last definition can reach it.

int wae_copy(int a) { int x; x = a; return x; }
int wae_two_locals(int a) { int x = a + 1; int y = x + 2; return y; }
int wae_dead_store(int a) { int x = 0; x = a + 1; return x; }
int wae_to_formal(int a) { a = a + 1; return a; }
int wae_literal(int a) { a = 7; return a; }

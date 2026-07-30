// **Characterization** — a call consumed as a *value*, which is the census bucket
// `expr-call-in-expr` (164,544 functions, 6.9%) plus much of `call-token-0x26`
// (125,226, 5.3%).
//
// There is no new opcode. `int z = g(a);` is an ordinary assignment statement whose
// right-hand side happens to be a CALL, so the production is already decoded — what
// is missing is that `parse_expr` has no representation for "the value a call
// produced", and blocks on the `26` that pushes the callee.
//
//   v_pass   int z = g1(a); return z;
//     26 <z>                                     the destination
//     26 <g1> bd 86 41 74 00 80 14 10 00 00      the call
//     b9 <a> 86 41 74 55 86 41 74 4c             its argument
//     32 86 41 74 4b                             store into z
//     b9 <z> 86 41 74 41 86 41 74                return z
//
// **The reference obj for that body is byte-identical to `return g1(a);`** — a bare
// `b ?g1@@YAHH@Z` with a REL24 relocation, no frame, no store. c2 register-allocates
// `z` away exactly as it does for a non-call initializer. And `v_plus1`
// (`int z = g1(a); return z + 1;`) is byte-identical to `return g1(a) + 1;`, i.e.
// the existing framed non-leaf class: the 0x24-byte frame, `bl g1`,
// `addi r3,r3,1`, epilogue.
//
// So the two already-implemented call shapes cover this bucket's simplest members,
// and the widening is in the *IL model* (an env entry that is a call rather than an
// operand stream), not in codegen. That is the cheapest remaining rung in this
// document and it is not attempted here — `try_parse_assign_body` resolves
// statements by substituting operand streams, and a call value is not one.
//
// The gate that must survive any such widening is `v_twice`: two calls in one body
// is a real non-leaf with a frame and two `bl`s, and nothing about the single-call
// shape generalizes to it. `v_arg` (a call whose result is another call's argument)
// and `v_sum` (two call results added) are the same warning.
//
// `v_void` is here to pin the contrast: a void call statement has no destination and
// no `32`, so `26 <callee> BD … 4C 4B` — which is why the leading `26` alone cannot
// tell an assignment from a call, and why the discriminator has to be whether the
// token that follows the first `26 <tok>` is `BD`.

extern int g1(int);
extern int g2(int, int);
extern void gv();

int v_pass(int a) {
    int z = g1(a);
    return z;
}

int v_plus1(int a) {
    int z = g1(a);
    return z + 1;
}

int v_direct(int a) { return g1(a); }

int v_twice(int a) {
    int z = g1(a);
    int w = g1(z);
    return w;
}

int v_arg(int a) { return g1(g1(a)); }
int v_sum(int a) { return g1(a) + g2(a, a); }

void v_void() { gv(); }

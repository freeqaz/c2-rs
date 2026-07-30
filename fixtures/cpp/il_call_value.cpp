// A call whose result is bound to a local and returned immediately. Byte-exact,
// and it needed no new codegen at all.
//
// `int z = g(a); return z;` compiles to exactly `b <callee>` — identical to
// `return g(a);`. c2 register-allocates the local and coalesces the copy away, so
// the two spellings are the same program and the same bytes. Captured on the one-,
// two- and three-argument forms, and on a permuted argument list:
//
//   int z = g1(a); return z;          ->  b <callee>
//   int z = g2(a,b); return z;        ->  b <callee>
//   int z = g3(b,a,c); return z;      ->  mr r11,r4 ; mr r4,r3 ; mr r3,r11 ; b <callee>
//
// This was the largest single census blocker after the gate migration —
// `expr-call-in-expr` at 12.3% of blocked functions — and the whole of it was a
// missing IL model rather than a missing lowering. The bodies route into the
// existing `IntTailCall` / `MultiArgTailCall` productions.
//
// Two restrictions, both load-bearing:
//
//   * it must be the FIRST statement. With earlier assignments already folded into
//     the substitution environment, handing the body to the call parser would drop
//     them silently.
//   * the token read back must be the very one just written. Returning a different
//     local is a different program, and there is nothing in the shape to make that
//     safe.
//
// This does not reopen the question `il_stmt_static.cpp` closed. That fixture is
// about a local's *address* — a store to a global or a `static` is a real memory
// write and must refuse. Here the local never becomes a memory object: the value is
// returned and never written anywhere, and the shape admits nothing between the
// store and the return.
//
// `arg_lit` stays out of class: a literal argument is materialized in place, which
// the permutation form cannot express (see `il_call_multi.cpp`).

int g1(int);
int g2(int, int);
int g3(int, int, int);

int one_arg(int a) { int z = g1(a); return z; }
int unused_param(int a, int b) { int z = g1(a); return z; }
int two_args(int a, int b) { int z = g2(a, b); return z; }
int three_args(int a, int b, int c) { int z = g3(a, b, c); return z; }
int permuted(int a, int b, int c) { int z = g3(b, a, c); return z; }

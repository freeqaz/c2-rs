// Automatic `int` **locals**, admitted on `.sy`'s positive evidence — the first
// feature to be gated on a fourth IL file. See `crates/c2-il/src/func/sy.rs` for
// the grammar and `docs/GAPS.md` §6 for why an absence test was not allowed to
// stand in for it.
//
// `.ex` pushes an assignment destination as `26 <tok>` whether the destination is
// a parameter, a local, a file-scope `static` or a global, so nothing in the body
// stream distinguishes them. `.sy` does: a flat sequence of per-function blocks,
// each with a formals section (`0D 01`) and a locals section (`0D 02`), and a
// global or `extern` appears in neither. That turns the question from "is this
// token absent from a table" — which silently dropped a store to `$sv` — into
// "is this token present in this function's locals section".
//
// Locals need no codegen at all. c2 register-allocates them and coalesces the
// copies, so the class collapses to the expression that reaches the `return`.
// Measured, not assumed: `two_locals` below and `int f(int a){return (a+1)+2;}`
// produce reference objs that differ in exactly three bytes, all of them the
// source filename.
//
// The negatives are the point of this file. Each is one field away from an
// admitted local, and each would be a wrong-bytes emit if that field were read as
// a constant:
//
//   volatile_local   `<enc>` 80 00 10 00 00, NOT 74   — the store must survive
//   const_local      `<enc>` 80 01 10 00 00           — same field, other value
//   addr_taken       `<F>`   21, NOT 01               — the value escapes
//   unsigned_local   `<T>` 02 / `<enc>` 75            — not `int`
//   ptr_local        `<T>` 03 / `<enc>` 80 74 04 00 00
//   static_local     a `07` record, not `01`          — a memory object
//   file_static      absent from `.sy` entirely       — the original `$sv` bug
//
// `volatile_local` and `const_local` share `<T> = 86 01` with a plain `int`, so a
// gate written against the type reference alone admits both and folds away a
// volatile store. `addr_taken` shares `<T>` *and* `<enc>` with a plain `int` and
// differs only in `<F>`; `mixed_addr` is the discriminating neighbour, holding one
// address-taken and one ordinary local in the same function so that "always 01"
// and "01 by coincidence" stop being the same observation.
//
// `static_local` is the load-bearing refusal: a function-scope `static` is a real
// memory object with a relocation, and it lives in the locals section, so only the
// record tag separates it. Its layout is not characterized, so `sy_blocks` refuses
// the whole file rather than guess a width and risk resyncing onto a later
// record — which would rebind a different function's token. That is why this TU
// admits no locals at all while `il_sy_locals_clean.cpp` does.

// `local_then_formal` is the one negative here that is not about `.sy` at all. It
// decodes — the census reports it in class — and then codegen refuses it, because
// substituting `s` into `s + b` yields `(a + 1) + b`, whose operand order the
// chain lowering does not accept. That gate predates locals: written without one,
// `int f(int a, int b) { return b + (a + 1); }` refuses identically, while
// `return (a + 1) + b` matches. Substitution does not create the gap, it just
// reaches it more often — worth keeping in sight, since it is the first case where
// admitting a decode moved the bottleneck into codegen rather than closing it.

int side(int*);

int one_local(int a) {
    int x = a + 1;
    return x;
}

int two_locals(int a) {
    int x = a + 1;
    int y = x + 2;
    return y;
}

int dead_store(int a) {
    int x = 0;
    x = a + 1;
    return x;
}

int reassigned(int a) {
    int x = a + 1;
    x = x + 2;
    return x;
}

int three_formals_two_locals(int a, int b, int c) {
    int p = a + b;
    int q = p + c;
    return q;
}

int volatile_local(int a) {
    volatile int x = a + 1;
    return x;
}

int const_local(int a) {
    const int x = a + 1;
    return x;
}

int addr_taken(int a) {
    int x = a + 1;
    return side(&x);
}

int mixed_addr(int a) {
    int x = a + 1;
    int y = a + 2;
    return side(&x) + y;
}

int unsigned_local(int a) {
    unsigned x = a + 1;
    return (int)x;
}

int ptr_local(int* a) {
    int* p = a;
    return *p;
}

int static_local(int a) {
    static int x;
    x = a;
    return x;
}

static int sv;

int file_static(int a) {
    sv = a;
    return a;
}

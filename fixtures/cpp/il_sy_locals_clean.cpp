// The positive half of `il_sy_locals.cpp`, and its control.
//
// That file's `static_local` makes `.sy` unreadable for the whole translation
// unit — a `07` record whose layout is not characterized, so `sy_blocks` refuses
// rather than guess a width and risk resyncing onto another function's record. So
// every function there is refused, including the ones that would otherwise be in
// class, and it cannot show that locals are admitted at all.
//
// This file is the same local shapes with nothing that spoils `.sy`. The pair is
// the measurement: if this one admits locals and that one does not, the
// TU-granularity refusal is real and deliberate rather than an accident. If this
// one ever stops admitting them, the reader broke — the two files move together
// only if the cause is the reader and not the `07` record.
//
// `chain_of_four` is here because substitution is what folds the class away, and
// its depth is bounded (`assign-too-many-locals`): a chain that outgrows the bound
// must refuse cleanly, not truncate. `passthrough` and `copy_chain` collapse to a
// bare `blr` — the local never becomes an instruction at all, which is the
// clearest statement of why this needed no codegen work.

int passthrough(int a) {
    int x = a;
    return x;
}

int copy_chain(int a) {
    int x = a;
    int y = x;
    return y;
}

int literal_local(int) {
    int x = 7;
    return x;
}

int one_local(int a) {
    int x = a + 1;
    return x;
}

int two_locals(int a) {
    int x = a + 1;
    int y = x + 2;
    return y;
}

int chain_of_four(int a) {
    int p = a + 1;
    int q = p + 2;
    int r = q + 3;
    int s = r + 4;
    return s;
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

int local_of_two_formals(int a, int b) {
    int s = a + b;
    return s;
}

int mul_local(int a, int b) {
    int p = a * b;
    return p;
}

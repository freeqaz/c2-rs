// **Negative** — a framed call in a function with more than eight formals.
//
// Past the eighth formal the value is stack-homed, so the argument setup is a
// load from the frame rather than a register move. Measured:
//
//   int f(int a,…,int i) { return g(i) + 1; }
//   c2:  7d8802a6 9181fff8 9421ffa0 [806100b4] 4bfffff1 …   lwz r3,180(r1)
//
// The constant-body emitter emitted *nothing* there, so this was part of the
// same live wrong-bytes emit `wfr_argreg.cpp` documents — and the half of it
// that a register-move model does not fix, because the slot displacement is a
// function of the whole parameter list's ABI footprint, which this port has not
// characterized (`il_param_aggr_neg.cpp` says the same about aggregates).
//
// The refusal is on the formals LIST, not on the argument's index, because that
// is the predicate `select_text` — which computes the setup — actually raises.
// It is therefore more conservative than the ABI requires: `useful` below has
// its argument in r3 and would emit the plain 0x24 body. Sizing that
// over-refusal rather than leaving it a rumour: on the 878-TU workload it costs
// **zero** functions, numerator identical with and without the gate.

int g(int);

// Argument past the eighth: genuinely needs the stack load.
int stack_arg(int a, int b, int c, int d, int e, int f, int h, int i, int j) {
    return g(j) + 1;
}

// Argument in r3, refused anyway — the measured cost of keeping the parser gate
// and `select_text` on one predicate.
int useful(int a, int b, int c, int d, int e, int f, int h, int i, int j) {
    return g(a) + 1;
}

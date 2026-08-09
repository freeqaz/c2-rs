// w-fltret probe v3 — the WIDTH cells. The callee's real result and the
// function's own real result are two types, and c2's `.cod` says whether the
// mismatch costs an instruction.
struct O {
    void   Poll();
    float  F();
    double D();
};

// same width — the reference cells
float  w_ff(O *o) { o->Poll(); return o->F(); }
double w_dd(O *o) { o->Poll(); return o->D(); }

// NARROWING: a double callee returned as float — `frsp f1,f0`?
float  w_df(O *o) { o->Poll(); return o->D(); }

// WIDENING: a float callee returned as double
double w_fd(O *o) { o->Poll(); return o->F(); }

// an FP post-op on the returned value
float  w_post(O *o) { o->Poll(); return o->F() + 1.0f; }

// a DISCARDED float member result — the `call-ret-fp` side of the obligation
void   w_disc(O *o) { o->F(); o->Poll(); }

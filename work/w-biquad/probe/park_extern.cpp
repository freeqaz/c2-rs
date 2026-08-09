// w-biquad PROBE — the ctor park register, cell E: the SAME ctor, but the
// forwarded callee is an UNDEFINED external. M-RULE predicts a CALLEE-SAVED
// park (`mr r31,r3` plus a `std`/`ld` pair).
struct E { float c[4]; E(float *f); void S(float *f); };
E::E(float *f) { S(f); }

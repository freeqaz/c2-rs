// w-biquad PROBE — the ctor park register, cell L: the forwarded callee is
// defined HERE (Biquad's own shape). M-RULE predicts a VOLATILE park.
struct L { float c[4]; L(float *f); void S(float *f); };
void L::S(float *f) { c[0] = f[0]; }
L::L(float *f) { S(f); }

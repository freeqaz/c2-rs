// w-counted lead cell: N counted-accumulate loops (all LEAVES) before the
// SAME framed z9, so the framed function's $M triple is the only thing that
// varies and each TU's own .gl counter cancels INSIDE the TU (board #3148).
// z9 is wbdnz_ctr_then_framed_neg.cpp's own framed function.
int gz(int);

int p_uns(unsigned n, int k) { int s = 0; for (unsigned i = 0; i < n; ++i) s -= k; return s; }

int z9(int a) { return gz(a) + 7; }

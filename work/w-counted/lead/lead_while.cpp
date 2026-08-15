// w-counted lead cell: N counted-accumulate loops (all LEAVES) before the
// SAME framed z9, so the framed function's $M triple is the only thing that
// varies and each TU's own .gl counter cancels INSIDE the TU (board #3148).
// z9 is wbdnz_ctr_then_framed_neg.cpp's own framed function.
int gz(int);

int p_while(int n, int k) { int s = 0; int i = 0; while (i < n) { s -= k; ++i; } return s; }

int z9(int a) { return gz(a) + 7; }

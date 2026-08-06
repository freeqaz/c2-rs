// GRID-S cell s10_seq_one_call_tail — a SEQ with one call and a non-void tail — the 816-function shape
int g(int a) { return a + 1; }
int f(int a) { int t = g(a); return t + t; }

void ext_anchor();
void anchor() { ext_anchor(); }

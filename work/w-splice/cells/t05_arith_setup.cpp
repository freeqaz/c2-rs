// GRID-T cell t05_arith_setup — S3 REFUSES — the setup is arithmetic; c2 folds the two literals
int g(int a) { return a + 1; }
int f(int a) { return g(a + 1); }

void ext_anchor();
void anchor() { ext_anchor(); }

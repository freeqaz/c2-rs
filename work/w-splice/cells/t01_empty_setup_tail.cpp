// GRID-T cell t01_empty_setup_tail — FIRES — empty setup, a leaf callee the port lowers
int g(int a) { return a + 1; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

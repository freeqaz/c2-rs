// GRID-T cell t04_reg_move_setup — S3 REFUSES — the setup is a register move; c2 renames a field of the callee's body (?Release@Object@Hmx@@, 286 pairs)
int g(int a) { return a + 1; }
int f(int a, int b) { return g(b); }

void ext_anchor();
void anchor() { ext_anchor(); }

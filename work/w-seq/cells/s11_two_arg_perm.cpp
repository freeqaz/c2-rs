// GRID-S cell s11_two_arg_perm — setup is a two-register permutation (#843: `sub` is not `subf`)
int g(int a, int b) { return a - b; }
int f(int a, int b) { return g(b, a); }

void ext_anchor();
void anchor() { ext_anchor(); }

// GRID-T cell t08_two_calls — S2 REFUSES — two call sites; SPLICE-N is 0 of 548
int p1;
int p2;
void g1() { p1 = 1; }
void g2() { p2 = 2; }
void f() { g1(); g2(); }

void ext_anchor();
void anchor() { ext_anchor(); }

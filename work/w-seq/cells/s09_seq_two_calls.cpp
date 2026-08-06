// GRID-S cell s09_seq_two_calls — the caller is a SEQ over two same-TU callees
int p1;
int p2;
void g1() { p1 = 1; }
void g2() { p2 = 2; }
void f() { g1(); g2(); }

void ext_anchor();
void anchor() { ext_anchor(); }

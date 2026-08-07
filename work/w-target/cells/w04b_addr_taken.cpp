// GRID-W cell w04b_addr_taken — w04 variant — the intermediate's address is taken. c2 must still emit g standalone; does it still inline at the direct site?
void ext();
void g() { ext(); }
void (*gp)() = g;
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

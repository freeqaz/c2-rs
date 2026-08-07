// GRID-W cell w07_next_link_extern — the `seq|local->extern|chain1` family's 16 — a seq caller whose chain's next link leaves the TU
void ext();
void g() { ext(); }
int side;
void f() { side = 1; g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

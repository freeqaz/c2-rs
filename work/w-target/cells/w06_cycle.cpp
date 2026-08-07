// GRID-W cell w06_cycle — TERMINATION — a two-cycle. The walk must refuse, not loop
void f();
void g() { f(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

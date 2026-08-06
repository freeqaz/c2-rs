// GRID-S cell s12_callee_calls_extern — CONTROL — the callee is not lowerable-leaf; c2 keeps a call
void ext();
void g() { ext(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

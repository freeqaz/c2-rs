// GRID-W cell w03_chain3 — DEPTH 3 — does the closure keep going, or stop at 2? The workload has no depth-3 witness, so this is the only place it can be asked
void ext();
void i() { ext(); }
void h() { i(); }
void g() { h(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

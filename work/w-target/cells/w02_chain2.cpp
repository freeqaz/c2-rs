// GRID-W cell w02_chain2 — DEPTH 2 — the `chain2` family's 73. Does ?f name ext, or h?
void ext();
void h() { ext(); }
void g() { h(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

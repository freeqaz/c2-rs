// GRID-T cell t14_extern_callee_control — S5 REFUSES (CONTROL) — the callee is not defined here; ?f keeps its REL24 against ?g
int g(int a);
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

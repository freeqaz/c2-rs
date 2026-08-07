// GRID-W cell w05_control_not_inlined — THE CONTROL CLASS — a callee c2 does not inline and the port cannot lower. R-CLOSE must NOT fire: ?f keeps its REL24 against ?g
int gsink;
int g(int a) { int t = 0; for (int i = 0; i < a; ++i) t += i * a; gsink = t; return t; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

// GRID-W cell w04a_noinline — **THE CELL THAT DECIDES THE LANE** — c2 is told not to inline the intermediate. If c2 obeys, ?f names g and R-CLOSE names ext: a DEMONSTRATED WRONG EMIT unless the port can read the attribute
void ext();
__declspec(noinline) void g() { ext(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

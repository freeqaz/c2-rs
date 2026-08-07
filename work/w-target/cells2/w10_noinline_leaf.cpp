// GRID-W2 cell w10_noinline_leaf — DOES THE SHIPPED SPLICE-0-PORT HAVE w04a's HAZARD?
//
// w09 is the cell where SPLICE-0-PORT fires and emits the callee's body for the
// caller. This is w09 with `__declspec(noinline)` on the callee. If c2 obeys the
// attribute, c2's ?f is a branch to ?g and the port's splice emits ?g's BODY —
// a byte differ, in code that already ships. Added AFTER GRID-W ran, and
// recorded as a follow-up to w04a rather than dressed up as part of the
// original grid.
int gsink;
__declspec(noinline) int g(int a) { return a + 1; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

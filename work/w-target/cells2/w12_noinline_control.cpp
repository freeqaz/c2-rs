// GRID-W2 cell w12_noinline_control — w10 WITHOUT the attribute.
//
// The negative control, in its own TU rather than inferred: a cell where the
// splice fires and is exact is the only thing that makes w10's verdict legible.
// A grid with only the suspicious cell cannot tell "the rule is wrong here"
// from "the rule is off in this build".
int gsink;
int g(int a) { return a + 1; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

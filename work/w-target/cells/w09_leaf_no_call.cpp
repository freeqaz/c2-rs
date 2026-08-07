// GRID-W cell w09_leaf_no_call — CONTROL — the chain's end carries NO call at all. c2 inlines g into f and there is no relocation left to get wrong; `close_target` must refuse with `callee-no-call`
int g(int a) { return a + 1; }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

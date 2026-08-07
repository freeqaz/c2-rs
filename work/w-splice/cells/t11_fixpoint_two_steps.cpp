// GRID-T cell t11_fixpoint_two_steps — THE FIXPOINT — a question, not a prediction. Does c2 close a two-step splice chain? The port takes ONE level in this rung either way
int h(int a) { return a + 1; }
int g(int a) { return h(a); }
int f(int a) { return g(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

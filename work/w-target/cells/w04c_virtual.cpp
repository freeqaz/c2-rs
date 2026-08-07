// GRID-W cell w04c_virtual — w04 variant — the intermediate is virtual, called non-virtually
void ext();
struct S { virtual void g(); };
void S::g() { ext(); }
void f(S* s) { s->S::g(); }

void ext_anchor();
void anchor() { ext_anchor(); }

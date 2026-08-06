// GRID-T cell t12_self_recursion — S4 REFUSES — direct self-recursion; INLINE_PREDICATE §4 grades `recurse` 336/336 refused by c2 too
int r(int a) { return a ? r(a - 1) : 0; }
int f(int a) { return r(a); }

void ext_anchor();
void anchor() { ext_anchor(); }

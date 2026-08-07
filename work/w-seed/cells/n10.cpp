// GRID-N n10 — TWO STATEMENTS. The leaf is the nothing-statement, twice.
//
// "Emits nothing" is a property of the WHOLE body, and the only thing that makes
// that claim checkable is that the walk is TOTAL — it starts at the body marker
// and must reach `eat_return_plumbing`'s fail-closed terminal, so there is
// nowhere for a second statement to hide. This cell is the source-reachable half
// of that; mutation M1 removes the terminal and a named unit test goes red.
//
// Registered: REFUSED. Two discarded pseudo-destructors emit nothing just as one
// does, so this is a match the reader declines on purpose — the shape it accepts
// is the one that was graded, and a body with something extra in it is a body it
// has not read.
struct S { int a; };

template <class T> inline void da2(T* p) { p->~T(); p->~T(); }

void use(S* p) { da2(p); }

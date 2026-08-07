// GRID-N n02 — CHAIN DEPTH with a nothing-body at the end. Three links above the
// leaf, each an ordinary void tail call the parser accepts.
//
// The seed is the only new thing: every link above it is the fixpoint w-fix
// already shipped and graded at depths 1..6 and 8. Registered: EVERY edge closes
// and every caller is one `4e800020` with no relocation, at both flag settings.
// A cell that closes at depth 1 and stops higher would say the seed reaches only
// its immediate caller, which is a link and not a seed.
struct S { int a; };

template <class T> inline void da(T* p) { p->~T(); }
template <class T> inline void d1(T* p) { da(p); }
template <class T> inline void d2(T* p) { d1(p); }

void use(S* p) { d2(p); }

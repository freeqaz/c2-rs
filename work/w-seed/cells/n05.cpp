// GRID-N n05 — A NOTHING-BODY REACHED THROUGH AN INLINE RATHER THAN AN ELISION.
// `mid` returns a value, so it is not a void tail call and c2 does not drop the
// call to it — it EXPANDS it. w-fix #954 is the trap this cell exists for: a
// mid-chain inline is a bare `blr` at every level at `/O1`, observationally
// identical to E, and only `/Ob0` tells them apart.
//
// Registered: the seed reaches `mid` and stops there. `?use` does NOT become
// `Exact` by way of E. Both callers' bytes are printed beside the verdict, and
// the `/Ob0` row is the one that says which mechanism was measured.
struct S { int a; };

template <class T> inline void da(T* p) { p->~T(); }

inline int mid(S* p, int a) { da(p); return a; }

void use(S* p, int a) { mid(p, a); }

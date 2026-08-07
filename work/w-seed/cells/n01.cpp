// GRID-N n01 — THE POSITIVE. A nothing-body reached DIRECTLY by a tail call, with
// no loop and no dead temporary between them. `p->~T()` on a class with a trivial
// destructor is an int literal, a void literal, a bind and a discard — no call in
// it at all — so for `?use` to close, the leaf must SEED E's fixpoint.
//
// Registered: `?use` grades `tail`/`Exact`, c2's own body for it is one
// `4e800020` with ZERO relocations, at `/O1` AND at `/Ob0` (E is not governed by
// `/Ob`; mechanism I is, and only that separates them — w-fix #954).
struct S { int a; };

template <class T> inline void da(T* p) { p->~T(); }

void use(S* p) { da(p); }

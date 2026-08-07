// GRID-L l09 — THE RESIDUE, and the registered STOP. The loop's leaf is a
// PSEUDO-DESTRUCTOR on a class with a trivial destructor: `p->~S()` compiles to
// an int literal, a void literal, a bind and a discard — a body with no call in
// it at all. For the chain to close, that body must SEED E's fixpoint, and
// `c2_core::elide::Reduction` documents that a refused body contributes a link
// and never a seed. c2 emits one `4e800020` for the whole chain; the port must
// convert NOTHING here, and the reason is not a missing production.
struct S { int a; };
struct false_tag {};

template <class T> inline void destroy_aux(T* p, const false_tag&) { p->~T(); }
template <class I> inline void aux(I f, I l, const false_tag&) { for (; f != l; ++f) destroy_aux(f, false_tag()); }
template <class I, class T> inline void dr(I f, I l, T*) { aux(f, l, false_tag()); }
template <class I> inline void destroy_range(I f, I l) { dr(f, l, (S*)0); }

void use(S* a, S* b) { destroy_range(a, b); }

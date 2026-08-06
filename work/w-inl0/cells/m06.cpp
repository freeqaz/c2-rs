// GRID-M m06 — THE RESIDUE. A class element type takes STLport's OTHER overload,
// the `__false_type` one, whose body is a LOOP over a destructor call. c2 still
// emits nothing for the whole chain; the port must not convert it, because the
// loop is a body the IL parser refuses. This cell is what the remaining members
// of board #980 look like.
struct T2 { int a; };
struct false_tag {};

inline void destroy_one(T2* p) { p->~T2(); }
inline void aux(T2* f, T2* l, const false_tag&) { for (; f != l; ++f) destroy_one(f); }
inline void dr(T2* f, T2* l, T2*) { aux(f, l, false_tag()); }
inline void destroy_range(T2* f, T2* l) { dr(f, l, (T2*)0); }

void use(T2* a, T2* b) { destroy_range(a, b); }

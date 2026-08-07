// GRID-N n03 — THE WORKLOAD'S OWN SHAPE, all five levels. STLport's
// `_Destroy_Range -> __destroy_range -> __destroy_range_aux(__false_type) ->
// _Destroy -> __destroy_aux(__false_type)` for a CLASS element type, with the
// names shortened. This is byte-for-byte the source of `work/w-memset/cells/l01.cpp`,
// carried over deliberately: l01/l09 are the cells that ASSERT the residue, and
// this cell is the same source with the opposite registered outcome.
//
// Registered: `?destroy_range` converts to `Exact`, and
// `destroy_loop_elision.rs::the_pseudo_destructor_leaf_is_the_residue_and_needs_a_seed`
// goes RED in the same commit. That is the intended signal, not a regression —
// w-memset put the assertion there for exactly this lane to break.
struct S { int a; };
struct false_tag {};

template <class T> inline void destroy_aux(T* p, const false_tag&) { p->~T(); }
template <class T> inline void destroy_one(T* p) { destroy_aux(p, false_tag()); }
template <class I> inline void aux(I f, I l, const false_tag&) { for (; f != l; ++f) destroy_one(f); }
template <class I, class T> inline void dr(I f, I l, T*) { aux(f, l, false_tag()); }
template <class I> inline void destroy_range(I f, I l) { dr(f, l, (S*)0); }

void use(S* a, S* b) { destroy_range(a, b); }

// GRID-N n09 — A BODY THAT KEEPS BYTES, one source character from n01. `T2` has a
// NON-TRIVIAL destructor, so `p->~T()` is a real call and not two discarded
// literals: the production changes at the front end and the reader must never see
// its shape.
//
// This is the cell that says the reader is keyed on the BODY and not on the
// spelling `p->~T()`. Registered: nothing admitted, c2 emits a real call for the
// leaf, and `?use` is an honest differ.
struct T2 { ~T2(); };

template <class T> inline void da(T* p) { p->~T(); }

void use(T2* p) { da(p); }

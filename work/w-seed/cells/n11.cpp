// GRID-N n11 — DIRECT SELF-RECURSION through a nothing-statement, and board #950
// is the reason it is graded on the BYTES.
//
// `void r(){r();}` emits a self-branch that takes NO RELOCATION AT ALL, so the
// relocation observable — the one the whole E family is built on — reads "nothing
// happened" for a body that is plainly not nothing. A cell scored by counting
// relocations would call this E. So the caller's whole `.text` is printed beside
// every verdict and the verdict is read off the bytes.
//
// Registered: `?r` is not admitted (its body carries a call, which is not in the
// reader's vocabulary), `?use` keeps its branch, and neither the port's bytes nor
// c2's are one `4e800020` for `?r`.
struct S { int a; };

template <class T> inline void r(T* p) { p->~T(); r(p); }

void use(S* p) { r(p); }

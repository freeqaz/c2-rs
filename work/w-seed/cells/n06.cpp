// GRID-N n06 — THE CYCLE, and it is PREREG §0.3 compiled.
//
// `a1` and `b1` each carry the nothing-statement AND a call to the other. The
// re-derivation says a seeded name has NO outgoing link, because the reader's walk
// is TOTAL and its vocabulary contains no call token — so a body that calls
// anything cannot be a seed, and a cycle member always calls something.
//
// Registered: the reader refuses BOTH bodies (they are not nothing-bodies with
// something extra; they are simply not in the language), the fixpoint admits
// neither, `overflowed()` is false, and `?use` keeps its branch. If either member
// were admitted, the least fixpoint's termination would rest on the round ceiling
// alone and the argument in §0.3 would be gone.
struct S { int a; };

template <class T> inline void a1(T* p);
template <class T> inline void b1(T* p) { p->~T(); a1(p); }
template <class T> inline void a1(T* p) { p->~T(); b1(p); }

void use(S* p) { a1(p); }

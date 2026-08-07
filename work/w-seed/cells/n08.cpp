// GRID-N n08 — A NOTHING-BODY WITH A DATA SYMBOL in the caller. The leaf is a
// genuine nothing-body and the caller materializes a global's address to reach it.
//
// `elide.rs`'s condition 3 exists because `g01_data_addr_arg` is E in c2 and no
// grid ever graded an elided tail call that ALSO materializes a data symbol — the
// predicate declines rather than letting the workload be the first case, and
// `data_refs_of` would in any event fail to locate a relocation half inside a
// one-word `blr`.
//
// Registered: the leaf is admitted (it really is a nothing-body) and `?use` is
// NOT elided. The refusal must come from condition 3 or from the parser, and the
// cell prints c2's own bytes so that "the port declined" and "c2 also emitted
// nothing" are told apart rather than conflated.
struct S { int a; };

int g_sink;

template <class T> inline void da(T* p, int* q) { p->~T(); }

void use(S* p) { da(p, &g_sink); }

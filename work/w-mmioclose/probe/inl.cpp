// w-mmioclose — THE DECISIVE CELL SET.  Does c2 EXPAND a same-TU callee whose
// result is USED, at the workload's own flags?
//
// `IlBundle::functions()` refuses any TU where a callee is also defined here
// ("c2 may inline it, and the port cannot").  `mmio.cpp` cannot convert while
// that clause stands, because `mmioClose` calls `mmioFlush`, defined in the
// same TU.  WB_INLINE_FINDINGS.md §7 licenses FIVE narrowings and every one is
// a DECLINE rule; none of them covers an 8-byte non-`inline` callee, which is
// exactly what `mmioFlush` is.  So the narrowing this TU needs is on the ACCEPT
// side, and §7 says in terms that the accept side is not offered.
//
// Each cell's caller uses the callee's RESULT (so no tail call, and no
// mechanism-E elision), tests it, and returns early — the `mmioClose` shape.
// If the call SURVIVES the cell emits a `bl`; if c2 expanded it, it does not.
//
// Frozen before the first compile: the prediction is that i1 KEEPS its call
// (it is `mmioFlush`'s own shape and the reference obj keeps that one) and
// that i3 (`__forceinline`) EXPANDS.  i2 is the one that decides whether a
// port-side rule can exist at all: if a plain `inline` marker changes the
// answer, the port must be able to SEE markedness in the IL, and if it cannot,
// no accept-side narrowing is sound.

int k1(int a) { return 0; }
int c1(int a) { int r = k1(a); if (r) return r; return 7; }

inline int k2(int a) { return 0; }
int c2f(int a) { int r = k2(a); if (r) return r; return 7; }

__forceinline int k3(int a) { return 0; }
int c3(int a) { int r = k3(a); if (r) return r; return 7; }

__declspec(noinline) int k4(int a) { return 0; }
int c4(int a) { int r = k4(a); if (r) return r; return 7; }

// A plain callee with a REAL body — still far under the 128-instruction
// candidacy ceiling WB_INLINE_FINDINGS §2.1 read off `0x10b5fb5f`.
int k5(int a) { return a * 3 + 1; }
int c5(int a) { int r = k5(a); if (r) return r; return 7; }

// The callee defined BELOW its caller, plain — declaration order was the
// control that separated nothing in `w-ifn`'s elision grid, asked again here
// for the expansion question.
int k6(int a);
int c6(int a) { int r = k6(a); if (r) return r; return 7; }
int k6(int a) { return 0; }

// A `static` callee — internal linkage, the case where c2 knows no other TU
// can call it.
static int k7(int a) { return 0; }
int c7(int a) { int r = k7(a); if (r) return r; return 7; }

// TWO calls to one plain callee, both results used — the budget arm of §2.2
// gets two sites instead of one.
int k8(int a) { return 0; }
int c8(int a) { int r = k8(a); int s = k8(r); if (s) return s; return 7; }

// w-ifn — the ELIDED CALL grid.  `mmioClose` calls
// `mmioSetBuffer(hmmio,0,0,0)`, the callee is `__declspec(noinline)` with a
// non-empty body, and the obj carries NO BRANCH for it (w-blockir's mechanism
// 11, board #2302).  The IL DOES carry the call — this lane read the `.ex`
// segment and found the `26 <tok> BD` at line 60 — so the deletion is c2's,
// not the front end's.  Each cell varies ONE property of that call site.
//
// Read the disassembly per function: the question is only ever "is there a
// `bl` to the callee in the caller's body".

// ---- E1: the mmio shape exactly — noinline callee, constant body, result unused
__declspec(noinline) int e1k(int a, int b, int c, int d) { return 0; }
void e1(int a) { e1k(a, 0, 0, 0); }

// ---- E2: the callee has a SIDE EFFECT (a store to a TU-static)
int e2g;
__declspec(noinline) int e2k(int a) { e2g = a; return 0; }
void e2(int a) { e2k(a); }

// ---- E3: the callee is EXTERNAL — declared, never defined here
int e3k(int a);
void e3(int a) { e3k(a); }

// ---- E4: the result IS used
__declspec(noinline) int e4k(int a) { return 0; }
int e4(int a) { return e4k(a); }

// ---- E5: the callee is NOT noinline, constant body, result unused
int e5k(int a) { return 0; }
void e5(int a) { e5k(a); }

// ---- E6: the callee's body READS its argument (no side effect, not constant)
__declspec(noinline) int e6k(int a) { return a + 1; }
void e6(int a) { e6k(a); }

// ---- E7: the callee itself CALLS something external
int e7x(int a);
__declspec(noinline) int e7k(int a) { return e7x(a); }
void e7(int a) { e7k(a); }

// ---- E8: the callee is defined AFTER the caller in the TU
__declspec(noinline) int e8k(int a);
void e8(int a) { e8k(a); }
__declspec(noinline) int e8k(int a) { return 0; }

// ---- E9: void callee with an empty body, result trivially unused, NON-tail
//         position (there is a statement after it) — separates from `elide.rs`
//         mechanism E, which is a TAIL call to an empty body.
__declspec(noinline) void e9k(int a) {}
int e9g;
int e9(int a) { e9k(a); return 7; }

// ---- E10: the caller is FRAMED for another reason and the elided call is in
//          the middle of a block — mmioClose's exact position.
int e10x(int a);
__declspec(noinline) int e10k(int a) { return 0; }
int e10(int a) { int r = e10x(a); if (r) return r; e10k(a); return 0; }

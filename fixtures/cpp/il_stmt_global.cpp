// **Negative** — a store to a GLOBAL must keep refusing, while the identical-looking
// store to a local is accepted.
//
// This is the sharpest case in the assignment class, because the IL gives you
// nothing to distinguish them. Both are the same five-part production and differ in
// one token:
//
//   int x = a + 1;   ->  53 26 e7 09  b9 e4 09 86 41 74 33 86 41 74 01 02  32 86 41 74  4b
//   gv   = a + 1;    ->  53 26 e3 09  b9 e8 09 86 41 74 33 86 41 74 01 02  32 86 41 74  4b
//                            ^^^^^^^ the only difference
//
// A local store is a register copy that c2 coalesces away entirely; a global store
// is a real memory write with a relocation. Treating the second as the first is a
// silent mis-emit, so the decision cannot be skipped or guessed.
//
// The distinguishing information is not in `.ex` at all — it is that a global
// carries a **name in `.gl`** and a local does not. That is why the body parser is
// given the `.gl` symbol index: without it, this fixture and `il_stmt_local_decl.cpp`
// are indistinguishable inputs.
//
// `w_gret` returns the parameter rather than the global, so the store is the only
// thing keeping it out of class — it separates "refuses because it *reads* a global"
// from "refuses because it *writes* one".

int gv;

int w_local(int a) { int x = a + 1; return x; }
int w_global(int a) { gv = a + 1; return gv; }
int w_gret(int a) { gv = a; return a; }

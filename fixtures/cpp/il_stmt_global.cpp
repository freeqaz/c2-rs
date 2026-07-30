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
// The distinguishing information is not in `.ex` at all. It is also NOT reliably in
// `.gl`: the first version of this gate refused a destination that `gl_symbol_index`
// named, which looked sound and was not — a file-scope `static int sv` appears there
// as `$sv`, whose leading `$` that index does not accept as an identifier, so the
// token looked local and the store was silently dropped (`il_stmt_static.cpp`).
//
// The destination is therefore established **positively**: it must be a formal, read
// from the `2D` list. Absence from a symbol table proves nothing — it only says the
// table did not happen to name it. Locals are out of class as a consequence, since
// `.ex` has no positive local signal at all.
//
// `w_gret` returns the parameter rather than the global, so the store is the only
// thing keeping it out of class — it separates "refuses because it *reads* a global"
// from "refuses because it *writes* one".

int gv;

int w_local(int a) { int x = a + 1; return x; }
int w_global(int a) { gv = a + 1; return gv; }
int w_gret(int a) { gv = a; return a; }

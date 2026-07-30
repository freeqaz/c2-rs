// **Negative** — a store to any memory object must refuse, including a file-scope
// `static`. This fixture exists because the first version of the gate let statics
// through and silently dropped the store.
//
// The assignment class accepts a body by resolving its statement list to whichever
// expression reaches the `return`, which is right for a local — c2 register-allocates
// locals and coalesces the copies away. It is wrong for anything with an address: a
// global or a `static` store is a real write with a relocation, and dropping it emits
// a function that computes the right return value and forgets the side effect.
//
// The IL cannot tell them apart. Every one of these is the same production, differing
// only in the destination token:
//
//   int x = a;   26 <tok>  b9 <a> 86 41 74  32 86 41 74  4b
//   gv   = a;    26 <tok>  b9 <a> 86 41 74  32 86 41 74  4b
//   sv   = a;    26 <tok>  b9 <a> 86 41 74  32 86 41 74  4b
//
// The gate first tried "refuse if the destination is in the `.gl` symbol index",
// which looked sound and is not. A `static int sv` appears in `.gl` as **`$sv`**, and
// `gl_symbol_index` accepts only identifier-shaped runs, so the leading `$` made the
// token look local. `sv = a; return a;` then compiled to a bare register move and
// `Port=Mismatch`.
//
// So the destination is now established **positively**: it must be a formal, read from
// the `2D` list. Absence from a symbol table proves nothing — it only says the table
// did not happen to name it. That is the general lesson; the specific `$` is incidental.
//
// Locals are out of class as a result. There is no positive local signal in `.ex` —
// the same `26 <tok>` push serves parameter, local and global — so admitting them
// needs a local-symbol production first. The coverage that costs is measured at ~0 on
// the real workload, which is not a reason to keep a mis-emit.
//
// `p_formal` is the separating case: assignment to a *parameter* is still accepted and
// byte-exact, so the gate cannot be "refuse all assignment".

int gv;
static int sv;

int w_global(int a) { gv = a; return a; }
int w_static(int a) { sv = a; return a; }
int w_local(int a) { int x = a; return x; }
int r_static(int a) { sv = a + 1; return sv; }

// **Negative** — a call whose callee is DEFINED in the same TU must refuse,
// because c2 inlines it and the port has no inliner.
//
// The IL says tail call. `w_use`'s body is the same `int-tail-call` shape as every
// external tail call the port emits byte-exact, and its callee token resolves
// cleanly through the `.gl` symbol index. Nothing local to the body distinguishes
// this from `il_call_args1.cpp`.
//
// What c2 actually emits is a `.text` of *two* copies of `addi r3,r3,1 ; blr` and
// **zero relocations** — `w_add` cloned into `w_use`, with both symbols pointing
// into the same section (`?w_add` at 0, `?w_use` at 8). The port emitted
// `b ?w_add` against an undefined external and mismatched at file offset 8, the
// symbol-table pointer, because its symbol table carried an extra external the
// reference does not have.
//
// So c2 — the *backend* — is doing the inlining here, which belongs on the same
// list as constant folding, strength reduction and reassociation: optimizations
// one would expect in the front end that are in fact c2's. See `docs/GAPS.md`.
//
// Refused wholesale rather than by callee size or shape. What makes c2 decide to
// inline, and what it does to the symbol table and `.pdata` when it does, is
// uncharacterized; a size threshold guessed from one capture is exactly the kind
// of acceptance region that ends up wider than the region anyone enumerated.
//
// Calls to true externals are unaffected — those are the tail calls the class was
// built on, and the forward declaration on line 1 is what makes this look like
// one right up to the point of emission.

int w_add(int a);
int w_use(int a) { return w_add(a); }
int w_add(int a) { return a + 1; }

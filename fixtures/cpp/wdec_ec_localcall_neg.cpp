// w-decouple `_neg` — the RESIDUE cell: the gate's inline-fence EXEMPTION keeps
// the narrow walk, so a newly-binding TU with an intra-TU call edge is refused
// wholesale rather than handed to the size test.
//
// This lane widened the BINDING and deliberately did not widen
// `gl::plain_external_defined_names`, which is the W-FENCE2 exemption:
// `defined` is now the full name list and `exempt` is empty, so
// `callee_defined_here_unmodelled` fires and `IlBundle::functions` returns
// `None` at `locally-defined-callee`. Widening an exemption is the LICENSING
// direction and this project only tolerates errors in the refusing one.
//
// It must read `Port=NotImplemented`, never `Match` and never `Mismatch`, and
// its gate cause must be **`locally-defined-callee`** — the clause it grades.
//
// `src/xdk/nuispeech/mmio.cpp` is the live instance and no published price for
// that TU names it: `mmioClose` calls `mmioFlush`, which mmio defines
// (`work/w-decouple/ref/mmio.dump`, `.text #14`, `REL24 -> [33] mmioFlush`).
// The gate cannot see it on mmio yet — `mmioClose` is out of class, so it is
// not among the functions the fence is asked about, and `diag`'s own comment
// says a gate that cannot fire on a partial function list is not evidence it
// would not fire on the whole one.
//
// # This cell was OVER-FENCED and the repair was MERGING
//
// It was first written with two-byte names (`cb` / `cf`) and it graded
// **nothing**: it read `shape-token-unresolved`, a refusal three gates earlier,
// and the exemption question was never asked. The cause is a THIRD name-length
// rule in the reader, independent of `INLINE_NAME_MAX` (8) and of
// `looks_mangled`: `gl::is_indexable_name` requires `b.len() >= 3`, so a
// two-byte callee never enters `gl_symbol_index` and `Bindings::resolve` returns
// `None` for its call token. Three bytes is the floor and this cell now sits one
// byte above it.
//
// Its sibling `wdec_ecshort_eight.cpp` deliberately keeps a ONE-byte name, which
// binds and matches — because `Bindings::per_record` reads symbol RUNS and never
// the index, so the two readers have different floors and only the one asked
// about a CALLEE has this one.

extern "C" {
int cbx(int a) { return a + 1; }
int cfx(int a) { return cbx(a + 1); }
}

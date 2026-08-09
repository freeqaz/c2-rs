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
// The cell is here so the residue is a graded row rather than a sentence. It
// must read `Port=NotImplemented`, never `Match` and never `Mismatch`.
//
// `src/xdk/nuispeech/mmio.cpp` is the live instance and no published price for
// that TU names it: `mmioClose` calls `mmioFlush`, which mmio defines
// (`work/w-decouple/ref/mmio.dump`, `.text #14`, `REL24 -> [33] mmioFlush`).
// The gate cannot see it yet — `mmioClose` is out of class, so it is not among
// the functions the fence is asked about, and `diag`'s own comment says a gate
// that cannot fire on a partial function list is not evidence it would not fire
// on the whole one.

extern "C" {
int cb(int a) { return a + 1; }
int cf(int a) { return cb(a); }
}

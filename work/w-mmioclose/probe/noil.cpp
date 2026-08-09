// w-mmioclose — IS `__declspec(noinline)` VISIBLE IN THE IL?
//
// The two callees are byte-identical as C++ except for the attribute, and the
// two callers are byte-identical except for which one they call.  If the port
// can convert `mmio.cpp` at all, the fact that separates `mmioFlush` (call
// KEPT) from probe cell `k1` (call INLINED) has to be readable from the `.gl` /
// `.sy` / `.ex` this captures.  If the two functions' records are identical,
// no accept-side narrowing of either inline fence is sound and the TU is
// declined for a MEASURED reason rather than an architectural one.
//
// The bodies are `return 0` so the two callees' own `.ex` segments are
// identical too — any difference the diff finds is the attribute and nothing
// else.

__declspec(noinline) int nk(int a) { return 0; }
int pk(int a) { return 0; }

int nc(int a) { int r = nk(a); if (r) return r; return 7; }
int pc(int a) { int r = pk(a); if (r) return r; return 7; }

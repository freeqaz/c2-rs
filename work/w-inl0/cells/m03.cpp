// GRID-M m03 — A REAL `memset`, not a temporary. `docs/IL_CAST_CONVERT.md` §1.3
// records that c2 lowers this one to `b <memset>` with a REL24 — the opposite
// of nothing — so the reader must not recognize it.
extern "C" void* memset(void*, int, unsigned int);
#pragma intrinsic(memset)

void clear(int* p, int n) { memset(p, 0, (unsigned int)(n * 4)); }

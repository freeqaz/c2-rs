// M4 = M3 with the callee named `memcpy` (the real one; #410 says c2 emits an
// ordinary REL24 `bl` here, not an inlined intrinsic).
extern "C" void *memcpy(void *, const void *, unsigned);
unsigned f(void *p, void *q, unsigned r) { if (p == 0) return 5; if (q == 0) return 11; memcpy(q, p, 72); return 0; }

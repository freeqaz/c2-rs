// w-ifn — the `memcpy` EXPANSION boundary, at the workload's own flags.
//
// Board #1925 records `expr-intrinsic-memcpy` as a MEASURED NON-RULE: four
// separately frozen thresholds all miss.  `mmio.cpp` needs only one fact — that
// a 72-byte copy at `/O1 /Oi` becomes a `bl memcpy` and not a run of stores —
// but a reader clause has to say WHERE its accepted window ends, so this grid
// brackets the boundary rather than pinning one size.
//
// Each cell: does the body contain a `bl memcpy` (CALL) or a run of
// loads/stores (EXPANDED)?

extern "C" void *memcpy(void *, const void *, unsigned int);

#define CELL(n) extern "C" void m##n(void *d, const void *s) { memcpy(d, s, n); }

CELL(1)
CELL(2)
CELL(3)
CELL(4)
CELL(5)
CELL(6)
CELL(7)
CELL(8)
CELL(12)
CELL(16)
CELL(20)
CELL(24)
CELL(28)
CELL(32)
CELL(36)
CELL(40)
CELL(48)
CELL(64)
CELL(68)
CELL(72)
CELL(76)
CELL(80)
CELL(96)
CELL(128)
CELL(256)

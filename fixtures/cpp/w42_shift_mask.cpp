// **W42 — `(formal >> k) & m` inside a conditional tail arm**, folded to one
// `rlwinm`. The first *computed* argument the port emits, and the second half
// of `src/xdk/nuispeech/xboxmem.cpp` (`w8_cond_tail.cpp` is `?MemFree`,
// `w8_cond_tail_value.cpp` is `?MemSize`; this is `?MemAlloc`).
//
//     ?MemAlloc@NUISPEECH@@YAPAXPAXKK@Z          .text COMDAT, 0x24 B, nrel 2
//       mr     r11,r4            entry: size parked, both arms want it, at
//       mr     r4,r5                    DIFFERENT registers
//       cmplwi cr6,r3,0          entry: attrs HOISTED to r4, because both arms
//       bne    cr6,+12                  want it THERE — raw in the then-arm,
//       mr     r3,r11                   folded in the else-arm
//       b      XMemAlloc
//       mr     r5,r11
//       rlwinm r4,r4,5,28,28     else: the fold, IN PLACE
//       b      RtlAllocateHeap
//
// ## The two rules, and what measures each
//
// **The fold** is `c2_il::shift_mask_rlwinm`, and it is not fitted on
// `?MemAlloc`: **70 cells** — `k ∈ {1,4,8,16,24,27,31}` × `m ∈ {1,2,3,8,12,15,
// 255,0x10,0x10000,0xFFFFFFF0}`, the whole cross product — were compiled by real
// `c2` at the workload's own `/O1 /Oi /EHsc /GR` profile and every one agrees
// with the three-line rule (`work/w-tu1/p/grid_sm.cpp`, replayable through
// `work/w-tu1/p/gradeo1.sh`). **61 of them are this file's `g*` functions and
// the whole obj is graded byte-exact**; the other 9 are the collapse cells in
// `w42_shift_mask_neg.cpp`.
//
// **The hoist** is `plan_cond_pair`'s rule 1, already in the tree and already
// documented against `?MemAlloc` — a formal both arms want in the SAME register
// has its move hoisted to the entry block. What is new is that a `ShiftMask`
// slot COUNTS as a use of its source formal, which is what makes rule 1 fire
// here. The neighbouring cell where it must NOT fire is `q2` in
// `w42_shift_mask_neg.cpp`.
//
// The disassembler prints `rlwinm rA,rS,32-k,k,31` as `srwi rA,rS,k`. Same
// word; six of the cells below land there and are not exceptions.
//
// ## The spelling this class does NOT take, and it is not the same tree
//
// Written **inline** — `h3(heap, (attrs >> 0x1b) & 8, size)` — c1xx emits the
// arithmetic *inside the argument operand stream* rather than as a preceding
// statement, and the emitted obj is byte-identical. The port refuses it anyway,
// because `IlOp` has no `Shr`/`BitAnd` and `parse_expr` therefore cannot even
// decode that stream; widening `IlOp` would widen every shape that consumes an
// operand list, on one witness. It lives in `w42_shift_mask_neg.cpp` §4 with
// its measured bytes, so the boundary is recorded rather than implied.

void *g2(unsigned long, unsigned long);
void *h3(void *, unsigned long, unsigned long);

// `?MemAlloc` itself, reduced to its externals — the local spelled exactly as
// the workload spells it.
void *memalloc(void *heap, unsigned long size, unsigned long attrs) {
    if (heap == 0) {
        return g2(size, attrs);
    }
    unsigned long flags = (attrs >> 0x1b) & 8;
    return h3(heap, flags, size);
}

#define CELL(n, k, m)                                                    \
    void *g##n(void *hp, unsigned long sz, unsigned long at) {           \
        if (hp == 0) {                                                   \
            return g2(sz, at);                                           \
        }                                                                \
        unsigned long f = (at >> k) & m##u;                              \
        return h3(hp, f, sz);                                            \
    }

// k = 1 — the widest shift, where the mask keeps 31 bits.
CELL(00, 1, 0x1) CELL(01, 1, 0x2) CELL(02, 1, 0x3) CELL(03, 1, 0x8)
CELL(04, 1, 0xc) CELL(05, 1, 0xf) CELL(06, 1, 0xff) CELL(07, 1, 0x10)
CELL(08, 1, 0x10000) CELL(09, 1, 0xfffffff0)
// k = 4
CELL(10, 4, 0x1) CELL(11, 4, 0x2) CELL(12, 4, 0x3) CELL(13, 4, 0x8)
CELL(14, 4, 0xc) CELL(15, 4, 0xf) CELL(16, 4, 0xff) CELL(17, 4, 0x10)
CELL(18, 4, 0x10000) CELL(19, 4, 0xfffffff0)
// k = 8
CELL(20, 8, 0x1) CELL(21, 8, 0x2) CELL(22, 8, 0x3) CELL(23, 8, 0x8)
CELL(24, 8, 0xc) CELL(25, 8, 0xf) CELL(26, 8, 0xff) CELL(27, 8, 0x10)
CELL(28, 8, 0x10000) CELL(29, 8, 0xfffffff0)
// k = 16 — `m = 0x10000` is EXACTLY at the boundary and collapses; it is in the
// negative file, and its two neighbours here are the reason that matters.
CELL(30, 16, 0x1) CELL(31, 16, 0x2) CELL(32, 16, 0x3) CELL(33, 16, 0x8)
CELL(34, 16, 0xc) CELL(35, 16, 0xf) CELL(36, 16, 0xff) CELL(37, 16, 0x10)
CELL(38, 16, 0xfffffff0)
// k = 24
CELL(40, 24, 0x1) CELL(41, 24, 0x2) CELL(42, 24, 0x3) CELL(43, 24, 0x8)
CELL(44, 24, 0xc) CELL(45, 24, 0xf) CELL(46, 24, 0xff) CELL(47, 24, 0x10)
CELL(49, 24, 0xfffffff0)
// k = 27 — the workload's own shift.
CELL(50, 27, 0x1) CELL(51, 27, 0x2) CELL(52, 27, 0x3) CELL(53, 27, 0x8)
CELL(54, 27, 0xc) CELL(55, 27, 0xf) CELL(56, 27, 0xff) CELL(57, 27, 0x10)
CELL(59, 27, 0xfffffff0)
// k = 31 — one bit survives, and only for the masks that name it. The other
// four `m` values at this `k` collapse and are in the negative file.
CELL(60, 31, 0x1) CELL(62, 31, 0x3) CELL(65, 31, 0xf) CELL(66, 31, 0xff)

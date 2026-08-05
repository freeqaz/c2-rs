// **w-hash** — the POINTER-WALK ACCUMULATE LOOP, the port's first body class
// with a **back edge**, and `src/system/math/Sort.cpp`'s whole content.
//
// This is `?HashString@@YAHPBDH@Z` transcribed from the workload, with the
// multiplier and the accumulator's initial value at values the class's own
// cross product grades (`work/w-hash/crossgrade.py`: 49 of 49 `(K0, K)` cells
// `match` against real `c2.dll` under wibo, beside 30 must-refuse cells at
// `vocab-gap` and 0 mismatches).
//
// **It is in class at `/O1` only, and that is the fixture's second job.** `/Ox`
// and `/O2` compile this same source to **twenty-one** words, not twenty: the
// `* 127` strength-reduces to `rlwinm` + `subf` where `/O1` emits one `mulli`,
// the zero-divisor `twi` hoists to the third slot, and the loop closes on an
// explicit `cmpli` instead of the `mr.` record form — so the branch reads a
// different condition-register field. The `/Ox`, `/O2` and `/Od` lanes must
// therefore read `NotImplemented` here, never `Match`, and the gate grades that
// across all eighteen.
//
// The twenty words, from `work/w-hash/Sort.obj` at the workload's own flags:
//
//     lbz r11,0(r3) · mr r9,r3 · li r10,0 · cmplwi cr0,r11,0 · bt 2,+56
//     mulli r8,r10,127 · lbzu r10,1(r9) · add r8,r8,r11 · mr. r11,r10
//     rotlwi r10,r8,1 · divw r7,r8,r4 · addi r10,r10,-1 · mullw r7,r7,r4
//     andc r6,r4,r10 · twi 6,r4,0 · subf r10,r7,r8 · twi 5,r6,-1
//     bf 2,-48 · mr r3,r10 · blr
int HashString(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}

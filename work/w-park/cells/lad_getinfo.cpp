// CONSTRUCT LADDER — from `w275_entry_park.cpp`'s shipped `pk2g2`, which is
// 7/7 in class at `/O1`, to `mmio.cpp`'s `mmioGetInfo`, one variation per rung.
// Board #401's method. Measurement only.
extern "C" void *memcpy(void *, const void *, unsigned int);
void g2(void *, void *);
void g3(void *, void *, void *);
void g3n(void *, void *, unsigned int);

// L0 — the shipped shape, verbatim. Control: must be in class.
int L0(void *a0, void *a1) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g2(a1, a0);
    return 0;
}

// L1 — L0 plus a THIRD, UNUSED formal. mmioGetInfo has `UINT fuInfo`.
int L1(void *a0, void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g2(a1, a0);
    return 0;
}

// L2 — L1 with the return type widened to `unsigned long` (MMRESULT).
// Board #1788: this changes the TYPE and can leave the bytes identical.
unsigned long L2(void *a0, void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g2(a1, a0);
    return 0;
}

// L3 — L2 with a THREE-argument call whose third argument is a LITERAL.
// This is the `li r5,72` slot: `callseq-multiarg-lit`.
unsigned long L3(void *a0, void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g3n(a1, a0, 0x48);
    return 0;
}

// L4 — L3 with the callee named `memcpy`, which `/Oi` makes an INTRINSIC.
// This is `expr-intrinsic-memcpy`.
unsigned long L4(void *a0, void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    memcpy(a1, a0, 0x48);
    return 0;
}

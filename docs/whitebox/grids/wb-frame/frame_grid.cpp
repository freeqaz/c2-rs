// wb-frame obj-check grid — one TU, one COMDAT per cell (/O1 implies /Gy).
//
// The judge for Lane WB-B's frame-opening predicate. Every cell's per-rival
// prediction is frozen in ../../WB_FRAME_FINDINGS.md BEFORE this file was first
// compiled; the commit that adds this file also adds that table.
//
// Read the frame decision out of the emitted `.text`: a frame is open iff the
// prologue contains `stwu r1,-F(r1)` (0x9421xxxx) or `stwux r1,r1,r12`
// (0x7C21616E). The `.pdata` prolog-length byte is recorded as an independent
// second read of the same decision.
//
// Externals are undefined on purpose so a call stays a call.

extern volatile int wbf_sink;
extern int *volatile wbf_pesc;
extern int wbf_g(int);
extern void wbf_h(int);

// ---- C1: leaf, no locals, no calls -----------------------------------------
int wbf_c1(int a) { return a + 1; }

// ---- C2: leaf, 256 bytes of runtime-indexed local array, no calls ----------
int wbf_c2(int i) {
    int b[64];
    for (int k = 0; k < 64; k++) b[k] = k + i;
    wbf_sink = b[i & 63];
    return b[0];
}

// ---- C3: leaf, FPR arithmetic, no locals, no calls -------------------------
double wbf_c3(double a, double b) { return a * b + 1.0; }

// ---- C4: leaf loop with 16 simultaneous accumulators (forces callee-saved) --
int wbf_c4(const int *p, int n) {
    int a0 = 0, a1 = 1, a2 = 2, a3 = 3, a4 = 4, a5 = 5, a6 = 6, a7 = 7;
    int a8 = 8, a9 = 9, aa = 10, ab = 11, ac = 12, ad = 13, ae = 14, af = 15;
    for (int k = 0; k < n; k++) {
        a0 += p[k];
        a1 ^= a0;
        a2 += a1;
        a3 ^= a2;
        a4 += a3;
        a5 ^= a4;
        a6 += a5;
        a7 ^= a6;
        a8 += a7;
        a9 ^= a8;
        aa += a9;
        ab ^= aa;
        ac += ab;
        ad ^= ac;
        ae += ad;
        af ^= ae;
    }
    return a0 + a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9 + aa + ab + ac + ad + ae + af;
}

// ---- C5: non-leaf, one call, result consumed (not a tail call) -------------
int wbf_c5(int a) { return wbf_g(a) + 1; }

// ---- C6: pure tail call, value returned unchanged --------------------------
int wbf_c6(int a) { return wbf_g(a); }

// ---- C6b: tail call with a transformed argument ----------------------------
int wbf_c6b(int a) { return wbf_g(a * 2); }

// ---- C6c: void tail call ---------------------------------------------------
void wbf_c6c(int a) { wbf_h(a); }

// ---- C7: no call in the source; the 64-bit divide is a helper call ---------
long long wbf_c7(long long a, long long b) { return a / b; }

// ---- C9: leaf, one 4-byte local whose address escapes, no calls ------------
int wbf_c9(int a) {
    int x = a + 1;
    wbf_pesc = &x;
    return x;
}

// ---- C9b: leaf, 16 bytes of local array whose address escapes, no calls ----
int wbf_c9b(int a) {
    int b[4] = {a, a + 1, a + 2, a + 3};
    wbf_pesc = b;
    return b[1];
}

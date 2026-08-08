// wb-frame AUXILIARY probe — declared separately from the frozen frame grid so
// it cannot contaminate that grid's scoring.
//
// It is about the mechanism this lane found the ?supershuffle anchor actually
// turns on, which is NOT the frame: c2 keeps the incoming pointer live in the
// volatile r3 across in-TU calls whose callees provably do not write r3, so it
// saves NO callee-saved GPR. The port instead parks the pointer in r31, which
// costs it the `std r31,-16(r1)` / `ld r31,-16(r1)` pair and five `mr r3,r31`.
//
// A: callees defined in this TU and demonstrably r3-preserving.
// B: the same shape with the callees UNDEFINED (extern only) — c2 cannot know
//    anything about them, so the ABI applies and r3 must be re-materialised.
// The contrast between A and B is the whole probe.

// ---- arm A: in-TU callees --------------------------------------------------
static void wbfa_leaf1(char *c) { c[0] = (char)(c[1] + 1); }
static void wbfa_leaf2(char *c) { c[2] = (char)(c[3] ^ 2); }
static void wbfa_leaf3(char *c) { c[4] = (char)(c[5] - 3); }

void wbfa_intu(char *c) {
    wbfa_leaf1(c);
    wbfa_leaf2(c);
    wbfa_leaf3(c);
}

// ---- arm B: extern callees -------------------------------------------------
extern void wbfb_ext1(char *c);
extern void wbfb_ext2(char *c);
extern void wbfb_ext3(char *c);

void wbfb_extern(char *c) {
    wbfb_ext1(c);
    wbfb_ext2(c);
    wbfb_ext3(c);
}

// w-ifn — THE PARK, and the interprocedural clobber question it raises.
//
// `mmioClose` parks `fuClose` (formal 2, arriving in r4) into **r5**, a
// VOLATILE register, at `+0x14` — and then calls `mmioFlush` at `+0x30` and
// reads r5 at the `bctrl` at `+0x50`.  That is only correct if c2 knows
// `mmioFlush` does not clobber r5, which it can know because `mmioFlush` is
// defined in the same TU (`li r3,0 ; blr`).
//
// If that is what is happening, replacing the same-TU callee with an EXTERNAL
// one must move the park to a callee-saved register.  Each cell below is the
// same body with one property of the first callee varied.

typedef unsigned int uint;

extern "C" void FreeHandleP(void *);

struct Info {
    unsigned a, b;
    uint (*proc)(void *, uint, uint, uint);
};

// ---- P1: mmioClose exactly — the first callee is same-TU and trivial.
extern "C" __declspec(noinline) uint p1k(void *h, uint f) { return 0; }
extern "C" uint p1(void *h, uint c) {
    if (h == 0) return 5;
    uint r = p1k(h, 0);
    if (r != 0) return r;
    Info *i = (Info *)h;
    uint s = i->proc(i, 4, c, 0);
    if (s != 0) return s;
    FreeHandleP(h);
    return 0;
}

// ---- P2: the first callee is EXTERNAL.  If the park is an interprocedural
//          clobber fact, `c` can no longer live in r5 across it.
extern "C" uint p2k(void *h, uint f);
extern "C" uint p2(void *h, uint c) {
    if (h == 0) return 5;
    uint r = p2k(h, 0);
    if (r != 0) return r;
    Info *i = (Info *)h;
    uint s = i->proc(i, 4, c, 0);
    if (s != 0) return s;
    FreeHandleP(h);
    return 0;
}

// ---- P3: the same-TU callee CLOBBERS r5 itself (it takes three arguments and
//          does arithmetic on them).  Does the park move?
extern "C" __declspec(noinline) uint p3k(void *h, uint f) { return f + 1; }
extern "C" uint p3(void *h, uint c) {
    if (h == 0) return 5;
    uint r = p3k(h, 0);
    if (r != 0) return r;
    Info *i = (Info *)h;
    uint s = i->proc(i, 4, c, 0);
    if (s != 0) return s;
    FreeHandleP(h);
    return 0;
}

// ---- P4: the same-TU callee is itself a caller of an external, so every
//          volatile is dead across it.
extern "C" uint p4x(uint);
extern "C" __declspec(noinline) uint p4k(void *h, uint f) { return p4x(f); }
extern "C" uint p4(void *h, uint c) {
    if (h == 0) return 5;
    uint r = p4k(h, 0);
    if (r != 0) return r;
    Info *i = (Info *)h;
    uint s = i->proc(i, 4, c, 0);
    if (s != 0) return s;
    FreeHandleP(h);
    return 0;
}

// ---- P5: mmioGetInfo's two-register park, isolated: two formals swap so the
//          call's operands are pre-positioned.
extern "C" void *memcpyP(void *, const void *, unsigned int);
extern "C" uint p5(void *h, void *p, uint f) {
    if (h == 0) return 5;
    if (p == 0) return 11;
    memcpyP(p, h, 0x48);
    return 0;
}

// live_grid.cpp — wb-live's obj grid for the liveness / interference reading.
//
// FROZEN BEFORE THE FIRST cl.exe OF THIS LANE. Predictions are in
// docs/whitebox/WB_LIVE_PREREG_R2.md, committed in the same commit as this file.
//
// Compile with the real cl.exe 16.00.11886.00 under wibo at the workload mode
//   /nologo /c /GR /O1 /Oi /EHsc
// and again at /nologo /O1 /GS- /c (wb-regalloc §7.7's second mode).
//
// Every cell is outside every shipped port class: none is a straight-line int
// add-chain, a tail call, a single framed non-leaf call, or one of the
// transcribed shapes (if_call_join, guard_chain_shared_tail, alloc_init_or_fail,
// osf_handle_guard, xlrc_create_guard, ptr_walk_loop).
//
// The axis this grid varies is LIVE-RANGE OVERLAP, which is exactly the axis
// wb-regalloc's grid held constant: its G1-G4 made N values live *at once* and
// found r11,r10,r9,r8. This grid makes N values live *in sequence* and asks
// whether the answer is still r11,r10,r9,r8 (the incumbent) or r11 every time
// (a liveness model).

extern int wbl_a, wbl_b, wbl_c, wbl_d, wbl_e, wbl_f, wbl_g, wbl_h;
extern int wbl_o0, wbl_o1, wbl_o2, wbl_o3, wbl_o4, wbl_o5, wbl_o6, wbl_o7;

extern "C" void wbl_void(int);
extern "C" int  wbl_ext(int);

// ---------------------------------------------------------------------------
// V-series — values live in SEQUENCE. The reuse axis.
// ---------------------------------------------------------------------------

// V1: three statements, each producing a temp that is dead at its own store.
extern "C" void wbl_v1(void)
{
    wbl_o0 = wbl_a + 1;
    wbl_o1 = wbl_b + 2;
    wbl_o2 = wbl_c + 3;
}

// V3: eight of them. Under the incumbent (one register per temp, descending
// from r11, never reused) this body cannot fit in the nine volatiles and must
// take callee-saved registers and open a frame.
extern "C" void wbl_v3(void)
{
    wbl_o0 = wbl_a + 1;
    wbl_o1 = wbl_b + 2;
    wbl_o2 = wbl_c + 3;
    wbl_o3 = wbl_d + 4;
    wbl_o4 = wbl_e + 5;
    wbl_o5 = wbl_f + 6;
    wbl_o6 = wbl_g + 7;
    wbl_o7 = wbl_h + 8;
}

// P2: the POSITIVE CONTROL — three values live AT ONCE. wb-regalloc's G3
// reproduced. If this does not show three distinct registers, the instrument
// is broken and V1/V3 mean nothing.
extern "C" int wbl_p2(void)
{
    return wbl_a * wbl_b + wbl_c;
}

// ---------------------------------------------------------------------------
// X-series — values live ACROSS A CALL. The callee-saved question.
// ---------------------------------------------------------------------------

// X1: one formal live across a call.
extern "C" int wbl_x1(int a)
{
    wbl_void(0);
    return a + 1;
}

// X2: two formals live across a call.
extern "C" int wbl_x2(int a, int b)
{
    wbl_void(0);
    return a + b;
}

// X5: three formals live across a call.
extern "C" int wbl_x5(int a, int b, int c)
{
    wbl_void(0);
    return a + b + c;
}

// X3: the NEGATIVE control for X1. A non-leaf body in which NOTHING is live
// across the call: the temp dies as the call's argument, and the value the
// result is built from is loaded afterwards.
extern "C" int wbl_x3(int a)
{
    wbl_void(a + 1);
    return wbl_a + 2;
}

// X4: docs/CFG_SHAPE.md section 6.2 item F's first measured case, reconstructed.
// `a` is live out of the entry block, both successors clobber the volatiles,
// and both join at the return.
extern "C" int wbl_x4(int a, int c)
{
    if (c) {
        wbl_void(1);
    } else {
        wbl_void(2);
    }
    return a;
}

// X6: a value live across a call in ONE arm only, and dead before the call in
// the other. Tests whether the interference is per-live-range or per-function.
extern "C" int wbl_x6(int a, int c)
{
    if (c) {
        wbl_void(0);
        return a + 1;
    }
    wbl_void(a + 2);
    return 0;
}

// ---------------------------------------------------------------------------
// R-series — reuse ACROSS a call boundary, no value crossing it.
// ---------------------------------------------------------------------------

// R1: two temps, one before the call and one after, neither crossing it.
extern "C" void wbl_r1(void)
{
    wbl_void(wbl_a + 1);
    wbl_o0 = wbl_b + 2;
}

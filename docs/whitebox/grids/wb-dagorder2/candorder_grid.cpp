// candorder_grid.cpp — lane w-dagorder (WB-DAGORDER2).
//
// The question: in what order does the register allocator 0x10b31c9a colour
// the candidates of a class? WB_LIVE_FINDINGS.md section 10 records exactly one
// datum -- wbl_x2's `a` took r30 and `b` took r31 -- and records it as
// UNEXPLAINED. Three of the five readings named in the prereg fit that single
// cell, which is why this grid is a SERIES over n and not a cell.
//
// Every cell keeps a set of int values live across a call, which is what forces
// them onto callee-saved colours (WB_LIVE section 6, wbl_x2/wbl_x5). The
// callee-saved run is handed out r31, r30, r29, ... (W-REGALLOC-1's order
// [r11..r3, r31..r14]), so READING WHICH FORMAL GOT r31 READS WHICH CANDIDATE
// WAS COLOURED FIRST. That is the whole instrument.
//
// Compiled against real cl.exe 16.00.11886.00 / c2.dll under wibo, at the
// workload profile /nologo /c /GR /O1 /Oi /EHsc AND at /Ox, because w-section
// measured /Ox disagreeing with /O1 on seven of eight fields of the section
// emitter and a characterization taken at the wrong profile is wrong almost
// everywhere.

extern "C" void cnd_void(int);
extern "C" int  cnd_val(void);

// ---------------------------------------------------------------------------
// A-SERIES -- n formals live across a call, n = 1..8. The base series.
// Summed in declaration order. Reads the order the callee-saved run is handed
// out as a function of n, which is the thing a single cell cannot show.
// ---------------------------------------------------------------------------

extern "C" int cnd_a1(int a)
{ cnd_void(0); return a; }

extern "C" int cnd_a2(int a, int b)
{ cnd_void(0); return a + b; }

extern "C" int cnd_a3(int a, int b, int c)
{ cnd_void(0); return a + b + c; }

extern "C" int cnd_a4(int a, int b, int c, int d)
{ cnd_void(0); return a + b + c + d; }

extern "C" int cnd_a5(int a, int b, int c, int d, int e)
{ cnd_void(0); return a + b + c + d + e; }

extern "C" int cnd_a6(int a, int b, int c, int d, int e, int f)
{ cnd_void(0); return a + b + c + d + e + f; }

extern "C" int cnd_a7(int a, int b, int c, int d, int e, int f, int g)
{ cnd_void(0); return a + b + c + d + e + f + g; }

extern "C" int cnd_a8(int a, int b, int c, int d, int e, int f, int g, int h)
{ cnd_void(0); return a + b + c + d + e + f + g + h; }

// ---------------------------------------------------------------------------
// X-SERIES -- the wbl_x2 reproduction (P1) and the commutative reversal (P3).
//
// CAVEAT REGISTERED BEFORE THE FIRST COMPILE: `a + b` and `b + a` are the same
// value, so a reassociating front end may normalize them to one tuple list, in
// which case this pair is INERT rather than a refutation of H-SCHED. The
// H-SERIES below is the test that does not have that defect.
// ---------------------------------------------------------------------------

extern "C" int cnd_x2(int a, int b)
{ cnd_void(0); return a + b; }            // == wbl_x2, byte for byte

extern "C" int cnd_x2r(int a, int b)
{ cnd_void(0); return b + a; }

extern "C" int cnd_x3(int a, int b, int c)
{ cnd_void(0); return a + b + c; }

extern "C" int cnd_x3r(int a, int b, int c)
{ cnd_void(0); return c + b + a; }

// Non-commutative: the operand order cannot be normalized away, because the
// two cells compute DIFFERENT values. wb-dagorder measured operand chains
// lowering RIGHT-FIRST (dg_sub/dg_sub2); if the candidate order follows the
// lowered order, these two disagree on which formal leads.
extern "C" int cnd_s2(int a, int b)
{ cnd_void(0); return a - b; }

extern "C" int cnd_s2r(int a, int b)
{ cnd_void(0); return b - a; }

// ---------------------------------------------------------------------------
// H-SERIES -- THE DISCRIMINATOR. Formal order, declaration order and the live
// set are all held FIXED; only the DEPENDENCE HEIGHT of each value's producer
// moves. wb-dagorder read the scheduler's priority as
// (height<<13)+(fanout<<8)+(symdest<<10) at 0x10be5df6, so a height swap moves
// the scheduled def order and nothing else.
//
// H-SCHED predicts hN and hNr DISAGREE on the register assignment.
// H-SRC, H-REV, H-ARR and H-USE all predict they AGREE -- none of them can see
// height at all. This is the pair that separates the readings.
// ---------------------------------------------------------------------------

extern "C" int cnd_h2(int a, int b)
{
    int x = a * 3 + 7;        // taller producer
    int y = b;                // trivial producer
    cnd_void(0);
    return x + y;
}

extern "C" int cnd_h2r(int a, int b)
{
    int x = a;                // trivial producer
    int y = b * 3 + 7;        // taller producer  -- the ONLY difference
    cnd_void(0);
    return x + y;
}

extern "C" int cnd_h3(int a, int b, int c)
{
    int x = a * 3 + 7;
    int y = b;
    int z = c;
    cnd_void(0);
    return x + y + z;
}

extern "C" int cnd_h3r(int a, int b, int c)
{
    int x = a;
    int y = b;
    int z = c * 3 + 7;
    cnd_void(0);
    return x + y + z;
}

// ---------------------------------------------------------------------------
// U-SERIES -- the USE COUNT axis. codegen::alloc clause 1 is "use count desc"
// and ORDER.md's black-box rank is (use count desc, first-use asc); wb-dagorder
// rediscovered that same rank as the SCHEDULER's priority second key. If the
// candidate order is use-count driven, these separate; if it is schedule
// driven, they follow the schedule instead.
// ---------------------------------------------------------------------------

extern "C" int cnd_u2(int a, int b)
{ cnd_void(0); return a + b + b + b; }    // b used 3x, a once

extern "C" int cnd_u2r(int a, int b)
{ cnd_void(0); return a + a + a + b; }    // a used 3x, b once

// ---------------------------------------------------------------------------
// C-SERIES -- the CONTROL. Values that are NOT live across the call must not
// take a callee-saved colour at all. If one of these frames, the instrument is
// reading something other than what it claims to read and the batch is void.
// ---------------------------------------------------------------------------

extern "C" int cnd_c0(int a)
{ cnd_void(a + 1); return cnd_val(); }

// regorder_grid.cpp — lane wb-regalloc (WB-D), campaign 2026-08-08.
//
// Grades the register-CHOICE policy and the instruction-ORDER policy read off
// c2.dll's `color.c` in docs/whitebox/WB_REGALLOC_FINDINGS.md.
//
// One COMDAT per cell (/Gy is on in the workload mode), so every cell can be
// read out of one obj by symbol name. Every cell is deliberately OUTSIDE every
// shipped port class (no straight-line int add-chain, no bare tail call, no
// single framed non-leaf call with nothing else going on).
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/wb-regalloc/regorder_grid.cpp \
//               /nologo /c /GR /O1 /Oi /EHsc
// Read:     scripts/gt_dump.py <obj>

extern "C" {

// ---- N1..N4: how many simultaneously-live integer temps does it take to walk
// ---- down the volatile preference list?  Predicted order r11,r10,r9,...,r3.
int wbr_n1(int *p)            { return p[0] + 1; }
int wbr_n2(int *p, int *q)    { return (p[0] + 1) * (q[0] + 1); }
int wbr_n3(int *a, int *b, int *c)
{
    return (a[0] + 1) * (b[0] + 2) + (c[0] + 3);
}
int wbr_n4(int *a, int *b, int *c, int *d)
{
    return ((a[0] + 1) * (b[0] + 2)) ^ ((c[0] + 3) * (d[0] + 4));
}

// ---- G1..G4: the SHARP cells.  Values materialised from GLOBALS have no
// ---- incoming argument register, so nothing pre-colours them and no copy
// ---- preference biases the cost.  These are the cells where the preference
// ---- LIST ORDER (r11, r10, r9, ...) is the only thing left to decide.
extern int wbr_g0, wbr_g1, wbr_g2, wbr_g3, wbr_g4;
int wbr_glob1(void) { return wbr_g0 + 1; }
int wbr_glob2(void) { return (wbr_g0 + 1) * (wbr_g1 + 2); }
int wbr_glob3(void) { return (wbr_g0 + 1) * (wbr_g1 + 2) + (wbr_g2 * 3); }
int wbr_glob4(void)
{
    return ((wbr_g0 + 1) * (wbr_g1 + 2)) ^ ((wbr_g2 + 3) * (wbr_g3 + 4));
}

// ---- L1: a counted for-loop over one array with one accumulator.  THE first
// ---- class-conversion candidate.  Loop-carried values are live across the
// ---- back edge; the reading says they still come from the same list.
int wbr_loop_sum(const int *a, int n)
{
    int s = 0;
    for (int i = 0; i < n; ++i) s += a[i];
    return s;
}

// ---- L2: the same loop with TWO accumulators (two loop-carried temps).
int wbr_loop_two(const int *a, int n)
{
    int s = 0, t = 1;
    for (int i = 0; i < n; ++i) { s += a[i]; t ^= a[i]; }
    return s + t;
}

// ---- L3: a loop whose body makes a call, so the loop-carried values cannot
// ---- live in volatiles.  Predicted: they move to r31, r30, ... in that order.
int wbr_extf(int);
int wbr_loop_call(const int *a, int n)
{
    int s = 0;
    for (int i = 0; i < n; ++i) s += wbr_extf(a[i]);
    return s;
}

// ---- M1: a multi-way if (3 arms, one join).  The OTHER named class.
int wbr_multiway(int x)
{
    if (x < 0)       return -x;
    else if (x == 0) return 100;
    else if (x < 10) return x * 3;
    else             return x + 7;
}

// ---- M2: a dense switch — a jump table, so block order is not source order.
int wbr_switch(int x)
{
    switch (x) {
    case 0: return 11;
    case 1: return 22;
    case 2: return 33;
    case 3: return 44;
    case 4: return 55;
    case 5: return 66;
    default: return -1;
    }
}

// ---- C1/C2: signedness of the compare (#1788) on int vs unsigned.
int wbr_cmp_s(int x)      { return x < 10 ? 1 : 2; }
int wbr_cmp_u(unsigned x) { return x < 10u ? 1 : 2; }

// ---- S1: two INDEPENDENT load-use chains written in one source order.  A list
// ---- scheduler would interleave the two loads to hide load-use latency; a
// ---- backend with no scheduler emits chain A then chain B.
void wbr_sched(int *out, const int *p, const int *q)
{
    out[0] = p[0] * 3;
    out[1] = q[0] * 5;
}

// ---- P1: pressure — 12 simultaneously live ints, more than the 9 allocatable
// ---- volatiles, so the allocator must reach into the callee-saved half of the
// ---- SAME list.  Predicted first callee-saved taken is r31, then r30, ...
int wbr_pressure(const int *a)
{
    int v0=a[0],v1=a[1],v2=a[2],v3=a[3],v4=a[4],v5=a[5];
    int v6=a[6],v7=a[7],v8=a[8],v9=a[9],v10=a[10],v11=a[11];
    return (v0^v1)+(v2^v3)+(v4^v5)+(v6^v7)+(v8^v9)+(v10^v11)
         + (v0*v11)+(v1*v10)+(v2*v9)+(v3*v8)+(v4*v7)+(v5*v6);
}

} // extern "C"

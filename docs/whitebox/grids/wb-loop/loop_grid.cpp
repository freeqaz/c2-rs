// loop_grid.cpp — lane wb-loop (WB-H), campaign 2026-08-08.  THE FROZEN GRID.
//
// Grades the counted-loop lowering read off c2.dll's p2\ppc\lower.c and
// p2\misc.c in docs/whitebox/WB_LOOP_FINDINGS.md.
//
// FROZEN BEFORE THE FIRST cl.exe OF THIS FILE.  Per-cell, per-rival predictions
// are in frozen.tsv (sha256 recorded in the findings doc).  The shapes these
// cells depend on were measured first in work/wb-loop/calib.cpp (a CALIBRATION
// pass, unscored) precisely so this grid is not refuted by its own cells the
// way wb-inline's v1 grid was.
//
// One COMDAT per cell (/Gy is implied by the workload mode), so every cell can
// be read out of one obj by symbol name.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/wb-loop/loop_grid.cpp \
//               /nologo /c /GR /O1 /Oi /EHsc
// Read:     scripts/gt_dump.py <obj> --text-only

extern "C" {

int  wbl_ext(int);
int  wbl_ext2(int);
extern int   wbl_g[64];
struct wbl_s3 { int x, y, z; };

// =====================================================================
// BLOCK A — the mtctr/bdnz CHOICE
// =====================================================================

// A1: early `return` out of the body — a second exit, no call.
int wbl_a1_ret(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i){ if (a[i]<0) return -1; s+=a[i]; } return s; }

// A2: `goto` out of the body — a second exit by another spelling.
int wbl_a2_goto(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i){ if (a[i]==7) goto out; s+=a[i]; } out: return s; }

// A3: the body calls a function the front end can inline away.
static int wbl_sq(int v) { return v*v; }
int wbl_a3_inline(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=wbl_sq(a[i]); return s; }

// A4: indirect call through a function pointer — a real bctrl.
int wbl_a4_indcall(const int *a, int n, int (*f)(int))
{ int s=0; for (int i=0;i<n;++i) s+=f(a[i]); return s; }

// A5: nested; the INNER loop has a break so it cannot take ctr.  Does the
//     OUTER take it?  This is the sharpest cell in block A.
int wbl_a5_nest_inner_break(const int *a, int n, int m)
{
    int s=0;
    for (int i=0;i<n;++i) { for (int j=0;j<m;++j) { if (a[j]==0) break; s+=a[j]; } }
    return s;
}

// A6: nested, both qualify.  Control for A5.
int wbl_a6_nest_both(const int *a, int n, int m)
{ int s=0; for (int i=0;i<n;++i) for (int j=0;j<m;++j) s+=a[j]; return s; }

// A7: a 12-case dense switch in the body — if it becomes a jump table it
//     needs bctr and must evict the loop from ctr.
int wbl_a7_switch(const int *a, int n)
{
    int s=0;
    for (int i=0;i<n;++i) {
        switch (a[i]) {
        case 0: s+=1;  break;   case 1: s+=2;  break;
        case 2: s+=3;  break;   case 3: s+=5;  break;
        case 4: s+=7;  break;   case 5: s+=11; break;
        case 6: s+=13; break;   case 7: s+=17; break;
        case 8: s+=19; break;   case 9: s+=23; break;
        case 10: s+=29; break;  case 11: s+=31; break;
        default: s-=1;  break;
        }
    }
    return s;
}

// A8: an integer divide in the body — expensive, but not a call.
int wbl_a8_div(const int *a, int n, int d)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]/d; return s; }

// A9: a 64-bit counter and a 64-bit bound.
int wbl_a9_i64(const int *a, long long n)
{ int s=0; for (long long i=0;i<n;++i) s+=a[(int)i]; return s; }

// A10: trip count is a loop-invariant EXPRESSION, not a bare parameter.
int wbl_a10_expr(const int *a, int n)
{ int s=0; for (int i=0;i<n/2+3;++i) s+=a[i]; return s; }

// A11: volatile elements — every access must survive.
int wbl_a11_vol(const volatile int *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// A12: `while` spelling of the counted loop, IV declared outside.
int wbl_a12_while(const int *a, int n)
{ int s=0,i=0; while (i<n) { s+=a[i]; ++i; } return s; }

// A13: floating-point body, no call.
double wbl_a13_fp(const float *a, int n)
{ double s=0; for (int i=0;i<n;++i) s+=a[i]*2.0; return s; }

// A14: a POINTER induction variable with a counted bound.
int wbl_a14_ptriv(const int *a, int n)
{ int s=0; for (const int *p=a; p<a+n; ++p) s+=*p; return s; }

// =====================================================================
// BLOCK B — the zero-trip guard
// =====================================================================

// B1: nonzero constant start, variable bound.
int wbl_b1_from3(const int *a, int n)
{ int s=0; for (int i=3;i<n;++i) s+=a[i]; return s; }

// B2: both ends constant — the trip count is 7 at compile time.
int wbl_b2_constrange(const int *a)
{ int s=0; for (int i=3;i<10;++i) s+=a[i]; return s; }

// B3: a DOMINATING test already proves n > 0.
int wbl_b3_dominated(const int *a, int n)
{ if (n<=0) return 0; int s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// B4: step 3.
int wbl_b4_step3(const int *a, int n)
{ int s=0; for (int i=0;i<n;i+=3) s+=a[i]; return s; }

// B5: `<=` bound — n+1 trips, and n == -1 is the zero-trip case.
int wbl_b5_le(const int *a, int n)
{ int s=0; for (int i=0;i<=n;++i) s+=a[i]; return s; }

// B6: unsigned counter starting at 1.
int wbl_b6_uns1(const int *a, unsigned n)
{ int s=0; for (unsigned i=1;i<n;++i) s+=a[i]; return s; }

// B7: code AFTER the loop, so the guard cannot be a conditional return.
int wbl_b7_notail(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]; return s*3+1; }

// =====================================================================
// BLOCK C — the induction rewrite / update-form selection
// =====================================================================

// C1: two arrays, second operand first in the source.
int wbl_c1_ba(const int *a, const int *b, int n)
{ int s=0; for (int i=0;i<n;++i) s+=b[i]+a[i]; return s; }

// C2: a straight copy — one load pointer, one store pointer.
void wbl_c2_copy(int *b, const int *a, int n)
{ for (int i=0;i<n;++i) b[i]=a[i]; }

// C3: two arrays with DIFFERENT element sizes, so one base-difference
//     cannot serve both.
int wbl_c3_mixed(const int *a, const char *b, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]+b[i]; return s; }

// C4: unsigned char elements — no sign extension needed.
int wbl_c4_uchar(const unsigned char *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// C5: float elements.
float wbl_c5_float(const float *a, int n)
{ float s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// C6: 64-bit integer elements.
long long wbl_c6_i64(const long long *a, int n)
{ long long s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// C7: 12-byte struct stride — not a power of two.
int wbl_c7_struct(const struct wbl_s3 *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i].y; return s; }

// C8: descending store.
void wbl_c8_downstore(int *a, int n, int k)
{ for (int i=n-1;i>=0;--i) a[i]=k; }

// C9: ONE pointer, TWO D-form references at different offsets.
int wbl_c9_selfoff(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]+a[i+1]; return s; }

// C10: ONE pointer, THREE D-form references.
int wbl_c10_three(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]+a[i+1]+a[i+2]; return s; }

// =====================================================================
// BLOCK D — block order (WB-D §9.2 item 3, left open)
// =====================================================================

// D1: statements after the loop.
int wbl_d1_after(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]; s=wbl_ext(s); return s+9; }

// D2: the loop is inside the THEN arm of an if/else.
int wbl_d2_ifloop(const int *a, int n, int f)
{ if (!f) return -1; int s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// D3: a 3-way if/else-if/else in the body.
int wbl_d3_body_if3(const int *a, int n)
{
    int s=0;
    for (int i=0;i<n;++i) { if (a[i]>0) s+=a[i]; else if (a[i]<0) s-=a[i]; else s+=100; }
    return s;
}

// D4: two nested loops with code between the levels.
int wbl_d4_twonest(const int *a, int n, int m)
{
    int s=0;
    for (int i=0;i<n;++i) { s=wbl_ext2(s); for (int j=0;j<m;++j) s+=a[j]; s^=i; }
    return s;
}

// D5: a loop inside one arm of a switch.
int wbl_d5_loopinswitch(const int *a, int n, int k)
{
    int s=0;
    switch (k) {
    case 0: for (int i=0;i<n;++i) s+=a[i]; break;
    case 1: s = -1; break;
    case 2: s = wbl_ext(n); break;
    default: s = 0; break;
    }
    return s;
}

}

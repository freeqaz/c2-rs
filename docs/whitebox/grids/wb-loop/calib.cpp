// calib.cpp — lane wb-loop (WB-H), campaign 2026-08-08.  CALIBRATION ONLY.
//
// NOT the grid.  This pass exists to measure the SHAPES my grid cells depend
// on, so the frozen grid is not refuted by its own cells the way wb-inline's
// v1 grid was (a folding compiler collapsed the ladder).  Nothing here is
// scored.  Results feed the design of grids/wb-loop/loop_grid.cpp, which is
// frozen with per-rival predictions BEFORE its first cl.exe.
//
// Mode: /nologo /c /GR /O1 /Oi /EHsc  (WB-D's workload mode, for comparability)

extern "C" {

int  wbl_ext(int);
void wbl_sink(int);
extern int wbl_g[64];

// --- C-A: does a constant trip count survive as a loop, and where is the
// --- unroll/fold ceiling?  (PREREG P2.5 registered <=4 unrolled, >=8 loops.)
int cal_c2 (const int *a){ int s=0; for (int i=0;i<2 ;++i) s+=a[i]; return s; }
int cal_c3 (const int *a){ int s=0; for (int i=0;i<3 ;++i) s+=a[i]; return s; }
int cal_c4 (const int *a){ int s=0; for (int i=0;i<4 ;++i) s+=a[i]; return s; }
int cal_c6 (const int *a){ int s=0; for (int i=0;i<6 ;++i) s+=a[i]; return s; }
int cal_c8 (const int *a){ int s=0; for (int i=0;i<8 ;++i) s+=a[i]; return s; }
int cal_c16(const int *a){ int s=0; for (int i=0;i<16;++i) s+=a[i]; return s; }
int cal_c64(const int *a){ int s=0; for (int i=0;i<64;++i) s+=a[i]; return s; }

// --- C-B: the WB-D baseline, re-measured here so every later delta has a
// --- same-obj reference.
int cal_base(const int *a, int n){ int s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// --- C-C: does a call in the body keep the ctr form?  (WB-D L3 says yes;
// --- FUN_10c09c81 says only CTR-touching opcodes refuse.)
int cal_call(const int *a, int n){ int s=0; for (int i=0;i<n;++i) s+=wbl_ext(a[i]); return s; }

// --- C-D: does `break` kill it?
int cal_break(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i){ if (a[i]<0) break; s+=a[i]; } return s; }

// --- C-E: is the IV live after the loop?  (P2.2: should refuse the ctr form.)
int cal_ivlive(const int *a, int n)
{ int i=0,s=0; for (;i<n;++i) s+=a[i]; return s+i*1000; }

// --- C-F: two arrays — one update-form pointer each, or one indexed?
int cal_two(const int *a, const int *b, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]*b[i]; return s; }

// --- C-G: write-only loop (stwu?).
void cal_store(int *a, int n, int k){ for (int i=0;i<n;++i) a[i]=k; }

// --- C-H: read-modify-write of ONE array at one index.
void cal_rmw(int *a, int n){ for (int i=0;i<n;++i) a[i]=a[i]+1; }

// --- C-I: element widths.
int  cal_char (const char  *a, int n){ int s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }
int  cal_short(const short *a, int n){ int s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }
double cal_dbl(const double*a, int n){ double s=0; for (int i=0;i<n;++i) s+=a[i]; return s; }

// --- C-J: non-unit constant stride.
int cal_stride2(const int *a, int n){ int s=0; for (int i=0;i<n;++i) s+=a[2*i]; return s; }

// --- C-K: down-counting.
int cal_down(const int *a, int n){ int s=0; for (int i=n-1;i>=0;--i) s+=a[i]; return s; }

// --- C-L: nested loops — which one gets ctr?
int cal_nest(const int *a, int n, int m)
{ int s=0; for (int i=0;i<n;++i) for (int j=0;j<m;++j) s+=a[i*8+j]; return s; }

// --- C-M: unsigned trip count (guard still needed? cmplwi?).
int cal_uns(const int *a, unsigned n){ int s=0; for (unsigned i=0;i<n;++i) s+=a[i]; return s; }

// --- C-N: do-while (source already guarantees >=1 trip).
int cal_dowhile(const int *a, int n)
{ int s=0,i=0; do { s+=a[i]; ++i; } while (i<n); return s; }

// --- C-O: increment by 2 (the +1 constant check in FUN_10c0f7f9).
int cal_inc2(const int *a, int n){ int s=0; for (int i=0;i<n;i+=2) s+=a[i]; return s; }

// --- C-P: two sequential loops (block order).
int cal_seq(const int *a, const int *b, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]; for (int i=0;i<n;++i) s^=b[i]; return s; }

// --- C-Q: an if/else inside the body (block order inside a loop).
int cal_ifelse(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i){ if (a[i]>0) s+=a[i]; else s-=a[i]; } return s; }

// --- C-R: pointer-walk source form (no index at all).
int cal_ptr(const int *p, const int *e){ int s=0; while (p!=e){ s+=*p; ++p; } return s; }

// --- C-S: a global array base (no incoming pointer register).
int cal_glob(int n){ int s=0; for (int i=0;i<n;++i) s+=wbl_g[i]; return s; }

// --- C-T: four arrays (register pressure vs per-array update form).
int cal_four(const int *a, const int *b, const int *c, const int *d, int n)
{ int s=0; for (int i=0;i<n;++i) s+=a[i]+b[i]+c[i]+d[i]; return s; }

// --- C-U: non-constant stride.
int cal_vstride(const int *a, int n, int k){ int s=0; for (int i=0;i<n;++i) s+=a[i*k]; return s; }

// --- C-V: `continue` in the body.
int cal_cont(const int *a, int n)
{ int s=0; for (int i=0;i<n;++i){ if (a[i]==0) continue; s+=a[i]; } return s; }

// --- C-W: body that calls sink (void call, no return value dependence).
void cal_sinkloop(const int *a, int n){ for (int i=0;i<n;++i) wbl_sink(a[i]); }

}

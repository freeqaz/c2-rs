// **Positive** — W34, the RUN of byte-offset adds in the indirect-load leaf.
// Every function here must emit, and the whole obj must be byte-exact.
//
// A nested member access `p->mid.in.b` is not one offset add, it is a *chain* of
// them, and c2 folds the whole chain into the single `lwz` displacement exactly
// as it folds one:
//
//   b9 <p> 86 43 8b 20     LOAD p                (Outer *)
//   33 86 41 74 08         LITERAL int 8         offsetof(Outer, mid)
//   27 86 43 86 20         byte-offset add       -> Mid *
//   33 86 41 74 04         LITERAL int 4         offsetof(Mid, in)
//   27 86 43 87 20         byte-offset add       -> Inner *
//   33 86 41 74 04         LITERAL int 4         offsetof(Inner, b)
//   27 86 43 f4 08         byte-offset add       -> int *
//   30 86 41 74            indirect load         -> int
//   41 86 41 74            result type
//                                                => lwz r3,16(r3) ; blr
//
// The leaf used to admit **exactly one** add and refuse the rest as
// `expr-op-0x27`. That limit was never measured and never shared: the ADDRESS
// leaf has folded an arbitrary run since it was written (`&s->arr[2]` is
// `LIT(40) 27 · LIT(8) 28` and emits one `addi r3,r3,48`) and the STORE leaf
// inherited that same walk, while the load leaf kept a private single-add copy.
// One rule, three implementations, and the third was missing a widening the
// other two had — `docs/GAPS.md` §6's recurring shape, costing coverage rather
// than correctness this time. On the 878-TU dc3 workload it refused **5,161**
// functions; every one of them converts here, 1:1, and no other census key
// moves.
//
// The claim the old comment made — "`p[i][j]` chains two of them and needs
// `slwi ; add ; slwi ; lwzx`" — is true and is a *different* construct. A
// variable index is not a `33 <literal>` at all, so it never enters this walk;
// see `w34_offset_run_neg.cpp`, which carries that case.
//
// Only the LAST `27` is asked what the pointee width is. An intermediate one
// re-types the address to a pointer to the enclosing sub-object, and an
// aggregate pointer's tag width nibble is the POINTER's alignment rather than
// the aggregate's size — a pointer to a 24,004-byte struct carries `86 43`
// (MEASURED). So an intermediate `27` says nothing about what is finally
// loaded, and only the last one is in a position to.

struct Inner { int a; int b; };
struct Mid   { int m0; Inner in; };
struct Outer { int o0; int o1; Mid mid; };

// --- depth ladder: two, three and four `27`s, all folding to one `lwz` -------
int d2(Mid* p)   { return p->in.b; }        // 4 + 4         -> lwz r3,8(r3)
int d3(Outer* p) { return p->mid.in.a; }    // 8 + 4 + 0     -> lwz r3,12(r3)
int d4(Outer* p) { return p->mid.in.b; }    // 8 + 4 + 4     -> lwz r3,16(r3)

// --- the tail width picks the instruction, and the run does not change that --
struct Tail { int t0; long long q; short s; char c; unsigned char u; int* pm; };
struct WrapT { int w0; Tail t; };
int           w_i(WrapT* p) { return p->t.t0; }   // lwz
long long     w_q(WrapT* p) { return p->t.q; }    // ld  (DS-form, offset 8|4)
short         w_s(WrapT* p) { return p->t.s; }    // lhz + extsh, per T3
char          w_c(WrapT* p) { return p->t.c; }    // lbz + extsb
unsigned char w_u(WrapT* p) { return p->t.u; }    // lbz
int*          w_p(WrapT* p) { return p->t.pm; }   // lwz (pointer value)

// --- cv-qualification, which changes no operator and no shape and is exactly
//     the axis `docs/GAPS.md` §6's thirteenth live mis-emit hid behind ---------
int cv_ptr_c(const Outer* p)  { return p->mid.in.b; }
int cv_ptr_k(Outer* const p)  { return p->mid.in.b; }
struct CvI  { const int ci; };
struct CvW  { int c0; CvI cv; };
int cv_mem(CvW* p)            { return p->cv.ci; }

// --- `28` in the run, mixed with `27`: arrays of structs and arrays in structs
struct Row  { int e[3]; };
struct Grid { int g0; Row rows[2]; };
int mix_a(Grid* p) { return p->rows[1].e[2]; }   // 4 + 12 + 8   -> lwz r3,24(r3)
int mix_b(Grid* p) { return p->rows[0].e[0]; }   // 4 + 0 + 0    -> lwz r3,4(r3)

// --- a union in the chain ----------------------------------------------------
union  U  { int ui; float uf; };
struct WU { int w; U u; };
struct WW { int v; WU wu; };
int uni(WW* p) { return p->wu.u.ui; }            // 4 + 4 + 0

// --- the 16-bit displacement is gated on the SUM, not on any one add ---------
struct Pad  { int a[8190]; int last; };          // `last` at 32760
struct Hold { int h; Pad pad; };                 // 4 + 32760 = 32764, the edge
int edge(Hold* p) { return p->pad.last; }        // lwz r3,32764(r3)

// --- the SAME run rule at its SECOND site: after intrinsic 2117
//     `base-member-addr`. A member inherited from a non-virtual base is not a
//     `27` at all — c1xx computes its address with the intrinsic — and a further
//     `.sub` chain then follows as ordinary offset adds. `docs/GAPS.md` §6's
//     "estimate the fix, not the finding": sizing only the plain site would have
//     under-counted this rung by 1,346 functions.
struct BaseA { int ba; Mid mid; };
struct DerA : BaseA { int da; };
int bm_0(DerA* p) { return p->ba; }        // 2117 alone (in class before W34)
int bm_1(DerA* p) { return p->mid.m0; }    // 2117 then one 27
int bm_2(DerA* p) { return p->mid.in.b; }  // 2117 then 27 . 27
struct BR    { int e[4]; };
struct BaseB { int bb; BR row; };
struct DerB : BaseB { int db; };
int bm_3(DerB* p) { return p->row.e[3]; }  // 2117 then 27 . 28

// --- the base pointer at argument positions other than r3, and behind `this`,
//     because `this` takes r3 and shifts every explicit formal up one ---------
struct Ob { int o0; Inner in; };
struct C {
    int g1(Ob* q) const;
    int g4(int, int, int, Ob* q) const;
};
int C::g1(Ob* q) const                  { return q->in.b; }
int C::g4(int, int, int, Ob* q) const   { return q->in.b; }

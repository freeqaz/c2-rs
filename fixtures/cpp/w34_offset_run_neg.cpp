// **Negative** — W34's boundary. NOT ONE function here may census in class, and
// the whole TU must read `Port=NotImplemented`.
//
// Every case is a neighbour of the folded offset run that lowers to something
// other than one `lwz rD, sum(rBase)`, or that this port has not separated. Each
// carries its measured cost on the 878-TU dc3 workload where one is known.

struct Inner { int a; int b; };
struct Mid   { int m0; Inner in; };
struct Outer { int o0; int o1; Mid mid; };

// (1) The SUM outside the 16-bit displacement. Each individual add fits; only
//     the total does not, which is exactly why the gate is on the sum. c2 emits
//     `lis`/`addi` (or `lwzx`) instead of folding.
struct Pad   { int a[8190]; int last; };     // `last` at 32760
struct Hold2 { int h0; int h1; Pad pad; };   // 8 + 32760 = 32768, one past the edge
int n_disp(Hold2* p) { return p->pad.last; }

// (2) A `#pragma pack(4)` 8-byte member behind a 4-byte tag. The `27` says the
//     pointee is 4-wide (the tag carries ALIGNMENT) and the `30` says 8 (the
//     kind carries SIZE) — `docs/GAPS.md` §6's third live mis-emit, which folded
//     an `lwz` at the wrong offset. The pair is not in `SIZED_PTEE`, so it
//     refuses rather than being assumed to behave like the aligned one.
#pragma pack(4)
struct P2 { char pc; long long pq; };
struct P1 { int py; P2 p2; };
#pragma pack()
long long n_packed(P1* p) { return p->p2.pq; }

// (3) TWO derefs. `p->m->in.b` is a second `30`, not a longer run: c2 must
//     materialize the intermediate pointer into a register first. Refusing this
//     is the whole reason the walk stops at the first non-offset-add token.
struct Ind  { Mid* m; };
int n_twoderef(Ind* p) { return p->m->in.b; }

// (4) A VARIABLE index in the chain — the case the leaf's old comment was
//     actually about. It is not a `33 <literal>` at all, so it never enters the
//     walk; c2 emits `slwi ; add` / `lwzx`.
struct Row  { int e[3]; };
struct Grid { int g0; Row rows[2]; };
int n_varidx(Grid* p, int i) { return p->rows[i].e[2]; }

// (5) Arithmetic AFTER the folded load. `*p + 1` puts the load in the SCRATCH
//     register — `lwz r11,k(r3) ; addi r3,r11,1` — and `*p * 3` is
//     strength-reduced, so the modeled chain cannot describe either. Measured
//     cost of this refusal on the workload: **214 functions** (198 with one
//     indirect load, 16 with two).
int n_arith(Outer* p) { return p->mid.in.b + 1; }

// (6) A floating-point member at the tail. `lfs`/`lfd` from a different register
//     file; the run changes nothing about that.
struct Fl { int f0; float fv; };
struct WF { int w0; Fl fl; };
float n_float(WF* p) { return p->fl.fv; }

// (7) A `volatile` BASE pointer. The pointer is a volatile object, so c2 homes
//     it in the frame and reloads it; the leaf would emit one `lwz`. This is
//     `docs/GAPS.md` §6's thirteenth live mis-emit and the gate predates W34 —
//     it is here because a run of adds is a new way to reach the same base.
int n_volbase(int x, Outer* volatile p) { return p->mid.in.b; }

// (8) A base-class member reached in the MIDDLE of a chain: `p->d.b0` is a `27`
//     for `->d` and only THEN intrinsic 2117 for the inherited `b0`. W34 folds a
//     run AFTER the 2117 designator (that is the rung's second site, +1,346) but
//     the intrinsic's own decoder re-reads the object pointer from inside its
//     argument list, so a run BEFORE it does not compose — it is a different
//     production, not a longer run. A GAP W34 deliberately did not take, sized
//     UNMEASURED in `docs/rungs/2026-07-31-offset-run.md`.
struct Base { int b0; };
struct Der : Base { int d0; };
struct WD { int w0; Der d; };
int n_basemem(WD* p) { return p->d.b0; }
